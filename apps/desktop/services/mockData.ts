import type { Section } from "../store/appState";

export const workspacePath = "D:\\BeiFeng_Agent\\Workspaces\\BeiFeng_WindFarm_01\\Turbine_T08";

export const recentWorkspaces = [
  { name: "BeiFeng_WindFarm_01", path: "D:\\BeiFeng_Agent\\Workspaces\\BeiFeng_WindFarm_01", tag: "Production", health: "Healthy" },
  { name: "Offshore_North_Trial", path: "D:\\BeiFeng_Agent\\Workspaces\\Offshore_North_Trial", tag: "Benchmark", health: "Review" },
  { name: "Gearbox_RootCause_Lab", path: "D:\\BeiFeng_Agent\\Workspaces\\Gearbox_RootCause_Lab", tag: "Archive", health: "Idle" }
];

export const workspaceTree = [
  { depth: 0, label: "BeiFeng_WindFarm_01", kind: "folder" },
  { depth: 1, label: "Turbine_T08", kind: "folder" },
  { depth: 2, label: "01_Data", kind: "folder" },
  { depth: 2, label: "02_Analysis", kind: "folder" },
  { depth: 2, label: "03_Reports", kind: "folder" },
  { depth: 2, label: "settings.json", kind: "file" },
  { depth: 1, label: "Turbine_T12", kind: "folder" },
  { depth: 1, label: "_Shared", kind: "folder" },
  { depth: 1, label: "_Archive", kind: "folder" }
];

export const chats = [
  { title: "Gearbox oil temp anomaly - T08", time: "09:42", active: true },
  { title: "Vibration increase - T12", time: "Yesterday", active: false },
  { title: "Pitch system fault - T23", time: "Yesterday", active: false },
  { title: "Monthly reliability review", time: "May 8", active: false },
  { title: "SCADA data gap analysis", time: "May 7", active: false }
];

export const reports = [
  { title: "gearbox_inspection_report.md", type: "Inspection", risk: "High", date: "2026-06-06" },
  { title: "oil_sample_lab_report.md", type: "Maintenance", risk: "Medium", date: "2026-06-05" },
  { title: "blade_icing_safety.md", type: "Risk", risk: "Critical", date: "2026-06-04" },
  { title: "monthly_reliability_review.md", type: "Summary", risk: "Low", date: "2026-06-03" }
];

export const reportMarkdown = `# Gearbox Oil Temperature Anomaly

## Summary

Turbine T08 shows sustained gearbox oil temperature above the configured threshold. Current risk level is High because the anomaly persists for more than 7 hours and appears under higher power output.

## Evidence

- SCADA trend: oil temperature 98.6 C, threshold 85 C.
- Knowledge hit: Gearbox Oil Temperature Troubleshooting Guide.
- Memory hit: Oil cooler fan replaced in 2024-11.
- Graph hit: Gearbox > Lubrication System > Oil Cooler.

## Recommendation

Schedule a field inspection, verify lubrication system cooling path, review alarm history, and require human confirmation for any safety-sensitive operation.
`;

export const memoryEvents = [
  { type: "Fault", title: "T08 oil cooler fan replaced", date: "2024-11-03", risk: "Medium" },
  { type: "Maintenance", title: "High temp alarm resolved", date: "2024-07-21", risk: "Medium" },
  { type: "Report", title: "Commissioning summary", date: "2021-06-18", risk: "Low" },
  { type: "Fault", title: "T12 vibration increase", date: "2026-05-28", risk: "High" },
  { type: "Maintenance", title: "Gearbox lubrication inspection", date: "2026-05-12", risk: "Medium" }
];

export const toolCalls = [
  ["query_scada", "success", "09:41:12"],
  ["get_condition_data", "success", "09:41:15"],
  ["get_alarm_events", "success", "09:41:18"],
  ["get_maintenance_history", "success", "09:41:22"],
  ["similar_case_search", "success", "09:41:27"],
  ["compute_trend", "success", "09:41:29"],
  ["anomaly_score", "success", "09:41:31"]
];

export const knowledgeHits = [
  ["Gearbox Oil Temperature Anomalies - Causes and Actions", "0.92"],
  ["High Oil Temperature Troubleshooting Guide", "0.89"],
  ["BF-3.6MW Gearbox Design and Limits", "0.86"],
  ["Lubrication System Best Practices", "0.84"],
  ["Environmental Effects in Gearbox Performance", "0.78"]
];

export const memoryHits = [
  ["T08 - 2024-11-03 - Oil cooler fan replaced", "0.91"],
  ["T08 - 2024-07-21 - High temp alarm resolved", "0.87"],
  ["T08 - Commissioning summary", "0.82"]
];

export const graphHits = [
  ["Component: Gearbox", "0.93"],
  ["System: Lubrication System", "0.90"],
  ["Part: Oil Cooler", "0.88"],
  ["Sensor: GB_OilTemp_Out", "0.86"]
];

export const executionTrace = [
  ["09:41:12", "Planner", "Create investigation plan", "342ms"],
  ["09:41:12", "Tool", "query_scada", "1.2s"],
  ["09:41:15", "Tool", "get_condition_data", "2.1s"],
  ["09:41:18", "Tool", "get_alarm_events", "1.1s"],
  ["09:41:22", "Tool", "get_maintenance_history", "1.6s"]
];

export const benchmarkDimensions = [
  { label: "Component inference", value: "100.0%", delta: "+0.0" },
  { label: "Graph matching", value: "100.0%", delta: "+0.0" },
  { label: "RAG recall", value: "99.5%", delta: "+0.3" },
  { label: "Risk assessment", value: "98.9%", delta: "+1.1" },
  { label: "Report generation", value: "99.2%", delta: "+0.8" }
];

export const connectors = [
  { name: "SCADA", status: "Reserved", health: "Schema ready" },
  { name: "CMMS", status: "Reserved", health: "Schema ready" },
  { name: "Weather", status: "Reserved", health: "Schema ready" },
  { name: "UAV", status: "Future", health: "Not implemented" }
];

export const skills = [
  "wind_fault_analysis",
  "scada_analysis",
  "report_generation",
  "gearbox_diagnosis",
  "blade_inspection"
];

export const sectionDescriptions: Record<Section, string> = {
  home: "Startup page, workspace status, and common actions.",
  workspace: "Workspace-first operating surface for project context.",
  chats: "Agent task execution and conversation tree.",
  files: "Workspace, knowledge, memory, reports, and config browser.",
  memory: "Fault, maintenance, report, and turbine memory.",
  reports: "Generated report list, search, preview, and export.",
  benchmark: "Current score, history trend, and regression checks.",
  skills: "Installed BeiFeng industrial skills.",
  connectors: "SCADA, CMMS, Weather, and future connector readiness.",
  system: "Runtime health, local files, services, and repair suggestions.",
  console: "Developer mode view for prompts, contexts, raw results, and logs.",
  settings: "GUI settings and settings.json editor."
};
