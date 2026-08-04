# Model Fidelity

The endpoint inventory proves that a method and path exist. Model-fidelity
evidence separately proves that public Rust types can consume and preserve the
fields Redis Software sends on those paths.

## Evidence sources

The checked-in evidence combines:

- the official Redis Software object references for
  [databases](https://redis.io/docs/latest/operate/rs/references/rest-api/objects/bdb/),
  [shards](https://redis.io/docs/latest/operate/rs/references/rest-api/objects/shard/),
  and [users](https://redis.io/docs/latest/operate/rs/references/rest-api/objects/user/);
- sanitized field-path observations in
  [`tests/fixtures/live_compliance_baseline.json`](../tests/fixtures/live_compliance_baseline.json)
  from Redis Software 7.4, 7.8, 7.22, 8.0, and 8.2;
- synthetic, non-sensitive response and request shapes in
  [`tests/fixtures/model_fidelity.json`](../tests/fixtures/model_fidelity.json).

The synthetic fixture stores no cluster names, addresses, credentials, license
values, or raw production responses. Its metadata records the capture date,
release families, source links, and sanitization policy.

## Compatibility strategy

Stable documented fields are strongly typed. Examples include an endpoint's
`oss_cluster_api_preferred_endpoint_type`, shard loading and detailed status,
and the user `last_login` timestamp. The 8.2 node ingress-throttling limit uses
a signed integer because the live API uses a negative sentinel for its default
state.

Cluster, database, node, shard, user, license, and endpoint objects also retain
unknown fields in `additional_fields`. Redis Software adds feature flags,
capability maps, and controls between supported release families; preserving
those values is more accurate than discarding them or assigning an unstable
field one false cross-version type. A field should move from
`additional_fields` to a named typed field once the object reference or
multi-version live evidence establishes a stable contract.

These server-owned response structs are `#[non_exhaustive]`. Consumers can
read their public fields and deserialize fixtures, while future typed field
promotions do not repeatedly break downstream struct literals.

An absent field and a JSON `null` both deserialize to `Option::None`. The live
round-trip comparison therefore treats a missing reserialized `null` as
information-preserving. Any missing non-null value remains a model-fidelity
failure. Arrays must also preserve their length and nested non-null fields.

## Regression gates

`tests/model_fidelity_tests.rs` enforces three properties:

1. Core response models deserialize and reserialize without losing non-null
   fields, including version-specific fields retained in `additional_fields`.
2. Promoted fields have the documented Rust types and values.
3. Public database and user request builders serialize to exact wire fixtures,
   including `data_persistence` rather than the historical `persistence` key.

`tests/live_compliance.rs` applies the same non-null field-loss rule to live raw
and typed responses. The versioned baseline is a reviewed allowlist: a newly
dropped non-null path fails, while a path disappearing because the model
improved is accepted.
