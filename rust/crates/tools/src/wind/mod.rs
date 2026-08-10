mod fault;
mod knowledge;
mod normalize;
mod report;

use runtime::PermissionMode;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{from_value, to_pretty_json, ToolSpec};

#[cfg(test)]
pub(crate) use knowledge::wind_knowledge_query_request;
#[cfg(test)]
pub(crate) use report::wind_report_markdown;

#[derive(Debug, Deserialize)]
pub(crate) struct WindKnowledgeQueryInput {
    pub(crate) query: String,
    pub(crate) component: Option<String>,
    pub(crate) symptom: Option<String>,
    pub(crate) domain: Option<String>,
    pub(crate) equipment: Option<String>,
    pub(crate) top_k: Option<u32>,
    pub(crate) debug: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WindFaultAnalysisInput {
    pub(crate) problem: String,
    pub(crate) component: Option<String>,
    pub(crate) symptom: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WindReportGenerateInput {
    pub(crate) problem: String,
    pub(crate) component: Option<String>,
    pub(crate) symptom: Option<String>,
    pub(crate) report_type: Option<String>,
    pub(crate) title: Option<String>,
}

pub(crate) fn tool_specs() -> Vec<ToolSpec> {
    vec![
        ToolSpec {
            name: "wind_knowledge_query",
            description:
                "Query the local Wind Knowledge Hub via claw-rag-service /v1/query. When the user asks about wind turbine inspection, defect detection, re-check intervals, maintenance advice, shutdown evaluation, or safety risk, call this tool first. Returns RAG hits, graph_suggestions, rule-based advice, and risk_assessment.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "minLength": 1 },
                    "component": { "type": "string" },
                    "symptom": { "type": "string" },
                    "domain": { "type": "string" },
                    "equipment": { "type": "string" },
                    "top_k": { "type": "number", "minimum": 1 },
                    "debug": { "type": "boolean" }
                },
                "required": ["query"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "wind_fault_analysis",
            description:
                "Run the Wind Fault Analysis workflow for wind turbine fault diagnosis, inspection anomaly analysis, inspection advice generation, and risk assessment. Prefer this for Blade, Gearbox, Generator, Yaw, Pitch, SCADA, and Safety related fault questions.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "problem": { "type": "string", "minLength": 1 },
                    "component": { "type": "string" },
                    "symptom": { "type": "string" }
                },
                "required": ["problem"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "wind_report_generate",
            description:
                "Generate a Markdown Wind O&M report from Wind Fault Analysis. Use this for inspection analysis reports, fault analysis reports, risk assessment reports, and maintenance advice reports for Blade, Gearbox, Generator, Yaw, Pitch, SCADA, and Safety related problems.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "problem": { "type": "string", "minLength": 1 },
                    "component": { "type": "string" },
                    "symptom": { "type": "string" },
                    "report_type": {
                        "type": "string",
                        "enum": [
                            "inspection_report",
                            "fault_report",
                            "maintenance_advice",
                            "risk_assessment_report"
                        ]
                    },
                    "title": { "type": "string" }
                },
                "required": ["problem"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::WorkspaceWrite,
        },
    ]
}

pub(crate) fn execute(name: &str, input: &Value) -> Result<String, String> {
    match name {
        "wind_knowledge_query" => {
            from_value::<WindKnowledgeQueryInput>(input).and_then(run_wind_knowledge_query)
        }
        "wind_fault_analysis" => {
            from_value::<WindFaultAnalysisInput>(input).and_then(run_wind_fault_analysis)
        }
        "wind_report_generate" => {
            from_value::<WindReportGenerateInput>(input).and_then(run_wind_report_generate)
        }
        _ => Err(format!("unsupported wind tool: {name}")),
    }
}

#[allow(clippy::needless_pass_by_value)]
fn run_wind_knowledge_query(input: WindKnowledgeQueryInput) -> Result<String, String> {
    to_pretty_json(knowledge::execute_wind_knowledge_query(&input)?)
}

#[allow(clippy::needless_pass_by_value)]
fn run_wind_fault_analysis(input: WindFaultAnalysisInput) -> Result<String, String> {
    to_pretty_json(fault::execute_wind_fault_analysis(&input)?)
}

#[allow(clippy::needless_pass_by_value)]
fn run_wind_report_generate(input: WindReportGenerateInput) -> Result<String, String> {
    to_pretty_json(report::execute_wind_report_generate(&input)?)
}
