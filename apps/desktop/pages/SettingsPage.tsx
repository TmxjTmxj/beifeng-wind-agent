import { useEffect, useState } from "react";
import { AlertTriangle, CheckCircle, Copy, Download, Eye, EyeOff, FileJson, Globe, Key, Monitor, Play, RefreshCw, RotateCcw, Save, Shield, Square, Upload, Zap } from "lucide-react";
import { Button } from "../components/Button";
import { desktopApi } from "../services/desktopApi";
import { useI18n } from "../store/i18n";
import { useSettings } from "../store/settingsStore";

type SettingsTab = "general" | "model" | "credentials" | "rag" | "paths" | "safety" | "advanced";

const tabs: { id: SettingsTab; label: string; icon: typeof Monitor }[] = [
  { id: "general", label: "通用", icon: Monitor },
  { id: "model", label: "模型", icon: Zap },
  { id: "credentials", label: "凭据", icon: Key },
  { id: "rag", label: "RAG 知识库", icon: FileJson },
  { id: "paths", label: "路径", icon: Globe },
  { id: "safety", label: "安全", icon: Shield },
  { id: "advanced", label: "高级", icon: FileJson },
];

function FormField({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="settings-field" style={{ marginBottom: 10 }}>
      <span>{label}</span>
      {children}
    </label>
  );
}

export function SettingsPage() {
  const { settings, updateSettings, resetSettings, getSettingsJson, importSettingsJson } = useSettings();
  const { t } = useI18n();
  const [activeTab, setActiveTab] = useState<SettingsTab>("general");
  const [showKey, setShowKey] = useState(false);
  const [jsonEditOpen, setJsonEditOpen] = useState(false);
  const [jsonText, setJsonText] = useState("");
  const [jsonError, setJsonError] = useState("");
  const [message, setMessage] = useState("");
  const [ragStatus, setRagStatus] = useState("未知");
  const [ragBusy, setRagBusy] = useState(false);

  const notify = (msg: string) => {
    setMessage(msg);
    setTimeout(() => setMessage(""), 4000);
  };

  useEffect(() => {
    let mounted = true;
    desktopApi.getAgentStatus().then((status) => {
      if (mounted) {
        setRagStatus(status.rag === "Running" ? "运行中" : status.rag === "Stopped" ? "已停止" : status.rag);
      }
    }).catch(() => {
      if (mounted) setRagStatus("未知");
    });
    return () => { mounted = false; };
  }, []);

  const handleRagAction = async (action: "start" | "stop" | "restart") => {
    setRagBusy(true);
    try {
      let status;
      if (action === "start") {
        status = await desktopApi.startRagService();
      } else if (action === "stop") {
        status = await desktopApi.stopRagService();
      } else {
        status = await desktopApi.restartRagService();
      }
      setRagStatus(status.rag === "Running" ? "运行中" : status.rag === "Stopped" ? "已停止" : status.rag);
      if (status.rag_error) {
        notify(`RAG 服务状态：${status.rag}，${status.rag_error}`);
      } else {
        notify(`RAG 服务已${action === "start" ? "启动" : action === "stop" ? "停止" : "重启"}`);
      }
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      setRagStatus("错误");
      notify(`操作失败：${detail}`);
    }
    setRagBusy(false);
  };

  const handleImportJson = () => {
    const result = importSettingsJson(jsonText);
    if (result.ok) {
      setJsonError("");
      notify("设置已应用");
      setJsonEditOpen(false);
    } else {
      setJsonError(result.errors.join("\n"));
    }
  };

  const openJsonEditor = () => {
    setJsonText(getSettingsJson());
    setJsonError("");
    setJsonEditOpen(true);
  };

  return (
    <main className="workbench-page">
      <header className="screen-header">
        <div>
          <h1>{t("settings.title")}</h1>
          <p>管理模型、凭据、RAG 数据库和应用路径 — 所有修改自动保存</p>
        </div>
        <div className="header-actions">
          <Button variant="ghost" icon={<FileJson size={15} />} onClick={openJsonEditor}>编辑 JSON</Button>
          <Button variant="ghost" icon={<RotateCcw size={15} />} onClick={() => { resetSettings(); notify("已重置为默认设置"); }}>重置</Button>
        </div>
      </header>

      {message ? <div className="settings-message compact" style={{ marginBottom: 10 }}>{message}</div> : null}

      {jsonEditOpen ? (
        <section className="panel-block" style={{ marginBottom: 12 }}>
          <div className="panel-heading-row">
            <h2>settings.json</h2>
            <div className="header-actions">
              <Button variant="ghost" icon={<Copy size={14} />} onClick={() => { navigator.clipboard.writeText(jsonText); notify("已复制"); }}>复制</Button>
              <Button variant="primary" icon={<Save size={14} />} onClick={handleImportJson}>应用</Button>
              <Button variant="ghost" onClick={() => setJsonEditOpen(false)}>取消</Button>
            </div>
          </div>
          <div style={{ padding: 10 }}>
            <textarea
              value={jsonText}
              onChange={(e) => { setJsonText(e.target.value); setJsonError(""); }}
              style={{
                width: "100%", minHeight: 420, border: "1px solid var(--border)", borderRadius: 6,
                padding: 12, background: "var(--bg-app)", color: "var(--text)",
                fontFamily: "var(--font-mono)", fontSize: 13, lineHeight: 1.5,
                resize: "vertical", outline: "none",
              }}
            />
          </div>
          {jsonError ? <div className="settings-message is-error compact" style={{ margin: "0 10px 10px", whiteSpace: "pre-wrap" }}>{jsonError}</div> : null}
        </section>
      ) : null}

      <div className="settings-layout">
        {/* Left: category tabs */}
        <nav className="panel-block">
          <div style={{ display: "grid", gap: 4, padding: 8 }}>
            {tabs.map((tab) => (
              <button
                key={tab.id}
                className={`${activeTab === tab.id ? "is-active" : ""}`}
                onClick={() => setActiveTab(tab.id)}
                type="button"
                style={{
                  display: "flex", alignItems: "center", gap: 8,
                  minHeight: 32, padding: "0 10px", border: "1px solid transparent",
                  borderRadius: 6, background: activeTab === tab.id ? "color-mix(in srgb, var(--accent) 14%, var(--bg-surface))" : "transparent",
                  color: activeTab === tab.id ? "var(--text)" : "var(--text-secondary)",
                  cursor: "pointer", fontSize: 13, textAlign: "left" as const,
                }}
              >
                <tab.icon size={15} /> {tab.label}
              </button>
            ))}
          </div>
        </nav>

        {/* Right: settings form */}
        <div style={{ display: "grid", gap: 12, alignContent: "start" }}>
          {/* General */}
          {activeTab === "general" && (
            <Panel>
              <FormField label="界面语言">
                <select value={settings.language} onChange={(e) => updateSettings({ language: e.target.value as "zh-CN" | "en-US" })}
                  style={selectStyle}>
                  <option value="zh-CN">中文</option>
                  <option value="en-US">English</option>
                </select>
              </FormField>
              <FormField label="主题">
                <select value={settings.theme} onChange={(e) => updateSettings({ theme: e.target.value as "dark" | "light" })}
                  style={selectStyle}>
                  <option value="dark">深色</option>
                  <option value="light">浅色</option>
                </select>
              </FormField>
              <FormField label="启动时展开侧边栏">
                <input type="checkbox" checked={!settings.sidebarCollapsed} onChange={(e) => updateSettings({ sidebarCollapsed: !e.target.checked })} />
              </FormField>
              <FormField label="显示右侧检查器">
                <input type="checkbox" checked={settings.inspectorVisible} onChange={(e) => updateSettings({ inspectorVisible: e.target.checked })} />
              </FormField>
            </Panel>
          )}

          {/* Model */}
          {activeTab === "model" && (
            <Panel>
              <FormField label="Provider">
                <input value={settings.provider} onChange={(e) => updateSettings({ provider: e.target.value })} placeholder="deepseek-compatible" style={inputStyle} />
              </FormField>
              <FormField label="Model">
                <input value={settings.model} onChange={(e) => updateSettings({ model: e.target.value })} placeholder="Qwen3-Coder-Next" style={inputStyle} />
              </FormField>
              <FormField label="Base URL">
                <input value={settings.baseUrl} onChange={(e) => updateSettings({ baseUrl: e.target.value })} placeholder="https://api.deepseek.com/v1" style={inputStyle} />
              </FormField>
            </Panel>
          )}

          {/* Credentials */}
          {activeTab === "credentials" && (
            <Panel>
              <FormField label="API Key">
                <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                  <input type={showKey ? "text" : "password"} value={settings.apiKey}
                    onChange={(e) => updateSettings({ apiKey: e.target.value })} placeholder="sk-..." style={inputStyle} />
                  <Button variant="ghost" icon={showKey ? <EyeOff size={14} /> : <Eye size={14} />} onClick={() => setShowKey(!showKey)}>
                    {showKey ? "隐藏" : "显示"}
                  </Button>
                </div>
              </FormField>
              <div className="settings-warning" style={{ marginTop: 10 }}>
                <AlertTriangle size={14} /> 密钥会保存到本机 AppData 的 secrets.json，不写入浏览器 localStorage；界面只显示脱敏值，运行时由后端读取真实密钥。
              </div>
            </Panel>
          )}

          {/* RAG */}
          {activeTab === "rag" && (
            <Panel>
              <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 12 }}>
                <span style={{ color: "var(--text-secondary)", fontSize: 13 }}>服务状态：</span>
                <span style={{
                  display: "inline-flex", alignItems: "center", gap: 5, padding: "4px 10px", borderRadius: 5, fontSize: 12,
                  background: ragStatus === "运行中" ? "color-mix(in srgb, var(--success) 15%, transparent)" : "color-mix(in srgb, var(--warning) 15%, transparent)",
                  color: ragStatus === "运行中" ? "var(--success)" : "var(--warning)",
                  border: `1px solid ${ragStatus === "运行中" ? "color-mix(in srgb, var(--success) 40%, var(--border))" : "color-mix(in srgb, var(--warning) 40%, var(--border))"}`,
                }}>
                  <CheckCircle size={12} /> {ragStatus}
                </span>
              </div>
              <div style={{ display: "flex", gap: 8, marginBottom: 16 }}>
                <Button variant="primary" icon={<Play size={14} />} onClick={() => handleRagAction("start")} disabled={ragBusy}>启动</Button>
                <Button icon={<Square size={14} />} onClick={() => handleRagAction("stop")} disabled={ragBusy}>停止</Button>
                <Button icon={<RefreshCw size={14} />} onClick={() => handleRagAction("restart")} disabled={ragBusy}>重启</Button>
              </div>
              <FormField label="RAG 服务地址">
                <input value={settings.ragServiceUrl} onChange={(e) => updateSettings({ ragServiceUrl: e.target.value })} placeholder="http://127.0.0.1:8787" style={inputStyle} />
              </FormField>
              <FormField label="SQLite 数据库路径">
                <input value={settings.ragDbPath} onChange={(e) => updateSettings({ ragDbPath: e.target.value })} placeholder="beifeng/data/wind.sqlite" style={inputStyle} />
              </FormField>
              <FormField label="启动时自动启动 RAG">
                <input type="checkbox" checked={settings.ragAutoStart} onChange={(e) => updateSettings({ ragAutoStart: e.target.checked })} />
              </FormField>
            </Panel>
          )}

          {/* Paths */}
          {activeTab === "paths" && (
            <Panel>
              <FormField label="知识库"><input value={settings.knowledgeBasePath} onChange={(e) => updateSettings({ knowledgeBasePath: e.target.value })} style={inputStyle} /></FormField>
              <FormField label="故障图谱"><input value={settings.knowledgeGraphPath} onChange={(e) => updateSettings({ knowledgeGraphPath: e.target.value })} style={inputStyle} /></FormField>
              <FormField label="报告输出"><input value={settings.reportsPath} onChange={(e) => updateSettings({ reportsPath: e.target.value })} style={inputStyle} /></FormField>
              <FormField label="记忆数据"><input value={settings.memoryPath} onChange={(e) => updateSettings({ memoryPath: e.target.value })} style={inputStyle} /></FormField>
            </Panel>
          )}

          {/* Safety */}
          {activeTab === "safety" && (
            <Panel>
              <FormField label="高风险操作需要人工确认">
                <input type="checkbox" checked={settings.requireHumanConfirmation} onChange={(e) => updateSettings({ requireHumanConfirmation: e.target.checked })} />
              </FormField>
              <div className="settings-warning" style={{ marginTop: 10 }}>
                <AlertTriangle size={14} /> 远程停机、复位、变桨等高风险操作始终需要人工确认。
              </div>
            </Panel>
          )}

          {/* Advanced */}
          {activeTab === "advanced" && (
            <Panel>
              <FormField label="启用 Connectors（SCADA/CMMS/Weather/UAV）">
                <input type="checkbox" checked={settings.connectorsEnabled} onChange={(e) => updateSettings({ connectorsEnabled: e.target.checked })} />
              </FormField>
              <FormField label="启用记忆层">
                <input type="checkbox" checked={settings.memoryEnabled} onChange={(e) => updateSettings({ memoryEnabled: e.target.checked })} />
              </FormField>
              <div style={{ marginTop: 20, padding: 14, border: "1px solid var(--border)", borderRadius: 8, background: "var(--bg-surface)" }}>
                <h3 style={{ margin: "0 0 8px", fontSize: 13 }}>导出 / 导入</h3>
                <p style={{ color: "var(--text-muted)", fontSize: 12, marginBottom: 10 }}>将当前配置导出为 JSON 文件，或从文件中导入。</p>
                <div style={{ display: "flex", gap: 8 }}>
                  <Button icon={<Download size={14} />} onClick={() => {
                    const blob = new Blob([getSettingsJson()], { type: "application/json" });
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement("a"); a.href = url; a.download = "beifeng-settings.json"; a.click();
                    URL.revokeObjectURL(url);
                  }}>导出</Button>
                  <Button icon={<Upload size={14} />} onClick={() => {
                    const input = document.createElement("input"); input.type = "file"; input.accept = ".json";
                    input.onchange = async (e) => {
                      const file = (e.target as HTMLInputElement).files?.[0];
                      if (file) {
                        const text = await file.text();
                        const result = importSettingsJson(text);
                        if (result.ok) notify("已导入"); else setJsonError(result.errors.join("\n"));
                      }
                    };
                    input.click();
                  }}>导入</Button>
                </div>
              </div>
            </Panel>
          )}
        </div>
      </div>
    </main>
  );
}

const inputStyle: React.CSSProperties = {
  width: "100%", minHeight: 32, border: "1px solid var(--border)", borderRadius: 6,
  padding: "0 9px", background: "var(--bg-app)", color: "var(--text)",
  fontFamily: "var(--font-mono)", fontSize: 12, outline: "none",
};
const selectStyle: React.CSSProperties = {
  minHeight: 32, border: "1px solid var(--border)", borderRadius: 6,
  padding: "0 8px", background: "var(--bg-app)", color: "var(--text)", fontSize: 13,
};

function Panel({ children }: { children: React.ReactNode }) {
  return <section className="panel-block" style={{ padding: 14 }}>{children}</section>;
}
