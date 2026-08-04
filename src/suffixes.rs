//! Database name suffix management
//!
//! ## Overview
//! - Configure database suffixes
//! - Manage suffix rules
//! - Query suffix usage

use crate::client::RestClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// DNS suffix configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suffix {
    /// Unique name identifier for the DNS suffix
    pub name: String,
    /// The DNS suffix string to be used for database endpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_suffix: Option<String>,
    /// Whether to use internal addresses for this suffix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_internal_addr: Option<bool>,
    /// Whether to use external addresses for this suffix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_external_addr: Option<bool>,
}

/// Create suffix request
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct CreateSuffixRequest {
    /// Unique name identifier for the DNS suffix
    #[builder(setter(into))]
    pub name: String,
    /// The DNS suffix string to be used for database endpoints
    #[builder(setter(into))]
    pub dns_suffix: String,
    /// Whether to use internal addresses for this suffix
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub use_internal_addr: Option<bool>,
    /// Whether to use external addresses for this suffix
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub use_external_addr: Option<bool>,
}

/// Suffixes handler
pub struct SuffixesHandler {
    client: RestClient,
}

impl SuffixesHandler {
    /// Create a new handler bound to the given REST client.
    pub fn new(client: RestClient) -> Self {
        SuffixesHandler { client }
    }

    /// List all DNS suffixes
    pub async fn list(&self) -> Result<Vec<Suffix>> {
        self.client.get("/v1/suffixes").await
    }

    /// Get specific suffix
    pub async fn get(&self, name: &str) -> Result<Suffix> {
        self.client.get(&format!("/v1/suffix/{}", name)).await
    }

    /// Create a new suffix
    pub async fn create(&self, request: CreateSuffixRequest) -> Result<Suffix> {
        self.client.post("/v1/suffix", &request).await
    }

    /// Update a suffix with the method registered by Redis Software 8.0+.
    ///
    /// Redis Software 7.x does not register a suffix update method. The old
    /// SDK implementation sent `PUT`, which is not registered by any supported
    /// family; current 8.x releases register `PATCH`.
    pub async fn update(&self, name: &str, request: CreateSuffixRequest) -> Result<Suffix> {
        let body = serde_json::to_value(request)?;
        let value = self
            .client
            .patch_raw(&format!("/v1/suffix/{}", name), body)
            .await?;
        serde_json::from_value(value).map_err(Into::into)
    }

    /// Delete a suffix
    pub async fn delete(&self, name: &str) -> Result<()> {
        self.client.delete(&format!("/v1/suffix/{}", name)).await
    }

    /// Get cluster DNS suffixes configuration
    pub async fn cluster_suffixes(&self) -> Result<Vec<Suffix>> {
        self.client.get("/v1/cluster/suffixes").await
    }
}
