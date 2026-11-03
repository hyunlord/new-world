"""Callable readiness does not settle actual pending native safety."""
import tempfile
import unittest
from contextlib import ExitStack
from pathlib import Path
from unittest.mock import patch

from generic_gate import GateContractError
from generic_whole_gate_families import evaluate
from mux2_gate_inputs import Node


class NamedIntegrationTests(unittest.TestCase):
    def collect(self, pending: bool, missing: bool = False, fail: bool = False):
        with tempfile.TemporaryDirectory() as temporary, ExitStack() as stack:
            steps = {(arm, step): Node({'arm': arm, 'step': step}) for arm in ('base', 'candidate')
                     for step in (13, 17, 32, 37, 38, 42, 43, 49) if not (missing and step == 32)}
            def native(session, base, candidate):
                return {'status': 'INCOMPLETE' if pending and base.field('step').value == 37 else 'PASS', 'safety': []}
            stack.enter_context(patch('generic_whole_gate_native.evaluate_family', side_effect=native))
            stack.enter_context(patch('generic_whole_gate_oracles.evaluate_family', return_value={'status': 'PASS'}))
            stack.enter_context(patch('generic_whole_gate_named.evaluate_family', return_value={'status': 'PASS'},
                side_effect=GateContractError('CONTRACT_REJECTED', '/oracle', 'original failed') if fail else None))
            return evaluate(Node({'output': str(Path(temporary).resolve())}), steps)

    def test_pending37_never_completes_named_family(self):
        result = self.collect(True)
        self.assertNotIn('named_integration_boundaries', result.completed)
        self.assertTrue(any('native step37' in value for value in result.incomplete))
        self.assertEqual(result.safety, ())

    def test_all_actual_named_joins_needed_for_completion(self):
        self.assertIn('named_integration_boundaries', self.collect(False).completed)
        self.assertNotIn('named_integration_boundaries', self.collect(False, missing=True).completed)

    def test_original_failure_remains_error(self):
        result = self.collect(False, fail=True)
        self.assertNotIn('named_integration_boundaries', result.completed)
        self.assertTrue(any('named step13' in error for error in result.errors))


if __name__ == '__main__':
    unittest.main()
