# M-CORE decisions

Status: implementation in progress; no acceptance or publication claim.

**Current performance rule: D10 supersedes the original ≤4 rule and D2's
historical admission requirement.**

## D1 — Published base and isolated ownership

Fetched origin on 2026-10-26. `origin/v8/main` resolved to
`9855bd082ad88a4c49296410f206f148d98c4df9`, containing
`docs/v8/mux2/MUX2_REPORT.md`. Created `/Users/rexxa/work/new-world-core-dev`
on `v8/core` from that SHA. The first commit, `001f236c3`, preserves the
owner's complete goal. Source changes and runs use this worktree only.

The release probe/perf build completed successfully before implementation.
Frozen executables, dependency lock, SHA256 manifest, build head and empty
Rust source diff against BASE are in `/Users/rexxa/work/core-data/base-bin`.
The build head differs from BASE only by the goal document.

## D2 — Measurement admission and evidence

The first observed one-minute load was 30.91; after the base build it was
67.20. Neither meets the owner's <=4 condition. No current decision share
is claimed; the historical 80% figure is not today's measurement.
Functional captures can run under load, but performance comparisons require
an admitted session without concurrent builds/probes from this task.

Use seed15100 colony stress scenes n200/n1000/n2000, 1440 warmup and 2880
measured ticks, three interleaved rounds. Decision mean is total_ms*1000/2880;
the reported share is the arithmetic mean decision cost divided by the
arithmetic mean tick cost. Final same-session comparison must measure both
frozen old and candidate executables and retain per-run load and survival.

Planner update (2026-10-26): concurrent M-UX2.1 Godot FPS runs and builds
occupied this Mac approximately 17:30–18:40 KST; the planner has stopped
that work until CORE publishes. All arm elapsed times from the initial
captures are explicitly **INVALID**, including values retained in the
inventory and external aggregate. Counts and identity remain functional
evidence. No share `s` or quiet performance number was accepted. Remeasure
both current per-system share and arm costs under the original load-≤4
condition; the update waives no goal requirement. Later observed load was
still 46.95, so the stopped planner work is not itself quiet-host evidence.

## D3 — Preserve behaviour until the single switch

Capture baseline identity, the nine OFF fixtures from `mux_safety.sh`, and
the two-seed24-day steward-ON fixture from the frozen source before edits.
Every pre-switch implementation commit must compare these outputs exactly.
Arm counters and shadow results live outside serialized simulation state:
they must not allocate causal IDs, evict tile events, consume RNG streams or
change fingerprints. Missing fixture coverage is a failure, not a zero.

Colony rules inherit relation, memory and routine settings from baseline.
V7 labels are insufficient evidence of unreachability. Port/replace/drop
verdicts require the requested execution counts and explicit replacements.

## D4 — World registration and save ordering

Concrete system constructors belong in sim-systems; sim-engine owns scheduler
metadata. Preserve this dependency direction. Current save loading registers
systems before installing saved resources (`save/mod.rs`); world-dependent
registration must receive the saved world rules first. New persistent mind
state requires explicit binary save version/migration handling. Append causal
event variants rather than renumbering existing variants. Verify active
behaviour, hysteresis and the bounded choice ring across save/load.

## D5 — Baseline boundary and existing wildlife ownership

Do not move shared need/clock/consumption helpers under baseline: colony jobs,
mood, wildlife, movement, resource regeneration and settlement systems use
them. The isolation check must reject indirect re-exports too.

DraftSystem (121) and FleeSystem (123) run before the decision slot (125).
The new behaviour registry must have explicit ownership of their continuation
and choice recording, without replaying their side effects. Shadow includes
these skipped old-seam ticks in its classification and coverage accounting.

## D6 — Evaluation arithmetic and held-out separation

Freeze policy after at most four logged calibration rounds on seeds0–7.
Held-out seeds are1232–1263 inclusive (`1232..1264`),48days, steward/defence ON,
without player orders. Compare BASE on those same seeds. Historical MUX2
measured values remain the acceptance reference: stockpile, houses/beds,
cold deaths and survival >=31/32; wolf deaths <=1 in >=29/32; ledger32/32.
Firewood: report exposed/32 and no-depletion count >=exposed-1. Do not reuse
the old scorer's fixed seed set or fixed32 cold denominator.

## D7 — Story-yield ancestry

Define depth as parent edges, not record count. Use durable chronicle cause
references because the tile ring evicts after eight events. Report missing
ancestors and cycles separately; do not manufacture chains from UI records.
The observer and definition must be identical for BASE and M-CORE.

The initial development instrument incorrectly excluded an entire path when
its older ancestor was missing, even after three verified edges. Schema 2
retains the verified prefix; a dangling reference adds no edge, and detected
cycles remain excluded and separately counted. Depth counts record edges,
not presentation-only pressure/modifier label nodes. Distinct chains are
distinct observed kind paths, excluding IDs and unresolved suffixes. A root
and a missing-ancestor path with the same verified kinds are one observed
kind path. Annual windows are half-open; partial years are not extrapolated.
Twelve focused tests pass, including depth-two/depth-three missing-ancestor
boundaries, cycles and ordinary-world fingerprint preservation. Development
schema-1 smoke evidence remains preserved and is not the reference metric.

## D8 — Arm observation scope and count correction

The off-by-default `perf_arms` feature records 247 registered keys. An early
source extraction counted 219 because its lowercase-only expression omitted
28 uppercase `.L` call-site keys; no source change explains that difference.
The runtime ON export independently confirms 247 keys. Timers include nested
work and instrumentation overhead; never sum them into a system cost or use
them as the quiet uninstrumented performance result. Count-only observations
have zero timing and distinguish evaluations, successful candidates, cascade
commits, continuations and cardinalities. Routine replay evaluations are not
additional unique person-ticks. Drivers discard setup and determinism-twin
observations and retain explicit zero rows.

Before the story observer correction, frozen default and feature binaries
both matched BASE's steward ON two-seed 24-day summary byte for byte, with
determinism PASS. The feature eight-seed 48-day ON summary also matched a
fresh BASE control exactly; ledger and reservation violations were zero.
Both initial binaries also pass the full baseline and nine OFF comparisons:
228,748 shared simulation values with zero differences and zero one-sided
paths, 64 chronicle files equal in content, equal raw sample digests and
determinism PASS. The strict arm aggregator validates all 11 fixture groups,
66 worlds, 3,824,640 world-ticks and 247 scope rows. The corrected final
observer binary also passes the same full comparison and the ON 24-day
fixture. Its schema-2 story command matches the ordinary probe fingerprints
on two 25-day worlds, including one full and one partial calendar year.
Raw provenance and receipts live in
`/Users/rexxa/work/core-data`; no held-out seed has been run.

## D9 — Separate CORE outcome scorer

`tools/v8/core_score.py` preserves the published MUX2 scorer and validates
CORE's exact seed set, complete observer coverage, physical housing and
fatal-predator provenance. It binds summaries and receipts to the scorer,
source and binary hashes, and validates matched BASE/new-brain controls.
Freeze chronology remains an explicit pre-run attestation, not a property
proved by equal hashes. Historical BASE outcomes are descriptive controls;
the new brain is tested against D6's registered thresholds.

Receipts bind the imported validator file as well as the scorer itself;
mutation and mismatch controls reject stale bindings. Independent literal
tests lock the six fixed thresholds to 31/31/31/31/29/32.

Unexposed firewood rows are null, neither pass nor fail. The denominator is
the exposed count, the required count is `max(0, exposed - 1)`, and zero
exposure is labelled `NOT_EXPOSED`. Ten scorer tests and synthetic CLI
controls pass. Outcome-row success does not claim shadow, why coverage,
performance, UI or full CORE acceptance.

## D10 — Planner corrects admission to the standing ledger rule

Authority: [planner correction, 2026-10-26 22:00 KST](CORE_PLANNER_CORRECTION_2026-10-26.md).
The original goal is retained verbatim in commit `001f236c3`; the working goal now incorporates this correction. The operative rule for §1.1 and §5
is now the [performance ledger](../../perf/PERF_LEDGER.md), with its M4
[§3.6 provenance](../m4/M4_GOAL.md): every repeat must start at one-minute
load **strictly below 8**; record all 1/5/15-minute values before each repeat.
A repeat starting at ≥8 is contended/INVALID, retained and never used as a
comparison base. A post-run load is diagnostic, not an added acceptance
condition. No extra ≤4 or sustained-minute gate applies.

Every old/new comparison interleaves both frozen binaries in the same
session. Use the within-session ratios for the mind and total-tick bars;
the initial old-brain-only share establishes the design baseline but cannot
replace that later matched comparison. Keep previous failed admission logs
and invalid arm timings; they lack the required per-repeat admission proof
and are not retroactively promoted by this correction.

If three recorded windows 15 minutes apart remain at ≥8, proceed with
inventory, mind, shadow and held-out evaluation without timing, and complete
timing before the gate. This explicit fallback supersedes the former
measure-first impasse only after its three-window evidence exists. The
remaining scope, calibration ceiling, held-out freeze, identity, publication
and isolation requirements are unchanged. The planner states M-UX2.1 is
paused until CORE publishes; actual host admission is still measured.

## D11 — Accepted share and replacement arm timings

All nine BASE repeats in `core-data/base-perf-ledger-1` satisfy D10 at both
controller admission and the benchmark's own start sample. At N=1,000,
the old decision mean is 1,462.832342 µs and total tick mean is
2,092.741204 µs, giving **s=0.6990029820337981**. The final paired total-tick
ratio bar is therefore **0.700498508983101**. These are design inputs,
not a completed old/new performance comparison; see [CORE_PERF](CORE_PERF.md).

The eleven separately admitted replacement arm groups retain identical
simulation summaries and all 247 aggregate/per-group invocation counts.
The accepted aggregate SHA256 is
`7eaf23ff385a0c5bb84d3b089282850a391547165f39618409d46876dce7d82e`.
The inventory now uses these costs and links the load log and hash-bound
validity receipt. The original arm captures and INVALID receipt remain
preserved; no historical timing was retroactively accepted.


## D12 — Routine rest draw follows the required person/tick/node key

CORE §3 requires randomness keyed by `(agent, tick, node)`. The legacy rest
sampler (`rules/routine.rs::rest_minutes`, called by `RoutineTick::end`)
uses `(world_seed, STREAM_ROUTINE, lineage_key_or_agent, tick, slot=2, replicate)`.
It has no node key and often uses lineage instead of AgentId, so retaining its
exact samples would violate the new mind's random-key contract.

**Explicit replacement:** keep `end_selected_node`'s rest draw using
`(world_seed, STREAM_ROUTINE XOR node.random_key, AgentId, tick, slot=2, replicate)`.
Both use the same configured inclusive interval `[rest_minutes_min,
rest_minutes_max]` and the same bounded keyed sampler. The sampled duration
and correlations change; the interval, completion condition, reservation
release, completion flag, Idle boundary, and deterministic activity duration
remain ports. Interruptions/invitations without a rest draw remain ports.
This replacement supplements the already explicit additive preference-plus-
noise replacement of the routine entry lottery; it does not replace all routine
continuation. No calibration or held-out result authorizes this decision:
the requirement itself fixes the key shape before those runs.

In the shipped policy, deadline completion is dispatched by
`continue_routine_{tend_fire,craft,repair,hearth_circle,explore,play,sit_by_fire}`;
the selected continuation node supplies the key. A data-added targeted `rest`
node may use the same lifecycle and supplies its own key. Do not mark these
whole continuation nodes `replace`: only their post-deadline rest draw changes.

**Denominator:** the accepted legacy inventory has 2,885
`selected.activity_seek_end` and 35,822 `selected.consume_activity_end` scope
entries (38,707 total), which identify rest-producing lifecycle exits, not a
unique-person-tick denominator. Their selected band/behaviour and one-tick
completion boundary remain port comparisons. This rest-draw replacement
removes **zero** additional person-ticks from the ≥97% port denominator;
shadow executes no new effects and hence does not sample the new rest.
Future rollout differences in rest duration must be attributed to this rule,
not used to exempt unrelated continuation or downstream choices. Routine entry
lottery replacements remain separately classified at actual entry only.
All held-out outcome, determinism, why, budget, and paired performance bars
remain unchanged. The ≤4 calibration-round ceiling is unchanged.

The arrival-at-walk-timeout edge is a fix, not a replacement: factual and effect
deadline checks use `Seeking(Activity) AND position != activity.place`, matching
legacy arrival before the walking-timeout decision at the reached tile.

## D13 — Explain the selected condition, not a fact-name heuristic

The inspector must not label a held job as motivated by hunger when its actual
predicate is hunger below the yield threshold. The immutable per-world record
dictionary now retains the selected node's predicates, score terms and constant,
with exact load-time binding to the compiled policy. This changes presentation
metadata, not selection or the packed per-choice payload.

The pure bridge read exposes the recorded selected facts, exact predicate
operators/thresholds and checked integer score contributions. Reasons rank by
positive contribution, then positive conditions, then other conditions, with
stable authored ordering. GDScript consumes that order and localized condition
wording; it does not prioritize hardcoded fact names or invent units. All facts
remain available, including low/absent conditions. The top-three list retains
actual scored candidates, including the distinction from a hysteresis-retained
selected node.

Validation: `core-data/mind-dev/bridge-tests-5.log` has three passing tests for
read purity, a positive job condition versus its low-hunger guard, and signed
score contributions. `docscheck-reasons.log` verifies the expanded snapshot
schema. Locale generation reports 1,135 strings per language without duplicate
keys. Native rendering and visual acceptance remain pending.

## D14 — Bind hidden mind state in determinism evidence

Trajectory fingerprints now stream the complete serialized `MindState` and the
next causal event ID into a domain-separated hash. This covers running choices,
history and its dictionary, policy source, and all provenance maps; visible
positions alone cannot establish that future decisions and reasons will agree.
Serialization failure is fatal rather than accepting a partial fingerprint.

An entirely empty mind appends no bytes, preserving the exact legacy fingerprint
contract before the switch and in baseline worlds. The implementation streams
RON bytes directly without allocating a full-state debug string. Six focused
tests in `core-data/mind-dev/fingerprint-tests-1.log` verify planted hidden-state
differences, stable map ordering, unchanged empty-state hashes and real generated
mind-world twins. Full post-switch determinism remains a separate required check.


## D15 — Distinguish urgent triggers from ordinary social completion

Calibration round 1 (only training seeds, no held-out inspection) showed band-only
natural-commit and gathering-exit differences. Retain these as an explicit band
semantics replacement: a critical band describes the triggering context, not a
claim that the selected need itself crossed the critical threshold. Natural
Hunger > Thirst > Sleep ordering and existing memory candidate arithmetic stay
unchanged. Thus hunger 56.9 with thirst 77.1 can select eating in a critical
context. The old observer maps natural commits to Need without inspecting their
urgency; that mapped band is not an original engine band.

Noncamp idle critical pools use authored hunger/thirst tenths >=750. Providers
truncate raw value times ten; they do not round ordinary values up. This is the
fixed 75-point policy classification, including when a custom rule config uses
a different interruption limit. Camp water uses exact raw thirst strictly above
the configured critical-preemption limit (default75). Gathering completion uses
exact raw hunger or thirst at or above that limit with gathering preemption
enabled. Its Interrupted finish preserves the one-tick Idle boundary, while the
old observer retains the dispatched Work band even on gather_critical_preempt.
These critical contexts can produce a critical-entry Chronicle record; this is
an intentional recording difference, not merely a display normalization.

Keep the raw old/new band disagreements visible and separately explained. These
cases exclude zero person-ticks from the behavior port denominator; neither a
band difference nor this decision excuses any behavior or completion mismatch.
Do not demote the job critical pools: their band must interrupt Duty holds.
Do not demote camp-water candidates into competing ordinary hungry candidates.

In contrast, ordinary social interruption only requires an actionable need and
has no critical trigger. Its three finish_social_need_* nodes belong in Need,
with their existing high score and Interrupted finish. This prevents an ordinary
social exit from manufacturing a critical-entry Chronicle event. Flee/draft
ownership and the actual job-owned 75-point seam remain unchanged. A focused
regression was added first; the parent confirmed its critical-versus-Need RED
before the three band edits. Full GREEN verification remains parent-owned.

## D16 — Preserve failed calibration and repair routine boundaries

Round 1 is retained as FAIL: behaviour agreement was 1,861,596/2,038,246
(91.333235%) OFF and 1,595,848/1,691,103 (94.367286%) ON. Both modes passed
the existing daily-fingerprint identity and complete observation checks. Raw
reports and the independently summed audit remain in `core-data/shadow-round-1`.
No held-out seeds were inspected and no production registration changed.

The redundant `rest_between_activities` node mislabeled legacy idle wandering
during a routine cooldown. Remove it; the ordinary wander node and existing
cooldown remain, while targeted hearth rest remains registered. A second bug
evaluated the shadow before a pending invitation's Idle handoff. Preview that
handoff in the shadow snapshot without consuming the invitation or writing the
world. Both focused regressions failed first and passed after these fixes.

A failed routine entry must retain the legacy 30-tick cooldown and NoPlace
observation, or the interrupted one-tick retry without a NoPlace event. Add
descriptive admission/weighted-place facts and an explicit targetless
`routine_no_place` node/effect. The failed diagnostic kind uses the required
person/tick/node key instead of the old lineage key. This narrowly extends the
declared routine lottery provenance replacement to the failed attempt's event;
it neither selects an activity nor changes the cooldown. Keep the raw old
`no_choice` label and the entire port denominator, including these mismatches.

Verification history remains visible: the initial no-place regression
accidentally loaded the two-node hearth test policy, so that RED result is not
evidence of the shipped-policy defect. The corrected regression loads the actual
shipped policy and compares cooldown/observation/Idle with the real legacy path.
Separate tests cover nonzero diagnostic weights and interrupted retries. An
exact registry-set assertion now distinguishes the seven activities from the
targetless denial behavior. Round 2 and the full pre-switch identity checks are
still required before acceptance.

## D17 — Keep job presence separate from current job ownership

Round 2 reached 99.913210% OFF and 99.804743% ON behaviour agreement with
unchanged fingerprints and full coverage. Review nevertheless found twelve ON
port defects: nine foraging choices and three drinking choices fell back to
wandering for Idle people retaining a Job component. Preserve all twelve in
the raw reports and reviewed counts; passing the numerical bar is not grounds
to relabel a concrete port defect as an intended replacement.

Thirty-eight idle need, camp and survival-work nodes now test absence of current
job ownership instead of absence of a Job component. Routine admission still
requires no Job, as the legacy path does. A consuming construction-site worker
with a Job is job-owned regardless of site occupancy. Occupancy still matters
in the separate post-choice restoration rule; do not conflate those decisions.

Job-release bookkeeping now snapshots only actual held job states. Finishing a
previous do_job choice after an external transition to Idle must not release a
retained Job that the legacy seam did not hold. Four regressions first failed
on the actual eligibility, ownership and release differences. Parent-confirmed
GREEN evidence is now recorded under `/Users/rexxa/work/core-data/mind-dev/`:
`systems-tests-11.log` (309 passed), `core-tests-4.log` (22 selected core tests
passed), `candidate-worldgen-tests-1.log` (5 passed),
`candidate-registration-tests-1.log` (3 passed), and `clippy-9.log` (completed
successfully). Round 3's independent audit confirms the twelve defects are
absent, zero unexplained categories, and shadow-stage acceptance; see
[CORE_SHADOW](CORE_SHADOW.md#round-3--shadow-bar-pass-accepted-for-the-shadow-stage).
The round-3 full-runtime calibration candidate subsequently completed all
48 days on seeds 0–7: all six outcome bars passed 8/8, cold-snap survival passed
5/5 exposed worlds (three unexposed), determinism passed, and reason coverage
passed 135,368/135,368 behaviour transitions and 2,984/2,984 critical choice
transitions with zero failures. Evidence is
`/Users/rexxa/work/core-data/shadow-round-3/candidate/outcomes.json` and the
round-3 independent audit; the policy freeze receipt is
`/Users/rexxa/work/core-data/policy-freeze-round-3.json` (SHA-256
`edd0c751c67c053032743864f9355a35d92a013a3fba8502cb3f302aae0e4b09`).
These are calibration results, not held-out or full-goal acceptance, and do
not verify the subsequent D18 correction. The superseded safety-2 run remains
INCOMPLETE, never identity proof.

An explicit `mind_candidate` build feature permits full training-world execution
before changing default production registration. It uses the same full runtime
registrar and initialized mind that the switch will use; baseline and the
explicit shadow factory remain old. Only seeds 0–7 may run for calibration.
Keep its Cargo declaration as a historical build marker after the switch, with
no alternative production routing, so archived build manifests remain bound.


## D18 — Replace only the mind gather quarrel random key

Status: actual rare-branch regression RED followed by verified GREEN after
the minimal correction. Final round-4 shadow and calibration checks PASS.
This is training evidence, not production-switch, held-out or timing acceptance.

The mind gather effect inherited the legacy cross-settlement quarrel
draw's lineage/pair-derived subject. The correction replaces only that draw's provenance with
the mandated current AgentId, tick and selected node key: use the node key in
the `STREAM_RELATIONS` random domain and the acting AgentId as subject. Keep
the world seed, replicate, draw slot, bound and probability calculation. The
legacy baseline draw remains unchanged.

This is a narrow RNG replacement within ported gather effects, not a replacement
of gather eligibility or dispute handling. Preserve rival/pair ordering,
dispute insertion, daily caps, relation losses, quarrel/chronicle accounting,
Idle transitions, blocked pickups and source conservation. The inventory's
observed colony rival/dispute/quarrel subarm counts remain zero; zero exposure
does not authorize deletion or establish equivalence. Retain all port shadow
denominators without exclusions for this change.

The real-effect regression
`cross_settlement_forage_quarrel_uses_selected_node_and_agent_identity` in
`runtime/mind/behaviours/gather_tests.rs` constructs two cross-settlement
foragers and exercises the actual consume/dispute path. Parent-confirmed RED
exited 101 at `gather_tests.rs:49` for seed 0 / node 7 (actual false,
expected true). Its checks cover repeated-input determinism, node-sensitive draws and
both outcomes, plus the retained effects above. Parent-verified GREEN is
`/Users/rexxa/work/core-data/mind-dev/systems-tests-12.log`: 310 tests passed,
including this regression. `fmt-10.log` exited 0, and `clippy-10.log` completed
all-workspace/all-target checks with the three requested features, exit 0.
This constructed mechanics test is not held-out gameplay.

Pre-switch identity run 3 completed with byte-equal baseline, nine colony-OFF
fixtures and colony-ON evidence (`preswitch-safety-3-compare` and
`preswitch-steward-3` logs). That identity evidence alone does not verify the
mind-only correction. Round 4 now passes shadow agreement at 99.913210% OFF
and 99.805452% ON, with zero unexplained errors and residual port defects.
All mismatches remain in the reports and port denominators. No observed colony
change is attributed to this narrowly corrected rare draw.

The corrected-source 48-day calibration candidate passes all six outcome bars
8/8, cold-snap survival 5/5 exposed worlds (required 4), and why coverage
135,368/135,368 and 2,984/2,984. The round-4 probe summary JSON is
byte-identical to its round-3 counterpart; summary SHA-256 is
`70cccdc7e6a00703cba8ea04ef9210e6d13de6e1d94ec5c95c10bea21dc7e68b`.
The independent audit and candidate evidence remain under
`/Users/rexxa/work/core-data/shadow-round-4/`.

The policy remains `7a6c90ac89db9cfb08acac6a8e14c91d1a1690d9fa40c3798c52323ead43eae8`.
The new policy-freeze receipt `/Users/rexxa/work/core-data/policy-freeze-round-4.json`
has SHA-256 `8306d91c24ee200f2fb308c332ecbbb93f6a90c14c18b5ca2a9e89b22a97675d`,
frozen for held-outs at `2026-10-26T16:35:29Z`, and binds source freeze
`ad4bf65d0bf1747f8e2572c9d630df48b36c2a14c7df1d80017aa243a1f80002`.
The four-round calibration allowance is exhausted; no further policy tuning
or held-out-informed changes are authorized. Identity run 4 subsequently completed:
`preswitch-safety-4-compare.log` reports zero baseline differences, 64 chronicle
files equal by content (19,683 entries), equal raw sample digests and all nine
colony-OFF summaries byte-identical to BASE. `preswitch-steward-4.log` reports
DETERMINISM PASS, and its summary is byte-identical to `base-steward/probe/summary.json`.
The complete parent capture/compare/ON chain exited 0 before the pre-switch
implementation commit. No timing claim follows from these training or identity
results. Preserve rounds 1–3 and their source-bound results.
