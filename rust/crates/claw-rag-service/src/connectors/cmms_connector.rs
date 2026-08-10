use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    assess_records, confidence_for_source, Connector as ConnectorTrait, ConnectorHealth,
    ConnectorRecord, ConnectorRequest, ConnectorResult, ConnectorStatus, SourceKind,
};

#[derive(Debug, Clone)]
pub enum CmmsSource {
    JsonFile(PathBuf),
    RestApi { endpoint: String },
    DatabasePlaceholder { connection_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkOrder {
    pub work_order_id: String,
    pub turbine_id: String,
    pub component: String,
    pub status: String,
    pub opened_at: String,
    #[serde(default)]
    pub closed_at: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MaintenanceRecord {
    pub record_id: String,
    pub turbine_id: String,
    pub component: String,
    pub action: String,
    pub performed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SparePartHistory {
    pub part_id: String,
    pub turbine_id: String,
    pub component: String,
    pub part_name: String,
    pub replaced_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
struct CmmsPayload {
    #[serde(default)]
    work_orders: Vec<WorkOrder>,
    #[serde(default)]
    maintenance_records: Vec<MaintenanceRecord>,
    #[serde(default)]
    spare_parts: Vec<SparePartHistory>,
}

#[derive(Debug, Clone)]
pub struct CmmsConnector {
    pub source: CmmsSource,
}

impl CmmsConnector {
    fn load_payload(&self, request: &ConnectorRequest) -> Result<CmmsPayload, String> {
        match &self.source {
            CmmsSource::JsonFile(path) => {
                let raw = std::fs::read_to_string(path)
                    .map_err(|e| format!("read CMMS json {}: {e}", path.display()))?;
                let mut payload: CmmsPayload = serde_json::from_str(&raw)
                    .map_err(|e| format!("parse CMMS json {}: {e}", path.display()))?;
                if let Some(turbine_id) = request.turbine_id.as_deref() {
                    payload.work_orders.retain(|item| item.turbine_id == turbine_id);
                    payload
                        .maintenance_records
                        .retain(|item| item.turbine_id == turbine_id);
                    payload.spare_parts.retain(|item| item.turbine_id == turbine_id);
                }
                Ok(payload)
            }
            CmmsSource::RestApi { endpoint } => Err(format!(
                "CMMS REST API source is configured ({endpoint}); enterprise auth mapping is required"
            )),
            CmmsSource::DatabasePlaceholder { connection_name } => Err(format!(
                "CMMS database source {connection_name} requires site-specific driver configuration"
            )),
        }
    }
}

impl ConnectorTrait for CmmsConnector {
    fn name(&self) -> String {
        "cmms".to_string()
    }

    fn health(&self) -> ConnectorHealth {
        match &self.source {
            CmmsSource::JsonFile(path) => {
                if path.is_file() {
                    ConnectorHealth::healthy("CMMS JSON file available")
                } else {
                    ConnectorHealth::unavailable(format!(
                        "CMMS JSON file not found: {}",
                        path.display()
                    ))
                }
            }
            CmmsSource::RestApi { .. } | CmmsSource::DatabasePlaceholder { .. } => {
                ConnectorHealth {
                    status: ConnectorStatus::Degraded,
                    message: "CMMS source requires enterprise runtime configuration".to_string(),
                }
            }
        }
    }

    fn query(&self, request: ConnectorRequest) -> ConnectorResult {
        match self.load_payload(&request) {
            Ok(payload) => {
                let mut records = Vec::new();
                push_records(
                    &mut records,
                    SourceKind::Cmms,
                    "WorkOrder",
                    &payload.work_orders,
                );
                push_records(
                    &mut records,
                    SourceKind::Cmms,
                    "MaintenanceRecord",
                    &payload.maintenance_records,
                );
                push_records(
                    &mut records,
                    SourceKind::Cmms,
                    "SparePartHistory",
                    &payload.spare_parts,
                );
                ConnectorResult {
                    connector_name: self.name(),
                    health: self.health(),
                    data_quality: Some(assess_records(&records)),
                    source_confidence: Some(confidence_for_source(SourceKind::Cmms)),
                    records,
                    warnings: Vec::new(),
                }
            }
            Err(e) => {
                let mut result = ConnectorResult::empty(
                    self.name(),
                    ConnectorHealth {
                        status: ConnectorStatus::Degraded,
                        message: e.clone(),
                    },
                );
                result.source_confidence = Some(confidence_for_source(SourceKind::Cmms));
                result.warnings.push(e);
                result
            }
        }
    }
}

fn push_records<T: Serialize>(
    records: &mut Vec<ConnectorRecord>,
    source: SourceKind,
    record_type: &str,
    values: &[T],
) {
    records.extend(values.iter().filter_map(|value| {
        serde_json::to_value(value)
            .ok()
            .map(|payload| ConnectorRecord {
                source,
                record_type: record_type.to_string(),
                payload,
            })
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn cmms_json_connector_filters_by_turbine() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("cmms.json");
        std::fs::write(
            &path,
            r#"{"work_orders":[{"work_order_id":"WO-1","turbine_id":"T-01","component":"Gearbox","status":"open","opened_at":"2026-06-05","description":"oil temp"}],"maintenance_records":[],"spare_parts":[]}"#,
        )
        .unwrap();
        let connector = CmmsConnector {
            source: CmmsSource::JsonFile(path),
        };
        let result = connector.query(ConnectorRequest {
            turbine_id: Some("T-01".to_string()),
            ..ConnectorRequest::default()
        });
        assert_eq!(result.records.len(), 1);
        assert_eq!(result.records[0].record_type, "WorkOrder");
    }
}
