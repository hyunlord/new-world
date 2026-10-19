"""Critical P13 evidence boundaries, independent of a Godot launch."""
import json
import tempfile
import unittest
from pathlib import Path

from cpu_probe_p13 import KERNEL_ID
from pixel_identity_p13 import Difference, accepts
from session_validity_p13 import require_valid, valid_snapshot
from summarize_perf_p13 import METRICS, Session, load_session, report
from summarize_ablation_p13 import markdown as ablation_markdown


def snapshot(load=1.0, cpu=0.0, pid=7, wall=2.0):
    return {'wall_seconds': wall, 'reference_seconds': 2.0, 'load_1m': load, 'ps': f'  PID  %CPU COMM\n{pid} {cpu} foreign\n', 'owned_pids': [8]}


class ToolingEvidenceTests(unittest.TestCase):
    def test_snapshot_thresholds(self):
        # Given independent snapshots; when checked; then exact bars apply.
        cases = [(7.99, 50, True), (8, 0, False), (1, 100, True),
                 (float('nan'), 0, False), (1, float('nan'), True)]
        for load, cpu, expected in cases:
            with self.subTest(load=load, cpu=cpu):
                self.assertIs(valid_snapshot(snapshot(load, cpu)), expected)

    def test_valid_start_cannot_hide_invalid_end(self):
        # Given valid starts and invalid ends; when revalidated; reject evidence.
        for end in (snapshot(8), snapshot(wall=2.10001), {}):
            with self.subTest(end=end), tempfile.TemporaryDirectory() as directory:
                path = Path(directory) / 'validity.json'
                path.write_text(json.dumps({'start': snapshot(), 'end': end,
                                            'exit_code': 0, 'valid': True, 'kernel_id': KERNEL_ID, 'reference_seconds': 2.0}))
                with self.assertRaises(ValueError):
                    require_valid(path)

    def test_missing_multi_identity_fails_closed(self):
        from compare_running_identity_p13 import compare
        # Given absent dumps; when checked; all required worlds must be reported.
        with tempfile.TemporaryDirectory() as directory:
            result = compare(Path(directory), Path(directory))
        self.assertFalse(result['valid'])
        self.assertEqual({row['world'] for row in result['worlds']},
                         {'W0', 'W200', 'W-multi', 'W500_FROZEN', 'W500_2X'})

    def test_missing_sidecar_prevents_perf_acceptance(self):
        # Given session metadata without snapshots; when loaded; fail closed.
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'session.json'
            path.write_text('{}')
            with self.assertRaises(FileNotFoundError):
                load_session(path)

    def test_nonzero_control_or_candidate_never_passes(self):
        # Given changed RGBA images; when comparing; grant no noise allowance.
        cases = [(Difference(1, 1), Difference(0, 0)),
                 (Difference(3, 2), Difference(2, 1)),
                 (Difference(0, 0), Difference(1, 1))]
        for control, candidate in cases:
            with self.subTest(control=control, candidate=candidate):
                self.assertFalse(accepts(control, candidate))

    def test_exact_images_pass(self):
        self.assertTrue(accepts(Difference(0, 0), Difference(0, 0)))

    def test_flagged_running_rows_remain_visible_but_leave_pools(self):
        # Given one fast valid block and one slow population-loss block.
        blocks = []
        for index, cost, flagged in [(0, 2.0, False), (1, 100.0, True)]:
            sample = {metric: cost for metric in METRICS}
            sample["ticks_run"] = 1
            blocks.append({"label": f"running_2x_{index}", "condition": "running",
                           "zoom": 4, "speed": 2, "samples": [sample],
                           "population_start": 500, "population_end": 300 if flagged else 500,
                           "population_flagged": flagged})
        session = Session(Path("fixture.json"), "W500", "baseline",
                          {"blocks": blocks, "ablation": [], "errors": []})
        # When rendered, both raw rows remain and only eligible samples are pooled.
        lines = report([session], set())
        self.assertTrue(any("running_2x_1 | primary_cpu_ms | 1 | 100.0000" in row for row in lines))
        self.assertTrue(any("POOLED running_Z4_2x | primary_cpu_ms | 1 | 2.0000" in row for row in lines))
        self.assertTrue(any("POOLED running_Z4_2x tick-only | primary_cpu_ms | 1 | 2.0000" in row for row in lines))
        self.assertTrue(any("EXCLUDED from pooled" in row and "running_2x_1" in row for row in lines))

    def test_all_flagged_running_pool_has_no_result(self):
        # Given only excluded samples, when rendered, no pooled number is claimed.
        block = {"label": "running_2x_0", "condition": "running", "zoom": 4,
                 "speed": 2, "samples": [{metric: 1 for metric in METRICS}],
                 "population_start": 500, "population_end": 300, "population_flagged": True}
        lines = report([Session(Path("fixture.json"), "W500", "baseline",
                               {"blocks": [block], "ablation": [], "errors": []})], set())
        self.assertTrue(any("POOLED running_Z4_2x | primary_cpu_ms | 0 | N/A" in row for row in lines))

    def test_ablation_heading_matches_measured_world_and_condition(self):
        # Given each supported ablation world, when rendered, label its condition.
        for world, condition in [("W200", "running 2×"), ("W500", "frozen")]:
            with self.subTest(world=world):
                text = ablation_markdown({"world": world, "source": "fixture.json",
                                          "actual_alive": 200 if world == "W200" else 500,
                                          "ranked": [], "top_three": [], "floor": {}})
                self.assertIn(f"# {world} {condition} ablation ranking", text)
                self.assertIn(f"release, {condition}", text)
                if world == "W200":
                    self.assertNotIn("frozen tick", text)


if __name__ == '__main__':
    unittest.main()
