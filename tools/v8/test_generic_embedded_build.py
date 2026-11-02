"""Real tiny shell children; predecessor admission and Cargo source qualification explicitly mocked."""
import fcntl
import tempfile
import unittest
from collections.abc import Callable
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import generic_embedded_build as embedded
from generic_process import ProcessResult, ProcessSpec, atomic_json, execute, file_hash
from generic_setup_clippy import (
    ClippyEndpoint,
    ClippyRequest,
    ClippyResult,
    ClippyWaitEvidence,
)
from generic_setup_prepare import PreparedSetup, Tool, prepare
from generic_workspace_capture import WorkspaceCapture
from generic_workspace_endpoints import (
    GeneratedState,
    WorkspaceEndpoint,
    WorkspaceRequest,
    WorkspaceResult,
    anchor,
    generated,
)
from generic_workspace_tests import Inventory
from mux2_gate_inputs import Json
from test_generic_setup_bridge import unused_wait
from test_generic_setup_prepare import Fixture


def fixture(home: Path, code: int = 0) -> tuple[PreparedSetup, ClippyEndpoint]:
    f = Fixture(home)
    gate = f.root/'tools/v8/gate.sh'
    _ = gate.write_text(gate.read_text()+'\n'+'\n'.join(line for step in (11,12,18) for line in embedded.SOURCES[step])+'\n')
    _ = (f.root/'tools/v8/identity.sh').write_text('\n'.join(embedded.SOURCES[10])+'\n')
    _ = f.git('add','.'); _ = f.git('-c','user.name=Test','-c','user.email=t@example.invalid','commit','-qm','embedded sources')
    (f.bin/'bash').unlink()
    _ = (f.bin/'cargo').write_text('#!/bin/sh\nprintf "%s\\n" "$@"\nprintf built > target/build-marker\nexit '+str(code)+'\n')
    tools = tuple(Tool(t.name,Path('/bin/bash') if t.name == 'bash' else t.path,
                       file_hash(Path('/bin/bash') if t.name == 'bash' else t.path)) for t in f.inputs.tools)
    inputs = replace(f.inputs,source_sha=f.git('rev-parse','HEAD').strip(),tools=tools,
        environment=tuple((k,v+':/bin' if k=='PATH' else v) for k,v in f.inputs.environment),
        effective_inputs=tuple(sorted((n,file_hash(f.root/n)) for n in (*f.git('ls-files').splitlines(),'rust/Cargo.lock'))))
    prepared = prepare(inputs)
    (f.root/'rust/target').mkdir(); _ = (f.root/'rust/target/prior').write_text('old')
    history = home/'old'; history.mkdir(); receipt = history/'receipt.json'; _ = receipt.write_text('{}')
    state = generated(prepared)
    spec = ProcessSpec(('/bin/bash',),f.root,Path('/bin/bash'),file_hash(Path('/bin/bash')),f.root,(),history/'process')
    request = WorkspaceRequest(5,'candidate','explicit-synthetic-history',spec,Inventory((),()),receipt,file_hash(receipt),history,-1)
    result = WorkspaceResult(receipt,'OBSERVED_CASES_ONLY',WorkspaceCapture((),(),(),(),'PASS',0,'PASS','COMPLETE','PASS'))
    prior = WorkspaceEndpoint(request,result,prepared,5,'candidate','explicit-synthetic-history',receipt,file_hash(receipt),state,state,(anchor(history),))
    clippy_request = ClippyRequest(prepared,ClippyWaitEvidence(unused_wait(prepared),()),history,-1,prior)
    return prepared,ClippyEndpoint(clippy_request,prior,ClippyResult(receipt,0,1),file_hash(receipt),state,state,(anchor(history),))


class EmbeddedTests(unittest.TestCase):
    def test_exact_occurrences_real_pid_and_immutable_history(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home=Path(tmp).resolve(); prepared,prior=fixture(home); lock=home/'lock'
            with lock.open('w') as lane, patch('generic_setup_bridge.LOCK',lock), patch.object(embedded,'check_clippy_endpoint'), patch.object(embedded,'qualify_wait',return_value=()):
                previous: ClippyEndpoint | embedded.EmbeddedEndpoint = prior
                for step in (10,11,12,18):
                    request=embedded.EmbeddedRequest(prepared,step,previous,unused_wait(prepared),home/f'step{step}',lane.fileno())
                    result=embedded.run_owned(request)
                    self.assertGreater(result.pid,0); self.assertFalse(result.publication_authority)
                    self.assertFalse(result.build_product_authority)
                    self.assertEqual(result.before,previous.after)
                    self.assertEqual((request.output/'process/stdout.log').read_text().splitlines(),
                        ['build','--release','-p','sim-test']+([] if step==12 else ['--bin','worldsim_probe']))
                    embedded.check_endpoint(result)
                    with self.assertRaises(ValueError): embedded.check_endpoint(replace(result,history=()))
                    changed=replace(result.process,spec=replace(result.process.spec,argv=('/bin/false',)))
                    with self.assertRaises(ValueError): embedded.check_endpoint(replace(result,process=changed))
                    previous=result
                with lock.open('a') as rival,self.assertRaises(BlockingIOError):
                    fcntl.flock(rival.fileno(),fcntl.LOCK_EX|fcntl.LOCK_NB)
                (prepared.inputs.root/'rust/target/later-producer').mkdir()
                assert isinstance(previous,embedded.EmbeddedEndpoint)
                embedded.check_endpoint(previous)
                with self.assertRaises(embedded.EmbeddedFailure):
                    _=embedded.run_owned(replace(request,output=home/'stale'))

    def test_native_failure_is_unknown_and_keeps_actual_exit(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home=Path(tmp).resolve(); prepared,prior=fixture(home,17); lock=home/'lock'
            with lock.open('w') as lane,patch('generic_setup_bridge.LOCK',lock),patch.object(embedded,'check_clippy_endpoint'),patch.object(embedded,'qualify_wait',return_value=()):
                request=embedded.EmbeddedRequest(prepared,11,prior,unused_wait(prepared),home/'step11',lane.fileno())
                with self.assertRaises(embedded.EmbeddedOwnershipUnknown): _=embedded.run_owned(request)
                self.assertIn('"native_exit": 17',(request.output/'receipt.json').read_text())
                self.assertTrue((request.output/'process/stderr.log').is_file())
                with lock.open('a') as rival,self.assertRaises(BlockingIOError):
                    fcntl.flock(rival.fileno(),fcntl.LOCK_EX|fcntl.LOCK_NB)

    def test_prelaunch_rejects_without_source_authority_or_current_endpoint(self) -> None:
        for fault in ('wait','prior','drift','descriptor','source'):
            with self.subTest(fault=fault),tempfile.TemporaryDirectory() as tmp:
                home=Path(tmp).resolve(); prepared,prior=fixture(home); lock=home/'lock'
                if fault=='drift': (prepared.inputs.root/'rust/target/unobserved').mkdir()
                if fault=='source': _=(prepared.inputs.root/'tools/v8/gate.sh').write_text('changed')
                with lock.open('w') as lane,patch('generic_setup_bridge.LOCK',lock),patch.object(embedded,'execute') as launch:
                    request=embedded.EmbeddedRequest(prepared,11,prior,unused_wait(prepared),home/'step11',-1 if fault=='descriptor' else lane.fileno())
                    with patch.object(embedded,'check_clippy_endpoint',side_effect=ValueError('unadmitted') if fault=='prior' else None),patch.object(embedded,'qualify_wait',side_effect=ValueError('unqualified') if fault=='wait' else None,return_value=()),self.assertRaises(embedded.EmbeddedFailure):
                        _=embedded.run_owned(request)
                    launch.assert_not_called()

    def test_unreturned_and_diagnostic_exception_objects_retained(self) -> None:
        for primary in (TypeError('primary'),KeyboardInterrupt(),SystemExit(7)):
            with self.subTest(kind=type(primary)),tempfile.TemporaryDirectory() as tmp:
                home=Path(tmp).resolve(); prepared,prior=fixture(home); lock=home/'lock'; secondary=OSError('secondary')
                def publish(path: Path,row: dict[str,Json], secondary: OSError = secondary) -> None:
                    if path.name=='receipt.json': raise secondary
                    atomic_json(path,row)
                with lock.open('w') as lane,patch('generic_setup_bridge.LOCK',lock),patch.object(embedded,'check_clippy_endpoint'),patch.object(embedded,'qualify_wait',return_value=()),patch.object(embedded,'execute',side_effect=primary),patch.object(embedded,'atomic_json',side_effect=publish):
                    request=embedded.EmbeddedRequest(prepared,11,prior,unused_wait(prepared),home/'step11',lane.fileno())
                    with self.assertRaises(embedded.EmbeddedOwnershipUnknown) as caught: _=embedded.run_owned(request)
                    self.assertIs(caught.exception.primary,primary);self.assertIs(caught.exception.secondary,secondary)

    def test_boundary_mutations_reject_without_rebasing(self) -> None:
        for fault in ('log','cache','settings','target-publication','history-publication','receipt-publication','directory-publication','mode-publication'):
            with self.subTest(fault=fault),tempfile.TemporaryDirectory() as tmp:
                home=Path(tmp).resolve(); prepared,prior=fixture(home); lock=home/'lock'
                def producer(spec: ProcessSpec, fault: str = fault, prepared: PreparedSetup = prepared) -> ProcessResult:
                    result=execute(spec)
                    if fault=='log': _=(spec.output_root/'stdout.log').write_text('forged')
                    if fault in ('cache','settings'):
                        directory=prepared.inputs.root/'.godot' if fault=='cache' else prepared.inputs.settings
                        directory.mkdir();(directory/'unexpected').mkdir()
                    return result
                def publish(path: Path,row: dict[str,Json], fault: str = fault, prepared: PreparedSetup = prepared, prior: ClippyEndpoint = prior) -> None:
                    atomic_json(path,row)
                    if path.name=='receipt.json':
                        if fault=='target-publication': (prepared.inputs.root/'rust/target/late').mkdir()
                        if fault=='history-publication': (prior.history[0].root/'late').mkdir()
                        if fault=='receipt-publication': _=path.write_text('{}')
                        if fault=='directory-publication': (path.parent/'late').mkdir()
                        if fault=='mode-publication': (path.parent/'started.json').chmod(0o600)
                with lock.open('w') as lane,patch('generic_setup_bridge.LOCK',lock),patch.object(embedded,'check_clippy_endpoint'),patch.object(embedded,'qualify_wait',return_value=()),patch.object(embedded,'execute',side_effect=producer),patch.object(embedded,'atomic_json',side_effect=publish):
                    request=embedded.EmbeddedRequest(prepared,11,prior,unused_wait(prepared),home/'step11',lane.fileno())
                    with self.assertRaises(embedded.EmbeddedFailure) as caught: _=embedded.run_owned(request)
                    self.assertEqual(isinstance(caught.exception,embedded.EmbeddedOwnershipUnknown),fault in ('log','cache','settings'))

    def test_synthetic_history_is_not_real_admission(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home=Path(tmp).resolve();prepared,prior=fixture(home);lock=home/'lock'
            with lock.open('w') as lane,patch('generic_setup_bridge.LOCK',lock),patch.object(embedded,'qualify_wait',return_value=()),patch.object(embedded,'execute') as launch:
                request=embedded.EmbeddedRequest(prepared,10,prior,unused_wait(prepared),home/'step10',lane.fileno())
                with self.assertRaises(embedded.EmbeddedFailure): _=embedded.run_owned(request)
                launch.assert_not_called()

    def test_settled_publication_failure_keeps_both_exceptions(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home=Path(tmp).resolve();prepared,prior=fixture(home);lock=home/'lock'
            first=RuntimeError('publication');second=OSError('diagnostic');attempts=0
            def publish(path: Path,row: dict[str,Json]) -> None:
                nonlocal attempts
                if path.name=='receipt.json':
                    attempts+=1
                    raise first if attempts==1 else second
                atomic_json(path,row)
            with lock.open('w') as lane,patch('generic_setup_bridge.LOCK',lock),patch.object(embedded,'check_clippy_endpoint'),patch.object(embedded,'qualify_wait',return_value=()),patch.object(embedded,'atomic_json',side_effect=publish):
                request=embedded.EmbeddedRequest(prepared,11,prior,unused_wait(prepared),home/'step11',lane.fileno())
                with self.assertRaises(embedded.EmbeddedFailure) as caught: _=embedded.run_owned(request)
                self.assertNotIsInstance(caught.exception,embedded.EmbeddedOwnershipUnknown)
                self.assertIs(caught.exception.primary,first);self.assertIs(caught.exception.secondary,second)
                self.assertTrue((request.output/'process/process.json').is_file())

    def test_started_and_last_callback_do_not_rebase_evidence(self) -> None:
        for fault in ('started-content','started-directory','prelaunch-callback','producer-started','final-callback'):
            with self.subTest(fault=fault),tempfile.TemporaryDirectory() as tmp:
                home=Path(tmp).resolve();prepared,prior=fixture(home);lock=home/'lock';launched=False
                original=embedded.recheck
                def publish(path: Path,row: dict[str,Json],fault: str = fault) -> None:
                    atomic_json(path,row)
                    if path.name=='started.json':
                        if fault=='started-content': _=path.write_text('{}')
                        if fault=='started-directory': (path.parent/'extra-empty').mkdir()
                def boundary(request: embedded.EmbeddedRequest,state: GeneratedState,fault: str = fault, original: Callable[[embedded.EmbeddedRequest, GeneratedState], None] = original) -> None:
                    original(request,state)
                    if fault=='prelaunch-callback' and (request.output/'started.json').exists() and not (request.output/'process').exists():
                        _=(request.output/'started.json').write_text('{}')
                    if fault=='final-callback' and (request.output/'receipt.json').exists():
                        (request.output/'extra-empty').mkdir(exist_ok=True)
                def producer(spec: ProcessSpec,fault: str = fault) -> ProcessResult:
                    nonlocal launched
                    launched=True
                    result=execute(spec)
                    if fault=='producer-started': _=(spec.output_root.parent/'started.json').write_text('{}')
                    return result
                with lock.open('w') as lane,patch('generic_setup_bridge.LOCK',lock),patch.object(embedded,'check_clippy_endpoint'),patch.object(embedded,'qualify_wait',return_value=()),patch.object(embedded,'atomic_json',side_effect=publish),patch.object(embedded,'recheck',side_effect=boundary),patch.object(embedded,'execute',side_effect=producer):
                    request=embedded.EmbeddedRequest(prepared,11,prior,unused_wait(prepared),home/'step11',lane.fileno())
                    with self.assertRaises(embedded.EmbeddedFailure) as caught: _=embedded.run_owned(request)
                    self.assertEqual(launched,fault in ('producer-started','final-callback'))
                    self.assertEqual(isinstance(caught.exception,embedded.EmbeddedOwnershipUnknown),fault=='producer-started')
