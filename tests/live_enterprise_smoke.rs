use std::time::{SystemTime, UNIX_EPOCH};

use redis_enterprise::{CreateDatabaseRequest, CreateUserRequest, EnterpriseClient, Node};
use serde_json::{Value, json};
use tokio::time::{Duration, sleep};

fn require_client() -> EnterpriseClient {
    EnterpriseClient::from_env().unwrap_or_else(|err| {
        panic!(
            "failed to build live Enterprise client from REDIS_ENTERPRISE_* env vars: {}",
            err
        )
    })
}

fn version_key(version: &str) -> (u32, u32, u32) {
    let mut parts = version.split('.').map(|part| part.parse().unwrap_or(0));
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

fn newest_advertised_redis_version(nodes: &[Node]) -> Option<String> {
    nodes
        .iter()
        .flat_map(|node| {
            node.supported_database_versions
                .as_deref()
                .unwrap_or_default()
        })
        .filter(|entry| entry["db_type"].as_str() == Some("redis"))
        .filter_map(|entry| entry["redis_version"].as_str())
        .max_by_key(|version| version_key(version))
        .map(str::to_owned)
}

#[test]
fn advertised_version_selection_uses_newest_redis_engine() {
    let nodes: Vec<Node> = serde_json::from_value(json!([{
        "uid": 1,
        "status": "active",
        "supported_database_versions": [
            {"db_type": "redis", "redis_version": "6.2", "version": "6.2.13"},
            {"db_type": "memcached", "version": "9.9.9"},
            {"db_type": "redis", "redis_version": "7.4", "version": "7.4.0"},
            {"db_type": "redis", "redis_version": "7.2", "version": "7.2.4"}
        ]
    }]))
    .expect("node fixture should deserialize");

    assert_eq!(
        newest_advertised_redis_version(&nodes).as_deref(),
        Some("7.4")
    );
}

async fn remove_live_test_databases(client: &EnterpriseClient, name_prefix: &str) {
    let databases = client
        .databases()
        .list()
        .await
        .expect("stale live-test database discovery should succeed");

    for database in databases
        .into_iter()
        .filter(|database| database.name.starts_with(name_prefix))
    {
        client
            .databases()
            .delete(database.uid)
            .await
            .unwrap_or_else(|err| {
                panic!(
                    "failed to remove stale live-test database {}: {}",
                    database.uid, err
                )
            });
    }
}

#[tokio::test]
#[ignore = "requires a live Redis Enterprise cluster configured via REDIS_ENTERPRISE_* env vars"]
async fn live_cluster_nodes_and_databases_smoke() {
    // api-audit-live: GET /v1/cluster
    // api-audit-live: GET /v1/nodes
    // api-audit-live: GET /v1/bdbs
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
    // api-audit-live: GET /v1/nodes
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
    // api-audit-live: GET /v1/cluster/check
    // api-audit-live: GET /v1/nodes
    // api-audit-live: GET /v1/nodes/check/{uid}
    // api-audit-live: GET /v1/shards
    // api-audit-live: GET /v1/shards/stats/{uid}
    // api-audit-live: GET /v1/shards/stats/last/{uid}
    // api-audit-live: GET /v1/shards/stats/last
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
#[ignore = "requires a disposable live Redis Enterprise cluster configured via REDIS_ENTERPRISE_* env vars"]
async fn live_database_create_delete_with_advertised_redis_version() {
    // api-audit-live: GET /v1/nodes
    // api-audit-live: GET /v1/bdbs
    // api-audit-live: POST /v1/bdbs
    // api-audit-live: GET /v1/bdbs/{uid}
    // api-audit-live: DELETE /v1/bdbs/{uid}
    const NAME_PREFIX: &str = "redis-enterprise-rs-live-";
    let client = require_client();

    remove_live_test_databases(&client, NAME_PREFIX).await;

    let nodes = client
        .nodes()
        .list()
        .await
        .expect("node list should succeed");
    let redis_version = newest_advertised_redis_version(&nodes)
        .expect("at least one node should advertise a Redis database version");
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_secs();
    let name = format!("{NAME_PREFIX}{unique}");

    let request = CreateDatabaseRequest::builder()
        .name(name.clone())
        .memory_size(104_857_600)
        .redis_version(redis_version.clone())
        .replication(false)
        .persistence("disabled")
        .build();
    let created = client
        .databases()
        .create(request)
        .await
        .expect("database create should accept an advertised Redis version");
    let uid = created.uid;

    let observation = async {
        for _ in 0..60 {
            let database = client
                .databases()
                .info(uid)
                .await
                .map_err(|err| format!("created database should be readable: {err}"))?;
            match database.status.as_deref() {
                Some("active") => {
                    if database.name != name {
                        return Err(format!(
                            "created database name mismatch: expected {name}, got {}",
                            database.name
                        ));
                    }
                    if database.redis_version.as_deref() != Some(redis_version.as_str()) {
                        return Err(format!(
                            "created database version mismatch: expected {redis_version}, got {:?}",
                            database.redis_version
                        ));
                    }
                    return Ok(());
                }
                Some("creation-failed" | "creation_failed" | "failed") => {
                    return Err(format!(
                        "database {uid} entered failure status {:?}",
                        database.status
                    ));
                }
                _ => sleep(Duration::from_secs(1)).await,
            }
        }

        Err(format!(
            "database {uid} did not become active within 60 seconds"
        ))
    }
    .await;

    let delete_result = client.databases().delete(uid).await;
    if let Err(err) = delete_result {
        panic!("failed to remove live-test database {uid}: {err}; observation: {observation:?}");
    }

    for _ in 0..30 {
        match client.databases().info(uid).await {
            Ok(_) => sleep(Duration::from_millis(200)).await,
            Err(err) if err.is_not_found() => {
                observation.expect("created database should match the requested version");
                return;
            }
            Err(err) => panic!("unexpected error while verifying database deletion: {err}"),
        }
    }

    panic!("deleted database {uid} was still visible after waiting for deletion to propagate");
}

#[tokio::test]
#[ignore = "requires a live Redis Enterprise cluster configured via REDIS_ENTERPRISE_* env vars"]
async fn live_user_create_get_delete_round_trip() {
    // api-audit-live: GET /v1/roles
    // api-audit-live: POST /v1/users
    // api-audit-live: GET /v1/users/{uid}
    // api-audit-live: DELETE /v1/users/{uid}
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
