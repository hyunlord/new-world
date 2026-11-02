# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Synthetic endpoint envelopes and mocked orchestration, never native proof."""
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
