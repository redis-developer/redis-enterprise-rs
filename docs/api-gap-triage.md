# API Gap Triage

Reviewed on April 21, 2026 against:

- official Redis Software REST docs under `latest` / `v7.22`
- a live Redis Enterprise Software `8.0.10-81` cluster

This document sits on top of the generated path audit in
[api-coverage-audit.md](./api-coverage-audit.md). The generated audit answers
"which documented paths do not have exact string evidence in `src/` or `tests/`".
This triage answers "which of those are real SDK gaps".

## Resolved In This Pass

- `POST /v1/crdb_tasks/{task_id}/actions/cancel`
  - `CrdbTasksHandler::cancel()` was using `DELETE /v1/crdb_tasks/{task_id}`.
  - Fixed to use the live route and added `cancel_with_force()`.
- `GET /v1/shards/stats/{uid}`
  - `StatsHandler::shard()` was using the wrong path shape.
  - Fixed to use `/v1/shards/stats/{uid}`.
- `GET /v1/shards/stats/last`
- `GET /v1/shards/stats/last/{uid}`
  - Added `StatsHandler::shards_last()` and `StatsHandler::shard_last()`.
- `GET /v1/cluster/check`
  - Added `ClusterHandler::check()`.
- `GET /v1/nodes/check/{uid}`
  - Added `NodeHandler::check()`.
- Shard stats payload shape
  - Live shard intervals use flat objects with `stime` / `etime`, not only `{ time, metrics }`.
  - `StatsInterval` now accepts both wire formats.

## Remaining Actionable SDK Gaps

- [#54](https://github.com/redis-developer/redis-enterprise-rs/issues/54) `crdb: add flush, purge, health_report, and updates endpoints`
- [#55](https://github.com/redis-developer/redis-enterprise-rs/issues/55) `modules: add user-defined module management endpoints`
- [#56](https://github.com/redis-developer/redis-enterprise-rs/issues/56) `cluster: add SSO endpoint coverage`
- [#57](https://github.com/redis-developer/redis-enterprise-rs/issues/57) `cluster: add change_password_hashing_algorithm coverage`

## Docs Artifacts And Path Mismatches

- `GET /v1/boostrap`
  - The docs table contains a typo.
  - Live `GET /v1/bootstrap` returned `200 OK`.
  - Live `GET /v1/boostrap` returned `404 Not Found`.
- `POST /v1/bdbs/{uid}/modules/config`
  - Live `POST /v1/bdbs/1/modules/config` returned the same schema error as the existing SDK path
    `/v1/modules/config/bdb/1`.
  - Treat this as an alias, not a missing feature.
- `POST /v1/bdbs/{uid}/modules/upgrade`
  - Live `POST /v1/bdbs/1/modules/upgrade` returned `404 Not Found` on `8.0.10-81`.
  - This currently looks like version skew or stale docs rather than a safe SDK target.
- `PUT /v1/bdbs/{uid}/{action}`
  - Docs describe a generic "update and perform action" route.
  - The SDK already exposes some explicit operations such as `flush()`, but does not expose this
    generic wrapper.
  - This is not filed yet because the safe public API shape is still unclear.

## Existing Version-Skew Findings

- Documented `suffix` endpoints returned `404 Not Found` on Redis Enterprise Software `8.0.10-81`
  during live validation, even though the SDK already has `SuffixesHandler`.
- The official docs inventory used here comes from the Redis docs `latest` tree, which currently
  resolves to the `v7.22` Redis Software docs, while the live validation cluster was `8.0.10-81`.

## Validation Notes

- Mock coverage updated for the newly fixed routes.
- Opt-in live smoke coverage now exercises:
  - cluster info, node list, database list
  - cluster check and node check
  - single-shard stats and shard last stats
  - user create/get/delete on an RBAC-enabled cluster
