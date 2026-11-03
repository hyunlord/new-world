"""Pure endpoint/history/source plan checks; aggregate process seam is explicit."""
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from generic_gate import GateContractError, document
from generic_observations import ordinary_pair
from generic_test_invocation import invocations
from generic_test_items_plans import TESTS, items_plan
from generic_whole_gate_native import collect_family
from generic_whole_gate_projection import project_endpoint
from items_fixture import Fixture
from test_colony_jobs_source import tokens


class ItemsTests(unittest.TestCase):
    def test_public_collector_eleven_worlds_three_cases_full_history(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            fixture = Fixture(Path(directory))
            with patch('generic_whole_gate_native.aggregate_output', return_value=(fixture.raw(), 987)):
                family = collect_family(fixture.session(), fixture.stage(), 'colony_items')
            self.assertEqual(len(family.plans), 11)
            self.assertEqual([p.occurrence.end_tick for p in family.plans], [0, 0, 4680, *([14400] * 8)])
            for endpoint, history in zip(family.endpoints, family.histories, strict=True):
                observed = project_endpoint(endpoint, history)
                self.assertEqual(ordinary_pair(observed, observed)['classification'], 'ordinary')

    def test_original_invocation_scope_adds_workspace_not_gate17(self) -> None:
        for name in ('development', 'gate04', 'gate05'):
            self.assertIn('colony_items', [row.target for row in invocations(name)])
        self.assertNotIn('colony_items', [row.target for row in invocations('gate17')])

    def test_missing_world_cannot_borrow_complete_case_outcomes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            fixture = Fixture(Path(directory))
            (fixture.parent / f'{TESTS[2]}_s7_primary/terminal.json').unlink()
            with patch('generic_whole_gate_native.aggregate_output', return_value=(fixture.raw(), 987)), self.assertRaises(ValueError):
                _ = collect_family(fixture.session(), fixture.stage(), 'colony_items')

    def test_actual_resolved_order_mutation_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            fixture = Fixture(Path(directory))
            path = fixture.parent / f'{TESTS[2]}_s0_primary/checkpoint_resolved_orders.json'
            node = document(path.read_text())
            row = node.mapping()
            row['detail'] = {'commands': []}
            Fixture.write(path, row)
            with self.assertRaises(GateContractError):
                _ = items_plan(fixture.graph())

    def test_quarry_exhaustion_does_not_shorten_original_loop(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            fixture = Fixture(Path(directory))
            for name, key in (('terminal.json', 'terminal'), ('checkpoint_quarry_completion.json', 'state')):
                path = fixture.parent / f'{TESTS[1]}_s1_primary' / name
                node = document(path.read_text())
                row = node.mapping()
                state = node.field(key).mapping()
                state['tick'] = 461
                row[key] = state
                Fixture.write(path, row)
            with self.assertRaisesRegex(GateContractError, 'horizon'):
                _ = items_plan(fixture.graph())

    def test_candidate_full_history_loss_still_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            fixture = Fixture(Path(directory))
            path = fixture.parent / f'{TESTS[2]}_s0_primary/terminal.json'
            node = document(path.read_text())
            row = node.mapping()
            state = node.field('terminal').mapping()
            state.update(chronicle_entries=1, knowledge_lost=[{'tick': 1000, 'node': 'oral_history', 'holder': 1}])
            row['terminal'] = state
            Fixture.write(path, row)
            with patch('generic_whole_gate_native.aggregate_output', return_value=(fixture.raw(), 987)):
                family = collect_family(fixture.session(), fixture.stage(), 'colony_items')
            index = next(i for i, plan in enumerate(family.plans) if plan.occurrence.test == TESTS[2] and plan.occurrence.seed == 0)
            observed = project_endpoint(family.endpoints[index], family.histories[index])
            with self.assertRaisesRegex(GateContractError, 'KnowledgeLost'):
                _ = ordinary_pair(observed, observed)

    def test_original_source_tokens_are_unchanged(self) -> None:
        base = Path(__file__).resolve().parents[2]
        original = (Path(__file__).parent / 'fixtures/colony_items.original.rs').read_text()
        observed = (base / 'rust/crates/sim-test/tests/colony_items.rs').read_text()
        observed = observed.replace('pub mod save_gate_observation;', '').replace('use save_gate_observation::Watch;', '')
        self.assertEqual(tokens(original), tokens(observed))
