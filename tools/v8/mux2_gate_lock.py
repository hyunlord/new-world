#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 tools/v8/mux2_gate_lock.py -- COMMAND [ARG...]
"""One inherited advisory lock for the entire gate and nested GPU captures."""
from collections.abc import Iterator
from contextlib import contextmanager
import fcntl
import os
from pathlib import Path
import subprocess
import sys
from typing import Final
from mux2_gate_inputs import EvidenceError, require

LOCK: Final = Path('/Users/rexxa/work/mux2-data/.mux2-gpu.lock')
FD_ENV: Final = 'WORLDSIM_MUX2_LANE_FD'


@contextmanager
def lane(path: Path = LOCK) -> Iterator[int]:
    """Reuse an inherited lock only when its descriptor names the canonical inode."""
    inherited = os.environ.get(FD_ENV)
    if inherited is not None:
        descriptor = int(inherited)
        held = os.fstat(descriptor)
        current = path.stat()
        require((held.st_dev, held.st_ino) == (current.st_dev, current.st_ino),
                'inherited gate lock names another file')
        # A forged but unlocked descriptor must acquire the real lock or fail.
        fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        yield descriptor
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        yield lock.fileno()


def run(command: list[str]) -> int:
    require(bool(command), 'expected command after --')
    with lane() as descriptor:
        environment = dict(os.environ, **{FD_ENV: str(descriptor)})
        return subprocess.run(command, env=environment, pass_fds=(descriptor,), check=False).returncode


if __name__ == '__main__':
    try:
        args = sys.argv[1:]
        if args == ['--check']:
            require(FD_ENV in os.environ, 'full gate requires inherited lane lock')
            with lane():
                print('MUX2_GATE_LOCK_HELD')
            raise SystemExit(0)
        require(bool(args) and args[0] == '--', 'expected -- COMMAND')
        raise SystemExit(run(args[1:]))
    except (EvidenceError, OSError, ValueError) as error:
        print(f'MUX2_GATE_LOCK_FAIL {error}', file=sys.stderr)
        raise SystemExit(1)
