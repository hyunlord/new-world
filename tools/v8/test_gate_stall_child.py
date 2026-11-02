"""Synthetic direct-child and byte-range regressions; no Godot execution."""
import json
import os
import shutil
import subprocess
import sys
import time
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

from gate_stall_child import (
    ChildPlan,
    bind_chain,
    capture_wrapper,
    close_capture,
    range_hash,
    supervise,
)
from generic_gate import GateContractError, document, integer
from generic_process import atomic_json, file_hash
from mux2_gate_inputs import Json


class ChildTests(unittest.TestCase):
    def test_native_nonzero_is_preserved(self) -> None:
        pids: list[int] = []
        self.assertEqual(supervise((sys.executable, '-c', 'raise SystemExit(1)'), 10, pids.append), (1, False))
        self.assertEqual(len(pids), 1)
        with self.assertRaises(ProcessLookupError):
            os.kill(pids[0], 0)

    def test_publication_exception_reaps_owned_child(self) -> None:
        pids: list[int] = []
        def reject(pid: int) -> None:
            pids.append(pid)
            raise RuntimeError('receipt unavailable')
        with self.assertRaisesRegex(RuntimeError, 'receipt unavailable'):
            _ = supervise((sys.executable, '-c', 'import time; time.sleep(30)'), 10, reject)
        with self.assertRaises(ProcessLookupError):
            os.kill(pids[0], 0)

    def test_timeout_reaps_owned_child(self) -> None:
        pids: list[int] = []
        code, timeout = supervise((sys.executable, '-c', 'import time; time.sleep(30)'), 0, pids.append)
        self.assertTrue(timeout)
        self.assertLess(code, 0)
        with self.assertRaises(ProcessLookupError):
            os.kill(pids[0], 0)

    def test_keyboard_interrupt_preserves_exception_and_reaps(self) -> None:
        pids: list[int] = []
        def interrupt(pid: int) -> None:
            pids.append(pid)
            raise KeyboardInterrupt
        with self.assertRaises(KeyboardInterrupt):
            _ = supervise((sys.executable, '-c', 'import time; time.sleep(30)'), 10, interrupt)
        with self.assertRaises(ProcessLookupError):
            os.kill(pids[0], 0)

    def test_range_hash_does_not_move_shared_writer(self) -> None:
        with TemporaryDirectory() as folder:
            path = Path(folder)/'log'
            with path.open('wb', buffering=0) as writer:
                _ = writer.write(b'prefix\n\xffnative\r\n')
                before = os.lseek(writer.fileno(), 0, os.SEEK_CUR)
                first = range_hash(path, 7, before)
                self.assertEqual(os.lseek(writer.fileno(), 0, os.SEEK_CUR), before)
                _ = writer.write(b'tail')
                self.assertEqual(first, range_hash(path, 7, before))

    def test_wait_error_preserves_exception_and_reaps(self) -> None:
        original = subprocess.Popen[bytes].wait
        first = True
        pids: list[int] = []
        def wait(process: subprocess.Popen[bytes], timeout: float | None = None) -> int:
            nonlocal first
            if first:
                first = False
                raise OSError('wait failed')
            return original(process, timeout=timeout)
        with patch.object(subprocess.Popen, 'wait', wait), self.assertRaisesRegex(OSError, 'wait failed'):
            _ = supervise((sys.executable, '-c', 'import time; time.sleep(30)'), 10, pids.append)
        with self.assertRaises(ProcessLookupError):
            os.kill(pids[0], 0)



    def test_default_off_wrapper_preserves_exit_and_heredoc_eof(self) -> None:
        wrapper=Path(__file__).with_name('run_bounded.sh')
        for code in (0,1,2):
            result=subprocess.run(('bash',str(wrapper),'10',sys.executable,'-c',
                'import os,sys; assert os.read(0,1)==b""; os.write(1,b"out\\xff"); os.write(2,b"err\\xfe"); sys.exit('+str(code)+')'),
                input=b'caller stdin must not replace heredoc',capture_output=True,check=False)
            self.assertEqual(result.returncode,code)
            self.assertEqual(result.stdout,b'out\xff')
            self.assertEqual(result.stderr,b'err\xfe')

    def test_exec_shell_environment_projection_is_measured(self) -> None:
        environment=dict(os.environ,SHLVL='2',PWD='/incorrect',_='preserved')
        result=subprocess.run(('bash','-c',"exec python3 - <<'PYCODE'\nimport os,json\nprint(json.dumps(dict(os.environ)))\nPYCODE"),
                              env=environment,capture_output=True,text=True,check=True)
        expected=dict(environment,PWD=str(Path.cwd()))
        _=expected.pop('OLDPWD',None)
        self.assertEqual(json.loads(result.stdout),expected)


    def test_frozen_chain_admits_only_exact_native_outcomes_and_ranges(self) -> None:
        with TemporaryDirectory() as folder:
            base=Path(folder).resolve();root=base/'product';capture=base/'capture';evidence=base/'evidence'
            for directory in (root,capture,evidence):directory.mkdir()
            log=capture/'command.log';_=log.write_bytes(b'command\nnative\xff\n')
            source=root/'source';_=source.write_text('fixed')
            lock=base/'lock';_=lock.write_text('')
            info=lock.stat();planfile=base/'plan.json'
            argv=('/native','--path',str(root))
            rawplan: dict[str,Json]={'schema':'stall_bounded_child_v1','protocol':'NONE','root':str(root),
                'capture':str(capture),'evidence':str(evidence),'argv':list(argv),'inputs':{str(source):file_hash(source)}}
            _=planfile.write_text(json.dumps(rawplan));digest=file_hash(planfile)
            plan=ChildPlan(planfile,digest,root,capture,evidence,argv,((source,file_hash(source)),))
            environment={'PATH':'/bin','SHLVL':'2','OLDPWD':'/old','_':'preserved',
                'WORLDSIM_TEST_SETTINGS_DIR':str(capture/'settings'),'WORLDSIM_MUX2_PROFILE':'1',
                'WORLDSIM_STALL_CHILD_PLAN':str(planfile),'WORLDSIM_STALL_CHILD_PLAN_SHA256':digest}
            childenv={k:v for k,v in environment.items() if k not in ('OLDPWD','_')};childenv['PWD']=str(root)
            stamp='2026-11-03T00:00:00+00:00'
            wrapper: dict[str,Json]={'capture_pid':11,'wrapper_pid':12,'bash':str(source),'bash_sha256':file_hash(source),'argv':['bash',str(root/'tools/v8/run_bounded.sh'),'960',*argv],
                'cwd':str(root),'environment':dict(environment),'plan_sha256':digest,'started_at':stamp,'ended_at':stamp,
                'native_exit':1,'errors':[], 'schema':'stall_bounded_child_v1',
                'duration_ns':2,'started_monotonic_ns':1,'ended_monotonic_ns':3,
                'wrapper_environment':{**environment,'WORLDSIM_STALL_CHILD_CONTROL_FD':'9'}}
            native: dict[str,Json]={'capture_pid':11,'wrapper_pid':12,'native_pid':13,'argv':list(argv),
                'cwd':str(root),'environment':dict(childenv),'plan_sha256':digest,'started_at':stamp,'ended_at':stamp,
                'inputs':{str(source):file_hash(source)},'python':str(source),'lane_device':info.st_dev,'lane_inode':info.st_ino,
                'environment_policy':'inherited_capture_wrapper_environment','timed_out':False,'timeout_seconds':960,
                'timeout_owner_pid':12,'native_exit':1,'wrapper_exit':1,'errors':[],
                'schema':'stall_bounded_child_v1','duration_ns':2,'started_monotonic_ns':1,'ended_monotonic_ns':3,
                'wrapper_environment':{**childenv,'WORLDSIM_STALL_CHILD_CONTROL_FD':'9'},
                'stream_start':8,'stream_end':log.stat().st_size,'stream_sha256':range_hash(log,8,log.stat().st_size)}
            closed: dict[str,Json]={'schema':'stall_bounded_child_v1','capture_pid':11,'plan_sha256':digest,'path':str(log),'sha256':file_hash(log),
                'size':log.stat().st_size,'closed_at':stamp}
            def freeze(changed: dict[str,Json]) -> tuple[tuple[str,str],...]:
                for name,row in (('wrapper-start',wrapper),('wrapper-terminal',wrapper),('native-start',native),
                                 ('native-terminal',changed),('capture-log-closed',closed)):
                    _=(evidence/(name+'.json')).write_text(json.dumps(row))
                return tuple((p.name,file_hash(p)) for p in sorted(evidence.iterdir()))
            with patch('gate_stall_child.LOCK',lock):
                self.assertEqual(bind_chain(plan,environment,1,freeze(native)),13)
                mutations: tuple[tuple[str,Json], ...] = (('native_exit',2),('native_exit',-9),('timed_out',True),('timeout_seconds',1),
                                  ('duration_ns',-1),('duration_ns',True),('started_at','2027-01-01T00:00:00+00:00'),('schema','fake'),
                                  ('native_pid',12),('stream_start',0),('stream_sha256','0'*64),
                                  ('environment',{**childenv,'UNAUTHORIZED':'1'}))
                for key,value in mutations:
                    changed=dict(native);changed[key]=value
                    with self.subTest(key=key),self.assertRaises(GateContractError):
                        _=bind_chain(plan,environment,1,freeze(changed))
                hashes=freeze(native);_=log.write_bytes(b'late mutation')
                with self.assertRaises(GateContractError):_=bind_chain(plan,environment,1,hashes)


    def test_actual_miniature_wrapper_chain_keeps_stdin_and_live_bytes(self) -> None:
        self.miniature_chain('normal')

    def test_parent_publication_failure_never_releases_native_start(self) -> None:
        self.miniature_chain('publication')

    def test_parent_wait_failure_reaps_wrapper_and_native(self) -> None:
        self.miniature_chain('wait')

    def test_parent_interrupt_reaps_wrapper_and_native(self) -> None:
        self.miniature_chain('interrupt')

    def miniature_chain(self, failure: str) -> None:
        with TemporaryDirectory() as folder:
            base=Path(folder).resolve();root=base/'product';capture=base/'capture';evidence=base/'evidence'
            tools=root/'tools/v8';tools.mkdir(parents=True);capture.mkdir();evidence.mkdir();(capture/'settings').mkdir()
            lock=base/'lock';_=lock.write_text('')
            here=Path(__file__).parent
            helper=tools/'gate_stall_child.py'
            _=helper.write_text((here/'gate_stall_child.py').read_text().replace('from mux2_gate_lock import FD_ENV, LOCK',
                'from mux2_gate_lock import FD_ENV\nLOCK = Path('+repr(str(lock))+')'))
            _=shutil.copyfile(here/'run_bounded.sh',tools/'run_bounded.sh')
            for name in ('generic_gate.py','generic_process.py','generic_sources.py',
                         'mux2_gate_inputs.py','mux2_gate_lock.py'):
                _=shutil.copyfile(here/name,tools/name)
            for name in ('tools/v8/mux2_stall_test.gd','sim_bridge.gdextension','rust/target/debug/libsim_bridge.dylib'):
                path=root/name;path.parent.mkdir(parents=True,exist_ok=True);_=path.write_text('synthetic input')
            native=base/'tiny-native'
            _=native.write_text('#!'+sys.executable+'\nimport os,sys\nassert os.read(0,1)==b""\nos.write(1,b"native stdout\\xff")\nos.write(2,b"native stderr\\xfe")\nsys.exit(1)\n')
            if failure in ('wait','interrupt'):
                _=native.write_text(native.read_text().replace('sys.exit(1)', 'import time; time.sleep(30); sys.exit(1)'))
            native.chmod(0o755)
            argv=(str(native),'--path',str(root),'--resolution','1920x1080','--always-on-top','--windowed',
                  '--script','res://tools/v8/mux2_stall_test.gd','--',str(capture/'run'))
            inputs=tuple((p,file_hash(p)) for p in (*[p for p in root.rglob('*') if p.is_file()],native,Path(sys.executable).resolve(),Path('/bin/bash').resolve()))
            planfile=base/'plan.json'
            _=planfile.write_text(json.dumps({'schema':'stall_bounded_child_v1','protocol':'NONE','root':str(root),
                'capture':str(capture),'evidence':str(evidence),'argv':argv,'inputs':{str(p):h for p,h in inputs}}))
            digest=file_hash(planfile);log=capture/'command.log'
            with lock.open('a') as held,log.open('w') as stream:
                environment=dict(os.environ,WORLDSIM_STALL_CHILD_PLAN=str(planfile),WORLDSIM_STALL_CHILD_PLAN_SHA256=digest,
                    WORLDSIM_TEST_SETTINGS_DIR=str(capture/'settings'),WORLDSIM_MUX2_PROFILE='1',WORLDSIM_MUX2_LANE_FD=str(held.fileno()))
                _=environment.pop('PYTHONPATH',None)
                command=['bash',str(tools/'run_bounded.sh'),'960',*argv]
                _=stream.write(json.dumps(command)+'\n');stream.flush()
                original_wait = subprocess.Popen[bytes].wait
                injected = False
                def fail_wait(process: subprocess.Popen[bytes], timeout: float | None = None) -> int:
                    nonlocal injected
                    if not injected:
                        injected = True
                        deadline=time.monotonic()+5
                        while not (evidence/'native-start.json').exists() and time.monotonic()<deadline:
                            time.sleep(0.01)
                        self.assertTrue((evidence/'native-start.json').exists())
                        if failure=='interrupt':raise KeyboardInterrupt
                        raise OSError('parent wait failed after native start')
                    return original_wait(process,timeout=timeout)
                def fail_publication(path: Path, value: dict[str,Json]) -> None:
                    if path.name=='wrapper-start.json':
                        self.assertFalse((evidence/'native-start.json').exists())
                        raise OSError('parent start publication failed')
                    atomic_json(path,value)
                if failure=='publication':
                    with patch('gate_stall_child.atomic_json',side_effect=fail_publication),self.assertRaises(OSError):
                        _=capture_wrapper(command,root,environment,stream.fileno(),held.fileno())
                elif failure in ('wait','interrupt'):
                    with patch.object(subprocess.Popen,'wait',fail_wait),self.assertRaises(KeyboardInterrupt if failure=='interrupt' else OSError):
                        _=capture_wrapper(command,root,environment,stream.fileno(),held.fileno())
                else:
                    self.assertEqual(capture_wrapper(command,root,environment,stream.fileno(),held.fileno()),1)
            close_capture(environment,log)
            if failure!='normal':
                wrapper_pid=integer(document((evidence/'wrapper-terminal.json').read_text()).field('wrapper_pid'))
                with self.assertRaises(ProcessLookupError):os.kill(wrapper_pid,0)
                if failure in ('wait','interrupt'):
                    native_pid=integer(document((evidence/'native-start.json').read_text()).field('native_pid'))
                    with self.assertRaises(ProcessLookupError):os.kill(native_pid,0)
                else:self.assertFalse((evidence/'native-start.json').exists())
                return
            self.assertIn(b'native stdout\xff',log.read_bytes());self.assertIn(b'native stderr\xfe',log.read_bytes())
            plan=ChildPlan(planfile,digest,root,capture,evidence,argv,inputs)
            frozen=tuple((p.name,file_hash(p)) for p in sorted(evidence.iterdir()))
            with patch('gate_stall_child.LOCK',lock):
                pid=bind_chain(plan,environment,1,frozen)
            with self.assertRaises(ProcessLookupError):os.kill(pid,0)

if __name__ == '__main__':
    _ = unittest.main()
