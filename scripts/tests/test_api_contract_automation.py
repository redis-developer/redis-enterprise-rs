from __future__ import annotations

import csv
import contextlib
import io
import json
import ssl
import sys
import tempfile
import unittest
import urllib.error
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import check_api_inventory_drift as drift
import export_api_inventory as exporter
import summarize_live_compliance as summary


def write_inventory(path: Path, rows: list[dict[str, str]]) -> None:
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(
            handle, fieldnames=["page", "method", "path", "description"]
        )
        writer.writeheader()
        writer.writerows({"page": "items", **row} for row in rows)


class InventoryDriftTests(unittest.TestCase):
    def test_comparison_is_semantic_and_deduplicated(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            expected_path = root / "expected.csv"
            actual_path = root / "actual.csv"
            write_inventory(
                expected_path,
                [
                    {"method": "get", "path": "/v1/items/{uid}", "description": "old"},
                    {"method": "GET", "path": "/v1/items/{id}", "description": "duplicate"},
                    {"method": "POST", "path": "/v1/items", "description": "create"},
                ],
            )
            write_inventory(
                actual_path,
                [
                    {
                        "method": "GET",
                        "path": "/v1/items/<item_id>/?verbose=true",
                        "description": "changed copy",
                    },
                    {"method": "POST", "path": "/v1/items/", "description": "new copy"},
                ],
            )

            expected = drift.load_operations(expected_path)
            actual = drift.load_operations(actual_path)

        self.assertEqual(expected, actual)
        self.assertEqual(drift.compare(expected, actual), ([], []))

    def test_added_and_removed_operations_are_reported_separately(self) -> None:
        expected = {
            drift.Operation("GET", "/v1/old"),
            drift.Operation("GET", "/v1/shared"),
        }
        actual = {
            drift.Operation("POST", "/v1/new"),
            drift.Operation("GET", "/v1/shared"),
        }
        added, removed = drift.compare(expected, actual)
        report = drift.render_report(len(expected), len(actual), added, removed)

        self.assertEqual(added, [drift.Operation("POST", "/v1/new")])
        self.assertEqual(removed, [drift.Operation("GET", "/v1/old")])
        self.assertIn("semantic drift detected", report)
        self.assertIn("`POST /v1/new`", report)
        self.assertIn("`GET /v1/old`", report)

    def test_malformed_inventory_is_a_comparison_failure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "bad.csv"
            path.write_text("description\nmissing operation\n", encoding="utf-8")
            with self.assertRaises(drift.InventoryFormatError):
                drift.load_operations(path)

    def test_cli_writes_machine_readable_semantic_drift(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            expected = root / "expected.csv"
            actual = root / "actual.csv"
            report = root / "report.md"
            status = root / "status.json"
            crawl_status = root / "crawl-status.json"
            write_inventory(
                expected,
                [{"method": "GET", "path": "/v1/old", "description": "old"}],
            )
            write_inventory(
                actual,
                [{"method": "GET", "path": "/v1/new", "description": "new"}],
            )
            crawl_status.write_text(
                json.dumps(
                    {
                        "classification": "success",
                        "pages_without_method_rows": [],
                    }
                ),
                encoding="utf-8",
            )
            argv = [
                "check_api_inventory_drift.py",
                "--expected",
                str(expected),
                "--actual",
                str(actual),
                "--crawl-status",
                str(crawl_status),
                "--report",
                str(report),
                "--status-output",
                str(status),
            ]
            with (
                mock.patch.object(sys, "argv", argv),
                contextlib.redirect_stderr(io.StringIO()),
            ):
                exit_code = drift.main()
            payload = json.loads(status.read_text(encoding="utf-8"))

        self.assertEqual(exit_code, drift.EXIT_SEMANTIC_DRIFT)
        self.assertEqual(payload["classification"], "semantic_drift")
        self.assertEqual(payload["added"], ["GET /v1/new"])
        self.assertEqual(payload["removed"], ["GET /v1/old"])

    def test_previously_inventoried_empty_page_is_a_parser_failure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            expected = root / "expected.csv"
            actual = root / "actual.csv"
            crawl_status = root / "crawl-status.json"
            report = root / "report.md"
            status = root / "status.json"
            write_inventory(
                expected,
                [
                    {
                        "page": "users",
                        "method": "GET",
                        "path": "/v1/users",
                        "description": "users",
                    }
                ],
            )
            write_inventory(
                actual,
                [
                    {
                        "page": "other",
                        "method": "GET",
                        "path": "/v1/other",
                        "description": "other",
                    }
                ],
            )
            crawl_status.write_text(
                json.dumps(
                    {
                        "classification": "success",
                        "pages_without_method_rows": ["users", "_index"],
                    }
                ),
                encoding="utf-8",
            )
            argv = [
                "check_api_inventory_drift.py",
                "--expected",
                str(expected),
                "--actual",
                str(actual),
                "--crawl-status",
                str(crawl_status),
                "--report",
                str(report),
                "--status-output",
                str(status),
            ]
            with (
                mock.patch.object(sys, "argv", argv),
                contextlib.redirect_stderr(io.StringIO()),
            ):
                exit_code = drift.main()
            payload = json.loads(status.read_text(encoding="utf-8"))

        self.assertEqual(exit_code, drift.EXIT_COMPARISON_FAILURE)
        self.assertEqual(payload["classification"], "parse_failure")
        self.assertIn("users", payload["message"])


class ExportFailureClassificationTests(unittest.TestCase):
    def test_network_failure_is_not_a_parse_or_semantic_failure(self) -> None:
        with mock.patch.object(
            exporter.urllib.request,
            "urlopen",
            side_effect=urllib.error.URLError("offline"),
        ):
            with self.assertRaises(exporter.DocsFetchError) as context:
                exporter.fetch_text("https://redis.io/docs/example")

        self.assertEqual(context.exception.category, "network")
        self.assertEqual(context.exception.url, "https://redis.io/docs/example")

    def test_tls_failure_is_a_distinct_fetch_failure(self) -> None:
        tls_error = ssl.SSLCertVerificationError(1, "untrusted issuer")
        with mock.patch.object(
            exporter.urllib.request,
            "urlopen",
            side_effect=urllib.error.URLError(tls_error),
        ):
            with self.assertRaises(exporter.DocsFetchError) as context:
                exporter.fetch_text("https://redis.io/docs/example")

        self.assertEqual(context.exception.category, "tls")

    def test_empty_parsed_inventory_is_a_parser_failure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            with (
                mock.patch.object(
                    exporter,
                    "discover_request_pages",
                    return_value=[exporter.REQUESTS_ROOT],
                ),
                mock.patch.object(exporter, "fetch_text", return_value="# No endpoint table"),
            ):
                with self.assertRaises(exporter.DocsParseError):
                    exporter.export_inventory(Path(directory) / "inventory.csv")

    def test_status_artifact_contains_only_classification_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "status.json"
            exporter.write_status(
                path,
                {"classification": "fetch_failure", "category": "network"},
            )
            payload = json.loads(path.read_text(encoding="utf-8"))

        self.assertEqual(
            payload,
            {"classification": "fetch_failure", "category": "network"},
        )

    def test_cli_records_parse_failure_with_distinct_exit_code(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            status = root / "status.json"
            argv = [
                "export_api_inventory.py",
                "--output",
                str(root / "inventory.csv"),
                "--status-output",
                str(status),
            ]
            with (
                mock.patch.object(sys, "argv", argv),
                mock.patch.object(
                    exporter,
                    "export_inventory",
                    side_effect=exporter.DocsParseError("layout changed"),
                ),
                contextlib.redirect_stderr(io.StringIO()),
            ):
                exit_code = exporter.main()
            payload = json.loads(status.read_text(encoding="utf-8"))

        self.assertEqual(exit_code, exporter.EXIT_PARSE_FAILURE)
        self.assertEqual(payload["classification"], "parse_failure")


class ComplianceSummaryTests(unittest.TestCase):
    def report(self) -> dict[str, object]:
        return {
            "server_version": "8.2.0-25",
            "version_family": "8.2",
            "image": "redislabs/redis:8.2.0-25.12",
            "summary": {
                "total": 203,
                "pass": 85,
                "known_difference": 3,
                "version_specific": 5,
                "skipped": 110,
                "unsupported": 0,
                "fail": 0,
                "model_dropped_fields": 0,
                "model_failed": 0,
            },
        }

    def test_summary_requires_exact_image_and_product_version(self) -> None:
        normalized = summary.validate_report(
            self.report(),
            "8.2.0-25",
            "redislabs/redis:8.2.0-25.12",
        )
        rendered = summary.render_report(self.report(), normalized, "safe")

        self.assertIn("**Outcome:** pass", rendered)
        self.assertIn("`redislabs/redis:8.2.0-25.12`", rendered)
        self.assertIn("Model failures: `0`", rendered)
        self.assertTrue(summary.compliance_passed(normalized))

    def test_summary_rejects_wrong_matrix_provenance(self) -> None:
        with self.assertRaises(summary.ComplianceReportError):
            summary.validate_report(
                self.report(),
                "8.0.20-68",
                "redislabs/redis:8.2.0-25.12",
            )

    def test_summary_rejects_boolean_or_negative_counts(self) -> None:
        report = self.report()
        report["summary"]["fail"] = True  # type: ignore[index]
        with self.assertRaises(summary.ComplianceReportError):
            summary.validate_report(
                report,
                "8.2.0-25",
                "redislabs/redis:8.2.0-25.12",
            )

    def test_summary_rejects_inconsistent_operation_totals(self) -> None:
        report = self.report()
        report["summary"]["total"] = 204  # type: ignore[index]
        with self.assertRaises(summary.ComplianceReportError):
            summary.validate_report(
                report,
                "8.2.0-25",
                "redislabs/redis:8.2.0-25.12",
            )

    def test_summary_fails_when_model_fields_are_dropped(self) -> None:
        normalized = summary.validate_report(
            self.report(),
            "8.2.0-25",
            "redislabs/redis:8.2.0-25.12",
        )
        normalized["model_dropped_fields"] = 1
        self.assertFalse(summary.compliance_passed(normalized))


class WorkflowContractTests(unittest.TestCase):
    def test_workflow_is_external_only_and_pins_the_support_matrix(self) -> None:
        workflow = (
            Path(__file__).resolve().parents[2]
            / ".github"
            / "workflows"
            / "enterprise-contract.yml"
        ).read_text(encoding="utf-8")

        self.assertIn('cron: "17 6 * * 1"', workflow)
        self.assertIn("workflow_dispatch:", workflow)
        self.assertNotIn("pull_request:", workflow)
        self.assertNotIn("push:", workflow)
        for image in [
            "redislabs/redis:8.2.0-25.12",
            "redislabs/redis:8.0.20-68.25",
            "redislabs/redis:7.22.2-170",
            "redislabs/redis:7.8.6-286",
            "redislabs/redis:7.4.6-272",
        ]:
            self.assertIn(image, workflow)

        compose = (
            Path(__file__).resolve().parents[2] / "docker-compose.yml"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "ghcr.io/redis-developer/redisctl@sha256:"
            "2d8b5148226705ae4c057611c4adcd2c9ba08cac0c02cec1c449fc31b92a2026",
            compose,
        )
        self.assertNotIn("redisctl:latest", compose)

    def test_scheduled_profile_is_safe_and_writes_require_manual_dispatch(self) -> None:
        workflow = (
            Path(__file__).resolve().parents[2]
            / ".github"
            / "workflows"
            / "enterprise-contract.yml"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "github.event_name == 'workflow_dispatch' && inputs.run_disposable_writes",
            workflow,
        )
        self.assertIn("REDIS_ENTERPRISE_LIVE_WRITES=true", workflow)
        self.assertIn("docker compose --profile init down -v", workflow)


if __name__ == "__main__":
    unittest.main()
