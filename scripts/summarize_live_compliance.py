#!/usr/bin/env python3
"""Render a sanitized live-compliance JSON report as a Markdown summary."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


class ComplianceReportError(RuntimeError):
    """A compliance report is missing required provenance or summary data."""


SUMMARY_FIELDS = [
    "total",
    "pass",
    "known_difference",
    "version_specific",
    "skipped",
    "unsupported",
    "fail",
    "model_dropped_fields",
    "model_failed",
]


def load_report(path: Path) -> dict[str, object]:
    try:
        report = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ComplianceReportError(
            f"could not read compliance report: {type(exc).__name__}"
        ) from exc
    if not isinstance(report, dict):
        raise ComplianceReportError("compliance report root must be an object")
    return report


def validate_report(
    report: dict[str, object], expected_version: str, expected_image: str
) -> dict[str, int]:
    version_family = report.get("version_family")
    if not isinstance(version_family, str) or not version_family:
        raise ComplianceReportError("compliance report version_family must be text")
    if report.get("server_version") != expected_version:
        raise ComplianceReportError(
            f"expected server version {expected_version!r}, got {report.get('server_version')!r}"
        )
    if report.get("image") != expected_image:
        raise ComplianceReportError(
            f"expected image {expected_image!r}, got {report.get('image')!r}"
        )
    summary = report.get("summary")
    if not isinstance(summary, dict):
        raise ComplianceReportError("compliance report summary must be an object")

    normalized = {}
    for field in SUMMARY_FIELDS:
        value = summary.get(field)
        if not isinstance(value, int) or isinstance(value, bool) or value < 0:
            raise ComplianceReportError(f"summary field {field!r} must be a nonnegative integer")
        normalized[field] = value
    classified_total = sum(
        normalized[field]
        for field in [
            "pass",
            "known_difference",
            "version_specific",
            "skipped",
            "unsupported",
            "fail",
        ]
    )
    if classified_total != normalized["total"]:
        raise ComplianceReportError(
            "compliance status counts do not add up to the reported total"
        )
    return normalized


def render_report(
    report: dict[str, object], summary: dict[str, int], profile: str
) -> str:
    result = "pass" if compliance_passed(summary) else "fail"
    lines = [
        f"# Redis Software {report['version_family']} compliance",
        "",
        f"**Outcome:** {result}",
        "",
        f"- Product version: `{report['server_version']}`",
        f"- Container image: `{report['image']}`",
        f"- Profile: `{profile}`",
        f"- Inventoried operations: `{summary['total']}`",
        f"- Pass: `{summary['pass']}`",
        f"- Known differences: `{summary['known_difference']}`",
        f"- Version-specific: `{summary['version_specific']}`",
        f"- Skipped: `{summary['skipped']}`",
        f"- Unsupported: `{summary['unsupported']}`",
        f"- Failures: `{summary['fail']}`",
        f"- Model dropped-field groups: `{summary['model_dropped_fields']}`",
        f"- Model failures: `{summary['model_failed']}`",
        "",
        "The source report contains only status metadata, field paths, and sanitized",
        "error classes; it does not contain response bodies or credentials.",
        "",
    ]
    return "\n".join(lines)


def compliance_passed(summary: dict[str, int]) -> bool:
    return (
        summary["fail"] == 0
        and summary["model_failed"] == 0
        and summary["model_dropped_fields"] == 0
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, help="Sanitized compliance JSON report")
    parser.add_argument("--output", required=True, help="Markdown summary path")
    parser.add_argument("--expected-version", required=True)
    parser.add_argument("--expected-image", required=True)
    parser.add_argument("--profile", choices=["safe", "writes"], required=True)
    args = parser.parse_args()

    try:
        report = load_report(Path(args.input))
        summary = validate_report(report, args.expected_version, args.expected_image)
    except ComplianceReportError as exc:
        print(f"error: invalid live-compliance report: {exc}", file=sys.stderr)
        return 1

    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(render_report(report, summary, args.profile), encoding="utf-8")
    print(f"Wrote sanitized compliance summary to {output}")
    if not compliance_passed(summary):
        print("error: compliance summary contains operation or model failures", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
