# Pass 17 execution plan

Authority: `PASS17_GOAL.md`, preserved unchanged. This plan implements the entire brief; it does not replace any threshold, identity check, cohort, artifact, or publication condition. The supplied brief overrides the repository's reset/main/merge workflow and skill approval/scaffolding defaults. Execute autonomously and record choices in `DECISIONS.md` under `## Pass 17`.

## Status and dependency order

| Part | Status | Deliverable | Depends on |
|---|---|---|---|
| 0 | completed | Original brief, standalone v20 pins, two opt-in rule fields, baseline I1–I4 | Current checkout/remote/lock inspection |
| 1 | completed | Complete routine v1, v21 wiring, activity bridge/trace | Part 0 complete and compatibility inventory recorded |
| 2 | completed | Stage A tuning freeze, I1–I4, performance | Part 1 verified |
| 3 | in_progress | Committed Stage A pre-registration, evaluation, committed verdicts, saved I5 baseline | Part 2 freeze |
| 4 | pending | Complete family v1, v22 wiring, family bridge/trace | Part 3 verdict commit exists |
| 5 | pending | Stage B tuning freeze, I1–I5, performance | Part 4 verified |
| 6 | pending | Committed Stage B pre-registration, evaluation and verdicts | Part 5 freeze |
| 8 | pending | English mechanics docs, independent review, exact-head gate, publication, Korean report | Parts 0–6, or explicitly recorded brief-authorized drop |

At each Part start and after compaction, re-read that Part's source sections in `PASS17_GOAL.md`. Record wall start/end, active duration, waits/pauses and notes in the RUNLOG. Push WIP after each Part. Finish the current step after 16 active hours, push and proceed to Part 8; preserve required Stage A/identity/performance evidence. Apply the brief's drop order only: family fission, household beds, all Stage B, Stage A tuning round 2. A missed metric bar is a recorded FAIL, never grounds to rewrite its bar or omit its value.

## Immutable execution contract

- Work only in `/Users/rexxa/work/new-world-pass4` and the one authorized detached base worktree `/Users/rexxa/work/new-world-p17-base`. Raw probe/recorder/timing data and retained binaries go in `/Users/rexxa/work/p17-data/`. Do not inspect or modify another checkout; the art remote ref is read through Git in this checkout only. No DGX, ComfyUI, dev server or unrelated process termination.
- `CARGO_BUILD_JOBS=6`; Cargo tests use `-- --test-threads=6`; probes and the recorder use `--workers 4`. Run heavy cohorts sequentially, so independent four-worker pools do not accumulate.
- Preserve `.claude/settings.json` and its skip-worktree bit, the user's `.gitignore` edit and `.ignore`. Stage explicit files only. Never run `harness_pipeline.sh`, install hooks, reset the checkout, push `lead/main`/`main`, force-push, or merge PR #510.
- Allowed changes: `rust/**`, `tools/revitalize/**`, `docs/revitalize/**`, except **nothing** beneath `rust/crates/sim-test/src/worldmodel/**`. Every other path in brief section 2 remains untouched.
- Only `AgentRules.routine` and `SocietyRules.family` may be new fields of existing public structs. No new fields in `Tallies`, `FissionRecord`, `AgentSnapshotRow`, `AgentDetailRow`, `ProbeArgs`, `ProbeOptions`, `RunSpec`, `SimResources`, or their existing dictionaries. New components/types/getters are allowed. `get_agent_detail` retains nine keys.
- Old scenes, variants, recorder and stress defaults remain unchanged. New runtime behavior must be behind the appropriate opt-in rule; `None` paths perform no new draws, insertions or mutations. Preserve `DEFAULT_GAME_SEED = 20261010`.
- New per-person arithmetic belongs in pure `sim-core` functions with extreme-value tests; integer and permitted `f64` arithmetic only. Register every new stream in `STREAMS`; identity traits use replicate zero. Deterministic component mutations use AgentId-sorted vectors, never hash iteration.
- Commit subjects use `revitalize(p17-N): <intent>`, with relevant Lore trailers. Before every push run `git remote -v` and `git branch --show-current`; afterward record `git ls-remote origin refs/heads/codex/revitalize-2026-10 refs/heads/codex/revitalize-2026-10-wip`. Only a green gate permits fast-forwarding the simulation branch.

## Verified implementation surfaces

These are current source paths, rather than the abbreviated names in the brief:

| Surface | Existing entry point or behavior |
|---|---|
| Rules and streams | `rust/crates/sim-core/src/rules.rs`; `rust/crates/sim-core/src/rng.rs` |
| Component exports | `rust/crates/sim-core/src/components/mod.rs` |
| Routine decision and movement | `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs`; `runtime/agent/movement.rs` |
| World generation/founders | `rust/crates/sim-systems/src/worldgen/mod.rs`; `worldgen/camp.rs` |
| Family registration | `rust/crates/sim-systems/src/lib.rs::register_rhythm_systems`, currently Maturation then Beds then Aging |
| Births, beds and fission | `runtime/settlement/settlement_system.rs::run_births`; `runtime/agent/beds.rs`; `runtime/settlement/fission.rs` under `sim-systems/src` |
| Bridge | `rust/crates/sim-bridge/src/ffi/world_node.rs`; `build_game_engine` currently v20 and `build_stress_engine` v19 |
| Scene selection | `rust/crates/sim-test/src/scenes.rs`; `probe/args.rs`; `probe/runner.rs::twin_checked` |
| Trace flag | `rust/crates/sim-test/src/probe/agent_trace.rs::ProbeOptions::split` removes `--agent-trace` before `ProbeArgs::parse`; no new public option field is needed |
| Existing trace files | `<scene>_s<seed>_people.jsonl`, `_agents.jsonl`, `_fissions.jsonl`, `_trace_meta.json` |
| Performance | `rust/crates/sim-test/src/perf.rs`: `stress --seeds A..B --agents LIST --side 64 --days N --feed fed --rules SCENE --out DIR`; `quick --out DIR`; no `--workers` option |
| Existing gate | `tools/revitalize/gate_pass16.sh <log_dir>` |
| Existing identity reference | `tools/revitalize/identity_p16.sh <release_bin_dir> <probe_out_dir> <data_dir> <git_sha>` |
| Existing timing reference | `tools/revitalize/perf_p16.sh <out_dir> <final_bin_dir> <base_bin_dir>`; `quiet_lock_p16.sh` |
| Scaling analysis | `tools/revitalize/perf_p15.py --natural DIR --stress DIR [--map DIR] --out DIR` |

Graft was used to locate these surfaces before source inspection. Before editing a symbol, the responsible executor should inspect its callers and nested instructions; discovery does not authorize edits outside its lane.

## Ownership and parallel lanes

The coordinator owns Git, RUNLOG/decisions, shared interfaces, execution scheduling and final integration. Assign one writer per file at a time. Each executor is told that others are working in the checkout and must not revert their edits. Read-only review/discovery may overlap; Stage B code may not start before Part 3's verdict commit.

| Lane | Exclusive write ownership | Starts after | Acceptance handoff |
|---|---|---|---|
| C: coordinator/interfaces | `rules.rs`, `rng.rs`, new `sim-core/src/components/activity.rs` and `family.rs`, component export modules, `sim-systems/src/lib.rs`; all Git and shared docs | Part 0 pin commit for Rust | Exact public contracts and opt-in defaults committed; other lanes use them |
| P: frozen proof (coordinator) | New `rust/crates/sim-test/tests/harness_revitalize_pass17_frozen.rs` | Pin capture before any other Rust edit | Standalone pin commit; original values never refreshed after behavioral code changes |
| A: routine runtime | `sim-systems/src/runtime/decision/agent_decision.rs`; `runtime/agent/movement.rs`; new `harness_revitalize_pass17_routine.rs` | C routine interface complete; Part 0 complete and pushed | All seven activities, arbitration/preemption/exit cases, deterministic tests |
| T: trace/evaluator/wiring | `sim-test/src/probe/agent_trace.rs`, `probe/args.rs`, `probe/runner.rs`, `scenes.rs`, `perf.rs`, optional private trace module; `sim-systems/src/worldgen/mod.rs`; new `tools/revitalize/evaluate_pass17.py` and `test_evaluate_pass17.py` | C enum/rule contract agreed; Part 0 complete and pushed | Numeric inputs for every stage metric, all scene dispatch/wiring, observer neutrality, evaluator tests |
| G: compatibility/bridge | `sim-bridge/src/ffi/world_node.rs`; probe `fingerprint.rs`, `deaths.rs`, `pass12.rs`; narrowly required None additions in older frozen whole-struct fixtures; focused bridge tests | C enum/components available | Exhaustive matches, preserved old fields/pins, pure collectors/getters, preserved row/dictionary shape |
| B: family runtime | New `sim-systems/src/runtime/agent/pairing.rs` and private household helper; `worldgen/camp.rs`, `runtime/agent/mod.rs`, `runtime/agent/beds.rs`, `runtime/settlement/settlement_system.rs`, `runtime/settlement/fission.rs`; new `harness_revitalize_pass17_family.rs` | Part 3 verdict commit, then C family interface and explicit worldgen ownership handoff | Mutual pairs, exact parent selection, households and deterministic fixtures |
| E: experiment tooling | New `tools/revitalize/identity_p17.sh`, `quiet_lock_p17.sh`, `perf_p17.sh`, `gate_pass17.sh`, and focused tooling tests where needed | Read-only survey now; write after coordinator assignment | Exact CLI shapes, full cohorts, fail-closed evidence validation, no forbidden paths |

If pure routine/family functions live beside their rules in `rules.rs`, C owns their integration: A/B submit the code through a single-owner handoff, rather than editing that file concurrently. G owns probe and bridge exhaustive matches; A owns decision/movement matches. T performs the scene/worldgen/performance wiring described under C below through this agreed ownership handoff; C retains integration responsibility. Existing public types are not expanded as an easy integration workaround.

## Part 0: immutable baseline and opt-in contracts

1. Verify the supplied attachment equals `docs/revitalize/PASS17_GOAL.md` byte for byte, record its SHA-256, and commit it as a Part 0 documentation commit. Inspect current branch, HEAD, remotes, local changes, skip-worktree status and existing processes/quiet lock. Do not reset user changes.
2. Before any other Rust change, capture v20 and v20_inert for seeds 0–4 at ticks 1,440 and 40,320 using the unchanged simulation. Follow `harness_revitalize_pass16_frozen.rs`: retain `fingerprint` and `worldmodel::digest::state_digest` values, then replace capture code with assertions. Assert each existing `WorldRules::game_v20(seed)` field individually, including nested values and seed-varying values. No whole WorldRules/AgentRules/SocietyRules literal.
3. Run the new frozen harness with six threads and capture the source SHA and output. Commit **only** `harness_revitalize_pass17_frozen.rs`. This commit must precede every other Rust change; store its hash in decisions and the final commit ledger.
4. Define new `RoutineRule` and `FamilyRule` types and the two permitted option fields. Keep both `None` in default, v8 and all existing scenes. Exhaustively find `AgentRules {`/`SocietyRules {` literals; only add the missing `None` fields to older frozen fixtures. Unit-test that old scenes remain disabled. Avoid enabling v21/v22 in this Part.
5. Create the single detached worktree at exact `adfdecfece79ab019964729319389fd602685fab`, release-build `sim-test`, and retain provenance plus hashes of `worldsim_probe`, `worldsim_wm`, and `worldsim_perf`. Keep base binaries separate from the main checkout's target directory.
6. Run base I1–I4 as defined below. Commit summaries and recorder digest list into `docs/revitalize/probe/p17-0_base/`; raw output stays under `p17-data/base/`. A runnable script and successfully spawned process are not completed identity evidence: require complete seed counts, expected days, determinism fields and terminal exit status.
7. Read-only art compatibility inventory before Part 1: `git grep` the remote art ref for `TargetKind`, `AgentState`, all changed public struct names and corresponding whole-struct literals. Inspect every relevant exhaustive match rather than relying on the literal search alone. Record ref SHA and findings in `docs/revitalize/COMPAT_PASS17.md`; do not change art files or that checkout.
8. Run focused rule/pin verification, commit permitted interfaces and Part 0 evidence, update RUNLOG, push WIP and record remote heads.

Part 0 QA: new v20/v20_inert fingerprints and digests pass unchanged; existing v19 pins still pass; base I1–I4 all present; only the two permitted public fields were added; standalone pin ordering is provable from Git history. Failure case: compare script must return nonzero for an altered non-timing value or a missing recorder array.

## Identity evidence: I1–I5

Use absolute binary/data paths and set `WORLDSIM_GIT_SHA` to the producing commit. The new P17 script preserves P16's measurement options but runs the cohorts sequentially. Existing P16 script's parallel I1/I3 behavior is not required and would duplicate worker pools.

```bash
export CARGO_BUILD_JOBS=6
export GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot
git worktree add --detach /Users/rexxa/work/new-world-p17-base adfdecfece79ab019964729319389fd602685fab
cargo build --manifest-path /Users/rexxa/work/new-world-p17-base/rust/Cargo.toml --release -p sim-test
```

The following are verified existing binary arguments. Replace `BIN`, `OUT`, `DATA` and `SHA` with the producer-specific absolute paths; do not overwrite an earlier run.

```bash
export WORLDSIM_GIT_SHA="$SHA"
"$BIN/worldsim_probe" --scene game_v19 --seeds 0..32 --days 120 --clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30 --sample-every 120 --workers 4 --check-determinism --out "$OUT/i1_v19"
"$BIN/worldsim_wm" --scene game_v19 --seeds 12000..12002 --days 120 --every 60 --workers 4 --out "$DATA/i2"
"$BIN/worldsim_probe" --scene game_v16 --seeds 0..32 --days 120 --clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30 --sample-every 120 --workers 4 --check-determinism --out "$OUT/i3_v16"
"$BIN/worldsim_probe" --scene game_v20 --seeds 0..32 --days 120 --clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30 --sample-every 120 --workers 4 --check-determinism --out "$OUT/i4_v20"
```

| ID | Immutable comparison | Required evidence |
|---|---|---|
| I1 | Base vs Stage A final and Stage B final, v19, 32×120 days | Summary JSON deep-equal after deleting only timing keys; exact omitted key paths logged |
| I2 | Base vs Stage A final and Stage B final, recorder v19 seeds 12000 and 12001, 120 days, every 60 ticks | Complete sorted relative-file SHA-256 manifest for every `.npy` and `interventions.json`; equal file count/names/hashes; metadata/timing excluded only as in P16 |
| I3 | Base vs Stage A final and Stage B final, v16, 32×120 days | Same deep comparison as I1 |
| I4 | Base vs Stage A final and Stage B final, v20, 32×120 days | Same deep comparison as I1, plus new v20 pins |
| I5 | Stage A final commit vs Stage B final commit, v21, 32×120 days | Same probe options and deep comparison; capture baseline before any Stage B code |

For I5 use the I4 command with `--scene game_v21` and an `i5_v21` output directory. Stage A's 17000–17031 evaluation cannot substitute for seeds 0–31. Save the Stage A release binaries and producer SHA in `p17-data/stage-a-bin/` before Stage B starts; do not create another worktree. Each stage's identity evidence has its own directory and comparison log: `p17-a_identity/` and `p17-b_identity/`. No tolerance, refreshed pin, JSON field whitelist expansion, or new recorder metadata is an acceptable identity repair.

## Part 1: daily routine v1

1. C exposes the precise ActivityKind/Activity/ActivityRest and RoutineRule contract from sections 6.1–6.2, `TargetKind::Activity`, the three routine RNG streams and distinctness registration. Implement pure weight, duration, rest-window and eligibility calculations. Verify P=0/500/1000, all trait weight extremes, rest endpoints and minimum duration.
2. A adds place discovery: allowed kind/life stage, walkability, no building tile, stated Chebyshev rings and hearth radius, no Craft place in daughters without stockpile. Build occupancy once per tick after stale activity cleanup in AgentId order; cap reserved places at two; update occupancy within the sorted decision loop. One redraw only, then a 30-minute no-place rest.
3. Add activity entry exactly after Gather and before settlement fallback. Preserve every preceding predicate and require no loaded haul. Use offset-adjusted clock and lineage-key start staggering; an active ActivityRest blocks entry.
4. Implement pre-match exits 1–5, setting Idle before the existing match so the unchanged Idle cascade executes that iteration. Work interruption must use the full existing forage/firewood/gather predicates including actual tile lookup under current claims. Include builders, social eligibility and peers, meal wait and person-specific fatigue threshold. If no response actually fires and execution reaches routine again, add only a one-tick rest.
5. Implement activity Seeking with a retained SeekTarget, exact-place arrival and arrival-based duration. Normal duration/day-end and settlement-change exits create keyed rest; walking timeout creates rest. Keep early preemption distinct from normal completion. Stale components removed before occupancy include combat-forced Idle holders. Make activity agents Idle-equivalent in builder/social/combat candidate pools without duplicating social event/bias logic.
6. Add child Play movement constrained to Chebyshev one of the reserved place; other Consuming activities remain at the place. Activities never alter inventories, production, consumption, resource amounts or Tallies.
7. C wires v21 as v20 plus routine, same generator, probe scene allowlists/builders/twin_checked and performance rule parsing/labels. Do not add v21 to recorder scene parsing or change recorder schema.
8. G maps Activity Seeking and Consuming to state 4/target 8 in both collectors; updates round-trip tests; adds a pure activity collector and `get_activity_snapshot()` with exactly `{ids,kinds,place_x,place_y,until_tick}`. Retain old row/dictionary shapes.
9. T handles every probe exhaustive match and extends trace with activity starts/minutes by kind, normal completed counts/durations, hourly states by life stage, no-place redraws and provenance. Record enough transition evidence to recompute one person's activity accounting by hand. Trace collection may add private observer state/new types, but no public fields or Tallies. Off/on trace runs must have identical simulation fingerprints.
10. Focused verification: extreme-value unit tests; crafted no-place and crowding cases; stale activity cleanup; loaded worker deposit; meal, social, builder and need interruption; no reachable work tile despite stock deficit; timeout; settlement change; exact exit-6 completion; clock offset; deterministic two-engine run; trace off/on identity; bridge dictionary shapes. Test seeds remain below 1000. Commit Part 1 code and logs, push WIP.

QA commands for new harnesses, once created, use `cargo test --manifest-path rust/Cargo.toml -p sim-test --test harness_revitalize_pass17_routine -- --test-threads=6` and the same command with `--test harness_revitalize_pass17_frozen`. Module tests run with the appropriate package/module filter. Preserve the output and exact code SHA under `verification/p17-1/`; these focused checks do not replace the final workspace gate.

## Part 2: Stage A tuning, proofs and performance

1. Build release binaries only when no foreign quiet lock is held. Run v21 and paired v20, seeds `17100..17116`, 60 days, `--agent-trace`, four workers, hourly clock samples. Store raw files in `p17-data/a-tuning/r1/`, summaries and evaluation outputs in `probe/p17-a_tuning/r1/`.
2. The new evaluator CLI is an implementation contract, not an existing command: `python3 tools/revitalize/evaluate_pass17.py --stage a --candidate DIR --baseline DIR --tuning --out-md FILE --out-json FILE`. It must check cohort completeness, reject evaluation seeds in tuning mode, and calculate all measurable R1–R3 and health context.
3. At most one round 2, using the same tuning seeds only. If too idle, choose either 10–30 rest or base duration 120; if too busy, apply the reverse adjustment. R3 misses may increase only the specified axis factor or duration span 60→90 with 20-minute floor. Log first values/results, rationale, exact final values and second values/results. Never retune after an evaluation probe begins.
4. Freeze Stage A code. Run I1–I4 and both pin harnesses against the base; collect complete performance evidence below. Repair implementation defects without weakening proofs or thresholds, and invalidate/re-run affected outputs if code changes.
5. Save Stage A final producer binary hashes and stage identity/performance outputs, commit Part 2 and push WIP. Stage A metric failures remain recorded failures; no missing numbers.

## Shared quiet-lock and performance protocol

Implement `quiet_lock_p17.sh` by adapting the verified P16 interface `LOG PURPOSE COMMAND...`, with P17/Codex ownership and P17 data stamp. P16 only logs excessive hold duration afterward; P17 must enforce the brief's 60-minute ceiling and release on normal exit, error or signal. Only terminate descendants launched by this wrapper, never unrelated processes. Split timing into bounded batches rather than letting a whole suite overrun the hold. If a timed process reaches the bound, retain timeout evidence and plan a bounded retry; never report partial output as a passing run.

Before any build/test/probe/Godot/gate, check the foreign lock; while held poll at five-minute intervals and record waits. Stale lock threshold is strictly older than two hours. Before a timed hold, require no other-run `cargo`, `rustc`, `worldsim_*` or `Godot` processes by exact process-name inspection and one-minute load below four. After 60 minutes waiting for quiet, acquire atomically if available and mark the measurement contended. A still-valid foreign lock is not overridden merely because the quiet-load wait expired. Holds are at most 60 minutes, spaced at least ten minutes, and contain timing only; builds and identity/evaluation probes run outside holds.

For each stage, use one frozen candidate executable for both rule labels in the six interleaved stress measurements. Stage A labels are v20 then v21; Stage B labels are v21 then v22. Repeat that pair three times; record all six files, run order, elapsed time, machine/load/process checks, producer SHA and binary hash. Run the reference-producer and final-producer `quick` before/after for context. Stage A reference producer is adfdecfe; Stage B reference producer is Stage A final. Baseline `quick` retains its default v19 world.

Verified stress invocations, with distinct output directories per repeat:

```bash
"$FINAL_BIN/worldsim_perf" stress --rules "$REFERENCE_RULES" --seeds 15100..15101 --agents 500 --side 64 --days 1 --feed fed --out "$PERF_OUT/a/r1"
"$FINAL_BIN/worldsim_perf" stress --rules "$CANDIDATE_RULES" --seeds 15100..15101 --agents 500 --side 64 --days 1 --feed fed --out "$PERF_OUT/a/r1"
"$REFERENCE_BIN/worldsim_perf" quick --out "$PERF_OUT/quick_base"
"$FINAL_BIN/worldsim_perf" quick --out "$PERF_OUT/quick_final"
"$FINAL_BIN/worldsim_perf" stress --rules "$CANDIDATE_RULES" --seeds 15100..15101 --agents 100,500,1000 --side 64 --days 2 --feed fed --out "$PERF_OUT/b_candidate"
"$FINAL_BIN/worldsim_perf" stress --rules "$REFERENCE_RULES" --seeds 15100..15101 --agents 100,500,1000 --side 64 --days 2 --feed fed --out "$PERF_OUT/b_reference"
```

The first two commands must also run into `a/r2` and `a/r3`; one repeat is insufficient. The last reference slope is useful context, while the candidate slope against the historical 1.06 is the required bar. Reuse the existing P15 scaling computation with its real `--natural`, `--stress`, `--out` shape, or expose an explicitly tested P17 analysis wrapper over the same calculation. Do not replace the slope with an endpoint runtime ratio.

Required acceptance and artifacts:

- Median of the three **mean tick** measurements: candidate/reference increase ≤15%, and candidate ≤4.0 ms.
- Median of the three **p99.9 tick** measurements: candidate/reference increase ≤35%. Report every repeat and medians; do not select the fastest run.
- `AgentDecisionSystem` three-size stress slope is strictly <1.7; report it beside historical 1.06 and retain all sizes' per-system data.
- Before/after quick JSON must be valid and nonempty; inventory six paired runs, both quick runs, full candidate slope and reference context. Size-limit or missing-file outputs are explicit failures/incomplete measurements, not zeros.
- Commit JSON summaries, analysis JSON/Markdown, runs/provenance and lock/wait logs under `probe/p17-a_perf/` or `probe/p17-b_perf/`; raw sidecars remain outside the repository. Copy each hold/wait into RUNLOG. Separate contended observations from quiet ones.

## Part 3: Stage A pre-registration and evaluation

1. Append the exact section 6.7 table under `METRICS.md` → `## Pass 17` → Stage A. Specify final tuned constants, seed ranges, 120-day duration, hourly sampling, exclusions, point-estimate bars, bootstrap and trace schema. Commit this registration **before either** v20 or v21 is probed on `17000..17032`; save the registration commit SHA in experiment metadata.
2. Use the following cohort form for each scene, serially. `--agent-trace` is already supported at the binary boundary; v21 scene support is delivered by Part 1.

```bash
"$BIN/worldsim_probe" --scene game_v21 --seeds 17000..17032 --days 120 --sample-every 120 --clock-samples 00:00,01:00,02:00,03:00,04:00,05:00,06:00,07:00,08:00,09:00,10:00,11:00,12:00,13:00,14:00,15:00,16:00,17:00,18:00,19:00,20:00,21:00,22:00,23:00 --workers 4 --check-determinism --agent-trace --out "$DATA/a-eval/v21"
```

3. Run the same command for v20 into its own directory. Evaluate using the new Stage A evaluator contract without `--tuning`, producing exactly `docs/revitalize/probe/p17-a_eval/eval.md` and `eval.json`. Preserve summaries, complete seed inventory, producer SHA, registration SHA and raw-data hashes.
4. R1 pools non-elder adult hourly observations 08:00–17:00 inclusive, Idle tag 0, and must be 0.10–0.45. R2 reports all paired components: day-120 alive ±10%, non-old-age deaths ≤+1.5/world, shelters ≥0.9×, social/non-elder-adult-day ≥0.9×, noon food and wood each ±15%.
5. R3 uses Spearman average ranks, non-elder adults with ≥10 adult days; zero-activity people remain in NS/RD share analyses with zero shares. P additionally requires ≥3 normally completed arrived activities. Bars: P and NS ≥0.25, RD ≥0.20; each interval excludes zero. World-level bootstrap B=2,000 with `numpy.random.default_rng(0)`, preserving within-world people and pairing. Activity minutes exclude travel. Report no-place redraws beside NS.
6. Report R4 identities/pins and R5 performance alongside estimates/intervals. Evaluator tests cover constant ranks, empty denominators, pooled rather than per-person idle, zero-activity inclusion, excluded interrupted durations, paired seed handling, deterministic bootstrap and altered/missing inputs. Missing data must never become a PASS.
7. Capture v21 I5 baseline at seeds `0..32`, 120 days on the final Stage A code, and retain Stage A release binaries outside the repo. Commit the Stage A verdicts and I5 baseline. Only after that commit may Part 4 begin. Push WIP and record exact commit ordering.

## Part 4: family v1

1. C/B add Sex and Kin components, pure sex/kinship helpers, STREAM_SEX and game_v22 = game_v21 plus FamilyRule. Founders/newborns receive Sex and Kin immediately beside the existing Lineage/Hardiness/Temperament insertion region, in a fixed order, only when family is enabled. Existing family-None code paths retain prior insertion order.
2. Add PairingSystem at priority 144, registered immediately after MaturationSystem and before BedAssignmentSystem 145. With family None return before any draw/write. At offset-adjusted 20:00 clear missing/dead partners first, then settlement-id order and female-AgentId order. Only living, unpaired, non-child, non-elder opposite-sex members; exclude direct recorded parents/children and shared known mother/father. Highest familiarity wins, then lower male id; ≥300 permille initially; make partnership mutual. Relationship hash keys are sorted before iteration.
3. In `run_births`, change only parent selection after the existing birth gates pass. Pick qualifying couple by oldest mother's last birth (None first), then lower mother id; both partners must be alive now, mother a non-elder adult. Mother drives Lineage::child and births count; both drive temperament; child Kin and causal parents are `[mother,father]`; update mother's last_birth_tick. With no qualifying couple preserve old lowest-two-id fallback and record it in trace. Under family None the old branch remains verbatim. Audit every `parent_ids[0]` reader and record semantic effects.
4. Build a deterministic household helper usable by beds/fission and trace: couple plus Child-stage children, mother's household if father dead, otherwise singleton. Handle duplicate/conflicting links once in AgentId order, with explicit fixtures. Bed assignment first-fits whole households by smallest AgentId into current/next shelter; oversized households span shelters; no room means no bed. Preserve independent housing_capacity and report bedless people/unused first-fit slots.
5. Fission preserves existing trigger/ranking/elders exclusion. Seed whole households by best-ranked adults until min_adults, allowing seed size above k up to group_max; skip any household exceeding group_max. Fill whole households in best-member rank if they fit remaining k. Only if below group_min after trying all households, top up ranked individuals and count splits. Store departure couples/splits and households in trace, never FissionRecord.
6. G adds pure family collector and `get_agent_family(entity_bits)` with exactly `{found,female,mother,father,partner,children}`, -1 missing ids and packed living-child ids. C/G move init/reset to v22; stress reset stays v19. If the authorized Stage B drop applies, live init/reset moves to v21 and family deliverables are explicitly marked dropped.
7. T records per-person sex, current partner/pairing tick, parents, household key, shelter at each 20:00 sample; birth fallback/day; fission household and couple denominators. Keep enough observation history to reconstruct one complete family and F3/F4/F5 denominators without event-ring guesses.
8. C wires v22 into all scene dispatch/allowlist/twin_checked/performance labels, flipping unknown-scene tests only for new valid scenes. Recorder remains untouched.
9. Focused family tests cover exact 20:00 offset, stable registration order, no-op family None, stale/dead partner clearance, deterministic matching/ties, all recorded close-kin exclusions, birth fairness/dead partner/fallback and untouched gates, oversized/no-capacity bed packing, household seed above k, skipped over-max household, group_min individual top-up, bridge getter found/missing/dead children, and trace off/on neutrality. Use seeds <1000. Commit Part 4 and push WIP.

## Part 5: Stage B tuning, proofs and performance

1. Run v22 and paired v21 on `18100..18116`, 60 days, hourly samples, `--agent-trace`, four workers. Store round-1 summaries/evaluation under `probe/p17-b_tuning/r1/` and raw outputs under `p17-data/b-tuning/r1/`.
2. If pooled partner share F1 misses 0.50 at day 60, one tuning change is allowed: familiarity threshold 300→200, then one round-2 cohort on the same tuning seeds. Preserve both rounds and freeze. No other family threshold or matching-rule tuning is authorized.
3. Run I1–I4 against adfdecfe, I5 against saved Stage A final, both frozen pin harnesses, and full v22-v21 performance protocol. I5 must be from Stage A final code, not rebuilt after adding family. Keep complete differences as evidence on any failure.
4. Commit Stage B frozen code/evidence, update decisions and RUNLOG, push WIP. A failed F1 or performance bar is retained as such; it does not authorize extra tuning or changed thresholds.

## Part 6: Stage B pre-registration and evaluation

1. Commit the exact section 7.7 rule table under METRICS Pass17 Stage B with final threshold and all statistical eligibility conventions, before either paired scene runs on `18000..18032`. Record the pre-registration SHA and producer SHA.
2. Run v22 and v21 separately for 120 days with the Part 3 hourly/trace/determinism options and seeds `18000..18032`. Run the new evaluator with `--stage b --candidate DIR --baseline DIR --out-md docs/revitalize/probe/p17-b_eval/eval.md --out-json docs/revitalize/probe/p17-b_eval/eval.json`.
3. F1: pooled partner share of non-elder adults at day 60 ≥0.50. F2: births from day 20 inclusive with couple parents ≥0.90, with daily total/fallback counts including earlier days. F3: mean of four Pearson sibling correlations minus analogous unrelated-pair mean ≥0.06, interval excludes zero; every eligible pair appears in both orders. F4: coupled alive bedded partner-nights at 20:00 sharing shelter ≥0.90. F5: split couples/completed-departure parent-settlement couples ≤0.10, with exact numerator/denominator. F6: all R2 health rules against v21. F7: I1–I5/pins. F8: performance.
4. Pre-register F3's unresolved sampling detail with the chosen default below; do not choose the cohort after seeing evaluation correlations. Use the same B=2,000 world bootstrap/default_rng(0) convention for stage intervals. Preserve ineligible/empty worlds and count reasons; do not report zero denominators as a successful fraction.
5. Commit complete numeric verdicts, raw-data hashes and eligibility counts, push WIP. No post-evaluation retuning; any actual bug repair requires updated producer evidence and explicit preservation of the original attempted results.

## Spec-consistent defaults and issues to record

These are planning decisions for `DECISIONS.md`, not permission requests and not changes to the source brief.

- **Activity place rank:** the supplied keyed rank has the same inputs for every candidate tile. Follow the literal comparison `(occupants, keyed_u64(seed, STREAM_ROUTINE_PLACE, key, tick, 0, rep), y, x)`; its same-person keyed value ties and final ordering is y/x. Do not invent a tile hash or change slots silently. Record this fact and test deterministic occupancy; report crowding/redraw consequences.
- **Rest precedence:** section 6.2's detailed exit instructions govern the earlier general sentence: exits 1–5 do not impose random rest, only a one-tick guard if the unchanged Idle cascade returns to routine; normal exits, timeout and failed placement use their specified rests.
- **F3 same-settlement reference:** as agreed with the coordinator, use people co-resident at a recorded snapshot, count each unordered pair once and then both orientations. Exclude unrelated controls with known direct parent-child or shared mother/father links. Freeze age/time eligibility in pre-registration: first qualifying shared hourly snapshot is the reference; record the unordered Child/adult/Elder stage pair and admit unrelated controls only in stage-pair categories represented by eligible full siblings in that world. Full siblings require both known parent ids equal. Report pair/world counts and stage categories so this convention is inspectable. Do not infer founders' unknown ancestry.
- **Trace evidence is an implementation prerequisite:** normal exit-6 completion, no-place redraws and fallback parent choice must be observable without adding fields to existing public structs or Tallies. The coordinator selected a new event component `RoutineObservation { tick, kind, outcome }`, with outcomes `Started { redrawn }`, `NoPlace { redrawn }`, and `Ended { completed }`, to expose exact transitions to the probe. It is a new type/component, never a cumulative production counter; insert/update/remove it deterministically and only under routine, and have the trace consume each tick's observation exactly once. Test multiple transitions in one decision tick so an end/start cannot silently overwrite required evidence; retain both in a new observation payload type if needed rather than changing an existing public type. Resolve fallback-parent trace evidence in Part 4 before tuning. Record the final mechanism and hand-verifiable fixtures; never estimate these counts from end-state snapshots alone.
- **Empty metric populations:** output explicit missing/undefined estimates, counts and a non-passing verdict; a missing required number triggers further evidence work or an honest limitation, never a fabricated zero. Failed bars with real numbers are valid deliverables.
- **Gate timing:** the inherited gate's quick timing is a validity check; the dedicated quiet-held stage protocol establishes the performance bar. The gate waits for foreign lock release, does not hold a timing lock over compilation/tests, and retains all inherited step semantics.
- **No localization changes:** this pass adds Rust simulation/bridge behavior only; art and all English/Korean localization files are outside scope. The English docs give the art line the new codes/getters and visible movement expectations.

## Part 8: documentation, review, final gate and publication

1. Write `ROUTINE.md` and `FAMILY.md` in English with exact final constants, formula arithmetic, entry/exit logic, Idle equivalence, pairing/parents/households, identity limitations and unmodelled features from section 8. State activities produce nothing; no extra courtship/divorce/remarriage/fertility-age rules; unknown founder kinship is unmodelled; sex has no other effect. Give art integration state 4/target 8, both getters and missing visuals. If a Stage B piece was dropped, state precisely which and preserve measurements of completed pieces.
2. Independent review: no more than two rounds, each with a fresh reviewer given only PASS17_GOAL, the diff and outputs. Audit I1–I5/pins, public-field freeze, exhaustive matches, decision-arm placement, complete preemption predicates, candidate-pool equivalence, birth gate preservation, no counters in Tallies, provenance, seeds/pre-registration order, metrics, allowed paths and quiet-lock protocol. Manually recompute one person's activity minutes/completion duration and one family's parent/partner/household/fission facts from trace. Record findings/dispositions under `REVIEW_PASS17.md`.
3. Drive the actual output surface: run the frozen probe/trace evaluator and bridge collector smoke cases, then Godot smoke through the normal game builder. Observe activity walking/dwelling and family getter values in actual runtime evidence; do not claim the separate art rendering is delivered by this pass. No scripts/scenes/assets edits are allowed for visual polish.
4. Create `gate_pass17.sh` from the verified P16 gate, preserving all steps: prepare bridge/import, debug and release workspace tests, normal and all-target clippy with warnings denied, forbidden-math scan, bridge build, import, strict check, bounded smoke with error scan, both Python discoveries with `WM_BIN=/nonexistent PROBE_BIN=/nonexistent`, quick JSON validity, and allowed_paths. Only update P17 labels/output locations and allowed-path base to exact `adfdecfece79ab019964729319389fd602685fab`; prohibit recorder subtree even though under rust. Preserve P16 and prior gate files unchanged.
5. Commit the final code/docs before the authoritative gate, then run `CARGO_BUILD_JOBS=6 GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot bash tools/revitalize/gate_pass17.sh docs/revitalize/verification/p17_gate/`. Record producing HEAD, every step's exit code, test/skip counts, smoke errors and allowed-path inventory. Run `git diff --check` and inspect staged paths separately, because gate's committed-tree comparison does not police uncommitted files.
6. If gate findings require code changes, commit repairs and re-run the whole authoritative gate. If logs/report are committed afterward, the final publication commit must itself receive a successful exact-head gate into external data; retain that run's bound SHA and use a subsequent evidence-only commit only if its own gate is also verified. Do not claim an ancestor's green gate proves a later HEAD. Keep a stable external gate log path to avoid a self-referential committed log/hash cycle.
7. Push WIP, verify its remote SHA, then fast-forward `codex/revitalize-2026-10` to a proven-green WIP commit using ordinary non-force Git operations. Re-read remote heads afterward. Update the description of existing draft PR #510 with Pass17 scope, Stage A/B outcomes, evidence links, remaining failures/limits and exact publication SHA. Keep the PR draft and unmerged. Remove only the authorized base worktree after its binaries/raw identities are safely retained.
8. Write `REPORT_PASS17.md` in Korean using **every heading in section 10, unchanged**, including complete R/F tables, intervals, hourly v20/v21/v22 state shares, activity-kind minutes, health comparisons, tuning rounds, performance, machine facts, lock waits/holds, review findings, decisions, per-Part times, limitations and honest judgment. Include changed files by commit and every commit hash/Part, both verified remote heads and PR update evidence. The final message repeats that report text as requested. Do not replace it with a short success summary.

## Completion audit and evidence index

| Requirement | Authoritative proof needed before claiming complete |
|---|---|
| Original goal and pin-first ordering | Attachment/document hashes; standalone pin commit parent/history; immutable values |
| Allowed changes/public shape | Commit diff from adfdecfe plus explicit staged/local inventory; field/literal audit; bridge nine-key test |
| Stage A behavior | Focused runtime fixtures, trace off/on identity, real cohort trace, bridge/runtime observation |
| Stage B behavior | Family fixtures, real birth/partner/bed/fission traces, getter/runtime observation or explicit authorized drop |
| Frozen old worlds | Base/A/B I1–I4 comparisons and v19/v20 pins; I5 Stage A→B |
| Tuning/pre-registration | All tuning rounds, frozen constants, registration commits preceding first evaluation process timestamp/producer log |
| Statistical claims | Complete 32-seed paired 120-day evaluation summaries, numeric R/F table, intervals, denominator/eligibility counts and evaluator tests |
| Performance | Six interleaved stress runs per stage, medians, both quick runs, full three-size slope, provenance and lock records |
| Review | At most two independent fresh reviewer rounds, recorded findings, hand-recomputed person/family examples |
| Final technical gate | Exact publication SHA's successful full P17 gate, test/skip counts, zero unacceptable smoke errors, allowed paths |
| Publication | WIP and simulation branch `git ls-remote` evidence, green fast-forward target, updated draft PR #510, base worktree removed |
| Report | Complete Korean section-10 report, commits/Parts, per-Part active/wait times and explicit limits; final response matches |

Do not mark a requirement complete from a planned command, a partial cohort, a successful build, a process launch, an uninspected JSON file or a reviewer's assertion alone. Inspect the artifact and its producing code/commit. Preserve failed attempts and incomplete/contended evidence. The coordinator owns updating this table's execution status as each Part's verifiable deliverable lands.
