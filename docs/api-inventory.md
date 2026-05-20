# API Inventory

Redis Enterprise does not publish an OpenAPI specification for the Enterprise
REST API, so this repo maintains a generated inventory seed based on the
official Redis request-reference pages.

The generated CSV lives at
[api-inventory.csv](/Users/josh.rotenberg/Code/active/redis-enterprise-rs/docs/api-inventory.csv).

## What The Inventory Contains

Each row records:

- the request-reference page where the endpoint was found
- the endpoint method
- the documented path
- the Redis docs page title and description
- a best-effort local SDK module guess

This is a discovery seed, not a final statement of completeness.

## Caveats

- The official docs are the best primary source currently available, but they
  are not an OpenAPI spec.
- The docs' `latest` Redis Software reference currently resolves to the 7.22
  documentation tree, while the local validation target used during this pass
  was Redis Enterprise Software `8.0.10-81`.
- Live clusters can expose behavior or fields that do not appear in the docs.
- Some SDK modules combine multiple documented request families, and some
  documented request families map to nested methods rather than standalone
  modules.

## Regenerate

Run:

```bash
python3 scripts/export_api_inventory.py
```

Or write to a custom path:

```bash
python3 scripts/export_api_inventory.py --output docs/api-inventory.csv
```

## Recommended Workflow

1. Regenerate the CSV from the official docs.
2. Run the live smoke suite against a real Redis Enterprise instance.
3. Compare live-tested endpoints to the generated inventory.
4. File follow-up issues for undocumented, missing, or mismatched paths.
