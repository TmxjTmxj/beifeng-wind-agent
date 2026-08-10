import { useEffect, useRef, useState, useCallback, type ReactNode } from "react";
import {
  ArrowDown, ArrowDownToLine, Bot, ChevronDown, ChevronRight, ClipboardList, Copy, FileSearch,
  Loader2, MessageSquare, Search, PanelLeftClose, PanelLeft, Plus, Send, Terminal, Trash2, User, Wrench,
  Play, ShieldCheck, Sparkles, Square, Clock, Activity,
} from "lucide-react";
import { Button } from "../components/Button";
import { StatusPill } from "../components/StatusPill";
import { AgentRunPayload, desktopApi, InspectorEvent, RuntimeStatus } from "../services/desktopApi";
import { useAppState } from "../store/appState";
import { useI18n } from "../store/i18n";

/* ─── types ─── */

interface Turn {
  id: string;
  role: "user" | "assistant";
  content: string;
  toolCalls?: InspectorEvent[];
  timestamp: string;
}

interface Conversation {
  id: string;
  title: string;
  turns: Turn[];
  chatPath?: string | null;
  modified: string;
}

type AgentMode = "explore" | "plan" | "execute" | "review";
type PredictedSkill = {
  name: string;
  label: string;
  confidence: "high" | "medium" | "fallback";
  reason: string;
};

const agentModeMeta: Record<AgentMode, { label: string; description: string; promptPrefix: string }> = {
  explore: {
    label: "探索",
    description: "先读上下文，只归纳事实、缺口和风险。",
    promptPrefix: "请以工程 agent 的探索模式处理：先梳理上下文、已有证据、缺失数据和风险边界，暂不直接给最终处置结论。"
  },
  plan: {
    label: "计划",
    description: "先给可审查计划，再进入执行建议。",
    promptPrefix: "请以计划优先的工程 agent 模式处理：先给出分步骤计划、需要调用的数据源、验证方法和人工确认点，再给出下一步建议。"
  },
  execute: {
    label: "执行",
    description: "按任务目标推进，并返回可操作结果。",
    promptPrefix: "请以执行模式处理：围绕任务目标推进分析，明确工具/数据假设、关键发现、结论和下一步动作。"
  },
  review: {
    label: "审查",
    description: "像代码审查一样优先指出高风险问题。",
    promptPrefix: "请以审查模式处理：优先列出高风险问题、证据不足、可能误判和需要人工确认的事项，再补充改进建议。"
  }
};

const quickActions: Array<{ command: string; label: string; mode: AgentMode; prompt: string }> = [
  {
    command: "/plan",
    label: "生成调查计划",
    mode: "plan",
    prompt: "基于当前风机运维上下文，生成一份可执行的调查计划：目标、数据源、工具调用顺序、风险边界、人工确认点。"
  },
  {
    command: "/inspect",
    label: "梳理上下文",
    mode: "explore",
    prompt: "先梳理当前问题的上下文：相关设备、告警、历史维护、知识库证据、缺失数据和可能的误判来源。"
  },
  {
    command: "/review",
    label: "风险审查",
    mode: "review",
    prompt: "审查当前分析是否可靠：指出高风险结论、证据缺口、需要复核的数据、不可自动执行的安全动作。"
  },
  {
    command: "/report",
    label: "生成报告草稿",
    mode: "execute",
    prompt: "生成一份运维报告草稿，包含摘要、证据、风险等级、处置建议、后续验证步骤和人工确认事项。"
  }
];

const emptyRun: AgentRunPayload = {
  output: "", error: null,
  inspector: { tool_calls: [], knowledge_hits: [], memory_hits: [], graph_hits: [], risk_level: null, execution_trace: [], current_session: null, current_workspace: null, current_model: null, current_provider: null },
  chat_path: null, events: [],
};

/* ─── helpers ─── */

function uid() { return crypto.randomUUID(); }
function nowISO() { return new Date().toISOString(); }
function fmtTime(iso: string) {
  return new Date(iso).toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" });
}
function extractMessageText(raw: string): string {
  for (const line of raw.split("\n")) {
    const t = line.trim();
    if (!t) continue;
    try { const o = JSON.parse(t); if (o.type === "assistant_turn" && typeof o.text === "string" && o.text) return o.text; } catch { /* */ }
  }
  try { const o = JSON.parse(raw.trim()); if (typeof o.message === "string" && o.message) return o.message; } catch { /* */ }
  return raw.trim();
}

function composeAgentPrompt(mode: AgentMode, text: string) {
  return `${agentModeMeta[mode].promptPrefix}\n\n用户任务：${text}`;
}

function predictSkill(text: string): PredictedSkill {
  const query = text.toLowerCase();
  if (/(报告|生成报告|report)/i.test(text)) {
    return { name: "report_generation", label: "报告生成", confidence: "high", reason: "命中报告生成意图" };
  }
  if (/(叶片|裂纹|桨叶|巡检|blade)/i.test(text)) {
    return { name: "blade_inspection", label: "叶片巡检", confidence: "high", reason: "命中叶片/巡检关键词" };
  }
  if (/(齿轮箱|油温|振动|gearbox)/i.test(text)) {
    return { name: "gearbox_diagnosis", label: "齿轮箱诊断", confidence: "high", reason: "命中齿轮箱/油温/振动关键词" };
  }
  if (query.includes("scada") || /(功率曲线|报警)/i.test(text)) {
    return { name: "scada_analysis", label: "SCADA 分析", confidence: "high", reason: "命中 SCADA/功率曲线/报警关键词" };
  }
  return { name: "wind_fault_analysis", label: "通用故障分析", confidence: text.trim() ? "fallback" : "medium", reason: text.trim() ? "未命中特定技能，使用兜底分析链" : "输入后会自动预判技能" };
}

function formatElapsed(ms: number) {
  if (ms < 1000) return `${ms} ms`;
  return `${(ms / 1000).toFixed(1)} s`;
}

/* ─── Markdown ─── */

function MarkdownBody({ text }: { text: string }) {
  const lines = text.split("\n");
  const blocks: React.ReactNode[] = [];
  let i = 0;
  let ul: string[] = [], ol: string[] = [], code: string[] = [], p: string[] = [];
  let inCode = false;
  const flush = () => {
    if (p.length) { blocks.push(<p key={blocks.length} className="md-paragraph">{p.map((l, j) => <span key={j}>{renderInline(l)}{j < p.length - 1 && <br />}</span>)}</p>); p = []; }
    if (ul.length) { blocks.push(<ul key={blocks.length} className="md-list">{ul.map((x, j) => <li key={j}>{renderInline(x)}</li>)}</ul>); ul = []; }
    if (ol.length) { blocks.push(<ol key={blocks.length} className="md-list">{ol.map((x, j) => <li key={j}>{renderInline(x)}</li>)}</ol>); ol = []; }
    if (code.length) { blocks.push(<pre key={blocks.length} className="md-code-block"><code>{code.join("\n")}</code></pre>); code = []; }
  };
  while (i < lines.length) {
    const t = lines[i].trim(); const next = lines[i];
    if (t.startsWith("```")) { flush(); inCode = !inCode; i++; continue; }
    if (inCode) { code.push(next); i++; continue; }
    if (!t) { flush(); i++; continue; }
    if (t.startsWith("### ")) { flush(); blocks.push(<h4 key={blocks.length} className="md-h4">{renderInline(t.slice(4))}</h4>); i++; continue; }
    if (t.startsWith("## ")) { flush(); blocks.push(<h3 key={blocks.length} className="md-h3">{renderInline(t.slice(3))}</h3>); i++; continue; }
    if (t.startsWith("# ")) { flush(); blocks.push(<h2 key={blocks.length} className="md-h2">{renderInline(t.slice(2))}</h2>); i++; continue; }
    if (t === "---" || t === "***") { flush(); blocks.push(<hr key={blocks.length} className="md-hr" />); i++; continue; }
    if (/^[-*]\s/.test(t)) { if (p.length || ol.length || code.length) flush(); ul.push(t.replace(/^[-*]\s+/, "")); i++; continue; }
    if (/^\d+\.\s/.test(t)) { if (p.length || ul.length || code.length) flush(); ol.push(t.replace(/^\d+\.\s+/, "")); i++; continue; }
    if (ul.length || ol.length || code.length) flush();
    p.push(next); i++;
  }
  flush();
  return blocks.length > 0 ? <>{blocks}</> : <p className="md-paragraph">{text}</p>;
}

function renderInline(text: string): React.ReactNode {
  const parts: React.ReactNode[] = [];
  let r = text, k = 0;
  while (r.length > 0) {
    const bi = r.indexOf("**");
    if (bi >= 0) { if (bi > 0) parts.push(<span key={k++}>{r.slice(0, bi)}</span>); const e = r.indexOf("**", bi + 2); if (e > bi) { parts.push(<strong key={k++}>{r.slice(bi + 2, e)}</strong>); r = r.slice(e + 2); } else { parts.push(<span key={k++}>{r.slice(0, bi + 2)}</span>); r = r.slice(bi + 2); } continue; }
    const ci = r.indexOf("`");
    if (ci >= 0) { if (ci > 0) parts.push(<span key={k++}>{r.slice(0, ci)}</span>); const e = r.indexOf("`", ci + 1); if (e > ci) { parts.push(<code key={k++} className="md-inline-code">{r.slice(ci + 1, e)}</code>); r = r.slice(e + 1); } else { parts.push(<span key={k++}>{r.slice(0, ci + 1)}</span>); r = r.slice(ci + 1); } continue; }
    parts.push(<span key={k++}>{r}</span>); break;
  }
  return <>{parts}</>;
}

function ThinkingBubble() {
  const [dots, setDots] = useState("");
  useEffect(() => { const t = setInterval(() => setDots((p) => p.length >= 3 ? "" : p + "."), 400); return () => clearInterval(t); }, []);
  return <div className="thinking-bubble"><Loader2 size={16} className="thinking-spinner" /><span>思考中{dots}</span></div>;
}

/* ─── create a fresh conversation ─── */

function freshConversation(): Conversation {
  const id = uid();
  return { id, title: "新对话", turns: [], modified: nowISO() };
}

/* ─── ChatPage ─── */

export function ChatPage() {
  const { t } = useI18n();
  const { workspaceState, setInspectorCollapsed } = useAppState();

  const [prompt, setPrompt] = useState("");
  const [runtime, setRuntime] = useState<RuntimeStatus>({ agent: "Offline", rag: "Stopped" });
  const [busy, setBusy] = useState(false);
  const [run, setRun] = useState<AgentRunPayload>(emptyRun);
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [expandedTools, setExpandedTools] = useState<Set<string>>(new Set());
  const [convSearch, setConvSearch] = useState("");
  const [hoveredTurnId, setHoveredTurnId] = useState<string | null>(null);
  const [conversations, setConversations] = useState<Conversation[]>(() => [freshConversation()]);
  const [activeId, setActiveId] = useState(() => conversations[0]?.id ?? "");
  const [agentMode, setAgentMode] = useState<AgentMode>("plan");
  const [stopping, setStopping] = useState(false);
  const [elapsedMs, setElapsedMs] = useState(0);
  const [liveOutput, setLiveOutput] = useState("");

  const [highlightedTurn, setHighlightedTurn] = useState<string | null>(null);
  const turnRefs = useRef<Map<string, HTMLDivElement>>(new Map());
  const messagesEnd = useRef<HTMLDivElement>(null);
  const didLoad = useRef(false);
  const runStartedAt = useRef<number | null>(null);

  const activeConv = conversations.find((c) => c.id === activeId) ?? conversations[0];
  const turns = activeConv?.turns ?? [];

  // ensure activeId is always valid
  useEffect(() => {
    if (!conversations.find((c) => c.id === activeId)) {
      setActiveId(conversations[0]?.id ?? "");
    }
  }, [conversations, activeId]);

  /* scroll helpers */
  const scrollToTurn = (id: string) => {
    setHighlightedTurn(id);
    turnRefs.current.get(id)?.scrollIntoView({ behavior: "smooth", block: "start" });
    setTimeout(() => setHighlightedTurn(null), 1500);
  };
  const scrollToBottom = () => messagesEnd.current?.scrollIntoView({ behavior: "smooth" });

  /* ── keyboard shortcuts ── */
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const mod = e.ctrlKey || e.metaKey;
      if (mod && e.key === "n") { e.preventDefault(); handleNewChat(); }
      if (mod && e.key === "Enter") { e.preventDefault(); handleSend(); }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [prompt, busy, activeId, conversations]);

  /* init */
  const refreshAll = useCallback(async () => {
    try { setRuntime(await desktopApi.getAgentStatus()); } catch { /* */ }
  }, []);
  useEffect(() => { void refreshAll(); }, [workspaceState.current_workspace]);

  useEffect(() => {
    if (!busy) return;
    const timer = window.setInterval(() => {
      if (runStartedAt.current) {
        setElapsedMs(Date.now() - runStartedAt.current);
      }
      void desktopApi.getAgentStatus().then(setRuntime).catch(() => undefined);
      void desktopApi.streamAgentOutput().then(setLiveOutput).catch(() => undefined);
    }, 800);
    return () => window.clearInterval(timer);
  }, [busy]);

  // Load saved convos once on mount
  useEffect(() => {
    if (didLoad.current) return;
    didLoad.current = true;
    (async () => {
      try {
        const chats = await desktopApi.listChats();
        if (!chats.length) return;
        const loaded: Conversation[] = [];
        for (const chat of chats) {
          try {
            const data = await desktopApi.loadChatHistory(chat.path);
            const ts: Turn[] = [];
            const prompt = typeof data.prompt === "string" ? data.prompt : "";
            const output = typeof data.output === "string" ? data.output : "";
            if (prompt) ts.push({ id: uid(), role: "user", content: prompt, timestamp: chat.modified });
            if (output) ts.push({ id: uid(), role: "assistant", content: extractMessageText(output), timestamp: chat.modified });
            // multi-turn format
            if (Array.isArray(data.turns)) {
              ts.length = 0;
              for (const item of data.turns as Array<Record<string, unknown>>) {
                ts.push({ id: uid(), role: (item.role as "user" | "assistant") ?? "user", content: String(item.content ?? ""), timestamp: String(item.timestamp ?? chat.modified) });
              }
            }
            if (ts.length) loaded.push({ id: chat.path, title: chat.title, turns: ts, chatPath: chat.path, modified: chat.modified });
          } catch { /* skip broken files */ }
        }
        if (loaded.length > 0) {
          setConversations(loaded);
          setActiveId(loaded[0].id);
        }
      } catch { /* not tauri */ }
    })();
  }, []);

  /* ── helpers for immutable array updates ── */

  function replaceConv(id: string, fn: (c: Conversation) => Conversation) {
    setConversations((prev) => prev.map((c) => (c.id === id ? fn({ ...c, turns: [...c.turns] }) : c)));
  }

  /* ── actions ── */

  async function handleSend() {
    const text = prompt.trim();
    if (!text || busy) return;

    const userTurn: Turn = { id: uid(), role: "user", content: text, timestamp: nowISO() };
    replaceConv(activeId, (c) => ({
      ...c,
      turns: [...c.turns, userTurn],
      title: c.title === "新对话" || c.title.startsWith("新对话") ? text.slice(0, 40) : c.title,
      modified: nowISO(),
    }));

    setPrompt("");
    setBusy(true);
    setStopping(false);
    setElapsedMs(0);
    setLiveOutput("");
    runStartedAt.current = Date.now();
    setRuntime((prev) => ({ ...prev, agent: "Running", agent_error: null }));
    setInspectorCollapsed(false);
    setTimeout(scrollToBottom, 50);

    let result: AgentRunPayload;
    try {
      result = await desktopApi.sendAgentPrompt(composeAgentPrompt(agentMode, text));
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      result = {
        ...emptyRun,
        error: message,
        output: message,
      };
    }
    const display = extractMessageText(result.output) || result.error || "Agent run completed without output.";
    const aiTurn: Turn = { id: uid(), role: "assistant", content: display, toolCalls: result.inspector.tool_calls, timestamp: nowISO() };

    replaceConv(activeId, (c) => ({
      ...c,
      turns: [...c.turns, aiTurn],
      modified: nowISO(),
    }));

    setRun(result);
    await refreshAll();
    setBusy(false);
    setStopping(false);
    runStartedAt.current = null;
    setTimeout(scrollToBottom, 100);
  }

  async function stopCurrentRun() {
    if (!busy || stopping) return;
    setStopping(true);
    try {
      const status = await desktopApi.stopAgentSession();
      setRuntime(status);
    } catch (err) {
      setRun((prev) => ({ ...prev, error: err instanceof Error ? err.message : String(err) }));
    }
  }

  function handleNewChat() {
    const conv = freshConversation();
    setConversations((prev) => [...prev, conv]);
    setActiveId(conv.id); setPrompt(""); setRun(emptyRun); setExpandedTools(new Set());
  }

  function switchConversation(id: string) {
    setActiveId(id); setPrompt(""); setRun(emptyRun); setExpandedTools(new Set());
  }

  async function handleDeleteChat() {
    const c = conversations.find((x) => x.id === activeId);
    if (c?.chatPath) { try { await desktopApi.deleteChat(c.chatPath); } catch { /* */ } }
    setConversations((prev) => prev.filter((x) => x.id !== activeId));
    const rest = conversations.filter((x) => x.id !== activeId);
    if (rest.length) setActiveId(rest[0].id);
    else {
      const conv = freshConversation();
      setConversations([conv]);
      setActiveId(conv.id);
    }
    setPrompt(""); setRun(emptyRun);
  }

  /* ── export / copy ── */
  function exportConversation() {
    const conv = conversations.find((c) => c.id === activeId);
    if (!conv || !conv.turns.length) return;
    const md = conv.turns.map((t) => {
      const role = t.role === "user" ? "### 👤 你" : "### 🤖 BeiFeng Agent";
      return `${role}  (${fmtTime(t.timestamp)})\n\n${t.content}\n`;
    }).join("\n\n---\n\n");
    const blob = new Blob([`# ${conv.title}\n\n${md}`], { type: "text/markdown" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a"); a.href = url; a.download = `${conv.title.slice(0, 30)}.md`; a.click();
    URL.revokeObjectURL(url);
  }

  async function copyTurnContent(content: string) {
    await navigator.clipboard?.writeText(content);
  }

  const toggleTool = (key: string) => setExpandedTools((prev) => { const n = new Set(prev); n.has(key) ? n.delete(key) : n.add(key); return n; });

  /* ── render ── */
  const sorted = [...conversations]
    .filter((c) => !convSearch || c.title.toLowerCase().includes(convSearch.toLowerCase()))
    .sort((a, b) => b.modified.localeCompare(a.modified));
  const title = activeConv?.title ?? "新对话";
  const anchors = turns.map((turn, idx) => ({ turn, idx })).filter(({ turn }) => turn.role === "user");
  const slashQuery = prompt.trimStart().startsWith("/") ? prompt.trimStart().slice(1).toLowerCase() : "";
  const visibleQuickActions = slashQuery
    ? quickActions.filter((action) => `${action.command} ${action.label}`.toLowerCase().includes(slashQuery))
    : quickActions;
  const modeIcons: Record<AgentMode, ReactNode> = {
    explore: <FileSearch size={13} />,
    plan: <ClipboardList size={13} />,
    execute: <Play size={13} />,
    review: <ShieldCheck size={13} />
  };
  const currentPrediction = predictSkill(prompt || turns[turns.length - 1]?.content || "");

  function applyQuickAction(action: (typeof quickActions)[number]) {
    const existing = prompt.trim().startsWith("/") ? "" : prompt.trim();
    setAgentMode(action.mode);
    setPrompt(existing ? `${action.prompt}\n\n${existing}` : action.prompt);
  }

  return (
    <main style={{ display: "flex", flexDirection: "column", height: "100%", minHeight: 0 }}>
      {/* Header */}
      <header style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "6px 14px", borderBottom: "1px solid var(--border)", background: "var(--bg-panel)", minHeight: 40, flexShrink: 0 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 10, minWidth: 0 }}>
          <Button variant="ghost" icon={sidebarOpen ? <PanelLeftClose size={15} /> : <PanelLeft size={15} />} onClick={() => setSidebarOpen(!sidebarOpen)} title="切换侧边栏" />
          <Bot size={16} style={{ color: "var(--accent)", flexShrink: 0 }} />
          <span style={{ fontSize: 13, fontWeight: 600, color: "var(--text)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{title}</span>
          <StatusPill label={runtime.agent} tone={runtime.agent === "Running" ? "success" : "warning"} />
          <div className="agent-mode-segment" role="group" aria-label="Agent 工作模式">
            {(Object.keys(agentModeMeta) as AgentMode[]).map((mode) => (
              <button
                key={mode}
                type="button"
                className={agentMode === mode ? "is-active" : ""}
                onClick={() => setAgentMode(mode)}
                title={agentModeMeta[mode].description}
              >
                {modeIcons[mode]}
                <span>{agentModeMeta[mode].label}</span>
              </button>
            ))}
          </div>
        </div>
        <div style={{ display: "flex", gap: 4, flexShrink: 0 }}>
          {busy ? (
            <Button variant="danger" icon={stopping ? <Loader2 size={14} className="thinking-spinner" /> : <Square size={14} />} onClick={stopCurrentRun} disabled={stopping}>
              {stopping ? "停止中" : "停止"}
            </Button>
          ) : null}
          <Button variant="ghost" icon={<Plus size={14} />} onClick={handleNewChat}>{t("actions.newChat")}</Button>
        </div>
      </header>

      {/* Body */}
      <div style={{ display: "grid", gridTemplateColumns: sidebarOpen ? "250px minmax(0, 1fr)" : "minmax(0, 1fr)", flex: 1, minHeight: 0, transition: "grid-template-columns 180ms ease" }}>
        {/* Sidebar */}
        {sidebarOpen && (
          <aside style={{ borderRight: "1px solid var(--border)", background: "var(--bg-panel)", display: "flex", flexDirection: "column", minHeight: 0, overflow: "auto" }}>
            <div style={{ padding: "8px 10px 4px", borderBottom: "1px solid var(--border)" }}>
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                <span style={{ fontSize: 10, color: "var(--text-muted)", textTransform: "uppercase", letterSpacing: "0.06em", fontWeight: 600 }}>对话</span>
                <Button variant="ghost" icon={<Plus size={13} />} onClick={handleNewChat} title="新建" />
              </div>
              <div style={{ marginTop: 6, display: "flex", alignItems: "center", gap: 6, padding: "4px 6px", border: "1px solid var(--border)", borderRadius: 5, background: "var(--bg-app)" }}>
                <Search size={12} style={{ color: "var(--text-muted)", flexShrink: 0 }} />
                <input
                  value={convSearch}
                  onChange={(e) => setConvSearch(e.target.value)}
                  placeholder="搜索对话…"
                  style={{ flex: 1, border: "none", background: "transparent", color: "var(--text)", fontSize: 11, outline: "none", minWidth: 0 }}
                />
              </div>
            </div>
            <div style={{ padding: 3, borderBottom: "1px solid var(--border-soft)" }}>
              {sorted.map((conv) => (
                <button key={conv.id} onClick={() => switchConversation(conv.id)} type="button"
                  style={{ display: "flex", flexDirection: "column", gap: 1, width: "100%", padding: "6px 9px", border: "1px solid", borderRadius: 5, textAlign: "left" as const, cursor: "pointer", marginBottom: 2, background: activeId === conv.id ? "color-mix(in srgb, var(--accent) 12%, transparent)" : "transparent", borderColor: activeId === conv.id ? "color-mix(in srgb, var(--accent) 35%, var(--border))" : "transparent", color: "var(--text-secondary)" }}>
                  <span style={{ color: activeId === conv.id ? "var(--text)" : "var(--text-secondary)", fontSize: 12, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", fontWeight: activeId === conv.id ? 600 : 400 }}>{conv.title}</span>
                  <span style={{ fontSize: 10, color: "var(--text-muted)" }}>{conv.turns.length}轮</span>
                </button>
              ))}
            </div>

            <div style={{ padding: "8px 10px 4px", borderBottom: "1px solid var(--border)" }}>
              <span style={{ fontSize: 10, color: "var(--text-muted)", textTransform: "uppercase", letterSpacing: "0.06em", fontWeight: 600 }}>消息锚点</span>
            </div>
            <div style={{ flex: 1, overflow: "auto", padding: 3 }}>
              {anchors.length === 0 ? (
                <p style={{ padding: "8px 10px", margin: 0, color: "var(--text-muted)", fontSize: 11, opacity: 0.6 }}>发送消息后出现锚点</p>
              ) : (
                anchors.map(({ turn, idx }) => (
                  <button key={turn.id} onClick={() => scrollToTurn(turn.id)} type="button"
                    style={{ display: "flex", alignItems: "flex-start", gap: 8, width: "100%", padding: "7px 9px", border: "1px solid transparent", borderRadius: 5, textAlign: "left" as const, cursor: "pointer", marginBottom: 2, background: highlightedTurn === turn.id ? "color-mix(in srgb, var(--accent) 18%, transparent)" : "transparent", borderColor: highlightedTurn === turn.id ? "color-mix(in srgb, var(--accent) 40%, var(--border))" : "transparent", color: "var(--text-muted)", transition: "background 150ms ease" }}>
                    <span style={{ fontSize: 10, color: "var(--accent)", fontWeight: 700, minWidth: 18, flexShrink: 0, marginTop: 1 }}>{idx + 1}</span>
                    <div style={{ minWidth: 0 }}>
                      <span style={{ fontSize: 12, color: "var(--text-secondary)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", display: "block" }}>{turn.content.slice(0, 48)}{turn.content.length > 48 ? "…" : ""}</span>
                      <span style={{ fontSize: 10, color: "var(--text-muted)" }}>{fmtTime(turn.timestamp)}</span>
                    </div>
                  </button>
                ))
              )}
            </div>
            <div className="agent-session-brief">
              <div>
                <span>模式</span>
                <strong>{agentModeMeta[agentMode].label}</strong>
              </div>
              <div>
                <span>消息</span>
                <strong>{turns.length}</strong>
              </div>
              <div>
                <span>工具</span>
                <strong>{run.inspector.tool_calls.length}</strong>
              </div>
              <div>
                <span>保存</span>
                <strong title={run.chat_path ?? activeConv?.chatPath ?? ""}>{run.chat_path || activeConv?.chatPath ? "已落盘" : "本轮内存"}</strong>
              </div>
            </div>
            {turns.length > 0 && (
              <>
                <div style={{ padding: "4px 8px", borderTop: "1px solid var(--border)", display: "grid", gap: 3 }}>
                  <Button variant="danger" icon={<Trash2 size={12} />} onClick={handleDeleteChat} style={{ width: "100%", fontSize: 12 }}>删除</Button>
                  <Button icon={<Copy size={12} />} onClick={exportConversation} style={{ width: "100%", fontSize: 12, color: "var(--text-secondary)" }}>导出 .md</Button>
                  <Button icon={<ArrowDown size={12} />} onClick={scrollToBottom} style={{ width: "100%", fontSize: 12, color: "var(--text-muted)" }}>跳到底部</Button>
                </div>
                <div style={{ padding: "6px 8px 8px", fontSize: 10, color: "var(--text-muted)", opacity: 0.5, lineHeight: 1.5 }}>
                  <code>Ctrl+N</code> 新建对话 · <code>Ctrl+Enter</code> 发送
                </div>
              </>
            )}
          </aside>
        )}

        {/* Chat area */}
        <div style={{ display: "flex", flexDirection: "column", minHeight: 0, minWidth: 0 }}>
          <section className="agent-run-strip" aria-label="Agent run state">
            <div>
              <Activity size={14} />
              <span>技能链</span>
              <strong>{currentPrediction.label}</strong>
              <small>{currentPrediction.reason}</small>
            </div>
            <div>
              <Sparkles size={14} />
              <span>工作模式</span>
              <strong>{agentModeMeta[agentMode].label}</strong>
              <small>{agentModeMeta[agentMode].description}</small>
            </div>
            <div>
              <Clock size={14} />
              <span>运行状态</span>
              <strong>{busy ? (stopping ? "Stopping" : runtime.agent) : runtime.agent}</strong>
              <small>{busy ? formatElapsed(elapsedMs) : run.events.length ? `${run.events.length} events` : "等待任务"}</small>
            </div>
            <div>
              <Terminal size={14} />
              <span>实时输出</span>
              <strong>{liveOutput ? "已收到输出" : busy ? "等待输出" : "空闲"}</strong>
              <small title={liveOutput}>{liveOutput ? extractMessageText(liveOutput).slice(0, 80) : run.error ?? "无错误"}</small>
            </div>
          </section>
          <div style={{ flex: 1, overflow: "auto", padding: "16px 24px" }}>
            {turns.length === 0 ? (
              <div style={{ display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", height: "100%", gap: 6, opacity: 0.55 }}>
                <MessageSquare size={34} />
                <p style={{ fontSize: 14, color: "var(--text-secondary)", margin: 0 }}>开始一段新的风电运维对话</p>
                <p style={{ fontSize: 12, color: "var(--text-muted)", margin: 0 }}>输入故障现象、巡检需求或运维问题</p>
              </div>
            ) : (
              <>
                {turns.map((turn, idx) => {
                  const isUser = turn.role === "user";
                  return (
                    <div key={turn.id}
                      ref={(el) => { if (el) turnRefs.current.set(turn.id, el); }}
                      onMouseEnter={() => setHoveredTurnId(turn.id)}
                      onMouseLeave={() => setHoveredTurnId((prev) => prev === turn.id ? null : prev)}
                      style={{ marginBottom: isUser ? 24 : 18, scrollMarginTop: 12, transition: "background 200ms ease", background: highlightedTurn === turn.id ? "color-mix(in srgb, var(--accent) 6%, transparent)" : "transparent", borderRadius: 8, padding: highlightedTurn === turn.id ? "8px" : "0", position: "relative" }}>
                      <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 6 }}>
                        {isUser ? (
                          <><div style={{ width: 26, height: 26, borderRadius: "50%", background: "color-mix(in srgb, var(--accent) 22%, var(--bg-surface))", display: "grid", placeItems: "center", flexShrink: 0 }}><User size={14} style={{ color: "var(--accent)" }} /></div><span style={{ fontSize: 12, fontWeight: 600, color: "var(--text-secondary)" }}>你</span></>
                        ) : (
                          <><div style={{ width: 26, height: 26, borderRadius: "50%", background: "color-mix(in srgb, var(--success) 22%, var(--bg-surface))", display: "grid", placeItems: "center", flexShrink: 0 }}><Bot size={14} style={{ color: "var(--success)" }} /></div><span style={{ fontSize: 12, fontWeight: 600, color: "var(--text-secondary)" }}>BeiFeng Agent</span></>
                        )}
                        <span style={{ fontSize: 10, color: "var(--text-muted)" }}>{fmtTime(turn.timestamp)}</span>
                        {hoveredTurnId === turn.id && (
                          <button type="button" onClick={() => copyTurnContent(turn.content)}
                            title="复制消息"
                            style={{ marginLeft: "auto", border: "1px solid var(--border-soft)", borderRadius: 4, background: "var(--bg-surface)", color: "var(--text-muted)", cursor: "pointer", padding: "2px 6px", fontSize: 10, display: "flex", alignItems: "center", gap: 3 }}>
                            <Copy size={10} /> 复制
                          </button>
                        )}
                      </div>
                      <div style={{ marginLeft: isUser ? "auto" : 0, marginRight: !isUser ? "auto" : 0, maxWidth: "88%", padding: "14px 18px", borderRadius: 12, background: isUser ? "color-mix(in srgb, var(--accent) 16%, var(--bg-surface))" : "color-mix(in srgb, var(--bg-panel) 80%, var(--bg-surface))", border: "1px solid var(--border-soft)", color: "var(--text)", fontSize: 13.5, lineHeight: 1.7 }}>
                        <MarkdownBody text={turn.content} />
                      </div>
                      {isUser && (
                        <div style={{ display: "flex", alignItems: "center", gap: 6, marginTop: 4, marginLeft: 34 }}>
                          <span style={{ fontSize: 10, color: "var(--text-muted)", fontWeight: 500 }}>#{idx + 1}</span>
                          <div style={{ flex: 1, height: 1, background: "var(--border-soft)" }} />
                        </div>
                      )}
                      {!isUser && turn.toolCalls && turn.toolCalls.length > 0 && (
                        <div style={{ marginTop: 6, marginLeft: 34 }}>
                          <button type="button" onClick={() => toggleTool(`tools-${turn.id}`)} style={{ display: "flex", alignItems: "center", gap: 5, border: "none", background: "transparent", color: "var(--text-muted)", cursor: "pointer", fontSize: 11, padding: "3px 0" }}>
                            {expandedTools.has(`tools-${turn.id}`) ? <ChevronDown size={13} /> : <ChevronRight size={13} />}
                            <Wrench size={12} /> 工具调用 ({turn.toolCalls.length})
                          </button>
                          {expandedTools.has(`tools-${turn.id}`) && (
                            <div style={{ display: "grid", gap: 3, marginTop: 3 }}>
                              {turn.toolCalls.map((tc, i) => (
                                <div key={i} style={{ display: "flex", alignItems: "center", gap: 6, padding: "4px 8px", borderRadius: 5, background: "var(--bg-surface)", border: "1px solid var(--border-soft)", fontSize: 11 }}>
                                  <Terminal size={11} style={{ color: "var(--accent)", flexShrink: 0 }} />
                                  <strong style={{ color: "var(--text)", fontSize: 11 }}>{tc.label}</strong>
                                  <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", flex: 1 }}>{tc.detail}</span>
                                  <span style={{ color: tc.status === "completed" ? "var(--success)" : "var(--warning)", fontSize: 10, flexShrink: 0 }}>{tc.status}</span>
                                </div>
                              ))}
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                  );
                })}
                {busy && <ThinkingBubble />}
                <div ref={messagesEnd} />
              </>
            )}
          </div>

          {run.error && (
            <div style={{ padding: "6px 14px", borderTop: "1px solid color-mix(in srgb, var(--risk) 30%, var(--border))", background: "color-mix(in srgb, var(--risk) 8%, transparent)", color: "var(--risk)", fontSize: 11 }}>{run.error}</div>
          )}

          {/* Input */}
          <div style={{ borderTop: "1px solid var(--border)", padding: "8px 14px", background: "var(--bg-panel)", flexShrink: 0 }}>
            <div className="composer-command-strip">
              <div className="composer-mode-copy">
                <Sparkles size={13} />
                <strong>{agentModeMeta[agentMode].label}模式</strong>
                <span>{agentModeMeta[agentMode].description}</span>
              </div>
              <div className="composer-action-list" aria-label="快捷 agent 命令">
                {visibleQuickActions.map((action) => (
                  <button key={action.command} type="button" onClick={() => applyQuickAction(action)}>
                    <code>{action.command}</code>
                    <span>{action.label}</span>
                  </button>
                ))}
              </div>
            </div>
            <div style={{ display: "flex", gap: 8, alignItems: "flex-end" }}>
              <textarea value={prompt} onChange={(e) => setPrompt(e.target.value)}
                onKeyDown={(e) => { if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); handleSend(); } }}
                placeholder="输入运维问题… (Enter 发送, Shift+Enter 换行)"
                style={{ flex: 1, minHeight: 40, maxHeight: 140, resize: "none", border: "1px solid var(--border)", borderRadius: 8, padding: "9px 12px", background: "var(--bg-app)", color: "var(--text)", fontSize: 13, lineHeight: 1.5, outline: "none", fontFamily: "inherit" }} />
              <Button variant="primary" icon={busy ? <Loader2 size={15} className="thinking-spinner" /> : <Send size={15} />} onClick={handleSend} disabled={busy || !prompt.trim()} style={{ minHeight: 40, minWidth: 40 }}>
                {busy ? "思考中" : "发送"}
              </Button>
            </div>
          </div>
        </div>
      </div>
    </main>
  );
}
