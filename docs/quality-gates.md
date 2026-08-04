# Coverage and Dependency Quality Gates

The main CI workflow enforces two local, deterministic quality gates before any
optional reporting service is contacted.

## Rust coverage

The coverage job runs the complete non-ignored Rust test suite with every crate
feature enabled. The Linux baseline measured on 2026-08-04 with
cargo-llvm-cov 0.8.7 was reproduced exactly on macOS:

| Metric | Covered | Total | Baseline | Enforced floor |
|---|---:|---:|---:|---:|
| Lines | 2,279 | 3,656 | 62.34% | 60.00% |
| Functions | 576 | 953 | 60.44% | 58.00% |

The reviewed source of truth is `quality-gates.toml`. The floors leave less
than three percentage points of headroom for instrumentation changes while
blocking a material regression. `scripts/check_coverage.py` rejects malformed
reports, inconsistent counts, and a floor more than three points below its
recorded baseline.

To reproduce the gate locally:

```bash
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov --version 0.8.7 --locked
mkdir -p target/coverage-artifacts
cargo llvm-cov --all-features --json --summary-only --output-path target/coverage-artifacts/coverage.json
python3 scripts/check_coverage.py \
  --config quality-gates.toml \
  --report target/coverage-artifacts/coverage.json \
  --summary target/coverage-artifacts/coverage-summary.md
cargo llvm-cov report --lcov --output-path target/coverage-artifacts/lcov.info
```

CI uploads the JSON, LCOV, and Markdown summary for 30 days. Codecov upload is
explicitly non-blocking: a reporting outage cannot hide or reverse the result
of the local floor check.

Threshold changes must update the measured counts and percentages in
`quality-gates.toml`, explain why the baseline moved, and receive explicit
review. Raise floors as sustained coverage improves. A reduction is not a way
to make an unrelated PR green; add tests or isolate a genuine instrumentation
change first.

Patch coverage is intentionally informational through Codecov rather than a
required gate. Handler-heavy generated-style modules make small diffs noisy,
and a required external patch check would couple source acceptance to a service
outage. The deterministic repository floor remains the required regression
gate.

## Dependency, license, and source policy

`deny.toml` is enforced by cargo-deny across all features. The gate fails for:

- actionable RustSec advisories;
- wildcard dependency requirements;
- licenses outside the narrow reviewed allowlist; and
- dependencies from unknown registries or Git repositories.

Duplicate transitive versions are warnings because platform support currently
pulls several unavoidable Windows and error-stack versions. They remain visible
for cleanup without blocking security fixes.

Run the same policy locally with:

```bash
cargo install cargo-deny --version 0.19.8 --locked
cargo deny --all-features check advisories bans licenses sources
```

Do not ignore an advisory merely because the affected API is not currently
called. Upgrade or remove the dependency first. If no fixed version exists and
an exception is unavoidable, add the narrow advisory ID or package rule with a
reason, impact analysis, owner, and removal condition in the reviewing PR.
License, ban, or source exceptions require the same evidence.
