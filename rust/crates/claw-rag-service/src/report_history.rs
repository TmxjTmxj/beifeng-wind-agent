use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReportHistoryRecord {
    pub report_id: String,
    pub turbine_id: String,
    pub report_type: String,
    pub problem: String,
    pub risk_level: String,
    pub report_path: String,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}
