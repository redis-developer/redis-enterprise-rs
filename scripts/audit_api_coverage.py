#!/usr/bin/env python3
"""Build a method-aware static evidence report for the Enterprise REST API."""

from __future__ import annotations

import argparse
import csv
import re
from collections import Counter
from dataclasses import dataclass, field
from pathlib import Path

SUPPORTED_METHODS = ("GET", "POST", "PUT", "PATCH", "DELETE")
WRITE_METHODS = {"POST", "PUT", "PATCH"}

PATH_LITERAL_RE = re.compile(r"/v[12]/[^\"\s]*")
PLACEHOLDER_RE = re.compile(r"\{[^}]+\}|<[^>]+>")
MOCK_START_RE = re.compile(
    r'Mock::given\(\s*(?:wiremock::matchers::)?method\(\s*"'
    r"(?P<method>GET|POST|PUT|PATCH|DELETE)"
    r'"\s*\)\s*\)'
)
MOCK_PATH_RE = re.compile(
    r'(?:wiremock::matchers::)?path\(\s*"(?P<path>/[^"\n]+)"\s*\)'
)
EVIDENCE_MARKER_RE = re.compile(
    r"^\s*//\s*api-audit-(?P<kind>live|response):\s*"
    r"(?P<method>GET|POST|PUT|PATCH|DELETE)\s+(?P<path>/\S+)\s*$",
    re.MULTILINE,
)

HANDLER_VERBS = {
    "get": "GET",
    "get_text": "GET",
    "get_binary": "GET",
    "get_raw": "GET",
    "post": "POST",
    "post_raw": "POST",
    "post_action": "POST",
    "post_multipart": "POST",
    "post_bootstrap": "POST",
    "put": "PUT",
    "put_raw": "PUT",
    "put_action": "PUT",
    "patch_raw": "PATCH",
    "delete": "DELETE",
    "delete_raw": "DELETE",
}
HANDLER_ALTERNATION = "|".join(map(re.escape, HANDLER_VERBS))
DIRECT_HANDLER_RE = re.compile(
    rf'\.(?P<verb>{HANDLER_ALTERNATION})\(\s*&?'
    rf'(?:format!\(\s*)?"(?P<path>/[^"\n]*)"'
)
IMPL_CRUD_PATTERNS = (
    (
        re.compile(r'list\s*=>\s*\w+\s*,\s*"(?P<path>/[^"\n]+)"\s*;'),
        "GET",
    ),
    (
        re.compile(
            r'get\s*\([^)]*\)\s*=>\s*\w+\s*,\s*"(?P<path>/[^"\n]+)"\s*;'
        ),
        "GET",
    ),
    (
        re.compile(
            r'create\s*\([^)]*\)\s*=>\s*\w+\s*,\s*"(?P<path>/[^"\n]+)"\s*;'
        ),
        "POST",
    ),
    (
        re.compile(
            r'update\s*\([^,)]*,\s*[^)]*\)\s*=>\s*\w+\s*,'
            r'\s*"(?P<path>/[^"\n]+)"\s*;'
        ),
        "PUT",
    ),
    (
        re.compile(
            r'delete\s*\([^)]*\)\s*,\s*"(?P<path>/[^"\n]+)"\s*;'
        ),
        "DELETE",
    ),
)

CSV_FIELDS = [
    "method",
    "path",
    "normalized_path",
    "pages",
    "titles",
    "sdk_module_guesses",
    "handler_declared",
    "mock_method_path",
    "mock_call_expectation",
    "request_body_matcher",
    "query_matcher",
    "response_fixture",
    "fixture_deserialization",
    "live_evidence",
    "handler_evidence",
    "mock_evidence",
    "request_evidence",
    "response_evidence",
    "fixture_evidence",
    "live_evidence_files",
    "uncertain_test_path_mentions",
    "audit_status",
    "notes",
]


@dataclass(frozen=True, order=True)
class OperationKey:
    method: str
    path: str


@dataclass
class InventoryOperation:
    key: OperationKey
    paths: set[str] = field(default_factory=set)
    pages: set[str] = field(default_factory=set)
    titles: set[str] = field(default_factory=set)
    sdk_module_guesses: set[str] = field(default_factory=set)

    @property
    def canonical_path(self) -> str:
        return sorted(self.paths, key=lambda value: ("{" in value or "<" in value, value))[0]


@dataclass
class EvidenceIndex:
    handler: dict[OperationKey, set[str]] = field(default_factory=dict)
    mock: dict[OperationKey, set[str]] = field(default_factory=dict)
    mock_expectation: dict[OperationKey, set[str]] = field(default_factory=dict)
    request_body: dict[OperationKey, set[str]] = field(default_factory=dict)
    query: dict[OperationKey, set[str]] = field(default_factory=dict)
    response_fixture: dict[OperationKey, set[str]] = field(default_factory=dict)
    fixture_deserialization: dict[OperationKey, set[str]] = field(default_factory=dict)
    live: dict[OperationKey, set[str]] = field(default_factory=dict)
    path_mentions: dict[str, set[str]] = field(default_factory=dict)


def normalize_path(path: str) -> str:
    """Normalize placeholders, query strings, and trailing slashes."""
    normalized = path.strip().split("?", 1)[0].replace("{int: uid}", "{uid}")
    normalized = PLACEHOLDER_RE.sub("{}", normalized)
    normalized = normalized.rstrip("/")
    return normalized or "/"


def template_matches(template: str, observed: str) -> bool:
    """Match a documented template against a concrete or templated observed path."""
    template_parts = normalize_path(template).split("/")
    observed_parts = normalize_path(observed).split("/")
    if len(template_parts) != len(observed_parts):
        return False
    observed_is_template = "{}" in observed_parts
    if observed_is_template:
        return all(expected == actual for expected, actual in zip(template_parts, observed_parts))
    return all(
        expected == "{}" or expected == actual
        for expected, actual in zip(template_parts, observed_parts)
    )


def strip_rust_comments(contents: str) -> str:
    """Remove Rust comments while preserving enough newlines for source locations."""

    def replace_block(match: re.Match[str]) -> str:
        return "\n" * match.group(0).count("\n")

    contents = re.sub(r"/\*.*?\*/", replace_block, contents, flags=re.DOTALL)
    lines = []
    for line in contents.splitlines():
        marker = line.find("//")
        lines.append(line if marker == -1 else line[:marker])
    return "\n".join(lines)


def relative_name(path: Path, repo_root: Path) -> str:
    try:
        return str(path.relative_to(repo_root))
    except ValueError:
        return str(path)


def location(path: Path, contents: str, offset: int, repo_root: Path) -> str:
    return f"{relative_name(path, repo_root)}:{contents.count(chr(10), 0, offset) + 1}"


def add_evidence(
    index: dict[OperationKey, set[str]], key: OperationKey, evidence: str
) -> None:
    index.setdefault(key, set()).add(evidence)


def source_files(src_root: Path) -> list[Path]:
    excluded_names = {"client.rs", "lib.rs", "lib_tests.rs", "macros.rs"}
    return [
        path
        for path in sorted(src_root.rglob("*.rs"))
        if "testing" not in path.relative_to(src_root).parts
        and path.name not in excluded_names
    ]


def extract_handler_evidence(src_root: Path, repo_root: Path) -> EvidenceIndex:
    evidence = EvidenceIndex()
    for path in source_files(src_root):
        raw_contents = path.read_text(encoding="utf-8")
        contents = strip_rust_comments(raw_contents)
        for match in DIRECT_HANDLER_RE.finditer(contents):
            key = OperationKey(
                HANDLER_VERBS[match.group("verb")], normalize_path(match.group("path"))
            )
            add_evidence(
                evidence.handler, key, location(path, contents, match.start(), repo_root)
            )
        for pattern, method in IMPL_CRUD_PATTERNS:
            for match in pattern.finditer(contents):
                key = OperationKey(method, normalize_path(match.group("path")))
                add_evidence(
                    evidence.handler, key, location(path, contents, match.start(), repo_root)
                )
    return evidence


def response_fixture_present(mock_block: str) -> bool:
    if re.search(r"\.set_body_(?:json|string|bytes)\s*\(", mock_block):
        return True
    return bool(
        re.search(
            r"respond_with\(\s*(?:success|created|error)_response\s*\(", mock_block
        )
    )


def extract_markers(
    contents: str, path: Path, repo_root: Path, evidence: EvidenceIndex
) -> None:
    for match in EVIDENCE_MARKER_RE.finditer(contents):
        key = OperationKey(match.group("method"), normalize_path(match.group("path")))
        if match.group("kind") == "live":
            if not path.name.startswith("live_") or "#[ignore" not in contents:
                raise ValueError(
                    f"{relative_name(path, repo_root)}: api-audit-live markers "
                    "must be in an ignored live_* test file"
                )
            destination = evidence.live
        else:
            destination = evidence.fixture_deserialization
        add_evidence(destination, key, location(path, contents, match.start(), repo_root))


def extract_test_evidence(tests_root: Path, repo_root: Path) -> EvidenceIndex:
    evidence = EvidenceIndex()
    for path in sorted(tests_root.rglob("*.rs")):
        raw_contents = path.read_text(encoding="utf-8")
        extract_markers(raw_contents, path, repo_root, evidence)
        contents = strip_rust_comments(raw_contents)

        for mention in PATH_LITERAL_RE.finditer(contents):
            normalized = normalize_path(mention.group(0))
            evidence.path_mentions.setdefault(normalized, set()).add(
                relative_name(path, repo_root)
            )

        starts = list(MOCK_START_RE.finditer(contents))
        for index, start in enumerate(starts):
            next_start = starts[index + 1].start() if index + 1 < len(starts) else len(contents)
            mount = re.search(r"\.mount(?:_as_scoped)?\s*\(", contents[start.end() : next_start])
            if mount is None:
                continue
            block_end = start.end() + mount.end()
            block = contents[start.start() : block_end]
            path_match = MOCK_PATH_RE.search(block)
            if path_match is None:
                continue

            key = OperationKey(
                start.group("method"), normalize_path(path_match.group("path"))
            )
            source = location(path, contents, start.start(), repo_root)
            add_evidence(evidence.mock, key, source)
            if re.search(r"\.expect\s*\(", block):
                add_evidence(evidence.mock_expectation, key, source)
            if re.search(
                r"(?<![A-Za-z0-9_])(?:wiremock::matchers::)?body_json\s*\(", block
            ):
                add_evidence(evidence.request_body, key, source)
            if re.search(r"(?:wiremock::matchers::)?query_param\s*\(", block):
                add_evidence(evidence.query, key, source)
            if response_fixture_present(block):
                add_evidence(evidence.response_fixture, key, source)
    return evidence


def merge_evidence(target: EvidenceIndex, source: EvidenceIndex) -> EvidenceIndex:
    for attribute in (
        "handler",
        "mock",
        "mock_expectation",
        "request_body",
        "query",
        "response_fixture",
        "fixture_deserialization",
        "live",
    ):
        destination = getattr(target, attribute)
        for key, values in getattr(source, attribute).items():
            destination.setdefault(key, set()).update(values)
    for path, values in source.path_mentions.items():
        target.path_mentions.setdefault(path, set()).update(values)
    return target


def load_inventory(path: Path) -> tuple[list[InventoryOperation], int]:
    operations: dict[OperationKey, InventoryOperation] = {}
    with path.open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    for row in rows:
        method = row["method"].strip().upper()
        raw_path = row["path"].strip()
        if method not in SUPPORTED_METHODS or not raw_path.startswith("/"):
            continue
        key = OperationKey(method, normalize_path(raw_path))
        operation = operations.setdefault(key, InventoryOperation(key=key))
        operation.paths.add(raw_path)
        operation.pages.add(row["page"].strip())
        operation.titles.add(row["title"].strip())
        module = row.get("sdk_module_guess", "").strip()
        if module:
            operation.sdk_module_guesses.add(module)
    return [operations[key] for key in sorted(operations)], len(rows)


def matching_evidence(
    index: dict[OperationKey, set[str]], operation: OperationKey
) -> set[str]:
    matches: set[str] = set()
    for observed, sources in index.items():
        if observed.method == operation.method and template_matches(operation.path, observed.path):
            matches.update(sources)
    return matches


def matching_path_mentions(
    index: dict[str, set[str]], operation: OperationKey
) -> set[str]:
    matches: set[str] = set()
    for observed, sources in index.items():
        if template_matches(operation.path, observed):
            matches.update(sources)
    return matches


def joined(values: set[str]) -> str:
    return ";".join(sorted(value for value in values if value))


def boolean(value: set[str]) -> str:
    return str(bool(value)).lower()


def audit_status(handler: set[str], mock: set[str], expectation: set[str]) -> str:
    if handler and expectation:
        return "handler_with_asserted_mock"
    if handler and mock:
        return "handler_with_mock_evidence"
    if handler:
        return "handler_without_mock_evidence"
    if mock:
        return "mock_without_handler"
    return "docs_only"


def build_audit_rows(
    inventory: list[InventoryOperation], evidence: EvidenceIndex
) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    for operation in inventory:
        key = operation.key
        handler = matching_evidence(evidence.handler, key)
        mock = matching_evidence(evidence.mock, key)
        expectation = matching_evidence(evidence.mock_expectation, key)
        request_body = matching_evidence(evidence.request_body, key)
        query = matching_evidence(evidence.query, key)
        response = matching_evidence(evidence.response_fixture, key)
        fixture = matching_evidence(evidence.fixture_deserialization, key)
        live = matching_evidence(evidence.live, key)
        mentions = matching_path_mentions(evidence.path_mentions, key)
        ambiguous_concrete_mock = not handler and key.path.count("{}") > 1 and bool(mock)
        if ambiguous_concrete_mock:
            mock = set()
            expectation = set()
            request_body = set()
            query = set()
            response = set()
        mock_files = {item.rsplit(":", 1)[0] for item in mock}
        uncertain_mentions = mentions.difference(mock_files)

        notes = []
        if mock and not expectation:
            notes.append("Wiremock method/path matcher has no explicit call-count expectation")
        if uncertain_mentions:
            notes.append("unscoped path literals are not counted as behavioral evidence")
        if key.method in WRITE_METHODS and mock and not request_body:
            notes.append("write route has no request body matcher")
        if ambiguous_concrete_mock:
            notes.append(
                "concrete mocks are ambiguous for a multi-placeholder route without an exact handler"
            )

        rows.append(
            {
                "method": key.method,
                "path": operation.canonical_path,
                "normalized_path": key.path,
                "pages": joined(operation.pages),
                "titles": joined(operation.titles),
                "sdk_module_guesses": joined(operation.sdk_module_guesses),
                "handler_declared": boolean(handler),
                "mock_method_path": boolean(mock),
                "mock_call_expectation": boolean(expectation),
                "request_body_matcher": boolean(request_body),
                "query_matcher": boolean(query),
                "response_fixture": boolean(response),
                "fixture_deserialization": boolean(fixture),
                "live_evidence": boolean(live),
                "handler_evidence": joined(handler),
                "mock_evidence": joined(mock),
                "request_evidence": joined(request_body | query),
                "response_evidence": joined(response),
                "fixture_evidence": joined(fixture),
                "live_evidence_files": joined(live),
                "uncertain_test_path_mentions": joined(uncertain_mentions),
                "audit_status": audit_status(handler, mock, expectation),
                "notes": "; ".join(notes),
            }
        )
    return rows


def write_csv(rows: list[dict[str, str]], path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=CSV_FIELDS, lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)


def count_true(rows: list[dict[str, str]], field_name: str) -> int:
    return sum(row[field_name] == "true" for row in rows)


def write_markdown(
    rows: list[dict[str, str]], path: Path, inventory_row_count: int
) -> None:
    counts = Counter(row["audit_status"] for row in rows)
    docs_only = [row for row in rows if row["audit_status"] == "docs_only"]
    handler_only = [
        row for row in rows if row["audit_status"] == "handler_without_mock_evidence"
    ]

    lines = [
        "# API Coverage Audit",
        "",
        "This generated report compares unique documented `METHOD + normalized path`",
        "operations with distinct static evidence from handlers, Wiremock matchers,",
        "request-shape matchers, response fixtures, fixture deserialization, and live",
        "validation annotations.",
        "",
        "## Summary",
        "",
        f"- Raw documented inventory rows: `{inventory_row_count}`",
        f"- Unique documented operations: `{len(rows)}`",
        f"- Collapsed duplicate aliases: `{inventory_row_count - len(rows)}`",
        f"- Handler declarations: `{count_true(rows, 'handler_declared')}`",
        f"- Wiremock method/path matchers: `{count_true(rows, 'mock_method_path')}`",
        f"- Explicit Wiremock call-count expectations: `{count_true(rows, 'mock_call_expectation')}`",
        f"- Request body matchers: `{count_true(rows, 'request_body_matcher')}`",
        f"- Query matchers: `{count_true(rows, 'query_matcher')}`",
        f"- Response fixtures: `{count_true(rows, 'response_fixture')}`",
        f"- Explicit fixture-deserialization evidence: `{count_true(rows, 'fixture_deserialization')}`",
        f"- Explicit live evidence: `{count_true(rows, 'live_evidence')}`",
        "",
        "### Dispositions",
        "",
        f"- `handler_with_asserted_mock`: `{counts['handler_with_asserted_mock']}`",
        f"- `handler_with_mock_evidence`: `{counts['handler_with_mock_evidence']}`",
        f"- `handler_without_mock_evidence`: `{counts['handler_without_mock_evidence']}`",
        f"- `mock_without_handler`: `{counts['mock_without_handler']}`",
        f"- `docs_only`: `{counts['docs_only']}`",
        "",
        "## Interpretation Limits",
        "",
        "- A handler declaration means the scanner found a recognized REST transport call;",
        "  it does not establish request or response correctness.",
        "- A Wiremock method/path matcher is stronger than a string mention, but only an",
        "  explicit call-count expectation proves that Wiremock itself requires the call.",
        "- Body and query columns report exact matcher presence independently. A matcher",
        "  does not prove the documented schema is complete.",
        "- A response fixture means a matching mock supplies a body. Explicit response",
        "  annotations identify standalone typed-fixture deserialization tests.",
        "- Live evidence is counted only from `api-audit-live` annotations attached to",
        "  opt-in tests. This report does not claim those ignored tests ran in CI.",
        "- Unscoped path literals and comments never count as behavioral evidence.",
        "- Static extraction recognizes the transport methods and `impl_crud!` forms used",
        "  in this repository. Dynamic path construction remains a review item.",
        "",
        "## Highest-Priority Follow-ups",
        "",
    ]

    for row in docs_only[:20]:
        lines.append(f"- `{row['method']} {row['path']}`: no handler or matching mock evidence")
    for row in handler_only[:20]:
        lines.append(f"- `{row['method']} {row['path']}`: handler has no matching mock evidence")

    lines.extend(
        [
            "",
            "## Reproduce",
            "",
            "```bash",
            "python3 -m unittest discover -s scripts/tests -p 'test_*.py'",
            "python3 scripts/audit_api_coverage.py",
            "git diff --exit-code -- docs/api-coverage-audit.csv docs/api-coverage-audit.md",
            "```",
            "",
            "## Artifacts",
            "",
            f"- CSV audit: [{path.with_suffix('.csv').name}](./{path.with_suffix('.csv').name})",
        ]
    )
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inventory", default="docs/api-inventory.csv")
    parser.add_argument("--src-root", default="src")
    parser.add_argument("--tests-root", default="tests")
    parser.add_argument("--csv-output", default="docs/api-coverage-audit.csv")
    parser.add_argument("--md-output", default="docs/api-coverage-audit.md")
    args = parser.parse_args()

    repo_root = Path.cwd()
    inventory, inventory_row_count = load_inventory(Path(args.inventory))
    evidence = extract_handler_evidence(Path(args.src_root), repo_root)
    merge_evidence(evidence, extract_test_evidence(Path(args.tests_root), repo_root))
    rows = build_audit_rows(inventory, evidence)

    csv_output = Path(args.csv_output)
    md_output = Path(args.md_output)
    write_csv(rows, csv_output)
    write_markdown(rows, md_output, inventory_row_count)

    counts = Counter(row["audit_status"] for row in rows)
    print(
        "Audit complete:"
        f" inventory_rows={inventory_row_count}"
        f" unique_operations={len(rows)}"
        f" handlers={count_true(rows, 'handler_declared')}"
        f" mocks={count_true(rows, 'mock_method_path')}"
        f" request_bodies={count_true(rows, 'request_body_matcher')}"
        f" response_fixtures={count_true(rows, 'response_fixture')}"
        f" live={count_true(rows, 'live_evidence')}"
        f" docs_only={counts['docs_only']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
