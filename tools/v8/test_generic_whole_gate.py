# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate
"""Real harmless shell/process tests; no product or native-world qualification."""
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path

import generic_gate
import generic_whole_gate_observations
from generic_gate import document

from generic_process import ProcessSpec, execute, file_hash
from generic_whole_gate import run_shell
from generic_whole_gate_contract import load_session
from generic_whole_gate_journal import ORDER, REFERENCE

HERE = Path(__file__).resolve().parent
ROOT = Path(generic_gate.__file__).resolve().parent
COLLECTOR = Path(generic_whole_gate_observations.__file__).resolve().parent


class WholeGateTests(unittest.TestCase):
    def test_fixture_session_cannot_gain_product_authority(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            home = Path(temporary).resolve()
            path = home/'session.json'
            _ = path.write_text(json.dumps({'schema':'whole_gate_session.v1',
                'qualification':'TEST_FIXTURE_NOT_PRODUCT_QUALIFIED', 'purpose':'publication',
                'arms':{},'harness':{'root':str(home),'inputs':{}},'external_inputs':{}}))
            with self.assertRaises(ValueError):
                _ = load_session(path,file_hash(path))

    def test_actual_51_shell_retains_incomplete_observations(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            home = Path(temporary).resolve()
            product = home/'product'
            (product/'tools/v8').mkdir(parents=True)
            output = home/'output'
            output.mkdir()
            script = product/'tools/v8/gate.sh'
            commands = '\n'.join(f'"{sys.executable}" "{HERE}/generic_whole_gate.py" step "$2" "$3" candidate {n} fixture_{n} -- /usr/bin/true || status=1' for n in ORDER)
            references = '\n'.join(f'"{sys.executable}" "{HERE}/generic_whole_gate.py" step "$2" "$3" base {n} fixture_{n} -- /usr/bin/true || status=1' for n in REFERENCE)
            _ = script.write_text('#!/bin/bash\nstatus=0\nif [[ "$4" == base ]]; then\n'+references+'\nelse\n'+commands+'\nfi\nexit "$status"\n')
            script.chmod(0o700)
            bash = Path('/bin/bash').resolve()
            session = home/'session.json'
            _ = session.write_text(json.dumps({'schema':'whole_gate_session.v1', 'qualification':'TEST_FIXTURE_NOT_PRODUCT_QUALIFIED', 'output':str(output),
                'arms': {'candidate': {'root':str(product),'source_inputs':{'tools/v8/gate.sh':file_hash(script)},'environment':{}},
                         'base':{'root':str(product),'source_inputs':{},'environment':{}}},
                'environment': {'PYTHONPATH':os.pathsep.join(map(str,(HERE,COLLECTOR,ROOT)))},
                'harness':{'root':str(product),'inputs':{}}, 'external_inputs':{},
                'bash':{'path':str(bash),'sha256':file_hash(bash)},
                'registry':{'path':str(ROOT/'gate_fixtures.json'),'sha256':file_hash(ROOT/'gate_fixtures.json')},
                'observations':{'probes':[]}}))
            reference = run_shell(session,file_hash(session),script,arm='base')
            document(reference.read_text()).field('shell_exit').equals(0)
            terminal = run_shell(session,file_hash(session),script)
            result = document(terminal.read_text())
            result.field('shell_exit').equals(0)
            result.field('complete_sequence').equals(True)
            result.field('status').equals('FAIL')
            self.assertEqual(len(list((output/'steps/candidate').glob('*/stage.json'))),51)
            self.assertEqual(len(list((output/'steps/base').glob('*/stage.json'))),31)

    def test_selected_fd_inherited_and_defaults_unchanged(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            home = Path(temporary).resolve()
            python = Path(sys.executable).resolve()
            with (home/'held').open('w+b') as held:
                descriptor = held.fileno()
                spec = ProcessSpec((str(python),'-c',f'import os; os.fstat({descriptor}); print("inherited")'),
                    home,python,file_hash(python),home,(),home/'process',pass_fds=(descriptor,))
                actual = execute(spec)
                self.assertEqual(actual.native_exit,0)
                document(actual.receipt.read_text()).field('pass_fds').equals([descriptor])
            default = ProcessSpec((str(python),'-c','pass'),home,python,file_hash(python),home,(),home/'default')
            result = execute(default)
            self.assertNotIn('pass_fds',document(result.receipt.read_text()).mapping())

    def test_stdio_descriptors_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            home = Path(temporary).resolve()
            python = Path(sys.executable).resolve()
            for fd in (0,1,2):
                spec = ProcessSpec((str(python),'-c','pass'),home,python,file_hash(python),home,(),home/f'fd{fd}',pass_fds=(fd,))
                with self.assertRaises(ValueError):
                    _ = execute(spec)
                self.assertFalse(spec.output_root.exists())

    def test_duplicate_and_closed_fd_reject_before_spawn(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            home = Path(temporary).resolve()
            python = Path(sys.executable).resolve()
            with (home/'held').open('w+b') as held:
                descriptor = held.fileno()
                spec = ProcessSpec((str(python),'-c','pass'),home,python,file_hash(python),home,(),home/'duplicate',pass_fds=(descriptor,descriptor))
                with self.assertRaises(ValueError):
                    _ = execute(spec)
            closed = ProcessSpec((str(python),'-c','pass'),home,python,file_hash(python),home,(),home/'closed',pass_fds=(descriptor,))
            with self.assertRaises(OSError):
                _ = execute(closed)
            self.assertFalse((home/'duplicate').exists())
            self.assertFalse((home/'closed').exists())


if __name__ == '__main__':
    _ = unittest.main()
