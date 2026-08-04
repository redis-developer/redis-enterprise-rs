//! Tests for the documented diagnostics configuration resource.

use redis_enterprise::{DiagnosticsHandler, EnterpriseClient};
use serde_json::json;
use wiremock::matchers::{basic_auth, body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(mock_server: &MockServer) -> EnterpriseClient {
    EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap()
}

#[tokio::test]
async fn get_diagnostics_config_uses_documented_route() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/diagnostics"))
        .and(basic_auth("admin", "password"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"enabled": true})))
        .mount(&mock_server)
        .await;

    let config = DiagnosticsHandler::new(client(&mock_server))
        .get_config()
        .await
        .unwrap();
    assert_eq!(config["enabled"], true);
}

#[tokio::test]
async fn update_diagnostics_config_uses_documented_route() {
    let mock_server = MockServer::start().await;
    let request = json!({"enabled": false});
    Mock::given(method("PUT"))
        .and(path("/v1/diagnostics"))
        .and(basic_auth("admin", "password"))
        .and(body_json(request.clone()))
        .respond_with(ResponseTemplate::new(200).set_body_json(request.clone()))
        .mount(&mock_server)
        .await;

    let config = DiagnosticsHandler::new(client(&mock_server))
        .update_config(request)
        .await
        .unwrap();
    assert_eq!(config["enabled"], false);
}
