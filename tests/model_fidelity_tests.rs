use redis_enterprise::{
    ClusterInfo, CreateDatabaseRequest, CreateUserRequest, Database, License, Node, Shard, User,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

const MODEL_FIDELITY_FIXTURE: &str = include_str!("fixtures/model_fidelity.json");

fn fixture() -> Value {
    serde_json::from_str(MODEL_FIDELITY_FIXTURE).expect("model-fidelity fixture should be JSON")
}

fn fixture_value(section: &str, name: &str) -> Value {
    fixture()[section][name].clone()
}

fn assert_preserves_non_null_fields(raw: &Value, typed: &Value, path: &str) {
    match (raw, typed) {
        (Value::Object(raw), Value::Object(typed)) => {
            for (key, raw_value) in raw {
                let child_path = format!("{path}/{key}");
                match typed.get(key) {
                    Some(typed_value) => {
                        assert_preserves_non_null_fields(raw_value, typed_value, &child_path)
                    }
                    None if raw_value.is_null() => {}
                    None => panic!("typed round-trip dropped non-null field {child_path}"),
                }
            }
        }
        (Value::Array(raw), Value::Array(typed)) => {
            assert_eq!(
                raw.len(),
                typed.len(),
                "typed round-trip changed array length at {path}"
            );
            for (index, (raw_value, typed_value)) in raw.iter().zip(typed).enumerate() {
                assert_preserves_non_null_fields(
                    raw_value,
                    typed_value,
                    &format!("{path}/{index}"),
                );
            }
        }
        _ => {}
    }
}

fn round_trip<T>(name: &str) -> T
where
    T: DeserializeOwned + Serialize,
{
    let raw = fixture_value("responses", name);
    let typed: T = serde_json::from_value(raw.clone())
        .unwrap_or_else(|error| panic!("{name} fixture should deserialize: {error}"));
    let serialized = serde_json::to_value(&typed)
        .unwrap_or_else(|error| panic!("{name} fixture should reserialize: {error}"));
    assert_preserves_non_null_fields(&raw, &serialized, "");
    typed
}

#[test]
fn core_response_models_preserve_sanitized_cross_version_fields() {
    // api-audit-response: GET /v1/cluster
    let cluster = round_trip::<ClusterInfo>("cluster");
    assert_eq!(
        cluster.additional_fields.get("availability_api"),
        Some(&Value::Bool(true))
    );

    // api-audit-response: GET /v1/bdbs
    let database = round_trip::<Database>("database");
    let endpoint = database
        .endpoints
        .as_ref()
        .and_then(|endpoints| endpoints.first())
        .expect("database fixture should contain an endpoint");
    assert_eq!(
        endpoint.oss_cluster_api_preferred_endpoint_type.as_deref(),
        Some("hostname")
    );
    assert!(
        database
            .additional_fields
            .contains_key("auto_shards_balancing")
    );

    // api-audit-response: GET /v1/license
    let license = round_trip::<License>("license");
    assert_eq!(license.cluster_name, None);
    assert_eq!(
        license.additional_fields.get("future_license_metadata"),
        Some(&Value::String("preserved".to_string()))
    );

    // api-audit-response: GET /v1/nodes
    let node = round_trip::<Node>("node");
    assert_eq!(
        node.node_guardrails_ingress_throttling_worker_limit_ops_per_sec,
        Some(-1)
    );

    // api-audit-response: GET /v1/shards
    let shard = round_trip::<Shard>("shard");
    assert_eq!(shard.detailed_status.as_deref(), Some("ok"));
    assert_eq!(
        shard.loading.as_ref().and_then(|loading| loading.progress),
        Some(100.0)
    );
    assert_eq!(shard.actual_role.as_deref(), Some("master"));

    // api-audit-response: GET /v1/users
    let user = round_trip::<User>("user");
    assert_eq!(user.last_login, Some(1_785_862_800));
}

#[test]
fn public_request_models_match_exact_wire_fixtures() {
    let database = CreateDatabaseRequest::builder()
        .name("model-fidelity-database")
        .memory_size(1_073_741_824)
        .redis_version("8.2")
        .replication(true)
        .persistence("aof")
        .shards_count(2)
        .build();
    assert_eq!(
        serde_json::to_value(database).expect("database request should serialize"),
        fixture_value("requests", "create_database")
    );

    let user = CreateUserRequest::builder()
        .email("model-fidelity@example.invalid")
        .password("synthetic-password")
        .name("Model Fidelity")
        .email_alerts(false)
        .role_uids(vec![1])
        .auth_method("regular")
        .build();
    assert_eq!(
        serde_json::to_value(user).expect("user request should serialize"),
        fixture_value("requests", "create_user")
    );
}

#[test]
fn fixture_provenance_is_versioned_and_sanitized() {
    let fixture = fixture();
    assert_eq!(fixture["schema_version"], 1);
    assert_eq!(
        fixture["provenance"]["redis_software_families"]
            .as_array()
            .expect("families should be an array")
            .len(),
        5
    );
    assert!(
        fixture["provenance"]["policy"]
            .as_str()
            .expect("policy should be text")
            .contains("no live payload values")
    );
}
