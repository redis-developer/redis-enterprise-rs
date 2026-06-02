"""Tests for error mapping in Python bindings."""

import pytest
from redis_enterprise import RedisEnterpriseError


# --- Methods using handle_response (cluster_info, databases, etc.) ---

def test_cluster_info_401_raises_redis_enterprise_error(httpserver, client):
    httpserver.expect_request("/v1/cluster").respond_with_data(
        "", status=401, content_type="application/json"
    )
    with pytest.raises(RedisEnterpriseError):
        client.cluster_info_sync()


async def test_cluster_info_401_async(httpserver, client):
    httpserver.expect_request("/v1/cluster").respond_with_data(
        "", status=401, content_type="application/json"
    )
    with pytest.raises(RedisEnterpriseError):
        await client.cluster_info()


def test_database_404_raises_value_error(httpserver, client):
    httpserver.expect_request("/v1/bdbs/999").respond_with_data(
        "", status=404, content_type="application/json"
    )
    with pytest.raises(ValueError):
        client.database_sync(999)


async def test_database_404_async(httpserver, client):
    httpserver.expect_request("/v1/bdbs/999").respond_with_data(
        "", status=404, content_type="application/json"
    )
    with pytest.raises(ValueError):
        await client.database(999)


def test_nodes_500_raises_runtime_error(httpserver, client):
    httpserver.expect_request("/v1/nodes").respond_with_data(
        "Internal Server Error", status=500, content_type="text/plain"
    )
    with pytest.raises(RuntimeError):
        client.nodes_sync()


async def test_nodes_500_async(httpserver, client):
    httpserver.expect_request("/v1/nodes").respond_with_data(
        "Internal Server Error", status=500, content_type="text/plain"
    )
    with pytest.raises(RuntimeError):
        await client.nodes()


# --- get_raw / post_raw also use handle_response ---

def test_get_raw_401_raises_redis_enterprise_error(httpserver, client):
    httpserver.expect_request("/v1/cluster").respond_with_data(
        "", status=401, content_type="application/json"
    )
    with pytest.raises(RedisEnterpriseError):
        client.get_sync("/v1/cluster")


def test_get_raw_404_raises_value_error(httpserver, client):
    httpserver.expect_request("/v1/bdbs/999").respond_with_data(
        "", status=404, content_type="application/json"
    )
    with pytest.raises(ValueError):
        client.get_sync("/v1/bdbs/999")


# --- delete_raw does NOT use handle_response: all errors → RuntimeError ---

def test_delete_raw_404_raises_runtime_error(httpserver, client):
    """delete_raw bypasses handle_response; 404 → ApiError → RuntimeError (not ValueError)."""
    httpserver.expect_request("/v1/bdbs/999", method="DELETE").respond_with_data(
        "", status=404, content_type="application/json"
    )
    with pytest.raises(RuntimeError):
        client.delete_sync("/v1/bdbs/999")


def test_delete_raw_401_raises_runtime_error(httpserver, client):
    """delete_raw bypasses handle_response; 401 → ApiError → RuntimeError (not RedisEnterpriseError)."""
    httpserver.expect_request("/v1/bdbs/1", method="DELETE").respond_with_data(
        "", status=401, content_type="application/json"
    )
    with pytest.raises(RuntimeError):
        client.delete_sync("/v1/bdbs/1")
