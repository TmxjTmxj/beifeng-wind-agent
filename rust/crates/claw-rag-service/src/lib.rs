//! Workspace RAG: ingest files → `SQLite` + embeddings, query via cosine similarity (linear scan MVP).
#![forbid(unsafe_code)]

mod advice;
mod chunk;
mod confidence;
mod config;
mod connector;
mod connectors;
mod db;
mod document;
mod embed;
mod fault_analysis;
mod graph;
mod ingest;
mod memory;
mod memory_context;
mod memory_service;
#[cfg(feature = "qdrant-index")]
mod qdrant_index;
mod quality;
mod report;
mod report_history;
mod risk;
mod search;
mod skill;

// Production hardening modules
pub mod infrastructure {
    //! Infrastructure layer for production connectors.
    //!
    //! Provides:
    //! - Authentication
    //! - Retry and timeout handling
    //! - Pagination
    //! - HTTP headers utilities

    pub mod auth;
    pub mod http_headers;
    pub mod pagination;
    pub mod retry;
    pub mod timeout;
}

pub mod production {
    //! Production-hardened modules for connectors.
    //!
    //! Provides:
    //! - Unified error model
    //! - Audit logging
    //! - Health checks
    //! - Field mapping
    //! - Metrics collection

    pub mod audit_log;
    pub mod error;
    pub mod field_mapping;
    pub mod health;
    pub mod metrics;
}

pub use advice::{generate_wind_inspection_advice, WindInspectionAdvice};
pub use confidence::{
    confidence_for_source, EvidenceConfidence, SourceConfidenceReport, SourceKind,
};
pub use config::{set_global_wind_rules_config, wind_rules_config, WindRulesConfig};
pub use connector::{ScadaCsvConnector, ScadaDataPoint};
pub use connectors::{
    derive_scada_metrics, CmmsConnector, CmmsSource, Connector, ConnectorHealth, ConnectorRecord,
    ConnectorRegistry, ConnectorRequest, ConnectorResult, ConnectorStatus, MaintenanceRecord,
    ScadaAlarm, ScadaConnector, ScadaDerivedMetrics, ScadaSource, ScadaTrend, SparePartHistory,
    WeatherConnector, WeatherContext, WeatherEvent, WeatherSource, WorkOrder,
};
pub use db::{chunk_count, document_record_count, embedding_count, open_db};
pub use document::{default_knowledge_base_path, detect_file_type, parse_document, DocumentRecord};
pub use embed::EmbedConfig;
pub use fault_analysis::{
    generate_fault_analysis_result, EvidenceSummary, FaultAnalysisInput, FaultAnalysisResult,
};
pub use graph::{
    default_graph_path, get_accompanying_symptoms, query_fault_graph_file,
    suggestions_for_multi_component_query, suggestions_for_query, walk_escalation_path,
    FaultEscalation, FaultEscalationRelation, FaultGraph, FaultGraphEntry, GraphQuery,
    GraphQueryResponse, GraphSuggestion,
};
pub use ingest::{run_ingest, IngestStats};
pub use memory::{
    new_fault_record_id, now_timestamp, summarize_recent_faults, FaultRecord, SqliteWindMemory,
    WindMemory,
};
pub use memory_context::{apply_memory_context, MemoryContext};
pub use memory_service::{
    FaultHistoryRecord, MaintenanceHistoryRecord, MemoryService, TurbineProfile,
};
pub use quality::{
    assess_records, assess_scada_points, DataQualityIssue, DataQualityIssueKind, DataQualityReport,
    DataQualitySeverity,
};
pub use report::{
    build_wind_report_markdown, default_reports_dir, generate_wind_report, WindReportGenerateInput,
    WindReportGeneration,
};
pub use report_history::ReportHistoryRecord;
pub use risk::{generate_wind_risk_assessment, WindRiskAssessment};
pub use search::query_index;
pub use skill::{
    builtin_skills, dispatch_skill, format_skill_output, SkillDefinition, SkillQueryResponse,
    SkillType,
};

// Re-export production modules for convenience
pub use infrastructure::auth::{mask_string, AuthConfig, AuthMethod};
pub use infrastructure::pagination::{PaginatedRequest, PaginationBuilder, PaginationStrategy};
pub use infrastructure::retry::{ConnectorRetryConfig, RetryState, RetryStrategy, RetryTimer};
pub use infrastructure::timeout::{DurationTimer, TimeoutConfig, TimeoutError, TimeoutResult};
pub use production::audit_log::{
    mask_headers, mask_turbine_id, AuditContext, AuditLogEntry, AuditLogWriter, AuditStatus,
    MemoryAuditLog, RequestIdGenerator,
};
pub use production::error::{
    err_auth, err_network, err_parse, err_rate_limit, err_server, err_timeout, err_unknown,
    ConnectorError, ErrorCategory, ErrorCode, ErrorInfo,
};
pub use production::field_mapping::{
    default_cf_mach_mappings, default_cmms_mappings, FieldConversion, FieldMapping,
    FieldMappingConfig, StandardField, UnitConversion,
};
pub use production::health::{
    HealthAggregator, HealthCheckHandler, HealthCheckResult, HealthConfig, HealthStatus,
    HealthTracker, SimpleHealthChecker,
};
pub use production::metrics::{
    ConnectorMetrics, FailureReason, MetricsCollector, OverallHealth, SharedMetrics, SimpleMetrics,
};

use serde::{Deserialize, Serialize};

/// One retrieved chunk for the model or UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagHit {
    pub path: String,
    pub snippet: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
    pub chunk_text: String,
    pub source_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equipment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parser_status: Option<String>,
    pub score_breakdown: ScoreBreakdown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryRequest {
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: u32,
    #[serde(default)]
    pub turbine_id: Option<String>,
    pub component: Option<String>,
    pub symptom: Option<String>,
    #[serde(default)]
    pub scada_context: Option<String>,
    pub domain: Option<String>,
    pub equipment: Option<String>,
    pub file_type: Option<String>,
    pub source_type: Option<String>,
    pub reserved_media: Option<bool>,
    #[serde(default)]
    pub search_mode: SearchMode,
}

fn default_top_k() -> u32 {
    8
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchMode {
    Vector,
    Keyword,
    #[default]
    Hybrid,
}

impl std::str::FromStr for SearchMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "vector" => Ok(Self::Vector),
            "keyword" => Ok(Self::Keyword),
            "hybrid" => Ok(Self::Hybrid),
            other => Err(format!("unsupported search mode: {other}")),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub vector_score: f32,
    pub keyword_score: f32,
    pub metadata_score: f32,
    pub final_score: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueryResponse {
    pub hits: Vec<RagHit>,
    pub graph_suggestions: Vec<GraphSuggestion>,
    pub advice: WindInspectionAdvice,
    pub risk_assessment: WindRiskAssessment,
    /// `0-stub` (legacy), `1-sqlite`, `1-sqlite-empty`, `1-sqlite-no-db`, `3-hybrid`
    pub phase: String,
    pub search_mode: SearchMode,
    pub fts5_enabled: bool,
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use reqwest::Client;
    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn query_missing_db_reports_phase() {
        let client = Client::new();
        let cfg = EmbedConfig {
            api_key: "x".into(),
            base_url: "mock://".into(),
            model: "m".into(),
        };
        let r = query_index(
            Path::new("/no/such/claw_rag.sqlite"),
            &client,
            &cfg,
            &QueryRequest {
                query: "hello".into(),
                top_k: 3,
                turbine_id: None,
                component: None,
                symptom: None,
                scada_context: None,
                domain: None,
                equipment: None,
                file_type: None,
                source_type: None,
                reserved_media: None,
                search_mode: SearchMode::Hybrid,
            },
        )
        .await
        .unwrap();
        assert_eq!(r.phase, "1-sqlite-no-db");
    }

    #[tokio::test]
    async fn ingest_and_query_roundtrip_mock() {
        std::env::set_var("CLAW_RAG_MOCK_PROVIDERS", "1");
        let dir = tempdir().unwrap();
        let ws1 = dir.path().join("ws1");
        let ws2 = dir.path().join("ws2");
        std::fs::create_dir_all(&ws1).unwrap();
        std::fs::create_dir_all(&ws2).unwrap();
        std::fs::write(ws1.join("note.md"), "hello RAG service test content").unwrap();
        std::fs::write(
            ws1.join("gearbox_note.txt"),
            "gearbox bearing vibration inspection",
        )
        .unwrap();
        std::fs::write(
            ws1.join("scada_power_curve.csv"),
            "wind_speed,power\n8.2,1500\n",
        )
        .unwrap();
        std::fs::write(ws1.join("uav_blade.jpg"), [0_u8, 1, 2, 3]).unwrap();
        std::fs::write(ws2.join("docs.md"), "secondary repo doc about embeddings").unwrap();
        let db = dir.path().join("idx.sqlite");
        let client = Client::new();
        let cfg = EmbedConfig::mock_from_env().expect("mock");
        let st = run_ingest(&[ws1.clone(), ws2.clone()], &db, &cfg, &client)
            .await
            .unwrap();
        assert!(st.embeddings_written >= 1);

        let r = query_index(
            &db,
            &client,
            &cfg,
            &QueryRequest {
                query: "RAG service".into(),
                top_k: 4,
                turbine_id: None,
                component: None,
                symptom: None,
                scada_context: None,
                domain: None,
                equipment: None,
                file_type: None,
                source_type: None,
                reserved_media: None,
                search_mode: SearchMode::Hybrid,
            },
        )
        .await
        .unwrap();
        assert_eq!(r.phase, "3-hybrid");
        assert!(!r.hits.is_empty());
        assert!(r.hits.iter().all(|h| h.path.contains(':')));
        assert!(r.hits[0].score_breakdown.final_score >= 0.0);

        let conn = open_db(&db).unwrap();
        let records: i64 = conn
            .query_row("SELECT COUNT(*) FROM document_records", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(records, 5);
        let media_status: String = conn
            .query_row(
                "SELECT parser_status FROM document_records WHERE file_type='jpg'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(media_status, "reserved_media_only");
        std::env::remove_var("CLAW_RAG_MOCK_PROVIDERS");
    }

    #[tokio::test]
    async fn hybrid_query_applies_metadata_filters() {
        std::env::set_var("CLAW_RAG_MOCK_PROVIDERS", "1");
        let dir = tempdir().unwrap();
        let kb = dir.path().join("knowledge_base");
        std::fs::create_dir_all(kb.join("fault_cases")).unwrap();
        std::fs::create_dir_all(kb.join("manuals")).unwrap();
        std::fs::write(
            kb.join("fault_cases").join("blade_crack_case.md"),
            "blade crack leading edge inspection repeat check",
        )
        .unwrap();
        std::fs::write(
            kb.join("manuals").join("gearbox_temp_manual.md"),
            "gearbox oil temperature bearing inspection",
        )
        .unwrap();
        let db = dir.path().join("idx.sqlite");
        let client = Client::new();
        let cfg = EmbedConfig::mock_from_env().expect("mock");
        run_ingest(std::slice::from_ref(&kb), &db, &cfg, &client)
            .await
            .unwrap();

        let r = query_index(
            &db,
            &client,
            &cfg,
            &QueryRequest {
                query: "blade crack inspection".into(),
                top_k: 5,
                turbine_id: None,
                component: None,
                symptom: None,
                scada_context: None,
                domain: Some("Blade".into()),
                equipment: Some("blade".into()),
                file_type: None,
                source_type: None,
                reserved_media: None,
                search_mode: SearchMode::Hybrid,
            },
        )
        .await
        .unwrap();

        assert!(!r.hits.is_empty());
        assert!(r
            .hits
            .iter()
            .all(|hit| hit.domain.as_deref() == Some("Blade")));
        assert!(r.hits[0].score_breakdown.metadata_score > 0.0);
        std::env::remove_var("CLAW_RAG_MOCK_PROVIDERS");
    }

    // New tests for production hardening features
    #[test]
    fn production_auth_method_creation() {
        let api_key = infrastructure::auth::AuthMethod::api_key("test-key");
        assert!(api_key.header_name().is_some());

        let bearer = infrastructure::auth::AuthMethod::bearer_token("token");
        assert!(bearer.header_name().is_some());

        let basic = infrastructure::auth::AuthMethod::basic("user", "pass");
        assert!(basic.header_name().is_some());
        assert!(basic.header_value().is_some());
    }

    #[test]
    fn production_pagination_strategies() {
        use infrastructure::pagination::PaginationStrategy;

        let offset = PaginationStrategy::Offset {
            limit: 50,
            offset: 0,
        };
        assert!(offset.is_offset());
        assert!(!offset.is_cursor());

        let cursor = PaginationStrategy::Cursor {
            limit: 50,
            after: Some("cursor123".to_string()),
            before: None,
        };
        assert!(cursor.is_cursor());
        assert!(!cursor.is_offset());
    }

    #[test]
    fn production_error_types() {
        use production::error::{ConnectorError, ErrorCategory, ErrorCode};

        let auth_err = ConnectorError::auth(ErrorCode::InvalidApiKey, "invalid key");
        assert_eq!(auth_err.category(), ErrorCategory::Auth);
        assert!(auth_err.is_auth_error());

        let timeout_err = ConnectorError::timeout(ErrorCode::ConnectTimeout, "timed out");
        assert_eq!(timeout_err.category(), ErrorCategory::Timeout);
        assert!(timeout_err.is_timeout());
    }

    #[test]
    fn production_field_mapping() {
        use production::field_mapping::{FieldMappingConfig, StandardField};

        let mut config = FieldMappingConfig::new();
        config.add_mapping("WT_ID", StandardField::TurbineId);

        assert!(config.is_mapped("WT_ID"));
        assert_eq!(config.map_field("WT_ID"), Some(StandardField::TurbineId));
    }
}
