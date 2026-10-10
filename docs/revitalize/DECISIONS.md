# Revitalize decisions

Run started 2026-10-10 12:07 KST (03:07 UTC); nine-hour boundary 21:07 KST.
Authoritative specification: owner attachment, copied outside the clone to
`/home/hyunlord/work/revitalize-goal.txt`.

- Dedicated clone `/home/hyunlord/work/new-world-revitalize`, branch
  `codex/revitalize-2026-10`, base `d409e9b941ccc868747e4eb63953eafa6831bff7`.
- The owner explicitly overrides stale preflight/reset, main-branch push,
  interactive harness, and planning approval instructions. Never install git
  hooks or write harness audit/baseline/reviews or OMX/OMO state.
- Existing macOS checkout is not a delivery workspace. All implementation,
  builds, probes, and captures run on DGX.
- Host has no Rust or Xvfb in its default environment. Install the requested
  toolchain and Godot using official distributions; retain PATH explicitly.

## Execution and evidence ledger

| Part | Deliverable | State | Evidence |
|---|---|---|---|
| 0 | arm64 extension, baseline gates | complete | 1729 debug + release tests; clippy and Godot gates pass |
| 1 | tallies, shared scenes, probe, captures, frozen legacy pin | complete (capture exception) | 1748 tests, both Clippy checks, strict14 and smoke900 PASS; four legacy repeats deterministic |
| 2 | default-legacy rules, world generation, terrain/building FFI | complete | 1764 tests; both Clippy checks; strict14/smoke900; live Godot 48 checks |
| 3 | gated survival/growth loop, 64-seed tuning | implemented; survival gate blocked | four 64-seed twin batches PASS; 3-day zero-dehydration test fails |
| 4 | deterministic art, truthful renderers/HUD, visual QA | implemented locally; visual exception, global gate blocked | UI322 PASS/GPU96 unobservable; strict17; smoke900; ledger230 |
| 5 | full verification, Korean report, handoff | verified partial delivery; no push/PR | debug+release1781 PASS/1 FAIL each; fresh68 twins PASS; gh absent |

P0–P2 passed the owner gates before push; remote landing was checked against local HEAD. P3–P5 remain local because the retained survival gate fails. No legacy behavioral assertion may be weakened.

## Environment exception: captures

`xvfb-run` is absent and `sudo -n true` reports a password requirement. Per
Part 0 item 5, all screenshot capture steps are skipped on this host; no
password request and no substituted screenshots. Capture tooling will still
be delivered. Headless smoke and strict checks remain mandatory. Visual
acceptance cannot be claimed from static checks.

## Native legacy precision and measurement

V7 `Hunger.value` is f32, whereas thirst, sleep, social needs and health are
f64. Preserve the existing component and its exact legacy trajectory. The
fingerprint hashes native `Hunger.value.to_bits()` (4 little-endian bytes),
and native 8-byte bits for the f64 values. Metric aggregation widens hunger
to f64 and sorts agents before summation. No FFI f32 snapshot downcasts are
used for fingerprints.

The remote clone has no Graft CLI/index; the local macOS graph describes an
older checkout. Use bounded source reads on the remote clone rather than
using that stale graph as evidence.

## Part 1 boundary

Completed 2026-10-10 12:28:46 KST; Part 1 began after Part 0 push at ~12:18.
Full debug tests: 1748 passed, zero failed (baseline1729 +19 new tests),
110 result groups. Both Clippy checks: zero warnings. Strict GDScript14 and
smoke900: PASS; capture/camera standalone check-only: PASS. Forbidden math
scan and added-production unwrap scan: zero hits. Independent read-only
review found no blockers. Four 10-day legacy repeats matched every sampled
fingerprint. Raw evidence remains ignored; summaries are committed.
# Sparse berry edge repair

The exact recipe fails for seeds 247, 338, 345, 383, 426, 728, 759 and 773 in the deterministic 1,156-seed geometry audit (0..=1024, u64::MAX, 2^63, 2^63-1 and 128 high hashed seeds). Camp selection itself succeeded for every audited seed. The failures are fewer than 12 Forest/Plain berry candidates or fewer than three centres separated by six tiles.

When and only when the original candidates fail, exchange a local Plain edge tile in camp radius 6..=16 with a distant Forest tile outside radius16. Rank both deterministically by hash salt9 then y,x; while centres are missing require each new edge to be at least six tiles from the current greedy centres. Never exchange building or water tiles. Re-run the original hash-salt8 cluster recipe after every swap. This preserves total Plain/Forest/Hill/Mountain/River counts exactly, elevation bytes, river geometry, camp, buildings, resident positions and resources. It changes the strict moisture-quantile classification only for deficient seeds, an explicit necessary relaxation to satisfy the requested exact bush count.

All eight failing audited seeds recover with 1..7 swaps and retain exact global class counts. Original successful audited seeds perform zero swaps. Evidence: p2-extended-geometry-results.json and p2-repair-geometry-results.json. These are standalone Python algorithm reproductions, not compiled Rust evidence or a proof over all 2^64 seeds. Runtime harness includes all eight regression seeds and passed the Part 2 full gate.

## Generated camp formation

The required tick-2 all-24-members invariant conflicts with the legacy
lexicographically-first resident anchor: movement precedes formation and a
ring-three camp can span more than radius five from its westernmost resident.
For worlds with shelter planning enabled, prepend real registered Hearth
tiles (sorted y,x) to the formation candidates. Preserve all thresholds,
private birth bookkeeping, causal events, and the exact default candidate
path. This narrow Part 2 rule read is necessary before Part 3 behavior wiring;
no settlement state is fabricated by worldgen or GDScript. A focused test
checks both legacy and generated anchors; worldgen invariants verify 24
members by tick two for normal and repaired regression seeds.

`reset_world` emits its notification deferred: synchronous listeners would
reenter snapshot getters while Rust still holds the mutable node borrow.
Reset preserves simulation speed and rebuilds only this node's engine.
Building snapshots contain registered buildings only; queued camp buildings
appear after the first simulation tick, with no invented render-only ids.

## Part 2 boundary

Completed 2026-10-10 12:43:22 KST; began after Part 1 push at ~12:29.
1764 debug tests passed, zero failed (Part1 +16); 111 result groups.
Both Clippy invocations, forbidden math, production unwrap scan, strict14,
and smoke900 pass. Actual Godot main-scene FFI/reset test passed 48 checks
and three deferred reset callbacks, including reentrant snapshot reads.
The 16-seed untuned 10-day probe is a diagnostic only: rules are defined but
Part 3 behavior is not wired yet; median dehydration deaths272 and alive103.5
are not survival acceptance. Renderer ledger records eight replaced static
test functions and three wording-only provenance corrections; no behavioral
predicate, frozen pin, or test retirement changed. Captures remain skipped
under the documented environment exception.

## Part 3 scope conflict and tuning stop

Initial seed1 trace identifies all 11 dehydration deaths as normal social Seeking Agent.
No construction or GatherFood state occurs among thirst>=80 samples. A resident
stands on infinite shore water at thirst100 while following a nonreciprocating
peer; normal social seeking has no urgent-need preemption. Legacy pin passes.
A real escape transition would change social behavior forbidden by sections8.3
and12, so do not add it. Three allowed knob rounds preserve determinism but do
not solve dehydration. Keep round2 under the stated priority order: it passes
non-extinction and has fewer deaths than the other non-extinct configuration.
The new survival assertion remains unchanged and failing; do not re-pin, ignore,
or delete it to make the gate green. Part3 cannot be pushed under the every-push
green rule. Continue Part4 locally and retain all evidence, then report this
explicit conflict and the last green remote boundary. Night-rhythm stretch
is skipped because T1 and T6 do not pass.

## Part 4 local boundary

Truthful terrain, registered buildings, depleted berry stages, simulation-time
HUD, localized inspector/feed, selected-only paths, daylight and agent
interpolation/shadows are implemented. Static renderer source pins were
replaced under the explicit ledger policy: 230 baseline assertion occurrences
map to 113 replacement function contracts; no tests were retired.
Actual headless main-scene QA passed 322 checks. An explicit nonzero sentinel
proved MultiMesh GPU transform readback unavailable in the dummy renderer:
96 coordinate checks are NOT_OBSERVABLE, never counted as passing. Reset
CPU position caches and interpolation origins pass. Strict check passed
17 files and 14 FFI calls; smoke900 passed. The fresh final full debug/release
runs are recorded separately in Part 5. All-target Clippy is clean after
removing orphan comments for deleted static-test helpers.
No Xvfb and no passwordless sudo: exercise the Part 0 item 5 capture exception.
No screenshot, contact sheet, visual-quality acceptance, or ten-minute 4x
render performance claim is made. Part 4 stays LOCAL because Part 3 retains
the genuine survival failure and every pushed head must be green.

## Final partial delivery

Both final native suites run with --no-fail-fast:1781 passed,1 failed,0 ignored,113 result groups each. All expected1782 tests accounted for. Only required three-day survival fails seed1:dehydration11 versus0. Seeds2/3 inside that failed function were not reached. Both Clippy checks, strict17/14, smoke900, diff check pass; forbidden math0, changed-production unwrap calls0. Fresh68 twin runs and full legacy/selected-round sample comparison pass.

No gh executable: draft PR skipped. Last green remote remains c438026. To preserve unpushed implementation without violating green-push policy, export local P3–P5 commits as a verified incremental git bundle plus complete evidence and report in an external handoff archive. Download only that delivery copy to macOS; all implementation remains on DGX. This is the explicit partial-delivery end state, not a claim that all goals were achieved.

## Pass 2

Start:2026-10-10 16:49:46 KST. Eight-hour boundary:2026-10-11 00:49:46 KST.
Authoritative goal:/home/hyunlord/work/revitalize-pass2-goal.txt.
PASS2_BASE_SHA=c70b0c67e7103f42b300ccc1ed892e8143803c89.
Pass1 P3–P5 are now secured on origin/codex/revitalize-2026-10-wip at that SHA.
Green branch remains c43802600db209b0b32b20fbaecb5f7f8ddf8cf5 until a full green gate.
All commits immediately push WIP, including failures; green only fast-forwards.
Persistent handoffs copied to /Users/rexxa/work/revitalize-handoff-2026-10-10/
and /home/hyunlord/work/revitalize-handoff-2026-10-10/; SHA256 verified.

Initial game_v8 values:move_interval_ticks8,idle_pause_chance0.4,
idle_tether_radius8; Provisioned cooldown360,food_cost3,max_mean_hunger60,
max_mean_thirst60; housing residents4,max_shelters12,plan_interval60,
rings2..5,required_progress240,max_builders4. Worldgen bushes8/capacity4.

Execution plan and gates:
- Part0 complete: remote safety line and persistent copies.
- Part1 in progress: every-tick death census, display-only social tally,
  schema2 additive metrics; measure unmodified P5 behavior before any fixes.
- Part2 queued: exact rule-gated social bounds and need fallthrough, clock
  offset480, new staged regressions; unchanged survival and legacy pins.
  Generic critical preemption only if post-fix16-seed census meets5% trigger.
- Part3 queued:64-seed10day and16-seed30day twins; at most3 permitted tuning
  rounds; retain best by ordered targets, stretch only if every target passes.
- Part4 queued: Mac Aqua tierA detected; fresh dedicated captureclone only.
  Clock-based final/baseline captures, entity sidecars, V1–V6, displayed418 UI
  harness and direct image inspection. If tierfails15min, follow B/C/D.
- Part5 queued: separate mutation worktree, exhaustive token mutants for
  canonical ledger contracts, per-mutant restore, strengthen survivors and
  explicit reasons for any survivor. Never replace source-text tests wholesale.
- Part6 queued:final full gate/probes, report, WIP push, green fast-forward
  only with gate evidence; draft PR via authenticated Mac gh then DGX.

Independent agents prepare code in owned paths or ignored staged copies;
leader integrates, runs heavyweight checks serially, commits and verifies
remote heads. Max6 active children. No unavailable Graft graph is used as
source evidence: DGX has no graft/rg, so bounded grep/source reads are used.

### Pass 2 Part 1 schema compatibility

Adding the requested public Tallies.social_completed field makes the frozen
legacy test full Rust struct literal fail to compile. Preserve every original
expected value and the fingerprint, tick, population and replay assertions;
mechanically represent the original four-counter comparison as a tuple.
This is a source compatibility adapter, not a re-pin: the test file is not
byte-unchanged, and no equality implementation ignores the new counter.
Other two full literals explicitly set the added field to0; the new social
unit test independently verifies exact per-pair counts and unchanged outputs.
The social counter test passed; full legacy trajectory verification follows.

### Part1 completed

20 probe unit tests, social counter regression, legacy/tallies tests and all-target Clippy PASS.64 twins PASS;7,744 original behavioral rows identical. Partner-alive is measured before lethal phase; simultaneous deaths can report alive. Part2 review completed two rounds with objections resolved. Part1 metrics prose follows in a supplemental commit because report generation raised a formatting exception; the code and complete JSON/Markdown summaries were already committed and pushed safely. Boundary UTC:2026-10-10T08:05:01.159403+00:00.
