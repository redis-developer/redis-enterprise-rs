# API Coverage Audit

This report compares the generated official-doc inventory to exact path
evidence found in `src/` and `tests/`.

## Summary

- Total documented endpoints audited: `209`
- `implemented_and_tested`: `77`
- `implemented_no_test_path_evidence`: `127`
- `test_only_path_evidence`: `0`
- `docs_only`: `5`

## Notes

- This is an initial path-based audit, not a semantic guarantee that request and response
  models are correct.
- `docs_only` rows are the most likely follow-up candidates, but some may be explained by
  path aliases, version-specific availability, or endpoint families that are expressed
  differently in the SDK than in the docs.
- Live validation on April 21, 2026 found that:
  - `POST /v1/users` on an RBAC-enabled cluster requires `role_uids` without `role`.
  - documented `suffix` endpoints returned `404 Not Found` on Redis Enterprise Software `8.0.10-81`.
  - creating a disposable database hit shard-license limits on the local trial cluster.

## Likely Follow-ups

- `PUT /v1/bdbs/{uid}/{action}` from `bdbs` (module guess: `bdb`)
- `POST /v1/bdbs/{uid}/modules/config` from `bdbs/modules` (module guess: `modules`)
- `POST /v1/bdbs/{uid}/modules/config` from `bdbs/modules/config`
- `POST /v1/bdbs/{uid}/modules/upgrade` from `bdbs/modules/upgrade`
- `GET /v1/boostrap` from `bootstrap` (module guess: `bootstrap`)

## Artifacts

- CSV audit: [api-coverage-audit.csv](./api-coverage-audit.csv)
