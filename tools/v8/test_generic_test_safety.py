"""Independent tiny Python children exercise pairing, not native qualification."""
import json
import unittest
from collections.abc import Callable
from dataclasses import replace

from generic_gate import GateContractError, document
from generic_test_safety import pair
from mux2_gate_inputs import Json
from test_generic_test_history import Graph, graph


def rewrite(data: Graph, edit: Callable[[dict[str, Json]], None]) -> None:
    def walk(value: Json) -> None:
        if isinstance(value, dict):
            if 'census_semantics' in value:
                edit(value)
            for child in value.values():
                walk(child)
        elif isinstance(value, list):
            for child in value:
                walk(child)
    for path in data.spec.output_root.rglob('*.json'):
        if path.name.startswith('process'):
            continue
        value = document(path.read_text()).value
        walk(value)
        _ = path.write_text(json.dumps(value))


def simple() -> list[dict[str, Json]]:
    return [{'test': 'ordinary_demo', 'seed': 0, 'branch': 'main', 'start': 0, 'end': 100}]


def loss(state: dict[str, Json]) -> None:
    if state['tick'] != 0:
        state.update(knowledge_lost=[{'tick': 50, 'node': 'clay_digging', 'holder': None}], chronicle_entries=1)


class SafetyTests(unittest.TestCase):
    def test_all46_keep_full_edges_and_zero_tick_loaded_occurrence(self) -> None:
        with graph() as base, graph() as candidate:
            result = pair(base.binding(), base.expected, candidate.binding(), candidate.expected)
            self.assertEqual(len(result.occurrences), 46)
            self.assertEqual(result.base_binding, base.binding())
            zero = next(row for row in result.occurrences if row.base.expected.branch == 'loaded_no_ticks')
            self.assertEqual(zero.safety['initial_tick'], 0)
            self.assertEqual(zero.safety['end_tick'], 1500)
            self.assertEqual(zero.candidate.observation.initial.tick, 1500)
            self.assertEqual(zero.candidate.observation.terminal.tick, 1500)
            self.assertTrue(zero.candidate.edges)
            self.assertFalse(zero.candidate.observation.history_complete)
            self.assertTrue(zero.candidate.history_complete)

    def test_base_loss_retained_but_matching_candidate_loss_rejected(self) -> None:
        with graph(simple()) as base, graph(simple()) as candidate:
            rewrite(base, loss)
            result = pair(base.binding(), base.expected, candidate.binding(), candidate.expected)
            self.assertEqual(result.occurrences[0].safety['base_lost'], [[50, 'clay_digging', None]])
            rewrite(candidate, loss)
            with self.assertRaisesRegex(GateContractError, 'KnowledgeLost'):
                _ = pair(base.binding(), base.expected, candidate.binding(), candidate.expected)

    def test_inherited_loaded_history_loss_cannot_pass(self) -> None:
        with graph() as base, graph() as candidate:
            rewrite(candidate, loss)
            with self.assertRaisesRegex(GateContractError, 'KnowledgeLost'):
                _ = pair(base.binding(), base.expected, candidate.binding(), candidate.expected)

    def test_founder_floor_uses_origin_and_terminal_live_census(self) -> None:
        def census(state: dict[str, Json], end_alive: int) -> None:
            count = 10 if state['tick'] == 0 else end_alive
            state.update(alive=count, alive_ids=list(range(count)), raw_agent_population=count, raw_agent_ids=list(range(count)))
        with graph() as base, graph() as candidate:
            rewrite(base, lambda row: census(row, 10))
            rewrite(candidate, lambda row: census(row, 8))
            result = pair(base.binding(), base.expected, candidate.binding(), candidate.expected)
            loaded = next(row for row in result.occurrences if row.candidate.expected.branch == 'loaded_no_ticks')
            self.assertEqual(loaded.safety['founders'], 10)
            self.assertEqual(loaded.safety['candidate_alive'], 8)
            rewrite(candidate, lambda row: census(row, 7))
            with self.assertRaisesRegex(GateContractError, 'below BASE-relative floor'):
                _ = pair(base.binding(), base.expected, candidate.binding(), candidate.expected)

    def test_disabled_capability_requires_real_null_histories(self) -> None:
        def disable(state: dict[str, Json]) -> None:
            state.update(knowledge_capability={'enabled': False, 'source': 'resources.rules.colony.knowledge'}, knowledge_lost=None)
        with graph(simple()) as base, graph(simple()) as candidate:
            rewrite(base, disable)
            rewrite(candidate, disable)
            self.assertEqual(len(pair(base.binding(), base.expected, candidate.binding(), candidate.expected).occurrences), 1)
            rewrite(candidate, lambda row: row.update(knowledge_lost=[]))
            with self.assertRaises(GateContractError):
                _ = pair(base.binding(), base.expected, candidate.binding(), candidate.expected)

    def test_incomplete_process_nonordinary_and_reused_binding_reject(self) -> None:
        with graph(simple()) as base, graph(simple(), exit_code=7) as candidate, self.assertRaises(GateContractError):
            _ = pair(base.binding(), base.expected, candidate.binding(), candidate.expected)
        with graph(simple()) as base, graph(simple()) as candidate:
            with self.assertRaises(GateContractError):
                _ = pair(base.binding(), base.expected, base.binding(), base.expected)
            altered = tuple(replace(row, ordinary=False) for row in candidate.expected)
            with self.assertRaises(GateContractError):
                _ = pair(base.binding(), base.expected, candidate.binding(), altered)
            rewrite(candidate, lambda row: row.update(raw_agent_ids=[1, 2, 3], raw_agent_population=3))
            with self.assertRaisesRegex(GateContractError, 'live/raw'):
                _ = pair(base.binding(), base.expected, candidate.binding(), candidate.expected)

    def test_missing_test_outcome_and_null_founding_reject(self) -> None:
        with graph(simple()) as base, graph(simple()) as candidate:
            base_plan = tuple(replace(row, native_test='absent_test') for row in base.expected)
            candidate_plan = tuple(replace(row, native_test='absent_test') for row in candidate.expected)
            with self.assertRaises(GateContractError):
                _ = pair(base.binding(), base_plan, candidate.binding(), candidate_plan)
            candidate.change('ordinary_demo_s0_main/terminal.json', lambda row: row.update(founding=None))
            with self.assertRaises(GateContractError):
                _ = pair(base.binding(), base.expected, candidate.binding(), candidate.expected)

    def test_external_null_ancestor_and_descendant_never_project(self) -> None:
        test = 'genuine_legacy_inventory_preserves_fields_and_current_policy_continuation'
        rows: list[dict[str, Json]] = [
            {'test': test, 'seed': 5, 'branch': 'fixture_0_initial', 'start': 2000, 'end': 2000,
             'external': True, 'external_origin': True},
            {'test': test, 'seed': 5, 'branch': 'fixture_0_twin', 'start': 2000, 'end': 3000,
             'parent': 'fixture_0_initial', 'label': 'current', 'external_origin': True},
        ]
        with graph(rows) as base, graph(rows) as candidate, self.assertRaisesRegex(GateContractError, 'unresolved'):
            _ = pair(base.binding(), base.expected, candidate.binding(), candidate.expected)

    def test_missing_coverage_and_unequal_actual_horizon_reject(self) -> None:
        rows = simple()
        rows[0]['end'] = 101
        with graph(simple()) as base, graph(rows) as candidate:
            with self.assertRaisesRegex(GateContractError, 'horizon'):
                _ = pair(base.binding(), base.expected, candidate.binding(), candidate.expected)
            with self.assertRaises(GateContractError):
                _ = pair(base.binding(), base.expected, candidate.binding(), ())
            altered = tuple(replace(row, branch='foreign') for row in candidate.expected)
            with self.assertRaises(GateContractError):
                _ = pair(base.binding(), base.expected, candidate.binding(), altered)


if __name__ == '__main__':
    _ = unittest.main()
