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

Initial existing-API regressions: `steward_defence`0passed/3failed and `colony_flee_mux2`0passed/5failed before production edits (external defence-red.log/flee-red.log). The latter includes a pending wolf bite after roof entry; immunity must apply at fight resolution as well as target selection.
