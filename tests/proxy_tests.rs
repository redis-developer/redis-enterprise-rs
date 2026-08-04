//! Proxy endpoint tests for Redis Enterprise

use redis_enterprise::{EnterpriseClient, ProxyHandler};
use serde_json::json;
use wiremock::matchers::{basic_auth, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// Test helper functions
fn success_response(body: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(body)
}

fn active_proxy() -> serde_json::Value {
    json!({
        "uid": 1,
        "bdb_uid": 1,
        "node_uid": 1,
        "status": "active",
        "addr": "10.0.0.1",
        "port": 12000,
        "max_connections": 1000,
        "threads": 4
    })
}

fn standby_proxy() -> serde_json::Value {
    json!({
        "uid": 2,
        "bdb_uid": 1,
        "node_uid": 2,
        "status": "standby",
        "addr": "10.0.0.2",
        "port": 12001,
        "max_connections": 1000,
        "threads": 4
    })
}

fn minimal_proxy() -> serde_json::Value {
    json!({
        "uid": 3,
        "bdb_uid": 2,
        "node_uid": 1,
        "status": "active"
    })
}

#[tokio::test]
async fn test_proxy_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/proxies"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([
            active_proxy(),
            standby_proxy(),
            minimal_proxy()
        ])))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ProxyHandler::new(client);
    let result = handler.list().await;

    assert!(result.is_ok());
    let proxies = result.unwrap();
    assert_eq!(proxies.len(), 3);

    // Verify active proxy details
    let active = &proxies[0];
    assert_eq!(active.uid, 1);
    assert_eq!(active.bdb_uid, Some(1));
    assert_eq!(active.node_uid, Some(1));
    assert_eq!(active.status.as_deref(), Some("active"));
    assert_eq!(active.addr, Some("10.0.0.1".to_string()));
    assert_eq!(active.port, Some(12000));
    assert_eq!(active.max_connections, Some(1000));
    assert_eq!(active.threads, Some(4));

    // Verify standby proxy
    let standby = &proxies[1];
    assert_eq!(standby.uid, 2);
    assert_eq!(standby.status.as_deref(), Some("standby"));
    assert_eq!(standby.addr, Some("10.0.0.2".to_string()));
    assert_eq!(standby.port, Some(12001));

    // Verify minimal proxy (no optional fields)
    let minimal = &proxies[2];
    assert_eq!(minimal.uid, 3);
    assert_eq!(minimal.bdb_uid, Some(2));
    assert_eq!(minimal.status.as_deref(), Some("active"));
    assert!(minimal.addr.is_none());
    assert!(minimal.port.is_none());
    assert!(minimal.max_connections.is_none());
    assert!(minimal.threads.is_none());
}

#[tokio::test]
async fn test_proxy_list_empty() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/proxies"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([])))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ProxyHandler::new(client);
    let result = handler.list().await;

    assert!(result.is_ok());
    let proxies = result.unwrap();
    assert_eq!(proxies.len(), 0);
}

#[tokio::test]
async fn test_proxy_get() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/proxies/1"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(active_proxy()))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ProxyHandler::new(client);
    let result = handler.get(1).await;

    assert!(result.is_ok());
    let proxy = result.unwrap();
    assert_eq!(proxy.uid, 1);
    assert_eq!(proxy.bdb_uid, Some(1));
    assert_eq!(proxy.node_uid, Some(1));
    assert_eq!(proxy.status.as_deref(), Some("active"));
    assert_eq!(proxy.addr, Some("10.0.0.1".to_string()));
    assert_eq!(proxy.port, Some(12000));
    assert_eq!(proxy.max_connections, Some(1000));
    assert_eq!(proxy.threads, Some(4));
}

#[tokio::test]
async fn test_proxy_get_minimal() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/proxies/3"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(minimal_proxy()))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ProxyHandler::new(client);
    let result = handler.get(3).await;

    assert!(result.is_ok());
    let proxy = result.unwrap();
    assert_eq!(proxy.uid, 3);
    assert_eq!(proxy.bdb_uid, Some(2));
    assert_eq!(proxy.node_uid, Some(1));
    assert_eq!(proxy.status.as_deref(), Some("active"));
    assert!(proxy.addr.is_none());
    assert!(proxy.port.is_none());
}

#[tokio::test]
async fn test_proxy_list_by_database() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/proxies"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([
            active_proxy(),
            standby_proxy(),
            minimal_proxy()
        ])))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ProxyHandler::new(client);
    let result = handler.list_by_database(1).await;

    assert!(result.is_ok());
    let proxies = result.unwrap();
    assert_eq!(proxies.len(), 2);

    // Both proxies should belong to database 1
    assert_eq!(proxies[0].bdb_uid, Some(1));
    assert_eq!(proxies[1].bdb_uid, Some(1));

    // Verify active and standby status
    assert_eq!(proxies[0].status.as_deref(), Some("active"));
    assert_eq!(proxies[1].status.as_deref(), Some("standby"));
}

#[tokio::test]
async fn test_proxy_list_by_database_empty() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/proxies"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([active_proxy()])))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ProxyHandler::new(client);
    let result = handler.list_by_database(999).await;

    assert!(result.is_ok());
    let proxies = result.unwrap();
    assert_eq!(proxies.len(), 0);
}

#[tokio::test]
async fn test_proxy_list_by_node() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/proxies"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([
            active_proxy(),
            standby_proxy(),
            minimal_proxy()
        ])))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ProxyHandler::new(client);
    let result = handler.list_by_node(1).await;

    assert!(result.is_ok());
    let proxies = result.unwrap();
    assert_eq!(proxies.len(), 2);

    // Both proxies should be on node 1
    assert_eq!(proxies[0].node_uid, Some(1));
    assert_eq!(proxies[1].node_uid, Some(1));

    // Verify different databases
    assert_eq!(proxies[0].bdb_uid, Some(1));
    assert_eq!(proxies[1].bdb_uid, Some(2));
}

#[tokio::test]
async fn test_proxy_list_by_node_empty() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/proxies"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([active_proxy()])))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ProxyHandler::new(client);
    let result = handler.list_by_node(999).await;

    assert!(result.is_ok());
    let proxies = result.unwrap();
    assert_eq!(proxies.len(), 0);
}

#[tokio::test]
async fn test_proxy_get_nonexistent() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/proxies/999"))
        .and(basic_auth("admin", "password"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Proxy not found"
        })))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ProxyHandler::new(client);
    let result = handler.get(999).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_proxy_list_global_config_fixture_decodes_with_u64_maxmemory_clients() {
    // Regression guard for #64: the recorded fixture has
    // maxmemory_clients = 4_294_967_296 (u32::MAX + 1), which used to
    // overflow the u32-typed field. Also exercises the global proxy
    // configuration shape where bdb_uid / node_uid / addr / status are
    // absent.
    let mock_server = MockServer::start().await;

    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/proxies_list.json"))
            .expect("fixture should be valid JSON");

    Mock::given(method("GET"))
        .and(path("/v1/proxies"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(fixture))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ProxyHandler::new(client);
    let proxies = handler.list().await.expect("fixture should decode");
    assert_eq!(proxies.len(), 1);

    let proxy = &proxies[0];
    assert_eq!(proxy.uid, 1);
    assert!(proxy.bdb_uid.is_none());
    assert!(proxy.node_uid.is_none());
    assert!(proxy.addr.is_none());
    assert!(proxy.status.is_none());

    // The u32-overflowing value comes through intact as u64.
    assert_eq!(proxy.maxmemory_clients, Some(4_294_967_296));
}
