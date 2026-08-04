# Enterprise API Contract Automation

The `Enterprise API contract` GitHub Actions workflow makes external Redis
Software contract drift visible without making pull-request CI depend on the
network, Docker Hub, or a live cluster.

## Cadence and ownership

The workflow runs every Monday at 06:17 UTC and can also be dispatched
manually. The redis-enterprise-rs maintainers own the result and should triage
a failed scheduled run within two business days. Artifacts are retained for 30
days so a transient external failure can be compared with a rerun.

Pull-request CI remains credential-free and deterministic. It runs unit tests
for the crawler, comparison, failure classification, and summary renderer, but
never fetches official docs or starts Redis Software.

## Official-doc inventory drift

The docs job performs three separate stages:

1. Crawl the official Redis Software request reference into a temporary CSV.
2. Compare the fresh and checked-in inventories by normalized HTTP method and
   path.
3. Upload the fresh CSV, machine-readable status, and Markdown drift report.

Descriptions, page order, duplicate rows, query strings, trailing slashes, and
placeholder names do not create route drift. The status artifacts distinguish:

- `fetch_failure`: the official source could not be reached (exporter exit 2);
- `parse_failure`: pages were fetched but no usable inventory was parsed, or a
  previously inventoried page yielded no method table (exit 3);
- `local_failure`: the runner could not write its artifacts (exporter exit 4);
- `comparison_failure`: either CSV was malformed (comparison exit 3);
- `semantic_drift`: normalized method/path operations were added or removed
  (comparison exit 1).

An outage therefore cannot appear as a mass API removal: semantic comparison
does not run unless the crawl succeeds. The crawl status also records pages
with no parsed method table; if a page that supplied checked-in routes moves to
that list, comparison reports a parser failure before calculating removals.

Some macOS framework Python installations do not inherit the system CA store.
For local crawls, point `SSL_CERT_FILE` at a trusted CA bundle (for example one
managed by `certifi`) rather than disabling certificate verification. GitHub's
Ubuntu runners use their system trust store.

### Responding to docs drift

1. Rerun once to rule out a transient fetch failure.
2. For a parser failure, inspect the fresh page layout and update parser tests
   before changing the checked inventory.
3. For semantic drift, review every added and removed operation in the Markdown
   artifact against the official version-specific references.
4. Regenerate `docs/api-inventory.csv`, run the method-aware coverage audit,
   and file or link implementation issues for real contract changes.
5. Commit the reviewed inventory and generated audit together. Never copy a
   failed or partial crawl into the repository.

## Supported-version live matrix

The live job starts one isolated disposable Docker stack for each image pinned
in the [version support policy](./version-support.md): 8.2, 8.0, 7.22, 7.8,
and 7.4. Each matrix runner:

- pulls the exact Redis Software image;
- initializes a local cluster and default database with the immutable redisctl
  image digest checked into `docker-compose.yml`;
- runs the versioned compliance baseline;
- verifies that the report's product version and image match the matrix;
- uploads sanitized JSON and Markdown summaries; and
- removes containers, networks, and volumes even when a step fails.

Scheduled runs use the safe read-only profile. Self-cleaning write lifecycles
can run only when a maintainer manually dispatches the workflow with
`run_disposable_writes` enabled. The workflow has no shared-cluster URL or
credential inputs and never targets production. Destructive cluster operations
are not part of this workflow; adding one requires a separate explicit dispatch
guard and safety review.

The report contains operation status, model field paths, exact image/version
provenance, and sanitized error classes. It never stores response bodies,
credentials, license values, resource identifiers, or customer data.

### Responding to live failures

Use the failed step to separate infrastructure from contract behavior:

- image-pull failures are registry or image-availability incidents;
- initialization failures are disposable-cluster bootstrap defects;
- version mismatches mean the image pin no longer packages the expected build;
- compliance failures identify operation, status, or typed-model drift in the
  sanitized report.

Do not weaken a baseline simply to make a scheduled run green. Reproduce the
failure against the same pinned image, review official version-specific docs,
then update code or record a narrowly justified version difference.

## Manual dispatch

From the GitHub Actions page, choose `Enterprise API contract` and `Run
workflow`. `run_docs` and `run_live` can be selected independently.
`run_disposable_writes` is off by default and only affects manually dispatched
live jobs.
