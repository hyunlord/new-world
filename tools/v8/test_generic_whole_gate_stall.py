# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_stall
"""Global conjunction tests with explicitly mocked already-authenticated capture."""

import json
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

from gate_stall_child import ChildPlan
from generic_gate import GateContractError
from generic_stall import StallEvidence, assess_timing
from mux2_gate_inputs import EvidenceError
from test_generic_stall import report_with

from generic_whole_gate_perf import Anchors
from generic_whole_gate_stall import (
    OtherStep,
    StallEvidenceJoin,
    StallRequest,
    admit_stall,
)


def joined(strict: bool) -> StallEvidenceJoin:
    timing = assess_timing(json.dumps(report_with() if strict else report_with(60, 2)))
    evidence = StallEvidence("a" * 40, timing, 0 if strict else 1, strict, "b" * 64, ())
    child = ChildPlan(
        Path("/fixture/plan"),
        "c" * 64,
        Path("/fixture/root"),
        Path("/fixture/capture"),
        Path("/fixture/chain"),
        (),
        (),
    )
    return StallEvidenceJoin(
        evidence,
        42,
        0 if strict else 1,
        True,
        StallRequest("a" * 40, child, (), (), Anchors((), ())),
    )


class StallTests(unittest.TestCase):
    def test_waiver_preserves_native_failure_only_after_all_steps(self) -> None:
        others = tuple(OtherStep(n, "PASS") for n in range(1, 52) if n != 41)
        for strict in (False, True):
            evidence = joined(strict)
            with patch(
                "generic_whole_gate_stall.authenticate_stall", return_value=evidence
            ):
                result = admit_stall(evidence, others, True)
                self.assertEqual(result.status, "STRICT_PASS" if strict else "WAIVED")
                self.assertEqual(result.native_exit, 0 if strict else 1)
                self.assertFalse(result.publication_authority)
                for rows in (
                    others[:-1],
                    others + (others[0],),
                    tuple(
                        replace(r, status="FAIL") if r.number == 1 else r
                        for r in others
                    ),
                ):
                    with self.assertRaises(GateContractError):
                        _ = admit_stall(evidence, rows, True)
                with self.assertRaises(GateContractError):
                    _ = admit_stall(evidence, others, False)

    def test_changed_capture_or_unsafe_timing_never_waived(self) -> None:
        evidence = joined(False)
        others = tuple(OtherStep(n, "PASS") for n in range(1, 52) if n != 41)
        with patch(
            "generic_whole_gate_stall.authenticate_stall",
            return_value=replace(evidence, capture_complete=False),
        ):
            with self.assertRaises(GateContractError):
                _ = admit_stall(evidence, others, True)
        for raw in (report_with(60.001, 1), report_with(55, 3)):
            with self.assertRaises(EvidenceError):
                _ = assess_timing(json.dumps(raw))


if __name__ == "__main__":
    _ = unittest.main()
