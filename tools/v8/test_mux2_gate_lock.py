#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_mux2_gate_lock.py
"""Real subprocess contention and inherited-lock checks, no runtime workloads."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from mux2_gate_lock import FD_ENV, lane


class GateLockTests(unittest.TestCase):
    def setUp(self):
        self.inherited = os.environ.pop(FD_ENV, None)

    def tearDown(self):
        if self.inherited is not None:
            os.environ[FD_ENV] = self.inherited

    def invoke(self, path: Path, descriptor: int | None) -> subprocess.CompletedProcess[str]:
        environment = dict(os.environ)
        environment.pop(FD_ENV, None)
        descriptors = ()
        if descriptor is not None:
            environment[FD_ENV] = str(descriptor)
            descriptors = (descriptor,)
        code = ('import sys; from pathlib import Path; '
                'from mux2_gate_lock import lane; '
                'ctx=lane(Path(sys.argv[1])); ctx.__enter__(); print("acquired"); ctx.__exit__(None,None,None)')
        return subprocess.run(['bash', '-c', 'exec "$@"', 'nested-gate', sys.executable, '-c', code, str(path)],
                              cwd=Path(__file__).parent, env=environment, pass_fds=descriptors,
                              capture_output=True, text=True, timeout=5, check=False)

    def test_other_gate_or_capture_fails_while_lane_is_held(self):
        with tempfile.TemporaryDirectory() as tmp, lane(Path(tmp) / 'lock'):
            result = self.invoke(Path(tmp) / 'lock', None)
            self.assertNotEqual(result.returncode, 0)

    def test_nested_capture_reuses_inherited_descriptor_without_deadlock(self):
        with tempfile.TemporaryDirectory() as tmp, lane(Path(tmp) / 'lock') as descriptor:
            result = self.invoke(Path(tmp) / 'lock', descriptor)
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_wrong_inherited_file_cannot_bypass_lane(self):
        with tempfile.TemporaryDirectory() as tmp:
            wrong = Path(tmp) / 'wrong'
            wanted = Path(tmp) / 'wanted'
            wanted.touch()
            with lane(wrong) as descriptor:
                result = self.invoke(wanted, descriptor)
                self.assertNotEqual(result.returncode, 0)

    def test_direct_helper_rejects_invalid_lane_before_any_work(self):
        # Invalid mode ensures even a broken guard cannot launch Cargo or Godot.
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / 'not-created'
            environment = dict(os.environ, **{FD_ENV: '999999'})
            helper = Path(__file__).parent / 'mux2_gate_checks.sh'
            result = subprocess.run(['bash', str(helper), 'invalid-mode', str(output)],
                                    env=environment, capture_output=True, text=True,
                                    timeout=5, check=False)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('MUX2_GATE_LOCK_FAIL', result.stderr)
            self.assertFalse(output.exists())
