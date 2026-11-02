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
from generic_workspace_cases import ListingProcess, process_output
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

    def test_explicit_empty_wrappers_survive_all_actual_receipts(self) -> None:
        for supplied in ((('RUSTC_WRAPPER',''),), (('RUSTC_WORKSPACE_WRAPPER',''),),
                         (('RUSTC_WRAPPER',''),('RUSTC_WORKSPACE_WRAPPER',''))):
            with self.subTest(supplied=supplied), TemporaryDirectory() as temporary:
                base=Path(temporary).resolve();request=replace(self.fixture(base),environment=supplied)
                with patch.object(runner,'LOCK',base/'lane.lock'):
                    result=runner.run49_owned(request)
                self.assertEqual(result.status,'PASS')
                for relative in ('outer/process-start.json','outer/process.json',
                                 'capture/000001/started.json','capture/000001/receipt.json',
                                 'capture/000002/started.json','capture/000002/receipt.json',
                                 'full/process.json','ignored/process.json'):
                    row=read(request.output/relative)
                    for field in ('environment','original_environment') if 'original_environment' in row.mapping() else ('environment',):
                        environment=row.field(field).mapping()
                        for name in ('RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER'):
                            if name in dict(supplied):self.assertEqual(environment[name],'')
                            else:self.assertNotIn(name,environment)

    def test_nonempty_wrapper_injection_rejected_before_launch(self) -> None:
        for name in ('RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER'):
            for value in ('/untrusted/wrapper',' '):
                with self.subTest(name=name,value=value), TemporaryDirectory() as temporary:
                    base=Path(temporary).resolve();request=replace(self.fixture(base),environment=((name,value),))
                    with patch.object(runner,'LOCK',base/'lane.lock'),self.assertRaises(GateContractError):
                        _=runner.run49_owned(request)
                    self.assertFalse(request.output.exists())

    def test_absent_wrappers_remain_absent_despite_ambient_values(self) -> None:
        with TemporaryDirectory() as temporary:
            base=Path(temporary).resolve();request=self.fixture(base)
            with patch.object(runner,'LOCK',base/'lane.lock'),patch.dict(os.environ,{'RUSTC_WRAPPER':'/ambient/wrapper','RUSTC_WORKSPACE_WRAPPER':''}):
                result=runner.run49_owned(request)
            self.assertEqual(result.status,'PASS')
            for relative in ('outer/process.json','capture/000001/receipt.json','capture/000002/receipt.json'):
                environment=read(request.output/relative).field('environment').mapping()
                self.assertNotIn('RUSTC_WRAPPER',environment);self.assertNotIn('RUSTC_WORKSPACE_WRAPPER',environment)


class SiblingRuntimeTests(unittest.TestCase):
    def fixture(self, base: Path, step: int, *, fail: bool = False) -> runner.Library49Request:
        from generic_library_siblings import LibraryStep, prepare, recipe
        fixture=RuntimeTests();self.addCleanup(fixture.doCleanups)
        request=fixture.fixture(base);selected=recipe(LibraryStep(step))
        target=request.root/'rust/crates/sim-systems';(target/'src/runtime/baseline').mkdir(parents=True)
        _=(target/'Cargo.toml').write_text('synthetic systems manifest')
        _=(target/'src/lib.rs').write_text('synthetic systems source')
        shell=request.root/'tools/v8/core_gate_checks.sh';shell.parent.mkdir(parents=True)
        _=shell.write_bytes((request.helper.parent/'core_gate_checks.sh').read_bytes())
        names=selected.expected;count=len(names)
        body=f'''import sys
names={names!r}
if '--list' in sys.argv:
 if '--ignored' in sys.argv: print('0 tests, 0 benchmarks')
 else: print(''.join(n+': test\\n' for n in names)+'other: test\\n\\n{count+1} tests, 0 benchmarks')
else:
 assert sys.argv[1:]=={[selected.filter,'--nocapture' if selected.nocapture else '--exact']!r}
 print('running {count} {"test" if count==1 else "tests"}')
 for name in names: print('test '+name+' ... {"FAILED" if fail else "ok"}')
 print('\\ntest result: {"FAILED" if fail else "ok"}. {0 if fail else count} passed; {count if fail else 0} failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s\\n')
 raise SystemExit({101 if fail else 0})
'''
        _=request.executable.binary.write_text('#!'+str(request.python)+'\n'+body)
        command=prepare(LibraryStep(step),request.root,request.output/'original.log')
        cargo=f'''import os,shlex,subprocess,sys
assert sys.argv[1:]=={list(command.cargo_argv[1:])!r}
print('synthetic Cargo diagnostic',file=sys.stderr,flush=True)
runner=shlex.split(os.environ['CARGO_TARGET_TEST_HOST_RUNNER'])
raise SystemExit(subprocess.call([*runner,{str(request.executable.binary)!r},{selected.filter!r},{'--nocapture' if selected.nocapture else '--exact'!r}],cwd={str(target)!r}))
'''
        _=request.cargo.write_text('#!'+str(request.python)+'\n'+cargo)
        tools=dict(request.tools)
        for path in (request.cargo,request.executable.binary,Path('/usr/bin/dirname'),Path('/usr/bin/tee'),Path('/usr/bin/grep')):
            tools[path.resolve()]=file_hash(path.resolve())
        inputs=(*request.inputs,*((str(path.relative_to(request.root)),file_hash(path)) for path in (shell,target/'Cargo.toml',target/'src/lib.rs')))
        return replace(request,invocation=f'synthetic{step}',executable=Executable(
            Target('sim-systems','sim-systems','sim_systems',('lib',),target/'src/lib.rs',target/'Cargo.toml',False,True),
            request.executable.binary,step,'debug',selected.filter),inputs=inputs,tools=tuple(sorted(tools.items())),
            environment=(('PATH',str(Path(sys.executable).parent)+':/usr/bin:/bin'),))

    def test_closed42_and43_execute_original_shell_and_actual_selected_cases(self) -> None:
        for step,count in ((42,4),(43,1)):
            with self.subTest(step=step),TemporaryDirectory() as temporary:
                base=Path(temporary).resolve();request=self.fixture(base,step)
                with patch.object(runner,'LOCK',base/'lane.lock'):
                    result=runner.run42_owned(request) if step==42 else runner.run43_owned(request)
                self.assertEqual(result.status,'PASS');self.assertEqual(len(result.accounting.selected),count)
                self.assertEqual(result.accounting.filtered_out,1)
                self.assertEqual(result.accounting.branch_observation,'NOT_RECORDED')
                receipt=read(result.receipt);self.assertEqual(receipt.field('step').value,step)
                self.assertEqual(receipt.field('source_build_admission').text(),'PENDING_CALLER_HANDOFF')
                self.assertIn(b'synthetic Cargo diagnostic',(request.output/'original.log').read_bytes())
                if step==42:self.assertTrue((request.output/'outer/stdout.log').read_bytes().startswith(b'BASELINE_ISOLATION source PASS\n'))

    def test_native_failure_is_not_hidden_by_tee(self) -> None:
        for step in (42,43):
            with self.subTest(step=step),TemporaryDirectory() as temporary:
                base=Path(temporary).resolve();request=self.fixture(base,step,fail=True)
                with patch.object(runner,'LOCK',base/'lane.lock'):
                    result=runner.run42_owned(request) if step==42 else runner.run43_owned(request)
                self.assertEqual(result.status,'FAIL')
                self.assertEqual(read(result.receipt).field('native_exit').value,101)
                self.assertEqual(read(result.receipt).field('outer_exit').value,101)

    def test_wrapper_failure_preserves_native_success(self) -> None:
        with TemporaryDirectory() as temporary:
            base=Path(temporary).resolve();request=self.fixture(base,43)
            body=request.cargo.read_text().replace('raise SystemExit(subprocess.call(', 'code=subprocess.call(')
            body=body.replace('))\n',')\nraise SystemExit(7)\n')
            _=request.cargo.write_text(body)
            request=replace(request,tools=tuple((path,file_hash(path)) for path,_ in request.tools))
            with patch.object(runner,'LOCK',base/'lane.lock'):
                result=runner.run43_owned(request)
            self.assertEqual(result.status,'FAIL')
            self.assertEqual(result.accounting.accounting_status,'PASS')
            self.assertEqual(read(result.receipt).field('native_exit').value,0)
            self.assertEqual(read(result.receipt).field('outer_exit').value,7)

    def test_script_or_shell_tool_missing_binding_rejects_before_launch(self) -> None:
        for missing in ('script','tee'):
            with self.subTest(missing=missing),TemporaryDirectory() as temporary:
                base=Path(temporary).resolve();request=self.fixture(base,42)
                if missing=='script':request=replace(request,inputs=tuple((name,digest) for name,digest in request.inputs if name!='tools/v8/core_gate_checks.sh'))
                else:request=replace(request,tools=tuple((path,digest) for path,digest in request.tools if path!=Path('/usr/bin/tee')))
                with patch.object(runner,'LOCK',base/'lane.lock'),self.assertRaises(GateContractError):
                    _=runner.run42_owned(request)
                self.assertFalse(request.output.exists())

    def test_failed_source_scan_does_not_fabricate_native_completion(self) -> None:
        with TemporaryDirectory() as temporary:
            base=Path(temporary).resolve();request=self.fixture(base,42)
            (request.root/'rust/crates/sim-systems/src/runtime/decision').mkdir()
            with patch.object(runner,'LOCK',base/'lane.lock'),self.assertRaises(runner.LibraryOwnershipUnknown) as caught:
                _=runner.run42_owned(request)
            self.assertIs(caught.exception.request,request)
            self.assertFalse((request.output/'result.json').exists())
            self.assertFalse((request.output/'capture/000001').exists())

    def test_source_scanner_new_input_during_listings_is_rejected(self) -> None:
        with TemporaryDirectory() as temporary:
            base=Path(temporary).resolve();request=self.fixture(base,42)
            actual=execute
            def added(spec: ProcessSpec) -> ProcessResult:
                result=actual(spec)
                if spec.output_root.name=='ignored':
                    _=(request.root/'rust/crates/sim-systems/src/runtime/added.rs').write_text('// added')
                return result
            with patch.object(runner,'LOCK',base/'lane.lock'),patch.object(runner,'execute',added),self.assertRaises(runner.LibrarySettledFailure):
                _=runner.run42_owned(request)

    def test_scanner_path_predicates_are_rechecked_after_each_child(self) -> None:
        for phase in ('outer','full','ignored'):
            for mutation in ('decision','baseline','seam'):
                with self.subTest(phase=phase,mutation=mutation),TemporaryDirectory() as temporary:
                    base=Path(temporary).resolve();request=self.fixture(base,42)
                    def changed(spec: ProcessSpec, phase: str = phase,
                                request: runner.Library49Request = request,
                                mutation: str = mutation) -> ProcessResult:
                        result=execute(spec)
                        if spec.output_root.name==phase:
                            runtime=request.root/'rust/crates/sim-systems/src/runtime'
                            if mutation=='decision':(runtime/'decision').mkdir()
                            elif mutation=='baseline':(runtime/'baseline').rmdir()
                            else:
                                (runtime/'jobs').mkdir(exist_ok=True)
                                (runtime/'jobs/seam.rs').mkdir()
                        return result
                    with patch.object(runner,'LOCK',base/'lane.lock'),patch.object(runner,'execute',changed),self.assertRaises(runner.LibrarySettledFailure):
                        _=runner.run42_owned(request)
                    self.assertFalse((request.output/'result.json').exists())

    def test_scanner_rechecked_after_final_parent_authentication_callback(self) -> None:
        with TemporaryDirectory() as temporary:
            base=Path(temporary).resolve();request=self.fixture(base,42)
            calls=0
            def changed(binding: ListingProcess, argv: tuple[str,...], cwd: Path,
                        environment: dict[str,str], inputs: dict[str,str]) -> bytes:
                nonlocal calls
                result=process_output(binding,argv,cwd,environment,inputs)
                if binding.spec.output_root.name=='outer':
                    calls+=1
                    if calls==2:(request.root/'rust/crates/sim-systems/src/runtime/decision').mkdir()
                return result
            with patch.object(runner,'LOCK',base/'lane.lock'),patch.object(runner,'process_output',changed),self.assertRaises(runner.LibrarySettledFailure):
                _=runner.run42_owned(request)


if __name__ == '__main__':
    _ = unittest.main()
