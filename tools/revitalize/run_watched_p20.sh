#!/usr/bin/env bash
# usage: run_watched_p20.sh <runlog> <purpose> <command...>
# Pass 20 (brief 1.2, as Pass 19): every gate step, probe batch and build runs in its own
# process group. A watcher polls every 10 s; while a foreign quiet lock exists
# (or while this run's own quiet hold has asked its untimed jobs to pause via
# p20-data/watched/PAUSE) it sends SIGSTOP to the whole group, and SIGCONT when
# the lock is gone. Stopped intervals are logged. The group id is registered in
# p20-data/watched/ so quiet_lock_p20.sh can tell this run's paused jobs from
# the other run's processes. If WATCH_STATUS is set, the total stopped seconds
# are written there (a timed step that was stopped must be re-run).
set -euo pipefail
(( $# >= 3 )) || { echo 'usage: run_watched_p20.sh <runlog> <purpose> <command...>' >&2; exit 2; }
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec python3 - "$SCRIPT_DIR" "$@" <<'PY'
import os
from pathlib import Path
import signal
import subprocess
import sys
import time
from types import FrameType
from typing import Final

LOCK: Final = Path('/Users/rexxa/work/.mac_quiet.lock')
WATCH_DIR: Final = Path('/Users/rexxa/work/p20-data/watched')
PAUSE: Final = WATCH_DIR / 'PAUSE'
OWNER: Final = 'pass20-sim (Claude Code, new-world-pass4)'
POLL_SECONDS: Final = 10
STALE_SECONDS: Final = 7200
script_dir = Path(sys.argv[1])
log_path = Path(sys.argv[2])
purpose = sys.argv[3]
command = sys.argv[4:]
log_path.parent.mkdir(parents=True, exist_ok=True)
WATCH_DIR.mkdir(parents=True, exist_ok=True)


def log(message: str) -> None:
    with log_path.open('a') as stream:
        stream.write(f'{time.strftime("%Y-%m-%d %H:%M:%S %Z")} [watch] {message}\n')


def owned_by_env(fields: dict[str, str]) -> bool:
    pid = os.environ.get('P20_QUIET_PID', '')
    return (
        fields.get('owner') == OWNER == os.environ.get('P20_QUIET_OWNER')
        and fields.get('pid') == pid and pid.isdigit()
        and fields.get('started') == os.environ.get('P20_QUIET_STARTED')
    )


def blocker() -> str | None:
    """Why this group must be stopped now, or None."""
    if PAUSE.exists() and not os.environ.get('P20_QUIET_PID'):
        return 'own quiet hold (PAUSE)'
    try:
        stat = LOCK.stat()
    except FileNotFoundError:
        return None
    if time.time() - stat.st_mtime > STALE_SECONDS:
        return None  # stale; wait_mac_p20.sh removes and records it
    info_path = LOCK / 'info.txt'
    try:
        info = info_path.read_text()
    except FileNotFoundError:
        info = ''
    fields = dict(line.split('=', 1) for line in info.splitlines() if '=' in line)
    if owned_by_env(fields):
        return None
    return 'lock ' + ' '.join(f'{k}={v}' for k, v in fields.items() if k in ('owner', 'purpose', 'started'))


child: subprocess.Popen[bytes] | None = None


def forward(signum: int, _frame: FrameType | None) -> None:
    if child is not None:
        try:
            os.killpg(child.pid, signal.SIGCONT)
            os.killpg(child.pid, signum)
        except ProcessLookupError:
            pass
    raise SystemExit(128 + signum)


for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
    signal.signal(sig, forward)

# Do not start while a foreign lock is held (same wait as Pass 18's steps).
subprocess.run(['bash', str(script_dir / 'wait_mac_p20.sh'), str(log_path), purpose], check=True)
while blocker() is not None:
    time.sleep(POLL_SECONDS)
child = subprocess.Popen(command, start_new_session=True)
pgid = child.pid
reg = WATCH_DIR / str(pgid)
reg.write_text(f'{purpose}\n')
log(f'start: {purpose}; pgid={pgid}')
stopped_at: float | None = None
stopped_total = 0.0
code = 1
try:
    while True:
        try:
            code = child.wait(timeout=POLL_SECONDS)
            break
        except subprocess.TimeoutExpired:
            pass
        why = blocker()
        if why is not None and stopped_at is None:
            try:
                os.killpg(pgid, signal.SIGSTOP)
            except ProcessLookupError:
                continue
            stopped_at = time.monotonic()
            log(f'STOP pgid={pgid} ({purpose}): {why}')
        elif why is None and stopped_at is not None:
            try:
                os.killpg(pgid, signal.SIGCONT)
            except ProcessLookupError:
                pass
            span = time.monotonic() - stopped_at
            stopped_total += span
            stopped_at = None
            log(f'CONT pgid={pgid} ({purpose}); stopped {span:.1f}s')
    if code < 0:
        code = 128 - code
finally:
    if stopped_at is not None:
        stopped_total += time.monotonic() - stopped_at
    reg.unlink(missing_ok=True)
    log(f'end: {purpose}; exit={code}; stopped_total={stopped_total:.1f}s')
    status = os.environ.get('WATCH_STATUS')
    if status:
        Path(status).write_text(f'{stopped_total:.1f}\n')
sys.exit(code)
PY
