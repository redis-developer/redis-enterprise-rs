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
redis-enterprise = "0.7"

# Optional: Enable Tower service integration
redis-enterprise = { version = "0.7", features = ["tower-integration"] }
```

## Quick Start

```rust
use redis_enterprise::EnterpriseClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client using builder pattern
    let client = EnterpriseClient::builder()
        .base_url("https://cluster.example.com:9443")
        .username("admin@example.com")
        .password("your-password")
        .insecure(false) // Set to true for self-signed certificates
        .build()?;

    // Get cluster information
    let cluster = client.cluster().info().await?;
    println!("Cluster: {:?}", cluster);

    // List databases (BDBs)
    let databases = client.databases().list().await?;
    println!("Databases: {:?}", databases);

    // Get node statistics
    let nodes = client.nodes().list().await?;
    println!("Nodes: {:?}", nodes);

    Ok(())
}
```

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

## Python Bindings

This library also provides Python bindings via PyO3:

```bash
pip install redis-enterprise
```

```python
from redis_enterprise import EnterpriseClient

# Create client
client = EnterpriseClient(
    base_url="https://cluster:9443",
    username="admin@redis.local",
    password="secret",
    insecure=True  # For self-signed certs
)

# Or from environment variables
client = EnterpriseClient.from_env()

# Async usage
async def main():
    dbs = await client.databases()
    for db in dbs:
        print(db["name"], db["uid"])

# Sync usage
dbs = client.databases_sync()
```

### Python API

- `EnterpriseClient(base_url, username, password, insecure=False, timeout_secs=None)`
- `EnterpriseClient.from_env()` - Create from environment variables

#### Cluster
- `cluster_info()` / `cluster_info_sync()` - Get cluster info
- `cluster_stats()` / `cluster_stats_sync()` - Get cluster statistics
- `license()` / `license_sync()` - Get license info

#### Databases
- `databases()` / `databases_sync()` - List all databases
- `database(uid)` / `database_sync(uid)` - Get database by UID

#### Nodes
- `nodes()` / `nodes_sync()` - List all nodes
- `node(uid)` / `node_sync(uid)` - Get node by UID

#### Users
- `users()` / `users_sync()` - List all users

#### Raw API
- `get(path)` / `get_sync(path)` - Raw GET request
- `post(path, body)` / `post_sync(path, body)` - Raw POST request
- `delete(path)` / `delete_sync(path)` - Raw DELETE request

### Environment Variables

- `REDIS_ENTERPRISE_URL` - Base URL (default: https://localhost:9443)
- `REDIS_ENTERPRISE_USER` - Username
- `REDIS_ENTERPRISE_PASSWORD` - Password
- `REDIS_ENTERPRISE_INSECURE` - Set to "true" for self-signed certs

## API Coverage

This library provides 100% coverage of the Redis Enterprise REST API, including:

- **Cluster Operations** - Bootstrap, configuration, topology
- **Database Management** - CRUD operations, actions, statistics
- **Node Management** - Add/remove nodes, statistics, actions
- **Security** - Users, roles, ACLs, LDAP integration
- **Modules** - Upload and manage Redis modules
- **Monitoring** - Stats, alerts, logs, diagnostics
- **Active-Active** - CRDB management and tasks
- **Administration** - License, certificates, services

## Documentation

- [API Documentation](https://docs.rs/redis-enterprise)
- [Redis Enterprise REST API Reference](https://docs.redis.com/latest/rs/references/rest-api/)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
