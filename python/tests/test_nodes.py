"""Tests for node-related Python bindings."""

import pytest


NODE1 = {"uid": 1, "status": "active", "addr": "10.0.0.1", "cores": 8}
NODE2 = {"uid": 2, "status": "active", "addr": "10.0.0.2", "cores": 8}


def test_nodes_sync(httpserver, client):
    httpserver.expect_request("/v1/nodes").respond_with_json([NODE1, NODE2])
    result = client.nodes_sync()
    assert len(result) == 2
    assert result[0]["uid"] == 1


async def test_nodes_async(httpserver, client):
    httpserver.expect_request("/v1/nodes").respond_with_json([NODE1, NODE2])
    result = await client.nodes()
    assert len(result) == 2
    assert result[1]["uid"] == 2


def test_node_sync(httpserver, client):
    httpserver.expect_request("/v1/nodes/1").respond_with_json(NODE1)
    result = client.node_sync(1)
    assert result["uid"] == 1
    assert result["addr"] == "10.0.0.1"


async def test_node_async(httpserver, client):
    httpserver.expect_request("/v1/nodes/1").respond_with_json(NODE1)
    result = await client.node(1)
    assert result["uid"] == 1
    assert result["addr"] == "10.0.0.1"
