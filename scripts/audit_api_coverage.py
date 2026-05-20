#!/usr/bin/env python3
"""Compare the generated API inventory to path evidence in src/ and tests/."""

from __future__ import annotations

import argparse
import csv
import re
from collections import Counter
from pathlib import Path

PATH_LITERAL_RE = re.compile(r"/v[12]/[A-Za-z0-9_./<>{}:-]*")


def normalize_path(path: str) -> str:
    normalized = path.strip().replace("{int: uid}", "{uid}")
    normalized = re.sub(r"\{[^}]+\}", "{}", normalized)
    normalized = re.sub(r"<[^>]+>", "{}", normalized)
    return normalized


def extract_path_evidence(root: Path) -> dict[str, set[str]]:
    evidence: dict[str, set[str]] = {}

    for file_path in sorted(root.rglob("*.rs")):
        text = file_path.read_text()
        for match in PATH_LITERAL_RE.findall(text):
            normalized = normalize_path(match)
            evidence.setdefault(normalized, set()).add(str(file_path.relative_to(root.parent)))

    return evidence


def load_inventory(path: Path) -> list[dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as handle:
        return list(csv.DictReader(handle))


def write_csv(rows: list[dict[str, str]], path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=[
                "method",
                "path",
                "normalized_path",
                "page",
                "title",
                "sdk_module_guess",
                "src_path_match",
                "test_path_match",
                "src_evidence",
                "test_evidence",
                "audit_status",
                "notes",
            ],
        )
        writer.writeheader()
        writer.writerows(rows)


def write_markdown(rows: list[dict[str, str]], path: Path) -> None:
    counts = Counter(row["audit_status"] for row in rows)
    docs_only = [row for row in rows if row["audit_status"] == "docs_only"]

    lines = [
        "# API Coverage Audit",
        "",
        "This report compares the generated official-doc inventory to exact path",
        "evidence found in `src/` and `tests/`.",
        "",
        "## Summary",
        "",
        f"- Total documented endpoints audited: `{len(rows)}`",
        f"- `implemented_and_tested`: `{counts['implemented_and_tested']}`",
        f"- `implemented_no_test_path_evidence`: `{counts['implemented_no_test_path_evidence']}`",
        f"- `test_only_path_evidence`: `{counts['test_only_path_evidence']}`",
        f"- `docs_only`: `{counts['docs_only']}`",
        "",
        "## Notes",
        "",
        "- This is an initial path-based audit, not a semantic guarantee that request and response",
        "  models are correct.",
        "- `docs_only` rows are the most likely follow-up candidates, but some may be explained by",
        "  path aliases, version-specific availability, or endpoint families that are expressed",
        "  differently in the SDK than in the docs.",
        "- Live validation on April 21, 2026 found that:",
        "  - `POST /v1/users` on an RBAC-enabled cluster requires `role_uids` without `role`.",
        "  - documented `suffix` endpoints returned `404 Not Found` on Redis Enterprise Software `8.0.10-81`.",
        "  - creating a disposable database hit shard-license limits on the local trial cluster.",
        "",
        "## Likely Follow-ups",
        "",
    ]

    for row in docs_only[:30]:
        lines.append(
            f"- `{row['method']} {row['path']}` from `{row['page']}`"
            + (f" (module guess: `{row['sdk_module_guess']}`)" if row["sdk_module_guess"] else "")
        )

    lines.extend(
        [
            "",
            "## Artifacts",
            "",
            f"- CSV audit: [{path.with_suffix('.csv').name}](./{path.with_suffix('.csv').name})",
        ]
    )

    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--inventory",
        default="docs/api-inventory.csv",
        help="Path to the generated inventory CSV",
    )
    parser.add_argument(
        "--csv-output",
        default="docs/api-coverage-audit.csv",
        help="Path to the generated audit CSV",
    )
    parser.add_argument(
        "--md-output",
        default="docs/api-coverage-audit.md",
        help="Path to the generated markdown summary",
    )
    args = parser.parse_args()

    inventory_rows = load_inventory(Path(args.inventory))
    src_evidence = extract_path_evidence(Path("src"))
    test_evidence = extract_path_evidence(Path("tests"))

    audit_rows: list[dict[str, str]] = []
    for row in inventory_rows:
        normalized_path = normalize_path(row["path"])
        src_matches = sorted(src_evidence.get(normalized_path, set()))
        test_matches = sorted(test_evidence.get(normalized_path, set()))

        if src_matches and test_matches:
            audit_status = "implemented_and_tested"
        elif src_matches:
            audit_status = "implemented_no_test_path_evidence"
        elif test_matches:
            audit_status = "test_only_path_evidence"
        else:
            audit_status = "docs_only"

        audit_rows.append(
            {
                "method": row["method"],
                "path": row["path"],
                "normalized_path": normalized_path,
                "page": row["page"],
                "title": row["title"],
                "sdk_module_guess": row["sdk_module_guess"],
                "src_path_match": str(bool(src_matches)).lower(),
                "test_path_match": str(bool(test_matches)).lower(),
                "src_evidence": ";".join(src_matches),
                "test_evidence": ";".join(test_matches),
                "audit_status": audit_status,
                "notes": "",
            }
        )

    csv_output = Path(args.csv_output)
    md_output = Path(args.md_output)
    write_csv(audit_rows, csv_output)
    write_markdown(audit_rows, md_output)

    counts = Counter(row["audit_status"] for row in audit_rows)
    print(
        "Audit complete:"
        f" total={len(audit_rows)}"
        f" implemented_and_tested={counts['implemented_and_tested']}"
        f" implemented_no_test_path_evidence={counts['implemented_no_test_path_evidence']}"
        f" test_only_path_evidence={counts['test_only_path_evidence']}"
        f" docs_only={counts['docs_only']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
