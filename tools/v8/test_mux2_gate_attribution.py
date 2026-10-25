#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p 'test_mux2_gate_*.py'
"""Current producer telemetry deletion and mutation counterexamples."""
from pathlib import Path
import unittest
from test_mux2_gate_receipts import stall_fixture
from mux2_gate_receipts import validate_stall


class RawAttributionTests(unittest.TestCase):
    def test_missing_raw_frame_fields_fail_even_with_green_summary(self):
        for key in ('telemetry', 'post_draw_usec', 'tick', 'ticks_advanced', 'sampling_overhead_usec'):
            with self.subTest(key=key):
                report = stall_fixture()
                report['arms'][0]['samples'][0].pop(key, None)
                with self.assertRaises(ValueError):
                    validate_stall(report)

    def test_missing_warmup_fails_even_with_green_summary(self):
        report = stall_fixture()
        report['arms'][0].pop('wall_warmup', None)
        with self.assertRaises(ValueError):
            validate_stall(report)

    def test_each_attribution_stream_rejects_mutation(self):
        report = stall_fixture()
        telemetry = report['arms'][0]['samples'][0]['telemetry']
        mutations = [(('native',), 'status', 'DISABLED'),
                     (('freeze',), 'status', 'NOT_MEASURED'),
                     (('ground',), 'frame', 999),
                     (('buildings', 'usec'), 'process', float('nan')),
                     (('alerts', 'usec'), 'poll', -1),
                     (('native', 'systems_usec'), 'FixtureSystem', -1),
                     (('native',), 'frame_ticks', 9),
                     (('frame_phases',), 'process_to_pre_draw_usec', 999999)]
        for path, key, value in mutations:
            with self.subTest(path=path, key=key):
                node = telemetry
                for part in path:
                    node = node[part]
                original = node[key]
                node[key] = value
                try:
                    with self.assertRaises(ValueError):
                        validate_stall(report)
                finally:
                    node[key] = original

    def test_warmup_must_be_in_recorded_samples(self):
        report = stall_fixture()
        report['arms'][0]['samples'][0]['telemetry']['ground']['usec']['wall_warmup'] = 0
        with self.assertRaisesRegex(ValueError, 'warmup'):
            validate_stall(report)

    def test_post_draw_timestamps_must_match_interval(self):
        report = stall_fixture()
        report['arms'][0]['samples'][1]['post_draw_usec'] += 2000
        with self.assertRaisesRegex(ValueError, 'post-draw'):
            validate_stall(report)

    def test_tick_jump_and_end_tick_mismatch_fail(self):
        report = stall_fixture()
        report['arms'][0]['samples'][1]['tick'] = 9
        with self.assertRaisesRegex(ValueError, 'tick delta'):
            validate_stall(report)
        report['arms'][0]['samples'][1]['tick'] = 0
        report['arms'][0]['end_tick'] += 1
        with self.assertRaisesRegex(ValueError, 'end tick'):
            validate_stall(report)

    def test_full_gate_enters_lock_before_any_command(self):
        gate = (Path(__file__).parent / 'gate.sh').read_text()
        self.assertIn('mux2_gate_lock.py', gate)
        self.assertLess(gate.index('mux2_gate_lock.py'), gate.index('step 1 fmt'))
