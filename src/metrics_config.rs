//! Cluster-wide v2 metrics stream engine configuration.

use crate::client::RestClient;
use crate::error::{RestError, Result};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// The complete cluster-wide v2 metrics stream engine configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Whether key-size and key-item distribution histograms are enabled.
    pub key_distribution_enabled: bool,
    /// Comma-separated key-size histogram bucket boundaries.
    pub key_size_buckets: String,
    /// Comma-separated key-item-count histogram bucket boundaries.
    pub key_items_buckets: String,
    /// Maximum on-node metrics storage size in megabytes.
    pub local_storage_max_size_mb: u64,
    /// Number of days metrics are retained in on-node storage.
    pub local_storage_retention_days: u64,
    /// Whether database tags are exported through the `db_tags` metric.
    pub expose_db_tags: bool,
    /// Database tag keys that may be exported as metric labels.
    pub metrics_tag_keys_exposed: Vec<String>,
    /// Maximum number of metrics requests processed concurrently.
    pub max_requests_in_flight: u64,
}

/// Partial update for the cluster-wide metrics configuration.
///
/// The API requires at least one field. Omitted fields retain their current
/// values, while a supplied list replaces the complete stored list.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, TypedBuilder)]
pub struct MetricsConfigUpdate {
    /// Enable or disable key distribution histograms.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub key_distribution_enabled: Option<bool>,
    /// Replace the key-size histogram bucket boundaries.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub key_size_buckets: Option<String>,
    /// Replace the key-item-count histogram bucket boundaries.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub key_items_buckets: Option<String>,
    /// Set the maximum on-node metrics storage size in megabytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub local_storage_max_size_mb: Option<u64>,
    /// Set the number of days metrics are retained in on-node storage.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub local_storage_retention_days: Option<u64>,
    /// Enable or disable exporting database tags through metrics.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub expose_db_tags: Option<bool>,
    /// Replace the database tag keys eligible for export as labels.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub metrics_tag_keys_exposed: Option<Vec<String>>,
    /// Set the maximum number of metrics requests processed concurrently.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub max_requests_in_flight: Option<u64>,
}

impl MetricsConfigUpdate {
    /// Returns `true` when the update contains no recognized fields.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.key_distribution_enabled.is_none()
            && self.key_size_buckets.is_none()
            && self.key_items_buckets.is_none()
            && self.local_storage_max_size_mb.is_none()
            && self.local_storage_retention_days.is_none()
            && self.expose_db_tags.is_none()
            && self.metrics_tag_keys_exposed.is_none()
            && self.max_requests_in_flight.is_none()
    }
}

/// Handler for cluster-wide metrics configuration operations.
pub struct MetricsConfigHandler {
    client: RestClient,
}

impl MetricsConfigHandler {
    /// Create a handler bound to the given REST client.
    pub fn new(client: RestClient) -> Self {
        Self { client }
    }

    /// Get the complete metrics configuration.
    ///
    /// Calls `GET /v1/metrics_config`.
    pub async fn get(&self) -> Result<MetricsConfig> {
        self.client.get("/v1/metrics_config").await
    }

    /// Partially update the metrics configuration and return the complete result.
    ///
    /// Calls `PUT /v1/metrics_config`. The server rejects an empty update.
    pub async fn update(&self, update: MetricsConfigUpdate) -> Result<MetricsConfig> {
        if update.is_empty() {
            return Err(RestError::ValidationError(
                "metrics configuration update requires at least one field".to_string(),
            ));
        }
        self.client.put("/v1/metrics_config", &update).await
    }
}
