"""Tests for user-related Python bindings."""

import pytest


USER1 = {"uid": 1, "email": "admin@example.com", "role": "admin"}
USER2 = {"uid": 2, "email": "readonly@example.com", "role": "db_viewer"}


def test_users_sync(httpserver, client):
    httpserver.expect_request("/v1/users").respond_with_json([USER1, USER2])
    result = client.users_sync()
    assert len(result) == 2
    assert result[0]["email"] == "admin@example.com"


async def test_users_async(httpserver, client):
    httpserver.expect_request("/v1/users").respond_with_json([USER1, USER2])
    result = await client.users()
    assert len(result) == 2
    assert result[1]["role"] == "db_viewer"
