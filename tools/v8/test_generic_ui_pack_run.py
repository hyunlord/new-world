"""Synthetic selected-pack receipt tests; not native, rendered or Rust-suite proof."""
import json
import unittest
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import generic_ui_pack_run as pack
import test_generic_ui_receipts as fixtures
from generic_fixture_run import Products
from generic_fixtures import ProbeProduct
from generic_gate import GateContractError
from generic_process import ProcessResult, ProcessSpec, environment_for, file_hash
from mux2_gate_inputs import Json


def spec() -> ProcessSpec:
    return ProcessSpec(('/godot',),Path('/source'),Path('/godot'),'a'*64,Path('/source'),((pack.PACK,'b'*64),),Path('/out'))


class PackTests(unittest.TestCase):
    def fixture(self) -> fixtures.UIReceiptTests:
        f=fixtures.UIReceiptTests();f.setUp();self.addCleanup(f.doCleanups)
        files=(pack.SCRIPT,pack.HELPER[6:],pack.PACK)
        for name in files:
            path=f.cwd/name;path.parent.mkdir(parents=True,exist_ok=True);_=path.write_text('synthetic')
        inputs=tuple((name,file_hash(f.cwd/name)) for name in files)
        argv=(str(f.spec.executable),'--headless','--path',str(f.cwd),'--script','res://'+pack.SCRIPT,'--',str(f.output/'ui'))
        env=(('HOME',str(Path.home())),('WORLDSIM_CONTENT_PACK',str(f.cwd/pack.PACK)),('WORLDSIM_GATE_OBSERVER','1'),
             ('WORLDSIM_GATE_INVOCATION_ID','pack.ui:'+str(f.output)),('WORLDSIM_GATE_UI_OUT',str(f.output)),
             ('WORLDSIM_TEST_SETTINGS_DIR',str(f.output/'settings')))
        f.spec=replace(f.spec,argv=argv,inputs=inputs,environment=env,fresh_directories=('settings',),timeout_seconds=600)
        f.path=f.output/'pack.ui-123.json'
        f.data.update(producer='pack.ui',invocation_id='pack.ui:'+str(f.output),script='res://'+pack.SCRIPT,
                      script_sha256=inputs[0][1],helper_sha256=inputs[1][1],argv=['--script','res://'+pack.SCRIPT],user_args=[str(f.output/'ui')])
        labels=('bootstrap','paused_selected_pack','before_explicit_tick','after_explicit_tick','terminal')
        ticks=(0,2,2,3,3)
        f.data['checkpoints']=[f.checkpoint(i,label,tick) for i,(label,tick) in enumerate(zip(labels,ticks))]
        for cp,tick in zip(fixtures.checkpoints(f.data),ticks):
            cp['commands']={'total':1,'ticks':[0],'kinds':['SetSteward'],'accepted':[1]}
            native=fixtures.obj(cp,'observation','instances',0)
            native.update(seed='20261010',initial_population=10,terminal_alive=10,terminal_tick=str(tick),
                          initial_agent_ids=[str(i) for i in range(10)],terminal_agent_ids=[str(i) for i in range(10)])
        fixtures.obj(f.data,'checkpoints',1)['context']={'manifest_path':str(f.cwd/pack.PACK),'manifest_sha256':inputs[2][1],
             'fixture_classification':'ordinary_selected_pack','synthetic_DTO_checks':'presentation_validation_only'}
        (f.output/'ui').mkdir()
        checks: list[Json]=[{'check':'synthetic check '+str(i),'passed':True} for i in range(20)]
        checks.append({'check':'rendered capture','status':'NOT_RUN_HEADLESS'})
        data: dict[str,Json]={'checks':checks,'failures':0,'selected_pack':str(f.cwd/pack.PACK),'rendered':False}
        _=(f.output/'ui/data-only-ext-results.json').write_text(json.dumps(data))
        _=(f.output/'stdout.log').write_text('DATA_ONLY_EXT_PASS synthetic\n')
        f.process.update(argv=list(argv),environment={k:v for k,v in environment_for(f.spec).items()},fresh_directories=['settings'],
                         timeout_seconds=600,timed_out=False,expected_inputs=dict(inputs),before_inputs=dict(inputs),after_inputs=dict(inputs),
                         stdout_sha256=file_hash(f.output/'stdout.log'))
        return f

    def inspect(self, f: fixtures.UIReceiptTests) -> pack.PackEvidence:
        _=f.path.write_text(json.dumps(f.data));_=f.result.receipt.write_text(json.dumps(f.process))
        return pack.inspect(f.spec,f.result)

    def test_continuous_world_plan_has_actual_prefix_not_assumed_tick1(self) -> None:
        p=pack.plan(spec(),6)
        self.assertEqual([cp.tick for cp in p.checkpoints],[0,6,6,7,7])
        self.assertEqual(len(p.instances[0]),1)
        self.assertEqual(p.instances[0][0].reason,'bootstrap')
        self.assertTrue(p.checkpoints[-1].releases_node)

    def test_actual_parser_composition_preserves_pack_identity(self) -> None:
        value=self.inspect(self.fixture())
        self.assertEqual((value.prefix.state.terminal_tick,value.terminal.state.terminal_tick),(2,3))

    def test_selected_manifest_hash_and_path_are_bound(self) -> None:
        for key,value in (('manifest_sha256','0'*64),('manifest_path','/foreign/pack.ron')):
            f=self.fixture();fixtures.obj(f.data,'checkpoints',1,'context')[key]=value
            with self.assertRaises(GateContractError):_=self.inspect(f)

    def test_fake_negative_dto_world_or_omitted_bootstrap_rejected(self) -> None:
        f=self.fixture();_=fixtures.arr(f.data,'checkpoints').pop(0)
        with self.assertRaises(GateContractError):_=self.inspect(f)
        f=self.fixture();fixtures.obj(f.data,'checkpoints',4,'observation','instances',0)['reason']='reset'
        with self.assertRaises(GateContractError):_=self.inspect(f)

    def test_minimap_wait_tick_drift_rejected(self) -> None:
        f=self.fixture();fixtures.obj(f.data,'checkpoints',4)['clock']={'tick':4}
        with self.assertRaises(GateContractError):_=self.inspect(f)

    def test_unrequested_setup_command_rejected(self) -> None:
        f=self.fixture();fixtures.obj(f.data,'checkpoints',2)['commands']={'total':2,'ticks':[0,2],'kinds':['SetSteward','Build'],'accepted':[1,1]}
        with self.assertRaises(GateContractError):_=self.inspect(f)

    def test_intrinsic_steward_commands_remain_accepted(self) -> None:
        f=self.fixture()
        for cp in fixtures.checkpoints(f.data)[1:]:
            cp['commands']={'total':2,'ticks':[0,1],'kinds':['SetSteward','Steward'],'accepted':[1,0]}
        self.assertEqual(self.inspect(f).terminal.state.alive,10)

    def test_presentation_success_does_not_waive_native_safety(self) -> None:
        value=self.inspect(self.fixture())
        changed=replace(value.terminal,state=replace(value.terminal.state,alive=7))
        with self.assertRaises(GateContractError):pack.compare(value,replace(value,terminal=changed))

    def test_unequal_actual_horizons_require_selected_pack_companion(self) -> None:
        value=self.inspect(self.fixture());changed=replace(value.prefix,state=replace(value.prefix.state,terminal_tick=3))
        with self.assertRaisesRegex(GateContractError,'UNRESOLVED'):pack.compare(value,replace(value,prefix=changed))

    def test_domain_retains_headless_and_minimum_original_check_contract(self) -> None:
        f=self.fixture();path=f.output/'ui/data-only-ext-results.json'
        data: dict[str,Json]={'checks':[],'failures':0,'selected_pack':str(f.cwd/pack.PACK),'rendered':False}
        _=path.write_text(json.dumps(data))
        with self.assertRaisesRegex(GateContractError,'assertion coverage'):_=self.inspect(f)

    def test_pack_reference_files_cannot_be_omitted_or_changed(self) -> None:
        names=set(pack.PINS)|{'project.godot','sim_bridge.gdextension','rust/target/debug/libsim_bridge.dylib','scenes/main.tscn'}
        inputs={name:pack.PINS.get(name,'c'*64) for name in names}
        novel=next(name for name in pack.PINS if name.endswith('items.ron'))
        for omitted in (True,False):
            changed=dict(inputs)
            if omitted:del changed[novel]
            else:changed[novel]='0'*64
            product=ProbeProduct('a'*40,Path('/godot'),'b'*64,Path('/source'),tuple(changed.items()))
            with self.assertRaises(GateContractError):pack.check_product(product)

    def test_original_headless_selected_pack_argv_and_fresh_own_output(self) -> None:
        product=ProbeProduct('a'*40,Path('/godot'),'b'*64,Path('/source'),())
        with patch.object(pack,'check_product'):p=pack.process_spec(product,Path('/out'))
        self.assertEqual(p.argv[-2:],('--','/out/ui'))
        self.assertIn('--headless',p.argv);self.assertEqual(p.timeout_seconds,600)
        self.assertEqual(dict(p.environment)['WORLDSIM_CONTENT_PACK'],'/source/'+pack.PACK)
        self.assertEqual(dict(p.environment)['HOME'],str(Path.home()))

    def test_second_child_cannot_rewrite_first_evidence(self) -> None:
        value=self.inspect(self.fixture())
        with TemporaryDirectory() as directory:
            root=Path(directory).resolve();products: list[ProbeProduct]=[]
            for name in ('base-source','candidate-source'):
                source=root/name;source.mkdir();products.append(ProbeProduct('a'*40,source/'godot','b'*64,source,()))
            def planned(product: ProbeProduct, output: Path) -> ProcessSpec:
                return ProcessSpec((str(product.executable),),product.input_root,product.executable,product.executable_sha256,
                                   product.input_root,(),output)
            def child(p: ProcessSpec) -> ProcessResult:
                p.output_root.mkdir();_=(p.output_root/'process.json').write_bytes(b'original')
                if p.output_root.name=='candidate':_=(p.output_root.parent/'base/process.json').write_bytes(b'changed')
                return ProcessResult(p.output_root/'process.json','PASS',0,123)
            with patch.object(pack,'process_spec',side_effect=planned),patch.object(pack,'check_product'),patch.object(pack,'execute',side_effect=child),patch.object(pack,'inspect',return_value=value):
                result=pack.run_pair(Products(*products),root/'out')
            self.assertEqual(result.status,'FAIL');self.assertIn('artifacts changed',result.receipt.read_text())


if __name__=='__main__':
    _=unittest.main()
