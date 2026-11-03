# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_acceptance
"""Dispatch mechanism fixtures; never native or timing qualification."""
import subprocess
import sys
import unittest

import generic_whole_gate_acceptance as acceptance


class ContractTests(unittest.TestCase):
    def test_actual_cli_rejects_unknown_slot_as_unknown_with_diagnostic(self) -> None:
        # Given no admitted session, when the actual CLI receives an invalid slot,
        # then it returns UNKNOWN/2 and preserves the diagnostic without a launch.
        result = subprocess.run([sys.executable, '-B', acceptance.__file__, 'run',
                                 '/unused', '0'*64, '26', '/unused-output'],
                                capture_output=True, text=True, check=False)
        self.assertEqual(result.returncode, 2)
        self.assertIn('unsupported acceptance slot', result.stderr)

    def test_private_umask_preserves_valid_publication_and_rejects_mode_tamper(self) -> None:
        # Given an isolated child with umask077, when exercising publication and
        # its mode-tamper negatives, then legitimate0600 is accepted and tampering rejected.
        result = subprocess.run([sys.executable, '-B', '-m', 'unittest',
            'test_generic_whole_gate_acceptance.RoutingTests.test_published_fail_and_unknown_preserve_component_status',
            'test_generic_whole_gate_acceptance.RoutingTests.test_publication_mutations_rejected_same_call'],
            cwd=Path(__file__).parent, capture_output=True, text=True, check=False, umask=0o077)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_closed_six_slot_contract(self) -> None:
        # Given the new whole-shell consumer; when inspecting its closed route map,
        # then no old parser or extra workload slot is admitted.
        self.assertEqual(getattr(acceptance, 'STEPS', None),
                         {25:'bench',44:'core_shadow',45:'core_evaluation',
                          47:'m6a_food_fixture',48:'m6a_food_data_only',50:'read_preservation'})


# These tests deliberately mock session admission and producer calls where noted.
# They prove consumer wiring, not materialization, native worlds, or performance.
import json
import tempfile
from dataclasses import asdict
from pathlib import Path
from unittest.mock import patch

import generic_fixture_registry as registry_module
from generic_declarations import Declarations
from generic_fixture_run import ComponentResult
from generic_gate import document
from generic_process import atomic_json, file_hash
from mux2_gate_inputs import Json, Node


class Fixture:
    def __init__(self, root: Path) -> None:
        self.root: Path = root
        self.output: Path = root/'run'
        normal = Path(acceptance.normalizer.__file__).resolve()
        script = root/'script.json'; _ = script.write_text('[]')
        registry = Path(registry_module.__file__).with_name('gate_fixtures.json')
        self.selected: dict[str,Node] = {}
        arms: dict[str,Json] = {}
        for arm,char in (('base','a'),('candidate','b')):
            folder = root/arm; folder.mkdir()
            source = folder/'source'; _ = source.write_text(arm)
            binary = folder/'probe'; _ = binary.write_text('#!/bin/sh\nexit 0\n'); binary.chmod(0o755)
            build = root/(arm+'-build.json'); _ = build.write_text('{}')
            inputs: dict[str,Json] = {'source':file_hash(source)}
            arms[arm] = {'root':str(folder),'source_sha':char*40,'source_inputs':inputs}
            self.selected[arm] = Node({'executable':str(binary),'executable_sha256':file_hash(binary),
                'source_inputs':inputs,'runtime_inputs':{},'build_receipt':str(build),'build_receipt_sha256':file_hash(build)})
        declarations = Declarations('b'*40,'a'*40,'publication',tuple(sorted(registry_module.DIRECT_IDS)),(),(),(),(),'d'*64)
        self.session: Node = Node({'schema':'whole_gate_session.v1','purpose':'publication','arms':arms,
            'output':str(self.output),'harness':{'root':str(normal.parents[2]),'source_sha':'c'*40,
                'inputs':{'tools/v8/read_summary_normalise.py':file_hash(normal)}},
            'declarations':document(json.dumps(asdict(declarations))).value,
            'registry':{'path':str(registry),'sha256':file_hash(registry)},
            'external_inputs':{str(normal):file_hash(normal),str(script):file_hash(script)},
            'acceptance':{'normalizer':{'path':str(normal),'sha256':file_hash(normal)},
                          'bench_script':{'path':str(script),'sha256':file_hash(script)},'justifications':[]}})
        self.manifest: Path = root/'session.json'; _ = self.manifest.write_text(json.dumps(self.session.value))
        self.digest: str = file_hash(self.manifest)

    def path(self, step: int) -> Path:
        return self.output/'steps'/'candidate'/f'{step:02d}-{acceptance.STEPS[step]}'/'acceptance'

    def selected_product(self, _session: Node, arm: str, _kind: str) -> Node:
        return self.selected[arm]


class RoutingTests(unittest.TestCase):
    def test_four_direct_slots_route_once_even_without_declarations(self) -> None:
        # Given authenticated-view stand-ins; when dispatching each direct slot,
        # then the complete existing producer receives both original products.
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            for step in (44,47,48,50):
                output = f.root/str(step)
                with patch.object(acceptance.direct,'run_step',return_value=ComponentResult(output/'receipt.json','FAIL')) as run:
                    result = acceptance._dispatch(f.session,(f.selected['base'],f.selected['candidate']),step,output)
                self.assertEqual(result.status,'FAIL'); run.assert_called_once()
                args = run.call_args.args
                self.assertEqual(args[3:],(step,output))
                self.assertEqual(args[1].base.source_sha,'a'*40)
                self.assertEqual(args[1].candidate.source_sha,'b'*40)
                self.assertEqual(args[2].changes,())

    def test_bench_slot_routes_to_owned_bench(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve()); selected = (f.selected['base'],f.selected['candidate'])
            with patch.object(acceptance,'_bench',return_value=acceptance.Result('FAIL','FAIL',None)) as run:
                result = acceptance._dispatch(f.session,selected,25,f.root/'bench')
            run.assert_called_once_with(f.session,selected,f.root/'bench'); self.assertEqual(result.status,'FAIL')

    def test_bench_unknown_is_never_authenticated_or_promoted(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            unknown = acceptance.bench.BenchOwnershipUnknown(RuntimeError('fixture'),None,None)
            with patch.object(acceptance.bench,'run_bench_owned',side_effect=unknown), \
                 patch.object(acceptance.bench,'authenticate_result') as authenticate:
                result = acceptance._bench(f.session,(f.selected['base'],f.selected['candidate']),f.root/'bench')
            self.assertEqual(result.status,'UNKNOWN'); authenticate.assert_not_called()

    def test_wrong_slot_rejected_before_session_or_launch(self) -> None:
        with patch.object(acceptance.shared,'load_session') as loader, self.assertRaises(ValueError):
            _ = acceptance.run_step(Path('/unused'),'0'*64,26,Path('/unused-out'))
        loader.assert_not_called()

    def test_published_fail_and_unknown_preserve_component_status(self) -> None:
        for status,component in (('FAIL','FAIL'),('UNKNOWN','UNKNOWN'),('COMPONENT_PASS','PENDING')):
            with self.subTest(status=status), tempfile.TemporaryDirectory() as temp:
                f = Fixture(Path(temp).resolve())
                with patch.object(acceptance.shared,'load_session',return_value=f.session), \
                     patch.object(acceptance.shared,'current_product_inputs',side_effect=f.selected_product), \
                     patch.object(acceptance,'_dispatch',return_value=acceptance.Result(status,component,None)):
                    result = acceptance.run_step(f.manifest,f.digest,47,f.path(47))
                self.assertEqual(result.status,status)
                receipt = document((f.path(47)/'acceptance.json').read_text())
                self.assertEqual(receipt.field('component_status').value,component)
                self.assertFalse(receipt.field('publication_authority').value)

    def test_original_component_inventory_is_retained(self) -> None:
        # Given an actual returned component tree, when publishing its acceptance,
        # then consumers receive the original bytes and modes, not a later scan.
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            def produce(_session: Node, _selected: tuple[Node,Node], _step: int, output: Path) -> acceptance.Result:
                output.mkdir(); receipt = output/'receipt.json'
                atomic_json(receipt, {'status':'FAIL'})
                return acceptance.Result('FAIL','FAIL',receipt)
            with patch.object(acceptance.shared,'load_session',return_value=f.session), \
                 patch.object(acceptance.shared,'current_product_inputs',side_effect=f.selected_product), \
                 patch.object(acceptance,'_dispatch',side_effect=produce):
                _ = acceptance.run_step(f.manifest,f.digest,47,f.path(47))
            receipt = f.path(47)/'component'/'receipt.json'
            terminal = document((f.path(47)/'acceptance.json').read_text())
            self.assertEqual(terminal.field('original_artifacts').field(str(receipt)).value,file_hash(receipt))
            self.assertEqual(terminal.field('original_modes').field(str(receipt.parent)).value,receipt.parent.stat().st_mode)

    def test_publication_mutations_rejected_same_call(self) -> None:
        for attack in ('directory','source','receipt','mode'):
            with self.subTest(attack=attack), tempfile.TemporaryDirectory() as temp:
                f = Fixture(Path(temp).resolve()); injected = False
                def publish(path: Path, value: dict[str,Json], attack: str = attack, f: Fixture = f) -> None:
                    nonlocal injected
                    atomic_json(path,value); injected = True
                    if attack=='directory': (path.parent/'unexpected').mkdir()
                    if attack=='source': _ = (f.root/'candidate'/'source').write_text('drift')
                    if attack=='receipt': _ = path.write_text('{}')
                    if attack=='mode': path.chmod(0o777)
                with patch.object(acceptance.shared,'load_session',return_value=f.session), \
                     patch.object(acceptance.shared,'current_product_inputs',side_effect=f.selected_product), \
                     patch.object(acceptance,'_dispatch',return_value=acceptance.Result('COMPONENT_PASS','PASS',None)), \
                     patch.object(acceptance,'atomic_json',side_effect=publish), self.assertRaises(ValueError):
                    _ = acceptance.run_step(f.manifest,f.digest,48,f.path(48))
                self.assertTrue(injected)


class NormalizerBindingTests(unittest.TestCase):
    def test_only_harness_or_exact_materialized_h_role_is_admitted(self) -> None:
        # These are view-boundary units; shared session materialization is not mocked as proof.
        for fault in ('valid', 'unchanged', 'original_hash', 'original_root', 'original_sha',
                      'original_mode', 'wrong_root', 'wrong_role', 'tampered', 'missing_effective', 'missing_harness'):
            with self.subTest(fault=fault), tempfile.TemporaryDirectory() as temp:
                f = Fixture(Path(temp).resolve())
                relative = 'tools/v8/read_summary_normalise.py'
                expected = f.session.field('acceptance').field('normalizer').field('sha256').text()
                loaded = f.root/'candidate'/relative; loaded.parent.mkdir(parents=True)
                _ = loaded.write_bytes(Path(acceptance.normalizer.__file__).read_bytes())
                arm = f.session.field('arms').field('candidate').mapping()
                arm['source_inputs'] = {**arm['source_inputs'],relative:expected}
                arm['effective_source'] = {'manifest':{relative:['100644',expected]}}
                unchanged = fault in {'unchanged', 'original_hash', 'original_root', 'original_sha', 'original_mode'}
                if unchanged:
                    arm['source_before'] = {'root':str(f.root/('base' if fault=='original_root' else 'candidate')),
                        'source_sha':('a' if fault=='original_sha' else 'b')*40, 'source_clean':True,
                        'tracked_manifest':{relative:['100755' if fault=='original_mode' else '100644',
                            'e'*40, 'f'*64 if fault=='original_hash' else expected]}}
                roles = f.root/'roles.json'
                atomic_json(roles,{'schema':'whole_gate_layers.v1','product_sha':'b'*40,'harness_sha':'c'*40,
                    'files':[] if unchanged else [{'path':relative,'role':'O' if fault=='wrong_role' else 'H','post_sha256':expected}]})
                arm['layer_manifest_path'] = str(roles); arm['layer_manifest_sha256'] = file_hash(roles)
                f.session.field('external_inputs').mapping()[str(roles)] = file_hash(roles)
                if fault=='wrong_root':
                    foreign = f.root/'foreign.py'; _ = foreign.write_bytes(loaded.read_bytes()); loaded = foreign
                if fault=='tampered': _ = loaded.write_text('changed')
                if fault=='missing_effective': arm['effective_source'] = {'manifest':{}}
                if fault=='missing_harness': f.session.field('harness').mapping()['inputs'] = {}
                with patch.object(acceptance.normalizer,'__file__',str(loaded)):
                    if fault in {'valid','unchanged'}: self.assertEqual(acceptance.normalizer_path(f.session),loaded)
                    else:
                        with self.assertRaises(ValueError): _ = acceptance.normalizer_path(f.session)


class ActualMechanismTests(unittest.TestCase):
    def test_actual_44_matrix_then_45_dispatch_has_no_second_launch(self) -> None:
        # Existing actual tiny Python matrix fixture, NOT real product/source admission.
        from test_generic_core_evaluation import EvaluationTests
        fixture = EvaluationTests()
        try:
            original_run = acceptance.direct.run_step
            def produce(registry, products, declarations, step, output):
                return original_run(registry,products,declarations,step,output/'component')
            with patch.object(acceptance.direct,'run_step',side_effect=produce):
                products,prior = fixture.produce()
            self.assertEqual(document(prior.receipt.read_text()).field('status').value,'PASS')
            with tempfile.TemporaryDirectory() as temp:
                f = Fixture(Path(temp).resolve())
                container = prior.receipt.parent.parent
                acceptance_path = container/'acceptance.json'
                atomic_json(acceptance_path,{'step':44,'status':'COMPONENT_PASS','component_receipt_sha256':prior.sha256})
                anchor = Node({'path':str(acceptance_path),'sha256':file_hash(acceptance_path),
                    'inventory':{str(container/'component'/name):digest for name,digest in prior.files}})
                # Actual A reader returns its original anchor; only journal admission
                # and fixed-path placement are fixture seams, not producer authority.
                stage = Node({'step':44,'state':'RETURNED','native_exit':0,
                    'acceptance_anchor':anchor.value})
                atomic_json(container.parent/'stage.json',stage.value)
                with patch.object(acceptance.shared,'stage_root',return_value=container.parent), \
                     patch('generic_whole_gate_journal.entries',return_value=(stage,)), \
                     patch.object(acceptance,'_probe',side_effect=(products.base,products.candidate)), \
                     patch.object(acceptance,'parse_declarations',return_value=fixture.declarations), \
                     patch.object(acceptance.core,'evaluate',wraps=acceptance.core.evaluate) as evaluate, \
                     patch('subprocess.Popen',side_effect=AssertionError('45 must not launch')):
                    result = acceptance._dispatch(f.session,(f.selected['base'],f.selected['candidate']),45,f.root/'evaluation')
                self.assertEqual(result.status,'COMPONENT_PASS'); evaluate.assert_called_once()
                self.assertEqual(result.component_status,'PENDING')
        finally:
            _ = fixture.doCleanups()

    def test_actual_six_harmless_aggregates_fail_real_safety(self) -> None:
        # Reuses existing metadata stand-ins; aggregate children are actual exit0
        # shells. Load is mocked; safety/authentication are real and must reject.
        from test_generic_bench import BenchOwnedLifecycleTests
        fixture = BenchOwnedLifecycleTests(); fixture.setUp()
        try:
            script,output = fixture.owned_fixture()
            with tempfile.TemporaryDirectory() as temp:
                f = Fixture(Path(temp).resolve())
                config = f.session.field('acceptance').mapping()
                config['bench_script'] = {'path':str(script),'sha256':file_hash(script)}
                with patch.object(acceptance,'_perf',side_effect=fixture.products), \
                     patch.object(acceptance.bench,'wait_for_load',return_value=(1.0,2.0,3.0)):
                    result = acceptance._bench(f.session,(f.selected['base'],f.selected['candidate']),output)
                self.assertEqual(result.status,'FAIL')
                self.assertEqual(len(list(output.glob('*.finished.json'))),6)
                self.assertEqual(result.component_status,'FAIL')
        finally:
            _ = fixture.doCleanups()

if __name__ == '__main__':
    unittest.main()
