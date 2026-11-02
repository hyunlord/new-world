"""Opt-in direct-child receipts for the unchanged 960-second stall wrapper."""
import hashlib
import os
import select
import shutil
import stat
import subprocess
import sys
import threading
import time
from collections.abc import Callable
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path

from generic_gate import boundary, document, integer, require, text
from generic_process import atomic_json, check_digest, file_hash, real_path
from mux2_gate_inputs import BRIDGE, Node
from mux2_gate_lock import FD_ENV, LOCK

PLAN_ENV = 'WORLDSIM_STALL_CHILD_PLAN'
HASH_ENV = 'WORLDSIM_STALL_CHILD_PLAN_SHA256'
CONTROL_ENV = 'WORLDSIM_STALL_CHILD_CONTROL_FD'


@dataclass(frozen=True, slots=True)
class ChildPlan:
    path: Path
    digest: str
    root: Path
    capture: Path
    evidence: Path
    argv: tuple[str, ...]
    inputs: tuple[tuple[Path, str], ...]


def load_plan(path: Path, digest: str, argv: tuple[str, ...], limit: int) -> ChildPlan:
    real_path(path, directory=False)
    check_digest(digest, '/plan')
    require(file_hash(path) == digest, '/plan', 'plan changed')
    node = document(path.read_text())
    require(set(node.mapping()) == {'schema', 'protocol', 'root', 'capture', 'evidence', 'argv', 'inputs'},
            '/plan', 'unknown or missing fields')
    node.field('schema').equals('stall_bounded_child_v1')
    node.field('protocol').equals('NONE')
    root = Path(text(node.field('root')))
    capture = Path(text(node.field('capture')))
    evidence = Path(text(node.field('evidence')))
    for directory in (root, capture, evidence):
        real_path(directory, directory=True)
    require(all(not a.is_relative_to(b) and not b.is_relative_to(a)
                for i, a in enumerate((root, capture, evidence)) for b in (root, capture, evidence)[i+1:]),
            '/roots', 'overlapping roots')
    expected = tuple(text(value) for value in node.field('argv').array())
    require(bool(expected), '/argv', 'missing executable')
    recipe = (expected[0], '--path', str(root), '--resolution', '1920x1080', '--always-on-top',
              '--windowed', '--script', 'res://tools/v8/mux2_stall_test.gd', '--', str(capture/'run'))
    require(argv == expected == recipe and limit == 960 and Path.cwd() == root,
            '/recipe', 'not original NONE stall recipe')
    require(not any(key.startswith('CORE_P7_') for key in os.environ), '/protocol', 'P7 forbidden')
    require(os.environ.get('WORLDSIM_TEST_SETTINGS_DIR') == str(capture/'settings')
            and os.environ.get('WORLDSIM_MUX2_PROFILE') == '1', '/environment', 'original settings/profile required')
    inputs = tuple((Path(name), text(Node(value, '/inputs/'+name))) for name, value in node.field('inputs').mapping().items())
    required = {Path(expected[0]), root/'tools/v8/run_bounded.sh', root/'tools/v8/gate_stall_child.py',
                root/'tools/v8/mux2_stall_test.gd', root/'sim_bridge.gdextension', root/BRIDGE, Path(sys.executable).resolve()}
    require(required <= {p for p, _ in inputs}, '/inputs', 'tool or script binding missing')
    for source, sha in inputs:
        check_digest(sha, str(source))
        require(not source.is_relative_to(capture) and not source.is_relative_to(evidence), '/inputs', 'evidence input overlap')
        require(file_hash(source) == sha, str(source), 'input changed')
    return ChildPlan(path, digest, root, capture, evidence, argv, inputs)


def stream_position(path: Path) -> int:
    """Inspect inherited regular-file endpoints without moving their shared offsets."""
    _ = sys.stdout.flush()
    _ = sys.stderr.flush()
    disk = path.stat()
    require(stat.S_ISREG(disk.st_mode), '/stream', 'regular merged log required')
    for descriptor in (1, 2):
        observed = os.fstat(descriptor)
        require((observed.st_dev, observed.st_ino) == (disk.st_dev, disk.st_ino),
                '/stream', 'stdout/stderr do not name planned merged log')
        require(os.lseek(descriptor, 0, os.SEEK_CUR) == disk.st_size, '/stream', 'writer is not at log end')
    return disk.st_size


def range_hash(path: Path, start: int, end: int) -> str:
    require(0 <= start <= end, '/stream', 'invalid byte range')
    digest = hashlib.sha256()
    with path.open('rb', buffering=0) as reader:
        require(end <= os.fstat(reader.fileno()).st_size, '/stream', 'range beyond log')
        position = start
        while position < end:
            block = os.pread(reader.fileno(), min(65536, end-position), position)
            require(bool(block), '/stream', 'short byte range')
            digest.update(block)
            position += len(block)
    return digest.hexdigest()


def supervise(argv: tuple[str, ...], limit: int, started: Callable[[int], None],
              control_fd: int | None = None) -> tuple[int, bool]:
    """Reap only the direct child; parent disconnect cannot orphan it."""
    stopped = threading.Event()
    cancelled = threading.Event()
    guard = threading.Lock()
    owned: subprocess.Popen[bytes] | None = None
    def monitor() -> None:
        assert control_fd is not None
        while not stopped.is_set():
            ready, _, _ = select.select((control_fd,), (), (), 0.05)
            if ready:
                _ = os.read(control_fd, 1)
                cancelled.set()
                with guard:
                    if owned is not None and owned.poll() is None:
                        owned.kill()
                return
    worker = threading.Thread(target=monitor) if control_fd is not None else None
    try:
        if worker is not None:
            worker.start()
        with guard:
            require(not cancelled.is_set(), '/control', 'capture parent disconnected before spawn')
            owned = subprocess.Popen(argv)
        with owned as process:
            try:
                started(process.pid)
                code = process.wait(timeout=limit)
                require(not cancelled.is_set(), '/control', 'capture parent disconnected')
                return code, False
            except subprocess.TimeoutExpired:
                process.kill()
                return process.wait(), True
            except BaseException:
                process.kill()
                _ = process.wait()
                raise
    finally:
        stopped.set()
        if worker is not None and worker.ident is not None:
            worker.join()
        if control_fd is not None:
            os.close(control_fd)


def run(path: Path, digest: str, argv: tuple[str, ...], limit: int) -> int:
    plan = load_plan(path, digest, argv, limit)
    lock_fd = int(os.environ[FD_ENV])
    held = os.fstat(lock_fd)
    canonical = LOCK.stat()
    require((held.st_dev, held.st_ino) == (canonical.st_dev, canonical.st_ino), '/lane', 'foreign inherited lane')
    log = plan.capture/'command.log'
    start = stream_position(log)
    began = time.monotonic_ns()
    before = datetime.now(UTC).isoformat()
    wrapper_environment = dict(os.environ)
    control_fd = int(os.environ.pop(CONTROL_ENV))
    require(stat.S_ISFIFO(os.fstat(control_fd).st_mode), '/control', 'anonymous control pipe required')
    require(os.read(control_fd, 1) == b'G', '/control', 'capture startup not authorized')
    environment = dict(os.environ)
    native_pid: int | None = None
    code: int | None = None
    timed_out = False
    errors: list[str] = []
    def started(pid: int) -> None:
        nonlocal native_pid
        native_pid = pid
        atomic_json(plan.evidence/'native-start.json', {'schema':'stall_bounded_child_v1',
                    'capture_pid':os.getppid(), 'wrapper_pid':os.getpid(), 'native_pid':pid,
                    'argv':list(argv), 'cwd':str(Path.cwd()), 'environment':dict(environment),
                    'wrapper_environment':dict(wrapper_environment),
                    'plan_sha256':digest, 'python':str(Path(sys.executable).resolve()),
                    'inputs':{str(p):sha for p,sha in plan.inputs},
                    'lane_device':held.st_dev, 'lane_inode':held.st_ino,
                    'stream_start':start, 'started_at':before, 'started_monotonic_ns':began})
    try:
        code, timed_out = supervise(argv, limit, started, control_fd)
    except BaseException as error:
        errors.append(type(error).__name__+': '+str(error))
        raise
    finally:
        # supervise has left the Popen context before this endpoint is sampled.
        end = stream_position(log)
        for source, sha in (*plan.inputs, (path, digest)):
            try:
                require(file_hash(source) == sha, str(source), 'input changed')
            except (OSError, ValueError) as error:
                errors.append(str(error))
        ended = time.monotonic_ns()
        atomic_json(plan.evidence/'native-terminal.json', {'schema':'stall_bounded_child_v1',
            'capture_pid':os.getppid(), 'wrapper_pid':os.getpid(), 'native_pid':native_pid,
            'argv':list(argv), 'cwd':str(Path.cwd()), 'environment':dict(environment),
                    'wrapper_environment':dict(wrapper_environment),
            'environment_policy':'inherited_capture_wrapper_environment', 'plan_sha256':digest,
            'python':str(Path(sys.executable).resolve()), 'inputs':{str(p):sha for p,sha in plan.inputs},
            'lane_device':held.st_dev, 'lane_inode':held.st_ino,
            'native_exit':code, 'timed_out':timed_out, 'timeout_seconds':limit,
            'timeout_owner_pid':os.getpid(), 'wrapper_exit':124 if timed_out else (code % 256 if code is not None else None),
            'started_at':before, 'ended_at':datetime.now(UTC).isoformat(),
            'started_monotonic_ns':began, 'ended_monotonic_ns':ended, 'duration_ns':ended-began, 'stream_start':start, 'stream_end':end,
            'stream_sha256':range_hash(log,start,end), 'errors':[e for e in errors]})
    require(not errors, '/child', 'integrity failure')
    if timed_out:
        print(f'command exceeded the {limit}-second limit', file=sys.stderr)
        return 124
    assert code is not None
    return code


def capture_wrapper(command: list[str], root: Path, environment: dict[str, str], log_fd: int, lane_fd: int) -> int:
    """Capture parent records its real direct wrapper without adding a supervisor."""
    path = Path(environment[PLAN_ENV])
    digest = environment[HASH_ENV]
    check_digest(digest, '/plan')
    require(file_hash(path) == digest, '/plan', 'plan changed')
    plan = document(path.read_text())
    plan.field('root').equals(str(root))
    expected = ['bash', str(root/'tools/v8/run_bounded.sh'), '960',
                *(text(value) for value in plan.field('argv').array())]
    require(command == expected, '/command', 'foreign bounded wrapper')
    evidence = Path(text(plan.field('evidence')))
    real_path(evidence, directory=True)
    bash_name = shutil.which(command[0], path=environment.get('PATH'))
    require(bash_name is not None, '/bash', 'missing wrapper shell')
    assert bash_name is not None
    bash = Path(bash_name).resolve(strict=True)
    plan.field('inputs').field(str(bash)).equals(file_hash(bash))
    before = datetime.now(UTC).isoformat()
    begin = time.monotonic_ns()
    code: int | None = None
    pid: int | None = None
    errors: list[str] = []
    control_read, control_write = os.pipe()
    launch_environment = dict(environment, **{CONTROL_ENV:str(control_read)})
    try:
        with subprocess.Popen(command, cwd=root, env=launch_environment, stdout=log_fd,
                              stderr=subprocess.STDOUT, pass_fds=(lane_fd, control_read)) as process:
            pid = process.pid
            os.close(control_read)
            control_read = -1
            try:
                atomic_json(evidence/'wrapper-start.json', {'schema':'stall_bounded_child_v1', 'capture_pid':os.getpid(), 'wrapper_pid':pid,
                    'bash':str(bash), 'bash_sha256':file_hash(bash), 'argv':list(command), 'cwd':str(root), 'environment':dict(environment),
                    'wrapper_environment':dict(launch_environment),
                    'plan_sha256':digest, 'started_at':before, 'started_monotonic_ns':begin})
                _ = os.write(control_write, b'G')
                code = process.wait()
            except BaseException:
                os.close(control_write)
                control_write = -1
                _ = process.wait()
                raise
    except BaseException as error:
        errors.append(type(error).__name__+': '+str(error))
        raise
    finally:
        if control_read >= 0:
            os.close(control_read)
        if control_write >= 0:
            os.close(control_write)
        ended = time.monotonic_ns()
        atomic_json(evidence/'wrapper-terminal.json', {'schema':'stall_bounded_child_v1', 'capture_pid':os.getpid(), 'wrapper_pid':pid,
            'bash':str(bash), 'bash_sha256':file_hash(bash), 'argv':list(command), 'cwd':str(root), 'environment':dict(environment),
                    'wrapper_environment':dict(launch_environment), 'plan_sha256':digest,
            'native_exit':code, 'started_at':before, 'ended_at':datetime.now(UTC).isoformat(),
            'started_monotonic_ns':begin, 'ended_monotonic_ns':ended, 'duration_ns':ended-begin, 'errors':[error for error in errors]})
    assert code is not None
    return code


def close_capture(environment: dict[str, str], log: Path) -> None:
    """Called only after the capture owner closed its command.log writer."""
    path = Path(environment[PLAN_ENV])
    digest = environment[HASH_ENV]
    require(file_hash(path) == digest, '/plan', 'plan changed')
    plan = document(path.read_text())
    require(log == Path(text(plan.field('capture')))/'command.log', '/log', 'foreign log')
    evidence = Path(text(plan.field('evidence')))
    atomic_json(evidence/'capture-log-closed.json', {'schema':'stall_bounded_child_v1', 'capture_pid':os.getpid(), 'plan_sha256':digest,
                'path':str(log), 'sha256':file_hash(log), 'size':log.stat().st_size,
                'closed_at':datetime.now(UTC).isoformat()})


def projected_environment(root: Path, supplied: dict[str, str]) -> dict[str, str]:
    """Measured pinned macOS Bash wrapper projection, not a general shell waiver."""
    result = dict(supplied)
    _ = result.pop('OLDPWD', None)
    _ = result.pop('_', None)
    result['PWD'] = str(root)
    result['SHLVL'] = str(int(result.get('SHLVL', '0')))
    return result


@boundary
def bind_chain(plan: ChildPlan, expected_environment: dict[str, str], expected_exit: int,
               expected_files: tuple[tuple[str, str], ...]) -> int:
    """Authenticate a frozen native/parent chain, without granting timing authority."""
    require(file_hash(plan.path) == plan.digest, '/plan', 'plan changed')
    declared = document(plan.path.read_text())
    declared.field('schema').equals('stall_bounded_child_v1')
    declared.field('protocol').equals('NONE')
    declared.field('root').equals(str(plan.root))
    declared.field('capture').equals(str(plan.capture))
    declared.field('evidence').equals(str(plan.evidence))
    declared.field('argv').equals(list(plan.argv))
    declared.field('inputs').equals({str(p):sha for p,sha in plan.inputs})
    require(expected_exit in (0, 1), '/exit', 'only recomputed stall outcomes')
    files = dict(expected_files)
    require(len(files) == len(expected_files) and set(files) == {
        'wrapper-start.json', 'wrapper-terminal.json', 'native-start.json',
        'native-terminal.json', 'capture-log-closed.json'}, '/chain', 'incomplete chain inventory')
    require({p.name for p in plan.evidence.iterdir()} == set(files), '/chain', 'untracked chain output')
    for name, digest in files.items():
        require(file_hash(plan.evidence/name) == digest, '/chain', 'receipt changed')
    wrapper_start = document((plan.evidence/'wrapper-start.json').read_text())
    wrapper = document((plan.evidence/'wrapper-terminal.json').read_text())
    native_start = document((plan.evidence/'native-start.json').read_text())
    native = document((plan.evidence/'native-terminal.json').read_text())
    closed = document((plan.evidence/'capture-log-closed.json').read_text())
    capture_pid = integer(wrapper.field('capture_pid'))
    wrapper_pid = integer(wrapper.field('wrapper_pid'))
    native_pid = integer(native.field('native_pid'))
    require(len({capture_pid, wrapper_pid, native_pid}) == 3 and min(capture_pid, wrapper_pid, native_pid) > 0,
            '/pid', 'invalid process chain')
    expected_wrapper = ['bash', str(plan.root/'tools/v8/run_bounded.sh'), '960', *plan.argv]
    for row in (wrapper_start, wrapper):
        bash = Path(text(row.field('bash')))
        require(bash in dict(plan.inputs), '/bash', 'unbound shell')
        row.field('bash_sha256').equals(dict(plan.inputs)[bash])
        row.field('capture_pid').equals(capture_pid)
        row.field('wrapper_pid').equals(wrapper_pid)
        row.field('argv').equals([value for value in expected_wrapper])
        row.field('cwd').equals(str(plan.root))
        row.field('environment').equals(dict(expected_environment))
    for row in (native_start, native):
        row.field('capture_pid').equals(capture_pid)
        row.field('wrapper_pid').equals(wrapper_pid)
        row.field('native_pid').equals(native_pid)
        row.field('argv').equals(list(plan.argv))
        row.field('cwd').equals(str(plan.root))
        row.field('inputs').equals({str(p):sha for p,sha in plan.inputs})
        python = Path(text(row.field('python')))
        require(python in dict(plan.inputs), '/python', 'unbound wrapper interpreter')
        current_lock = LOCK.stat()
        row.field('lane_device').equals(current_lock.st_dev)
        row.field('lane_inode').equals(current_lock.st_ino)
    for row in (wrapper_start, wrapper, native_start, native, closed):
        row.field('schema').equals('stall_bounded_child_v1')
        row.field('plan_sha256').equals(plan.digest)
    environment = {k: Node(v, '/environment/'+k).text() for k,v in native.field('environment').mapping().items()}
    native_start.field('environment').equals(dict(environment))
    launch = dict(expected_environment)
    control = wrapper.field('wrapper_environment').field(CONTROL_ENV).text()
    require(control.isdecimal(), '/control', 'invalid control descriptor')
    launch[CONTROL_ENV] = control
    for row in (wrapper_start, wrapper):
        row.field('wrapper_environment').equals(dict(launch))
    # Bash sets these three values during the existing exec transition.
    expected = projected_environment(plan.root, expected_environment)
    observed_wrapper = dict(expected, **{CONTROL_ENV:control})
    for row in (native_start, native):
        row.field('wrapper_environment').equals(dict(observed_wrapper))
    require(not any(key.startswith('CORE_P7_') for key in environment), '/protocol', 'P7 environment forbidden')
    require(environment.get('WORLDSIM_TEST_SETTINGS_DIR') == str(plan.capture/'settings')
            and environment.get('WORLDSIM_MUX2_PROFILE') == '1'
            and environment.get(PLAN_ENV) == str(plan.path) and environment.get(HASH_ENV) == plan.digest,
            '/environment', 'wrong native settings/profile/plan destinations')
    require(environment == expected, '/environment', 'unqualified shell environment transformation: '+','.join(sorted(k for k in set(environment)|set(expected) if environment.get(k)!=expected.get(k))))
    native.field('environment_policy').equals('inherited_capture_wrapper_environment')
    native.field('timed_out').equals(False)
    native.field('timeout_seconds').equals(960)
    native.field('timeout_owner_pid').equals(wrapper_pid)
    for row in (native, wrapper):
        row.field('native_exit').equals(expected_exit)
        row.field('errors').equals([])
    native.field('wrapper_exit').equals(expected_exit)
    begin = integer(native.field('stream_start'))
    end = integer(native.field('stream_end'))
    native_start.field('stream_start').equals(begin)
    log = plan.capture/'command.log'
    closed.field('capture_pid').equals(capture_pid)
    closed.field('path').equals(str(log))
    closed.field('size').equals(log.stat().st_size)
    closed.field('sha256').equals(file_hash(log))
    native.field('stream_sha256').equals(range_hash(log, begin, end))
    for start_row, end_row in ((wrapper_start,wrapper),(native_start,native)):
        start_row.field('started_at').equals(end_row.field('started_at').value)
        start_ns = integer(end_row.field('started_monotonic_ns'))
        end_ns = integer(end_row.field('ended_monotonic_ns'))
        duration = integer(end_row.field('duration_ns'))
        require(start_ns > 0 and end_ns > start_ns and duration == end_ns-start_ns, '/duration', 'invalid monotonic duration')
        start_row.field('started_monotonic_ns').equals(start_ns)
    times = [datetime.fromisoformat(text(row.field(key))) for row,key in (
        (wrapper,'started_at'),(native,'started_at'),(native,'ended_at'),(wrapper,'ended_at'),(closed,'closed_at'))]
    require(all(t.tzinfo is not None for t in times) and times == sorted(times), '/time', 'invalid chain ordering')
    for source,digest in (*plan.inputs,(plan.path,plan.digest)):
        require(file_hash(source) == digest, '/input', 'source or plan changed')
    for name,digest in files.items():
        require(file_hash(plan.evidence/name) == digest, '/chain', 'receipt changed during join')
    return native_pid
