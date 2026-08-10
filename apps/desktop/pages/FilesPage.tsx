import { useEffect, useState } from "react";
import { ExternalLink, File, Folder, Pencil, RefreshCw, SearchCode } from "lucide-react";
import { Button } from "../components/Button";
import { FileNode, desktopApi } from "../services/desktopApi";
import { useAppState } from "../store/appState";
import { useI18n } from "../store/i18n";

const scopes = [
  ["workspace", "files.workspace"],
  ["knowledge", "files.knowledge"],
  ["memory", "files.memory"],
  ["reports", "files.reports"]
] as const;

export function FilesPage() {
  const { t } = useI18n();
  const { workspaceState } = useAppState();
  const [scope, setScope] = useState("workspace");
  const [files, setFiles] = useState<FileNode[]>([]);
  const [selected, setSelected] = useState<FileNode | null>(null);
  const [message, setMessage] = useState("");

  const refresh = async () => {
    const rows = await desktopApi.listWorkspaceFiles(scope);
    setFiles(rows);
    setSelected((current) => rows.find((row) => row.path === current?.path) ?? rows[0] ?? null);
  };

  const run = async (operation: () => Promise<unknown>, success = t("message.ready")) => {
    try {
      await operation();
      setMessage(success);
      await refresh();
    } catch (error) {
      setMessage(`${t("message.error")}: ${String(error)}`);
    }
  };

  useEffect(() => {
    void refresh();
  }, [scope, workspaceState.current_workspace]);

  async function renameSelected() {
    if (!selected) {
      return;
    }
    const nextName = window.prompt(t("actions.rename"), selected.name);
    if (!nextName) {
      return;
    }
    await run(async () => {
      const nextPath = await desktopApi.renamePath(selected.path, nextName);
      setSelected({ ...selected, name: nextName, path: nextPath });
    }, t("actions.rename"));
  }

  return (
    <main className="workbench-page">
      <header className="screen-header">
        <div>
          <h1>{t("files.title")}</h1>
          <p>{t("resource.files.description")}</p>
        </div>
        <div className="header-actions">
          <label className="select-control">
            {t("files.scope")}
            <select value={scope} onChange={(event) => setScope(event.target.value)}>
              {scopes.map(([value, label]) => <option value={value} key={value}>{t(label)}</option>)}
            </select>
          </label>
          <Button icon={<RefreshCw size={15} />} onClick={refresh}>{t("actions.reload")}</Button>
          <Button icon={<File size={15} />} disabled={!selected || selected.kind !== "file"} onClick={() => selected && run(() => desktopApi.openFile(selected.path))}>
            {t("actions.openFile")}
          </Button>
          <Button icon={<SearchCode size={15} />} disabled={!selected} onClick={() => selected && run(() => desktopApi.openInVSCode(selected.path))}>
            {t("actions.openVSCode")}
          </Button>
          <Button icon={<ExternalLink size={15} />} disabled={!selected} onClick={() => selected && run(() => desktopApi.revealInExplorer(selected.path))}>
            {t("actions.reveal")}
          </Button>
          <Button icon={<Pencil size={15} />} disabled={!selected} onClick={renameSelected}>{t("actions.rename")}</Button>
        </div>
      </header>

      {message ? <div className={`settings-message ${message.startsWith(t("message.error")) ? "is-error" : ""}`}>{message}</div> : null}

      <section className="split-content">
        <article className="panel-block list-pane">
          <h2>{t("files.workspace")}</h2>
          <div className="tree-list roomy">
            {files.map((node) => (
              <button
                type="button"
                className={`file-tree-row ${selected?.path === node.path ? "is-active" : ""}`}
                style={{ paddingLeft: 10 + node.depth * 18 }}
                key={node.path}
                onClick={() => setSelected(node)}
              >
                {node.kind === "folder" ? <Folder size={14} /> : <File size={14} />}
                <strong>{node.name}</strong>
                <small>{node.modified}</small>
              </button>
            ))}
            {files.length === 0 ? <p className="empty-copy">{t("inspector.empty")}</p> : null}
          </div>
        </article>
        <article className="panel-block markdown-preview">
          <h2>{selected?.name ?? t("actions.openFile")}</h2>
          {selected ? (
            <div className="metadata-list">
              <span>{t("system.path")}</span><strong>{selected.path}</strong>
              <span>{t("system.updated")}</span><strong>{selected.modified}</strong>
              <span>{t("files.scope")}</span><strong>{selected.kind}</strong>
              <span>{t("files.size")}</span><strong>{selected.size}</strong>
            </div>
          ) : (
            <p className="empty-copy">{t("inspector.empty")}</p>
          )}
        </article>
      </section>
    </main>
  );
}
