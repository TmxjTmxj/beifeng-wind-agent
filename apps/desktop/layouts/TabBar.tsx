import { FileText, MessageSquare, Settings } from "lucide-react";
import { useAppState, TabId } from "../store/appState";
import { useI18n } from "../store/i18n";

const tabs: Array<{ id: TabId; labelKey: string; icon: typeof MessageSquare }> = [
  { id: "chat", labelKey: "tabs.chat", icon: MessageSquare },
  { id: "report", labelKey: "tabs.report", icon: FileText },
  { id: "settings", labelKey: "tabs.settings", icon: Settings }
];

export function TabBar() {
  const { activeTab, setActiveTab } = useAppState();
  const { t } = useI18n();

  return (
    <div className="tab-bar" role="tablist">
      {tabs.map((tab) => {
        const Icon = tab.icon;
        return (
          <button
            type="button"
            role="tab"
            aria-selected={activeTab === tab.id}
            className={`tab-item ${activeTab === tab.id ? "is-active" : ""}`}
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
          >
            <Icon size={14} />
            <span>{t(tab.labelKey)}</span>
          </button>
        );
      })}
    </div>
  );
}
