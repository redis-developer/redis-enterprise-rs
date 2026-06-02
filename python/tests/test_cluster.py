"""Tests for cluster-related Python bindings."""

import pytest
from redis_enterprise import EnterpriseClient


CLUSTER_INFO = {"name": "test-cluster", "nodes": [1, 2]}
CLUSTER_STATS = {"intervals": [], "total": {}}
LICENSE_INFO = {"expired": False, "shards_limit": 100, "expiration_date": "2030-12-31"}


def test_cluster_info_sync(httpserver, client):
    httpserver.expect_request("/v1/cluster").respond_with_json(CLUSTER_INFO)
    result = client.cluster_info_sync()
    assert result["name"] == "test-cluster"
    assert result["nodes"] == [1, 2]


async def test_cluster_info_async(httpserver, client):
    httpserver.expect_request("/v1/cluster").respond_with_json(CLUSTER_INFO)
    result = await client.cluster_info()
    assert result["name"] == "test-cluster"
    assert result["nodes"] == [1, 2]


def test_cluster_stats_sync(httpserver, client):
    httpserver.expect_request("/v1/cluster/stats").respond_with_json(CLUSTER_STATS)
    result = client.cluster_stats_sync()
    assert isinstance(result, dict)


async def test_cluster_stats_async(httpserver, client):
    httpserver.expect_request("/v1/cluster/stats").respond_with_json(CLUSTER_STATS)
    result = await client.cluster_stats()
    assert isinstance(result, dict)


def test_license_sync(httpserver, client):
    httpserver.expect_request("/v1/license").respond_with_json(LICENSE_INFO)
    result = client.license_sync()
    assert result["expired"] is False
    assert result["shards_limit"] == 100


async def test_license_async(httpserver, client):
    httpserver.expect_request("/v1/license").respond_with_json(LICENSE_INFO)
    result = await client.license()
    assert result["expired"] is False
    assert result["shards_limit"] == 100
