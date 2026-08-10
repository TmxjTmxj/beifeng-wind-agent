use std::collections::HashMap;
use std::path::Path;

use reqwest::StatusCode;
use serde_json::{json, Value};

pub trait RagService: Send + Sync {
    fn query(&self, req: &Value) -> Result<Value, String>;
    fn ingest(&self, path: &Path) -> Result<Value, String>;
    fn health(&self) -> Result<bool, String>;
    fn query_url(&self) -> Option<reqwest::Url> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct HttpRagService {
    base_url: reqwest::Url,
    client: reqwest::blocking::Client,
}

impl HttpRagService {
    pub fn new(base_url: reqwest::Url, client: reqwest::blocking::Client) -> Self {
        Self { base_url, client }
    }

    pub fn from_env(client: reqwest::blocking::Client) -> Result<Self, String> {
        Ok(Self::new(rag_service_base_url_from_env()?, client))
    }

    pub fn base_url(&self) -> &reqwest::Url {
        &self.base_url
    }

    fn endpoint(&self, path: &str) -> Result<reqwest::Url, String> {
        self.base_url
            .join(path)
            .map_err(|error| format!("invalid claw-rag-service URL for {path}: {error}"))
    }
}

impl RagService for HttpRagService {
    fn query(&self, req: &Value) -> Result<Value, String> {
        let query_url = self.endpoint("v1/query")?;
        let response = self
            .client
            .post(query_url.clone())
            .json(req)
            .send()
            .map_err(|error| {
                format!(
                    "claw-rag-service is not reachable at {}. Start it with `cargo run --manifest-path .\\rust\\Cargo.toml -p claw-rag-service -- serve --db .\\beifeng\\data\\wind.sqlite`. Details: {error}",
                    self.base_url.as_str().trim_end_matches('/')
                )
            })?;

        let status = response.status();
        let body = response.text().map_err(|error| error.to_string())?;
        if !status.is_success() {
            if matches!(
                status,
                StatusCode::BAD_GATEWAY
                    | StatusCode::SERVICE_UNAVAILABLE
                    | StatusCode::GATEWAY_TIMEOUT
            ) {
                return Err(format!(
                    "claw-rag-service is not reachable at {} (HTTP {} from {}): {}",
                    self.base_url.as_str().trim_end_matches('/'),
                    status.as_u16(),
                    query_url,
                    body
                ));
            }
            return Err(format!(
                "claw-rag-service returned HTTP {} from {}: {}",
                status.as_u16(),
                query_url,
                body
            ));
        }
        serde_json::from_str(&body)
            .map_err(|error| format!("claw-rag-service returned invalid JSON: {error}"))
    }

    fn ingest(&self, _path: &Path) -> Result<Value, String> {
        Err("HTTP RAG ingest is not exposed by claw-rag-service; use the ingest CLI command".into())
    }

    fn health(&self) -> Result<bool, String> {
        let url = self.endpoint("health")?;
        let response = self
            .client
            .get(url)
            .send()
            .map_err(|error| error.to_string())?;
        Ok(response.status().is_success())
    }

    fn query_url(&self) -> Option<reqwest::Url> {
        self.endpoint("v1/query").ok()
    }
}

#[derive(Debug, Clone, Default)]
pub struct MockRagService {
    preset_responses: HashMap<String, Value>,
}

impl MockRagService {
    pub fn new(preset_responses: HashMap<String, Value>) -> Self {
        Self { preset_responses }
    }

    pub fn with_response(query: impl Into<String>, response: Value) -> Self {
        let mut preset_responses = HashMap::new();
        preset_responses.insert(query.into(), response);
        Self { preset_responses }
    }
}

impl RagService for MockRagService {
    fn query(&self, req: &Value) -> Result<Value, String> {
        let query = req.get("query").and_then(Value::as_str).unwrap_or_default();
        Ok(self
            .preset_responses
            .get(query)
            .cloned()
            .unwrap_or_else(|| json!({"hits": [], "graph_suggestions": [], "advice": null, "risk_assessment": null})))
    }

    fn ingest(&self, path: &Path) -> Result<Value, String> {
        Ok(json!({"mock": true, "path": path.to_string_lossy()}))
    }

    fn health(&self) -> Result<bool, String> {
        Ok(true)
    }

    fn query_url(&self) -> Option<reqwest::Url> {
        reqwest::Url::parse("mock://claw-rag-service/v1/query").ok()
    }
}

pub fn rag_service_base_url_from_env() -> Result<reqwest::Url, String> {
    let raw = std::env::var("CLAW_RAG_SERVICE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8787".to_string());
    let normalized = if raw.ends_with('/') {
        raw
    } else {
        format!("{raw}/")
    };
    reqwest::Url::parse(&normalized)
        .map_err(|error| format!("invalid CLAW_RAG_SERVICE_URL `{normalized}`: {error}"))
}
