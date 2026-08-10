import { useEffect, useMemo, useState } from "react";
import { Archive, File, Folder, RefreshCw } from "lucide-react";
import { Button } from "../components/Button";
import { PanelHeader } from "../components/PanelHeader";
import { SearchBox } from "../components/SearchBox";
import { ChatSummary, FileNode, ReportSummary, desktopApi } from "../services/desktopApi";
import { Section, useAppState } from "../store/appState";
import { useI18n } from "../store/i18n";

function rowMatches(value: string, query: string) {
  return value.toLowerCase().includes(query.trim().toLowerCase());
}

function TreeRow({ depth, name, kind, path }: FileNode) {
  return (
    <button className="tree-row" style={{ paddingLeft: 10 + depth * 16 }} onClick={() => desktopApi.openFile(path)} type="button">
      {kind === "folder" ? <Folder size={14} /> : <File size={14} />}
      <span>{name}</span>
    </button>
  );
}

function ChatList({ refreshKey, workspaceKey, query }: { refreshKey: number; workspaceKey: string | null; query: string }) {
  const { t } = useI18n();
  const [items, setItems] = useState<ChatSummary[]>([]);

  useEffect(() => {
    void desktopApi.listChats().then(setItems);
  }, [refreshKey, workspaceKey]);

  const visible = useMemo(
    () => items.filter((chat) => rowMatches(`${chat.title} ${chat.preview} ${chat.modified}`, query)),
    [items, query],
  );

  return (
    <div className="resource-list">
      {visible.length ? visible.map((chat) => (
        <button className="resource-row" key={chat.path} onClick={() => desktopApi.openFile(chat.path)} type="button">
          <span>{chat.title}</span>
          <small>{chat.archived ? t("actions.archive") : chat.modified}</small>
        </button>
      )) : <p className="empty-copy">{t("inspector.empty")}</p>}
    </div>
  );
}

function ReportList({ refreshKey, workspaceKey, query }: { refreshKey: number; workspaceKey: string | null; query: string }) {
  const { t } = useI18n();
  const [items, setItems] = useState<ReportSummary[]>([]);

  useEffect(() => {
    void desktopApi.listReports().then(setItems);
  }, [refreshKey, workspaceKey]);

  const visible = useMemo(
    () => items.filter((report) => rowMatches(`${report.title} ${report.report_type} ${report.modified}`, query)),
    [items, query],
  );

  return (
    <div className="resource-list">
      {visible.length ? visible.map((report) => (
        <button className="resource-row" key={report.path} onClick={() => desktopApi.openFile(report.path)} type="button">
          <span>{report.title}</span>
          <small>{report.modified}</small>
        </button>
      )) : <p className="empty-copy">{t("inspector.empty")}</p>}
    </div>
  );
}

function SettingsCategories({ query }: { query: string }) {
  const { t } = useI18n();
  const keys = [
    "settings.general",
    "settings.model",
    "settings.memory",
    "settings.knowledge",
    "settings.connector",
    "settings.risk",
    "settings.benchmark",
    "settings.advanced"
  ];

  return (
    <div className="resource-list">
      {keys.filter((key) => rowMatches(t(key), query)).map((key) => (
        <button className="resource-row" key={key} type="button">
          <span>{t(key)}</span>
        </button>
      ))}
    </div>
  );
}

function panelTitle(section: Section, t: (key: string) => string) {
  const keyMap: Record<Section, string> = {
    home: "nav.home",
    workspace: "nav.workspace",
    chats: "nav.chats",
    files: "nav.files",
    memory: "nav.memory",
    reports: "nav.reports",
    benchmark: "nav.benchmark",
    skills: "nav.skills",
    connectors: "nav.connectors",
    system: "nav.system",
    console: "nav.console",
    settings: "nav.settings"
  };
  return t(keyMap[section]);
}

export function ResourcePanel() {
  const { activeSection, workspaceState } = useAppState();
  const { t } = useI18n();
  const [refreshKey, setRefreshKey] = useState(0);
  const [treeRows, setTreeRows] = useState<FileNode[]>([]);
  const [query, setQuery] = useState("");

  const showSearch = activeSection !== "benchmark";

  useEffect(() => {
    setQuery("");
  }, [activeSection]);

  useEffect(() => {
    if (activeSection === "workspace" || activeSection === "files") {
      void desktopApi.listWorkspaceFiles(activeSection === "files" ? "workspace" : "workspace").then((rows) => setTreeRows(rows.slice(0, 80)));
    }
  }, [activeSection, refreshKey, workspaceState.current_workspace]);

  const visibleTreeRows = useMemo(
    () => treeRows.filter((row) => rowMatches(`${row.name} ${row.path}`, query)),
    [treeRows, query],
  );

  return (
    <aside className="resource-panel">
      <PanelHeader
        title={panelTitle(activeSection, t)}
        actions={
          <Button variant="ghost" icon={<RefreshCw size={14} />} onClick={() => setRefreshKey((value) => value + 1)} title={t("actions.reload")} />
        }
      />
      {showSearch ? <SearchBox value={query} onChange={setQuery} /> : null}
      <p className="resource-description">{t(`resource.${activeSection}.description`)}</p>
      {activeSection === "workspace" || activeSection === "files" ? (
        <div className="tree-list">
          {visibleTreeRows.map((row) => (
            <TreeRow key={row.path} {...row} />
          ))}
          {visibleTreeRows.length === 0 ? <p className="empty-copy">{t("inspector.empty")}</p> : null}
        </div>
      ) : null}
      {activeSection === "chats" ? <ChatList refreshKey={refreshKey} workspaceKey={workspaceState.current_workspace} query={query} /> : null}
      {activeSection === "reports" ? <ReportList refreshKey={refreshKey} workspaceKey={workspaceState.current_workspace} query={query} /> : null}
      {activeSection === "settings" ? <SettingsCategories query={query} /> : null}
      {["memory", "benchmark", "skills", "connectors", "system", "console"].includes(activeSection) ? (
        <div className="resource-list">
          <button className="resource-row is-active" type="button"><span>{panelTitle(activeSection, t)} {t("resource.overview")}</span></button>
          <button className="resource-row" type="button" onClick={() => setRefreshKey((value) => value + 1)}>
            <span>{t("resource.active")}</span><small>{t("actions.reload")}</small>
          </button>
          <button className="resource-row" type="button" onClick={() => desktopApi.revealInExplorer(workspaceState.current_workspace ?? undefined)}>
            <span><Archive size={13} /> Workspace</span><small>Explorer</small>
          </button>
        </div>
      ) : null}
    </aside>
  );
}
