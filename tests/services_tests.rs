//! Services endpoint tests for Redis Enterprise.
//!
//! `ServicesHandler` now only exposes `create()` — the list/get/update/status
//! and start/stop/restart methods were removed in the #65 audit follow-up
//! because their `/v1/services/*` paths are not in the REST API docs at any
//! version.

use redis_enterprise::{EnterpriseClient, ServicesHandler};
use serde_json::json;
use wiremock::matchers::{basic_auth, body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_services_create() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/services"))
        .and(basic_auth("admin", "password"))
        .and(body_json(json!({
            "name": "stats_archiver",
            "service_type": "stats_archiver",
            "enabled": true
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "service_id": "stats_archiver",
            "name": "stats_archiver",
            "service_type": "stats_archiver",
            "enabled": true
        })))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ServicesHandler::new(client);
    let result = handler
        .create(json!({
            "name": "stats_archiver",
            "service_type": "stats_archiver",
            "enabled": true
        }))
        .await
        .unwrap();
    assert_eq!(result.service_id, "stats_archiver");
    assert!(result.enabled);
}
