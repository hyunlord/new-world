#!/usr/bin/env bash
# usage: perf_repeat_p15.sh — Pass 15 repeatability (7.2): seed 15100, n = 500, fed, 2 days,
# three runs one after another into docs/revitalize/probe/p15-2_stress_repeat/r{1,2,3}/.
# Run it through perf_batch_p15.sh (one load gate for the three runs).
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
status=0
for r in 1 2 3; do
    echo "-- repeat r$r $(date '+%F %T %Z') loadavg $(sysctl -n vm.loadavg)" >> "${P15_DATA:-/Users/rexxa/work/p15-data}/batches.log"
    rust/target/release/worldsim_perf stress --seeds 15100..15101 --agents 500 --side 64 --days 2 --feed fed \
        --out "docs/revitalize/probe/p15-2_stress_repeat/r$r" || status=1
done
exit "$status"
