"""Synthetic children only; these tests do not run any original gate check."""
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import final
from unittest.mock import Mock, patch

import generic_static_run as runner
from generic_fixture_run import ComponentResult
from generic_gate import GateContractError, document
from generic_process import file_hash
from generic_sources import git
from generic_sources import inventory as actual_inventory
from mux2_gate_inputs import Json


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
        self.refs_patch = patch.object(runner, '_refs', return_value={})
        _ = self.refs_patch.start()
        _ = patch.object(runner, 'inventory', side_effect=self.inventory).start()

    def inventory(self, root: Path, names: list[str]) -> dict[str, str]:
        return {} if root == self.root else actual_inventory(root, names)

    def run_one(self, step: int = 13, name: str = 'out') -> ComponentResult:
        return runner.run_step(self.product, step, self.base / name, self.context)


@final
class StaticTests(unittest.TestCase):
    def test_all_exact_commands_cwd_and_component_scope(self) -> None:
        fixture = Fixture(self)
        self.assertEqual(set(runner.COMMANDS), {1, 7, 13, 15, 16, 20, 22, 31, 33, 51})
        for step, (_, argv) in runner.COMMANDS.items():
            if step in {16, 51}:
                continue
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


@final
class Step16Tests(unittest.TestCase):
    def fixture(self, summary: str = 'Ran 7 tests in 0.01s\n\nOK (skipped=1, expected failures=2)', code: int = 0) -> Fixture:
        fixture = Fixture(self)
        _ = fixture.child.write_text("#!/bin/sh\ncat >&2 <<'END'\n" + summary + '\nEND\nexit ' + str(code) + '\n')
        names = ('python3', 'git', 'bash', 'grep', 'sh', 'sleep', 'dirname', 'mkdir', 'sed', 'date', 'touch', 'env', 'shasum', 'cmp', 'perl')
        paths = {name: (Path('/bin', name).resolve() if name in ('bash', 'sh', 'sleep') else
                       Path('/usr/bin/perl').resolve() if name == 'perl' else fixture.child) for name in names}
        def selected(name: str, *, path: str) -> str:
            _ = path
            return str(paths[name])
        _ = patch.object(shutil, 'which', side_effect=selected).start()
        fixture.context = runner.ExecutionEnvironment(
            (*fixture.context.values, ('HOME', str(fixture.base)), ('TMPDIR', str(fixture.base)),
             ('PYTHONDONTWRITEBYTECODE', '1'), ('PYTHONNOUSERSITE', '1'),
             ('GIT_CONFIG_GLOBAL', '/dev/null'), ('GIT_CONFIG_NOSYSTEM', '1')),
            tuple((name, path, file_hash(path)) for name, path in paths.items()), ())
        return fixture

    def test_exact_discovery_and_actual_counts_with_docs_inputs(self) -> None:
        fixture = self.fixture()
        seen: list[list[str]] = []
        def observe(root: Path, names: list[str]) -> dict[str, str]:
            if root == fixture.root:
                seen.append(names)
            return fixture.inventory(root, names)
        with patch.object(runner, 'inventory', side_effect=observe):
            result = fixture.run_one(16)
        self.assertEqual(result.status, 'PASS')
        child = document((result.receipt.parent / 'process/process.json').read_text())
        self.assertEqual(child.field('argv').value,
                         [str(fixture.child), '-m', 'unittest', 'discover', '-s', 'tools/v8', '-p', 'test_*.py'])
        self.assertNotIn('timeout_seconds', child.mapping())
        self.assertTrue(seen and all('docs' in names for names in seen))
        report = document(result.receipt.read_text()).field('unittest')
        self.assertEqual(document(result.receipt.read_text()).field('name').value, 'python_unittest')
        self.assertEqual(report.field('tests_run').value, 7)
        self.assertEqual(report.field('counts').field('skipped').value, 1)
        self.assertEqual(report.field('counts').field('expected failures').value, 2)
        runner.verify_receipt(result.receipt, file_hash(result.receipt))

    def test_native_failure_retains_actual_terminal_and_logs(self) -> None:
        fixture = self.fixture('Ran 3 tests in 0.02s\n\nFAILED (failures=1, errors=1)', 1)
        result = fixture.run_one(16)
        report = document(result.receipt.read_text())
        self.assertEqual(result.status, 'FAIL')
        self.assertEqual(report.field('unittest').field('tests_run').value, 3)
        self.assertEqual(report.field('native_exit').value, 1)
        self.assertIn('FAILED', (result.receipt.parent / 'process/stderr.log').read_text())
        self.assertIn('source_end', report.mapping())

    def test_each_required_tool_and_environment_policy_before_spawn(self) -> None:
        fixture = self.fixture()
        original = fixture.context
        for name in ('bash', 'grep', 'sh', 'sleep', 'dirname', 'mkdir', 'sed', 'date', 'touch', 'env', 'shasum', 'cmp', 'perl'):
            fixture.context = runner.ExecutionEnvironment(original.values,
                tuple(row for row in original.tools if row[0] != name), ())
            result = fixture.run_one(16, 'missing-' + name)
            self.assertEqual(result.status, 'FAIL')
            self.assertFalse((result.receipt.parent / 'process').exists())
        for name in ('HOME', 'TMPDIR', 'PYTHONDONTWRITEBYTECODE', 'PYTHONNOUSERSITE', 'GIT_CONFIG_GLOBAL', 'GIT_CONFIG_NOSYSTEM'):
            fixture.context = runner.ExecutionEnvironment(tuple(row for row in original.values if row[0] != name), original.tools, ())
            result = fixture.run_one(16, 'env-' + name)
            self.assertEqual(result.status, 'FAIL')
            self.assertFalse((result.receipt.parent / 'process').exists())
        fixture.context = runner.ExecutionEnvironment((*original.values, ('WORLDSIM_MUX2_LANE_FD', '999')), original.tools, ())
        self.assertEqual(fixture.run_one(16, 'lane').status, 'FAIL')

    def test_invalid_or_contradictory_terminal(self) -> None:
        for summary in ('OK', 'Ran 0 tests in 0.1s\n\nOK', 'Ran 2 tests in 0.1s\n\nOK\ntrailing',
                        'Ran 2 tests in 0.1s\n\nOK (errors=1)', 'Ran 2 tests in 0.1s\n\nFAILED',
                        'Ran 2 tests in 0.1s\n\nOK (skipped=3)',
                        'Ran 2 tests in 0.1s\n\nOK (skipped=1, skipped=1)',
                        'Ran 2 tests in 0.1s\n\nOK (unknown=1)',
                        'Ran 2 tests in 0.1s\n\nOK\nRan 2 tests in 0.1s\n\nOK'):
            with self.subTest(summary=summary), self.assertRaises(GateContractError):
                _ = runner.unittest_terminal(summary)

    def test_unexpected_success_is_failed_not_an_allowed_skip(self) -> None:
        row = runner.unittest_terminal('Ran 1 test in 0.001s\n\nFAILED (unexpected successes=1)')
        self.assertEqual(row['outcome'], 'FAILED')

    def test_real_source_refs_include_both_art_commits_without_import(self) -> None:
        fixture = self.fixture()
        fixture.refs_patch.stop()
        root = fixture.root
        _ = (root / 'tools/v8/art_import.json').write_text('{"source_commit":"' + 'f' * 40 + '"}')
        _ = (root / 'tools/v8/art_masks.py').write_text("raise RuntimeError('must not import')\nSOURCE_COMMIT = '" + 'c' * 40 + "'\n")
        calls: list[tuple[str, ...]] = []
        def resolve(path: Path, *args: str) -> bytes:
            self.assertEqual(path, root)
            calls.append(args)
            return args[-1].removesuffix('^{commit}').encode()
        with patch.object(runner, 'git', side_effect=resolve):
            result = fixture.run_one(16)
        self.assertEqual(result.status, 'PASS')
        refs = document(result.receipt.read_text()).field('refs_before').mapping()
        self.assertEqual(set(refs), {'f' * 40, 'c' * 40})
        self.assertEqual(calls, [('rev-parse', '--verify', 'f' * 40 + '^{commit}'),
                                 ('rev-parse', '--verify', 'c' * 40 + '^{commit}')] * 2)


@final
class DiagnosisTests(unittest.TestCase):
    def diagnosis_fixture(self, payload: str | None) -> Fixture:
        fixture = Fixture(self)
        path = fixture.root / runner.DIAGNOSIS_RESULT
        path.parent.mkdir(parents=True)
        if payload is not None:
            _ = path.write_text(payload)
        fixture.product = runner.StaticProduct(fixture.root, 'a' * 40,
            ((runner.DIAGNOSIS_RESULT, file_hash(path)),) if payload is not None else ())
        python = Path(sys.executable).resolve()
        tools = {'python3': python, 'bash': Path('/bin/bash').resolve(), 'git': Path('/usr/bin/git').resolve()}
        fixture.context = runner.ExecutionEnvironment(
            (('PATH', str(python.parent) + ':/usr/bin:/bin'), ('LANG', 'C'), ('LC_ALL', 'C'),
             ('CARGO_BUILD_JOBS', '8'), ('PYTHONNOUSERSITE', '1'), ('PYTHONDONTWRITEBYTECODE', '1')),
            tuple((name, tool, file_hash(tool)) for name, tool in tools.items()), ())
        def resolve_tool(name: str, path: str) -> str:
            _ = path
            return str(tools[name])
        _ = patch.object(shutil, 'which', side_effect=resolve_tool).start()
        def runtime_inventory(root: Path, names: list[str]) -> dict[str, str]:
            if root == fixture.root:
                self.assertIn(runner.DIAGNOSIS_RESULT, names)
                self.assertNotIn('docs', names)
                return {runner.DIAGNOSIS_RESULT: file_hash(path)} if path.exists() else {}
            return actual_inventory(root, names)
        _ = patch.object(runner, 'inventory', side_effect=runtime_inventory).start()
        return fixture

    def payload(self, passes: tuple[Json, ...] = (True,) * 6) -> str:
        return json.dumps({'questions': [{'id': 'Q' + str(i + 1), 'pass': value} for i, value in enumerate(passes)]})

    def test_exact_source_body(self) -> None:
        source = Path(git.__code__.co_filename).parent / 'gate.sh'
        body = source.read_text().split("step 51 read_diagnosis_test bash -c '", 1)[1].split("\n'", 1)[0] + '\n'
        self.assertEqual(runner.COMMANDS[51], ('read_diagnosis_test', ('bash', '-c', body)))

    def test_original_assertions_and_truthiness(self) -> None:
        for passes in ((True,) * 6, (True, True, True, True, True, False), ('truthy',) * 7):
            with self.subTest(passes=passes):
                fixture = self.diagnosis_fixture(self.payload(passes))
                result = fixture.run_one(51)
                self.assertEqual(result.status, 'PASS')
                self.assertIn('READ_DIAGNOSIS_TEST PASS', (result.receipt.parent / 'process/stdout.log').read_text())
                runner.verify_receipt(result.receipt, file_hash(result.receipt))
                patch.stopall()

    def test_failed_original_assertions_and_bad_data_retain_native_failures(self) -> None:
        cases = [self.payload((True, True, False, False, True, False)),
                 self.payload((False, True, True, True, True, True)),
                 self.payload((True, True, True, True, False, True)),
                 '{bad json', '{"questions": []}', '{"questions": [{"id": "Q1", "pass": true}]}', None]
        for payload in cases:
            with self.subTest(payload=payload):
                fixture = self.diagnosis_fixture(payload)
                result = fixture.run_one(51)
                self.assertEqual(result.status, 'FAIL')
                child = document((result.receipt.parent / 'process/process.json').read_text())
                self.assertNotEqual(child.field('native_exit').value, 0)
                self.assertTrue((result.receipt.parent / 'process/stderr.log').is_file())
                patch.stopall()

    def test_original_first_q1_match_controls_assertion(self) -> None:
        for first in (True, False):
            with self.subTest(first=first):
                rows: list[Json] = [{'id': 'Q1', 'pass': first}, {'id': 'Q1', 'pass': not first}]
                rows.extend({'id': 'Q' + str(i), 'pass': True} for i in range(2, 7))
                fixture = self.diagnosis_fixture(json.dumps({'questions': rows}))
                self.assertEqual(fixture.run_one(51).status, 'PASS' if first else 'FAIL')
                patch.stopall()

    def test_startup_overrides_and_missing_python_reject_before_launch(self) -> None:
        for override in ('PYTHONOPTIMIZE', 'PYTHONPATH', 'PYTHONHOME', 'BASH_ENV', 'ENV', 'SHELLOPTS', 'BASHOPTS',
                         'BASH_FUNC_python3%%', 'BASH_FUNC_python3()', 'LD_PRELOAD', 'DYLD_INSERT_LIBRARIES', 'missing-python'):
            with self.subTest(override=override):
                fixture = self.diagnosis_fixture(self.payload())
                values = fixture.context.values if override == 'missing-python' else (*fixture.context.values, (override, '1'))
                tools = tuple(row for row in fixture.context.tools if override != 'missing-python' or row[0] != 'python3')
                fixture.context = runner.ExecutionEnvironment(values, tools, ())
                result = fixture.run_one(51)
                self.assertEqual(result.status, 'FAIL')
                self.assertFalse((result.receipt.parent / 'process').exists())
                patch.stopall()

    def test_actual_bash_startup_assertion_bypass_is_rejected_before_capture(self) -> None:
        fixture = self.diagnosis_fixture(self.payload((False,) * 6))
        startup = fixture.base / 'startup.sh'
        _ = startup.write_text('export PYTHONOPTIMIZE=1\n')
        values = (*fixture.context.values, ('BASH_ENV', str(startup)))
        # Demonstrate why a marker/native0 join alone is insufficient: the original
        # shell command bypasses every Python assertion after this startup script.
        raw = subprocess.run(['/bin/bash', *runner.COMMANDS[51][1][1:]], cwd=fixture.root,
                             env=dict(values), capture_output=True, text=True, check=False)
        self.assertEqual(raw.returncode, 0, raw.stderr)
        self.assertIn('30s test: 0/6', raw.stdout)
        self.assertIn('READ_DIAGNOSIS_TEST PASS', raw.stdout)
        fixture.context = runner.ExecutionEnvironment(values, fixture.context.tools, ((startup,file_hash(startup)),))
        result = fixture.run_one(51)
        self.assertEqual(result.status, 'FAIL')
        self.assertFalse((result.receipt.parent / 'process').exists())

    def test_missing_data_inventory_and_postflight_data_mutation_fail(self) -> None:
        fixture = self.diagnosis_fixture(self.payload())
        fixture.product = runner.StaticProduct(fixture.root, 'a' * 40, ())
        result = fixture.run_one(51, 'missing-data')
        self.assertEqual(result.status, 'FAIL')
        self.assertFalse((result.receipt.parent / 'process').exists())
        path = fixture.root / runner.DIAGNOSIS_RESULT
        fixture.product = runner.StaticProduct(fixture.root, 'a' * 40, ((runner.DIAGNOSIS_RESULT, file_hash(path)),))
        def mutate(root: Path, sha: str) -> dict[str, str]:
            _ = root, sha
            if (fixture.base / 'mutated/process/process.json').exists():
                _ = path.write_text(self.payload((False,) * 6))
            return {'source': 'synthetic'}
        fixture.source.side_effect = mutate
        result = fixture.run_one(51, 'mutated')
        self.assertEqual(result.status, 'FAIL')
        self.assertEqual(document((result.receipt.parent / 'process/process.json').read_text()).field('native_exit').value, 0)

    def test_terminal_marker_must_be_unique_last_and_joined_to_zero_exit(self) -> None:
        bodies = ('echo READ_DIAGNOSIS_TEST PASS; echo READ_DIAGNOSIS_TEST PASS',
                  'echo READ_DIAGNOSIS_TEST PASS; echo unexpected-tail',
                  'echo READ_DIAGNOSIS_TEST PASS; exit 9')
        for body in bodies:
            with self.subTest(body=body):
                fixture = Fixture(self)
                _ = fixture.child.write_text('#!/bin/sh\n' + body + '\n')
                fixture.context = runner.ExecutionEnvironment(
                    (*fixture.context.values, ('PYTHONNOUSERSITE', '1'), ('PYTHONDONTWRITEBYTECODE', '1')),
                    tuple((name, path, file_hash(path)) for name, path, _ in fixture.context.tools), ())
                self.assertEqual(fixture.run_one(51).status, 'FAIL')
                patch.stopall()

    def test_zero_exit_without_original_terminal_rejects(self) -> None:
        fixture = Fixture(self)
        fixture.context = runner.ExecutionEnvironment((*fixture.context.values, ('PYTHONNOUSERSITE', '1'),
                                                      ('PYTHONDONTWRITEBYTECODE', '1')), fixture.context.tools, ())
        self.assertEqual(fixture.run_one(51).status, 'FAIL')


if __name__ == '__main__':
    _ = unittest.main()
