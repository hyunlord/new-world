# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Synthetic endpoint envelopes and mocked orchestration, never native proof."""
import json
import unittest
from contextlib import nullcontext
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Literal
from unittest.mock import patch

import generic_direct_run as direct
import generic_full_gate as gate
import generic_perf_run as perf
import generic_renderer_run as renderer
import generic_static_run as static
import generic_test_run as native
import generic_ui_fixed_run as fixed
import generic_ui_pack_run as pack
import generic_ui_smoke_run as smoke
import generic_ui_visibility_run as visibility
import generic_ui_why_run as why
from generic_bench import Product
from generic_declarations import Declarations
from generic_fixture_registry import (
    DIRECT_IDS,
    ExpansionContext,
    Fixture,
    Registry,
    expand,
)
from generic_fixture_run import ComponentResult, Products
from generic_fixtures import ProbeProduct
from generic_gate import GateContractError, document, integer
from generic_process import atomic_json, file_hash
from generic_sources import inventory
from mux2_gate_inputs import INPUT_DIRS, INPUT_FILES, Json


def inputs(root: Path) -> gate.Inputs:
    source = root/'source'
    source.mkdir()
    binary = source/'fake'
    _ = binary.write_text('synthetic')
    product = ProbeProduct('a'*40, binary, file_hash(binary), source, (('fake', file_hash(binary)),))
    products = Products(product, product)
    declarations = Declarations('a'*40, 'a'*40, 'self_proof', (), (), (), (), (), 'b'*64)
    return gate.Inputs(Registry('b'*64, (), (), '{}'), products, products, declarations)


def fixed_endpoint(out: Path, producer: str = 'toast', status: str = 'PASS') -> ComponentResult:
    out.mkdir()
    path = out/'fixed-ui.json'
    atomic_json(path, {'schema_version':1, 'producer':producer, 'status':status, 'processes':[], 'errors':[],
        'build_provenance':'caller_owned_not_certified_here', 'full_gate_claim':False,
        'scope':'fixed UI domain plus exact native observations; headless render is not pixel acceptance'})
    return ComponentResult(path,status)


def synthetic_endpoint(name: str, out: Path, context: gate.Inputs) -> ComponentResult:
    """Match real terminal envelopes; lists are synthetic and not native proof."""
    if name in fixed.FIXTURES:return fixed_endpoint(out,name)
    out.mkdir()
    data: dict[str,Json]={'schema_version':1,'status':'PASS','errors':[],
        'build_provenance':'caller_owned_not_certified_here','full_gate_claim':False}
    if name=='direct':
        data={'schema':'direct_components_v1','status':'PASS','publication_authority':False,
            'full_gate_status':'NOT_QUALIFIED','registry_status':'INCOMPLETE_ADAPTERS','registry_sha256':context.registry.sha256,
            'base':context.probes.base.source_sha,'candidate':context.probes.candidate.source_sha,
            'declaration_manifest_sha256':context.declarations.source_manifest_sha256,'components':[],
            'errors':[],'unresolved_backlog':[],'limit':'synthetic fixture',
            'products':{arm:{'root':str(context.probes.arm(arm).input_root),'executable':str(context.probes.arm(arm).executable),
                'executable_sha256':context.probes.arm(arm).executable_sha256} for arm in ('base','candidate')}}
    elif name=='renderer':
        data.update(processes=[],safety=[],pending=[],outputs={},sources={'base':'a'*40,'candidate':'a'*40})
    elif name=='visibility':data.update(artifacts={},history_chains=[])
    elif name=='pack':
        data.update(classification='ordinary_selected_pack_continuous_world',negative_DTO_checks='presentation_validation_only',
            selected_pack_inputs={key:value for key,value in pack.PINS.items() if key.startswith(str(Path(pack.PACK).parent)+'/')},
            worlds=[],artifacts={},rendered_capture='NOT_RUN_HEADLESS',rust_mechanism_children='outside_this_component')
    else:data.update(variant=name.removeprefix('why.'),classification='ordinary_boot_prefix_then_direct_building_placed_event_mechanism',
                     mechanisms=[],artifacts={})
    path=out/gate.endpoint(name);atomic_json(path,data)
    return ComponentResult(path,'PASS')


class DispatchTests(unittest.TestCase):
    def test_every_step_and_exact_preservation_mapping(self) -> None:
        rows = gate.obligations()
        self.assertEqual({row.step for row in rows}, set(range(1, 52)))
        self.assertEqual(len({row.identity for row in rows}), len(rows))
        self.assertEqual({row.fixture for row in rows if row.step == 50}, {
            'read.determinism','read.soak20','read.story48','read.neighbours48',
            'read.steward_on24','read.m6a_steward_on72'})
        names = {row.identity for row in rows}
        self.assertTrue({'4:workspace.debug','5:workspace.release','24:save_load.release',
                        '17:sim-test.lib.probe::colony.release','37:b5_flora.exact_ignored.release'} <= names)

    def test_order_is_explicitly_development_not_original(self) -> None:
        with TemporaryDirectory() as temp:
            result = gate.write_plan(Path(temp).resolve()/'out')
            plan = document(result.read_text())
            self.assertFalse(plan.field('original_order_executed').value)
            self.assertEqual(tuple(row.text() for row in plan.field('component_sequence').array()), gate.COMPONENTS)
            self.assertEqual([integer(row) for row in plan.field('original_step_order').array()], list(gate.ORDER))

    def test_status_disagreement_rejected_and_receipt_retained(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve(); context=inputs(root)
            def child(out: Path) -> ComponentResult:
                value=fixed_endpoint(out)
                return ComponentResult(value.receipt,'FAIL')
            row=gate.invoke('toast',child,root/'toast',context)
            self.assertEqual(row.status,'FAIL');self.assertIsNotNone(row.receipt)
            self.assertIn('status',row.error)
            self.assertTrue(row.files)

    def test_wrong_identity_schema_unknown_status_and_status_only_rejected(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=inputs(root)
            changes: tuple[dict[str,Json],...]=({'producer':'render.people'},{'schema_version':2},{'status':'WAIVED'},{'status_only':True})
            for index,change in enumerate(changes):
                def child(out: Path, change: dict[str,Json] = change) -> ComponentResult:
                    value=fixed_endpoint(out)
                    doc: dict[str,Json]=document(value.receipt.read_text()).mapping().copy()
                    if 'status_only' in change: doc={'status':'PASS'}
                    else: doc.update(change)
                    value.receipt.unlink();atomic_json(value.receipt,doc)
                    return value
                self.assertEqual(gate.invoke('toast',child,root/str(index),context).status,'FAIL')

    def test_exception_written_evidence_is_frozen_and_mutation_rejected(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=inputs(root)
            def child(out: Path) -> ComponentResult:
                out.mkdir();_=(out/'native.txt').write_text('raw failed producer')
                raise OSError('producer raised after writing')
            row=gate.invoke('toast',child,root/'toast',context)
            self.assertEqual(row.status,'FAIL');self.assertTrue(row.files)
            _=(root/'toast/native.txt').write_text('later mutation')
            with self.assertRaises(GateContractError):gate.recheck(row)

    def test_failed_terminal_written_then_exception_is_retained(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=inputs(root)
            def child(out: Path) -> ComponentResult:
                _=fixed_endpoint(out,status='FAIL')
                raise OSError('after terminal')
            row=gate.invoke('toast',child,root/'toast',context)
            self.assertIsNotNone(row.receipt);self.assertIsNotNone(row.sha256)
            self.assertEqual(row.reported_status,'FAIL')

    def test_lane_failure_has_terminal_and_not_run_components(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=inputs(root)
            with patch.object(gate,'lane',side_effect=OSError('lane unavailable')):
                terminal=gate.run(context,root/'out')
            node=document(terminal.read_text());self.assertEqual(node.field('status').value,'FAIL')
            self.assertTrue(all(row.field('status').value=='NOT_RUN' for row in node.field('components').array()))

    def test_plan_creation_failure_has_explicit_terminal(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=inputs(root)
            with patch.object(gate,'write_plan',side_effect=OSError('plan write failed')):
                terminal=gate.run(context,root/'out')
            doc=document(terminal.read_text());self.assertEqual(doc.field('status').value,'FAIL')
            self.assertIsNone(doc.field('plan_sha256').value)
            self.assertTrue(all(row.field('status').value=='NOT_RUN' for row in doc.field('components').array()))

    def test_deleted_plan_has_failure_terminal_and_original_hash(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=inputs(root)
            def failed(name: str, _action: gate.Action, out: Path, _context: gate.Inputs) -> gate.Outcome:
                (out.parent/'plan.json').unlink(missing_ok=True)
                return gate.Outcome(name,'FAIL',None,None,'synthetic failure')
            with patch.object(gate,'lane',return_value=nullcontext()),patch.object(gate,'invoke',side_effect=failed):
                terminal=gate.run(context,root/'out')
            node=document(terminal.read_text());self.assertEqual(node.field('status').value,'FAIL')
            self.assertEqual(len(node.field('plan_sha256').text()),64)

    def test_shared_preservation_references_are_concrete_expanded_invocations(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=inputs(root)
            names=tuple(row.fixture for row in gate.obligations() if row.step==50 and row.fixture is not None)
            fixtures=tuple(Fixture(name,('{probe}','--out','{out}'),(0,1),('1',),('ordinary',),'colony_sidecar','{}') for name in names)
            context=replace(context,registry=Registry('b'*64,fixtures,(),'{}'))
            plan=document(gate.write_plan(root/'out',context).read_text())
            for row in plan.field('obligations').array():
                if row.field('step').value==50:
                    refs=row.field('invocation_references').array();fixture=row.field('fixture').text()
                    self.assertEqual([ref.field('id').text() for ref in refs],['base/'+fixture+'/1','candidate/'+fixture+'/1'])
                    self.assertTrue(all('/direct/runs/' in ref.field('out').text() for ref in refs))

    def test_all_schema_faithful_adapters_run_in_recorded_component_order(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=inputs(root);called: list[str]=[]
            def child(*args: Registry | Products | Declarations | renderer.RendererProducts | Path | str) -> ComponentResult:
                out=args[-1];assert isinstance(out,Path)
                called.append(out.name)
                return synthetic_endpoint(out.name,out,context)
            with patch.object(gate,'lane',return_value=nullcontext()),patch.object(direct,'run_all',side_effect=child), \
                 patch.object(renderer,'run_pair',side_effect=child),patch.object(fixed,'run_pair',side_effect=child), \
                 patch.object(pack,'run_pair',side_effect=child),patch.object(visibility,'run_pair',side_effect=child), \
                 patch.object(why,'run_pair',side_effect=child):
                terminal=gate.run(context,root/'out')
            doc=document(terminal.read_text());self.assertEqual(doc.field('status').value,'INCOMPLETE')
            self.assertEqual(called,list(gate.COMPONENTS))
            self.assertEqual([row.text() for row in doc.field('actual_component_sequence').array()],called)
            self.assertTrue(all(row.field('status').value=='PASS' for row in doc.field('components').array()))

    def test_direct_source_and_declaration_identity_cannot_be_relabelled(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=inputs(root)
            for field in ('base','registry_sha256','declaration_manifest_sha256'):
                def child(out: Path, field: str = field) -> ComponentResult:
                    value=synthetic_endpoint('direct',out,context)
                    data=document(value.receipt.read_text()).mapping().copy();data[field]='foreign'
                    value.receipt.unlink();atomic_json(value.receipt,data)
                    return value
                self.assertEqual(gate.invoke('direct',child,root/field,context).status,'FAIL')

    def test_later_component_mutating_failed_raw_evidence_fails_final_barrier(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=inputs(root)
            def child(*args: Registry | Products | Declarations | renderer.RendererProducts | Path | str) -> ComponentResult:
                out=args[-1];assert isinstance(out,Path)
                if out.name=='direct':
                    out.mkdir();_=(out/'failed-native.txt').write_text('original')
                    raise OSError('native producer failed')
                if out.name=='renderer':_=(out.parent/'direct/failed-native.txt').write_text('mutated')
                return synthetic_endpoint(out.name,out,context)
            with patch.object(gate,'lane',return_value=nullcontext()),patch.object(direct,'run_all',side_effect=child), \
                 patch.object(renderer,'run_pair',side_effect=child),patch.object(fixed,'run_pair',side_effect=child), \
                 patch.object(pack,'run_pair',side_effect=child),patch.object(visibility,'run_pair',side_effect=child), \
                 patch.object(why,'run_pair',side_effect=child):
                terminal=gate.run(context,root/'out')
            doc=document(terminal.read_text());self.assertEqual(doc.field('status').value,'FAIL')
            self.assertIn('completed artifacts changed',terminal.read_text())

    def test_core_loss_and_all_success_never_publish(self) -> None:
        self.assertEqual(gate.aggregate((gate.Outcome('direct','FAIL',None,None,'KnowledgeLost'),)),'FAIL')
        self.assertEqual(gate.aggregate(tuple(gate.Outcome(name,'PASS',None,None,'') for name in gate.COMPONENTS)),'INCOMPLETE')
        with self.assertRaises(GateContractError):_=gate.aggregate((gate.Outcome('foreign','PASS',None,None,''),))



def routing_native_bindings(context: gate.OrderedInputs) -> None:
    """Existing routing fixtures do not claim native compiler admission."""
    del context


def routing_native_status(path: Path, products: gate.SaveRoundtripProducts | None) -> str:
    assert products is not None
    return document(path.read_text()).field('status').text()


def routing_perf_status(path: Path, name: str, step: int | None, context: gate.PerfContext | None) -> str:
    assert context is not None
    _=gate.perf_fixture(name,step)
    return document(path.read_text()).field('status').text()


@patch.object(gate,'perf_bindings',new=routing_native_bindings)
@patch.object(gate,'authenticate_perf',new=routing_perf_status)
@patch.object(gate,'save_roundtrip_bindings',new=routing_native_bindings)
@patch.object(gate,'authenticate_save_roundtrip',new=routing_native_status)
class DiagnosisCoordinatorTests(unittest.TestCase):
    def test_selected_data_context_and_final_order(self) -> None:
        with TemporaryDirectory() as temp:
            context=ordered_inputs(Path(temp).resolve())
            self.assertEqual(gate.static_context(context,51),(context.diagnosis_product,context.diagnosis_environment))
            self.assertIn(static.DIAGNOSIS_RESULT,dict(context.diagnosis_product.inputs))
            self.assertNotIn('docs/scope.md',dict(context.diagnosis_product.inputs))
            self.assertNotIn(static.DIAGNOSIS_RESULT,dict(context.static_product.inputs))
            self.assertIn('docs/scope.md',dict(context.discovery_product.inputs))
            self.assertEqual(gate.ordered_calls()[-1],gate.OrderedCall(51,'static',('51:read_diagnosis_test',)))
            gate.ordered_bindings(context)

    def test_missing_wrong_or_foreign_diagnosis_context_blocks_every_launch(self) -> None:
        for fault in ('missing_data','full_docs','foreign_root','source','optimize','bash_env','helpers','duplicate'):
            with self.subTest(fault=fault),TemporaryDirectory() as temp:
                root=Path(temp).resolve();context=ordered_inputs(root)
                if fault=='missing_data':context=replace(context,diagnosis_product=context.static_product)
                elif fault=='full_docs':context=replace(context,diagnosis_product=context.discovery_product)
                elif fault=='foreign_root':context=replace(context,diagnosis_product=replace(context.diagnosis_product,root=root/'foreign'))
                elif fault=='source':context=replace(context,diagnosis_product=replace(context.diagnosis_product,source_sha='f'*40))
                elif fault in ('optimize','bash_env'):
                    key='PYTHONOPTIMIZE' if fault=='optimize' else 'BASH_ENV'
                    context=replace(context,diagnosis_environment=replace(context.diagnosis_environment,
                        values=(*context.diagnosis_environment.values,(key,'1'))))
                elif fault=='helpers':context=replace(context,diagnosis_environment=replace(context.diagnosis_environment,tools=()))
                else:context=replace(context,diagnosis_product=replace(context.diagnosis_product,
                    inputs=(*context.diagnosis_product.inputs,context.diagnosis_product.inputs[0])))
                with patch.object(gate,'ordered_action',side_effect=AssertionError('no launch')):
                    result=document(gate.run_ordered(context,root/'out').read_text())
                self.assertEqual(result.field('status').value,'FAIL')
                self.assertEqual(result.field('actual_component_sequence').value,[])

    def test_success_binds_bash_endpoint_without_inner_python_identity_claim(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            result=ordered_endpoint(gate.OrderedCall(51,'static',('51:read_diagnosis_test',)),root/'out',context)
            self.assertEqual(gate.authenticate_static(result.receipt,51,context.diagnosis_product,context.diagnosis_environment),'PASS')
            native=document((root/'out/process/process.json').read_text())
            self.assertEqual(native.field('executable').value,str(Path('/bin/bash').resolve()))
            self.assertNotIn('unittest',document(result.receipt.read_text()).mapping())

    def test_self_consistently_rehashed_marker_and_process_forgery_rejects(self) -> None:
        for fault in ('missing_marker','duplicate_marker','trailing_marker','null_pid','zero_pid','spawn_pid',
                      'start_argv','duration','native_exit','inner_python','unittest','environment','timeout'):
            with self.subTest(fault=fault),TemporaryDirectory() as temp:
                root=Path(temp).resolve();context=ordered_inputs(root)
                result=ordered_endpoint(gate.OrderedCall(51,'static',('51:read_diagnosis_test',)),root/'out',context)
                data=document(result.receipt.read_text()).mapping();process=root/'out/process/process.json'
                native=document(process.read_text()).mapping()
                if fault in ('missing_marker','duplicate_marker','trailing_marker'):
                    raw={'missing_marker':'no oracle\n','duplicate_marker':'READ_DIAGNOSIS_TEST PASS\n'*2,
                         'trailing_marker':'READ_DIAGNOSIS_TEST PASS\nextra\n'}[fault]
                    _=(process.parent/'stdout.log').write_text(raw)
                    native['stdout_sha256']=file_hash(process.parent/'stdout.log')
                elif fault in ('null_pid','zero_pid'):
                    native['pid']=None if fault=='null_pid' else 0;data['pid']=native['pid']
                elif fault=='spawn_pid':_=(process.parent/'process-spawn.json').write_text('{"pid":999,"spawned_at":"2026-01-01T00:00:01+00:00"}')
                elif fault=='start_argv':
                    start=document((process.parent/'process-start.json').read_text()).mapping();start['argv']=['forged']
                    _=(process.parent/'process-start.json').write_text(json.dumps(start))
                elif fault=='duration':native['duration_ns']=0
                elif fault=='native_exit':native['native_exit']=9;data['native_exit']=9
                elif fault=='inner_python':
                    python,digest=next((p,h) for name,p,h in context.diagnosis_environment.tools if name=='python3')
                    native['executable']=str(python);native['expected_executable_sha256']=digest
                    native['argv']=[str(python),*static.COMMANDS[51][1][1:]]
                elif fault=='unittest':data['unittest']={}
                elif fault=='environment':native['environment']={}
                else:native['timeout_seconds']=5
                _=process.write_text(json.dumps(native));data['process_receipt_sha256']=file_hash(process)
                data['artifacts']={key:value for key,value in inventory(result.receipt.parent,['process']).items()}
                _=result.receipt.write_text(json.dumps(data))
                with self.assertRaises((ValueError,OSError)):
                    _=gate.authenticate_static(result.receipt,51,context.diagnosis_product,context.diagnosis_environment)

    def test_early_and_process_failures_remain_fail_without_marker(self) -> None:
        for early in (False,True):
            with self.subTest(early=early),TemporaryDirectory() as temp:
                root=Path(temp).resolve();context=ordered_inputs(root)
                result=ordered_endpoint(gate.OrderedCall(51,'static',('51:read_diagnosis_test',)),root/'out',context,'FAIL')
                if early:
                    data=document(result.receipt.read_text()).mapping()
                    for key in ('process_receipt','process_receipt_sha256','native_exit','pid'):del data[key]
                    for path in (root/'out/process').iterdir():path.unlink()
                    (root/'out/process').rmdir();data['artifacts']={};data['process_status']='NOT_RUN'
                    _=result.receipt.write_text(json.dumps(data))
                self.assertEqual(gate.authenticate_static(result.receipt,51,context.diagnosis_product,context.diagnosis_environment),'FAIL')

    def test_three_way_prior_union_keeps_caller_diagnosis_and_earlier_failed_static(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            prior=ordered_endpoint(gate.OrderedCall(51,'static',('51:read_diagnosis_test',)),root/'prior',context,'FAIL')
            binding=(prior.receipt,file_hash(prior.receipt))
            context=replace(context,diagnosis_environment=replace(context.diagnosis_environment,prior_receipts=(binding,)))
            def failed(out: Path) -> ComponentResult:
                return ordered_endpoint(gate.OrderedCall(15,'static',('15:inventory_closure',)),out,context,'FAIL')
            row=gate.invoke('static',failed,root/'earlier',context.components,step=15,static_product=context.static_product)
            merged=gate.with_static_priors(context,(row,))
            expected=(binding,(root/'earlier/receipt.json',file_hash(root/'earlier/receipt.json')))
            for env in (merged.environment,merged.discovery_environment,merged.diagnosis_environment):
                self.assertEqual(env.prior_receipts,expected)
            self.assertEqual(context.environment.prior_receipts,())
            self.assertEqual(row.reported_status,'FAIL')
            conflicting=replace(context,environment=replace(context.environment,prior_receipts=((prior.receipt,'f'*64),)))
            with self.assertRaises(ValueError):_=gate.with_static_priors(conflicting,(row,))

    def test_late_diagnosis_artifact_mutation_rejects_frozen_outcome(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            def action(out: Path) -> ComponentResult:
                return ordered_endpoint(gate.OrderedCall(51,'static',('51:read_diagnosis_test',)),out,context)
            row=gate.invoke('static',action,root/'out',context.components,step=51,
                static_product=context.diagnosis_product,static_environment=context.diagnosis_environment)
            self.assertEqual(row.status,'PASS')
            _=(root/'out/process/stdout.log').write_text('changed')
            with self.assertRaises(ValueError):gate.recheck(row)

if __name__ == '__main__':
    _ = unittest.main()


class OrderedDispatchTests(unittest.TestCase):
    def test_closed_order_retains_every_missing_obligation(self) -> None:
        calls = gate.ordered_calls()
        self.assertEqual([call.step for call in calls], sorted([call.step for call in calls], key=gate.ORDER.index))
        self.assertEqual([call.step for call in calls if call.component == 'static'], [1,7,13,15,16,20,22,31,33,51])
        self.assertEqual([call.step for call in calls if call.component == 'direct'], [10,11,18,19,26,28,29,30,35,44,47,48,50])
        self.assertEqual({row.step for row in gate.obligations()}, set(range(1,52)))
        self.assertEqual(next(call for call in calls if call.step == 50).identities,
                         tuple('50:'+name for name in direct.DIRECT_STEP_FIXTURES[50]))

    def test_direct_step_authentication_binds_exact_step(self) -> None:
        with TemporaryDirectory() as temp:
            root = Path(temp).resolve(); context = inputs(root)
            result = synthetic_endpoint('direct', root/'direct', context)
            data = document(result.receipt.read_text()).mapping()
            data.update(schema='direct_step_components_v1',step=50,
                        obligations=['50:'+name for name in direct.DIRECT_STEP_FIXTURES[50]],
                        fixtures=list(direct.DIRECT_STEP_FIXTURES[50]),whole_step_complete=False,original_order_executed=False)
            self.assertEqual(gate.authenticate('direct', document(json.dumps(data)), context, step=50),'PASS')
            with self.assertRaises(ValueError):
                _=gate.authenticate('direct', document(json.dumps(data)), context, step=44)


def ordered_inputs(root: Path) -> gate.OrderedInputs:
    import generic_fixture_registry as registry_module
    context=inputs(root)
    registry=registry_module.load_registry(Path(registry_module.__file__).with_name('gate_fixtures.json'))
    context=replace(context,registry=registry,declarations=replace(context.declarations,known_fixtures=tuple(sorted(DIRECT_IDS))))
    product=context.probes.candidate
    for name in (*INPUT_DIRS, 'docs'):(product.input_root/name).mkdir(parents=True,exist_ok=True)
    for name in INPUT_FILES:
        target=product.input_root/name;target.parent.mkdir(parents=True,exist_ok=True);_=target.write_text('input')
    _=(product.input_root/'docs/scope.md').write_text('discovery-only')
    diagnosis_path=product.input_root/static.DIAGNOSIS_RESULT
    diagnosis_path.parent.mkdir(parents=True)
    _=diagnosis_path.write_text('{"questions": []}')
    bindir=root/'bin';bindir.mkdir()
    tool_rows: list[tuple[str,Path,str]]=[]
    for name in ('python3','git','bash','grep','sh','sleep','dirname','mkdir','sed','date','touch','env','shasum','cmp','perl',
                 'cargo','cargo-fmt','rustfmt'):
        executable=bindir/name
        if name in ('bash','sh','sleep','perl'):
            executable.symlink_to(Path('/usr/bin' if name=='perl' else '/bin',name).resolve(strict=True))
        else:
            _=executable.write_text('#!/bin/sh\nexit 0\n');executable.chmod(0o755)
        target=executable.resolve(strict=True);tool_rows.append((name,target,file_hash(target)))
    tools=tuple(tool_rows)
    values=(('PATH',str(bindir)),('LANG','C'),('LC_ALL','C'),('CARGO_BUILD_JOBS','8'))
    discovery_values=(*values,('HOME',str(root)),('TMPDIR',str(root)),('PYTHONDONTWRITEBYTECODE','1'),
                      ('PYTHONNOUSERSITE','1'),('GIT_CONFIG_GLOBAL','/dev/null'),('GIT_CONFIG_NOSYSTEM','1'))
    old=static.StaticProduct(product.input_root,product.source_sha,
        tuple(inventory(product.input_root,[*INPUT_DIRS,*INPUT_FILES]).items()))
    discovery=replace(old,inputs=tuple(inventory(product.input_root,[*INPUT_DIRS,*INPUT_FILES,'docs']).items()))
    diagnosis=replace(old,inputs=tuple(inventory(product.input_root,[*INPUT_DIRS,*INPUT_FILES,static.DIAGNOSIS_RESULT]).items()))
    diagnosis_values=(*values,('PYTHONDONTWRITEBYTECODE','1'),('PYTHONNOUSERSITE','1'))
    native_product=Product(product.input_root,product.executable,product.source_sha,product.executable_sha256,root/'unused-artifact','b'*64,product.input_files)
    return gate.OrderedInputs(context,old,static.ExecutionEnvironment(values,tuple(row for row in tools if row[0] in {'python3','git','bash','grep','cargo','cargo-fmt','rustfmt'}),()),discovery,
                              static.ExecutionEnvironment(discovery_values,tools,()),diagnosis,
                              static.ExecutionEnvironment(diagnosis_values,tools,()),gate.SaveRoundtripProducts(native_product,native_product),
                              gate.PerfContext(native_product,native_product,123456.0,'defect_proof'))


def ordered_endpoint(call: gate.OrderedCall, out: Path, context: gate.OrderedInputs, status: str='PASS') -> ComponentResult:
    if call.component=='static':
        out.mkdir();path=out/'receipt.json'
        product,_environment=gate.static_context(context,call.step)
        data: dict[str,Json]={'schema':'generic_static_run.v1','step':call.step,'name':static.COMMANDS[call.step][0],
            'scope':'COMPONENT_ONLY','original_order_executed':False,'full_gate_complete':False,'source_admission_granted':False,
            'source_root':str(product.root),'source_sha':product.source_sha,
            'source_invocation':list(static.COMMANDS[call.step][1]),'started_at':'synthetic','status':status,
            'process_status':status,'artifacts':{},'tool_dependency_closure':'caller_supplied_not_independently_complete',
            'errors':[],'finished_at':'synthetic'}
    elif call.component in {'perf','nav'}:
        out.mkdir();path=out/'result.json';data={'status':'COMPONENT_PASS' if status=='PASS' else status}
        status=str(data['status'])
    elif call.component=='save_roundtrip':
        out.mkdir();path=out/'result.json';data={'status':status}
    elif call.component=='smoke':
        out.mkdir();path=out/'smoke.json'
        data={'schema_version':1,'status':status,'errors':[],'counts':{},'matched_companions':[],'originals':[],
              'artifacts':{},'build_provenance':'caller_owned_not_certified_here','full_gate_claim':False}
    else:
        result=synthetic_endpoint(call.component,out,context.components)
        path=result.receipt;data=document(path.read_text()).mapping();path.unlink()
        data['status']=status
        if call.component=='direct':
            data.update(schema='direct_step_components_v1',step=call.step,obligations=list(call.identities),
                        fixtures=list(direct.step_fixtures(call.step)),whole_step_complete=False,original_order_executed=False)
    if call.component=='static' and call.step in (16,51):
        discovery_process(out,data,context,status,call.step)
    atomic_json(path,data)
    return ComponentResult(path,status)


def discovery_process(out: Path, data: dict[str,Json], context: gate.OrderedInputs, status: str, step: int=16) -> None:
    product,environment=gate.static_context(context,step)
    process=out/'process';process.mkdir()
    _=(process/'stdout.log').write_text('READ_DIAGNOSIS_TEST PASS\n' if step==51 and status=='PASS' else 'raw output')
    _=(process/'stderr.log').write_text('Ran 2 tests in 0.001s\n\nOK\n' if status=='PASS'
                                      else 'Ran 2 tests in 0.001s\n\nFAILED (errors=1)\n')
    executable,digest=next((path,digest) for name,path,digest in environment.tools if name==static.COMMANDS[step][1][0])
    native: dict[str,Json]={'schema_version':1,'argv':[str(executable),*static.COMMANDS[step][1][1:]],
        'cwd':str(product.root),'input_root':str(product.root),'output_root':str(process),'executable':str(executable),
        'expected_executable_sha256':digest,'expected_inputs':dict(product.inputs),'environment':dict(environment.values),
        'status':status,'native_exit':0 if status=='PASS' else 1,'pid':123,'errors':[] if status=='PASS' else ['failed'],
        'stdout_sha256':file_hash(process/'stdout.log'),'stderr_sha256':file_hash(process/'stderr.log')}
    externals = {str(path):value for path,value in environment.external_inputs}
    externals.update({str(path):value for _,path,value in environment.tools})
    native.update(expected_external_inputs=dict(externals),before_external_inputs=dict(externals),after_external_inputs=dict(externals),
                  before_inputs=dict(product.inputs),after_inputs=dict(product.inputs),
                  before_executable_sha256=digest,after_executable_sha256=digest)
    native.update(started_at='2026-01-01T00:00:00+00:00',ended_at='2026-01-01T00:00:02+00:00',
                  duration_ns=2000000000,environment_policy='fixed_defaults_plus_explicit_only_no_inheritance',
                  fresh_directories=[],build_provenance_verified=False)
    start=dict(native)
    for name in ('ended_at','duration_ns','stdout_sha256','stderr_sha256','after_inputs','after_executable_sha256'):del start[name]
    start.update(status='FAIL',native_exit=None,pid=None,errors=[],after_external_inputs={})
    atomic_json(process/'process-start.json',start)
    atomic_json(process/'process-spawn.json',{'pid':123,'spawned_at':'2026-01-01T00:00:01+00:00'})
    atomic_json(process/'process.json',native)
    data.update(process_receipt=str(process/'process.json'),process_receipt_sha256=file_hash(process/'process.json'),
                native_exit=native['native_exit'],pid=123,artifacts={key:value for key,value in inventory(out,['process']).items()})
    if step==16:data['unittest']=static.unittest_terminal((process/'stderr.log').read_text())


@patch.object(gate,'perf_bindings',new=routing_native_bindings)
@patch.object(gate,'authenticate_perf',new=routing_perf_status)
@patch.object(gate,'save_roundtrip_bindings',new=routing_native_bindings)
@patch.object(gate,'authenticate_save_roundtrip',new=routing_native_status)
class OrderedExecutionTests(unittest.TestCase):
    def test_supported_trace_and_fresh_step50_are_bound_without_full_pass(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root);trace: list[str]=[]
            def action(call: gate.OrderedCall, bound: gate.OrderedInputs) -> gate.Action:
                def run(out: Path) -> ComponentResult:
                    trace.append(call.identity)
                    return ordered_endpoint(call,out,bound)
                return run
            with patch.object(gate,'ordered_action',side_effect=action),patch.object(gate,'lane',return_value=nullcontext()) as lock, \
                 patch.object(direct,'run_all',side_effect=AssertionError('not ordered')):
                terminal=gate.run_ordered(context,root/'out')
            node=document(terminal.read_text());plan=document((terminal.parent/'plan.json').read_text())
            self.assertEqual(trace,[call.identity for call in gate.ordered_calls()])
            self.assertEqual(lock.call_count,len(trace)-13)
            self.assertEqual(node.field('status').value,'INCOMPLETE')
            self.assertFalse(node.field('full_gate_claim').value)
            missing={row.field('identity').text() for row in node.field('obligations').array() if row.field('status').value=='NOT_IMPLEMENTED'}
            self.assertTrue({'28:colony_knowledge.release','36:data_only_ext.release','46:gdscript.four_files',
                             '48:m6a.plant_art_structural','4:workspace.debug','5:workspace.release'} <= missing)
            self.assertLess(trace.index('47:direct'),trace.index('48:direct'))
            for identity in ('47:m6a.food72','48:m6a.food_data_only'):
                row=next(row for row in node.field('obligations').array() if row.field('identity').value==identity)
                self.assertEqual(row.field('status').value,'COMPONENT_ONLY')
                self.assertEqual(row.field('component_status').value,'PASS')
            structural=next(row for row in plan.field('obligations').array()
                            if row.field('identity').value=='48:m6a.plant_art_structural')
            self.assertIsNone(structural.field('fixture').value)
            self.assertEqual(structural.field('invocation_references').value,[])
            for row in plan.field('obligations').array():
                if row.field('step').value==50:
                    refs=row.field('invocation_references').array()
                    self.assertTrue(refs)
                    self.assertTrue(all('/step-50/direct/runs/' in ref.field('out').text() for ref in refs))

    def test_core_failure_is_retained_after_later_success(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            def action(call: gate.OrderedCall, bound: gate.OrderedInputs) -> gate.Action:
                return lambda out:ordered_endpoint(call,out,bound,'FAIL' if call.step==44 else 'PASS')
            with patch.object(gate,'ordered_action',side_effect=action),patch.object(gate,'lane',return_value=nullcontext()):
                node=document(gate.run_ordered(context,root/'out').read_text())
            self.assertEqual(node.field('status').value,'FAIL')
            states={row.field('identity').text():row.field('status').text() for row in node.field('components').array()}
            self.assertEqual((states['44:direct'],states['50:direct']),('FAIL','PASS'))

    def test_failed_sibling_stops_remaining_step_children(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root);trace: list[str]=[]
            def action(call: gate.OrderedCall, bound: gate.OrderedInputs) -> gate.Action:
                def run(out: Path) -> ComponentResult:
                    trace.append(call.identity)
                    return ordered_endpoint(call,out,bound,'FAIL' if call.component in {'render.people','toast'} else 'PASS')
                return run
            with patch.object(gate,'ordered_action',side_effect=action),patch.object(gate,'lane',return_value=nullcontext()):
                node=document(gate.run_ordered(context,root/'out').read_text())
            self.assertEqual(node.field('status').value,'FAIL')
            for blocked in ('34:render.trees','46:marker.layout','46:why.en','46:why.ko'):self.assertNotIn(blocked,trace)
            self.assertIn('50:direct',trace)

    def test_late_mutation_of_earlier_failed_artifact_fails_finalizer(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            def action(call: gate.OrderedCall, bound: gate.OrderedInputs) -> gate.Action:
                def run(out: Path) -> ComponentResult:
                    result=ordered_endpoint(call,out,bound,'FAIL' if call.step==9 else 'PASS')
                    if call.step==9:_=(out/'raw.log').write_text('retained failure')
                    if call.step==50:_=(root/'out/step-09/smoke/raw.log').write_text('mutated')
                    return result
                return run
            with patch.object(gate,'ordered_action',side_effect=action),patch.object(gate,'lane',return_value=nullcontext()):
                node=document(gate.run_ordered(context,root/'out').read_text())
            self.assertTrue(any('artifacts changed' in row.text() for row in node.field('errors').array()))

    def test_lane_failure_has_terminal_and_no_launch_claim(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            def action(call: gate.OrderedCall, bound: gate.OrderedInputs) -> gate.Action:
                return lambda out:ordered_endpoint(call,out,bound)
            with patch.object(gate,'ordered_action',side_effect=action),patch.object(gate,'lane',side_effect=OSError('lane unavailable')):
                node=document(gate.run_ordered(context,root/'out').read_text())
            self.assertEqual(node.field('status').value,'FAIL')
            self.assertEqual([row.text() for row in node.field('actual_component_sequence').array()],
                             [call.identity for call in gate.ordered_calls() if call.component in {'static','save_roundtrip','perf','nav'}])

    def test_invalid_global_registry_fails_before_any_call(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            context=replace(context,components=replace(context.components,registry=replace(context.components.registry,fixtures=())))
            with patch.object(gate,'ordered_action',side_effect=AssertionError('must not launch')):
                node=document(gate.run_ordered(context,root/'out').read_text())
            self.assertEqual(node.field('status').value,'FAIL')
            self.assertEqual(node.field('actual_component_sequence').array(),[])

    def test_lane_exit_failure_preserves_completed_output_authentication(self) -> None:
        from contextlib import contextmanager
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            @contextmanager
            def failed_exit():
                yield
                raise OSError('unlock failed')
            def action(call: gate.OrderedCall, bound: gate.OrderedInputs) -> gate.Action:
                return lambda out:ordered_endpoint(call,out,bound)
            with patch.object(gate,'ordered_action',side_effect=action),patch.object(gate,'lane',side_effect=failed_exit):
                node=document(gate.run_ordered(context,root/'out').read_text())
            row=next(row for row in node.field('components').array() if row.field('identity').value=='9:smoke')
            self.assertEqual(row.field('status').value,'FAIL')
            self.assertEqual(row.field('reported_status').value,'PASS')
            self.assertTrue(row.field('files').mapping())
            self.assertEqual(len(row.field('sha256').text()),64)

    def test_real_adapter_entrypoints_receive_own_step_and_lane_boundary(self) -> None:
        from contextlib import ExitStack, contextmanager
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root);held=False;trace: list[str]=[]
            @contextmanager
            def lock():
                nonlocal held
                self.assertFalse(held);held=True
                try:yield
                finally:held=False
            calls={call.identity:call for call in gate.ordered_calls()}
            def child(*args: Path | str | int | Registry | Products | Declarations | static.StaticProduct | static.ExecutionEnvironment | renderer.RendererProducts | gate.SaveRoundtripProducts | gate.PerfContext) -> ComponentResult:
                out=next(arg for arg in reversed(args) if isinstance(arg,Path))
                identity=str(int(out.parent.name.removeprefix('step-')))+':'+out.name
                call=calls[identity];trace.append(identity)
                self.assertEqual(held,call.component not in {'static','save_roundtrip','perf','nav'})
                if call.component=='direct':self.assertEqual(args[3],call.step)
                if call.component=='static':self.assertEqual(args[1],call.step)
                return ordered_endpoint(call,out,context)
            with ExitStack() as stack:
                _=stack.enter_context(patch.object(gate,'lane',side_effect=lock))
                _=stack.enter_context(patch.object(gate,'run_save_roundtrip',side_effect=child))
                _=stack.enter_context(patch.object(gate,'run_perf',side_effect=child))
                _=stack.enter_context(patch.object(direct,'run_all',side_effect=AssertionError('no aggregate')))
                for module,name in ((direct,'run_step'),(static,'run_step'),(smoke,'run_pair'),
                                    (renderer,'run_pair'),(fixed,'run_pair'),(pack,'run_pair'),(visibility,'run_pair'),(why,'run_pair')):
                    _=stack.enter_context(patch.object(module,name,side_effect=child))
                node=document(gate.run_ordered(context,root/'out').read_text())
            self.assertEqual(trace,list(calls))
            self.assertEqual(node.field('status').value,'INCOMPLETE')
            self.assertEqual([(row.field('call').text(),row.field('event').text()) for row in node.field('dispatch_events').array()],
                             [(identity,event) for identity in trace for event in ('start','end')])

    def test_plan_write_failure_retains_terminal_without_launches(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            def write(path: Path, data: dict[str,Json]) -> None:
                if path.name=='plan.json':raise OSError('plan failed')
                return atomic_json(path,data)
            with patch.object(gate,'atomic_json',side_effect=write),patch.object(gate,'ordered_action',side_effect=AssertionError('no call')):
                node=document(gate.run_ordered(context,root/'out').read_text())
            self.assertEqual(node.field('status').value,'FAIL')
            self.assertIsNone(node.field('plan_sha256').value)
            self.assertEqual(node.field('actual_component_sequence').array(),[])

    def test_duplicate_expansion_is_rejected_before_any_producer(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root);original=expand
            def duplicate(registry: Registry, expansion: ExpansionContext):
                rows=original(registry,expansion)
                return (*rows,rows[0])
            with patch.object(gate,'expand',side_effect=duplicate),patch.object(gate,'ordered_action',side_effect=AssertionError('no call')):
                node=document(gate.run_ordered(context,root/'out').read_text())
            self.assertEqual(node.field('status').value,'FAIL')
            self.assertTrue(any('duplicate identity/output' in row.text() for row in node.field('errors').array()))

    def test_supported_direct_step_incomplete_schema_is_not_a_pass(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            call=gate.OrderedCall(47,'direct',('47:m6a.food72',))
            result=ordered_endpoint(call,root/'out',context,'INCOMPLETE')
            self.assertEqual(gate.authenticate('direct',document(result.receipt.read_text()),context.components,step=47),'INCOMPLETE')
            self.assertIn(47,[call.step for call in gate.ordered_calls()])
            with self.assertRaises(ValueError):
                _=gate.authenticate('direct',document(result.receipt.read_text()),context.components,step=48)


@patch.object(gate,'perf_bindings',new=routing_native_bindings)
@patch.object(gate,'authenticate_perf',new=routing_perf_status)
@patch.object(gate,'save_roundtrip_bindings',new=routing_native_bindings)
@patch.object(gate,'authenticate_save_roundtrip',new=routing_native_status)
class DiscoveryCoordinatorTests(unittest.TestCase):
    def test_explicit_context_selection_preserves_static_eight(self) -> None:
        with TemporaryDirectory() as temp:
            context=ordered_inputs(Path(temp).resolve())
            for step in (1,7,13,15,20,22,31,33):
                self.assertEqual(gate.static_context(context,step),(context.static_product,context.environment))
            self.assertEqual(gate.static_context(context,16),(context.discovery_product,context.discovery_environment))
            self.assertNotIn('docs/scope.md',dict(context.static_product.inputs))
            self.assertIn('docs/scope.md',dict(context.discovery_product.inputs))
            with patch.dict(static.COMMANDS,{14:('future',('python3','future'))}):
                self.assertNotIn(14,[call.step for call in gate.ordered_calls()])

    def test_discovery_preflight_rejects_missing_docs_source_or_helpers_before_child(self) -> None:
        for fault in ('docs','source','helpers','duplicate'):
            with self.subTest(fault=fault),TemporaryDirectory() as temp:
                root=Path(temp).resolve();context=ordered_inputs(root)
                if fault=='docs':context=replace(context,discovery_product=context.static_product)
                if fault=='source':context=replace(context,discovery_product=replace(context.discovery_product,source_sha='f'*40))
                if fault=='helpers':context=replace(context,discovery_environment=replace(context.discovery_environment,tools=()))
                if fault=='duplicate':context=replace(context,discovery_product=replace(context.discovery_product,inputs=(*context.discovery_product.inputs,context.discovery_product.inputs[0])))
                with patch.object(gate,'ordered_action',side_effect=AssertionError('no producer')):
                    node=document(gate.run_ordered(context,root/'out').read_text())
                self.assertEqual(node.field('status').value,'FAIL')
                self.assertEqual(node.field('actual_component_sequence').value,[])

    def test_exact_discovery_process_and_summary_joins_reject_forgery(self) -> None:
        for fault in ('argv','cwd','executable','input_root','expected_inputs','environment','native_exit','status',
                      'stderr_sha256','timeout_seconds','summary','missing_summary','raw','name','source',
                      'before_inputs','after_inputs','before_executable_sha256','after_executable_sha256',
                      'expected_external_inputs','before_external_inputs','after_external_inputs'):
            with self.subTest(fault=fault),TemporaryDirectory() as temp:
                root=Path(temp).resolve();context=ordered_inputs(root)
                result=ordered_endpoint(gate.OrderedCall(16,'static',('16:python_unittest',)),root/'out',context)
                data=document(result.receipt.read_text()).mapping();process=root/'out/process/process.json'
                native=document(process.read_text()).mapping()
                if fault=='summary':data['unittest']={}
                elif fault=='missing_summary':del data['unittest']
                elif fault=='raw':_=(process.parent/'stderr.log').write_text('OK\n')
                elif fault=='name':data['name']='wrong'
                elif fault=='source':data['source_sha']='f'*40
                elif fault=='timeout_seconds':native[fault]=1
                else:native[fault]='forged'
                _=process.write_text(json.dumps(native))
                data['process_receipt_sha256']=file_hash(process)
                data['artifacts']={key:value for key,value in inventory(result.receipt.parent,['process']).items()}
                _=result.receipt.write_text(json.dumps(data))
                with self.assertRaises(ValueError):
                    _=gate.authenticate_static(result.receipt,16,context.discovery_product,context.discovery_environment)

    def test_original_eight_reject_discovery_summary(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            result=ordered_endpoint(gate.OrderedCall(15,'static',('15:inventory_closure',)),root/'out',context)
            data=document(result.receipt.read_text()).mapping();data['unittest']={};_=result.receipt.write_text(json.dumps(data))
            with self.assertRaises(ValueError):_=gate.authenticate_static(result.receipt,15,context.static_product)

    def test_early_failed_discovery_is_preserved_without_fake_count(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            def early(out: Path) -> ComponentResult:
                out.mkdir();receipt=out/'receipt.json'
                atomic_json(receipt,{'schema':'generic_static_run.v1','step':16,'name':'python_unittest',
                    'scope':'COMPONENT_ONLY','original_order_executed':False,'full_gate_complete':False,
                    'source_admission_granted':False,'source_root':str(context.discovery_product.root),
                    'source_sha':context.discovery_product.source_sha,'source_invocation':list(static.COMMANDS[16][1]),
                    'started_at':'synthetic','finished_at':'synthetic','status':'FAIL','process_status':'NOT_RUN',
                    'artifacts':{},'tool_dependency_closure':'caller_supplied_not_independently_complete','errors':['import startup']})
                return ComponentResult(receipt,'FAIL')
            row=gate.invoke('static',early,root/'out',context.components,step=16,
                            static_product=context.discovery_product,static_environment=context.discovery_environment)
            self.assertEqual((row.status,row.reported_status),('FAIL','FAIL'))
            merged=gate.with_static_priors(context,(row,))
            self.assertEqual(merged.environment.prior_receipts,merged.discovery_environment.prior_receipts)
            self.assertEqual(len(merged.environment.prior_receipts),1)
            self.assertNotIn('unittest',document((root/'out/receipt.json').read_text()).mapping())

    def test_cross_context_failed_receipts_and_return_mismatch_keep_separate_statuses(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            def failed(out: Path) -> ComponentResult:
                result=ordered_endpoint(gate.OrderedCall(15,'static',('15:inventory_closure',)),out,context,'FAIL')
                return ComponentResult(result.receipt,'PASS')
            row=gate.invoke('static',failed,root/'out',context.components,step=15,static_product=context.static_product)
            self.assertEqual((row.status,row.returned_status,row.reported_status),('FAIL','PASS','FAIL'))
            self.assertIn('returned and on-disk',row.error)
            merged=gate.with_static_priors(context,(row,))
            self.assertEqual(merged.discovery_environment.prior_receipts,((root/'out/receipt.json',file_hash(root/'out/receipt.json')),))
            self.assertEqual(context.discovery_environment.prior_receipts,())

    def test_malformed_static_output_retained_but_not_used_as_authenticated_prior(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            def malformed(out: Path) -> ComponentResult:
                result=ordered_endpoint(gate.OrderedCall(15,'static',('15:inventory_closure',)),out,context,'FAIL')
                _=result.receipt.write_text('{}');return result
            row=gate.invoke('static',malformed,root/'out',context.components,step=15,static_product=context.static_product)
            self.assertEqual(row.status,'FAIL');self.assertIsNone(row.reported_status);self.assertTrue(row.files)
            self.assertEqual(gate.with_static_priors(context,(row,)).discovery_environment.prior_receipts,())

    def test_actual_static_entrypoints_receive_cross_context_priors_in_order(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root);seen: list[int]=[]
            def child(product: static.StaticProduct, step: int, out: Path,
                      environment: static.ExecutionEnvironment) -> ComponentResult:
                expected_product,expected_environment=gate.static_context(context,step)
                self.assertEqual(product,expected_product)
                self.assertEqual(environment.tools,expected_environment.tools)
                self.assertEqual(environment.values,expected_environment.values)
                self.assertEqual([integer(document(path.read_text()).field('step')) for path,_ in environment.prior_receipts],seen)
                seen.append(step)
                return ordered_endpoint(gate.OrderedCall(step,'static',(str(step)+':test',)),out,context,'FAIL' if step in (15,16) else 'PASS')
            calls=tuple(call for call in gate.ordered_calls() if call.component=='static')
            with patch.object(gate,'ordered_calls',return_value=calls),patch.object(static,'run_step',side_effect=child):
                node=document(gate.run_ordered(context,root/'out').read_text())
            self.assertEqual(seen,[1,7,13,15,16,20,22,31,33,51])
            self.assertEqual(node.field('status').value,'FAIL')
            self.assertEqual(node.field('components').array()[-1].field('status').value,'PASS')

    def test_later_mutation_of_discovery_raw_output_fails_and_stops_next_dispatch(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            def action(call: gate.OrderedCall, bound: gate.OrderedInputs) -> gate.Action:
                def run(out: Path) -> ComponentResult:
                    result=ordered_endpoint(call,out,bound)
                    if call.step==20:_=(root/'out/step-16/static/process/stderr.log').write_text('late mutation')
                    return result
                return run
            with patch.object(gate,'ordered_action',side_effect=action),patch.object(gate,'lane',return_value=nullcontext()):
                node=document(gate.run_ordered(context,root/'out').read_text())
            self.assertEqual(node.field('status').value,'FAIL')
            self.assertTrue(any('artifacts changed' in value.text() for value in node.field('errors').array()))
            self.assertEqual(next(row for row in node.field('components').array() if row.field('identity').value=='22:static').field('status').value,'NOT_RUN')

    def test_terminal_checks_retain_discovery_failure_and_input_drift(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            def action(call: gate.OrderedCall, bound: gate.OrderedInputs) -> gate.Action:
                def run(out: Path) -> ComponentResult:
                    result=ordered_endpoint(call,out,bound,'FAIL' if call.step==16 else 'PASS')
                    if call.step==16:_=(context.discovery_product.root/'docs/scope.md').write_text('drift')
                    return result
                return run
            with patch.object(gate,'ordered_action',side_effect=action),patch.object(gate,'lane',return_value=nullcontext()):
                node=document(gate.run_ordered(context,root/'out').read_text())
            row=next(row for row in node.field('components').array() if row.field('identity').value=='16:static')
            self.assertEqual(row.field('reported_status').value,'FAIL')
            self.assertTrue(any('static input scope differs' in value.text() for value in node.field('errors').array()))

    def test_artifact_mutation_during_terminal_parsing_is_rejected(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            result=ordered_endpoint(gate.OrderedCall(16,'static',('16:python_unittest',)),root/'out',context)
            original=static.unittest_terminal
            def parser(text: str) -> dict[str,Json]:
                value=original(text)
                _=(root/'out/process/stdout.log').write_text('parser-time mutation')
                return value
            with patch.object(static,'unittest_terminal',side_effect=parser),self.assertRaisesRegex(ValueError,'artifacts changed'):
                _=gate.authenticate_static(result.receipt,16,context.discovery_product,context.discovery_environment)

    def test_discovery_mutates_prior_failed_static_receipt_without_erasing_failure(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            def action(call: gate.OrderedCall, bound: gate.OrderedInputs) -> gate.Action:
                def run(out: Path) -> ComponentResult:
                    result=ordered_endpoint(call,out,bound,'FAIL' if call.step==15 else 'PASS')
                    if call.step==16:_=(root/'out/step-15/static/receipt.json').write_text('{}')
                    return result
                return run
            with patch.object(gate,'ordered_action',side_effect=action),patch.object(gate,'lane',return_value=nullcontext()):
                node=document(gate.run_ordered(context,root/'out').read_text())
            row=next(row for row in node.field('components').array() if row.field('identity').value=='15:static')
            self.assertEqual(row.field('reported_status').value,'FAIL')
            self.assertEqual(node.field('status').value,'FAIL')
            self.assertTrue(node.field('errors').array())

    def test_conflicting_or_foreign_supplied_prior_fails_before_dispatch(self) -> None:
        for foreign in (False,True):
            with self.subTest(foreign=foreign),TemporaryDirectory() as temp:
                root=Path(temp).resolve();context=ordered_inputs(root)
                result=ordered_endpoint(gate.OrderedCall(15,'static',('15:inventory_closure',)),root/'prior',context,'FAIL')
                if foreign:
                    node=document(result.receipt.read_text()).mapping();node['source_sha']='f'*40
                    _=result.receipt.write_text(json.dumps(node))
                digest=file_hash(result.receipt)
                context=replace(context,environment=replace(context.environment,prior_receipts=((result.receipt,digest),)),
                    discovery_environment=replace(context.discovery_environment,prior_receipts=((result.receipt,digest if foreign else 'f'*64),)))
                with patch.object(gate,'ordered_action',side_effect=AssertionError('no producer')):
                    node=document(gate.run_ordered(context,root/'out').read_text())
                self.assertEqual(node.field('status').value,'FAIL')
                self.assertEqual(node.field('actual_component_sequence').value,[])

    def test_source_owned_runtime_policy_rejects_before_any_ordered_call(self) -> None:
        for fault in ('PERL5OPT','PYTHONPATH','WORLDSIM_MUX2_LANE_FD','PATH','HOME','PYTHONNOUSERSITE','static_PATH'):
            with self.subTest(fault=fault),TemporaryDirectory() as temp:
                root=Path(temp).resolve();context=ordered_inputs(root)
                environment=context.environment if fault=='static_PATH' else context.discovery_environment
                values=dict(environment.values)
                values['PATH' if fault=='static_PATH' else fault]='-Mforeign'
                changed=replace(environment,values=tuple(values.items()))
                context=replace(context,environment=changed) if fault=='static_PATH' else replace(context,discovery_environment=changed)
                with patch.object(gate,'ordered_action',side_effect=AssertionError('policy must reject before any child')):
                    node=document(gate.run_ordered(context,root/'out').read_text())
                self.assertEqual(node.field('status').value,'FAIL')
                self.assertEqual(node.field('actual_component_sequence').value,[])

    def test_pass_process_requires_actual_spawn_and_completion_shape(self) -> None:
        for fault in ('null_pid','zero_pid','bool_pid','spawn_pid','missing_spawn','missing_start',
                      'duration','duration_zero','duration_bool','ended','naive_started','policy','timed_out'):
            with self.subTest(fault=fault),TemporaryDirectory() as temp:
                root=Path(temp).resolve();context=ordered_inputs(root)
                result=ordered_endpoint(gate.OrderedCall(16,'static',('16:python_unittest',)),root/'out',context)
                data=document(result.receipt.read_text()).mapping();process=root/'out/process/process.json'
                native=document(process.read_text()).mapping()
                if fault in ('null_pid','zero_pid','bool_pid'):
                    pid=None if fault=='null_pid' else True if fault=='bool_pid' else 0
                    native['pid']=pid;data['pid']=pid
                elif fault=='spawn_pid':_=(process.parent/'process-spawn.json').write_text('{"pid":999,"spawned_at":"2026-01-01T00:00:01+00:00"}')
                elif fault=='missing_spawn':(process.parent/'process-spawn.json').unlink(missing_ok=True)
                elif fault=='missing_start':(process.parent/'process-start.json').unlink(missing_ok=True)
                elif fault=='duration':native['duration_ns']=-1
                elif fault=='duration_zero':native['duration_ns']=0
                elif fault=='duration_bool':native['duration_ns']=True
                elif fault=='ended':native['ended_at']='2025-01-01T00:00:00+00:00'
                elif fault=='naive_started':native['started_at']='2026-01-01T00:00:00'
                elif fault=='policy':native['environment_policy']='inherited'
                else:native['timed_out']=False
                _=process.write_text(json.dumps(native));data['process_receipt_sha256']=file_hash(process)
                data['artifacts']={key:value for key,value in inventory(result.receipt.parent,['process']).items()}
                _=result.receipt.write_text(json.dumps(data))
                with self.assertRaises((ValueError,OSError)):
                    _=gate.authenticate_static(result.receipt,16,context.discovery_product,context.discovery_environment)


    def test_public_environment_validation_uses_unchanged_static_policy(self) -> None:
        with TemporaryDirectory() as temp:
            context=ordered_inputs(Path(temp).resolve())
            for step in (1,7,13,15,16,20,22,31,33):
                _,environment=gate.static_context(context,step)
                self.assertEqual(static.validate_environment(environment,step),{name:path for name,path,_ in environment.tools})
            with self.assertRaises(ValueError):_=static.validate_environment(context.environment,2)


class SaveRoundtripRoutingTests(unittest.TestCase):
    def test_gate24_is_one_closed_ordered_call(self) -> None:
        calls = gate.ordered_calls()
        selected = [row for row in calls if row.step == 24]
        self.assertEqual(selected, [gate.OrderedCall(24, 'save_roundtrip', ('24:save_load.release',))])
        self.assertNotIn('save_roundtrip', gate.COMPONENTS)
        self.assertFalse(any(row.step == 17 for row in calls))


class SaveRoundtripAuthenticationTests(unittest.TestCase):
    def exercise(self, defect: str = '') -> None:
        """Real Python child/process/save receipts; compiler admission mocked by original fixture."""
        import test_generic_test_run as fixtures
        original = native.run_pair
        observed: list[str] = []
        def wrapped(base: dict[str,Product], candidate: dict[str,Product], output: Path, *, invocation: str='development') -> Path:
            path = original(base,candidate,output,invocation=invocation)
            products = gate.SaveRoundtripProducts(base['save_load'],candidate['save_load'])
            raw = path.read_bytes()
            originals={entry:entry.read_bytes() for entry in output.rglob('*') if entry.is_file()}
            data = document(raw.decode()).mapping()
            if defect == 'coverage':data['invocation_coverage']={'save_load':45}
            if defect == 'development':data['invocation_plan']='development'
            if defect == 'status':data['status']='PASS'
            if defect == 'family':data['families']=[]
            if defect == 'inventory':data['output_sha256']={}
            if defect == 'errors':data['errors']=['pretend pass']
            if defect == 'arm':
                receipt=output/'base-save_load-invocation.json'
                changed=document(receipt.read_text()).mapping();changed['arm']='candidate'
                _=receipt.write_text(json.dumps(changed))
                hashes=native.freeze(output);_=hashes.pop('result.json');data['output_sha256']=dict(hashes)
            if defect == 'malformed':_ = path.write_bytes(b'\xff')
            else:_ = path.write_text(json.dumps(data))
            try:
                if defect:
                    with self.assertRaises((GateContractError,ValueError)):
                        _ = gate.authenticate_save_roundtrip(path,products)
                    observed.append('rejected')
                else:observed.append(gate.authenticate_save_roundtrip(path,products))
            finally:
                for entry,content in originals.items():_=entry.write_bytes(content)
            return path
        with patch.object(fixtures,'run_pair',side_effect=wrapped):
            result = fixtures.RunnerTests().exercise(invocation='gate24')
        self.assertEqual(result.field('status').text(),'COMPONENT_PASS')
        self.assertEqual(observed,['rejected' if defect else 'COMPONENT_PASS'])

    def test_actual_synthetic_child_receipts_join_all46_histories(self) -> None:
        self.exercise()

    def test_terminal_forgery_does_not_replace_real_scope_or_safety(self) -> None:
        for defect in ('coverage','development','status','family','inventory','errors','arm','malformed'):
            with self.subTest(defect=defect):self.exercise(defect)

    def test_missing_product_binding_is_rejected(self) -> None:
        with self.assertRaises(GateContractError):
            _ = gate.authenticate_save_roundtrip(Path('/unused'),None)

    def test_direct_native_action_preserves_gate24_and_component_status(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root);out=root/'native'
            out.mkdir();terminal=out/'result.json';atomic_json(terminal,{'status':'FAIL'})
            with patch.object(native,'run_pair',return_value=terminal) as runner:
                result=gate.ordered_action(gate.OrderedCall(24,'save_roundtrip',('24:save_load.release',)),context)(out)
            self.assertEqual(result,ComponentResult(terminal,'FAIL'))
            runner.assert_called_once_with({'save_load':context.save_roundtrip.base},
                {'save_load':context.save_roundtrip.candidate},out,invocation='gate24')

    def test_missing_native_terminal_is_retained_as_failed_call(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            def child(out: Path) -> ComponentResult:
                raise RuntimeError(str(out)+' native preflight rejected')
            result=gate.invoke('save_roundtrip',child,root/'out',context.components,step=24,save_products=context.save_roundtrip)
            self.assertEqual(result.status,'FAIL')
            self.assertIsNone(result.receipt)
            self.assertIn('native preflight rejected',result.error)


    def test_native_failure_is_retained_without_successful_candidate_receipt(self) -> None:
        import test_generic_test_run as fixtures
        original=native.run_pair
        seen: list[str]=[]
        def wrapped(base: dict[str,Product], candidate: dict[str,Product], output: Path, *, invocation: str='development') -> Path:
            path=original(base,candidate,output,invocation=invocation)
            self.assertFalse((output/'candidate-save_load-invocation.json').exists())
            result=gate.authenticate_save_roundtrip(path,gate.SaveRoundtripProducts(base['save_load'],candidate['save_load']))
            seen.append(result)
            self.assertEqual(document((output/'candidate-save_load/process.json').read_text()).field('native_exit').value,7)
            return path
        with patch.object(fixtures,'run_pair',side_effect=wrapped):
            result=fixtures.RunnerTests().exercise('exit',invocation='gate24')
        self.assertEqual(result.field('status').value,'FAIL')
        self.assertEqual(seen,['FAIL'])

    def test_native_arm_binding_rejects_shared_root_and_wrong_source(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            with self.assertRaisesRegex(GateContractError,'independent roots'):
                gate.save_roundtrip_bindings(context)
            other=root/'other';other.mkdir();binary=other/'native';_=binary.write_text('independent')
            base=replace(context.save_roundtrip.base,root=other,binary=binary,binary_sha256=file_hash(binary))
            context=replace(context,save_roundtrip=gate.SaveRoundtripProducts(base,context.save_roundtrip.candidate))
            with self.assertRaisesRegex(GateContractError,'source identity'):
                gate.save_roundtrip_bindings(context)

    def test_completed_native_output_mutation_is_terminal_failure(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            def action(call: gate.OrderedCall, bound: gate.OrderedInputs) -> gate.Action:
                def child(out: Path) -> ComponentResult:
                    if call.step == 26:
                        _=(root/'out/step-24/save_roundtrip/result.json').write_text('changed')
                    return ordered_endpoint(call,out,bound)
                return child
            def status(path: Path, products: gate.SaveRoundtripProducts | None) -> str:
                self.assertIsNotNone(products)
                return document(path.read_text()).field('status').text()
            with patch.object(gate,'ordered_action',side_effect=action),patch.object(gate,'lane',return_value=nullcontext()), \
                 patch.object(gate,'save_roundtrip_bindings'),patch.object(gate,'authenticate_save_roundtrip',side_effect=status), \
                 patch.object(gate,'perf_bindings'),patch.object(gate,'authenticate_perf',new=routing_perf_status):
                result=document(gate.run_ordered(context,root/'out').read_text())
            self.assertEqual(result.field('status').value,'FAIL')
            self.assertTrue(any('changed' in row.text() for row in result.field('errors').array()))


    def test_reused_os_pid_does_not_alias_distinct_authenticated_invocations(self) -> None:
        from test_generic_test_history import graph
        # Identity boundary only: these modified bindings do not claim native PID evidence.
        with graph() as base,graph() as candidate:
            left=replace(base.binding(),pid=123)
            right=replace(candidate.binding(),pid=123)
            gate.separate_save_invocations(left,right)
            with self.assertRaisesRegex(GateContractError,'reused authenticated invocation'):
                gate.separate_save_invocations(left,replace(left,pid=456))


class PerfCoordinatorShapeTests(unittest.TestCase):
    def test_original_perf_nav_positions_and_unordered_exclusion(self) -> None:
        selected=[call for call in gate.ordered_calls() if call.step in (12,27)]
        self.assertEqual(selected,[gate.OrderedCall(12,'perf',('12:perf',)),gate.OrderedCall(27,'nav',('27:nav_bench',))])
        self.assertNotIn('perf',gate.COMPONENTS);self.assertNotIn('nav',gate.COMPONENTS)
        self.assertEqual(gate.endpoint('perf'),'result.json');self.assertEqual(gate.endpoint('nav'),'result.json')


class PerfCoordinatorTests(unittest.TestCase):
    def test_source_owned_authentication_and_raw_statuses(self) -> None:
        statuses: tuple[Literal['COMPONENT_PASS','FAIL','PENDING'], ...]=('COMPONENT_PASS','FAIL','PENDING')
        for status in statuses:
            with self.subTest(status=status), TemporaryDirectory() as temp:
                root=Path(temp).resolve();context=ordered_inputs(root);out=root/'perf';out.mkdir()
                path=out/'result.json';atomic_json(path,{'status':status})
                result=perf.PerfResult(path,file_hash(path),status)
                with patch.object(perf,'run_pair',return_value=path) as run, \
                     patch.object(perf,'authenticate_result',return_value=result) as authenticate:
                    self.assertEqual(gate.run_perf(context.perf,'perf.natural_quick',out),result)
                    self.assertEqual(gate.authenticate_perf(path,'perf',12,context.perf),status)
                run.assert_called_once_with('perf.natural_quick',context.perf.base,context.perf.candidate,out,
                    deadline=context.perf.deadline,purpose=context.perf.purpose)
                self.assertEqual(authenticate.call_count,2)
                authenticate.assert_called_with('perf.natural_quick',context.perf.base,context.perf.candidate,out,
                    receipt_sha256=file_hash(path),deadline=context.perf.deadline,purpose=context.perf.purpose)

    def test_foreign_endpoint_and_wrong_step_never_authenticate(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root);foreign=root/'foreign';_=foreign.write_text('{}')
            with patch.object(perf,'run_pair',return_value=foreign), \
                 patch.object(perf,'authenticate_result') as authenticate, self.assertRaises(ValueError):
                _=gate.run_perf(context.perf,'nav.soak',root/'out')
            authenticate.assert_not_called()
            with self.assertRaises(ValueError):_=gate.perf_fixture('nav',12)
            with patch.object(gate,'run_perf') as child,self.assertRaises(ValueError):
                _=gate.invoke('nav',lambda out:gate.run_perf(context.perf,'nav.soak',out),root/'wrong',
                    context.components,step=12,perf_context=context.perf)
            child.assert_not_called()

    def test_invocation_retains_failed_partial_and_pending_receipts(self) -> None:
        statuses: tuple[Literal['COMPONENT_PASS','FAIL','PENDING'], ...]=('FAIL','PENDING','COMPONENT_PASS')
        for status in statuses:
            with self.subTest(status=status),TemporaryDirectory() as temp:
                root=Path(temp).resolve();context=ordered_inputs(root);out=root/'out'
                def action(destination: Path, raw: Literal['COMPONENT_PASS','FAIL','PENDING']=status) -> perf.PerfResult:
                    destination.mkdir();_= (destination/'raw.log').write_text('partial evidence')
                    path=destination/'result.json';atomic_json(path,{'status':raw})
                    return perf.PerfResult(path,file_hash(path),raw)
                with patch.object(gate,'authenticate_perf',new=routing_perf_status):
                    row=gate.invoke('perf',action,out,context.components,step=12,perf_context=context.perf)
                self.assertEqual((row.status,row.returned_status,row.reported_status),(status,status,status))
                self.assertIn('raw.log',dict(row.files))
                _=(out/'raw.log').write_text('later component changed raw bytes')
                with self.assertRaisesRegex(ValueError,'changed'):gate.recheck(row)

    def test_bindings_reject_wrong_deadline_purpose_source_binary_and_inputs(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            products: list[Product]=[]
            for arm in ('base','candidate'):
                product_root=root/arm;product_root.mkdir()
                for name in INPUT_DIRS:(product_root/name).mkdir(parents=True,exist_ok=True)
                for name in INPUT_FILES:
                    path=product_root/name;path.parent.mkdir(parents=True,exist_ok=True);_=path.write_text('input')
                script=product_root/'tools/v8/soak_script.json';script.parent.mkdir(parents=True,exist_ok=True);_=script.write_text('same')
                binary=product_root/'worldsim_perf';_=binary.write_text(arm)
                products.append(Product(product_root,binary,'a'*40,file_hash(binary),root/'receipt','b'*64,
                    tuple(inventory(product_root,[*INPUT_DIRS,*INPUT_FILES]).items())))
            base,candidate=products
            bound=gate.PerfContext(base,candidate,123.0,'self_proof');context=replace(context,perf=bound)
            with patch.object(gate,'validate_products') as validate:
                gate.perf_bindings(context)
                validate.assert_called_once_with(base,candidate,'self_proof')
                bad=(replace(bound,deadline=float('nan')),replace(bound,deadline=True),replace(bound,purpose='publication'),
                     replace(bound,candidate=replace(candidate,source_sha='c'*40)),
                     replace(bound,candidate=replace(candidate,binary=candidate.root/'wrong')),
                     replace(bound,candidate=replace(candidate,input_files=())))
                for value in bad:
                    with self.subTest(value=value),self.assertRaises(ValueError):gate.perf_bindings(replace(context,perf=value))
                script=candidate.root/'tools/v8/soak_script.json';_=script.write_text('different')
                changed=replace(candidate,input_files=tuple(inventory(candidate.root,[*INPUT_DIRS,*INPUT_FILES]).items()))
                with self.assertRaisesRegex(ValueError,'script'):gate.perf_bindings(replace(context,perf=replace(bound,candidate=changed)))
            dependency=gate.perf_dependencies(bound)
            self.assertEqual(dependency['embedded_preparation'],'NOT_QUALIFIED')
            self.assertEqual(dependency['compile_root_authority'],'CALLER_OWNED_NOT_CERTIFIED_HERE')
