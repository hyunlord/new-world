"""Real tiny-process ownership regressions; no Godot or WorldSim execution."""
import os
import shutil
import subprocess
import sys
import time
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import gate_oracle_child as child
from generic_gate import GateContractError, document, integer
from generic_process import atomic_json, file_hash
from mux2_gate_inputs import Json

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


if __name__=='__main__':
    _=unittest.main()
