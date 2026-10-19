#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest discover -s tools/art_pipeline -p test_agent_sections_session_p10.py
"""Offline rejection tests for diagnostic section measurement evidence."""
import copy
import unittest

from generate_agent_sections_p10 import NAMES
from run_agent_sections_p10 import validate_capture
from summarize_perf_p10 import EvidenceError


class SectionSessionTests(unittest.TestCase):
    def setUp(self) -> None:
        initial = {'tick': 1440, 'population': 500, 'zoom': [4, 4], 'camera': [456, 36], 'simulation_frozen': False}
        conditions = {'inspector_visible': False, 'influence_channel': -1, 'layers_enabled': True, 'renderer_callback_enabled': True, 'world_callback_enabled': True, 'speed_setting': 0.0}
        self.payload = {
            'valid': True, 'errors': [], 'world': 'W500', 'seed': 15100,
            'requested_tick': 1440, 'requested_population': 500, 'zoom': 4,
            'window_size': [1920, 1080], 'warmup_frames': 30, 'counted_frames': 300,
            'diagnostic_only': True, 'instrumented': True,
            'founding_groups_status': 'unavailable_separate_callback',
            'initial': initial, 'final': initial, 'framing_tile': [0, 0], 'conditions': conditions, 'speed_setting': 0.0,
            'installation': {'valid': True, 'source_sha256': 'a' * 64, 'output_sha256': 'b' * 64, 'clock_sha256': 'c' * 64},
            'restoration': dict.fromkeys(('vsync_restored', 'layers_restored', 'camera_process_restored', 'probes_removed'), True),
            'max_fps_restored': True,
        }
        rows = []
        for frame in range(330):
            rows.append({'process_frame': frame + 1, 'paired': True, 'state': initial, 'conditions': conditions,
                         'process_ms': 1.0, 'render_cpu_ms': 1.0, 'setup_cpu_ms': 0.0, 'primary_cpu_ms': 2.0,
                         'agent_sections': {'valid': True, 'complete': True,
                                            'process_frame': frame + 1,
                                            'exclusive_usec': {name + '_usec': 1 for name in NAMES},
                                            'total_usec': len(NAMES), 'remainder_usec': 0,
                                            'founding_groups_usec': None,
                                            'founding_groups_status': 'unavailable_separate_callback'}})
        self.payload.update(discarded=rows[:30], samples=rows[30:])

    def test_complete_capture_passes(self) -> None:
        validate_capture(self.payload)

    def test_partial_and_stale_callbacks_fail(self) -> None:
        for field, value in (('complete', False), ('valid', False), ('process_frame', -1)):
            with self.subTest(field=field):
                payload = copy.deepcopy(self.payload)
                payload['samples'][0]['agent_sections'][field] = value
                with self.assertRaises(EvidenceError):
                    validate_capture(payload)

    def test_negative_missing_or_inconsistent_spans_fail(self) -> None:
        for mutation in ('negative', 'missing', 'total'):
            with self.subTest(mutation=mutation):
                payload = copy.deepcopy(self.payload)
                sections = payload['samples'][0]['agent_sections']
                if mutation == 'negative':
                    sections['exclusive_usec']['reads_usec'] = -1
                elif mutation == 'missing':
                    del sections['exclusive_usec']['reads_usec']
                else:
                    sections['total_usec'] = 9999
                with self.assertRaises(EvidenceError):
                    validate_capture(payload)

    def test_short_capture_and_changed_world_fail(self) -> None:
        payload = copy.deepcopy(self.payload)
        payload['samples'].pop()
        with self.assertRaises(EvidenceError):
            validate_capture(payload)
        payload = copy.deepcopy(self.payload)
        payload['final'] = {'tick': 1441, 'population': 500}
        with self.assertRaises(EvidenceError):
            validate_capture(payload)

    def test_changed_ui_population_camera_and_installation_fail(self) -> None:
        for field in ('conditions', 'population', 'camera', 'installation'):
            with self.subTest(field=field):
                payload = copy.deepcopy(self.payload)
                if field == 'conditions':
                    payload['samples'][0]['conditions'] = {}
                elif field == 'installation':
                    payload['installation']['valid'] = False
                else:
                    payload['initial'][field] = -1
                with self.assertRaises(EvidenceError):
                    validate_capture(payload)

    def test_negative_or_nonfinite_frame_metrics_fail(self) -> None:
        for value in (-1.0, float('nan'), float('inf')):
            with self.subTest(value=value):
                payload = copy.deepcopy(self.payload)
                payload['samples'][0]['process_ms'] = value
                with self.assertRaises(EvidenceError):
                    validate_capture(payload)

    def test_claiming_child_callback_as_parent_fails(self) -> None:
        self.payload['samples'][0]['agent_sections']['founding_groups_usec'] = 1
        with self.assertRaises(EvidenceError):
            validate_capture(self.payload)


if __name__ == '__main__':
    unittest.main()
