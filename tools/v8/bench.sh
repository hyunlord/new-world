#!/usr/bin/env bash
# usage: bench.sh <log_dir> [--justified "TEXT quoted from CHANGELOG.md"]
#
# Gate step `bench` (M1.5 goal 6.1, ADR 0016 C3): builds the release
# benchmark, runs `worldsim_perf bench` (5 colony scenes × 3 repeats, each
# scene in its own process) into <log_dir>/bench, then compares the run with
# the latest pass in docs/perf/PERF_LEDGER.md (`tools/v8/bench_ledger.py
# check`): more than +10 % on any scene's tick mean or p99, or on RSS per
# agent, fails unless --justified quotes the CHANGELOG. Since M2 a C3 ceiling
# MISS fails too (no justification). The host load is recorded in
# every run file. This step never writes the ledger; a pass appends its line
# with `bench_ledger.py append` from an admitted session.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
LOG="${1:?usage: bench.sh <log_dir> [--justified TEXT]}"
JUSTIFIED=()
[[ "${2:-}" == --justified ]] && JUSTIFIED=(--justified "$3")
mkdir -p "$LOG"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-8}"
(cd rust && cargo build --release -p sim-test --bin worldsim_perf) || exit 2
echo "load before: $(uptime)"
rust/target/release/worldsim_perf bench --out "$LOG/bench" --commands tools/v8/bench_script.json \
    --repeats 3 --commit "$(git rev-parse --short=8 HEAD)" || exit 2
echo "load after: $(uptime)"
python3 tools/v8/bench_ledger.py check "$LOG/bench/bench.json" docs/perf/PERF_LEDGER.md ${JUSTIFIED[@]+"${JUSTIFIED[@]}"}
