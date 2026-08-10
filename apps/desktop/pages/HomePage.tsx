import { useEffect, useState } from "react";
import { FileCog, FolderOpen, MessageSquarePlus, Play, Settings, Workflow } from "lucide-react";
import { Button } from "../components/Button";
import { StatusPill } from "../components/StatusPill";
import { RuntimeStatus, desktopApi } from "../services/desktopApi";
import { useAppState } from "../store/appState";
import { useI18n } from "../store/i18n";
import { useSettings } from "../store/settingsStore";

export function HomePage() {
  const { t } = useI18n();
  const { settings } = useSettings();
  const {
    workspaceState, workspaceLoading, selectWorkspaceFolder,
    setCurrentWorkspacePath, setActiveSection, setInspectorCollapsed,
  } = useAppState();
  const currentWorkspace = workspaceState.current_workspace;
  const activeRecent = workspaceState.recent_workspaces.filter((w) => !w.archived).slice(0, 5);

  const [runtime, setRuntime] = useState<RuntimeStatus>({ agent: "检查中", rag: "检查中" });
  useEffect(() => {
    let cancelled = false;
    const poll = async () => { const s = await desktopApi.getAgentStatus(); if (!cancelled) setRuntime(s); };
    poll();
    const t = setInterval(poll, 8000);
    return () => { cancelled = true; clearInterval(t); };
  }, [workspaceState.current_workspace]);

  async function openWorkspace() { await selectWorkspaceFolder(); setActiveSection("workspace"); }
  async function openRecent(path: string) { await setCurrentWorkspacePath(path); setActiveSection("workspace"); }
  async function startRag() { await desktopApi.startRagService(); setActiveSection("system"); }
  function newChat() { setActiveSection("chats"); setInspectorCollapsed(false); }

  return (
    <main className="home-page">
      <section className="home-hero">
        <div className="home-mark"><Workflow size={30} /></div>
        <div>
          <p className="eyebrow">{t("home.eyebrow")}</p>
          <h1>{t("home.title")}</h1>
          <p>{t("home.subtitle")}</p>
          <div className="home-status-row">
            <StatusPill label={currentWorkspace ? t("workspace.current") : t("home.selectPrompt")} tone={currentWorkspace ? "success" : "warning"} />
            <span>{currentWorkspace ?? t("home.noWorkspace")}</span>
          </div>
          <div className="home-status-row" style={{ marginTop: 6 }}>
            <StatusPill label={runtime.agent} tone={runtime.agent === "Running" ? "success" : runtime.agent === "Offline" ? "risk" : "warning"} />
            <span>Agent: {runtime.agent}</span>
            <StatusPill label={`RAG ${runtime.rag}`} tone={runtime.rag === "Running" ? "success" : "warning"} />
          </div>
        </div>
      </section>

      <section className="home-action-grid">
        <button className="home-action-card" onClick={newChat}><MessageSquarePlus size={20} /><strong>{t("actions.newChat")}</strong><span>{t("home.newChatHint")}</span></button>
        <button className="home-action-card" onClick={openWorkspace}><FolderOpen size={20} /><strong>{t("actions.openFolder")}</strong><span>{t("home.openWorkspaceHint")}</span></button>
        <button className="home-action-card" onClick={startRag}><Play size={20} /><strong>{t("home.startRag")}</strong><span>{t("home.startRagHint")}</span></button>
        <button className="home-action-card" onClick={() => setActiveSection("settings")}><Settings size={20} /><strong>{t("settings.title")}</strong><span>{t("home.settingsHint")}</span></button>
      </section>

      <section className="home-content-grid">
        <article className="panel-block">
          <h2>{t("workspace.recent")}</h2>
          <div className="home-recent-list">
            {activeRecent.length ? activeRecent.map((w) => (
              <button className="home-recent-row" key={w.path} onClick={() => openRecent(w.path)}>
                <FileCog size={16} /><div><strong>{w.name}</strong><span>{w.path}</span></div><small>{w.last_opened}</small>
              </button>
            )) : (
              <div className="empty-state"><FolderOpen size={22} /><p>{t("workspace.noRecent")}</p><Button variant="primary" icon={<FolderOpen size={15} />} onClick={openWorkspace}>{t("actions.openFolder")}</Button></div>
            )}
          </div>
        </article>
        <article className="panel-block home-workspace-card">
          <h2>{t("home.workspaceStatus")}</h2>
          <dl>
            <div><dt>{t("status.workspace")}</dt><dd>{currentWorkspace ?? t("home.noWorkspace")}</dd></div>
            <div><dt>{t("status.model")}</dt><dd>{settings.model || "未配置"}</dd></div>
            <div><dt>{t("status.provider")}</dt><dd>{settings.provider || "未配置"}</dd></div>
            <div><dt>{t("status.rag")}</dt><dd>{runtime.rag === "Running" ? "运行中" : runtime.rag === "Stopped" ? "已停止" : runtime.rag}</dd></div>
          </dl>
        </article>
      </section>
    </main>
  );
}
