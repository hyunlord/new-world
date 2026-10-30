#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_m6a_r1_stall.py
"""Synthetic R1 waiver counterexamples; these are never GPU acceptance evidence."""
import math
import unittest

from m6a_r1_stall import authorize, evaluate
from mux2_gate_inputs import EvidenceError
from mux2_gate_stall import validate_stall
from mux2_gate_testdata import stall_fixture


def changed_frames(changes):
    """Recompute producer fields after independently choosing raw frame intervals."""
    report = stall_fixture()
    for arm_index, edits in changes.items():
        arm = report['arms'][arm_index]
        elapsed = 0.0
        counts = [0] * 5
        slow = []
        over50 = 0
        for index, row in enumerate(arm['samples']):
            row['interval_ms'] = edits.get(index, row['interval_ms'])
            elapsed = round(elapsed + row['interval_ms'] / 1000, 6)
            row['elapsed_seconds'] = elapsed
            row['post_draw_usec'] = 1000000 + round(elapsed * 1000000)
            row['telemetry']['frame_phases']['pre_to_post_draw_usec'] = round(row['interval_ms'] * 1000) - 4000
            if row['interval_ms'] > 33:
                slow.append(row)
                if elapsed > 10:
                    counts[min(4, math.ceil((elapsed - 10) / 60) - 1)] += 1
            if elapsed > 10 and row['interval_ms'] > 50:
                over50 += 1
        arm['score'].update(actual_play_seconds=elapsed, actual_post_warmup_seconds=elapsed - 10,
                            post_warmup_over33=sum(counts), over33_per_minute=counts,
                            all_over33_frames=slow, post_warmup_over50=over50,
                            **{'pass': over50 == 0 and all(n <= 3 for n in counts)})
    passed = all(arm['score']['pass'] for arm in report['arms'])
    report.update(**{'pass': passed, 'bar_pass': passed})
    return report


class WaiverTests(unittest.TestCase):
    def test_strict_pass_uses_no_waiver(self):
        verdict = evaluate(stall_fixture())
        self.assertTrue(verdict.strict_pass)
        self.assertTrue(verdict.waiver_eligible)
        self.assertEqual(authorize(verdict, True)['waivers'], [])

    def test_two_frames_per_zoom_and_sixty_are_eligible(self):
        raw = changed_frames({0: {2000: 51, 2209: 60}, 1: {3000: 50.001, 3100: 59}})
        verdict = evaluate(raw)
        self.assertFalse(verdict.strict_pass)
        self.assertTrue(verdict.waiver_eligible)
        self.assertEqual([len(arm.over50) for arm in verdict.arms], [2, 2])
        self.assertEqual(verdict.arms[0].over50[1].ticks_advanced, 1)
        self.assertTrue(authorize(verdict, True)['waivers'])
        self.assertFalse(raw['pass'])
        with self.assertRaises(EvidenceError):
            validate_stall(raw)

    def test_exact_fifty_and_warmup_do_not_use_waiver(self):
        verdict = evaluate(changed_frames({0: {5: 70, 2000: 50}}))
        self.assertTrue(verdict.strict_pass)
        self.assertEqual(verdict.arms[0].over50, ())

    def test_outside_waiver_stays_ineligible(self):
        for edits in ({2000: 60.001}, {2000: 51, 2100: 51, 2200: 51},
                      {2000: 34, 2010: 34, 2020: 34, 2030: 34}):
            with self.subTest(edits=edits):
                verdict = evaluate(changed_frames({0: edits}))
                self.assertFalse(verdict.waiver_eligible)
                with self.assertRaises(EvidenceError):
                    authorize(verdict, True)

    def test_any_other_gate_failure_blocks_authorization(self):
        for raw in (stall_fixture(), changed_frames({0: {2000: 51}})):
            with self.assertRaisesRegex(EvidenceError, 'other gate'):
                authorize(evaluate(raw), False)

    def test_invalid_integrity_never_becomes_waiver(self):
        for mutate in (
            lambda raw: raw.update(complete=False),
            lambda raw: raw.update(errors=['capture error']),
            lambda raw: raw.update(**{'pass': True}),
            lambda raw: raw['arms'][0]['score'].update(post_warmup_over50=0),
            lambda raw: raw['arms'][0]['samples'][2000].update(ticks_advanced=7),
            lambda raw: raw['arms'][0]['instrumentation'].update(native='NOT_MEASURED'),
            lambda raw: raw['environment'].update(render_driver='headless'),
        ):
            raw = changed_frames({0: {2000: 51}})
            mutate(raw)
            with self.assertRaises(EvidenceError):
                evaluate(raw)


if __name__ == '__main__':
    unittest.main()
