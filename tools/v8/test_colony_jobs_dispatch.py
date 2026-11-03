"""Real source/checkpoint collector with isolated sibling dispatch seams, never gate evidence."""
import tempfile
import unittest
from contextlib import ExitStack
from pathlib import Path
from unittest.mock import patch

from generic_gate import GateContractError, document
from generic_whole_gate_native import evaluate_family
from jobs_fixture import Fixture
from mux2_gate_inputs import Json, Node


def run_pair(base: Fixture, candidate: Fixture) -> dict[str, Json]:
    session = Node({'arms': {arm: {'root': str(fixture.source), 'source_inputs': dict(fixture.inputs)}
                             for arm, fixture in (('base', base), ('candidate', candidate))}})
    accounting: dict[str, Json] = {'remaining_world_routes': [
        {'package': 'sim-test', 'target': 'colony_jobs', 'kind': 'test'},
        {'package': 'sim-test', 'target': 'other', 'kind': 'test'}], 'doctest_world_routes': []}
    with ExitStack() as stack:
        _ = stack.enter_context(patch('generic_whole_gate_native.TARGETS', {4: ('colony_jobs',)}))
        _ = stack.enter_context(patch('generic_whole_gate_native.STEWARD_PINS', dict[str, str]()))
        _ = stack.enter_context(patch('generic_whole_gate_native.aggregate_output', return_value=(base.raw(), 987)))
        for name in ('generic_whole_gate_native_probes.evaluate_replays', 'generic_whole_gate_appearance.evaluate_appearance',
                     'generic_whole_gate_baseline_observer.evaluate_family', 'generic_whole_gate_twins.evaluate_family',
                     'generic_whole_gate_knowledge_fresh.evaluate_family', 'generic_whole_gate_knowledge_defaults.evaluate_knowledge_defaults', 'generic_whole_gate_ascii.evaluate_ascii'):
            _ = stack.enter_context(patch(name, return_value={'safety': [], 'coverage': [], 'unsupported': []}))
        _ = stack.enter_context(patch('generic_whole_gate_colony_ffi.evaluate_family', return_value={'status': 'PASS', 'safety': [], 'coverage': []}))
        _ = stack.enter_context(patch('generic_whole_gate_workspace.evaluate_family', return_value={
            'status': 'PASS', 'scope': 'WORKSPACE_AGGREGATE_ACCOUNTING_ONLY', 'base': accounting, 'candidate': accounting}))
        return evaluate_family(session, base.stage(), candidate.stage('candidate'))


class DispatchTests(unittest.TestCase):
    def test_ten_worlds_remove_only_target_gap_not_whole_stage(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = run_pair(Fixture(root / 'base'), Fixture(root / 'candidate'))
            self.assertEqual(result['status'], 'INCOMPLETE')
            self.assertEqual(len(Node(result).field('safety').array()), 10)
            self.assertEqual(len(Node(result).field('coverage').array()), 10)
            gaps = Node(result).field('unsupported').array()
            self.assertEqual(len(gaps), 2)
            for gap in gaps:
                gap.field('world_route').field('target').equals('other')

    def test_actual_horizon_mismatch_not_normalized(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            base, candidate = Fixture(root / 'base'), Fixture(root / 'candidate')
            folder = candidate.parent / 'jobs_haul_to_stockpile_s0_primary'
            for filename, state_key in (('terminal.json', 'terminal'), ('checkpoint_event_stop.json', 'state')):
                path = folder / filename
                row = document(path.read_text()).mapping()
                state = Node(row[state_key]).mapping()
                state['tick'] = 101
                row[state_key] = state
                if filename == 'terminal.json':
                    row['completed_ticks'] = 101
                else:
                    detail = Node(row['detail']).mapping()
                    detail['used'] = 101
                    row['detail'] = detail
                Fixture.write(path, row)
            with self.assertRaisesRegex(GateContractError, 'horizons differ'):
                _ = run_pair(base, candidate)

    def test_full_history_candidate_loss_still_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            base, candidate = Fixture(root / 'base'), Fixture(root / 'candidate')
            folder = candidate.parent / 'jobs_cancel_on_hunger_s1_primary'
            for filename, key in (('terminal.json', 'terminal'), ('checkpoint_event_stop.json', 'state')):
                path = folder / filename
                row = document(path.read_text()).mapping()
                state = Node(row[key]).mapping()
                state.update(chronicle_entries=1, knowledge_lost=[{'tick': 101, 'node': 'oral_history', 'holder': 1}])
                row[key] = state
                Fixture.write(path, row)
            with self.assertRaisesRegex(GateContractError, 'KnowledgeLost'):
                _ = run_pair(base, candidate)
