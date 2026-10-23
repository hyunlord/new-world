#!/usr/bin/env bash
# usage: [BENCH_PASS=NAME] [BENCH_BASE=COMMIT] bench.sh <log_dir> [--justified "TEXT quoted from CHANGELOG.md"]
#
# Gate step `bench` (M1.5 goal 6.1, ADR 0016 C3): builds the release
# benchmark, runs `worldsim_perf bench` (5 colony scenes × 3 repeats, each
# scene in its own process) into <log_dir>/bench, then compares the run with
# the latest quiet pass in docs/perf/PERF_LEDGER.md (`tools/v8/bench_ledger.py
# check`): more than +10 % on any scene's tick mean or p99, or on the memory
# per agent, fails unless --justified quotes the CHANGELOG. Since M2 a C3
# ceiling MISS fails too (no justification). Since M5 (goal §5.3, the ledger
# rule, second fix) the candidate's pass label is BENCH_PASS (default
# "candidate"): rows with that label are never its base, and with BENCH_BASE
# set (a commit) rows whose commit descends from it are skipped too; the
# memory rule is on alloc/agent (the allocator counter) when the base rows
# have it, else on RSS/agent. Before measuring it waits (up to 45 min) for the
# 1-minute load to fall below 8 (D23). The host load is recorded in every run
# file. This step never writes the ledger; a pass appends its rows in its
# report step with `bench_ledger.py append … --gate-log <log_dir>/gate.txt`
# after the gate is green.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
LOG="${1:?usage: bench.sh <log_dir> [--justified TEXT]}"
JUSTIFIED=()
[[ "${2:-}" == --justified ]] && JUSTIFIED=(--justified "$3")
mkdir -p "$LOG"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-8}"
(cd rust && cargo build --release -p sim-test --bin worldsim_perf) || exit 2
# Admission (M2 DECISIONS D23): the ledger rows come from a quiet session, so
# wait up to BENCH_WAIT_S (default 2700 s) for the 1-minute load to fall below
# BENCH_MAX_LOAD (default 8) before measuring; a run that still starts above it
# is labelled contended in the log.
max_load="${BENCH_MAX_LOAD:-8}"
waited=0
load1() { sysctl -n vm.loadavg 2>/dev/null | awk '{print $2}'; }
while awk -v l="$(load1)" -v m="$max_load" 'BEGIN { exit !(l >= m) }' && (( waited < ${BENCH_WAIT_S:-2700} )); do
    sleep 30
    waited=$((waited + 30))
done
if awk -v l="$(load1)" -v m="$max_load" 'BEGIN { exit !(l >= m) }'; then
    echo "admission: CONTENDED (1-min load $(load1) >= $max_load after ${waited} s)"
else
    echo "admission: quiet (1-min load $(load1) < $max_load after waiting ${waited} s)"
fi
echo "load before: $(uptime)"
rust/target/release/worldsim_perf bench --out "$LOG/bench" --commands tools/v8/bench_script.json \
    --repeats 3 --commit "$(git rev-parse --short=8 HEAD)" || exit 2
echo "load after: $(uptime)"
BASE=()
[[ -n "${BENCH_BASE:-}" ]] && BASE=(--base "$BENCH_BASE")
python3 tools/v8/bench_ledger.py check "$LOG/bench/bench.json" docs/perf/PERF_LEDGER.md \
    --pass "${BENCH_PASS:-candidate}" ${BASE[@]+"${BASE[@]}"} ${JUSTIFIED[@]+"${JUSTIFIED[@]}"}
