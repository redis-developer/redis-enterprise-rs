# Non-Inventory Route Evidence

The SDK originally contained 95 method/path pairs that were not present in the
generated public Redis Software REST API inventory. The remaining entries are
tracked in the machine-readable
[`live_non_inventory_routes.json`](../tests/fixtures/live_non_inventory_routes.json)
registry. A route's presence in that registry is not evidence that it is a
supported public API.

## Reviewed dispositions

| Disposition | Count | Meaning |
|---|---:|---|
| `verified_undocumented` | 13 | The exact method/path is registered by at least one current 8.x family, but is absent from the public-doc inventory. Keep it visible as an intentional compatibility exception while documentation and behavioral coverage are evaluated. |
| `compatibility_legacy` | 9 | The method/path is registered in 7.4, 7.8, and 7.22, but not in 8.0 or 8.2. Keep only for the crate's older supported families and do not recommend it for new integrations. |
| `invalid` | 49 | None of the five supported families registers the claimed method/path. Treat the handler as a removal, deprecation, or canonical-path correction candidate. |

Seven original invalid routes have already left the registry because their
public methods now use the documented collection-first alert and statistics
paths, and node removal now uses the documented node action. Six more live
routes are recognized as literal values of documented `{action}` templates
instead of false non-inventory positives. The shard stats handler was also
moved to its documented collection-first path. The registry therefore contains
81 exceptions and cleanup candidates at the first checkpoint. Six additional
invented aliases or relationship paths now delegate to canonical collection
routes: cluster license, cluster suffixes, CRDB tasks by CRDB, proxies by
database or node, and shards by node. The registry therefore contains 75
exceptions and cleanup candidates at the second checkpoint. Database metrics
and users-by-role now also delegate to their canonical statistics and users
collections. The registry therefore contains 73 current exceptions and cleanup
candidates at the third checkpoint. Per-endpoint statistics now select from the
documented global endpoint-statistics collection, leaving 72 current exceptions
and cleanup candidates at the fourth checkpoint. Single-metric shard statistics
now select the requested values from the documented shard-statistics response,
leaving 71 current exceptions and cleanup candidates.

The 13 current compatibility exceptions are:

- all five `/v1/bdb_groups` CRUD operations;
- `GET /v1/bootstrap`;
- `PUT /v1/cluster/policy/restore_default`;
- `DELETE /v1/suffix/{name}`;
- `PATCH /v1/suffix/{name}`, registered in 8.0 and 8.2. The previous SDK
  `PUT` implementation was absent in every tested family;
- `GET /v1/cluster/witness_disk`, `GET /v1/nodes/wd_status`,
  `GET /v1/nodes/{uid}/wd_status`, and
  `POST /v1/nodes/{uid}/snapshots/{name}`. These four are absent in 7.4 but
  registered in 7.8 and newer supported families.

The nine legacy operations are v1/v2 module upload or mutation, Redis ACL
validation, and user password/permission helpers. Their exact method/path
pairs and tested versions are in the registry.

## Evidence matrix

The review used exact Docker images from the
[version support policy](./version-support.md):

| Redis Software | Claimed method registered | Claimed method absent |
|---|---:|---:|
| 7.4.6-272 | 17 | 54 |
| 7.8.6-286 | 21 | 50 |
| 7.22.2-170 | 21 | 50 |
| 8.0.20-68 | 13 | 58 |
| 8.2.0-25 | 13 | 58 |

The runs were completed on August 4, 2026. Every path was probed with
`OPTIONS`; no route mutation was executed. Redis Software returns `404` for an
unregistered path and an `Allow` header for a registered path. The audit only
accepts a route as present when the expected SDK method appears in that
header. A route can therefore be present while a different SDK method is
invalid, such as a claimed `PUT` where the server registers `PATCH`.

This is path-and-method registration evidence, not full behavioral evidence.
It does not prove request schema accuracy, response model accuracy, permissions,
or side effects. Those require self-cleaning live lifecycles or safe read probes.

## Keeping the registry honest

`tests/route_coverage.rs` compares source-extracted handlers to the public-doc
inventory and the evidence registry. A new unexplained handler route fails the
test, and a registry entry becomes stale when its handler is removed or moved
to a documented canonical path.

`tests/live_non_inventory_routes.rs` validates registry shape and dispositions
in normal test runs. Its ignored live test reruns the non-mutating `OPTIONS`
matrix against one exact supported version and fails on either kind of drift:
a reviewed route disappearing or a reviewed invalid route unexpectedly
appearing.

The next cleanup step is to correct, deprecate, or remove the 49 invalid
handlers module by module, checking downstream redisctl use before making a
breaking API change. The registry should shrink as those decisions land; it is
not a permanent exceptions budget.
