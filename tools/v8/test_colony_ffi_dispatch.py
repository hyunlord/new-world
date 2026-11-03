"""Real new collector inside dispatcher; independent siblings are synthetic seams."""
import tempfile
import unittest
from contextlib import ExitStack
from pathlib import Path
from unittest.mock import patch

from colony_ffi_fixture import Fixture
from generic_gate import GateContractError, document
from generic_whole_gate_native import evaluate_family
from mux2_gate_inputs import Json, Node


def accounting() -> dict[str, Json]:
    arm: dict[str, Json] = {'remaining_world_routes': [
        {'package': 'sim-test', 'target': 'engine_colony_ffi', 'kind': 'test'},
        {'package': 'sim-test', 'target': 'colony_knowledge', 'kind': 'test'}], 'doctest_world_routes': []}
    return {'status': 'PASS', 'scope': 'WORKSPACE_AGGREGATE_ACCOUNTING_ONLY', 'base': arm, 'candidate': arm}


def dispatch(run: Fixture) -> dict[str, Json]:
    with ExitStack() as stack:
        _ = stack.enter_context(patch('generic_whole_gate_native.TARGETS', {4: (), 5: (), 17: (), 24: ()}))
        empty_pins: dict[str, str] = {}
        _ = stack.enter_context(patch('generic_whole_gate_native.STEWARD_PINS', empty_pins))
        for name in ('generic_whole_gate_native_probes.evaluate_replays',
                     'generic_whole_gate_baseline_observer.evaluate_family',
                     'generic_whole_gate_twins.evaluate_family',
                     'generic_whole_gate_appearance.evaluate_appearance', 'generic_whole_gate_ascii.evaluate_ascii',
                     'generic_whole_gate_knowledge_defaults.evaluate_knowledge_defaults'):
            _ = stack.enter_context(patch(name, return_value={'safety': [], 'coverage': []}))
        _ = stack.enter_context(patch('generic_whole_gate_m1.collect_m1', return_value={}))
        _ = stack.enter_context(patch('generic_whole_gate_workspace.evaluate_family', return_value=accounting()))
        return evaluate_family(Node(run.session), Node(run.stages['base']), Node(run.stages['candidate']))


class DispatchTests(unittest.TestCase):
    def test_actual_new_collector_reached_on_all_three_original_routes(self) -> None:
        for step in (4, 5, 17):
            with self.subTest(step=step), tempfile.TemporaryDirectory() as temporary:
                run = Fixture(Path(temporary), step)
                result = Node(dispatch(run))
                result.field('colony_ffi').field('scope').equals('TWO_ORIGINAL_COLONY_FFI_CASES_ONLY')
                self.assertEqual(len(result.field('safety').array()), 2)
                self.assertEqual(len(result.field('coverage').array()), 2)
                result.field('coverage').array()[1].field('classification').equals('CONTROLLED_DIRECT_CARRY_INSERT_TICK0')
                if step in {4, 5}:
                    result.field('status').equals('INCOMPLETE')
                    gaps = result.field('unsupported').array()
                    self.assertEqual(len(gaps), 2)
                    for gap in gaps:
                        gap.field('world_route').field('target').equals('colony_knowledge')

    def test_actual_bad_endpoint_cannot_remove_target_gap(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run = Fixture(Path(temporary))
            (run.folder('candidate', 1) / 'checkpoint_carry_intervention.json').unlink()
            run.seal('candidate')
            with self.assertRaisesRegex(GateContractError, 'missing retained intervention'):
                _ = dispatch(run)

    def test_incomplete_new_callable_is_not_silently_admitted(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run = Fixture(Path(temporary))
            with patch('generic_whole_gate_colony_ffi.evaluate_family', return_value={'status': 'INCOMPLETE', 'safety': [], 'coverage': []}), self.assertRaises(GateContractError):
                _ = dispatch(run)

    def test_actual_blueprint_result_count_is_one(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run = Fixture(Path(temporary))
            path = run.folder('candidate', 0) / 'terminal.json'
            node = run.read(path)
            terminal = dict(Node(node['terminal']).mapping())
            commands = [row.value for row in Node(terminal['command_log']).array()]
            blueprint = dict(Node(commands[2]).mapping())
            blueprint['count'] = 2
            commands[2] = blueprint
            terminal['command_log'] = commands
            node['terminal'] = terminal
            run.write(path, node)
            run.seal('candidate')
            with self.assertRaisesRegex(GateContractError, 'one accepted hut'):
                _ = dispatch(run)

    def test_new_observer_entry_remains_unavailable(self) -> None:
        matrix = document(Path(__file__).with_name('whole_gate_observer_matrix.json').read_text())
        rows = [row for row in matrix.field('rows').array() if row.field('id').value == 'native_engine_colony_ffi']
        self.assertEqual(len(rows), 1)
        rows[0].field('comparator').equals('UNAVAILABLE')
        rows[0].field('evidence').equals('NOT_RUN')
        rows[0].field('artifacts').equals([])
        path = 'rust/crates/sim-test/tests/engine_colony_ffi.rs'
        digest = rows[0].field('source_refs').field(path).text()
        matrix.field('observer_postimages').field(path).equals(['100644', digest])
        from generic_build import OBSERVER_EXACT_PATHS
        from generic_whole_gate_m1 import SOURCE_PINS
        self.assertIn(path, OBSERVER_EXACT_PATHS)
        self.assertEqual(SOURCE_PINS[path], digest)


if __name__ == '__main__':
    _ = unittest.main()
