"""Dispatcher wiring tests; mocked family seams confer no native qualification."""
import unittest
from contextlib import ExitStack
from unittest.mock import patch

from generic_gate import GateContractError
from generic_whole_gate_native import evaluate_family
from mux2_gate_inputs import Node


def accounting() -> dict:
    arm = {'remaining_world_routes': [
        {'package': 'sim-test', 'target': 'engine_ascii', 'kind': 'test', 'reason': 'old generic gap'},
        {'package': 'sim-test', 'target': 'engine_knowledge_ffi', 'kind': 'test', 'reason': 'old knowledge gap'},
        {'package': 'sim-test', 'target': 'm6b_presentation_defaults', 'kind': 'test', 'reason': 'old defaults gap'},
        {'package': 'sim-test', 'target': 'other', 'kind': 'test', 'reason': 'source classification required'}],
        'doctest_world_routes': [{'package': 'sim-core', 'reason': 'snippet classification'}]}
    return {'status': 'PASS', 'scope': 'WORKSPACE_AGGREGATE_ACCOUNTING_ONLY', 'base': arm, 'candidate': arm,
            'safety': ['must never enter safety']}


class WorkspaceDispatchTests(unittest.TestCase):
    def run_dispatch(self, step: int, result: dict | Exception, ascii_gap: bool = False, knowledge_rows: bool = False) -> tuple[dict, int]:
        session = Node({'arms': {arm: {'root': '/synthetic', 'source_inputs': {}} for arm in ('base', 'candidate')}})
        with ExitStack() as stack:
            for name, value in [('TARGETS', {4: (), 5: (), 24: ()}), ('STEWARD_PINS', {})]:
                _ = stack.enter_context(patch('generic_whole_gate_native.'+name, value))
            _ = stack.enter_context(patch('generic_whole_gate_native.aggregate_output', return_value=('', 123)))
            for name in ('generic_whole_gate_native_probes.evaluate_replays', 'generic_whole_gate_appearance.evaluate_appearance',
                         'generic_whole_gate_baseline_observer.evaluate_family',
                         'generic_whole_gate_knowledge_defaults.evaluate_knowledge_defaults'):
                count = 6 if knowledge_rows and name.endswith('evaluate_knowledge_defaults') else 0
                _ = stack.enter_context(patch(name, return_value={'safety': [{} for _ in range(count)], 'coverage': [{} for _ in range(count)]}))
            _ = stack.enter_context(patch('generic_whole_gate_colony_ffi.evaluate_family', return_value={'status': 'PASS', 'safety': [], 'coverage': []}))
            _ = stack.enter_context(patch('generic_whole_gate_ascii.evaluate_ascii', return_value={
                'safety': [], 'coverage': [], 'unsupported': ['actual carrier horizon differs'] if ascii_gap else []}))
            mock = stack.enter_context(patch('generic_whole_gate_workspace.evaluate_family',
                side_effect=result if isinstance(result, Exception) else None,
                return_value=result if isinstance(result, dict) else None))
            output = evaluate_family(session, Node({'step': step, 'arm': 'base'}), Node({'step': step, 'arm': 'candidate'}))
            return output, mock.call_count

    def test_debug_and_release_invoke_accounting_but_stay_incomplete(self) -> None:
        for step in (4, 5):
            with self.subTest(step=step):
                result, calls = self.run_dispatch(step, accounting())
                self.assertEqual(calls, 1)
                self.assertEqual(result['status'], 'INCOMPLETE')
                self.assertEqual(result['safety'], [])
                self.assertEqual(result['workspace_accounting'], accounting())
                self.assertEqual(len(result['unsupported']), 4)
                self.assertNotIn('old generic gap', str(result['unsupported']))

    def test_six_knowledge_defaults_rows_remove_only_their_exact_generic_gaps(self) -> None:
        result, _ = self.run_dispatch(4, accounting(), knowledge_rows=True)
        self.assertEqual(result['status'], 'INCOMPLETE')
        self.assertEqual(len(result['safety']), 6)
        self.assertEqual(len(result['coverage']), 6)
        self.assertEqual(len(result['unsupported']), 4)
        self.assertNotIn('old knowledge gap', str(result['unsupported']))
        self.assertNotIn('old defaults gap', str(result['unsupported']))
        self.assertIn('source classification required', str(result['unsupported']))

    def test_actual_ascii_pending_survives(self) -> None:
        result, _ = self.run_dispatch(4, accounting(), True)
        self.assertIn('actual carrier horizon differs', result['unsupported'])

    def test_accounting_failure_is_not_masked(self) -> None:
        with self.assertRaisesRegex(GateContractError, 'missing original target'):
            _ = self.run_dispatch(4, GateContractError('CONTRACT_REJECTED', '/targets', 'missing original target'))

    def test_incomplete_accounting_cannot_be_accepted(self) -> None:
        result = accounting()
        result['status'] = 'INCOMPLETE'
        with self.assertRaises(GateContractError):
            _ = self.run_dispatch(5, result)

    def test_save24_does_not_run_workspace_again(self) -> None:
        result, calls = self.run_dispatch(24, accounting())
        self.assertEqual(calls, 0)
        self.assertEqual(result['status'], 'PASS')
        self.assertEqual(result['workspace_accounting'], {})


if __name__ == '__main__':
    _ = unittest.main()
