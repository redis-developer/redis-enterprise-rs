from __future__ import annotations

import contextlib
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import check_coverage as coverage


ROOT = Path(__file__).resolve().parents[2]


def write_report(path: Path, line_percent: float, function_percent: float) -> None:
    line_count = 1000
    function_count = 1000
    payload = {
        "type": "llvm.coverage.json.export",
        "data": [
            {
                "totals": {
                    "lines": {
                        "count": line_count,
                        "covered": round(line_percent * 10),
                        "percent": line_percent,
                    },
                    "functions": {
                        "count": function_count,
                        "covered": round(function_percent * 10),
                        "percent": function_percent,
                    },
                }
            }
        ],
    }
    path.write_text(json.dumps(payload), encoding="utf-8")


class CoveragePolicyTests(unittest.TestCase):
    def test_checked_in_baseline_and_floor_are_consistent(self) -> None:
        policy = coverage.load_policy(ROOT / "quality-gates.toml")

        self.assertAlmostEqual(policy.baseline_lines.percent, 62.33588621444201)
        self.assertAlmostEqual(policy.baseline_functions.percent, 60.44071353620147)
        self.assertEqual(policy.minimum_lines, 60.0)
        self.assertEqual(policy.minimum_functions, 58.0)

    def test_current_style_report_passes_and_renders_counts(self) -> None:
        policy = coverage.load_policy(ROOT / "quality-gates.toml")
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "coverage.json"
            write_report(report, 62.3, 60.4)
            lines, functions = coverage.load_report(report)
            summary = coverage.render_summary(policy, lines, functions)

        self.assertTrue(coverage.coverage_passed(policy, lines, functions))
        self.assertIn("**Outcome:** pass", summary)
        self.assertIn("623/1000", summary)

    def test_material_regression_fails_the_cli_after_writing_summary(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report = root / "coverage.json"
            summary = root / "summary.md"
            write_report(report, 59.9, 57.9)
            argv = [
                "check_coverage.py",
                "--config",
                str(ROOT / "quality-gates.toml"),
                "--report",
                str(report),
                "--summary",
                str(summary),
            ]
            with (
                mock.patch.object(sys, "argv", argv),
                contextlib.redirect_stdout(io.StringIO()),
                contextlib.redirect_stderr(io.StringIO()),
            ):
                exit_code = coverage.main()
            rendered = summary.read_text(encoding="utf-8")

        self.assertEqual(exit_code, coverage.EXIT_REGRESSION)
        self.assertIn("**Outcome:** fail", rendered)

    def test_non_llvm_report_is_an_input_failure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "coverage.json"
            report.write_text("{}", encoding="utf-8")
            with self.assertRaises(coverage.CoverageInputError):
                coverage.load_report(report)

    def test_zero_coverage_is_a_regression_not_malformed_input(self) -> None:
        policy = coverage.load_policy(ROOT / "quality-gates.toml")
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "coverage.json"
            write_report(report, 0.0, 0.0)
            lines, functions = coverage.load_report(report)

        self.assertFalse(coverage.coverage_passed(policy, lines, functions))

    def test_floor_cannot_drift_far_below_recorded_baseline(self) -> None:
        source = (ROOT / "quality-gates.toml").read_text(encoding="utf-8")
        with tempfile.TemporaryDirectory() as directory:
            config = Path(directory) / "quality-gates.toml"
            config.write_text(
                source.replace("line_percent = 60.0", "line_percent = 50.0"),
                encoding="utf-8",
            )
            with self.assertRaises(coverage.CoverageInputError):
                coverage.load_policy(config)


class DependencyPolicyTests(unittest.TestCase):
    def test_security_policy_has_no_unreviewed_ignores_or_sources(self) -> None:
        import tomllib

        policy = tomllib.loads((ROOT / "deny.toml").read_text(encoding="utf-8"))

        self.assertEqual(policy["advisories"]["ignore"], [])
        self.assertEqual(policy["bans"]["wildcards"], "deny")
        self.assertEqual(policy["sources"]["unknown-registry"], "deny")
        self.assertEqual(policy["sources"]["unknown-git"], "deny")
        self.assertEqual(policy["sources"]["allow-git"], [])

    def test_ci_requires_local_gates_and_isolates_optional_reporting(self) -> None:
        workflow = (ROOT / ".github" / "workflows" / "ci.yml").read_text(
            encoding="utf-8"
        )

        self.assertIn("EmbarkStudios/cargo-deny-action@v2.1.1", workflow)
        self.assertIn("taiki-e/install-action@v2.85.7", workflow)
        self.assertIn("cargo-llvm-cov@0.8.7", workflow)
        self.assertIn("python3 scripts/check_coverage.py", workflow)
        self.assertIn("codecov/codecov-action@v5.5.5", workflow)
        self.assertIn("continue-on-error: true", workflow)
        self.assertIn("needs: [check, audit, test, coverage, build]", workflow)


if __name__ == "__main__":
    unittest.main()
