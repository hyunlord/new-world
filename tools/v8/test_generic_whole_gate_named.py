"""Actual harmless original Python13 and synthetic resolver evidence; no Godot run."""
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import generic_process
from generic_gate import GateContractError
from generic_process import ProcessSpec, environment_for, execute, file_hash
from generic_resolver_run import COMMAND, PINS
from generic_whole_gate_named import RESOLVER_INPUTS, bridge_terminal, collect
from mux2_gate_inputs import Node

ROOT = Path(generic_process.__file__).resolve().parents[2]


class NamedTests(unittest.TestCase):
    def test_actual_original_python13(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary).resolve()
            bash = Path('/bin/bash')
            paths = [ROOT/'tools/v8/bridge_api.py', ROOT/'docs/v8/bridge_api_read.txt', *(ROOT/'rust/crates/sim-bridge/src').rglob('*.rs')]
            inputs = {p.relative_to(ROOT).as_posix(): file_hash(p) for p in paths}
            argv = ('python3', 'tools/v8/bridge_api.py', '.', '--check', 'docs/v8/bridge_api_read.txt')
            spec = ProcessSpec((str(bash), '-c', 'exec "$@" 2>&1', 'whole-step', *argv), ROOT,
                               bash, file_hash(bash), ROOT, tuple(inputs.items()), output/'process', ())
            actual = execute(spec)
            self.assertEqual(actual.native_exit, 0)
            stage = Node({'step': 13, 'arm': 'candidate', 'state': 'RETURNED', 'native_exit': actual.native_exit,
                          'pid': actual.pid, 'argv': list(argv), 'cwd': str(ROOT), 'environment': environment_for(spec),
                          'process_receipt': str(actual.receipt), 'process_receipt_sha256': file_hash(actual.receipt)})
            session = Node({'output': str(output), 'arms': {'candidate': {'root': str(ROOT), 'source_inputs': inputs}},
                            'harness': {'root': str(ROOT), 'inputs': {'tools/v8/gate.sh': file_hash(ROOT/'tools/v8/gate.sh')}},
                            'bash': {'path': str(bash), 'sha256': file_hash(bash)}})
            result = collect(session, stage)
            self.assertEqual(result['status'], 'PASS')
            self.assertEqual(result['worlds'], [])
            self.assertIsNone(result['founding_census'])

    def resolver(self, mutation: str = '') -> dict:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary).resolve()
            log = output/'candidate'
            log.mkdir()
            expected = json.loads((ROOT/'tools/v8/resolver_parity_expected.json').read_text())
            raw = {'problems': [], 'flora_problems': [], 'results': expected['results'], 'flora_results': expected['flora_results']}
            path = log/'resolver_godot.json'
            path.write_text(json.dumps(raw))
            names = {*PINS, *RESOLVER_INPUTS, 'assets/v8art/flora/tests/test_alder.json'} - {'tools/v8/gate.sh'}
            inputs = {name: file_hash(ROOT/name) for name in names}
            sha = file_hash(path)
            if mutation == 'artifact':
                path.write_text('{}')
            if mutation == 'source':
                inputs['scripts/world/art/material_resolver.gd'] = '0'*64
            env = {'WORLDSIM_ORACLE_CHILD_PLAN': '/foreign'} if mutation == 'environment' else {}
            stage = Node({'step': 32, 'arm': 'candidate', 'argv': ['bash', '-c', COMMAND, str(log)],
                          'environment': env, 'observer_artifacts': {str(path): sha},
                          'observer_modes': {str(path): path.lstat().st_mode}})
            session = Node({'output': str(output), 'arms': {'candidate': {'root': str(ROOT), 'source_inputs': inputs}}})
            m, f = len(expected['results']), len(expected['flora_results'])
            text = f'RESOLVER_PARITY_DONE cases={m} problems=0 flora_cases={f} flora_problems=0\nresolver_parity: PASS ({m} material and {f} flora cases, 0 differences)\n'
            if mutation == 'marker':
                text = text.split('\n', 1)[1]
            if mutation == 'oracle':
                text = text.replace('0 differences', '1 differences')
            with patch('generic_whole_gate_named.aggregate_output', return_value=(text, 123)):
                return collect(session, stage)

    def test_resolver_source_qualified_synthetic_output(self) -> None:
        self.assertEqual(self.resolver()['status'], 'PASS')

    def test_resolver_rejects_changed_evidence(self) -> None:
        for mutation in ('artifact', 'source', 'environment', 'marker', 'oracle'):
            with self.subTest(mutation=mutation), self.assertRaises(GateContractError):
                self.resolver(mutation)

    def test_bridge_mismatch_is_not_no_world_success(self) -> None:
        with self.assertRaises(GateContractError):
            bridge_terminal(ROOT, 'bridge api: 0 items; expected 0 (0 at base minus 0 removed)\n')


if __name__ == '__main__':
    unittest.main()
