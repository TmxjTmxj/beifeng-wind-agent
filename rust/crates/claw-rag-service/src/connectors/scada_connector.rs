use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    assess_scada_points, confidence_for_source, Connector as ConnectorTrait, ConnectorHealth,
    ConnectorRecord, ConnectorRequest, ConnectorResult, ConnectorStatus, ScadaCsvConnector,
    ScadaDataPoint, SourceKind,
};

#[derive(Debug, Clone)]
pub enum ScadaSource {
    CsvDir(PathBuf),
    JsonFile(PathBuf),
    RestApi { endpoint: String },
    DatabasePlaceholder { connection_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScadaTrend {
    pub turbine_id: String,
    pub metric: String,
    pub direction: String,
    pub sample_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScadaAlarm {
    pub turbine_id: String,
    pub timestamp: String,
    pub code: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ScadaDerivedMetrics {
    pub turbine_id: String,
    pub latest_power_kw: Option<f64>,
    pub latest_wind_speed_ms: Option<f64>,
    pub power_curve_underperformance: bool,
    pub high_temperature: bool,
    pub high_vibration: bool,
}

#[derive(Debug, Clone)]
pub struct ScadaConnector {
    pub source: ScadaSource,
}

impl ScadaConnector {
    fn load_points(&self, request: &ConnectorRequest) -> Result<Vec<ScadaDataPoint>, String> {
        match &self.source {
            ScadaSource::CsvDir(path) => ScadaCsvConnector {
                data_dir: path.clone(),
            }
            .load_latest(request.turbine_id.as_deref())
            .map_err(|e| e.to_string()),
            ScadaSource::JsonFile(path) => {
                let raw = std::fs::read_to_string(path)
                    .map_err(|e| format!("read SCADA json {}: {e}", path.display()))?;
                let mut points: Vec<ScadaDataPoint> = serde_json::from_str(&raw)
                    .map_err(|e| format!("parse SCADA json {}: {e}", path.display()))?;
                if let Some(turbine_id) = request.turbine_id.as_deref() {
                    points.retain(|point| point.turbine_id == turbine_id);
                }
                points.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                points.truncate(request.limit.unwrap_or(32).max(1));
                Ok(points)
            }
            ScadaSource::RestApi { endpoint } => Err(format!(
                "REST API source is configured ({endpoint}); production HTTP client wiring is intentionally deferred"
            )),
            ScadaSource::DatabasePlaceholder { connection_name } => Err(format!(
                "database source {connection_name} requires site-specific driver configuration"
            )),
        }
    }
}

impl ConnectorTrait for ScadaConnector {
    fn name(&self) -> String {
        "scada".to_string()
    }

    fn health(&self) -> ConnectorHealth {
        match &self.source {
            ScadaSource::CsvDir(path) => {
                if path.is_dir() {
                    ConnectorHealth::healthy("SCADA CSV directory available")
                } else {
                    ConnectorHealth::unavailable(format!(
                        "SCADA CSV dir not found: {}",
                        path.display()
                    ))
                }
            }
            ScadaSource::JsonFile(path) => {
                if path.is_file() {
                    ConnectorHealth::healthy("SCADA JSON file available")
                } else {
                    ConnectorHealth::unavailable(format!(
                        "SCADA JSON file not found: {}",
                        path.display()
                    ))
                }
            }
            ScadaSource::RestApi { .. } | ScadaSource::DatabasePlaceholder { .. } => {
                ConnectorHealth {
                    status: ConnectorStatus::Degraded,
                    message: "SCADA source requires enterprise runtime configuration".to_string(),
                }
            }
        }
    }

    fn query(&self, request: ConnectorRequest) -> ConnectorResult {
        match self.load_points(&request) {
            Ok(points) => {
                let records = points
                    .iter()
                    .filter_map(|point| {
                        serde_json::to_value(point)
                            .ok()
                            .map(|payload| ConnectorRecord {
                                source: SourceKind::Scada,
                                record_type: "ScadaDataPoint".to_string(),
                                payload,
                            })
                    })
                    .collect::<Vec<_>>();
                ConnectorResult {
                    connector_name: self.name(),
                    health: self.health(),
                    data_quality: Some(assess_scada_points(&points)),
                    source_confidence: Some(confidence_for_source(SourceKind::Scada)),
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
                result.source_confidence = Some(confidence_for_source(SourceKind::Scada));
                result.warnings.push(e);
                result
            }
        }
    }
}

#[must_use]
pub fn derive_scada_metrics(points: &[ScadaDataPoint]) -> Option<ScadaDerivedMetrics> {
    let latest = points.first()?;
    Some(ScadaDerivedMetrics {
        turbine_id: latest.turbine_id.clone(),
        latest_power_kw: latest.power,
        latest_wind_speed_ms: latest.wind_speed,
        power_curve_underperformance: matches!(
            (latest.wind_speed, latest.power),
            (Some(wind), Some(power)) if wind >= 7.0 && power < 500.0
        ),
        high_temperature: latest.gearbox_oil_temp.is_some_and(|value| value >= 80.0)
            || latest.generator_temp.is_some_and(|value| value >= 95.0),
        high_vibration: latest.vibration.is_some_and(|value| value >= 8.0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn scada_json_connector_returns_uniform_records() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("scada.json");
        std::fs::write(
            &path,
            r#"[{"turbine_id":"T-01","timestamp":"2026-06-05T10:00:00Z","wind_speed":8.0,"power":420.0,"gearbox_oil_temp":82.0,"generator_temp":80.0,"vibration":9.0,"alarm_codes":["A01"]}]"#,
        )
        .unwrap();
        let connector = ScadaConnector {
            source: ScadaSource::JsonFile(path),
        };
        let result = connector.query(ConnectorRequest {
            turbine_id: Some("T-01".to_string()),
            ..ConnectorRequest::default()
        });
        assert_eq!(result.records.len(), 1);
        assert!(result.data_quality.unwrap().score > 0);
    }
}
