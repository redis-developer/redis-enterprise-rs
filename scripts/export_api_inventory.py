#!/usr/bin/env python3
"""Export a Redis Enterprise REST API inventory seed from the official docs."""

from __future__ import annotations

import argparse
import csv
import json
import re
import ssl
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

EXIT_FETCH_FAILURE = 2
EXIT_PARSE_FAILURE = 3
EXIT_LOCAL_FAILURE = 4


class DocsFetchError(RuntimeError):
    """The official documentation could not be fetched."""

    def __init__(self, url: str, category: str):
        super().__init__(f"{category}: {url}")
        self.url = url
        self.category = category


class DocsParseError(RuntimeError):
    """Fetched documentation did not contain a usable inventory."""

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
    try:
        with urllib.request.urlopen(request, timeout=20) as response:
            payload = response.read()
    except urllib.error.HTTPError as exc:
        raise DocsFetchError(url, f"http_{exc.code}") from exc
    except urllib.error.URLError as exc:
        category = "tls" if isinstance(exc.reason, ssl.SSLError) else "network"
        raise DocsFetchError(url, category) from exc
    except (TimeoutError, OSError) as exc:
        raise DocsFetchError(url, "network") from exc

    try:
        return payload.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise DocsParseError(f"official docs response was not UTF-8: {url}") from exc


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


def export_inventory(output_path: Path) -> tuple[int, int, list[str]]:
    repo_root = Path(__file__).resolve().parent.parent
    src_root = repo_root / "src"
    pages = discover_request_pages()
    records: list[dict[str, str]] = []
    pages_without_method_rows: list[str] = []

    for page_url in pages:
        page_rel = relative_page(page_url)
        markdown = fetch_text(markdown_url(page_url))
        title = parse_title(markdown)
        module_guess = MODULE_GUESSES.get(page_rel, "")
        module_exists = str((src_root / f"{module_guess}.rs").exists()).lower() if module_guess else ""
        method_rows = parse_method_rows(markdown)
        if not method_rows:
            pages_without_method_rows.append(page_rel or "_index")

        for method, path, description in method_rows:
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

    if not pages:
        raise DocsParseError("official docs crawl discovered no request pages")
    if not records:
        raise DocsParseError(
            "official docs crawl found no method/path rows; the page layout may have changed"
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

    return len(pages), len(records), sorted(pages_without_method_rows)


def write_status(path: Path | None, payload: dict[str, object]) -> None:
    if path is None:
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output",
        default="docs/api-inventory.csv",
        help="Path to the generated CSV file (default: docs/api-inventory.csv)",
    )
    parser.add_argument(
        "--status-output",
        help="Optional JSON status artifact distinguishing fetch, parse, and local failures",
    )
    args = parser.parse_args()
    output_path = Path(args.output)
    status_path = Path(args.status_output) if args.status_output else None

    try:
        page_count, endpoint_count, pages_without_method_rows = export_inventory(output_path)
    except DocsFetchError as exc:
        write_status(
            status_path,
            {
                "classification": "fetch_failure",
                "category": exc.category,
                "source": exc.url,
            },
        )
        print(
            f"error: official docs fetch failed ({exc.category}) for {exc.url}",
            file=sys.stderr,
        )
        return EXIT_FETCH_FAILURE
    except DocsParseError as exc:
        write_status(
            status_path,
            {"classification": "parse_failure", "message": str(exc)},
        )
        print(f"error: official docs parse failed: {exc}", file=sys.stderr)
        return EXIT_PARSE_FAILURE
    except OSError as exc:
        write_status(
            status_path,
            {"classification": "local_failure", "category": type(exc).__name__},
        )
        print(f"error: could not write inventory artifacts: {type(exc).__name__}", file=sys.stderr)
        return EXIT_LOCAL_FAILURE

    write_status(
        status_path,
        {
            "classification": "success",
            "docs_pages": page_count,
            "inventory_rows": endpoint_count,
            "output": str(output_path),
            "pages_without_method_rows": pages_without_method_rows,
        },
    )

    print(f"Exported {endpoint_count} endpoints from {page_count} docs pages to {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
