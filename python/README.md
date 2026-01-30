# redis-enterprise (Python)

Python bindings for the Redis Enterprise REST API client.

## Installation

```bash
pip install redis-enterprise
```

## Quick Start

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

## API

### EnterpriseClient

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

## Environment Variables

- `REDIS_ENTERPRISE_URL` - Base URL (default: https://localhost:9443)
- `REDIS_ENTERPRISE_USER` - Username
- `REDIS_ENTERPRISE_PASSWORD` - Password
- `REDIS_ENTERPRISE_INSECURE` - Set to "true" for self-signed certs

## License

MIT OR Apache-2.0
