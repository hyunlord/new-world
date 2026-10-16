"""Fake-only orchestration: no Godot, cargo, project code or real shared locks."""
import os
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from supervise_matrix import supervise, locked

SCRIPT = Path(__file__).with_name('supervise_matrix.py')

class CutoffTests(unittest.TestCase):
    def setup_case(self, root):
        folder = root / 'out'; folder.mkdir()
        fake = root / 'measurement'
        fake.write_text('#!/bin/sh\nprintf start >> "$P10_SUPERVISOR_DIR/measurement-events"\nsleep 1\nprintf finish >> "$P10_SUPERVISOR_DIR/measurement-events"\n')
        fake.chmod(0o700)
        shim = root / 'shim'
        shim.write_text(f'#!/bin/sh\nexec "{sys.executable}" "{SCRIPT}" --godot-shim "$@"\n')
        shim.chmod(0o700)
        return folder, fake, shim

    def test_active_measurement_finishes_and_next_launch_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); folder, fake, shim = self.setup_case(root)
            wrapper = root / 'quiet.sh'
            wrapper.write_text('"$GODOT_BIN"\n')
            runner = root / 'runner.sh'
            runner.write_text(f'bash "{wrapper}"; echo $? > "{folder}/first.exit"\n"{shim}"; echo $? > "{folder}/second.exit"\n')
            deadline = time.time() + 0.5
            env = dict(os.environ, GODOT_BIN=str(shim), P10_REAL_GODOT=str(fake), P10_SUPERVISOR_DIR=str(folder), P10_DEADLINE_EPOCH=str(deadline))
            supervise(['bash', str(runner)], folder, deadline, wrapper, env)
            self.assertEqual((folder/'measurement-events').read_text(), 'startfinish')
            self.assertEqual((folder/'first.exit').read_text().strip(), '0')
            self.assertEqual((folder/'second.exit').read_text().strip(), '124')

    def test_waiting_owned_wrapper_stopped_foreign_unaffected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); folder, fake, shim = self.setup_case(root)
            wrapper = root / 'quiet.sh'; wrapper.write_text('trap "exit 143" TERM\nsleep 20\n"$GODOT_BIN"\n')
            runner = root / 'runner.sh';runner.write_text(f'bash "{wrapper}"; echo $? > "{folder}/wait.exit"\n')
            deadline = time.time() + 0.2
            env = dict(os.environ, GODOT_BIN=str(shim), P10_REAL_GODOT=str(fake), P10_SUPERVISOR_DIR=str(folder), P10_DEADLINE_EPOCH=str(deadline))
            foreign = subprocess.Popen(['bash', str(wrapper)], env=env, start_new_session=True)
            try:
                started=time.time();supervise(['bash',str(runner)],folder,deadline,wrapper,env)
                self.assertLess(time.time()-started,5)
                self.assertIsNone(foreign.poll())
                self.assertFalse((folder/'measurement-events').exists())
                self.assertEqual((folder/'wait.exit').read_text().strip(),'143')
            finally:
                os.killpg(foreign.pid, signal.SIGTERM);foreign.wait()

    def test_preclosed_gate_never_launches(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);folder,fake,shim=self.setup_case(root)
            with locked(folder) as state:state['closed']=True
            env=dict(os.environ,P10_REAL_GODOT=str(fake),P10_SUPERVISOR_DIR=str(folder),P10_DEADLINE_EPOCH=str(time.time()+10))
            result=subprocess.run([str(shim)],env=env,check=False)
            self.assertEqual(result.returncode,124)
            self.assertFalse((folder/'measurement-events').exists())

if __name__=='__main__':unittest.main()
