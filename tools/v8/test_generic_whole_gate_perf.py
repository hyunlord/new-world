# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_perf
"""Synthetic retained-output contract tests; no native performance claim."""

import json
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path

from generic_gate import GateContractError
from generic_process import file_hash
from mux2_gate_inputs import Json, Node
from test_generic_perf_observations import fixture

from generic_whole_gate_perf import (
    Anchors,
    collect_performance,
    paired_performance,
    selected_runs,
)


def anchors(root: Path) -> Anchors:
    paths = (root, *root.rglob("*"))
    return Anchors(
        tuple((str(p), file_hash(p)) for p in paths if p.is_file()),
        tuple((str(p), p.lstat().st_mode) for p in paths),
    )


class PerfTests(unittest.TestCase):
    def test_nav_actual_default_script_horizon_and_pair(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            out = Path(temporary).resolve()
            raw = out / "nav_bench/nav_bench.json"
            raw.parent.mkdir()
            _, native, _ = fixture()
            native.update(
                requested_ticks=4320,
                requested_terminal_tick=4320,
                terminal_tick=4320,
                warmup_alive=10,
                interventions={"gifts": 1, "gift_requested": 10, "gift_applied": 10},
            )
            report = {
                "script": "soak_script.json",
                "agents": 1000,
                "alive_end": 8,
                "seed": 15100,
                "warmup_ticks": 1440,
                "measured_ticks": 2880,
                "build_profile": "release",
            }
            _ = raw.write_text(json.dumps(report))
            side = raw.parent / "gate_observations" / (raw.name + ".safety.json")
            side.parent.mkdir()
            _ = side.write_text(
                json.dumps(
                    {
                        "schema_version": 1,
                        "producer_pid": 42,
                        "raw_report_path": str(raw),
                        "raw_report_bytes": raw.read_text(),
                        "observation": native,
                    }
                )
            )
            original = anchors(out)
            left = collect_performance(27, "base", out, original)
            right = collect_performance(27, "candidate", out, original)
            self.assertEqual(
                Node(paired_performance(left, right)[0]).field("candidate_alive").value,
                8,
            )
            self.assertEqual(
                left.attribution, "SOURCE_QUALIFIED_AGGREGATE_WITH_OBSERVER_PID"
            )
            with self.assertRaises(GateContractError):
                _ = paired_performance(left, replace(right, worlds=()))
            _ = raw.write_text(raw.read_text() + " ")
            with self.assertRaisesRegex(GateContractError, "artifact changed"):
                _ = collect_performance(27, "base", out, original)

    def test_namespace_modes_and_missing_stress_never_pass(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            file = root / "data"
            _ = file.write_bytes(b"\xff\x00")
            original = anchors(root)
            original.subtree(root)
            self.assertEqual(original.bytes(file), b"\xff\x00")
            (root / "extra").mkdir()
            with self.assertRaises(GateContractError):
                original.subtree(root)
            with self.assertRaises((GateContractError, FileNotFoundError)):
                _ = selected_runs(12, root, original)

    def test_invalid_family_and_arm_reject(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            original = anchors(root)
            with self.assertRaises(GateContractError):
                _ = selected_runs(25, root, original)
            with self.assertRaises(GateContractError):
                _ = collect_performance(27, "other", root, original)


if __name__ == "__main__":
    _ = unittest.main()


class BindingTests(unittest.TestCase):
    def test_real_outer_receipt_recipe_is_checked_before_collection(self) -> None:
        from unittest.mock import patch

        from generic_whole_gate_perf import PerfEvidence
        from generic_whole_gate_perf_binding import authenticate_family

        with tempfile.TemporaryDirectory() as temporary:
            top = Path(temporary).resolve()
            root = top / "product"
            root.mkdir()
            out = top / "out"
            out.mkdir()
            process = out / "process"
            process.mkdir()
            sources = {}
            for name in (
                "rust/crates/sim-test/src/nav_bench.rs",
                "tools/v8/soak_script.json",
            ):
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                _ = path.write_text("synthetic source; semantic collector mocked")
                sources[name] = file_hash(path)
            for name in ("stdout", "stderr"):
                _ = (process / (name + ".log")).write_text("")
            argv = [
                "rust/target/release/worldsim_perf",
                "nav-bench",
                "--out",
                str(out / "base/nav_bench/nav_bench.json"),
            ]
            environment = {"WORLDSIM_GATE_OBSERVER": "1"}
            native = {
                "argv": ["/bin/bash", "-c", 'exec "$@" 2>&1', "whole-step", *argv],
                "pid": 42,
                "cwd": str(root),
                "environment": {k: v for k, v in environment.items()},
                "native_exit": 0,
                "status": "PASS",
                "errors": [],
                "stdout_sha256": file_hash(process / "stdout.log"),
                "stderr_sha256": file_hash(process / "stderr.log"),
            }
            receipt = process / "process.json"
            _ = receipt.write_text(json.dumps(native))
            stage: dict[str, Json] = {
                "step": 27,
                "arm": "base",
                "state": "RETURNED",
                "native_exit": 0,
                "pid": 42,
                "cwd": str(root),
                "environment": {k: v for k, v in environment.items()},
                "argv": [item for item in argv],
                "process_receipt": str(receipt),
                "process_receipt_sha256": file_hash(receipt),
                "observer_artifacts": {},
                "observer_modes": {},
            }
            session = Node(
                {
                    "output": str(out),
                    "bash": {"path": "/bin/bash"},
                    "arms": {"base": {"root": str(root), "source_inputs": sources}},
                }
            )
            semantic = PerfEvidence(27, "base", (), Anchors((), ()))
            with patch(
                "generic_whole_gate_perf_binding.collect_performance",
                return_value=semantic,
            ):
                self.assertEqual(authenticate_family(session, Node(stage)), semantic)
                stage["argv"] = [*argv, "--commands", "foreign"]
                with self.assertRaises(GateContractError):
                    _ = authenticate_family(session, Node(stage))
                stage["argv"] = [item for item in argv]
                native["argv"] = ["/bin/true"]
                _ = receipt.write_text(json.dumps(native))
                stage["process_receipt_sha256"] = file_hash(receipt)
                with self.assertRaises(GateContractError):
                    _ = authenticate_family(session, Node(stage))
