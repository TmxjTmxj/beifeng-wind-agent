use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use claw_rag_service::{
    apply_memory_context, chunk_count, dispatch_skill, document_record_count, embedding_count,
    format_skill_output, generate_fault_analysis_result, new_fault_record_id, now_timestamp,
    open_db, query_index, summarize_recent_faults, FaultAnalysisInput, FaultHistoryRecord,
    FaultRecord, MaintenanceHistoryRecord, QueryRequest, QueryResponse, SearchMode,
    SkillQueryResponse, WindMemory,
};
use serde::Deserialize;

use crate::state::AppState;

/// HTTP request body for `/v1/skill-query`.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SkillQueryHttpRequest {
    pub query: String,
    pub turbine_id: Option<String>,
    pub component: Option<String>,
    pub symptom: Option<String>,
    pub scada_context: Option<String>,
    pub mode: Option<SearchMode>,
    pub top_k: Option<u32>,
}

/// Single-page UI for phase 3 (served at `GET /`).
pub(crate) static INDEX_HTML: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/index.html"));

async fn ui_index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

pub(crate) fn rag_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(ui_index))
        .route("/health", get(|| async { "ok" }))
        .route("/v1/stats", get(stats))
        .route("/v1/query", post(query))
        .route("/v1/skill-query", post(skill_query))
        .with_state(state)
}

async fn stats(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, StatusCode> {
    let path = state.db_path.clone();
    if !path.is_file() {
        return Ok(Json(serde_json::json!({
            "chunks": 0,
            "phase": "1-sqlite-no-db"
        })));
    }
    let res = tokio::task::spawn_blocking(move || {
        let conn = open_db(&path).map_err(|_| ())?;
        let chunks = chunk_count(&conn).map_err(|_| ())?;
        let embeddings = embedding_count(&conn).map_err(|_| ())?;
        let document_records = document_record_count(&conn).map_err(|_| ())?;
        Ok::<_, ()>((chunks, embeddings, document_records))
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|()| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({
        "chunks": res.0,
        "embeddings": res.1,
        "document_records": res.2,
        "phase": "1-sqlite"
    })))
}

async fn skill_query(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SkillQueryHttpRequest>,
) -> Result<Json<SkillQueryResponse>, (StatusCode, String)> {
    let skill_type = dispatch_skill(&req.query, req.component.as_deref());
    let query_req = QueryRequest {
        query: req.query.clone(),
        top_k: req.top_k.unwrap_or(8),
        turbine_id: req.turbine_id.clone(),
        component: req.component.clone(),
        symptom: req.symptom.clone(),
        scada_context: req.scada_context.clone(),
        domain: req.component.clone(),
        equipment: None,
        file_type: None,
        source_type: None,
        reserved_media: None,
        search_mode: req.mode.unwrap_or_default(),
    };
    let mut response = query_index(&state.db_path, &state.client, &state.cfg, &query_req)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    enrich_response_with_memory(state.as_ref(), &query_req, &mut response);

    let input = FaultAnalysisInput {
        problem: req.query.clone(),
        component: req.component.clone(),
        symptom: req.symptom.clone(),
    };
    let fault_analysis = generate_fault_analysis_result(&input, &response);
    let skill_output = format_skill_output(
        &skill_type,
        &fault_analysis,
        &response.advice,
        &response.risk_assessment,
        &response.graph_suggestions,
    );
    Ok(Json(SkillQueryResponse {
        skill_name: skill_type.to_string(),
        query: req.query,
        component: input.component,
        fault_analysis,
        skill_output,
    }))
}

async fn query(
    State(state): State<Arc<AppState>>,
    Json(req): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, (StatusCode, String)> {
    let mut response = query_index(&state.db_path, &state.client, &state.cfg, &req)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    enrich_response_with_memory(state.as_ref(), &req, &mut response);
    Ok(Json(response))
}

fn enrich_response_with_memory(state: &AppState, req: &QueryRequest, response: &mut QueryResponse) {
    let component = response_component(req, response);
    let symptom = response_symptom(req, response);

    match state.json_memory.load_turbine_history(
        req.turbine_id.as_deref(),
        Some(component.as_str()),
        Some(symptom.as_str()),
    ) {
        Ok(context) => apply_memory_context(response, &context),
        Err(e) => eprintln!("memory-json: skip history load: {e}"),
    }

    if component != "Unknown" {
        match state.memory.query_recent_faults(&component, 5) {
            Ok(records) => {
                if let Some(summary) = summarize_recent_faults(&records) {
                    response.advice.add_additional_context(summary);
                }
            }
            Err(e) => eprintln!("memory: skip recent faults: {e}"),
        }
    }

    let record = FaultRecord {
        id: new_fault_record_id(),
        turbine_id: req.turbine_id.clone(),
        component: component.clone(),
        symptom: symptom.clone(),
        risk_level: response.risk_assessment.risk_level.clone(),
        created_at: now_timestamp(),
        query: req.query.clone(),
    };
    if let Err(e) = state.memory.record_fault_query(&record) {
        eprintln!("memory: skip record fault query: {e}");
    }

    let turbine_id = req
        .turbine_id
        .clone()
        .unwrap_or_else(|| "UNKNOWN".to_string());
    let json_fault = FaultHistoryRecord {
        fault_id: record.id.clone(),
        turbine_id: turbine_id.clone(),
        component: component.clone(),
        symptom,
        risk_level: response.risk_assessment.risk_level.clone(),
        created_at: record.created_at.clone(),
        query: Some(req.query.clone()),
        notes: Some("auto-recorded by memory runtime".to_string()),
    };
    if let Err(e) = state.json_memory.append_fault_record(&json_fault) {
        eprintln!("memory-json: skip append fault: {e}");
    }
    if let Some(action) = response.advice.maintenance_actions.first() {
        let maintenance = MaintenanceHistoryRecord {
            record_id: format!("maintenance-{}", now_timestamp()),
            turbine_id,
            component,
            maintenance_action: action.clone(),
            performed_at: now_timestamp(),
            notes: Some("auto-recorded recommended action by memory runtime".to_string()),
        };
        if let Err(e) = state.json_memory.append_maintenance_record(&maintenance) {
            eprintln!("memory-json: skip append maintenance: {e}");
        }
    }
}

fn response_component(req: &QueryRequest, response: &QueryResponse) -> String {
    req.component
        .clone()
        .or_else(|| {
            response
                .graph_suggestions
                .first()
                .map(|suggestion| suggestion.component.clone())
        })
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn response_symptom(req: &QueryRequest, response: &QueryResponse) -> String {
    req.symptom
        .clone()
        .or_else(|| {
            response
                .graph_suggestions
                .first()
                .map(|suggestion| suggestion.symptom.clone())
        })
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| req.query.clone())
}

#[cfg(test)]
mod tests {
    use super::INDEX_HTML;

    #[test]
    fn index_html_wires_api_paths() {
        assert!(INDEX_HTML.contains("/v1/stats"));
        assert!(INDEX_HTML.contains("/v1/query"));
    }

    #[test]
    fn skill_query_http_request_deserializes() {
        let json = r#"{"query":"叶片裂纹","component":"Blade"}"#;
        let req: super::SkillQueryHttpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.query, "叶片裂纹");
        assert_eq!(req.component.as_deref(), Some("Blade"));
        assert!(req.symptom.is_none());
    }
}
