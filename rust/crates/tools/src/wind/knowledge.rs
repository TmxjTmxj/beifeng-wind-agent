use serde_json::{json, Value};

use crate::rag_service::{HttpRagService, RagService};

use super::normalize::normalize_wind_knowledge_query;
use super::WindKnowledgeQueryInput;

pub(super) fn execute_wind_knowledge_query(
    input: &WindKnowledgeQueryInput,
) -> Result<Value, String> {
    let client = crate::build_http_client()?;
    let service = HttpRagService::from_env(client)?;
    execute_wind_knowledge_query_with_service(input, &service)
}

pub(super) fn execute_wind_knowledge_query_with_service(
    input: &WindKnowledgeQueryInput,
    service: &dyn RagService,
) -> Result<Value, String> {
    let request_body = wind_knowledge_query_request(input);
    let query_url = service_query_url(service);
    let debug = input
        .debug
        .unwrap_or(false)
        .then(|| wind_knowledge_debug_payload(input, &query_url, &request_body));

    let value = service.query(&request_body).map_err(|error| {
        format!(
            "wind_knowledge_query failed: {error}. Start it with `cargo run --manifest-path .\\rust\\Cargo.toml -p claw-rag-service -- serve --db .\\beifeng\\data\\wind.sqlite`."
        )
    })?;
    let mut output = json!({
        "hits": value.get("hits").cloned().unwrap_or_else(|| json!([])),
        "graph_suggestions": value.get("graph_suggestions").cloned().unwrap_or_else(|| json!([])),
        "advice": value.get("advice").cloned().unwrap_or(Value::Null),
        "risk_assessment": value.get("risk_assessment").cloned().unwrap_or(Value::Null)
    });
    if let Some(debug) = debug {
        output["debug"] = debug;
    }
    Ok(output)
}

pub(crate) fn wind_knowledge_query_request(input: &WindKnowledgeQueryInput) -> Value {
    let normalized = normalize_wind_knowledge_query(input);
    json!({
        "query": normalized.query,
        "top_k": input.top_k.unwrap_or(8),
        "component": normalized.component,
        "domain": normalized.domain,
        "equipment": normalized.equipment,
        "symptom": normalized.symptom,
        "search_mode": "hybrid"
    })
}

fn service_query_url(service: &dyn RagService) -> reqwest::Url {
    service.query_url().unwrap_or_else(|| {
        reqwest::Url::parse("mock://claw-rag-service/v1/query").expect("static mock URL")
    })
}

fn wind_knowledge_debug_payload(
    input: &WindKnowledgeQueryInput,
    query_url: &reqwest::Url,
    request_body: &Value,
) -> Value {
    json!({
        "raw_input": {
            "query": input.query,
            "component": input.component,
            "symptom": input.symptom,
            "domain": input.domain,
            "equipment": input.equipment,
            "top_k": input.top_k
        },
        "normalized": request_body,
        "post": {
            "method": "POST",
            "path": "/v1/query",
            "url": query_url.as_str(),
            "body": request_body
        }
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::rag_service::MockRagService;

    use super::*;

    #[test]
    fn executes_with_mock_rag_service() {
        let input = WindKnowledgeQueryInput {
            query: "叶片裂纹是否需要停机".to_string(),
            component: Some("叶片".to_string()),
            symptom: Some("裂纹".to_string()),
            domain: None,
            equipment: None,
            top_k: Some(3),
            debug: Some(true),
        };
        let service = MockRagService::with_response(
            "叶片裂纹是否需要停机 Blade",
            json!({
                "hits": [{"source_path": "knowledge_base/fault_cases/blade.md"}],
                "graph_suggestions": [{"component": "Blade", "symptom": "叶片裂纹"}],
                "advice": {"should_inspect": true},
                "risk_assessment": {"risk_level": "Medium"}
            }),
        );

        let output = execute_wind_knowledge_query_with_service(&input, &service)
            .expect("mock query should succeed");

        assert_eq!(
            output["hits"][0]["source_path"],
            "knowledge_base/fault_cases/blade.md"
        );
        assert_eq!(output["graph_suggestions"][0]["component"], "Blade");
        assert_eq!(output["advice"]["should_inspect"], true);
        assert_eq!(
            output["debug"]["post"]["url"],
            "mock://claw-rag-service/v1/query"
        );
    }
}
