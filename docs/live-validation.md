# Live Validation

This crate has extensive mock and fixture coverage, but live validation is still
important for catching schema drift, undocumented fields, and version-specific
behavior in Redis Enterprise.

This document describes the current live-validation path for local development.

## Sources

- Official Docker quickstart for Redis Enterprise Software:
  [redis.io/docs/latest/operate/rs/7.22/installing-upgrading/quickstarts/docker-quickstart/](https://redis.io/docs/latest/operate/rs/7.22/installing-upgrading/quickstarts/docker-quickstart/)
- Official REST API reference landing page:
  [redis.io/docs/latest/operate/rs/references/rest-api/](https://redis.io/docs/latest/operate/rs/references/rest-api/)
- Official REST API requests index:
  [redis.io/docs/latest/operate/rs/references/rest-api/requests/](https://redis.io/docs/latest/operate/rs/references/rest-api/requests/)

## Local Docker Path

The repo includes a `docker-compose.yml` for local Redis Enterprise testing.
This matches the official Redis guidance that Docker is suitable for development
and test environments.

The compose file defaults to:

- `REDIS_ENTERPRISE_IMAGE=redislabs/redis:8.0.10-81`
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

The init profile configures:

- cluster name: `test-cluster`
- admin user: `admin@redis.local`
- admin password: `Redis123!`

If you prefer, you can also initialize through the UI on the configured admin
UI port, such as `https://localhost:8443`.

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

## Current Validation Snapshot

On April 21, 2026, the existing `basic_enterprise` example completed
successfully against a local Redis Enterprise Software `8.0.10-81` cluster and
returned live data for:

- `GET /v1/cluster`
- `GET /v1/nodes`
- `GET /v1/bdbs`

That validates the current typed client against a real cluster for a minimal
read-only flow, but it does not yet establish full endpoint completeness.

## Next Steps

- Expand the ignored live smoke suite beyond cluster, node, and database reads
- Add disposable-resource CRUD flows for higher-confidence validation
- Compare live responses against the generated API inventory in
  [api-inventory.md](/Users/josh.rotenberg/Code/active/redis-enterprise-rs/docs/api-inventory.md)
