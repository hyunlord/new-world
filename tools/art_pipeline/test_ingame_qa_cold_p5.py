"""Cold evidence must include the rendered cycle and executed scene checks."""
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from ingame_qa_cold_p5 import cold
from test_ingame_qa import capture


class ColdEvidenceTests(unittest.TestCase):
    def test_missing_cycle_cannot_pass(self):
        with TemporaryDirectory() as folder:
            root = Path(folder)
            (root / 'results.tsv').write_text('test_cold_presentation\t0\t0\ntest_hud_wood\t0\t0\ntest_inspector_cold\t0\t0\n')
            data = {'agent_snapshot': {'agent_ids': [7], 'colds': [1]}, 'entities': [
                {'kind': 'agent', 'id': 7, 'visible': True, 'cold': True, 'pose': 'crossed'}]}
            (root / 'frame.entities.json').write_text(json.dumps(data))
            check = cold([capture(root / 'frame.png', ())], root)
            self.assertEqual(check.status, 'FAIL')
            self.assertIn('missing observed cold cycle', check.detail)
            samples = [{'real_time_usec': stamp, 'agents': [{'id': 7, 'cold': True, 'puff_alpha': alpha}]} for stamp, alpha in [(0, 0.5), (2_500_000, 0)]]
            (root / 'frame.cold_cycle.json').write_text(json.dumps({'samples': samples}))
            self.assertEqual(cold([capture(root / 'frame.png', ())], root).status, 'PASS')

    def test_missing_scene_test_is_not_a_pass(self):
        with TemporaryDirectory() as folder:
            root = Path(folder)
            (root / 'frame.entities.json').write_text('{"entities":[],"agent_snapshot":{"agent_ids":[],"colds":[]}}')
            self.assertEqual(cold([capture(root / 'frame.png', ())], root).status, 'FAIL')
