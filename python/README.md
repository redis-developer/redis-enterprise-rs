# redis-enterprise Python bindings

Python bindings for the Redis Enterprise REST API, built with [PyO3](https://pyo3.rs)
on top of the [`redis-enterprise`](https://crates.io/crates/redis-enterprise) Rust crate.

## Scope and philosophy

### Goals

- **Convenience, not parity.** The Python layer exposes the most common read and
  inspection operations — cluster info, database listing, node status, user listing —
  plus raw HTTP access for anything else.
- **Sync and async.** Every named method ships as both a blocking `_sync` variant and
  an `async`/`await` coroutine so it fits naturally in scripts, notebooks, or async
  applications.
- **Plain Python dicts.** All responses are deserialized to Python `dict`/`list`
  (via `serde_json`). No custom model classes to learn; use `SimpleNamespace`,
  `pydantic`, or `dataclasses` as you prefer.
- **Zero required event loop.** The `_sync` variants drive a dedicated Tokio runtime
  internally, so they work from plain Python scripts or Jupyter notebooks without any
  `asyncio` setup.

### Non-goals

- **Full API parity with the Rust crate.** Write operations (database create/update/
  delete), CRDB management, module uploads, and so on are outside the initial scope.
- **Typed response models.** Raw dicts keep the binding thin and avoid a maintenance
  burden on the Python side.
- **Re-exporting every Rust builder knob.** For advanced configuration use the Rust
  crate directly.

---

## Installation

```bash
pip install redis-enterprise
```

Requires Python ≥ 3.9. Wheels are published for Linux (x86\_64, aarch64),
macOS (x86\_64, arm64), and Windows (x86\_64) via the `publish-python` CI workflow.

---

## Quick start

```python
from redis_enterprise import EnterpriseClient

# Explicit configuration
client = EnterpriseClient(
    base_url="https://cluster.example.com:9443",
    username="admin@redis.local",
    password="secret",
    insecure=True,       # skip TLS verification — dev/lab only
    timeout_secs=30,     # optional; omit to use the default
)

# Or let the client read credentials from environment variables
client = EnterpriseClient.from_env()

# ── Synchronous ──────────────────────────────────────────────
cluster = client.cluster_info_sync()
print(cluster["name"])

dbs = client.databases_sync()
for db in dbs:
    print(db["name"], db["uid"])

# ── Async ────────────────────────────────────────────────────
import asyncio

async def main():
    cluster = await client.cluster_info()
    print(cluster["name"])

asyncio.run(main())
```

---

## API reference

All methods return plain Python `dict` or `list[dict]`.

### Constructor

| Signature | Description |
|-----------|-------------|
| `EnterpriseClient(base_url, username, password, insecure=False, timeout_secs=None)` | Build a client explicitly |
| `EnterpriseClient.from_env()` | Read credentials from environment variables (see [Environment variables](#environment-variables)) |

### Cluster

| Async | Sync | Returns | Description |
|-------|------|---------|-------------|
| `cluster_info()` | `cluster_info_sync()` | `dict` | Cluster name, version, and topology summary |
| `cluster_stats()` | `cluster_stats_sync()` | `dict` | Cluster-level performance counters |
| `license()` | `license_sync()` | `dict` | License details (expiry, limits, active features) |

### Databases

| Async | Sync | Returns | Description |
|-------|------|---------|-------------|
| `databases()` | `databases_sync()` | `list[dict]` | All databases (BDBs) on the cluster |
| `database(uid)` | `database_sync(uid)` | `dict` | Single database by numeric UID |

### Nodes

| Async | Sync | Returns | Description |
|-------|------|---------|-------------|
| `nodes()` | `nodes_sync()` | `list[dict]` | All nodes in the cluster |
| `node(uid)` | `node_sync(uid)` | `dict` | Single node by numeric UID |

### Users

| Async | Sync | Returns | Description |
|-------|------|---------|-------------|
| `users()` | `users_sync()` | `list[dict]` | All users configured on the cluster |

---

## Raw HTTP access

Use the raw pass-through methods for any endpoint not yet covered by a named method.
Paths are relative to the cluster base URL (e.g. `"/v1/bdbs"`).

```python
# GET
roles = client.get_sync("/v1/roles")

# POST — body is a plain Python dict
new_role = client.post_sync(
    "/v1/roles",
    {"name": "ops-reader", "management": "db_viewer"},
)

# DELETE
client.delete_sync(f"/v1/roles/{role_id}")

# Async equivalents
roles = await client.get("/v1/roles")
new_role = await client.post(
    "/v1/roles",
    {"name": "ops-reader", "management": "db_viewer"},
)
await client.delete(f"/v1/roles/{role_id}")
```

| Async | Sync | Signature | Description |
|-------|------|-----------|-------------|
| `get(path)` | `get_sync(path)` | `path: str` | Raw GET — returns parsed JSON as `dict` or `list` |
| `post(path, body)` | `post_sync(path, body)` | `path: str, body: dict` | Raw POST — body is serialized to JSON |
| `delete(path)` | `delete_sync(path)` | `path: str` | Raw DELETE — returns parsed JSON response |

---

## Error handling

```python
from redis_enterprise import EnterpriseClient, RedisEnterpriseError

try:
    db = client.database_sync(9999)
except ValueError:
    print("Database not found")
except RedisEnterpriseError as e:
    print(f"API error: {e}")
except ConnectionError as e:
    print(f"Could not reach cluster: {e}")
```

| Rust error variant | Python exception | Typical cause |
|--------------------|-----------------|---------------|
| `ConnectionError` | `ConnectionError` | Network unreachable, DNS failure, TLS handshake error |
| `AuthenticationFailed` | `RedisEnterpriseError` | Wrong credentials |
| `Unauthorized` | `RedisEnterpriseError` | Valid credentials but insufficient privilege |
| `NotFound` | `ValueError` | UID or path does not exist on the cluster |
| `ValidationError` | `ValueError` | Malformed request body or invalid parameter |
| Other | `RuntimeError` | Unexpected server-side or deserialization error |

`RedisEnterpriseError` is a custom exception class exported by the module and is a
subclass of `Exception`.

---

## Planned expansion — first tranche

The following operations are targeted for the next minor release:

| Method | Description |
|--------|-------------|
| `node_stats(uid)` / `node_stats_sync(uid)` | Per-node performance counters |
| `database_stats(uid)` / `database_stats_sync(uid)` | Per-database performance counters |
| `roles()` / `roles_sync()` | RBAC role listing |
| `put(path, body)` / `put_sync(path, body)` | Raw PUT for the HTTP pass-through layer |

Mutation operations (database create/update/delete, user management) are under
consideration for a subsequent tranche but are not committed.

---

## Testing strategy

Unit tests live in `python/tests/` and use `pytest` + `pytest-asyncio`.
Integration tests require a live Redis Enterprise cluster; see the
[live-validation runbook](../docs/live-validation.md) for standing one up with Docker
Compose.

```bash
# Install dev extras
pip install -e "python/[dev]"

# Unit tests (no cluster required)
pytest python/tests/unit/

# Integration tests (cluster required — see docs/live-validation.md)
pytest python/tests/integration/
```

---

## Release and packaging

The `publish-python` GitHub Actions workflow fires on version tags (`v*`).
It uses [maturin](https://www.maturin.rs) to compile the PyO3 extension module for
each supported platform and architecture, then publishes the wheels and an sdist to
[PyPI](https://pypi.org/project/redis-enterprise/).

The Python package version is derived from `python/Cargo.toml`, which is kept in sync
with the root crate version.

---

## Environment variables

These variables are read by `EnterpriseClient.from_env()`:

| Variable | Default | Description |
|----------|---------|-------------|
| `REDIS_ENTERPRISE_URL` | `https://localhost:9443` | Cluster base URL |
| `REDIS_ENTERPRISE_USER` | _(required)_ | Username |
| `REDIS_ENTERPRISE_PASSWORD` | _(required)_ | Password |
| `REDIS_ENTERPRISE_INSECURE` | `false` | Set `true` to skip TLS certificate verification (dev/lab only) |
| `REDIS_ENTERPRISE_CA_CERT` | _(optional)_ | Path to a custom CA certificate PEM file |
