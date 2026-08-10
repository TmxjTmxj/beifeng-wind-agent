use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    assess_records, confidence_for_source, Connector as ConnectorTrait, ConnectorHealth,
    ConnectorRecord, ConnectorRequest, ConnectorResult, ConnectorStatus, SourceKind,
};

#[derive(Debug, Clone)]
pub enum WeatherSource {
    JsonFile(PathBuf),
    RestApi { endpoint: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeatherEvent {
    pub turbine_id: Option<String>,
    pub timestamp: String,
    pub event_type: String,
    pub severity: String,
    #[serde(default)]
    pub wind_speed: Option<f64>,
    #[serde(default)]
    pub temperature_c: Option<f64>,
    #[serde(default)]
    pub icing_mm: Option<f64>,
    #[serde(default)]
    pub lightning_distance_km: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct WeatherContext {
    pub turbine_id: Option<String>,
    pub events: Vec<WeatherEvent>,
    pub icing_risk: bool,
    pub lightning_risk: bool,
    pub yaw_risk: bool,
    pub power_curve_risk: bool,
}

#[derive(Debug, Clone)]
pub struct WeatherConnector {
    pub source: WeatherSource,
}

impl WeatherConnector {
    fn load_events(&self, request: &ConnectorRequest) -> Result<Vec<WeatherEvent>, String> {
        match &self.source {
            WeatherSource::JsonFile(path) => {
                let raw = std::fs::read_to_string(path)
                    .map_err(|e| format!("read weather json {}: {e}", path.display()))?;
                let mut events: Vec<WeatherEvent> = serde_json::from_str(&raw)
                    .map_err(|e| format!("parse weather json {}: {e}", path.display()))?;
                if let Some(turbine_id) = request.turbine_id.as_deref() {
                    events.retain(|event| {
                        event
                            .turbine_id
                            .as_deref()
                            .map(|value| value == turbine_id)
                            .unwrap_or(true)
                    });
                }
                events.truncate(request.limit.unwrap_or(32).max(1));
                Ok(events)
            }
            WeatherSource::RestApi { endpoint } => Err(format!(
                "weather REST API source is configured ({endpoint}); site API adapter is required"
            )),
        }
    }
}

impl ConnectorTrait for WeatherConnector {
    fn name(&self) -> String {
        "weather".to_string()
    }

    fn health(&self) -> ConnectorHealth {
        match &self.source {
            WeatherSource::JsonFile(path) => {
                if path.is_file() {
                    ConnectorHealth::healthy("Weather JSON file available")
                } else {
                    ConnectorHealth::unavailable(format!(
                        "Weather JSON file not found: {}",
                        path.display()
                    ))
                }
            }
            WeatherSource::RestApi { .. } => ConnectorHealth {
                status: ConnectorStatus::Degraded,
                message: "Weather source requires enterprise runtime configuration".to_string(),
            },
        }
    }

    fn query(&self, request: ConnectorRequest) -> ConnectorResult {
        match self.load_events(&request) {
            Ok(events) => {
                let context = derive_weather_context(request.turbine_id.clone(), events);
                let records = vec![ConnectorRecord {
                    source: SourceKind::Weather,
                    record_type: "WeatherContext".to_string(),
                    payload: serde_json::to_value(&context).unwrap_or(serde_json::Value::Null),
                }];
                ConnectorResult {
                    connector_name: self.name(),
                    health: self.health(),
                    data_quality: Some(assess_records(&context.events)),
                    source_confidence: Some(confidence_for_source(SourceKind::Weather)),
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
                result.source_confidence = Some(confidence_for_source(SourceKind::Weather));
                result.warnings.push(e);
                result
            }
        }
    }
}

#[must_use]
pub fn derive_weather_context(
    turbine_id: Option<String>,
    events: Vec<WeatherEvent>,
) -> WeatherContext {
    WeatherContext {
        turbine_id,
        icing_risk: events.iter().any(|event| {
            event.event_type.contains("icing") || event.icing_mm.is_some_and(|value| value >= 10.0)
        }),
        lightning_risk: events.iter().any(|event| {
            event.event_type.contains("lightning")
                || event
                    .lightning_distance_km
                    .is_some_and(|value| value <= 5.0)
        }),
        yaw_risk: events
            .iter()
            .any(|event| event.wind_speed.is_some_and(|value| value >= 20.0)),
        power_curve_risk: events.iter().any(|event| {
            event.event_type.contains("curtailment") || event.event_type.contains("storm")
        }),
        events,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weather_context_flags_icing_and_lightning() {
        let context = derive_weather_context(
            Some("T-01".to_string()),
            vec![
                WeatherEvent {
                    turbine_id: Some("T-01".to_string()),
                    timestamp: "2026-06-05T10:00:00Z".to_string(),
                    event_type: "icing".to_string(),
                    severity: "high".to_string(),
                    wind_speed: Some(12.0),
                    temperature_c: Some(-5.0),
                    icing_mm: Some(12.0),
                    lightning_distance_km: None,
                },
                WeatherEvent {
                    turbine_id: Some("T-01".to_string()),
                    timestamp: "2026-06-05T11:00:00Z".to_string(),
                    event_type: "lightning".to_string(),
                    severity: "medium".to_string(),
                    wind_speed: None,
                    temperature_c: None,
                    icing_mm: None,
                    lightning_distance_km: Some(3.0),
                },
            ],
        );
        assert!(context.icing_risk);
        assert!(context.lightning_risk);
    }
}
