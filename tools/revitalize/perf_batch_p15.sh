#!/usr/bin/env bash
# usage: perf_batch_p15.sh <batch_log> <command...>
# Pass 15 timed-batch launcher (PASS15_GOAL.md section 1): records date, load
# averages and the matching processes, then starts <command> only when the
# 1-minute load average is below 4, no Godot process runs and no cargo /
# rustc / other worldsim_ process runs (one timed process at a time, never
# beside a build or a test). Otherwise it checks again every 10 minutes; after
# 2 hours of waiting it runs anyway with PERF_CONTENDED=1 (the run JSON and the
# log mark the batch `contended`). Every check is appended to <batch_log>.
# `pgrep -fl` is logged in full, but the decision uses process names
# (`pgrep -x`), because the -f form also matches unrelated processes whose
# arguments carry a PATH with `.cargo` (DECISIONS.md Pass 15).
set -uo pipefail
LOG="${1:?usage: perf_batch_p15.sh <batch_log> <command...>}"; shift
WAIT_SECONDS="${PERF_WAIT_SECONDS:-600}"
MAX_WAIT_SECONDS="${PERF_MAX_WAIT_SECONDS:-7200}"
LOAD_LIMIT="${PERF_LOAD_LIMIT:-4}"
mkdir -p "$(dirname "$LOG")"
waited=0
contended=0
while true; do
    load1=$(sysctl -n vm.loadavg | awk '{print $2}')
    busy=$(for name in Godot cargo rustc clippy-driver worldsim_perf worldsim_probe worldsim_wm; do pgrep -x "$name" | sed "s/^/$name:/"; done | tr '\n' ' ')
    {
        echo "check $(date '+%F %T %Z') waited ${waited}s loadavg $(sysctl -n vm.loadavg)"
        echo "  pgrep -fl: $(pgrep -fl 'Godot|cargo|rustc|worldsim_' | cut -c1-160 | tr '\n' ';')"
        echo "  blocking by name: ${busy:-none}"
    } >> "$LOG"
    if awk -v l="$load1" -v m="$LOAD_LIMIT" 'BEGIN{exit !(l < m)}' && [ -z "$busy" ]; then
        break
    fi
    if (( waited >= MAX_WAIT_SECONDS )); then
        contended=1
        echo "  waited ${waited}s: running anyway, contended" >> "$LOG"
        break
    fi
    sleep "$WAIT_SECONDS"
    waited=$((waited + WAIT_SECONDS))
done
export PERF_CONTENDED=$contended
echo "start $(date '+%F %T %Z') contended $contended waited ${waited}s: $*" >> "$LOG"
"$@"
code=$?
echo "end $(date '+%F %T %Z') exit $code loadavg $(sysctl -n vm.loadavg)" >> "$LOG"
exit $code
