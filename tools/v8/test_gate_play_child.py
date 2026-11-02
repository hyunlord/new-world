"""Real harmless subprocess tests for the launcher receipt boundary."""
import json
import os
import subprocess
import unittest
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import gate_play_child as child
import generic_process
from generic_gate import GateContractError, document, integer
from generic_process import file_hash
from mux2_gate_inputs import Json


class ChildTests(unittest.TestCase):
    def exercise(self, argv: tuple[str,...], *, timeout: float | None = None,
                 merged: bool = False, mismatch: bool = False, overrides: dict[str,Json] | None = None, repeat: bool = False, role: str = "version") -> tuple[int, str, str, str]:
        with TemporaryDirectory() as folder:
            root=Path(folder).resolve();output=root/'child';plan=root/'plan.json'
            executable=Path('/bin/sh').resolve()
            body: dict[str,Json]={'schema_version':1,'roles':{role:{
                'argv':[a for a in argv],'cwd':str(Path.cwd()),'executable':str(executable),
                'executable_sha256':file_hash(executable),'inputs':{},'artifacts':[],
                'output':str(output),'timeout_seconds':timeout,'merged':merged}}}
            item=body['roles'];assert isinstance(item,dict)
            entry=item[role];assert isinstance(entry,dict)
            entry.update(overrides or {})
            _=plan.write_text(json.dumps(body))
            digest=file_hash(plan)
            if mismatch:digest='0'*64
            original_admit=child.admit
            def admitted(path: Path, checksum: str, name: str, args: tuple[str,...]) -> child.Child:
                value=original_admit(path,checksum,name,args)
                return replace(value,timeout=0.02) if timeout==600 else value
            with patch('sys.stdout') as stdout,patch('sys.stderr') as stderr,patch.object(child,'admit',side_effect=admitted):
                result=child.run(plan,digest,role,argv)
                _=stdout;_=stderr
                if repeat:_=child.run(plan,digest,role,argv)
            return result,(output/'receipt.json').read_text(),(output/'stdout.log').read_text(),(output/'stderr.log').read_text()

    def test_native_failure_and_outputs_preserved(self) -> None:
        code,receipt,stdout,stderr=self.exercise(('/bin/sh','-c','printf output; printf error >&2; exit 7'))
        self.assertEqual((code,stdout,stderr),(7,'output','error'))
        native=document(receipt);native.field('native_exit').equals(7)
        self.assertGreater(integer(native.field('pid')),0)

    def test_merged_original_output_preserved(self) -> None:
        code,_,stdout,stderr=self.exercise(('/bin/sh','-c','printf output; printf error >&2'),merged=True,role='build')
        self.assertEqual((code,stdout,stderr),(0,'outputerror',''))

    def test_wrong_plan_hash_prevents_child(self) -> None:
        with self.assertRaises(GateContractError):
            _=self.exercise(('/bin/sh','-c','exit 0'),mismatch=True)

    def test_timeout_retains_native_signal_and_124(self) -> None:
        code,receipt,stdout,stderr=self.exercise(('/bin/sh','-c','exec /bin/sleep 5'),timeout=600,merged=True,role='headless')
        self.assertEqual(code,124)
        self.assertEqual((stdout,stderr),('command exceeded the 600-second limit\n',''))
        value=document(receipt);value.field('timed_out').equals(True)
        value.field('native_exit').equals(-9)

    def test_bounded_children_see_eof_even_with_launcher_stdin(self) -> None:
        for role in ('import-1','import-2','headless'):
            read_fd,write_fd=os.pipe()
            saved_fd=os.dup(0)
            try:
                _=os.write(write_fd,b'launcher-input\n');os.close(write_fd)
                _=os.dup2(read_fd,0);os.close(read_fd)
                code,_,stdout,_=self.exercise(('/bin/sh','-c','if read value; then printf unexpected; else printf eof; fi'),timeout=600,merged=True,role=role)
                self.assertEqual((code,stdout),(0,'eof'))
            finally:
                _=os.dup2(saved_fd,0);os.close(saved_fd)

    def test_bounded_signal_exit_matches_python_wrapper(self) -> None:
        for role in ('import-1','import-2','headless'):
            code,receipt,_,_=self.exercise(('/bin/sh','-c','kill -KILL $$'),timeout=600,merged=True,role=role)
            self.assertEqual(code,247)
            document(receipt).field('native_exit').equals(-9)

    def test_original_bounded_wrapper_confirms_eof_and_signal_contract(self) -> None:
        wrapper=Path(generic_process.__file__).with_name('run_bounded.sh')
        for script,expected in (('if read value; then printf unexpected; else printf eof; fi',(0,b'eof')),
                                ('kill -KILL $$',(247,b''))):
            original=subprocess.run(['/bin/bash',str(wrapper),'600','/bin/sh','-c',script],
                                    input=b'launcher-input\n',capture_output=True,check=False)
            self.assertEqual((original.returncode,original.stdout),expected)

    def test_unbounded_signal_exit_keeps_shell_mapping(self) -> None:
        code,receipt,_,_=self.exercise(('/bin/sh','-c','kill -KILL $$'),merged=True,role='build')
        self.assertEqual(code,137)
        document(receipt).field('native_exit').equals(-9)

    def test_actual_environment_recorded(self) -> None:
        with patch.dict(os.environ,{'PLAY_CHILD_TEST':'actual'}):
            _,receipt,_,_=self.exercise(('/bin/sh','-c','exit 0'))
        document(receipt).field('environment').field('PLAY_CHILD_TEST').equals('actual')

    def test_argv_change_and_shell_metacharacters_cannot_execute(self) -> None:
        with self.assertRaises(GateContractError):
            _=self.exercise(('/bin/sh','-c','exit 0'),overrides={'argv':['/bin/sh','-c','touch /tmp/forbidden']})

    def test_wrong_working_directory_rejected(self) -> None:
        with self.assertRaises(GateContractError):
            _=self.exercise(('/bin/sh','-c','exit 0'),overrides={'cwd':'/'})

    def test_wrong_executable_hash_rejected(self) -> None:
        with self.assertRaises(GateContractError):
            _=self.exercise(('/bin/sh','-c','exit 0'),overrides={'executable_sha256':'0'*64})

    def test_reused_child_output_rejected(self) -> None:
        with self.assertRaises(GateContractError):
            _=self.exercise(('/bin/sh','-c','exit 0'),repeat=True)

    def test_invalid_input_hash_rejected_before_launch(self) -> None:
        with self.assertRaises(GateContractError):
            _=self.exercise(('/bin/sh','-c','exit 0'),overrides={'inputs':{str(Path('/bin/sh').resolve()):'0'*64}})

    def test_default_argument_rejections_unchanged(self) -> None:
        launcher=Path(__file__).with_name('play.sh')
        if not launcher.is_file():launcher=Path(__file__).resolve().parent.parent/'play.sh'
        for arguments,code,env in ((['m1','--speed','7'],4,{}),(['m1','--headless-check'],2,{'GODOT_BIN':'/nonexistent'})):
            result=subprocess.run(['/bin/bash',str(launcher),*arguments],env={'PATH':'/usr/bin:/bin','HOME':str(Path.home()),**env},capture_output=True,check=False)
            self.assertEqual(result.returncode,code)

if __name__=='__main__':
    _=unittest.main()
