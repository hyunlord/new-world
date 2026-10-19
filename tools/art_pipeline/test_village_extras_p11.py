import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import run_village_extras_p11 as runner


class VillageExtrasTests(unittest.TestCase):
    def test_fixed_order_and_no_inherited_center(self):
        cases = runner.fixed_cases(20261018)
        self.assertEqual([(c.day, c.hour, c.zoom) for c in cases],
                         [(30, 11, 1), (60, 11, 1), (30, 14, 1), (60, 14, 1)])
        with patch.dict(os.environ, {'P11_ACTIVITY_CENTER_KIND': '5'}):
            env = runner.capture_environment(Path('/tmp/example'), cases[0], 'debug')
        self.assertNotIn('P11_ACTIVITY_CENTER_KIND', env)
        self.assertEqual(env['P11_ACTIVITY_ZOOM'], '1')

    def test_no_answers_refuses_before_any_launch(self):
        with tempfile.TemporaryDirectory() as temp, patch.object(runner.base, 'bounded_launch') as launch:
            with self.assertRaises(runner.base.InvalidInput):
                runner.run([temp, str(Path(temp) / 'out'), str(Path(temp) / 'missing')])
            launch.assert_not_called()
            self.assertFalse((Path(temp) / 'out').exists())

    def test_all_four_preregistered_and_failures_preserved(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            checkout, out, answers = root / 'repo', root / 'out', root / 'answers'
            selection = checkout / 'docs/art/production11/activity/selection.json'
            selection.parent.mkdir(parents=True)
            selection.write_text(json.dumps({'winner': {'seed': 20261018}}))
            script = checkout / runner.base.SCRIPT
            script.parent.mkdir(parents=True)
            script.write_text('fixture')
            answers.write_text('frozen fixture')
            calls = []
            def fake_launch(command, cwd, env, log, deadline, seconds):
                registration = json.loads((out / 'preregistration.json').read_text())
                self.assertEqual(len(registration['cases']), 4)
                self.assertEqual(registration['c_answers_sha256'], runner.base.sha256(answers))
                calls.append((env['P11_ACTIVITY_DAY'], env['P11_ACTIVITY_HOUR']))
                return 1  # No screenshot; every failed case must survive, all four attempted.
            with patch.dict(os.environ, {'P11_DEADLINE_EPOCH': '9999999999'}), \
                 patch.object(runner.os, 'access', return_value=True), \
                 patch.object(runner.base, 'bridge_identity', return_value=runner.base.BridgeIdentity('debug', 'loaded', None)), \
                 patch.object(runner.base, 'source_digest', return_value='stable'), \
                 patch.object(runner.base, 'wait_for_quiet'), \
                 patch.object(runner.base, 'launch_command', return_value=['fake']), \
                 patch.object(runner.base, 'bounded_launch', side_effect=fake_launch), \
                 patch.object(runner.subprocess, 'check_output', side_effect=lambda cmd, **kw: 'head' if kw.get('text') else b''):
                self.assertEqual(runner.run([str(checkout), str(out), str(answers)]), 1)
            self.assertEqual(calls, [('30', '11'), ('60', '11'), ('30', '14'), ('60', '14')])
            results = json.loads((out / 'results.json').read_text())
            self.assertEqual(len(results['cases']), 4)
            self.assertTrue(all(not row['capture_contract_pass'] for row in results['cases']))


if __name__ == '__main__':
    unittest.main()
