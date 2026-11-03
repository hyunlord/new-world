"""Private process fixtures; no native Godot qualification."""
import json
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import generic_ui_smoke_run as smoke
from generic_fixture_run import Products
from generic_fixtures import ProbeProduct
from generic_process import ProcessResult, ProcessSpec, atomic_json, file_hash


class RetentionTests(unittest.TestCase):
    def test_unreturned_first_child_stops_pair_and_preserves_original(self) -> None:
        with TemporaryDirectory() as temporary:
            home=Path(temporary).resolve();a=home/'a';b=home/'b';a.mkdir();b.mkdir()
            executable=Path(sys.executable).resolve();digest=file_hash(executable)
            products=Products(*(ProbeProduct('a'*40,executable,digest,root,()) for root in (a,b)))
            failure=OSError('unreturned');started: list[ProcessSpec]=[];returned: list[ProcessResult]=[]
            def retain(_spec: ProcessSpec,result: ProcessResult,returned: list[ProcessResult]=returned) -> None:returned.append(result)
            with patch.object(smoke,'check_product'),patch.object(smoke,'execute',side_effect=failure) as execute, \
                 self.assertRaises(RuntimeError) as caught:
                _=smoke.run_pair(products,home/'output',before=started.append,retain=retain)
            self.assertIs(caught.exception.__cause__,failure)
            self.assertEqual(execute.call_count,1);self.assertEqual(len(started),1);self.assertEqual(returned,[])

    def test_actual_return_is_retained_before_semantics_and_callback_failure_stops(self) -> None:
        from generic_gate import GateContractError
        from generic_ui_history import UIEvidence
        with TemporaryDirectory() as temporary:
            home=Path(temporary).resolve();a=home/'a';b=home/'b';a.mkdir();b.mkdir()
            executable=Path(sys.executable).resolve();digest=file_hash(executable)
            products=Products(*(ProbeProduct('a'*40,executable,digest,root,()) for root in (a,b)))
            returned: list[tuple[ProcessSpec,ProcessResult]]=[]
            def spec(product: ProbeProduct,_kind: str,output: Path,_original: UIEvidence | None) -> ProcessSpec:
                return ProcessSpec((str(executable),'-c','print("retained")'),product.input_root,executable,digest,
                                   product.input_root,(),output,timeout_seconds=600)
            def before(_spec: ProcessSpec) -> None:pass
            def retain(spec: ProcessSpec,result: ProcessResult) -> None:
                returned.append((spec,result))
                raise GateContractError('INVALID_EVIDENCE','/fixture','reject after actual wait')
            with patch.object(smoke,'check_product'),patch.object(smoke,'process_spec',side_effect=spec), \
                 patch.object(smoke,'inspect') as inspect,self.assertRaises(RuntimeError):
                _=smoke.run_pair(products,home/'output',before=before,retain=retain)
            self.assertEqual(len(returned),1);self.assertEqual(inspect.call_count,0)
            self.assertEqual(returned[0][1].native_exit,0)
            self.assertIsNotNone(returned[0][1].pid)
            self.assertEqual((returned[0][0].output_root/'stdout.log').read_text(),'retained\n')

    def test_prelaunch_callback_rejection_does_not_launch(self) -> None:
        from generic_gate import GateContractError
        with TemporaryDirectory() as temporary:
            home=Path(temporary).resolve();a=home/'a';b=home/'b';a.mkdir();b.mkdir()
            executable=Path(sys.executable).resolve();digest=file_hash(executable)
            products=Products(*(ProbeProduct('a'*40,executable,digest,root,()) for root in (a,b)))
            def before(_spec: ProcessSpec) -> None:
                raise GateContractError('INVALID_EVIDENCE','/fixture','prelaunch rejected')
            def retain(_spec: ProcessSpec,_result: ProcessResult) -> None:pass
            with patch.object(smoke,'check_product'),patch.object(smoke,'execute') as execute,self.assertRaises(RuntimeError):
                _=smoke.run_pair(products,home/'output',before=before,retain=retain)
            self.assertEqual(execute.call_count,0)


class CompositionTests(unittest.TestCase):
    def test_actual_auxiliary_and_candidate_chain_then_original_smoke_calls(self) -> None:
        self.exercise(False)

    def test_final_paired_callback_cannot_change_selector(self) -> None:
        self.exercise(True)

    def exercise(self, selector_attack: bool) -> None:
        import fcntl
        import os
        from contextlib import ExitStack
        from dataclasses import replace
        from types import SimpleNamespace
        from typing import Literal

        import gate_oracle_child as child
        import generic_cli_execute as cli
        import generic_cli_godot_products as godot
        import generic_full_gate as gate
        import generic_setup_clippy as clippy
        import generic_static_run as static
        import generic_workspace_endpoints as endpoints
        import generic_workspace_run as workspace
        from generic_gate import GateContractError, document
        from generic_process import atomic_json, execute
        from generic_setup_bridge import CargoWaitEvidence
        from generic_setup_prepare import Tool
        from test_generic_cli_godot_products import GodotTests
        from test_generic_setup_bridge import unused_wait
        from test_generic_ui_smoke_run import SmokeTests

        fixture=GodotTests();self.addCleanup(fixture.doCleanups)
        smoke_fixture=SmokeTests();self.addCleanup(smoke_fixture.doCleanups)
        proof=smoke_fixture.inspect(smoke_fixture.fixture())
        with TemporaryDirectory() as temporary:
            home=Path(temporary).resolve();read,write=os.pipe()
            try:
                _helper,views,base,candidate,template=fixture.fixture(home)
                paired=godot.begin_godot_setup(views,base,candidate)
                lock=home/'runtime/lane'
                def factory(step: Literal[4,5],chain: endpoints.WorkspaceChain,output: Path,prep: Path,fd: int) -> endpoints.WorkspaceRequest:
                    data=document(template.manifest.read_text()).mapping()
                    data.update(step=step,profile='debug' if step==4 else 'release',output_root=str(output/'capture'))
                    manifest=prep/'manifest.json';atomic_json(manifest,data)
                    external=dict(template.spec.external_inputs);_=external.pop(template.manifest);external[manifest]=file_hash(manifest)
                    env=dict(template.spec.environment);env.update(CARGO_CAPTURE_MANIFEST=str(manifest),CARGO_CAPTURE_MANIFEST_SHA256=file_hash(manifest))
                    spec=replace(template.spec,argv=workspace.workspace_argv(str(template.spec.executable),step),
                        output_root=output/'parent',external_inputs=tuple(sorted(external.items())),environment=tuple(sorted(env.items())))
                    inventory=replace(template.inventory,executables=tuple(replace(item,step=step,
                        profile='debug' if step==4 else 'release') for item in template.inventory.executables))
                    return replace(template,step=step,chain=chain,output=output,held_lane_fd=fd,
                        manifest=manifest,manifest_sha256=file_hash(manifest),inventory=inventory,spec=spec)
                wait=unused_wait(candidate)
                setup=gate.SetupContext(candidate,wait,clippy.ClippyWaitEvidence(wait,()),factory)
                tool=next(t for t in candidate.inputs.tools if t.name=='godot')
                environment=static.ExecutionEnvironment((),(),())
                setup_selector=cli.SetupSelectors(candidate.inputs.tools,candidate.inputs.environment,
                    candidate.inputs.lock_origin,candidate.inputs.lock_origin_sha256,(),())
                spec=cli.PrefixSpec(home/'cli',read,views.builds.prepared.selectors,setup_selector,environment,environment,
                    Tool('rustdoc',tool.path,tool.sha256),setup_selector)
                selection=home/'selectors.json';_=selection.write_text('{}')
                bound=cli.PrefixDocument(selection,file_hash(selection),spec)
                spec.output.mkdir()
                initialized=SimpleNamespace(spec=spec,products=views,handoff=paired.candidate,
                    wait=wait,clippy_wait=setup.clippy_wait,godot_setup=paired,
                    static_product=static.StaticProduct(candidate.inputs.root,candidate.inputs.source_sha,views.probes.candidate.input_files))
                def workspace_request(_bound: cli.PrefixPrepared,step: Literal[4,5],chain: endpoints.WorkspaceChain,
                                      output: Path,prep: Path,fd: int) -> endpoints.WorkspaceRequest:
                    return factory(step,chain,output,prep,fd)
                postpublication_checks: list[Path]=[]
                original_selector=cli.check_spec
                original_products=godot.check_godot_products
                def selector_check(selected: cli.PrefixDocument) -> None:
                    original_selector(selected)
                    if (spec.output/'ordered/setup-prefix.json').exists():postpublication_checks.append(selected.path)
                def products_check(actual: godot.GodotProducts) -> None:
                    original_products(actual)
                    if selector_attack and len(postpublication_checks)==2:
                        _=selection.write_text('{"changed_after_last_selector_check":true}')
                actual_smoke: list[tuple[ProcessSpec,ProcessResult]]=[]
                base_completed: list[godot.BaseGodotReady]=[]
                original_base=godot.run_base_setup_owned
                def auxiliary(setup: godot.GodotSetup, wait: CargoWaitEvidence,
                              output: Path,descriptor: int) -> godot.BaseGodotReady:
                    ready=original_base(setup,wait,output,descriptor)
                    base_completed.append(ready)
                    return ready
                def inspect(spec: ProcessSpec,result: ProcessResult,producer: str,_original: object) -> smoke.SmokeEvidence:
                    # Only semantic world evidence is substituted. Calls/waits/argv/output ownership are real.
                    actual_smoke.append((spec,result))
                    plan=replace(proof.evidence.plan,process=spec,producer=producer)
                    return replace(proof,evidence=replace(proof.evidence,plan=plan))
                def static_step(_step: Literal[1,7,8],product: static.StaticProduct,_env: static.ExecutionEnvironment,
                                output: Path,_control: int) -> tuple[gate.Outcome,endpoints.EvidenceAnchor]:
                    executable=Path(sys.executable).resolve()
                    process=ProcessSpec((str(executable),'-c','pass'),product.root,executable,file_hash(executable),
                                        product.root,product.inputs,output)
                    result=execute(process);retained=endpoints.anchor(output)
                    self.assertEqual(result.native_exit,0)
                    return gate.Outcome('static','COMPONENT_PASS',result.receipt,file_hash(result.receipt),'',
                        gate.setup_files(retained.state),output,'PASS','PASS'),retained
                with ExitStack() as stack:
                    _=stack.enter_context(patch.object(gate, 'LOCK', lock))
                    _=stack.enter_context(patch('generic_setup_bridge.LOCK', lock))
                    _=stack.enter_context(patch.object(child, 'LOCK', lock))
                    _=stack.enter_context(patch.object(workspace, 'LOCK', lock))
                    _=stack.enter_context(patch('generic_setup_bridge.qualify_wait', return_value=()))
                    _=stack.enter_context(patch('generic_setup_import.qualify_wait', return_value=()))
                    _=stack.enter_context(patch.object(clippy, 'qualify_clippy', return_value=()))
                    _=stack.enter_context(patch.object(smoke, 'PINS', dict[str, str]()))
                    _=stack.enter_context(patch.object(smoke, 'inspect', side_effect=inspect))
                    _=stack.enter_context(patch.object(smoke, 'oracle', return_value={}))
                    _=stack.enter_context(patch.object(smoke, 'validate_smoke_companion'))
                    _=stack.enter_context(patch.object(gate, '_prefix_static', side_effect=static_step))
                    _=stack.enter_context(patch.object(gate, 'authenticate_static', return_value='PASS'))
                    _=stack.enter_context(patch.object(static, 'strict_input_names', return_value=[n for n, _ in views.probes.candidate.input_files]))
                    _=stack.enter_context(patch.object(cli, 'read_spec', return_value=bound))
                    _=stack.enter_context(patch.object(cli, 'initialize', return_value=initialized))
                    _=stack.enter_context(patch.object(cli, 'workspace_request', side_effect=workspace_request))
                    _=stack.enter_context(patch.object(godot, 'run_base_setup_owned', side_effect=auxiliary))
                    _=stack.enter_context(patch.object(cli, 'check_spec', side_effect=selector_check))
                    _=stack.enter_context(patch.object(godot, 'check_godot_products', side_effect=products_check))
                    fcntl.flock(template.held_lane_fd,fcntl.LOCK_UN)
                    if selector_attack:
                        with self.assertRaises(GateContractError):_=gate.execute_setup_prefix(selection,smoke_prefix=True)
                        self.assertEqual(len(postpublication_checks),2)
                        self.assertEqual(len(actual_smoke),8)
                        self.assertEqual(len(base_completed),1)
                        with self.assertRaises(OSError):_=os.fstat(base_completed[0].predecessor.owner.controller.lane_fd)
                        return
                    receipt=gate.execute_setup_prefix(selection,smoke_prefix=True)
                self.assertEqual(len(base_completed),1)
                with self.assertRaises(OSError):_=os.fstat(base_completed[0].predecessor.owner.controller.lane_fd)
                payload=document(receipt.read_text())
                self.assertEqual(payload.field('errors').value,[])
                self.assertEqual(payload.field('status').value,'INCOMPLETE')
                self.assertEqual(payload.field('actual_steps').value,list(range(1,10)))
                self.assertEqual(len(actual_smoke),8)
                self.assertTrue(all(result.native_exit==0 and result.pid is not None for _,result in actual_smoke))
                self.assertEqual(actual_smoke[0][0].argv[-2:],('--quit-after','900'))
                self.assertEqual(actual_smoke[2][0].argv[-3:],('--','--frames','900'))
                self.assertTrue(all(row.field('status').value=='NOT_RUN' for row in payload.field('obligations').array()
                                    if row.field('step').value not in range(1,10)))
            finally:os.close(read);os.close(write)


class SchemaTests(unittest.TestCase):
    def test_closed_modes_and_explicit_base_selectors(self) -> None:
        import generic_cli_execute as cli
        from generic_gate import GateContractError, document
        from generic_setup_prepare import TOOLS
        from mux2_gate_inputs import Json, Node

        def tool(name: str) -> dict[str,Json]:return {'name':name,'path':'/tools/'+name,'sha256':'a'*64}
        def arm(name: str) -> dict[str,Json]:return {'root':'/'+name,'source_sha':'b'*40,'cargo_home':'/'+name+'-cache',
            'configs':[],'environment':[],'overlay':None}
        setup: dict[str,Json]={'tools':[tool(name) for name in sorted(TOOLS)],'environment':[],
            'lock_origin':'/lock','lock_origin_sha256':'a'*64,'cargo_wait_sources':[],'clippy_wait_sources':[]}
        env: dict[str,Json]={'values':[],'tools':[],'external_inputs':[]}
        raw: dict[str,Json]={'schema':'cli_setup_prefix.v1','output':'/output','control_fd':5,
            'selectors':{'harness_root':'/harness','harness_sha':'b'*40,'published_sha':'b'*40,'purpose':'self_proof',
                         'base':arm('base'),'candidate':arm('candidate'),'cargo':tool('cargo'),'rustc':tool('rustc')},
            'setup':setup,'static':env,'strict':env,'rustdoc':tool('rustdoc')}
        self.assertIsNone(cli.parse_spec(Node(raw)).base_setup)
        raw['base_setup']=setup
        with self.assertRaises(GateContractError):_=cli.parse_spec(Node(raw))
        raw['schema']='cli_smoke_prefix.v1'
        self.assertIsNotNone(cli.parse_spec(Node(raw)).base_setup)
        changed=document(json.dumps(raw)).mapping()
        _=changed.pop('base_setup')
        with self.assertRaises(GateContractError):_=cli.parse_spec(Node(changed))


class SmokePublicationTests(unittest.TestCase):
    def test_intended_terminal_and_existing_namespace_survive_publication(self) -> None:
        from generic_gate import GateContractError
        from generic_ui_history import UIEvidence
        from mux2_gate_inputs import Json
        from test_generic_ui_smoke_run import SmokeTests
        fixture=SmokeTests();self.addCleanup(fixture.doCleanups)
        proof=fixture.inspect(fixture.fixture())
        for attack in ('control','counts','mode','empty-directory'):
            with self.subTest(attack=attack),TemporaryDirectory() as temporary:
                home=Path(temporary).resolve();a=home/'a';b=home/'b';a.mkdir();b.mkdir()
                executable=Path(sys.executable).resolve();digest=file_hash(executable)
                products=Products(*(ProbeProduct('a'*40,executable,digest,root,()) for root in (a,b)))
                def spec(product: ProbeProduct,_kind: str,output: Path,_original: UIEvidence | None) -> ProcessSpec:
                    return ProcessSpec((str(product.executable),'-c','pass'),product.input_root,product.executable,product.executable_sha256,
                                       product.input_root,(),output,timeout_seconds=600)
                def publish(path: Path,payload: dict[str,Json],attack: str=attack) -> None:
                    atomic_json(path,payload)
                    if attack=='counts':
                        altered=dict(payload);altered['counts']={'forged':{'errors':999}};altered['originals']=[]
                        _=path.write_text(json.dumps(altered))
                    elif attack=='mode':path.chmod(0o777)
                    elif attack=='empty-directory':(path.parent/'late-empty').mkdir()
                returned: list[ProcessResult]=[]
                def before(_spec: ProcessSpec) -> None:pass
                def retain(_spec: ProcessSpec,result: ProcessResult,returned: list[ProcessResult]=returned) -> None:returned.append(result)
                with patch.object(smoke,'check_product'),patch.object(smoke,'process_spec',side_effect=spec), \
                     patch.object(smoke,'inspect',return_value=proof),patch.object(smoke,'oracle',return_value={}), \
                     patch.object(smoke,'validate_smoke_companion'),patch.object(smoke,'atomic_json',side_effect=publish):
                    if attack=='control':
                        result=smoke.run_pair(products,home/'output',before=before,retain=retain)
                        self.assertEqual(result.status,'PASS')
                    else:
                        with self.assertRaises(GateContractError):
                            _=smoke.run_pair(products,home/'output',before=before,retain=retain)
                self.assertEqual(len(returned),8)
                self.assertTrue(all(result.native_exit==0 for result in returned))
