#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 run_bound_gate.py FULL_HEAD OUTPUT_DIR RECEIPT_JSON [REPO_DIR]

from collections.abc import Sequence
from contextlib import suppress
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import time
from types import FrameType
from typing import Final

STEPS: Final = (
    'prepare_bridge', 'prepare_import', 'test_debug', 'test_release', 'clippy',
    'clippy_all_targets', 'forbidden_math', 'bridge_build', 'godot_import',
    'strict_check', 'smoke', 'python_unittest', 'python_worldmodel', 'perf_quick',
    'allowed_paths',
)
DEFAULT_REPO: Final = Path('/Users/rexxa/work/new-world-pass4')
SOURCE_SUFFIXES: Final = frozenset({'.rs', '.py', '.pyi', '.sh', '.bash', '.gd',
                                   '.tscn', '.tres', '.ron', '.toml', '.js', '.ts'})


class GateError(RuntimeError):
    pass


@dataclass(frozen=True, slots=True)
class Snapshot:
    head: str
    staged: tuple[str, ...]
    unstaged: tuple[str, ...]
    untracked: tuple[str, ...]
    ignored_source: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class Artifact:
    path: str
    bytes: int
    sha256: str


def artifact(path: Path) -> Artifact:
    if path.is_symlink() or not path.is_file():
        raise GateError(f'expected regular file: {path}')
    with path.open('rb') as stream:
        digest = hashlib.file_digest(stream, 'sha256').hexdigest()
    return Artifact(str(path), path.stat().st_size, digest)


def git(repo: Path, *args: str) -> str:
    return subprocess.check_output(['git', *args], cwd=repo, text=True)


def snapshot(repo: Path) -> Snapshot:
    """Include both index/worktree dirtiness and discoverable ignored source."""
    def paths(*args: str) -> tuple[str, ...]:
        return tuple(filter(None, git(repo, *args, '-z', '--', 'rust', 'tools').split('\0')))
    ignored = paths('ls-files', '--others', '--ignored', '--exclude-standard')
    discoverable = tuple(p for p in ignored if not p.startswith('rust/target/')
                         and '__pycache__' not in Path(p).parts
                         and Path(p).suffix in SOURCE_SUFFIXES)
    return Snapshot(git(repo, 'rev-parse', 'HEAD').strip(),
                    paths('diff', '--cached', '--name-only'),
                    paths('diff', '--name-only'),
                    paths('ls-files', '--others', '--exclude-standard'), discoverable)


def require_snapshot(state: Snapshot, expected: str) -> None:
    if state.head != expected:
        raise GateError(f'HEAD mismatch: expected {expected}, found {state.head}')
    if state.staged or state.unstaged or state.untracked or state.ignored_source:
        raise GateError('rust/tools source or test discovery is dirty: ' + json.dumps(asdict(state)))


def stamp() -> tuple[str, float]:
    epoch = time.time()
    return datetime.fromtimestamp(epoch, timezone.utc).isoformat().replace('+00:00', 'Z'), epoch


def gate_errors(output: Path, expected: str) -> list[str]:
    """Validate the exact step order, statuses, terminal marker and smoke proof."""
    lines = (output / 'gate.txt').read_text().splitlines()
    errors: list[str] = []
    rows = [m.groups() for line in lines if (m := re.fullmatch(r'(\w+) exit (-?\d+)', line))]
    if rows != [(name, '0') for name in STEPS]:
        errors.append('expected exactly the 15 ordered step exits, all zero')
    if [line for line in lines if line.startswith('head ')] != ['head ' + expected]:
        errors.append('gate.txt HEAD differs from expected HEAD')
    if [line for line in lines if line.startswith('smoke_errors ')] != ['smoke_errors none']:
        errors.append('smoke_errors none is absent or contradicted')
    finished = [line for line in lines if line.startswith('finished ')]
    if len(finished) != 1 or finished[0] != lines[-1] or not finished[0].endswith(' status 0'):
        errors.append('missing unique terminal finished status 0')
    if any('wait_failed' in line for line in lines):
        errors.append('a lock guard failed')
    for name in STEPS:
        if not (output / (name + '.log')).is_file():
            errors.append(f'missing step log: {name}')
    smoke = output / 'smoke_errors.log'
    if not smoke.is_file() or smoke.read_bytes():
        errors.append('smoke_errors.log is missing or nonempty')
    return errors


def invoke(command: list[str], repo: Path, output: Path) -> tuple[int, int | None]:
    """Terminate only the process group launched here if the wrapper is signaled."""
    interrupted: list[int] = []
    process: subprocess.Popen[bytes] | None = None
    def stop(signum: int, _frame: FrameType | None) -> None:
        interrupted.append(signum)
        if process is not None:
            with suppress(ProcessLookupError):
                os.killpg(process.pid, signal.SIGTERM)
    old_handlers = {sig: signal.signal(sig, stop) for sig in (signal.SIGINT, signal.SIGTERM)}
    try:
        with (output / 'launcher.log').open('xb') as stream:
            process = subprocess.Popen(command, cwd=repo, stdout=stream, stderr=subprocess.STDOUT,
                                       start_new_session=True)
            deadline: float | None = None
            while process.poll() is None:
                if interrupted and deadline is None:
                    stop(interrupted[0], None)
                    deadline = time.monotonic() + 5.0
                if deadline is not None and time.monotonic() >= deadline:
                    with suppress(ProcessLookupError):
                        os.killpg(process.pid, signal.SIGKILL)
                try:
                    process.wait(timeout=0.25)
                except subprocess.TimeoutExpired:
                    continue
            return process.wait(), interrupted[0] if interrupted else None
    finally:
        if process is not None and interrupted:
            with suppress(ProcessLookupError):
                os.killpg(process.pid, signal.SIGKILL)
            process.wait()
        for sig, handler in old_handlers.items():
            signal.signal(sig, handler)


def main(argv: Sequence[str]) -> int:
    if len(argv) not in (4, 5) or re.fullmatch('[0-9a-f]{40}', argv[1]) is None:
        print('usage: run_bound_gate.py FULL_HEAD OUTPUT_DIR RECEIPT_JSON [REPO_DIR]', file=sys.stderr)
        return 2
    expected = argv[1]
    output, receipt = (Path(value).resolve() for value in argv[2:4])
    repo = (Path(argv[4]) if len(argv) == 5 else DEFAULT_REPO).resolve()
    checksums = receipt.with_suffix(receipt.suffix + '.sha256')
    gate = repo / 'tools/revitalize/gate_pass17.sh'
    if receipt.is_relative_to(output) or receipt.is_relative_to(repo):
        print('receipt must be outside the gate output and repository', file=sys.stderr)
        return 2
    if output.is_relative_to(repo) and not output.is_relative_to(repo / 'docs/revitalize/verification'):
        print('repository output must be under docs/revitalize/verification', file=sys.stderr)
        return 2
    if any(p.exists() for p in (receipt, checksums)):
        print('receipt or SHA manifest already exists; refusing overwrite', file=sys.stderr)
        return 2
    receipt.parent.mkdir(parents=True, exist_ok=True)
    with receipt.open('x') as receipt_stream, checksums.open('x') as hash_stream:
        begin_utc, begin_epoch = stamp()
        before: Snapshot | None = None
        after: Snapshot | None = None
        gate_before: Artifact | None = None
        gate_after: Artifact | None = None
        exit_code: int | None = None
        interrupted: int | None = None
        gate_begin: tuple[str, float] | None = None
        gate_end: tuple[str, float] | None = None
        created = False
        errors: list[str] = []
        command = ['bash', str(gate), str(output)]
        try:
            if output.exists():
                raise GateError(f'output already exists; preserved unchanged: {output}')
            before = snapshot(repo)
            require_snapshot(before, expected)
            gate_before = artifact(gate)
            committed = subprocess.check_output(['git', 'show', expected + ':tools/revitalize/gate_pass17.sh'], cwd=repo)
            if hashlib.sha256(committed).hexdigest() != gate_before.sha256:
                raise GateError('gate script differs from the expected commit')
            output.mkdir(parents=True, exist_ok=False)
            created = True
            gate_begin = stamp()
            try:
                exit_code, interrupted = invoke(command, repo, output)
            finally:
                gate_end = stamp()
            after = snapshot(repo)
            gate_after = artifact(gate)
            require_snapshot(after, expected)
            if gate_after.sha256 != gate_before.sha256:
                raise GateError('gate script changed while running')
            errors.extend(gate_errors(output, expected))
            if exit_code != 0 or interrupted is not None:
                errors.append(f'gate exit={exit_code}; wrapper signal={interrupted}')
        except (OSError, subprocess.SubprocessError, GateError) as error:
            errors.append(f'{type(error).__name__}: {error}')
        end_utc, end_epoch = stamp()
        files: list[Artifact] = []
        if created:
            for path in sorted(output.rglob('*')):
                if path.is_file() or path.is_symlink():
                    try:
                        files.append(artifact(path))
                    except (OSError, GateError) as error:
                        errors.append(f'artifact error: {error}')
        passed = created and not errors and exit_code == 0
        payload = {'schema_version': 1, 'status': 'PASS' if passed else 'FAIL', 'passed': passed,
                   'expected_head': expected, 'repo': str(repo), 'output_directory': str(output),
                   'command': command, 'started_utc': begin_utc, 'started_epoch': begin_epoch,
                   'finished_utc': end_utc, 'finished_epoch': end_epoch, 'elapsed_seconds': end_epoch - begin_epoch,
                   'gate_started_utc': gate_begin[0] if gate_begin else None,
                   'gate_started_epoch': gate_begin[1] if gate_begin else None,
                   'gate_finished_utc': gate_end[0] if gate_end else None,
                   'gate_finished_epoch': gate_end[1] if gate_end else None,
                   'gate_exit_code': exit_code, 'wrapper_signal': interrupted, 'gate_launched': exit_code is not None,
                   'before': asdict(before) if before else None, 'after': asdict(after) if after else None,
                   'gate_script_before': asdict(gate_before) if gate_before else None,
                   'gate_script_after': asdict(gate_after) if gate_after else None,
                   'expected_steps': STEPS, 'errors': errors, 'artifacts': [asdict(f) for f in files],
                   'sha_manifest': str(checksums), 'receipt': str(receipt), 'perf_quick_is_timing_evidence': False}
        receipt_stream.write(json.dumps(payload, indent=2) + '\n')
        receipt_stream.flush()
        os.fsync(receipt_stream.fileno())
        for entry in (*files, artifact(receipt)):
            hash_stream.write(f'{entry.sha256}  {entry.path}\n')
        hash_stream.flush()
        os.fsync(hash_stream.fileno())
    print(f'{"PASS" if passed else "FAIL"}: {receipt}')
    return 0 if passed else 1


if __name__ == '__main__':
    raise SystemExit(main(sys.argv))
