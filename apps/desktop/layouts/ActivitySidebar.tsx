import {
  Archive,
  BarChart3,
  Brain,
  ChevronLeft,
  ChevronRight,
  Database,
  FileText,
  FolderTree,
  Home,
  MessageSquare,
  Plug,
  Settings,
  ShieldCheck,
  TerminalSquare,
  Workflow
} from "lucide-react";
import { Button } from "../components/Button";
import { Section, useAppState } from "../store/appState";
import { useI18n } from "../store/i18n";

const navItems: Array<{ id: Section; labelKey: string; icon: typeof FolderTree }> = [
  { id: "home", labelKey: "nav.home", icon: Home },
  { id: "workspace", labelKey: "nav.workspace", icon: FolderTree },
  { id: "chats", labelKey: "nav.chats", icon: MessageSquare },
  { id: "files", labelKey: "nav.files", icon: Archive },
  { id: "memory", labelKey: "nav.memory", icon: Brain },
  { id: "reports", labelKey: "nav.reports", icon: FileText },
  { id: "benchmark", labelKey: "nav.benchmark", icon: BarChart3 },
  { id: "skills", labelKey: "nav.skills", icon: Workflow },
  { id: "connectors", labelKey: "nav.connectors", icon: Plug },
  { id: "system", labelKey: "nav.system", icon: ShieldCheck },
  { id: "console", labelKey: "nav.console", icon: TerminalSquare },
  { id: "settings", labelKey: "nav.settings", icon: Settings }
];

export function ActivitySidebar() {
  const { activeSection, setActiveSection, sidebarCollapsed, setSidebarCollapsed } = useAppState();
  const { t } = useI18n();

  return (
    <aside className={`activity-sidebar ${sidebarCollapsed ? "is-collapsed" : ""}`} aria-label="Primary navigation">
      <div className="activity-nav">
        {navItems.map((item) => {
          const Icon = item.icon;
          const active = activeSection === item.id;
          return (
            <button
              className={`activity-item ${active ? "is-active" : ""}`}
              key={item.id}
              title={t(item.labelKey)}
              onClick={() => setActiveSection(item.id)}
              type="button"
            >
              <Icon size={19} />
              <span>{t(item.labelKey)}</span>
            </button>
          );
        })}
      </div>
      <div className="activity-footer">
        <ShieldCheck size={18} />
        <Button
          className="collapse-button"
          variant="ghost"
          icon={sidebarCollapsed ? <ChevronRight size={16} /> : <ChevronLeft size={16} />}
          title={sidebarCollapsed ? t("nav.expand") : t("nav.collapse")}
          onClick={() => setSidebarCollapsed(!sidebarCollapsed)}
        />
      </div>
    </aside>
  );
}
