//! Testing utilities for Redis Enterprise API client consumers
//!
//! This module provides a complete testing infrastructure for applications that use
//! the redis-enterprise client library. It includes:
//!
//! - **Mock server**: A pre-configured wiremock server for simulating the Enterprise API
//! - **Fixtures**: Builder-pattern fixtures for creating test data
//! - **Response helpers**: Convenience functions for building common HTTP responses
//!
//! # Feature Flag
//!
//! This module is only available when the `test-support` feature is enabled:
//!
//! ```toml
//! [dev-dependencies]
//! redis-enterprise = { version = "0.8", features = ["test-support"] }
//! ```
//!
//! # Quick Start
//!
//! ```ignore
//! use redis_enterprise::testing::{MockEnterpriseServer, fixtures, responses};
//!
//! #[tokio::test]
//! async fn test_my_app() {
//!     // Start a mock server
//!     let server = MockEnterpriseServer::start().await;
//!
//!     // Set up expected responses using fixtures
//!     server.mock_databases_list(vec![
//!         fixtures::DatabaseFixture::new(1, "cache").build(),
//!         fixtures::DatabaseFixture::new(2, "sessions").memory_size(2_000_000_000).build(),
//!     ]).await;
//!
//!     // Get a client configured to use the mock
//!     let client = server.client();
//!
//!     // Test your application code
//!     let dbs = client.databases().list().await.unwrap();
//!     assert_eq!(dbs.len(), 2);
//! }
//! ```
//!
//! # Testing Error Scenarios
//!
//! ```ignore
//! use redis_enterprise::testing::{MockEnterpriseServer, responses};
//! use redis_enterprise::RestError;
//!
//! #[tokio::test]
//! async fn test_not_found_error() {
//!     let server = MockEnterpriseServer::start().await;
//!
//!     // Mock a 404 response
//!     server.mock_path("GET", "/v1/bdbs/999", responses::not_found("Database not found")).await;
//!
//!     let client = server.client();
//!     let result = client.databases().get(999).await;
//!
//!     assert!(matches!(result, Err(RestError::NotFound)));
//! }
//! ```
//!
//! # Custom Mocking
//!
//! For advanced scenarios, access the underlying wiremock server directly:
//!
//! ```ignore
//! use redis_enterprise::testing::MockEnterpriseServer;
//! use wiremock::{Mock, matchers::{method, path, body_json}, ResponseTemplate};
//!
//! #[tokio::test]
//! async fn test_custom_scenario() {
//!     let server = MockEnterpriseServer::start().await;
//!
//!     // Create a custom mock with request body matching
//!     Mock::given(method("POST"))
//!         .and(path("/v1/bdbs"))
//!         .and(body_json(serde_json::json!({
//!             "name": "new-db",
//!             "memory_size": 1000000000
//!         })))
//!         .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
//!             "uid": 1,
//!             "name": "new-db"
//!         })))
//!         .mount(server.inner())
//!         .await;
//! }
//! ```

pub mod fixtures;
pub mod responses;
pub mod server;

// Re-export main types for convenience
pub use fixtures::{
    ActionFixture, ClusterFixture, DatabaseFixture, LicenseFixture, NodeFixture, UserFixture,
};
pub use server::MockEnterpriseServer;

// Re-export wiremock types that consumers will commonly need
pub use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_json, method, path, path_regex},
};
