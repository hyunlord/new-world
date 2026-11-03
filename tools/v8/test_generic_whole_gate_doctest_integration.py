"""Synthetic aggregate lifecycle seam; source/accounting remains real and pure."""
import unittest
from pathlib import Path
from unittest.mock import patch

import generic_process
from generic_whole_gate_doctests import EXPECTED, source_closure
from generic_whole_gate_workspace import evaluate_family
from generic_whole_gate_workspace_plan import MANIFESTS
from mux2_gate_inputs import Json, Node
from test_generic_whole_gate_workspace import fixture

ROOT = Path(generic_process.__file__).resolve().parents[2]


class IntegrationTests(unittest.TestCase):
    def test_no_doctest_gap_does_not_grant_whole_world_pass(self) -> None:
        raw, inputs = fixture()
        inputs.update(MANIFESTS)
        for package in EXPECTED:
            inputs.update(source_closure(ROOT, package))
        historical = (Path(__file__).parent / 'fixtures/original_doctests.txt').read_text()
        raw = raw[:raw.index('   Doc-tests ')] + historical
        product: dict[str, Json] = {'root': str(ROOT), 'source_inputs': dict(inputs)}
        session = Node({'arms': {'base': product, 'candidate': product}})
        stage: dict[str, Json] = {'step': 4, 'environment': {}, 'argv': ['bash', '-c',
                 'cd rust && cargo test --workspace --no-fail-fast -- --test-threads=8']}
        with patch('generic_whole_gate_workspace.aggregate_output', return_value=(raw, 123)), \
             patch('generic_whole_gate_workspace.classify_core_pure', return_value=[]), \
             patch('generic_whole_gate_workspace.classify_core_material', return_value=[]), \
             patch('generic_whole_gate_workspace.classify_config_pure', return_value=[]), \
             patch('generic_whole_gate_workspace.classify_routes', return_value=[]):
            result = evaluate_family(session, Node(stage | {'arm': 'base'}), Node(stage | {'arm': 'candidate'}))
        self.assertEqual(result['scope'], 'WORKSPACE_AGGREGATE_ACCOUNTING_ONLY')
        self.assertFalse(result['publication_authority'])
        self.assertEqual(result['safety'], [])
        for arm in ('base', 'candidate'):
            row = result[arm]
            assert isinstance(row, dict)
            self.assertEqual(row['doctest_world_routes'], [])
            self.assertTrue(row['remaining_world_routes'])
            self.assertEqual(row['world_safety'], 'SEPARATE_SOURCE_FAMILY_JOINS_REQUIRED')
            reviewed = row['reviewed_doctest_routes']
            assert isinstance(reviewed, list)
            self.assertEqual(len(reviewed), 5)

    def test_named_original_case_keeps_doctest_gap(self) -> None:
        raw, inputs = fixture()  # One source-labelled synthetic doctest per package.
        inputs.update(MANIFESTS)
        for package in EXPECTED:
            inputs.update(source_closure(ROOT, package))
        product: dict[str, Json] = {'root': str(ROOT), 'source_inputs': dict(inputs)}
        session = Node({'arms': {'base': product, 'candidate': product}})
        stage: dict[str, Json] = {'step': 4, 'environment': {}, 'argv': ['bash', '-c',
                 'cd rust && cargo test --workspace --no-fail-fast -- --test-threads=8']}
        with patch('generic_whole_gate_workspace.aggregate_output', return_value=(raw, 123)), \
             patch('generic_whole_gate_workspace.classify_core_pure', return_value=[]), \
             patch('generic_whole_gate_workspace.classify_core_material', return_value=[]), \
             patch('generic_whole_gate_workspace.classify_config_pure', return_value=[]), \
             patch('generic_whole_gate_workspace.classify_routes', return_value=[]):
            result = evaluate_family(session, Node(stage | {'arm': 'base'}), Node(stage | {'arm': 'candidate'}))
        for arm in ('base', 'candidate'):
            row = result[arm]
            assert isinstance(row, dict)
            gaps = row['doctest_world_routes']
            assert isinstance(gaps, list)
            self.assertEqual(len(gaps), 5)
            self.assertTrue(all(isinstance(gap, dict) and gap['status'] == 'INCOMPLETE' for gap in gaps))


if __name__ == '__main__':
    _ = unittest.main()
