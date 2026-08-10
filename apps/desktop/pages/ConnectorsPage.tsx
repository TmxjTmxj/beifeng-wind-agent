import { useEffect, useMemo, useState } from "react";
import { Activity, Database, FileJson, RefreshCw, Wifi } from "lucide-react";
import { Button } from "../components/Button";
import { desktopApi, FileNode, HealthItem, SystemMonitorPayload } from "../services/desktopApi";
import { useI18n } from "../store/i18n";

type ConnectorDefinition = {
  id: string;
  name: string;
  source: string;
  purpose: string;
  schema: string;
};

const connectorDefinitions: ConnectorDefinition[] = [
  {
    id: "scada",
    name: "SCADA",
    source: "CSV / JSON / REST",
    purpose: "风机实时点位、告警和趋势数据",
    schema: "beifeng/connectors/scada/schema.json",
  },
  {
    id: "cmms",
    name: "CMMS",
    source: "工单 / 备件 / 检修记录",
    purpose: "维修历史、备件消耗和闭环工单",
    schema: "beifeng/connectors/cmms/schema.json",
  },
  {
    id: "weather",
    name: "Weather",
    source: "JSON / REST",
    purpose: "环境风速、温度、湿度和极端天气上下文",
    schema: "beifeng/connectors/weather/schema.json",
  },
  {
    id: "uav",
    name: "UAV",
    source: "巡检影像 / 缺陷清单",
    purpose: "叶片、塔筒和外观巡检缺陷",
    schema: "beifeng/connectors/uav/schema.json",
  },
];

const emptyMonitor: SystemMonitorPayload = {
  agent: "-",
  rag: "-",
  model: "-",
  workspace: "-",
  knowledge_docs: 0,
  memory_records: 0,
  graph_nodes: 0,
  reports: 0,
  benchmark: "N/A",
  connectors: "0 configured",
};

function normalizePath(path: string) {
  return path.replace(/\\/g, "/").toLowerCase();
}

function statusClass(status: string) {
  const normalized = status.toLowerCase();
  if (normalized.includes("healthy") || normalized.includes("running") || normalized.includes("ready")) {
    return "status-pill status-pill-success";
  }
  if (normalized.includes("warning") || normalized.includes("degraded") || normalized.includes("schema")) {
    return "status-pill status-pill-warning";
  }
  return "status-pill status-pill-risk";
}

function findHealth(health: HealthItem[], name: string) {
  return health.find((item) => item.name.toLowerCase() === name.toLowerCase());
}

export function ConnectorsPage() {
  const { t } = useI18n();
  const [files, setFiles] = useState<FileNode[]>([]);
  const [health, setHealth] = useState<HealthItem[]>([]);
  const [monitor, setMonitor] = useState<SystemMonitorPayload>(emptyMonitor);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = async () => {
    setLoading(true);
    setError(null);
    try {
      const [fileRows, healthRows, monitorPayload] = await Promise.all([
        desktopApi.listWorkspaceFiles("workspace"),
        desktopApi.getHealthDashboard(),
        desktopApi.getSystemMonitor(),
      ]);
      setFiles(fileRows);
      setHealth(healthRows);
      setMonitor(monitorPayload);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
  }, []);

  const connectorRows = useMemo(() => {
    const fileSet = new Set(files.map((file) => normalizePath(file.path)));
    return connectorDefinitions.map((connector) => {
      const schemaPath = normalizePath(connector.schema);
      const hasSchema = Array.from(fileSet).some((path) => path.endsWith(schemaPath));
      return {
        ...connector,
        hasSchema,
        status: hasSchema ? "Schema ready" : "Schema missing",
      };
    });
  }, [files]);

  const registry = findHealth(health, "Connector Registry");
  const ragService = findHealth(health, "RAG Service");
  const modelProvider = findHealth(health, "Model Provider");

  return (
    <main className="workbench-page">
      <header className="screen-header">
        <div>
          <h1>{t("nav.connectors")}</h1>
          <p>{t("resource.connectors.description")}</p>
        </div>
        <div className="header-actions">
          <Button variant="ghost" icon={<RefreshCw size={15} />} onClick={load} disabled={loading}>
            {loading ? "刷新中" : "刷新"}
          </Button>
        </div>
      </header>

      {error && (
        <section className="settings-warning">
          <Wifi size={14} /> 连接器状态读取失败：{error}
        </section>
      )}

      <section className="metrics-grid">
        <article className="panel-block metric-card">
          <Database size={16} />
          <strong>{monitor.connectors}</strong>
          <small>工作区连接器目录</small>
        </article>
        <article className="panel-block metric-card">
          <Activity size={16} />
          <strong>{ragService?.status ?? monitor.rag}</strong>
          <small>RAG Service</small>
        </article>
        <article className="panel-block metric-card">
          <FileJson size={16} />
          <strong>{registry?.status ?? "-"}</strong>
          <small>Connector Registry</small>
        </article>
        <article className="panel-block metric-card">
          <Wifi size={16} />
          <strong>{modelProvider?.status ?? "-"}</strong>
          <small>Model Provider</small>
        </article>
      </section>

      <section className="connector-grid">
        {connectorRows.map((connector) => (
          <article className="panel-block connector-card" key={connector.id}>
            <strong>{connector.name}</strong>
            <span className={statusClass(connector.status)}>{connector.status}</span>
            <p>{connector.purpose}</p>
            <p>{connector.source}</p>
            <small>{connector.schema}</small>
          </article>
        ))}
      </section>
    </main>
  );
}
