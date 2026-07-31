from __future__ import annotations

import csv
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import audit_api_coverage as audit


class PathMatchingTests(unittest.TestCase):
    def test_concrete_paths_match_templates_but_specialized_templates_do_not(self) -> None:
        self.assertTrue(audit.template_matches("/v1/items/{uid}", "/v1/items/42"))
        self.assertTrue(audit.template_matches("/v1/items/{uid}", "/v1/items/{id}"))
        self.assertFalse(
            audit.template_matches(
                "/v1/items/{uid}/{action}", "/v1/items/{id}/upgrade"
            )
        )

    def test_normalization_removes_queries_and_placeholder_names(self) -> None:
        self.assertEqual(audit.normalize_path("/v1/items/<uid>/?view=full"), "/v1/items/{}")


class InventoryTests(unittest.TestCase):
    def test_inventory_deduplicates_method_and_path_while_preserving_pages(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            inventory = Path(directory) / "inventory.csv"
            with inventory.open("w", newline="", encoding="utf-8") as handle:
                writer = csv.DictWriter(
                    handle,
                    fieldnames=["page", "title", "method", "path", "sdk_module_guess"],
                )
                writer.writeheader()
                writer.writerows(
                    [
                        {
                            "page": "items",
                            "title": "Items",
                            "method": "GET",
                            "path": "/v1/items/{uid}",
                            "sdk_module_guess": "items",
                        },
                        {
                            "page": "items/detail",
                            "title": "Item detail",
                            "method": "GET",
                            "path": "/v1/items/<id>",
                            "sdk_module_guess": "items",
                        },
                        {
                            "page": "items",
                            "title": "Items",
                            "method": "POST",
                            "path": "/v1/items/{uid}",
                            "sdk_module_guess": "items",
                        },
                    ]
                )

            operations, raw_count = audit.load_inventory(inventory)

        self.assertEqual(raw_count, 3)
        self.assertEqual(len(operations), 2)
        get_operation = next(item for item in operations if item.key.method == "GET")
        self.assertEqual(get_operation.pages, {"items", "items/detail"})


class EvidenceExtractionTests(unittest.TestCase):
    def test_handler_extraction_is_method_aware_and_ignores_comments(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            src = root / "src"
            src.mkdir()
            (src / "items.rs").write_text(
                """
                // self.client.delete(\"/v1/items\").await
                self.client.get(\"/v1/items\").await;
                self.client.post(\"/v1/items\", &body).await;
                """,
                encoding="utf-8",
            )

            evidence = audit.extract_handler_evidence(src, root)

        self.assertIn(audit.OperationKey("GET", "/v1/items"), evidence.handler)
        self.assertIn(audit.OperationKey("POST", "/v1/items"), evidence.handler)
        self.assertNotIn(audit.OperationKey("DELETE", "/v1/items"), evidence.handler)

    def test_mock_dimensions_and_explicit_markers_stay_separate(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            tests = root / "tests"
            tests.mkdir()
            (tests / "live_items.rs").write_text(
                """
                #[ignore = \"requires live service\"]
                // api-audit-live: GET /v1/items
                // api-audit-response: GET /v1/items
                Mock::given(method(\"GET\"))
                    .and(path(\"/v1/items\"))
                    .and(query_param(\"limit\", \"1\"))
                    .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
                    .mount(&server).await;

                Mock::given(method(\"POST\"))
                    .and(path(\"/v1/items\"))
                    .and(body_json(json!({\"name\": \"one\"})))
                    .respond_with(ResponseTemplate::new(204))
                    .expect(1)
                    .mount(&server).await;

                let mention_only = \"/v1/orphan\";
                """,
                encoding="utf-8",
            )

            evidence = audit.extract_test_evidence(tests, root)

        get_key = audit.OperationKey("GET", "/v1/items")
        post_key = audit.OperationKey("POST", "/v1/items")
        self.assertIn(get_key, evidence.mock)
        self.assertIn(get_key, evidence.query)
        self.assertIn(get_key, evidence.response_fixture)
        self.assertNotIn(get_key, evidence.request_body)
        self.assertIn(get_key, evidence.live)
        self.assertIn(get_key, evidence.fixture_deserialization)
        self.assertIn(post_key, evidence.mock)
        self.assertIn(post_key, evidence.request_body)
        self.assertIn(post_key, evidence.mock_expectation)
        self.assertNotIn(post_key, evidence.response_fixture)
        self.assertIn("/v1/orphan", evidence.path_mentions)

    def test_get_mock_never_covers_post_operation(self) -> None:
        operation = audit.InventoryOperation(
            key=audit.OperationKey("POST", "/v1/items"),
            paths={"/v1/items"},
            pages={"items"},
            titles={"Items"},
        )
        evidence = audit.EvidenceIndex(
            handler={audit.OperationKey("POST", "/v1/items"): {"src/items.rs:1"}},
            mock={audit.OperationKey("GET", "/v1/items"): {"tests/items.rs:1"}},
            path_mentions={"/v1/items": {"tests/items.rs"}},
        )

        row = audit.build_audit_rows([operation], evidence)[0]

        self.assertEqual(row["handler_declared"], "true")
        self.assertEqual(row["mock_method_path"], "false")
        self.assertEqual(row["audit_status"], "handler_without_mock_evidence")
        self.assertEqual(row["uncertain_test_path_mentions"], "tests/items.rs")

    def test_live_marker_requires_an_ignored_live_test_file(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            tests = root / "tests"
            tests.mkdir()
            (tests / "items_tests.rs").write_text(
                "// api-audit-live: GET /v1/items\n", encoding="utf-8"
            )

            with self.assertRaisesRegex(ValueError, "ignored live_\\* test file"):
                audit.extract_test_evidence(tests, root)

    def test_specialized_concrete_mock_does_not_cover_generic_dispatch(self) -> None:
        operation = audit.InventoryOperation(
            key=audit.OperationKey("PUT", "/v1/items/{}/{}"),
            paths={"/v1/items/{uid}/{action}"},
            pages={"items"},
            titles={"Items"},
        )
        evidence = audit.EvidenceIndex(
            mock={audit.OperationKey("PUT", "/v1/items/1/upgrade"): {"tests/items.rs:1"}},
            request_body={
                audit.OperationKey("PUT", "/v1/items/1/upgrade"): {"tests/items.rs:1"}
            },
            path_mentions={"/v1/items/1/upgrade": {"tests/items.rs"}},
        )

        row = audit.build_audit_rows([operation], evidence)[0]

        self.assertEqual(row["mock_method_path"], "false")
        self.assertEqual(row["request_body_matcher"], "false")
        self.assertEqual(row["audit_status"], "docs_only")
        self.assertIn("ambiguous", row["notes"])


if __name__ == "__main__":
    unittest.main()
