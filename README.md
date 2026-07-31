# redis-enterprise

[![Crates.io](https://img.shields.io/crates/v/redis-enterprise.svg)](https://crates.io/crates/redis-enterprise)
[![Documentation](https://docs.rs/redis-enterprise/badge.svg)](https://docs.rs/redis-enterprise)
[![CI](https://github.com/redis-developer/redis-enterprise-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/redis-developer/redis-enterprise-rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/redis-enterprise.svg)](https://github.com/redis-developer/redis-enterprise-rs)
[![PyPI](https://img.shields.io/pypi/v/redis-enterprise.svg)](https://pypi.org/project/redis-enterprise/)

A comprehensive Rust client library for the Redis Enterprise REST API, with Python bindings.

## Features

- Complete coverage of Redis Enterprise REST API endpoints
- Async/await support with tokio
- Strong typing for API requests and responses
- Comprehensive error handling
- Optional Tower service integration for middleware composition
- Support for all Redis Enterprise features including:
  - Cluster management and bootstrap
  - Database (BDB) operations
  - Node management and statistics
  - User and role management
  - Redis modules
  - Active-Active (CRDB) databases
  - Monitoring and alerts

## Installation

```toml
[dependencies]
redis-enterprise = "0.9"

# Optional: Enable Tower service integration
redis-enterprise = { version = "0.9", features = ["tower-integration"] }
```

## Quick Start

```rust
use redis_enterprise::EnterpriseClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build from environment variables (REDIS_ENTERPRISE_URL/USER/PASSWORD/INSECURE)
    let client = EnterpriseClient::from_env()?;

    // Or build explicitly:
    // let client = EnterpriseClient::builder()
    //     .base_url("https://cluster.example.com:9443")
    //     .username("admin@example.com")
    //     .password("your-password")
    //     .insecure(false) // Set to true for self-signed certificates
    //     .build()?;

    // Get cluster information
    let cluster = client.cluster().info().await?;
    println!("Cluster: {}", cluster.name);

    // List databases (BDBs)
    let databases = client.databases().list().await?;
    for db in &databases {
        println!("  {} (uid {})", db.name, db.uid);
    }

    // List nodes
    let nodes = client.nodes().list().await?;
    for node in &nodes {
        println!("  Node {}: {} ({})", node.uid, node.addr.as_deref().unwrap_or("?"), node.status);
    }

    Ok(())
}
```

See the [`examples/`](examples) directory for end-to-end workflows: `basic_enterprise`,
`cluster_setup_simple`, `crdb_basics`, and `database_actions`.

## Tower Integration

Enable the `tower-integration` feature to use the client with Tower middleware:

```rust
use redis_enterprise::EnterpriseClient;
use redis_enterprise::tower_support::ApiRequest;
use tower::ServiceExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = EnterpriseClient::builder()
        .base_url("https://localhost:9443")
        .username("admin")
        .password("password")
        .insecure(true)
        .build()?;

    // Convert to a Tower service
    let mut service = client.into_service();

    // Use the service
    let response = service
        .oneshot(ApiRequest::get("/v1/cluster"))
        .await?;

    println!("Response: {:?}", response.body);
    Ok(())
}
```

This enables composition with Tower middleware like circuit breakers, retry, rate limiting, and more.

## Environment Variables

- `REDIS_ENTERPRISE_URL` — Base URL (default: `https://localhost:9443`)
- `REDIS_ENTERPRISE_USER` — Username
- `REDIS_ENTERPRISE_PASSWORD` — Password
- `REDIS_ENTERPRISE_INSECURE` — Set to `true` to skip TLS verification (dev only)
- `REDIS_ENTERPRISE_CA_CERT` — Path to a custom CA certificate PEM file

## Python Bindings

A PyO3-based convenience layer published at
[redis-enterprise on PyPI](https://pypi.org/project/redis-enterprise/).

The bindings cover the most common read and inspection operations — cluster info,
databases, nodes, users — and include raw HTTP pass-through methods (`get`, `post`,
`delete`) for anything else. Every method is available in both an `async`/`await`
form and a blocking `_sync` form that works without an event loop.

```bash
pip install redis-enterprise
```

See [`python/README.md`](python/README.md) for the full API reference, error handling,
planned expansion, and packaging notes.

## API Coverage

The crate aims for comprehensive coverage of the documented Redis Enterprise
REST API surface. Handler organization:

- **Cluster** — bootstrap, info, topology, SSO/SAML, settings, license
- **Databases (BDB)** — CRUD, actions (recover/export/import/flush/upgrade),
  per-DB alerts, endpoints, stats
- **Nodes** — list, get, stats, actions, snapshots, check
- **Security** — users, roles, RBAC, LDAP mappings, redis_acls
- **Modules** — v1 + v2 module management, user-defined module artifacts
- **Active-Active (CRDB)** — list / create / update / delete / tasks plus
  flush, health_report, purge, updates
- **Monitoring** — stats, alerts, logs, diagnostics, job_scheduler
- **Administration** — license, certificates, OCSP, debug info, services

For an authoritative per-endpoint inventory see
[`docs/api-inventory.csv`](docs/api-inventory.csv). The generated
[method-aware coverage audit](docs/api-coverage-audit.md) keeps handler, mock,
request, response, fixture, and live evidence separate. For the historical
audit of routes the SDK once exposed but the docs do not, see
[`docs/api-gap-triage.md`](docs/api-gap-triage.md).

## Documentation

- [API Documentation](https://docs.rs/redis-enterprise)
- [Redis Enterprise REST API Reference](https://docs.redis.com/latest/rs/references/rest-api/)
- [Local live-validation runbook](docs/live-validation.md) — bring up a disposable Redis Enterprise cluster via Docker Compose for SDK validation against the real API.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
