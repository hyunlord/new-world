"""Synthetic Why policy/receipt regressions, not native or visual qualification."""
import json
import unittest
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import generic_ui_why_run as why
import test_generic_ui_receipts as fixtures
from generic_fixture_run import Products
from generic_fixtures import ProbeProduct
from generic_gate import GateContractError
from generic_process import ProcessResult, ProcessSpec, environment_for, file_hash
from mux2_gate_inputs import Json


def spec() -> ProcessSpec:
    return ProcessSpec(('/godot',),Path('/project'),Path('/godot'),'a'*64,Path('/project'),(),Path('/out'))


class WhyTests(unittest.TestCase):
    def fixture(self) -> fixtures.UIReceiptTests:
        f=fixtures.UIReceiptTests();f.setUp();self.addCleanup(f.doCleanups)
        files=(why.PARENT,why.HELPER[6:])
        for name in files:
            path=f.cwd/name;path.parent.mkdir(parents=True,exist_ok=True);_=path.write_text('synthetic')
        inputs=tuple((name,file_hash(f.cwd/name)) for name in files)
        argv=(str(f.spec.executable),'--path',str(f.cwd),'--resolution','1920x1080','--script','res://'+why.PARENT)
        env=(('HOME',str(Path.home())),('WORLDSIM_GATE_OBSERVER','1'),
             ('WORLDSIM_GATE_INVOCATION_ID','why.base:'+str(f.output)),('WORLDSIM_GATE_UI_OUT',str(f.output)),
             ('WORLDSIM_TEST_SETTINGS_DIR',str(f.output/'settings')))
        f.spec=replace(f.spec,argv=argv,inputs=inputs,environment=env,fresh_directories=('settings',),timeout_seconds=180)
        f.path=f.output/'why-123.json'
        f.data.update(producer='why',invocation_id='why.base:'+str(f.output),script='res://'+why.PARENT,
                      script_sha256=inputs[0][1],helper_sha256=inputs[1][1],argv=['--script','res://'+why.PARENT],user_args=[])
        labels=('bootstrap','ordinary_prefix','before_intervention','intervention_queued','after_intervention_tick','terminal')
        ticks=(0,2,2,2,3,3)
        f.data['checkpoints']=[f.checkpoint(i,label,tick) for i,(label,tick) in enumerate(zip(labels,ticks))]
        for cp,tick in zip(fixtures.checkpoints(f.data),ticks):
            cp['commands']={'total':1,'ticks':[0],'kinds':['SetSteward'],'accepted':[1]}
            native=fixtures.obj(cp,'observation','instances',0)
            native.update(seed='20261010',initial_population=10,terminal_alive=10,terminal_tick=str(tick),
                          initial_agent_ids=[str(i) for i in range(10)],terminal_agent_ids=[str(i) for i in range(10)])
        fixtures.obj(f.data,'checkpoints',2)['context']={'method':'on_building_placed','args':[96,96,1]}
        fixtures.obj(f.data,'checkpoints',3)['context']={'method':'on_building_placed','args':[96,96,1],'accepted':True}
        output=''.join('MUX2_WHY_PASS check'+str(i)+'\n' for i in range(53))
        output+='MUX2_WHY_OUTPUT '+str(f.output/'why')+'\nMUX2_WHY_DONE failures=0 checks=53\n'
        _=(f.output/'stdout.log').write_text(output)
        for folder,names in (('why',('why-1.0.png','why-1.25.png')),('placement',('why-placement-1.0.png','why-placement-1.25.png'))):
            (f.output/folder).mkdir()
            for name in names:_=(f.output/folder/name).write_bytes(b'\x89PNG\r\n\x1a\nsynthetic-not-real-image')
        f.process.update(argv=list(argv),environment={k:v for k,v in environment_for(f.spec).items()},fresh_directories=['settings'],
                         timeout_seconds=180,timed_out=False,expected_inputs=dict(inputs),before_inputs=dict(inputs),after_inputs=dict(inputs),
                         stdout_sha256=file_hash(f.output/'stdout.log'))
        return f

    def inspect(self, fixture: fixtures.UIReceiptTests) -> why.WhyEvidence:
        _=fixture.path.write_text(json.dumps(fixture.data));_=fixture.result.receipt.write_text(json.dumps(fixture.process))
        return why.inspect(fixture.spec,fixture.result,'base')

    def test_exact_plan_keeps_ordinary_prefix_and_direct_event(self) -> None:
        p=why.plan(spec(),'ko',7,(96,95,1))
        self.assertEqual([cp.tick for cp in p.checkpoints],[0,7,7,7,8,8])
        self.assertEqual(p.script,'res://'+why.PARENT)
        self.assertEqual(len(p.instances[0]),1)
        self.assertTrue(p.checkpoints[-1].releases_node)

    def test_actual_receipt_composition(self) -> None:
        result=self.inspect(self.fixture())
        self.assertEqual((result.prefix.state.terminal_tick,result.terminal.state.terminal_tick,result.args),(2,3,(96,96,1)))

    def test_locale_invocation_binds_subclass_but_parent_sidecar(self) -> None:
        f=self.fixture()
        path=f.cwd/why.LOCALE;_=path.write_text('synthetic locale subclass')
        inputs=f.spec.inputs+((why.LOCALE,file_hash(path)),)
        argv=(*f.spec.argv[:-1],'res://'+why.LOCALE,'--','--locale=en')
        env=tuple((key,'why.en:'+str(f.output) if key=='WORLDSIM_GATE_INVOCATION_ID' else value) for key,value in f.spec.environment)
        f.spec=replace(f.spec,argv=argv,inputs=inputs,environment=env)
        f.data.update(invocation_id='why.en:'+str(f.output),argv=['--script','res://'+why.LOCALE],user_args=['--locale=en'])
        stdout='MUX21_WHY_LOCALE en\n'+''.join('MUX2_WHY_PASS check'+str(i)+'\n' for i in range(64))
        stdout+='MUX2_WHY_OUTPUT '+str(f.output/'why')+'\nMUX2_WHY_DONE failures=0 checks=64\n'
        _=(f.output/'stdout.log').write_text(stdout)
        _=(f.output/'why/why-1.5.png').write_bytes(b'\x89PNG\r\n\x1a\nsynthetic')
        f.process.update(argv=list(argv),environment={k:v for k,v in environment_for(f.spec).items()},expected_inputs=dict(inputs),
                         before_inputs=dict(inputs),after_inputs=dict(inputs),stdout_sha256=file_hash(f.output/'stdout.log'))
        _=f.path.write_text(json.dumps(f.data));_=f.result.receipt.write_text(json.dumps(f.process))
        actual=why.inspect(f.spec,f.result,'en')
        self.assertEqual(actual.evidence.plan.script,'res://'+why.PARENT)

    def test_omitted_bootstrap_and_wrong_horizon_rejected(self) -> None:
        for mutation in ('omit','extra_tick'):
            f=self.fixture()
            if mutation=='omit':_=fixtures.arr(f.data,'checkpoints').pop(0)
            else:fixtures.obj(f.data,'checkpoints',5)['clock']={'tick':4}
            with self.assertRaises(GateContractError):_=self.inspect(f)

    def test_intervention_cannot_be_omitted_or_rejected(self) -> None:
        mutations: tuple[tuple[str,Json],...]=(('accepted',False),('args',[95,96,1]),('method','cmd_build'))
        for field,value in mutations:
            f=self.fixture();fixtures.obj(f.data,'checkpoints',3,'context')[field]=value
            with self.assertRaises(GateContractError):_=self.inspect(f)

    def test_extra_ui_command_rejected_even_with_positive_oracle(self) -> None:
        f=self.fixture()
        fixtures.obj(f.data,'checkpoints',4)['commands']={'total':2,'ticks':[0,2],'kinds':['SetSteward','Build'],'accepted':[1,1]}
        with self.assertRaises(GateContractError):_=self.inspect(f)

    def test_native_steward_records_are_not_erased(self) -> None:
        f=self.fixture()
        for cp in fixtures.checkpoints(f.data)[1:]:
            cp['commands']={'total':2,'ticks':[0,1],'kinds':['SetSteward','Steward'],'accepted':[1,0]}
        self.assertEqual(self.inspect(f).prefix.state.terminal_tick,2)

    def test_foreign_screenshot_or_missing_placement_rejected(self) -> None:
        f=self.fixture();(f.output/'placement/why-placement-1.0.png').unlink()
        with self.assertRaises((GateContractError,OSError)):_=self.inspect(f)

    def test_runtime_error_not_overridden_by_complete_checks(self) -> None:
        f=self.fixture();_=(f.output/'stderr.log').write_text('ERROR: shader\n')
        f.process['stderr_sha256']=file_hash(f.output/'stderr.log')
        with self.assertRaisesRegex(GateContractError,'runtime error'):_=self.inspect(f)

    def test_horizon_and_event_input_mismatch_require_companion(self) -> None:
        evidence=self.inspect(self.fixture())
        changed=replace(evidence.prefix,state=replace(evidence.prefix.state,terminal_tick=3))
        for candidate in (replace(evidence,prefix=changed),replace(evidence,args=(95,96,1))):
            with self.assertRaisesRegex(GateContractError,'UNRESOLVED'):why.compare(evidence,candidate)

    def test_visual_pass_cannot_waive_ordinary_prefix_safety(self) -> None:
        evidence=self.inspect(self.fixture())
        changed=replace(evidence.prefix,state=replace(evidence.prefix.state,alive=7))
        with self.assertRaises(GateContractError):why.compare(evidence,replace(evidence,prefix=changed))

    def test_locale_original_argv_and_fresh_paths(self) -> None:
        product=ProbeProduct('a'*40,Path('/godot'),'b'*64,Path('/project'),())
        with patch.object(why,'check_product'):
            value=why.process_spec(product,'en',Path('/out'))
        self.assertEqual(value.argv[-4:],('--script','res://'+why.LOCALE,'--','--locale=en'))
        self.assertNotIn('--headless',value.argv)
        self.assertEqual(value.timeout_seconds,180)
        self.assertEqual(dict(value.environment)['WORLDSIM_TEST_SETTINGS_DIR'],'/out/settings')

    def test_second_child_mutation_retains_fail(self) -> None:
        evidence=self.inspect(self.fixture())
        with TemporaryDirectory() as folder:
            root=Path(folder).resolve();products: list[ProbeProduct]=[]
            for name in ('base-source','candidate-source'):
                source=root/name;source.mkdir();products.append(ProbeProduct('a'*40,source/'godot','b'*64,source,()))
            def planned(product: ProbeProduct, _variant: str, output: Path) -> ProcessSpec:
                return ProcessSpec((str(product.executable),),product.input_root,product.executable,product.executable_sha256,
                                   product.input_root,(),output)
            def child(specification: ProcessSpec) -> ProcessResult:
                specification.output_root.mkdir();_= (specification.output_root/'process.json').write_bytes(b'original')
                if specification.output_root.name=='candidate':
                    _=(specification.output_root.parent/'base/process.json').write_bytes(b'changed')
                return ProcessResult(specification.output_root/'process.json','PASS',0,123)
            with patch.object(why,'process_spec',side_effect=planned),patch.object(why,'check_product'),patch.object(why,'execute',side_effect=child),patch.object(why,'inspect',return_value=evidence):
                result=why.run_pair(Products(*products),'base',root/'output')
            self.assertEqual(result.status,'FAIL')
            self.assertIn('artifacts changed',result.receipt.read_text())


if __name__=='__main__':
    _=unittest.main()
