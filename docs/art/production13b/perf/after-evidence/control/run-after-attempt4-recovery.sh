#!/usr/bin/env bash
set -euo pipefail
cd /Users/rexxa/work/new-world-art-prod
source /Users/rexxa/work/art-production13b-evidence/environment.sh
export P13B_WALKER_KERNEL=1 P12_DECLUMP_KERNEL=1 P13_WRITE_SKIP=0
export P13_BUILD_PROFILE=release P13_DEADLINE_EPOCH=1792447680
measure=/Users/rexxa/work/art-p13b-measure
out=/Users/rexxa/work/art-production13b-evidence/after
[[ -d "$out" ]]
snapshot() { bash tools/art_pipeline/run_running_identity_p13b.sh --snapshot "$measure"; }
snapshot > "$out/retry4-recovery-initial-manifest.json"
cmp "$out/frozen-manifest.json" "$out/retry4-recovery-initial-manifest.json"
record() {
 local item=$1
 shift
 snapshot > "$out/$item.before.json"
 cmp "$out/frozen-manifest.json" "$out/$item.before.json"
 local rc=0
 "$@" > "$out/$item.runner.log" 2>&1 || rc=$?
 printf '%s\n' "$rc" > "$out/$item.runner.exit"
 snapshot > "$out/$item.after.json"
 cmp "$out/frozen-manifest.json" "$out/$item.after.json"
}
unset P13_ITEM P13_VALIDITY_OUT
record matrix-attempt4-recovery env P13_OUTPUT_PREFIX=after bash tools/art_pipeline/run_perf_matrix_p13b.sh after "$measure" "$out"
snapshot > "$out/terminal-manifest-attempt4-recovery.json"
cmp "$out/frozen-manifest.json" "$out/terminal-manifest-attempt4-recovery.json"
