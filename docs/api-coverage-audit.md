# API Coverage Audit

This report compares the generated official-doc inventory to exact path
evidence found in `src/` and `tests/`.

## Summary

- Total documented endpoints audited: `206`
- `implemented_and_tested`: `66`
- `implemented_no_test_path_evidence`: `120`
- `test_only_path_evidence`: `0`
- `docs_only`: `20`

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
- `PATCH /v1/cluster/change_password_hashing_algorithm` from `cluster/change_password_hashing_algorithm`
- `GET /v1/cluster/sso` from `cluster/sso`
- `PUT /v1/cluster/sso` from `cluster/sso`
- `DELETE /v1/cluster/sso` from `cluster/sso`
- `GET /v1/cluster/sso/saml/metadata/sp` from `cluster/sso`
- `POST /v1/cluster/sso/saml/metadata/idp` from `cluster/sso`
- `PUT /v1/crdbs/{crdb_guid}/flush` from `crdbs/flush`
- `GET /v1/crdbs/{crdb_guid}/health_report` from `crdbs/health_report`
- `PUT /v1/crdbs/{crdb_guid}/purge` from `crdbs/purge`
- `POST /v1/crdbs/{crdb_guid}/updates` from `crdbs/updates`
- `GET /v2/local/modules/user-defined/artifacts` from `modules/user-defined`
- `POST /v2/modules/user-defined` from `modules/user-defined`
- `POST /v2/local/modules/user-defined/artifacts` from `modules/user-defined`
- `DELETE /v2/modules/user-defined/<uid>` from `modules/user-defined`
- `DELETE /v2/local/modules/user-defined/artifacts/<module_name>/<version>` from `modules/user-defined`

## Artifacts

- CSV audit: [api-coverage-audit.csv](./api-coverage-audit.csv)
