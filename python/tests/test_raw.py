"""Tests for raw HTTP passthrough bindings."""

import pytest


def test_get_sync(httpserver, client):
    httpserver.expect_request("/v1/cluster").respond_with_json({"name": "raw-cluster"})
    result = client.get_sync("/v1/cluster")
    assert result["name"] == "raw-cluster"


async def test_get_async(httpserver, client):
    httpserver.expect_request("/v1/cluster").respond_with_json({"name": "raw-cluster"})
    result = await client.get("/v1/cluster")
    assert result["name"] == "raw-cluster"


def test_post_sync(httpserver, client):
    httpserver.expect_request("/v1/bdbs", method="POST").respond_with_json(
        {"uid": 3, "name": "new-db"}
    )
    result = client.post_sync("/v1/bdbs", {"name": "new-db", "memory_size": 100000000})
    assert result["uid"] == 3
    assert result["name"] == "new-db"


async def test_post_async(httpserver, client):
    httpserver.expect_request("/v1/bdbs", method="POST").respond_with_json(
        {"uid": 3, "name": "new-db"}
    )
    result = await client.post("/v1/bdbs", {"name": "new-db", "memory_size": 100000000})
    assert result["uid"] == 3


def test_delete_sync(httpserver, client):
    httpserver.expect_request("/v1/bdbs/1", method="DELETE").respond_with_json(
        {"status": "deleted"}
    )
    result = client.delete_sync("/v1/bdbs/1")
    assert isinstance(result, dict)


async def test_delete_async(httpserver, client):
    httpserver.expect_request("/v1/bdbs/1", method="DELETE").respond_with_json(
        {"status": "deleted"}
    )
    result = await client.delete("/v1/bdbs/1")
    assert isinstance(result, dict)
