use serde::{Deserialize, Serialize};

use crate::{DataQualityReport, SourceConfidenceReport, SourceKind};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConnectorStatus {
    Healthy,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConnectorHealth {
    pub status: ConnectorStatus,
    pub message: String,
}

impl ConnectorHealth {
    #[must_use]
    pub fn healthy(message: impl Into<String>) -> Self {
        Self {
            status: ConnectorStatus::Healthy,
            message: message.into(),
        }
    }

    #[must_use]
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self {
            status: ConnectorStatus::Unavailable,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConnectorRecord {
    pub source: SourceKind,
    pub record_type: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConnectorResult {
    pub connector_name: String,
    pub health: ConnectorHealth,
    pub records: Vec<ConnectorRecord>,
    pub data_quality: Option<DataQualityReport>,
    pub source_confidence: Option<SourceConfidenceReport>,
    pub warnings: Vec<String>,
}

impl ConnectorResult {
    #[must_use]
    pub fn empty(connector_name: impl Into<String>, health: ConnectorHealth) -> Self {
        Self {
            connector_name: connector_name.into(),
            health,
            records: Vec::new(),
            data_quality: None,
            source_confidence: None,
            warnings: Vec::new(),
        }
    }
}
