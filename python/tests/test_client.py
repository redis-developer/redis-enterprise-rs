"""Tests for EnterpriseClient construction."""

import os
import pytest
from redis_enterprise import EnterpriseClient, RedisEnterpriseError


def test_from_env_with_all_vars(monkeypatch):
    monkeypatch.setenv("REDIS_ENTERPRISE_URL", "http://localhost:9443")
    monkeypatch.setenv("REDIS_ENTERPRISE_USER", "admin@example.com")
    monkeypatch.setenv("REDIS_ENTERPRISE_PASSWORD", "supersecret")
    monkeypatch.setenv("REDIS_ENTERPRISE_INSECURE", "true")
    client = EnterpriseClient.from_env()
    assert client is not None


def test_from_env_missing_password_raises(monkeypatch):
    monkeypatch.delenv("REDIS_ENTERPRISE_PASSWORD", raising=False)
    monkeypatch.setenv("REDIS_ENTERPRISE_URL", "http://localhost:9443")
    with pytest.raises(RedisEnterpriseError):
        EnterpriseClient.from_env()


def test_from_env_uses_defaults_when_url_and_user_absent(monkeypatch):
    monkeypatch.delenv("REDIS_ENTERPRISE_URL", raising=False)
    monkeypatch.delenv("REDIS_ENTERPRISE_USER", raising=False)
    monkeypatch.setenv("REDIS_ENTERPRISE_PASSWORD", "supersecret")
    client = EnterpriseClient.from_env()
    assert client is not None


def test_constructor_basic():
    client = EnterpriseClient(
        base_url="http://localhost:9443",
        username="admin",
        password="secret",
        insecure=True,
    )
    assert client is not None
