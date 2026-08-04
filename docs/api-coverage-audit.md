# API Coverage Audit

This generated report compares unique documented `METHOD + normalized path`
operations with distinct static evidence from handlers, Wiremock matchers,
request-shape matchers, response fixtures, fixture deserialization, and live
validation annotations.

## Summary

- Raw documented inventory rows: `209`
- Unique documented operations: `203`
- Collapsed duplicate aliases: `6`
- Handler declarations: `195`
- Wiremock method/path matchers: `132`
- Explicit Wiremock call-count expectations: `2`
- Request body matchers: `25`
- Query matchers: `5`
- Response fixtures: `120`
- Explicit fixture-deserialization evidence: `7`
- Explicit live evidence: `13`

### Dispositions

- `handler_with_asserted_mock`: `2`
- `handler_with_mock_evidence`: `130`
- `handler_without_mock_evidence`: `63`
- `mock_without_handler`: `0`
- `docs_only`: `8`

## Interpretation Limits

- A handler declaration means the scanner found a recognized REST transport call;
  it does not establish request or response correctness.
- A Wiremock method/path matcher is stronger than a string mention, but only an
  explicit call-count expectation proves that Wiremock itself requires the call.
- Body and query columns report exact matcher presence independently. A matcher
  does not prove the documented schema is complete.
- A response fixture means a matching mock supplies a body. Explicit response
  annotations identify standalone typed-fixture deserialization tests.
- Live evidence is counted only from `api-audit-live` annotations attached to
  opt-in tests. This report does not claim those ignored tests ran in CI.
- Unscoped path literals and comments never count as behavioral evidence.
- Static extraction recognizes the transport methods and `impl_crud!` forms used
  in this repository. Dynamic path construction remains a review item.

## Highest-Priority Follow-ups

- `GET /v1/boostrap`: no handler or matching mock evidence
- `POST /v1/bdbs/alerts/{uid}`: no handler or matching mock evidence
- `POST /v1/bdbs/{uid}/modules/config`: no handler or matching mock evidence
- `POST /v1/bdbs/{uid}/modules/upgrade`: no handler or matching mock evidence
- `POST /v1/bdbs/{uid}/passwords`: no handler or matching mock evidence
- `PUT /v1/bdbs/{uid}/passwords`: no handler or matching mock evidence
- `PUT /v1/bdbs/{uid}/{action}`: no handler or matching mock evidence
- `PUT /v1/cluster/certificates`: no handler or matching mock evidence
- `DELETE /v1/cluster/actions/{action}`: handler has no matching mock evidence
- `DELETE /v1/cluster/auditing/db_conns`: handler has no matching mock evidence
- `DELETE /v1/cluster/certificates/{certificate_name}`: handler has no matching mock evidence
- `GET /v1/actions/bdb/{bdb_uid}`: handler has no matching mock evidence
- `GET /v1/bdbs/alerts/{uid}`: handler has no matching mock evidence
- `GET /v1/bdbs/alerts/{uid}/{alert}`: handler has no matching mock evidence
- `GET /v1/bdbs/crdt_sources/alerts`: handler has no matching mock evidence
- `GET /v1/bdbs/crdt_sources/alerts/{uid}`: handler has no matching mock evidence
- `GET /v1/bdbs/crdt_sources/alerts/{uid}/{crdt_src_id}`: handler has no matching mock evidence
- `GET /v1/bdbs/replica_sources/alerts`: handler has no matching mock evidence
- `GET /v1/bdbs/replica_sources/alerts/{uid}`: handler has no matching mock evidence
- `GET /v1/bdbs/replica_sources/alerts/{uid}/{replica_src_id}`: handler has no matching mock evidence
- `GET /v1/bdbs/replica_sources/alerts/{uid}/{replica_src_id}/{alert}`: handler has no matching mock evidence
- `GET /v1/bdbs/stats/last`: handler has no matching mock evidence
- `GET /v1/bdbs/stats/last/{uid}`: handler has no matching mock evidence
- `GET /v1/bdbs/stats/{uid}`: handler has no matching mock evidence
- `GET /v1/bdbs/{uid}/actions/recover`: handler has no matching mock evidence
- `GET /v1/bdbs/{uid}/availability`: handler has no matching mock evidence
- `GET /v1/bdbs/{bdb_uid}/peer_stats/{uid}`: handler has no matching mock evidence
- `GET /v1/bdbs/{bdb_uid}/sync_source_stats`: handler has no matching mock evidence

## Reproduce

```bash
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
python3 scripts/audit_api_coverage.py
git diff --exit-code -- docs/api-coverage-audit.csv docs/api-coverage-audit.md
```

## Artifacts

- CSV audit: [api-coverage-audit.csv](./api-coverage-audit.csv)
