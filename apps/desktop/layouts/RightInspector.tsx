import { useEffect, useState } from "react";
import { ChevronLeft, ChevronRight, RefreshCw } from "lucide-react";
import { Button } from "../components/Button";
import { PanelHeader } from "../components/PanelHeader";
import { InspectorEvent, RuntimeInspectorPayload, desktopApi } from "../services/desktopApi";
import { useAppState } from "../store/appState";
import { useI18n } from "../store/i18n";

const emptyInspector: RuntimeInspectorPayload = {
  tool_calls: [],
  knowledge_hits: [],
  memory_hits: [],
  graph_hits: [],
  risk_level: null,
  execution_trace: [],
  current_session: null,
  current_workspace: null,
  current_model: null,
  current_provider: null
};

function InspectorList({ rows, kind, emptyLabel }: { rows: InspectorEvent[]; kind: string; emptyLabel: string }) {
  if (!rows.length) {
    return <p className="inspector-empty">{emptyLabel}</p>;
  }

  return (
    <div className={`inspector-list inspector-list-${kind}`}>
      {rows.map((row) => (
        <div className="inspector-row" key={`${row.time}-${row.label}-${row.detail}`}>
          <span>{row.label}</span>
          <strong>{row.status}</strong>
          <span>{row.detail}</span>
        </div>
      ))}
    </div>
  );
}

export function RightInspector() {
  const { inspectorCollapsed, setInspectorCollapsed } = useAppState();
  const { t } = useI18n();
  const [inspector, setInspector] = useState<RuntimeInspectorPayload>(emptyInspector);
  const loadInspector = async () => {
    setInspector(await desktopApi.getRuntimeInspector());
  };

  useEffect(() => {
    let mounted = true;
    const load = async () => {
      const payload = await desktopApi.getRuntimeInspector();
      if (mounted) {
        setInspector(payload);
      }
    };
    void load();
    const timer = window.setInterval(load, 4000);
    return () => {
      mounted = false;
      window.clearInterval(timer);
    };
  }, []);

  if (inspectorCollapsed) {
    return (
      <aside className="right-inspector collapsed">
        <Button
          variant="ghost"
          icon={<ChevronLeft size={16} />}
          title={t("inspector.title")}
          onClick={() => setInspectorCollapsed(false)}
        />
      </aside>
    );
  }

  return (
    <aside className="right-inspector">
      <PanelHeader
        title={t("inspector.title")}
        actions={
          <>
            <Button variant="ghost" icon={<RefreshCw size={14} />} onClick={loadInspector} title={t("actions.reload")} />
            <Button variant="ghost" icon={<ChevronRight size={15} />} onClick={() => setInspectorCollapsed(true)} />
          </>
        }
      />
      <div className="inspector-content">
        <section>
          <h3>{t("inspector.currentSession")}</h3>
          <div className="inspector-context-grid">
            <div><span>{t("inspector.session")}</span><strong>{inspector.current_session ?? t("runtime.offline")}</strong></div>
            <div><span>{t("status.workspace")}</span><strong title={inspector.current_workspace ?? ""}>{inspector.current_workspace ?? t("runtime.unknown")}</strong></div>
            <div><span>{t("status.model")}</span><strong>{inspector.current_model ?? t("runtime.unknown")}</strong></div>
            <div><span>{t("settings.provider")}</span><strong>{inspector.current_provider ?? t("runtime.unknown")}</strong></div>
          </div>
        </section>
        <section>
          <h3>{t("inspector.toolCalls")} <span>{inspector.tool_calls.length}</span></h3>
          <InspectorList rows={inspector.tool_calls} kind="tool" emptyLabel={t("inspector.empty")} />
        </section>
        <section>
          <h3>{t("inspector.knowledgeHits")} <span>{inspector.knowledge_hits.length}</span></h3>
          <InspectorList rows={inspector.knowledge_hits} kind="knowledge" emptyLabel={t("inspector.empty")} />
        </section>
        <section>
          <h3>{t("inspector.memoryHits")} <span>{inspector.memory_hits.length}</span></h3>
          <InspectorList rows={inspector.memory_hits} kind="memory" emptyLabel={t("inspector.empty")} />
        </section>
        <section>
          <h3>{t("inspector.graphHits")} <span>{inspector.graph_hits.length}</span></h3>
          <InspectorList rows={inspector.graph_hits} kind="graph" emptyLabel={t("inspector.empty")} />
        </section>
        <section>
          <h3>{t("inspector.riskLevel")}</h3>
          <div className="risk-panel">
            <div className="risk-title">
              <strong>{inspector.risk_level ?? t("inspector.empty")}</strong>
              <span>{t("runtime.agent")}</span>
            </div>
          </div>
        </section>
        <section>
          <h3>{t("inspector.executionTrace")} <span>{inspector.execution_trace.length}</span></h3>
          <InspectorList rows={inspector.execution_trace} kind="trace" emptyLabel={t("inspector.empty")} />
        </section>
      </div>
    </aside>
  );
}
