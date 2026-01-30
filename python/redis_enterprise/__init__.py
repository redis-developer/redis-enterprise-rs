"""Redis Enterprise Python client.

This module provides Python bindings for the Redis Enterprise REST API.

Example:
    from redis_enterprise import EnterpriseClient

    # Create client
    client = EnterpriseClient(
        base_url="https://cluster:9443",
        username="admin@redis.local",
        password="secret",
        insecure=True
    )

    # Async usage
    async def main():
        dbs = await client.databases()
        print(dbs)

    # Sync usage
    dbs = client.databases_sync()
"""

from .redis_enterprise import EnterpriseClient, RedisEnterpriseError, __version__

__all__ = ["EnterpriseClient", "RedisEnterpriseError", "__version__"]
