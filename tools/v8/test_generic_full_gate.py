# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Synthetic endpoint envelopes and mocked orchestration, never native proof."""
import fcntl
import json
import os
import subprocess
import sys
import time
import unittest
from contextlib import nullcontext
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Literal, override
from unittest.mock import patch

import gate_oracle_child as oracle_child
import generic_bench_run as bench
import generic_direct_run as direct
import generic_full_gate as gate
import generic_m6a_structural as structural
import generic_oracles_run as oracles
import generic_perf_run as perf
import generic_renderer_run as renderer
import generic_resolver_run as resolver
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
from generic_m6a_structural import Tooling
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
            self.assertEqual(row.status,'FAIL');self.assertIsNotNone(row.receipt,repr(row))
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
    if call.component=='static' and call.step in (8,14,16,46,51):
        discovery_process(out,data,context,status,call.step)
    atomic_json(path,data)
    return ComponentResult(path,status)


def discovery_process(out: Path, data: dict[str,Json], context: gate.OrderedInputs, status: str, step: int=16) -> None:
    product,environment=gate.static_context(context,step)
    process=out/'process';process.mkdir()
    if step==14:
        data.update(reviewed_world_scope='construction_no_tick_advance',reviewed_initial_state_mutations=True,
                    current_product_world_scope='CALLER_REQUIRED',build_provenance='CALLER_REQUIRED',
                    docs_before={'synthetic':True},docs_end={'synthetic':True},errors=[] if status=='PASS' else ['synthetic validation failure'])
        status='PASS';data['process_status']='PASS'
    _=(process/'stdout.log').write_text(strict_transcript(len(static.STRICT_FILES) if step==46 else 1) if step in (8,46) and status=='PASS' else 'READ_DIAGNOSIS_TEST PASS\n' if step==51 and status=='PASS' else 'docs: PASS\n' if step==14 else 'raw output')
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
    if step in (8,14,46):
        source: dict[str,Json]={'source_sha':product.source_sha,'root':str(product.root),'source_clean':True}
        data.update(source_before=source,source_end=source,inputs_before=dict(product.inputs),inputs_end=dict(product.inputs),
                    refs_before={},refs_end={},tool_bindings={name:{'path':str(entry),'sha256':sha} for name,entry,sha in environment.tools},
                    prior_receipts={str(entry):sha for entry,sha in environment.prior_receipts})


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


class StrictOwnedRoutingTests(unittest.TestCase):
    def test_owned_map_adds_only_exact_strict_obligations(self) -> None:
        legacy=gate.ordered_calls()
        owned=gate.ordered_owned_calls()
        self.assertEqual(tuple(row for row in owned if row not in legacy),(
            gate.OrderedCall(8,'static',('8:gdscript_strict',)),
            gate.OrderedCall(46,'static',('46:gdscript.four_files',))))
        self.assertEqual(tuple(row for row in owned if row in legacy),legacy)
        self.assertEqual(next(row.identities for row in owned if row.step==46),('46:gdscript.four_files',))


def strict_transcript(count: int) -> str:
    return f'[gdcheck] parse + warnings: {count} file(s) clean ✓\n[gdcheck] FFI bindings: 2 GDScript call(s) all match Rust #[func] exports ✓\n[gdcheck] ALL CHECKS PASS\n'


def owned_inputs(root: Path, control: int) -> gate.OwnedOrderedInputs:
    context=ordered_inputs(root)
    for name in static.STRICT_FILES:
        target=context.static_product.root/name
        target.parent.mkdir(parents=True,exist_ok=True)
        _=target.write_text('extends Node\n')
    product=replace(context.static_product,inputs=tuple(inventory(context.static_product.root,[*INPUT_DIRS,*INPUT_FILES]).items()))
    strict=gate.StrictContext(product,replace(context.environment,
        values=(*context.environment.values,('GODOT_BIN',str(context.components.godot.candidate.executable)))))
    return gate.OwnedOrderedInputs(context.components,context.static_product,context.environment,
        context.discovery_product,context.discovery_environment,context.diagnosis_product,
        context.diagnosis_environment,context.save_roundtrip,context.perf,strict,control)


def lock_contender(path: Path) -> int:
    return subprocess.run([sys.executable,'-c',
        'import fcntl,sys; f=open(sys.argv[1],"a"); fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB)',str(path)],
        stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,check=False).returncode


class StrictOwnershipTests(unittest.TestCase):
    def test_normal_completion_authenticates_before_lane_release(self) -> None:
        for status in ('PASS','FAIL'):
            with self.subTest(status=status),TemporaryDirectory() as temp:
                root=Path(temp).resolve();read,write=os.pipe()
                try:
                    context=owned_inputs(root,read);lock=root/'lane';call=gate.OrderedCall(46,'static',('46:gdscript.four_files',))
                    def child(_product: static.StaticProduct,_step: int,out: Path,env: static.ExecutionEnvironment, lock: Path=lock, call: gate.OrderedCall=call, context: gate.OwnedOrderedInputs=context, status: str=status) -> ComponentResult:
                        self.assertIsNotNone(env.held_lane_fd);self.assertNotEqual(lock_contender(lock),0)
                        return ordered_endpoint(call,out,context,status)
                    with patch.object(gate,'LOCK',lock),patch.object(gate,'strict_bindings'),patch.object(static,'validate_environment',return_value={}),patch.object(static,'run_step',side_effect=child):
                        row=gate.invoke_strict(call,context,root/'out')
                    self.assertEqual(row.status,status,row.error)
                    self.assertEqual(lock_contender(lock),0)
                    self.assertTrue(row.files)
                finally:os.close(read);os.close(write)

    def test_prelaunch_rejection_releases_without_child(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();read,write=os.pipe()
            try:
                context=owned_inputs(root,read);lock=root/'lane'
                with patch.object(gate,'LOCK',lock),patch.object(gate,'strict_bindings',side_effect=ValueError('preflight')),patch.object(static,'run_step') as child,self.assertRaises(ValueError):
                    _=gate.invoke_strict(gate.OrderedCall(8,'static',('8:gdscript_strict',)),context,root/'out')
                child.assert_not_called();self.assertEqual(lock_contender(lock),0)
            finally:os.close(read);os.close(write)

    def test_history_authentication_never_requires_live_descriptor(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=owned_inputs(root,-1)
            result=ordered_endpoint(gate.OrderedCall(46,'static',('46:gdscript.four_files',)),root/'prior',context)
            with patch.object(static,'validate_environment',side_effect=AssertionError('historical FD check')):
                self.assertEqual(gate.authenticate_static(result.receipt,46,context.strict.product,context.strict.environment),'PASS')
                bound=gate.with_static_priors(context,(gate.Outcome('static','PASS',result.receipt,file_hash(result.receipt),'',reported_status='PASS'),))
            self.assertIsInstance(bound,gate.OwnedOrderedInputs)
            if not isinstance(bound,gate.OwnedOrderedInputs):self.fail('lost strict context')
            self.assertEqual(bound.strict.environment.prior_receipts,bound.environment.prior_receipts)
            self.assertIsNone(bound.strict.environment.held_lane_fd)

    def test_owned_46_failure_blocks_all_siblings_without_changing_legacy(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();read,write=os.pipe()
            try:
                context=owned_inputs(root,read);calls=tuple(c for c in gate.ordered_owned_calls() if c.step==46)
                def child(_product: static.StaticProduct,_step: int,out: Path,_env: static.ExecutionEnvironment) -> ComponentResult:
                    return ordered_endpoint(calls[0],out,context,'FAIL')
                with patch.object(gate,'LOCK',root/'lane'),patch.object(gate,'ordered_owned_calls',return_value=calls),patch.object(gate,'ordered_bindings'),patch.object(gate,'perf_bindings'),patch.object(gate,'strict_bindings'),patch.object(static,'validate_environment',return_value={}),patch.object(static,'run_step',side_effect=child),patch.object(fixed,'run_pair',side_effect=AssertionError('blocked sibling')):
                    result=gate.run_ordered_owned(context,context.strict,root/'out',read)
                node=document(result.read_text());rows=node.field('components').array()
                self.assertEqual(rows[0].field('status').text(),'FAIL')
                self.assertTrue(all(row.field('status').text()=='NOT_RUN' for row in rows[1:]))
                self.assertFalse(node.field('full_gate_claim').value)
            finally:os.close(read);os.close(write)

    def test_unknown_actual_child_keeps_driver_lane_and_never_finalizes(self) -> None:
        for fault in ('unknown','runtime','type','interrupt','evil','status'):
            with self.subTest(fault=fault),TemporaryDirectory() as temp:
                root=Path(temp).resolve();read,write=os.pipe()
                script=r"""
import os,sys,signal,subprocess,atexit
from pathlib import Path
from unittest.mock import patch
sys.path[:0]=sys.argv[4:]
import generic_full_gate as g
import generic_static_run as s
from test_generic_full_gate import owned_inputs
root=Path(sys.argv[1]);control=int(sys.argv[2]);fault=sys.argv[3]
context=owned_inputs(root,control);children=[]
def reap():
 for child in children:
  if child.poll() is None:child.terminate()
  child.wait()
def stop(_signum,_frame):reap();os._exit(0)
atexit.register(reap);signal.signal(signal.SIGTERM,stop)
class Evil(Exception):
 def __str__(self):raise RuntimeError('unrenderable')
def child(*_args):
 process=subprocess.Popen([sys.executable,'-c','import time;time.sleep(60)']);children.append(process)
 (root/'child.pid').write_text(str(process.pid))
 if fault=='runtime':raise RuntimeError('after dispatch')
 if fault=='type':raise TypeError('after dispatch')
 if fault=='interrupt':raise KeyboardInterrupt()
 if fault=='evil':raise Evil()
 raise s.StrictOwnershipUnknown('unknown')
def bad_status(self):raise Evil()
if fault=='status':g._StrictOwner.status=bad_status
calls=(g.OrderedCall(8,'static',('8:gdscript_strict',)),g.OrderedCall(9,'smoke',('9:smoke',)))
with patch.object(g,'LOCK',root/'lane'),patch.object(g,'ordered_owned_calls',return_value=calls),patch.object(g,'ordered_bindings'),patch.object(g,'perf_bindings'),patch.object(g,'strict_bindings'),patch.object(s,'validate_environment',return_value={}),patch.object(s,'run_step',side_effect=child),patch.object(g,'invoke',side_effect=AssertionError('unexpected snapshot/nextcall')):
 g.run_ordered_owned(context,context.strict,root/'out',control)
raise AssertionError('unknown unwound')
"""
                command=[sys.executable,'-B','-c',script,str(root),str(read),fault,str(Path(__file__).resolve().parent),str(Path(gate.__file__).resolve().parent),str(Path(static.__file__).resolve().parent)]
                with subprocess.Popen(command,pass_fds=(read,),env={k:v for k,v in os.environ.items() if k!='PYTHONPATH'},stdout=subprocess.PIPE,stderr=subprocess.PIPE) as driver:
                    os.close(read)
                    try:
                        deadline=time.monotonic()+10
                        while not (root/'child.pid').exists() and driver.poll() is None and time.monotonic()<deadline:time.sleep(0.01)
                        if not (root/'child.pid').exists():
                            if driver.poll() is None:driver.terminate()
                            stdout,stderr=driver.communicate(timeout=5)
                            self.fail(f'startup: {stdout!r} {stderr!r}')
                        _=os.write(write,b'inspect\n');os.close(write);write=-1;time.sleep(0.1)
                        self.assertIsNone(driver.poll());os.kill(int((root/'child.pid').read_text()),0)
                        self.assertNotEqual(lock_contender(root/'lane'),0)
                        self.assertFalse((root/'out/ordered-dispatch.json').exists())
                        self.assertFalse((root/'out/step-09').exists())
                    finally:
                        if write>=0:os.close(write)
                        if driver.poll() is None:driver.terminate()
                        _=driver.wait(timeout=5)
                self.assertEqual(lock_contender(root/'lane'),0)


class StrictPriorRejectionTests(unittest.TestCase):
    def test_signal_exit_cannot_be_resealed_as_normal_failed_prior(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=owned_inputs(root,-1)
            result=ordered_endpoint(gate.OrderedCall(46,'static',('46:gdscript.four_files',)),root/'out',context,'FAIL')
            receipt=document(result.receipt.read_text()).mapping().copy()
            process=result.receipt.parent/'process/process.json'
            native=document(process.read_text()).mapping().copy();native['native_exit']=-9
            process.unlink();atomic_json(process,native)
            receipt['native_exit']=-9;receipt['process_receipt_sha256']=file_hash(process)
            receipt['artifacts']={name:digest for name,digest in inventory(result.receipt.parent,['process']).items()}
            result.receipt.unlink();atomic_json(result.receipt,receipt)
            with self.assertRaises(GateContractError):
                _=gate.authenticate_static(result.receipt,46,context.strict.product,context.strict.environment)

    def test_missing_control_pipe_rejects_before_any_dispatch(self) -> None:
        with TemporaryDirectory() as temp:
            context=owned_inputs(Path(temp).resolve(),-1)
            with patch.object(gate,'_run_ordered') as run,self.assertRaises(GateContractError):
                _=gate.run_ordered_owned(context,context.strict,Path(temp).resolve()/'out',-1)
            run.assert_not_called()


class StrictHistoricalChainTests(unittest.TestCase):
    def chain(self, root: Path, status: str = 'PASS') -> tuple[gate.OwnedOrderedInputs,gate.Outcome,gate.Outcome]:
        original=owned_inputs(root,-1)
        _=(original.strict.product.root/'scripts/a.gd').write_text('extends Node\n')
        product=replace(original.strict.product,inputs=tuple(inventory(original.strict.product.root,[*INPUT_DIRS,*INPUT_FILES]).items()))
        original=replace(original,strict=replace(original.strict,product=product))
        first=ordered_endpoint(gate.OrderedCall(1,'static',('1:fmt',)),root/'first',original)
        prior=gate.Outcome('static','PASS',first.receipt,file_hash(first.receipt),'',reported_status='PASS')
        selected=gate.with_static_priors(original,(prior,))
        result=ordered_endpoint(gate.OrderedCall(8,'static',('8:gdscript_strict',)),root/'strict',selected,status)
        row=gate.Outcome('static',status,result.receipt,file_hash(result.receipt),'',reported_status=status)
        return original,prior,row

    def test_nonempty_historical_prefix_survives_next_selection(self) -> None:
        with TemporaryDirectory() as temp:
            original,prior,row=self.chain(Path(temp).resolve())
            selected=gate.with_static_priors(original,(prior,row))
            self.assertIsInstance(selected,gate.OwnedOrderedInputs)
            if not isinstance(selected,gate.OwnedOrderedInputs):self.fail('lost owned context')
            self.assertEqual(len(selected.strict.environment.prior_receipts),2)
            again=gate.with_static_priors(selected,(prior,row))
            self.assertEqual(again,selected)
            with patch.object(static,'strict_input_names',return_value=[*INPUT_DIRS,*INPUT_FILES]):
                gate.strict_bindings(selected)
            later=ordered_endpoint(gate.OrderedCall(13,'static',('13:bridge_api',)),Path(temp).resolve()/'later',selected)
            later_row=gate.Outcome('static','PASS',later.receipt,file_hash(later.receipt),'',reported_status='PASS')
            selected=gate.with_static_priors(original,(prior,row,later_row))
            final=ordered_endpoint(gate.OrderedCall(46,'static',('46:gdscript.four_files',)),Path(temp).resolve()/'final',selected)
            final_row=gate.Outcome('static','PASS',final.receipt,file_hash(final.receipt),'',reported_status='PASS')
            selected=gate.with_static_priors(original,(prior,row,later_row,final_row))
            if not isinstance(selected,gate.OwnedOrderedInputs):self.fail('lost strict context')
            with patch.object(static,'strict_input_names',return_value=[*INPUT_DIRS,*INPUT_FILES]):
                gate.strict_bindings(selected)
            self.assertEqual(len(selected.strict.environment.prior_receipts),4)

    def test_resealed_missing_extra_corrupt_prefix_is_rejected(self) -> None:
        for status,defect in ((status,defect) for status in ('PASS','FAIL') for defect in ('missing','extra','corrupt')):
            with self.subTest(status=status,defect=defect),TemporaryDirectory() as temp:
                original,prior,row=self.chain(Path(temp).resolve(),status)
                if row.receipt is None or prior.receipt is None:self.fail('fixture lacks receipt')
                node=document(row.receipt.read_text()).mapping().copy()
                prefix=node['prior_receipts']
                if not isinstance(prefix,dict):self.fail('fixture prior type')
                if defect=='missing':prefix.clear()
                elif defect=='extra':prefix['/unadmitted/extra.json']='a'*64
                else:prefix[str(prior.receipt)]='c'*64
                row.receipt.unlink();atomic_json(row.receipt,node)
                row=replace(row,sha256=file_hash(row.receipt))
                with self.assertRaises(ValueError):_=gate.with_static_priors(original,(prior,row))

    def test_omitted_process_link_cannot_hide_retained_unknown_shell(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=owned_inputs(root,-1)
            result=ordered_endpoint(gate.OrderedCall(46,'static',('46:gdscript.four_files',)),root/'failed',context,'FAIL')
            node=document(result.receipt.read_text()).mapping().copy()
            for name in ('process_receipt','process_receipt_sha256','native_exit','pid'):_=node.pop(name)
            result.receipt.unlink();atomic_json(result.receipt,node)
            with self.assertRaises(GateContractError):
                _=gate.authenticate_static(result.receipt,46,context.strict.product,context.strict.environment)

    def test_preflight_without_process_tree_is_still_a_failed_prior(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=owned_inputs(root,-1)
            result=ordered_endpoint(gate.OrderedCall(46,'static',('46:gdscript.four_files',)),root/'failed',context,'FAIL')
            node=document(result.receipt.read_text()).mapping().copy()
            for name in ('process_receipt','process_receipt_sha256','native_exit','pid'):_=node.pop(name)
            import shutil
            shutil.rmtree(result.receipt.parent/'process')
            node['artifacts']={};node['process_status']='NOT_RUN'
            result.receipt.unlink();atomic_json(result.receipt,node)
            self.assertEqual(gate.authenticate_static(result.receipt,46,context.strict.product,context.strict.environment),'FAIL')


class BenchOwnedRoutingTests(unittest.TestCase):
    def test_optional_bench_adds_only_exact25(self) -> None:
        original=gate.ordered_owned_calls()
        selected=gate.ordered_owned_calls(bench_enabled=True)
        self.assertEqual(tuple(c for c in selected if c not in original),(gate.OrderedCall(25,'bench',('25:bench',)),))
        self.assertEqual(tuple(c for c in selected if c in original),original)
        self.assertNotIn(25,[c.step for c in gate.ordered_calls()])


def bench_owned_inputs(root: Path, control: int) -> gate.OwnedOrderedInputs:
    context=owned_inputs(root,control)
    products: list[Product]=[]
    for probe in (context.components.probes.base,context.components.probes.candidate):
        for name in INPUT_DIRS:(probe.input_root/name).mkdir(parents=True,exist_ok=True)
        for name in INPUT_FILES:
            path=probe.input_root/name;path.parent.mkdir(parents=True,exist_ok=True)
            if not path.exists():_=path.write_text('input')
        script=probe.input_root/'tools/v8/bench_script.json';script.parent.mkdir(parents=True,exist_ok=True)
        import generic_fixture_registry as registry_module
        _=script.write_bytes(Path(registry_module.__file__).with_name('bench_script.json').read_bytes())
        binary=probe.input_root/'worldsim_perf';_=binary.write_text('synthetic executable');binary.chmod(0o755)
        products.append(Product(probe.input_root,binary,probe.source_sha,file_hash(binary),root/'not-certified-build','b'*64,
                                tuple(inventory(probe.input_root,[*INPUT_DIRS,*INPUT_FILES]).items())))
    script=products[1].root/'tools/v8/bench_script.json'
    return replace(context,bench=gate.BenchContext(products[0],products[1],'self_proof',script,file_hash(script)))


class BenchBindingsTests(unittest.TestCase):
    def test_exact_source_script_and_registry_join(self) -> None:
        with TemporaryDirectory() as temp:
            context=bench_owned_inputs(Path(temp).resolve(),-1)
            self.assertIsNotNone(context.bench)
            assert context.bench is not None
            with patch.object(gate,'validate_products'):
                self.assertEqual(gate.bench_bindings(context),context.bench)
                for bad in (replace(context.bench,script=context.bench.candidate.root/'tools/v8/u_script.json'),
                            replace(context.bench,script_sha256='a'*64),
                            replace(context.bench,candidate=replace(context.bench.candidate,source_sha='f'*40)),
                            replace(context.bench,candidate=replace(context.bench.candidate,input_files=()))):
                    with self.assertRaises(ValueError):_=gate.bench_bindings(replace(context,bench=bad))
                changed=tuple(row.replace('bench_script.json','u_script.json') for row in context.components.registry.backlog)
                wrong=replace(context,components=replace(context.components,registry=replace(context.components.registry,backlog=changed)))
                with self.assertRaises(ValueError):_=gate.bench_bindings(wrong)


class BenchCoordinatorLifecycleTests(unittest.TestCase):
    def test_settled_success_failure_and_pending_release_one_lane(self) -> None:
        for state in ('COMPONENT_PASS','FAIL','PENDING'):
            with self.subTest(state=state),TemporaryDirectory() as temp:
                root=Path(temp).resolve();read,write=os.pipe()
                try:
                    context=bench_owned_inputs(root,read);out=root/'out';lock=root/'lane'
                    def producer(_base: Product,_candidate: Product,_purpose: str,_script: Path,_digest: str,output: Path,
                                 _quotes: tuple[dict[str,str],...],state: str=state,lock: Path=lock) -> dict[str,Json]:
                        self.assertNotEqual(lock_contender(lock),0)
                        output.mkdir()
                        if state=='FAIL':raise bench.BenchSettledFailure(ValueError('raw failure'),None,None)
                        if state!='PENDING':_= (output/'result.json').write_text('{}')
                        return {}
                    def auth(_base: Product,_candidate: Product,_purpose: str,_script: Path,_digest: str,output: Path,
                             *,receipt_sha256: str | None,justifications: tuple[dict[str,str],...],state: str=state,lock: Path=lock) -> bench.BenchResult:
                        self.assertNotEqual(lock_contender(lock),0)
                        del justifications
                        return bench.BenchResult(output/'result.json' if state=='COMPONENT_PASS' else None,
                            receipt_sha256,state if state in ('COMPONENT_PASS','FAIL','PENDING') else 'FAIL')
                    with patch.object(gate,'LOCK',lock),patch.object(gate,'validate_products'),patch.object(bench,'run_bench_owned',side_effect=producer),patch.object(bench,'authenticate_result',side_effect=auth):
                        row=gate.invoke_bench(gate.OrderedCall(25,'bench',('25:bench',)),context,out)
                    self.assertEqual(row.status,state)
                    self.assertEqual(lock_contender(lock),0)
                    if state=='FAIL':self.assertIn('settled benchmark failure',row.error)
                finally:os.close(read);os.close(write)

    def test_preflight_rejection_releases_without_producer(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();read,write=os.pipe()
            try:
                context=bench_owned_inputs(root,read)
                with patch.object(gate,'LOCK',root/'lane'),patch.object(gate,'bench_bindings',side_effect=ValueError('preflight')),patch.object(bench,'run_bench_owned') as producer,self.assertRaises(ValueError):
                    _=gate.invoke_bench(gate.OrderedCall(25,'bench',('25:bench',)),context,root/'out')
                producer.assert_not_called();self.assertEqual(lock_contender(root/'lane'),0)
            finally:os.close(read);os.close(write)

    def test_any_nonpass_bench_blocks_every_later_call(self) -> None:
        for state in ('FAIL','PENDING'):
            with self.subTest(state=state),TemporaryDirectory() as temp:
                root=Path(temp).resolve();read,write=os.pipe()
                try:
                    context=bench_owned_inputs(root,read)
                    calls=tuple(c for c in gate.ordered_owned_calls(bench_enabled=True) if gate.ORDER.index(c.step)>=gate.ORDER.index(25))
                    with patch.object(gate,'ordered_owned_calls',return_value=calls),patch.object(gate,'ordered_bindings'),patch.object(gate,'perf_bindings'),patch.object(gate,'bench_bindings',return_value=context.bench),patch.object(gate,'invoke_bench',return_value=gate.Outcome('bench',state,None,None,'')) as invoke_bench,patch.object(gate,'ordered_action',side_effect=AssertionError('downstream attempted')),patch.object(gate,'lane',side_effect=AssertionError('nested lane')):
                        path=gate.run_ordered_owned(context,context.strict,root/'out',read,bench=context.bench)
                    invoke_bench.assert_called_once()
                    result=document(path.read_text());result.field('actual_component_sequence').equals(['25:bench'])
                    for node in result.field('components').array()[1:]:
                        node.field('status').equals('NOT_RUN');node.field('error').equals('blocked by 25:bench')
                    self.assertFalse((root/'out/step-26').exists())
                    for node in result.field('obligations').array():
                        if gate.ORDER.index(integer(node.field('step')))>gate.ORDER.index(25):
                            node.field('status').equals('NOT_RUN');node.field('blocked_by').equals('25:bench')
                    self.assertNotEqual(result.field('status').text(),'PASS')
                finally:os.close(read);os.close(write)

    def test_unknown_keeps_live_lock_through_controller_eof_and_faults(self) -> None:
        for fault in ('unknown','type','interrupt','evil','status'):
            with self.subTest(fault=fault),TemporaryDirectory() as temp:
                root=Path(temp).resolve();read,write=os.pipe()
                script=r"""
import os,sys,signal
from pathlib import Path
from unittest.mock import patch
sys.path[:0]=sys.argv[4:]
import generic_full_gate as g
import generic_bench_run as b
from test_generic_full_gate import bench_owned_inputs
root=Path(sys.argv[1]);control=int(sys.argv[2]);fault=sys.argv[3]
context=bench_owned_inputs(root,control)
signal.signal(signal.SIGTERM,lambda _a,_b:os._exit(0))
class Evil(Exception):
 def __str__(self):raise AssertionError('never render unknown')
def child(*_args):
 (root/'entered').write_text('attempted')
 if fault=='type':raise TypeError('after execute entered')
 if fault=='interrupt':raise KeyboardInterrupt()
 if fault=='evil':raise Evil()
 raise b.BenchOwnershipUnknown(ValueError('unknown aggregate'),None,None)
def bad_status(self):raise Evil()
if fault=='status':g._BenchOwner.status=bad_status
calls=(g.OrderedCall(25,'bench',('25:bench',)),g.OrderedCall(26,'direct',('26:economy.soak_no_beasts',)))
with patch.object(g,'LOCK',root/'lane'),patch.object(g,'ordered_owned_calls',return_value=calls),patch.object(g,'ordered_bindings'),patch.object(g,'perf_bindings'),patch.object(g,'bench_bindings',return_value=context.bench),patch.object(b,'run_bench_owned',side_effect=child),patch.object(g,'bench_outcome',side_effect=AssertionError('unexpected snapshot')),patch.object(g,'ordered_action',side_effect=AssertionError('downstream')):
 g.run_ordered_owned(context,context.strict,root/'out',control,bench=context.bench)
raise AssertionError('unknown unwound')
"""
                command=[sys.executable,'-B','-c',script,str(root),str(read),fault,str(Path(__file__).resolve().parent),str(Path(gate.__file__).resolve().parent),str(Path(bench.__file__).resolve().parent),str(Path(static.__file__).resolve().parent)]
                with subprocess.Popen(command,pass_fds=(read,),env={k:v for k,v in os.environ.items() if k!='PYTHONPATH'},stdout=subprocess.PIPE,stderr=subprocess.PIPE) as driver:
                    os.close(read)
                    try:
                        deadline=time.monotonic()+10
                        while not (root/'entered').exists() and driver.poll() is None and time.monotonic()<deadline:time.sleep(0.01)
                        if not (root/'entered').exists():
                            if driver.poll() is None:driver.terminate()
                            stdout,stderr=driver.communicate(timeout=5)
                            self.fail(f'startup: {stdout!r} {stderr!r}')
                        _=os.write(write,b'inspect\n');os.close(write);write=-1;time.sleep(0.1)
                        self.assertIsNone(driver.poll())
                        self.assertNotEqual(lock_contender(root/'lane'),0)
                        self.assertFalse((root/'out/ordered-dispatch.json').exists())
                        self.assertFalse((root/'out/step-26').exists())
                        self.assertFalse((root/'out/step-25/bench/result.json').exists())
                    finally:
                        if write>=0:os.close(write)
                        if driver.poll() is None:driver.terminate()
                        _=driver.wait(timeout=5)
                self.assertEqual(lock_contender(root/'lane'),0)

    def test_success_continues_at26_without_second_bench_lease(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();read,write=os.pipe()
            try:
                context=bench_owned_inputs(root,read)
                calls=(gate.OrderedCall(25,'bench',('25:bench',)),gate.OrderedCall(26,'direct',('26:economy.soak_no_beasts',)))
                with patch.object(gate,'ordered_owned_calls',return_value=calls),patch.object(gate,'ordered_bindings'),patch.object(gate,'perf_bindings'),patch.object(gate,'bench_bindings',return_value=context.bench),patch.object(gate,'invoke_bench',return_value=gate.Outcome('bench','COMPONENT_PASS',None,None,'')) as first,patch.object(gate,'invoke',return_value=gate.Outcome('direct','PASS',None,None,'')) as second,patch.object(gate,'lane',return_value=nullcontext()) as lease:
                    path=gate.run_ordered_owned(context,context.strict,root/'out',read,bench=context.bench)
                first.assert_called_once();second.assert_called_once();lease.assert_called_once()
                result=document(path.read_text());result.field('actual_component_sequence').equals(['25:bench','26:direct'])
                result.field('status').equals('INCOMPLETE')
            finally:os.close(read);os.close(write)

    def test_raw_pass_dictionary_cannot_admit_bench_result(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=bench_owned_inputs(root,-1)
            assert context.bench is not None
            out=root/'out';out.mkdir();_=(out/'result.json').write_text('{"status":"PASS"}')
            row=gate.bench_outcome(context.bench,out,failed=False)
            self.assertEqual(row.status,'FAIL')
            self.assertEqual(row.receipt,out/'result.json')
            self.assertTrue(row.error)


class DocsOwnedRoutingTests(unittest.TestCase):
    def test_optional_docs_adds_only_original14_and_preserves_bench(self) -> None:
        before=gate.ordered_owned_calls(bench_enabled=True)
        after=gate.ordered_owned_calls(bench_enabled=True,docs_enabled=True)
        self.assertEqual(tuple(row for row in after if row not in before),(gate.OrderedCall(14,'static',('14:docs',)),))
        self.assertEqual(tuple(row for row in after if row in before),before)
        self.assertNotIn(14,[row.step for row in gate.ordered_calls()])


def docs_owned_inputs(root: Path, control: int) -> gate.OwnedOrderedInputs:
    context=owned_inputs(root,control)
    _=(context.strict.product.root/'scripts/a.gd').write_text('extends Node\n')
    context=replace(context,strict=replace(context.strict,product=replace(context.strict.product,inputs=tuple(inventory(context.strict.product.root,[*INPUT_DIRS,*INPUT_FILES]).items()))))
    product=replace(context.discovery_product,inputs=tuple(inventory(context.discovery_product.root,[*INPUT_DIRS,*INPUT_FILES,'docs']).items()))
    return replace(context,docs=gate.DocsContext(product,context.discovery_environment))


class DocsCoordinatorTests(unittest.TestCase):
    def test_actual_schema_pass_and_settled_fail_with_bound_prior_prefix(self) -> None:
        for state in ('PASS','FAIL'):
            with self.subTest(state=state),TemporaryDirectory() as temp:
                root=Path(temp).resolve();original=docs_owned_inputs(root,-1)
                prior=ordered_endpoint(gate.OrderedCall(13,'static',('13:bridge_api',)),root/'prior',original)
                prior_row=gate.Outcome('static','PASS',prior.receipt,file_hash(prior.receipt),'',reported_status='PASS')
                selected=gate.with_static_priors(original,(prior_row,))
                result=ordered_endpoint(gate.OrderedCall(14,'static',('14:docs',)),root/'docs',selected,state)
                row=gate.Outcome('static',state,result.receipt,file_hash(result.receipt),'',reported_status=state)
                with patch.object(static,'docs_bindings',return_value={'synthetic':True}):
                    product,environment=gate.static_context(selected,14)
                    self.assertEqual(gate.authenticate_static(result.receipt,14,product,environment),state)
                    joined=gate.with_static_priors(original,(prior_row,row))
                    self.assertIsInstance(joined,gate.OwnedOrderedInputs)
                    assert isinstance(joined,gate.OwnedOrderedInputs) and joined.docs is not None
                    self.assertEqual(len(joined.docs.environment.prior_receipts),2)
                    self.assertEqual(joined.docs.environment.prior_receipts,joined.strict.environment.prior_receipts)
                    self.assertEqual(gate.with_static_priors(joined,(prior_row,row)),joined)

    def test_resealed_bad_native_or_prefix_rejected_as_completed_prior(self) -> None:
        for fault in ('exit','pid','duration','spawn','prefix','classification'):
            with self.subTest(fault=fault),TemporaryDirectory() as temp:
                root=Path(temp).resolve();context=docs_owned_inputs(root,-1)
                result=ordered_endpoint(gate.OrderedCall(14,'static',('14:docs',)),root/'docs',context,'FAIL')
                receipt=document(result.receipt.read_text()).mapping()
                path=root/'docs/process/process.json';native=document(path.read_text()).mapping()
                if fault=='exit':native['native_exit']=1;receipt['native_exit']=1
                elif fault=='pid':native['pid']=None;receipt['pid']=None
                elif fault=='duration':native['duration_ns']=0
                elif fault=='spawn':(path.parent/'process-spawn.json').unlink()
                elif fault=='prefix':receipt['prior_receipts']={'/foreign':'a'*64}
                else:receipt['current_product_world_scope']='QUALIFIED'
                _=path.write_text(json.dumps(native));receipt['process_receipt_sha256']=file_hash(path)
                receipt['artifacts']=dict(inventory(result.receipt.parent,['process']))
                _=result.receipt.write_text(json.dumps(receipt))
                product,environment=gate.static_context(context,14)
                with patch.object(static,'docs_bindings',return_value={'synthetic':True}),self.assertRaises((OSError,ValueError)):
                    _=gate.authenticate_static(result.receipt,14,product,environment)

    def test_docs_context_scope_and_candidate_source_are_checked(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=docs_owned_inputs(root,-1)
            assert context.docs is not None
            with patch.object(static,'docs_input_names',return_value=[*INPUT_DIRS,*INPUT_FILES,'docs']),patch.object(static,'docs_bindings',return_value={'synthetic':True}):
                self.assertEqual(gate.docs_bindings(context),context.docs)
                for changed in (replace(context.docs,product=replace(context.docs.product,source_sha='f'*40)),
                                replace(context.docs,product=replace(context.docs.product,inputs=())),
                                replace(context.docs,environment=replace(context.docs.environment,held_lane_fd=123))):
                    with self.assertRaises(ValueError):_=gate.docs_bindings(replace(context,docs=changed))

    def test_one_lane_success_and_typed_settled_failure_release(self) -> None:
        for state in ('PASS','FAIL'):
            with self.subTest(state=state),TemporaryDirectory() as temp:
                root=Path(temp).resolve();read,write=os.pipe()
                try:
                    context=docs_owned_inputs(root,read);lock=root/'lane'
                    def child(_product: static.StaticProduct,out: Path,env: static.ExecutionEnvironment,state: str=state,lock: Path=lock,context: gate.OwnedOrderedInputs=context) -> ComponentResult:
                        self.assertNotEqual(lock_contender(lock),0);self.assertIsNotNone(env.held_lane_fd)
                        result=ordered_endpoint(gate.OrderedCall(14,'static',('14:docs',)),out,context,state)
                        if state=='FAIL':raise static.DocsSettledFailure(ValueError('validation failed'),None,result.receipt)
                        return result
                    with patch.object(gate,'LOCK',lock),patch.object(gate,'docs_bindings',return_value=context.docs),patch.object(static,'docs_bindings',return_value={'synthetic':True}),patch.object(static,'validate_environment',return_value={}),patch.object(static,'run_docs_owned',side_effect=child):
                        row=gate.invoke_docs(gate.OrderedCall(14,'static',('14:docs',)),context,root/'out')
                    self.assertEqual(row.status,state);self.assertEqual(row.reported_status,state)
                    self.assertEqual(lock_contender(lock),0)
                finally:os.close(read);os.close(write)

    def test_settled_failure_does_not_add_step25_style_tailstop(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();read,write=os.pipe()
            try:
                context=docs_owned_inputs(root,read)
                calls=(gate.OrderedCall(14,'static',('14:docs',)),gate.OrderedCall(15,'static',('15:inventory_closure',)))
                with patch.object(gate,'ordered_owned_calls',return_value=calls),patch.object(gate,'ordered_bindings'),patch.object(gate,'perf_bindings'),patch.object(gate,'docs_bindings',return_value=context.docs),patch.object(static,'docs_bindings',return_value={'synthetic':True}),patch.object(gate,'invoke_docs',return_value=gate.Outcome('static','FAIL',None,None,'settled failure')),patch.object(gate,'invoke',return_value=gate.Outcome('static','PASS',None,None,'')) as later,patch.object(gate,'lane',side_effect=AssertionError('nested lease')):
                    path=gate.run_ordered_owned(context,context.strict,root/'out',read,docs=context.docs)
                later.assert_called_once();result=document(path.read_text())
                result.field('actual_component_sequence').equals(['14:static','15:static']);result.field('status').equals('FAIL')
            finally:os.close(read);os.close(write)

    def test_historical_chain_through_strict_docs_discovery_and_diagnosis(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=docs_owned_inputs(root,-1);rows: list[gate.Outcome]=[]
            with patch.object(static,'docs_bindings',return_value={'synthetic':True}):
                for step,identity in ((8,'gdscript_strict'),(14,'docs'),(16,'python_unittest'),(46,'gdscript.four_files'),(51,'read_diagnosis_test')):
                    selected=gate.with_static_priors(context,tuple(rows));status='FAIL' if step==14 else 'PASS'
                    result=ordered_endpoint(gate.OrderedCall(step,'static',(f'{step}:{identity}',)),root/f'step-{step}',selected,status)
                    rows.append(gate.Outcome('static',status,result.receipt,file_hash(result.receipt),'',reported_status=status))
                final_context=gate.with_static_priors(context,tuple(rows))
                assert isinstance(final_context,gate.OwnedOrderedInputs) and final_context.docs is not None
                self.assertEqual(len(final_context.docs.environment.prior_receipts),5)
                self.assertEqual(final_context.docs.environment.prior_receipts,final_context.diagnosis_environment.prior_receipts)

    def test_missing_settled_diagnostic_is_fail_and_releases(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();read,write=os.pipe()
            try:
                context=docs_owned_inputs(root,read)
                failure=static.DocsSettledFailure(ValueError('prelaunch validation'),OSError('write failure'),None)
                with patch.object(gate,'LOCK',root/'lane'),patch.object(gate,'docs_bindings',return_value=context.docs),patch.object(static,'validate_environment',return_value={}),patch.object(static,'run_docs_owned',side_effect=failure):
                    row=gate.invoke_docs(gate.OrderedCall(14,'static',('14:docs',)),context,root/'out')
                self.assertEqual(row.status,'FAIL');self.assertIsNone(row.receipt)
                self.assertEqual(lock_contender(root/'lane'),0)
            finally:os.close(read);os.close(write)

    def test_unknown_docs_retains_actual_lane_and_never_continues15(self) -> None:
        for fault in ('native','type','interrupt','status'):
            with self.subTest(fault=fault),TemporaryDirectory() as temp:
                root=Path(temp).resolve();read,write=os.pipe()
                script=r"""
import os,sys,signal
from pathlib import Path
from unittest.mock import patch
sys.path[:0]=sys.argv[4:]
import generic_full_gate as g
import generic_static_run as s
from test_generic_full_gate import docs_owned_inputs
root=Path(sys.argv[1]);control=int(sys.argv[2]);fault=sys.argv[3]
context=docs_owned_inputs(root,control)
signal.signal(signal.SIGTERM,lambda _a,_b:os._exit(0))
class Evil(Exception):
 def __str__(self):raise AssertionError('must not render unknown')
def child(*_args):
 (root/'entered').write_text('attempted')
 if fault=='type':raise TypeError('after dispatch')
 if fault=='interrupt':raise KeyboardInterrupt()
 raise s.DocsOwnershipUnknown(ValueError('native failed; waits unresolved'),None,None)
def bad_status(self):raise Evil()
if fault=='status':g._DocsOwner.status=bad_status
calls=(g.OrderedCall(14,'static',('14:docs',)),g.OrderedCall(15,'static',('15:inventory_closure',)))
with patch.object(g,'LOCK',root/'lane'),patch.object(g,'ordered_owned_calls',return_value=calls),patch.object(g,'ordered_bindings'),patch.object(g,'perf_bindings'),patch.object(g,'docs_bindings',return_value=context.docs),patch.object(s,'docs_bindings',return_value={'synthetic':True}),patch.object(s,'validate_environment',return_value={}),patch.object(s,'run_docs_owned',side_effect=child),patch.object(g,'invoke',side_effect=AssertionError('snapshot/downstream')):
 g.run_ordered_owned(context,context.strict,root/'out',control,docs=context.docs)
raise AssertionError('unknown returned')
"""
                command=[sys.executable,'-B','-c',script,str(root),str(read),fault,str(Path(__file__).resolve().parent),str(Path(gate.__file__).resolve().parent),str(Path(static.__file__).resolve().parent),str(Path(direct.__file__).resolve().parent)]
                with subprocess.Popen(command,pass_fds=(read,),env={k:v for k,v in os.environ.items() if k!='PYTHONPATH'},stdout=subprocess.PIPE,stderr=subprocess.PIPE) as driver:
                    os.close(read)
                    try:
                        deadline=time.monotonic()+10
                        while not (root/'entered').exists() and driver.poll() is None and time.monotonic()<deadline:time.sleep(0.01)
                        if not (root/'entered').exists():
                            if driver.poll() is None:driver.terminate()
                            stdout,stderr=driver.communicate(timeout=5)
                            self.fail(f'startup: {stdout!r} {stderr!r}')
                        _=os.write(write,b'inspect\n');os.close(write);write=-1;time.sleep(0.1)
                        self.assertIsNone(driver.poll());self.assertNotEqual(lock_contender(root/'lane'),0)
                        self.assertFalse((root/'out/ordered-dispatch.json').exists());self.assertFalse((root/'out/step-15').exists())
                        if fault!='status':
                            status_path=root/'out/step-14/docs-ownership-1.json'
                            document(status_path.read_text()).field('schema').equals('docs_ordered_live_owner_v1')
                    finally:
                        if write>=0:os.close(write)
                        if driver.poll() is None:driver.terminate()
                        _=driver.wait(timeout=5)
                self.assertEqual(lock_contender(root/'lane'),0)

    def test_no_launch_failure_cannot_hide_unjoined_native_fields(self) -> None:
        import shutil
        defects: tuple[tuple[str,Json],...]=(('process_status','PASS'),('pid',123),('native_exit',1),('process_receipt_sha256','a'*64))
        for field,value in defects:
            with self.subTest(field=field),TemporaryDirectory() as temp:
                root=Path(temp).resolve();context=docs_owned_inputs(root,-1)
                result=ordered_endpoint(gate.OrderedCall(14,'static',('14:docs',)),root/'docs',context,'FAIL')
                node=document(result.receipt.read_text()).mapping()
                for name in ('process_receipt','process_receipt_sha256','pid','native_exit'):_=node.pop(name)
                shutil.rmtree(result.receipt.parent/'process')
                node.update(process_status='NOT_RUN',artifacts={})
                node[field]=value;_=result.receipt.write_text(json.dumps(node))
                product,environment=gate.static_context(context,14)
                with self.assertRaises(ValueError):_=gate.authenticate_static(result.receipt,14,product,environment)

    def test_settled_failure_retains_safe_original_and_secondary_diagnostics(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();read,write=os.pipe()
            try:
                context=docs_owned_inputs(root,read)
                failure=static.DocsSettledFailure(ValueError('original-marker'),OSError('diagnostic-marker'),None)
                with patch.object(gate,'LOCK',root/'lane'),patch.object(gate,'docs_bindings',return_value=context.docs),patch.object(static,'validate_environment',return_value={}),patch.object(static,'run_docs_owned',side_effect=failure):
                    row=gate.invoke_docs(gate.OrderedCall(14,'static',('14:docs',)),context,root/'out')
                self.assertEqual(row.status,'FAIL')
                for text in ('ValueError','original-marker','OSError','diagnostic-marker'):self.assertIn(text,row.error)
                self.assertEqual(lock_contender(root/'lane'),0)
            finally:os.close(read);os.close(write)

    def test_outer_failed_docs_requires_native_pass_endpoint_consistency(self) -> None:
        for field in ('before_inputs','after_inputs','before_executable_sha256','after_executable_sha256','before_external_inputs','after_external_inputs'):
            with self.subTest(field=field),TemporaryDirectory() as temp:
                root=Path(temp).resolve();context=docs_owned_inputs(root,-1)
                result=ordered_endpoint(gate.OrderedCall(14,'static',('14:docs',)),root/'docs',context,'FAIL')
                receipt=document(result.receipt.read_text()).mapping();path=result.receipt.parent/'process/process.json'
                native=document(path.read_text()).mapping();native[field]='a'*64 if 'executable' in field else {}
                _=path.write_text(json.dumps(native));receipt['process_receipt_sha256']=file_hash(path)
                receipt['artifacts']=dict(inventory(result.receipt.parent,['process']));_=result.receipt.write_text(json.dumps(receipt))
                product,environment=gate.static_context(context,14)
                with self.assertRaises(ValueError):_=gate.authenticate_static(result.receipt,14,product,environment)

    def test_diagnostics_do_not_render_unknown_arguments_or_properties(self) -> None:
        from unittest.mock import PropertyMock
        class Hostile:
            pass
        class FailureSubtype(static.DocsSettledFailure):
            pass
        value=Hostile()
        with patch.object(Hostile,'__str__',side_effect=AssertionError('str invoked')),patch.object(Hostile,'__repr__',side_effect=AssertionError('repr invoked')):
            message=gate.docs_failure_message(static.DocsSettledFailure(ValueError('safe-marker',value),None,None))
        self.assertIn('safe-marker',message);self.assertIn('<uninspected argument>',message)
        failure=FailureSubtype(ValueError('primary'),None,None)
        with patch.object(FailureSubtype,'original',create=True,new_callable=PropertyMock,side_effect=AssertionError('property invoked')):
            self.assertIn('uninspected failure subtype',gate.docs_failure_message(failure))
        self.assertIn('uninspected exception subtype',gate.safe_docs_exception(failure))


def oracle_context(root: Path, control: int) -> gate.OwnedOrderedInputs:
    root.mkdir(exist_ok=True)
    context=owned_inputs(root,control)
    python=Path(sys.executable).resolve();bash=Path('/bin/bash').resolve()
    tools=Tooling(python,file_hash(python),bash,file_hash(bash),root,(),str(python.parent))
    return replace(context,oracles=gate.OracleContext(oracles.OracleInputs(context.components.godot.candidate,tools)))


class OracleCoordinatorTests(unittest.TestCase):
    def test_optional_single_call_exact_sixteen_position(self) -> None:
        old=gate.ordered_owned_calls(bench_enabled=True,docs_enabled=True)
        new=gate.ordered_owned_calls(bench_enabled=True,docs_enabled=True,oracles_enabled=True)
        added=tuple(row for row in new if row not in old)
        self.assertEqual(added,(gate.oracle_call(),))
        self.assertEqual(tuple(row for row in new if row in old),old)
        self.assertEqual(len(added[0].identities),16)
        self.assertEqual(added[0].identities,tuple(row.identity for row in gate.obligations() if row.step==38))
        self.assertEqual([row.step for row in new],sorted([row.step for row in new],key=gate.ORDER.index))

    def test_candidate_and_admission_reject_before_launch(self) -> None:
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            try:
                context=oracle_context(root,read);assert context.oracles is not None
                wrong=replace(context.oracles.inputs.product,source_sha='f'*40)
                context=replace(context,oracles=replace(context.oracles,inputs=replace(context.oracles.inputs,product=wrong)))
                with patch.object(gate,'LOCK',root/'lane'),patch.object(oracles,'run_owned_component') as launch:
                    with self.assertRaisesRegex(ValueError,'candidate'):_=gate.invoke_oracles(gate.oracle_call(),context,root/'out')
                    launch.assert_not_called()
                self.assertEqual(lock_contender(root/'lane'),0)
            finally:os.close(read);os.close(write)

    def test_owned_settled_failure_does_not_stringify_primary(self) -> None:
        class Hostile(RuntimeError):
            @override
            def __str__(self) -> str:raise AssertionError('unsafe formatting')
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            try:
                context=oracle_context(root,read);out=root/'out'
                def child(_inputs: oracles.OracleInputs, output: Path, descriptor: int) -> oracles.OracleResult:
                    self.assertGreater(descriptor,2)
                    self.assertNotEqual(lock_contender(root/'lane'),0)
                    output.mkdir();_=(output/'raw.log').write_text('retained')
                    raise oracles.OracleSettledFailure(output,Hostile())
                with patch.object(gate,'LOCK',root/'lane'),patch.object(oracles,'inputs_current',return_value=()),patch.object(oracles,'run_owned_component',side_effect=child):
                    row=gate.invoke_oracles(gate.oracle_call(),context,out)
                self.assertEqual(row.status,'FAIL');self.assertIn('raw.log',dict(row.files))
                self.assertEqual(lock_contender(root/'lane'),0)
            finally:os.close(read);os.close(write)

    def test_actual_tiny_suite_result_and_history_authentication(self) -> None:
        from test_generic_oracles_run import synthetic
        for fail in (0,1):
            with self.subTest(fail=fail),synthetic(fail_native=fail) as fixture:
                read,write=os.pipe()
                try:
                    context=oracle_context(fixture.base/'coordinator',read)
                    context=replace(context,components=replace(context.components,godot=Products(fixture.inputs.product,fixture.inputs.product)),oracles=gate.OracleContext(fixture.inputs))
                    fcntl.flock(fixture.lane_fd,fcntl.LOCK_UN)
                    with patch.object(gate,'LOCK',fixture.base/'lock'),patch.object(oracles,'inputs_current',return_value=fixture.plan.inputs):
                        row=gate.invoke_oracles(gate.oracle_call(),context,fixture.base/'component')
                        self.assertEqual(row.status,'FAIL' if fail else 'PASS');self.assertIsNotNone(row.receipt,repr(row))
                        gate.recheck(row);gate.recheck_oracle(context,row)
                        assert row.receipt is not None
                        _=row.receipt.write_text(row.receipt.read_text()+' ')
                        with self.assertRaises(ValueError):gate.recheck_oracle(context,row)
                finally:os.close(read);os.close(write)

    def test_normal_settled_failure_continues39_without_nested_lane(self) -> None:
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            try:
                context=oracle_context(root,read)
                calls=(gate.oracle_call(),gate.OrderedCall(39,'visibility',('39:mux2_visibility',)))
                with patch.object(gate,'ordered_owned_calls',return_value=calls),patch.object(gate,'ordered_bindings'),patch.object(gate,'perf_bindings'),patch.object(oracles,'inputs_current',return_value=()),patch.object(gate,'invoke_oracles',return_value=gate.Outcome('oracles','FAIL',None,None,'settled')),patch.object(gate,'invoke',return_value=gate.Outcome('visibility','FAIL',None,None,'synthetic')) as later,patch.object(gate,'lane',return_value=nullcontext()) as lease:
                    path=gate.run_ordered_owned(context,context.strict,root/'out',read,oracles=context.oracles)
                later.assert_called_once();lease.assert_called_once()
                node=document(path.read_text());node.field('status').equals('FAIL')
                self.assertEqual(len(node.field('dispatch_events').array()),4)
            finally:os.close(read);os.close(write)

    def test_unknown_retains_handles_across_diagnostic_failure_and_eof(self) -> None:
        for fault in ('status','read','evil'):
            with self.subTest(fault=fault),TemporaryDirectory() as temporary:
                root=Path(temporary).resolve();read,write=os.pipe()
                script=r'''
import os,sys,signal,subprocess
from pathlib import Path
from unittest.mock import patch
sys.path[:0]=sys.argv[4:]
import generic_full_gate as g
import generic_oracles_run as o
from test_generic_full_gate import oracle_context
from test_generic_oracles_run import synthetic
root=Path(sys.argv[1]);control=int(sys.argv[2]);fault=sys.argv[3]
context=oracle_context(root,control)
class Evil(Exception):
 def __str__(self):raise RuntimeError('unsafe diagnostic')
with synthetic() as fixture:
 channels=o.child.prepare_suite(fixture.plan,fixture.lane_fd)
 stdout=(root/'tiny.stdout').open('wb');stderr=(root/'tiny.stderr').open('wb')
 native=subprocess.Popen([sys.executable,'-c','import time;time.sleep(60)'],stdout=stdout,stderr=stderr)
 live=o.LiveInvocation(fixture.plan,-1,channels,stdout,stderr,native,launch_intent=True)
 def cleanup(_a,_b):
  native.terminate();native.wait(timeout=5);os._exit(0)
 signal.signal(signal.SIGTERM,cleanup)
 def launch(_inputs,_output,fd):
  live.lane_fd=fd
  (root/'entered').write_text(str(native.pid))
  raise o.OracleSuiteOwnershipUnknown(live,(Evil(),))
 original=g._OraclesOwner.status
 def status(self):
  assert self.retained_live is live and self.retained_live.shell is native
  (root/'retained').write_text('same actual handle')
  if fault in ('status','evil'):raise Evil()
  original(self)
 g._OraclesOwner.status=status
 if fault=='read':
  def bad_read(*_args):raise OSError('read fault')
  g.os.read=bad_read
 calls=(g.oracle_call(),g.OrderedCall(39,'visibility',('39:mux2_visibility',)))
 with patch.object(g,'LOCK',root/'lane'),patch.object(g,'ordered_owned_calls',return_value=calls),patch.object(g,'ordered_bindings'),patch.object(g,'perf_bindings'),patch.object(o,'inputs_current',return_value=()),patch.object(o,'run_owned_component',side_effect=launch),patch.object(g,'oracle_outcome',side_effect=AssertionError('unknown snapshot')),patch.object(g,'ordered_action',side_effect=AssertionError('downstream')):
  g.run_ordered_owned(context,context.strict,root/'out',control,oracles=context.oracles)
 raise AssertionError('unknown returned')
'''
                command=[sys.executable,'-B','-c',script,str(root),str(read),fault,
                    str(Path(__file__).resolve().parent),str(Path(gate.__file__).resolve().parent),
                    str(Path(oracles.__file__).resolve().parent)]
                with subprocess.Popen(command,pass_fds=(read,),env={k:v for k,v in os.environ.items() if k!='PYTHONPATH'},stdout=subprocess.PIPE,stderr=subprocess.PIPE) as driver:
                    os.close(read)
                    try:
                        deadline=time.monotonic()+10
                        while not (root/'retained').exists() and driver.poll() is None and time.monotonic()<deadline:time.sleep(0.01)
                        if not (root/'retained').exists():
                            if driver.poll() is None:driver.terminate()
                            stdout,stderr=driver.communicate(timeout=5)
                            self.fail(f'startup: {stdout!r} {stderr!r}')
                        os.close(write);write=-1;time.sleep(0.1)
                        self.assertIsNone(driver.poll());self.assertNotEqual(lock_contender(root/'lane'),0)
                        self.assertFalse((root/'out/ordered-dispatch.json').exists())
                        self.assertFalse((root/'out/step-39').exists())
                        self.assertFalse((root/'out/step-38/oracles').exists())
                    finally:
                        if write>=0:os.close(write)
                        if driver.poll() is None:driver.terminate()
                        _=driver.wait(timeout=5)
                self.assertEqual(lock_contender(root/'lane'),0)

    def test_authentication_mismatch_never_promotes_result(self) -> None:
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();context=oracle_context(root,-1)
            assert context.oracles is not None
            out=root/'out';out.mkdir();receipt=out/'result.json'
            _=receipt.write_text('{"status":"PASS"}')
            row=gate.oracle_outcome(context.oracles,out,oracles.OracleResult('PASS',receipt,file_hash(receipt)))
            self.assertEqual(row.status,'FAIL');self.assertIsNone(row.reported_status)
            self.assertIn('result.json',dict(row.files))

    def test_source_admission_failure_prevents_launch_and_releases_lane(self) -> None:
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();context=oracle_context(root,-1)
            with patch.object(gate,'LOCK',root/'lane'),patch.object(oracles,'inputs_current',side_effect=ValueError('changed tool manifest')),patch.object(oracles,'run_owned_component') as launch:
                with self.assertRaisesRegex(ValueError,'changed tool'):
                    _=gate.invoke_oracles(gate.oracle_call(),context,root/'out')
                launch.assert_not_called()
            self.assertEqual(lock_contender(root/'lane'),0)


def resolver_context(root: Path, control: int) -> gate.OwnedOrderedInputs:
    context=owned_inputs(root,control)
    environment=replace(context.environment,values=(*context.environment.values,('GODOT_BIN',str(context.components.godot.candidate.executable))))
    return replace(context,resolver=gate.ResolverContext(context.components.godot.candidate,environment))


class ResolverCoordinatorTests(unittest.TestCase):
    def test_optional_call32_preserves_oracle_docs_bench_order(self) -> None:
        before=gate.ordered_owned_calls(bench_enabled=True,docs_enabled=True,oracles_enabled=True)
        after=gate.ordered_owned_calls(bench_enabled=True,docs_enabled=True,oracles_enabled=True,resolver_enabled=True)
        self.assertEqual(tuple(call for call in after if call not in before),(gate.OrderedCall(32,'resolver',('32:resolver_parity',)),))
        self.assertEqual(tuple(call for call in after if call in before),before)
        self.assertEqual([call.step for call in after],sorted([call.step for call in after],key=gate.ORDER.index))

    def test_actual_original_checker_receipt_and_historical_recheck(self) -> None:
        from test_generic_resolver_run import fixture
        with fixture() as f:
            root=f.root/'coordinator';root.mkdir();context=owned_inputs(root,-1)
            bound=gate.ResolverContext(f.product,replace(f.context,held_lane_fd=None))
            context=replace(context,components=replace(context.components,godot=Products(f.product,f.product)),resolver=bound)
            with patch.object(gate,'LOCK',f.root/'lane'),patch.object(resolver,'bindings',return_value=f.external):
                row=gate.invoke_resolver(gate.OrderedCall(32,'resolver',('32:resolver_parity',)),context,f.output)
                self.assertEqual(row.status,'PASS');self.assertEqual(lock_contender(f.root/'lane'),0)
                gate.recheck(row);gate.recheck_resolver(context,row)
                _=(f.output/'resolver_godot.json').write_text('{}')
                with self.assertRaises(ValueError):gate.recheck_resolver(context,row)

    def test_candidate_and_transient_descriptor_rejected_before_launch(self) -> None:
        for defect in ('candidate','descriptor','source'):
            with self.subTest(defect=defect),TemporaryDirectory() as temporary:
                root=Path(temporary).resolve();context=resolver_context(root,-1);bound=context.resolver;assert bound is not None
                if defect=='candidate':context=replace(context,resolver=replace(bound,product=replace(bound.product,source_sha='b'*40)))
                if defect=='descriptor':context=replace(context,resolver=replace(bound,environment=replace(bound.environment,held_lane_fd=99)))
                with patch.object(gate,'LOCK',root/'lane'),patch.object(resolver,'bindings',side_effect=ValueError('changed source')),patch.object(resolver,'run_owned') as launch:
                    with self.assertRaises(ValueError):_=gate.invoke_resolver(gate.OrderedCall(32,'resolver',('32:resolver_parity',)),context,root/'out')
                    launch.assert_not_called()
                self.assertEqual(lock_contender(root/'lane'),0)

    def test_settled_hostile_failure_keeps_raw_evidence_and_releases(self) -> None:
        class Evil(RuntimeError):
            @override
            def __str__(self) -> str:raise AssertionError('unsafe rendering')
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();context=resolver_context(root,-1)
            def child(_product: ProbeProduct, environment: static.ExecutionEnvironment, output: Path) -> ComponentResult:
                self.assertIsNotNone(environment.held_lane_fd);self.assertNotEqual(lock_contender(root/'lane'),0)
                output.mkdir();_=(output/'raw.log').write_text('retained')
                raise resolver.ResolverSettledFailure(Evil(),(Evil(),),None)
            with patch.object(gate,'LOCK',root/'lane'),patch.object(resolver,'bindings',return_value=()),patch.object(resolver,'run_owned',side_effect=child):
                row=gate.invoke_resolver(gate.OrderedCall(32,'resolver',('32:resolver_parity',)),context,root/'out')
            self.assertEqual(row.status,'FAIL');self.assertIn('raw.log',dict(row.files));self.assertIsNone(row.receipt)
            self.assertEqual(lock_contender(root/'lane'),0)

    def test_settled_failure_may_continue33_without_nested_lane(self) -> None:
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            try:
                context=resolver_context(root,read)
                calls=(gate.OrderedCall(32,'resolver',('32:resolver_parity',)),gate.OrderedCall(33,'static',('33:extensibility_grep',)))
                with patch.object(gate,'ordered_owned_calls',return_value=calls),patch.object(gate,'ordered_bindings'),patch.object(gate,'perf_bindings'),patch.object(resolver,'bindings',return_value=()),patch.object(gate,'invoke_resolver',return_value=gate.Outcome('resolver','FAIL',None,None,'settled')),patch.object(gate,'invoke',return_value=gate.Outcome('static','FAIL',None,None,'synthetic')) as later,patch.object(gate,'lane',side_effect=AssertionError('nested lane')):
                    path=gate.run_ordered_owned(context,context.strict,root/'out',read,resolver=context.resolver)
                later.assert_called_once();node=document(path.read_text());node.field('status').equals('FAIL')
                self.assertEqual(len(node.field('dispatch_events').array()),4)
            finally:os.close(read);os.close(write)

    def test_status_only_endpoint_rejected(self) -> None:
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();context=resolver_context(root,-1);assert context.resolver is not None
            out=root/'out';out.mkdir();receipt=out/'receipt.json';_=receipt.write_text('{"status":"PASS"}')
            row=gate.resolver_outcome(context.resolver,out,ComponentResult(receipt,'PASS'))
            self.assertEqual(row.status,'FAIL');self.assertIsNone(row.reported_status)
            self.assertIn('receipt.json',dict(row.files))

    def test_actual_failed_chain_unknown_retains_lane_through_diagnostic_faults(self) -> None:
        for fault in ('ordinary','status','read'):
            with self.subTest(fault=fault),TemporaryDirectory() as temporary:
                root=Path(temporary).resolve();read,write=os.pipe()
                script=r'''
import os,sys,signal
from pathlib import Path
from dataclasses import replace
from unittest.mock import patch
sys.path[:0]=sys.argv[4:]
import generic_full_gate as g
import generic_resolver_run as r
from generic_fixture_run import Products
from test_generic_full_gate import owned_inputs
from test_generic_resolver_run import fixture
root=Path(sys.argv[1]);control=int(sys.argv[2]);fault=sys.argv[3]
signal.signal(signal.SIGTERM,lambda _a,_b:os._exit(0))
class Evil(Exception):
 def __str__(self):raise AssertionError('unsafe diagnostic')
with fixture(native_code=3) as f:
 context=owned_inputs(root,control)
 bound=g.ResolverContext(f.product,replace(f.context,held_lane_fd=None))
 context=replace(context,components=replace(context.components,godot=Products(f.product,f.product)),resolver=bound)
 original=g._ResolverOwner.status
 retained=[]
 def status(self):
  assert isinstance(self.retained_failure,r.ResolverOwnershipUnknown)
  if retained:assert self.retained_failure is retained[0]
  else:retained.append(self.retained_failure)
  (root/'retained').write_text('actual failed shell chain; descriptor held')
  if fault=='status':raise Evil()
  original(self)
 g._ResolverOwner.status=status
 if fault=='read':
  original_read=g.os.read
  def bad_read(fd,*args):
   if fd==control:raise OSError('controller read fault')
   return original_read(fd,*args)
  g.os.read=bad_read
 calls=(g.OrderedCall(32,'resolver',('32:resolver_parity',)),g.OrderedCall(33,'static',('33:extensibility_grep',)))
 with patch.object(g,'LOCK',root/'lane'),patch.object(r,'LOCK',root/'lane'),patch.object(g,'ordered_owned_calls',return_value=calls),patch.object(g,'ordered_bindings'),patch.object(g,'perf_bindings'),patch.object(r,'bindings',return_value=f.external),patch.object(g,'resolver_outcome',side_effect=AssertionError('unknown snapshot')),patch.object(g,'ordered_action',side_effect=AssertionError('downstream')):
  g.run_ordered_owned(context,context.strict,root/'out',control,resolver=context.resolver)
 raise AssertionError('unknown returned')
'''
                command=[sys.executable,'-B','-c',script,str(root),str(read),fault,str(Path(__file__).resolve().parent),str(Path(gate.__file__).resolve().parent),str(Path(resolver.__file__).resolve().parent)]
                with subprocess.Popen(command,pass_fds=(read,),env={k:v for k,v in os.environ.items() if k!='PYTHONPATH'},stdout=subprocess.PIPE,stderr=subprocess.PIPE) as driver:
                    os.close(read)
                    try:
                        deadline=time.monotonic()+10
                        while not (root/'retained').exists() and driver.poll() is None and time.monotonic()<deadline:time.sleep(0.01)
                        if not (root/'retained').exists():
                            if driver.poll() is None:driver.terminate()
                            stdout,stderr=driver.communicate(timeout=5)
                            self.fail(f'startup: {stdout!r} {stderr!r}')
                        _=os.write(write,b'inspect\n');os.close(write);write=-1;time.sleep(0.1)
                        self.assertIsNone(driver.poll());self.assertNotEqual(lock_contender(root/'lane'),0)
                        self.assertFalse((root/'out/ordered-dispatch.json').exists())
                        self.assertFalse((root/'out/step-33').exists())
                        process=document((root/'out/step-32/resolver/process/process.json').read_text())
                        process.field('native_exit').equals(3)
                        self.assertFalse((root/'out/step-32/resolver/receipt.json').exists())
                    finally:
                        if write>=0:os.close(write)
                        if driver.poll() is None:driver.terminate()
                        _=driver.wait(timeout=5)
                self.assertEqual(lock_contender(root/'lane'),0)


def structural_context(root: Path, control: int) -> gate.OwnedOrderedInputs:
    context=owned_inputs(root,control)
    separate=root/'separate-base';separate.mkdir()
    base=replace(context.components.godot.base,input_root=separate)
    return replace(context,components=replace(context.components,godot=Products(base,context.components.godot.candidate)))


class Structural48CoordinatorTests(unittest.TestCase):
    def test_optional_structural_sibling_exact_order_and_coverage(self) -> None:
        old=gate.ordered_owned_calls(bench_enabled=True,docs_enabled=True,oracles_enabled=True)
        new=gate.ordered_owned_calls(bench_enabled=True,docs_enabled=True,oracles_enabled=True,structural_enabled=True)
        added=tuple(row for row in new if row not in old)
        self.assertEqual(added,(gate.OrderedCall(48,'structural',('48:m6a.plant_art_structural',)),))
        self.assertEqual(tuple(row for row in new if row in old),old)
        index=new.index(added[0])
        self.assertEqual(new[index-1].identity,'48:direct')
        self.assertGreater(new[index+1].step,48)
        self.assertFalse(any(row.step==49 for row in new))

    def test_predecessor_rejects_every_nonpass_and_missing_or_changed_endpoint(self) -> None:
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();context=ordered_inputs(root)
            call=gate.OrderedCall(48,'direct',('48:m6a.food_data_only',))
            result=ordered_endpoint(call,root/'prior',context)
            row=gate.Outcome('direct','PASS',result.receipt,file_hash(result.receipt),'',
                tuple(inventory(root/'prior',[p.name for p in (root/'prior').iterdir()]).items()),root/'prior','PASS','PASS')
            gate.structural_predecessor(context.components,{'48:direct':row})
            for state in ('FAIL','INCOMPLETE','PENDING','NOT_RUN'):
                with self.subTest(state=state),self.assertRaises(ValueError):
                    gate.structural_predecessor(context.components,{'48:direct':replace(row,status=state)})
            for bad in (replace(row,receipt=None),replace(row,reported_status=None),replace(row,returned_status='FAIL')):
                with self.assertRaises(ValueError):gate.structural_predecessor(context.components,{'48:direct':bad})
            with self.assertRaises(ValueError):gate.structural_predecessor(context.components,{})
            _=result.receipt.write_text(result.receipt.read_text()+'\n')
            with self.assertRaises(ValueError):gate.structural_predecessor(context.components,{'48:direct':row})

    def test_real_structural_owner_and_historical_rechecks_without_outer_lane(self) -> None:
        from test_generic_m6a_structural import owned_fixture
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();fixture=root/'fixture';fixture.mkdir()
            products,tools,lock=owned_fixture(fixture)
            base=root/'coordinator';base.mkdir();read,write=os.pipe()
            try:
                context=owned_inputs(base,read)
                context=replace(context,components=replace(context.components,godot=products),
                    structural=gate.Structural48Context(products,tools))
                with patch.object(structural,'LOCK',lock),patch.object(oracle_child,'LOCK',lock),patch.object(structural,'check_owned_helpers'),patch.object(gate,'lane',side_effect=AssertionError('outer lane')),patch.object(gate,'_StrictOwner',side_effect=AssertionError('outer owner')):
                    row=gate.invoke_structural(gate.OrderedCall(48,'structural',('48:m6a.plant_art_structural',)),context,fixture/'owned')
                    self.assertEqual(row.status,'PASS',row.error+row.snapshot_error)
                    self.assertEqual(lock_contender(lock),0)
                    gate.recheck(row);gate.recheck_structural(context,row)
                    assert row.receipt is not None
                    _=row.receipt.write_text(row.receipt.read_text()+'\n')
                    with self.assertRaises(ValueError):gate.recheck_structural(context,row)
            finally:os.close(read);os.close(write)

    def test_paired_godot_binding_rejects_candidate_reuse_before_entry(self) -> None:
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            try:
                context=owned_inputs(root,read)
                context=replace(context,components=replace(context.components,godot=Products(replace(context.components.godot.base,source_sha='b'*40),context.components.godot.candidate)))
                python=Path(sys.executable).resolve();bash=Path('/bin/bash').resolve()
                tools=Tooling(python,file_hash(python),bash,file_hash(bash),root,(),str(python.parent))
                context=replace(context,structural=gate.Structural48Context(Products(context.components.godot.candidate,context.components.godot.candidate),tools))
                with patch.object(structural,'run_owned_result') as launch,self.assertRaisesRegex(ValueError,'paired'):
                    _=gate.invoke_structural(gate.OrderedCall(48,'structural',('48:m6a.plant_art_structural',)),context,root/'out')
                launch.assert_not_called()
            finally:os.close(read);os.close(write)

    def test_step48_nonpass_blocks_only_structural_and_settled_failure_continues(self) -> None:
        for state in ('PASS','FAIL','INCOMPLETE'):
            with self.subTest(state=state),TemporaryDirectory() as temporary:
                root=Path(temporary).resolve();read,write=os.pipe()
                try:
                    context=owned_inputs(root,read)
                    python=Path(sys.executable).resolve();bash=Path('/bin/bash').resolve()
                    tools=Tooling(python,file_hash(python),bash,file_hash(bash),root,(),str(python.parent))
                    context=replace(context,structural=gate.Structural48Context(context.components.godot,tools))
                    calls=(gate.OrderedCall(48,'direct',('48:m6a.food_data_only',)),gate.OrderedCall(48,'structural',('48:m6a.plant_art_structural',)),gate.OrderedCall(50,'direct',tuple('50:'+name for name in direct.step_fixtures(50))))
                    trace: list[str]=[]
                    def action(call: gate.OrderedCall,bound: gate.OrderedInputs,selected: str=state, observed: list[str]=trace) -> gate.Action:
                        def execute(out: Path) -> ComponentResult:
                            observed.append(call.identity)
                            return ordered_endpoint(call,out,bound,selected if call.step==48 else 'PASS')
                        return execute
                    with patch.object(gate,'ordered_owned_calls',return_value=calls),patch.object(gate,'ordered_bindings'),patch.object(gate,'perf_bindings'),patch.object(gate,'structural_bindings',return_value=context.structural),patch.object(gate,'structural_dependencies',return_value={'synthetic':True}),patch.object(gate,'ordered_action',side_effect=action),patch.object(gate,'invoke_structural',return_value=gate.Outcome('structural','FAIL',None,None,'settled')) as launch,patch.object(gate,'lane',return_value=nullcontext()) as lease:
                        path=gate.run_ordered_owned(context,context.strict,root/'out',read,structural=context.structural)
                    row=document(path.read_text());self.assertEqual(trace,['48:direct','50:direct'])
                    self.assertEqual(lease.call_count,2)
                    self.assertEqual(launch.call_count,1 if state=='PASS' else 0)
                    sibling=next(item for item in row.field('obligations').array() if item.field('identity').value=='48:m6a.plant_art_structural')
                    self.assertEqual(sibling.field('component_status').value,'FAIL' if state=='PASS' else 'NOT_RUN')
                    self.assertEqual(sibling.field('blocked_by').value,None if state=='PASS' else '48:direct')
                    self.assertFalse(row.field('full_gate_claim').value)
                finally:os.close(read);os.close(write)

    def test_typed_settled_failure_retains_diagnostics_and_partial_inventory(self) -> None:
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            try:
                context=structural_context(root,read);toolroot=root/'toolroot';toolroot.mkdir()
                python=Path(sys.executable).resolve();bash=Path('/bin/bash').resolve()
                tools=Tooling(python,file_hash(python),bash,file_hash(bash),toolroot,(),str(python.parent))
                context=replace(context,structural=gate.Structural48Context(context.components.godot,tools))
                def child(_products: Products,_tools: Tooling,output: Path,_control: int) -> structural.OwnedComponentResult:
                    output.mkdir();_=(output/'partial.log').write_text('retained')
                    raise structural.StructuralSettledFailure(ValueError('primary-marker'),OSError('secondary-marker'),None)
                with patch.object(gate,'structural_bindings',return_value=context.structural),patch.object(structural,'run_owned_result',side_effect=child):
                    row=gate.invoke_structural(gate.OrderedCall(48,'structural',('48:m6a.plant_art_structural',)),context,root/'out')
                self.assertEqual(row.status,'FAIL');self.assertIn('primary-marker',row.error);self.assertIn('secondary-marker',row.error)
                self.assertIn('partial.log',dict(row.files));self.assertIsNone(row.receipt)
            finally:os.close(read);os.close(write)

    def test_unclassified_entry_fault_retains_actual_token_and_prevents_finalization(self) -> None:
        script=r"""
import os,sys
from pathlib import Path
from unittest.mock import patch
sys.path[:0]=sys.argv[4:]
import generic_full_gate as g
import generic_m6a_structural as s
from dataclasses import replace
from test_generic_full_gate import structural_context,ordered_endpoint
from generic_process import file_hash
root=Path(sys.argv[1]);control=int(sys.argv[2]);token=sys.argv[3]=='token'
context=structural_context(root,control);toolsroot=root/'tools';toolsroot.mkdir()
python=Path(sys.executable).resolve();bash=Path('/bin/bash').resolve()
tools=s.Tooling(python,file_hash(python),bash,file_hash(bash),toolsroot,(),str(python.parent))
context=replace(context,structural=g.Structural48Context(context.components.godot,tools))
class Hostile(Exception):
 def __str__(self):raise AssertionError('unsafe rendering')
def launch(products,tools,out,control):
 if token:
  owner=s.StructuralLaneOwner.acquire(out,s.OwnedAdmission(products,tools))
  (root/'entered').write_text('actual owner retained in exception')
  raise s.StructuralReleaseUnknown(owner,Hostile(),None)
 (root/'entered').write_text('unclassified disposition')
 raise Hostile()
def action(call,bound):
 if call.step!=48:raise AssertionError('downstream')
 return lambda out:ordered_endpoint(call,out,bound)
calls=(g.OrderedCall(48,'direct',('48:m6a.food_data_only',)),g.OrderedCall(48,'structural',('48:m6a.plant_art_structural',)),g.OrderedCall(50,'direct',()))
from contextlib import nullcontext
with patch.object(s,'LOCK',root/'lane'),patch.object(g,'lane',return_value=nullcontext()),patch.object(g,'ordered_owned_calls',return_value=calls),patch.object(g,'ordered_bindings'),patch.object(g,'perf_bindings'),patch.object(g,'structural_bindings',return_value=context.structural),patch.object(g,'structural_dependencies',return_value={'synthetic':True}),patch.object(g,'ordered_action',side_effect=action),patch.object(s,'run_owned_result',side_effect=launch),patch.object(g,'structural_outcome',side_effect=AssertionError('snapshot before settlement')):
 g.run_ordered_owned(context,context.strict,root/'out',control,structural=context.structural)
"""
        for token in (False,True):
            with self.subTest(token=token),TemporaryDirectory() as temporary:
                root=Path(temporary).resolve();read,write=os.pipe()
                args=[sys.executable,'-c',script,str(root),str(read),'token' if token else 'plain',str(Path(__file__).resolve().parent),str(Path(gate.__file__).resolve().parent),str(Path(structural.__file__).resolve().parent)]
                with subprocess.Popen(args,pass_fds=(read,),stdout=subprocess.PIPE,stderr=subprocess.PIPE) as child:
                    os.close(read)
                    try:
                        deadline=time.monotonic()+10
                        while not (root/'entered').exists() and child.poll() is None and time.monotonic()<deadline:time.sleep(.01)
                        self.assertTrue((root/'entered').exists())
                        self.assertIsNone(child.poll())
                        if token:self.assertNotEqual(lock_contender(root/'lane'),0)
                        os.close(write);write=-1
                        time.sleep(.1)
                        self.assertIsNone(child.poll())
                        self.assertFalse((root/'out/ordered-dispatch.json').exists())
                        self.assertFalse((root/'out/step-50').exists())
                        self.assertFalse((root/'out/step-48/structural').exists())
                    finally:
                        if write>=0:os.close(write)
                        if child.poll() is None:child.terminate()
                        _=child.communicate(timeout=5)


class CombinedOwnedCoordinatorTests(unittest.TestCase):
    def test_resolver_oracle_structural_flags_and_context_slots_survive_combination(self) -> None:
        selected=gate.ordered_owned_calls(bench_enabled=True,docs_enabled=True,oracles_enabled=True,resolver_enabled=True,structural_enabled=True)
        identities=[row.identity for row in selected]
        for identity in ('14:static','25:bench','32:resolver','38:oracles','48:structural'):
            self.assertEqual(identities.count(identity),1)
        self.assertLess(identities.index('32:resolver'),identities.index('33:static'))
        self.assertLess(identities.index('38:oracles'),identities.index('39:visibility'))
        self.assertEqual(identities.index('48:structural'),identities.index('48:direct')+1)
        self.assertEqual([row.step for row in selected],sorted([row.step for row in selected],key=gate.ORDER.index))
        self.assertEqual(len([name for row in selected for name in row.identities]),len({name for row in selected for name in row.identities}))
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            try:
                context=resolver_context(root,read);assert context.resolver is not None
                python=Path(sys.executable).resolve();bash=Path('/bin/bash').resolve()
                tools=Tooling(python,file_hash(python),bash,file_hash(bash),root,(),str(python.parent))
                structural_bound=gate.Structural48Context(context.components.godot,tools)
                oracle_bound=gate.OracleContext(oracles.OracleInputs(context.components.godot.candidate,tools))
                def capture(bound: gate.OrderedInputs,output: Path) -> Path:
                    self.assertIsInstance(bound,gate.OwnedOrderedInputs)
                    assert isinstance(bound,gate.OwnedOrderedInputs)
                    self.assertIs(bound.resolver,context.resolver)
                    self.assertIs(bound.oracles,oracle_bound)
                    self.assertIs(bound.structural,structural_bound)
                    self.assertEqual(bound.control_fd,read)
                    return output
                with patch.object(gate,'_run_ordered',side_effect=capture):
                    self.assertEqual(gate.run_ordered_owned(context,context.strict,root/'out',read,
                        oracles=oracle_bound,resolver=context.resolver,structural=structural_bound),root/'out')
            finally:os.close(read);os.close(write)
