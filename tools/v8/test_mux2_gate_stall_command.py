#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_mux2_gate_stall_command.py
"""Exercise shell sequencing with synthetic commands, never performance acceptance."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


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
