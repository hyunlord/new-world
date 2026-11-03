# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_stall_capture
"""Pure recipe and rejection checks; no native capture or timing qualification."""

import json
import os
import tempfile
import unittest
from contextlib import nullcontext
from pathlib import Path
from unittest.mock import patch

from gate_stall_child import ChildPlan
from generic_gate import GateContractError, document
from generic_process import file_hash
from mux2_gate_inputs import BRIDGE, EvidenceError, Json, Node
from test_generic_stall import report_with

from generic_whole_gate_stall import OtherStep
from generic_whole_gate_stall_binding import conjunction, evaluate_stage
from generic_whole_gate_stall_capture import (
    capture,
    make_plan,
    plan_inputs,
    strict_result,
)


class CaptureTests(unittest.TestCase):
    def test_real_plan_selects_bound_bridge_and_original_recipe(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            base = Path(tmp).resolve()
            root, out = base / "root", base / "output"
            root.mkdir()
            binaries = [root / BRIDGE, base / "godot", base / "python", base / "bash"]
            for binary in binaries:
                binary.parent.mkdir(parents=True, exist_ok=True)
                _ = binary.write_text(str(binary))
            bridge, godot, python, bash = binaries
            source = root / "tools/v8/mux2_stall_test.gd"
            source.parent.mkdir(parents=True)
            _ = source.write_text("fixture")
            source_inputs: dict[str, Json] = {
                "tools/v8/mux2_stall_test.gd": file_hash(source)
            }
            session = Node(
                {
                    "output": str(out),
                    "arms": {
                        "candidate": {"root": str(root)},
                        "base": {"root": str(root)},
                    },
                    "environment": {"GODOT_BIN": str(godot)},
                    "external_inputs": {str(godot): file_hash(godot)},
                    "python": {"path": str(python), "sha256": file_hash(python)},
                    "bash": {"path": str(bash), "sha256": file_hash(bash)},
                }
            )
            product = Node(
                {
                    "source_inputs": source_inputs,
                    "executable": str(bridge),
                    "executable_sha256": file_hash(bridge),
                }
            )
            output = out / "candidate/mux2-stall"
            plan = make_plan(session, product, output)
            self.assertEqual(dict(plan.inputs), plan_inputs(session, product))
            self.assertEqual(plan.argv[-1], str(output / "capture/run"))
            self.assertNotIn("--headless", plan.argv)
            self.assertFalse(plan.capture.exists())
            self.assertTrue(plan.evidence.is_dir())
            self.assertEqual(
                document(plan.path.read_text()).field("protocol").text(), "NONE"
            )
            with self.assertRaises(GateContractError):
                _ = make_plan(session, product, output)
            base_plan = make_plan(session, product, out / "base/mux2-stall", "base")
            self.assertEqual(base_plan.capture, out / "base/mux2-stall/capture")
            self.assertEqual(
                dict(base_plan.inputs), plan_inputs(session, product, "base")
            )
            _ = bridge.write_text("changed")
            with self.assertRaises(GateContractError):
                _ = plan_inputs(session, product)

    def test_failed_launch_retains_original_error_and_incomplete_receipt(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp).resolve()
            root = output / "root"
            root.mkdir()
            child = ChildPlan(
                output / "plan.json",
                "a" * 64,
                root,
                output / "capture",
                output / "chain",
                ("fixture",),
                (),
            )
            with (
                patch.dict(os.environ, {"WORLDSIM_MUX2_LANE_FD": "12"}, clear=True),
                patch(
                    "generic_whole_gate_stall_capture.source_binding",
                    return_value=(Node({}), Node({})),
                ),
                patch("generic_whole_gate_stall_capture.make_plan", return_value=child),
                patch(
                    "generic_whole_gate_stall_capture.lane",
                    return_value=nullcontext(12),
                ),
                patch(
                    "generic_whole_gate_stall_capture.capture_wrapper",
                    side_effect=RuntimeError("synthetic launch failure"),
                ),
                patch(
                    "generic_whole_gate_stall_capture.close_capture",
                    side_effect=ValueError("synthetic cleanup failure"),
                ),
            ):
                with self.assertRaisesRegex(RuntimeError, "synthetic launch failure"):
                    _ = capture(output / "session.json", "b" * 64, "candidate", output)
            receipt = document((child.capture / "receipt.json").read_text())
            receipt.field("integrity").equals("INCOMPLETE")
            receipt.field("capture_complete").equals(False)
            receipt.field("errors").equals(["RuntimeError: synthetic launch failure"])
            receipt.field("cleanup_errors").equals(
                ["ValueError: synthetic cleanup failure"]
            )
            self.assertTrue((output / "capture-started.json").is_file())

    def test_base_reference_does_not_inherit_candidate_acceptance_bars(self) -> None:
        raw = json.dumps(report_with(61, 3))
        self.assertFalse(strict_result(raw, "base"))
        with self.assertRaises(EvidenceError):
            _ = strict_result(raw, "candidate")

    def test_missing_stage_never_inherits_conjunction_authority(self) -> None:
        others = tuple(OtherStep(n, "PASS") for n in range(1, 52) if n != 41)
        conjunction(others, True)
        for selected, safety in (
            (others[:-1], True),
            (others, False),
            (others + (others[0],), True),
        ):
            with self.assertRaises(GateContractError):
                conjunction(selected, safety)
        with self.assertRaises(GateContractError):
            _ = evaluate_stage(Node({}), Node({}), others, True)


if __name__ == "__main__":
    _ = unittest.main()
