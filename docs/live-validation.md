# Live Validation

This crate has extensive mock and fixture coverage, but live validation is still
important for catching schema drift, undocumented fields, and version-specific
behavior in Redis Enterprise.

This document describes the current live-validation path for local development.
The required release families and compatibility rules are defined in the
[version support policy](./version-support.md).

## Sources

- Official Docker quickstart for Redis Enterprise Software:
  [redis.io/docs/latest/operate/rs/7.22/installing-upgrading/quickstarts/docker-quickstart/](https://redis.io/docs/latest/operate/rs/7.22/installing-upgrading/quickstarts/docker-quickstart/)
- Official REST API reference landing page:
  [redis.io/docs/latest/operate/rs/references/rest-api/](https://redis.io/docs/latest/operate/rs/references/rest-api/)
- Official REST API requests index:
  [redis.io/docs/latest/operate/rs/references/rest-api/requests/](https://redis.io/docs/latest/operate/rs/references/rest-api/requests/)
- Redis Software product lifecycle:
  [redis.io/docs/latest/operate/rs/installing-upgrading/product-lifecycle/](https://redis.io/docs/latest/operate/rs/installing-upgrading/product-lifecycle/)
- Redis Software release notes:
  [redis.io/docs/latest/operate/rs/release-notes/](https://redis.io/docs/latest/operate/rs/release-notes/)

## Local Docker Path

The repo includes a `docker-compose.yml` for local Redis Enterprise testing.
This matches the official Redis guidance that Docker is suitable for development
and test environments.

The compose file defaults to:

- `REDIS_ENTERPRISE_IMAGE=redislabs/redis:8.2.0-25.12`
- `REDIS_ENTERPRISE_API_PORT=9443`
- `REDIS_ENTERPRISE_UI_PORT=8443`
- `REDIS_ENTERPRISE_DB_PORT=12000`

You can override the host ports to avoid collisions with an existing local
cluster:

```bash
REDIS_ENTERPRISE_API_PORT=19443 \
REDIS_ENTERPRISE_UI_PORT=18443 \
REDIS_ENTERPRISE_DB_PORT=22000 \
docker compose up -d redis-enterprise
```

Initialize the cluster once the container is healthy:

```bash
REDIS_ENTERPRISE_API_PORT=19443 \
REDIS_ENTERPRISE_UI_PORT=18443 \
REDIS_ENTERPRISE_DB_PORT=22000 \
docker compose --profile init up init
```

If you use the default ports, you can omit those overrides. If you use custom
ports for `up`, reuse the same overrides for `init` so Compose treats the stack
as the same service definition.

## Required Version Matrix

Release validation uses the exact images in the
[version support policy](./version-support.md):

| Family | Image |
|---|---|
| 8.2 | `redislabs/redis:8.2.0-25.12` |
| 8.0 | `redislabs/redis:8.0.20-68.25` |
| 7.22 | `redislabs/redis:7.22.2-170` |
| 7.8 | `redislabs/redis:7.8.6-286` |
| 7.4 | `redislabs/redis:7.4.6-272` |

The Compose default is the newest family. To test another family, export its
pin before every Compose command for that cluster lifecycle:

```bash
export REDIS_ENTERPRISE_IMAGE="redislabs/redis:7.4.6-272"
docker compose up -d redis-enterprise
docker compose --profile init up init
```

Each matrix entry must use a fresh named volume or run after `docker compose
down -v`; cluster state is not portable between server versions. Docker images
are for development and testing only, as stated in the official Redis
documentation.

The init profile configures:

- cluster name: `test-cluster`
- admin user: `admin@redis.local`
- admin password: `Redis123!`

If you prefer, you can also initialize through the UI on the configured admin
UI port, such as `https://localhost:8443`.

## Licensing for local validation

Redis Enterprise runs in trial mode by default when no license is uploaded. The
trial allows enough shards and features for SDK validation against every
endpoint covered by `live_enterprise_smoke.rs`. There is no need to apply for a
trial key to bring up the local validation cluster.

If you do need to test license-gated behavior, the Admin UI accepts a license
file via Cluster -> License. Treat that license file the same way as any other
credential — never commit it to the repository.

## TLS

The local Redis Enterprise cluster ships with a self-signed certificate. You
have three options for talking to it from this SDK:

1. **Insecure mode (recommended for the local runbook)** — set
   `REDIS_ENTERPRISE_INSECURE=true`. The client skips certificate validation.
   Use only against trusted local development clusters.
2. **Trust the self-signed cert** — set `REDIS_ENTERPRISE_CA_CERT` to a PEM
   file containing the cluster's self-signed CA. Recommended whenever you need
   to validate the full TLS path (for example when reproducing a TLS-related
   bug). The cluster exposes its CA through the Admin UI under Cluster ->
   Security, or directly via `docker compose exec redis-enterprise
   /opt/redislabs/bin/openssl s_client -showcerts -connect 127.0.0.1:9443`.
3. **Real TLS** — for cloud or shared clusters, configure the cluster with a
   certificate signed by a CA your client already trusts and leave both
   `REDIS_ENTERPRISE_INSECURE` and `REDIS_ENTERPRISE_CA_CERT` unset.

Production deployments should never use option 1. Bias toward option 2 in
shared dev environments so that a misconfigured DNS name fails loudly.

## Smoke Checks

Export the client environment variables:

```bash
export REDIS_ENTERPRISE_URL="https://localhost:9443"
export REDIS_ENTERPRISE_USER="admin@redis.local"
export REDIS_ENTERPRISE_PASSWORD="Redis123!"
export REDIS_ENTERPRISE_INSECURE="true"
```

If you started the stack on alternate ports, point `REDIS_ENTERPRISE_URL` at the
chosen API port instead.

Run the example client:

```bash
cargo run --example basic_enterprise
```

Run the opt-in live smoke test:

```bash
cargo test --test live_enterprise_smoke -- --ignored
```

## Inventory Compliance Matrix

`tests/live_compliance.rs` turns the checked-in API inventory into one visible
result for every documented method and path. Normal test runs exercise its CSV
parsing, resource resolution, sanitization, and baseline comparison without a
server. The ignored live test:

- probes safe `GET` operations and skips debug-info downloads;
- resolves path parameters only from resources discovered on that cluster;
- compares selected raw responses with their typed model round trips;
- records the actual server version and discovered API capabilities;
- classifies every operation as `pass`, `known_difference`,
  `version_specific`, `skipped`, `unsupported`, or `fail`; and
- writes only status metadata, JSON field paths, and sanitized error classes.
  It never writes response bodies, credentials, resource IDs, or field values.

To create a candidate for review against the pinned 8.2 image, first export the
client variables from the smoke-check section, then run:

```bash
REDIS_ENTERPRISE_EXPECTED_VERSION="8.2.0-25" \
REDIS_ENTERPRISE_IMAGE="redislabs/redis:8.2.0-25.12" \
REDIS_ENTERPRISE_COMPLIANCE_RECORD=true \
cargo test --test live_compliance live_inventory_compliance -- --ignored --nocapture
```

The sanitized report and one-family baseline candidate are written under
`target/` by default. Override their locations with
`REDIS_ENTERPRISE_COMPLIANCE_OUTPUT` and
`REDIS_ENTERPRISE_COMPLIANCE_BASELINE_OUTPUT`. Review the candidate before
merging its `safe` profile into
`tests/fixtures/live_compliance_baseline.json`; recording never updates the
checked-in baseline automatically.

After a baseline is reviewed, omit `REDIS_ENTERPRISE_COMPLIANCE_RECORD` to make
any operation, status-code, typed-model, or dropped-field drift fail the run:

```bash
REDIS_ENTERPRISE_EXPECTED_VERSION="8.2.0-25" \
REDIS_ENTERPRISE_IMAGE="redislabs/redis:8.2.0-25.12" \
cargo test --test live_compliance live_inventory_compliance -- --ignored --nocapture
```

Run the disposable user lifecycle separately by adding
`REDIS_ENTERPRISE_LIVE_WRITES=true`. Write-enabled reports use a distinct
`writes` baseline profile so safe-read runs cannot accidentally accept or
invalidate write coverage. The lifecycle removes stale compliance-prefixed
users before it begins, which cleans up resources left by an interrupted prior
run, and attempts follow-up cleanup on create or delete errors as well as the
normal delete before returning. Do not run write-enabled compliance jobs
concurrently against the same cluster. All other writes stay skipped with an
explicit reason until they have a self-cleaning implementation.

A missing family or profile baseline is a test failure, not implicit approval.
Each required version in the support matrix must be recorded on its exact image
and reviewed independently.

## Teardown and cleanup

The validation cluster is fully disposable. Tear it down with:

```bash
docker compose down -v
```

`-v` removes the named volume that holds the cluster state (license,
configuration, databases). Re-running `docker compose up -d` after a `-v`
teardown starts from a fresh, uninitialized cluster, so the `init` profile
must run again.

If you want to keep cluster state between runs (faster iteration, but cluster
config drifts as you make changes), drop the `-v`:

```bash
docker compose down
docker compose up -d
```

To rebuild the entire local environment from scratch (for example after a
Redis Enterprise image bump), remove the image as well:

```bash
docker compose down -v --rmi local
docker image rm "${REDIS_ENTERPRISE_IMAGE:-redislabs/redis:8.2.0-25.12}"
docker compose pull
docker compose up -d
```

## Current Validation Snapshot

On April 21, 2026, the existing `basic_enterprise` example completed
successfully against a local Redis Enterprise Software `8.0.10-81` cluster and
returned live data for:

- `GET /v1/cluster`
- `GET /v1/nodes`
- `GET /v1/bdbs`

That validates the current typed client against a real cluster for a minimal
read-only flow, but it does not yet establish full endpoint completeness.

This snapshot is historical evidence, not the complete support matrix. Record
future runs with the exact product version, image tag, date, and test scope.

## Remaining Coverage

The matrix starts with all safe reads and a disposable user CRUD lifecycle.
Additional write routes should move out of `skipped` only when they have a
self-cleaning implementation and an explicit safety review. Destructive cluster
operations remain outside the default live suite.
