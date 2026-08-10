use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    env,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Mutex,
    thread,
    time::{Duration, SystemTime},
};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceRecord {
    path: String,
    name: String,
    last_opened: String,
    archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct WorkspaceStore {
    current_workspace: Option<String>,
    recent_workspaces: Vec<WorkspaceRecord>,
}

#[derive(Debug, Clone, Serialize)]
struct WorkspaceState {
    current_workspace: Option<String>,
    recent_workspaces: Vec<WorkspaceRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppPreferences {
    language: String,
}

#[derive(Debug, Serialize)]
struct SettingsPayload {
    source: String,
    json: String,
    data: Value,
}

#[derive(Debug, Serialize)]
struct CredentialTestResult {
    ok: bool,
    message: String,
    env_name: String,
    has_key: bool,
    base_url: String,
    model: String,
}

#[derive(Debug, Serialize)]
struct ValidationResult {
    valid: bool,
    errors: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ReportSummary {
    path: String,
    title: String,
    file_name: String,
    modified: String,
    report_type: String,
}

#[derive(Debug, Serialize)]
struct ReportDetail {
    summary: ReportSummary,
    markdown: String,
    generated_time: String,
    risk_level: String,
    source_documents: Vec<String>,
    confidence: String,
}

#[derive(Debug, Serialize)]
struct FileNode {
    path: String,
    name: String,
    kind: String,
    depth: usize,
    modified: String,
    size: u64,
}

#[derive(Debug, Serialize)]
struct SkillSummary {
    name: String,
    category: String,
    description: String,
    path: String,
    directory: String,
    examples_path: Option<String>,
    updated: String,
    size: u64,
    enabled: bool,
}

#[derive(Debug, Serialize)]
struct MemoryPayload {
    turbine_profiles: Value,
    fault_history: Value,
    maintenance_history: Value,
    report_history: Value,
    timeline: Vec<MemoryTimelineItem>,
}

#[derive(Debug, Serialize)]
struct MemoryTimelineItem {
    date: String,
    item_type: String,
    title: String,
    turbine_id: Option<String>,
    risk_level: Option<String>,
}

#[derive(Debug, Serialize)]
struct BenchmarkReportSummary {
    path: String,
    title: String,
    modified: String,
}

#[derive(Debug, Serialize)]
struct BenchmarkPayload {
    latest_report: Option<BenchmarkReportSummary>,
    markdown: String,
    scores: BTreeMap<String, String>,
}

#[derive(Default)]
struct RuntimeState {
    agent: Mutex<ProcessSlot>,
    rag: Mutex<ProcessSlot>,
    last_agent_output: Mutex<String>,
    last_inspector: Mutex<RuntimeInspectorPayload>,
    events: Mutex<Vec<AgentEvent>>,
}

#[derive(Default)]
struct ProcessSlot {
    child: Option<Child>,
    status: String,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct RuntimeInspectorPayload {
    tool_calls: Vec<InspectorEvent>,
    knowledge_hits: Vec<InspectorEvent>,
    memory_hits: Vec<InspectorEvent>,
    graph_hits: Vec<InspectorEvent>,
    risk_level: Option<String>,
    execution_trace: Vec<InspectorEvent>,
    current_session: Option<String>,
    current_workspace: Option<String>,
    current_model: Option<String>,
    current_provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InspectorEvent {
    label: String,
    detail: String,
    status: String,
    time: String,
}

#[derive(Debug, Serialize)]
struct RuntimeStatus {
    agent: String,
    rag: String,
    agent_error: Option<String>,
    rag_error: Option<String>,
}

#[derive(Debug, Serialize)]
struct AgentRunPayload {
    output: String,
    error: Option<String>,
    inspector: RuntimeInspectorPayload,
    chat_path: Option<String>,
    events: Vec<AgentEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentEvent {
    event_type: String,
    timestamp: String,
    session_id: String,
    payload: Value,
}

#[derive(Debug, Clone, Serialize, Default)]
struct EventMetrics {
    tool_duration_ms: u64,
    tool_success_rate: f64,
    knowledge_query_latency_ms: u64,
    rag_latency_ms: u64,
    memory_query_latency_ms: u64,
    connector_latency_ms: u64,
    total_events: usize,
    error_events: usize,
}

#[derive(Debug, Serialize)]
struct RagHealth {
    status: String,
    service_url: String,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct SystemMonitorPayload {
    agent: String,
    rag: String,
    model: String,
    workspace: String,
    knowledge_docs: usize,
    memory_records: usize,
    graph_nodes: usize,
    reports: usize,
    benchmark: String,
    connectors: String,
}

#[derive(Debug, Serialize)]
struct HealthItem {
    name: String,
    status: String,
    path: String,
    updated: String,
    error: Option<String>,
    suggestion: String,
    response_time_ms: u64,
    health_check_result: String,
}

#[derive(Debug, Serialize)]
struct ConsolePayload {
    prompt: String,
    system_context: Value,
    tool_calls: Vec<InspectorEvent>,
    raw_tool_results: String,
    memory_context: Value,
    knowledge_context: String,
    graph_context: String,
    risk_assessment_json: Value,
    report_path: Option<String>,
    runtime_logs: String,
    event_timeline: Vec<AgentEvent>,
    raw_event_json: Value,
    metrics: EventMetrics,
}

#[derive(Debug, Serialize)]
struct ChatSummary {
    path: String,
    title: String,
    modified: String,
    archived: bool,
    preview: String,
    session_id: Option<String>,
    parent_session_id: Option<String>,
    branch_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct ConversationBranch {
    path: String,
    session_id: String,
    parent_session_id: Option<String>,
    branch_name: String,
    title: String,
    modified: String,
    active: bool,
    events: usize,
}

#[derive(Debug, Serialize)]
struct BranchComparison {
    left: Option<ConversationBranch>,
    right: Option<ConversationBranch>,
    event_delta: isize,
    summary: String,
}

#[derive(Debug, Serialize)]
struct ArtifactSummary {
    id: String,
    artifact_type: String,
    title: String,
    path: String,
    session_id: Option<String>,
    modified: String,
    size: u64,
}

#[derive(Debug, Serialize)]
struct WorkspaceCleanupIssue {
    category: String,
    path: String,
    severity: String,
    detail: String,
    suggestion: String,
}

#[derive(Debug, Serialize)]
struct WorkspaceHealthReport {
    workspace: String,
    generated_at: String,
    duplicate_files: usize,
    legacy_folders: usize,
    unused_caches: usize,
    orphan_reports: usize,
    old_benchmark_files: usize,
    issues: Vec<WorkspaceCleanupIssue>,
    markdown: String,
}

fn default_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join(".."))
}

fn app_data_dir() -> Result<PathBuf, String> {
    let dir = dirs::data_local_dir()
        .ok_or_else(|| "Unable to resolve local app data directory".to_string())?
        .join("BeiFeng Agent Desktop");
    fs::create_dir_all(&dir).map_err(|err| format!("Unable to create {}: {}", dir.display(), err))?;
    Ok(dir)
}

fn workspace_store_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("workspaces.json"))
}

fn preferences_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("preferences.json"))
}

fn app_secrets_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("secrets.json"))
}

fn default_credentials_file() -> String {
    app_secrets_path()
        .unwrap_or_else(|_| PathBuf::from("beifeng/config/secrets.json"))
        .to_string_lossy()
        .to_string()
}

fn now_string() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn path_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Workspace")
        .to_string()
}

fn load_workspace_store() -> Result<WorkspaceStore, String> {
    let path = workspace_store_path()?;
    if !path.exists() {
        let default_root = default_workspace_root();
        let store = WorkspaceStore {
            current_workspace: Some(default_root.to_string_lossy().to_string()),
            recent_workspaces: vec![WorkspaceRecord {
                name: path_name(&default_root),
                path: default_root.to_string_lossy().to_string(),
                last_opened: now_string(),
                archived: false,
            }],
        };
        save_workspace_store(&store)?;
        return Ok(store);
    }

    let raw = fs::read_to_string(&path).map_err(|err| format!("Unable to read {}: {}", path.display(), err))?;
    serde_json::from_str(&raw).map_err(|err| format!("Unable to parse {}: {}", path.display(), err))
}

fn save_workspace_store(store: &WorkspaceStore) -> Result<(), String> {
    let path = workspace_store_path()?;
    let raw = serde_json::to_string_pretty(store).map_err(|err| err.to_string())?;
    fs::write(&path, raw).map_err(|err| format!("Unable to write {}: {}", path.display(), err))
}

fn load_preferences() -> AppPreferences {
    let path = match preferences_path() {
        Ok(path) => path,
        Err(_) => {
            return AppPreferences {
                language: "zh-CN".to_string(),
            }
        }
    };
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<AppPreferences>(&raw).ok())
        .unwrap_or(AppPreferences {
            language: "zh-CN".to_string(),
        })
}

fn save_preferences(preferences: &AppPreferences) -> Result<(), String> {
    let path = preferences_path()?;
    let raw = serde_json::to_string_pretty(preferences).map_err(|err| err.to_string())?;
    fs::write(&path, raw).map_err(|err| format!("Unable to write {}: {}", path.display(), err))
}

fn add_recent_workspace(store: &mut WorkspaceStore, workspace_path: &Path) {
    let path_string = workspace_path.to_string_lossy().to_string();
    store.recent_workspaces.retain(|item| item.path != path_string);
    store.recent_workspaces.insert(
        0,
        WorkspaceRecord {
            name: path_name(workspace_path),
            path: path_string.clone(),
            last_opened: now_string(),
            archived: false,
        },
    );
    store.recent_workspaces.truncate(12);
    store.current_workspace = Some(path_string);
}

fn valid_workspace_name(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| !name.trim().is_empty())
        .unwrap_or(false)
}

fn current_workspace_path() -> Result<PathBuf, String> {
    let store = load_workspace_store()?;
    let path = store
        .current_workspace
        .map(PathBuf::from)
        .unwrap_or_else(default_workspace_root);
    Ok(path)
}

fn settings_path(workspace: &Path) -> PathBuf {
    workspace.join("beifeng").join("config").join("settings.json")
}

fn secrets_path(workspace: &Path, settings: &Value) -> PathBuf {
    setting_path(workspace, settings, "credentials.file", &default_credentials_file())
}

fn reports_dir(workspace: &Path) -> PathBuf {
    workspace.join("beifeng").join("reports").join("generated")
}

fn memory_dir(workspace: &Path) -> PathBuf {
    workspace.join("beifeng").join("memory")
}

fn skills_dir(workspace: &Path, settings: &Value) -> PathBuf {
    setting_path(workspace, settings, "paths.skills", "beifeng/skills")
}

fn evals_dir(workspace: &Path) -> PathBuf {
    workspace.join("beifeng").join("evals")
}

fn logs_dir(workspace: &Path) -> PathBuf {
    workspace.join("beifeng").join("logs")
}

fn chats_dir(workspace: &Path) -> PathBuf {
    workspace.join("beifeng").join("workspaces").join(workspace_id(workspace)).join("chats")
}

fn workspace_id(workspace: &Path) -> String {
    path_name(workspace)
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

fn rust_root(workspace: &Path) -> PathBuf {
    workspace.join("rust")
}

fn claw_exe(workspace: &Path) -> PathBuf {
    rust_root(workspace).join("target").join("debug").join("claw.exe")
}

fn rag_exe(workspace: &Path) -> PathBuf {
    let release = rust_root(workspace)
        .join("target")
        .join("release")
        .join("claw-rag-service.exe");
    if release.exists() {
        release
    } else {
        rust_root(workspace).join("target").join("debug").join("claw-rag-service.exe")
    }
}

fn load_settings_value(workspace: &Path) -> Value {
    let path = settings_path(workspace);
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .unwrap_or_else(|| safe_settings_template(workspace))
}

fn safe_secrets_template() -> Value {
    json!({
        "DEEPSEEK_API_KEY": "",
        "DEEPSEEK_BASE_URL": "",
        "ANTHROPIC_API_KEY": "",
        "OPENAI_API_KEY": ""
    })
}

fn load_secrets_value(workspace: &Path, settings: &Value) -> Value {
    let path = secrets_path(workspace, settings);
    if let Some(value) = fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
    {
        return value;
    }

    let legacy_path = workspace.join("beifeng").join("config").join("secrets.json");
    fs::read_to_string(legacy_path)
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .unwrap_or_else(safe_secrets_template)
}

fn ensure_secrets_file(workspace: &Path, settings: &Value) -> Result<PathBuf, String> {
    let path = secrets_path(workspace, settings);
    if !path.exists() {
        let legacy_path = workspace.join("beifeng").join("config").join("secrets.json");
        if legacy_path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|err| format!("Unable to create {}: {}", parent.display(), err))?;
            }
            fs::copy(&legacy_path, &path)
                .map_err(|err| format!("Unable to migrate secrets from {} to {}: {}", legacy_path.display(), path.display(), err))?;
            return Ok(path);
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| format!("Unable to create {}: {}", parent.display(), err))?;
        }
        fs::write(
            &path,
            serde_json::to_string_pretty(&safe_secrets_template()).map_err(|err| err.to_string())?,
        )
        .map_err(|err| format!("Unable to write {}: {}", path.display(), err))?;
    }
    Ok(path)
}

fn setting_string(settings: &Value, key: &str, fallback: &str) -> String {
    settings
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn value_string(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn setting_path(workspace: &Path, settings: &Value, key: &str, fallback: &str) -> PathBuf {
    let configured = setting_string(settings, key, fallback);
    let path = PathBuf::from(configured);
    if path.is_absolute() {
        path
    } else {
        workspace.join(path)
    }
}

fn apply_runtime_env(command: &mut Command, workspace: &Path, settings: &Value) {
    let secrets = load_secrets_value(workspace, settings);
    let rag_url = setting_string(settings, "rag.service_url", "http://127.0.0.1:8787");
    let memory = setting_path(workspace, settings, "paths.memory", "beifeng/memory");
    command.env("CLAW_RAG_SERVICE_URL", rag_url);
    command.env("CLAW_RAG_MEMORY_DIR", memory);

    let base_url = setting_string(settings, "model.base_url", "");
    if base_url.starts_with("${") && base_url.ends_with('}') {
        let env_name = base_url.trim_start_matches("${").trim_end_matches('}');
        let secret_value = value_string(&secrets, env_name);
        if !secret_value.is_empty() {
            command.env(env_name, secret_value);
        } else if let Ok(value) = env::var(env_name) {
            command.env(env_name, value);
        }
    } else if !base_url.is_empty() && !base_url.contains('*') {
        command.env("DEEPSEEK_BASE_URL", base_url);
    }

    let api_key_env = setting_string(settings, "model.api_key_env", "");
    if !api_key_env.is_empty() && !api_key_env.starts_with("sk-") {
        let secret_value = value_string(&secrets, &api_key_env);
        if !secret_value.is_empty() {
            command.env(&api_key_env, secret_value);
        } else if let Ok(value) = env::var(&api_key_env) {
            command.env(api_key_env, value);
        }
    }
}

fn runtime_api_key(settings: &Value, secrets: &Value) -> (String, String) {
    let env_name = setting_string(settings, "model.api_key_env", "DEEPSEEK_API_KEY");
    let value = if env_name.starts_with("sk-") {
        String::new()
    } else {
        let from_secret = value_string(secrets, &env_name);
        if from_secret.is_empty() {
            env::var(&env_name).unwrap_or_default()
        } else {
            from_secret
        }
    };
    (env_name, value)
}

fn count_files_with_extensions(dir: &Path, extensions: &[&str]) -> usize {
    if !dir.exists() {
        return 0;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .map(|path| {
            if path.is_dir() {
                count_files_with_extensions(&path, extensions)
            } else if path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| extensions.iter().any(|allowed| ext.eq_ignore_ascii_case(allowed)))
                .unwrap_or(false)
            {
                1
            } else {
                0
            }
        })
        .sum()
}

fn count_json_records(value: &Value) -> usize {
    match value {
        Value::Array(items) => items.len(),
        Value::Object(map) => map
            .values()
            .map(|value| match value {
                Value::Array(items) => items.len(),
                Value::Object(_) => 1,
                _ => 0,
            })
            .sum(),
        _ => 0,
    }
}

fn safe_settings_template(workspace: &Path) -> Value {
    json!({
        "workspace.root": workspace.to_string_lossy(),
        "agent.name": "BeiFeng Wind O&M Agent",
        "agent.version": "1.0",
        "model.provider": "deepseek-compatible",
        "model.name": "Qwen3-Coder-Next",
        "model.base_url": "${DEEPSEEK_BASE_URL}",
        "model.api_key_env": "DEEPSEEK_API_KEY",
        "credentials.mode": "local-file",
        "credentials.file": default_credentials_file(),
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
        "safety.forbidden_actions": [
            "unauthorized remote shutdown",
            "unauthorized remote reset",
            "bypass safety interlocks"
        ]
    })
}

fn secret_like(key: &str, value: &Value) -> bool {
    let key_lower = key.to_ascii_lowercase();
    let Some(text) = value.as_str() else {
        return false;
    };
    if key_lower.contains("api_key_env") {
        return text.starts_with("sk-");
    }
    key_lower.ends_with("api_key")
        || key_lower.contains(".api_key")
        || key_lower.contains("secret")
        || key_lower.contains("token")
        || text.starts_with("sk-")
}

fn mask_secret_text(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    if !text.starts_with("sk-") {
        return "********".to_string();
    }
    let suffix: String = text
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("sk-************{}", suffix)
}

fn sanitize_text_with_secrets(text: &str, secrets: &Value) -> String {
    let mut sanitized = text.to_string();
    if let Value::Object(map) = secrets {
        for value in map.values().filter_map(Value::as_str).filter(|value| !value.is_empty()) {
            sanitized = sanitized.replace(value, &mask_secret_text(value));
        }
    }
    sanitized
}

fn redact_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let redacted = map
                .iter()
                .map(|(key, value)| {
                    if secret_like(key, value) {
                        let masked = value.as_str().map(mask_secret_text).unwrap_or_else(|| "********".to_string());
                        (key.clone(), Value::String(masked))
                    } else {
                        (key.clone(), redact_json(value))
                    }
                })
                .collect();
            Value::Object(redacted)
        }
        Value::Array(items) => Value::Array(items.iter().map(redact_json).collect()),
        _ => value.clone(),
    }
}

fn validate_settings_value(value: &Value) -> ValidationResult {
    let required = [
        "workspace.root",
        "agent.name",
        "agent.version",
        "model.provider",
        "model.name",
        "model.base_url",
        "model.api_key_env",
        "credentials.mode",
        "credentials.file",
        "rag.service_url",
        "rag.db_path",
        "paths.knowledge_base",
        "paths.knowledge_graph",
        "paths.memory",
        "paths.reports",
        "paths.skills",
        "paths.connectors",
        "paths.evals",
        "frontend.enabled",
        "frontend.path",
        "safety.require_human_confirmation",
        "safety.forbidden_actions",
    ];

    let mut errors = Vec::new();
    let Some(object) = value.as_object() else {
        return ValidationResult {
            valid: false,
            errors: vec!["settings.json must be a JSON object".to_string()],
        };
    };

    for key in required {
        if !object.contains_key(key) {
            errors.push(format!("Missing required setting: {}", key));
        }
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
    }
}

fn ensure_settings_defaults(workspace: &Path, value: &mut Value) -> bool {
    let Some(map) = value.as_object_mut() else {
        return false;
    };
    let mut changed = false;
    let defaults = safe_settings_template(workspace);
    for key in [
        "credentials.mode",
        "credentials.file",
        "model.provider",
        "model.name",
        "model.base_url",
        "model.api_key_env",
    ] {
        if !map.contains_key(key) {
            if let Some(default_value) = defaults.get(key) {
                map.insert(key.to_string(), default_value.clone());
                changed = true;
            }
        }
    }
    changed
}

fn file_modified_string(path: &Path) -> String {
    let modified = fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .unwrap_or_else(|_| SystemTime::now());
    let dt: DateTime<Local> = modified.into();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn read_json_file(path: &Path) -> Value {
    if !path.exists() {
        return Value::Array(vec![]);
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .unwrap_or(Value::Array(vec![]))
}

fn value_text<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(|item| item.as_str()))
}

fn memory_items(value: &Value, item_type: &str) -> Vec<MemoryTimelineItem> {
    let rows: Vec<Value> = match value {
        Value::Array(items) => items.clone(),
        Value::Object(map) => map
            .values()
            .flat_map(|item| match item {
                Value::Array(items) => items.clone(),
                other => vec![other.clone()],
            })
            .collect(),
        _ => vec![],
    };

    rows.into_iter()
        .filter_map(|row| {
            let title = value_text(
                &row,
                &["title", "fault", "fault_type", "description", "summary", "report_title", "maintenance_action"],
            )
            .unwrap_or(item_type)
            .to_string();
            let date = value_text(&row, &["date", "timestamp", "time", "created_at", "inspection_date"])
                .unwrap_or("Unknown date")
                .to_string();
            Some(MemoryTimelineItem {
                date,
                item_type: item_type.to_string(),
                title,
                turbine_id: value_text(&row, &["turbine_id", "turbine", "asset_id"]).map(str::to_string),
                risk_level: value_text(&row, &["risk_level", "risk", "severity"]).map(str::to_string),
            })
        })
        .collect()
}

fn report_type_from(path: &Path, content: &str) -> String {
    let lower = format!(
        "{} {}",
        path.file_name().and_then(|name| name.to_str()).unwrap_or(""),
        content.lines().take(8).collect::<Vec<_>>().join(" ")
    )
    .to_ascii_lowercase();

    if lower.contains("risk") {
        "Risk".to_string()
    } else if lower.contains("inspection") || lower.contains("巡检") {
        "Inspection".to_string()
    } else if lower.contains("maintenance") || lower.contains("维护") {
        "Maintenance".to_string()
    } else if lower.contains("benchmark") {
        "Benchmark".to_string()
    } else {
        "Report".to_string()
    }
}

fn report_summary(path: &Path, content: &str) -> ReportSummary {
    ReportSummary {
        title: title_from_markdown(path, content),
        report_type: report_type_from(path, content),
        modified: file_modified_string(path),
        file_name: path.file_name().and_then(|name| name.to_str()).unwrap_or("").to_string(),
        path: path.to_string_lossy().to_string(),
    }
}

fn value_after_label(line: &str, labels: &[&str]) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    labels.iter().find_map(|label| {
        let label_lower = label.to_ascii_lowercase();
        if !lower.contains(&label_lower) {
            return None;
        }
        line.split_once(':')
            .or_else(|| line.split_once('：'))
            .map(|(_, value)| value.trim().trim_matches('*').to_string())
            .filter(|value| !value.is_empty())
    })
}

fn report_detail_from(path: &Path, markdown: String) -> ReportDetail {
    let summary = report_summary(path, &markdown);
    let mut generated_time = summary.modified.clone();
    let mut risk_level = "N/A".to_string();
    let mut confidence = "N/A".to_string();
    let mut source_documents = Vec::new();

    for line in markdown.lines().take(160) {
        if let Some(value) = value_after_label(line, &["generated time", "generated_at", "time", "生成时间"]) {
            generated_time = value;
        }
        if let Some(value) = value_after_label(line, &["risk level", "risk", "风险等级"]) {
            risk_level = value;
        }
        if let Some(value) = value_after_label(line, &["confidence", "置信度"]) {
            confidence = value;
        }
        if let Some(value) = value_after_label(line, &["source document", "source documents", "sources", "来源文档"]) {
            source_documents.push(value);
        }
    }

    ReportDetail {
        summary,
        markdown,
        generated_time,
        risk_level,
        source_documents,
        confidence,
    }
}

fn title_from_markdown(path: &Path, content: &str) -> String {
    content
        .lines()
        .find_map(|line| line.strip_prefix("# ").map(str::trim))
        .filter(|title| !title.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("Untitled report")
                .replace('_', " ")
        })
}

fn ensure_existing_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    Ok(())
}

fn collect_file_nodes(root: &Path, depth: usize, rows: &mut Vec<FileNode>, max_depth: usize) -> Result<(), String> {
    if depth > max_depth || !root.exists() {
        return Ok(());
    }
    let mut entries = fs::read_dir(root)
        .map_err(|err| format!("Unable to read {}: {}", root.display(), err))?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| {
        let path = entry.path();
        (
            if path.is_dir() { 0 } else { 1 },
            path.file_name().and_then(|name| name.to_str()).unwrap_or("").to_ascii_lowercase(),
        )
    });

    for entry in entries {
        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        rows.push(FileNode {
            name: path.file_name().and_then(|name| name.to_str()).unwrap_or("").to_string(),
            kind: if metadata.is_dir() { "folder".to_string() } else { "file".to_string() },
            modified: file_modified_string(&path),
            size: if metadata.is_file() { metadata.len() } else { 0 },
            depth,
            path: path.to_string_lossy().to_string(),
        });
        if metadata.is_dir() {
            collect_file_nodes(&path, depth + 1, rows, max_depth)?;
        }
    }
    Ok(())
}

fn parse_score_line(line: &str) -> Option<(String, String)> {
    let lowered = line.to_ascii_lowercase();
    let keys = [
        "overall",
        "risk_assessment",
        "multi_component",
        "scada_derived",
        "advice_consistency",
        "safety_compliance",
        "report_generation",
        "rag_recall",
        "graph_matching",
    ];
    let key = keys.iter().find(|key| lowered.contains(**key))?;
    let score = line
        .split_whitespace()
        .find(|part| part.contains('%'))
        .map(|part| part.trim_matches(|c: char| !c.is_ascii_digit() && c != '.' && c != '%').to_string())?;
    Some((key.to_string(), score))
}

#[tauri::command]
fn select_workspace_folder() -> Result<WorkspaceState, String> {
    let Some(path) = rfd::FileDialog::new().pick_folder() else {
        return get_current_workspace();
    };
    set_current_workspace(path.to_string_lossy().to_string())
}

#[tauri::command]
fn create_workspace(path: String) -> Result<WorkspaceState, String> {
    let workspace_path = PathBuf::from(path);
    if !valid_workspace_name(&workspace_path) {
        return Err("Workspace path must end with a folder name".to_string());
    }
    fs::create_dir_all(&workspace_path)
        .map_err(|err| format!("Unable to create workspace {}: {}", workspace_path.display(), err))?;
    set_current_workspace(workspace_path.to_string_lossy().to_string())
}

#[tauri::command]
fn import_workspace_folder() -> Result<WorkspaceState, String> {
    select_workspace_folder()
}

#[tauri::command]
fn get_current_workspace() -> Result<WorkspaceState, String> {
    let store = load_workspace_store()?;
    Ok(WorkspaceState {
        current_workspace: store.current_workspace,
        recent_workspaces: store.recent_workspaces,
    })
}

#[tauri::command]
fn set_current_workspace(path: String) -> Result<WorkspaceState, String> {
    let workspace_path = PathBuf::from(path);
    if !workspace_path.exists() || !workspace_path.is_dir() {
        return Err(format!("Workspace folder does not exist: {}", workspace_path.display()));
    }
    let mut store = load_workspace_store()?;
    add_recent_workspace(&mut store, &workspace_path);
    save_workspace_store(&store)?;
    Ok(WorkspaceState {
        current_workspace: store.current_workspace,
        recent_workspaces: store.recent_workspaces,
    })
}

#[tauri::command]
fn remove_workspace_from_list(path: String) -> Result<WorkspaceState, String> {
    let mut store = load_workspace_store()?;
    store.recent_workspaces.retain(|item| item.path != path);
    if store.current_workspace.as_deref() == Some(path.as_str()) {
        store.current_workspace = store
            .recent_workspaces
            .iter()
            .find(|item| !item.archived)
            .map(|item| item.path.clone());
    }
    save_workspace_store(&store)?;
    Ok(WorkspaceState {
        current_workspace: store.current_workspace,
        recent_workspaces: store.recent_workspaces,
    })
}

#[tauri::command]
fn get_language_preference() -> Result<String, String> {
    Ok(load_preferences().language)
}

#[tauri::command]
fn set_language_preference(language: String) -> Result<String, String> {
    if language != "zh-CN" && language != "en-US" {
        return Err(format!("Unsupported language: {}", language));
    }
    save_preferences(&AppPreferences {
        language: language.clone(),
    })?;
    Ok(language)
}

#[tauri::command]
fn list_recent_workspaces() -> Result<Vec<WorkspaceRecord>, String> {
    Ok(load_workspace_store()?.recent_workspaces)
}

#[tauri::command]
fn archive_workspace(path: String) -> Result<WorkspaceState, String> {
    let mut store = load_workspace_store()?;
    for item in &mut store.recent_workspaces {
        if item.path == path {
            item.archived = true;
        }
    }
    if store.current_workspace.as_deref() == Some(path.as_str()) {
        store.current_workspace = store
            .recent_workspaces
            .iter()
            .find(|item| !item.archived)
            .map(|item| item.path.clone());
    }
    save_workspace_store(&store)?;
    Ok(WorkspaceState {
        current_workspace: store.current_workspace,
        recent_workspaces: store.recent_workspaces,
    })
}

#[tauri::command]
fn reveal_in_explorer(path: Option<String>) -> Result<(), String> {
    let target = path.map(PathBuf::from).unwrap_or(current_workspace_path()?);
    ensure_existing_path(&target)?;
    Command::new("explorer")
        .arg(&target)
        .spawn()
        .map_err(|err| format!("Unable to open Windows Explorer for {}: {}", target.display(), err))?;
    Ok(())
}

#[tauri::command]
fn open_in_vscode(path: Option<String>) -> Result<(), String> {
    let target = path.map(PathBuf::from).unwrap_or(current_workspace_path()?);
    ensure_existing_path(&target)?;
    Command::new("code")
        .arg(&target)
        .spawn()
        .map_err(|err| format!("Unable to run `code {}`. Install VSCode CLI or add it to PATH. Error: {}", target.display(), err))?;
    Ok(())
}

#[tauri::command]
fn open_file(path: String) -> Result<(), String> {
    let target = PathBuf::from(path);
    ensure_existing_path(&target)?;
    Command::new("explorer")
        .arg(&target)
        .spawn()
        .map_err(|err| format!("Unable to open {}: {}", target.display(), err))?;
    Ok(())
}

#[tauri::command]
fn rename_path(path: String, new_name: String) -> Result<String, String> {
    let target = PathBuf::from(path);
    ensure_existing_path(&target)?;
    let clean_name = new_name.trim();
    if clean_name.is_empty() || clean_name.contains('\\') || clean_name.contains('/') {
        return Err("New name must be a file or folder name, not a path".to_string());
    }
    let parent = target.parent().ok_or_else(|| "Cannot rename path without a parent directory".to_string())?;
    let next = parent.join(clean_name);
    if next.exists() {
        return Err(format!("Target already exists: {}", next.display()));
    }
    fs::rename(&target, &next).map_err(|err| format!("Unable to rename {}: {}", target.display(), err))?;
    Ok(next.to_string_lossy().to_string())
}

#[tauri::command]
fn list_workspace_files(scope: String) -> Result<Vec<FileNode>, String> {
    let workspace = current_workspace_path()?;
    let root = match scope.as_str() {
        "knowledge" => workspace.join("beifeng").join("knowledge"),
        "memory" => memory_dir(&workspace),
        "reports" => reports_dir(&workspace),
        _ => workspace,
    };
    let mut rows = Vec::new();
    collect_file_nodes(&root, 0, &mut rows, 4)?;
    Ok(rows)
}

fn frontmatter_value(raw: &str, key: &str) -> Option<String> {
    let mut lines = raw.lines();
    if lines.next()? != "---" {
        return None;
    }
    for line in lines {
        if line == "---" {
            break;
        }
        let Some((left, right)) = line.split_once(':') else {
            continue;
        };
        if left.trim() == key {
            return Some(right.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn first_markdown_heading(raw: &str) -> Option<String> {
    raw.lines()
        .find_map(|line| line.strip_prefix("# ").map(str::trim))
        .filter(|line| !line.is_empty())
        .map(str::to_string)
}

fn section_text(raw: &str, heading: &str) -> Option<String> {
    let mut in_section = false;
    let mut lines = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            if in_section {
                break;
            }
            in_section = trimmed.trim_start_matches("## ").trim() == heading;
            continue;
        }
        if in_section && !trimmed.is_empty() && !trimmed.starts_with("```") {
            lines.push(trimmed.to_string());
        }
    }
    (!lines.is_empty()).then(|| lines.join(" "))
}

fn fallback_description(raw: &str) -> String {
    raw.lines()
        .map(str::trim)
        .find(|line| {
            !line.is_empty()
                && !line.starts_with('#')
                && !line.starts_with('-')
                && !line.starts_with("```")
        })
        .unwrap_or("Local skill prompt")
        .chars()
        .take(180)
        .collect()
}

fn skill_summary_from_file(path: PathBuf) -> Result<SkillSummary, String> {
    let raw = fs::read_to_string(&path).map_err(|err| format!("Unable to read {}: {}", path.display(), err))?;
    let directory = path
        .parent()
        .ok_or_else(|| format!("Skill path has no parent: {}", path.display()))?
        .to_path_buf();
    let category = directory
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("skill")
        .to_string();
    let examples = directory.join("examples.md");
    let metadata = fs::metadata(&path).map_err(|err| format!("Unable to stat {}: {}", path.display(), err))?;
    let name = frontmatter_value(&raw, "name")
        .or_else(|| first_markdown_heading(&raw))
        .unwrap_or_else(|| category.clone());
    let description = frontmatter_value(&raw, "description")
        .or_else(|| section_text(&raw, "适用场景"))
        .unwrap_or_else(|| fallback_description(&raw));
    Ok(SkillSummary {
        name,
        category,
        description,
        path: path.to_string_lossy().to_string(),
        directory: directory.to_string_lossy().to_string(),
        examples_path: examples.exists().then(|| examples.to_string_lossy().to_string()),
        updated: file_modified_string(&path),
        size: metadata.len(),
        enabled: true,
    })
}

#[tauri::command]
fn list_skills() -> Result<Vec<SkillSummary>, String> {
    let workspace = current_workspace_path()?;
    let settings = load_settings_value(&workspace);
    let root = skills_dir(&workspace, &settings);
    if !root.exists() {
        return Ok(vec![]);
    }
    let mut rows = Vec::new();
    for entry in fs::read_dir(&root).map_err(|err| format!("Unable to read {}: {}", root.display(), err))? {
        let entry = entry.map_err(|err| err.to_string())?;
        let skill_file = entry.path().join("SKILL.md");
        if skill_file.is_file() {
            rows.push(skill_summary_from_file(skill_file)?);
        }
    }
    rows.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(rows)
}

#[tauri::command]
fn read_settings_json() -> Result<SettingsPayload, String> {
    let workspace = current_workspace_path()?;
    let path = settings_path(&workspace);
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| format!("Unable to create {}: {}", parent.display(), err))?;
        }
        let template = safe_settings_template(&workspace);
        fs::write(
            &path,
            serde_json::to_string_pretty(&template).map_err(|err| err.to_string())?,
        )
        .map_err(|err| format!("Unable to write {}: {}", path.display(), err))?;
    }

    let raw = fs::read_to_string(&path).map_err(|err| format!("Unable to read {}: {}", path.display(), err))?;
    let mut value: Value = serde_json::from_str(&raw).map_err(|err| format!("Invalid settings JSON: {}", err))?;
    if ensure_settings_defaults(&workspace, &mut value) {
        fs::write(
            &path,
            serde_json::to_string_pretty(&value).map_err(|err| err.to_string())?,
        )
        .map_err(|err| format!("Unable to update {}: {}", path.display(), err))?;
    }
    let redacted = redact_json(&value);
    Ok(SettingsPayload {
        source: path.to_string_lossy().to_string(),
        json: serde_json::to_string_pretty(&redacted).map_err(|err| err.to_string())?,
        data: redacted,
    })
}

#[tauri::command]
fn write_settings_json(json_text: String) -> Result<SettingsPayload, String> {
    let value: Value = serde_json::from_str(&json_text).map_err(|err| format!("Invalid JSON: {}", err))?;
    let validation = validate_settings_value(&value);
    if !validation.valid {
        return Err(validation.errors.join("\n"));
    }

    let workspace = current_workspace_path()?;
    let path = settings_path(&workspace);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("Unable to create {}: {}", parent.display(), err))?;
    }
    let formatted = serde_json::to_string_pretty(&value).map_err(|err| err.to_string())?;
    fs::write(&path, formatted).map_err(|err| format!("Unable to write {}: {}", path.display(), err))?;
    read_settings_json()
}

#[tauri::command]
fn reset_settings_json() -> Result<SettingsPayload, String> {
    let workspace = current_workspace_path()?;
    let path = settings_path(&workspace);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("Unable to create {}: {}", parent.display(), err))?;
    }
    let template = safe_settings_template(&workspace);
    fs::write(
        &path,
        serde_json::to_string_pretty(&template).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("Unable to write {}: {}", path.display(), err))?;
    read_settings_json()
}

#[tauri::command]
fn validate_settings_json(json_text: String) -> Result<ValidationResult, String> {
    let value: Value = match serde_json::from_str(&json_text) {
        Ok(value) => value,
        Err(err) => {
            return Ok(ValidationResult {
                valid: false,
                errors: vec![format!("Invalid JSON: {}", err)],
            })
        }
    };
    Ok(validate_settings_value(&value))
}

#[tauri::command]
fn open_settings_json_in_vscode() -> Result<(), String> {
    let workspace = current_workspace_path()?;
    let path = settings_path(&workspace);
    if !path.exists() {
        let _ = read_settings_json()?;
    }
    open_in_vscode(Some(path.to_string_lossy().to_string()))
}

#[tauri::command]
fn read_secrets_json() -> Result<SettingsPayload, String> {
    let workspace = current_workspace_path()?;
    let settings = load_settings_value(&workspace);
    let path = ensure_secrets_file(&workspace, &settings)?;
    let raw = fs::read_to_string(&path).map_err(|err| format!("Unable to read {}: {}", path.display(), err))?;
    let value: Value = serde_json::from_str(&raw).map_err(|err| format!("Invalid secrets JSON: {}", err))?;
    let redacted = redact_json(&value);
    Ok(SettingsPayload {
        source: path.to_string_lossy().to_string(),
        json: serde_json::to_string_pretty(&redacted).map_err(|err| err.to_string())?,
        data: redacted,
    })
}

#[tauri::command]
fn write_secrets_json(json_text: String) -> Result<SettingsPayload, String> {
    let value: Value = serde_json::from_str(&json_text).map_err(|err| format!("Invalid JSON: {}", err))?;
    let workspace = current_workspace_path()?;
    let settings = load_settings_value(&workspace);
    let path = ensure_secrets_file(&workspace, &settings)?;
    fs::write(
        &path,
        serde_json::to_string_pretty(&value).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("Unable to write {}: {}", path.display(), err))?;
    read_secrets_json()
}

#[tauri::command]
fn open_secrets_json_in_vscode() -> Result<(), String> {
    let workspace = current_workspace_path()?;
    let settings = load_settings_value(&workspace);
    let path = ensure_secrets_file(&workspace, &settings)?;
    open_in_vscode(Some(path.to_string_lossy().to_string()))
}

#[tauri::command]
fn save_model_credential(
    provider: String,
    model: String,
    base_url: String,
    api_key_env: String,
    api_key: String,
) -> Result<SettingsPayload, String> {
    let workspace = current_workspace_path()?;
    let settings_path = settings_path(&workspace);
    let mut settings = load_settings_value(&workspace);
    let Some(settings_map) = settings.as_object_mut() else {
        return Err("settings.json must be a JSON object".to_string());
    };
    settings_map.insert("model.provider".to_string(), Value::String(provider));
    settings_map.insert("model.name".to_string(), Value::String(model));
    settings_map.insert("model.base_url".to_string(), Value::String(base_url.clone()));
    settings_map.insert("model.api_key_env".to_string(), Value::String(api_key_env.clone()));
    settings_map.insert("credentials.mode".to_string(), Value::String("local-file".to_string()));
    settings_map.insert("credentials.file".to_string(), Value::String(default_credentials_file()));

    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("Unable to create {}: {}", parent.display(), err))?;
    }
    fs::write(
        &settings_path,
        serde_json::to_string_pretty(&settings).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("Unable to write {}: {}", settings_path.display(), err))?;

    let secrets_path = ensure_secrets_file(&workspace, &settings)?;
    let mut secrets = load_secrets_value(&workspace, &settings);
    let Some(secrets_map) = secrets.as_object_mut() else {
        return Err("secrets.json must be a JSON object".to_string());
    };
    if !api_key.trim().is_empty() && !api_key.contains('*') {
        secrets_map.insert(api_key_env, Value::String(api_key));
    }
    if !base_url.trim().is_empty() && !base_url.starts_with("${") {
        secrets_map.insert("DEEPSEEK_BASE_URL".to_string(), Value::String(base_url));
    }
    fs::write(
        &secrets_path,
        serde_json::to_string_pretty(&secrets).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("Unable to write {}: {}", secrets_path.display(), err))?;

    read_settings_json()
}

#[tauri::command]
fn clear_model_credential(api_key_env: String) -> Result<SettingsPayload, String> {
    let workspace = current_workspace_path()?;
    let settings = load_settings_value(&workspace);
    let path = ensure_secrets_file(&workspace, &settings)?;
    let mut secrets = load_secrets_value(&workspace, &settings);
    if let Some(map) = secrets.as_object_mut() {
        map.insert(api_key_env, Value::String(String::new()));
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(&secrets).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("Unable to write {}: {}", path.display(), err))?;
    read_secrets_json()
}

#[tauri::command]
fn test_model_credentials() -> Result<CredentialTestResult, String> {
    let workspace = current_workspace_path()?;
    let settings = load_settings_value(&workspace);
    let secrets = load_secrets_value(&workspace, &settings);
    let (env_name, key) = runtime_api_key(&settings, &secrets);
    let model = setting_string(&settings, "model.name", "Qwen3-Coder-Next");
    let base_url = setting_string(&settings, "model.base_url", "");
    let has_key = !key.is_empty();
    let ok = has_key && !model.trim().is_empty();
    Ok(CredentialTestResult {
        ok,
        message: if ok {
            "Model credential is configured locally.".to_string()
        } else {
            "Missing API Key. Configure it in Settings > Model Credentials.".to_string()
        },
        env_name,
        has_key,
        base_url,
        model,
    })
}

#[tauri::command]
fn list_reports() -> Result<Vec<ReportSummary>, String> {
    let dir = reports_dir(&current_workspace_path()?);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut reports = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|err| format!("Unable to read {}: {}", dir.display(), err))? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let content = fs::read_to_string(&path).unwrap_or_default();
        reports.push(report_summary(&path, &content));
    }
    reports.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(reports)
}

#[tauri::command]
fn read_report(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|err| format!("Unable to read {}: {}", path, err))
}

#[tauri::command]
fn read_report_detail(path: String) -> Result<ReportDetail, String> {
    let report_path = PathBuf::from(&path);
    ensure_existing_path(&report_path)?;
    let markdown = fs::read_to_string(&report_path).map_err(|err| format!("Unable to read {}: {}", path, err))?;
    Ok(report_detail_from(&report_path, markdown))
}

#[tauri::command]
fn export_report_markdown(path: String) -> Result<String, String> {
    let report_path = PathBuf::from(&path);
    ensure_existing_path(&report_path)?;
    let Some(save_path) = rfd::FileDialog::new()
        .set_file_name(
            report_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("report.md"),
        )
        .add_filter("Markdown", &["md"])
        .save_file()
    else {
        return Ok(String::new());
    };
    fs::copy(&report_path, &save_path)
        .map_err(|err| format!("Unable to export {} to {}: {}", report_path.display(), save_path.display(), err))?;
    Ok(save_path.to_string_lossy().to_string())
}

#[tauri::command]
fn reveal_report(path: String) -> Result<(), String> {
    reveal_in_explorer(Some(path))
}

#[tauri::command]
fn open_report_in_vscode(path: String) -> Result<(), String> {
    open_in_vscode(Some(path))
}

#[tauri::command]
fn read_turbine_profiles() -> Result<Value, String> {
    Ok(read_json_file(&memory_dir(&current_workspace_path()?).join("turbine_profiles.json")))
}

#[tauri::command]
fn read_fault_history() -> Result<Value, String> {
    Ok(read_json_file(&memory_dir(&current_workspace_path()?).join("fault_history.json")))
}

#[tauri::command]
fn read_maintenance_history() -> Result<Value, String> {
    Ok(read_json_file(&memory_dir(&current_workspace_path()?).join("maintenance_history.json")))
}

#[tauri::command]
fn read_report_history() -> Result<Value, String> {
    Ok(read_json_file(&memory_dir(&current_workspace_path()?).join("report_history.json")))
}

#[tauri::command]
fn read_memory_payload() -> Result<MemoryPayload, String> {
    let turbine_profiles = read_turbine_profiles()?;
    let fault_history = read_fault_history()?;
    let maintenance_history = read_maintenance_history()?;
    let report_history = read_report_history()?;
    let mut timeline = Vec::new();
    timeline.extend(memory_items(&fault_history, "Fault"));
    timeline.extend(memory_items(&maintenance_history, "Maintenance"));
    timeline.extend(memory_items(&report_history, "Report"));
    timeline.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(MemoryPayload {
        turbine_profiles,
        fault_history,
        maintenance_history,
        report_history,
        timeline,
    })
}

#[tauri::command]
fn list_benchmark_reports() -> Result<Vec<BenchmarkReportSummary>, String> {
    let dir = evals_dir(&current_workspace_path()?);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut reports = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|err| format!("Unable to read {}: {}", dir.display(), err))? {
        let path = entry.map_err(|err| err.to_string())?.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.starts_with("benchmark_report_") || path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let content = fs::read_to_string(&path).unwrap_or_default();
        reports.push(BenchmarkReportSummary {
            title: title_from_markdown(&path, &content),
            modified: file_modified_string(&path),
            path: path.to_string_lossy().to_string(),
        });
    }
    reports.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(reports)
}

#[tauri::command]
fn read_latest_benchmark_report() -> Result<BenchmarkPayload, String> {
    let reports = list_benchmark_reports()?;
    let Some(latest) = reports.first() else {
        return Ok(BenchmarkPayload {
            latest_report: None,
            markdown: String::new(),
            scores: BTreeMap::new(),
        });
    };
    let markdown = fs::read_to_string(&latest.path).map_err(|err| format!("Unable to read {}: {}", latest.path, err))?;
    let scores = parse_key_scores_from_markdown(&markdown)?;
    Ok(BenchmarkPayload {
        latest_report: Some(BenchmarkReportSummary {
            path: latest.path.clone(),
            title: latest.title.clone(),
            modified: latest.modified.clone(),
        }),
        markdown,
        scores,
    })
}

#[tauri::command]
fn parse_key_scores(markdown: String) -> Result<BTreeMap<String, String>, String> {
    parse_key_scores_from_markdown(&markdown)
}

fn parse_key_scores_from_markdown(markdown: &str) -> Result<BTreeMap<String, String>, String> {
    let mut scores = BTreeMap::new();
    for line in markdown.lines() {
        if let Some((key, score)) = parse_score_line(line) {
            scores.insert(key, score);
        }
    }
    Ok(scores)
}

fn session_id() -> String {
    format!("session-{}", Local::now().format("%Y%m%d%H%M%S%.3f"))
}

fn agent_event(session_id: &str, event_type: &str, payload: Value) -> AgentEvent {
    AgentEvent {
        event_type: event_type.to_string(),
        timestamp: now_string(),
        session_id: session_id.to_string(),
        payload,
    }
}

fn event_payload_text(value: &Value) -> String {
    ["detail", "message", "name", "query", "result", "path", "status", "level"]
        .iter()
        .find_map(|key| value.get(key).and_then(Value::as_str))
        .map(str::to_string)
        .unwrap_or_else(|| value.to_string())
}

fn inspector_from_events(events: &[AgentEvent], workspace: &Path, settings: &Value) -> RuntimeInspectorPayload {
    let mut payload = RuntimeInspectorPayload {
        current_session: events.last().map(|event| event.session_id.clone()),
        current_workspace: Some(workspace.to_string_lossy().to_string()),
        current_model: Some(setting_string(settings, "model.name", "Unknown")),
        current_provider: Some(setting_string(settings, "model.provider", "Unknown")),
        ..Default::default()
    };
    for agent_event in events {
        let detail = event_payload_text(&agent_event.payload);
        let label = agent_event
            .payload
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or(agent_event.event_type.as_str());
        let status = agent_event
            .payload
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or(match agent_event.event_type.as_str() {
                "error" => "error",
                "warning" => "warning",
                "tool_call_finished" | "assistant_message" | "system_status" => "completed",
                _ => "observed",
            });
        let inspector_event = InspectorEvent {
            label: label.to_string(),
            detail: detail.clone(),
            status: status.to_string(),
            time: agent_event.timestamp.clone(),
        };
        match agent_event.event_type.as_str() {
            "tool_call_started" | "tool_call_finished" | "connector_query" | "connector_result" => {
                payload.tool_calls.push(inspector_event.clone());
            }
            "knowledge_hit" => payload.knowledge_hits.push(inspector_event.clone()),
            "memory_hit" => payload.memory_hits.push(inspector_event.clone()),
            "graph_hit" => payload.graph_hits.push(inspector_event.clone()),
            "risk_assessment" => {
                payload.risk_level = Some(detail);
            }
            _ => {}
        }
        payload.execution_trace.push(inspector_event);
    }
    payload
}

fn extract_structured_events(output: &str, session_id: &str, secrets: &Value) -> Vec<AgentEvent> {
    output
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line.trim()).ok())
        .filter_map(|value| {
            let event_type = value.get("event_type").and_then(Value::as_str)?.to_string();
            let payload = value.get("payload").cloned().unwrap_or_else(|| json!({}));
            let sanitized_payload = redact_json(&serde_json::from_str::<Value>(
                &sanitize_text_with_secrets(&payload.to_string(), secrets),
            ).unwrap_or(payload));
            Some(AgentEvent {
                event_type,
                timestamp: value
                    .get("timestamp")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(now_string),
                session_id: value
                    .get("session_id")
                    .and_then(Value::as_str)
                    .unwrap_or(session_id)
                    .to_string(),
                payload: sanitized_payload,
            })
        })
        .collect()
}

fn event_metrics(events: &[AgentEvent]) -> EventMetrics {
    let total_events = events.len();
    let error_events = events.iter().filter(|event| event.event_type == "error").count();
    let tool_finished = events.iter().filter(|event| event.event_type == "tool_call_finished").count();
    let tool_failed = events
        .iter()
        .filter(|event| event.event_type == "tool_call_finished")
        .filter(|event| event.payload.get("success").and_then(Value::as_bool) == Some(false))
        .count();
    let latency_for = |event_type: &str, fallback: u64| {
        events
            .iter()
            .filter(|event| event.event_type == event_type)
            .filter_map(|event| event.payload.get("latency_ms").and_then(Value::as_u64))
            .max()
            .unwrap_or(fallback)
    };
    EventMetrics {
        tool_duration_ms: latency_for("tool_call_finished", 0),
        tool_success_rate: if tool_finished == 0 {
            1.0
        } else {
            ((tool_finished - tool_failed) as f64 / tool_finished as f64 * 100.0).round() / 100.0
        },
        knowledge_query_latency_ms: latency_for("knowledge_hit", 0),
        rag_latency_ms: latency_for("system_status", 0),
        memory_query_latency_ms: latency_for("memory_hit", 0),
        connector_latency_ms: latency_for("connector_result", 0),
        total_events,
        error_events,
    }
}

fn append_runtime_events(state: &RuntimeState, events: &[AgentEvent]) -> Result<(), String> {
    let mut slot = state.events.lock().map_err(|_| "event stream lock poisoned".to_string())?;
    slot.extend(events.iter().cloned());
    if slot.len() > 500 {
        let overflow = slot.len() - 500;
        slot.drain(0..overflow);
    }
    Ok(())
}

fn save_chat_record(
    workspace: &Path,
    prompt: &str,
    output: &str,
    error: Option<&str>,
    events: &[AgentEvent],
    parent_session_id: Option<&str>,
    branch_name: Option<&str>,
) -> Result<String, String> {
    let dir = chats_dir(workspace);
    fs::create_dir_all(&dir).map_err(|err| format!("Unable to create {}: {}", dir.display(), err))?;
    let filename = format!("chat_{}.json", Local::now().format("%Y%m%d_%H%M%S"));
    let path = dir.join(filename);
    let session_id = events
        .last()
        .map(|event| event.session_id.clone())
        .unwrap_or_else(session_id);
    let payload = json!({
        "created_at": now_string(),
        "session_id": session_id,
        "parent_session_id": parent_session_id,
        "branch_name": branch_name.unwrap_or("main"),
        "prompt": prompt,
        "output": output,
        "error": error,
        "archived": false,
        "events": events
    });
    fs::write(&path, serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?)
        .map_err(|err| format!("Unable to write {}: {}", path.display(), err))?;
    Ok(path.to_string_lossy().to_string())
}

fn ensure_chat_path(workspace: &Path, path: &str) -> Result<PathBuf, String> {
    let chat_dir = chats_dir(workspace);
    let target = PathBuf::from(path);
    let canonical_target = target
        .canonicalize()
        .map_err(|err| format!("Unable to resolve chat file {}: {}", target.display(), err))?;
    let canonical_dir = chat_dir
        .canonicalize()
        .map_err(|err| format!("Unable to resolve chats directory {}: {}", chat_dir.display(), err))?;
    if !canonical_target.starts_with(&canonical_dir) {
        return Err("Chat path is outside the workspace chats directory".to_string());
    }
    Ok(canonical_target)
}

#[tauri::command]
fn list_chats() -> Result<Vec<ChatSummary>, String> {
    let workspace = current_workspace_path()?;
    let dir = chats_dir(&workspace);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut rows = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|err| format!("Unable to read {}: {}", dir.display(), err))? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let value = read_json_file(&path);
        let prompt = value.get("prompt").and_then(Value::as_str).unwrap_or("Untitled chat");
        let preview = value.get("output").and_then(Value::as_str).unwrap_or_default();
        rows.push(ChatSummary {
            path: path.to_string_lossy().to_string(),
            title: prompt.chars().take(54).collect(),
            modified: file_modified_string(&path),
            archived: value.get("archived").and_then(Value::as_bool).unwrap_or(false),
            preview: preview.chars().take(120).collect(),
            session_id: value.get("session_id").and_then(Value::as_str).map(str::to_string),
            parent_session_id: value.get("parent_session_id").and_then(Value::as_str).map(str::to_string),
            branch_name: value.get("branch_name").and_then(Value::as_str).map(str::to_string),
        });
    }
    rows.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(rows)
}

#[tauri::command]
fn load_chat_history(path: String) -> Result<Value, String> {
    let workspace = current_workspace_path()?;
    let chat_path = ensure_chat_path(&workspace, &path)?;
    Ok(read_json_file(&chat_path))
}

#[tauri::command]
fn save_chat(prompt: String, output: String) -> Result<String, String> {
    let session = session_id();
    let events = vec![
        agent_event(&session, "user_message", json!({ "message": prompt })),
        agent_event(&session, "assistant_message", json!({ "message": output })),
    ];
    save_chat_record(&current_workspace_path()?, &prompt, &output, None, &events, None, Some("manual"))
}

#[tauri::command]
fn archive_chat(path: String) -> Result<Vec<ChatSummary>, String> {
    let workspace = current_workspace_path()?;
    let chat_path = ensure_chat_path(&workspace, &path)?;
    let mut value = read_json_file(&chat_path);
    if let Value::Object(map) = &mut value {
        map.insert("archived".to_string(), Value::Bool(true));
    }
    fs::write(
        &chat_path,
        serde_json::to_string_pretty(&value).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("Unable to write {}: {}", chat_path.display(), err))?;
    list_chats()
}

#[tauri::command]
fn delete_chat(path: String) -> Result<Vec<ChatSummary>, String> {
    let workspace = current_workspace_path()?;
    let chat_path = ensure_chat_path(&workspace, &path)?;
    fs::remove_file(&chat_path).map_err(|err| format!("Unable to delete {}: {}", chat_path.display(), err))?;
    list_chats()
}

fn save_runtime_log(workspace: &Path, name: &str, content: &str) -> Result<String, String> {
    let dir = logs_dir(workspace);
    fs::create_dir_all(&dir).map_err(|err| format!("Unable to create {}: {}", dir.display(), err))?;
    let settings = load_settings_value(workspace);
    let secrets = load_secrets_value(workspace, &settings);
    let sanitized = sanitize_text_with_secrets(content, &secrets);
    let path = dir.join(format!("{}_{}.log", name, Local::now().format("%Y%m%d_%H%M%S")));
    fs::write(&path, sanitized).map_err(|err| format!("Unable to write {}: {}", path.display(), err))?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn start_agent_session(state: State<RuntimeState>) -> Result<RuntimeStatus, String> {
    let workspace = current_workspace_path()?;
    let settings = load_settings_value(&workspace);
    let secrets = load_secrets_value(&workspace, &settings);
    let (env_name, api_key) = runtime_api_key(&settings, &secrets);
    let exe = claw_exe(&workspace);
    let mut slot = state.agent.lock().map_err(|_| "agent state lock poisoned".to_string())?;
    if slot.child.is_some() {
        slot.status = "Running".to_string();
        drop(slot);
        return get_agent_status(state);
    }
    if !exe.exists() {
        slot.status = "Error".to_string();
        slot.error = Some(format!("claw.exe not found: {}", exe.display()));
        drop(slot);
        return get_agent_status(state);
    }
    if api_key.is_empty() {
        slot.status = "Error".to_string();
        slot.error = Some(format!("Missing API Key. Configure {} in Settings > Model Credentials.", env_name));
        drop(slot);
        return get_agent_status(state);
    }
    let mut command = Command::new(exe);
    apply_runtime_env(&mut command, &workspace, &settings);
    let child = command
        .current_dir(&workspace)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("Unable to start claw.exe: {}", err))?;
    slot.child = Some(child);
    slot.status = "Running".to_string();
    slot.error = None;
    drop(slot);
    get_agent_status(state)
}

#[tauri::command]
fn stop_agent_session(state: State<RuntimeState>) -> Result<RuntimeStatus, String> {
    let mut slot = state.agent.lock().map_err(|_| "agent state lock poisoned".to_string())?;
    if let Some(child) = &mut slot.child {
        let _ = child.kill();
        slot.status = "Stopping".to_string();
        slot.error = Some("Agent process stopped by user".to_string());
        drop(slot);
        return get_agent_status(state);
    }
    slot.child = None;
    slot.status = "Stopped".to_string();
    slot.error = None;
    drop(slot);
    get_agent_status(state)
}

#[tauri::command]
fn send_agent_prompt(prompt: String, state: State<RuntimeState>) -> Result<AgentRunPayload, String> {
    let workspace = current_workspace_path()?;
    let settings = load_settings_value(&workspace);
    let secrets = load_secrets_value(&workspace, &settings);
    let (env_name, api_key) = runtime_api_key(&settings, &secrets);
    let model = setting_string(&settings, "model.name", "Qwen3-Coder-Next");
    let provider = setting_string(&settings, "model.provider", "Unknown");
    let session = session_id();
    let mut events = vec![
        agent_event(&session, "user_message", json!({ "message": prompt })),
        agent_event(&session, "system_status", json!({
            "status": "starting",
            "workspace": workspace.to_string_lossy().to_string(),
            "model": model,
            "provider": provider
        })),
    ];
    let exe = claw_exe(&workspace);
    if !exe.exists() {
        let message = format!("claw.exe not found: {}", exe.display());
        events.push(agent_event(&session, "error", json!({ "message": message, "status": "error" })));
        let inspector = inspector_from_events(&events, &workspace, &settings);
        let _ = append_runtime_events(&state, &events);
        *state.last_inspector.lock().map_err(|_| "inspector lock poisoned".to_string())? = inspector.clone();
        return Ok(AgentRunPayload {
            output: String::new(),
            error: Some(message),
            inspector,
            chat_path: None,
            events,
        });
    }
    if api_key.is_empty() {
        let message = format!("Missing API Key. Configure {} in Settings > Model Credentials.", env_name);
        events.push(agent_event(&session, "warning", json!({ "message": message, "status": "warning" })));
        let inspector = inspector_from_events(&events, &workspace, &settings);
        let _ = append_runtime_events(&state, &events);
        *state.last_inspector.lock().map_err(|_| "inspector lock poisoned".to_string())? = inspector.clone();
        return Ok(AgentRunPayload {
            output: String::new(),
            error: Some(message),
            inspector,
            chat_path: None,
            events,
        });
    }

    {
        let slot = state.agent.lock().map_err(|_| "agent state lock poisoned".to_string())?;
        if slot.child.is_some() {
            return Err("Another agent process is already running".to_string());
        }
    }

    let mut command = Command::new(exe);
    apply_runtime_env(&mut command, &workspace, &settings);
    let started = SystemTime::now();
    let mut child = command
        .current_dir(&workspace)
        .arg("--model")
        .arg(setting_string(&settings, "model.name", "Qwen3-Coder-Next"))
        .arg("--output-format")
        .arg("json")
        .arg("prompt")
        .arg(&prompt)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("Unable to run claw.exe: {}", err))?;

    let stdout_handle = child.stdout.take().map(|mut stdout| {
        thread::spawn(move || {
            let mut buffer = String::new();
            let _ = stdout.read_to_string(&mut buffer);
            buffer
        })
    });
    let stderr_handle = child.stderr.take().map(|mut stderr| {
        thread::spawn(move || {
            let mut buffer = String::new();
            let _ = stderr.read_to_string(&mut buffer);
            buffer
        })
    });

    {
        let mut slot = state.agent.lock().map_err(|_| "agent state lock poisoned".to_string())?;
        slot.child = Some(child);
        slot.status = "Running".to_string();
        slot.error = None;
    }

    let (exit_status, stopped_by_user) = loop {
        thread::sleep(Duration::from_millis(120));
        let mut slot = state.agent.lock().map_err(|_| "agent state lock poisoned".to_string())?;
        let stopping = slot.status == "Stopping";
        let Some(child) = slot.child.as_mut() else {
            break (None, true);
        };
        match child.try_wait().map_err(|err| format!("Unable to wait for claw.exe: {}", err))? {
            Some(status) => {
                let stopped_by_user = stopping || slot.error.as_deref() == Some("Agent process stopped by user");
                slot.child = None;
                slot.status = "Stopped".to_string();
                if !stopped_by_user {
                    slot.error = None;
                }
                break (Some(status), stopped_by_user);
            }
            None => {}
        }
    };
    let runtime_ms = started.elapsed().map(|elapsed| elapsed.as_millis() as u64).unwrap_or(0);

    let stdout = stdout_handle
        .map(|handle| handle.join().unwrap_or_default())
        .unwrap_or_default();
    let stderr = stderr_handle
        .map(|handle| handle.join().unwrap_or_default())
        .unwrap_or_default();
    let raw_combined = if stderr.trim().is_empty() {
        stdout.clone()
    } else {
        format!("{}\n\n[stderr]\n{}", stdout, stderr)
    };
    let combined = sanitize_text_with_secrets(&raw_combined, &secrets);
    let sanitized_stderr = sanitize_text_with_secrets(&stderr, &secrets);
    let success = exit_status.map(|status| status.success()).unwrap_or(false) && !stopped_by_user;
    let exit_code = exit_status.and_then(|status| status.code());
    let error = if stopped_by_user {
        Some("Agent run stopped by user".to_string())
    } else if success {
        None
    } else {
        Some(sanitized_stderr)
    };
    events.extend(extract_structured_events(&combined, &session, &secrets));
    if success {
        events.push(agent_event(&session, "assistant_message", json!({
            "message": combined,
            "status": "completed",
            "latency_ms": runtime_ms
        })));
    } else if stopped_by_user {
        events.push(agent_event(&session, "agent_stopped", json!({
            "message": "Agent run stopped by user",
            "status": "stopped",
            "latency_ms": runtime_ms
        })));
    } else {
        events.push(agent_event(&session, "error", json!({
            "message": error.clone().unwrap_or_else(|| "Agent runtime exited with an error".to_string()),
            "status": "error",
            "latency_ms": runtime_ms
        })));
    }
    events.push(agent_event(&session, "system_status", json!({
        "status": if success { "completed" } else if stopped_by_user { "stopped" } else { "error" },
        "latency_ms": runtime_ms,
        "exit_code": exit_code
    })));
    let inspector = inspector_from_events(&events, &workspace, &settings);
    let chat_path = save_chat_record(&workspace, &prompt, &combined, error.as_deref(), &events, None, Some("main")).ok();
    let _ = save_runtime_log(&workspace, "agent", &combined);
    append_runtime_events(&state, &events)?;
    *state.last_agent_output.lock().map_err(|_| "agent output lock poisoned".to_string())? = combined.clone();
    *state.last_inspector.lock().map_err(|_| "inspector lock poisoned".to_string())? = inspector.clone();

    Ok(AgentRunPayload {
        output: combined,
        error,
        inspector,
        chat_path,
        events,
    })
}

#[tauri::command]
fn stream_agent_output(state: State<RuntimeState>) -> Result<String, String> {
    Ok(state
        .last_agent_output
        .lock()
        .map_err(|_| "agent output lock poisoned".to_string())?
        .clone())
}

#[tauri::command]
fn get_agent_status(state: State<RuntimeState>) -> Result<RuntimeStatus, String> {
    let agent = state.agent.lock().map_err(|_| "agent state lock poisoned".to_string())?;
    let rag = state.rag.lock().map_err(|_| "rag state lock poisoned".to_string())?;
    Ok(RuntimeStatus {
        agent: if agent.status.is_empty() { "Offline".to_string() } else { agent.status.clone() },
        rag: if rag.status.is_empty() { "Stopped".to_string() } else { rag.status.clone() },
        agent_error: agent.error.clone(),
        rag_error: rag.error.clone(),
    })
}

#[tauri::command]
fn start_rag_service(state: State<RuntimeState>) -> Result<RuntimeStatus, String> {
    let workspace = current_workspace_path()?;
    let settings = load_settings_value(&workspace);
    let exe = rag_exe(&workspace);
    let db_path = setting_path(&workspace, &settings, "rag.db_path", "beifeng/data/wind.sqlite");
    let mut slot = state.rag.lock().map_err(|_| "rag state lock poisoned".to_string())?;
    if slot.child.is_some() {
        slot.status = "Running".to_string();
        drop(slot);
        return get_agent_status(state);
    }
    let mut command = if exe.exists() {
        let mut command = Command::new(exe);
        command.current_dir(&workspace);
        command.arg("serve");
        command
    } else {
        let mut command = Command::new("cargo");
        command.current_dir(rust_root(&workspace));
        command.arg("run").arg("-p").arg("claw-rag-service").arg("--").arg("serve");
        command
    };
    apply_runtime_env(&mut command, &workspace, &settings);
    let child = command
        .env("CLAW_RAG_MOCK_PROVIDERS", "1")
        .arg("--db")
        .arg(db_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| {
            slot.status = "Error".to_string();
            slot.error = Some(format!("Unable to start RAG service: {}", err));
            slot.error.clone().unwrap_or_else(|| "Unable to start RAG service".to_string())
        })?;
    slot.child = Some(child);
    slot.status = "Running".to_string();
    slot.error = None;
    drop(slot);
    get_agent_status(state)
}

#[tauri::command]
fn stop_rag_service(state: State<RuntimeState>) -> Result<RuntimeStatus, String> {
    let mut slot = state.rag.lock().map_err(|_| "rag state lock poisoned".to_string())?;
    if let Some(child) = &mut slot.child {
        let _ = child.kill();
    }
    slot.child = None;
    slot.status = "Stopped".to_string();
    drop(slot);
    get_agent_status(state)
}

#[tauri::command]
fn restart_rag_service(state: State<RuntimeState>) -> Result<RuntimeStatus, String> {
    let _ = stop_rag_service(state.clone());
    start_rag_service(state)
}

#[tauri::command]
fn get_rag_health() -> Result<RagHealth, String> {
    let workspace = current_workspace_path()?;
    let settings = load_settings_value(&workspace);
    let service_url = setting_string(&settings, "rag.service_url", "http://127.0.0.1:8787");
    let status = if service_url.contains("127.0.0.1") || service_url.contains("localhost") {
        "Configured".to_string()
    } else {
        "RemoteConfigured".to_string()
    };
    Ok(RagHealth {
        status,
        service_url,
        error: None,
    })
}

#[tauri::command]
fn get_runtime_inspector(state: State<RuntimeState>) -> Result<RuntimeInspectorPayload, String> {
    Ok(state
        .last_inspector
        .lock()
        .map_err(|_| "inspector lock poisoned".to_string())?
        .clone())
}

#[tauri::command]
fn get_system_monitor(state: State<RuntimeState>) -> Result<SystemMonitorPayload, String> {
    let workspace = current_workspace_path()?;
    let settings = load_settings_value(&workspace);
    let runtime = get_agent_status(state)?;
    let knowledge_docs = count_files_with_extensions(&workspace.join("beifeng").join("knowledge"), &["md", "txt", "json"]);
    let memory_records = count_json_records(&read_fault_history()?) + count_json_records(&read_maintenance_history()?) + count_json_records(&read_report_history()?);
    let graph_nodes = count_json_records(&read_json_file(&workspace.join("beifeng").join("knowledge").join("knowledge_graph").join("wind_fault_graph.json")));
    let reports = list_reports()?.len();
    let benchmark = read_latest_benchmark_report()?
        .scores
        .get("overall")
        .cloned()
        .unwrap_or_else(|| "N/A".to_string());
    let connectors_count = fs::read_dir(workspace.join("beifeng").join("connectors"))
        .map(|entries| entries.filter_map(Result::ok).filter(|entry| entry.path().is_dir()).count())
        .unwrap_or(0);
    Ok(SystemMonitorPayload {
        agent: runtime.agent,
        rag: runtime.rag,
        model: setting_string(&settings, "model.name", "Unknown"),
        workspace: workspace.to_string_lossy().to_string(),
        knowledge_docs,
        memory_records,
        graph_nodes,
        reports,
        benchmark,
        connectors: format!("{connectors_count} configured"),
    })
}

fn health_item(name: &str, path: PathBuf, ok_hint: &str, missing_hint: &str) -> HealthItem {
    let started = SystemTime::now();
    let exists = path.exists();
    let response_time_ms = started.elapsed().map(|elapsed| elapsed.as_millis() as u64).unwrap_or(0);
    HealthItem {
        name: name.to_string(),
        status: if exists { "Healthy".to_string() } else { "Offline".to_string() },
        updated: if exists { file_modified_string(&path) } else { "-".to_string() },
        error: (!exists).then(|| format!("{} not found", path.display())),
        suggestion: if exists { ok_hint.to_string() } else { missing_hint.to_string() },
        path: path.to_string_lossy().to_string(),
        response_time_ms,
        health_check_result: if exists { ok_hint.to_string() } else { missing_hint.to_string() },
    }
}

#[tauri::command]
fn get_health_dashboard(state: State<RuntimeState>) -> Result<Vec<HealthItem>, String> {
    let workspace = current_workspace_path()?;
    let settings = load_settings_value(&workspace);
    let mut items = vec![
        health_item("Settings", settings_path(&workspace), "settings.json is available", "Create settings.json from the Settings page"),
        health_item("Knowledge Base", workspace.join(setting_string(&settings, "paths.knowledge_base", "beifeng/knowledge/knowledge_base")), "Knowledge documents are available", "Add Markdown knowledge documents"),
        health_item("Knowledge Graph", workspace.join(setting_string(&settings, "paths.knowledge_graph", "beifeng/knowledge/knowledge_graph/wind_fault_graph.json")), "Knowledge graph file is available", "Restore wind_fault_graph.json"),
        health_item("Memory Runtime", memory_dir(&workspace), "Memory directory exists", "Create beifeng/memory or restore memory files"),
        health_item("Report Engine", reports_dir(&workspace), "Reports directory exists", "Generate a report or create beifeng/reports/generated"),
        health_item("Benchmark", evals_dir(&workspace), "Benchmark directory exists", "Run benchmark or restore beifeng/evals"),
        health_item("Connector Registry", workspace.join("beifeng").join("connectors"), "Connector registry exists", "Restore connector schema directory"),
        health_item("Agent Runtime", claw_exe(&workspace), "claw.exe exists", "Build the Rust CLI before starting Agent Runtime"),
        health_item("RAG Database", workspace.join(setting_string(&settings, "rag.db_path", "beifeng/data/wind.sqlite")), "RAG database exists", "Run knowledge ingestion before starting RAG"),
    ];
    let rag = get_agent_status(state)?.rag;
    items.push(HealthItem {
        name: "RAG Service".to_string(),
        status: if rag == "Running" { "Healthy".to_string() } else { "Warning".to_string() },
        path: setting_string(&settings, "rag.service_url", "http://127.0.0.1:8787"),
        updated: now_string(),
        error: (rag != "Running").then(|| "RAG service is not running".to_string()),
        suggestion: if rag == "Running" { "RAG service is running".to_string() } else { "Use System page to start RAG service".to_string() },
        response_time_ms: 0,
        health_check_result: rag,
    });
    let secrets = load_secrets_value(&workspace, &settings);
    let (api_key_env, api_key) = runtime_api_key(&settings, &secrets);
    let api_key_available = !api_key.is_empty();
    items.push(HealthItem {
        name: "Model Provider".to_string(),
        status: if api_key_available { "Healthy".to_string() } else { "Warning".to_string() },
        path: setting_string(&settings, "model.provider", "Unknown"),
        updated: now_string(),
        error: (!api_key_available).then(|| format!("Credential {} is not configured", api_key_env)),
        suggestion: if api_key_available {
            "Model provider credential is available from local secrets or environment".to_string()
        } else {
            format!("Save {} in Settings > Model Credentials", api_key_env)
        },
        response_time_ms: 0,
        health_check_result: if api_key_available { "Credential available".to_string() } else { "Credential missing".to_string() },
    });
    Ok(items)
}

#[tauri::command]
fn list_agent_events(session_id_filter: Option<String>, state: State<RuntimeState>) -> Result<Vec<AgentEvent>, String> {
    let events = state.events.lock().map_err(|_| "event stream lock poisoned".to_string())?;
    Ok(events
        .iter()
        .filter(|event| session_id_filter.as_ref().map(|session| &event.session_id == session).unwrap_or(true))
        .cloned()
        .collect())
}

fn chat_branch_from_path(path: PathBuf, current_path: Option<&str>) -> ConversationBranch {
    let value = read_json_file(&path);
    let prompt = value.get("prompt").and_then(Value::as_str).unwrap_or("Untitled chat");
    let session = value.get("session_id").and_then(Value::as_str).unwrap_or("unknown-session");
    let events = value.get("events").and_then(Value::as_array).map(Vec::len).unwrap_or(0);
    ConversationBranch {
        path: path.to_string_lossy().to_string(),
        session_id: session.to_string(),
        parent_session_id: value.get("parent_session_id").and_then(Value::as_str).map(str::to_string),
        branch_name: value.get("branch_name").and_then(Value::as_str).unwrap_or("main").to_string(),
        title: prompt.chars().take(54).collect(),
        modified: file_modified_string(&path),
        active: current_path.map(|current| current == path.to_string_lossy().as_ref()).unwrap_or(false),
        events,
    }
}

#[tauri::command]
fn list_conversation_branches(current_path: Option<String>) -> Result<Vec<ConversationBranch>, String> {
    let workspace = current_workspace_path()?;
    let dir = chats_dir(&workspace);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut rows = fs::read_dir(&dir)
        .map_err(|err| format!("Unable to read {}: {}", dir.display(), err))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .map(|path| chat_branch_from_path(path, current_path.as_deref()))
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(rows)
}

#[tauri::command]
fn fork_chat_session(path: String) -> Result<String, String> {
    let workspace = current_workspace_path()?;
    let chat_path = ensure_chat_path(&workspace, &path)?;
    let mut value = read_json_file(&chat_path);
    let parent = value
        .get("session_id")
        .and_then(Value::as_str)
        .unwrap_or("unknown-session")
        .to_string();
    let next_session = session_id();
    if let Value::Object(map) = &mut value {
        map.insert("session_id".to_string(), Value::String(next_session.clone()));
        map.insert("parent_session_id".to_string(), Value::String(parent));
        map.insert("branch_name".to_string(), Value::String(format!("fork {}", Local::now().format("%H:%M:%S"))));
        map.insert("created_at".to_string(), Value::String(now_string()));
        if let Some(events) = map.get_mut("events").and_then(Value::as_array_mut) {
            events.push(json!(agent_event(&next_session, "system_status", json!({ "status": "forked", "detail": "Session forked from chat tree" }))));
        }
    }
    let target = chats_dir(&workspace).join(format!("chat_fork_{}.json", Local::now().format("%Y%m%d_%H%M%S")));
    fs::write(
        &target,
        serde_json::to_string_pretty(&value).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("Unable to write {}: {}", target.display(), err))?;
    Ok(target.to_string_lossy().to_string())
}

#[tauri::command]
fn restore_chat_branch(path: String) -> Result<Vec<ChatSummary>, String> {
    let workspace = current_workspace_path()?;
    let chat_path = ensure_chat_path(&workspace, &path)?;
    let mut value = read_json_file(&chat_path);
    if let Value::Object(map) = &mut value {
        map.insert("branch_name".to_string(), Value::String("restored".to_string()));
        map.insert("restored_at".to_string(), Value::String(now_string()));
    }
    fs::write(
        &chat_path,
        serde_json::to_string_pretty(&value).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("Unable to write {}: {}", chat_path.display(), err))?;
    list_chats()
}

#[tauri::command]
fn compare_chat_branches(left_path: String, right_path: String) -> Result<BranchComparison, String> {
    let workspace = current_workspace_path()?;
    let left = chat_branch_from_path(ensure_chat_path(&workspace, &left_path)?, Some(&left_path));
    let right = chat_branch_from_path(ensure_chat_path(&workspace, &right_path)?, Some(&right_path));
    let event_delta = left.events as isize - right.events as isize;
    Ok(BranchComparison {
        summary: format!("{} vs {}: event delta {}", left.branch_name, right.branch_name, event_delta),
        left: Some(left),
        right: Some(right),
        event_delta,
    })
}

fn artifact_from_path(path: PathBuf, artifact_type: &str, session_id: Option<String>) -> ArtifactSummary {
    let metadata = fs::metadata(&path).ok();
    ArtifactSummary {
        id: path.to_string_lossy().to_string(),
        artifact_type: artifact_type.to_string(),
        title: path.file_name().and_then(|name| name.to_str()).unwrap_or("artifact").to_string(),
        path: path.to_string_lossy().to_string(),
        session_id,
        modified: file_modified_string(&path),
        size: metadata.map(|meta| meta.len()).unwrap_or(0),
    }
}

#[tauri::command]
fn list_artifacts(session_id_filter: Option<String>) -> Result<Vec<ArtifactSummary>, String> {
    let workspace = current_workspace_path()?;
    let mut artifacts = Vec::new();
    for (dir, artifact_type) in [(reports_dir(&workspace), "report"), (logs_dir(&workspace), "runtime_log"), (chats_dir(&workspace), "chat_session")] {
        if !dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&dir).map_err(|err| format!("Unable to read {}: {}", dir.display(), err))?.filter_map(Result::ok) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let session_id = if artifact_type == "chat_session" {
                read_json_file(&path).get("session_id").and_then(Value::as_str).map(str::to_string)
            } else {
                None
            };
            if session_id_filter.as_ref().map(|filter| session_id.as_ref() != Some(filter)).unwrap_or(false) {
                continue;
            }
            artifacts.push(artifact_from_path(path, artifact_type, session_id));
        }
    }
    artifacts.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(artifacts)
}

#[tauri::command]
fn open_artifact(path: String) -> Result<(), String> {
    open_file(path)
}

#[tauri::command]
fn reveal_artifact(path: String) -> Result<(), String> {
    reveal_in_explorer(Some(path))
}

#[tauri::command]
fn export_artifact(path: String) -> Result<String, String> {
    let source = PathBuf::from(&path);
    ensure_existing_path(&source)?;
    let workspace = current_workspace_path()?;
    let export_dir = workspace.join("beifeng").join("artifacts").join("exports");
    fs::create_dir_all(&export_dir).map_err(|err| format!("Unable to create {}: {}", export_dir.display(), err))?;
    let filename = source.file_name().and_then(|name| name.to_str()).unwrap_or("artifact.txt");
    let target = export_dir.join(filename);
    fs::copy(&source, &target).map_err(|err| format!("Unable to export {}: {}", source.display(), err))?;
    Ok(target.to_string_lossy().to_string())
}

#[tauri::command]
fn delete_artifact(path: String) -> Result<Vec<ArtifactSummary>, String> {
    let target = PathBuf::from(&path);
    ensure_existing_path(&target)?;
    let workspace = current_workspace_path()?;
    let canonical_workspace = workspace.canonicalize().map_err(|err| format!("Unable to resolve workspace: {}", err))?;
    let canonical_target = target.canonicalize().map_err(|err| format!("Unable to resolve {}: {}", target.display(), err))?;
    if !canonical_target.starts_with(canonical_workspace) {
        return Err("Artifact path is outside the current workspace".to_string());
    }
    fs::remove_file(&canonical_target).map_err(|err| format!("Unable to delete {}: {}", canonical_target.display(), err))?;
    list_artifacts(None)
}

fn collect_workspace_scan_files(root: &Path, rows: &mut Vec<PathBuf>, depth: usize) {
    if depth > 5 || rows.len() > 1200 || !root.exists() {
        return;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_workspace_scan_files(&path, rows, depth + 1);
        } else {
            rows.push(path);
        }
    }
}

#[tauri::command]
fn analyze_workspace_cleanup() -> Result<WorkspaceHealthReport, String> {
    let workspace = current_workspace_path()?;
    let mut files = Vec::new();
    collect_workspace_scan_files(&workspace, &mut files, 0);
    let mut issues = Vec::new();
    let mut seen = BTreeMap::<String, PathBuf>::new();
    let mut duplicate_files = 0;
    let mut unused_caches = 0;
    let mut old_benchmark_files = 0;
    let legacy_folders = ["old_outputs", "archive", "archives"]
        .iter()
        .filter(|name| workspace.join(name).exists())
        .count();
    for path in &files {
        let metadata = fs::metadata(path).ok();
        let size = metadata.as_ref().map(|meta| meta.len()).unwrap_or(0);
        let key = format!("{}:{}", path.file_name().and_then(|name| name.to_str()).unwrap_or(""), size);
        if let Some(first) = seen.get(&key) {
            duplicate_files += 1;
            issues.push(WorkspaceCleanupIssue {
                category: "duplicate files".to_string(),
                path: path.to_string_lossy().to_string(),
                severity: "Warning".to_string(),
                detail: format!("Possible duplicate of {}", first.display()),
                suggestion: "Review both files before deleting either copy.".to_string(),
            });
        } else {
            seen.insert(key, path.clone());
        }
        let text = path.to_string_lossy().to_ascii_lowercase();
        if text.contains("__pycache__") || text.ends_with(".pyc") || text.contains("\\target\\") {
            unused_caches += 1;
            issues.push(WorkspaceCleanupIssue {
                category: "unused caches".to_string(),
                path: path.to_string_lossy().to_string(),
                severity: "Info".to_string(),
                detail: "Generated cache/build artifact detected".to_string(),
                suggestion: "Safe to review for cleanup after builds are complete.".to_string(),
            });
        }
        if text.contains("beifeng\\evals") && (text.ends_with(".json") || text.ends_with(".md")) {
            old_benchmark_files += 1;
        }
    }
    let report_paths = list_reports()?.into_iter().map(|report| report.path).collect::<Vec<_>>();
    let orphan_reports = report_paths
        .iter()
        .filter(|path| !path.to_ascii_lowercase().contains("chat"))
        .count();
    if orphan_reports > 0 {
        issues.push(WorkspaceCleanupIssue {
            category: "orphan reports".to_string(),
            path: reports_dir(&workspace).to_string_lossy().to_string(),
            severity: "Info".to_string(),
            detail: format!("{} reports are not explicitly linked to a chat session", orphan_reports),
            suggestion: "Open Reports and link important reports to the relevant session before archiving.".to_string(),
        });
    }
    for name in ["old_outputs", "archive", "archives"] {
        let path = workspace.join(name);
        if path.exists() {
            issues.push(WorkspaceCleanupIssue {
                category: "legacy folders".to_string(),
                path: path.to_string_lossy().to_string(),
                severity: "Warning".to_string(),
                detail: "Legacy/archive folder detected".to_string(),
                suggestion: "Review contents and move active artifacts into workspace folders.".to_string(),
            });
        }
    }
    let markdown = format!(
        "# Workspace Health Report\n\nGenerated: {}\n\nWorkspace: {}\n\n- Duplicate files: {}\n- Legacy folders: {}\n- Unused caches: {}\n- Orphan reports: {}\n- Old benchmark files: {}\n\nNo files were deleted.\n",
        now_string(),
        workspace.display(),
        duplicate_files,
        legacy_folders,
        unused_caches,
        orphan_reports,
        old_benchmark_files
    );
    Ok(WorkspaceHealthReport {
        workspace: workspace.to_string_lossy().to_string(),
        generated_at: now_string(),
        duplicate_files,
        legacy_folders,
        unused_caches,
        orphan_reports,
        old_benchmark_files,
        issues,
        markdown,
    })
}

#[tauri::command]
fn get_agent_console(state: State<RuntimeState>) -> Result<ConsolePayload, String> {
    let workspace = current_workspace_path()?;
    let loaded_settings = load_settings_value(&workspace);
    let secrets = load_secrets_value(&workspace, &loaded_settings);
    let settings = redact_json(&loaded_settings);
    let inspector = get_runtime_inspector(state.clone())?;
    let logs = sanitize_text_with_secrets(&stream_agent_output(state.clone())?, &secrets);
    let events = list_agent_events(None, state.clone())?;
    let metrics = event_metrics(&events);
    Ok(ConsolePayload {
        prompt: "Latest prompt is stored in chat JSON after execution.".to_string(),
        system_context: settings,
        tool_calls: inspector.tool_calls,
        raw_tool_results: logs.clone(),
        memory_context: read_json_file(&memory_dir(&workspace).join("fault_history.json")),
        knowledge_context: "Knowledge context is populated after wind_knowledge_query events.".to_string(),
        graph_context: "Graph context is populated after graph matching events.".to_string(),
        risk_assessment_json: json!({ "risk_level": inspector.risk_level }),
        report_path: None,
        runtime_logs: logs,
        event_timeline: events.clone(),
        raw_event_json: json!(events),
        metrics,
    })
}

#[tauri::command]
fn save_debug_log(content: String) -> Result<String, String> {
    save_runtime_log(&current_workspace_path()?, "debug", &content)
}

pub fn run() {
    tauri::Builder::default()
        .manage(RuntimeState::default())
        .invoke_handler(tauri::generate_handler![
            select_workspace_folder,
            create_workspace,
            import_workspace_folder,
            get_current_workspace,
            set_current_workspace,
            list_recent_workspaces,
            archive_workspace,
            remove_workspace_from_list,
            get_language_preference,
            set_language_preference,
            reveal_in_explorer,
            open_in_vscode,
            open_file,
            rename_path,
            list_workspace_files,
            list_skills,
            read_settings_json,
            write_settings_json,
            reset_settings_json,
            validate_settings_json,
            open_settings_json_in_vscode,
            read_secrets_json,
            write_secrets_json,
            open_secrets_json_in_vscode,
            save_model_credential,
            clear_model_credential,
            test_model_credentials,
            list_reports,
            read_report,
            read_report_detail,
            export_report_markdown,
            reveal_report,
            open_report_in_vscode,
            read_turbine_profiles,
            read_fault_history,
            read_maintenance_history,
            read_report_history,
            read_memory_payload,
            list_benchmark_reports,
            read_latest_benchmark_report,
            parse_key_scores,
            start_agent_session,
            stop_agent_session,
            send_agent_prompt,
            stream_agent_output,
            list_agent_events,
            list_chats,
            load_chat_history,
            save_chat,
            archive_chat,
            delete_chat,
            list_conversation_branches,
            fork_chat_session,
            restore_chat_branch,
            compare_chat_branches,
            list_artifacts,
            open_artifact,
            reveal_artifact,
            export_artifact,
            delete_artifact,
            analyze_workspace_cleanup,
            get_agent_status,
            start_rag_service,
            stop_rag_service,
            restart_rag_service,
            get_rag_health,
            get_runtime_inspector,
            get_system_monitor,
            get_health_dashboard,
            get_agent_console,
            save_debug_log
        ])
        .run(tauri::generate_context!())
        .expect("error while running BeiFeng Agent Desktop");
}
