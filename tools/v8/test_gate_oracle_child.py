"""Real tiny-process ownership regressions; no Godot or WorldSim execution."""
import fcntl
import os
import select
import shutil
import subprocess
import sys
import time
import unittest
from collections.abc import Callable, Generator
from contextlib import contextmanager
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import override
from unittest.mock import patch

import gate_oracle_child as child
from generic_gate import GateContractError, document, integer
from generic_process import atomic_json, file_hash
from mux2_gate_inputs import Json
from mux2_gate_lock import FD_ENV

ROOT = Path('/Users/rexxa/work/new-world-m6b-dev/tools/v8')
DRAFT = Path(__file__).parent


class OracleChildTests(unittest.TestCase):
    def test_closed_eof_is_rejected_without_monitor_or_spawn(self) -> None:
        read, write = os.pipe()
        os.close(write)
        try:
            with patch('gate_stall_child.threading.Thread.start') as monitor, patch('subprocess.Popen') as spawn:
                with self.assertRaises(GateContractError):
                    child.require_connected(read)
                monitor.assert_not_called()
                spawn.assert_not_called()
        finally:
            os.close(read)

    def test_original_unset_wrapper_keeps_exit_streams_stdin(self) -> None:
        for code in (0, 1, 2):
            result = subprocess.run(('bash', str(DRAFT/'run_bounded.sh'), '10', sys.executable, '-c',
                'import os; assert os.read(0,1)==b""; os.write(1,b"a\\xff"); os.write(2,b"b\\xfe"); raise SystemExit('+str(code)+')'),
                capture_output=True, check=False)
            self.assertEqual((result.returncode,result.stdout,result.stderr),(code,b'a\xff',b'b\xfe'))

    def test_partial_and_conflicting_modes_reject(self) -> None:
        for environment in ({'WORLDSIM_ORACLE_CHILD_PLAN':'missing'},
                            {'WORLDSIM_ORACLE_CHILD_PLAN':'missing','WORLDSIM_ORACLE_CHILD_PLAN_SHA256':'0'*64,
                             'WORLDSIM_STALL_CHILD_PLAN':'missing'}):
            result=subprocess.run(('bash',str(DRAFT/'run_bounded.sh'),'180','/usr/bin/true'),
                                  env={'PATH':os.environ['PATH'],**environment},capture_output=True,check=False)
            self.assertNotEqual(result.returncode,0)

    def test_real_structural_recipe_success_and_nonzero(self) -> None:
        self.chain(0)
        self.chain(7)

    def test_parent_publication_failure_has_no_native(self) -> None:
        self.chain(0, publication=True)

    def test_missing_terminal_is_unknown_not_wrapper_completion(self) -> None:
        self.chain(0, missing=True)

    def test_suite_mode_rejected(self) -> None:
        self.chain(0, suite=True)

    def test_parent_wait_fault_cancels_and_reaps_real_native(self) -> None:
        self.chain(0, wait_fault=True)

    def test_parent_interrupt_cancels_and_reaps_real_native(self) -> None:
        self.chain(0, interrupt=True)

    def test_supervisor_death_before_release_is_unknown(self) -> None:
        self.chain(0, death=True)

    def test_owned_timeout_preserves_negative_native_exit(self) -> None:
        self.chain(0, timeout=True)

    def chain(self, code: int, *, publication: bool=False, missing: bool=False, suite: bool=False, wait_fault: bool=False, interrupt: bool=False, death: bool=False, timeout: bool=False) -> None:
        with TemporaryDirectory() as folder:
            base=Path(folder).resolve();root=base/'product';tools=root/'tools/v8';tools.mkdir(parents=True)
            evidence=base/'evidence';evidence.mkdir();log=base/'merged.log';lock=base/'lock';lock.touch()
            names=('gate_oracle_child.py','gate_stall_child.py','generic_gate.py','generic_process.py',
                   'generic_sources.py','mux2_gate_inputs.py','mux2_gate_lock.py')
            for name in names:
                source=DRAFT/name if (DRAFT/name).exists() else ROOT/name
                value=source.read_text()
                if name=='gate_oracle_child.py':
                    value=value.replace('from mux2_gate_lock import FD_ENV, LOCK',
                                        'from mux2_gate_lock import FD_ENV\nLOCK = Path('+repr(str(lock))+')')
                if name=='gate_oracle_child.py' and timeout:
                    value=value.replace('supervise(argv,limit,started,control)','supervise(argv,0,started,control)')
                _=(tools/name).write_text(value)
            _=shutil.copyfile(DRAFT/'run_bounded.sh',tools/'run_bounded.sh')
            _=(tools/'m6a_plant_art_test.gd').write_text('synthetic source, no Godot')
            native=base/'tiny'
            _=native.write_text('#!'+sys.executable+'\nimport os\nassert os.read(0,1)==b""\nos.write(1,b"native\\xff")\nos.write(2,b"stderr\\xfe")\nraise SystemExit('+str(code)+')\n')
            if wait_fault or interrupt or timeout:
                _=native.write_text(native.read_text().replace('raise SystemExit(', 'import time; time.sleep(30); raise SystemExit('))
            native.chmod(0o755)
            argv=(str(native),'--rendering-driver','opengl3','--path','.', '--script','res://tools/v8/m6a_plant_art_test.gd')
            environment={'PATH':str(Path(sys.executable).parent)+':/usr/bin:/bin','LANG':'C','LC_ALL':'C',
                         'PYTHONNOUSERSITE':'1','PYTHONDONTWRITEBYTECODE':'1','PYTHONCOERCECLOCALE':'0',
                         '__CF_USER_TEXT_ENCODING':f'0x{os.getuid():X}:0x3:0x33'}
            inputs={str(p):file_hash(p) for p in (*tools.iterdir(), native, Path(sys.executable).resolve(),Path('/bin/bash').resolve())}
            planfile=base/'plan.json'
            raw: dict[str,Json]={'schema':'oracle_bounded_child_v1','mode':'suite' if suite else 'structural',
                'root':str(root),'evidence':str(evidence),'log':str(log),'argv':list(argv),'inputs':{key:value for key,value in inputs.items()},'environment':{key:value for key,value in environment.items()}}
            atomic_json(planfile,raw)
            def publish(path: Path, value: dict[str,Json]) -> None:
                if publication and path.name=='wrapper-start.json':raise OSError('publication failed')
                atomic_json(path,value)
                if death and path.name=='wrapper-start.json':
                    os.kill(integer(document(path.read_text()).field('wrapper_pid')),9)
            original_wait=subprocess.Popen[bytes].wait
            injected=False
            def wait(process: subprocess.Popen[bytes], timeout: float | None=None) -> int:
                nonlocal injected
                if (wait_fault or interrupt) and not injected:
                    injected=True
                    deadline=time.monotonic()+5
                    while not (evidence/'native-start.json').exists() and time.monotonic()<deadline:
                        time.sleep(0.01)
                    self.assertTrue((evidence/'native-start.json').exists())
                    if interrupt:raise KeyboardInterrupt
                    raise OSError('injected parent wait failure')
                return original_wait(process,timeout=timeout)
            with lock.open('a') as lane,patch('gate_oracle_child.LOCK',lock),patch('gate_oracle_child.atomic_json',side_effect=publish),patch.object(subprocess.Popen,'wait',wait):
                if suite:
                    with self.assertRaises(GateContractError):
                        _=child.capture(planfile,file_hash(planfile),lane.fileno())
                    return
                result=child.capture(planfile,file_hash(planfile),lane.fileno())
            if publication or wait_fault or interrupt or death:
                self.assertEqual(result.settlement,'UNKNOWN')
                if wait_fault or interrupt:
                    native_pid=integer(document((evidence/'native-start.json').read_text()).field('native_pid'))
                    with self.assertRaises(ProcessLookupError):os.kill(native_pid,0)
                else:self.assertFalse((evidence/'native-start.json').exists())
            else:
                self.assertEqual(result.settlement,'SETTLED',result.error+' LOG '+log.read_text(errors='replace'))
                if timeout:
                    self.assertEqual(result.wrapper_exit,124)
                    self.assertIsNotNone(result.native_exit)
                    assert result.native_exit is not None
                    self.assertLess(result.native_exit,0)
                else:self.assertEqual((result.native_exit,result.wrapper_exit),(code,code))
                if not timeout:self.assertIn(b'native\xff',log.read_bytes())
                if not timeout:self.assertIn(b'stderr\xfe',log.read_bytes())
                native_pid=integer(document((evidence/'native-start.json').read_text()).field('native_pid'))
                with self.assertRaises(ProcessLookupError):os.kill(native_pid,0)
                if missing:
                    (evidence/'native-terminal.json').unlink()
                    with patch('gate_oracle_child.LOCK',lock):
                        checked=child.inspect(planfile,file_hash(planfile),result.wrapper_pid,result.wrapper_exit)
                    self.assertEqual(checked.settlement,'UNKNOWN')
            self.assertIsNotNone(result.wrapper_pid)
            assert result.wrapper_pid is not None
            with self.assertRaises(ProcessLookupError):os.kill(result.wrapper_pid,0)


class SuiteRecipeTests(unittest.TestCase):
    def test_partial_suite_and_cross_protocol_reject_before_command(self) -> None:
        for values in ({child.SUITE_PLAN:'missing'},
            {child.SUITE_PLAN:'missing',child.SUITE_HASH:'a'*64,child.SUITE_REQUEST:'9',child.SUITE_CONTROL:'10',child.PLAN:'other'}):
            result=subprocess.run(('bash',str(DRAFT/'run_bounded.sh'),'300',sys.executable,'-c','print("UNEXPECTED")'),
                env={'PATH':os.environ['PATH'],**values},capture_output=True,check=False)
            self.assertNotEqual(result.returncode,0)
            self.assertNotIn(b'UNEXPECTED',result.stdout)

    def test_suite_table_preserves_original_limits_and_tails(self) -> None:
        rows=child.suite_rows(Path('/product'),Path('/evidence'),Path('/logs'),Path('/godot'))
        self.assertEqual(len(rows),16)
        self.assertEqual([row.limit for row in rows],[300]*15+[120])
        self.assertEqual(rows[12].argv[-2:],('--','--expect-lazy'))
        self.assertEqual(rows[13].argv[-2:],('--','--expect-cache'))
        self.assertEqual(rows[-1].log,Path('/logs/stall-self-test.log'))
        self.assertEqual(rows[-1].argv[-2:],('--','--self-test'))


class SuiteProcessTests(unittest.TestCase):
    def test_original_shell_runs_sixteen_admitted_tiny_producers(self) -> None:
        self.suite()

    def test_producer_nonzero_preserves_prefix_and_original_shell_stop(self) -> None:
        self.suite(native_fail=3)

    def test_checker_failure_prevents_next_admission(self) -> None:
        self.suite(checker_fail=2)

    def test_admission_publication_failure_releases_no_native(self) -> None:
        self.suite(publication=True)

    def test_native_start_publication_failure_keeps_unknown_and_reaps(self) -> None:
        self.suite(fault='native_start_write')

    def test_native_terminal_publication_failure_stays_unknown(self) -> None:
        self.suite(fault='native_terminal_write')

    def test_completion_notification_failure_stays_unknown(self) -> None:
        self.suite(fault='completion_write')

    def test_first_request_can_arrive_before_suite_start_publication(self) -> None:
        self.suite(fault='early_request')

    def test_suite_timeout_preserves_raw_negative_and_shell_124(self) -> None:
        self.suite(fault='timeout')

    def test_source_drift_during_admission_prevents_native(self) -> None:
        self.suite(fault='source_mutation')

    def test_suite_start_failure_retains_live_owner(self) -> None:
        self.suite(fault='suite_publication')

    def test_controller_runtime_error_cancels_and_reaps_owned_native(self) -> None:
        self.suite(fault='reader')

    def test_controller_interrupt_cancels_and_retains_live_owner(self) -> None:
        self.suite(fault='interrupt')

    def test_request_mutation_during_publication_rejects_before_grant(self) -> None:
        self.suite(fault='request_mutation')

    def test_closed_control_before_first_request_creates_no_native(self) -> None:
        self.suite(fault='early_cancel')

    def test_supervisor_death_before_grant_is_unknown(self) -> None:
        self.suite(fault='wrapper_death')

    def test_late_raw_log_mutation_is_rejected(self) -> None:
        self.suite(fault='log_mutation')

    def suite(self, *, native_fail: int=0, checker_fail: int=0, publication: bool=False, fault: str='') -> None:
        with TemporaryDirectory() as directory:
            base=Path(directory).resolve();root=base/'product';tools=root/'tools/v8';tools.mkdir(parents=True)
            evidence=base/'evidence';evidence.mkdir();logs=base/'logs';logs.mkdir();lock=base/'lock';lock.touch()
            names=('gate_oracle_child.py','gate_stall_child.py','generic_gate.py','generic_process.py','generic_sources.py',
                   'mux2_gate_inputs.py','mux2_gate_lock.py','run_bounded.sh','mux2_gate_checks.sh','godot_bin.sh')
            for name in names:
                source=DRAFT/name if (DRAFT/name).exists() else ROOT/name
                value=source.read_text()
                if name=='gate_oracle_child.py' and fault=='native_start_write':
                    value=value.replace("atomic_json(row.evidence/'native-start.json',","raise OSError('injected native start publication')\n        atomic_json(row.evidence/'native-start.json',")
                if name=='gate_oracle_child.py' and fault=='native_terminal_write':
                    value=value.replace("atomic_json(row.evidence/'native-terminal.json',","raise OSError('injected native terminal publication')\n        atomic_json(row.evidence/'native-terminal.json',")
                if name=='gate_oracle_child.py' and fault=='completion_write':
                    value=value.replace("suite_send(request,{'event':'finished'","raise OSError('injected completion write')\n        suite_send(request,{'event':'finished'")
                if name=='gate_oracle_child.py' and fault=='timeout':
                    value=value.replace('code,timed_out=supervise(argv,limit,started,control);settled=True','code,timed_out=supervise(argv,0,started,control);settled=True')
                if name=='mux2_gate_lock.py':value=value.replace("Path('/Users/rexxa/work/mux2-data/.mux2-gpu.lock')",'Path('+repr(str(lock))+')')
                _=(tools/name).write_text(value)
            for name in child.SUITE_NAMES:_=(tools/(name+'.gd')).write_text('tiny fixture, no Godot execution')
            checker=tools/'mux2_gate_oracle_log.py'
            _=checker.write_text('import sys\nfrom pathlib import Path\np=Path('+repr(str(base/'checkers'))+')\np.open("a").write(sys.argv[1]+"\\n")\nraise SystemExit(9 if len(p.read_text().splitlines())=='+str(checker_fail)+' else 0)\n')
            native=base/'tiny'
            _=native.write_text('#!'+sys.executable+'\nimport os,sys\nfrom pathlib import Path\nassert not any(k.startswith("WORLDSIM_ORACLE_") for k in os.environ)\nassert "WORLDSIM_MUX2_LANE_FD" not in os.environ\nassert os.read(0,1)==b""\np=Path('+repr(str(base/'natives'))+')\np.open("a").write(str(os.getpid())+" "+str(os.getppid())+"\\n")\nprint("MUX2_STALL_SELF_TEST failures=[]")\nraise SystemExit(7 if len(p.read_text().splitlines())=='+str(native_fail)+' else 0)\n')
            if fault in ('reader','interrupt','timeout'):
                _=native.write_text(native.read_text().replace('print("MUX2_STALL', 'import time;time.sleep(30)\nprint("MUX2_STALL'))
            native.chmod(0o755)
            environment={'PATH':str(Path(sys.executable).parent)+':/usr/bin:/bin','LANG':'C','LC_ALL':'C',
                'PYTHONNOUSERSITE':'1','PYTHONDONTWRITEBYTECODE':'1','PYTHONCOERCECLOCALE':'0',
                '__CF_USER_TEXT_ENCODING':f'0x{os.getuid():X}:0x3:0x33','GODOT_BIN':str(native)}
            inputs={str(p):file_hash(p) for p in (*tools.iterdir(),native,Path(sys.executable).resolve(),Path('/bin/bash').resolve())}
            planfile=base/'plan.json'
            atomic_json(planfile,{'schema':child.SUITE_SCHEMA,'root':str(root),'evidence':str(evidence),'logs':str(logs),
                'godot':str(native),'inputs':{k:v for k,v in inputs.items()},'environment':{k:v for k,v in environment.items()}})
            plan=child.load_suite(planfile,file_hash(planfile))
            controller: child.SuiteController | None=None
            def publish(path: Path, value: dict[str,Json]) -> None:
                if publication and path.name=='wrapper-start.json':raise OSError('injected admission publication')
                if fault=='suite_publication' and path.name=='suite-start.json':raise OSError('injected suite publication')
                atomic_json(path,value)
                if fault=='request_mutation' and path.name=='wrapper-start.json':
                    _=(path.parent/'request.json').write_text('{}')
                if fault=='source_mutation' and path.name=='wrapper-start.json':
                    _=native.write_text(native.read_text()+'\n# changed source\n')
                if fault=='wrapper_death' and path.name=='wrapper-start.json':
                    os.kill(integer(document(path.read_text()).field('request').field('wrapper_pid')),9)
            with lock.open('a') as lane,(base/'shell.log').open('wb') as stream,patch.object(child,'LOCK',lock),patch.object(child,'atomic_json',side_effect=publish):
                channels=child.prepare_suite(plan,lane.fileno())
                request_read,request_write=channels.request_read,channels.request_write
                control_read,control_write=channels.control_read,channels.control_write
                env=dict(environment,**{child.SUITE_PLAN:str(planfile),child.SUITE_HASH:plan.digest,
                    child.SUITE_REQUEST:str(request_write),child.SUITE_CONTROL:str(control_read),FD_ENV:str(lane.fileno())})
                shell=subprocess.Popen(('bash','tools/v8/mux2_gate_checks.sh','oracles',str(logs)),cwd=root,env=env,
                    stdout=stream,stderr=subprocess.STDOUT,pass_fds=(request_write,control_read,lane.fileno()))
                os.close(request_write);os.close(control_read)
                try:
                    controller=child.SuiteController(plan,shell,lane.fileno(),channels)
                    if fault=='early_request':
                        ready,_,_=select.select((request_read,),(),(),5)
                        self.assertEqual(ready,[request_read])
                    if fault=='suite_publication':
                        with self.assertRaises(OSError):controller.start()
                    else:controller.start()
                    if fault=='early_cancel':controller.cancel('synthetic preexisting cancellation')
                    deadline=time.monotonic()+15;injected=False
                    while shell.poll() is None and time.monotonic()<deadline:
                        try:
                            starts=tuple(evidence.glob('*/native-start.json'))
                            if fault in ('reader','interrupt') and starts and not injected:
                                injected=True
                                failure=RuntimeError('synthetic reader failure') if fault=='reader' else KeyboardInterrupt('synthetic interruption')
                                with patch.object(select,'select',side_effect=failure):controller.service_ready()
                            else:controller.service_ready()
                        except (OSError,ValueError,RuntimeError,KeyboardInterrupt):
                            if not publication and not fault:raise
                        _=select.select((request_read,),(),(),0.02)
                    _=shell.wait(timeout=2)
                    if not publication and (not fault or fault in ('log_mutation','timeout','early_request')):controller.service_ready()
                    if fault in ('request_mutation','log_mutation','source_mutation'):
                        if fault=='log_mutation':
                            _=plan.rows[0].log.write_text('replaced raw')
                        with self.assertRaisesRegex(ValueError,'changed|range beyond log'): _=controller.snapshot()
                        if fault in ('request_mutation','source_mutation'):self.assertFalse((base/'natives').exists())
                        return
                    result=controller.snapshot()
                    if publication or fault not in ('','timeout','early_request'):
                        self.assertEqual(result.settlement,'UNKNOWN')
                        with lock.open('a') as other,self.assertRaises(BlockingIOError):
                            fcntl.flock(other.fileno(),fcntl.LOCK_EX|fcntl.LOCK_NB)
                        if fault not in ('reader','interrupt','native_start_write','native_terminal_write','completion_write'):self.assertFalse((base/'natives').exists())
                        with self.assertRaises(GateContractError):_=controller.close_settled()
                    else:
                        self.assertEqual(result.settlement,'SETTLED',result.error+' '+(base/'shell.log').read_text()+''.join(p.read_text() for p in logs.iterdir()))
                        expected=1 if fault=='timeout' else native_fail or checker_fail or 16
                        self.assertEqual(len(result.completed),expected,(base/'shell.log').read_text()+''.join(p.read_text() for p in logs.iterdir()))
                        self.assertEqual(result.shell_exit,124 if fault=='timeout' else 7 if native_fail else 9 if checker_fail else 0)
                        if fault=='timeout':
                            self.assertFalse((base/'checkers').exists())
                            terminal=document((plan.rows[0].evidence/'native-terminal.json').read_text())
                            raw_exit=terminal.field('native_exit').value
                            assert type(raw_exit) is int
                            self.assertLess(raw_exit,0)
                            self.assertTrue(terminal.field('timed_out').value)
                            self.assertIn('command exceeded the 300-second limit',plan.rows[0].log.read_text())
                        else:self.assertEqual(len((base/'checkers').read_text().splitlines()),expected-1 if native_fail else expected if checker_fail else 15)
                        _=controller.close_settled()
                    for receipt in (*evidence.glob('*/native-start.json'),*evidence.glob('*/native-terminal.json')):
                        actual_pid=document(receipt.read_text()).field('native_pid').value
                        if actual_pid is not None:
                            assert type(actual_pid) is int
                            with self.assertRaises(ProcessLookupError):os.kill(actual_pid,0)
                    if (base/'natives').exists():
                        for line in (base/'natives').read_text().splitlines():
                            pid=int(line.split()[0])
                            with self.assertRaises(ProcessLookupError):os.kill(pid,0)
                finally:
                    if shell.poll() is None:
                        if controller is not None:controller.cancel('test cleanup')
                        _=shell.wait(timeout=5)
                    if controller is None or not controller.closed:
                        if controller is not None and controller.control_write>=0:os.close(controller.control_write)
                        elif controller is None:os.close(control_write)
                        os.close(request_read)


class UninspectableFailureType(type):
    @override
    def __getattribute__(cls, name: str) -> object:
        if name=='__name__':raise RuntimeError('type name lookup failed')
        read_attribute: Callable[[str], object] = super().__getattribute__
        return read_attribute(name)


class UninspectableSuiteFailure(RuntimeError, metaclass=UninspectableFailureType):
    pass


class UnprintableSuiteFailure(RuntimeError):
    @override
    def __str__(self) -> str:
        raise TypeError('diagnostic rendering failed')


@contextmanager
def boundary_owner(*, signaled: bool=False) -> Generator[child.SuiteController]:
    """Tiny owned process boundary, not authenticated original-suite evidence."""
    with TemporaryDirectory() as directory:
        base=Path(directory).resolve()
        plan=child.SuitePlan(base/'plan','a'*64,base,base,base,(),(),())
        rr,rw=os.pipe();cr,cw=os.pipe()
        channels=child.SuiteChannels(rr,rw,cr,cw,plan.digest,os.getpid(),'b'*64)
        command=('/bin/bash','-c','kill -KILL $$') if signaled else (sys.executable,'-c','raise SystemExit(7)')
        with subprocess.Popen(command) as process:
            _=process.wait(timeout=5)
            owner=child.SuiteController(plan,process,-1,channels)
            owner.started=True
            try:
                yield owner
            finally:
                for descriptor in (rr,rw,cr,cw):
                    try:_=os.fstat(descriptor)
                    except OSError:continue
                    os.close(descriptor)


class SuiteFailureBoundaryTests(unittest.TestCase):
    def test_signaled_outer_shell_without_active_row_remains_unknown(self) -> None:
        with boundary_owner(signaled=True) as owner,patch.object(child.SuiteController,'current'):
            result=owner.snapshot()
            self.assertEqual(result.shell_exit,-9)
            self.assertEqual(result.settlement,'UNKNOWN')
            self.assertTrue(owner.halted)
            self.assertTrue(owner.error)
            self.assertEqual(owner.control_write,-1)
            with self.assertRaises(ValueError):_=owner.close_settled()

    def test_all_controller_failure_boundaries_cancel_before_formatting(self) -> None:
        for operation in ('start','snapshot','close_settled'):
            with self.subTest(operation=operation),boundary_owner() as owner:
                original=UnprintableSuiteFailure('primary')
                try:
                    if operation=='start':
                        owner.started=False
                        with patch.object(child,'suite_lane'),patch.object(child,'suite_current',side_effect=original):owner.start()
                    elif operation=='snapshot':
                        with patch.object(child.SuiteController,'current',side_effect=original):_=owner.snapshot()
                    else:
                        result=child.SuiteResult('SETTLED',owner.shell.pid,7,(),'')
                        with patch.object(child.SuiteController,'snapshot',return_value=result),patch.object(child,'atomic_json',side_effect=original):_=owner.close_settled()
                except (UnprintableSuiteFailure,TypeError,OSError) as actual:self.assertIs(actual,original)
                else:self.fail('original failure did not propagate')
                self.assertTrue(owner.halted)
                self.assertTrue(owner.error)
                self.assertEqual(owner.control_write,-1)

    def test_all_controller_boundaries_cancel_without_exception_inspection(self) -> None:
        for operation in ('start','service_ready','snapshot','close_settled'):
            with self.subTest(operation=operation),boundary_owner() as owner:
                original=UninspectableSuiteFailure('primary')
                try:
                    if operation=='start':
                        owner.started=False
                        with patch.object(child,'suite_lane'),patch.object(child,'suite_current',side_effect=original):owner.start()
                    elif operation=='service_ready':
                        with patch.object(child.SuiteController,'current',side_effect=original):owner.service_ready()
                    elif operation=='snapshot':
                        with patch.object(child.SuiteController,'current',side_effect=original):_=owner.snapshot()
                    else:
                        result=child.SuiteResult('SETTLED',owner.shell.pid,7,(),'')
                        with patch.object(child.SuiteController,'snapshot',return_value=result),patch.object(child,'atomic_json',side_effect=original):_=owner.close_settled()
                except (UninspectableSuiteFailure,RuntimeError,TypeError,OSError) as actual:self.assertIs(actual,original)
                else:self.fail('original failure did not propagate')
                self.assertTrue(owner.halted)
                self.assertTrue(owner.error)
                self.assertEqual(owner.control_write,-1)

    def test_unprintable_failure_cancels_before_diagnostic_and_preserves_original(self) -> None:
        for close_failure in (False,True):
            with self.subTest(close_failure=close_failure),boundary_owner() as owner:
                original=UnprintableSuiteFailure('primary')
                with patch.object(child.SuiteController,'current',side_effect=original):
                    try:
                        if close_failure:
                            with patch.object(os,'close',side_effect=OSError('close failed')):owner.service_ready()
                        else:owner.service_ready()
                    except (UnprintableSuiteFailure,TypeError,OSError) as actual:
                        self.assertIs(actual,original)
                    else:self.fail('original exception did not propagate')
                self.assertTrue(owner.halted)
                self.assertTrue(owner.error)
                self.assertEqual(owner.control_write,-1)
                with patch.object(child.SuiteController,'current'):
                    self.assertEqual(owner.snapshot().settlement,'UNKNOWN')


if __name__=='__main__':
    _=unittest.main()
