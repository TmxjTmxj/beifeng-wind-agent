import { useEffect, useState } from "react";
import { Archive, ExternalLink, FolderOpen, Plus, RefreshCw, SearchCode, Trash2 } from "lucide-react";
import { Button } from "../components/Button";
import { StatusPill } from "../components/StatusPill";
import { FileNode, desktopApi } from "../services/desktopApi";
import { useI18n } from "../store/i18n";
import { useWorkspace } from "../hooks/useWorkspace";

export function WorkspacePage() {
  const { t } = useI18n();
  const {
    workspaceState,
    refresh,
    selectFolder,
    createWorkspace,
    importWorkspace,
    setWorkspace,
    archiveWorkspace,
    removeWorkspace,
    openInVSCode,
    revealInExplorer
  } = useWorkspace();
  const [workspacePath, setWorkspacePath] = useState("");
  const [tree, setTree] = useState<FileNode[]>([]);
  const [message, setMessage] = useState("");
  const activeWorkspaces = workspaceState.recent_workspaces.filter((workspace) => !workspace.archived);
  const archivedWorkspaces = workspaceState.recent_workspaces.filter((workspace) => workspace.archived);

  const run = async (operation: () => Promise<unknown>, success = t("message.ready")) => {
    try {
      await operation();
      setMessage(success);
      await loadTree();
    } catch (error) {
      setMessage(`${t("message.error")}: ${String(error)}`);
    }
  };

  const loadTree = async () => {
    setTree(await desktopApi.listWorkspaceFiles("workspace"));
  };

  useEffect(() => {
    void loadTree();
  }, [workspaceState.current_workspace]);

  return (
    <main className="workbench-page">
      <header className="screen-header">
        <div>
          <h1>{t("workspace.title")}</h1>
          <p>{t("workspace.subtitle")}</p>
        </div>
        <div className="header-actions">
          <Button variant="ghost" icon={<RefreshCw size={15} />} onClick={() => run(refresh)}>
            {t("actions.reload")}
          </Button>
          <Button variant="primary" icon={<FolderOpen size={15} />} onClick={() => run(selectFolder)}>
            {t("actions.openFolder")}
          </Button>
          <Button icon={<FolderOpen size={15} />} onClick={() => run(importWorkspace)}>
            {t("actions.importWorkspace")}
          </Button>
          <Button icon={<SearchCode size={15} />} onClick={() => run(() => openInVSCode(workspaceState.current_workspace ?? undefined))}>
            {t("actions.openVSCode")}
          </Button>
          <Button icon={<ExternalLink size={15} />} onClick={() => run(() => revealInExplorer(workspaceState.current_workspace ?? undefined))}>
            {t("actions.reveal")}
          </Button>
        </div>
      </header>

      {message ? <div className={`settings-message ${message.startsWith(t("message.error")) ? "is-error" : ""}`}>{message}</div> : null}

      <section className="workspace-create-row">
        <label className="settings-field">
          {t("workspace.createPath")}
          <input value={workspacePath} onChange={(event) => setWorkspacePath(event.target.value)} placeholder="D:\\WindFarm_A" />
        </label>
        <Button
          variant="primary"
          icon={<Plus size={15} />}
          onClick={() => run(() => createWorkspace(workspacePath), t("actions.createWorkspace"))}
          disabled={!workspacePath.trim()}
        >
          {t("actions.createWorkspace")}
        </Button>
      </section>

      <section className="workspace-grid">
        <article className="panel-block">
          <h2>{t("workspace.recent")}</h2>
          <div className="workspace-cards">
            {activeWorkspaces.map((workspace) => (
              <div className="workspace-card" key={workspace.path}>
                <strong>{workspace.name}</strong>
                <span>{workspace.path}</span>
                <div>
                  <StatusPill label={workspace.path === workspaceState.current_workspace ? t("workspace.current") : t("workspace.recent")} />
                  <StatusPill label={workspace.last_opened} tone="success" />
                  <Button className="inline-row-button" onClick={() => run(() => setWorkspace(workspace.path), t("actions.switch"))}>
                    {t("actions.switch")}
                  </Button>
                  <Button className="inline-row-button" icon={<Archive size={13} />} onClick={() => run(() => archiveWorkspace(workspace.path), t("actions.archive"))}>
                    {t("actions.archive")}
                  </Button>
                  <Button className="inline-row-button" icon={<Trash2 size={13} />} onClick={() => run(() => removeWorkspace(workspace.path), t("actions.remove"))}>
                    {t("actions.remove")}
                  </Button>
                </div>
              </div>
            ))}
            {activeWorkspaces.length === 0 ? <p className="empty-copy">{t("workspace.noRecent")}</p> : null}
          </div>
        </article>
        <article className="panel-block">
          <h2>{t("workspace.tree")}</h2>
          <div className="tree-list roomy">
            {tree.slice(0, 80).map((node) => (
              <div className="tree-row" style={{ paddingLeft: 10 + node.depth * 18 }} key={node.path}>
                <span>{node.kind === "folder" ? "folder" : "file"}</span>
                <strong>{node.name}</strong>
              </div>
            ))}
            {tree.length === 0 ? <p className="empty-copy">{t("inspector.empty")}</p> : null}
          </div>
        </article>
        <article className="panel-block">
          <h2>{t("workspace.archive")}</h2>
          {archivedWorkspaces.length === 0 ? (
            <div className="empty-state">
              <Archive size={22} />
              <p>{t("workspace.noArchive")}</p>
            </div>
          ) : (
            <div className="workspace-cards">
              {archivedWorkspaces.map((workspace) => (
                <div className="workspace-card" key={workspace.path}>
                  <strong>{workspace.name}</strong>
                  <span>{workspace.path}</span>
                  <div>
                    <Button className="inline-row-button" onClick={() => run(() => setWorkspace(workspace.path), t("actions.switch"))}>
                      {t("actions.switch")}
                    </Button>
                    <Button className="inline-row-button" icon={<Trash2 size={13} />} onClick={() => run(() => removeWorkspace(workspace.path), t("actions.remove"))}>
                      {t("actions.remove")}
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </article>
      </section>
    </main>
  );
}
