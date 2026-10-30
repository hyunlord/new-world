"""R2: the gate's inherited GPU lane descriptor survives the nested stall capture (step 41)."""
from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import unittest

from m6a_gate import lane_fds
from mux2_gate_lock import FD_ENV

CHILD = 'import os,sys; os.fstat(int(os.environ[sys.argv[1]])); print("held")'


class LaneDescriptorTests(unittest.TestCase):
    def _run(self, pass_fds: tuple[int, ...]) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryFile() as lock:
            env = dict(os.environ, **{FD_ENV: str(lock.fileno())})
            fds = pass_fds or lane_fds(env)
            return subprocess.run([sys.executable, '-c', CHILD, FD_ENV], env=env, text=True,
                                  capture_output=True, pass_fds=fds, check=False)

    def test_without_pass_fds_child_sees_ebadf(self) -> None:
        # The R1 failure: close_fds=True drops the descriptor named in the environment.
        with tempfile.TemporaryFile() as lock:
            env = dict(os.environ, **{FD_ENV: str(lock.fileno())})
            result = subprocess.run([sys.executable, '-c', CHILD, FD_ENV], env=env, text=True,
                                    capture_output=True, check=False)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('Bad file descriptor', result.stderr)

    def test_lane_fds_keeps_the_inherited_descriptor(self) -> None:
        result = self._run(())
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), 'held')

    def test_no_inherited_lane_passes_nothing(self) -> None:
        self.assertEqual(lane_fds({}), ())
        self.assertEqual(lane_fds({FD_ENV: '7'}), (7,))


if __name__ == '__main__':
    unittest.main()
