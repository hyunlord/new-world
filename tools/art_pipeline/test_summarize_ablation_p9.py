#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/art_pipeline -p test_summarize_ablation_p9.py
"""Ablation evidence boundaries tested with synthetic sessions, never timed worlds."""
from __future__ import annotations

import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path

import summarize_ablation_p9 as ablation
import summarize_perf_p9 as perf
from test_summarize_perf_p9 import fixture, mutable, items


def ablation_fixture() -> ablation.Record:
    data = fixture("W500", "after")
    data["ablation_only"] = True
    data["blocks"] = [block for block in items(data["blocks"]) if str(mutable(block)["condition"]).startswith("ablation")]
    return data


def first_sample(data: ablation.Record) -> ablation.Record:
    return mutable(items(mutable(items(data["blocks"])[0])["samples"])[0])


class AblationEvidenceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="p9-ablation-test-")
        self.addCleanup(self.temporary.cleanup)
        self.path = Path(self.temporary.name) / "session.json"

    def load(self, data: ablation.Record) -> ablation.Record:
        self.path.write_text(json.dumps(data))
        return ablation.load(self.path)

    def test_ablation_only_does_not_weaken_full_matrix_loader(self) -> None:
        result = self.load(ablation_fixture())
        self.assertEqual(len(items(result["ranked"])), 4)
        with self.assertRaisesRegex(perf.EvidenceError, "ablation-only"):
            perf.load_session(self.path)

    def test_full_w500_baseline_and_after_accepted(self) -> None:
        for phase in ("baseline", "after"):
            with self.subTest(phase=phase):
                self.assertEqual(len(items(self.load(fixture("W500", phase))["ranked"])), 4)

    def test_costs_recomputed_and_negative_cost_retained(self) -> None:
        data = ablation_fixture()
        for row in items(data["ablation"]):
            mutable(row)["cost_ms"] = 999
        for raw_block in items(data["blocks"]):
            block = mutable(raw_block)
            if block["label"] in ("off_AgentRenderer", "off_Shadows"):
                value = 1.0 if block["label"] == "off_AgentRenderer" else 5.0
                for raw in [*items(block["discarded"]), *items(block["samples"])]:
                    mutable(raw)["process_ms"] = value / 2
                    mutable(raw)["render_cpu_ms"] = value / 2
                    mutable(raw)["primary_cpu_ms"] = value
        result = self.load(data)
        ranked = [mutable(row) for row in items(result["ranked"])]
        self.assertEqual(ranked[0]["node"], "AgentRenderer")
        self.assertEqual(ranked[0]["cost_ms"], 2)
        self.assertEqual(ranked[-1]["cost_ms"], -2)
        self.assertEqual(mutable(ranked[0]["neighbour_statistics"])["count"], 300)
        self.assertEqual(mutable(ranked[0]["off_statistics"])["count"], 150)

    def test_invalid_flags_and_restoration_rejected(self) -> None:
        for key, value in (("valid", False), ("errors", ["failure"]), ("restoration", {}), ("ablation_visibility_restored", False)):
            with self.subTest(key=key):
                data = ablation_fixture()
                data[key] = value
                with self.assertRaises(perf.EvidenceError):
                    self.load(data)

    def test_invalid_sample_boundaries_rejected(self) -> None:
        for key, value in (("paired", False), ("process_frame", -1), ("ffi_counter_frame", -1),
                           ("primary_cpu_ms", 999), ("ticks_run", 1),
                           ("direct_read_context", {"inspector_visible": True, "influence_channel": -1})):
            with self.subTest(key=key):
                data = ablation_fixture()
                first_sample(data)[key] = value
                with self.assertRaises(perf.EvidenceError):
                    self.load(data)
        data = ablation_fixture()
        del first_sample(data)["ffi_calls"]
        with self.assertRaises(perf.EvidenceError):
            self.load(data)

    def test_sample_counts_and_neighbour_adjacency_rejected(self) -> None:
        for key in ("samples", "discarded"):
            with self.subTest(key=key):
                data = ablation_fixture()
                items(mutable(items(data["blocks"])[0])[key]).pop()
                with self.assertRaisesRegex(perf.EvidenceError, "150 counted and 30 discarded"):
                    self.load(data)
        data = ablation_fixture()
        mutable(items(data["ablation"])[0])["neighbour_blocks"] = ["ablation_restored", "on_after_WorldObjects"]
        with self.assertRaisesRegex(perf.EvidenceError, "adjacent"):
            self.load(data)

    def test_missing_shared_layer_or_floor_rejected(self) -> None:
        data = ablation_fixture()
        data["ablation"] = [row for row in items(data["ablation"]) if mutable(row)["node"] != "WorldObjects"]
        with self.assertRaisesRegex(perf.EvidenceError, "shared-layer"):
            self.load(data)
        data = ablation_fixture()
        data["blocks"] = [block for block in items(data["blocks"]) if mutable(block)["label"] != "ablation_floor"]
        with self.assertRaisesRegex(perf.EvidenceError, "blocks/floor"):
            self.load(data)

    def test_row_comparison_requires_same_group(self) -> None:
        result = self.load(ablation_fixture())
        before = copy.deepcopy(result)
        mutable(items(before["ranked"])[0])["cost_ms"] = 2
        rows = ablation.comparison(result, before)
        self.assertEqual(mutable(rows[0])["saved_ms"], 2)
        mutable(items(before["ranked"])[0])["descendants_disabled_as_group"] = ["different"]
        rows = ablation.comparison(result, before)
        self.assertEqual(mutable(rows[0])["status"], "NOT_COMPARABLE")
        self.assertIsNone(mutable(rows[0])["saved_ms"])

    def test_cli_outputs_ranked_table_and_optional_comparison(self) -> None:
        self.path.write_text(json.dumps(ablation_fixture()))
        prefix = self.path.parent / "ranked"
        original = sys.argv
        try:
            sys.argv = ["summarize_ablation_p9.py", str(self.path), str(prefix), str(self.path)]
            ablation.main()
        finally:
            sys.argv = original
        raw: perf.Json = json.loads(prefix.with_suffix(".json").read_text())
        self.assertEqual(len(items(mutable(raw)["top_three"])), 3)
        self.assertIn("COMPARABLE", prefix.with_suffix(".md").read_text())
        self.assertIn("Saved ms", prefix.with_suffix(".md").read_text())


if __name__ == "__main__":
    unittest.main()
