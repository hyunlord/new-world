# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/art_pipeline -p 'test_*p10.py'
"""Offline safety checks; these never launch Godot or capture a fabricated world."""
from pathlib import Path
from copy import deepcopy
from io import StringIO
import signal
import subprocess
from unittest.mock import Mock, patch

import unittest

from run_activity_captures_p10 import (Capture, InvalidInput, cases, launch_command, main,
                                       validate_capture, wait_for_quiet, bounded_launch)


class ActivityCaptureTests(unittest.TestCase):
    def test_six_cases_require_explicit_probe_cohort_seed(self) -> None:
        matrix = cases(20261027)
        assert len(matrix) == 6
        assert len({case.name for case in matrix}) == 6
        assert {(c.day, c.hour, c.zoom) for c in matrix} == {
            (30, 10, 4), (30, 10, 1), (30, 15, 4), (30, 15, 1), (60, 10, 4), (60, 10, 1)}
        assert all(c.seed == 20261010 for c in matrix if c.day == 30)
        assert all(c.seed == 20261027 for c in matrix if c.day == 60)
        for invalid in [0, 20261010, 20261028, 20261042]:
            with self.assertRaises(InvalidInput):
                cases(invalid)


    def test_window_wrapper_rechecks_quiet_absence_without_acquiring_it(self) -> None:
        command = launch_command(Path('/checkout with spaces'), Path('/Godot app/Godot'), 1800)
        assert command[:3] == ['bash', 'tools/art_pipeline/with_window_lock.sh', '1800']
        assert 'while [[ -d "$1" ]]' in command[5]
        assert 'mkdir' not in command[5]
        assert command[command.index('--fixed-fps') + 1] == '60'
        assert '--headless' not in command
        assert '/checkout with spaces' in command


    def test_missing_cli_arguments_do_not_start_any_launch(self) -> None:
        assert main([]) == 2
        assert main(['checkout', 'output', '20261011']) == 2


    def test_capture_refuses_legacy_metadata_and_never_uses_stress_world(self) -> None:
        source = (Path(__file__).resolve().parents[2] / 'scripts/test/art/capture_activities_p10.gd').read_text()
        assert 'reset_world_stress' not in source
        assert 'p10_activity_schema' in source
        assert 'incomplete final activity metadata' in source
        assert 'activity holder lacks live agent/sprite' in source
        assert '"pose_oracle_checked": false' in source
        assert '"fully_inside_viewport"' in source
        assert 'ReadRoute.read(_world, "get_activity_snapshot")' in source


    def fixture(self):
        meta = {"p10_activity_schema": 1, "activity_kind": 5, "activity_place_known": True,
                "entity_bits": 10, "art_id": 7, "activity": "play", "pose": "hop",
                "body": "child", "facing": "S", "sprite_id": "test-sprite",
                "cold_pose": "hop", "cold_base_pose": "hop", "activity_frame_view": "S",
                "activity_frame_activity": "hop", "activity_overrides": []}
        return {"schema_version": 1, "seed": 20261010, "display_day": 30, "hour": 10,
                "zoom": 4, "state_change_frame": 10, "capture_frame": 130, "fixed_fps": 60,
                "viewport": [1920, 1080], "errors": [], "complete": True,
                "pose_oracle_checked": False, "fixed_delta_valid": True, "delta_tolerance": 1e-9,
                "delta_samples": 130, "minimum_delta": 1/60, "maximum_delta": 1/60,
                "world_info": {"ticks_per_day": 1440, "clock_offset_ticks": 480, "seed": 20261010},
                "clock": {"tick": 41880, "day_index": 29, "time_of_day": 10.0}, "tick": 41880,
                "population": 1, "holder_count": 1, "camera": [456, 36],
                "raw_agent_snapshot": {"ids": [10], "agent_ids": [7], "states": [4]},
                "raw_activity_snapshot": {"ids": [10], "kinds": [5], "place_x": [1], "place_y": [2], "until_tick": [42000]},
                "activity_holders": [{"entity_bits": 10, "agent_id": 7, "kind": 5,
                    "until_tick": 42000, "place": [1, 2], "renderer_present": True,
                    "missing_metadata": [], "metadata": meta, "state_tag": 4,
                    "viewport_rect": [10, 20, 32, 64], "visible": True, "visible_in_tree": True,
                    "intersects_viewport": True, "fully_inside_viewport": True, "flip_h": False,
                    "region_enabled": True, "region_rect": [1, 2, 8, 16], "texture_path": "res://test.png"}]}

    def test_sidecar_contract_accepts_valid_fixture_not_h7(self) -> None:
        payload = self.fixture()
        validate_capture(payload, Capture(20261010, 30, 10, 4))
        self.assertIs(payload["pose_oracle_checked"], False)

    def test_process_frame_zero_is_included_in_delta_samples(self) -> None:
        payload = self.fixture()
        payload["delta_samples"] = 131
        validate_capture(payload, Capture(20261010, 30, 10, 4))
        payload["delta_samples"] = 132
        with self.assertRaises(InvalidInput):
            validate_capture(payload, Capture(20261010, 30, 10, 4))

    def test_calendar_for_all_six_cases_and_honest_offscreen_holder(self) -> None:
        for case in cases(20261011):
            with self.subTest(case=case):
                payload = self.fixture()
                tick = (case.day - 1) * 1440 + case.hour * 60 - 480
                payload.update(seed=case.seed, display_day=case.day, hour=case.hour, zoom=case.zoom, tick=tick)
                payload["world_info"]["seed"] = case.seed
                payload["clock"].update(tick=tick, day_index=case.day - 1, time_of_day=float(case.hour))
                row = payload["activity_holders"][0]
                row.update(viewport_rect=[-100, -100, 32, 64], visible=False,
                           visible_in_tree=False, intersects_viewport=False, fully_inside_viewport=False)
                validate_capture(payload, case)
                self.assertIn(tick, [41880, 42180, 85080])

    def test_rejects_wrong_requested_state_or_fixed_frame_evidence(self) -> None:
        for key, value in [("seed", 20261011), ("display_day", 60), ("hour", 15),
                ("zoom", 1), ("tick", 41881), ("capture_frame", 129), ("fixed_fps", 30),
                ("minimum_delta", 0.017), ("maximum_delta", float('nan')),
                ("delta_samples", True), ("fixed_delta_valid", False),
                ("pose_oracle_checked", True), ("holder_count", 2)]:
            with self.subTest(key=key), self.assertRaises(InvalidInput):
                payload = self.fixture()
                payload[key] = value
                validate_capture(payload, Capture(20261010, 30, 10, 4))
        for malformed in [None, [], "not an object"]:
            with self.assertRaises(InvalidInput):
                validate_capture(malformed, Capture(20261010, 30, 10, 4))

    def test_rejects_raw_mismatch_missing_metadata_and_duplicate_ids(self) -> None:
        for mutation in ["raw_length", "raw_kind", "raw_bool", "metadata", "arrival", "duplicate_entity", "duplicate_agent", "nonfinite_rect"]:
            with self.subTest(mutation=mutation), self.assertRaises(InvalidInput):
                payload = self.fixture()
                row = payload["activity_holders"][0]
                if mutation == "raw_length":
                    payload["raw_activity_snapshot"]["ids"] = []
                elif mutation == "raw_kind":
                    payload["raw_activity_snapshot"]["kinds"] = [4]
                elif mutation == "raw_bool":
                    payload["raw_activity_snapshot"]["place_x"] = [True]
                elif mutation == "metadata":
                    del row["metadata"]["activity_frame_view"]
                elif mutation == "arrival":
                    row["metadata"]["activity_place_known"] = False
                elif mutation == "nonfinite_rect":
                    row["viewport_rect"][0] = float('inf')
                else:
                    payload["holder_count"] = 2
                    payload["population"] = 2
                    for values in payload["raw_agent_snapshot"].values():
                        values.append(values[0])
                    payload["activity_holders"].append(deepcopy(row))
                    payload["raw_agent_snapshot"]["ids"][1] = 11
                    payload["raw_agent_snapshot"]["agent_ids"][1] = 8
                    for values in payload["raw_activity_snapshot"].values():
                        values.append(values[0])
                    if mutation == "duplicate_agent":
                        payload["activity_holders"][1]["entity_bits"] = 11
                        payload["activity_holders"][1]["metadata"]["entity_bits"] = 11
                        payload["raw_activity_snapshot"]["ids"][1] = 11
                validate_capture(payload, Capture(20261010, 30, 10, 4))

    def test_quiet_wait_is_cut_off_at_deadline_without_launch(self) -> None:
        with patch('run_activity_captures_p10.QUIET') as quiet, \
                patch('run_activity_captures_p10.time.time', side_effect=[10.0, 15.0]), \
                patch('run_activity_captures_p10.time.sleep') as sleep:
            quiet.is_dir.return_value = True
            with self.assertRaises(InvalidInput):
                wait_for_quiet(15.0, Path('/dev/null'), 'case')
            sleep.assert_called_once_with(5.0)

    def test_launch_deadline_kills_only_own_process_group(self) -> None:
        process = Mock(pid=12345)
        process.wait.side_effect = [subprocess.TimeoutExpired('capture', 5), 0]
        with patch('run_activity_captures_p10.time.time', return_value=10.0), \
                patch('run_activity_captures_p10.subprocess.Popen', return_value=process) as popen, \
                patch('run_activity_captures_p10.os.killpg') as kill:
            self.assertEqual(bounded_launch(['never-executed'], Path('.'), {}, StringIO(), 15, 1800), 124)
            self.assertEqual(process.wait.call_args_list[0].kwargs['timeout'], 5)
            self.assertTrue(popen.call_args.kwargs['start_new_session'])
            kill.assert_called_once_with(12345, signal.SIGTERM)
        with patch('run_activity_captures_p10.time.time', return_value=15.0), \
                patch('run_activity_captures_p10.subprocess.Popen') as popen:
            with self.assertRaises(InvalidInput):
                bounded_launch(['never-executed'], Path('.'), {}, StringIO(), 15, 1800)
            popen.assert_not_called()


if __name__ == "__main__":
    unittest.main()
