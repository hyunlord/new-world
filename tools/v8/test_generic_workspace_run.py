"""Offline adapter boundaries; never a workspace qualification."""
import unittest

import generic_workspace_run as runner


class ContractTests(unittest.TestCase):
    def test_explicit_original_commands(self) -> None:
        self.assertEqual(runner.workspace_argv('/cargo', 4), ('/cargo', 'test', '--workspace', '--no-fail-fast', '--', '--test-threads=8'))
        self.assertEqual(runner.workspace_argv('/cargo', 5), ('/cargo', 'test', '--workspace', '--release', '--no-fail-fast', '--', '--test-threads=8'))


import fcntl
import json
import os
import shlex
import sys
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import generic_cargo_child
from generic_gate import GateContractError
from generic_process import ProcessResult, ProcessSpec, atomic_json, execute, file_hash
from generic_workspace_capture import read
from generic_workspace_tests import Executable, Inventory, Target


class RuntimeTests(unittest.TestCase):
    def fixture(self, base: Path, *, step: int = 4, fail: bool = False, omit: bool = False) -> runner.WorkspaceRequest:
        root = base/'source'; package = root/'rust/crate'; package.mkdir(parents=True)
        source = package/'lib.rs'; manifest = package/'Cargo.toml'
        _ = source.write_text('synthetic source')
        _ = manifest.write_text('[package]\nname="tiny"\n[lib]\nname="tiny"\npath="lib.rs"\n')
        binary = base/'libtest'; python = Path(sys.executable).resolve()
        _ = binary.write_text('#!'+str(python)+'\n'+f'''import sys
if '--list' in sys.argv:
 print('0 tests, 0 benchmarks' if '--ignored' in sys.argv else 'a: test\\n\\n1 test, 0 benchmarks')
else:
 print('\\nrunning 1 test\\ntest a ... {"FAILED" if fail else "ok"}\\n\\ntest result: {"FAILED" if fail else "ok"}. {0 if fail else 1} passed; {1 if fail else 0} failed; 0 ignored; 0 measured; 0 filtered out; finished in 0s\\n')
 raise SystemExit({101 if fail else 0})
'''); binary.chmod(0o755)
        cargo = base/'cargo'
        body = f'''import os,shlex,subprocess,sys
argv=shlex.split(os.environ['CARGO_TARGET_TEST_HOST_RUNNER'])+[{str(binary)!r},'--test-threads=8']
print('     Running `'+shlex.join(argv)+'`',file=sys.stderr)
raise SystemExit(subprocess.call(argv,cwd={str(package)!r}))
'''
        if omit: body = 'raise SystemExit(0)\n'
        _ = cargo.write_text('#!'+str(python)+'\n'+body); cargo.chmod(0o755)
        helper = Path(generic_cargo_child.__file__).resolve()
        tools = {cargo, binary, python, *helper.parent.glob('*.py')}
        files = {str(path): file_hash(path) for path in tools if path not in {cargo, binary}}
        files.update({str(path): file_hash(path) for path in (source, manifest)})
        output = base/'output'; capture_manifest = base/'capture-manifest.json'
        profile = 'debug' if step == 4 else 'release'
        atomic_json(capture_manifest, {'schema_version': 1, 'invocation_id': 'tiny', 'step': step, 'arm': 'candidate',
            'profile': profile, 'output_root': str(output/'capture'), 'launcher_sha256': file_hash(helper),
            'python_sha256': file_hash(python), 'inputs': {key: value for key, value in files.items()},
            'artifacts': [{'id': 'tiny', 'path': str(binary), 'sha256': file_hash(binary)}], 'observer_variable': None})
        tools.add(capture_manifest)
        env = (('CARGO_TARGET_TEST_HOST_RUNNER', shlex.join((str(python), str(helper)))),
            ('CARGO_CAPTURE_MANIFEST', str(capture_manifest)), ('CARGO_CAPTURE_MANIFEST_SHA256', file_hash(capture_manifest)),
            ('RUSTC_WRAPPER', ''), ('RUSTC_WORKSPACE_WRAPPER', ''))
        spec = ProcessSpec(runner.workspace_argv(str(cargo), step), root/'rust', cargo, file_hash(cargo), root,
            tuple((str(path.relative_to(root)), file_hash(path)) for path in (source, manifest)), output/'parent', env,
            tuple(sorted((path, file_hash(path)) for path in tools)))
        descriptor = os.open(base/'lane', os.O_CREAT | os.O_RDWR, 0o600); self.addCleanup(os.close, descriptor)
        fcntl.flock(descriptor, fcntl.LOCK_EX)
        executable = Executable(Target('tiny', 'tiny', 'tiny', ('lib',), source, manifest, False, True), binary,
            4 if step == 4 else 5, profile)
        return runner.WorkspaceRequest(4 if step == 4 else 5, 'candidate', 'tiny', spec, Inventory((executable,), ()),
            capture_manifest, file_hash(capture_manifest), output, descriptor)

    def test_actual_debug_and_release_command_capture_and_listings(self) -> None:
        for step in (4, 5):
            with self.subTest(step=step), TemporaryDirectory() as tmp:
                base = Path(tmp).resolve(); request = self.fixture(base, step=step)
                with patch.object(runner, 'LOCK', base/'lane'): result = runner.run_workspace_owned(request)
                self.assertEqual(result.status, 'OBSERVED_CASES_ONLY')
                self.assertEqual(result.capture.completeness, 'COMPLETE')
                self.assertEqual(result.source_build_admission, 'PENDING_CALLER_HANDOFF')
                self.assertFalse(result.publication_authority)
                terminal = read(result.receipt)
                self.assertEqual(terminal.field('invocation_id').value, request.invocation)
                self.assertEqual(terminal.field('arm').value, request.arm)
                self.assertEqual(terminal.field('profile').value, 'debug' if step == 4 else 'release')

    def test_actual_native101_retained_fail(self) -> None:
        with TemporaryDirectory() as tmp:
            base = Path(tmp).resolve(); request = self.fixture(base, fail=True)
            with patch.object(runner, 'LOCK', base/'lane'): result = runner.run_workspace_owned(request)
            self.assertEqual(result.status, 'FAIL')
            self.assertEqual(read(result.receipt).field('parent_exit').value, 101)

    def test_missing_child_never_lists_or_finalizes(self) -> None:
        with TemporaryDirectory() as tmp:
            base = Path(tmp).resolve(); request = self.fixture(base, omit=True)
            with patch.object(runner, 'LOCK', base/'lane'), self.assertRaises(runner.WorkspaceOwnershipUnknown):
                _ = runner.run_workspace_owned(request)
            self.assertFalse((request.output/'listing-0-all').exists())
            self.assertFalse((request.output/'result.json').exists())

    def test_postspawn_error_preserves_fd_error_and_raw_evidence_without_handle_claim(self) -> None:
        with TemporaryDirectory() as tmp:
            base = Path(tmp).resolve(); request = self.fixture(base)
            original = KeyboardInterrupt()
            def interrupted(spec: ProcessSpec) -> ProcessResult:
                _ = execute(spec)
                raise original
            with patch.object(runner, 'LOCK', base/'lane'), patch.object(runner, 'execute', interrupted), \
                    self.assertRaises(runner.WorkspaceOwnershipUnknown) as caught:
                _ = runner.run_workspace_owned(request)
            self.assertIs(caught.exception.original, original)
            self.assertIs(caught.exception.request, request)
            self.assertEqual(caught.exception.process_handle, 'NOT_EXPOSED_BY_GENERIC_PROCESS')
            self.assertGreater(os.fstat(request.held_lane_fd).st_ino, 0)
            self.assertTrue((request.output/'parent/process-spawn.json').is_file())
            self.assertFalse((request.output/'result.json').exists())

    def test_wrapper_absent_is_not_equivalent_to_present_empty(self) -> None:
        with TemporaryDirectory() as tmp:
            base = Path(tmp).resolve(); request = self.fixture(base)
            request = replace(request, spec=replace(request.spec, environment=tuple((k,v) for k,v in request.spec.environment if k != 'RUSTC_WRAPPER')))
            with patch.object(runner, 'LOCK', base/'lane'), self.assertRaises(GateContractError):
                _ = runner.run_workspace_owned(request)
            self.assertFalse(request.output.exists())

    def test_late_additions_and_prior_listing_mutation_fail(self) -> None:
        for path in ('capture/000002/extra', 'parent/extra', 'listing-0-all/stdout.log'):
            with self.subTest(path=path), TemporaryDirectory() as tmp:
                base = Path(tmp).resolve(); request = self.fixture(base)
                def changed(spec: ProcessSpec, request: runner.WorkspaceRequest = request, path: str = path) -> ProcessResult:
                    result = execute(spec)
                    if spec.output_root.name == 'listing-0-ignored':
                        target = request.output/path; target.parent.mkdir(parents=True, exist_ok=True)
                        _ = target.write_text('changed')
                    return result
                with patch.object(runner, 'LOCK', base/'lane'), patch.object(runner, 'execute', changed), \
                        self.assertRaises(runner.WorkspaceSettledFailure):
                    _ = runner.run_workspace_owned(request)
                self.assertFalse((request.output/'result.json').exists())

    def test_completed_listing_membership_remains_exact(self) -> None:
        for mutation in ('empty-directory', 'added-file', 'type-change'):
            with self.subTest(mutation=mutation), TemporaryDirectory() as tmp:
                base = Path(tmp).resolve(); request = self.fixture(base)
                def changed(spec: ProcessSpec, mutation: str = mutation, request: runner.WorkspaceRequest = request) -> ProcessResult:
                    result = execute(spec)
                    target = request.output/'listing-0-all/member'
                    if spec.output_root.name == 'listing-0-all' and mutation == 'type-change':
                        target.mkdir()
                    if spec.output_root.name == 'listing-0-ignored':
                        if mutation == 'empty-directory': target.mkdir()
                        else:
                            if mutation == 'type-change': target.rmdir()
                            _ = target.write_text('late')
                    return result
                with patch.object(runner, 'LOCK', base/'lane'), patch.object(runner, 'execute', changed), \
                        self.assertRaises(runner.WorkspaceSettledFailure):
                    _ = runner.run_workspace_owned(request)
                self.assertFalse((request.output/'result.json').exists())

    def test_listing_jobserver_not_silently_stripped(self) -> None:
        with TemporaryDirectory() as tmp:
            base = Path(tmp).resolve(); request = self.fixture(base)
            with self.assertRaises(GateContractError):
                _ = runner.listing_spec(request, 'list', (str(request.inventory.executables[0].binary), '--list'),
                    request.spec.cwd, (('CARGO_MAKEFLAGS', '--jobserver-auth=3,4'),), base/'future', '0'*64)

    def test_source_owned_rustdoc_listing_jobs_use_actual_intent(self) -> None:
        from generic_workspace_capture import reconcile_capture
        from test_generic_workspace_capture import (
            exec_record,
            policy_fixture,
            policy_plan,
            policy_static,
        )
        f = policy_fixture(self)
        _ = policy_static(f, 1); _ = policy_static(f, 2); intent = exec_record(f)
        plan = policy_plan(f); capture = reconcile_capture(plan, f.result)
        self.assertEqual(capture.integrity, 'PASS')
        request = runner.WorkspaceRequest(4, 'candidate', 'test', f.spec, plan.inventory,
            f.manifest, file_hash(f.manifest), f.base/'unused', -1)
        jobs = runner.listing_jobs(request, replace(capture, targets=()))
        self.assertEqual(len(jobs), 1)
        policy = read(f.manifest).field('doctest_policy'); row = policy.field('invocations').array()[0]
        self.assertEqual(jobs[0][0], row.field('id').text())
        self.assertEqual(jobs[0][1], (policy.field('rustdoc').text(), *(arg.text() for arg in row.field('argv').array())))
        self.assertEqual(jobs[0][4], intent)
        self.assertEqual(dict(jobs[0][3]), read(intent).field('environment').mapping())
        with self.assertRaises(GateContractError):
            _ = runner.listing_jobs(request, replace(capture, targets=(), exec_intents=()))

    def test_ignored_and_zero_native_case_accounting(self) -> None:
        for zero in (False, True):
            with self.subTest(zero=zero), TemporaryDirectory() as tmp:
                base = Path(tmp).resolve(); request = self.fixture(base); binary = request.inventory.executables[0].binary
                raw = binary.read_text()
                if zero:
                    raw = raw.replace("'a: test\\n\\n1 test, 0 benchmarks'", "'0 tests, 0 benchmarks'")
                    raw = raw.replace('running 1 test\\ntest a ... ok', 'running 0 tests').replace('1 passed;', '0 passed;')
                else:
                    raw = raw.replace("'0 tests, 0 benchmarks' if", "'b: test\\n\\n1 test, 0 benchmarks' if")
                    raw = raw.replace('a: test\\n\\n1 test,', 'a: test\\nb: test\\n\\n2 tests,')
                    raw = raw.replace('running 1 test\\ntest a ... ok', 'running 2 tests\\ntest a ... ok\\ntest b ... ignored').replace('0 ignored;', '1 ignored;')
                _ = binary.write_text(raw)
                manifest = read(request.manifest).mapping()
                manifest['artifacts'] = [{'id': 'tiny', 'path': str(binary), 'sha256': file_hash(binary)}]
                _ = request.manifest.write_text(json.dumps(manifest))
                env = tuple((k, file_hash(request.manifest) if k == 'CARGO_CAPTURE_MANIFEST_SHA256' else v) for k,v in request.spec.environment)
                external = tuple((path, file_hash(path)) for path,_ in request.spec.external_inputs)
                request = replace(request, manifest_sha256=file_hash(request.manifest), spec=replace(request.spec, environment=env, external_inputs=external))
                with patch.object(runner, 'LOCK', base/'lane'): result = runner.run_workspace_owned(request)
                self.assertEqual(result.status, 'OBSERVED_CASES_ONLY')

    def test_changed_command_timeout_or_startup_environment_never_dispatches(self) -> None:
        with TemporaryDirectory() as tmp:
            base = Path(tmp).resolve(); request = self.fixture(base)
            specs = (replace(request.spec, argv=(*request.spec.argv, '--ignored')),
                replace(request.spec, timeout_seconds=1),
                replace(request.spec, environment=(*request.spec.environment, ('BASH_ENV', '/foreign'))))
            for spec in specs:
                with self.subTest(spec=spec.argv), patch.object(runner, 'LOCK', base/'lane'), patch.object(runner, 'execute') as launch, \
                        self.assertRaises(GateContractError):
                    _ = runner.run_workspace_owned(replace(request, spec=spec))
                launch.assert_not_called()
                self.assertFalse(request.output.exists())
