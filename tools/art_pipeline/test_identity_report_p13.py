"""A zero-pixel report cannot override a rejected candidate-state verdict."""
from __future__ import annotations

import unittest

from compare_perf_p13 import Record, identity_verdict, running_verdict
from summarize_perf_p13 import EvidenceError


def report() -> Record:
    return {
        "scope": "candidate_validation", "candidate": "declump-kernel",
        "passed": True, "errors": [],
        "framings": [{"framing": f"P{i}", "control": {"pixels": 0, "max_channel": 0},
                      "candidate": {"pixels": 0, "max_channel": 0}, "passed": True,
                      "capture_frame": 240, "state_change_frame": 120,
                      "rgba_sha256": ["a" * 64] * 3} for i in range(1, 7)],
    }


class IdentityReportTests(unittest.TestCase):
    def test_explicit_candidate_pass(self) -> None:
        for candidate in ("declump-kernel", "write-skip"):
            payload = report()
            payload["candidate"] = candidate
            payload["kernel_state"] = "on"
            self.assertEqual(identity_verdict(payload)["status"], "PASS")

    def test_reference_is_not_candidate_acceptance(self) -> None:
        for scope in (None, "reference_integrity_only", "unknown"):
            payload = report()
            payload["scope"] = scope
            self.assertEqual(identity_verdict(payload)["status"], "NOT_VERIFIED")

    def test_failed_stamp_verdict_cannot_be_overridden_by_pixels(self) -> None:
        for changes in ({"passed": False}, {"errors": ["B requested_kernel must be True"]},
                        {"passed": False, "errors": ["frame 300 kernel changed"]}):
            payload = report()
            payload.update(changes)
            self.assertEqual(identity_verdict(payload)["status"], "FAIL")

    def test_missing_or_malformed_verdict_is_unverified(self) -> None:
        for key, value in (("passed", None), ("passed", 1), ("errors", None),
                           ("errors", ""), ("candidate", None), ("candidate", "unknown")):
            payload = report()
            payload[key] = value
            self.assertEqual(identity_verdict(payload)["status"], "NOT_VERIFIED")
        for key in ("passed", "errors", "candidate"):
            payload = report()
            del payload[key]
            self.assertEqual(identity_verdict(payload)["status"], "NOT_VERIFIED")

    def test_running_scope_and_rejection_are_preserved(self) -> None:
        self.assertEqual(running_verdict({"scope": "reference_integrity_only"})["status"], "NOT_VERIFIED")
        self.assertEqual(running_verdict({"scope": "candidate_validation", "candidate": "write-skip",
                                        "valid": False, "status": "FAIL", "kernel_state": "on"})["status"], "FAIL")
        self.assertEqual(running_verdict({"scope": "candidate_validation", "candidate": "write-skip",
                                        "valid": True, "status": "PASS"})["status"], "NOT_VERIFIED")

    def test_exact_pixels_schedule_and_six_framings_remain_required(self) -> None:
        for field in ("pixels", "max_channel"):
            payload = report()
            payload["framings"][0]["candidate"][field] = 1
            self.assertEqual(identity_verdict(payload)["status"], "FAIL")
        payload = report()
        payload["framings"][0]["capture_frame"] = 239
        self.assertEqual(identity_verdict(payload)["status"], "NOT_VERIFIED")
        payload = report()
        payload["framings"].pop()
        self.assertEqual(identity_verdict(payload)["status"], "NOT_VERIFIED")
        payload = report()
        payload["framings"][0]["control"]["pixels"] = -1
        with self.assertRaises(EvidenceError):
            identity_verdict(payload)


if __name__ == "__main__":
    unittest.main()
