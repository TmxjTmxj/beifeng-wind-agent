import { useEffect, useMemo, useState } from "react";
import { Clipboard, Download, ExternalLink, FileCode, RefreshCw, Search } from "lucide-react";
import { Button } from "../components/Button";
import { useReports } from "../hooks/useReports";
import { ReportDetail, desktopApi } from "../services/desktopApi";
import { useI18n } from "../store/i18n";

export function ReportsPage() {
  const { t } = useI18n();
  const { reports, selectedReport, markdown, query, setQuery, loading, refresh, selectReport, revealReport, openReportInVSCode } = useReports();
  const [typeFilter, setTypeFilter] = useState("all");
  const [detail, setDetail] = useState<ReportDetail | null>(null);
  const [message, setMessage] = useState("");

  const reportTypes = useMemo(() => ["all", ...Array.from(new Set(reports.map((report) => report.report_type)))], [reports]);
  const visibleReports = reports.filter((report) => typeFilter === "all" || report.report_type === typeFilter);

  useEffect(() => {
    if (!selectedReport) {
      setDetail(null);
      return;
    }
    void desktopApi.readReportDetail(selectedReport.path).then(setDetail);
  }, [selectedReport]);

  const run = async (operation: () => Promise<unknown>, success = t("message.ready")) => {
    try {
      await operation();
      setMessage(success);
    } catch (error) {
      setMessage(`${t("message.error")}: ${String(error)}`);
    }
  };

  const copyMarkdown = async () => {
    await navigator.clipboard?.writeText(markdown);
    setMessage(t("message.copied"));
  };

  const exportMarkdown = async () => {
    if (!selectedReport) return;
    try {
      const path = await desktopApi.exportReportMarkdown(selectedReport.path);
      setMessage(path ? `${t("message.exported")}: ${path}` : t("message.ready"));
    } catch (error) {
      setMessage(`${t("message.error")}: ${String(error)}`);
    }
  };

  return (
    <main className="workbench-page reports-page">
      <header className="screen-header">
        <div>
          <h1>{t("reports.title")}</h1>
          <p>{t("resource.reports.description")}</p>
        </div>
        <div className="header-actions">
          <Button icon={<RefreshCw size={15} />} onClick={refresh}>{t("actions.reload")}</Button>
          <Button icon={<FileCode size={15} />} disabled={!selectedReport} onClick={() => selectedReport && run(() => openReportInVSCode(selectedReport.path))}>
            {t("actions.openVSCode")}
          </Button>
          <Button icon={<ExternalLink size={15} />} disabled={!selectedReport} onClick={() => selectedReport && run(() => revealReport(selectedReport.path))}>
            {t("actions.reveal")}
          </Button>
          <Button icon={<Clipboard size={15} />} disabled={!selectedReport} onClick={copyMarkdown}>{t("actions.copyMarkdown")}</Button>
          <Button icon={<Download size={15} />} disabled={!selectedReport} onClick={exportMarkdown}>{t("actions.export")}</Button>
        </div>
      </header>
      {message ? <div className={`settings-message ${message.startsWith(t("message.error")) ? "is-error" : ""}`}>{message}</div> : null}
      <section className="report-experience-grid">
        <aside className="panel-block list-pane">
          <h2>{t("reports.list")}</h2>
          <label className="search-box">
            <Search size={15} />
            <input aria-label={t("actions.search")} placeholder={t("actions.search")} value={query} onChange={(event) => setQuery(event.target.value)} />
          </label>
          <label className="select-control report-filter">
            {t("actions.filter")}
            <select value={typeFilter} onChange={(event) => setTypeFilter(event.target.value)}>
              {reportTypes.map((type) => <option value={type} key={type}>{type === "all" ? t("memory.all") : type}</option>)}
            </select>
          </label>
          <div className="report-list">
            {loading ? <p className="empty-copy">{t("runtime.running")}</p> : null}
            {!loading && visibleReports.length === 0 ? <p className="empty-copy">{t("inspector.empty")}</p> : null}
            {visibleReports.map((report) => (
              <button className={`report-item ${selectedReport?.path === report.path ? "is-active" : ""}`} type="button" key={report.path} onClick={() => selectReport(report)}>
                <strong>{report.title}</strong>
                <span>{report.report_type} · {report.modified}</span>
              </button>
            ))}
          </div>
        </aside>
        <article className="panel-block markdown-preview">
          <h2>{t("reports.preview")}</h2>
          <div className="markdown-toolbar"><Search size={14} /> {selectedReport?.file_name ?? t("reports.preview")}</div>
          <pre>{markdown || t("inspector.empty")}</pre>
        </article>
        <aside className="panel-block report-metadata">
          <h2>{t("reports.metadata")}</h2>
          {detail ? (
            <div className="metadata-list">
              <span>{t("reports.generatedTime")}</span><strong>{detail.generated_time}</strong>
              <span>{t("reports.riskLevel")}</span><strong>{detail.risk_level}</strong>
              <span>{t("reports.confidence")}</span><strong>{detail.confidence}</strong>
              <span>{t("reports.sourceDocuments")}</span><strong>{detail.source_documents.join(", ") || "N/A"}</strong>
              <span>{t("system.path")}</span><strong>{detail.summary.path}</strong>
            </div>
          ) : (
            <p className="empty-copy">{t("inspector.empty")}</p>
          )}
        </aside>
      </section>
    </main>
  );
}
