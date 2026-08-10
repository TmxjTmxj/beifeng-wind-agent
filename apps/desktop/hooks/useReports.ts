import { useEffect, useMemo, useState } from "react";
import { desktopApi, ReportSummary } from "../services/desktopApi";
import { useAppState } from "../store/appState";

export function useReports() {
  const { workspaceState } = useAppState();
  const [reports, setReports] = useState<ReportSummary[]>([]);
  const [selectedReport, setSelectedReport] = useState<ReportSummary | null>(null);
  const [markdown, setMarkdown] = useState("");
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(true);

  const refresh = async () => {
    setLoading(true);
    const nextReports = await desktopApi.listReports();
    setReports(nextReports);
    const nextSelected = nextReports[0] ?? null;
    setSelectedReport(nextSelected);
    setMarkdown(nextSelected ? await desktopApi.readReport(nextSelected.path) : "");
    setLoading(false);
  };

  const selectReport = async (report: ReportSummary) => {
    setSelectedReport(report);
    setMarkdown(await desktopApi.readReport(report.path));
  };

  useEffect(() => {
    refresh();
  }, [workspaceState.current_workspace]);

  const filteredReports = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    if (!normalized) return reports;
    return reports.filter((report) =>
      [report.title, report.file_name, report.report_type, report.modified].some((value) =>
        value.toLowerCase().includes(normalized)
      )
    );
  }, [query, reports]);

  return {
    reports: filteredReports,
    selectedReport,
    markdown,
    query,
    setQuery,
    loading,
    refresh,
    selectReport,
    revealReport: desktopApi.revealReport,
    openReportInVSCode: desktopApi.openReportInVSCode
  };
}
