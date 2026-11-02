"""Tiny Python/shell fixtures exercise the capture chain; no Cargo or WorldSim runs."""
import fcntl
import os
import sys
import unittest
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import override
from unittest.mock import patch

import generic_cargo_child
import generic_library_run as runner
from generic_gate import GateContractError
from generic_library_siblings import DIAGNOSIS
from generic_process import ProcessResult, ProcessSpec, execute, file_hash
from generic_workspace_capture import read
from generic_workspace_tests import Executable, Target


class RuntimeTests(unittest.TestCase):
    def fixture(self, base: Path, *, fail: bool = False, omit: bool = False) -> runner.Library49Request:
        root = base/'source'; target = root/'rust/crates/sim-core'; (target/'src').mkdir(parents=True)
        _ = (target/'Cargo.toml').write_text('synthetic manifest')
        _ = (target/'src/lib.rs').write_text('synthetic source')
        binary = base/'libtest'
        native = f'''import sys
name={DIAGNOSIS!r}
if '--list' in sys.argv:
 if '--ignored' in sys.argv: print('0 tests, 0 benchmarks')
 else: print(name+': test\\nother: test\\n\\n2 tests, 0 benchmarks')
else:
 print('running 1 test\\ntest '+name+' ... {"FAILED" if fail else "ok"}\\n\\ntest result: {"FAILED" if fail else "ok"}. {0 if fail else 1} passed; {1 if fail else 0} failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s\\n')
 raise SystemExit({101 if fail else 0})
'''
        _ = binary.write_text('#!'+str(Path(sys.executable).resolve())+'\n'+native); binary.chmod(0o755)
        cargo = base/'actual-cargo'
        body = f'''import os,shlex,subprocess,sys
runner=shlex.split(os.environ['CARGO_TARGET_TEST_HOST_RUNNER'])
raise SystemExit(subprocess.call([*runner,{str(binary)!r},{DIAGNOSIS!r},'--exact'],cwd={str(target)!r}))
'''
        if omit: body = 'raise SystemExit(0)\n'
        _ = cargo.write_text('#!'+str(Path(sys.executable).resolve())+'\n'+body); cargo.chmod(0o755)
        helper = Path(generic_cargo_child.__file__).resolve()
        tools = {Path('/bin/bash').resolve(), Path(sys.executable).resolve(), cargo, binary, *helper.parent.glob('*.py')}
        inputs = tuple((str(path.relative_to(root)), file_hash(path)) for path in (target/'Cargo.toml', target/'src/lib.rs'))
        descriptor = os.open(base/'lane.lock', os.O_RDWR | os.O_CREAT, 0o600)
        self.addCleanup(os.close, descriptor)
        fcntl.flock(descriptor, fcntl.LOCK_EX)
        return runner.Library49Request(root, base/'result', 'synthetic49', 'candidate',
            Executable(Target('sim-core', 'sim-core', 'sim_core', ('lib',), target/'src/lib.rs', target/'Cargo.toml', False, True),
                       binary, 49, 'release', DIAGNOSIS), cargo, Path('/bin/bash').resolve(), Path(sys.executable).resolve(), helper,
            'test-host', inputs, tuple(sorted((path, file_hash(path)) for path in tools)), (), descriptor)

    def test_original_bash_to_cargo_to_native_and_independent_listings(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve(); request = self.fixture(base)
            with patch.object(runner, 'LOCK', base/'lane.lock'):
                result = runner.run49_owned(request)
            self.assertEqual(result.status, 'PASS')
            receipt = read(result.receipt)
            self.assertEqual(receipt.field('observer_qualification').text(), 'PENDING_ORIGINAL_DIRECT_CONTROL')
            self.assertNotEqual(receipt.field('cargo_pid').value, receipt.field('native_pid').value)
            self.assertEqual(result.accounting.filtered_out, 1)
            self.assertTrue((request.output/'full/process.json').is_file())
            self.assertTrue((request.output/'ignored/process.json').is_file())

    def test_native_failure_is_settled_fail(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve(); request = self.fixture(base, fail=True)
            with patch.object(runner, 'LOCK', base/'lane.lock'):
                result = runner.run49_owned(request)
            self.assertEqual(result.status, 'FAIL')
            self.assertEqual(read(result.receipt).field('native_exit').value, 101)

    def test_missing_native_join_retains_actual_owner_unknown(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve(); request = self.fixture(base, omit=True)
            with patch.object(runner, 'LOCK', base/'lane.lock'), self.assertRaises(runner.LibraryOwnershipUnknown) as caught:
                _ = runner.run49_owned(request)
            self.assertIs(caught.exception.request, request)
            self.assertGreater(os.fstat(request.held_lane_fd).st_ino, 0)
            self.assertFalse((request.output/'result.json').exists())

    def test_shell_startup_injection_rejected_before_launch(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve(); request = self.fixture(base)
            with patch.object(runner, 'LOCK', base/'lane.lock'), self.assertRaises(GateContractError):
                _ = runner.run49_owned(replace(request, environment=(('BASH_ENV', '/untrusted'),)))
            self.assertFalse(request.output.exists())

    def test_step42_remains_unsupported(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve(); request = self.fixture(base)
            with patch.object(runner, 'LOCK', base/'lane.lock'), self.assertRaises(GateContractError):
                _ = runner.run49_owned(replace(request, executable=replace(request.executable, step=42)))
            self.assertFalse(request.output.exists())

    def test_stale_output_rejected_before_launch(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve(); request = self.fixture(base); request.output.mkdir()
            with patch.object(runner, 'LOCK', base/'lane.lock'), self.assertRaises(GateContractError):
                _ = runner.run49_owned(request)
            self.assertEqual(list(request.output.iterdir()), [])

    def test_changed_source_rejected_before_launch(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve(); request = self.fixture(base)
            _ = request.executable.target.source.write_text('changed')
            with patch.object(runner, 'LOCK', base/'lane.lock'), self.assertRaises(GateContractError):
                _ = runner.run49_owned(request)
            self.assertFalse(request.output.exists())

    def test_postlaunch_exception_without_printable_message_retains_unknown(self) -> None:
        class Unprintable(RuntimeError):
            @override
            def __str__(self) -> str:
                raise TypeError('diagnostic unsafe')
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve(); request = self.fixture(base)
            original = execute
            def fail_after_wait(spec: ProcessSpec) -> ProcessResult:
                _ = original(spec)
                raise Unprintable
            with patch.object(runner, 'LOCK', base/'lane.lock'), patch.object(runner, 'execute', fail_after_wait), self.assertRaises(runner.LibraryOwnershipUnknown) as caught:
                _ = runner.run49_owned(request)
            self.assertIsInstance(caught.exception.original, Unprintable)
            self.assertFalse((request.output/'result.json').exists())

    def test_prior_output_mutation_during_listing_rejects_after_settlement(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve(); request = self.fixture(base)
            original = execute
            def mutate(spec: ProcessSpec) -> ProcessResult:
                result = original(spec)
                if spec.output_root.name == 'ignored':
                    with (request.output/'outer/stdout.log').open('ab') as stream:
                        _ = stream.write(b'changed')
                return result
            with patch.object(runner, 'LOCK', base/'lane.lock'), patch.object(runner, 'execute', mutate), self.assertRaises(runner.LibrarySettledFailure):
                _ = runner.run49_owned(request)
            self.assertFalse((request.output/'result.json').exists())

    def test_wrong_native_parent_join_retains_unknown(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve(); request = self.fixture(base)
            original = execute
            def corrupt(spec: ProcessSpec) -> ProcessResult:
                import json
                result = original(spec)
                path = request.output/'capture/000002/pending.json'
                data = read(path).mapping(); data['parent_pid'] = 1
                _ = path.write_text(json.dumps(data))
                return result
            with patch.object(runner, 'LOCK', base/'lane.lock'), patch.object(runner, 'execute', corrupt), self.assertRaises(runner.LibraryOwnershipUnknown):
                _ = runner.run49_owned(request)
            self.assertFalse((request.output/'result.json').exists())

    def test_zero_selected_tests_cannot_pass_from_native_exit_zero(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve(); request = self.fixture(base)
            binary = request.executable.binary
            source = binary.read_text()
            source = source.replace("print('running 1 test", "print('running 0 tests")
            source = source.replace("1 passed; 0 failed", "0 passed; 0 failed")
            _ = binary.write_text(source)
            request = replace(request, tools=tuple((path, file_hash(path) if path == binary else digest) for path, digest in request.tools))
            with patch.object(runner, 'LOCK', base/'lane.lock'), self.assertRaises(runner.LibrarySettledFailure):
                _ = runner.run49_owned(request)
            self.assertFalse((request.output/'result.json').exists())

    def test_added_capture_members_during_listing_rejected(self) -> None:
        for member in ('capture/000003/pending.json', 'capture/artifact-extra.claim', 'outer/extra.json', 'capture/000003/'):
            with self.subTest(member=member), TemporaryDirectory() as temporary:
                base = Path(temporary).resolve(); request = self.fixture(base)
                def add_member(spec: ProcessSpec, request: runner.Library49Request = request, member: str = member) -> ProcessResult:
                    result = execute(spec)
                    if spec.output_root.name == 'ignored':
                        path = request.output/member
                        path.parent.mkdir(parents=True, exist_ok=True)
                        if member.endswith('/'):
                            path.mkdir()
                        else:
                            _ = path.write_text('{}')
                    return result
                with patch.object(runner, 'LOCK', base/'lane.lock'), patch.object(runner, 'execute', add_member), self.assertRaises(runner.LibrarySettledFailure):
                    _ = runner.run49_owned(request)
                self.assertFalse((request.output/'result.json').exists())


if __name__ == '__main__':
    _ = unittest.main()
