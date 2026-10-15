#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/art_pipeline -p test_compare_perf_p10.py
"""Synthetic boundary evidence only; no measured-performance claims."""
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

import compare_perf_p10 as compare
import summarize_perf_p10 as perf


def sample(value: float, ticks: int = 0, population: int = 500) -> compare.Record:
    return {"primary_cpu_ms": value, "process_ms": value / 2, "render_cpu_ms": value / 2,
            "setup_cpu_ms": 0, "gpu_ms": 0, "pacing_ms": 16, "ticks_run": ticks,
            "state": {"population": population}}


def sessions(value: float) -> compare.Record:
    return {world: {"blocks": [{"condition": "frozen", "zoom": 4, "speed": 0, "label": "frozen", "samples": [sample(value)]}]} for world in ("W0", "W200", "W500")}


def evidence() -> compare.Evidence:
    return compare.Evidence({}, {}, {}, {})


class ComparePerfTests(unittest.TestCase):
    def test_exact_h1_h2_h3_thresholds(self) -> None:
        for key, limit in (("H1", 6.0), ("H2", 12.0), ("H3", 16.67)):
            for value, expected in ((limit, "PASS"), (limit + 0.0001, "FAIL")):
                result = compare.compare(sessions(50), sessions(value), evidence())
                self.assertEqual(perf.mapping(perf.mapping(result["verdicts"])[key])["status"], expected)
        self.assertEqual(set(perf.mapping(result["verdicts"])), {f"H{i}" for i in range(1, 9)})

    def test_h2_p99_is_independent(self) -> None:
        values: list[perf.Json] = [sample(12)] * 98 + [sample(16.67)] * 2
        self.assertEqual(compare.verdict([sample(40)], values, 12, 16.67)["status"], "PASS")
        values[-2:] = [sample(16.6701)] * 2
        self.assertEqual(compare.verdict([sample(40)], values, 12, 16.67)["status"], "FAIL")

    def test_all_missing_never_pass(self) -> None:
        result = compare.compare({}, {}, evidence())
        for row in perf.mapping(result["verdicts"]).values():
            self.assertEqual(perf.mapping(row)["status"], "NOT_VERIFIED")
        self.assertEqual(compare.verdict([], [sample(1)], 6)["status"], "NOT_VERIFIED")

    def test_h4_uses_tick_frames_and_reports_population(self) -> None:
        before: compare.Record = {"W500": {"blocks": [{"label": "running", "condition": "running", "zoom": 4, "speed": 2, "samples": [sample(1, 0, 500), sample(17, 1, 100)]}]}}
        result = compare.compare(before, before, evidence())
        h4 = perf.mapping(perf.mapping(result["verdicts"])["H4"])
        self.assertEqual(h4["status"], "FAIL")
        context = perf.mapping(h4["after_context"])
        self.assertEqual(context["population_start"], 500)
        self.assertEqual(context["population_end"], 100)
        self.assertEqual(context["ticks_per_frame"], 0.5)

    def test_pixels_alone_do_not_pass_h5(self) -> None:
        identity: compare.Record = {"framings": [{"framing": f"P{i}", "control": {"pixels": 0, "max_channel": 0}, "candidate": {"pixels": 0, "max_channel": 0}, "capture_frame": 130, "state_change_frame": 10, "rgba_sha256": ["a" * 64] * 3} for i in range(1, 7)]}
        self.assertEqual(compare.identity_verdict(identity)["status"], "PASS")
        result = compare.compare({}, {}, compare.Evidence(identity, {}, {}, {}))
        self.assertEqual(perf.mapping(perf.mapping(result["verdicts"])["H5"])["status"], "NOT_VERIFIED")

    def test_running_rejects_dummy_dumps_and_missing_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            folder = Path(temporary)
            rows: list[perf.Json] = []
            for world in ("W0", "W200"):
                path = folder / f"{world}.jsonl"
                path.write_text('{}\n' * 600)
                rows.append({"world": world, "base_dump": str(path), "candidate_dump": str(path)})
            self.assertEqual(compare.running_verdict({"worlds": rows})["status"], "NOT_VERIFIED")
            for world in ("W0", "W200"):
                (folder / f"{world}.json").write_text('{}')
            self.assertEqual(compare.running_verdict({"worlds": rows})["status"], "FAIL")
            self.assertEqual(compare.running_verdict({"worlds": rows[:1]})["status"], "NOT_VERIFIED")

    def test_self_reported_counts_cannot_pass_h7_h8(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "empty.json"
            path.write_text('{}')
            claim: compare.Record = {"source": str(path), "passed": 100, "failed": 0}
            self.assertEqual(compare.activity_verdict({"unit_tests": claim, "sidecar_check": claim, "blind": claim})["status"], "NOT_VERIFIED")
            self.assertEqual(compare.inspector_verdict({"tests": claim, "cases": ["all_fields", "missing_component", "dead_partner", "click_selects"]})["status"], "NOT_VERIFIED")

    def test_inspector_requires_real_rows_and_every_case(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "inspector.json"
            rows: list[perf.Json] = [{"name": name, "passed": True} for name in ("all_fields", "missing_component", "dead_partner", "click_selects")]
            for selected, expected in ((rows, "PASS"), (rows[:3], "NOT_VERIFIED"), (rows + [{"name": "extra", "passed": False}], "FAIL")):
                path.write_text(json.dumps({"valid": True, "tests": selected}))
                self.assertEqual(compare.inspector_verdict({"tests": {"source": str(path)}})["status"], expected)

    def test_activities_full_coverage_and_blind_has_no_accuracy_bar(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            folder = Path(temporary)
            units, sidecars, blind = (folder / name for name in ("units.json", "sidecars.json", "blind.json"))
            units.write_text(json.dumps({"valid": True, "resolve_cases": [{"state": 4, "kind": kind, "passed": True} for kind in range(7)], "pose_cases": [{"kind": kind, "body": body, "view": view, "passed": True} for kind in range(7) for body in ("adult_a", "adult_b", "child") for view in ("E", "N", "S", "W")]}))
            captures: list[perf.Json] = [{"id": case, "zoom": zoom, "holders": [{"kind": 0, "expected_pose": "sit", "drawn_pose": "sit"}]} for case in ("day30_1000", "day30_1500", "day60_1000") for zoom in (1, 4)]
            sidecars.write_text(json.dumps({"valid": True, "captures": captures}))
            blind.write_text(json.dumps({"valid": True, "answers_frozen_before_key": True, "answers": [{"expected_kind": kind, "predicted_kind": (kind + 1) % 7} for kind in range(7) for _ in range(2)]}))
            payload: compare.Record = {"unit_tests": {"source": str(units)}, "sidecar_check": {"source": str(sidecars)}, "blind": {"source": str(blind)}}
            result = compare.activity_verdict(payload)
            self.assertEqual(result["status"], "PASS")
            self.assertEqual(result["blind_correct_share"], 0)
            sidecars.write_text(json.dumps({"valid": True, "captures": captures[:5]}))
            self.assertEqual(compare.activity_verdict(payload)["status"], "NOT_VERIFIED")

    def test_markdown_displays_new_rules_without_legacy_verdicts(self) -> None:
        rendered = compare.markdown(compare.compare({}, {}, evidence()))
        self.assertIn('| H8 | NOT_VERIFIED |', rendered)
        self.assertNotIn('| F', rendered)


if __name__ == "__main__":
    unittest.main()
