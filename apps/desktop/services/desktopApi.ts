import { invoke } from "@tauri-apps/api/core";
import {
  benchmarkDimensions,
  memoryEvents,
  recentWorkspaces as mockRecentWorkspaces,
  reportMarkdown,
  reports as mockReports
} from "./mockData";

export type WorkspaceRecord = {
  path: string;
  name: string;
  last_opened: string;
  archived: boolean;
};

export type WorkspaceState = {
  current_workspace: string | null;
  recent_workspaces: WorkspaceRecord[];
};

export type SettingsPayload = {
  source: string;
  json: string;
  data: Record<string, unknown>;
};

export type ValidationResult = {
  valid: boolean;
  errors: string[];
};

export type CredentialTestResult = {
  ok: boolean;
  message: string;
  env_name: string;
  has_key: boolean;
  base_url: string;
  model: string;
};

export type ReportSummary = {
  path: string;
  title: string;
  file_name: string;
  modified: string;
  report_type: string;
};

export type ReportDetail = {
  summary: ReportSummary;
  markdown: string;
  generated_time: string;
  risk_level: string;
  source_documents: string[];
  confidence: string;
};

export type FileNode = {
  path: string;
  name: string;
  kind: "file" | "folder";
  depth: number;
  modified: string;
  size: number;
};

export type SkillSummary = {
  name: string;
  category: string;
  description: string;
  path: string;
  directory: string;
  examples_path?: string | null;
  updated: string;
  size: number;
  enabled: boolean;
};

export type MemoryTimelineItem = {
  date: string;
  item_type: string;
  title: string;
  turbine_id?: string | null;
  risk_level?: string | null;
};

export type MemoryPayload = {
  turbine_profiles: unknown;
  fault_history: unknown;
  maintenance_history: unknown;
  report_history: unknown;
  timeline: MemoryTimelineItem[];
};

export type BenchmarkReportSummary = {
  path: string;
  title: string;
  modified: string;
};

export type BenchmarkPayload = {
  latest_report: BenchmarkReportSummary | null;
  markdown: string;
  scores: Record<string, string>;
};

export type InspectorEvent = {
  label: string;
  detail: string;
  status: string;
  time: string;
};

export type AgentEvent = {
  event_type: string;
  timestamp: string;
  session_id: string;
  payload: Record<string, unknown>;
};

export type EventMetrics = {
  tool_duration_ms: number;
  tool_success_rate: number;
  knowledge_query_latency_ms: number;
  rag_latency_ms: number;
  memory_query_latency_ms: number;
  connector_latency_ms: number;
  total_events: number;
  error_events: number;
};

export type RuntimeInspectorPayload = {
  tool_calls: InspectorEvent[];
  knowledge_hits: InspectorEvent[];
  memory_hits: InspectorEvent[];
  graph_hits: InspectorEvent[];
  risk_level?: string | null;
  execution_trace: InspectorEvent[];
  current_session?: string | null;
  current_workspace?: string | null;
  current_model?: string | null;
  current_provider?: string | null;
};

export type RuntimeStatus = {
  agent: string;
  rag: string;
  agent_error?: string | null;
  rag_error?: string | null;
};

export type AgentRunPayload = {
  output: string;
  error?: string | null;
  inspector: RuntimeInspectorPayload;
  chat_path?: string | null;
  events: AgentEvent[];
};

export type RagHealth = {
  status: string;
  service_url: string;
  error?: string | null;
};

export type SystemMonitorPayload = {
  agent: string;
  rag: string;
  model: string;
  workspace: string;
  knowledge_docs: number;
  memory_records: number;
  graph_nodes: number;
  reports: number;
  benchmark: string;
  connectors: string;
};

export type HealthItem = {
  name: string;
  status: string;
  path: string;
  updated: string;
  error?: string | null;
  suggestion: string;
  response_time_ms: number;
  health_check_result: string;
};

export type ConsolePayload = {
  prompt: string;
  system_context: unknown;
  tool_calls: InspectorEvent[];
  raw_tool_results: string;
  memory_context: unknown;
  knowledge_context: string;
  graph_context: string;
  risk_assessment_json: unknown;
  report_path?: string | null;
  runtime_logs: string;
  event_timeline: AgentEvent[];
  raw_event_json: unknown;
  metrics: EventMetrics;
};

export type ChatSummary = {
  path: string;
  title: string;
  modified: string;
  archived: boolean;
  preview: string;
  session_id?: string | null;
  parent_session_id?: string | null;
  branch_name?: string | null;
};

export type ConversationBranch = {
  path: string;
  session_id: string;
  parent_session_id?: string | null;
  branch_name: string;
  title: string;
  modified: string;
  active: boolean;
  events: number;
};

export type BranchComparison = {
  left?: ConversationBranch | null;
  right?: ConversationBranch | null;
  event_delta: number;
  summary: string;
};

export type ArtifactSummary = {
  id: string;
  artifact_type: string;
  title: string;
  path: string;
  session_id?: string | null;
  modified: string;
  size: number;
};

export type WorkspaceCleanupIssue = {
  category: string;
  path: string;
  severity: string;
  detail: string;
  suggestion: string;
};

export type WorkspaceHealthReport = {
  workspace: string;
  generated_at: string;
  duplicate_files: number;
  legacy_folders: number;
  unused_caches: number;
  orphan_reports: number;
  old_benchmark_files: number;
  issues: WorkspaceCleanupIssue[];
  markdown: string;
};

const fallbackWorkspace = ".";

const fallbackSettings: SettingsPayload = {
  source: "fallback",
  data: {
    "workspace.root": fallbackWorkspace,
    "agent.name": "BeiFeng Wind O&M Agent",
    "agent.version": "1.0",
    "model.provider": "deepseek-compatible",
    "model.name": "Qwen3-Coder-Next",
    "model.base_url": "${DEEPSEEK_BASE_URL}",
    "model.api_key_env": "DEEPSEEK_API_KEY",
    "credentials.mode": "local-file",
    "credentials.file": "beifeng/config/secrets.json",
    "rag.service_url": "http://127.0.0.1:8787",
    "rag.db_path": "beifeng/data/wind.sqlite",
    "paths.knowledge_base": "beifeng/knowledge/knowledge_base",
    "paths.knowledge_graph": "beifeng/knowledge/knowledge_graph/wind_fault_graph.json",
    "paths.memory": "beifeng/memory",
    "paths.reports": "beifeng/reports/generated",
    "paths.skills": "beifeng/skills",
    "paths.connectors": "beifeng/connectors",
    "paths.evals": "beifeng/evals",
    "frontend.enabled": true,
    "frontend.path": "apps/desktop",
    "safety.require_human_confirmation": true,
    "safety.forbidden_actions": ["unauthorized remote shutdown", "unauthorized remote reset", "bypass safety interlocks"]
  },
  json: ""
};
fallbackSettings.json = JSON.stringify(fallbackSettings.data, null, 2);

const fallbackWorkspaceState: WorkspaceState = {
  current_workspace: fallbackWorkspace,
  recent_workspaces: mockRecentWorkspaces.map((workspace) => ({
    path: workspace.path,
    name: workspace.name,
    last_opened: "Mock",
    archived: workspace.tag === "Archive"
  }))
};

const fallbackReports: ReportSummary[] = mockReports.map((report, index) => ({
  path: `mock://${report.title}`,
  title: report.title.replace(".md", "").replace(/_/g, " "),
  file_name: report.title,
  modified: `${report.date} 09:4${index}:00`,
  report_type: report.type
}));

const fallbackReportDetail: ReportDetail = {
  summary: fallbackReports[0],
  markdown: reportMarkdown,
  generated_time: "Browser preview",
  risk_level: "N/A",
  source_documents: [],
  confidence: "N/A"
};

const fallbackFiles: FileNode[] = [
  { path: fallbackWorkspace, name: "BeiFeng-Agent", kind: "folder", depth: 0, modified: "Browser preview", size: 0 },
  { path: `${fallbackWorkspace}\\beifeng`, name: "beifeng", kind: "folder", depth: 1, modified: "Browser preview", size: 0 },
  { path: `${fallbackWorkspace}\\beifeng\\config\\settings.json`, name: "settings.json", kind: "file", depth: 2, modified: "Browser preview", size: 0 }
];

const fallbackSkills: SkillSummary[] = [
  {
    name: "Wind Fault Analysis",
    category: "wind_fault_analysis",
    description: "Open the desktop app to inspect local skill prompts from beifeng/skills.",
    path: `${fallbackWorkspace}\\beifeng\\skills\\wind_fault_analysis\\SKILL.md`,
    directory: `${fallbackWorkspace}\\beifeng\\skills\\wind_fault_analysis`,
    examples_path: null,
    updated: "Browser preview",
    size: 0,
    enabled: true
  }
];

const fallbackMemory: MemoryPayload = {
  turbine_profiles: [],
  fault_history: [],
  maintenance_history: [],
  report_history: [],
  timeline: memoryEvents.map((event) => ({
    date: event.date,
    item_type: event.type,
    title: event.title,
    turbine_id: event.title.match(/T\d+/)?.[0] ?? null,
    risk_level: event.risk
  }))
};

const fallbackBenchmark: BenchmarkPayload = {
  latest_report: {
    path: "mock://benchmark",
    title: "Mock Benchmark Report",
    modified: "Mock"
  },
  markdown: "# Benchmark Report\n\nMock benchmark report for browser preview.",
  scores: Object.fromEntries(
    benchmarkDimensions.map((dimension) => [
      dimension.label.toLowerCase().replace(/ /g, "_"),
      dimension.value
    ])
  )
};

const fallbackInspector: RuntimeInspectorPayload = {
  tool_calls: [],
  knowledge_hits: [],
  memory_hits: [],
  graph_hits: [],
  risk_level: null,
  execution_trace: [],
  current_session: null,
  current_workspace: fallbackWorkspace,
  current_model: "Qwen3-Coder-Next",
  current_provider: "deepseek-compatible"
};

const fallbackRuntimeStatus: RuntimeStatus = {
  agent: "Offline",
  rag: "Stopped",
  agent_error: null,
  rag_error: null
};

const fallbackMonitor: SystemMonitorPayload = {
  agent: "Offline",
  rag: "Stopped",
  model: "Qwen3-Coder-Next",
  workspace: fallbackWorkspace,
  knowledge_docs: 0,
  memory_records: 0,
  graph_nodes: 0,
  reports: 0,
  benchmark: "N/A",
  connectors: "0 configured"
};

const fallbackConsole: ConsolePayload = {
  prompt: "",
  system_context: fallbackSettings.data,
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

const fallbackChats: ChatSummary[] = [
  {
    path: "mock://chat",
    title: "Gearbox oil temperature anomaly",
    modified: "Browser preview",
    archived: false,
    preview: "Open the Tauri desktop app to load local chat history."
  }
];

const fallbackHealth: HealthItem[] = [
  {
    name: "Settings",
    status: "Preview",
    path: "beifeng/config/settings.json",
    updated: "Browser preview",
    error: null,
    suggestion: "Open the Tauri desktop app to inspect local files.",
    response_time_ms: 0,
    health_check_result: "Browser preview"
  },
  {
    name: "RAG Service",
    status: "Stopped",
    path: "http://127.0.0.1:8787",
    updated: "Browser preview",
    error: null,
    suggestion: "Use the desktop runtime command to start the local RAG service.",
    response_time_ms: 0,
    health_check_result: "Browser preview"
  },
  {
    name: "Agent Runtime",
    status: "Offline",
    path: "rust/target/debug/claw.exe",
    updated: "Browser preview",
    error: null,
    suggestion: "Use the desktop runtime command to start the Agent runtime.",
    response_time_ms: 0,
    health_check_result: "Browser preview"
  }
];

function isTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function call<T>(command: string, args: Record<string, unknown> | undefined, fallback: T): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    if (isTauriRuntime()) {
      throw error;
    }
    return fallback;
  }
}

async function callStrict<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(command, args);
}

export const desktopApi = {
  getCurrentWorkspace: () => call<WorkspaceState>("get_current_workspace", undefined, fallbackWorkspaceState),
  selectWorkspaceFolder: () => call<WorkspaceState>("select_workspace_folder", undefined, fallbackWorkspaceState),
  createWorkspace: (path: string) => callStrict<WorkspaceState>("create_workspace", { path }),
  importWorkspaceFolder: () => callStrict<WorkspaceState>("import_workspace_folder"),
  setCurrentWorkspace: (path: string) => call<WorkspaceState>("set_current_workspace", { path }, fallbackWorkspaceState),
  listRecentWorkspaces: () => call<WorkspaceRecord[]>("list_recent_workspaces", undefined, fallbackWorkspaceState.recent_workspaces),
  archiveWorkspace: (path: string) => call<WorkspaceState>("archive_workspace", { path }, fallbackWorkspaceState),
  removeWorkspaceFromList: (path: string) => callStrict<WorkspaceState>("remove_workspace_from_list", { path }),
  getLanguagePreference: () => call<string>("get_language_preference", undefined, "zh-CN"),
  setLanguagePreference: (language: string) => callStrict<string>("set_language_preference", { language }),
  revealInExplorer: (path?: string) => callStrict<void>("reveal_in_explorer", { path: path ?? null }),
  openInVSCode: (path?: string) => callStrict<void>("open_in_vscode", { path: path ?? null }),
  openFile: (path: string) => callStrict<void>("open_file", { path }),
  renamePath: (path: string, newName: string) => callStrict<string>("rename_path", { path, newName }),
  listWorkspaceFiles: (scope = "workspace") => call<FileNode[]>("list_workspace_files", { scope }, fallbackFiles),
  listSkills: () => call<SkillSummary[]>("list_skills", undefined, fallbackSkills),
  readSettingsJson: () => call<SettingsPayload>("read_settings_json", undefined, fallbackSettings),
  writeSettingsJson: (jsonText: string) => call<SettingsPayload>("write_settings_json", { jsonText }, fallbackSettings),
  resetSettingsJson: () => callStrict<SettingsPayload>("reset_settings_json"),
  validateSettingsJson: (jsonText: string) =>
    call<ValidationResult>("validate_settings_json", { jsonText }, { valid: true, errors: [] }),
  openSettingsJsonInVSCode: () => callStrict<void>("open_settings_json_in_vscode"),
  readSecretsJson: () => call<SettingsPayload>("read_secrets_json", undefined, {
    source: "fallback",
    json: JSON.stringify({
      DEEPSEEK_API_KEY: "",
      DEEPSEEK_BASE_URL: "",
      ANTHROPIC_API_KEY: "",
      OPENAI_API_KEY: ""
    }, null, 2),
    data: {
      DEEPSEEK_API_KEY: "",
      DEEPSEEK_BASE_URL: "",
      ANTHROPIC_API_KEY: "",
      OPENAI_API_KEY: ""
    }
  }),
  writeSecretsJson: (jsonText: string) => callStrict<SettingsPayload>("write_secrets_json", { jsonText }),
  openSecretsJsonInVSCode: () => callStrict<void>("open_secrets_json_in_vscode"),
  saveModelCredential: (provider: string, model: string, baseUrl: string, apiKeyEnv: string, apiKey: string) =>
    callStrict<SettingsPayload>("save_model_credential", { provider, model, baseUrl, apiKeyEnv, apiKey }),
  clearModelCredential: (apiKeyEnv: string) => callStrict<SettingsPayload>("clear_model_credential", { apiKeyEnv }),
  testModelCredentials: () =>
    call<CredentialTestResult>("test_model_credentials", undefined, {
      ok: false,
      message: "Desktop credential command is not available.",
      env_name: "DEEPSEEK_API_KEY",
      has_key: false,
      base_url: "",
      model: ""
    }),
  listReports: () => call<ReportSummary[]>("list_reports", undefined, fallbackReports),
  readReport: (path: string) => call<string>("read_report", { path }, reportMarkdown),
  readReportDetail: (path: string) => call<ReportDetail>("read_report_detail", { path }, fallbackReportDetail),
  exportReportMarkdown: (path: string) => callStrict<string>("export_report_markdown", { path }),
  revealReport: (path: string) => callStrict<void>("reveal_report", { path }),
  openReportInVSCode: (path: string) => callStrict<void>("open_report_in_vscode", { path }),
  readMemoryPayload: () => call<MemoryPayload>("read_memory_payload", undefined, fallbackMemory),
  listBenchmarkReports: () => call<BenchmarkReportSummary[]>("list_benchmark_reports", undefined, []),
  readLatestBenchmarkReport: () => call<BenchmarkPayload>("read_latest_benchmark_report", undefined, fallbackBenchmark),
  startAgentSession: () => call<RuntimeStatus>("start_agent_session", undefined, fallbackRuntimeStatus),
  stopAgentSession: () => call<RuntimeStatus>("stop_agent_session", undefined, fallbackRuntimeStatus),
  sendAgentPrompt: (prompt: string) =>
    call<AgentRunPayload>("send_agent_prompt", { prompt }, {
      output: "",
      error: "Agent runtime is not available in browser preview.",
      inspector: fallbackInspector,
      chat_path: null,
      events: []
    }),
  streamAgentOutput: () => call<string>("stream_agent_output", undefined, ""),
  listAgentEvents: (sessionId?: string) => call<AgentEvent[]>("list_agent_events", { sessionIdFilter: sessionId ?? null }, []),
  listChats: () => call<ChatSummary[]>("list_chats", undefined, fallbackChats),
  loadChatHistory: (path: string) => call<Record<string, unknown>>("load_chat_history", { path }, {}),
  saveChat: (prompt: string, output: string) => call<string>("save_chat", { prompt, output }, ""),
  archiveChat: (path: string) => call<ChatSummary[]>("archive_chat", { path }, fallbackChats),
  deleteChat: (path: string) => call<ChatSummary[]>("delete_chat", { path }, fallbackChats),
  listConversationBranches: (currentPath?: string) =>
    call<ConversationBranch[]>("list_conversation_branches", { currentPath: currentPath ?? null }, []),
  forkChatSession: (path: string) => callStrict<string>("fork_chat_session", { path }),
  restoreChatBranch: (path: string) => call<ChatSummary[]>("restore_chat_branch", { path }, fallbackChats),
  compareChatBranches: (leftPath: string, rightPath: string) =>
    call<BranchComparison>("compare_chat_branches", { leftPath, rightPath }, { event_delta: 0, summary: "Browser preview" }),
  listArtifacts: (sessionId?: string) => call<ArtifactSummary[]>("list_artifacts", { sessionIdFilter: sessionId ?? null }, []),
  openArtifact: (path: string) => callStrict<void>("open_artifact", { path }),
  revealArtifact: (path: string) => callStrict<void>("reveal_artifact", { path }),
  exportArtifact: (path: string) => callStrict<string>("export_artifact", { path }),
  deleteArtifact: (path: string) => call<ArtifactSummary[]>("delete_artifact", { path }, []),
  analyzeWorkspaceCleanup: () =>
    call<WorkspaceHealthReport>("analyze_workspace_cleanup", undefined, {
      workspace: fallbackWorkspace,
      generated_at: "Browser preview",
      duplicate_files: 0,
      legacy_folders: 0,
      unused_caches: 0,
      orphan_reports: 0,
      old_benchmark_files: 0,
      issues: [],
      markdown: "# Workspace Health Report\n\nBrowser preview."
    }),
  getAgentStatus: () => call<RuntimeStatus>("get_agent_status", undefined, fallbackRuntimeStatus),
  startRagService: () => callStrict<RuntimeStatus>("start_rag_service"),
  stopRagService: () => callStrict<RuntimeStatus>("stop_rag_service"),
  restartRagService: () => callStrict<RuntimeStatus>("restart_rag_service"),
  getRagHealth: () =>
    call<RagHealth>("get_rag_health", undefined, {
      status: "Stopped",
      service_url: "http://127.0.0.1:8787",
      error: null
    }),
  getRuntimeInspector: () => call<RuntimeInspectorPayload>("get_runtime_inspector", undefined, fallbackInspector),
  getSystemMonitor: () => call<SystemMonitorPayload>("get_system_monitor", undefined, fallbackMonitor),
  getHealthDashboard: () => call<HealthItem[]>("get_health_dashboard", undefined, fallbackHealth),
  getAgentConsole: () => call<ConsolePayload>("get_agent_console", undefined, fallbackConsole),
  saveDebugLog: (content: string) => call<string>("save_debug_log", { content }, "")
};
