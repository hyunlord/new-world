#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_mux2_gate_stall_command.py
"""Exercise shell sequencing with synthetic commands, never performance acceptance."""
import os
import copy
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import mux2_gate_receipts as receipts
from mux2_gate_inputs import Node


class P7StallAdmissionTests(unittest.TestCase):
    def fixture(self) -> dict:
        return {'p7_admission': {'deadline_epoch': 5000, 'wait_seconds': 900.0,
                                 'max_waits': 8, 'load1_limit_exclusive': 8.0}, 'arms': [{
            'sampling_started_epoch': 1901,
            'admission': {'status': 'ADMITTED', 'samples': [
                {'epoch': 1000, 'loads_1_5_15': [8.0, 3.0, 2.0],
                 'waits': 0, 'sysctl_exit': 0, 'action': 'WAIT'},
                {'epoch': 1900, 'loads_1_5_15': [7.99, 10.0, 12.0],
                 'waits': 1, 'sysctl_exit': 0, 'action': 'ADMITTED'},
            ]}}]}

    def test_wait_then_admit_records_all_three_loads(self) -> None:
        receipts.validate_p7_admission(Node(self.fixture()))

    def test_rejects_contended_missing_invalid_or_reordered_admission(self) -> None:
        mutations = [
            ('loads_1_5_15', [8.0, 1.0, 1.0]),
            ('loads_1_5_15', [7.0, float('nan'), 1.0]),
            ('loads_1_5_15', [7.0, -1.0, 1.0]),
            ('loads_1_5_15', [7.0, 1.0]),
            ('epoch', 1899), ('epoch', 1902), ('waits', 8),
            ('action', 'NOT_RUN'), ('sysctl_exit', 1),
        ]
        for field, value in mutations:
            report = copy.deepcopy(self.fixture())
            report['arms'][0]['admission']['samples'][-1][field] = value
            with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                receipts.validate_p7_admission(Node(report))
        report = self.fixture()
        del report['arms'][0]['admission']
        with self.assertRaises(ValueError):
            receipts.validate_p7_admission(Node(report))

    def test_rejects_changed_missing_or_nonfinite_admission_policy(self) -> None:
        for field, value in [('wait_seconds', 899), ('max_waits', 9),
                             ('load1_limit_exclusive', 8.1), ('deadline_epoch', float('inf'))]:
            report = self.fixture()
            report['p7_admission'][field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                receipts.validate_p7_admission(Node(report))
        report = self.fixture()
        del report['p7_admission']['wait_seconds']
        with self.assertRaises(ValueError):
            receipts.validate_p7_admission(Node(report))

    def test_allows_eighth_wait_but_rejects_ninth(self) -> None:
        report = self.fixture()
        report['p7_admission']['deadline_epoch'] = 20000
        report['arms'][0]['sampling_started_epoch'] = 8201
        samples = [{'epoch': 1000 + index * 900, 'loads_1_5_15': [8.0, 1.0, 1.0],
                    'waits': index, 'sysctl_exit': 0, 'action': 'WAIT'} for index in range(9)]
        samples[-1].update(loads_1_5_15=[7.0, 1.0, 1.0], action='ADMITTED')
        report['arms'][0]['admission']['samples'] = samples
        receipts.validate_p7_admission(Node(report))
        samples.append(dict(samples[-1], epoch=9100, waits=9))
        with self.assertRaises(ValueError):
            receipts.validate_p7_admission(Node(report))

    def test_timeout_preserves_non_p7_and_bounds_p7_waits_by_deadline(self) -> None:
        with patch.dict(os.environ, {}, clear=True):
            self.assertEqual(receipts.stall_timeout(), 960)
        with patch.dict(os.environ, {'CORE_P7_RUN': '1', 'CORE_P7_DEADLINE_EPOCH': '20000'}, clear=True), patch.object(receipts.time, 'time', return_value=1000):
            self.assertEqual(receipts.stall_timeout(), 15360)
        with patch.dict(os.environ, {'CORE_P7_RUN': '1', 'CORE_P7_DEADLINE_EPOCH': '3000.9'}, clear=True), patch.object(receipts.time, 'time', return_value=1000):
            self.assertEqual(receipts.stall_timeout(), 2000)
        for deadline in ('', 'nan', 'inf', '1600'):
            with patch.dict(os.environ, {'CORE_P7_RUN': '1', 'CORE_P7_DEADLINE_EPOCH': deadline}, clear=True), patch.object(receipts.time, 'time', return_value=1000):
                with self.assertRaises(ValueError):
                    receipts.stall_timeout()


class StallGateCommandTests(unittest.TestCase):
    def invoke(self, fail_action: str = '', receipt: str = '') -> tuple[int, list[list[str]]]:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binary = root / 'bin'
            binary.mkdir()
            calls = root / 'calls'
            fake = binary / 'python3'
            fake.write_text('''#!/bin/bash
printf '%s\\t' "$@" >> "$CALLS"
printf '\\n' >> "$CALLS"
if [[ "$1" == *mux2_gate_lock.py ]]; then exit 0; fi
if [[ "$2" == "$FAIL_ACTION" ]]; then exit 23; fi
exit 0
''')
            fake.chmod(0o755)
            environment = dict(os.environ, PATH=f'{binary}{os.pathsep}{os.environ["PATH"]}',
                               CALLS=str(calls), FAIL_ACTION=fail_action,
                               WORLDSIM_MUX2_LANE_FD='synthetic-sequence-only')
            environment.pop('MUX2_STALL_RECEIPT', None)
            if receipt:
                environment['MUX2_STALL_RECEIPT'] = receipt
            output = root / 'gate-stall'
            result = subprocess.run(
                ['bash', str(Path(__file__).with_name('mux2_gate_checks.sh')), 'stall', str(output)],
                env=environment, capture_output=True, text=True, timeout=5, check=False)
            rows = [line.rstrip('\t').split('\t') for line in calls.read_text().splitlines()]
            commands = [row for row in rows if row[0].endswith('mux2_gate_receipts.py')]
            for row in commands:
                if row[1] == 'capture':
                    self.assertEqual(Path(row[row.index('--out') + 1]), output / 'capture')
            return result.returncode, commands

    def test_default_captures_then_verifies_the_new_receipt(self) -> None:
        code, commands = self.invoke()
        self.assertEqual(code, 0)
        self.assertEqual([row[1] for row in commands], ['capture', 'verify'])
        output = Path(commands[0][commands[0].index('--out') + 1])
        self.assertEqual(Path(commands[1][commands[1].index('--receipt') + 1]), output / 'receipt.json')

    def test_failed_capture_cannot_become_a_verified_pass(self) -> None:
        code, commands = self.invoke(fail_action='capture')
        self.assertEqual(code, 23)
        self.assertEqual([row[1] for row in commands], ['capture'])

    def test_failed_verification_fails_the_step(self) -> None:
        code, commands = self.invoke(fail_action='verify')
        self.assertEqual(code, 23)
        self.assertEqual([row[1] for row in commands], ['capture', 'verify'])

    def test_explicit_stale_receipt_is_not_replaced_by_a_new_capture(self) -> None:
        path = '/missing/stale-receipt.json'
        code, commands = self.invoke(fail_action='verify', receipt=path)
        self.assertEqual(code, 23)
        self.assertEqual([row[1] for row in commands], ['verify'])
        self.assertEqual(commands[0][commands[0].index('--receipt') + 1], path)


if __name__ == '__main__':
    unittest.main()
