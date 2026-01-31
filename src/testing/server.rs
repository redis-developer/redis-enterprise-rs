//! Mock server wrapper for testing Redis Enterprise API clients
//!
//! # Example
//!
//! ```ignore
//! use redis_enterprise::testing::MockEnterpriseServer;
//! use redis_enterprise::testing::fixtures::DatabaseFixture;
//! use redis_enterprise::testing::responses;
//!
//! #[tokio::test]
//! async fn test_my_app() {
//!     let server = MockEnterpriseServer::start().await;
//!
//!     // Set up mock responses
//!     server.mock_databases_list(vec![
//!         DatabaseFixture::new(1, "cache").build(),
//!         DatabaseFixture::new(2, "sessions").build(),
//!     ]).await;
//!
//!     // Create a client pointing to the mock
//!     let client = server.client();
//!
//!     // Test your application code
//!     let dbs = client.databases().list().await.unwrap();
//!     assert_eq!(dbs.len(), 2);
//! }
//! ```

use crate::EnterpriseClient;
use serde_json::Value;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// A wrapper around wiremock's MockServer configured for Redis Enterprise API testing
pub struct MockEnterpriseServer {
    server: MockServer,
}

impl MockEnterpriseServer {
    /// Start a new mock server
    pub async fn start() -> Self {
        Self {
            server: MockServer::start().await,
        }
    }

    /// Get the base URI of the mock server
    pub fn uri(&self) -> String {
        self.server.uri()
    }

    /// Create an EnterpriseClient configured to use this mock server
    pub fn client(&self) -> EnterpriseClient {
        EnterpriseClient::builder()
            .base_url(self.uri())
            .username("test@example.com")
            .password("password")
            .insecure(true)
            .build()
            .expect("Failed to build test client")
    }

    /// Get a reference to the underlying MockServer for custom mocking
    pub fn inner(&self) -> &MockServer {
        &self.server
    }

    // Database mocks

    /// Mock GET /v1/bdbs to return a list of databases
    pub async fn mock_databases_list(&self, databases: Vec<Value>) {
        Mock::given(method("GET"))
            .and(path("/v1/bdbs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(databases))
            .mount(&self.server)
            .await;
    }

    /// Mock GET /v1/bdbs/{uid} to return a specific database
    pub async fn mock_database_get(&self, uid: u32, database: Value) {
        Mock::given(method("GET"))
            .and(path(format!("/v1/bdbs/{}", uid)))
            .respond_with(ResponseTemplate::new(200).set_body_json(database))
            .mount(&self.server)
            .await;
    }

    /// Mock POST /v1/bdbs to create a database
    pub async fn mock_database_create(&self, response: Value) {
        Mock::given(method("POST"))
            .and(path("/v1/bdbs"))
            .respond_with(ResponseTemplate::new(201).set_body_json(response))
            .mount(&self.server)
            .await;
    }

    /// Mock DELETE /v1/bdbs/{uid}
    pub async fn mock_database_delete(&self, uid: u32) {
        Mock::given(method("DELETE"))
            .and(path(format!("/v1/bdbs/{}", uid)))
            .respond_with(ResponseTemplate::new(204))
            .mount(&self.server)
            .await;
    }

    // Node mocks

    /// Mock GET /v1/nodes to return a list of nodes
    pub async fn mock_nodes_list(&self, nodes: Vec<Value>) {
        Mock::given(method("GET"))
            .and(path("/v1/nodes"))
            .respond_with(ResponseTemplate::new(200).set_body_json(nodes))
            .mount(&self.server)
            .await;
    }

    /// Mock GET /v1/nodes/{uid} to return a specific node
    pub async fn mock_node_get(&self, uid: u32, node: Value) {
        Mock::given(method("GET"))
            .and(path(format!("/v1/nodes/{}", uid)))
            .respond_with(ResponseTemplate::new(200).set_body_json(node))
            .mount(&self.server)
            .await;
    }

    // Cluster mocks

    /// Mock GET /v1/cluster to return cluster info
    pub async fn mock_cluster_info(&self, cluster: Value) {
        Mock::given(method("GET"))
            .and(path("/v1/cluster"))
            .respond_with(ResponseTemplate::new(200).set_body_json(cluster))
            .mount(&self.server)
            .await;
    }

    /// Mock GET /v1/cluster/stats/last to return cluster stats
    pub async fn mock_cluster_stats(&self, stats: Value) {
        Mock::given(method("GET"))
            .and(path("/v1/cluster/stats/last"))
            .respond_with(ResponseTemplate::new(200).set_body_json(stats))
            .mount(&self.server)
            .await;
    }

    /// Mock GET /v1/license to return license info
    pub async fn mock_license(&self, license: Value) {
        Mock::given(method("GET"))
            .and(path("/v1/license"))
            .respond_with(ResponseTemplate::new(200).set_body_json(license))
            .mount(&self.server)
            .await;
    }

    // User mocks

    /// Mock GET /v1/users to return a list of users
    pub async fn mock_users_list(&self, users: Vec<Value>) {
        Mock::given(method("GET"))
            .and(path("/v1/users"))
            .respond_with(ResponseTemplate::new(200).set_body_json(users))
            .mount(&self.server)
            .await;
    }

    /// Mock GET /v1/users/{uid} to return a specific user
    pub async fn mock_user_get(&self, uid: u32, user: Value) {
        Mock::given(method("GET"))
            .and(path(format!("/v1/users/{}", uid)))
            .respond_with(ResponseTemplate::new(200).set_body_json(user))
            .mount(&self.server)
            .await;
    }

    // Error mocks

    /// Mock any GET request to a path pattern to return 404
    pub async fn mock_not_found(&self, path_pattern: &str) {
        Mock::given(method("GET"))
            .and(path_regex(path_pattern))
            .respond_with(super::responses::not_found("Resource not found"))
            .mount(&self.server)
            .await;
    }

    /// Mock any request to return 401 Unauthorized
    pub async fn mock_unauthorized(&self) {
        Mock::given(method("GET"))
            .respond_with(super::responses::unauthorized())
            .mount(&self.server)
            .await;
    }

    /// Mock any request to a path to return 500 Server Error
    pub async fn mock_server_error(&self, path_str: &str, message: &str) {
        Mock::given(method("GET"))
            .and(path(path_str))
            .respond_with(super::responses::server_error(message))
            .mount(&self.server)
            .await;
    }

    // Custom mock support

    /// Mount a custom mock on the server
    pub async fn mount(&self, mock: Mock) {
        mock.mount(&self.server).await;
    }

    /// Mount a custom response template at a specific path
    pub async fn mock_path(&self, http_method: &str, path_str: &str, response: ResponseTemplate) {
        Mock::given(method(http_method))
            .and(path(path_str))
            .respond_with(response)
            .mount(&self.server)
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::fixtures::{ClusterFixture, DatabaseFixture, NodeFixture};

    #[tokio::test]
    async fn test_mock_server_starts() {
        let server = MockEnterpriseServer::start().await;
        assert!(server.uri().starts_with("http://"));
    }

    #[tokio::test]
    async fn test_mock_databases_list() {
        let server = MockEnterpriseServer::start().await;
        server
            .mock_databases_list(vec![
                DatabaseFixture::new(1, "test-db").build(),
                DatabaseFixture::new(2, "other-db").build(),
            ])
            .await;

        let client = server.client();
        let dbs = client.databases().list().await.unwrap();
        assert_eq!(dbs.len(), 2);
        assert_eq!(dbs[0].name, "test-db");
        assert_eq!(dbs[1].name, "other-db");
    }

    #[tokio::test]
    async fn test_mock_database_get() {
        let server = MockEnterpriseServer::start().await;
        server
            .mock_database_get(
                1,
                DatabaseFixture::new(1, "my-cache")
                    .memory_size(2 * 1024 * 1024 * 1024)
                    .build(),
            )
            .await;

        let client = server.client();
        let db = client.databases().get(1).await.unwrap();
        assert_eq!(db.name, "my-cache");
        assert_eq!(db.memory_size, Some(2 * 1024 * 1024 * 1024));
    }

    #[tokio::test]
    async fn test_mock_nodes_list() {
        let server = MockEnterpriseServer::start().await;
        server
            .mock_nodes_list(vec![
                NodeFixture::new(1, "10.0.0.1").build(),
                NodeFixture::new(2, "10.0.0.2").cores(8).build(),
            ])
            .await;

        let client = server.client();
        let nodes = client.nodes().list().await.unwrap();
        assert_eq!(nodes.len(), 2);
    }

    #[tokio::test]
    async fn test_mock_cluster_info() {
        let server = MockEnterpriseServer::start().await;
        server
            .mock_cluster_info(
                ClusterFixture::new("test-cluster")
                    .nodes(vec![1, 2, 3])
                    .build(),
            )
            .await;

        let client = server.client();
        let info = client.cluster().info().await.unwrap();
        assert_eq!(info.name, "test-cluster");
    }

    #[tokio::test]
    async fn test_custom_mock() {
        use wiremock::ResponseTemplate;
        use wiremock::matchers::{method, path};

        let server = MockEnterpriseServer::start().await;

        // Use the inner MockServer for custom mocking
        Mock::given(method("GET"))
            .and(path("/v1/custom"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "custom": "response"
            })))
            .mount(server.inner())
            .await;
    }
}
