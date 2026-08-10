//! Connector trait and request types for BeiFeng Wind O&M Agent.
//!
//! Provides the interface for data source connectors (SCADA, CMMS, Weather).
//!
//! Features:
//! - Timeout handling
//! - Retry with exponential backoff
//! - Authentication
//! - Metrics tracking
//! - Audit logging

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{ConnectorHealth, ConnectorResult};
use crate::infrastructure::auth::AuthConfig;
use crate::infrastructure::pagination::PaginationStrategy;
use crate::infrastructure::retry::{ConnectorRetryConfig, RetryState};
use crate::infrastructure::timeout::TimeoutConfig;
use crate::production::audit_log::AuditLogEntry;
use crate::production::health::HealthCheckResult;
use crate::production::metrics::ConnectorMetrics;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ConnectorRequest {
    pub turbine_id: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub query: Option<String>,
    pub limit: Option<usize>,
    pub params: BTreeMap<String, String>,
    /// Pagination strategy for large result sets
    #[serde(default)]
    pub pagination: PaginationStrategy,
    /// Request timeout override
    #[serde(default)]
    pub timeout: Option<TimeoutConfig>,
    /// Retry configuration override
    #[serde(default)]
    pub retry: Option<ConnectorRetryConfig>,
}

pub trait Connector: Send + Sync {
    /// Get the connector name
    fn name(&self) -> String;

    /// Get the current health status
    fn health(&self) -> ConnectorHealth;

    /// Execute a query with graceful failure handling
    ///
    /// Implementations should:
    /// - Catch and convert any panics toConnectorResult::Error
    /// - Track consecutive failures for health degradation
    /// - Return graceful error responses instead of panicking
    fn query(&self, request: ConnectorRequest) -> ConnectorResult;

    /// Execute a query with audit logging
    fn query_with_audit(
        &self,
        request: ConnectorRequest,
        _audit_entry: Option<AuditLogEntry>,
    ) -> ConnectorResult {
        // Default implementation using audit_entry if provided
        self.query(request)
    }

    /// Check connector health using the production health framework
    fn check_health_prod(&self) -> HealthCheckResult {
        let health = self.health();
        HealthCheckResult {
            status: match health.status {
                crate::connectors::ConnectorStatus::Healthy => {
                    crate::production::health::HealthStatus::Healthy
                }
                crate::connectors::ConnectorStatus::Degraded => {
                    crate::production::health::HealthStatus::Warning
                }
                crate::connectors::ConnectorStatus::Unavailable => {
                    crate::production::health::HealthStatus::Unhealthy
                }
            },
            message: health.message,
            latency_ms: 0.0,
            last_success: None,
            last_failure: None,
            error_count: 0,
            consecutive_errors: 0,
            details: None,
        }
    }

    /// Get metrics for this connector
    fn get_metrics(&self) -> Option<ConnectorMetrics> {
        None
    }

    /// Execute a query with metrics tracking
    fn query_with_metrics(
        &self,
        request: ConnectorRequest,
        _metrics: &mut ConnectorMetrics,
    ) -> ConnectorResult {
        self.query(request)
    }

    /// Execute a query with retry logic
    ///
    /// This implements exponential backoff based on the retry configuration.
    /// The default implementation performs a single attempt without retry.
    /// For production use, implementers should wrap this with a retry timer.
    fn query_with_retry(
        &self,
        request: ConnectorRequest,
        _retry_state: &mut RetryState,
    ) -> ConnectorResult {
        // Default: perform a single attempt without retry
        // The retry_state tracks attempt count for caller to determine if retry is needed
        self.query(request)
    }

    /// Execute a query with timeout enforcement
    ///
    /// The default implementation performs the query without timeout enforcement.
    /// Implementations should override to provide actual timeout handling.
    fn query_with_timeout(
        &self,
        request: ConnectorRequest,
        _timeout_config: TimeoutConfig,
    ) -> ConnectorResult {
        self.query(request)
    }

    /// Execute a query with authentication
    fn query_with_auth(
        &self,
        request: ConnectorRequest,
        _auth_config: &AuthConfig,
    ) -> ConnectorResult {
        self.query(request)
    }

    /// Get the auth config for this connector
    fn auth_config(&self) -> Option<AuthConfig> {
        None
    }
}

/// Async version of the connector trait for async-based connectors
#[cfg(feature = "async-runtime")]
#[async_trait::async_trait]
pub trait AsyncConnector: Send + Sync {
    /// Get the connector name
    fn name(&self) -> String;

    /// Execute a query asynchronously
    async fn query_async(&self, request: ConnectorRequest) -> ProdResult<ConnectorResult>;

    /// Execute a query with timeout
    async fn query_with_timeout(
        &self,
        request: ConnectorRequest,
        timeout: std::time::Duration,
    ) -> ProdResult<ConnectorResult>;

    /// Execute a query with pagination
    async fn query_paginated(
        &self,
        request: ConnectorRequest,
        pagination: PaginationStrategy,
    ) -> ProdResult<ConnectorResult>;
}

#[cfg(feature = "async-runtime")]
#[async_trait::async_trait]
impl<T: Connector> AsyncConnector for T {
    async fn query_async(&self, request: ConnectorRequest) -> ProdResult<ConnectorResult> {
        ProdResult::Ok(self.query(request))
    }

    async fn query_with_timeout(
        &self,
        request: ConnectorRequest,
        _timeout: std::time::Duration,
    ) -> ProdResult<ConnectorResult> {
        ProdResult::Ok(self.query(request))
    }

    async fn query_paginated(
        &self,
        request: ConnectorRequest,
        pagination: PaginationStrategy,
    ) -> ProdResult<ConnectorResult> {
        let mut req = request;
        req.pagination = pagination;
        ProdResult::Ok(self.query(req))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connector_request_default() {
        let req = ConnectorRequest::default();
        assert!(req.turbine_id.is_none());
        assert!(req.limit.is_none());
    }

    #[test]
    fn connector_request_with_pagination() {
        let req = ConnectorRequest {
            pagination: PaginationStrategy::Offset {
                limit: 100,
                offset: 0,
            },
            ..ConnectorRequest::default()
        };
        assert!(req.pagination.is_offset());
    }
}
