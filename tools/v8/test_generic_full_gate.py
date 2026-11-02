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
from unittest.mock import patch

import generic_direct_run as direct
import generic_full_gate as gate
import generic_renderer_run as renderer
import generic_ui_fixed_run as fixed
import generic_ui_pack_run as pack
import generic_ui_visibility_run as visibility
import generic_ui_why_run as why
from generic_declarations import Declarations
from generic_fixture_registry import Fixture, Registry
from generic_fixture_run import ComponentResult, Products
from generic_fixtures import ProbeProduct
from generic_gate import GateContractError, document, integer
from generic_process import atomic_json, file_hash
from mux2_gate_inputs import Json


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


if __name__ == '__main__':
    _ = unittest.main()


class OrderedDispatchTests(unittest.TestCase):
    def test_closed_order_retains_every_missing_obligation(self) -> None:
        calls = gate.ordered_calls()
        self.assertEqual([call.step for call in calls], sorted([call.step for call in calls], key=gate.ORDER.index))
        self.assertEqual([call.step for call in calls if call.component == 'static'], [1,7,13,15,20,22,31,33])
        self.assertEqual([call.step for call in calls if call.component == 'direct'], [10,11,18,19,26,28,29,30,35,44,50])
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
                gate.authenticate('direct', document(json.dumps(data)), context, step=44)


def ordered_inputs(root: Path) -> gate.OrderedInputs:
    import generic_fixture_registry as registry_module
    context=inputs(root)
    registry=registry_module.load_registry(Path(registry_module.__file__).with_name('gate_fixtures.json'))
    context=replace(context,registry=registry,declarations=replace(context.declarations,known_fixtures=tuple(sorted(direct.DIRECT_IDS))))
    product=context.probes.candidate
    return gate.OrderedInputs(context,gate.static.StaticProduct(product.input_root,product.source_sha,product.input_files),
                              gate.static.ExecutionEnvironment((),(),()))


def ordered_endpoint(call: gate.OrderedCall, out: Path, context: gate.OrderedInputs, status: str='PASS') -> ComponentResult:
    if call.component=='static':
        out.mkdir();path=out/'receipt.json'
        data: dict[str,Json]={'schema':'generic_static_run.v1','step':call.step,'name':gate.static.COMMANDS[call.step][0],
            'scope':'COMPONENT_ONLY','original_order_executed':False,'full_gate_complete':False,'source_admission_granted':False,
            'source_root':str(context.static_product.root),'source_sha':context.static_product.source_sha,
            'source_invocation':list(gate.static.COMMANDS[call.step][1]),'started_at':'synthetic','status':status,
            'process_status':status,'artifacts':{},'tool_dependency_closure':'caller_supplied_not_independently_complete',
            'errors':[],'finished_at':'synthetic'}
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
    atomic_json(path,data)
    return ComponentResult(path,status)


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
            self.assertEqual(lock.call_count,len(trace)-8)
            self.assertEqual(node.field('status').value,'INCOMPLETE')
            self.assertFalse(node.field('full_gate_claim').value)
            missing={row.field('identity').text() for row in node.field('obligations').array() if row.field('status').value=='NOT_IMPLEMENTED'}
            self.assertTrue({'28:colony_knowledge.release','36:data_only_ext.release','46:gdscript.four_files',
                             '47:m6a.food72','48:m6a.food_data_only','4:workspace.debug','5:workspace.release'} <= missing)
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
                    if call.step==9:(out/'raw.log').write_text('retained failure')
                    if call.step==50:(root/'out/step-09/smoke/raw.log').write_text('mutated')
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
                             [call.identity for call in gate.ordered_calls() if call.component=='static'])

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
            def child(*args):
                out=next(arg for arg in reversed(args) if isinstance(arg,Path))
                identity=str(int(out.parent.name.removeprefix('step-')))+':'+out.name
                call=calls[identity];trace.append(identity)
                self.assertEqual(held,call.component!='static')
                if call.component=='direct':self.assertEqual(args[3],call.step)
                if call.component=='static':self.assertEqual(args[1],call.step)
                return ordered_endpoint(call,out,context)
            with ExitStack() as stack:
                stack.enter_context(patch.object(gate,'lane',side_effect=lock))
                stack.enter_context(patch.object(direct,'run_all',side_effect=AssertionError('no aggregate')))
                for module,name in ((direct,'run_step'),(gate.static,'run_step'),(gate.smoke,'run_pair'),
                                    (renderer,'run_pair'),(fixed,'run_pair'),(pack,'run_pair'),(visibility,'run_pair'),(why,'run_pair')):
                    stack.enter_context(patch.object(module,name,side_effect=child))
                node=document(gate.run_ordered(context,root/'out').read_text())
            self.assertEqual(trace,list(calls))
            self.assertEqual(node.field('status').value,'INCOMPLETE')
            self.assertEqual([(row.field('call').text(),row.field('event').text()) for row in node.field('dispatch_events').array()],
                             [(identity,event) for identity in trace for event in ('start','end')])

    def test_plan_write_failure_retains_terminal_without_launches(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            def write(path: Path, data):
                if path.name=='plan.json':raise OSError('plan failed')
                return atomic_json(path,data)
            with patch.object(gate,'atomic_json',side_effect=write),patch.object(gate,'ordered_action',side_effect=AssertionError('no call')):
                node=document(gate.run_ordered(context,root/'out').read_text())
            self.assertEqual(node.field('status').value,'FAIL')
            self.assertIsNone(node.field('plan_sha256').value)
            self.assertEqual(node.field('actual_component_sequence').array(),[])

    def test_duplicate_expansion_is_rejected_before_any_producer(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root);original=gate.expand
            def duplicate(registry, expansion):
                rows=original(registry,expansion)
                return (*rows,rows[0])
            with patch.object(gate,'expand',side_effect=duplicate),patch.object(gate,'ordered_action',side_effect=AssertionError('no call')):
                node=document(gate.run_ordered(context,root/'out').read_text())
            self.assertEqual(node.field('status').value,'FAIL')
            self.assertTrue(any('duplicate identity/output' in row.text() for row in node.field('errors').array()))

    def test_unsupported_direct_step_incomplete_schema_is_not_a_pass(self) -> None:
        with TemporaryDirectory() as temp:
            root=Path(temp).resolve();context=ordered_inputs(root)
            call=gate.OrderedCall(47,'direct',('47:m6a.food72',))
            result=ordered_endpoint(call,root/'out',context,'INCOMPLETE')
            self.assertEqual(gate.authenticate('direct',document(result.receipt.read_text()),context.components,step=47),'INCOMPLETE')
            self.assertNotIn(47,[call.step for call in gate.ordered_calls()])
