# Art Pass 13b tooling inventory

Read-only source inspection; commands below are execution recipes, not evidence that a run passed. No Godot or cargo was launched during this inventory.

## Copies required to preserve Pass 13 evidence

- `with_quiet_lock_p13.sh` unconditionally exports `P13_HOLDS=$repo/docs/art/production13/perf/holds.md`; an incoming environment value is overwritten. Copy to `_p13b`, redirect that destination or introduce an override in the copy.
- `run_alternating_p13.py`, `run_agent_sections_p13.py`, and `run_perf_matrix_p13.sh` explicitly invoke that wrapper. Copy them to `_p13b` to select the copied wrapper.
- The matrix runner also hardcodes its final summary prefix under `docs/art/production13/perf/`; redirect the copy to production13b.
- Underlying collectors, validators, probe and identity runners can remain `_p13` unchanged. Their output paths are arguments or environment variables.
- `session_validity_p13.py` supports `P13_REFERENCE`, `P13_ATTEMPT_LEDGER`, and `P13_HOLDS`; use absolute production13b paths. Its admission cap is six attempts with 30-minute spacing and three hours total waiting. Independently enforce Pass 13b's three-strike failing-target cap, including debug runs.
- Do not assume copied-tool names below already exist. Complete their bounded routing changes before executing those recipes.

## Common environment

```bash
export P13_REFERENCE=/Users/rexxa/work/new-world-art-prod/docs/art/production13b/perf/calibration.json
export P13_ATTEMPT_LEDGER=/Users/rexxa/work/new-world-art-prod/docs/art/production13b/perf/attempts.json
export P13_BUILD_PROFILE=release
```

Set `P13_DEADLINE_EPOCH` from this run's original start and ten-hour bound, not a renewed deadline. Section runner requires a future deadline. All measured launches require the release library and debug-path library to have identical SHA-256. The gate rebuilds debug, so restore verified release parity after gate execution and before measurements.

## Part 0: calibration and gate

```bash
python3 tools/art_pipeline/cpu_probe_p13.py calibrate "$P13_REFERENCE"

CARGO_BUILD_JOBS=6 CARGO_PROFILE_DEV_OPT_LEVEL=2 CARGO_PROFILE_TEST_OPT_LEVEL=2 \
GATE_OUTPUT_DIR=docs/art/production13b/gates/p0 \
bash tools/art_pipeline/gate.sh p0
```

Calibration performs exactly 15 probes, waits 20 seconds after each completed probe except the last, preserves partial JSON and process snapshots, and computes R as the median of the fastest five. Run calibration before Godot sessions.

The existing gate routes only `test_real_input_p12.gd` and `test_real_input_p13.gd` through the window lock/windowed Godot. Other tests, including layout and scroll, run headless in this gate. The explicitly requested windowed layout/scroll reruns remain separate obligations. Use new gate output directories for retries because `results.tsv` is overwritten.

## Part 0: identity reference

The base worktree must be at `8c1b18be8b05bfeaa062a937526fe4d9c2f53414`, kernel on, write skip off. These unchanged shell runners accept external output directories and preserve previous runs by refusing existing run directories.

```bash
export P12_DECLUMP_KERNEL=1 P13_WRITE_SKIP=0
export P13_P3_PLAN=/Users/rexxa/work/art-production13-evidence/identity/six/A/p3-plan.json
export P13_P3_TARGET_TICK=46189

bash tools/art_pipeline/run_identity_p13.sh /Users/rexxa/work/art-p13b-base /Users/rexxa/work/art-production13b-evidence/identity/six A
bash tools/art_pipeline/run_identity_p13.sh /Users/rexxa/work/art-p13b-base /Users/rexxa/work/art-production13b-evidence/identity/six Aprime
bash tools/art_pipeline/run_running_identity_p13.sh /Users/rexxa/work/art-p13b-base /Users/rexxa/work/art-production13b-evidence/identity/running-base
```

Existing read-only P3 plan: `[["RESET",20261013],[33,"09:49","FV","all"]]`. Prior sidecar confirms tick 46189. A production13b evidence copy may be used instead. The identity runner copies the supplied plan into each run and hashes it.

Running runner defaults to all five required cases: W0, W200, W-multi, W500_FROZEN, W500_2X. The collector implements the 600-frame running cases and 240-frame W500 pan cases. Do not narrow `P13_RUNNING_WORLDS` and then claim complete identity. Both runners use `with_window_lock.sh`, wait for the quiet lock, and hash source/build/HEAD before and after launches.

## Part 1: write-skip identity and pairs

Capture candidate B on the same Part 0 source/build with `P12_DECLUMP_KERNEL=1 P13_WRITE_SKIP=1`; retain independent A and Aprime with skip off. Changing the font between reference and candidate introduces unrelated differences. Use a new running-candidate directory.

Validator contracts (replace placeholder paths with actual directories):

```bash
python3 tools/art_pipeline/pixel_identity_p13.py A_DIR APRIME_DIR B_DIR NEW_COMPARISON_DIR --candidate write-skip --kernel-state on
python3 tools/art_pipeline/compare_running_identity_p13.py RUNNING_BASE RUNNING_CANDIDATE NEW_REPORT_JSON --candidate write-skip --kernel-state on
```

The candidate flags verify both skip polarity and the fixed enabled declump kernel in metadata. Pixel comparison requires exact zero RGBA differences for both A/Aprime and A/B in every framing. `pixel_identity_p13.py` imports Pillow, Typer and Pydantic; check the installed Python environment before running, without adding dependencies unsolicited.

After copied-runner/wrapper routing:

```bash
python3 tools/art_pipeline/run_alternating_p13b.py /Users/rexxa/work/art-p13b-measure write-skip /Users/rexxa/work/art-production13b-evidence/write-skip/W500-attempt1 W500 --kernel-state on
python3 tools/art_pipeline/run_alternating_p13b.py /Users/rexxa/work/art-p13b-measure write-skip /Users/rexxa/work/art-production13b-evidence/write-skip/W200-attempt1 W200 --kernel-state on
```

Each invocation already executes three alternating pairs: six blocks, each with 30 discarded and 150 counted frames. The runner pins the kernel on, starts skip off, and toggles skip per block. It requires release profile/parity and writes launch, session, validity and summary evidence into the supplied fresh directory. W500 is frozen; W200 runs at 2x. Acceptance requires valid admission, pooled median <=0.97 on both workloads and at least two of three faster pairs on each, in addition to complete identity.

## Part 4: frozen-build after measurements

After the build is frozen and release parity is verified:

```bash
bash tools/art_pipeline/run_perf_matrix_p13b.sh after /Users/rexxa/work/art-p13b-measure /Users/rexxa/work/art-production13b-evidence/after
P13_SECTIONS_WORLD=W500 python3 tools/art_pipeline/run_agent_sections_p13b.py /Users/rexxa/work/art-p13b-measure /Users/rexxa/work/art-production13b-evidence/after-sections-W500
P13_SECTIONS_WORLD=W200 python3 tools/art_pipeline/run_agent_sections_p13b.py /Users/rexxa/work/art-p13b-measure /Users/rexxa/work/art-production13b-evidence/after-sections-W200
P13_SECTIONS_WORLD=W-multi python3 tools/art_pipeline/run_agent_sections_p13b.py /Users/rexxa/work/art-p13b-measure /Users/rexxa/work/art-production13b-evidence/after-sections-Wmulti
```

The matrix defaults W0, W500, W1, W200 and W-multi. W1000 is optional explicit opt-in. It resumes previously accepted sessions, preserves rejected attempts, and executes at most one fresh attempt per world per invocation. A successful process exit alone does not establish a complete matrix: inspect all required accepted sessions.

Section runner generates instrumented GDScript in the external output directory, validates 300 counted frames and installation hashes, and preserves ordinary source. W500 supplies frozen section timers; W200 and W-multi supply both required running timers. Do not sum exclusive sections with node-ablation differences or getter costs.

Inherited Pass 13 section 4.5 also names SimReads getter timing and W200 running node ablation. `perf_frames_p13.gd` exposes `--ablation-only`, while the matrix defaults to skipping ablation. Execute an explicit separately admitted W200 ablation when fulfilling the full inherited matrix; optional W-multi ablation is droppable. Preserve build/collector provenance environment and use the copied quiet wrapper for that direct collector launch. The existing `summarize_ablation_p13.py` accepts output paths and can be reused.

## Inspection provenance

Sources inspected: `docs/art/production13{,b}/GOAL.md`, production13 TASKS/DECISIONS/worlds/running-diagnosis, the runners and validators named above, `gate.sh`, identity/performance GDScript collectors, and the previous P3 plan/sidecar. No nested AGENTS.md was returned by the scoped tracked-file search of tools/scripts/docs. This document records tooling behavior, not runtime acceptance.
