"""Regression tests use synthetic clocks and harmless subprocesses, never probes."""
from __future__ import annotations
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import session_validity_p13 as validity

ROOT = Path(__file__).resolve().parent


class AdmissionTests(unittest.TestCase):
    def test_retry_waits_from_completion_not_start(self):
        with tempfile.TemporaryDirectory() as directory, patch.dict(os.environ, {
            'P13_ATTEMPT_LEDGER': directory + '/ledger.json', 'P13_ITEM': 'world',
        }):
            with patch('session_validity_p13.time.time', return_value=100):
                self.assertTrue(validity.record_attempt(Path('first')))
            with patch('session_validity_p13.time.time', return_value=4000):
                self.assertFalse(validity.record_attempt(Path('unfinished')))
                validity.update_ledger(None, finish=Path('first'))
            with patch('session_validity_p13.time.time', return_value=5799):
                self.assertFalse(validity.record_attempt(Path('early')))
            with patch('session_validity_p13.time.time', return_value=5800):
                self.assertTrue(validity.record_attempt(Path('second')))
            data = json.loads(Path(directory + '/ledger.json').read_text())
            self.assertEqual(len(data['items']['world']), 2)

    def test_six_actual_attempts_despite_refusal_slots(self):
        with tempfile.TemporaryDirectory() as directory, patch.dict(os.environ, {
            'P13_ATTEMPT_LEDGER': directory + '/ledger.json', 'P13_ITEM': 'world',
        }):
            for index in range(6):
                with patch('session_validity_p13.time.time', return_value=100 + index * 1801):
                    output = Path(str(index))
                    self.assertTrue(validity.record_attempt(output))
                    validity.update_ledger(None, finish=output)
                    self.assertFalse(validity.record_attempt(Path('early')))
            with patch('session_validity_p13.time.time', return_value=99999):
                self.assertFalse(validity.record_attempt(Path('seventh')))

    def test_normalized_attempt_names_share_completion_clock(self):
        with tempfile.TemporaryDirectory() as directory, patch.dict(os.environ, {
            'P13_ATTEMPT_LEDGER': directory + '/ledger.json',
        }):
            os.environ.pop('P13_ITEM', None)
            with patch.dict(os.environ, {'P13_PURPOSE': 'baseline-W200-attempt1'}):
                with patch('session_validity_p13.time.time', return_value=100):
                    self.assertTrue(validity.record_attempt(Path('first')))
                    validity.update_ledger(None, finish=Path('first'))
            with patch.dict(os.environ, {'P13_PURPOSE': 'baseline-W200-attempt9'}):
                with patch('session_validity_p13.time.time', return_value=101):
                    self.assertFalse(validity.record_attempt(Path('ninth-slot')))
            self.assertEqual(list(json.loads(Path(directory + '/ledger.json').read_text())['items']), ['baseline-W200'])

    def test_wait_budget_accounts_union_across_quiet_and_window(self):
        with tempfile.TemporaryDirectory() as directory, patch.dict(os.environ, {
            'P13_ATTEMPT_LEDGER': directory + '/ledger.json',
        }):
            self.assertTrue(validity.update_ledger(None, (0, 10000)))
            self.assertTrue(validity.update_ledger(None, (9900, 10799)))
            self.assertTrue(validity.update_ledger(None, (10799, 10800)))
            self.assertFalse(validity.update_ledger(None, (10800, 10801)))
            self.assertEqual(json.loads(Path(directory + '/ledger.json').read_text())['waiting_seconds'], 10801)

    def test_window_callback_refuses_without_launching_command(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            lock = base / 'lock'
            lock.mkdir()
            callback = base / 'callback.py'
            callback.write_text('raise SystemExit(75)\n')
            script = base / 'window.sh'
            script.write_text((ROOT / 'with_window_lock.sh').read_text().replace('/Users/rexxa/work/.worldsim-godot-window.lock', str(lock)))
            result = subprocess.run(['bash', str(script), '1', 'touch', str(base / 'launched')],
                                    env=dict(os.environ, P13_WAIT_CALLBACK=str(callback)), capture_output=True)
            self.assertEqual(result.returncode, 75)
            self.assertFalse((base / 'launched').exists())
            self.assertTrue(lock.exists())

    def test_window_default_still_executes_and_releases_lock(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            script = base / 'window.sh'
            script.write_text((ROOT / 'with_window_lock.sh').read_text().replace('/Users/rexxa/work/.worldsim-godot-window.lock', str(base / 'lock')))
            environment = dict(os.environ)
            environment.pop('P13_WAIT_CALLBACK', None)
            result = subprocess.run(['bash', str(script), '2', 'touch', str(base / 'launched')],
                                    env=environment, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((base / 'launched').exists())
            self.assertFalse((base / 'lock').exists())

    def test_empty_filename_slots_under_macos_nounset(self):
        with tempfile.TemporaryDirectory() as directory:
            source = (ROOT / 'run_perf_matrix_p13.sh').read_text()
            selection = source[source.index('  slots=()'):source.index('    output=', source.index('  for attempt in ${slots[@]+'))]
            result = subprocess.run(['bash', '-eu', '-c', selection + '\n :; done'],
                                    env=dict(os.environ, evidence=directory, phase='baseline', world='W200'),
                                    text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_filename_slots_include_refusals_beyond_six(self):
        with tempfile.TemporaryDirectory() as directory:
            for index in range(1, 8):
                Path(directory, f'baseline-W200-attempt{index}.exit').write_text('75\n')
            source = (ROOT / 'run_perf_matrix_p13.sh').read_text()
            selection = source[source.index('  slots=()'):source.index('  for attempt in ${slots[@]+')]
            result = subprocess.run(['bash', '-c', selection + '\nprintf "%s\\n" "${slots[@]}"'],
                                    env=dict(os.environ, evidence=directory, phase='baseline', world='W200'),
                                    text=True, capture_output=True, check=True)
            self.assertEqual(set(result.stdout.split()), {str(index) for index in range(1, 8)})


if __name__ == '__main__':
    unittest.main()
