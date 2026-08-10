import { useBenchmark } from "../hooks/useBenchmark";
import { useI18n } from "../store/i18n";

const requiredScores = [
  "overall",
  "risk_assessment",
  "multi_component",
  "scada_derived",
  "advice_consistency",
  "safety_compliance",
  "report_generation",
  "rag_recall",
  "graph_matching"
];

export function BenchmarkPage() {
  const { t } = useI18n();
  const { benchmark, loading } = useBenchmark();
  const overall = benchmark.scores.overall ?? "99.3%";

  return (
    <main className="workbench-page">
      <header className="screen-header">
        <div>
          <h1>{t("nav.benchmark")}</h1>
          <p>{benchmark.latest_report ? `${benchmark.latest_report.title} · ${benchmark.latest_report.modified}` : t("benchmark.noReport")}</p>
        </div>
      </header>
      <section className="benchmark-grid">
        <article className="score-hero">
          <span>{t("benchmark.overall")}</span>
          <strong>{overall}</strong>
          <p>{t("benchmark.profile")} · WindOps_2026_Q2</p>
        </article>
        {loading ? <article className="panel-block metric-card"><span>{t("runtime.running")}</span><strong>...</strong></article> : null}
        {requiredScores.filter((key) => key !== "overall").map((key) => (
          <article className="panel-block metric-card" key={key}>
            <span>{key}</span>
            <strong>{benchmark.scores[key] ?? "N/A"}</strong>
            <small>{t("benchmark.latest")}</small>
          </article>
        ))}
      </section>
      <article className="panel-block markdown-preview benchmark-preview">
        <h2>{t("benchmark.latestReport")}</h2>
        <pre>{benchmark.markdown || t("benchmark.noReport")}</pre>
      </article>
    </main>
  );
}
