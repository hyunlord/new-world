"""Fixed UI plan/policy regressions; synthetic inputs are not runtime qualification."""
import json
import unittest
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import generic_ui_fixed_run as fixed
import test_generic_ui_receipts as fixtures
from generic_fixture_run import Products
from generic_fixtures import ProbeProduct
from generic_gate import GateContractError, document
from generic_process import ProcessResult, ProcessSpec, environment_for, file_hash
from mux2_gate_inputs import Json, Node


def spec() -> ProcessSpec:
    return ProcessSpec(('/godot',), Path('/project'), Path('/godot'), 'a'*64,
                       Path('/project'), (), Path('/out'))


def log(*rows: tuple[int, str, int]) -> str:
    value: dict[str, Json] = {'total':len(rows), 'ticks':[r[0] for r in rows],
                             'kinds':[r[1] for r in rows], 'accepted':[r[2] for r in rows]}
    return json.dumps(value)


class FixedTests(unittest.TestCase):
    def test_tree_plan_retains_every_reset_and_bootstrap(self) -> None:
        # Given a variable bootstrap; when planning trees; then retain four engines.
        plan = fixed.make_plan('render.trees', spec(), 3)
        self.assertEqual([(i.reason,i.seed,i.terminal_tick) for i in plan.instances[0]],
                         [('bootstrap',20261010,3),('reset',0,1),('stress_reset',0,1),('reset',0,1)])
        self.assertEqual(len(plan.checkpoints),14)

    def test_toast_does_not_admit_dynamic_horizon(self) -> None:
        with self.assertRaisesRegex(GateContractError, 'toast.*zero'):
            _ = fixed.make_plan('toast', spec(), 1)

    def test_layout_has_no_native_worlds(self) -> None:
        plan = fixed.make_plan('marker.layout', spec(), 0)
        self.assertEqual((plan.instances,plan.checkpoints,plan.classification),
                         ((),(),'synthetic_renderer_only'))

    def test_ground_same_seed_resets_remain_distinct(self) -> None:
        plan = fixed.make_plan('render.ground', spec(), 0)
        self.assertEqual(len(plan.instances[0]),2)
        self.assertEqual(plan.checkpoints[-1].tick,1)
        self.assertTrue(plan.checkpoints[-1].releases_node)

    def test_reset_commands_preserve_factory_and_fixture_off(self) -> None:
        plan=fixed.make_plan('render.people',spec(),0)
        values=[log((0,'SetSteward',1))]*3 + [log((0,'SetSteward',1),(0,'SetSteward',1))]*3
        self.assertEqual(len(fixed.command_policy(plan,tuple(document(v) for v in values))),6)

    def test_rejected_or_additional_reset_command_fails(self) -> None:
        plan=fixed.make_plan('render.people',spec(),0)
        for bad in [log((0,'SetSteward',1),(0,'SetSteward',0)),
                    log((0,'SetSteward',1),(0,'SetSteward',1),(1,'Steward',1))]:
            values=[log((0,'SetSteward',1))]*3+[bad]*3
            with self.assertRaises(GateContractError):
                _=fixed.command_policy(plan,tuple(document(v) for v in values))

    def test_bootstrap_native_steward_is_preserved(self) -> None:
        plan=fixed.make_plan('render.ground',spec(),2)
        values=[log((0,'SetSteward',1)),log((0,'SetSteward',1),(1,'Steward',1)),
                log((0,'SetSteward',1))]+[log((0,'SetSteward',1),(0,'SetSteward',1))]*3
        self.assertEqual(len(fixed.command_policy(plan,tuple(document(v) for v in values))[1]),2)

    def test_toast_mock_chronicle_does_not_allow_native_commands(self) -> None:
        plan=fixed.make_plan('toast',spec(),0)
        with self.assertRaises(GateContractError):
            _=fixed.command_policy(plan,(document(log((0,'SetSteward',1))),document(log((0,'DebugKillAgent',1)))))

    def test_render_oracle_requires_original_completion_and_checks(self) -> None:
        fixed.domain('render.ground','MART2_GROUND PASS example\nMART2_GROUND_DONE failures=0\n','')
        with self.assertRaises(GateContractError):
            fixed.domain('render.ground','MART2_GROUND_DONE failures=0\n','')

    def test_render_oracle_rejects_script_error_even_with_done(self) -> None:
        with self.assertRaises(GateContractError):
            fixed.domain('render.people','MART2_PEOPLE PASS example\nMART2_PEOPLE_DONE failures=0\n','SCRIPT ERROR: bad')

    def test_layout_requires_actual_renderer(self) -> None:
        with self.assertRaisesRegex(GateContractError,'render'):
            fixed.domain('marker.layout','MUX21_LAYOUT_DONE {"checks":1,"cases":1,"failures":0,"display":"headless"}\n','')

    def test_toast_failed_or_duplicate_done_rejected(self) -> None:
        for output in ['MUX21_TOAST_DONE {"checks":1,"failures":1}\n',
                       'MUX21_TOAST_DONE {"checks":1,"failures":0}\n'*2]:
            with self.assertRaises(GateContractError):fixed.domain('toast',output,'')


    def test_explicit_real_home_preserves_fresh_application_settings(self) -> None:
        names=set(fixed.PINS)|{'project.godot','sim_bridge.gdextension','rust/target/debug/libsim_bridge.dylib','scenes/main.tscn'}
        inputs={name:fixed.PINS.get(name,'c'*64) for name in names}
        product=ProbeProduct('a'*40,Path('/godot'),'b'*64,Path('/project'),tuple(sorted(inputs.items())))
        def digest(path: Path) -> str:
            return 'b'*64 if path==product.executable else inputs[str(path.relative_to(product.input_root))]
        with patch.object(fixed,'file_hash',side_effect=digest):
            actual=fixed.process_spec(product,'marker.layout',Path('/evidence'))
        environment=dict(actual.environment)
        self.assertEqual(environment['HOME'],str(Path.home()))
        self.assertEqual(environment['WORLDSIM_TEST_SETTINGS_DIR'],'/evidence/settings')
        self.assertEqual(actual.fresh_directories,('settings',))
        self.assertEqual(set(environment),{'HOME','WORLDSIM_GATE_OBSERVER','WORLDSIM_GATE_INVOCATION_ID',
                                          'WORLDSIM_GATE_UI_OUT','WORLDSIM_TEST_SETTINGS_DIR','WORLDSIM_GIT_SHA'})

    def test_product_missing_authenticated_bridge_is_rejected(self) -> None:
        product=ProbeProduct('a'*40,Path('/godot'),'b'*64,Path('/project'),())
        with self.assertRaisesRegex(GateContractError,'inventory'):
            _=fixed.process_spec(product,'toast',Path('/out'))

    def test_unreviewed_source_pin_is_rejected_before_launch(self) -> None:
        names=set(fixed.PINS)|{'project.godot','sim_bridge.gdextension','rust/target/debug/libsim_bridge.dylib','scenes/main.tscn'}
        product=ProbeProduct('a'*40,Path('/godot'),'b'*64,Path('/project'),tuple((n,'0'*64) for n in sorted(names)))
        with self.assertRaisesRegex(GateContractError,'unreviewed'):
            _=fixed.process_spec(product,'toast',Path('/out'))

    def test_stress_engine_cannot_hide_command_in_return_colony(self) -> None:
        plan=fixed.make_plan('render.trees',spec(),0)
        logs: list[Node] = []
        for cp in plan.checkpoints:
            count=0 if cp.instance_count==3 else (1 if cp.instance_count==1 or cp.label=='after_colony_reset' else 2)
            logs.append(document(log(*((0,'SetSteward',1),)*count)))
        self.assertEqual(len(fixed.command_policy(plan,tuple(logs))),14)
        logs[9]=document(log((0,'SetSteward',1)))
        with self.assertRaises(GateContractError):_=fixed.command_policy(plan,tuple(logs))


class ReceiptTests(unittest.TestCase):
    def fixture(self) -> 'fixtures.UIReceiptTests':
        f=fixtures.UIReceiptTests(); f.setUp(); self.addCleanup(f.doCleanups)
        names=('tools/v8/mux21_toast_test.gd','tools/v8/gate_ui_observer.gd')
        for name in names:
            p=f.cwd/name;p.parent.mkdir(parents=True,exist_ok=True);_=p.write_text('synthetic')
        inputs=tuple((name,file_hash(f.cwd/name)) for name in names)
        argv=(str(f.spec.executable),'--path',str(f.cwd),'--headless','--script','res://'+names[0])
        environment=(('WORLDSIM_GATE_OBSERVER','1'),('WORLDSIM_GATE_INVOCATION_ID','toast:'+str(f.output)),
                     ('WORLDSIM_GATE_UI_OUT',str(f.output)),('WORLDSIM_TEST_SETTINGS_DIR',str(f.output/'settings')))
        f.spec=replace(f.spec,argv=argv,inputs=inputs,environment=environment,fresh_directories=('settings',),timeout_seconds=180)
        f.path=f.output/'toast-123.json'
        f.data.update(producer='toast',invocation_id='toast:'+str(f.output),script='res://'+names[0],
                      script_sha256=inputs[0][1],helper_sha256=inputs[1][1],argv=['--script','res://'+names[0]],user_args=[])
        for cp in fixtures.checkpoints(f.data):
            cp['clock']={'tick':0};cp['commands']={'total':1,'ticks':[0],'kinds':['SetSteward'],'accepted':[1]}
            instance=fixtures.obj(cp,'observation','instances',0)
            instance.update(seed='20261010',terminal_tick='0',initial_population=10,terminal_alive=10,
                            initial_agent_ids=[str(i) for i in range(10)],terminal_agent_ids=[str(i) for i in range(10)])
        _=(f.output/'stdout.log').write_text('MUX21_TOAST_DONE {"checks":12,"failures":0}\n')
        f.process.update(argv=list(argv),environment={k:v for k,v in environment_for(f.spec).items()},
                         fresh_directories=['settings'],timeout_seconds=180,timed_out=False,
                         expected_inputs=dict(inputs),before_inputs=dict(inputs),after_inputs=dict(inputs),
                         stdout_sha256=file_hash(f.output/'stdout.log'))
        return f

    def inspect(self,f: 'fixtures.UIReceiptTests') -> fixed.FixedEvidence:
        _=f.path.write_text(json.dumps(f.data));_=f.result.receipt.write_text(json.dumps(f.process))
        return fixed.inspect_output('toast',f.spec,f.result)

    def test_real_parser_composition_accepts_synthetic_bound_toast(self) -> None:
        f=self.fixture()
        self.assertEqual(self.inspect(f).instances[0].state.terminal_tick,0)

    def test_native_timeout_even_with_exit_zero_is_rejected(self) -> None:
        f=self.fixture();f.process['timed_out']=True
        with self.assertRaises(GateContractError):_=self.inspect(f)

    def test_omitted_bootstrap_is_rejected(self) -> None:
        f=self.fixture();_=fixtures.arr(f.data,'checkpoints').pop(0)
        with self.assertRaises(GateContractError):_=self.inspect(f)

    def test_wrong_process_input_inventory_is_rejected(self) -> None:
        f=self.fixture();f.process['before_inputs']={}
        with self.assertRaises(GateContractError):_=self.inspect(f)

    def test_pair_retains_knowledge_loss_failure(self) -> None:
        a=self.inspect(self.fixture());b=self.inspect(self.fixture())
        native=b.instances[0];changed=replace(native,state=replace(native.state,losses=((0,'tool',1),)))
        with self.assertRaises(GateContractError):fixed.compare(a,replace(b,instances=(changed,)))

    def test_unequal_bootstrap_horizon_explicitly_unresolved(self) -> None:
        a=self.inspect(self.fixture());b=self.inspect(self.fixture())
        native=b.instances[0];changed=replace(native,state=replace(native.state,terminal_tick=1))
        with self.assertRaisesRegex(GateContractError,'UNRESOLVED.*horizons'):
            fixed.compare(a,replace(b,instances=(changed,)))


class PairIntegrityTests(ReceiptTests):
    def test_second_child_cannot_rewrite_first_outputs_or_products(self) -> None:
        for target in ('stdout.log', 'process.json', 'sidecar.json', 'new-file', 'input', 'binary'):
            with self.subTest(target=target), TemporaryDirectory() as folder:
                root=Path(folder).resolve()
                products: list[ProbeProduct]=[]
                for arm in ('base','candidate'):
                    source=root/arm;source.mkdir()
                    _=(source/'input').write_bytes(b'input')
                    _=(source/'godot').write_bytes(b'godot')
                    products.append(ProbeProduct('a'*40,source/'godot',file_hash(source/'godot'),source,
                                                 (('input',file_hash(source/'input')),)))
                evidence=self.inspect(self.fixture())
                def planned(product: ProbeProduct, _producer: str, output: Path) -> ProcessSpec:
                    return ProcessSpec((str(product.executable),),product.input_root,product.executable,
                                       product.executable_sha256,product.input_root,product.input_files,output)
                frozen_products=tuple(products)
                def child(spec: ProcessSpec, products: tuple[ProbeProduct, ...] = frozen_products,
                          target: str = target, evidence: fixed.FixedEvidence = evidence) -> ProcessResult:
                    spec.output_root.mkdir()
                    for name in ('stdout.log','process.json','sidecar.json'):
                        _=(spec.output_root/name).write_bytes(b'original\r\n')
                    if spec.output_root.name == 'candidate':
                        changed=(products[0].input_root/('godot' if target=='binary' else 'input')) if target in ('binary','input') else spec.output_root.parent/'base'/target
                        _=changed.write_bytes(b'changed')
                    return replace(evidence.process,receipt=spec.output_root/'process.json')
                with patch.object(fixed,'process_spec',side_effect=planned), patch.object(fixed,'execute',side_effect=child), patch.object(fixed,'inspect_output',return_value=evidence):
                    result=fixed.run_pair(Products(*products),'toast',root/'evidence')
                self.assertEqual(result.status,'FAIL')
                self.assertIn('changed during pair',result.receipt.read_text())

    def test_second_successful_child_invalid_utf8_retains_failure(self) -> None:
        with TemporaryDirectory() as folder:
            root=Path(folder).resolve()
            products: list[ProbeProduct]=[]
            for arm in ('base','candidate'):
                source=root/arm;source.mkdir()
                _=(source/'input').write_bytes(b'input')
                _=(source/'godot').write_bytes(b'godot')
                products.append(ProbeProduct('a'*40,source/'godot',file_hash(source/'godot'),source,
                                             (('input',file_hash(source/'input')),)))
            evidence=self.inspect(self.fixture())
            original_inspect=fixed.inspect_output
            def planned(product: ProbeProduct, _producer: str, output: Path) -> ProcessSpec:
                return ProcessSpec((str(product.executable),),product.input_root,product.executable,
                                   product.executable_sha256,product.input_root,product.input_files,output)
            def child(spec: ProcessSpec) -> ProcessResult:
                spec.output_root.mkdir()
                _=(spec.output_root/'process.json').write_bytes(b'\xff')
                return replace(evidence.process,receipt=spec.output_root/'process.json',native_exit=0,status='PASS')
            def inspected(producer: str, spec: ProcessSpec, result: ProcessResult) -> fixed.FixedEvidence:
                return original_inspect(producer,spec,result) if spec.output_root.name=='candidate' else evidence
            with patch.object(fixed,'process_spec',side_effect=planned), patch.object(fixed,'execute',side_effect=child), patch.object(fixed,'inspect_output',side_effect=inspected):
                result=fixed.run_pair(Products(*products),'toast',root/'evidence')
            self.assertEqual(result.status,'FAIL')
            self.assertIn('decode',result.receipt.read_text())
            self.assertEqual((root/'evidence/candidate/process.json').read_bytes(),b'\xff')

    def test_shared_product_root_rejected(self) -> None:
        with TemporaryDirectory() as folder:
            source=Path(folder).resolve()
            product=ProbeProduct('a'*40,source/'godot','b'*64,source,())
            with self.assertRaisesRegex(GateContractError,'independent'):
                _=fixed.run_pair(Products(product,product),'toast',source.parent/(source.name+'-output'))


if __name__ == '__main__':
    _ = unittest.main()
