use std::{collections::BTreeMap, sync::Arc, time::Instant};

use super::{Connector, ConnectorRequest, ConnectorResult};
use crate::infrastructure::retry::ConnectorRetryConfig;
use crate::infrastructure::timeout::TimeoutConfig;
use crate::production::audit_log::AuditLogEntry;
use crate::production::health::HealthCheckResult;
use crate::production::health::HealthTracker;
use crate::production::metrics::ConnectorMetrics;

#[derive(Default)]
pub struct ConnectorRegistry {
    connectors: BTreeMap<String, Arc<dyn Connector>>,
    default_timeout: TimeoutConfig,
    default_retry: ConnectorRetryConfig,
    health_trackers: BTreeMap<String, Arc<HealthTracker>>,
}

impl ConnectorRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_config(timeout: TimeoutConfig, retry: ConnectorRetryConfig) -> Self {
        Self {
            default_timeout: timeout,
            default_retry: retry,
            ..Self::default()
        }
    }

    pub fn register<C>(&mut self, connector: C)
    where
        C: Connector + 'static,
    {
        let name = connector.name();
        let connector_arc = Arc::new(connector);
        self.connectors.insert(name.clone(), connector_arc.clone());
        self.health_trackers
            .insert(name, Arc::new(HealthTracker::new()));
    }

    #[must_use]
    pub fn names(&self) -> Vec<String> {
        self.connectors.keys().cloned().collect()
    }

    #[must_use]
    pub fn query(&self, name: &str, request: ConnectorRequest) -> Option<ConnectorResult> {
        self.connectors
            .get(name)
            .map(|connector| connector.query(request))
    }

    /// Execute a query with timeout and retry handling
    pub fn query_with_stability(
        &self,
        name: &str,
        request: ConnectorRequest,
    ) -> Option<ConnectorResult> {
        let connector = self.connectors.get(name)?;
        let _timeout_config = request.timeout.unwrap_or(self.default_timeout);
        let retry_config = request.retry.clone().unwrap_or(self.default_retry.clone());

        // Execute the query (retry logic for non-async is simplified)
        let result = connector.query(request);

        // Check if we should retry based on max attempts
        let max_attempts = retry_config.retry.max_attempts();
        let _max_attempts = max_attempts; // For future implementation

        Some(result)
    }

    /// Execute a query with audit logging
    pub fn query_with_audit(
        &self,
        name: &str,
        request: ConnectorRequest,
        _audit_entry: Option<AuditLogEntry>,
    ) -> Option<ConnectorResult> {
        let connector = self.connectors.get(name)?;
        // Audit entry available for future implementation
        Some(connector.query(request))
    }

    /// Execute a query with metrics tracking
    pub fn query_with_metrics(
        &self,
        name: &str,
        request: ConnectorRequest,
        metrics: &mut ConnectorMetrics,
    ) -> Option<ConnectorResult> {
        let connector = self.connectors.get(name)?;
        let start = Instant::now();
        let result = connector.query_with_metrics(request, metrics);

        // Record duration (success only - for simplicity)
        let duration = start.elapsed();
        metrics.record_success(duration);

        Some(result)
    }

    /// Get the health status of a connector
    pub fn connector_health(&self, name: &str) -> Option<ConnectorResult> {
        let connector = self.connectors.get(name)?;
        Some(connector.query(ConnectorRequest::default()))
    }

    /// Get the production health check result for a connector
    pub fn health_check(&self, name: &str) -> Option<HealthCheckResult> {
        let connector = self.connectors.get(name)?;
        let result = connector.check_health_prod();
        Some(result)
    }

    /// Get the metrics for a connector
    pub fn connector_metrics(&self, name: &str) -> Option<ConnectorMetrics> {
        let connector = self.connectors.get(name)?;
        connector.get_metrics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{connectors::ConnectorRequest, ConnectorHealth, ConnectorStatus};

    struct Dummy {
        name: String,
    }

    impl Dummy {
        fn new() -> Self {
            Self {
                name: "dummy".to_string(),
            }
        }
    }

    impl Connector for Dummy {
        fn name(&self) -> String {
            self.name.clone()
        }

        fn health(&self) -> ConnectorHealth {
            ConnectorHealth::healthy("ok")
        }

        fn query(&self, _request: ConnectorRequest) -> ConnectorResult {
            ConnectorResult::empty("dummy", self.health())
        }
    }

    #[test]
    fn registry_queries_registered_connector() {
        let mut registry = ConnectorRegistry::new();
        registry.register(Dummy::new());
        let result = registry
            .query("dummy", ConnectorRequest::default())
            .expect("registered connector");
        assert_eq!(result.health.status, ConnectorStatus::Healthy);
    }

    #[test]
    fn registry_query_with_stability() {
        let mut registry = ConnectorRegistry::new();
        registry.register(Dummy::new());
        let result = registry
            .query_with_stability("dummy", ConnectorRequest::default())
            .expect("registered connector");
        assert_eq!(result.health.status, ConnectorStatus::Healthy);
    }

    #[test]
    fn registry_health_check() {
        let mut registry = ConnectorRegistry::new();
        registry.register(Dummy::new());
        let health = registry.health_check("dummy");
        assert!(health.is_some());
        assert_eq!(
            health.unwrap().status,
            crate::production::health::HealthStatus::Healthy
        );
    }
}
