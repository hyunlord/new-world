#!/usr/bin/env bash
# usage: quiet_lock_p16.sh <log> <purpose> <command...>
# Pass 16 timed run under the shared Mac quiet lock (PASS16_GOAL.md 1.2):
#  1. if /Users/rexxa/work/.mac_quiet.lock is held by someone else, wait (check every
#     5 minutes); a lock older than 2 hours is stale and is removed (logged);
#  2. keep at least 10 minutes after this run's previous hold (stamp file);
#  3. wait until no cargo / rustc / clippy-driver / worldsim_* / Godot process runs
#     (by name) and the 1-minute load average is below 4; after 60 minutes of this
#     wait take the lock anyway and run with PERF_CONTENDED=1;
#  4. acquire with mkdir (atomic), write info.txt (owner, pid, purpose, started),
#     run <command>, release; a hold longer than 60 minutes is logged as such.
# Every check, hold and wait is appended to <log>.
set -uo pipefail
LOG="${1:?log}"; PURPOSE="${2:?purpose}"; shift 2
LOCK=/Users/rexxa/work/.mac_quiet.lock
STAMP=/Users/rexxa/work/p16-data/quiet_lock_last_release
OWNER="pass16-sim (Claude Code, new-world-pass4)"
mkdir -p "$(dirname "$LOG")"
log() { echo "$(date '+%F %T %Z') $*" >> "$LOG"; }
log "request: $PURPOSE"
# 2. spacing between our own holds.
if [ -f "$STAMP" ]; then
    last=$(cat "$STAMP"); now=$(date +%s)
    if (( now - last < 600 )); then
        log "spacing: waiting $((600 - (now - last)))s after our previous hold"
        sleep $((600 - (now - last)))
    fi
fi
contended=0
waited=0
while true; do
    # 1. someone else's lock.
    if [ -d "$LOCK" ]; then
        age=$(( $(date +%s) - $(stat -f %m "$LOCK") ))
        if (( age > 7200 )); then
            log "stale lock (${age}s): $(tr '\n' ' ' < "$LOCK/info.txt" 2>/dev/null) — removed"
            rm -rf "$LOCK"
        else
            log "lock held by: $(tr '\n' ' ' < "$LOCK/info.txt" 2>/dev/null) — waiting 5 min"
            sleep 300
            continue
        fi
    fi
    # 3. quiet machine.
    load1=$(sysctl -n vm.loadavg | awk '{print $2}')
    busy=$(for name in Godot cargo rustc clippy-driver worldsim_perf worldsim_probe worldsim_wm; do pgrep -x "$name" | sed "s/^/$name:/"; done | tr '\n' ' ')
    if awk -v l="$load1" 'BEGIN{exit !(l < 4)}' && [ -z "$busy" ]; then
        break
    fi
    if (( waited >= 3600 )); then
        contended=1
        log "waited ${waited}s (load $load1, busy: ${busy:-none}): taking the lock anyway, contended"
        break
    fi
    log "not quiet: load $load1, busy: ${busy:-none} (waited ${waited}s) — checking again in 5 min"
    sleep 300
    waited=$((waited + 300))
done
if ! mkdir "$LOCK" 2>/dev/null; then
    log "lost the race for the lock — retrying"
    exec "$0" "$LOG" "$PURPOSE" "$@"
fi
started=$(date +%s)
printf 'owner=%s\npid=%s\npurpose=%s\nstarted=%s\n' "$OWNER" "$$" "$PURPOSE" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > "$LOCK/info.txt"
log "HOLD start: $PURPOSE (load $(sysctl -n vm.loadavg), contended=$contended, waited ${waited}s)"
PERF_CONTENDED=$contended "$@"
code=$?
held=$(( $(date +%s) - started ))
rm -rf "$LOCK"
date +%s > "$STAMP"
log "HOLD end: $PURPOSE exit $code held ${held}s$( (( held > 3600 )) && echo ' (over 60 min)')"
exit "$code"
