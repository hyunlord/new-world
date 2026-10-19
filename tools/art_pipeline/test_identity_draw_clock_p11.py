# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest discover -s tools/art_pipeline -p test_identity_draw_clock_p11.py
from __future__ import annotations

import copy
import json
import subprocess
import tempfile
import unittest
from pathlib import Path

VERIFIER = Path(__file__).with_name('verify_draw_clock_p11.py')


def fixture() -> dict[str, object]:
    frames = [{'frame': i, 'delta': 1 / 60, 'time_scale': 1.0} for i in range(131)]
    events = [{'kind': 'manual', 'process_frame': i, 'step_seconds': 1 / 60, 'time_scale': 1.0} for i in range(131)]
    events.insert(1, {'kind': 'automatic', 'process_frame': 0, 'step_seconds': 1 / 60, 'time_scale': 1.0})
    return {'capture_frame': 130, 'draw_clock': {'schema_version': 2, 'valid': True, 'errors': [], 'automatic_loop_disabled': True, 'initial_engine_frames_drawn': 0, 'origin_frame': 0, 'last_frame': 130, 'manual_steps': 131, 'automatic_steps': 1, 'step_seconds': 1 / 60, 'pre_draws': 132, 'post_draws': 132, 'engine_frames_drawn': 1, 'process_samples': frames, 'draw_events': events}}


class ClockTests(unittest.TestCase):
    def invoke(self, stamps: list[dict[str, object]]) -> int:
        with tempfile.TemporaryDirectory() as temporary:
            directories = [Path(temporary) / label for label in ('A', 'Aprime', 'B')]
            for directory, stamp in zip(directories, stamps, strict=True):
                directory.mkdir()
                for framing in ('P1', 'P2', 'P3', 'P4', 'P5', 'P6'):
                    (directory / f'{framing}.json').write_text(json.dumps(stamp))
            return subprocess.run(['python3', str(VERIFIER), *map(str, directories)], capture_output=True, check=False).returncode

    def test_complete_132_draws_pass(self) -> None:
        self.assertEqual(self.invoke([fixture(), fixture(), fixture()]), 0)

    def test_godot_serialization_passes(self) -> None:
        stamp = fixture()
        clock = stamp['draw_clock']
        assert isinstance(clock, dict)
        clock['step_seconds'] = 0.0166666666666667
        for sample in clock['process_samples']:
            sample['delta'] = 0.0166666666666667
        for event in clock['draw_events']:
            event['step_seconds'] = 0.0166666666666667
        self.assertEqual(self.invoke([stamp, stamp, stamp]), 0)

    def test_tolerance_is_not_cross_capture_tolerance(self) -> None:
        stamp = fixture()
        clock = stamp['draw_clock']
        assert isinstance(clock, dict)
        clock['step_seconds'] = 0.0166666666666667
        self.assertNotEqual(self.invoke([fixture(), fixture(), stamp]), 0)

    def test_step_tolerance_boundary(self) -> None:
        for offset in (0.5e-9, 2e-9):
            stamp = fixture()
            clock = stamp['draw_clock']
            assert isinstance(clock, dict)
            clock['step_seconds'] = 1 / 60 + offset
            self.assertEqual(self.invoke([stamp, stamp, stamp]) == 0, offset < 1e-9)

    def test_tampered_field_fails(self) -> None:
        for field, value in [('initial_engine_frames_drawn', 1), ('post_draws', 131), ('engine_frames_drawn', 0), ('automatic_steps', 0), ('automatic_loop_disabled', False), ('manual_steps', True)]:
            with self.subTest(field=field):
                stamp = fixture()
                clock = stamp['draw_clock']
                assert isinstance(clock, dict)
                clock[field] = value
                self.assertNotEqual(self.invoke([stamp, stamp, stamp]), 0)

    def test_different_automatic_frame_fails(self) -> None:
        stamp = fixture()
        clock = stamp['draw_clock']
        assert isinstance(clock, dict)
        events = clock['draw_events']
        assert isinstance(events, list)
        event = events.pop(1)
        event['process_frame'] = 1
        events.insert(2, event)
        self.assertNotEqual(self.invoke([fixture(), fixture(), stamp]), 0)

    def test_missing_and_bad_step_fail(self) -> None:
        for mode in ('missing', 'delta', 'event'):
            stamp = copy.deepcopy(fixture())
            clock = stamp['draw_clock']
            assert isinstance(clock, dict)
            if mode == 'missing':
                del clock['draw_events']
            elif mode == 'delta':
                clock['process_samples'][0]['delta'] = 0.02
            else:
                clock['draw_events'][1]['step_seconds'] = 0.02
            self.assertNotEqual(self.invoke([stamp, stamp, stamp]), 0)


if __name__ == '__main__':
    unittest.main()
