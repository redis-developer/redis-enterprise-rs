use std::time::{SystemTime, UNIX_EPOCH};

use redis_enterprise::{CreateUserRequest, EnterpriseClient};
use serde_json::Value;
use tokio::time::{Duration, sleep};

fn require_client() -> EnterpriseClient {
    EnterpriseClient::from_env().unwrap_or_else(|err| {
        panic!(
            "failed to build live Enterprise client from REDIS_ENTERPRISE_* env vars: {}",
            err
        )
    })
}

#[tokio::test]
#[ignore = "requires a live Redis Enterprise cluster configured via REDIS_ENTERPRISE_* env vars"]
async fn live_cluster_nodes_and_databases_smoke() {
    let client = require_client();

    let cluster = client
        .cluster()
        .info()
        .await
        .expect("cluster info should succeed");
    assert!(!cluster.name.is_empty(), "cluster name should not be empty");

    let nodes = client
        .nodes()
        .list()
        .await
        .expect("node list should succeed");
    assert!(!nodes.is_empty(), "expected at least one node");
    assert!(
        nodes.iter().all(|node| !node.status.is_empty()),
        "expected all live nodes to report a status"
    );

    let databases = client
        .databases()
        .list()
        .await
        .expect("database list should succeed");
    assert!(!databases.is_empty(), "expected at least one database");
    assert!(
        databases.iter().all(|db| !db.name.is_empty()),
        "expected all live databases to have a name"
    );
}

#[tokio::test]
#[ignore = "requires a live Redis Enterprise cluster configured via REDIS_ENTERPRISE_* env vars"]
async fn live_typed_and_raw_nodes_counts_match() {
    let client = require_client();

    let typed_nodes = client
        .nodes()
        .list()
        .await
        .expect("typed node list should succeed");
    let raw_nodes: Value = client
        .get_raw("/v1/nodes")
        .await
        .expect("raw node list should succeed");
    let raw_nodes = raw_nodes
        .as_array()
        .expect("raw node payload should be an array");

    assert_eq!(
        typed_nodes.len(),
        raw_nodes.len(),
        "typed and raw node counts should match"
    );
}

#[tokio::test]
#[ignore = "requires a live Redis Enterprise cluster configured via REDIS_ENTERPRISE_* env vars"]
async fn live_cluster_check_and_shard_stats_smoke() {
    let client = require_client();

    let check = client
        .cluster()
        .check()
        .await
        .expect("cluster check should succeed");
    assert!(
        check["cluster_test_result"].is_boolean(),
        "cluster check should include a boolean cluster_test_result"
    );

    let nodes = client
        .nodes()
        .list()
        .await
        .expect("node list should succeed");
    let first_node = nodes.first().expect("expected at least one node");

    let node_check = client
        .nodes()
        .check(first_node.uid)
        .await
        .expect("node check should succeed");
    assert_eq!(
        node_check["node_uid"].as_u64(),
        Some(first_node.uid.into()),
        "node check should report the requested node uid"
    );

    let shards = client
        .shards()
        .list()
        .await
        .expect("shard list should succeed");
    let first_shard = shards.first().expect("expected at least one shard");
    let shard_uid = first_shard
        .uid
        .parse::<u32>()
        .expect("live shard uid should parse as u32");

    let shard_stats = client
        .stats()
        .shard(shard_uid, None)
        .await
        .expect("single shard stats should succeed");
    assert!(
        !shard_stats.intervals.is_empty(),
        "single shard stats should include intervals"
    );

    let shard_last = client
        .stats()
        .shard_last(shard_uid)
        .await
        .expect("single shard last stats should succeed");
    assert!(
        shard_last.get(shard_uid.to_string()).is_some(),
        "single shard last stats should include the shard uid key"
    );

    let shards_last = client
        .stats()
        .shards_last()
        .await
        .expect("all shards last stats should succeed");
    assert!(
        shards_last.as_object().is_some(),
        "all shards last stats should be returned as an object keyed by shard uid"
    );
}

#[tokio::test]
#[ignore = "requires a live Redis Enterprise cluster configured via REDIS_ENTERPRISE_* env vars"]
async fn live_user_create_get_delete_round_trip() {
    let client = require_client();

    let admin_role_uid = client
        .roles()
        .list()
        .await
        .expect("role list should succeed")
        .into_iter()
        .find(|role| role.management.as_deref() == Some("admin") || role.name == "Admin")
        .expect("expected at least one admin-capable role")
        .uid;

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_secs();
    let email = format!("codex-live-{}@example.com", unique);

    let request = CreateUserRequest::builder()
        .email(email.clone())
        .password("CodexTest123!")
        .role_uids(vec![admin_role_uid])
        .name("Codex Live Test")
        .email_alerts(false)
        .auth_method("regular")
        .build();

    let created = client
        .users()
        .create(request)
        .await
        .expect("user create should succeed on an RBAC-enabled live cluster");

    assert_eq!(created.email, email);
    assert_eq!(created.role_uids, Some(vec![admin_role_uid]));

    let fetched = client
        .users()
        .get(created.uid)
        .await
        .expect("created user should be fetchable");
    assert_eq!(fetched.email, email);

    client
        .users()
        .delete(created.uid)
        .await
        .expect("user delete should succeed");

    for _ in 0..10 {
        match client.users().get(created.uid).await {
            Ok(_) => sleep(Duration::from_millis(200)).await,
            Err(err) if err.is_not_found() => return,
            Err(err) => panic!("unexpected error while verifying user deletion: {}", err),
        }
    }

    panic!("deleted user was still visible after waiting for deletion to propagate");
}
