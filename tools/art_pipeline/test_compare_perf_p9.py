#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/art_pipeline -p test_compare_perf_p9.py
"""Exact reporting-boundary checks using synthetic evidence, never performance claims."""
from __future__ import annotations

import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path

import compare_perf_p9 as compare
import summarize_perf_p9 as perf
from test_summarize_perf_p9 import fixture


def sample(value: float, ticks: int = 0) -> compare.Record:
    return {"primary_cpu_ms": value, "process_ms": value / 2, "render_cpu_ms": value / 2,
            "setup_cpu_ms": 0, "gpu_ms": 0, "pacing_ms": 16, "ticks_run": ticks,
            "ffi_calls": {"get_time_snapshot": 19, "get_tile_causal_events": 1},
            "ffi_call_usec": {"get_time_snapshot": 7, "get_tile_causal_events": 3},
            "direct_read_context": {"inspector_visible": False, "influence_channel": -1},
            "state": {"population": 475}}


def inventory() -> compare.Record:
    return {"confirmed": True, "no_scaling": True, "source": "synthetic audited inventory",
            "causal_source": "sim_reads", "direct_calls": {"get_agent_detail": 0,
            "get_influence_overlay": 0, "get_tile_causal_events": 0}}


def identity() -> compare.Record:
    return {"framings": [{"framing": f"P{i}", "control": {"pixels": 0, "max_channel": 0},
            "candidate": {"pixels": 0, "max_channel": 0}, "capture_frame": 130,
            "state_change_frame": 10, "rgba_sha256": ["a" * 64] * 3} for i in range(1, 7)]}


class ComparePerfTests(unittest.TestCase):
    def test_inclusive_median_boundaries(self) -> None:
        for limit in (8.0, 12.0, 16.67):
            with self.subTest(limit=limit):
                self.assertEqual(compare.verdict([sample(20)], [sample(limit)], limit)["status"], "PASS")
                self.assertEqual(compare.verdict([sample(20)], [sample(limit + 0.0001)], limit)["status"], "FAIL")

    def test_p99_independent_boundary(self) -> None:
        values: list[perf.Json] = [sample(12)] * 98 + [sample(16.67)] * 2
        self.assertEqual(compare.verdict([sample(20)], values, 12, 16.67)["status"], "PASS")
        values[-1] = sample(16.6701)
        values[-2] = sample(16.6701)
        self.assertEqual(compare.verdict([sample(20)], values, 12, 16.67)["status"], "FAIL")

    def test_missing_evidence_is_not_verified(self) -> None:
        self.assertEqual(compare.verdict([], [sample(1)], 8)["status"], "NOT_VERIFIED")
        result = compare.compare({}, {}, {}, {}, {}, {})
        for row in perf.mapping(result["verdicts"]).values():
            self.assertEqual(perf.mapping(row)["status"], "NOT_VERIFIED")

    def test_ffi_exact_twenty_and_no_double_count(self) -> None:
        row = compare.ffi_verdict([sample(1)], inventory())
        self.assertEqual(row["status"], "PASS")
        self.assertEqual(perf.mapping(row["total_calls"])["maximum"], 20)
        self.assertEqual(row["baseline_counts"], "NOT_MEASURED")
        bad = inventory()
        bad["direct_calls"] = {"get_agent_detail": 0, "get_influence_overlay": 0, "get_tile_causal_events": 1}
        with self.assertRaisesRegex(perf.EvidenceError, "counted twice"):
            compare.ffi_verdict([sample(1)], bad)
        over = sample(1)
        over["ffi_calls"] = {"get_time_snapshot": 20, "get_tile_causal_events": 1}
        self.assertEqual(compare.ffi_verdict([over], inventory())["status"], "FAIL")

    def test_direct_causal_counted_once(self) -> None:
        row = sample(1)
        row["ffi_calls"] = {"get_time_snapshot": 19}
        proof = inventory()
        proof["causal_source"] = "direct"
        proof["direct_calls"] = {"get_agent_detail": 0, "get_influence_overlay": 0, "get_tile_causal_events": 1}
        result = compare.ffi_verdict([row], proof)
        self.assertEqual(perf.mapping(result["total_calls"])["maximum"], 20)

    def test_ffi_requires_inventory_and_measured_closed_context(self) -> None:
        self.assertEqual(compare.ffi_verdict([sample(1)], {})["status"], "NOT_VERIFIED")
        for context in ({}, {"inspector_visible": True, "influence_channel": -1}, {"inspector_visible": False, "influence_channel": 0}):
            with self.subTest(context=context):
                row = sample(1)
                row["direct_read_context"] = context
                self.assertEqual(compare.ffi_verdict([row], inventory())["status"], "NOT_VERIFIED")

    def test_missing_getter_timing_is_not_fabricated(self) -> None:
        row = sample(1)
        row["ffi_call_usec"] = {"get_time_snapshot": 7}
        result = compare.ffi_verdict([row], inventory())
        causal = perf.mapping(perf.mapping(result["getters"])["get_tile_causal_events"])
        self.assertIsNone(causal["microseconds"])
        self.assertNotIn("over_budget", perf.mapping(result["total_calls"]))

    def test_identity_requires_all_six_and_both_bounds(self) -> None:
        proof = identity()
        self.assertEqual(compare.identity_verdict(proof)["status"], "PASS")
        rows = list(perf.sequence(proof["framings"]))
        rows.pop()
        self.assertEqual(compare.identity_verdict({"framings": rows})["status"], "NOT_VERIFIED")
        rows = list(perf.sequence(identity()["framings"]))
        first = dict(perf.mapping(rows[0]))
        first["control"] = {"pixels": 2, "max_channel": 1}
        first["candidate"] = {"pixels": 1, "max_channel": 2}
        rows[0] = first
        self.assertEqual(compare.identity_verdict({"framings": rows})["status"], "FAIL")

    def test_tick_only_f3_and_headroom_actual_alive(self) -> None:
        before: compare.Record = {"W500": {"blocks": [
            {"label": "running", "condition": "running", "zoom": 4, "speed": 2, "samples": [sample(2), sample(30, 1)]},
            {"label": "frozen", "condition": "frozen", "zoom": 4, "speed": 0, "samples": [sample(10)]}]}}
        after = copy.deepcopy(before)
        data = dict(perf.mapping(after["W500"]))
        data["blocks"] = [{"label": "running", "condition": "running", "zoom": 4, "speed": 2, "samples": [sample(1)] * 99 + [sample(17, 1)]},
                          {"label": "frozen", "condition": "frozen", "zoom": 4, "speed": 0, "samples": [sample(10)]}]
        after["W500"] = data
        result = compare.compare(before, after, {}, {}, {}, {})
        rules = perf.mapping(result["verdicts"])
        self.assertEqual(perf.mapping(rules["F3"])["status"], "FAIL")
        headroom = perf.mapping(perf.mapping(rules["F6"])["after"])
        self.assertEqual(headroom["largest_requested_n_measured_meeting_rule"], 500)
        self.assertEqual(perf.mapping(headroom["actual_alive_by_requested_n"])["500"], 475)
        self.assertIn(2000, perf.sequence(headroom["unmeasured_n"]))
        self.assertEqual(perf.mapping(perf.mapping(result["conditions"])["W2000/frozen/Z4/0x"])["after_status"], "NOT_MEASURED")

    def test_ablation_cost_is_recomputed(self) -> None:
        data = fixture("W0")
        rows = list(perf.sequence(data["ablation"]))
        first = dict(perf.mapping(rows[0]))
        first["cost_ms"] = 999
        rows[0] = first
        data["ablation"] = rows
        self.assertEqual(perf.mapping(compare.ablation_rows(data)[0])["cost_ms"], 0)

    def test_aggregate_revalidates_raw_samples(self) -> None:
        with tempfile.TemporaryDirectory(prefix="p9-compare-test-") as directory:
            path = Path(directory) / "before.json"
            data = fixture("W1")
            path.write_text(json.dumps({"phase": "baseline", "sessions": [data]}))
            self.assertIn("W1", compare.aggregates(str(path), "baseline"))
            data["restoration"] = {}
            path.write_text(json.dumps({"phase": "baseline", "sessions": [data]}))
            with self.assertRaises(perf.EvidenceError):
                compare.aggregates(str(path), "baseline")

    def test_a30_recomputed_from_actual_artifact_not_claimed_median(self) -> None:
        with tempfile.TemporaryDirectory(prefix="p9-a30-test-") as directory:
            path = Path(directory) / "frame_times_soc_layers.json"
            data: compare.Record = {"valid": True, "errors": [], "median_primary_cpu_ms": {"on": 999, "off": 999},
                    "blocks": [{"layers_enabled": i % 2 == 0, "samples": [{"process_ms": 2, "render_cpu_ms": 1}] * 50} for i in range(12)]}
            path.write_text(json.dumps(data))
            result = compare.a30(str(path))
            self.assertEqual(perf.mapping(perf.mapping(result["layers"])["on"])["median"], 3)
            self.assertEqual(compare.a30("-"), {})

    def test_cli_writes_missing_evidence_without_pass(self) -> None:
        with tempfile.TemporaryDirectory(prefix="p9-cli-test-") as directory:
            prefix = Path(directory) / "comparison"
            original = sys.argv
            try:
                sys.argv = ["compare_perf_p9.py", "-", "-", "-", str(prefix)]
                compare.main()
            finally:
                sys.argv = original
            self.assertTrue(prefix.with_suffix(".json").is_file())
            self.assertNotIn("| PASS |", prefix.with_suffix(".md").read_text())

    def test_a30_native_payload_reports_even_when_own_threshold_fails(self) -> None:
        state: compare.Record = {"tick": 126994, "alive_count": 3, "population": 50,
                                 "simulation_frozen": True, "camera": [960, 540], "zoom": [1, 1]}
        blocks: list[perf.Json] = []
        for block in range(12):
            enabled = block % 2 == 0
            blocks.append({"block": block, "layers_enabled": enabled, "tick": 126994,
                           "camera": [960, 540], "samples": [{"process_ms": 3 if enabled else 1,
                           "render_cpu_ms": 1, "setup_cpu_ms": 0.1, "gpu_ms": 0,
                           "paired": True, "state": state, "frame_index": index,
                           "block_index": block, "layers_enabled": enabled} for index in range(50)]})
        native: compare.Record = {"blocks": blocks, "before": state, "after": state,
                  "discarded_transition_frames": [], "transition_frames_per_block": 3,
                  "vsync_disabled": True, "vsync_restored": True, "forced_draw": True,
                  "process_priorities": [-1000, 1000], "simulation_frozen": True,
                  "tick": 126994, "alive_count": 3, "population": 50, "framing": "FV",
                  "median_primary_cpu_ms": {"on": 4, "off": 2}, "threshold_pass": False,
                  "valid": True, "errors": [], "render_cpu_positive_samples": 600,
                  "selection_restored": True, "seed": 20261013, "run_id": "synthetic",
                  "git_sha": "synthetic", "load_before": {}, "load_after": {}}
        with tempfile.TemporaryDirectory(prefix="p9-a30-schema-") as directory:
            path = Path(directory) / "frame_times_soc_layers.json"
            path.write_text(json.dumps(native))
            measured = compare.a30(str(path))
            layers = perf.mapping(measured["layers"])
            self.assertEqual(perf.mapping(layers["on"])["median"], 4)
            self.assertEqual(perf.mapping(layers["off"])["median"], 2)
            result = compare.compare({}, {}, {}, {}, measured, measured)
            self.assertEqual(perf.mapping(perf.mapping(result["verdicts"])["F7"])["status"], "REPORT_ONLY")
            native["valid"] = False
            path.write_text(json.dumps(native))
            self.assertEqual(compare.a30(str(path)), {})

    def test_a30_rejects_partial_block_payload(self) -> None:
        with tempfile.TemporaryDirectory(prefix="p9-a30-partial-") as directory:
            path = Path(directory) / "frame_times_soc_layers.json"
            path.write_text(json.dumps({"valid": True, "errors": [], "blocks": []}))
            with self.assertRaisesRegex(perf.EvidenceError, "twelve 50-frame blocks"):
                compare.a30(str(path))


if __name__ == "__main__":
    unittest.main()
