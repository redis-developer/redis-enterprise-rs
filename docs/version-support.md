# Redis Software Version Support

This policy defines which Redis Software server releases the
`redis-enterprise` crate is expected to work with. It covers the Cluster
Manager REST API exposed on port 9443; it does not describe compatibility
between a Redis client and the Redis database protocol.

## Supported Release Families

Crate releases built under this policy support every Redis Software release
family that Redis lists as active on the release date. Support is exercised
against one pinned maintenance container per family. Older patches in the same
family are best effort: users should run a current maintenance build before
reporting a compatibility defect.

As of 2026-07-31, the required matrix is:

| Redis Software family | Lifecycle end | Validation image |
|---|---|---|
| 8.2 | Determined after the next major release | `redislabs/redis:8.2.0-25.12` |
| 8.0 | 2028-07-31 | `redislabs/redis:8.0.20-68.25` |
| 7.22 | 2027-10-30 | `redislabs/redis:7.22.2-170` |
| 7.8 | 2027-05-30 | `redislabs/redis:7.8.6-286` |
| 7.4 | 2026-11-30 | `redislabs/redis:7.4.6-272` |

The image tags above were verified in the official `redislabs/redis` Docker
Hub repository on 2026-07-31. A Docker packaging suffix can differ from the
Redis Software product build shown in release notes. For example,
`8.2.0-25.12` packages Redis Software `8.2.0-25`.
The newest public container for a family can also lag the newest product build;
the matrix advances only to tags that are actually available to test.

Redis Software versions that have reached end of life, prerelease builds, and
older maintenance patches are not release-blocking. Reports against them are
welcome, but fixes must not weaken behavior on the supported matrix.

Sources:

- [Redis Software product lifecycle](https://redis.io/docs/latest/operate/rs/installing-upgrading/product-lifecycle/)
- [Redis Software 8.2 release notes](https://redis.io/docs/latest/operate/rs/release-notes/rs-8-2-releases/)
- [Redis Software release notes](https://redis.io/docs/latest/operate/rs/release-notes/)
- [Official Redis Software Docker images](https://hub.docker.com/r/redislabs/redis/tags)

## What Support Means

For every supported family:

- the client must connect over HTTPS, authenticate, and preserve structured API
  errors;
- the safe live suite must pass for cluster, node, database, RBAC, and other
  non-destructive reads represented by that server version;
- disposable write tests must create, read, update, and remove their own test
  resources without depending on shared state;
- documented version differences may produce an explicit unsupported or API
  error, but must not silently send a different operation;
- typed response models must accept additive fields and accurately deserialize
  fields that the crate exposes.

Support does not mean every method exists on every server. Redis adds,
deprecates, and removes REST operations and fields between release families.
Public methods with a known minimum or maximum server version must say so in
their Rust documentation. Calling such a method outside its documented range
returns the server's response through the normal error model.

## API Classification

The public surface uses these classifications:

1. **Stable documented**: present in official documentation for at least one
   supported family and covered by method/path tests.
2. **Version-specific**: documented or live-verified only for named release
   families. The method remains public with an explicit version note.
3. **Compatibility or legacy**: retained for existing users while a documented
   replacement exists. It must be deprecated in Rust before removal.
4. **Verified undocumented**: accepted by named server builds but absent from
   public documentation. It must carry provenance and is not assumed portable.
5. **Internal or unsupported**: not part of the stable public contract. New
   code should use raw requests only when access is intentionally required.

The `v1` and `v2` prefixes are separate wire contracts, not interchangeable
aliases. The crate can expose both when both are supported, but must not
silently reroute a call between them.

## Documentation and Evidence

The official version-specific Redis Software request and object references are
the primary source. The generated inventory based on the `latest` selector is a
discovery aid, not a universal schema. When sources disagree, evidence is
ranked as follows:

1. version-specific official documentation;
2. sanitized live behavior from a pinned supported image;
3. checked-in request and response fixtures with server-version provenance;
4. generic `latest` documentation;
5. historical mocks without live or documentation provenance.

Live results must record the exact product version, container tag, test date,
and whether the operation was read-only or used disposable resources. Raw
production payloads, credentials, license files, and customer identifiers must
never be committed.

## Release Criteria

Before publishing a crate release:

- formatting, Clippy, unit, fixture, and route-coverage checks must pass;
- the safe live suite must pass on every pinned supported family;
- disposable write lifecycles must pass on at least the oldest and newest
  supported families;
- every skip or known difference must identify the affected operation,
  release family, and rationale in a reviewed baseline;
- the support table and container pins must be reviewed against Redis's current
  lifecycle and release notes.

The weekly [Enterprise API contract workflow](./contract-automation.md) runs the
safe profile across the full pinned image matrix and publishes sanitized
per-version artifacts. Self-cleaning write lifecycles remain an explicit manual
dispatch option. Release review should inspect the newest scheduled result plus
[the live-validation log](./live-validation.md).

## Adding and Retiring Server Versions

A newly generally available Redis Software family enters the support matrix in
a crate release after an official test image is available and the safe live
suite passes. New version-specific fields should normally be added as optional
fields or tolerant enums so older supported responses continue to deserialize.

When Redis marks a family end of life, the next crate release may remove it
from the required matrix. Public methods are not removed merely because the
only server family that implemented them reached end of life: they first follow
the crate's normal Rust deprecation and semantic-versioning policy.

Server-side changes are handled as follows:

- additive models, new methods, and support for a new family are normally
  backward-compatible crate changes;
- correcting an incorrectly encoded request or response is a bug fix when it
  restores the documented wire contract;
- removing or incompatibly changing a public Rust API requires a semver-major
  release unless the API was already explicitly unstable;
- unavoidable server incompatibilities are documented by release family and
  tested as known differences rather than hidden behind automatic fallbacks.
