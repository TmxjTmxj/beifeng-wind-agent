import { useMemo, useState } from "react";
import { FileText, RefreshCw } from "lucide-react";
import { Button } from "../components/Button";
import { useMemory } from "../hooks/useMemory";
import { useAppState } from "../store/appState";
import { useI18n } from "../store/i18n";

export function MemoryPage() {
  const { t } = useI18n();
  const { setActiveSection } = useAppState();
  const { timeline, turbineIds, selectedTurbine, setSelectedTurbine, loading, refresh } = useMemory();
  const [faultType, setFaultType] = useState("all");
  const [riskLevel, setRiskLevel] = useState("all");

  const faultTypes = useMemo(() => {
    return ["all", ...Array.from(new Set(timeline.map((event) => event.title).filter(Boolean)))];
  }, [timeline]);
  const riskLevels = useMemo(() => {
    return ["all", ...Array.from(new Set(timeline.map((event) => event.risk_level).filter(Boolean) as string[]))];
  }, [timeline]);
  const filteredTimeline = timeline.filter((event) => {
    return (faultType === "all" || event.title === faultType) && (riskLevel === "all" || event.risk_level === riskLevel);
  });

  const grouped = {
    [t("memory.fault")]: filteredTimeline.filter((event) => event.item_type === "Fault"),
    [t("memory.maintenance")]: filteredTimeline.filter((event) => event.item_type === "Maintenance"),
    [t("memory.report")]: filteredTimeline.filter((event) => event.item_type === "Report")
  };

  return (
    <main className="workbench-page">
      <header className="screen-header">
        <div>
          <h1>{t("memory.title")}</h1>
          <p>{t("resource.memory.description")}</p>
        </div>
        <div className="header-actions">
          <Button icon={<RefreshCw size={15} />} onClick={refresh}>{t("actions.reload")}</Button>
          <label className="select-control">
            <span>{t("memory.filterTurbine")}</span>
            <select value={selectedTurbine} onChange={(event) => setSelectedTurbine(event.target.value)}>
              <option value="all">{t("memory.all")}</option>
              {turbineIds.map((id) => <option key={id} value={id}>{id}</option>)}
            </select>
          </label>
          <label className="select-control">
            <span>{t("memory.filterFault")}</span>
            <select value={faultType} onChange={(event) => setFaultType(event.target.value)}>
              {faultTypes.map((value) => <option key={value} value={value}>{value === "all" ? t("memory.all") : value}</option>)}
            </select>
          </label>
          <label className="select-control">
            <span>{t("memory.filterRisk")}</span>
            <select value={riskLevel} onChange={(event) => setRiskLevel(event.target.value)}>
              {riskLevels.map((value) => <option key={value} value={value}>{value === "all" ? t("memory.all") : value}</option>)}
            </select>
          </label>
        </div>
      </header>
      <section className="memory-grid">
        {loading ? <article className="panel-block"><h2>{t("runtime.running")}</h2><p className="empty-copy">{t("runtime.running")}</p></article> : null}
        {!loading && filteredTimeline.length === 0 ? <article className="panel-block"><h2>{t("memory.title")}</h2><p className="empty-copy">{t("inspector.empty")}</p></article> : null}
        {Object.entries(grouped).map(([title, events]) => (
          <article className="panel-block" key={title}>
            <h2>{title}</h2>
            <div className="memory-list">
              {events.map((event) => (
                <button className="memory-row memory-action-row" key={`${event.date}-${event.title}`} type="button" onClick={() => setActiveSection("reports")}>
                  <strong>{event.title}</strong>
                  <span>{event.date}</span>
                  <small>{event.risk_level ?? "-"}</small>
                </button>
              ))}
              {events.length === 0 ? <p className="empty-copy">{t("inspector.empty")}</p> : null}
            </div>
          </article>
        ))}
        <article className="panel-block timeline-panel">
          <h2>{t("memory.timeline")}</h2>
          <div className="timeline">
            {filteredTimeline.map((event) => (
              <div className="timeline-row" key={`${event.date}-${event.title}`}>
                <span />
                <div>
                  <strong>{event.title}</strong>
                  <p>{event.item_type} · {event.date} · {event.turbine_id ?? "-"} · {event.risk_level ?? "-"}</p>
                  <Button className="inline-row-button" icon={<FileText size={13} />} onClick={() => setActiveSection("reports")}>
                    {t("reports.title")}
                  </Button>
                </div>
              </div>
            ))}
          </div>
        </article>
      </section>
    </main>
  );
}
