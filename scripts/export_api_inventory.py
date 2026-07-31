#!/usr/bin/env python3
"""Export a Redis Enterprise REST API inventory seed from the official docs."""

from __future__ import annotations

import argparse
import csv
import re
import sys
import urllib.error
import urllib.parse
import urllib.request
from collections import deque
from pathlib import Path

DOCS_ROOT = "https://redis.io"
REQUESTS_PREFIX = "/docs/latest/operate/rs/references/rest-api/requests/"
REQUESTS_ROOT = urllib.parse.urljoin(DOCS_ROOT, REQUESTS_PREFIX)
USER_AGENT = "redis-enterprise-rs-api-inventory/0.1"

MODULE_GUESSES = {
    "actions": "actions",
    "bdbs": "bdb",
    "bdbs/actions": "actions",
    "bdbs/alerts": "alerts",
    "bdbs/availability": "bdb",
    "bdbs/crdt_sources-alerts": "alerts",
    "bdbs/debuginfo": "debuginfo",
    "bdbs/modules": "modules",
    "bdbs/passwords": "bdb",
    "bdbs/peer_stats": "bdb",
    "bdbs/replica_sources-alerts": "alerts",
    "bdbs/shards": "bdb",
    "bdbs/stats": "bdb",
    "bdbs/sync_source_stats": "bdb",
    "bdbs/syncer_state": "bdb",
    "bdbs/upgrade": "bdb",
    "bootstrap": "bootstrap",
    "cluster": "cluster",
    "cm_settings": "cm_settings",
    "crdb_tasks": "crdb_tasks",
    "crdbs": "crdb",
    "crdbs/upgrade": "crdb",
    "debuginfo": "debuginfo",
    "diagnostics": "diagnostics",
    "endpoints-stats": "endpoints",
    "job_scheduler": "job_scheduler",
    "jsonschema": "jsonschema",
    "ldap_mappings": "ldap_mappings",
    "license": "license",
    "logs": "logs",
    "metrics_config": "metrics_config",
    "migrations": "migrations",
    "modules": "modules",
    "node_master_healthcheck": "local",
    "nodes": "nodes",
    "ocsp": "ocsp",
    "proxies": "proxies",
    "redis_acls": "redis_acls",
    "roles": "roles",
    "services": "services",
    "shards": "shards",
    "suffix": "suffixes",
    "suffixes": "suffixes",
    "usage_report": "usage_report",
    "users": "users",
}


def fetch_text(url: str) -> str:
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request, timeout=20) as response:
        return response.read().decode("utf-8")


def normalize_page_url(url: str) -> str:
    parsed = urllib.parse.urlparse(url)
    path = parsed.path
    if not path.endswith("/"):
        path = f"{path}/"
    return urllib.parse.urlunparse((parsed.scheme, parsed.netloc, path, "", "", ""))


def markdown_url(page_url: str) -> str:
    return urllib.parse.urljoin(page_url, "index.html.md")


def relative_page(page_url: str) -> str:
    parsed = urllib.parse.urlparse(page_url)
    rel = parsed.path.removeprefix(REQUESTS_PREFIX).strip("/")
    return rel


def discover_request_pages() -> list[str]:
    queue = deque([REQUESTS_ROOT])
    seen: set[str] = set()
    discovered: list[str] = []

    href_pattern = re.compile(r'href="(/docs/latest/operate/rs/references/rest-api/requests/[^"#?]+/)"')

    while queue:
        page_url = normalize_page_url(queue.popleft())
        if page_url in seen:
            continue

        seen.add(page_url)
        discovered.append(page_url)
        html = fetch_text(page_url)

        for match in href_pattern.findall(html):
            child_url = normalize_page_url(urllib.parse.urljoin(DOCS_ROOT, match))
            if child_url not in seen:
                queue.append(child_url)

    return sorted(discovered)


def strip_markdown_link(cell: str) -> str:
    match = re.search(r"\[([^\]]+)\]", cell)
    if match:
        return match.group(1).strip()
    return cell.strip()


def strip_code(cell: str) -> str:
    return cell.replace("`", "").strip()


def parse_title(markdown: str) -> str:
    for line in markdown.splitlines():
        if line.startswith("# "):
            return line[2:].strip()
    return ""


def parse_method_rows(markdown: str) -> list[tuple[str, str, str]]:
    lines = markdown.splitlines()
    rows: list[tuple[str, str, str]] = []

    for index, line in enumerate(lines):
        if line.strip() != "| Method | Path | Description |":
            continue

        cursor = index + 2
        while cursor < len(lines):
            row = lines[cursor].strip()
            if not row.startswith("|") or row.count("|") < 4:
                break

            parts = [part.strip() for part in row.strip("|").split("|")]
            if len(parts) != 3:
                break

            method = strip_markdown_link(parts[0])
            path = strip_code(parts[1])
            description = parts[2].strip()
            rows.append((method, path, description))
            cursor += 1

        break

    return rows


def export_inventory(output_path: Path) -> tuple[int, int]:
    repo_root = Path(__file__).resolve().parent.parent
    src_root = repo_root / "src"
    pages = discover_request_pages()
    records: list[dict[str, str]] = []

    for page_url in pages:
        page_rel = relative_page(page_url)
        markdown = fetch_text(markdown_url(page_url))
        title = parse_title(markdown)
        module_guess = MODULE_GUESSES.get(page_rel, "")
        module_exists = str((src_root / f"{module_guess}.rs").exists()).lower() if module_guess else ""

        for method, path, description in parse_method_rows(markdown):
            records.append(
                {
                    "page": page_rel or "_index",
                    "title": title,
                    "method": method,
                    "path": path,
                    "description": description,
                    "source": page_url,
                    "source_type": "official_docs",
                    "sdk_module_guess": module_guess,
                    "repo_module_exists": module_exists,
                    "status": "unreviewed",
                    "notes": "",
                }
            )

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(
            handle,
            lineterminator="\n",
            fieldnames=[
                "page",
                "title",
                "method",
                "path",
                "description",
                "source",
                "source_type",
                "sdk_module_guess",
                "repo_module_exists",
                "status",
                "notes",
            ],
        )
        writer.writeheader()
        writer.writerows(records)

    return len(pages), len(records)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output",
        default="docs/api-inventory.csv",
        help="Path to the generated CSV file (default: docs/api-inventory.csv)",
    )
    args = parser.parse_args()

    try:
        page_count, endpoint_count = export_inventory(Path(args.output))
    except urllib.error.URLError as exc:
        print(f"error: failed to fetch Redis docs: {exc}", file=sys.stderr)
        return 1

    print(f"Exported {endpoint_count} endpoints from {page_count} docs pages to {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
