# redis-enterprise Python bindings

Python bindings for the `redis-enterprise-rs` client.

Install from PyPI:

```bash
pip install redis-enterprise
```

Basic usage:

```python
from redis_enterprise import EnterpriseClient

client = EnterpriseClient(
    base_url="https://cluster:9443",
    username="admin@redis.local",
    password="secret",
    insecure=True,
)

dbs = client.databases_sync()
```

For full project documentation and examples, see the repository root `README.md`.
