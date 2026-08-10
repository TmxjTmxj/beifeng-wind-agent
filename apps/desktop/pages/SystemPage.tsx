import { useEffect, useState } from "react";
import { Activity, Database, Gauge, HardDrive, RefreshCcw, ShieldCheck, Zap } from "lucide-react";
import { Button } from "../components/Button";
import { StatusPill } from "../components/StatusPill";
import { HealthItem, RuntimeStatus, SystemMonitorPayload, desktopApi } from "../services/desktopApi";
import { useAppState } from "../store/appState";
import { useI18n } from "../store/i18n";

const emptyMonitor: SystemMonitorPayload = {
  agent: "Offline",
  rag: "Stopped",
  model: "",
  workspace: "",
  knowledge_docs: 0,
  memory_records: 0,
  graph_nodes: 0,
  reports: 0,
  benchmark: "N/A",
  connectors: "0 configured"
};

function statusTone(status: string): "success" | "warning" | "risk" {
  const lower = status.toLowerCase();
  if (lower.includes("healthy") || lower.includes("ok") || lower.includes("running")) {
    return "success";
  }
  if (lower.includes("offline") || lower.includes("error") || lower.includes("missing")) {
    return "risk";
  }
  return "warning";
}

export function SystemPage() {
  const { t } = useI18n();
  const { workspaceState } = useAppState();
  const [monitor, setMonitor] = useState<SystemMonitorPayload>(emptyMonitor);
  const [health, setHealth] = useState<HealthItem[]>([]);
  const [busy, setBusy] = useState(false);

  const refresh = async () => {
    const [monitorPayload, healthPayload] = await Promise.all([
      desktopApi.getSystemMonitor(),
      desktopApi.getHealthDashboard()
    ]);
    setMonitor(monitorPayload);
    setHealth(healthPayload);
  };

  useEffect(() => {
    void refresh();
  }, [workspaceState.current_workspace]);

  async function updateRuntime(action: () => Promise<RuntimeStatus>) {
    setBusy(true);
    await action();
    await refresh();
    setBusy(false);
  }

  const metrics = [
    [t("runtime.agent"), monitor.agent, <Activity size={17} />],
    ["RAG", monitor.rag, <Database size={17} />],
    [t("status.model"), monitor.model || t("runtime.unknown"), <Zap size={17} />],
    [t("status.benchmark"), monitor.benchmark, <Gauge size={17} />],
    [t("inspector.knowledgeHits"), String(monitor.knowledge_docs), <HardDrive size={17} />],
    [t("inspector.memoryHits"), String(monitor.memory_records), <ShieldCheck size={17} />]
  ] as const;

  return (
    <main className="workbench-page">
      <header className="screen-header">
        <div>
          <h1>{t("system.title")}</h1>
          <p>{monitor.workspace}</p>
        </div>
        <div className="header-actions">
          <Button variant="ghost" icon={<RefreshCcw size={15} />} onClick={refresh}>
            {t("actions.reload")}
          </Button>
          <Button variant="ghost" onClick={() => updateRuntime(desktopApi.startRagService)} disabled={busy}>
            {t("actions.start")} RAG
          </Button>
          <Button variant="ghost" onClick={() => updateRuntime(desktopApi.restartRagService)} disabled={busy}>
            {t("actions.restart")} RAG
          </Button>
          <Button variant="danger" onClick={() => updateRuntime(desktopApi.stopRagService)} disabled={busy}>
            {t("actions.stop")} RAG
          </Button>
        </div>
      </header>

      <section className="system-metric-grid">
        {metrics.map(([label, value, icon]) => (
          <article className="system-metric" key={label}>
            {icon}
            <span>{label}</span>
            <strong>{value}</strong>
          </article>
        ))}
      </section>

      <section className="panel-block health-panel">
        <h2>{t("system.monitor")}</h2>
        <div className="health-table">
          <div className="health-row health-head">
            <span>{t("system.status")}</span>
            <span>{t("health.health")}</span>
            <span>{t("health.warning")}</span>
            <span>{t("system.path")}</span>
            <span>{t("system.updated")}</span>
            <span>{t("system.responseTime")}</span>
            <span>{t("system.healthCheckResult")}</span>
            <span>{t("system.suggestion")}</span>
          </div>
          {health.length ? (
            health.map((item) => (
              <div className="health-row" key={`${item.name}-${item.path}`}>
                <span>
                  <strong>{item.name}</strong>
                  <StatusPill label={item.status} tone={statusTone(item.status)} />
                </span>
                <span><StatusPill label={statusTone(item.status) === "success" ? "OK" : "Check"} tone={statusTone(item.status)} /></span>
                <span>{item.error ?? "-"}</span>
                <span title={item.path}>{item.path}</span>
                <span>{item.updated}</span>
                <span>{item.response_time_ms} ms</span>
                <span>{item.health_check_result}</span>
                <span>{item.suggestion}</span>
              </div>
            ))
          ) : (
            <p className="empty-copy">{t("inspector.empty")}</p>
          )}
        </div>
      </section>
    </main>
  );
}
