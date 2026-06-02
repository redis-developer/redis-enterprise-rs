"""Shared test fixtures."""

import pytest
from redis_enterprise import EnterpriseClient


@pytest.fixture
def client(httpserver):
    """EnterpriseClient pointed at the local mock HTTP server."""
    base_url = httpserver.url_for("").rstrip("/")
    return EnterpriseClient(
        base_url=base_url,
        username="testuser",
        password="testpass",
        insecure=True,
    )
