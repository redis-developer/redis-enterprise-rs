"""Tests for database-related Python bindings."""

import pytest


DB1 = {"uid": 1, "name": "cache", "status": "active", "port": 12001}
DB2 = {"uid": 2, "name": "sessions", "status": "active", "port": 12002}


def test_databases_sync(httpserver, client):
    httpserver.expect_request("/v1/bdbs").respond_with_json([DB1, DB2])
    result = client.databases_sync()
    assert len(result) == 2
    assert result[0]["name"] == "cache"
    assert result[1]["uid"] == 2


async def test_databases_async(httpserver, client):
    httpserver.expect_request("/v1/bdbs").respond_with_json([DB1, DB2])
    result = await client.databases()
    assert len(result) == 2
    assert result[0]["name"] == "cache"


def test_database_sync(httpserver, client):
    httpserver.expect_request("/v1/bdbs/1").respond_with_json(DB1)
    result = client.database_sync(1)
    assert result["uid"] == 1
    assert result["name"] == "cache"


async def test_database_async(httpserver, client):
    httpserver.expect_request("/v1/bdbs/1").respond_with_json(DB1)
    result = await client.database(1)
    assert result["uid"] == 1
    assert result["name"] == "cache"
