import { useEffect, useState } from "react";
import { CheckCircle2, Circle, Folder, Gauge, ShieldAlert, Zap } from "lucide-react";
import { SystemMonitorPayload, desktopApi } from "../services/desktopApi";
import { useI18n } from "../store/i18n";

const emptyMonitor: SystemMonitorPayload = {
  agent: "Offline",
  rag: "Stopped",
  model: "Qwen3-Coder-Next",
  workspace: "",
  knowledge_docs: 0,
  memory_records: 0,
  graph_nodes: 0,
  reports: 0,
  benchmark: "N/A",
  connectors: "0 configured"
};

export function StatusBar() {
  const { t } = useI18n();
  const [monitor, setMonitor] = useState<SystemMonitorPayload>(emptyMonitor);

  useEffect(() => {
    let mounted = true;
    const load = async () => {
      const payload = await desktopApi.getSystemMonitor();
      if (mounted) {
        setMonitor(payload);
      }
    };
    void load();
    const timer = window.setInterval(load, 5000);
    return () => {
      mounted = false;
      window.clearInterval(timer);
    };
  }, []);

  return (
    <footer className="status-bar">
      <div className="status-segment status-path">
        <Folder size={15} />
        <span>{t("status.workspace")}: {monitor.workspace || t("runtime.offline")}</span>
      </div>
      <div className="status-segment">
        <Circle size={10} className="status-dot" />
        <span>{t("status.rag")}: {monitor.rag}</span>
      </div>
      <div className="status-segment">
        <Zap size={14} />
        <span>{t("status.model")}: {monitor.model}</span>
      </div>
      <div className="status-segment risk-text">
        <ShieldAlert size={14} />
        <span>{t("status.riskPolicy")}: {t("risk.highValue")}</span>
      </div>
      <div className="status-segment">
        <Gauge size={14} />
        <span>{t("status.benchmark")}: {monitor.benchmark}</span>
        <CheckCircle2 size={14} className="success-icon" />
      </div>
    </footer>
  );
}
