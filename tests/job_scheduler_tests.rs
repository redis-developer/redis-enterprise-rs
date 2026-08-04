//! Tests for the documented global job scheduler configuration resource.

use redis_enterprise::{EnterpriseClient, JobSchedulerHandler};
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
async fn get_job_scheduler_config_uses_documented_route() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/job_scheduler"))
        .and(basic_auth("admin", "password"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"enabled": true})))
        .mount(&mock_server)
        .await;

    let config = JobSchedulerHandler::new(client(&mock_server))
        .get_config()
        .await
        .unwrap();
    assert_eq!(config["enabled"], true);
}

#[tokio::test]
async fn update_job_scheduler_config_uses_documented_route() {
    let mock_server = MockServer::start().await;
    let request = json!({"enabled": false});
    Mock::given(method("PUT"))
        .and(path("/v1/job_scheduler"))
        .and(basic_auth("admin", "password"))
        .and(body_json(request.clone()))
        .respond_with(ResponseTemplate::new(200).set_body_json(request.clone()))
        .mount(&mock_server)
        .await;

    let config = JobSchedulerHandler::new(client(&mock_server))
        .update_config(request)
        .await
        .unwrap();
    assert_eq!(config["enabled"], false);
}
