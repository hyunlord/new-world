"""Public twin APIs with real disk anchors and synthetic receipts; never native proof."""
import tempfile
import unittest
from contextlib import ExitStack
from pathlib import Path
from typing import final
from unittest.mock import patch

from generic_gate import GateContractError
from generic_whole_gate_native import evaluate_family as dispatch_family
from generic_whole_gate_twins import collect, evaluate_family
from mux2_gate_inputs import Json, Node
from twin_fixture import Fixture


def dispatch(run: Fixture) -> dict[str, Json]:
    with ExitStack() as stack:
        _ = stack.enter_context(patch('generic_whole_gate_native.TARGETS', {4: (), 5: ()}))
        empty_pins: dict[str, str] = {}
        _ = stack.enter_context(patch('generic_whole_gate_native.STEWARD_PINS', empty_pins))
        for name in ('generic_whole_gate_native_probes.evaluate_replays',
                     'generic_whole_gate_baseline_observer.evaluate_family',
                     'generic_whole_gate_colony_ffi.evaluate_family',
                     'generic_whole_gate_appearance.evaluate_appearance', 'generic_whole_gate_ascii.evaluate_ascii',
                     'generic_whole_gate_knowledge_defaults.evaluate_knowledge_defaults'):
            _ = stack.enter_context(patch(name, return_value={'status': 'PASS', 'safety': [], 'coverage': []}))
        unresolved: dict[str, Json] = {'remaining_world_routes': [
            {'package': 'sim-test', 'target': 'sim_test', 'kind': 'lib', 'reason': 'remaining library worlds'}],
            'doctest_world_routes': []}
        accounting: dict[str, Json] = {'status': 'PASS', 'scope': 'WORKSPACE_AGGREGATE_ACCOUNTING_ONLY',
                                       'base': unresolved, 'candidate': unresolved}
        _ = stack.enter_context(patch('generic_whole_gate_workspace.evaluate_family', return_value=accounting))
        return dispatch_family(*run.nodes())


@final
class PublicTwinTests(unittest.TestCase):
    def test_collect_evaluate_dispatch_disk_anchors_both_workspace_profiles(self) -> None:
        for step in (4, 5):
            with self.subTest(step=step), tempfile.TemporaryDirectory() as temporary:
                run = Fixture(Path(temporary), step)
                session, base, candidate = run.nodes()
                self.assertEqual(len(collect(session, base)), 8)
                self.assertEqual(len(collect(session, candidate)), 8)
                result = Node(evaluate_family(session, base, candidate))
                self.assertEqual(len(result.field('safety').array()), 8)
                self.assertEqual(len(result.field('coverage').array()), 8)
                result.field('library_complete').equals(False)
                integrated = Node(dispatch(run))
                integrated.field('status').equals('INCOMPLETE')
                self.assertEqual(len(integrated.field('coverage').array()), 8)
                self.assertEqual(len(integrated.field('safety').array()), 8)
                self.assertTrue(integrated.field('unsupported').array())

    def test_resealed_order_violation_rejects_public_evaluate_and_dispatch(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run = Fixture(Path(temporary))
            a = run.observer('candidate') / 'probe_pid1123_world1005.json'
            b = run.observer('candidate') / 'probe_pid1123_world1060.json'
            left, right = run.read(a), run.read(b)
            left['world_sequence'], right['world_sequence'] = right['world_sequence'], left['world_sequence']
            run.write(a, right)
            run.write(b, left)
            run.seal('candidate')
            for invoke in (lambda: evaluate_family(*run.nodes()), lambda: dispatch(run)):
                with self.assertRaisesRegex(GateContractError, 'twin precedes|before ordinary'):
                    _ = invoke()

    def test_missing_inventory_and_changed_bytes_reject(self) -> None:
        for reseal in (False, True):
            with self.subTest(reseal=reseal), tempfile.TemporaryDirectory() as temporary:
                run = Fixture(Path(temporary))
                path = run.observer('candidate') / 'probe_pid1123_world1005.json'
                path.unlink()
                if reseal:
                    run.seal('candidate')
                with self.assertRaises(GateContractError):
                    _ = evaluate_family(*run.nodes())
        with tempfile.TemporaryDirectory() as temporary:
            run = Fixture(Path(temporary))
            path = run.observer('candidate') / 'probe_pid1123_world1005.json'
            _ = path.write_text('{}')
            with self.assertRaises(GateContractError):
                _ = evaluate_family(*run.nodes())

    def test_resealed_missing_outcome_and_changed_source_reject(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run = Fixture(Path(temporary))
            path = run.process('candidate').with_name('stdout.log')
            _ = path.write_text(path.read_text().replace('native_colony_twins_', 'foreign_'))
            run.seal('candidate')
            with self.assertRaises(GateContractError):
                _ = evaluate_family(*run.nodes())
        with tempfile.TemporaryDirectory() as temporary:
            run = Fixture(Path(temporary))
            path = run.source / 'rust/crates/sim-test/src/probe/gate_observation_tests.rs'
            _ = path.write_text(path.read_text() + '\n// drift\n')
            with self.assertRaises(GateContractError):
                _ = evaluate_family(*run.nodes())

    def test_environment_receipt_and_directory_anchors_reject(self) -> None:
        for defect in ('environment', 'receipt', 'directory', 'unbound_extra'):
            with self.subTest(defect=defect), tempfile.TemporaryDirectory() as temporary:
                run = Fixture(Path(temporary))
                if defect == 'environment':
                    Node(run.stages['candidate']['environment']).mapping()['WORLDSIM_GATE_OBSERVER'] = '0'
                    run.seal('candidate')
                elif defect == 'receipt':
                    run.stages['candidate']['process_receipt_sha256'] = '0' * 64
                elif defect == 'directory':
                    run.stages['candidate']['observer_directories'] = {}
                else:
                    run.write(run.observer('candidate') / 'probe_pid1123_world9999.json', {})
                with self.assertRaises(GateContractError):
                    _ = evaluate_family(*run.nodes())
