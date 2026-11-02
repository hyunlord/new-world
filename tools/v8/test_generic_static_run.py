"""Synthetic children only; these tests do not run any original gate check."""
import shutil
import tempfile
import unittest
from pathlib import Path
from typing import final
from unittest.mock import Mock, patch

import generic_static_run as runner
from generic_fixture_run import ComponentResult
from generic_gate import GateContractError, document
from generic_process import file_hash
from generic_sources import inventory as actual_inventory


@final
class Fixture:
    def __init__(self, test: unittest.TestCase) -> None:
        self.temp = tempfile.TemporaryDirectory()
        test.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name).resolve()
        self.root = self.base / 'root'
        (self.root / 'tools/v8').mkdir(parents=True)
        _ = (self.root / 'tools/v8/gate.sh').write_text('synthetic command source')
        self.child = self.base / 'tiny-child'
        _ = self.child.write_text('#!/bin/sh\necho synthetic-only\nexit 0\n')
        self.child.chmod(0o755)
        names = ('bash', 'python3', 'git', 'cargo', 'cargo-fmt', 'rustfmt', 'grep')
        self.context = runner.ExecutionEnvironment(
            (('PATH', '/usr/bin:/bin'), ('LANG', 'C'), ('LC_ALL', 'C'), ('CARGO_BUILD_JOBS', '8')),
            tuple((name, self.child, file_hash(self.child)) for name in names), ())
        self.product = runner.StaticProduct(self.root, 'a' * 40, ())
        test.addCleanup(patch.stopall)
        _ = patch.object(runner, 'GATE_SHA256', file_hash(self.root / 'tools/v8/gate.sh')).start()
        _ = patch.object(shutil, 'which', return_value=str(self.child)).start()
        self.source = Mock(return_value={'source': 'synthetic'})
        _ = patch.object(runner, 'freeze_source', new=self.source).start()
        _ = patch.object(runner, '_refs', return_value={}).start()
        _ = patch.object(runner, 'inventory', side_effect=self.inventory).start()

    def inventory(self, root: Path, names: list[str]) -> dict[str, str]:
        return {} if root == self.root else actual_inventory(root, names)

    def run_one(self, step: int = 13, name: str = 'out') -> ComponentResult:
        return runner.run_step(self.product, step, self.base / name, self.context)


@final
class StaticTests(unittest.TestCase):
    def test_all_exact_commands_cwd_and_component_scope(self) -> None:
        fixture = Fixture(self)
        self.assertEqual(set(runner.COMMANDS), {1, 7, 13, 15, 20, 22, 31, 33})
        for step, (_, argv) in runner.COMMANDS.items():
            with self.subTest(step=step):
                result = fixture.run_one(step, str(step))
                self.assertEqual(result.status, 'PASS')
                receipt = document(result.receipt.read_text())
                child = document((result.receipt.parent / 'process/process.json').read_text())
                self.assertEqual(child.field('argv').value, [str(fixture.child), *argv[1:]])
                self.assertEqual(child.field('cwd').value, str(fixture.root))
                self.assertEqual(receipt.field('source_invocation').value, list(argv))
                self.assertFalse(receipt.field('full_gate_complete').value)
                self.assertFalse(receipt.field('original_order_executed').value)
                runner.verify_receipt(result.receipt, file_hash(result.receipt))

    def test_unsupported_and_stale_before_launch(self) -> None:
        fixture = Fixture(self)
        with self.assertRaises(GateContractError):
            _ = fixture.run_one(6)
        self.assertFalse((fixture.base / 'out').exists())
        (fixture.base / 'out').mkdir()
        with self.assertRaises(GateContractError):
            _ = fixture.run_one()

    def test_nonzero_retains_logs(self) -> None:
        fixture = Fixture(self)
        _ = fixture.child.write_text('#!/bin/sh\necho actual-failure\nexit 9\n')
        fixture.context = runner.ExecutionEnvironment(fixture.context.values,
            tuple((name, path, file_hash(path)) for name, path, _ in fixture.context.tools), ())
        result = fixture.run_one()
        self.assertEqual(result.status, 'FAIL')
        self.assertIn('actual-failure', (result.receipt.parent / 'process/stdout.log').read_text())

    def test_missing_explicit_environment(self) -> None:
        fixture = Fixture(self)
        fixture.context = runner.ExecutionEnvironment((), fixture.context.tools, ())
        result = fixture.run_one()
        self.assertEqual(result.status, 'FAIL')
        self.assertFalse((result.receipt.parent / 'process').exists())

    def test_tool_drift_before_launch(self) -> None:
        fixture = Fixture(self)
        _ = fixture.child.write_text('changed')
        result = fixture.run_one()
        self.assertEqual(result.status, 'FAIL')
        self.assertFalse((result.receipt.parent / 'process').exists())

    def test_source_mutation_terminal(self) -> None:
        fixture = Fixture(self)
        fixture.source.side_effect = [{'source': 'synthetic'}, {'source': 'changed'}]
        self.assertEqual(fixture.run_one().status, 'FAIL')

    def test_late_raw_mutation_during_terminal_source_check(self) -> None:
        fixture = Fixture(self)
        def mutate(root: Path, sha: str) -> dict[str, str]:
            _ = root, sha
            log = fixture.base / 'out/process/stdout.log'
            if log.exists():
                _ = log.write_text('tampered')
            return {'source': 'synthetic'}
        fixture.source.side_effect = mutate
        self.assertEqual(fixture.run_one().status, 'FAIL')

    def test_prior_success_mutated_by_later_step(self) -> None:
        fixture = Fixture(self)
        first = fixture.run_one(name='first')
        fixture.context = runner.ExecutionEnvironment(fixture.context.values, fixture.context.tools, (),
                                                   ((first.receipt, file_hash(first.receipt)),))
        def mutate(root: Path, sha: str) -> dict[str, str]:
            _ = root, sha
            _ = (first.receipt.parent / 'process/stdout.log').write_text('later mutation')
            return {'source': 'synthetic'}
        fixture.source.side_effect = mutate
        self.assertEqual(fixture.run_one(name='second').status, 'FAIL')

    def test_ignored_input_addition_fails_before_child(self) -> None:
        fixture = Fixture(self)
        def changed(root: Path, names: list[str]) -> dict[str, str]:
            return {'assets/extra': '0' * 64} if root == fixture.root else actual_inventory(root, names)
        with patch.object(runner, 'inventory', side_effect=changed):
            result = fixture.run_one()
        self.assertEqual(result.status, 'FAIL')
        self.assertFalse((result.receipt.parent / 'process').exists())

    def test_ref_change_after_child_retains_failure(self) -> None:
        fixture = Fixture(self)
        with patch.object(runner, '_refs', side_effect=[{'v7-final-art': 'a'}, {'v7-final-art': 'b'}]):
            self.assertEqual(fixture.run_one(15).status, 'FAIL')

    def test_prior_failure_is_protected_at_final_boundary(self) -> None:
        fixture = Fixture(self)
        fixture.source.side_effect = [{'source': 'synthetic'}, {'source': 'changed'}]
        first = fixture.run_one(name='first')
        self.assertEqual(first.status, 'FAIL')
        fixture.context = runner.ExecutionEnvironment(fixture.context.values, fixture.context.tools, (),
                                                   ((first.receipt, file_hash(first.receipt)),))
        calls = 0
        def late_mutate(root: Path, sha: str) -> dict[str, str]:
            nonlocal calls
            _ = root, sha
            calls += 1
            if calls == 2:
                _ = (first.receipt.parent / 'process/stderr.log').write_text('late mutation')
            return {'source': 'synthetic'}
        fixture.source.side_effect = late_mutate
        self.assertEqual(fixture.run_one(name='second').status, 'FAIL')

    def test_changed_path_resolution_is_not_an_authorized_tool(self) -> None:
        fixture = Fixture(self)
        with patch.object(shutil, 'which', return_value='/bin/sh'):
            result = fixture.run_one()
        self.assertEqual(result.status, 'FAIL')
        self.assertFalse((result.receipt.parent / 'process').exists())

    def test_added_output_rejected(self) -> None:
        fixture = Fixture(self)
        result = fixture.run_one()
        _ = (result.receipt.parent / 'extra').write_text('unbound')
        with self.assertRaises(GateContractError):
            runner.verify_receipt(result.receipt, file_hash(result.receipt))


if __name__ == '__main__':
    _ = unittest.main()
