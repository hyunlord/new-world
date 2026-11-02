"""Offline selection and original outer-shell fixtures; no real launcher worlds."""
import json
import os
import subprocess
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import gate_play_child as child
from generic_gate import GateContractError, document
from generic_process import file_hash
from mux2_gate_inputs import Json


class CaseTests(unittest.TestCase):
    def suite(self, root: Path) -> tuple[Path, str]:
        cases: dict[str, Json] = {}
        for index, label in enumerate(child.CASES):
            if index in (1, 2):
                cases[label] = None
            else:
                p = root / f'plan-{index}.json'
                _ = p.write_text('{}')
                cases[label] = {'path': str(p), 'sha256': file_hash(p)}
        path = root / 'suite.json'
        _ = path.write_text(json.dumps({'schema_version': 1, 'cases': cases}))
        return path, file_hash(path)

    def test_exact_six_mapping_and_distinct_plans(self) -> None:
        with TemporaryDirectory() as folder:
            path, digest = self.suite(Path(folder).resolve())
            selected = [child.select_case(path, digest, label, 'version') for label in child.CASES if label not in child.CASES[1:3]]
            self.assertEqual(len({row.plan for row in selected}), 4)
            for label in child.CASES[1:3]:
                with self.assertRaises(GateContractError):
                    _ = child.select_case(path, digest, label, 'version')

    def test_missing_extra_duplicate_changed_plans_rejected(self) -> None:
        for fault in ('missing', 'extra', 'duplicate', 'changed'):
            with self.subTest(fault=fault), TemporaryDirectory() as folder:
                root = Path(folder).resolve(); path, digest = self.suite(root)
                node = document(path.read_text()).mapping(); cases = document(path.read_text()).field('cases').mapping()
                if fault == 'missing':
                    _ = cases.pop('bad --speed')
                elif fault == 'extra':
                    cases['other'] = None
                elif fault == 'duplicate':
                    cases[child.CASES[3]] = cases[child.CASES[0]]
                else:
                    _ = (root / 'plan-0.json').write_text('changed')
                if fault != 'changed':
                    node['cases'] = cases; _ = path.write_text(json.dumps(node)); digest = file_hash(path)
                with self.assertRaises(GateContractError):
                    _ = child.select_case(path, digest, child.CASES[0], 'version')

    def test_unknown_role_and_bad_hash_rejected(self) -> None:
        with TemporaryDirectory() as folder:
            path, digest = self.suite(Path(folder).resolve())
            for checksum, label, role in ((digest, child.CASES[0], 'build'), (digest, child.CASES[3], 'windowed'), ('0'*64, child.CASES[0], 'version'), (digest, 'arbitrary', 'version')):
                with self.assertRaises(GateContractError):
                    _ = child.select_case(path, checksum, label, role)

    def test_actual_child_parent_identity_and_selection(self) -> None:
        with TemporaryDirectory() as folder:
            root = Path(folder).resolve(); suite, _ = self.suite(root)
            plan = root / 'plan-0.json'; executable = Path('/bin/sh').resolve()
            argv = ('/bin/sh', '-c', 'printf actual; exit 3')
            _ = plan.write_text(json.dumps({'schema_version': 1, 'roles': {'version': {
                'argv': list(argv), 'cwd': str(Path.cwd()), 'executable': str(executable),
                'executable_sha256': file_hash(executable), 'inputs': {}, 'artifacts': [],
                'output': str(root / 'child'), 'timeout_seconds': None, 'merged': False}}}))
            node = document(suite.read_text()).mapping(); cases = document(suite.read_text()).field('cases').mapping()
            cases[child.CASES[0]] = {'path': str(plan), 'sha256': file_hash(plan)}; node['cases'] = cases
            _ = suite.write_text(json.dumps(node)); selected = child.select_case(suite, file_hash(suite), child.CASES[0], 'version')
            with patch('sys.stdout'), patch('sys.stderr'):
                code = child.run(plan, file_hash(plan), 'version', argv, selected)
            self.assertEqual(code, 3)
            row = document((root / 'child/receipt.json').read_text())
            row.field('wrapper_pid').equals(os.getpid()); row.field('parent_pid').equals(os.getppid())
            row.field('parent_pid_after_wait').equals(os.getppid()); row.field('case').equals(child.CASES[0])
            row.field('historical_observer_qualified').equals(False)

    def test_suite_mutation_during_child_retains_failure(self) -> None:
        with TemporaryDirectory() as folder:
            root = Path(folder).resolve(); suite, _ = self.suite(root)
            plan = root / 'plan-0.json'; executable = Path('/bin/sh').resolve()
            argv = ('/bin/sh', '-c', 'printf changed > "$1"', 'fixture', str(suite))
            _ = plan.write_text(json.dumps({'schema_version': 1, 'roles': {'version': {
                'argv': list(argv), 'cwd': str(Path.cwd()), 'executable': str(executable),
                'executable_sha256': file_hash(executable), 'inputs': {}, 'artifacts': [],
                'output': str(root / 'child'), 'timeout_seconds': None, 'merged': False}}}))
            node = document(suite.read_text()).mapping(); cases = document(suite.read_text()).field('cases').mapping()
            cases[child.CASES[0]] = {'path': str(plan), 'sha256': file_hash(plan)}; node['cases'] = cases
            _ = suite.write_text(json.dumps(node)); selected = child.select_case(suite, file_hash(suite), child.CASES[0], 'version')
            with patch('sys.stdout'), patch('sys.stderr'):
                code = child.run(plan, file_hash(plan), 'version', argv, selected)
            self.assertEqual(code, 125)
            row = document((root / 'child/receipt.json').read_text())
            row.field('native_exit').equals(0); self.assertTrue(row.field('errors').array())

    def test_main_rejects_raw_single_plan_presence_with_suite(self) -> None:
        with TemporaryDirectory() as folder:
            path, digest = self.suite(Path(folder).resolve())
            for legacy in ({'WORLDSIM_GATE_PLAY_PLAN': '/'}, {'WORLDSIM_GATE_PLAY_PLAN': '.'},
                           {'WORLDSIM_GATE_PLAY_PLAN': ''}, {'WORLDSIM_GATE_PLAY_PLAN_SHA256': ''},
                           {'WORLDSIM_GATE_PLAY_PLAN_SHA256': '0'*64}):
                environment = {'WORLDSIM_GATE_PLAY_SUITE_PLAN': str(path),
                    'WORLDSIM_GATE_PLAY_SUITE_PLAN_SHA256': digest,
                    'WORLDSIM_GATE_PLAY_CASE': child.CASES[0], **legacy}
                with self.subTest(legacy=legacy), patch.dict(os.environ, environment, clear=True), \
                        patch('sys.argv', ['gate_play_child.py', 'version', '--', '/bin/sh']), \
                        patch('sys.stderr'), patch.object(child, 'run', return_value=0) as run:
                    with self.assertRaises(SystemExit) as failure:
                        child.main()
                    self.assertEqual(failure.exception.code, 125)
                    run.assert_not_called()

    def test_original_full_outer_argv_order_and_default_off(self) -> None:
        source = Path(__file__).with_name('test_play.sh').read_bytes()
        for enabled in (False, True):
            with self.subTest(enabled=enabled), TemporaryDirectory() as folder:
                root = Path(folder).resolve(); scripts = root / 'tools/v8'; scripts.mkdir(parents=True)
                _ = (scripts / 'test_play.sh').write_bytes(source)
                # Deliberately synthetic play.sh: does not fetch, build, invoke Godot or create worlds.
                fixture = '''#!/bin/bash
printf '%s|%s|%s|%s|%s\\n' "${WORLDSIM_GATE_PLAY_CASE:-absent}" "$1" "${2:-}" "${3:-}" "${GODOT_BIN:-}" >> "$FIXTURE_LOG"
case "$1 ${2:-} ${3:-}" in
'no-such-version-v8play  ') exit 3;;
'm1 --speed 7') exit 4;;
esac
[[ "${GODOT_BIN:-}" == /nonexistent ]] && exit 2
exit 0
'''
                _ = (root / 'tools/play.sh').write_text(fixture)
                env = {'PATH': '/usr/bin:/bin', 'HOME': str(root), 'FIXTURE_LOG': str(root / 'calls')}
                if enabled:
                    env.update(WORLDSIM_GATE_PLAY_RECEIPTS='1', WORLDSIM_GATE_PLAY_SUITE_PLAN=str(root / 'unread-by-fixture'))
                result = subprocess.run(['/bin/bash', str(scripts / 'test_play.sh')], env=env, capture_output=True, check=False)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(result.stdout.count(b'PLAY_TEST_PASS '), 6)
                self.assertTrue(result.stdout.endswith(b'PLAY_TEST_DONE failures=0\n'))
                calls = (root / 'calls').read_text().splitlines()
                self.assertEqual([line.split('|')[0] for line in calls], list(child.CASES) if enabled else ['absent']*6)
                self.assertEqual([line.split('|')[1:4] for line in calls], [
                    ['no-such-version-v8play', '', ''], ['m1', '--headless-check', ''],
                    ['m1', '--speed', '7'], ['main', '--headless-check', ''],
                    ['m1', '--headless-check', ''], ['v7', '--headless-check', '']])
                self.assertEqual(calls[1].split('|')[4], '/nonexistent')
