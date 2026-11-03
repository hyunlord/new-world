"""Synthetic whole-output arithmetic and actual read-only source contracts; no native runs."""
import unittest
from pathlib import Path

import generic_process
from generic_gate import GateContractError
from generic_process import file_hash
from generic_whole_gate_workspace import account_payload, account_workspace
from generic_whole_gate_workspace_plan import TARGETS
from generic_workspace_cases import nested_policy
from mux2_gate_inputs import Node

ROOT = Path(generic_process.__file__).resolve().parents[2]


def payload(names: tuple[str, ...], ignored: tuple[str, ...] = (), children: tuple[str, ...] = ()) -> str:
    count = len(names)
    rows = [f'\nrunning {count} '+('test' if count == 1 else 'tests')]
    rows += ['running 1 test' for _ in children]
    rows += [f'test {name} ... '+('ignored' if name in ignored else 'ok') for name in names]
    for child in children:
        rows += [f'test {child} ... ok', f'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; {count-1} filtered out; finished in 0.01s']
    rows += [f'\ntest result: ok. {count-len(ignored)} passed; 0 failed; {len(ignored)} ignored; 0 measured; 0 filtered out; finished in 0.01s\n\n']
    return '\n'.join(rows)


def fixture() -> tuple[str, dict[str, str]]:
    parts: list[str] = []
    inputs: dict[str, str] = {}
    for package, name, kind, source in TARGETS:
        inputs[source] = file_hash(ROOT/source)
        rules, _, _ = nested_policy(package, name, kind)
        for row in rules:
            inputs[row[0]] = file_hash(ROOT/row[0])
        names = tuple(sorted({'outer_other', *(name for row in rules for name in row[1:3])})) if rules else ('plain',)
        if name == 'integration':
            names = ('test_zero_systems_baseline',) if package == 'sim-engine' else ('harness_influence_update_name',)
        children = tuple(row[2] for row in rules for _ in range(row[3]))
        label = 'src/lib.rs' if kind == 'lib' else f'src/bin/{name}.rs' if kind == 'bin' else f'tests/{name}.rs'
        parts.append('     Running '+('unittests ' if kind in {'lib', 'bin'} else '')+f'{label} (target/debug/deps/{name}-abcdef)\n'+payload(names, (), children))
    for package in sorted({row[0] for row in TARGETS}):
        parts.append('   Doc-tests '+package.replace('-', '_')+'\n'+payload(('src/lib.rs - module::example (line 12)',)))
    return ''.join(parts), inputs


class WorkspaceAggregateTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.raw, cls.inputs = fixture()

    def test_full_actual_source_topology_with_synthetic_outcomes(self) -> None:
        result = account_workspace(self.raw, 4, ROOT, self.inputs, Node({}))
        self.assertEqual(len(result), len(TARGETS)+5)

    def test_missing_target(self) -> None:
        first = self.raw.index('     Running ', 1)
        with self.assertRaisesRegex(GateContractError, 'missing original'):
            _ = account_workspace(self.raw[first:], 4, ROOT, self.inputs, Node({}))

    def test_duplicate_target(self) -> None:
        end = self.raw.index('     Running ', 1)
        with self.assertRaisesRegex(GateContractError, 'duplicate original'):
            _ = account_workspace(self.raw[:end]+self.raw, 4, ROOT, self.inputs, Node({}))

    def test_unknown_target(self) -> None:
        changed = self.raw.replace('src/lib.rs (target/debug/deps/sim_core-', 'src/foreign.rs (target/debug/deps/foreign-', 1)
        with self.assertRaisesRegex(GateContractError, 'unknown or ambiguous'):
            _ = account_workspace(changed, 4, ROOT, self.inputs, Node({}))

    def test_wrong_profile(self) -> None:
        with self.assertRaisesRegex(GateContractError, 'profile'):
            _ = account_workspace(self.raw, 5, ROOT, self.inputs, Node({}))

    def test_missing_docs(self) -> None:
        with self.assertRaisesRegex(GateContractError, 'missing original doctest'):
            _ = account_workspace(self.raw[:self.raw.index('   Doc-tests ')], 4, ROOT, self.inputs, Node({}))

    def test_nested_child_environment_rejected(self) -> None:
        with self.assertRaisesRegex(GateContractError, 'selects nested child'):
            _ = account_workspace(self.raw, 4, ROOT, self.inputs, Node({'WORLDSIM_CONTENT_TEST_CASE': 'one'}))

    def test_bound_nested_source_required(self) -> None:
        inputs = dict(self.inputs)
        inputs['rust/crates/sim-core/src/colony/content/tests.rs'] = '0'*64
        with self.assertRaisesRegex(GateContractError, 'unbound nested source'):
            _ = account_workspace(self.raw, 4, ROOT, inputs, Node({}))

    def test_ignored_outcomes_preserved(self) -> None:
        result = account_payload(payload(('ordinary', 'external'), ('external',)))
        self.assertEqual(result['ignored'], 1)
        self.assertIn({'name': 'external', 'status': 'ignored', 'mode': None}, result['cases'])

    def test_unapproved_duplicate_case(self) -> None:
        with self.assertRaises(GateContractError):
            _ = account_payload(payload(('duplicate', 'duplicate')))


if __name__ == '__main__':
    _ = unittest.main()
