"""Offline prefix and nested envelope boundaries; no Rust or Godot execution."""
import fcntl
import os
import shutil
import sys
import unittest
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import generic_cargo_child
import generic_integration_prefix_run as runner
from generic_gate import GateContractError
from generic_integration_accounting import PARENT, account36
from generic_library_run import (
    Library49Request,
    LibraryOwnershipUnknown,
    LibrarySettledFailure,
)
from generic_process import ProcessResult, ProcessSpec, atomic_json, execute, file_hash
from generic_workspace_capture import read
from generic_workspace_cases import ListingProcess, parse_listing, process_output
from generic_workspace_tests import Executable, Target
from mux2_gate_inputs import Json


class AccountingTests(unittest.TestCase):
    def test_exact_nine_nested_children(self) -> None:
        child = '\nrunning 1 test\ntest selected_pack_child ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.01s\n\n'
        outer = ('\nrunning 2 tests\ntest selected_pack_child ... ok\ntest '+PARENT+' ... '+child*9+
                 'ok\n\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s\n\n').encode()
        names = parse_listing(('selected_pack_child: test\n'+PARENT+': test\n\n2 tests, 0 benchmarks\n').encode())
        result = account36(outer, names, parse_listing(b'0 tests, 0 benchmarks\n'))
        self.assertEqual(len(result.nested_ranges), 9)
        self.assertEqual(len(result.section.outcomes), 2)

ROOT = Path(generic_cargo_child.__file__).resolve().parents[2]
CHILD = '\nrunning 1 test\ntest selected_pack_child ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.01s\n\n'
NESTED = ('\nrunning 2 tests\ntest selected_pack_child ... ok\ntest '+PARENT+' ... '+CHILD*9+
          'ok\n\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s\n\n')


class RuntimeTests(unittest.TestCase):
    def fixture(self, base: Path, step: int, *, fail: bool = False, omit: bool = False) -> Library49Request:
        name, profile, threads, argv = runner.recipe(step)
        root = base/'source'; target = root/'rust/crates/sim-test'; (target/'tests').mkdir(parents=True)
        (root/'tools/v8').mkdir(parents=True)
        _ = (target/'Cargo.toml').write_text('synthetic manifest')
        _ = (target/f'tests/{name}.rs').write_bytes((ROOT/f'rust/crates/sim-test/tests/{name}.rs').read_bytes())
        for script in ('gate.sh', 'data_only_ext.sh', 'godot_bin.sh'):
            _ = (root/'tools/v8'/script).write_bytes((ROOT/'tools/v8'/script).read_bytes())
        binary = base/'integration-test'
        listing = ('selected_pack_child: test\n'+PARENT+': test\n\n2 tests, 0 benchmarks\n') if step == 36 else 'mechanism: test\n\n1 test, 0 benchmarks\n'
        output = NESTED if step == 36 else ('\nrunning 1 test\ntest mechanism ... '+('FAILED' if fail else 'ok')+
            '\n\ntest result: '+('FAILED' if fail else 'ok')+f'. {0 if fail else 1} passed; {1 if fail else 0} failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n\n')
        native = f'''import sys
if '--list' in sys.argv:
 sys.stdout.write('0 tests, 0 benchmarks\\n' if '--ignored' in sys.argv else {listing!r})
else:
 assert sys.argv[1:]==[{f'--test-threads={threads}'!r}]
 sys.stdout.write({output!r})
 raise SystemExit({101 if fail else 0})
'''
        _ = binary.write_text('#!'+str(Path(sys.executable).resolve())+'\n'+native); binary.chmod(0o755)
        cargo = base/'actual-cargo'
        body = f'''import os,shlex,subprocess,sys
assert sys.argv[1:]=={list(argv[1:])!r}
assert os.getcwd()=={str(root/'rust')!r}
print('synthetic compiler diagnostic',file=sys.stderr)
runner=shlex.split(os.environ['CARGO_TARGET_TEST_HOST_RUNNER'])
raise SystemExit(subprocess.call([*runner,{str(binary)!r},{f'--test-threads={threads}'!r}],cwd={str(target)!r}))
'''
        if omit: body = 'raise SystemExit(0)\n'
        _ = cargo.write_text('#!'+str(Path(sys.executable).resolve())+'\n'+body); cargo.chmod(0o755)
        helper = Path(generic_cargo_child.__file__).resolve()
        tools = {Path('/bin/bash').resolve(), Path(sys.executable).resolve(), cargo, binary, *helper.parent.glob('*.py')}
        for command in ('mkdir', 'cat'):
            found = shutil.which(command)
            assert found is not None
            tools.add(Path(found).resolve())
        inputs = tuple((str(path.relative_to(root)), file_hash(path)) for path in sorted(root.rglob('*')) if path.is_file())
        descriptor = os.open(base/'lane.lock', os.O_RDWR | os.O_CREAT, 0o600)
        self.addCleanup(os.close, descriptor); fcntl.flock(descriptor, fcntl.LOCK_EX)
        return Library49Request(root, base/'result', f'synthetic{step}', 'candidate',
            Executable(Target('sim-test', 'sim-test', name, ('test',), target/f'tests/{name}.rs', target/'Cargo.toml', False, True),
                       binary, step, profile, threads=threads), cargo, Path('/bin/bash').resolve(), Path(sys.executable).resolve(), helper,
            'test-host', inputs, tuple(sorted((path, file_hash(path)) for path in tools)), (), descriptor)

    def test_both_actual_prefixes_and_independent_lists(self) -> None:
        for step in (28, 36):
            with self.subTest(step=step), TemporaryDirectory() as temporary:
                base = Path(temporary).resolve(); request = self.fixture(base, step)
                with patch.object(runner, 'LOCK', base/'lane.lock'):
                    result = (runner.run28_owned if step == 28 else runner.run36_owned)(request)
                self.assertEqual(result.status, 'PASS_PREFIX_ONLY')
                row = read(result.receipt)
                self.assertEqual(row.field('compound_tail').text(), 'NOT_RUN')
                self.assertIsNone(row.field('nested_pids').value)
                self.assertTrue((request.output/'ignored/process.json').is_file())
                if step == 36:
                    self.assertIn(b'synthetic compiler diagnostic', (request.output/'simulation.log').read_bytes())
                    self.assertNotEqual((request.output/'simulation.log').read_bytes(), (request.output/'capture/000002/stdout.log').read_bytes())

    def test_28_native_failure_is_retained(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve(); request = self.fixture(base, 28, fail=True)
            with patch.object(runner, 'LOCK', base/'lane.lock'):
                result = runner.run28_owned(request)
            self.assertEqual(result.status, 'FAIL')
            self.assertEqual(read(result.receipt).field('native_exit').value, 101)

    def test_36_failure_and_missing_chain_retain_actual_fd(self) -> None:
        for step, fail, omit in ((36, True, False), (28, False, True)):
            with self.subTest(step=step), TemporaryDirectory() as temporary:
                base = Path(temporary).resolve(); request = self.fixture(base, step, fail=fail, omit=omit)
                with patch.object(runner, 'LOCK', base/'lane.lock'), self.assertRaises(LibraryOwnershipUnknown) as caught:
                    _ = runner.run_owned(request, step)
                self.assertIs(caught.exception.request, request)
                self.assertGreater(os.fstat(request.held_lane_fd).st_ino, 0)
                self.assertFalse((request.output/'result.json').exists())

    def test_wrong_target_profile_threads_and_environment_fail_prelaunch(self) -> None:
        for mutation in ('kind', 'threads', 'profile', 'filter', 'environment'):
            with self.subTest(mutation=mutation), TemporaryDirectory() as temporary:
                base = Path(temporary).resolve(); request = self.fixture(base, 36)
                item = request.executable
                if mutation == 'kind': item = replace(item, target=replace(item.target, kinds=('lib',)))
                if mutation == 'threads': item = replace(item, threads=8)
                if mutation == 'profile': item = replace(item, profile='release')
                if mutation == 'filter': item = replace(item, filter='selected_pack_child')
                request = replace(request, executable=item, environment=(('WORLDSIM_DATA_ONLY_CASE', 'base_before'),) if mutation == 'environment' else ())
                with patch.object(runner, 'LOCK', base/'lane.lock'), self.assertRaises(GateContractError):
                    _ = runner.run36_owned(request)
                self.assertFalse(request.output.exists())

    def test_source_topology_drift_rejects_even_when_rehashed(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve(); request = self.fixture(base, 36)
            source = request.executable.target.source
            _ = source.write_text(source.read_text().replace('child.status()', 'child.spawn()'))
            request = replace(request, inputs=tuple((name, file_hash(request.root/name)) for name, _sha in request.inputs))
            with patch.object(runner, 'LOCK', base/'lane.lock'), self.assertRaises(GateContractError):
                _ = runner.run36_owned(request)

    def test_prior_empty_directory_mutation_rejected(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve(); request = self.fixture(base, 28)
            def mutate(spec: ProcessSpec) -> ProcessResult:
                result = execute(spec)
                if spec.output_root.name == 'ignored': (request.output/'capture/late-empty').mkdir()
                return result
            with patch.object(runner, 'LOCK', base/'lane.lock'), patch.object(runner, 'execute', mutate), self.assertRaises(LibrarySettledFailure):
                _ = runner.run28_owned(request)

    def test_signal_or_postwait_exception_is_unknown(self) -> None:
        for signal in (False, True):
            with self.subTest(signal=signal), TemporaryDirectory() as temporary:
                base = Path(temporary).resolve(); request = self.fixture(base, 28)
                if signal:
                    binary = request.executable.binary
                    _ = binary.write_text(binary.read_text().replace('import sys', 'import sys, os, signal\nif "--list" not in sys.argv: os.kill(os.getpid(), signal.SIGTERM)'))
                    request = replace(request, tools=tuple((path, file_hash(path)) for path, _sha in request.tools))
                def interrupt(spec: ProcessSpec, signal: bool = signal) -> ProcessResult:
                    result = execute(spec)
                    if not signal: raise KeyboardInterrupt
                    return result
                with patch.object(runner, 'LOCK', base/'lane.lock'), patch.object(runner, 'execute', interrupt), self.assertRaises(LibraryOwnershipUnknown):
                    _ = runner.run28_owned(request)
                with (base/'lane.lock').open('rb') as competing, self.assertRaises(BlockingIOError):
                    fcntl.flock(competing, fcntl.LOCK_EX | fcntl.LOCK_NB)
                self.assertFalse((request.output/'result.json').exists())

    def test_changed_native_parent_rejects(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve(); request = self.fixture(base, 36)
            def corrupt(spec: ProcessSpec) -> ProcessResult:
                result = execute(spec)
                from generic_process import atomic_json
                path = request.output/'capture/000002/pending.json'
                row = read(path).mapping(); row['parent_pid'] = 1
                atomic_json(path, row)
                return result
            with patch.object(runner, 'LOCK', base/'lane.lock'), patch.object(runner, 'execute', corrupt), self.assertRaises(LibraryOwnershipUnknown):
                _ = runner.run36_owned(request)

    def test_completed_outputs_reject_later_files_directories_and_publication_mutations(self) -> None:
        for stage in ('ignored_execution', 'listing_authentication', 'final_authentication', 'publication'):
            for directory in (False, True):
                with self.subTest(stage=stage, directory=directory), TemporaryDirectory() as temporary:
                    self.closure_case(Path(temporary).resolve(), stage, directory)

    def closure_case(self, base: Path, stage: str, directory: bool) -> None:
        request = self.fixture(base, 28)
        original_auth = process_output
        def mutate(stage: str = stage, directory: bool = directory) -> None:
            target = request.output/('full' if stage == 'ignored_execution' else 'ignored' if stage == 'listing_authentication' else 'capture')/'late-member'
            if directory: target.mkdir()
            else: _ = target.write_text('late output')
        def execution(spec: ProcessSpec, stage: str = stage) -> ProcessResult:
            result = execute(spec)
            if stage == 'ignored_execution' and spec.output_root.name == 'ignored': mutate()
            return result
        def authentication(binding: ListingProcess, argv: tuple[str, ...], cwd: Path,
                           environment: dict[str, str], required: dict[str, str], stage: str = stage) -> bytes:
            result = original_auth(binding, argv, cwd, environment, required)
            if (stage == 'final_authentication' and binding.spec.output_root.name == 'outer' and (request.output/'ignored').exists()
                    or stage == 'listing_authentication' and binding.spec.output_root.name == 'ignored'):
                mutate()
            return result
        def publication(path: Path, value: dict[str, Json], stage: str = stage) -> None:
            atomic_json(path, value)
            if stage == 'publication' and path.name == 'result.json': mutate()
        with patch.object(runner, 'LOCK', base/'lane.lock'), patch.object(runner, 'execute', execution), patch.object(runner, 'process_output', authentication), patch.object(runner, 'atomic_json', publication), self.assertRaises(LibrarySettledFailure):
            _ = runner.run28_owned(request)
        self.assertGreater(os.fstat(request.held_lane_fd).st_ino, 0)

    def test_zero_case_listing_rejects(self) -> None:
        with TemporaryDirectory() as temporary:
            base = Path(temporary).resolve(); request = self.fixture(base, 28)
            binary = request.executable.binary
            _ = binary.write_text(binary.read_text().replace('mechanism: test\\n\\n1 test, 0 benchmarks\\n', '0 tests, 0 benchmarks\\n'))
            request = replace(request, tools=tuple((path, file_hash(path)) for path, _sha in request.tools))
            with patch.object(runner, 'LOCK', base/'lane.lock'), self.assertRaises(LibrarySettledFailure):
                _ = runner.run28_owned(request)


class NestedNegativeTests(unittest.TestCase):
    def test_missing_extra_failed_and_truncated_child_envelopes_reject(self) -> None:
        full = parse_listing(('selected_pack_child: test\n'+PARENT+': test\n\n2 tests, 0 benchmarks\n').encode())
        ignored = parse_listing(b'0 tests, 0 benchmarks\n')
        for raw in (NESTED.replace(CHILD, '', 1), NESTED.replace(CHILD, CHILD*2, 1), NESTED.replace('1 passed; 0 failed', '0 passed; 1 failed', 1), NESTED[:-1], NESTED+'extra\n'):
            with self.subTest(raw=raw[-100:]), self.assertRaises(GateContractError):
                _ = account36(raw.encode(), full, ignored)
