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
snapshot > "$out/retry2-initial-manifest.json"
cmp "$out/frozen-manifest.json" "$out/retry2-initial-manifest.json"
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
record matrix-attempt2 env P13_OUTPUT_PREFIX=after bash tools/art_pipeline/run_perf_matrix_p13b.sh after "$measure" "$out"
for world in W200 W-multi W500; do
 record "sections-$world-attempt2" env P13_ITEM="after-sections-$world" P13_SECTIONS_WORLD="$world" python3 tools/art_pipeline/run_agent_sections_p13b.py "$measure" "$out/sections-$world-attempt2"
done
export P13_MEASURE_CHECKOUT="$measure"
export P13_MEASURE_HEAD P13_MEASURE_DYLIB_SHA256 P13_MEASURE_SCRIPT_SHA256
P13_MEASURE_HEAD=$(git -C "$measure" rev-parse HEAD)
P13_MEASURE_DYLIB_SHA256=$(shasum -a 256 "$measure/rust/target/debug/libsim_bridge.dylib" | awk '{print $1}')
[[ "$P13_MEASURE_DYLIB_SHA256" == $(shasum -a 256 "$measure/rust/target/release/libsim_bridge.dylib" | awk '{print $1}') ]]
P13_MEASURE_SCRIPT_SHA256=$(shasum -a 256 "$measure/scripts/test/art/perf_frames_p13.gd" | awk '{print $1}')
export P13_VALIDITY_OUT="$out/ablation-W200-attempt2.json.validity.json" P13_ITEM=after-ablation-W200
record ablation-W200-attempt2 bash tools/art_pipeline/with_quiet_lock_p13b.sh after-ablation-W200-attempt2 3600 "$GODOT_BIN" --path "$measure" --resolution 1920x1080 --script res://scripts/test/art/perf_frames_p13.gd -- --world W200 --phase after --ablation-only --out "$out/ablation-W200-attempt2.json"
snapshot > "$out/terminal-manifest-attempt2.json"
cmp "$out/frozen-manifest.json" "$out/terminal-manifest-attempt2.json"
