#!/usr/bin/env bash
# usage: wait_mac_p21.sh <runlog> [purpose]
# Wait before every build, test, probe or Godot launch. Only a live quiet-lock
# supervisor with the exact owner/PID/start token may run through its own lock.
set -euo pipefail
(( $# >= 1 && $# <= 2 )) || { echo 'usage: wait_mac_p21.sh <runlog> [purpose]' >&2; exit 2; }
exec python3 - "$1" "${2:-heavy step}" <<'PY'
import os
from pathlib import Path
import sys
import time
from typing import Final

LOCK: Final = Path('/Users/rexxa/work/.mac_quiet.lock')
OWNER: Final = 'pass21-sim (Claude Code, new-world-pass4)'
POLL_SECONDS: Final = 300
STALE_SECONDS: Final = 7200
log_path = Path(sys.argv[1])
purpose = sys.argv[2]
log_path.parent.mkdir(parents=True, exist_ok=True)
wait_started: float | None = None


def log(message: str) -> None:
    with log_path.open('a') as stream:
        stream.write(f'{time.strftime("%Y-%m-%d %H:%M:%S %Z")} {message}\n')


while LOCK.exists():
    try:
        inode = LOCK.stat().st_ino
        age = time.time() - LOCK.stat().st_mtime
        info_path = LOCK / 'info.txt'
        info = info_path.read_text() if info_path.exists() else ''
        fields = dict(line.split('=', 1) for line in info.splitlines() if '=' in line)
        owner_pid = os.environ.get('P21_QUIET_PID', '')
        owned = (
            fields.get('owner') == OWNER == os.environ.get('P21_QUIET_OWNER')
            and fields.get('pid') == owner_pid and owner_pid.isdigit()
            and fields.get('started') == os.environ.get('P21_QUIET_STARTED')
        )
        if owned and int(owner_pid) > 1:
            try:
                os.kill(int(owner_pid), 0)
            except ProcessLookupError:
                owned = False
            if owned:
                log(f'owned lock permitted: {purpose}; pid={owner_pid}')
                break
        if age > STALE_SECONDS:
            # Delete only the observed directory's known metadata. A replacement
            # lock or unexpected files are never recursively removed.
            if LOCK.stat().st_ino == inode:
                info_path.unlink(missing_ok=True)
                if LOCK.stat().st_ino == inode:
                    LOCK.rmdir()
                    log(f'stale lock removed: age={age:.0f}s; {info.strip()!r}')
            continue
    except FileNotFoundError:
        continue
    if wait_started is None:
        wait_started = time.monotonic()
        log(f'WAIT start: {purpose}')
    log(f'foreign lock: {info.strip()!r}; poll in {POLL_SECONDS}s')
    time.sleep(POLL_SECONDS)

if wait_started is not None:
    log(f'WAIT end: {purpose}; waited={time.monotonic() - wait_started:.1f}s')
PY
