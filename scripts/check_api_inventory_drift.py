#!/usr/bin/env python3
"""Compare official-doc API inventories by normalized HTTP method and path."""

from __future__ import annotations

import argparse
import csv
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path

EXIT_SEMANTIC_DRIFT = 1
EXIT_COMPARISON_FAILURE = 3
HTTP_METHODS = {"GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"}


class InventoryFormatError(RuntimeError):
    """An inventory CSV is missing required or valid operation fields."""


class InventoryPageParseError(InventoryFormatError):
    """A previously inventoried docs page no longer yielded a method table."""


@dataclass(frozen=True, order=True)
class Operation:
    method: str
    path: str

    def display(self) -> str:
        return f"{self.method} {self.path}"


def normalize_path(path: str) -> str:
    path = path.strip().split("?", 1)[0]
    path = re.sub(r"\{[^}]+\}|<[^>]+>", "{}", path)
    if path != "/":
        path = path.rstrip("/")
    return path


def load_operations(path: Path) -> set[Operation]:
    try:
        handle = path.open(newline="", encoding="utf-8")
    except OSError as exc:
        raise InventoryFormatError(f"could not read {path}: {type(exc).__name__}") from exc

    with handle:
        reader = csv.DictReader(handle)
        if reader.fieldnames is None or not {"method", "path"}.issubset(reader.fieldnames):
            raise InventoryFormatError(f"{path} must contain method and path columns")

        operations = set()
        for line_number, row in enumerate(reader, start=2):
            method = (row.get("method") or "").strip().upper()
            operation_path = normalize_path(row.get("path") or "")
            if method not in HTTP_METHODS:
                raise InventoryFormatError(
                    f"{path}:{line_number} has unsupported HTTP method {method!r}"
                )
            if not operation_path.startswith("/"):
                raise InventoryFormatError(
                    f"{path}:{line_number} has invalid API path {operation_path!r}"
                )
            operations.add(Operation(method, operation_path))

    if not operations:
        raise InventoryFormatError(f"{path} contains no operations")
    return operations


def load_expected_pages(path: Path) -> set[str]:
    try:
        with path.open(newline="", encoding="utf-8") as handle:
            reader = csv.DictReader(handle)
            if reader.fieldnames is None or "page" not in reader.fieldnames:
                raise InventoryFormatError(f"{path} must contain a page column")
            pages = {(row.get("page") or "").strip() for row in reader}
    except OSError as exc:
        raise InventoryFormatError(f"could not read {path}: {type(exc).__name__}") from exc
    pages.discard("")
    if not pages:
        raise InventoryFormatError(f"{path} contains no source pages")
    return pages


def load_crawl_status(path: Path) -> set[str]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise InventoryFormatError(
            f"could not read crawl status {path}: {type(exc).__name__}"
        ) from exc
    if not isinstance(payload, dict) or payload.get("classification") != "success":
        raise InventoryFormatError(f"{path} is not a successful crawl status")
    empty_pages = payload.get("pages_without_method_rows")
    if not isinstance(empty_pages, list) or not all(
        isinstance(page, str) for page in empty_pages
    ):
        raise InventoryFormatError(
            f"{path} must contain a pages_without_method_rows string array"
        )
    return set(empty_pages)


def compare(
    expected: set[Operation], actual: set[Operation]
) -> tuple[list[Operation], list[Operation]]:
    return sorted(actual - expected), sorted(expected - actual)


def render_report(
    expected_count: int,
    actual_count: int,
    added: list[Operation],
    removed: list[Operation],
) -> str:
    outcome = "semantic drift detected" if added or removed else "no semantic drift"
    lines = [
        "# Official Redis API inventory drift",
        "",
        f"**Outcome:** {outcome}",
        "",
        f"- Checked-in unique operations: `{expected_count}`",
        f"- Fresh official-doc unique operations: `{actual_count}`",
        f"- Added by official docs: `{len(added)}`",
        f"- Removed from official docs: `{len(removed)}`",
    ]

    for heading, operations in [
        ("Added by official docs", added),
        ("Absent from fresh official docs", removed),
    ]:
        if not operations:
            continue
        lines.extend(["", f"## {heading}", ""])
        lines.extend(f"- `{operation.display()}`" for operation in operations)

    lines.extend(
        [
            "",
            "Descriptions, page ordering, duplicate rows, trailing slashes, query strings,",
            "and placeholder names do not affect this comparison.",
            "",
        ]
    )
    return "\n".join(lines)


def write_json(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--expected", required=True, help="Checked-in inventory CSV")
    parser.add_argument("--actual", required=True, help="Fresh official-doc inventory CSV")
    parser.add_argument("--crawl-status", required=True, help="Fresh crawl status JSON")
    parser.add_argument("--report", required=True, help="Markdown drift report path")
    parser.add_argument("--status-output", required=True, help="Machine-readable JSON status path")
    args = parser.parse_args()

    report_path = Path(args.report)
    status_path = Path(args.status_output)
    try:
        expected_path = Path(args.expected)
        expected = load_operations(expected_path)
        actual = load_operations(Path(args.actual))
        expected_pages = load_expected_pages(expected_path)
        pages_without_method_rows = load_crawl_status(Path(args.crawl_status))
        parser_regressions = sorted(expected_pages & pages_without_method_rows)
        if parser_regressions:
            raise InventoryPageParseError(
                "fresh crawl yielded no method rows for previously inventoried pages: "
                + ", ".join(parser_regressions)
            )
    except InventoryFormatError as exc:
        classification = (
            "parse_failure"
            if isinstance(exc, InventoryPageParseError)
            else "comparison_failure"
        )
        outcome = (
            "parser failure"
            if classification == "parse_failure"
            else "comparison failure"
        )
        report_path.parent.mkdir(parents=True, exist_ok=True)
        report_path.write_text(
            "# Official Redis API inventory drift\n\n"
            f"**Outcome:** {outcome}\n\n{exc}\n",
            encoding="utf-8",
        )
        write_json(
            status_path,
            {"classification": classification, "message": str(exc)},
        )
        print(f"error: inventory comparison failed: {exc}", file=sys.stderr)
        return EXIT_COMPARISON_FAILURE

    added, removed = compare(expected, actual)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(
        render_report(len(expected), len(actual), added, removed), encoding="utf-8"
    )
    classification = "semantic_drift" if added or removed else "success"
    write_json(
        status_path,
        {
            "classification": classification,
            "expected_operations": len(expected),
            "actual_operations": len(actual),
            "added": [operation.display() for operation in added],
            "removed": [operation.display() for operation in removed],
        },
    )

    if added or removed:
        print(
            f"Official-doc inventory drift: {len(added)} added, {len(removed)} removed",
            file=sys.stderr,
        )
        return EXIT_SEMANTIC_DRIFT

    print(f"No official-doc route drift across {len(expected)} unique operations")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
