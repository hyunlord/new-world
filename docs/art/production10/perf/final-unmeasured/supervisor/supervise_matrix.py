#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Atomic final-matrix launch cutoff; an active measurement may finish."""
from __future__ import annotations

import fcntl
import json
import os
import shlex
import signal
import subprocess
import sys
import time
from contextlib import contextmanager
from pathlib import Path
from collections.abc import Iterator
from typing import TypeAlias

Json: TypeAlias = None | bool | int | float | str | list['Json'] | dict[str, 'Json']
State: TypeAlias = dict[str, Json]


@contextmanager
def locked(folder: Path) -> Iterator[State]:
    """Serialize cutoff and measurement launch decisions across processes."""
    with (folder / 'gate.lock').open('a+') as stream:
        fcntl.flock(stream, fcntl.LOCK_EX)
        path = folder / 'gate.json'
        state: State = json.loads(path.read_text()) if path.exists() else {'closed': False, 'events': []}
        try:
            yield state
        finally:
            temporary = folder / 'gate.tmp'
            temporary.write_text(json.dumps(state, indent=2) + '\n')
            temporary.replace(path)
            fcntl.flock(stream, fcntl.LOCK_UN)


def event(state: State, name: str) -> None:
    values = state['events']
    if not isinstance(values, list):
        raise RuntimeError('Invalid event list')
    values.append({'event': name, 'epoch': time.time(), 'pid': os.getpid()})


def shim(arguments: list[str]) -> int:
    """Start only under the gate lock; never signal the measured child."""
    folder = Path(os.environ['P10_SUPERVISOR_DIR'])
    deadline = float(os.environ['P10_DEADLINE_EPOCH'])
    with locked(folder) as state:
        if state['closed'] is True or time.time() >= deadline:
            state['closed'] = True
            event(state, 'LAUNCH_REFUSED_AT_CUTOFF')
            return 124
        process = subprocess.Popen([os.environ['P10_REAL_GODOT'], *arguments])
        state['active_pid'] = process.pid
        state['shim_pid'] = os.getpid()
        event(state, 'MEASUREMENT_STARTED')
    code = process.wait()
    with locked(folder) as state:
        state['active_pid'] = None
        state['shim_pid'] = None
        state['measurement_exit'] = code
        event(state, 'MEASUREMENT_FINISHED')
    return code


def waiting_wrappers(root_pid: int, wrapper: Path) -> list[int]:
    """Find exact wrapper argv only inside this still-live runner's child tree."""
    output = subprocess.check_output(['ps', '-axo', 'pid=,ppid=,command='], text=True)
    rows: dict[int, tuple[int, str]] = {}
    for line in output.splitlines():
        fields = line.strip().split(None, 2)
        if len(fields) == 3:
            rows[int(fields[0])] = (int(fields[1]), fields[2])
    owned = {root_pid}
    while True:
        expanded = owned | {pid for pid, (parent, _) in rows.items() if parent in owned}
        if expanded == owned:
            break
        owned = expanded
    matches = []
    for pid in owned - {root_pid}:
        command = rows[pid][1]
        try:
            args = shlex.split(command)
        except ValueError:
            continue
        if len(args) >= 2 and Path(args[0]).name == 'bash' and args[1] == str(wrapper):
            matches.append(pid)
    sleepers = []
    wrapper_tree = set(matches)
    while True:
        expanded = wrapper_tree | {pid for pid, (parent, _) in rows.items() if parent in wrapper_tree}
        if expanded == wrapper_tree:
            break
        wrapper_tree = expanded
    for pid in wrapper_tree - set(matches):
        args = rows[pid][1].split()
        if args and Path(args[0]).name == 'sleep':
            sleepers.append(pid)
    return matches + sleepers


def supervise(command: list[str], folder: Path, deadline: float, wrapper: Path, environment: dict[str, str]) -> int:
    """Close launches at cutoff; TERM only owned waiting wrapper, preserving runner status."""
    closed = False
    with (folder / 'runner.log').open('x') as stream:
        runner = subprocess.Popen(command, env=environment, stdout=stream, stderr=subprocess.STDOUT, start_new_session=True)
        (folder / 'runner-pid.txt').write_text(str(runner.pid) + '\n')
        signalled: set[int] = set()
        while runner.poll() is None:
            if time.time() >= deadline:
                with locked(folder) as state:
                    if not closed:
                        state['closed'] = True
                        event(state, 'CUTOFF_CLOSED')
                        closed = True
                    active = state.get('active_pid') is not None
                if not active and runner.poll() is None:
                    for pid in waiting_wrappers(runner.pid, wrapper):
                        if pid in signalled:
                            continue
                        try:
                            os.kill(pid, signal.SIGTERM)
                        except ProcessLookupError:
                            continue
                        signalled.add(pid)
                        with locked(folder) as state:
                            event(state, f'WAITING_OWNED_WRAPPER_TERM_{pid}')
            time.sleep(0.2)
        code = runner.wait()
    (folder / 'runner.exit').write_text(str(code) + '\n')
    return code


def main(arguments: list[str]) -> int:
    if arguments and arguments[0] == '--godot-shim':
        return shim(arguments[1:])
    if len(arguments) != 3:
        print('Usage: supervise_matrix.py CHECKOUT NEW_OUTPUT_DIR ORIGINAL_DEADLINE_EPOCH', file=sys.stderr)
        return 2
    checkout, output = Path(arguments[0]).resolve(), Path(arguments[1]).resolve()
    deadline = float(arguments[2])
    if deadline <= time.time():
        raise SystemExit('Original cutoff already passed; no measurement launched')
    for suffix in ('.json', '.md'):
        if (checkout / 'docs/art/production10/perf' / ('after' + suffix)).exists():
            raise SystemExit('Existing after aggregate: preserve it before a fresh final attempt')
    output.mkdir(parents=True, exist_ok=False)
    sys.path.insert(0, str(checkout / 'tools/art_pipeline'))
    from run_alternating_p10 import provenance
    before = provenance(checkout)
    tool_files = sorted(path for path in (checkout / 'tools/art_pipeline').glob('*p10*') if path.is_file())
    import hashlib
    tools_before = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in tool_files}
    tool_files.extend([Path(__file__).resolve(), Path(__file__).with_name('priority_matrix.sh').resolve()])
    tools_before = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in tool_files}
    real_godot = os.environ.get('GODOT_BIN', '/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot')
    shim_path = output / 'godot-launch-gate'
    shim_path.write_text('#!/bin/sh\nexec ' + shlex.quote(sys.executable) + ' ' + shlex.quote(str(Path(__file__).resolve())) + ' --godot-shim "$@"\n')
    shim_path.chmod(0o700)
    env = dict(os.environ, GODOT_BIN=str(shim_path), P10_REAL_GODOT=real_godot,
               P10_SUPERVISOR_DIR=str(output), P10_DEADLINE_EPOCH=str(int(deadline)), P10_REQUIRED_ONLY='1')
    wrapper = checkout / 'tools/art_pipeline/with_quiet_lock_p10.sh'
    command = ['bash', str(Path(__file__).with_name('priority_matrix.sh').resolve()), 'after', str(checkout), str(output / 'matrix')]
    launch = {'before': before, 'tool_sha256_before': tools_before, 'command': command, 'deadline_epoch': deadline, 'started_epoch': time.time()}
    (output / 'supervisor-launch.json').write_text(json.dumps(launch, indent=2) + '\n')
    code = supervise(command, output, deadline, wrapper, env)
    after = provenance(checkout)
    tools_after = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in tool_files}
    stable = before == after and tools_before == tools_after
    result = {**launch, 'after': after, 'tool_sha256_after': tools_after, 'finished_epoch': time.time(), 'runner_exit': code, 'source_build_stable': stable, 'complete': code == 0 and stable, 'acceptance': 'LEADER_DECISION_REQUIRED'}
    (output / 'supervisor-result.json').write_text(json.dumps(result, indent=2) + '\n')
    return code if code else (0 if stable else 1)


if __name__ == '__main__':
    raise SystemExit(main(sys.argv[1:]))
