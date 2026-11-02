"""Actual tiny shell children; Cargo source/tool qualification explicitly mocked."""
import fcntl
import os
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import gate_oracle_child as child
import generic_process
import generic_setup_import as imports
from generic_gate import document
from generic_process import file_hash
from generic_setup_bridge import run_bridge_owned
from generic_setup_prepare import PreparedSetup, Tool, prepare
from mux2_gate_inputs import Json
from test_generic_setup_bridge import unused_wait
from test_generic_setup_prepare import Fixture

ROOT=Path(__file__).resolve().parent
DEPS=Path(generic_process.__file__).resolve().parent


def make_fixture(home: Path, first: int, second: int) -> Fixture:
    f=Fixture(home)
    for name in ('bash','python3'):(f.bin/name).unlink()
    (f.bin/'python3').symlink_to(Path(sys.executable).resolve())
    _=(f.bin/'cargo').write_text('#!/bin/sh\n/bin/mkdir -p target/debug\nprintf bridge > target/debug/libsim_bridge.dylib\n')
    _=(f.bin/'godot').write_text('#!/bin/sh\n/bin/mkdir -p .godot\nif test -f .godot/first; then\n'
        +'printf second > .godot/second\n'+(f'exit {second}' if second>=0 else 'kill -TERM $$')+'\nelse\nprintf first > .godot/first\n'+(f'exit {first}' if first>=0 else 'kill -TERM $$')+'\nfi\n')
    names=('run_bounded.sh','gate_oracle_child.py','gate_stall_child.py','generic_gate.py','generic_process.py',
           'generic_sources.py','mux2_gate_inputs.py','mux2_gate_lock.py','generic_setup_import.py')
    for name in names:
        source=ROOT/name if (ROOT/name).exists() else DEPS/name
        value=source.read_text()
        if name=='gate_oracle_child.py':
            value=value.replace('from mux2_gate_lock import FD_ENV, LOCK',
                'from mux2_gate_lock import FD_ENV\nLOCK = Path('+repr(str(home/'lock'))+')')
        _=(f.root/'tools/v8'/name).write_text(value)
    _=f.git('add','.')
    _=f.git('-c','user.name=Test','-c','user.email=t@example.invalid','commit','-qm','bound tiny helpers')
    names=(*f.git('ls-files').splitlines(),'rust/Cargo.lock')
    tools=tuple(Tool(t.name,Path('/bin/bash') if t.name=='bash' else Path(sys.executable).resolve() if t.name=='python3' else t.path,
          file_hash(Path('/bin/bash') if t.name=='bash' else Path(sys.executable).resolve() if t.name=='python3' else t.path)) for t in f.inputs.tools)
    env=tuple((k,v+':/bin' if k=='PATH' else v) for k,v in f.inputs.environment)+(('__CF_USER_TEXT_ENCODING',f'0x{os.getuid():X}:0x3:0x33'),('PYTHONCOERCECLOCALE','0'))
    f.inputs=replace(f.inputs,source_sha=f.git('rev-parse','HEAD').strip(),tools=tools,environment=env,
                     effective_inputs=tuple(sorted((name,file_hash(f.root/name)) for name in names)))
    return f


class ImportTests(unittest.TestCase):
    def exercise(self, first: int, second: int, mutation: str='') -> None:
        with tempfile.TemporaryDirectory() as temp:
            home=Path(temp).resolve();f=make_fixture(home,first,second);prepared=prepare(f.inputs)
            lock=home/'lock'
            with lock.open('w') as lane,patch('generic_setup_bridge.LOCK',lock),patch.object(child,'LOCK',lock),\
                 patch('generic_setup_bridge.qualify_wait',return_value=()),patch.object(imports,'qualify_wait',return_value=()):
                fcntl.flock(lane.fileno(),fcntl.LOCK_EX|fcntl.LOCK_NB)
                bridge=run_bridge_owned(prepared,unused_wait(prepared),lane.fileno())
                prior=imports.PriorBridge(bridge,file_hash(bridge.receipt),unused_wait(prepared))
                if mutation=='settings':
                    with self.assertRaises(imports.ImportFailure) as caught:
                        _=imports.start_import_pair(prepared,prior,f.inputs.settings,lane.fileno())
                    self.assertIsNone(caught.exception.owner);self.assertFalse(f.inputs.settings.exists())
                    return
                if mutation=='python':
                    (f.bin/'python3').unlink();(f.bin/'python3').symlink_to('/bin/false')
                    with self.assertRaises(imports.ImportFailure) as caught:
                        _=imports.start_import_pair(prepared,prior,home/'imports',lane.fileno())
                    self.assertIsNone(caught.exception.owner)
                    self.assertFalse((home/'imports/children/suite-start.json').exists())
                    return
                if mutation:
                    path=bridge.receipt if mutation=='receipt' else f.root/'rust/target/debug/libsim_bridge.dylib' if mutation=='bridge' else f.inputs.output/'process/stdout.log'
                    _=path.write_text('changed')
                    with self.assertRaises(imports.ImportFailure) as caught:
                        _=imports.start_import_pair(prepared,prior,home/'imports',lane.fileno())
                    self.assertIsNone(caught.exception.owner)
                    self.assertFalse((home/'imports').exists())
                    return
                owner=imports.start_import_pair(prepared,prior,home/'imports',lane.fileno())
                try:result=owner.finish()
                finally:
                    if owner.controller.shell.poll() is None:
                        owner.controller.cancel('fixture cleanup');_=owner.controller.shell.wait(timeout=5)
                    if not owner.controller.closed:
                        if owner.controller.control_write>=0:os.close(owner.controller.control_write)
                        os.close(owner.controller.request_read)
                    owner.stream.close()
                self.assertEqual((result.first_exit,result.second_exit,result.shell_exit),(first,second,second%256))
                self.assertEqual(result.status,'PASS_STEP3_ONLY' if second==0 else 'FAIL_STEP3')
                self.assertTrue((f.root/'.godot/first').exists());self.assertTrue((f.root/'.godot/second').exists())
                self.assertFalse(result.build_product_authority)
                first_state=document((owner.controller.plan.rows[0].evidence/'import-checkpoint.json').read_text()).field('cache').mapping()
                second_state=document((owner.controller.plan.rows[1].evidence/'import-checkpoint.json').read_text()).field('cache').mapping()
                self.assertIn('first',first_state);self.assertNotIn('second',first_state);self.assertIn('second',second_state)
                for row in owner.controller.plan.rows:
                    raw=document((row.evidence/'native-terminal.json').read_text())
                    self.assertEqual(raw.field('settlement').value,'SETTLED')
                    self.assertEqual(raw.field('request').field('limit').value,600)
                with lock.open('a') as other,self.assertRaises(BlockingIOError):fcntl.flock(other.fileno(),fcntl.LOCK_EX|fcntl.LOCK_NB)

    def test_pair_success(self) -> None:self.exercise(0,0)
    def test_first_failure_still_runs_second(self) -> None:self.exercise(7,0)
    def test_second_failure_sets_original_aggregate(self) -> None:self.exercise(0,9)
    def test_both_failures_retained(self) -> None:self.exercise(7,9)
    def test_first_signal_still_runs_second(self) -> None:self.exercise(-15,0)
    def test_second_signal_retains_raw_and_shell_mapping(self) -> None:self.exercise(0,-15)
    def test_prior_receipt_mutation(self) -> None:self.exercise(0,0,'receipt')
    def test_prior_bridge_mutation(self) -> None:self.exercise(0,0,'bridge')
    def test_prior_raw_log_mutation(self) -> None:self.exercise(0,0,'log')
    def test_actual_entry_rejects_retargeted_python(self) -> None:self.exercise(0,0,'python')
    def test_output_cannot_overlap_generated_settings(self) -> None:self.exercise(0,0,'settings')

    def test_interruption_retains_owner_and_lane_without_second_native(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            home=Path(temp).resolve();f=make_fixture(home,0,0);prepared=prepare(f.inputs);lock=home/'lock'
            with lock.open('w') as lane,patch('generic_setup_bridge.LOCK',lock),patch.object(child,'LOCK',lock),patch('generic_setup_bridge.qualify_wait',return_value=()),patch.object(imports,'qualify_wait',return_value=()):
                bridge=run_bridge_owned(prepared,unused_wait(prepared),lane.fileno())
                prior=imports.PriorBridge(bridge,file_hash(bridge.receipt),unused_wait(prepared))
                owner=imports.start_import_pair(prepared,prior,home/'imports',lane.fileno())
                failure=KeyboardInterrupt('fixture interruption')
                try:
                    with patch.object(child.SuiteController,'service_ready',side_effect=failure),self.assertRaises(imports.ImportFailure) as caught:
                        _=owner.finish()
                    self.assertIs(caught.exception.owner,owner);self.assertIs(caught.exception.primary,failure)
                    self.assertTrue(owner.controller.halted)
                    self.assertFalse((home/'imports/receipt.json').exists())
                    self.assertEqual(document((home/'imports/failure.json').read_text()).field('ownership').value,'UNKNOWN_KEEP_CALLER_LANE')
                    with lock.open('a') as other,self.assertRaises(BlockingIOError):fcntl.flock(other.fileno(),fcntl.LOCK_EX|fcntl.LOCK_NB)
                finally:
                    _=owner.controller.shell.wait(timeout=5)
                    os.close(owner.controller.request_read);owner.stream.close()
                self.assertFalse((f.root/'.godot').exists())

    def test_python_target_replaced_after_preparation_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f=make_fixture(Path(temp).resolve(),0,0);prepared=prepare(f.inputs)
            (f.bin/'python3').unlink();(f.bin/'python3').symlink_to('/bin/false')
            # Initial preparation binds interpreter resolution, not only the lexical alias.
            with self.assertRaisesRegex(ValueError,'PATH tool mismatch'):_=prepare(prepared.inputs)

    def test_postflight_interruption_after_both_waits_is_not_success(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            home=Path(temp).resolve();f=make_fixture(home,0,0);prepared=prepare(f.inputs);lock=home/'lock'
            with lock.open('w') as lane,patch('generic_setup_bridge.LOCK',lock),patch.object(child,'LOCK',lock),patch('generic_setup_bridge.qualify_wait',return_value=()),patch.object(imports,'qualify_wait',return_value=()):
                bridge=run_bridge_owned(prepared,unused_wait(prepared),lane.fileno())
                prior=imports.PriorBridge(bridge,file_hash(bridge.receipt),unused_wait(prepared))
                owner=imports.start_import_pair(prepared,prior,home/'imports',lane.fileno())
                failure=SystemExit(31)
                try:
                    with patch.object(imports,'authenticate_bridge',side_effect=failure),self.assertRaises(imports.ImportFailure) as caught:
                        _=owner.finish()
                    self.assertIs(caught.exception.primary,failure)
                    self.assertEqual(owner.controller.completed,['import.first','import.second'])
                    self.assertEqual(owner.controller.shell.returncode,0)
                    self.assertFalse((home/'imports/receipt.json').exists())
                    self.assertEqual(document((home/'imports/failure.json').read_text()).field('status').value,'FAIL')
                finally:
                    if owner.controller.shell.poll() is None:_=owner.controller.shell.wait(timeout=5)
                    if owner.controller.control_write>=0:os.close(owner.controller.control_write)
                    os.close(owner.controller.request_read);owner.stream.close()

    def test_terminal_state_must_match_last_native_checkpoint(self) -> None:
        for phase in ('before','after'):
            for change in ('cache','settings','settings_bytes'):
                with self.subTest(phase=phase,change=change):self.terminal_change(phase,change)

    def terminal_change(self, phase: str, change: str) -> None:
        with tempfile.TemporaryDirectory() as temp:
            home=Path(temp).resolve();f=make_fixture(home,0,0)
            godot=f.bin/'godot'
            _=godot.write_text(godot.read_text().replace('#!/bin/sh\n',
                '#!/bin/sh\n/bin/mkdir -p "$WORLDSIM_TEST_SETTINGS_DIR"\nprintf stable > "$WORLDSIM_TEST_SETTINGS_DIR/state"\n'))
            f.inputs=replace(f.inputs,tools=tuple(replace(t,sha256=file_hash(godot)) if t.name=='godot' else t for t in f.inputs.tools))
            prepared=prepare(f.inputs);lock=home/'lock'
            with lock.open('w') as lane,patch('generic_setup_bridge.LOCK',lock),patch.object(child,'LOCK',lock),patch('generic_setup_bridge.qualify_wait',return_value=()),patch.object(imports,'qualify_wait',return_value=()):
                bridge=run_bridge_owned(prepared,unused_wait(prepared),lane.fileno())
                prior=imports.PriorBridge(bridge,file_hash(bridge.receipt),unused_wait(prepared))
                owner=imports.start_import_pair(prepared,prior,home/'imports',lane.fileno())
                def mutate() -> None:
                    self.assertEqual(owner.controller.completed,['import.first','import.second'])
                    if change=='settings_bytes':_=(f.inputs.settings/'state').write_text('changed')
                    else:((f.root/'.godot' if change=='cache' else f.inputs.settings)/'late-empty').mkdir()
                authenticate=imports.authenticate_bridge;publish=generic_process.atomic_json
                def after_auth(p: PreparedSetup, b: imports.PriorBridge) -> tuple[tuple[Path,str],...]:
                    result=authenticate(p,b)
                    if phase=='before':mutate()
                    return result
                def after_publish(path: Path, value: dict[str, Json]) -> None:
                    publish(path,value)
                    if phase=='after' and path.name=='receipt.json':mutate()
                try:
                    with patch.object(imports,'authenticate_bridge',side_effect=after_auth),patch.object(imports,'atomic_json',side_effect=after_publish),self.assertRaises(imports.ImportFailure) as caught:
                        _=owner.finish()
                    self.assertIs(caught.exception.owner,owner)
                    self.assertEqual(owner.controller.shell.returncode,0)
                    self.assertEqual(document((home/'imports/failure.json').read_text()).field('ownership').value,'UNKNOWN_KEEP_CALLER_LANE')
                    with lock.open('a') as rival,self.assertRaises(BlockingIOError):fcntl.flock(rival.fileno(),fcntl.LOCK_EX|fcntl.LOCK_NB)
                    for row in owner.controller.plan.rows:
                        self.assertEqual(document((row.evidence/'native-terminal.json').read_text()).field('native_exit').value,0)
                finally:
                    if owner.controller.shell.poll() is None:
                        owner.controller.cancel('fixture cleanup');_=owner.controller.shell.wait(timeout=5)
                    if not owner.controller.closed:
                        if owner.controller.control_write>=0:os.close(owner.controller.control_write)
                        os.close(owner.controller.request_read)
                    owner.stream.close()
