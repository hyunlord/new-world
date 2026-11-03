# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_owner
"""Actual self-signalled owned child and isolated lane retention boundary."""
import fcntl
import json
import os
import signal
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from generic_gate import document

from generic_process import file_hash
from generic_whole_gate import run_shell


class OwnerTests(unittest.TestCase):
    def test_signal_retains_real_negative_wait_and_live_descriptor(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            product = root/'product'
            (product/'tools/v8').mkdir(parents=True)
            script = product/'tools/v8/gate.sh'
            _ = script.write_text('#!/bin/bash\nkill -TERM $$\n')
            script.chmod(0o700)
            output = root/'output'
            output.mkdir()
            bash = Path('/bin/bash').resolve()
            session = root/'session.json'
            _ = session.write_text(json.dumps({'schema':'whole_gate_session.v1',
                'qualification':'TEST_FIXTURE_NOT_PRODUCT_QUALIFIED', 'output':str(output),
                'arms':{'candidate':{'root':str(product),'source_inputs':{'tools/v8/gate.sh':file_hash(script)},'environment':{}}},
                'harness':{'root':str(product),'inputs':{}},'external_inputs':{},'environment':{},
                'bash':{'path':str(bash),'sha256':file_hash(bash)}}))
            with (root/'private-lane').open('w+b') as held:
                fcntl.flock(held.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                with patch('generic_whole_gate.signal.pause',side_effect=InterruptedError('bounded fixture')), self.assertRaises(InterruptedError):
                    _ = run_shell(session,file_hash(session),script,held.fileno())
                unknown = document((output/'ownership-unknown.json').read_text())
                unknown.field('status').equals('UNKNOWN')
                unknown.field('inode').equals(os.fstat(held.fileno()).st_ino)
                with (root/'private-lane').open('a') as other, self.assertRaises(BlockingIOError):
                    fcntl.flock(other.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            document((output/'candidate-shell/process.json').read_text()).field('native_exit').equals(-signal.SIGTERM)

    def test_direct_prepare_without_owner_rejects_before_selector_read(self) -> None:
        environment = dict(os.environ)
        _ = environment.pop('WORLDSIM_MUX2_LANE_FD',None)
        command = [sys.executable,str(Path(__file__).with_name('generic_whole_gate.py')),'prepare','/no/selector']
        actual = subprocess.run(command,env=environment,capture_output=True,text=True,check=False)
        self.assertNotEqual(actual.returncode,0)
        self.assertIn('prepare requires inherited owner',actual.stderr)


if __name__ == '__main__':
    _ = unittest.main()
