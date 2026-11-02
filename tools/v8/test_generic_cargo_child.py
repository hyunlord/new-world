"""Synthetic child processes only, not Cargo/toolchain qualification."""
import hashlib
import json
import os
import signal
import subprocess
import sys
import threading
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import generic_cargo_child as capture
from generic_gate import document
from generic_process import atomic_json, file_hash
from mux2_gate_inputs import Json


class CaptureTests(unittest.TestCase):
    def make(self, root: Path, body: str) -> tuple[Path,dict[str,str]]:
        work=root/'work';work.mkdir();binary=work/'child.py';_=binary.write_text('#!'+sys.executable+'\n'+body);binary.chmod(0o755)
        output=root/'receipts';output.mkdir()
        source=root/'source.txt';_=source.write_text('immutable')
        manifest: dict[str,Json]={'schema_version':1,'invocation_id':'synthetic-cargo','step':4,'arm':'candidate','profile':'debug',
            'output_root':str(output),'launcher_sha256':file_hash(Path(capture.__file__).resolve()),
            'python_sha256':file_hash(Path(sys.executable).resolve()),'inputs':{str(source):file_hash(source)},
            'artifacts':[{'id':'test-one','path':str(binary),'sha256':file_hash(binary)}],'observer_variable':None}
        path=root/'manifest.json';_=path.write_text(json.dumps(manifest))
        env=dict(os.environ,CARGO_CAPTURE_MANIFEST=str(path),CARGO_CAPTURE_MANIFEST_SHA256=file_hash(path),
                 PYTHONPATH='/Users/rexxa/work/new-world-m6b-dev/tools/v8',UNUSUAL_TEST_VALUE='a b\n한글',LD_LIBRARY_PATH='/actual cargo path')
        return binary,env

    def launch(self,binary: Path,env: dict[str,str],args: tuple[str,...]=()) -> subprocess.CompletedProcess[bytes]:
        return subprocess.run([sys.executable,capture.__file__,str(binary),*args],cwd=binary.parent,env=env,
                              input=b'original stdin',capture_output=True,check=False)

    def receipt(self, root: Path) -> str:
        return (root/'receipts/000001/receipt.json').read_text()

    def test_exact_bytes_environment_cwd_and_pid(self) -> None:
        with TemporaryDirectory() as folder:
            root=Path(folder).resolve();binary,env=self.make(root,"import os,sys\nos.write(1,sys.stdin.buffer.read()+b'\\x00\\xff')\nos.write(2,b'err\\x00\\xff')\n")
            result=self.launch(binary,env,('space arg','--test-threads=8'))
            self.assertEqual((result.returncode,result.stdout,result.stderr),(0,b'original stdin\x00\xff',b'err\x00\xff'))
            receipt=document(self.receipt(root));receipt.field('argv').equals([str(binary),'space arg','--test-threads=8'])
            receipt.field('environment').field('UNUSUAL_TEST_VALUE').equals('a b\n한글')
            receipt.field('environment').field('LD_LIBRARY_PATH').equals('/actual cargo path')
            receipt.field('cwd').equals(str(binary.parent));receipt.field('parent_pid').equals(os.getpid())
            receipt.field('workspace_case_accounting').equals('INCOMPLETE')

    def test_large_both_streams_not_truncated(self) -> None:
        with TemporaryDirectory() as folder:
            root=Path(folder).resolve();binary,env=self.make(root,"import os\nfor _ in range(32):\n os.write(1,b'a'*65536)\n os.write(2,b'b'*65536)\n")
            result=self.launch(binary,env)
            self.assertEqual((len(result.stdout),len(result.stderr)),(2097152,2097152))
            receipt=document(self.receipt(root));receipt.field('stdout_sha256').equals(hashlib.sha256(result.stdout).hexdigest())

    def test_self_signal_is_retained_and_propagated(self) -> None:
        with TemporaryDirectory() as folder:
            root=Path(folder).resolve();binary,env=self.make(root,'import os,signal\nos.kill(os.getpid(),signal.SIGTERM)\n')
            result=self.launch(binary,env)
            self.assertEqual(result.returncode,-signal.SIGTERM)
            document(self.receipt(root)).field('native_exit').equals(-signal.SIGTERM)

    def test_duplicate_and_changed_binary_are_rejected(self) -> None:
        for mutation in (False,True):
            with self.subTest(mutation=mutation),TemporaryDirectory() as folder:
                root=Path(folder).resolve();binary,env=self.make(root,'raise SystemExit(7)\n')
                self.assertEqual(self.launch(binary,env).returncode,7)
                if mutation:_=binary.write_text('#!'+sys.executable+'\nraise SystemExit(0)\n')
                self.assertEqual(self.launch(binary,env).returncode,125)
                self.assertTrue((root/'receipts/000002/receipt.json').is_file())

    def test_launcher_signal_is_forwarded_and_retained(self) -> None:
        with TemporaryDirectory() as folder:
            root=Path(folder).resolve();binary,env=self.make(root,"import os,signal\nos.write(1,b'READY\\n')\nsignal.pause()\n")
            with subprocess.Popen([sys.executable,capture.__file__,str(binary)],cwd=binary.parent,env=env,
                                  stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE) as launcher:
                assert launcher.stdout is not None
                self.assertEqual(launcher.stdout.readline(),b'READY\n')
                launcher.send_signal(signal.SIGTERM)
                _,stderr=launcher.communicate(timeout=5)
                self.assertEqual((launcher.returncode,stderr),(-signal.SIGTERM,b''))
            receipt=document(self.receipt(root));receipt.field('received_signals').equals([signal.SIGTERM])
            receipt.field('native_exit').equals(-signal.SIGTERM)

    def test_source_mutation_fails_capture_without_changing_native_exit(self) -> None:
        with TemporaryDirectory() as folder:
            root=Path(folder).resolve();binary,env=self.make(root,"from pathlib import Path\nPath(__file__).resolve().parents[1].joinpath('source.txt').write_text('changed')\n")
            result=self.launch(binary,env);self.assertEqual(result.returncode,125)
            receipt=document(self.receipt(root));receipt.field('native_exit').equals(0)
            receipt.field('capture_status').equals('FAIL')

    def test_spawn_failure_retains_pending_and_terminal(self) -> None:
        with TemporaryDirectory() as folder:
            root=Path(folder).resolve();binary,env=self.make(root,'pass\n')
            _=binary.write_bytes(b'invalid executable format')
            manifest=Path(env['CARGO_CAPTURE_MANIFEST']);body=dict(document(manifest.read_text()).mapping())
            body['artifacts']=[{'id':'test-one','path':str(binary),'sha256':file_hash(binary)}]
            _=manifest.write_text(json.dumps(body));env['CARGO_CAPTURE_MANIFEST_SHA256']=file_hash(manifest)
            self.assertEqual(self.launch(binary,env).returncode,125)
            document(self.receipt(root)).field('native_exit').equals(None)
            self.assertTrue((root/'receipts/000001/pending.json').exists())

    def test_tty_rejected_without_world_or_child(self) -> None:
        with TemporaryDirectory() as folder:
            root=Path(folder).resolve();binary,env=self.make(root,'pass\n')
            with patch.object(os,'isatty',return_value=True):
                code=capture.execute(Path(env['CARGO_CAPTURE_MANIFEST']),env['CARGO_CAPTURE_MANIFEST_SHA256'],(str(binary),))
            self.assertEqual(code,125)
            document(self.receipt(root)).field('child_pid').equals(None)

    def test_parent_can_continue_distinct_child_after_failure(self) -> None:
        with TemporaryDirectory() as folder:
            root=Path(folder).resolve();binary,env=self.make(root,'raise SystemExit(7)\n')
            second=root/'second';second.mkdir();other=second/'child.py'
            _=other.write_text('#!'+sys.executable+'\nraise SystemExit(0)\n');other.chmod(0o755)
            manifest=Path(env['CARGO_CAPTURE_MANIFEST']);body=dict(document(manifest.read_text()).mapping())
            artifacts=body['artifacts'];assert isinstance(artifacts,list)
            artifacts.append({'id':'test-two','path':str(other),'sha256':file_hash(other)})
            _=manifest.write_text(json.dumps(body));env['CARGO_CAPTURE_MANIFEST_SHA256']=file_hash(manifest)
            self.assertEqual(self.launch(binary,env).returncode,7)
            self.assertEqual(self.launch(other,env).returncode,0)
            receipt=document((root/'receipts/000002/receipt.json').read_text())
            receipt.field('sequence').equals(2);receipt.field('artifact_id').equals('test-two')
            receipt.field('parent_pid').equals(os.getpid());receipt.field('cwd').equals(str(second))

    def test_unauthorized_binary_never_spawns(self) -> None:
        with TemporaryDirectory() as folder:
            root=Path(folder).resolve();binary,env=self.make(root,'pass\n')
            other=binary.with_name('other.py');_=other.write_bytes(binary.read_bytes());other.chmod(0o755)
            self.assertEqual(self.launch(other,env).returncode,125)
            document(self.receipt(root)).field('child_pid').equals(None)

    def test_sigkill_self_signal_does_not_attempt_handler_install(self) -> None:
        with TemporaryDirectory() as folder:
            root=Path(folder).resolve();binary,env=self.make(root,'import os,signal\nos.kill(os.getpid(),signal.SIGKILL)\n')
            self.assertEqual(self.launch(binary,env).returncode,-signal.SIGKILL)
            document(self.receipt(root)).field('native_signal').equals(int(signal.SIGKILL))

    def test_explicit_observer_scope_is_fresh_and_recorded(self) -> None:
        with TemporaryDirectory() as folder:
            root=Path(folder).resolve();binary,env=self.make(root,"import os,pathlib\npathlib.Path(os.environ['WORLDSIM_GATE_TEST_OUT']).joinpath('native.json').write_text('{}')\n")
            manifest=Path(env['CARGO_CAPTURE_MANIFEST']);body=dict(document(manifest.read_text()).mapping())
            body['observer_variable']='WORLDSIM_GATE_TEST_OUT'
            _=manifest.write_text(json.dumps(body));env['CARGO_CAPTURE_MANIFEST_SHA256']=file_hash(manifest)
            self.assertEqual(self.launch(binary,env).returncode,0)
            receipt=document(self.receipt(root))
            receipt.field('observer_additions').field('WORLDSIM_GATE_TEST_OUT').equals(str(root/'receipts/000001/observer'))
            receipt.field('output_inventory').field('observer/native.json').equals(hashlib.sha256(b'{}').hexdigest())

    def exercise_lifecycle_failure(self, failure: str) -> None:
        with TemporaryDirectory() as folder:
            root=Path(folder).resolve();binary,env=self.make(root,'import time\ntime.sleep(0.05)\n')
            real_write=atomic_json;real_send=subprocess.Popen[bytes].send_signal;real_popen=subprocess.Popen[bytes]
            real_start=threading.Thread.start;real_wait=subprocess.Popen[bytes].wait;real_hash=file_hash
            sent: list[int]=[];workers: list[threading.Thread]=[];processes: list[subprocess.Popen[bytes]]=[]
            waits=0;starts=0
            def send(process: subprocess.Popen[bytes], signum: int) -> None:
                sent.append(signum)
                if failure=='send':raise OSError('injected send failure')
                real_send(process,signum)
            def spawn(argv: tuple[str,...], *, executable: Path, cwd: Path, env: dict[str,str], stdin: None, stdout: int, stderr: int) -> subprocess.Popen[bytes]:
                if failure=='queued':
                    handler=signal.getsignal(signal.SIGTERM);assert callable(handler)
                    handler(int(signal.SIGTERM),None)
                return real_popen(argv,executable=executable,cwd=cwd,env=env,stdin=stdin,stdout=stdout,stderr=stderr)
            def write(path: Path, value: dict[str,Json]) -> None:
                real_write(path,value)
                if path.name=='started.json' and failure in ('signal','send'):
                    handler=signal.getsignal(signal.SIGTERM)
                    assert callable(handler)
                    handler(int(signal.SIGTERM),None)
            def start(worker: threading.Thread) -> None:
                nonlocal starts
                starts+=1
                if failure=='start' and starts==2:raise RuntimeError('injected second start failure')
                real_start(worker);workers.append(worker)
            def wait(process: subprocess.Popen[bytes], timeout: float | None = None) -> int:
                nonlocal waits
                waits+=1
                if process not in processes:processes.append(process)
                if failure=='wait' and waits==1:raise OSError('injected wait failure')
                return real_wait(process,timeout)
            def hashed(path: Path) -> str:
                if path.name in ('stdout.log','stderr.log'):
                    self.assertTrue(all(not worker.is_alive() for worker in workers))
                    self.assertTrue(all(process.poll() is not None for process in processes))
                return real_hash(path)
            with patch.object(os,'isatty',return_value=False),patch.object(capture,'atomic_json',side_effect=write),patch.object(subprocess.Popen,'send_signal',send),patch.object(subprocess.Popen,'wait',wait),patch.object(threading.Thread,'start',start),patch.object(capture,'file_hash',side_effect=hashed),patch.object(subprocess,'Popen',side_effect=spawn):
                result=capture.execute(Path(env['CARGO_CAPTURE_MANIFEST']),env['CARGO_CAPTURE_MANIFEST_SHA256'],(str(binary),))
            if failure in ('signal','queued'):
                self.assertEqual(sent,[signal.SIGTERM]);self.assertEqual(result,-signal.SIGTERM)
            else:self.assertEqual(result,125)
            self.assertTrue(all(not worker.is_alive() for worker in workers))
            receipt=document(self.receipt(root))
            if failure in ('signal','send','queued'):receipt.field('received_signals').equals([int(signal.SIGTERM)])

    def test_signal_after_spawn_is_forwarded_exactly_once(self) -> None:
        self.exercise_lifecycle_failure('signal')

    def test_signal_before_spawn_is_drained_exactly_once(self) -> None:
        self.exercise_lifecycle_failure('queued')

    def test_second_thread_start_failure_settles_before_hash(self) -> None:
        self.exercise_lifecycle_failure('start')

    def test_wait_failure_settles_before_hash(self) -> None:
        self.exercise_lifecycle_failure('wait')

    def test_signal_forward_failure_settles_before_hash(self) -> None:
        self.exercise_lifecycle_failure('send')

if __name__=='__main__':
    _=unittest.main()
