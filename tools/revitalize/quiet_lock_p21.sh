#!/usr/bin/env bash
# usage: quiet_lock_p21.sh <runlog> <purpose> <command...>
# Timed measurements only. The supervisor owns a dedicated child process group;
# timeout, signal and normal exit all stop that group before releasing the lock.
# Pass 21 (as Pass 19): before checking quiet, it writes p21-data/watched/PAUSE so this run's
# own untimed jobs (run_watched_p21.sh groups) stop, waits (polling every
# 15 s, at most 10 min) for the 1-minute load to settle, and ignores those stopped groups in the busy check; PAUSE is removed
# on exit.
set -euo pipefail
(( $# >= 3 )) || { echo 'usage: quiet_lock_p21.sh <runlog> <purpose> <command...>' >&2; exit 2; }
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
STAMP: Final = Path('/Users/rexxa/work/p21-data/quiet_lock_last_release')
OWNER: Final = 'pass21-sim (Claude Code, new-world-pass4)'
POLL_SECONDS: Final = 300
SPACING_SECONDS: Final = 600
QUIET_WAIT_SECONDS: Final = 3600
HOLD_SECONDS: Final = 3600
CLEANUP_SECONDS: Final = 3
WATCH_DIR: Final = Path('/Users/rexxa/work/p21-data/watched')
PAUSE: Final = WATCH_DIR / 'PAUSE'
SETTLE_POLL_SECONDS: Final = 15
SETTLE_MAX_SECONDS: Final = 600
script_dir = Path(sys.argv[1])
log_path = Path(sys.argv[2])
purpose = sys.argv[3]
command = sys.argv[4:]
log_path.parent.mkdir(parents=True, exist_ok=True)
STAMP.parent.mkdir(parents=True, exist_ok=True)


def log(message: str) -> None:
    with log_path.open('a') as stream:
        stream.write(f'{time.strftime("%Y-%m-%d %H:%M:%S %Z")} {message}\n')


def stop_signal(signum: int, _frame: FrameType | None) -> None:
    raise SystemExit(128 + signum)


def stop_group(process: subprocess.Popen[bytes]) -> None:
    for sig in (signal.SIGTERM, signal.SIGKILL):
        try:
            os.killpg(process.pid, sig)
        except ProcessLookupError:
            break
        if sig == signal.SIGTERM:
            time.sleep(CLEANUP_SECONDS)
            process.poll()
    process.wait()


if os.environ.get('P21_QUIET_PID'):
    sys.exit('nested quiet holds are forbidden; run the measurement inside the existing hold')
for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
    signal.signal(sig, stop_signal)
log(f'request: {purpose}')
lock_inode: int | None = None
child: subprocess.Popen[bytes] | None = None
hold_started: float | None = None
info = ''
code = 1
contended = 0
try:
    if STAMP.exists():
        remaining = SPACING_SECONDS - (time.time() - float(STAMP.read_text()))
        if remaining > 0:
            log(f'WAIT spacing start: {purpose}; remaining={remaining:.1f}s')
            time.sleep(remaining)
            log(f'WAIT spacing end: {purpose}')
    WATCH_DIR.mkdir(parents=True, exist_ok=True)
    quiet_started: float | None = None
    while True:
        # The foreign-lock helper runs in its own group too, so cancellation
        # while waiting cannot leave an orphan poller.
        child = subprocess.Popen(
            ['bash', str(script_dir / 'wait_mac_p21.sh'), str(log_path), purpose],
            start_new_session=True,
        )
        child.wait()
        if child.returncode != 0:
            sys.exit(child.returncode)
        child = None
        if quiet_started is None:
            quiet_started = time.monotonic()
        # Pause this run's own untimed jobs, let the 1-minute load settle,
        # then judge the other run's activity.
        PAUSE.write_text(f'{os.getpid()}\n')
        log(f'PAUSE own watched groups: {sorted(p.name for p in WATCH_DIR.iterdir() if p.name.isdigit())}; settle up to {SETTLE_MAX_SECONDS}s')
        settle_started = time.monotonic()
        while True:
            time.sleep(SETTLE_POLL_SECONDS)
            load_output = subprocess.check_output(['sysctl', '-n', 'vm.loadavg'], text=True)
            load1 = float(load_output.split()[1])
            busy: list[str] = []
            for name in ('cargo', 'rustc', 'clippy-driver', 'worldsim_.*', 'Godot'):
                result = subprocess.run(['pgrep', '-x', name], capture_output=True, text=True, check=False)
                if result.returncode not in (0, 1):
                    sys.exit(f'pgrep failed for {name}: {result.stderr}')
                for pid in result.stdout.split():
                    pgid = subprocess.run(['ps', '-o', 'pgid=', '-p', pid], capture_output=True, text=True, check=False).stdout.strip()
                    if pgid and (WATCH_DIR / pgid).exists():
                        continue  # this run's own job, stopped by its watcher
                    busy.append(f'{name}:{pid}')
            # My own paused jobs no longer add load; wait for the 1-minute
            # average to decay (or for foreign work to show as `busy`).
            if busy or load1 < 4 or time.monotonic() - settle_started >= SETTLE_MAX_SECONDS:
                break
        log(f'settled: load={load1}; busy={busy}; after {time.monotonic() - settle_started:.0f}s')
        waited = time.monotonic() - quiet_started
        if load1 >= 4 or busy:
            if waited < QUIET_WAIT_SECONDS:
                log(f'WAIT quiet: {purpose}; load={load1}; busy={busy}; waited={waited:.1f}s')
                PAUSE.unlink(missing_ok=True)
                time.sleep(POLL_SECONDS)
                continue
            contended = 1
            log(f'contended: {purpose}; load={load1}; busy={busy}; waited={waited:.1f}s')
        try:
            LOCK.mkdir()
        except FileExistsError:
            log(f'acquire race lost: {purpose}; retry')
            continue
        lock_inode = LOCK.stat().st_ino
        hold_started = time.monotonic()
        started = time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())
        info = f'owner={OWNER}\npid={os.getpid()}\npurpose={purpose}\nstarted={started}\n'
        (LOCK / 'info.txt').write_text(info)
        log(f'HOLD start: {purpose}; load={load1}; contended={contended}; waited={waited:.1f}s')
        child_env = os.environ | {
            'PERF_CONTENDED': str(contended), 'P21_QUIET_OWNER': OWNER,
            'P21_QUIET_PID': str(os.getpid()), 'P21_QUIET_STARTED': started,
            'CARGO_BUILD_JOBS': '6',
        }
        child = subprocess.Popen(command, env=child_env, start_new_session=True)
        try:
            code = child.wait(timeout=max(0, HOLD_SECONDS - CLEANUP_SECONDS - (time.monotonic() - hold_started)))
            if code < 0:
                code = 128 - code
        except subprocess.TimeoutExpired:
            code = 124
            log(f'hard hold limit: stopping only launched process group {child.pid}')
        break
except SystemExit as error:
    code = error.code if isinstance(error.code, int) else 1
    raise
finally:
    for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
        signal.signal(sig, signal.SIG_IGN)
    if child is not None:
        stop_group(child)
    try:
        if PAUSE.read_text().strip() == str(os.getpid()):
            PAUSE.unlink()
            log('PAUSE released')
    except FileNotFoundError:
        pass
    if lock_inode is not None:
        try:
            if LOCK.stat().st_ino == lock_inode:
                info_path = LOCK / 'info.txt'
                if not info_path.exists() or info_path.read_text() == info:
                    info_path.unlink(missing_ok=True)
                    LOCK.rmdir()
                    STAMP.write_text(str(time.time()))
                else:
                    log('ownership changed: preserving lock')
        except FileNotFoundError:
            log('owned lock disappeared before cleanup')
    if hold_started is not None:
        log(f'HOLD end: {purpose}; exit={code}; held={time.monotonic() - hold_started:.1f}s')
sys.exit(code)
PY
