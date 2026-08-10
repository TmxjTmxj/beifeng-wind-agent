import { useEffect, useMemo, useState } from "react";
import { Clipboard, FileText, RefreshCcw, Save } from "lucide-react";
import { Button } from "../components/Button";
import { ConsolePayload, desktopApi } from "../services/desktopApi";
import { useI18n } from "../store/i18n";

const emptyConsole: ConsolePayload = {
  prompt: "",
  system_context: {},
  tool_calls: [],
  raw_tool_results: "",
  memory_context: [],
  knowledge_context: "",
  graph_context: "",
  risk_assessment_json: {},
  report_path: null,
  runtime_logs: "",
  event_timeline: [],
  raw_event_json: [],
  metrics: {
    tool_duration_ms: 0,
    tool_success_rate: 1,
    knowledge_query_latency_ms: 0,
    rag_latency_ms: 0,
    memory_query_latency_ms: 0,
    connector_latency_ms: 0,
    total_events: 0,
    error_events: 0
  }
};

function JsonBlock({ value }: { value: unknown }) {
  return <pre className="console-pre">{JSON.stringify(value, null, 2)}</pre>;
}

export function AgentConsolePage() {
  const { t } = useI18n();
  const [payload, setPayload] = useState<ConsolePayload>(emptyConsole);
  const [message, setMessage] = useState("");

  const rawJson = useMemo(() => JSON.stringify(payload, null, 2), [payload]);

  const refresh = async () => {
    setPayload(await desktopApi.getAgentConsole());
  };

  useEffect(() => {
    void refresh();
  }, []);

  async function copyJson() {
    await navigator.clipboard?.writeText(rawJson);
    setMessage(t("actions.copyJson"));
  }

  async function saveLog() {
    const path = await desktopApi.saveDebugLog(rawJson);
    setMessage(path || t("actions.saveLog"));
  }

  return (
    <main className="workbench-page console-page">
      <header className="screen-header">
        <div>
          <h1>{t("console.title")}</h1>
          <p>{t("console.runtimeLogs")}</p>
        </div>
        <div className="header-actions">
          <Button variant="ghost" icon={<RefreshCcw size={15} />} onClick={refresh}>
            {t("actions.reload")}
          </Button>
          <Button variant="ghost" icon={<Clipboard size={15} />} onClick={copyJson}>
            {t("actions.copyJson")}
          </Button>
          <Button variant="primary" icon={<Save size={15} />} onClick={saveLog}>
            {t("actions.saveLog")}
          </Button>
        </div>
      </header>

      {message ? <div className="settings-message">{message}</div> : null}

      <section className="console-metric-grid">
        <article><span>{t("console.totalEvents")}</span><strong>{payload.metrics.total_events}</strong></article>
        <article><span>{t("console.toolSuccessRate")}</span><strong>{Math.round(payload.metrics.tool_success_rate * 100)}%</strong></article>
        <article><span>{t("console.toolDuration")}</span><strong>{payload.metrics.tool_duration_ms} ms</strong></article>
        <article><span>{t("console.ragLatency")}</span><strong>{payload.metrics.rag_latency_ms} ms</strong></article>
        <article><span>{t("console.knowledgeLatency")}</span><strong>{payload.metrics.knowledge_query_latency_ms} ms</strong></article>
        <article><span>{t("console.memoryLatency")}</span><strong>{payload.metrics.memory_query_latency_ms} ms</strong></article>
        <article><span>{t("console.connectorLatency")}</span><strong>{payload.metrics.connector_latency_ms} ms</strong></article>
        <article><span>{t("console.errorEvents")}</span><strong>{payload.metrics.error_events}</strong></article>
      </section>

      <section className="console-layout">
        <article className="panel-block console-wide">
          <h2>{t("console.eventTimeline")}</h2>
          <div className="event-timeline">
            {payload.event_timeline.length ? payload.event_timeline.map((event) => (
              <div className="event-row" key={`${event.timestamp}-${event.session_id}-${event.event_type}`}>
                <span>{event.timestamp}</span>
                <strong>{event.event_type}</strong>
                <small>{event.session_id}</small>
                <code>{JSON.stringify(event.payload)}</code>
              </div>
            )) : <p className="empty-copy">{t("inspector.empty")}</p>}
          </div>
        </article>
        <article className="panel-block console-wide">
          <h2>{t("console.rawEventJson")}</h2>
          <JsonBlock value={payload.raw_event_json} />
        </article>
        <article className="panel-block">
          <h2>{t("console.prompt")}</h2>
          <pre className="console-pre">{payload.prompt || t("inspector.empty")}</pre>
        </article>
        <article className="panel-block">
          <h2>{t("console.systemContext")}</h2>
          <JsonBlock value={payload.system_context} />
        </article>
        <article className="panel-block">
          <h2>{t("console.toolCalls")}</h2>
          <JsonBlock value={payload.tool_calls} />
        </article>
        <article className="panel-block">
          <h2>{t("console.rawToolResults")}</h2>
          <pre className="console-pre">{payload.raw_tool_results || t("inspector.empty")}</pre>
        </article>
        <article className="panel-block">
          <h2>{t("console.memoryContext")}</h2>
          <JsonBlock value={payload.memory_context} />
        </article>
        <article className="panel-block">
          <h2>{t("console.knowledgeContext")}</h2>
          <pre className="console-pre">{payload.knowledge_context || t("inspector.empty")}</pre>
        </article>
        <article className="panel-block">
          <h2>{t("console.graphContext")}</h2>
          <pre className="console-pre">{payload.graph_context || t("inspector.empty")}</pre>
        </article>
        <article className="panel-block">
          <h2>{t("console.riskAssessment")}</h2>
          <JsonBlock value={payload.risk_assessment_json} />
        </article>
        <article className="panel-block">
          <h2>{t("console.reportPath")}</h2>
          <pre className="console-pre">
            {payload.report_path ? (
              <>
                <FileText size={14} />
                {payload.report_path}
              </>
            ) : (
              t("inspector.empty")
            )}
          </pre>
        </article>
        <article className="panel-block console-wide">
          <h2>{t("console.runtimeLogs")}</h2>
          <pre className="console-pre">{payload.runtime_logs || t("inspector.empty")}</pre>
        </article>
      </section>
    </main>
  );
}
