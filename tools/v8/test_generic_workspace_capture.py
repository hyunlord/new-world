"""Static synthetic receipts only; no Cargo/build/native execution claims."""
import hashlib
import json
import os
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from typing import final

from generic_process import ProcessResult, ProcessSpec, environment_for, file_hash
from generic_workspace_capture import CapturePlan, read, reconcile_capture, snapshot
from generic_workspace_tests import Executable, Inventory, Target
from mux2_gate_inputs import Json


def write(path: Path, data: dict[str, Json]) -> None:
    _ = path.write_text(json.dumps(data))


@final
class Fixture:
    def __init__(self, test: unittest.TestCase) -> None:
        temporary = tempfile.TemporaryDirectory()
        test.addCleanup(temporary.cleanup)
        self.base = Path(temporary.name).resolve()
        self.source = self.base/'source'
        self.capture = self.base/'capture'
        self.output = self.base/'parent'
        for path in (self.source/'rust', self.capture, self.output): path.mkdir(parents=True)
        self.cargo = self.base/'cargo'
        _ = self.cargo.write_bytes(b'synthetic cargo')
        self.helper = self.base/'helper.py'
        _ = self.helper.write_bytes(b'synthetic helper')
        self.manifest = self.base/'manifest.json'
        self.executables: list[Executable] = []
        artifacts: list[Json] = []
        for name in ('alpha', 'beta'):
            root = self.source/'rust'/name
            root.mkdir()
            _ = (root/'Cargo.toml').write_bytes(b'synthetic manifest')
            _ = (root/'lib.rs').write_bytes(b'synthetic source')
            binary = root/'binary'
            _ = binary.write_bytes(name.encode())
            target = Target(name, name, 'same_name', ('lib',), root/'lib.rs', root/'Cargo.toml', True, True)
            self.executables.append(Executable(target, binary, 4, 'debug'))
            artifacts.append({'id': name, 'path': str(binary), 'sha256': file_hash(binary)})
        self.manifest_data: dict[str, Json] = {'schema_version': 1, 'invocation_id': 'test', 'step': 4, 'arm': 'candidate',
            'profile': 'debug', 'output_root': str(self.capture), 'launcher_sha256': file_hash(self.helper),
            'python_sha256': file_hash(self.cargo), 'inputs': {str(self.helper): file_hash(self.helper)},
            'artifacts': artifacts, 'observer_variable': 'WORLDSIM_GATE_TEST_OUT'}
        write(self.manifest, self.manifest_data)
        self.spec = ProcessSpec((str(self.cargo), 'test', '--workspace', '--no-fail-fast', '--', '--test-threads=8'),
            self.source/'rust', self.cargo, file_hash(self.cargo), self.source, (), self.output,
            (('CARGO_CAPTURE_MANIFEST', str(self.manifest)), ('CARGO_CAPTURE_MANIFEST_SHA256', file_hash(self.manifest))),
            ((self.manifest, file_hash(self.manifest)), (self.helper, file_hash(self.helper))))
        self.parent_data: dict[str, Json] = {}
        self.parent(0)
        _ = (self.capture/'sequence.lock').write_bytes(b'')

    def parent(self, code: int) -> None:
        status = 'PASS' if code == 0 else 'FAIL'
        for name in ('stdout', 'stderr'): _ = (self.output/(name+'.log')).write_bytes(b'synthetic Cargo stream')
        self.parent_data = {'schema_version': 1, 'argv': list(self.spec.argv), 'cwd': str(self.spec.cwd),
            'executable': str(self.cargo), 'input_root': str(self.source), 'output_root': str(self.output),
            'environment': {key: value for key, value in environment_for(self.spec).items()}, 'environment_policy': 'fixed_defaults_plus_explicit_only_no_inheritance',
            'fresh_directories': [], 'expected_executable_sha256': file_hash(self.cargo),
            'before_executable_sha256': file_hash(self.cargo), 'after_executable_sha256': file_hash(self.cargo),
            'status': status, 'native_exit': code, 'pid': 100, 'errors': [] if code == 0 else [f'native exit {code}'],
            'started_at': '2026-11-01T00:00:00+00:00', 'ended_at': '2026-11-01T00:00:03+00:00', 'duration_ns': 2}
        for suffix, rows in (('inputs', {}), ('external_inputs', {str(p): h for p, h in self.spec.external_inputs})):
            for prefix in ('expected_', 'before_', 'after_'): self.parent_data[prefix+suffix] = {key: value for key, value in rows.items()}
        for name in ('stdout', 'stderr'): self.parent_data[name+'_sha256'] = file_hash(self.output/(name+'.log'))
        write(self.output/'process.json', self.parent_data)
        write(self.output/'process-start.json', self.parent_data)
        write(self.output/'process-spawn.json', {'pid': 100})
        self.result = ProcessResult(self.output/'process.json', 'PASS' if code == 0 else 'FAIL', code, 100)

    def child(self, index: int, code: int = 0, *, pending: bool = False) -> Path:
        executable = self.executables[index-1]
        directory = self.capture/f'{index:06d}'
        directory.mkdir()
        _ = (self.capture/'sequence.lock').write_text(str(index))
        claim = self.capture/('artifact-'+hashlib.sha256(str(executable.binary).encode()).hexdigest()+'.claim')
        _ = claim.write_text(str(index))
        identity: dict[str, Json] = {'schema_version': 1, 'component': 'NATIVE_CAPTURE_COMPONENT', 'invocation_id': 'test',
            'step': 4, 'arm': 'candidate', 'profile': 'debug', 'sequence': index, 'parent_pid': 100, 'launcher_pid': 200,
            'argv': [str(executable.binary), '--test-threads=8'], 'cwd': str(executable.cwd),
            'manifest_sha256': file_hash(self.manifest), 'started_ns': 30+index}
        write(directory/'pending.json', identity)
        if pending: return directory
        (directory/'observer').mkdir()
        _ = (directory/'observer/world.json').write_bytes(b'synthetic observer, no world claim')
        original = {**environment_for(self.spec), 'CARGO_MANIFEST_DIR': str(executable.cwd), 'DYLD_LIBRARY_PATH': '/cargo/actual'}
        additions = {'WORLDSIM_GATE_TEST_OUT': str(directory/'observer')}
        started: dict[str, Json] = {**identity, 'child_pid': 300, 'artifact_id': executable.target.package,
                   'before_binary_sha256': file_hash(executable.binary), 'environment': {**original, **additions}, 'observer_additions': {key: value for key, value in additions.items()}}
        write(directory/'started.json', started)
        for name in ('stdout', 'stderr'): _ = (directory/(name+'.log')).write_bytes(b'running 0 tests\n')
        terminal: dict[str, Json] = {**started, 'native_exit': code, 'native_signal': -code if code < 0 else None,
            'received_signals': [], 'original_environment': {key: value for key, value in original.items()}, 'after_binary_sha256': file_hash(executable.binary),
            'expected_inputs': {str(self.helper): file_hash(self.helper)}, 'after_inputs': {str(self.helper): file_hash(self.helper)},
            'ended_ns': 10, 'output_inventory': {k: v for k, v in snapshot(directory).items()}, 'errors': [],
            'capture_status': 'COMPLETE', 'workspace_case_accounting': 'INCOMPLETE', 'doctests': 'INCOMPLETE', 'world_coverage': 'INCOMPLETE'}
        for name in ('stdout', 'stderr'): terminal[name+'_sha256'] = file_hash(directory/(name+'.log'))
        write(directory/'receipt.json', terminal)
        return directory

    def plan(self) -> CapturePlan:
        return CapturePlan('test', 4, 'candidate', 'debug', self.spec, file_hash(self.result.receipt), self.manifest,
            file_hash(self.manifest), Inventory(tuple(self.executables), ()), self.capture, tuple(snapshot(self.capture).items()))


class CaptureTests(unittest.TestCase):
    def test_complete_same_name_targets_pid_reuse_and_wallclock_adjustment(self) -> None:
        f = Fixture(self)
        _ = f.child(1)
        _ = f.child(2)
        result = reconcile_capture(f.plan(), f.result)
        self.assertEqual((result.integrity, result.completeness, result.outcome), ('PASS', 'COMPLETE', 'SUCCEEDED'))
        self.assertEqual([row.executable.target.package for row in result.targets], ['alpha', 'beta'])
        self.assertEqual([row.child_pid for row in result.targets], [300, 300])
        self.assertEqual(result.world_safety, 'INCOMPLETE')
        self.assertIn('UNRESOLVED', result.cargo_environment_qualification)
        self.assertIn(('DYLD_LIBRARY_PATH', '/cargo/actual'), result.targets[0].original_environment)

    def test_complete_nonzero_and_signal_are_native_failures_not_integrity_failures(self) -> None:
        f = Fixture(self)
        f.parent(101)
        _ = f.child(1, 1)
        _ = f.child(2, -15)
        result = reconcile_capture(f.plan(), f.result)
        self.assertEqual((result.integrity, result.completeness, result.outcome), ('PASS', 'COMPLETE', 'FAILED'))
        self.assertEqual([row.native_exit for row in result.targets], [1, -15])

    def test_pending_parent_failure_missing_target_and_claim_only_retained(self) -> None:
        f = Fixture(self)
        f.parent(101)
        _ = f.child(1, pending=True)
        _ = (f.capture/'artifact-orphan.claim').write_text('2')
        result = reconcile_capture(f.plan(), f.result)
        self.assertEqual(result.completeness, 'INCOMPLETE')
        self.assertEqual(result.outcome, 'FAILED')
        self.assertEqual(len(result.missing), 1)
        self.assertTrue(any('artifact-orphan.claim' in path for path, _ in result.rejected))
        self.assertTrue(any(path.endswith('pending.json') for path, _ in result.hashes))

    def test_parent_zero_with_missing_targets_remains_incomplete(self) -> None:
        f = Fixture(self)
        _ = f.child(1)
        self.assertEqual(reconcile_capture(f.plan(), f.result).completeness, 'INCOMPLETE')

    def test_mutated_evidence_or_source_rejected_without_discarding_hashes(self) -> None:
        for defect in ('parent', 'raw', 'binary', 'manifest', 'source'):
            with self.subTest(defect=defect):
                f = Fixture(self)
                child = f.child(1)
                _ = f.child(2)
                plan = f.plan()
                path = {'parent': f.result.receipt, 'raw': child/'stdout.log', 'binary': f.executables[0].binary,
                        'manifest': f.manifest, 'source': f.helper}[defect]
                _ = path.write_bytes(b'mutated')
                result = reconcile_capture(plan, f.result)
                self.assertEqual(result.integrity, 'FAIL')
                self.assertTrue(result.hashes)

    def test_wrong_child_identity_env_and_subtree_rejected(self) -> None:
        cases: tuple[tuple[str, Json], ...] = (('parent_pid', 999), ('argv', ['other']), ('cwd', '/foreign'), ('native_signal', 15),
                             ('environment', {}), ('original_environment', {'WORLDSIM_GATE_TEST_OUT': '/preexisting'}),
                             ('output_inventory', {}))
        for field, wrong in cases:
            with self.subTest(field=field):
                f = Fixture(self)
                child = f.child(1)
                _ = f.child(2)
                data = read(child/'receipt.json').mapping()
                data[field] = wrong
                write(child/'receipt.json', data)
                self.assertEqual(reconcile_capture(f.plan(), f.result).integrity, 'FAIL')

    def test_unknown_rustdoc_launcher_retained_without_native_completion_claim(self) -> None:
        f = Fixture(self)
        f.parent(101)
        _ = f.child(1)
        _ = f.child(2)
        unknown = f.capture/'000003'
        unknown.mkdir()
        write(unknown/'pending.json', {'argv': ['/var/transient/rustdoctestXYZ/rust_out'], 'parent_pid': 999})
        write(unknown/'receipt.json', {'capture_status': 'FAIL', 'child_pid': None, 'native_exit': None})
        _ = (f.capture/'sequence.lock').write_text('3')
        result = reconcile_capture(f.plan(), f.result)
        self.assertEqual(len(result.targets), 2)
        self.assertEqual(result.completeness, 'INCOMPLETE')
        self.assertEqual(result.doctests, 'INCOMPLETE')
        self.assertTrue(any(path == str(unknown) for path, _ in result.rejected))
        self.assertTrue(any(path == str(unknown/'receipt.json') for path, _ in result.hashes))

    def test_started_without_terminal_retains_child_pid(self) -> None:
        f = Fixture(self)
        child = f.child(1)
        (child/'receipt.json').unlink()
        result = reconcile_capture(f.plan(), f.result)
        self.assertEqual(result.targets[0].child_pid, 300)
        self.assertFalse(result.targets[0].capture_complete)
        self.assertEqual(result.completeness, 'INCOMPLETE')

    def test_unreadable_record_preserves_frozen_ledger(self) -> None:
        f = Fixture(self)
        child = f.child(1)
        _ = (child/'pending.json').write_bytes(b'not JSON')
        plan = f.plan()
        result = reconcile_capture(plan, f.result)
        self.assertTrue(result.rejected)
        self.assertEqual(result.declared_snapshot, plan.captured_files)
        self.assertTrue(any(path.endswith('pending.json') for path, _ in result.hashes))

    def test_profile_and_parent_integrity_are_not_native_failure(self) -> None:
        f = Fixture(self)
        _ = f.child(1)
        _ = f.child(2)
        self.assertEqual(reconcile_capture(replace(f.plan(), profile='release'), f.result).integrity, 'FAIL')
        f.parent_data['errors'] = ['postflight: source drift']
        write(f.result.receipt, f.parent_data)
        self.assertEqual(reconcile_capture(f.plan(), f.result).integrity, 'FAIL')


if __name__ == '__main__':
    _ = unittest.main()


class PolicyCaptureTests(unittest.TestCase):
    def test_policy_two_retains_static_prefix_without_inventing_doctest_cases(self) -> None:
        f = Fixture(self)
        f.manifest_data['doctest_policy'] = {'schema_version': 2}
        write(f.manifest, f.manifest_data)
        f.spec = replace(f.spec, environment=(('CARGO_CAPTURE_MANIFEST', str(f.manifest)),
            ('CARGO_CAPTURE_MANIFEST_SHA256', file_hash(f.manifest))),
            external_inputs=((f.manifest, file_hash(f.manifest)), (f.helper, file_hash(f.helper))))
        f.parent(0)
        _ = f.child(1)
        result = reconcile_capture(f.plan(), f.result)
        self.assertIn('000001', result.ledger[0])
        self.assertEqual(result.integrity, 'FAIL')
        self.assertEqual(result.doctests, 'INCOMPLETE')


def policy_fixture(test: unittest.TestCase) -> Fixture:
    import sys

    import gate_rustdoc_exec as wrapper

    f = Fixture(test)
    root = f.base/'exec'; root.mkdir()
    temporary = f.base/'temporary'; temporary.mkdir()
    tools = [f.base/name for name in ('rustdoc', 'version', 'shim')]
    for path in tools: _ = path.write_bytes(path.name.encode())
    target = f.executables[0].target
    args = ['--test', str(target.source), '--crate-name', 'alpha', '--edition', '2021', '--test-run-directory', str(target.manifest.parent)]
    inputs = {str(path): file_hash(path) for path in (*tools, f.helper, target.source, target.manifest, Path(wrapper.__file__).resolve())}
    f.manifest_data['python_sha256'] = file_hash(Path(sys.executable).resolve())
    f.manifest_data['inputs'] = {k: v for k, v in inputs.items()}
    f.manifest_data['doctest_policy'] = {'schema_version': 2, 'parent_receipt': str(f.output/'process.json'),
        'rustdoc': str(tools[0]), 'rustdoc_sha256': file_hash(tools[0]), 'version_receipt': str(tools[1]),
        'version_sha256': file_hash(tools[1]), 'entrypoint': str(tools[2]), 'entrypoint_sha256': file_hash(tools[2]),
        'wrapper_sha256': file_hash(Path(wrapper.__file__).resolve()), 'exec_root': str(root), 'temporary_root': str(temporary),
        'artifact_dependencies': [], 'dependency_searches': [], 'invocations': [{'id': 'alpha-doc', 'package_id': 'alpha',
            'source': str(target.source), 'source_argument': str(target.source), 'package_manifest': str(target.manifest),
            'compiler_cwd': str(f.source/'rust'), 'run_cwd': str(target.manifest.parent), 'crate_name': 'alpha', 'edition': '2021', 'argv': [arg for arg in args]}]}
    write(f.manifest, f.manifest_data)
    f.spec = replace(f.spec, environment=(('CARGO_CAPTURE_MANIFEST', str(f.manifest)),
        ('CARGO_CAPTURE_MANIFEST_SHA256', file_hash(f.manifest))), external_inputs=((f.manifest, file_hash(f.manifest)),
        (f.helper, file_hash(f.helper)), (Path(sys.executable).resolve(), file_hash(Path(sys.executable).resolve()))))
    f.parent(0)
    return f


def exec_record(f: Fixture, name: str = 'exec-one', probe: bool = False) -> Path:
    import gate_rustdoc_exec as wrapper

    policy = read(f.manifest).field('doctest_policy')
    directory = Path(policy.field('exec_root').text())/name; directory.mkdir()
    invocation = policy.field('invocations').array()[0]
    args = ['-vV'] if probe else [arg.text() for arg in invocation.field('argv').array()]
    row: dict[str, Json] = {'schema_version': 1, 'component': 'RUSTDOC_EXEC_INTENT', 'occurrence': name,
        'invocation_id': 'test', 'step': 4, 'arm': 'candidate', 'profile': 'debug', 'manifest_sha256': file_hash(f.manifest),
        'wrapper_pid': 200, 'cargo_parent_pid': 100, 'cwd': str(f.source/'rust'), 'started_ns': 42,
        'rustdoc_argv': [policy.field('rustdoc').text(), *args], 'wrapper_argv': [str(Path(wrapper.__file__).resolve()), *args],
        'environment': dict(environment_for(f.spec))}
    write(directory/'exec-entry.json', row)
    row.update({'source_invocation': None if probe else 'alpha-doc', 'ancestor': {'expected_terminal': str(f.result.receipt),
        'cargo_pid': 100, 'spawn_receipt': str(f.output/'process-spawn.json'), 'spawn_sha256': file_hash(f.output/'process-spawn.json'),
        'final_parent_qualification': 'PENDING_AUTHENTICATED_PROCESS_RECEIPT_JOIN'}})
    write(directory/'exec-intent.json', row)
    return directory/'exec-intent.json'


def policy_plan(f: Fixture) -> CapturePlan:
    root = Path(read(f.manifest).field('doctest_policy').field('exec_root').text())
    return replace(f.plan(), exec_files=tuple(snapshot(root).items()), inventory=Inventory(tuple(f.executables), (f.executables[0].target,)))


def policy_static(f: Fixture, index: int) -> Path:
    directory = f.child(index)
    row = read(directory/'receipt.json').mapping()
    inputs = read(f.manifest).field('inputs').value
    row['expected_inputs'] = inputs; row['after_inputs'] = inputs
    write(directory/'receipt.json', row)
    return directory


def transient_record(f: Fixture, intent: Path, code: int = 0) -> Path:
    policy = read(f.manifest).field('doctest_policy')
    directory = f.capture/'000003'; directory.mkdir()
    _ = (f.capture/'sequence.lock').write_text('3')
    endpoint = Path(policy.field('temporary_root').text())/'rustdoctestGone/rust_out'
    saved = directory/'transient-binary'; _ = saved.write_bytes(b'retained executable')
    identity: dict[str, Json] = {'lexical': str(endpoint), 'canonical': str(endpoint), 'alias': None,
        'device': 1, 'inode': 1, 'uid': os.getuid(), 'size': saved.stat().st_size, 'mode': 0o100755}
    invocation = policy.field('invocations').array()[0]
    admission: dict[str, Json] = {'classification': 'RUSTDOC_TRANSIENT', 'exec_intent': str(intent), 'exec_sha256': file_hash(intent),
        'ancestor': read(intent).field('ancestor').value, 'live_parent_pid': 200, 'live_parent_path': policy.field('rustdoc').value,
        'identity_before': identity, 'saved_binary': str(saved), 'sha256': file_hash(saved),
        'temporary_root': policy.field('temporary_root').value, 'endpoint_limit': 'NO_ATOMIC_EXEC_OR_HOSTILE_PATH_RACE_PROOF',
        'doctest_cases': 'INCOMPLETE', 'compiler_cwd': invocation.field('compiler_cwd').value, 'run_cwd': invocation.field('run_cwd').value}
    write(directory/'transient-admission.json', admission)
    write(directory/'live-parent.json', {'parent_before': 200, 'parent_after': 200, 'path': policy.field('rustdoc').value, 'error': None})
    pending: dict[str, Json] = {'schema_version': 1, 'component': 'NATIVE_CAPTURE_COMPONENT', 'invocation_id': 'test', 'step': 4,
        'arm': 'candidate', 'profile': 'debug', 'sequence': 3, 'parent_pid': 200, 'launcher_pid': 200,
        'argv': [str(endpoint)], 'cwd': invocation.field('run_cwd').value, 'manifest_sha256': file_hash(f.manifest), 'started_ns': 40}
    write(directory/'pending.json', pending)
    original = dict(environment_for(f.spec)); additions = {'WORLDSIM_GATE_TEST_OUT': str(directory/'observer')}
    started: dict[str, Json] = {**pending, 'child_pid': 300, 'artifact_id': None, 'before_binary_sha256': file_hash(saved),
        'environment': {**original, **additions}, 'observer_additions': {k: v for k, v in additions.items()}}
    write(directory/'started.json', started)
    for name in ('stdout', 'stderr'): _ = (directory/(name+'.log')).write_bytes(b'')
    terminal: dict[str, Json] = {**started, 'native_exit': code, 'native_signal': -code if code < 0 else None,
        'received_signals': [], 'original_environment': {k: v for k, v in original.items()}, 'after_binary_sha256': file_hash(saved),
        'expected_inputs': read(f.manifest).field('inputs').value, 'after_inputs': read(f.manifest).field('inputs').value,
        'ended_ns': 30, 'transient': {**admission, 'identity_after': identity}, 'output_inventory': {k: v for k, v in snapshot(directory).items()},
        'errors': [], 'capture_status': 'COMPLETE', 'workspace_case_accounting': 'INCOMPLETE', 'doctests': 'INCOMPLETE', 'world_coverage': 'INCOMPLETE'}
    for name in ('stdout', 'stderr'): terminal[name+'_sha256'] = file_hash(directory/(name+'.log'))
    write(directory/'receipt.json', terminal)
    _ = (f.capture/('artifact-'+hashlib.sha256(str(endpoint).encode()).hexdigest()+'.claim')).write_text('3')
    return directory


class TransientCaptureTests(unittest.TestCase):
    def populated(self, code: int = 0) -> tuple[Fixture, Path]:
        f = policy_fixture(self)
        _ = policy_static(f, 1); _ = policy_static(f, 2)
        intent = exec_record(f)
        return f, transient_record(f, intent, code)

    def test_removed_endpoint_joins_without_live_probe_or_case_upgrade(self) -> None:
        from unittest.mock import patch

        f, _ = self.populated()
        with patch('gate_rustdoc_exec.admit_transient', side_effect=AssertionError('live admission forbidden')), \
             patch('gate_rustdoc_exec.postflight', side_effect=AssertionError('live postflight forbidden')):
            result = reconcile_capture(policy_plan(f), f.result)
        self.assertEqual(result.rejected, ())
        self.assertEqual((result.integrity, result.completeness), ('PASS', 'COMPLETE'))
        self.assertEqual(len(result.transients), 1)
        self.assertEqual(len(result.targets), 2)
        self.assertEqual((result.doctests, result.case_accounting, result.world_safety), ('INCOMPLETE',)*3)

    def test_transient_signal_retains_integrity(self) -> None:
        f, _ = self.populated(-15); f.parent(101)
        result = reconcile_capture(policy_plan(f), f.result)
        self.assertEqual((result.integrity, result.outcome), ('PASS', 'FAILED'))

    def test_missing_exec_is_not_zero_case(self) -> None:
        f = policy_fixture(self); _ = policy_static(f, 1); _ = policy_static(f, 2)
        _ = exec_record(f, probe=True)
        result = reconcile_capture(policy_plan(f), f.result)
        self.assertEqual(result.missing_exec_invocations, ('alpha-doc',))
        self.assertEqual(result.completeness, 'INCOMPLETE')

    def test_zero_transient_exec_remains_case_incomplete(self) -> None:
        f = policy_fixture(self); _ = policy_static(f, 1); _ = policy_static(f, 2); _ = exec_record(f)
        result = reconcile_capture(policy_plan(f), f.result)
        self.assertEqual(result.integrity, 'PASS')
        self.assertEqual(result.doctests, 'INCOMPLETE')

    def test_retained_transient_mutations_fail(self) -> None:
        for field in ('parent_pid', 'argv', 'artifact_id', 'native_signal', 'after_binary_sha256', 'cwd'):
            with self.subTest(field=field):
                f, directory = self.populated()
                row = read(directory/'receipt.json').mapping(); row[field] = 'changed'; write(directory/'receipt.json', row)
                result = reconcile_capture(policy_plan(f), f.result)
                self.assertEqual(result.integrity, 'FAIL')
                self.assertEqual(len(result.targets), 2)

    def test_mutated_copy_or_intent_or_probe_rejected(self) -> None:
        for name in ('transient-binary', 'exec-intent.json', 'live-parent.json'):
            with self.subTest(name=name):
                f, directory = self.populated()
                path = f.base/'exec/exec-one/exec-intent.json' if name == 'exec-intent.json' else directory/name
                _ = path.write_bytes(path.read_bytes()+b' ')
                result = reconcile_capture(policy_plan(f), f.result)
                self.assertEqual(result.integrity, 'FAIL')

    def test_probe_exec_failure_and_unknown_exec_file_retained(self) -> None:
        for name in ('exec-failure.json', 'foreign.json'):
            with self.subTest(name=name):
                f, _ = self.populated()
                path = exec_record(f, 'exec-probe', probe=True)
                write(path.parent/name, {'error': 'synthetic'})
                result = reconcile_capture(policy_plan(f), f.result)
                self.assertEqual(result.integrity, 'FAIL')
                self.assertTrue(any('exec-probe' in item for item in result.ledger))

    def test_static_only_exec_probe_is_frozen_and_rechecked(self) -> None:
        from unittest.mock import patch

        import gate_rustdoc_exec as wrapper

        f = policy_fixture(self); _ = policy_static(f, 1); _ = policy_static(f, 2)
        _ = exec_record(f); probe = exec_record(f, 'exec-probe', probe=True)
        plan = policy_plan(f)
        with patch.object(wrapper, 'closure_for', wraps=wrapper.closure_for) as closure:
            result = reconcile_capture(plan, f.result)
        self.assertEqual(result.integrity, 'PASS')
        self.assertEqual(closure.call_count, 1)
        _ = probe.write_bytes(probe.read_bytes()+b' ')
        self.assertEqual(reconcile_capture(plan, f.result).integrity, 'FAIL')

    def test_partial_transient_and_claim_only_remain_visible(self) -> None:
        f, directory = self.populated()
        (directory/'receipt.json').unlink()
        _ = (f.capture/'artifact-unknown.claim').write_text('4')
        result = reconcile_capture(policy_plan(f), f.result)
        self.assertEqual(result.integrity, 'FAIL')
        self.assertEqual(len(result.targets), 2)
        self.assertIn(str(directory), result.ledger)
        self.assertIn(str(f.capture/'artifact-unknown.claim'), result.ledger)

    def test_parent_failure_does_not_discard_raw_ledger(self) -> None:
        f, directory = self.populated()
        f.parent_data['pid'] = 999; write(f.result.receipt, f.parent_data)
        result = reconcile_capture(policy_plan(f), f.result)
        self.assertEqual(result.integrity, 'FAIL')
        self.assertIn(str(directory), result.ledger)
        self.assertTrue(result.declared_snapshot)

    def test_doctest_inventory_mismatch_is_not_admitted(self) -> None:
        f, _ = self.populated()
        plan = replace(policy_plan(f), inventory=Inventory(tuple(f.executables), ()))
        self.assertEqual(reconcile_capture(plan, f.result).integrity, 'FAIL')

    def test_resealed_transient_endpoint_and_parent_evidence_fail(self) -> None:
        for field in ('compiler_cwd', 'run_cwd', 'endpoint_limit', 'live_parent_pid', 'identity_before'):
            with self.subTest(field=field):
                f, directory = self.populated()
                admission = read(directory/'transient-admission.json').mapping()
                admission[field] = 'changed'
                write(directory/'transient-admission.json', admission)
                terminal = read(directory/'receipt.json').mapping()
                terminal['transient'] = {**admission, 'identity_after': admission['identity_before']}
                terminal['output_inventory'] = {k: v for k, v in snapshot(directory).items() if k != 'receipt.json'}
                write(directory/'receipt.json', terminal)
                self.assertEqual(reconcile_capture(policy_plan(f), f.result).integrity, 'FAIL')


class RetainedOwnerEnvironmentTests(unittest.TestCase):
    def test_foreign_owner_rejected(self) -> None:
        f = policy_fixture(self)
        _ = policy_static(f, 1); _ = policy_static(f, 2)
        directory = transient_record(f, exec_record(f))
        admission = read(directory/'transient-admission.json').mapping()
        identity = read(directory/'transient-admission.json').field('identity_before').mapping()
        identity['uid'] = os.getuid()+1
        admission['identity_before'] = identity
        write(directory/'transient-admission.json', admission)
        terminal = read(directory/'receipt.json').mapping()
        terminal['transient'] = {**admission, 'identity_after': identity}
        terminal['output_inventory'] = {k: v for k, v in snapshot(directory).items() if k != 'receipt.json'}
        write(directory/'receipt.json', terminal)
        self.assertEqual(reconcile_capture(policy_plan(f), f.result).integrity, 'FAIL')

    def test_child_environment_drift_from_exec_rejected(self) -> None:
        f = policy_fixture(self)
        _ = policy_static(f, 1); _ = policy_static(f, 2)
        directory = transient_record(f, exec_record(f))
        terminal = read(directory/'receipt.json').mapping()
        original = read(directory/'receipt.json').field('original_environment').mapping()
        original['UNBOUND_RUSTDOC_CHILD_DELTA'] = 'injected'
        environment = {**original, 'WORLDSIM_GATE_TEST_OUT': str(directory/'observer')}
        terminal['original_environment'] = original; terminal['environment'] = environment
        started = read(directory/'started.json').mapping(); started['environment'] = environment
        write(directory/'started.json', started)
        terminal['output_inventory'] = {k: v for k, v in snapshot(directory).items() if k != 'receipt.json'}
        write(directory/'receipt.json', terminal)
        self.assertEqual(reconcile_capture(policy_plan(f), f.result).integrity, 'FAIL')


    def test_child_environment_removal_or_value_drift_rejected(self) -> None:
        for remove in (True, False):
            with self.subTest(remove=remove):
                f = policy_fixture(self)
                _ = policy_static(f, 1); _ = policy_static(f, 2)
                directory = transient_record(f, exec_record(f))
                terminal = read(directory/'receipt.json').mapping()
                original = read(directory/'receipt.json').field('original_environment').mapping()
                if remove:
                    del original['PATH']
                else:
                    original['PATH'] = '/unbound-path'
                environment = {**original, 'WORLDSIM_GATE_TEST_OUT': str(directory/'observer')}
                terminal['original_environment'] = original; terminal['environment'] = environment
                started = read(directory/'started.json').mapping(); started['environment'] = environment
                write(directory/'started.json', started)
                terminal['output_inventory'] = {k: v for k, v in snapshot(directory).items() if k != 'receipt.json'}
                write(directory/'receipt.json', terminal)
                self.assertEqual(reconcile_capture(policy_plan(f), f.result).integrity, 'FAIL')
