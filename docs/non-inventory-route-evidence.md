# Non-Inventory Route Evidence

The SDK originally contained 95 method/path pairs that were not present in the
generated public Redis Software REST API inventory. Active compatibility routes
and the retired invalid paths are tracked separately in the machine-readable
[`live_non_inventory_routes.json`](../tests/fixtures/live_non_inventory_routes.json)
registry. A route's presence in the active registry is not evidence that it is
a supported public API.

## Reviewed dispositions

| Disposition | Count | Meaning |
|---|---:|---|
| `verified_undocumented` | 13 | The exact method/path is registered by at least one current 8.x family, but is absent from the public-doc inventory. Keep it visible as an intentional compatibility exception while documentation and behavioral coverage are evaluated. |
| `compatibility_legacy` | 9 | The method/path is registered in 7.4, 7.8, and 7.22, but not in 8.0 or 8.2. Keep only for the crate's older supported families and do not recommend it for new integrations. |
| retired `invalid` | 59 | None of the five supported families registers the claimed method/path. These entries are historical evidence, not active HTTP routes. |

The 95 original entries now have a durable outcome:

| Outcome | Count |
|---|---:|
| Resolved before registry schema v2: canonical paths, documented action-template specializations, and shard stats correction | 14 |
| Active verified-undocumented compatibility routes | 13 |
| Active legacy compatibility routes | 9 |
| Retired wrong paths whose public methods now use canonical operations | 12 |
| Retired fictional operations whose deprecated methods return `RestError::UnsupportedOperation` locally | 47 |
| **Total** | **95** |

The local-error shims preserve source compatibility while ensuring the client
cannot send a request that every supported Redis Software family rejects. Their
old paths live under `retired_routes`; the active `routes` object contains only
the 22 intentional compatibility operations.

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

| Redis Software | Active method registered | Active method absent |
|---|---:|---:|
| 7.4.6-272 | 17 | 5 |
| 7.8.6-286 | 21 | 1 |
| 7.22.2-170 | 21 | 1 |
| 8.0.20-68 | 13 | 9 |
| 8.2.0-25 | 13 | 9 |

All 59 retired invalid method/path pairs were absent on all five versions.

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
inventory and the active evidence registry. A new unexplained handler route
fails the test, an active registry entry becomes stale when its handler is
removed or canonicalized, and any HTTP handler that reintroduces a retired path
also fails.

`tests/live_non_inventory_routes.rs` validates registry shape and dispositions
in normal test runs. Its ignored live test reruns the non-mutating `OPTIONS`
matrix against one exact supported version and fails when a reviewed active
route's method registration changes. Retired routes are not reprobed on every
run because the SDK no longer sends them; their five-version evidence remains
checked into the same file.

The next cleanup step is to migrate downstream redisctl commands away from the
47 deprecated local-error shims, then remove those shims in an explicitly
breaking release. The 22 active compatibility routes should receive behavioral
tests or documented replacements; they are not a permanent exceptions budget.
