#!/usr/bin/env python3
"""Validate an llvm-cov JSON summary against reviewed repository thresholds."""

from __future__ import annotations

import argparse
import json
import math
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path

MAX_BASELINE_SLACK = 3.0
EXIT_REGRESSION = 1
EXIT_INPUT_FAILURE = 2


class CoverageInputError(RuntimeError):
    """Coverage configuration or report data is malformed."""


@dataclass(frozen=True)
class Metric:
    covered: int
    count: int
    percent: float


@dataclass(frozen=True)
class CoveragePolicy:
    baseline_lines: Metric
    baseline_functions: Metric
    minimum_lines: float
    minimum_functions: float
    measured_on: str
    tool: str


def _table(parent: object, key: str) -> dict[str, object]:
    if not isinstance(parent, dict) or not isinstance(parent.get(key), dict):
        raise CoverageInputError(f"missing or invalid {key!r} table")
    return parent[key]


def _number(table: dict[str, object], key: str) -> float:
    value = table.get(key)
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise CoverageInputError(f"{key!r} must be numeric")
    value = float(value)
    if not math.isfinite(value) or value < 0:
        raise CoverageInputError(f"{key!r} must be finite and nonnegative")
    return value


def _count(table: dict[str, object], key: str) -> int:
    value = table.get(key)
    if isinstance(value, bool) or not isinstance(value, int) or value <= 0:
        raise CoverageInputError(f"{key!r} must be a positive integer")
    return value


def _covered(table: dict[str, object], key: str) -> int:
    value = table.get(key)
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise CoverageInputError(f"{key!r} must be a nonnegative integer")
    return value


def _text(table: dict[str, object], key: str) -> str:
    value = table.get(key)
    if not isinstance(value, str) or not value.strip():
        raise CoverageInputError(f"{key!r} must be non-empty text")
    return value


def _metric(
    table: dict[str, object], covered_key: str, count_key: str, percent_key: str
) -> Metric:
    covered = _covered(table, covered_key)
    count = _count(table, count_key)
    percent = _number(table, percent_key)
    if covered > count:
        raise CoverageInputError(f"{covered_key!r} cannot exceed {count_key!r}")
    calculated = covered / count * 100
    if not math.isclose(calculated, percent, abs_tol=0.01):
        raise CoverageInputError(
            f"{percent_key!r} does not match {covered_key!r}/{count_key!r}"
        )
    return Metric(covered, count, percent)


def load_policy(path: Path) -> CoveragePolicy:
    try:
        payload = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise CoverageInputError(
            f"could not read coverage policy: {type(exc).__name__}"
        ) from exc
    if payload.get("schema") != 1:
        raise CoverageInputError("coverage policy schema must be 1")

    coverage = _table(payload, "coverage")
    baseline = _table(coverage, "baseline")
    minimum = _table(coverage, "minimum")
    baseline_lines = _metric(
        baseline, "line_covered", "line_count", "line_percent"
    )
    baseline_functions = _metric(
        baseline,
        "function_covered",
        "function_count",
        "function_percent",
    )
    minimum_lines = _number(minimum, "line_percent")
    minimum_functions = _number(minimum, "function_percent")

    for name, baseline_value, minimum_value in [
        ("line", baseline_lines.percent, minimum_lines),
        ("function", baseline_functions.percent, minimum_functions),
    ]:
        if minimum_value > baseline_value:
            raise CoverageInputError(f"{name} minimum exceeds its recorded baseline")
        if baseline_value - minimum_value > MAX_BASELINE_SLACK:
            raise CoverageInputError(
                f"{name} minimum is more than {MAX_BASELINE_SLACK:.1f} points below "
                "its recorded baseline"
            )

    return CoveragePolicy(
        baseline_lines=baseline_lines,
        baseline_functions=baseline_functions,
        minimum_lines=minimum_lines,
        minimum_functions=minimum_functions,
        measured_on=_text(baseline, "measured_on"),
        tool=_text(baseline, "tool"),
    )


def load_report(path: Path) -> tuple[Metric, Metric]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise CoverageInputError(
            f"could not read llvm-cov report: {type(exc).__name__}"
        ) from exc
    if not isinstance(payload, dict) or payload.get("type") != "llvm.coverage.json.export":
        raise CoverageInputError("report is not an llvm-cov JSON export")
    data = payload.get("data")
    if not isinstance(data, list) or len(data) != 1:
        raise CoverageInputError("llvm-cov report must contain exactly one data set")
    totals = _table(data[0], "totals")

    def report_metric(name: str) -> Metric:
        table = _table(totals, name)
        covered = _covered(table, "covered")
        count = _count(table, "count")
        percent = _number(table, "percent")
        if covered > count:
            raise CoverageInputError(f"{name} covered count exceeds total")
        calculated = covered / count * 100
        if not math.isclose(calculated, percent, abs_tol=0.01):
            raise CoverageInputError(f"{name} percentage does not match its counts")
        return Metric(covered, count, percent)

    return report_metric("lines"), report_metric("functions")


def coverage_passed(
    policy: CoveragePolicy, lines: Metric, functions: Metric
) -> bool:
    return (
        lines.percent >= policy.minimum_lines
        and functions.percent >= policy.minimum_functions
    )


def render_summary(
    policy: CoveragePolicy, lines: Metric, functions: Metric
) -> str:
    outcome = "pass" if coverage_passed(policy, lines, functions) else "fail"
    return "\n".join(
        [
            "# Rust coverage gate",
            "",
            f"**Outcome:** {outcome}",
            "",
            "| Metric | Current | Floor | Recorded baseline |",
            "|---|---:|---:|---:|",
            (
                f"| Lines | {lines.percent:.2f}% ({lines.covered}/{lines.count}) | "
                f"{policy.minimum_lines:.2f}% | {policy.baseline_lines.percent:.2f}% |"
            ),
            (
                f"| Functions | {functions.percent:.2f}% "
                f"({functions.covered}/{functions.count}) | "
                f"{policy.minimum_functions:.2f}% | "
                f"{policy.baseline_functions.percent:.2f}% |"
            ),
            "",
            f"Baseline measured {policy.measured_on} with `{policy.tool}`.",
            "The floor is enforced locally before any optional reporting upload.",
            "",
        ]
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", required=True, help="Reviewed quality-gates TOML")
    parser.add_argument("--report", required=True, help="llvm-cov summary JSON")
    parser.add_argument("--summary", required=True, help="Markdown result path")
    args = parser.parse_args()

    try:
        policy = load_policy(Path(args.config))
        lines, functions = load_report(Path(args.report))
    except CoverageInputError as exc:
        print(f"error: invalid coverage input: {exc}", file=sys.stderr)
        return EXIT_INPUT_FAILURE

    summary = render_summary(policy, lines, functions)
    output = Path(args.summary)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(summary, encoding="utf-8")
    print(summary)

    if not coverage_passed(policy, lines, functions):
        print("error: Rust coverage is below the reviewed floor", file=sys.stderr)
        return EXIT_REGRESSION
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
