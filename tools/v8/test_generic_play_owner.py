"""Tiny synthetic launcher tree only; no actual play, Cargo, Git or Godot."""
import fcntl
import json
import os
import shutil
import sys
import unittest
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import gate_oracle_child
import gate_play_child
import generic_play_owner as owner
from generic_gate import GateContractError
from generic_process import atomic_json, file_hash
from mux2_gate_inputs import EvidenceError, Json, Node


class OwnerTests(unittest.TestCase):
    def fixture(self, base: Path, *, omit: bool = False) -> owner.PlayPlan:
        root = base / 'root'; scripts = root / 'tools/v8'; scripts.mkdir(parents=True)
        here = Path(__file__).parent
        upstream = Path(gate_oracle_child.__file__).parent
        modules = ('gate_oracle_child.py', 'gate_stall_child.py', 'generic_gate.py', 'generic_process.py',
            'generic_sources.py', 'generic_observations.py', 'mux2_gate_inputs.py', 'mux2_gate_lock.py')
        for name in modules:
            _ = shutil.copy2(upstream / name, scripts / name)
        for name in ('generic_play_owner.py', 'gate_play_protocol.py', 'gate_play_child.py', 'test_play.sh'):
            _ = shutil.copy2(here / name, scripts / name)
        python = str(Path(sys.executable).resolve())
        _ = (root / 'tools/play.sh').write_text('#!/bin/bash\nexec ' + python + ' "$(dirname "$0")/fixture.py" "$@"\n')
        fixture = '''import os,subprocess,sys,json
from pathlib import Path
root=Path(__file__).resolve().parent.parent
case=os.environ['WORLDSIM_GATE_PLAY_CASE']
if os.environ.get('GODOT_BIN')=='/nonexistent':sys.exit(2)
if sys.argv[1:]==['m1','--speed','7']:sys.exit(4)
env=json.loads((root/'child-env.json').read_text());env['WORLDSIM_GATE_PLAY_CASE']=case
for k in ['WORLDSIM_GATE_PLAY_REQUEST_FD','WORLDSIM_GATE_PLAY_CONTROL_FD','WORLDSIM_GATE_PLAY_OWNER_OUT','WORLDSIM_MUX2_LANE_FD']:env[k]=os.environ[k]
roles=['version'] if case=='unknown version' else ['version','build','import-1','import-2','headless']
if (root/'omit').exists():roles=[]
for role in roles:
 p=subprocess.run([sys.executable,str(root/'tools/v8/gate_play_child.py'),role,'--','/bin/sh','-c','exit 0'],cwd=root,env=env,pass_fds=tuple(int(env[k]) for k in ['WORLDSIM_GATE_PLAY_REQUEST_FD','WORLDSIM_GATE_PLAY_CONTROL_FD','WORLDSIM_MUX2_LANE_FD']))
 if p.returncode:sys.exit(p.returncode)
sys.exit(3 if case=='unknown version' else 0)
'''
        _ = (root / 'tools/fixture.py').write_text(fixture)
        if omit: _ = (root / 'omit').write_text('fixture omission')
        output = base / 'output'; cases: dict[str, Json] = {}
        for index, case in enumerate(gate_play_child.CASES):
            if index in (1, 2): cases[case] = None; continue
            roles: dict[str, Json] = {}
            for role in (('version',) if index == 0 else ('version', 'build', 'import-1', 'import-2', 'headless')):
                roles[role] = {'argv': ['/bin/sh', '-c', 'exit 0'], 'cwd': str(root),
                    'executable': str(Path('/bin/sh').resolve()), 'executable_sha256': file_hash(Path('/bin/sh')),
                    'inputs': {}, 'artifacts': [], 'output': str(base / f'native-{index}-{role}'),
                    'timeout_seconds': 600 if role in ('import-1', 'import-2', 'headless') else None,
                    'merged': role != 'version'}
            path = base / f'case-{index}.json'; _ = path.write_text(json.dumps({'schema_version': 1, 'roles': roles}))
            cases[case] = {'path': str(path), 'sha256': file_hash(path)}
        suite = base / 'suite.json'; _ = suite.write_text(json.dumps({'schema_version': 1, 'cases': cases}))
        env = {'PATH': str(Path(python).parent) + ':/usr/bin:/bin', 'HOME': str(base), 'LANG': 'C', 'LC_ALL': 'C',
            'PYTHONDONTWRITEBYTECODE': '1', 'PYTHONNOUSERSITE': '1', 'PLAY_ROOT': str(base / 'play'), 'PLAY_DATA': str(base / 'data')}
        child_env = {**env, 'WORLDSIM_GATE_PLAY_RECEIPTS': '1', 'WORLDSIM_GATE_PLAY_SUITE_PLAN': str(suite),
            'WORLDSIM_GATE_PLAY_SUITE_PLAN_SHA256': file_hash(suite)}
        if '__CF_USER_TEXT_ENCODING' in os.environ:
            child_env['__CF_USER_TEXT_ENCODING'] = os.environ['__CF_USER_TEXT_ENCODING']
        _ = (root / 'child-env.json').write_text(json.dumps(child_env))
        projections = tuple((case, role, tuple({**child_env, 'WORLDSIM_GATE_PLAY_CASE': case}.items()))
            for case in gate_play_child.CASES if case not in gate_play_child.CASES[1:3]
            for role in (('version',) if case == gate_play_child.CASES[0] else ('version', 'build', 'import-1', 'import-2', 'headless')))
        paths = [p for p in root.rglob('*') if p.is_file()] + [Path('/bin/bash').resolve(), Path(python)]
        inputs = tuple((p, file_hash(p)) for p in paths)
        return owner.PlayPlan(root, output, suite, file_hash(suite), inputs, tuple(env.items()), projections)

    def test_full_original_outer_with_nested_actual_waits(self) -> None:
        with TemporaryDirectory() as folder:
            base = Path(folder).resolve(); plan = self.fixture(base)
            lock = base / 'lane'; lock.touch()
            with lock.open('r') as held, patch.object(gate_oracle_child, 'LOCK', lock):
                fcntl.flock(held.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                live = owner.PlayOwner(plan, held.fileno())
                try:
                    live.launch(); result = live.wait()
                except owner.PlayOwnershipUnknown:
                    print((plan.output / 'stdout.log').read_text(), (plan.output / 'stderr.log').read_text())
                    for path in (plan.output / 'protocol').rglob('*.json'):
                        print(path.name, path.read_text())
                    # Fixture-only closure: no workload child is granted after a fault.
                    for descriptor in (live.control_write, live.request_read):
                        if descriptor >= 0: os.close(descriptor)
                    if live.shell is not None: _ = live.shell.wait(timeout=10)
                    os.close(live.lane_fd)
                    if live.stdout is not None: live.stdout.close()
                    if live.stderr is not None: live.stderr.close()
                    raise
            self.assertEqual(result.case_exits, (3, 2, 4, 0, 0, 0))
            self.assertEqual((result.outer_exit, result.qualification), (0, 'INCOMPLETE'))
            self.assertEqual(len(list((plan.output / 'protocol').glob('case-*/returned.json'))), 6)
            self.assertEqual(len(list((plan.output / 'protocol').glob('case-*/*/returned.json'))), 16)

    def test_missing_child_wait_retains_real_lane_and_live_owner(self) -> None:
        with TemporaryDirectory() as folder:
            base = Path(folder).resolve(); plan = self.fixture(base, omit=True)
            lock = base / 'lane'; lock.touch()
            with lock.open('r') as held, patch.object(gate_oracle_child, 'LOCK', lock):
                fcntl.flock(held.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                live = owner.PlayOwner(plan, held.fileno()); live.launch()
                try:
                    with self.assertRaises(owner.PlayOwnershipUnknown) as caught:
                        _ = live.wait()
                    self.assertIs(caught.exception.owner, live)
                    self.assertFalse(live.closed)
                    self.assertEqual(os.fstat(live.lane_fd).st_ino, lock.stat().st_ino)
                    held.close()
                    with lock.open('r') as contender, self.assertRaises(BlockingIOError):
                        fcntl.flock(contender.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                    self.assertFalse((plan.output / 'terminal.json').exists())
                    self.assertIsNotNone(live.active_case)
                finally:
                    # Only this synthetic fixture: no native child was requested/granted.
                    for descriptor in (live.control_write, live.request_read):
                        if descriptor >= 0: os.close(descriptor)
                    if live.shell is not None: _ = live.shell.wait(timeout=10)
                    os.close(live.lane_fd)
                    if live.stdout is not None: live.stdout.close()
                    if live.stderr is not None: live.stderr.close()

    def test_spawn_interrupt_and_diagnostic_fault_keep_original_and_lane(self) -> None:
        with TemporaryDirectory() as folder:
            base = Path(folder).resolve(); plan = self.fixture(base)
            lock = base / 'lane'; lock.touch()
            with lock.open('r') as held, patch.object(gate_oracle_child, 'LOCK', lock):
                fcntl.flock(held.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                live = owner.PlayOwner(plan, held.fileno())
                original = atomic_json
                def publish(path: Path, value: dict[str, Json]) -> None:
                    if path.name == 'unknown.json': raise OSError('diagnostic fixture')
                    original(path, value)
                primary = KeyboardInterrupt('fixture before spawn handle')
                try:
                    with patch('generic_play_owner.subprocess.Popen', side_effect=primary), \
                            patch.object(owner, 'atomic_json', side_effect=publish), \
                            self.assertRaises(owner.PlayOwnershipUnknown) as caught:
                        live.launch()
                    self.assertIs(caught.exception.__cause__, primary)
                    self.assertIs(caught.exception.owner, live)
                    self.assertIsInstance(live.diagnostic_error, OSError)
                    self.assertIsNone(live.shell)
                    self.assertTrue((plan.output / 'intent.json').is_file())
                    held.close()
                    with lock.open('r') as contender, self.assertRaises(BlockingIOError):
                        fcntl.flock(contender.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                finally:
                    for descriptor in (live.request_read, live.request_write, live.control_read, live.control_write, live.lane_fd):
                        if descriptor >= 0: os.close(descriptor)
                    if live.stdout is not None: live.stdout.close()
                    if live.stderr is not None: live.stderr.close()

    def test_terminal_write_failure_retains_waited_handle_and_lane(self) -> None:
        with TemporaryDirectory() as folder:
            base = Path(folder).resolve(); plan = self.fixture(base)
            lock = base / 'lane'; lock.touch()
            with lock.open('r') as held, patch.object(gate_oracle_child, 'LOCK', lock):
                fcntl.flock(held.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                live = owner.PlayOwner(plan, held.fileno())
                def publish(path: Path, value: dict[str, Json]) -> None:
                    if path.name == 'terminal.json': raise OSError('final terminal fixture')
                    atomic_json(path, value)
                try:
                    live.launch()
                    with patch.object(owner, 'atomic_json', side_effect=publish), self.assertRaises(owner.PlayOwnershipUnknown) as caught:
                        _ = live.wait()
                    self.assertIsInstance(caught.exception.__cause__, OSError)
                    self.assertIsNotNone(live.shell)
                    assert live.shell is not None
                    self.assertEqual(live.shell.returncode, 0)
                    self.assertEqual(len(live.case_exits), 6)
                    self.assertFalse(live.closed)
                    held.close()
                    with lock.open('r') as contender, self.assertRaises(BlockingIOError):
                        fcntl.flock(contender.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                finally:
                    for descriptor in (live.control_write, live.request_read, live.lane_fd):
                        if descriptor >= 0: os.close(descriptor)
                    if live.stdout is not None: live.stdout.close()
                    if live.stderr is not None: live.stderr.close()

    def test_wrong_child_environment_never_grants_native(self) -> None:
        with TemporaryDirectory() as folder:
            base = Path(folder).resolve(); original = self.fixture(base)
            projections = list(original.child_environments)
            case, role, values = projections[0]
            projections[0] = (case, role, (*values, ('UNPLANNED', 'yes')))
            plan = replace(original, child_environments=tuple(projections))
            lock = base / 'lane'; lock.touch()
            with lock.open('r') as held, patch.object(gate_oracle_child, 'LOCK', lock):
                fcntl.flock(held.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                live = owner.PlayOwner(plan, held.fileno())
                try:
                    live.launch()
                    with self.assertRaises(owner.PlayOwnershipUnknown): _ = live.wait()
                    self.assertFalse((base / 'native-0-version').exists())
                    self.assertFalse(live.closed)
                finally:
                    for descriptor in (live.control_write, live.request_read):
                        if descriptor >= 0: os.close(descriptor)
                    if live.shell is not None: _ = live.shell.wait(timeout=10)
                    os.close(live.lane_fd)
                    if live.stdout is not None: live.stdout.close()
                    if live.stderr is not None: live.stderr.close()

    def test_completed_native_membership_and_bytes_remain_frozen(self) -> None:
        for change in ('file', 'empty-directory', 'bytes'):
            with self.subTest(change=change):
                self.assert_completed_native_change_retained(change)

    def assert_completed_native_change_retained(self, change: str) -> None:
        with TemporaryDirectory() as folder:
            base = Path(folder).resolve(); plan = self.fixture(base)
            lock = base / 'lane'; lock.touch()
            original_message = owner.PlayOwner.message
            changed = False
            def message(live: owner.PlayOwner, notification: Node) -> None:
                nonlocal changed
                original_message(live, notification)
                if live.roles == ['version'] and not live.case_exits and not changed:
                    changed = True
                    output = base / 'native-0-version'
                    if change == 'file': _ = (output / 'late.txt').write_text('late')
                    elif change == 'empty-directory': (output / 'late' / 'empty').mkdir(parents=True)
                    else: _ = (output / 'stdout.log').write_text('mutated')
            with lock.open('r') as held, patch.object(gate_oracle_child, 'LOCK', lock):
                fcntl.flock(held.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                live = owner.PlayOwner(plan, held.fileno())
                try:
                    live.launch()
                    with patch.object(owner.PlayOwner, 'message', message), self.assertRaises(owner.PlayOwnershipUnknown):
                        _ = live.wait()
                    self.assertTrue(changed)
                    self.assertFalse(live.closed)
                    held.close()
                    with lock.open('r') as contender, self.assertRaises(BlockingIOError):
                        fcntl.flock(contender.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                    self.assertFalse((plan.output / 'terminal.json').exists())
                finally:
                    if not live.closed:
                        for descriptor in (live.control_write, live.request_read):
                            if descriptor >= 0: os.close(descriptor)
                        if live.shell is not None: _ = live.shell.wait(timeout=10)
                        os.close(live.lane_fd)
                        if live.stdout is not None: live.stdout.close()
                        if live.stderr is not None: live.stderr.close()


    def test_final_check_rejects_empty_directory_after_all_waits(self) -> None:
        with TemporaryDirectory() as folder:
            base = Path(folder).resolve(); plan = self.fixture(base)
            lock = base / 'lane'; lock.touch()
            def publish(path: Path, value: dict[str, Json]) -> None:
                atomic_json(path, value)
                if path.name == 'terminal.json':
                    (base / 'native-0-version' / 'late-empty').mkdir()
            with lock.open('r') as held, patch.object(gate_oracle_child, 'LOCK', lock):
                fcntl.flock(held.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                live = owner.PlayOwner(plan, held.fileno())
                try:
                    live.launch()
                    with patch.object(owner, 'atomic_json', side_effect=publish), self.assertRaises(owner.PlayOwnershipUnknown):
                        _ = live.wait()
                    self.assertEqual(len(live.case_exits), 6)
                    self.assertFalse(live.closed)
                    self.assertTrue((plan.output / 'unknown.json').exists())
                    held.close()
                    with lock.open('r') as contender, self.assertRaises(BlockingIOError):
                        fcntl.flock(contender.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                finally:
                    for descriptor in (live.control_write, live.request_read, live.lane_fd):
                        if descriptor >= 0: os.close(descriptor)
                    if live.stdout is not None: live.stdout.close()
                    if live.stderr is not None: live.stderr.close()

    def test_no_launch_rejects_unbound_source(self) -> None:
        with TemporaryDirectory() as folder:
            base = Path(folder).resolve(); plan = self.fixture(base)
            _ = (plan.root / 'tools/play.sh').write_text('changed')
            with self.assertRaises((GateContractError, EvidenceError)): owner.validate(plan)
            self.assertFalse(plan.output.exists())
