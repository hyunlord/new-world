"""Static synthetic receipts only; no Cargo/build/native execution claims."""
import hashlib
import json
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
