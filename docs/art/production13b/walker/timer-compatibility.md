# Walker section-timer compatibility and W200 ablation launch

Status: OFFLINE COMPATIBILITY VERIFIED; runtime installation and measurements NOT_RUN in this task. No workspace edits, source changes, commits, Godot or builds. Only external diagnostic output and this report were written while the leader's capture freeze remained in force.

## Current renderer compatibility

Executed the unchanged generator:

```sh
python3 tools/art_pipeline/generate_agent_sections_p13.py scripts/ui/agent_renderer.gd /Users/rexxa/work/art-production13b-evidence/walker-agent-sections-compatibility.gd
```

Exit0. Source SHA256 before/after `eeb00094a8dbc57dbced5c815dfcc528271b7ed844bb08032a1b10b3d83b8aa9`; generated SHA256 `0d1fbd492bc4ec6ab40a8b4e9381f08219059a7b9af591f3ed7360f9316e2ef5`; generator SHA256 `24306a27b73c4ef941999da12bdb0f1945312698cf2b62356541df10ae2ea6b4`. Sidecar: `walker-agent-sections-compatibility.manifest.json`.

An independent offline comparison removed only `# P13_PROFILE` additions and recovered every nonblank ordinary source statement in order, including ArtWalkerP13B preload/flag/getter, the fixture sink, candidate/reference conditional call and result application. The generated line890 is the entire current `var lateral:` conditional: checkpoint18 immediately precedes it and checkpoint19 immediately follows it. Thus `walker_offsets_usec` includes adapter packing, bridge/kernel and output unpacking for ON, or original CampSeats computation for OFF. It does not accidentally include walker application. Existing sections remain25exclusive buckets; the clock requires the exact sum equal callback total with zero remainder, and existing runtime validator checks every sample's completion/alignment/partition.

The fixture sink is preserved immediately before checkpoint18, in the existing `easing_usec` interval. The ordinary diagnostic run leaves the sink null. If someone enables fixture recording during profiling, that recording work would contaminate easing, not walker_offsets; do not combine those runs. This is not a new marker failure or a reason to move reference boundaries during comparison.

Also ran offline only:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover -s tools/art_pipeline -p test_generate_agent_sections_p13.py -v
```

All3tests passed (guarded markers fail closed, write-miss checkpoints stay inside guard, ordinary statements preserved). Log: `walker-timer-generator-tests.log`; direct walker-call/sink/source assertions: `walker-timer-offline-checks.json`. No GDScript parser or runtime timer readout was exercised. `FoundingGroups` remains unavailable as a separate callback, explicitly not folded into parent timings.

No generator copy/edit is necessary. The existing `tools/art_pipeline/run_agent_sections_p13b.py:141` already calls `with_quiet_lock_p13b.sh` while reusing unchanged generator/clock/loader/profile scripts. For Part4's two running section sessions use the frozen release checkout and inherited calibration/deadline environment:

```sh
P13_ITEM=after-sections-W200 P13_SECTIONS_WORLD=W200 python3 tools/art_pipeline/run_agent_sections_p13b.py /Users/rexxa/work/art-p13b-measure /Users/rexxa/work/art-production13b-evidence/after/sections-W200-attempt1
P13_ITEM=after-sections-W-multi P13_SECTIONS_WORLD=W-multi python3 tools/art_pipeline/run_agent_sections_p13b.py /Users/rexxa/work/art-p13b-measure /Users/rexxa/work/art-production13b-evidence/after/sections-W-multi-attempt1
```

These are recipes, NOT executed. Supply P13_REFERENCE, P13_BUILD_PROFILE=release, future bounded P13_DEADLINE_EPOCH and the frozen accepted candidate env before launch; do not reset the retry ledger or reuse an output directory. Profile capture script does not itself stamp walker getter state, so retain candidate env/source identity separately and do not mistake the absence of a stamp for an OFF result.

## Exact prior W200 node-ablation invocation

Authoritative recorded script: `/Users/rexxa/work/art-production13-evidence/baseline/round2.sh:30`. It sets:

- checkout `/Users/rexxa/work/art-p13-measure`;
- P13_REFERENCE=`/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/calibration.json`;
- P13_BUILD_PROFILE=release;
- P13_BASELINE_SHA and P13_MEASURE_HEAD=`6d493fa8ade6d368eca8e7efa7ab3f668eea1827`;
- P13_MEASURE_CHECKOUT=measure checkout;
- P13_MEASURE_DYLIB_SHA256 from the debug-path dylib, after asserting identical release-path bytes;
- P13_MEASURE_SCRIPT_SHA256 from `scripts/test/art/perf_frames_p13.gd`;
- P13_VALIDITY_OUT=`/Users/rexxa/work/art-production13-evidence/baseline/ablation-W200-attempt2.json.validity.json`;
- P13_ITEM=baseline-ablation-W200; GODOT_BIN=`/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`.

Literal launch line37:

```sh
bash tools/art_pipeline/with_quiet_lock_p13.sh baseline-ablation-W200-attempt2 3600 "$GODOT_BIN" --path "$measure" --resolution 1920x1080 --script res://scripts/test/art/perf_frames_p13.gd -- --world W200 --phase baseline --ablation-only --out "$evidence/ablation-W200-attempt2.json" > "$evidence/ablation-W200-attempt2-runner.log" 2>&1
```

This is the57-block,150frames/block running2× W200 diagnostic described in `docs/art/production13/perf/running-diagnosis.md`; source raw data is `/Users/rexxa/work/art-production13-evidence/baseline/ablation-W200-attempt2.json`, summarized by `docs/art/production13/perf/baseline-ablation-W200.json`. The historical deadline is expired and must not be reused.

## Part4 equivalent and quiet-lock conclusion

No new `_p13b` collector copy is needed for lock routing. `perf_frames_p13.gd` explicitly leaves quiet/window lock ownership to its caller and already supports `--phase after --ablation-only --world W200`. Directly choose existing `with_quiet_lock_p13b.sh` instead of the historical wrapper. Both use the SAME physical lock `/Users/rexxa/work/.mac_quiet.lock`; the p13b wrapper changes hold/attempt ledger destinations to `docs/art/production13b/perf/holds.md` and `attempts.json`. It still invokes the existing calibrated `session_validity_p13.py` and window lock. This prevents accidental reuse of production13's ledger without inventing a new physical lock.

Concrete after recipe, NOT executed (leader must set accepted candidate switches and future task deadline from current authoritative state):

```sh
measure=/Users/rexxa/work/art-p13b-measure
evidence=/Users/rexxa/work/art-production13b-evidence/after
export GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot
export P13_REFERENCE=/Users/rexxa/work/new-world-art-prod/docs/art/production13b/perf/calibration.json
export P13_BUILD_PROFILE=release
: "${P13_DEADLINE_EPOCH:?set current task deadline, not historical Part13 deadline}"
export P13_MEASURE_CHECKOUT="$measure"
export P13_MEASURE_HEAD P13_MEASURE_DYLIB_SHA256 P13_MEASURE_SCRIPT_SHA256
P13_MEASURE_HEAD=$(git -C "$measure" rev-parse HEAD)
P13_MEASURE_DYLIB_SHA256=$(shasum -a 256 "$measure/rust/target/debug/libsim_bridge.dylib" | awk '{print $1}')
[[ "$P13_MEASURE_DYLIB_SHA256" == $(shasum -a 256 "$measure/rust/target/release/libsim_bridge.dylib" | awk '{print $1}') ]] || exit 2
P13_MEASURE_SCRIPT_SHA256=$(shasum -a 256 "$measure/scripts/test/art/perf_frames_p13.gd" | awk '{print $1}')
export P13_VALIDITY_OUT="$evidence/ablation-W200-attempt1.json.validity.json" P13_ITEM=after-ablation-W200
bash tools/art_pipeline/with_quiet_lock_p13b.sh after-ablation-W200-attempt1 3600 "$GODOT_BIN" --path "$measure" --resolution 1920x1080 --script res://scripts/test/art/perf_frames_p13.gd -- --world W200 --phase after --ablation-only --out "$evidence/ablation-W200-attempt1.json" > "$evidence/ablation-W200-attempt1-runner.log" 2>&1
```

Use a fresh attempt suffix and ensure the evidence parent directory exists. Preserve terminal status/log plus validity snapshot. Then use unchanged `summarize_ablation_p13.py AFTER_SESSION OUTPUT_PREFIX BEFORE_SESSION`; it validates all required nodes, neighboring all-on control blocks, simulator-off/floor roles, release provenance and restoration, and recomputes medians. Do not run matrix `load_session` on ablation-only output: it intentionally rejects that shape. Neither node ablation nor exclusive sections is end-to-end kernel performance acceptance, and their numbers must not be added together.
