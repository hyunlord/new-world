"""Offline smoke admission regressions; no native qualification claim."""
import json
import unittest
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import generic_ui_smoke_run as smoke
import test_generic_ui_receipts as fixtures
from generic_fixture_run import Products
from generic_fixtures import ProbeProduct
from generic_gate import GateContractError
from generic_process import ProcessResult, ProcessSpec, environment_for, file_hash
from generic_ui_history import UIEvidence


class SmokeTests(unittest.TestCase):
    def fixture(self, producer: str = 'smoke.labels') -> fixtures.UIReceiptTests:
        f=fixtures.UIReceiptTests();f.setUp();self.addCleanup(f.doCleanups)
        files=(smoke.SCRIPTS[producer][6:],smoke.HELPER[6:],'tools/v8/smoke_base_counts.txt')
        for name in files:
            path=f.cwd/name;path.parent.mkdir(parents=True,exist_ok=True);_=path.write_text('synthetic')
        _=(f.cwd/files[2]).write_text('script_errors 0\nerrors 0\nlocale_warnings 0\nraw_keys 0\n')
        inputs=tuple((name,file_hash(f.cwd/name)) for name in files)
        product=ProbeProduct('a'*40,f.spec.executable,f.spec.executable_sha256,f.cwd,inputs)
        with patch.object(smoke,'check_product'):f.spec=smoke.process_spec(product,producer,f.output)
        f.path=f.output/f'{producer}-123.json'
        boot=producer=='smoke.boot'
        f.data['native_exit_code']=-1 if boot else 0
        f.data.update(producer=producer,invocation_id=producer+':'+str(f.output),script=smoke.SCRIPTS[producer],
                      script_sha256=inputs[0][1],helper_sha256=inputs[1][1],
                      argv=[] if boot else ['--script',smoke.SCRIPTS[producer]],
                      user_args=[] if boot else ['--frames','900'],completion='native_tree_exit' if boot else 'producer_finish')
        labels=('bootstrap','native_tree_exit') if boot else ('bootstrap','initial_frames_complete','terminal')
        ticks=(0,3) if boot else (0,2,3)
        f.data['checkpoints']=[f.checkpoint(i,label,tick) for i,(label,tick) in enumerate(zip(labels,ticks))]
        for cp,tick in zip(fixtures.checkpoints(f.data),ticks):
            cp['commands']={'total':1,'ticks':[0],'kinds':['SetSteward'],'accepted':[1]}
            native=fixtures.obj(cp,'observation','instances',0)
            native.update(seed='20261010',initial_population=10,terminal_alive=10,terminal_tick=str(tick),
                          initial_agent_ids=[str(i) for i in range(10)],terminal_agent_ids=[str(i) for i in range(10)])
        if not boot:fixtures.obj(f.data,'checkpoints',1)['context']={'requested_frames':900}
        _=(f.output/'stdout.log').write_text('' if boot else 'SMOKE_DONE frames=900 labels=592 raw_keys=0\n')
        f.process.update(argv=list(f.spec.argv),environment=dict(environment_for(f.spec)),fresh_directories=['settings'],
                         timeout_seconds=600,timed_out=False,expected_inputs=dict(inputs),before_inputs=dict(inputs),after_inputs=dict(inputs),
                         stdout_sha256=file_hash(f.output/'stdout.log'))
        return f

    def inspect(self,f: fixtures.UIReceiptTests,producer: str = 'smoke.labels') -> smoke.SmokeEvidence:
        _=f.path.write_text(json.dumps(f.data));_=f.result.receipt.write_text(json.dumps(f.process))
        return smoke.inspect(f.spec,f.result,producer)

    def test_original_labels_receipt(self) -> None:
        actual=self.inspect(self.fixture())
        self.assertEqual(actual.state.terminal_tick,3)
        self.assertEqual([cp.tick for cp in actual.evidence.plan.checkpoints],[0,2,3])

    def test_original_boot_receipt(self) -> None:
        self.assertEqual(self.inspect(self.fixture('smoke.boot'),'smoke.boot').state.founders,10)

    def test_original_900_frames_and_timeout_preserved(self) -> None:
        f=self.fixture();self.assertEqual(f.spec.argv[-3:],('--','--frames','900'))
        self.assertEqual(f.spec.timeout_seconds,600);self.assertEqual(f.spec.fresh_directories,('settings',))

    def test_extra_commands_rejected(self) -> None:
        f=self.fixture();fixtures.obj(f.data,'checkpoints',2)['commands']={'total':2,'ticks':[0,1],'kinds':['SetSteward','Build'],'accepted':[1,1]}
        with self.assertRaises(GateContractError):_=self.inspect(f)

    def test_missing_lifecycle_rejected(self) -> None:
        f=self.fixture();_=fixtures.arr(f.data,'checkpoints').pop(1)
        with self.assertRaises(GateContractError):_=self.inspect(f)

    def test_wrong_frames_context_rejected(self) -> None:
        f=self.fixture();fixtures.obj(f.data,'checkpoints',1,'context')['requested_frames']=10
        with self.assertRaises(GateContractError):_=self.inspect(f)

    def test_original_oracle(self) -> None:
        boot=self.inspect(self.fixture('smoke.boot'),'smoke.boot');labels=self.inspect(self.fixture())
        self.assertEqual(smoke.oracle(boot,labels),{'script_errors':0,'errors':0,'locale_warnings':0,'raw_keys':0})

    def test_shorter_missing_duplicate_or_raw_keys_fail(self) -> None:
        boot=self.inspect(self.fixture('smoke.boot'),'smoke.boot')
        for output in ('','SMOKE_DONE frames=10 labels=1 raw_keys=0\n','SMOKE_DONE frames=900 labels=2 raw_keys=1\n',
                       'SMOKE_DONE frames=900 labels=2 raw_keys=0\n'*2):
            f=self.fixture();_= (f.output/'stdout.log').write_text(output)
            f.process['stdout_sha256']=file_hash(f.output/'stdout.log');labels=self.inspect(f)
            with self.assertRaises(GateContractError):_=smoke.oracle(boot,labels)

    def test_audio_exclusion_matches_original(self) -> None:
        boot=self.inspect(self.fixture('smoke.boot'),'smoke.boot');f=self.fixture()
        with (f.output/'stderr.log').open('w') as stream:_=stream.write('ERROR: audio unavailable\n')
        f.process['stderr_sha256']=file_hash(f.output/'stderr.log')
        self.assertEqual(smoke.oracle(boot,self.inspect(f))['errors'],0)

    def test_companion_request_uses_actual_horizon_and_hash(self) -> None:
        original=self.inspect(self.fixture()).evidence
        product=ProbeProduct('a'*40,Path('/godot'),'b'*64,Path('/project'),())
        with patch.object(smoke,'check_product'):
            spec=smoke.process_spec(product,'smoke.companion',Path('/out'),original)
        self.assertEqual(spec.argv[-6:],('--ticks','3','--original-producer','smoke.labels','--original-sidecar-sha256',original.sidecar_sha256))
        changed=replace(original,plan=replace(original.plan,instances=((replace(original.plan.instances[0][0],terminal_tick=200001),),)))
        with patch.object(smoke,'check_product'),self.assertRaises(GateContractError):
            _=smoke.process_spec(product,'smoke.companion',Path('/out'),changed)

    def exercise_pair_boundary(self, mode: str) -> None:
        evidence=self.inspect(self.fixture())
        for mutate in ((False,True) if mode=="mutation" else (False,)):
            with TemporaryDirectory() as folder:
                root=Path(folder).resolve();products: list[ProbeProduct]=[]
                for name in ('base-source','candidate-source'):
                    source=root/name;source.mkdir();products.append(ProbeProduct('a'*40,source/'godot','b'*64,source,()))
                calls: list[str]=[]
                def planned(product: ProbeProduct, _producer: str, output: Path, _original: object) -> ProcessSpec:
                    return ProcessSpec((str(product.executable),),product.input_root,product.executable,product.executable_sha256,
                                       product.input_root,(),output)
                def child(spec: ProcessSpec, calls: list[str] = calls, mutate: bool = mutate) -> ProcessResult:
                    spec.output_root.mkdir();_=(spec.output_root/'process.json').write_bytes(b'original');calls.append(spec.output_root.name)
                    if mutate and len(calls)==2:_=(spec.output_root.parent/'base-boot/process.json').write_bytes(b'changed')
                    return ProcessResult(spec.output_root/'process.json','PASS',0,123)
                def inspected(spec: ProcessSpec, _result: ProcessResult, _producer: str, original: object) -> smoke.SmokeEvidence:
                    horizon=4 if mode!='equal' and spec.output_root.name.startswith('candidate') else 3
                    if isinstance(original,type(evidence.evidence)):horizon=original.plan.instances[0][0].terminal_tick
                    native=replace(evidence.state,terminal_tick=horizon)
                    if spec.output_root.name.startswith('candidate') and spec.output_root.name.endswith('-matched'):
                        if mode=='survival':native=replace(native,alive=7)
                        if mode=='knowledge':native=replace(native,knowledge_enabled=True,losses=((1,'fire',None),))
                    plan=replace(evidence.evidence.plan,process=spec,instances=((replace(evidence.evidence.plan.instances[0][0],terminal_tick=horizon),),))
                    return replace(evidence,state=native,evidence=replace(evidence.evidence,plan=plan))
                def qualify(_actual: UIEvidence, _companion: UIEvidence) -> None:
                    if mode=='own_failure':raise GateContractError('INVALID_EVIDENCE','/companion','injected own-companion qualification failure')
                with patch.object(smoke,'check_product'),patch.object(smoke,'process_spec',side_effect=planned),patch.object(smoke,'execute',side_effect=child),patch.object(smoke,'inspect',side_effect=inspected),patch.object(smoke,'oracle',return_value={}),patch.object(smoke,'validate_smoke_companion',side_effect=qualify):
                    result=smoke.run_pair(Products(*products),root/'output')
                self.assertEqual(len(calls),8 if mode=='equal' else (10 if mode in ('survival','knowledge') else 12))
                self.assertEqual(result.status,'FAIL' if mutate or mode in ('own_failure','survival','knowledge') else 'PASS')
                if mode=='own_failure':self.assertIn('injected own-companion qualification failure',result.receipt.read_text())
                if mode in ('survival','knowledge'):
                    self.assertIn('safety:',result.receipt.read_text())
                    self.assertIn('candidate-boot-matched',calls)
                if mutate:self.assertIn('artifacts changed',result.receipt.read_text())

    def test_pair_runs_cross_horizons_and_rechecks_earlier_artifacts(self) -> None:
        self.exercise_pair_boundary('mutation')

    def test_own_companion_failure_cannot_be_overridden(self) -> None:
        self.exercise_pair_boundary('own_failure')

    def test_cross_matched_candidate_safety_regressions_fail(self) -> None:
        for mode in ('survival','knowledge'):
            with self.subTest(mode=mode):self.exercise_pair_boundary(mode)

    def test_equal_horizons_use_eight_children(self) -> None:
        self.exercise_pair_boundary('equal')

    def test_malformed_native_sidecar_retains_final_failure(self) -> None:
        with TemporaryDirectory() as folder:
            root=Path(folder).resolve();products: list[ProbeProduct]=[]
            for name in ('base','candidate'):
                source=root/name;source.mkdir();products.append(ProbeProduct('a'*40,source/'godot','b'*64,source,()))
            def child(spec: ProcessSpec) -> ProcessResult:
                spec.output_root.mkdir();_=(spec.output_root/'process.json').write_text('{}')
                producer='smoke.boot' if spec.output_root.name.endswith('boot') else 'smoke.labels'
                _=(spec.output_root/(producer+'-123.json')).write_bytes(b'\xff')
                return ProcessResult(spec.output_root/'process.json','PASS',0,123)
            with patch.object(smoke,'check_product'),patch.object(smoke,'execute',side_effect=child):
                result=smoke.run_pair(Products(*products),root/'output')
            self.assertEqual(result.status,'FAIL');self.assertIn('decode',result.receipt.read_text())

if __name__=='__main__':
    _=unittest.main()
