# M-UX2 decisions

## D1 — Scope, execution and evidence

The owner supplied `MUX2_GOAL.md` verbatim; commit `f2103ad47` preserves it before implementation. The isolated branch is `v8/mux2`, based on verified remote `0207b49760a31a71968b441298fb52b073791cb4`. The explicit branch and unattended execution instructions override generic work-on-wip and planning approval defaults. No owner question or extra approval handoff is required. No writes, builds or game runs occur in pass4 or play2-dev. No other process is signalled.

Raw evidence belongs in `/Users/rexxa/work/mux2-data`. The previous M-UX dependency lock is preserved there as `base.Cargo.lock` (SHA256 `ddc99ac9fc161d9b610c0f5d8dbf5e279c8908c10717647ba50ab2fef70c36d1`), recovered from the prior archived dependency input, not the differing pass4 lock. The baseline debug bridge is built with this lock before product edits. Runtime checks share one Godot lane; strict import, capture and performance never overlap.

Target ten hours, hard stop fourteen hours from this request (approximately 2026-10-26 03:06 KST). A hard stop does not authorize lowered bars, publication of a failed gate, or a completion claim. Optional drop order remains exactly the goal's order.

## D2 — Behaviour and acceptance boundaries

Renderer/UI/performance-preserving work and simulation behaviour changes are separate commits. Baseline identity remains mandatory. Steward-OFF colony fixtures are byte-compared before claiming a preserving change; defence, flee and hauling changes have predicted differences recorded before their measurements. No golden rewrite, metric exclusion or threshold relaxation is permitted to obtain green.

Calibration is seeds 0–7 only, at most four logged rounds. Seeds 1200–1231 are held out until policy/source and scorer hashes are frozen. Both defence ON and OFF use the same frozen policy and 48-day duration, steward ON, no player orders. The OFF arm is a measured comparison, not an assertion that old A8 counts must repeat on new seeds.

## D3 — Measurement contracts

Visibility is measured in rendered screen pixels for every on-screen person, seed0 day1/day20, zoom0.5/0.25, day and night. Use same-scene ground controls and per-person attribution; do not silently skip overlap, small silhouettes or hidden persons. Retain the original 60/140-pixel and luminance-difference 0.20 bars. Record camera, time, all ids, image hashes and counterexample crops; test the validator with missing people and deliberately poor contrast.

The stall measurement uses the normal debug bridge and automatic wall-time accumulator, seed0, steward ON, five minutes per zoom1/0.5. Keep startup frames and all intervals over33ms; assess the bar only after the first10 seconds. Require no interval over50ms and at most3 over33ms in each full minute, with partial-minute count and aggregate rate reported separately. Attribute measured work spans; unmeasured costs are unknown, not zero. Before/after use the same harness. Windowed visibility, frame timing and visual QA cannot be replaced by headless dummy-renderer results.

## D4 — Publication and issue ownership

Only verified fixed registry-created issues in #513–#619 may be closed, with a comment linking the fixing commit and registry row. Preserve unrelated issues. Publish only a frozen committed HEAD with the complete gate status0, then explicit fast-forwards `v8/mux2:v8/wip` and `v8/mux2:v8/main`; a refusal stops publication without force. Verify remote refs, remove only owned worktrees, and deliver the prescribed Korean report. The committed report records completed checks available before its commit; final exact-HEAD gate and publication receipts bind the subsequent facts without a self-referential commit hash.

## D5 — Defence interpretation and ADR reference correction

The goal calls this an ADR0035 amendment, but repository ADR0035 is terrain colour and ADR0036 defines the steward. Amend ADR0036 and explicitly preserve this correction; do not rewrite the terrain decision. The contradictory phrases 'armed adults' and 'arm the unarmed among them' are implemented as at most four eligible weapon-capable adults: existing weapon rank first, then adults with distinct reachable available stock weapons, health descending, camp distance and stable ID. Never count one available spear for several prospective defenders. Exclude every person below50HP and every child. A valid player's explicit draft/undraft intent, including a no-op state reaffirmation and choices made while steward OFF, protects for one calendar day; player-owned drafted people are never auto-undrafted. The independent defence toggle defaults ON whenever overall steward is enabled, without changing Housing/Food scope selection.

Quiet radius is measured from the village hearth. Retreat releases owned defenders immediately; otherwise require60 consecutive ticks with no threatening wolf within24tiles, resetting on re-entry. Recruitment records actual accepted count once per batch. Existing StewardAction event value remains its monotonic sequence; a new mechanical Chronicle event may carry defence count. This is a behavior/event type, not a content enum.

Initial existing-API regressions: `steward_defence`0passed/3failed. The first `colony_flee_mux2` run had four behavioural failures and one missing-BodyHealth fixture error (external flee-red.log). After correcting that fixture, haul-flee-red2.log records five actual behavioural failures before production edits, including a pending wolf bite after roof entry. Immunity must apply at fight resolution as well as target selection.

Recruit once per approach response, not every frame: normal critical-need self-undrafting must allow eating rather than immediate automatic re-recruitment. Existing defenders can update targets; a new pack or a retired pack entering the camp radius permits a new response. This preserves the specified approach trigger without inventing an extra hunger eligibility exclusion.

## D6 — Baseline measurements and causal limits

Diagnostic commit `15b282a1e` and the frozen debug bridge completed both windowed baseline arms with no run errors and complete instrumentation. At1×, after10seconds,10frames exceeded33ms and7 exceeded50ms; maximum210.816ms, minute counts[3,1,2,1,3]. At½×,21frames exceeded33ms and6 exceeded50ms; maximum201.641ms, minute counts[3,1,4,3,10]. Both fail the unchanged bar. Raw frames and source/library hashes are in stall-before/stall-report.json; startup spikes remain recorded.

Some slow frames are dominated by GroundLayer work outside chunk-bake spans; others have no corresponding measured script spike. These are causal leads, not proof that minimap, alerts or GC caused the stalls. Add finer ground and native simulation timing before selecting fixes. Native GDScript has no tracing GC; reference-count destruction costs remain part of the measured frame.

The controlled hauling regression fails2positive cases and passes4legality/control cases: sustained splitting leaves144new fuel on the ground and0stocked within1440ticks. This proves starvation in the controlled workload, but does not yet prove the cause of historical seed5's90ground fuel. Reproduce that saved-evidence scenario separately and retain any different cause.

The evaluation observer uses exact transient fatal-animal records from the resolver, not generic Combat deaths or proximity inference. These records do not enter saved state or influence policy. A physical-wall housing oracle replaces the older roof-plus-door shortcut only for the new evaluation. The old scorer remains unchanged.

## D7 — First implemented simulation policy, before calibration

The first policy uses single-batch approach response, at most4eligible defenders, HP>=50, one-day manual veto, deterministic augmenting physical-weapon assignment and complete proposed-team weapon/health/distance/id ranking. This fixes measured3-of4 under-recruitment and a measured3-spears-plus-axe selection despite a feasible4-spear team. Manual claims remain fixed. Fast source commands export/replay before flight; ordinary sources remain after assignment. OFF defence state participates in fingerprints when nondefault. Defence16/fingerprint1 pass, including a720tick full-engine JSON replay.

Flight/race/roof attribution tests12 and existing wildlife tests12 pass. Fuel hauling moves before competing jobs only at that worker's Haul priority; full or incompatible or unreachable stockpiles and higher player priorities still prevent it. Hauling11pass. Historical seed5's old binary reproduced the90ground/0stock state; its loaded geometry has9compatible reachable cells, no fuel reservations and one worker repeatedly hunting/chopping. This is broader same-priority starvation, not only split-before-haul. Raw reproduction and inspection are preserved under hauling/.

The migration fixture initially failed because its process had not installed the content catalogue; corrected bootstrap loads all4genuine format10 files, resaves format11 and verifies continuation. Retain both v10-migration.log and v10-migration2.log. Legacy wire layout test also passes. The paired evaluation validator requires identical source/policy/binary provenance and complete32seed arms; OFF bar failures are descriptive, not acceptance failures. Observer4 and scorer14tests pass. Calibration and held-out runs have not occurred at this checkpoint.

Visibility candidate1/2 are diagnostic pixel evidence only: a concurrent test build could replace the on-disk bridge during capture. Preserve those reports but do not use them as final binary-bound acceptance. The amended harness hashes the bridge and all relevant inputs at start/end and fails any mutation; all subsequent acceptance captures serialize builds and Godot, with frozen source. Baseline visibility and stall captures were frozen and remain valid.

## D8 — Revalidation and attributed stalls before calibration

Root full GDScript strict check passed 72 scripts/59 FFI calls. Extended real-window why inputs passed 30/30; Korean text candidate3 passed 1,652 checked node instances at both scales with the actual day20 save. Candidate2 used an incorrect `.save` path rather than `.wsave`; its load failure and missing-room/item results remain retained, not accepted. Independent screenshot review remains authoritative for visible content beyond shaping checks.

Visibility oracle now passes 20 negative/control checks including source mutations. Final GPU candidate acceptance remains pending. Diagnostic1 failed before sampling due to a typed conditional Array literal; only the owned failed Godot process was terminated. Diagnostic2 captured 120 seconds and 14,033 current-frame native samples, but its initial paused-world coverage check incorrectly made completeness false. Preserve this report as diagnostic, not acceptance. Measured long frames include StewardSystem128.568ms and wall preparation170.860/78.745ms; one81.581ms frame remains outside timed paths. No threshold is relaxed.

Additional review exposed stock matching downgrading a constrained defender to an axe despite a free farther spear. Root regression failed before repair (defence-free-spear-red.log); calibration has not started. The correction must optimize feasible actual weapon ranks, not merely find any matching.

The documentation system-table generator implicitly invoked a release build in the child lane. It finished before these measurements; no diagnostic2 performance capture overlapped a build. Future table refreshes use the already-captured systems JSON.

D8 verification update: rank-constrained feasible matching passes17 defence tests and2,058 exhaustive small-graph optima plus reversed-input determinism; the same results pass after splitting the pure solver into a bounded child module. Raw logs defence-green4.log and defence-green5-split.log. The slow planner optimization has a separately executed frozen eager-reference comparison before edits (16generated fixtures); it is not part of this policy correction.
