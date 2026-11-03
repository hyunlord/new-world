"""Synthetic children only; these tests do not run any original gate check."""
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Literal, final
from unittest.mock import Mock, patch

import bridge_api
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
        _ = (self.root / 'project.godot').write_text('synthetic project')
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
    def test_current_gate_source_pin(self) -> None:
        self.assertEqual(runner.GATE_SHA256,file_hash(Path(runner.__file__).with_name('gate.sh')))

    def test_all_exact_commands_cwd_and_component_scope(self) -> None:
        fixture = Fixture(self)
        self.assertEqual(set(runner.COMMANDS), {1, 7, 8, 13, 14, 15, 16, 20, 22, 31, 33, 46, 51})
        for step, (_, argv) in runner.COMMANDS.items():
            if step in {8, 14, 16, 46, 51}:
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


class StrictTests(unittest.TestCase):
    def test_exact_closed_commands(self) -> None:
        self.assertIn(8, runner.COMMANDS)
        self.assertEqual(runner.COMMANDS[8][1], ('bash', 'tools/v8/gdscript_strict_check.sh'))
        self.assertEqual(runner.COMMANDS[46][1], ('bash', 'tools/v8/gdscript_strict_check.sh',
            'tools/v8/mux21_layout_test.gd', 'tools/v8/mux21_toast_test.gd',
            'tools/v8/mux21_play_capture.gd', 'tools/v8/mux21_why_locale_test.gd'))

    def test_terminal_requires_count_ffi_and_final_marker(self) -> None:
        good = '[gdcheck] parse + warnings: 4 file(s) clean ✓\n[gdcheck] FFI bindings: 2 GDScript call(s) all match Rust #[func] exports ✓\n[gdcheck] ALL CHECKS PASS\n'
        runner.strict_terminal(good, '', 4)
        for text in (good.replace('4 file(s)', '3 file(s)'), good + 'other\n',
                     good.replace('[gdcheck] ALL CHECKS PASS', ''),
                     good.replace('[gdcheck] FFI bindings:', '[gdcheck] WARN:'),
                     '[gdcheck] no .gd files to check\n'):
            with self.subTest(text=text), self.assertRaises(ValueError):
                runner.strict_terminal(text, '', 4)
        with self.assertRaises(ValueError):
            runner.strict_terminal(good, '[gdcheck] FAIL parse/warning: a.gd', 4)

    def test_original_shell_full_selection_masked_exit_and_restoration(self) -> None:
        import os
        source = Path(bridge_api.__file__).resolve().parent
        for output, code, expected in (('', 7, 0), ('SCRIPT ERROR synthetic', 0, 2)):
            with self.subTest(output=output), tempfile.TemporaryDirectory() as temp:
                root = Path(temp).resolve()
                (root/'tools/v8').mkdir(parents=True)
                (root/'scripts/test').mkdir(parents=True)
                _ = (root/'scripts/a.gd').write_text('extends Node\n')
                _ = (root/'scripts/b.gd').write_text('extends Node\n')
                _ = (root/'scripts/test/skip.gd').write_text('extends Node\n')
                (root/'rust/crates/sim-bridge/src').mkdir(parents=True)
                _ = (root/'rust/crates/sim-bridge/src/lib.rs').write_text('')
                for name in ('gdscript_strict_check.sh', 'godot_bin.sh', 'bridge_api.py'):
                    _ = shutil.copyfile(source/name, root/'tools/v8'/name)
                project=root/'project.godot';_ = project.write_text('config_version=5\n')
                original=project.read_bytes()
                fake=root/'fake-godot'
                _ = fake.write_text('#!/bin/sh\necho "$*" >> "'+str(root/'calls')+'"\necho "'+output+'"\nexit '+str(code)+'\n')
                fake.chmod(0o755)
                result=subprocess.run(('/bin/bash', 'tools/v8/gdscript_strict_check.sh'), cwd=root,
                    env={'PATH':str(Path(sys.executable).parent)+':/usr/bin:/bin','GODOT_BIN':str(fake),
                         'HOME':str(root),'TMPDIR':str(root),'LANG':'C','LC_ALL':'C','PYTHONDONTWRITEBYTECODE':'1'},
                    capture_output=True, check=False)
                self.assertEqual(result.returncode,expected, result.stderr.decode())
                self.assertEqual(project.read_bytes(),original)
                calls=(root/'calls').read_text().splitlines()
                self.assertEqual(len(calls),2)
                self.assertTrue(all('--headless --check-only --script scripts/' in row for row in calls))
                self.assertFalse(any('skip.gd' in row for row in calls))
                self.assertEqual(runner.strict_files(root,8),('scripts/a.gd','scripts/b.gd'))
                if expected==0:runner.strict_terminal(result.stdout.decode(),result.stderr.decode(),2)
                self.assertEqual(os.stat(project).st_mode & 0o777, 0o644)

    def test_runtime_scope_reuses_resolver_without_overlapping_input_scopes(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp).resolve();(root/'.godot/imported').mkdir(parents=True)
            with patch.object(runner,'resolution_paths',return_value={'scripts/a.gd','.godot/imported/a.ctex','extra.gd'}):
                names=runner.strict_input_names(root)
            self.assertIn('extra.gd',names)
            self.assertNotIn('scripts/a.gd',names)
            self.assertNotIn('.godot/imported/a.ctex',names)
            self.assertIn('rust/target/debug/libsim_bridge.dylib',names)

    def test_unknown_shell_ownership_is_not_returned_as_ordinary_failure(self) -> None:
        from generic_process import ProcessResult, ProcessSpec
        fixture=Fixture(self)
        def unknown(spec: ProcessSpec) -> ProcessResult:
            spec.output_root.mkdir()
            path=spec.output_root/'process.json';_ = path.write_text('{}')
            return ProcessResult(path,'FAIL',-9,123)
        with patch.object(runner,'_tools',return_value={'bash':fixture.child}), \
             patch.object(runner,'strict_files',return_value=('scripts/a.gd',)), \
             patch.object(runner,'strict_input_names',return_value=['scripts']), \
             patch.object(runner,'execute',side_effect=unknown), \
             self.assertRaises(runner.StrictOwnershipUnknown):
            _ = fixture.run_one(8)
        receipt=document((fixture.base/'out/receipt.json').read_text())
        self.assertEqual(receipt.field('ownership').text(),'INCOMPLETE_DESCENDANTS_UNKNOWN')
        self.assertEqual(receipt.field('native_exit').value,-9)
        self.assertEqual(receipt.field('artifacts').value,{})

    def test_strict_environment_requires_explicit_canonical_lane_and_godot(self) -> None:
        from dataclasses import replace
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp).resolve()
            fake=root/'godot';_ = fake.write_text('#!/bin/sh\nexit 0\n');fake.chmod(0o755)
            path=str(Path(sys.executable).parent)+':/usr/bin:/bin'
            names=('git','bash','python3','find','mktemp','rm','cp','cat','wc','tr','grep','head','sed','sort')
            tools: list[tuple[str,Path,str]]=[]
            for name in names:
                found=shutil.which(name,path=path)
                self.assertIsNotNone(found)
                if found is None:raise AssertionError(name)
                binary=Path(found).resolve();tools.append((name,binary,file_hash(binary)))
            values=(('PATH',path),('LANG','C'),('LC_ALL','C'),('CARGO_BUILD_JOBS','2'),
                    ('HOME',str(root)),('TMPDIR',str(root)),('GODOT_BIN',str(fake)),
                    ('PYTHONNOUSERSITE','1'),('PYTHONDONTWRITEBYTECODE','1'))
            context=runner.ExecutionEnvironment(values,tuple(tools),((fake,file_hash(fake)),))
            lock=root/'lane'
            with lock.open('w') as held, patch.object(runner,'LOCK',lock):
                with self.assertRaises(ValueError):_ = runner.validate_environment(context,8)
                context=replace(context,held_lane_fd=held.fileno())
                self.assertEqual(runner.validate_environment(context,8)['bash'],{n:p for n,p,_ in tools}['bash'])
                for key in ('BASH_ENV','PYTHONPATH','LD_PRELOAD','WORLDSIM_MUX2_LANE_FD'):
                    with self.subTest(key=key),self.assertRaises(ValueError):
                        _ = runner.validate_environment(replace(context,values=(*values,(key,'foreign'))),46)
                with self.assertRaises(ValueError):_ = runner.validate_environment(replace(context,external_inputs=()),8)
                with (root/'foreign').open('w') as other,self.assertRaises(ValueError):
                    _ = runner.validate_environment(replace(context,held_lane_fd=other.fileno()),8)
                self.assertFalse(held.closed)

    def test_step46_selection_remains_exact_four_files(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp).resolve()
            for name in runner.STRICT_FILES:
                path=root/name;path.parent.mkdir(parents=True,exist_ok=True);_ = path.write_text('extends Node\n')
            self.assertEqual(runner.strict_files(root,46),runner.STRICT_FILES)

    def test_strict_adapter_retains_original_terminal_and_endpoint_checks(self) -> None:
        fixture=Fixture(self)
        _ = fixture.child.write_text('#!/bin/sh\necho "[gdcheck] parse + warnings: 1 file(s) clean ✓"\necho "[gdcheck] FFI bindings: 0 GDScript call(s) all match Rust #[func] exports ✓"\necho "[gdcheck] ALL CHECKS PASS"\n')
        fixture.context=runner.ExecutionEnvironment(fixture.context.values,
            tuple((name,path,file_hash(path)) for name,path,_ in fixture.context.tools),())
        with patch.object(runner,'_tools',return_value={'bash':fixture.child}), \
             patch.object(runner,'strict_files',return_value=('scripts/a.gd',)), \
             patch.object(runner,'strict_input_names',return_value=['scripts']):
            result=fixture.run_one(8)
        self.assertEqual(result.status,'PASS')
        runner.verify_receipt(result.receipt,file_hash(result.receipt))
        self.assertEqual(document(result.receipt.read_text()).field('source_invocation').value,
                         ['bash','tools/v8/gdscript_strict_check.sh'])

    def test_strict_unreturned_execution_cannot_freeze_unknown_descendants(self) -> None:
        from generic_process import ProcessSpec
        fixture=Fixture(self)
        def unknown(spec: ProcessSpec) -> None:
            spec.output_root.mkdir()
            _ = (spec.output_root/'process-spawn.json').write_text('{"pid":123,"spawned_at":"2026-11-03T00:00:00+00:00"}')
            raise OSError('injected terminal write failure after spawn')
        with patch.object(runner,'_tools',return_value={'bash':fixture.child}), \
             patch.object(runner,'strict_files',return_value=('scripts/a.gd',)), \
             patch.object(runner,'strict_input_names',return_value=['scripts']), \
             patch.object(runner,'execute',side_effect=unknown), \
             self.assertRaises(runner.StrictOwnershipUnknown):
            _ = fixture.run_one(8)
        receipt=document((fixture.base/'out/receipt.json').read_text())
        self.assertEqual(receipt.field('ownership').text(),'INCOMPLETE_DESCENDANTS_UNKNOWN')
        self.assertNotIn('source_end',receipt.mapping())
        self.assertEqual(receipt.field('artifacts').value,{})

    def test_strict_negative_exit_missing_receipt_stays_unknown(self) -> None:
        from generic_process import ProcessResult, ProcessSpec
        fixture=Fixture(self)
        def unknown(spec: ProcessSpec) -> ProcessResult:
            spec.output_root.mkdir()
            return ProcessResult(spec.output_root/'process.json','FAIL',-9,123)
        with patch.object(runner,'_tools',return_value={'bash':fixture.child}), \
             patch.object(runner,'strict_files',return_value=('scripts/a.gd',)), \
             patch.object(runner,'strict_input_names',return_value=['scripts']), \
             patch.object(runner,'execute',side_effect=unknown), \
             self.assertRaises(runner.StrictOwnershipUnknown):
            _ = fixture.run_one(8)
        receipt=document((fixture.base/'out/receipt.json').read_text())
        self.assertEqual(receipt.field('ownership').text(),'INCOMPLETE_DESCENDANTS_UNKNOWN')
        self.assertEqual(receipt.field('native_exit').value,-9)
        self.assertNotIn('source_end',receipt.mapping())

    def test_strict_project_full_mode_change_is_rejected_not_repaired(self) -> None:
        fixture=Fixture(self)
        project=fixture.root/'project.godot';_ = project.write_text('config_version=5\n');project.chmod(0o644)
        _ = fixture.child.write_text('#!/bin/sh\nchmod 600 project.godot\necho "[gdcheck] parse + warnings: 1 file(s) clean ✓"\necho "[gdcheck] FFI bindings: 0 GDScript call(s) all match Rust #[func] exports ✓"\necho "[gdcheck] ALL CHECKS PASS"\n')
        fixture.context=runner.ExecutionEnvironment(fixture.context.values,
            tuple((name,path,file_hash(path)) for name,path,_ in fixture.context.tools),())
        with patch.object(runner,'_tools',return_value={'bash':fixture.child}), \
             patch.object(runner,'strict_files',return_value=('scripts/a.gd',)), \
             patch.object(runner,'strict_input_names',return_value=['scripts']):
            result=fixture.run_one(8)
        self.assertEqual(result.status,'FAIL')
        self.assertIn('project mode',result.receipt.read_text())
        self.assertEqual(project.stat().st_mode & 0o777,0o600)
        self.assertEqual(project.read_text(),'config_version=5\n')

    def test_strict_unknown_diagnostic_failure_cannot_downgrade_ownership(self) -> None:
        fixture=Fixture(self)
        with patch.object(runner,'_tools',return_value={'bash':fixture.child}), \
             patch.object(runner,'strict_files',return_value=('scripts/a.gd',)), \
             patch.object(runner,'strict_input_names',return_value=['scripts']), \
             patch.object(runner,'execute',side_effect=OSError('execution unknown')), \
             patch.object(runner,'atomic_json',side_effect=OSError('diagnostic unavailable')), \
             self.assertRaisesRegex(runner.StrictOwnershipUnknown,'diagnostic write failed'):
            _ = fixture.run_one(8)
        self.assertFalse((fixture.base/'out/receipt.json').exists())
        self.assertEqual(fixture.source.call_count,1)

    def test_strict_all_execution_exceptions_retain_unknown(self) -> None:
        fixture=Fixture(self)
        for index, error in enumerate((TypeError('after spawn'), AssertionError('after spawn'),
                                       KeyboardInterrupt(), SystemExit(2))):
            with self.subTest(error=type(error).__name__), \
                 patch.object(runner,'_tools',return_value={'bash':fixture.child}), \
                 patch.object(runner,'strict_files',return_value=('scripts/a.gd',)), \
                 patch.object(runner,'strict_input_names',return_value=['scripts']), \
                 patch.object(runner,'execute',side_effect=error), \
                 self.assertRaises(runner.StrictOwnershipUnknown):
                _ = fixture.run_one(8,f'exception-{index}')

    def test_strict_all_diagnostic_exceptions_retain_unknown(self) -> None:
        fixture=Fixture(self)
        index=0
        for target in ('atomic_json','timestamp'):
            for error in (TypeError('diagnostic'), RuntimeError('diagnostic'), KeyboardInterrupt(), SystemExit(2)):
                effects = ['2026-11-03T00:00:00+00:00',error] if target == 'timestamp' else error
                with self.subTest(target=target,error=type(error).__name__), \
                     patch.object(runner,'_tools',return_value={'bash':fixture.child}), \
                     patch.object(runner,'strict_files',return_value=('scripts/a.gd',)), \
                     patch.object(runner,'strict_input_names',return_value=['scripts']), \
                     patch.object(runner,'execute',side_effect=TypeError('execution unknown')), \
                     patch.object(runner,target,side_effect=effects), \
                     self.assertRaises(runner.StrictOwnershipUnknown):
                    _ = fixture.run_one(8,f'diagnostic-{index}')
                index+=1


class DocsTests(unittest.TestCase):
    def test_original_command_includes_system_table_without_extra_flags(self) -> None:
        self.assertEqual(runner.COMMANDS.get(14), ('docs', ('bash', 'tools/v8/check_docs.sh')))

    def test_docs_terminal_is_original_unique_final_marker(self) -> None:
        runner.docs_terminal('ok: system table PASS\ndocs: PASS\n')
        for raw in ('', 'docs: FAIL\n', 'docs: PASS\ntrailing\n', 'docs: PASS\ndocs: PASS\n',
                    'FAIL: system table\ndocs: PASS\n'):
            with self.subTest(raw=raw), self.assertRaises(ValueError):runner.docs_terminal(raw)

    def test_docs_scope_requires_exact_skill_links_and_complete_targets(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp).resolve()
            for name in runner.DOCS_SKILLS:
                target=root/'.claude/skills'/name;target.mkdir(parents=True)
                _ = (target/'SKILL.md').write_text('synthetic skill')
                link=root/'.agents/skills'/name;link.parent.mkdir(parents=True,exist_ok=True)
                link.symlink_to('../../.claude/skills/'+name)
            context=runner.ExecutionEnvironment((('CARGO_HOME',str(root/'cargo-home')),),(),())
            names=runner.docs_input_names(root,context)
            self.assertIn('docs',names)
            self.assertIn('.claude/skills',names)
            self.assertIn('rust/Cargo.lock',names)
            self.assertNotIn('.agents',names)
            link=root/'.agents/skills'/runner.DOCS_SKILLS[0]
            link.unlink();link.symlink_to('../../.claude/skills/'+runner.DOCS_SKILLS[1])
            with self.assertRaises(ValueError):_ = runner.docs_input_names(root,context)
            link.unlink();link.symlink_to('../../.claude/skills/'+runner.DOCS_SKILLS[0])
            (root/'.claude/skills'/runner.DOCS_SKILLS[0]/'SKILL.md').unlink()
            with self.assertRaises(OSError):_ = runner.docs_input_names(root,context)

    @staticmethod
    def docs_fixture(test: unittest.TestCase, body: str='echo "docs: PASS"') -> Fixture:
        fixture=Fixture(test)
        _ = fixture.child.write_text('#!/bin/sh\n'+body+'\n')
        fixture.context=runner.ExecutionEnvironment(fixture.context.values,
            tuple((name,path,file_hash(path)) for name,path,_ in fixture.context.tools),())
        _ = patch.object(runner,'_tools',return_value={'bash':fixture.child}).start()
        _ = patch.object(runner,'docs_input_names',return_value=['docs']).start()
        _ = patch.object(runner,'docs_bindings',return_value={'synthetic':True}).start()
        return fixture

    def test_docs_actual_tiny_process_retains_native_and_classification_limits(self) -> None:
        fixture=self.docs_fixture(self)
        result=fixture.run_one(14)
        self.assertEqual(result.status,'PASS')
        receipt=document(result.receipt.read_text())
        self.assertEqual(receipt.field('source_invocation').value,['bash','tools/v8/check_docs.sh'])
        self.assertEqual(receipt.field('reviewed_world_scope').text(),'construction_no_tick_advance')
        self.assertTrue(receipt.field('reviewed_initial_state_mutations').value)
        self.assertEqual(receipt.field('current_product_world_scope').text(),'CALLER_REQUIRED')
        self.assertEqual(receipt.field('build_provenance').text(),'CALLER_REQUIRED')
        self.assertFalse(receipt.field('full_gate_complete').value)
        runner.verify_receipt(result.receipt,file_hash(result.receipt))

    def test_docs_misleading_marker_or_nonzero_never_pass(self) -> None:
        for index,body in enumerate(('echo "docs: PASS"; exit 7','echo unrelated','echo "FAIL: source"; echo "docs: PASS"')):
            with self.subTest(body=body):
                fixture=self.docs_fixture(self,body)
                result=fixture.run_one(14,str(index))
                self.assertEqual(result.status,'FAIL')
                self.assertTrue((result.receipt.parent/'process/stdout.log').is_file())
                patch.stopall()

    def test_docs_unreturned_or_negative_parent_retains_unknown(self) -> None:
        from generic_process import ProcessResult, ProcessSpec
        for index,code in enumerate((-9,None)):
            fixture=self.docs_fixture(self)
            def unknown(spec: ProcessSpec, native_exit: int | None=code) -> ProcessResult:
                spec.output_root.mkdir()
                return ProcessResult(spec.output_root/'process.json','FAIL',native_exit,123)
            with patch.object(runner,'execute',side_effect=unknown),self.assertRaises(runner.StrictOwnershipUnknown):
                _ = fixture.run_one(14,str(index))
            receipt=document((fixture.base/str(index)/'receipt.json').read_text())
            self.assertEqual(receipt.field('artifacts').value,{})
            self.assertNotIn('source_end',receipt.mapping())
            patch.stopall()
        fixture=self.docs_fixture(self)
        with patch.object(runner,'execute',side_effect=KeyboardInterrupt()),self.assertRaises(runner.StrictOwnershipUnknown):
            _ = fixture.run_one(14)
        self.assertEqual(document((fixture.base/'out/receipt.json').read_text()).field('process_status').text(),'UNKNOWN')


class DocsBindingTests(unittest.TestCase):
    def test_all_original_shell_tools_and_caller_lane_are_required(self) -> None:
        from dataclasses import replace
        fixture=Fixture(self)
        expected={'python3','cargo','rustc','dirname','grep','wc','tr','cat','tail','rm',
                  'head','readlink','awk','sed','xargs','stat','ls'}
        self.assertEqual(runner.DOCS_TOOLS,expected)
        names={'git','bash',*expected}
        values=(*fixture.context.values,('HOME',str(fixture.base)),('TMPDIR',str(fixture.base)),
                ('CARGO_HOME',str(fixture.base)),('RUSTUP_HOME',str(fixture.base)),
                ('PYTHONNOUSERSITE','1'),('PYTHONDONTWRITEBYTECODE','1'))
        context=runner.ExecutionEnvironment(values,tuple((name,fixture.child,file_hash(fixture.child)) for name in names),())
        lock=fixture.base/'lane'
        with lock.open('w') as held,patch.object(runner,'LOCK',lock):
            with self.assertRaises(ValueError):_ = runner.validate_environment(context,14)
            context=replace(context,held_lane_fd=held.fileno())
            self.assertEqual(runner.validate_environment(context,14)['bash'],fixture.child)
            for missing in names:
                with self.subTest(missing=missing),self.assertRaises(ValueError):
                    _ = runner.validate_environment(replace(context,tools=tuple(row for row in context.tools if row[0]!=missing)),14)
            for key in ('BASH_ENV','PYTHONOPTIMIZE','RUSTC_WRAPPER','RUSTDOC','CARGO_ENCODED_RUSTFLAGS','WORLDSIM_GATE_OBSERVE'):
                with self.subTest(key=key),self.assertRaises(ValueError):
                    _ = runner.validate_environment(replace(context,values=(*values,(key,'injected'))),14)
            self.assertFalse(held.closed)

    def test_config_absence_membership_is_bound_without_per_pass_rust_pin(self) -> None:
        from dataclasses import replace

        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp).resolve();root=base/'root';(root/'rust/crates').mkdir(parents=True)
            (root/'docs').mkdir();_ = (root/'docs/ARCHITECTURE.md').write_text('`rust/crates` `docs/missing`')
            home=base/'cargo-home';home.mkdir()
            source=root/'rust/crates/main.rs';_ = source.write_text('synthetic no-advance source')
            context=runner.ExecutionEnvironment((('CARGO_HOME',str(home)),),(),())
            with patch.object(runner,'DOCS_CHECKERS',dict[str,str]()),patch.object(runner,'git',return_value=b'rust/crates/main.rs\0'):
                before=runner.docs_bindings(root,context)
                self.assertEqual(before['architecture_exists'],{'rust/crates':True,'docs/missing':False})
                config=home/'config.toml';_ = config.write_text('[build]\njobs=2\n')
                with self.assertRaises(ValueError):_ = runner.docs_bindings(root,context)
                context=replace(context,external_inputs=((config,file_hash(config)),))
                after=runner.docs_bindings(root,context)
                self.assertNotEqual(before,after)
                _ = source.write_text('synthetic future farming source')
                self.assertEqual(runner.docs_bindings(root,context),after)

    def test_docs_missing_positive_terminal_cannot_claim_settlement(self) -> None:
        from generic_process import ProcessResult, ProcessSpec
        fixture=DocsTests.docs_fixture(self)
        def missing(spec: ProcessSpec) -> ProcessResult:
            spec.output_root.mkdir()
            return ProcessResult(spec.output_root/'process.json','PASS',0,123)
        with patch.object(runner,'execute',side_effect=missing),self.assertRaises(runner.StrictOwnershipUnknown):
            _ = fixture.run_one(14)
        receipt=document((fixture.base/'out/receipt.json').read_text())
        self.assertEqual(receipt.field('ownership').text(),'INCOMPLETE_DESCENDANTS_UNKNOWN')
        self.assertEqual(receipt.field('artifacts').value,{})

    def test_unexpected_docs_oracle_keeps_failed_checkpoint_and_propagates(self) -> None:
        for index,error in enumerate((RuntimeError('oracle'),KeyboardInterrupt(),SystemExit(3))):
            fixture=DocsTests.docs_fixture(self)
            with patch.object(runner,'docs_terminal',side_effect=error):
                try:_ = fixture.run_one(14,str(index))
                except (RuntimeError,KeyboardInterrupt,SystemExit) as actual:self.assertIs(actual,error)
                else:self.fail('oracle exception did not propagate')
            receipt=document((fixture.base/str(index)/'receipt.json').read_text())
            self.assertEqual(receipt.field('status').text(),'FAIL')
            self.assertEqual(receipt.field('native_exit').value,0)
            self.assertTrue(receipt.field('artifacts').mapping())
            patch.stopall()


class DocsClosureTests(unittest.TestCase):
    def test_lookup_endpoint_mutation_fails_after_real_tiny_child(self) -> None:
        fixture=DocsTests.docs_fixture(self)
        with patch.object(runner,'docs_bindings',side_effect=[{'config':None},{'config':'changed'}]):
            result=fixture.run_one(14)
        self.assertEqual(result.status,'FAIL')
        receipt=document(result.receipt.read_text())
        self.assertEqual(receipt.field('native_exit').value,0)
        self.assertIn('lookup bindings changed',result.receipt.read_text())

    def test_input_binding_mismatch_prevents_spawn(self) -> None:
        fixture=DocsTests.docs_fixture(self)
        fixture.product=runner.StaticProduct(fixture.root,'a'*40,(('docs/unbound','f'*64),))
        result=fixture.run_one(14)
        self.assertEqual(result.status,'FAIL')
        self.assertFalse((result.receipt.parent/'process').exists())

    def test_unknown_does_not_close_or_release_caller_lane(self) -> None:
        import fcntl
        from dataclasses import replace
        fixture=DocsTests.docs_fixture(self)
        lock=fixture.base/'lane'
        with lock.open('w') as held,lock.open('r') as contender:
            fcntl.flock(held,fcntl.LOCK_EX|fcntl.LOCK_NB)
            fixture.context=replace(fixture.context,held_lane_fd=held.fileno())
            with patch.object(runner,'execute',side_effect=OSError('terminal unavailable')),self.assertRaises(runner.StrictOwnershipUnknown):
                _ = fixture.run_one(14)
            self.assertFalse(held.closed)
            with self.assertRaises(BlockingIOError):fcntl.flock(contender,fcntl.LOCK_EX|fcntl.LOCK_NB)

    def test_system_table_and_checker_bytes_are_reviewed_inputs(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp).resolve();(root/'tools/v8').mkdir(parents=True)
            path=root/'tools/v8/system_table.py';_ = path.write_text('synthetic original cargo argv')
            context=runner.ExecutionEnvironment((),(),())
            expected={'system_table.py':file_hash(path)}
            _ = path.write_text('synthetic substituted JSON or shorter command')
            with patch.object(runner,'DOCS_CHECKERS',expected),self.assertRaisesRegex(ValueError,'checker changed'):
                _ = runner.docs_bindings(root,context)


    def test_unreadable_optional_config_is_not_recorded_as_absent(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            config=Path(temp).resolve()/'config.toml'
            self.assertIsNone(runner.docs_optional_hash(config))
            with patch.object(Path,'lstat',side_effect=PermissionError('unreadable config ancestor')),self.assertRaises(PermissionError):
                _ = runner.docs_optional_hash(config)


class DocsConfigPartitionTests(unittest.TestCase):
    def test_internal_configs_reach_actual_process_as_product_inputs(self) -> None:
        from mux2_gate_inputs import INPUT_DIRS, INPUT_FILES
        for index,config_name in enumerate(('.cargo/config.toml','rust/.cargo/config.toml','cargo-home/config')):
            with self.subTest(config=config_name):
                fixture=Fixture(self)
                for folder in (*INPUT_DIRS,'docs','.claude/skills','.agents/skills'):
                    (fixture.root/folder).mkdir(parents=True,exist_ok=True)
                for name in (*INPUT_FILES,'README.md','AGENTS.md','CLAUDE.md','CHANGELOG.md','docs/ARCHITECTURE.md'):
                    path=fixture.root/name;path.parent.mkdir(parents=True,exist_ok=True)
                    if not path.exists():_ = path.write_text('synthetic docs input')
                for skill in runner.DOCS_SKILLS:
                    target=fixture.root/'.claude/skills'/skill;target.mkdir()
                    _ = (target/'SKILL.md').write_text('synthetic skill')
                    (fixture.root/'.agents/skills'/skill).symlink_to('../../.claude/skills/'+skill)
                home=fixture.root/'cargo-home';home.mkdir()
                config=fixture.root/config_name;config.parent.mkdir(parents=True,exist_ok=True)
                _ = config.write_text('[build]\njobs=2\n')
                _ = fixture.child.write_text('#!/bin/sh\ntest -f '+str(config)+' || exit 9\necho "docs: PASS"\n')
                fixture.context=runner.ExecutionEnvironment((*fixture.context.values,('CARGO_HOME',str(home))),
                    tuple((name,path,file_hash(path)) for name,path,_ in fixture.context.tools),())
                with patch.object(runner,'_tools',return_value={'bash':fixture.child}), \
                     patch.object(runner,'DOCS_CHECKERS',dict[str,str]()), \
                     patch.object(runner,'git',return_value=b''), \
                     patch.object(runner,'inventory',side_effect=actual_inventory):
                    names=runner.docs_input_names(fixture.root,fixture.context)
                    fixture.product=runner.StaticProduct(fixture.root,'a'*40,tuple(sorted(actual_inventory(fixture.root,names).items())))
                    self.assertIn(config_name,dict(fixture.product.inputs))
                    result=fixture.run_one(14,str(index))
                    complete=fixture.product
                    fixture.product=runner.StaticProduct(fixture.root,'a'*40,tuple(row for row in complete.inputs if row[0]!=config_name))
                    omitted=fixture.run_one(14,'omitted-'+str(index))
                    self.assertEqual(omitted.status,'FAIL')
                    self.assertFalse((omitted.receipt.parent/'process').exists())
                    fixture.product=complete
                    fixture.context=runner.ExecutionEnvironment(fixture.context.values,fixture.context.tools,((config,file_hash(config)),))
                    misplaced=fixture.run_one(14,'external-'+str(index))
                    self.assertEqual(misplaced.status,'FAIL')
                    self.assertFalse((misplaced.receipt.parent/'process').exists())
                self.assertEqual(result.status,'PASS',result.receipt.read_text())
                native=document((result.receipt.parent/'process/process.json').read_text())
                self.assertEqual(native.field('native_exit').value,0)
                self.assertIn(config_name,native.field('before_inputs').mapping())
                self.assertNotIn(str(config),native.field('before_external_inputs').mapping())
                patch.stopall()


class DocsOwnedTests(unittest.TestCase):
    def test_owned_success_keeps_original_receipt(self) -> None:
        fixture=DocsTests.docs_fixture(self)
        result=runner.run_docs_owned(fixture.product,fixture.base/'out',fixture.context)
        self.assertEqual(result.status,'PASS')
        runner.verify_receipt(result.receipt,file_hash(result.receipt))
        document(result.receipt.read_text()).field('source_invocation').equals(['bash','tools/v8/check_docs.sh'])

    def test_positive_native_failure_is_unknown_before_inventory(self) -> None:
        fixture=DocsTests.docs_fixture(self,'echo "docs: FAIL"; exit 1')
        with self.assertRaises(runner.DocsOwnershipUnknown):
            _=runner.run_docs_owned(fixture.product,fixture.base/'out',fixture.context)
        self.assertFalse((fixture.base/'out/receipt.json').exists())
        self.assertEqual(fixture.source.call_count,1)
        document((fixture.base/'out/process/process.json').read_text()).field('native_exit').equals(1)
        document((fixture.base/'out/ownership.json').read_text()).field('final_inventory').equals(False)

    def test_prelaunch_rejection_is_settled_and_does_not_launch(self) -> None:
        fixture=DocsTests.docs_fixture(self);primary=ValueError('admission failure')
        with patch.object(runner,'_tools',side_effect=primary),patch.object(runner,'execute') as child,self.assertRaises(runner.DocsSettledFailure) as caught:
            _=runner.run_docs_owned(fixture.product,fixture.base/'out',fixture.context)
        child.assert_not_called();self.assertIs(caught.exception.original,primary)
        self.assertEqual(caught.exception.receipt,fixture.base/'out/receipt.json')
        document((fixture.base/'out/receipt.json').read_text()).field('process_status').equals('NOT_RUN')

    def test_stale_output_is_preserved_without_diagnostics(self) -> None:
        fixture=DocsTests.docs_fixture(self);out=fixture.base/'out';out.mkdir();_=(out/'raw').write_text('keep')
        with self.assertRaises(runner.DocsSettledFailure) as caught:
            _=runner.run_docs_owned(fixture.product,out,fixture.context)
        self.assertIsNone(caught.exception.receipt)
        self.assertEqual([p.name for p in out.iterdir()],['raw'])

    def test_pass0_then_oracle_or_postflight_failure_is_settled(self) -> None:
        for location in ('oracle','postflight'):
            with self.subTest(location=location):
                fixture=DocsTests.docs_fixture(self);primary=ValueError('failed '+location)
                selected=patch.object(runner,'docs_terminal',side_effect=primary) if location=='oracle' else patch.object(runner,'docs_bindings',side_effect=[{'synthetic':True},primary])
                with selected,self.assertRaises(runner.DocsSettledFailure) as caught:
                    _=runner.run_docs_owned(fixture.product,fixture.base/'out',fixture.context)
                self.assertIs(caught.exception.original,primary)
                self.assertEqual(fixture.source.call_count,2)
                document((fixture.base/'out/process/process.json').read_text()).field('native_exit').equals(0)
                document((fixture.base/'out/receipt.json').read_text()).field('status').equals('FAIL')
                self.assertFalse((fixture.base/'out/ownership.json').exists())

    def test_all_unreturned_exceptions_remain_unknown(self) -> None:
        for primary in (OSError('after spawn'),TypeError('after spawn'),KeyboardInterrupt(),SystemExit(2)):
            with self.subTest(exception=type(primary).__name__):
                fixture=DocsTests.docs_fixture(self)
                with patch.object(runner,'execute',side_effect=primary),self.assertRaises(runner.DocsOwnershipUnknown) as caught:
                    _=runner.run_docs_owned(fixture.product,fixture.base/'out',fixture.context)
                self.assertIs(caught.exception.original,primary)
                self.assertFalse((fixture.base/'out/receipt.json').exists())
                self.assertEqual(fixture.source.call_count,1)

    def test_failed_missing_or_unbound_completions_are_unknown(self) -> None:
        from generic_process import ProcessResult, ProcessSpec
        rows: tuple[tuple[int | None,int,Literal['PASS','FAIL']],...]=((1,123,'FAIL'),(-9,123,'FAIL'),(None,123,'FAIL'),(0,123,'FAIL'),(0,0,'PASS'),(0,123,'PASS'))
        for code,pid,status in rows:
            with self.subTest(code=code,pid=pid,status=status):
                fixture=DocsTests.docs_fixture(self)
                def child(spec: ProcessSpec, code: int | None=code,pid: int=pid,status: Literal['PASS','FAIL']=status) -> ProcessResult:
                    return ProcessResult(spec.output_root/'process.json',status,code,pid)
                with patch.object(runner,'execute',side_effect=child),self.assertRaises(runner.DocsOwnershipUnknown):
                    _=runner.run_docs_owned(fixture.product,fixture.base/'out',fixture.context)
                self.assertFalse((fixture.base/'out/receipt.json').exists())
                self.assertEqual(fixture.source.call_count,1)

    def test_contradictory_actual_completion_receipt_stays_unknown(self) -> None:
        from generic_process import ProcessResult, ProcessSpec, execute
        fixture=DocsTests.docs_fixture(self)
        def child(spec: ProcessSpec) -> ProcessResult:
            result=execute(spec)
            raw=document(result.receipt.read_text()).mapping();raw['native_exit']=1
            _=result.receipt.write_text(json.dumps(raw))
            return result
        with patch.object(runner,'execute',side_effect=child),self.assertRaises(runner.DocsOwnershipUnknown):
            _=runner.run_docs_owned(fixture.product,fixture.base/'out',fixture.context)
        self.assertEqual(fixture.source.call_count,1)
        self.assertFalse((fixture.base/'out/receipt.json').exists())

    def test_unknown_diagnostic_failure_retains_primary(self) -> None:
        fixture=DocsTests.docs_fixture(self);primary=TypeError('primary execution failure');diagnostic=KeyboardInterrupt()
        with patch.object(runner,'execute',side_effect=primary),patch.object(runner,'atomic_json',side_effect=diagnostic),self.assertRaises(runner.DocsOwnershipUnknown) as caught:
            _=runner.run_docs_owned(fixture.product,fixture.base/'out',fixture.context)
        self.assertIs(caught.exception.original,primary);self.assertIs(caught.exception.diagnostic,diagnostic)
        self.assertFalse((fixture.base/'out/receipt.json').exists());self.assertEqual(fixture.source.call_count,1)

    def test_terminal_write_failure_does_not_mask_first_validation_failure(self) -> None:
        from generic_process import atomic_json
        fixture=DocsTests.docs_fixture(self);primary=ValueError('oracle failure');secondary=OSError('terminal write')
        def writer(path: Path, value: dict[str,Json]) -> None:
            if path.name=='receipt.json':raise secondary
            atomic_json(path,value)
        with patch.object(runner,'docs_terminal',side_effect=primary),patch.object(runner,'atomic_json',side_effect=writer),self.assertRaises(runner.DocsSettledFailure) as caught:
            _=runner.run_docs_owned(fixture.product,fixture.base/'out',fixture.context)
        self.assertIs(caught.exception.original,primary);self.assertIs(caught.exception.diagnostic,secondary)
        self.assertEqual(caught.exception.receipt,fixture.base/'out/settled-failure.json')

    def test_existing_raw_receipt_is_never_overwritten_on_unknown(self) -> None:
        from generic_process import ProcessSpec
        fixture=DocsTests.docs_fixture(self);raw=b'raw diagnostic already written\n'
        def child(spec: ProcessSpec) -> None:
            _=(spec.output_root.parent/'receipt.json').write_bytes(raw)
            raise OSError('execution uncertain')
        with patch.object(runner,'execute',side_effect=child),self.assertRaises(runner.DocsOwnershipUnknown):
            _=runner.run_docs_owned(fixture.product,fixture.base/'out',fixture.context)
        self.assertEqual((fixture.base/'out/receipt.json').read_bytes(),raw)
        self.assertTrue((fixture.base/'out/ownership.json').exists())

    def test_hostile_primary_is_not_rendered_before_safe_diagnostics(self) -> None:
        class Hostile(ValueError):
            pass
        for phase in ('execute','oracle'):
            with self.subTest(phase=phase):
                fixture=DocsTests.docs_fixture(self);primary=Hostile('never rendered')
                target='execute' if phase=='execute' else 'docs_terminal'
                expected=runner.DocsOwnershipUnknown if phase=='execute' else runner.DocsSettledFailure
                with patch.object(Hostile,'__str__',side_effect=AssertionError('primary was rendered')),patch.object(runner,target,side_effect=primary),self.assertRaises(expected) as caught:
                    _=runner.run_docs_owned(fixture.product,fixture.base/'out',fixture.context)
                self.assertIs(caught.exception.original,primary)

    def test_second_diagnostic_failure_keeps_first_validation_primary(self) -> None:
        fixture=DocsTests.docs_fixture(self);primary=ValueError('oracle failure');secondary=OSError('all diagnostic writes fail')
        with patch.object(runner,'docs_terminal',side_effect=primary),patch.object(runner,'atomic_json',side_effect=secondary),self.assertRaises(runner.DocsSettledFailure) as caught:
            _=runner.run_docs_owned(fixture.product,fixture.base/'out',fixture.context)
        self.assertIs(caught.exception.original,primary);self.assertIs(caught.exception.diagnostic,secondary)
        self.assertIsNone(caught.exception.receipt)
        self.assertFalse((fixture.base/'out/ownership.json').exists())
