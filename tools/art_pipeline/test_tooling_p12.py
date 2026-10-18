"""Critical P12 evidence boundaries, independent of a Godot launch."""
import json
import tempfile
import unittest
from pathlib import Path

from pixel_identity_p12 import Difference, accepts
from session_validity_p12 import require_valid, valid_snapshot
from summarize_perf_p12 import load_session


def snapshot(load=1.0, cpu=0.0, pid=7):
    return {'load_1m': load, 'ps': f'  PID  %CPU COMM\n{pid} {cpu} foreign\n', 'owned_pids': [8]}


class ToolingEvidenceTests(unittest.TestCase):
    def test_snapshot_thresholds(self):
        # Given independent snapshots; when checked; then exact bars apply.
        cases = [(5.99, 50, True), (6, 0, False), (1, 50.01, False),
                 (float('nan'), 0, False), (1, float('nan'), False)]
        for load, cpu, expected in cases:
            with self.subTest(load=load, cpu=cpu):
                self.assertIs(valid_snapshot(snapshot(load, cpu)), expected)

    def test_valid_start_cannot_hide_invalid_end(self):
        # Given valid starts and invalid ends; when revalidated; reject evidence.
        for end in (snapshot(6), snapshot(cpu=51), {}):
            with self.subTest(end=end), tempfile.TemporaryDirectory() as directory:
                path = Path(directory) / 'validity.json'
                path.write_text(json.dumps({'start': snapshot(), 'end': end,
                                            'exit_code': 0, 'valid': True}))
                with self.assertRaises(ValueError):
                    require_valid(path)

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


if __name__ == '__main__':
    unittest.main()
