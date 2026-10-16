#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/art_pipeline -p test_deadline_p11.py
"""Admission/hold boundary checks with temporary locks and harmless children."""
from __future__ import annotations

import ast
import os
from pathlib import Path
import subprocess
import tempfile
import time
import unittest

SOURCE = Path(__file__).resolve().parent


class DeadlineTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(prefix='p11-deadline-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.tools = self.root / 'tools/art_pipeline'
        self.tools.mkdir(parents=True)
        self.lock = self.root / 'quiet.lock'
        source = (SOURCE / 'with_quiet_lock_p11.sh').read_text()
        source = source.replace('/Users/rexxa/work/.mac_quiet.lock', str(self.lock))
        (self.tools / 'with_quiet_lock_p11.sh').write_text(source)
        # The adapter deliberately has no global window lock and no Godot.
        (self.tools / 'with_window_lock.sh').write_text('shift\n"$@"\n')
        self.bin = self.root / 'bin'
        self.bin.mkdir()
        for name, body in [('sysctl', "echo '{ 0.10 0.10 0.10 }'"), ('pgrep', 'exit 1')]:
            path = self.bin / name
            path.write_text('#!/bin/sh\n' + body + '\n')
            path.chmod(0o755)
        self.marker = self.root / 'launched'
        self.env = dict(os.environ, PATH=str(self.bin) + os.pathsep + os.environ['PATH'])

    def run_hold(self, deadline: int, limit: int, seconds: int) -> subprocess.CompletedProcess[str]:
        env = dict(self.env, P11_DEADLINE_EPOCH=str(deadline))
        return subprocess.run(['bash', str(self.tools / 'with_quiet_lock_p11.sh'), 'synthetic', str(limit),
                               'bash', '-c', 'touch "$1"; sleep "$2"', '_', str(self.marker), str(seconds)],
                              env=env, capture_output=True, text=True, timeout=15, check=False)

    def test_expired_admission_never_launches(self) -> None:
        result = self.run_hold(int(time.time()) - 1, 8, 0)
        self.assertEqual(result.returncode, 124)
        self.assertFalse(self.marker.exists())
        self.assertFalse(self.lock.exists())

    def test_foreign_lock_survives_wait_deadline(self) -> None:
        self.lock.mkdir()
        info = self.lock / 'info.txt'
        info.write_text('foreign-owner\n')
        result = self.run_hold(int(time.time()) + 2, 8, 0)
        self.assertEqual(result.returncode, 124)
        self.assertFalse(self.marker.exists())
        self.assertEqual(info.read_text(), 'foreign-owner\n')

    def test_admitted_child_can_finish_past_admission_cutoff(self) -> None:
        cutoff = int(time.time()) + 2
        result = self.run_hold(cutoff, 8, 3)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertGreater(time.time(), cutoff)
        self.assertTrue(self.marker.exists())
        self.assertFalse(self.lock.exists())

    def test_hold_cap_still_terminates_child(self) -> None:
        start = time.monotonic()
        result = self.run_hold(int(time.time()) + 20, 3, 10)
        self.assertNotEqual(result.returncode, 0)
        self.assertLess(time.monotonic() - start, 6)
        self.assertTrue(self.marker.exists())
        self.assertFalse(self.lock.exists())

    def test_section_outer_wait_allows_one_hold_and_cleanup(self) -> None:
        tree = ast.parse((SOURCE / 'run_agent_sections_p11.py').read_text())
        expressions = [keyword.value for node in ast.walk(tree) if isinstance(node, ast.Call)
                       and isinstance(node.func, ast.Attribute) and node.func.attr == 'wait'
                       for keyword in node.keywords if keyword.arg == 'timeout'
                       and any(isinstance(part, ast.Name) and part.id == 'remaining' for part in ast.walk(keyword.value))]
        self.assertEqual(len(expressions), 1)
        expression = compile(ast.Expression(expressions[0]), '<timeout-policy>', 'eval')
        self.assertEqual(eval(expression, {'__builtins__': {}}, {'remaining': 2}), 3612)


if __name__ == '__main__':
    unittest.main()
