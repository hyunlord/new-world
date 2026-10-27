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


## D19 — Switch production colony registration while retaining baseline routing

Status: switch implemented in the working tree; focused tests and post-switch
captures are complete. Switch commit remains pending; the full gate is not
claimed here. The
round-4 frozen policy remains unchanged at SHA-256
`7a6c90ac89db9cfb08acac6a8e14c91d1a1690d9fa40c3798c52323ead43eae8`.

Production registration now assigns `AgentDecisionSystem` only to baseline
and `ColonyMindSystem` only to colony, at the existing decision priority and
cadence. Other system ordering remains intact. Fresh world generation routes
through the requested world rules and initializes colony policy/history before
the first tick or player command. `mind_candidate` no longer selects a different
production route; its declaration remains a historical build marker. Explicit
shadow factories retain the old selector for comparison.

Save loading chooses registration from decoded rules and presents saved
resources to registration, preserving saved mind policy/history or the
initializer for a legacy colony. A colony save without mind policy may migrate
exactly one empty `AgentDecisionSystem` slot to an empty `ColonyMindSystem`
slot in the same position. System count, all other names/order and state
payload checks remain strict; mismatched or nonempty replacement slots fail.
Existing baseline saves keep the legacy registrar. This is a bounded lifecycle
slot migration, not permission to accept arbitrary system-list mismatches.

Parent-confirmed focused checks passed: switch 10, save 41, registration 4,
world generation 6 and shadow 3 tests. These checks do not substitute for
post-switch baseline identity, old/new colony digests, frozen held-outs,
native UI, story yield, admitted paired timing or the full gate. Final baseline
isolation is still pending: retain a separately reviewable pure move into
`runtime/baseline/decision`, then remove proven-dead colony gates/seam and
verify the import boundary without changing baseline behavior.


### Post-switch fixture maintenance and bounded freeze amendment

Retain the initial failures under `/Users/rexxa/work/core-data/switch-dev/`:
`systems.log` has five failures and `integration-1.log` has two. Custom-policy
fixtures now change exactly two test imports to the `cfg(test)` unscheduled
world factory; the strict production policy guard is unchanged. The repaired
systems run passes all 310 tests in `systems-2.log`.

The colony-command fixture expected natural V7 construction already classified
as dropped in the inventory. Its replacement tests the explicit legacy seeker
boundary at tick 132 while retaining real job construction, cleanup and
survival coverage. All eleven command tests pass in `commands-2.log`. Further
parent-confirmed checks pass: seven full-save tests including pregnancy/birth
and baseline, nine jobs tests, twelve flee tests and three `engine_mind_ffi`
tests; the second formatting and Clippy runs also pass. This is fixture
maintenance, not policy tuning or a newly claimed gameplay fix.

Freeze 4 remains intact. The additive pinned test-only amendment
`/Users/rexxa/work/core-data/shadow-test-migration.json` has SHA-256
`b15b4b7b1b0b2bbee788a604131b728fbe6f3c7e924c871d110e4ea2745f3e50`.
It permits exactly the two import changes; the original manifest, checker and
binaries are unchanged, and all other included files match exactly. Seven
controls pass. The live wrapper passes in `switch-dev/test-migration-live.log`:
it projects the two original test byte sequences into a temporary directory
and invokes the unchanged checker, without mutating the repository. This is
not a claim that the entire current source closure is byte-identical to the
original freeze. The world-generation helper remains bound by its strict full
hash; retiring the shadow factory will require a fresh explicit amendment.

At the fixture-maintenance checkpoint captures were still running; the completed
results are recorded below. No held-out worlds have been accessed and policy
bytes remain frozen.


### Completed switch capture and bounded review

The full capture chain exited 0. Independent `switch-dev/fixture-audit.json`
(SHA-256 `b6bcc6231c7640c6548eb03eefa950188da1023d1dba6d4eee44dbb8d65e77e6`)
audits all nine OFF fixtures plus two-seed ×24-day ON: **58 worlds,
3,271,680 ticks**, exact seed/day coverage, zero ledger/reservation violation
ticks and null first violation. Behaviour coverage is **468,387/468,387** and
strict critical-choice coverage **22,080/22,080**. Soak and ON determinism pass.
Baseline compares 228,748 values with zero differences, 64 chronicle/relations
files and 19,683 entries equal, and equal raw digests. The baseline comparison
log SHA-256 is `8f5601d2424a458af813d5c42c1ac819b295ec1d7c1bff64b23b7772b7c2beae`.

The actual probe binary is
`629ea00c191e8df2e5f2a826fcf3e1a30615071be245311566316629e4f442dc`, built from
p5 `0ef68a1d2b6eb946e150963071b2e623fa58d4b0` plus the captured source patch
`9411d508336f0fd6c125463d0431ffe5734f23de1780b0b06259cdfbd956dfc6`.
It is not described as a binary compiled at a future p6 commit. The audit
checks the captured Rust diff against current source. Bounded switch review
approves this scope, and 55 focused Python checks pass; neither is a full-gate
or publication claim.

| Fixture | BASE summary SHA-256 | Switch summary SHA-256 |
|---|---|---|
| soak | `c07dab3200fc3be1cfa87dd5abee629964ba8c17ac62469ae86c29bd73d78b69` | `203d098630d061261c76e83f6b1d3951d662884846758004ccd0c52bc4ca2b5f` |
| story | `d6952313fe88f1caab37b18c85d42a352689a56467d5af8109927c21dc227feb` | `94f177b3ef924a1ab59357ded06a5e0a7fed22ac02b5646eaabed6040e149d28` |
| neighbours | `eab12f6964891d9b2675b538ec1e3d6141ad4d3637eb4a9198e33d773e6f677c` | `0281170659f77c905ff7b9a10b63550587c9881e688521d70e45020f66493267` |
| knowledge_f10 | `f39c21ae1d9bf6a819714be84510ad1f6d9d12664236f37d0f1942efec4b6d46` | `fc48d580d159a03d83d00125d2f2d2d9e41bbbb606930c071a185c9f280ea985` |
| m55_pressure | `74ec901346b50a7ba6dad2a34993101c16b57fbaeaf3153deb321044a7d2cd96` | `4a0cda9ea85041210834e8124e04005b2cad3b5c1cd5e636ae897b638f674a36` |
| m55_nodraft | `32ab17b017a6311fea0556071c01d8d0ed594ae2002aaf0dcf70895e9051d5ee` | `1f319da7c4b76ad59861c810862667e02e0afa0fbdee6d87345f917b83b9adb2` |
| m55_storyoff | `9065d5679657056c0f407564a381843fa5a6cdc4204473c10fbd9b81d33bb97c` | `139682522580c142f1d5f715efdf33b5a9ed724ee4d099c1642ab257c7cb4f93` |
| m55_noneighbours | `fef5b45dd8eae35533c5d6738f1c430ef525fecc13b41816ead26449823deecb` | `bc651e11dda612aa54f3e349ccc56c26e35980ba36adbedbdd183f28710b3f41` |
| m55_quiet | `b62668f754ed0b1e1d251b9b81a63ecbfde64e8e3040bc03587de8e4814978fa` | `a08bbb39f93a889c4a1cbece1879220cddddf2809b056eaa44910e8fdffacb33` |
| steward_on | `7de55ce7ff34aebc978769de8fc63037ecda67c0929b1b783ea6939799087196` | `cad9811728445bba5d8aa9027d6450ceb9aec3b40635258572577b711c387958` |

These changed summaries contain real gameplay drift as well as new mind
metadata. Work/craft throughput, stocks, knowledge, demographic and social
trajectories differ; fingerprints alone would not prove those differences.
The authorized switch replaces the legacy cascade with authored band/node
ordering and node-keyed lotteries, which can change downstream trajectories.
This explains a mechanism for divergence, not proven causal attribution for
every changed leaf. Identical command scripts also need not yield identical
accepted command totals. No policy tuning follows from this audit.

Retain all 21 selected outcome-warning rows below (BASE → switch); improvements
and regressions are both evidence. In particular four no-draft seeds have
increased combat deaths; noneighbours seed 6 has two exposure deaths; five
fixtures lose seed-6 U3; craft-loss changes are not hidden.

| Fixture / seed | Changed outcome |
|---|---|
| story / 1 | crafts_lost: 2 → 0 |
| knowledge_f10 / 7 | crafts_lost: 0 → 1 |
| m55_pressure / 2 | crafts_lost: 0 → 1 |
| m55_pressure / 3 | crafts_lost: 1 → 0 |
| m55_pressure / 4 | crafts_lost: 1 → 0 |
| m55_pressure / 5 | crafts_lost: 2 → 5 |
| m55_pressure / 6 | u3_no_loose_logs_day5: true → false |
| m55_pressure / 7 | crafts_lost: 0 → 1 |
| m55_nodraft / 0 | crafts_lost: 1 → 3 |
| m55_nodraft / 1 | combat deaths 0 → 2 |
| m55_nodraft / 4 | combat deaths 0 → 2 |
| m55_nodraft / 5 | crafts_lost: 3 → 0 |
| m55_nodraft / 6 | combat deaths 0 → 2; u3_no_loose_logs_day5: true → false |
| m55_nodraft / 7 | combat deaths 1 → 2 |
| m55_storyoff / 6 | u3_no_loose_logs_day5: true → false |
| m55_noneighbours / 3 | crafts_lost: 0 → 2 |
| m55_noneighbours / 5 | crafts_lost: 2 → 3 |
| m55_noneighbours / 6 | crafts_lost: 0 → 1; exposure deaths 0 → 2; u3_no_loose_logs_day5: true → false |
| m55_noneighbours / 7 | crafts_lost: 2 → 1 |
| m55_quiet / 6 | u3_no_loose_logs_day5: true → false |
| steward_on / 0 | crafts_lost: 2 → 1 |

The complete audit retains per-seed values and structural differences; leaf
counts are not independent events. The earlier partial audit remains archived.
These OFF/short-ON captures establish the stated safety and coverage checks,
not OFF performance PASS or full outcome acceptance. The full guarded ON
held-out comparison has not yet been measured. Baseline isolation, paired
performance, native UI, story comparison, full gate and publication remain
separate obligations; no whole-current-closure byte identity is claimed.


## D20 — Isolate supported baseline execution without a new crate

P6 `fbc9a9babc9bb5df1d06ca1e655901e480521f66` commits the approved production
switch with frozen policy unchanged. P7 isolation is committed as `be12050c951dfb025f3440d34b657eec0663b817`.
Systems/feature/formatting/strict-Clippy and source/registration isolation
checks pass; post-isolation identity capture now passes as recorded below.

Keep the work distinguishable: A moves `decision/{system,mod}.rs` into
`runtime/baseline/decision`; the exact two-file move is verified in
`/Users/rexxa/work/core-data/baseline-dev/pure-move-proof.txt` (system SHA-256
`2e65dfef4083b1de4f7cbf8d518ff1e2e4eafcae08f96ff7c1fe875af6f8605a`,
module `867c44e08f622dd228566bf7ba1e58fd706bc75d1df9bc270d7aa460e11d8d15`).
B separately removes dead colony gates and the job seam. C retires live old
colony shadow execution; baseline read-only feature observer hooks remain,
without a live old-colony factory enabling them. The import boundary forbids
colony/shared code importing baseline, not baseline importing a neutral observer.
Frozen old-runner source/binaries remain externally bound
by freeze-4/p5 manifests. Shared neutral `agent_actions` stays in place.
These source changes do not themselves prove final baseline identity.

D preserves the original bodies of six fuel regressions while exercising the
real mind; all six pass. The move-test evidence includes agent-decision tests
and refusal of the retired CLI path. Preservation applies to supported baseline
constructors and the saved baseline corpus; it is not a guarantee for arbitrary
injected Job/draft/flee components that baseline constructors never produce.

Drop the optional standalone `sim-baseline` crate for this milestone. Actual
imports require crate-private `agent_actions` and settlement food/firewood plus
chronicle helpers; `sim-systems` owns registration. A selector crate depending
back on `sim-systems` while its registrar depends on that selector would be
circular. Avoiding it needs another shared-helper/registrar extraction and
visibility expansion; module separation plus the import gate is the narrower
boundary. No new crate or measured speed benefit is claimed. Final isolation,
identity, held-out/performance/UI/story and full-gate evidence remain required.


### Verified p7 checks and external BASE lock provenance

Persisted `/Users/rexxa/work/core-data/baseline-dev/` evidence now confirms:
`systems-tests.log` 309 passed; `move-tests.log` and `feature-tests.log` each
19 baseline decision +2 retired-CLI +6 fuel/sleep passed; `perf-arms-tests.log`
4 passed; `isolation-check.log` source PASS and registration 4 passed, also
recorded in `isolation-registration.log`. Formatting and strict Clippy pass
(`fmt.log`, `clippy-2.log`). `independent-review.md` approves the bounded staged
implementation without blocking findings, conditional on final identity/capture
verification. Its supported-constructor/save qualification remains in force;
no arbitrary Job-target state guarantee is added. Identity capture subsequently completed successfully (receipt below); p7
is committed as `be12050c951dfb025f3440d34b657eec0663b817`.

The final performance preparation also corrects only external BASE lock
provenance validation. The immutable `base-bin/Cargo.lock` SHA-256 is
`ddc99ac9fc161d9b610c0f5d8dbf5e279c8908c10717647ba50ab2fef70c36d1`, bound to
the preserved external manifest and an independent manifest pin. Retain the
original BASE ancestor/equal-Rust-tree rule: build head
`001f236c308a88615a650fca9af83d80041aa7a8` descends from nominated BASE
`9855bd082ad88a4c49296410f206f148d98c4df9`, with preserved empty source diff.
No BASE checkout/rebuild or relaxed source identity is substituted.

`/Users/rexxa/work/core-data/final-perf-plan.md` records the initial missing
validator RED and the first GREEN attempt's macOS `/var` versus `/private`
path-alias failure. Canonical absolute-path comparison resolves that boundary.
All 15 validator fixtures pass (`final-perf-lock-fixtures.log`), including
changed/absent lock, absent/empty manifest and rewritten lock+manifest against
an unchanged independent pin; syntax compilation and diff checks also pass.
These are evidence-validator tests, not a performance freeze/run or speedup
claim. The successful build receipt must still establish compilation provenance
for the actual final binary before timing.


### Completed post-isolation identity capture

`/Users/rexxa/work/core-data/isolation-capture-1.log` terminates with
`ISOLATION_CAPTURE COMPLETE baseline and 9OFF and ON equal to switch`; the
parent capture session exits 0. `isolation-safety-1-compare.log` SHA-256 is
`39b246855ca52c88359390b8b1b561aae4447e8867c72f18e6981834e7e92155`:
228,748 shared baseline values, zero differences, 64 chronicle files and
19,683 entries equal excluding only `git_sha`, and all nine OFF summaries
identical to the switch capture. Exact fixture coverage and zero invariant
violations pass. `isolation-steward-1.log` reports DETERMINISM PASS; its
`summary.json` equals the switch ON summary with SHA-256
`cad9811728445bba5d8aa9027d6450ceb9aec3b40635258572577b711c387958`.
The captured Rust tree is `c3a0e98c30e1758bf785f9a6936056d87b58b370`.
This completes the stated supported-corpus post-isolation identity check;
p7 is committed as `be12050c951dfb025f3440d34b657eec0663b817`. It does not extend the supported-world contract,
resolve ISS336, or claim held-out/performance/native/full-gate/publication
acceptance.


## D21 — Defer complete malformed-save chronology validation explicitly

The final bounded source review found a P2 fail-fast gap for inconsistent
persisted mind state. Retained/tile choice IDs or ticks and the private
`MindHistory::last_event_id` can exceed `next_event_id`/`current_tick` without
being rejected during load, then cause a later normal choice insertion to fail.
No reachable valid-production save path violating this ordering was found:
ordinary recording allocates its ID and uses the current simulation tick.
The synthetic `engine_with_history` fixture demonstrates the acceptance gap in
source, but this review did not execute a malformed-save reproduction. Do not
report this as a demonstrated ordinary gameplay/save failure.

Complete validation requires an explicit read-only history chronology API in
the frozen sim-core closure. Defer that work rather than silently altering the
freeze or bypassing private state through serialization layout or synthetic
insertion. Retained-record bounds alone do not cover an empty history's private
high-water mark and must not be called complete chronology validation. A future
repair must reject future/unallocated chronology, preserve valid references to
evicted causal events, correct fixture allocation/clock handling, and test both
malformed-save rejection and valid next-choice continuation.

Track this as local **deferred** ISS336 and remote **OPEN**
[#642](https://github.com/hyunlord/new-world/issues/642), created and read-back
verified by the parent. Evidence: `/Users/rexxa/work/core-data/final-code-review.md`
and `issue336-created.json`. No runtime/policy change, repair claim, issue
closure or final-goal approval follows from this disposition.


## D22 — Retain native UI failures and separate completed evaluation evidence

Native capture `mind-ui/native-final-02` exposes two actual UI defects requiring
repair: the inspector Why button keeps its previous locale after switching to
English, and MindChoice kind 47 is skipped by the relevant causal UI handling.
The locale defect is visible in `mind-ui/visual-review-02.md` (REVISE). Fixes
are in progress; no corrected native PASS is claimed. The completed capture
records same-agent hunger and threat transitions, but an automated empty
failure list did not detect the stale button language. Retain the failing
captures and add focused assertions/recapture for the repaired paths. Multirow
candidate layout and already-open-popup locale refresh need their own evidence;
readable observed samples do not prove every UI state.

Separately, final features[] build at p7 passed: `final-bin-1/build-receipt.json`
binds unchanged pre/post Rust hash, policy, lock and binaries. The completed
`heldout-final-1/session-terminal.json` reports CAPTURE_AND_CHECKS_PASS and all
seven subprocess exits 0 (UTC 2026-10-26T18:18:37.050714). Authoritative
outcomes/why equal their prechecks byte-for-byte. All outcome bars pass: new
mind32/32 except wolf31/32 (required29), with cold-snap firewood19/19 exposed
versus BASE17/17; unexposed13 versus15 remains explicit. Seed1238 wolf deaths
worsen BASE0→mind3 and its individual ≤1 criterion fails; do not hide it behind
the aggregate PASS. Why coverage is469721/469721 behaviour,12159/12159 strict
critical and805730/805730 total expected records, zero failures. No policy tuning.

Story is MEASURED reference evidence with no numerical bar. Each side has64
complete world-years: BASE entries7676 (119.9375/year), mind19722
(308.15625/year); people with ≥3 yearly entries556 (8.6875/year) versus651
(10.171875/year). Both deep entries and distinct deep chains are0. Missing
ancestry121 versus125 is retained, cycles0. The BASE story instrument is p2
runtime plus frozen observational tooling, not an exact BASE-source binary.
Zero deep chains does not demonstrate deep-story success, and more entries
does not by itself establish greater story quality.

Evidence root is `/Users/rexxa/work/core-data/`. Authoritative SHA-256:
`heldout-final-1/outcomes.json` =
`42e9de5b1649ced049ca37a9741f15f9b900baaa61ce128eb7f84d8d5781ebed`;
`why.json` = `552ccf8b9932e23d1243b185c6fb4a35759df21a10d364f6e633430ce618a6a8`;
`story.json` = `7fb7a6ceb55ac1d08cd5667e679e7e3064e19c8aa21761e8c8bede594970d9ce`.
These completed captures do not grant native UI, final performance, whole gate
or publication acceptance. The UI repairs do not authorize changes to frozen
simulation policy or measurement tools.


D22 tracking update: ISS337 is registered as
[#643](https://github.com/hyunlord/new-world/issues/643), and ISS338 as
[#644](https://github.com/hyunlord/new-world/issues/644); both remain local and
remote OPEN pending QA/publication. Parent is performing remote read-back.
The Why-label source repair is applied, but native03 locale recapture is still
pending. The kind47 formatter repair is being applied; preserve its actual
Godot RED `mind-ui/chronicle-red.log` (exit1, CORE_CHRONICLE_DONE failures=10).
The formatter previously stopped at LAST_KIND46 and skipped the critical-choice
chronicle, including the person's story sentence. Repair must preserve person
and causal links and localize the recorded urgent choice without inferring a
policy node from its numeric slot. No GREEN/native03 PASS is claimed here.
Canonical issue bodies are `mind-ui/issues-body-337.md` and
`mind-ui/issues-body-338.md` under the external evidence root.


D22 final native03 result: `mind-ui/visual-review-03.md` and
`integrity-review-03.md` both PASS within their stated scope, covering14 actual
PNGs. Godot/wrapper/provenance exits0, inputs-unchanged=1; before/after hashes
match. Hunger Agent8 changes do_job/event82/tick1 → eat/event90/tick154;
threat Agent4 changes do_job/event4129/tick18787 → flee/event4187/tick18906,
with fleeing0→1 and cause4186. The hunger record proves a behaviour change,
not an observed hunger_actionable0→1 edge absent from its prior record.
Locale-correct Why buttons and actual kind47 critical feed now render in EN/KO.

Retain chronicle RED10 failures, the initial GREEN's one joiner-normalization
fixture failure, and final GREEN12 checks/0 failures. Strict checks of five UI
files and60 FFI checks pass. ISS337/338 are now locally fixed; GitHub643/644
remain OPEN until verified publication. This capture is p7 plus the UI patch,
not a claim of a p8-built binary. Each captured popup has one eligible candidate;
no real three-row ordering/layout proof exists. The summary is a single field
that wraps2–3 lines, not a literal one-rendered-line fit. The separate chronicle
Why link was visible but not clicked. Exact copied originals are
`docs/images/core_mind_hunger_en.png` and `core_mind_threat_ko.png`.
Native acceptance covers this demonstrated presentation/provenance only;
performance, full gate and publication remain pending.

## D23 — Preserve valid final performance failure and block publication

Status: measurement complete, **FAIL**; no waiver. The complete 45-step gate is still pending. [ISS339/#645](https://github.com/hyunlord/new-world/issues/645) is registered/readback-verified and OPEN.

The final interleaved session accepted 33 pairs/66 members (18 timed pairs, 15 build-only RSS pairs), invalid0. Admission one-minute loads span4.1201171875–7.06201171875, all<8; all1/5/15-minute samples and alternating order remain in the raw evidence. Receipt `/Users/rexxa/work/core-data/final-perf-session-1/receipt.json` SHA-256 `966c6cd06e7fd7c78da4d9bdb6090fb5113e69b80b084163ef3c3452a4e81cf3` belongs to p7 `be12050c951dfb025f3440d34b657eec0663b817` and source SHA-256 `9a6cdde2052d909b816dcd0e5a0a5674dfdce75eed873b5ab6412b6bfe55fb21`. Independent check exit1 and byte comparison exit0 reproduce the same failure. [CORE_PERF](CORE_PERF.md) retains the full comparison and standing tables.

N1000 three-repeat mean tick rises2082.978819→19927.196181µs (ratio9.566682097070183); mind/old decision is13.150984160978739. N2000 mean is38421.873495µs. The receipt has16 failures: twelve standing1.1 mean/p99 regressions, mandatory C3 N1000≤5ms/N2000≤10ms misses, and two report-only CORE ratios (mind≤0.5; tick≤0.700498508983101 from accepted s=0.6990029820337981). Report-only treatment of the latter does not waive standing C3. Publication is **BLOCKED**; do not label the not-yet-run full gate failed.

All four standing after-build allocator/RSS ceilings PASS; allocation per agent ratio1.0. Synthetic full-ring capacity is9060.066666666668B/agent (MISS4096),80.076416015625B/tile (PASS96), report-only. Sixteen copies of each person's largest retained complete record, fresh IDs/final-tick headers and no further ticks model capacity, not natural occupancy. Keep this separate from after-build C3.

The frozen policy, exhausted four calibration rounds and completed held-out results are unchanged. `final-perf-source-review.md` proposes eager provider/preview work, global Person-version cache invalidation and repeated policy validation/allocation as source hypotheses, not measured internal attribution. No production tuning or acceptance relaxation was authorized by this result. Any later optimization requires a separately authorized verification boundary and fresh evidence, preserving this failure and the earlier functional successes.

## D24 — Stop after the completed p8 gate fails

Final status: **FAIL / not published**. D23's pending full gate was its then-current state; it is now complete at p8 `44e0a0266c085d409c16c77f4a2a0181b8a2a750`, clean start (`dirty0`),42/45 steps PASS, status1. Exact gate time2026-10-27 04:02:02–05:58:54 KST,7012seconds. This is gate duration, not an inferred exact goal duration. `core-data/full-gate-p8-1/gate.txt` SHA-256 `f437bd3e64a96af0382f5806d8c314fbe8ad70e60bc50fc8c1fc8af76237a394`.

| Failed step | Actual result |
|---|---|
|25 bench|Standing six-scene mean/p99 regressions; N1000 mean19045.5µs>N1000 5ms and N2000 mean36432.2µs>10ms; after-build memory ceilings PASS. Separate gate measurement, not replacement of D23's paired evidence.|
|41 mux2_stall_receipt|Both runtime arms complete, but1× stall bar fails; wrapper receipt rejects capture exit1.|
|45 core_evaluation|Rejects pinned final performance receipt for standing/C3 failure. Prior held-out outcome/why PASS and reference-only story remain recorded.|

Source isolation42, real data-only target/arrival/record test43 (one test), and seed0 shadow44 each exit0. Other42 passing steps are retained in gate.txt; they do not override the three failures.

Stall raw report: `core-data/full-gate-p8-1/mux2-stall/capture/run/stall-report.json`, SHA-256 `419bc23b95e104380b9447e086057e30a2127bca9317dde3ec74581773f72883`. At1×, post-warmup300.000826seconds/35755frames, max91.652ms, >33ms2 and >50ms2, FAIL. At½×,300.006963seconds/34902frames, max49.522ms, >33ms2 and >50ms0, PASS. Startup maxima3775.521/4378.179ms remain visible. Top-level complete=true/bar_pass=false and both arm capture_complete=true establish finished capture; wrapper receipt complete=false/exit1 records failure to accept, not runtime incompleteness. diagnostic_complete=false prevents claiming complete internal attribution.

The goal requires stopping and reporting after this failure. No tuning, waiver, retry or push follows; preserve the worktree and raw evidence. All locally fixed issues remain remotely OPEN because publication has not occurred. No later documentation HEAD inherits a newly executed passing gate. Parent verified both remote refs remain BASE `9855bd082ad88a4c49296410f206f148d98c4df9`; no remote update occurred. Stall is tracked as [ISS340/#646](https://github.com/hyunlord/new-world/issues/646), OPEN.

D24 bounded audit: all45 step IDs are unique. Fresh step44 seed0 matches are OFF251664/251676 and ON210955/211127,23 categories each, PASS. Step45 regenerated outputs are byte-identical to original receipts; fourteen standing failures block acceptance, while the exception includes all sixteen strings including the two nonblocking CORE bars. Stall1× >50ms events occur at34.466878s (50.194ms) and218.325537s (91.652ms). The latter advances zero simulation ticks and has85.418ms pre/post draw duration; no claim attributes this stall to mind. Raw profile completeness is true and errors[]; diagnostic_complete=false is a distinct field, not missing capture. Both runtime measurements cover the full300seconds.

## D25 — Planner-authorized performance repair (2026-10-27, P-1 through P-6)

Authority: the user's **“Planner decisions for M-CORE (2026-10-27), after reading CORE_REPORT/CORE_PERF/D24 on v8/core-review (61260ec8c)”** note in this thread. This entry supersedes D24's stop interpretation without erasing the failed gate. Work resumes on `v8/core` in `/Users/rexxa/work/new-world-core-dev` at `61260ec8c245d88c6d12ef015de815dae6849017`. Repair budget starts 2026-10-27 08:43 KST: target10h (18:43), hard stop14h (22:43).

### P-1 — Red gate repair authority

“On a conflict or a red gate, stop and report” applies only to rebase after refused push. Otherwise diagnose, fix and rerun the full gate. Stop only for a rebase conflict, an unfixable identity/determinism failure, or the hard stop. The previous 42/45 failure remains evidence, not a waiver or a successful gate.

### P-2 — Accepted functional evidence and frozen policy

Keep shadow round4, held-out1232–1263 outcomes/why, baseline identity and isolation. All157 policy nodes stay frozen, including scores; no policy tuning. Held-out seed1238 wolf deaths BASE0→mind3 stays recorded. Original policy SHA256 remains `7a6c90ac89db9cfb08acac6a8e14c91d1a1690d9fa40c3798c52323ead43eae8`.

### P-3 — Profile, preserve behaviour, then prove the performance bar

Before code changes, sample release `worldsim_perf` N1000 with samply or macOS sample/xctrace and record the top15 functions by self and inclusive time in CORE_PERF. Source hypotheses alone do not justify optimization. Start with behaviour-preserving changes: lazy per-person-tick memoized facts; load-time indexed predicates/bitsets; band-order early exit; life-stage/tag filtering before predicates; debug-only record validation; reusable hot-loop buffers; shared spatial/claim snapshots with correct invalidation. These are candidates, not permission to alter outcomes or skip required evidence.

For every implementation commit, prove the10 colony summary SHA256s in CORE_REPORT's switch table right column, steward-ON summary `cad9811728445bba5d8aa9027d6450ceb9aec3b40635258572577b711c387958`, held-out outcomes `42e9de5b1649ced049ca37a9741f15f9b900baaa61ce128eb7f84d8d5781ebed` and why `552ccf8b9932e23d1243b185c6fb4a35759df21a10d364f6e633430ce618a6a8` byte-equal, plus baseline identity byte-identical. Preserve the original evidence and bind each new build/capture to its actual source.

Required: step25 bench passes with no `--justified`, including C3 N1000≤5ms and N2000≤10ms. The planner's approximate arithmetic is1.10×2083µs≈2291µs total, leaving mind≈1663µs≈1.14×old decision, requiring≈11.5× improvement over19136µs. The actual standing checker remains authoritative for every scene/mean/p99/memory constraint. Report mind/old ratio; ≤0.5 remains a target, not required. Step45 must validate a fresh same-session interleaved BASE/M-CORE receipt, every repeat starting load1<8 with1/5/15 samples (D10).

Only if preserving behaviour cannot reach the required bar may a separate behaviour-changing optimization commit alter rethink scheduling without changing nodes/scores. It requires shadow seeds0–7 under the same≥97% rule and fresh held-out1264–1295 under the same outcome/why/table rules. Do not use these fresh seeds before that conditional path is justified.

### P-4 — Narrow, conditional renderer stall waiver

Planner identifies the prior91.652ms zero-sim-tick frame with85.4ms pre/post draw as the known renderer issue ISS030/#515. For this pass only, if fresh step41 fails and **every** frame>50ms advanced0 simulation ticks, publication may proceed with that single red step. Any>50ms frame advancing≥1 tick must be fixed. Preserve failed raw captures and inspect every qualifying frame; an unverified/partial receipt cannot establish the condition. Record actual waiver use in DECISIONS, CHANGELOG and comments on #515/#646. The historical first50.194ms frame advanced a tick, so the old failure is not automatically waived.

### P-5 — Story findings, report-only

Break chronicle growth≈120→308 entries/world-year down by kind. Determine whether zero depth≥3 chains and121/125 missing-ancestry chains arise from the instrument or broken cause links, and open an issue either way. No fix is required in this pass; retain original measured totals and provide evidence for the diagnosis.

### P-6 — Publication and issue closure

After acceptance, fast-forward `v8/core:v8/wip`, then `v8/core:v8/main` from BASE9855bd082ad88a4c49296410f206f148d98c4df9. If main advanced, use the original conflict-free rebase-and-full-regate rule. Close fixed registry #639–#641/#643/#644, and #645 if the bar passes, with public commit links. Leave `v8/core-review`; no branch deletion is authorized. Final Korean addendum includes profiletop15, optimizations/preservation, before/after performance and arithmetic, gate/step41 detail, P5 findings, published SHA and `git ls-remote`.

## D26 — Exact provenance-only payload comparison (2026-10-27)

Authority: planner's same-thread Option1 reply on2026-10-27: list excluded provenance keys explicitly in a committed file, use one committed script to delete exactly those keys on both sides and compare canonical JSON bytes, and record original plus both normalized payload hashes. No other fields may be excluded. The reply reiterates that no timing run counts until the1-minute load is below8.

`CORE_PROVENANCE_KEYS.json` lists exact JSON pointers by artifact, and `tools/v8/core_equivalence.py` is the single comparison implementation. Outcomes excludes only mind source/binary SHA and the four BASE/mind summary/receipt input hashes. Why excludes only source/binary SHA and summary/receipt input hashes. Policy, scorer, validator, why-checker hashes, seeds, counts, per-seed results, bars and all other fields remain compared. These output schemas contain no build/run timestamp or path fields requiring exclusion; none are permitted by this contract. Colony summaries have **no excluded keys** and still require literal byte equality as well as canonical equality.

Canonical JSON sorts object keys and represents every number using exact decimal coefficient/exponent formatting on both sides, without binary-float rounding. Duplicate keys and nonfinite numbers are rejected. The comparison refuses to run as production evidence unless script and key-file bytes match committed HEAD. Receipts retain both raw hashes, both canonical payload hashes, exact excluded pointers and checker/contract hashes. Original outcomes SHA256 remains `42e9de5b1649ced049ca37a9741f15f9b900baaa61ce128eb7f84d8d5781ebed`; original why remains `552ccf8b9932e23d1243b185c6fb4a35759df21a10d364f6e633430ce618a6a8`. Normalized reference and candidate hashes will be recorded below after the prerequisite tooling commit and actual captures; this entry makes no equivalence claim yet.

The original frozen shadow checker/archive/binaries remain unchanged. A separate measured-successor verifier authenticates their original replay, exact source transitions and per-commit preservation evidence. Explicit documentation/tooling-only links require unchanged Rust trees and enumerated allowed paths; they do not claim new runtime evidence. Fresh optimized captures and paired timing remain required before publication.

After prerequisite commit `3c5c93f444a9020573238b4f6d5a36e9f327e9c8`, the committed comparator produced these original reference payloads:

| Artifact | Original raw SHA256 | Original canonical payload SHA256 |
|---|---|---|
| outcomes | `42e9de5b1649ced049ca37a9741f15f9b900baaa61ce128eb7f84d8d5781ebed` | `13fe08fd870a2f90b1a5b6fa3a89bebb9936e6956c545992228d2a3b4c97d257` |
| why | `552ccf8b9932e23d1243b185c6fb4a35759df21a10d364f6e633430ce618a6a8` | `08f179819d9d72e06141fa56a23ed061b14fb13cdc5cb1439d935bc325614207` |

Checker SHA256 `72b7f957e86b0b601dad75bd809da304886cd56751dce57e85522acfdf6f129d`; key-file SHA256 `ff6a24f77389907f97af2c22ce460af64a34dad89d48547420356ba56e2339c6`. External `repair-tests-1/original-{outcomes,why}-normalization.json` receipts are self-comparisons of original evidence, **not candidate equivalence**. Candidate payload hashes remain pending actual capture/comparison.

### Candidate1 actual preservation capture (2026-10-27)

The exploratory candidate1 binary `fa623d9174d8fd4bdf81ae4dc1f16352ea888752824f62fc9ea77f65d7e9af51` was built from archived Rust inventory `f114f7815af48baa3ca8cf6edaddd904e6c6c4f65b26b2f71df251110c834df5` (uncommitted source, parent48f4b2fa2; not a final-source acceptance claim). `repair-preservation-1/safety-compare.log` proves all nine OFF summary bytes and baseline summary/raw/chronicle identity; ON summary remains literal `cad9811728445bba5d8aa9027d6450ceb9aec3b40635258572577b711c387958` with determinism PASS. No colony-summary exclusions were used.

| Artifact | Original raw SHA256 | Candidate1 raw SHA256 | Original normalized payload SHA256 | Candidate1 normalized payload SHA256 |
|---|---|---|---|---|
| outcomes | `42e9de5b1649ced049ca37a9741f15f9b900baaa61ce128eb7f84d8d5781ebed` | `62a898eae7f9e806bf121c2124e9dfa303aa58a0fa7e79c892fc5d0951c311dc` | `13fe08fd870a2f90b1a5b6fa3a89bebb9936e6956c545992228d2a3b4c97d257` | `13fe08fd870a2f90b1a5b6fa3a89bebb9936e6956c545992228d2a3b4c97d257` |
| why | `552ccf8b9932e23d1243b185c6fb4a35759df21a10d364f6e633430ce618a6a8` | `1c0583f922c4f37fd6dc0e249b8e9bea669137733a25f84b1f3b0fb682a4aaf9` | `08f179819d9d72e06141fa56a23ed061b14fb13cdc5cb1439d935bc325614207` | `08f179819d9d72e06141fa56a23ed061b14fb13cdc5cb1439d935bc325614207` |

Both comparisons ran the committed D26 script and exact key contract. Raw receipts and terminal completion are retained in `repair-heldout-1`; frozen seeds1232–1263 and all outcome/why fields are unchanged. Further optimization is a new candidate requiring fresh preservation proof before the implementation commit. Candidate1 timing remains exploratory; it does not satisfy the required performance bar.

### Load-window follow-up

The retained15-minute samples at09:11:30/09:26:30/09:41:30 KST were respectively1/5/15 loads `19.1553/16.7246/11.9209`, `7.7690/16.3403/17.6074`, and `9.6279/10.6021/13.1069` (`repair-load-windows-1.jsonl`). None launched a timing run. They are not three consecutive contended windows because the middle sample was below8. Untimed captures/builds contributed load; these samples do not attribute all contention to another job. Every new timing repeat still checks load immediately before launch.

## D27 — P-5 observer-only diagnosis (2026-10-27)

Authority: planner P-5 in the 2026-10-27 note recorded by D25. Additive observer captures reproduce all original annual metrics and 32 final fingerprints per arm. Full kind/ancestry evidence is in CORE_STORY_DIAGNOSTICS.md. Counts7676→19722 comprise mind_choice +12142 and other kinds −96 across64 complete world-years per arm. Missing121/125 comprises explicit unknown108/107 plus descendants13/18; no missing concrete entry/sample and no cycles were observed. The Chronicle-only instrument omits the complete Mind EventId ancestry graph; depth≥3=0 cannot establish full causal-story depth. No policy or cause links were changed to improve these figures.

Opened ISS341/#647 and posted the bounded evidence follow-up https://github.com/hyunlord/new-world/issues/647#issuecomment-6431336839; create and comment bodies/IDs were read back and verified in p5-github-journal.jsonl. Keep the issue OPEN. Observer candidate1 is honestly pinned to its uncommitted Rust inventory, not represented as the eventual publication binary. Original raw evidence remains unchanged.

## D28 — Preserve evaluation semantics while removing repeated work (2026-10-27, in progress)

Authority: planner P-3b in D25. Before-change sampling was committed in p10; subsequent attribution-only profiles retain their INVALID timing status. No policy node, score, cadence, held-out seed set or causal-outcome rule is tuned. Candidate6 is a single uncommitted implementation candidate; proof and final acceptance remain pending.

The current design uses lazy per-selection fact memoization; load-time routine metadata; current person-row indices; existing routine reservation cursors; fixed dependency stamps; static overlay columns with arbitrary-name fallback; a food spatial index with known-tile incremental updates and generic stale-index fallback; and bulk index construction. Record dictionary validation remains strict at initialization/load/public checked APIs, with a trusted initialized insertion path and debug assertions. Pure origin construction is skipped only for the exact unfinished-node continuation already skipped by the recorder.

Compiled predicate identities reuse successful comparisons only for readers explicitly promising stable, side-effect-free repeat reads. The default FactReader contract remains uncached, including changed values and later errors. Authored node/predicate traversal, first-fact witness order, role unions, top3, incumbent hysteresis and first reached errors remain unchanged. Unit score weights use the exact identity `(value * 1000) / 1000 = value`; all checked accumulation and nonunit arithmetic remain in place. Policy revision/dictionary metadata excludes ephemeral cache IDs.

Candidate6 archive Rust SHA256 `b914056b9066ad88b9a6b3b69724f74dc9015cfed1e03a03d5b653978f63cdd6`; probe `36e613f2456703d59216a8c6f4bb62f9133e29d187dba288c8391dc828990320`; perf `c16a4a510a68a342b7095517be429e00ea33f325370318a140ebf5d969dd1b5e`. Parent is p11 `3c5c93f444a9020573238b4f6d5a36e9f327e9c8`; these are honestly uncommitted-source build pins, not publication evidence. Build/source inventory and archive are under repair-candidate-bin-6. Focused selector24, runtime-mind69 and spatial7 tests pass, as does strict clippy for all targets with mind_shadow. Independent read-only semantic audit found no concrete defect within its scope; it does not replace golden captures.

The overlay layout is measured with size_of: optional derived cell96B, optional target cell104B,33 column headers792B, all columns populated3304B/person and TickContext1128B. Columns allocate lazily; these figures are a storage tradeoff, not a measured peak-memory improvement. Final standing memory and timing gates remain required. Bulk-boundary testing exposed the existing debug-only intermediate overflow in `lo + CHUNK - 1`; reassociation to `lo + (CHUNK - 1)` preserves the release result while covering the final u32 chunk in debug. The failing test logs remain retained.

### Candidate6 held-out preservation result

Capture finished at2026-10-27 10:41:15 KST with determinism PASS and committed D26 comparisons PASS. This is exploratory source preservation, not final gate/performance acceptance. All non-provenance outcome/why fields and original seed1238 adverse result remain equal. Baseline/ten-summary safety capture is still pending.

| Artifact | Original raw SHA256 | Candidate6 raw SHA256 | Original normalized SHA256 | Candidate6 normalized SHA256 |
|---|---|---|---|---|
| outcomes | `42e9de5b1649ced049ca37a9741f15f9b900baaa61ce128eb7f84d8d5781ebed` | `ea06f889f5f209477bbc8636fc850d088daf59266622b790766a1c68f9744af1` | `13fe08fd870a2f90b1a5b6fa3a89bebb9936e6956c545992228d2a3b4c97d257` | `13fe08fd870a2f90b1a5b6fa3a89bebb9936e6956c545992228d2a3b4c97d257` |
| why | `552ccf8b9932e23d1243b185c6fb4a35759df21a10d364f6e633430ce618a6a8` | `3d0840a8f0373427d7cf38741805bf50443e0d97ec7fe3d8d0ae05ec1180525d` | `08f179819d9d72e06141fa56a23ed061b14fb13cdc5cb1439d935bc325614207` | `08f179819d9d72e06141fa56a23ed061b14fb13cdc5cb1439d935bc325614207` |

### Three contended windows: defer timing to after untimed work

At10:13:51,10:28:51 and10:43:51 KST on2026-10-27, the15-minute samples were respectively `18.1382/25.4297/19.7192`, `42.1709/50.2847/40.6108`, and `28.2959/75.1196/75.0122` (1/5/15-minute loads). All three meet the planner's contention condition. No timing run was launched by this monitor; raw samples remain in repair-load-windows-2.jsonl. Continue untimed inventory/implementation/preservation/report work and perform timing at the end before the gate. Owned builds and preservation probes were also active during parts of this window, so this is not an attribution of all load to external jobs. Any attribution-only profiler output is explicitly INVALID timing.

Candidate6 safety completed at10:50:51 KST: all nine OFF summaries and the ON summary are literally byte-identical to the ten frozen hashes in CORE_REPORT, and all ten committed D26 canonical comparisons pass with zero excluded keys. Existing baseline identity comparison passes summary/raw/chronicle checks. Observed pre/post Rust hashes both equal the candidate6 source hash above; terminal and per-command exit/hash receipts are retained in repair-preservation-6. This completes candidate6 exploratory behavior preservation; no implementation commit or valid performance acceptance is claimed yet.

### Candidate7 compact storage and snapshot lookup

Candidate7 retains evaluation cadence and all policy bytes. Predicate cache entries pack validity, match and role bits into one byte; a valid entry is published only after its witness slot exists. Snapshot capture resolves the ECS entity once while retaining ordered component reads and borrow/error semantics. Dependency stamps store the exact u128 sum of selected counters: private counters only increase by checked increments within one TickContext, so nonnegative differences cannot cancel; at most nine u64 counters sum below2^68. Stamps must never migrate across contexts or survive counter resets.

Core mind37 and runtime mind76 tests PASS; strict clippy all targets with mind_shadow PASS. Differential tests cover all512 dependency subsets, duplicate/empty domains, maximum counters and overflow, snapshot missing components/borrow guards, and predicate partial errors/reset. Read-only semantic review found no actionable defect (`candidate7-policy-preservation-review.md`). Overlay fully populated storage is1584B/person versus3304B in candidate6; this is layout evidence, not peak-memory or timing acceptance.

Rust source `ca3c884968c0e3a8d08310850a69ddfe8cf147140a4ea9394ee0f644716b4860`, probe `60874074968368874920c68c7eb8db7eff73baa01948af5d015c253c9042823d`, perf `ddb689072d6387a46a420e9a1735c25e225807033fac7d544b20a8ad99f9c6b0`; archive/build receipts retained in repair-candidate-bin-7. This is an uncommitted-source exploratory build. Full preservation captures are running; no final acceptance is claimed. Candidate6 diagnostic waited without launching any repeat and was closed NOT_RUN with all load samples retained.

Candidate7 held-out preservation completed at11:13:48 KST with determinism and both committed canonical comparisons PASS. Safety/ten summaries remain running. Original raw receipts are unchanged; all non-excluded fields and seed1238 outcome remain equal.

| Artifact | Original raw SHA256 | Candidate7 raw SHA256 | Original normalized SHA256 | Candidate7 normalized SHA256 |
|---|---|---|---|---|
| outcomes | `42e9de5b1649ced049ca37a9741f15f9b900baaa61ce128eb7f84d8d5781ebed` | `c564b0209124c106ebdc24ade9ef10ed6056c6ef28ffad62f88964980cf6ed13` | `13fe08fd870a2f90b1a5b6fa3a89bebb9936e6956c545992228d2a3b4c97d257` | `13fe08fd870a2f90b1a5b6fa3a89bebb9936e6956c545992228d2a3b4c97d257` |
| why | `552ccf8b9932e23d1243b185c6fb4a35759df21a10d364f6e633430ce618a6a8` | `8b47cf16db831c519403f27904bc1e4fb31dc42c60d136b3eb7f4ac8da2545e6` | `08f179819d9d72e06141fa56a23ed061b14fb13cdc5cb1439d935bc325614207` | `08f179819d9d72e06141fa56a23ed061b14fb13cdc5cb1439d935bc325614207` |

Candidate7 safety finished at11:23:20 KST: all nine OFF summaries and ON summary literal SHA matches, baseline228748 shared values/zero differences and64 chronicle files19683 entries match. All ten D26 canonical comparisons pass with no excluded summary keys. Pre/post Rust hashes both ca3c884968c0e3a8d08310850a69ddfe8cf147140a4ea9394ee0f644716b4860. This completes exploratory preservation; no performance acceptance or implementation commit yet.

## D29 — Candidate8 preservation and measurement boundaries (2026-10-27, in progress)

Authority: planner P-3b and repeated load<8 rule, as recorded in D25/D26. Policy and scheduling remain frozen. The selector keeps only ordered top3 and the incumbent instead of storing/rescanning every eligible candidate; it still evaluates every node in the chosen band and preserves all later errors and witness reads. Memory observations use one ordered pass instead of twelve, retaining separate negative-zero accumulators, per-arm addition order, exact recency/product expressions and last-on-equal/unordered contributor selection. Differential tests compare the original algorithms.

Read-only review found a trust-boundary gap in the uncommitted initialized-history optimization: replacing public MindState.history could inherit the system boolean's trust despite a corrupt same-revision dictionary. Validation is now cached privately on the immutable history object, cleared on deserialize, checked before tick effects/continuation/insertion. The cache is excluded from serialization and semantic equality; public strict APIs remain strict. This fixes the introduced gap without repeated full dictionary scans in the valid hot path. No failing evidence was removed. Initial test compilation exposed undeclared test-only serializer dependencies; tests were adapted to the crates' existing JSON/bincode dependencies, with no new dependency. Failed logs remain in repair-tests-8.

Step25 previously admitted the whole bench once, and its numerical ledger checker did not reject high-load repeats. The parent bench now admits each timed and RSS child independently at fixed load1<8, records all1/5/15 values in a create-new admission.jsonl, retains rejected samples, and stops before launch on missing/invalid load or timeout. Timed child load_before is independently checked after completion; >=8/malformed is journaled INVALID and excluded from aggregation. RSS retains the existing instrument and its historically after-build load_before field, explicitly distinguished from the new true prelaunch sidecar. Child workload, memory measurements, summary schema and numerical bars are unchanged. Original BASE binaries remain immutable.

Candidate7 exploratory diagnostic was closed NOT_RUN with zero launched repeats at11:25 KST after load rebounded to38 despite owned captures ending. The binary and all wait samples remain retained. Candidate8 tests/build/preservation and admitted timing remain required; this entry makes no performance acceptance claim.

Candidate8 focused verification: debug core43/runtime83/bench8 PASS; release core43/runtime83 PASS, including exact memory-value bits and same-revision malformed-history rejection without panic. Strict clippy all targets with mind_shadow PASS. Original failed compilation logs remain retained; tests use existing dependencies only. Final-source build, full preservation and performance acceptance remain pending.

Candidate8 exploratory build: Rust `b3f9061f3bd1505ddef08f94780e19726c2653f504a403567ffc8e4280cd19c5`, probe `7e3ecf00746be11c46c59df4384b1b3d47a79217f2634f6257ff5003a175d541`, perf `04a1487617d12581cbe29e8ab459500207751f666953b1712e7a52f9bbd648a3`; parent p11, default features[], pre/post build source hashes equal. Build/archive/inventory receipts retained in repair-candidate-bin-8. Full source-pinned preservation is running; these pins are not a committed-source publication claim.

### Candidate8 held-out preservation

Completed at 2026-10-27T02:41:04.084171+00:00 with determinism and committed D26 canonical comparisons PASS. Safety/ten-summary capture remains running; no timing acceptance is claimed.

| Artifact | Original raw SHA256 | Candidate8 raw SHA256 | Original normalized SHA256 | Candidate8 normalized SHA256 |
|---|---|---|---|---|
| outcomes | `42e9de5b1649ced049ca37a9741f15f9b900baaa61ce128eb7f84d8d5781ebed` | `1655bdefdbe6fecbb3c97af1e8add5c200d738ed3d5711db4330357e354cd215` | `13fe08fd870a2f90b1a5b6fa3a89bebb9936e6956c545992228d2a3b4c97d257` | `13fe08fd870a2f90b1a5b6fa3a89bebb9936e6956c545992228d2a3b4c97d257` |
| why | `552ccf8b9932e23d1243b185c6fb4a35759df21a10d364f6e633430ce618a6a8` | `0c32c086b033f159ffbf4955ca8581108a1508fa7c02dfdc8305844b48d1eb45` | `08f179819d9d72e06141fa56a23ed061b14fb13cdc5cb1439d935bc325614207` | `08f179819d9d72e06141fa56a23ed061b14fb13cdc5cb1439d935bc325614207` |

Candidate8 full exploratory preservation completed at11:49:54 KST. All nine OFF summaries and the ON summary are literal matches to the ten frozen CORE_REPORT hashes; all ten canonical comparisons pass with zero excluded keys. Baseline228748 shared values/zero differences and64 chronicle files19683 entries pass the standing identity comparison. Pre/post Rust source hashes both b3f9061f3bd1505ddef08f94780e19726c2653f504a403567ffc8e4280cd19c5. Proof tree repair-preservation-8 and heldout8 remain immutable. This completes source preservation, not performance/gate acceptance.

Candidate8 admitted N1000 diagnostic completes all four interleaved members with no sleep: tick2072.718→4079.104µs (1.967998×), old/mind1447.824→3351.900µs (2.315129×), p99 average5181→7117µs. This remains an exploratory FAIL of the standing bar, not a final gate. Full preservation is retained while additional preserving feasibility is assessed; P-3d has not been invoked and fresh1264–1295 seeds remain untouched.


## D30 — Invoke the separately validated scheduling fallback (2026-10-27)

Authority: planner note 2026-10-27 P-3d, conditional on preserving optimization failing P-3c. Candidate8 completes all preservation proofs in D29, but admitted interleaved N1000 tick mean is4079.104µs against BASE2072.718µs; mind3351.900µs against old1447.824µs. The paired +10% tick budget is2280.0µs, leaving approximately1552.8µs for mind after727.2µs other work, requiring another53.7% reduction. These exploratory numbers do not replace the standing ledger or final paired gate.

Read-only feasibility review found no evidence-backed preserving design delivering that remaining reduction with margin. Prefix-sharing and snapshot opportunities remain hypotheses, not proven impossibility or accepted performance. Preserve this completed behavior-preserving implementation in its own commit, then apply the planner-authorized continuing-behavior rethink scheduling as a separate implementation commit. The157 policy nodes and scores remain frozen. Re-run shadow seeds0–7 under the same97% rule; only after design/shadow freeze use fresh held-out1264–1295 under the original table rules. Original1232–1263 results, canonical proof and seed1238 adverse outcome remain retained. No fresh held-out seeds have been accessed at this decision.

The predecessor projection tool was also repaired: the historical measured Rust inventory excludes ignored Cargo.lock, while the original migration checker requires the lock to build its projection. Authenticate Cargo.lock independently using the original frozen manifest and require the archive to contain exactly measured inventory plus that support file; never insert it into the measured source digest. Seven archive controls plus13 existing successor tests pass. The actual p8 projection passes the unchanged migration checker with TEST_ONLY_AMENDMENT_VERIFIED. Evidence: repair-successor-inventory-proof-v2.json and p8-projection-v2-verification-1. No original receipt, archive, manifest or checker was rewritten.

Preserving implementation committed as p12 `1ca297b82f738cdcd881a344bc863bde669abe1a`. Its clean committed release build (features[]) completed at12:07:35 KST, exit0. Rust digest remains `b3f9061f3bd1505ddef08f94780e19726c2653f504a403567ffc8e4280cd19c5`; probe `8d89f6ec93429ccdc6b99876cf9913c2ccfcad938f60c2096c42fa32c9d26b0e`, perf `2485fb1c00402d15a2b659941dc37d2383bc81d0138bfc2b00021f9bbadb8f67`. Build receipt SHA256 `6d843a87bbac2110c5787aa52cacfeafc437f408d4ee38ffc0c856cdb56d6dc6` under repair-preserving-bin-p12. Committed-source canonical recapture is running in repair-preserving-proof-p12; no new performance or gate acceptance is claimed.

For historical checker-byte binding, retain detached p12 verification view `/Users/rexxa/work/core-data/p12-verifier-root`; inventory manifest `/Users/rexxa/work/core-data/p12-verifier-view.json` SHA256 `d9ae619f7229d5da96cafeb8daffca8fe67be3c836e76899a36cb61d021bad3e` binds911 Rust/tool files including the independently copied frozen lock. Its source digest was independently recomputed and matches p12. This is a verification context, not a new build or run. P3d protocol extensions must use new receipts with actual checker hashes; old receipts will be revalidated with these original bytes and never rewritten.

Standing-step25 arithmetic checked directly: bench.sh/bench_ledger.py selects the latest quiet non-candidate ledger pass. Current latest M-ART2.1 n1000 mean2018.6µs gives a1.10 ceiling2220.46µs, stricter than the planner's approximate2291µs or diagnostic8 paired2280.0µs budget. Holding candidate8's other work727.204µs fixed leaves about1493.26µs mind (55.45% further reduction). Actual scene/p99/memory checks remain authoritative; no ledger row or threshold is changed.


### Committed preserving boundary and bounded scheduling diagnostic

p12 `1ca297b82f738cdcd881a344bc863bde669abe1a` recapture completed at12:38:30 KST. All ten colony summaries remain literal matches (no provenance exclusions), baseline228748 values/zero differences and64 chronicle files19683 entries match. D26 canonical outcomes and why comparisons pass. Raw outcomes SHA256 is `dda1abd0fc930757ad5c4c252316147917595348ad049b6d1de175c6b4e9e4e2`; raw why is `fe702800e24ebb03b4380ca94f68cc356aa5409bd05c901db6ee9203c7f8f26c`. Original raw SHAs remain `42e9de5b1649ced049ca37a9741f15f9b900baaa61ce128eb7f84d8d5781ebed` and `552ccf8b9932e23d1243b185c6fb4a35759df21a10d364f6e633430ce618a6a8`. Original and p12 normalized outcomes both `13fe08fd870a2f90b1a5b6fa3a89bebb9936e6956c545992228d2a3b4c97d257`; normalized why both `08f179819d9d72e06141fa56a23ed061b14fb13cdc5cb1439d935bc325614207`. Capture receipt `eff9e19e5feeffb2f158603d049d8b1891d32b40c9a5044bd359b89b664c2ccd` binds equal pre/post Rust hashes. Strict successor construction re-executed the original evidence chain successfully; `repair-preserving-successor-p12/construction-verified.json` is the proof endpoint. This establishes preservation, not performance or full-gate acceptance.

An observation-only external diagnostic always executed the full selector and effects. Its4320309 visits include1440 warmup ticks:2436222 (56.3899943%) could retain an active resource/routine;5745 prospective different choices (0.1329766% overall) are diagnostic only, not scheduled shadow results. Guard errors were zero. Social579346 and wander591262 visits were excluded; routine_no_place97122 is a Finished callback and must remain full because it reissues admission denial/cooldown. Data and source/binary pins are in p3d-diagnostic-counts-1 and p3d-diagnostic-bin-2. All its timings are INVALID_ATTRIBUTION_ONLY.

Based on those counts, the first actual P3d implementation extends the bounded design to registered active social continuation and Idle/wander no-op continuation, retaining immediate authored critical/finish interrupts and fixed phase8. Ordinary noncritical opportunities can wait at most7ticks; no_place remains full. Shared snapshot preparation must preserve original peer/cohort observation timing while deferring memory reductions and target scopes. This is explicitly behavior-changing under planner P-3d, requires new shadow/fresh held-out proof, and makes no performance prediction from coverage. Policy nodes/scores remain unchanged. Fresh1264–1295 remain unread and unexecuted at this decision.


### First actual scheduling candidate (uncommitted exploratory source)

Runtime source `fa162f92c2d10adeb4590ac925d031ad6894c58db8e7de0c584af1b859890f28` is identical across all740 Rust inputs in the tested external tree and checkout. Core52/systems115 focused tests pass, including actual effects/completion and save/resume at off-phase and phase boundaries; workspace/all-targets clippy with mind_shadow and warnings denied passes. The first clippy failure (helpers placed after a test module) remains in p3d-runtime-tests-1/clippy.log. Review also found Idle capability included hearth_meal; eligibility now requires a targetless Idle registry entry, excluding hearth_meal without a per-tick name lookup. No-place remains full because it is finished. Policy and save schema are unchanged.

Retained ticks use the prior validated record identity and causal ID, execute real continuation callbacks and the common completion/meal/invalidation tail, and create no fabricated score/witness record. Full fallback refreshes complete person facts and executes the unchanged selector. Tick-start capture computes the social contributor needed by frozen peer context; full readers reject incomplete rich rows. Claims, target scopes and social timing remain captured at their original boundary. This first slice still pays light component refresh, routine lifecycle refresh, scope lookup, night-rest and handshake setup; it does not claim to eliminate those costs.

Default-feature exploratory release build succeeded: perf `6d5ea4879c9009083065490aa3e2a21cc73c190eb4807745f8556d8b9f3ad13a`, probe `d1751e8a6b370ddd73cd4e76230629672552504c0e099a79a17c9396352e0bde`. Build/archive/source receipts are p3d-scheduled-bin-1; source label is honestly parentp12 plus uncommitted inventory, not a committed final binary. First N1000/N2000 ABBA session waits for load1<8 after the build (observed19.25/12.14/9.71 at13:07KST). 5/15-minute values are witnesses only. Final timing, shadow, fresh held-out and full gate remain pending.


At13:13:16 KST the parent ended the owned exploratory admission controller while it was waiting (SIGTERM, recorded exit130). p3d-diagnostic-pairs-1 is NOT_RUN with zero launched repeats and no comparisons; every rejected load sample remains. Recent samples13:12:03=9.5894/10.7178/9.6421 and13:13:03=11.7393/11.1714/9.8838 remained contended after owned builds stopped. No timing result is inferred. Following the earlier three-window deferral in D28, continue untimed host integration/tests and retry timing before the gate. This is a scheduling change, not an INVALID measured run or a lowered admission bar.


### Scheduled candidate2: measured attribution and bounded overhead removal

The first scheduled candidate was sampled before this slice. p3d-profile-scheduled-1 retains actual sample/benchmark exits0 and top15 self/inclusive aggregation in CORE_PERF. Preload37.3682/20.8501/13.8384 makes every elapsed timing INVALID_ATTRIBUTION_ONLY. The sample nominates continuation eligibility, lifecycle overlays, metadata allocation and history lookup; it establishes no speedup or bar result.

Candidate2 replaces the guard predicate cache with reset tri-state bytes without changing ordered reads/errors, batches the four existing routine lifecycle slots at their unchanged dependency stamp, reuses validated running metadata allocations, and reuses the prior history lookup only for retained effects. Full selections keep their post-record causal lookup. Cadence8, owner eligibility, policy nodes/scores and behavior-changing scope remain exactly candidate1. Original guard/metadata algorithms are retained as test-only differential references. The cleanup plan precedes edits at p3d-scheduled-opt-plan.md; patch d24791f099d3c001b2826512218f00851e1c3288f3e0ab967bebbf4950e00cfb and independent read-only review are retained.

Core53/runtime118 focused tests pass, including ordered guard errors, lifecycle invalidation, metadata error atomicity and actual retained/full social causal IDs. Checkout and external candidate agree on all742 Rust inputs: 11be938525756c6060d0ca8e84a95aa71f77d4fe55ab3a2132318157b75464de. Integrated Python140 tests pass, including the builder-v3 exact external-input contract. This is uncommitted exploratory validation, not fresh shadow/held-out/performance acceptance. Final clippy/release build and actual shadow remain pending.

The first hybrid host unit-test build failed honestly: one adapter import was stale, and production unit-test modules reference a current-worldgen helper absent from the frozen old registration/worldgen host. Preserve shadow-tests-1.log. Fix the adapter import; do not suppress tests or invent a worldgen shim. Production tests run the production tree; the hybrid host is validated through its real compiled probe, primitive fixture factory/replay and shadow execution. Earlier prepare/build attempts remain immutable.

Candidate2 default-feature release build exited0; pre/post source agrees with11be9385 above. Probe `820f63f3c60ecf7d0923bc1f38f5d41a791c47c382528af238b21c0414d96e5b`; perf `c510f72c29d64bc55cfe0d3a15de3b0e79878740e29ed705c40d5f1831657b30`; build receipt `bd3caf6684098b93db7d18c71c4efccc123b2ce4f3f3e1101d8d3c98a2dbd86e`. Strict workspace/all-targets clippy with mind_shadow and denied warnings also passes. Binary provenance remains exploratory/uncommitted. External target cache use is explicitly recorded by p3d-exploratory-build-v2.py and does not change source closure. At13:26:26 KST load42.5215/29.9287/20.3555 remained above admission, so no timing was launched.

Candidate2 hybrid host debug build and13 actual primitive fixture cases pass (production/host equality and independently regenerated replay; only retained, idle_owner, social_owner continue). Raw receipt SHA256 `b9724782d6159be6085da2b29dbfb4b6d773c061177e0756152128ddb37ad625`, debug binary `746ae9fd7d8e2d1e447e1a7a7f4aae63ff901d0215314e2d8085fc5df52ffc39`; evidence p3d-shadow-host-exploratory-4. Earlier missing social-peer hunger fixture failure remains in exploratory-3; complete peer need components fix the fixture, not production behavior. Controlled release control/shadow build remains in progress.

Historical p12 verifier audit found two generated .pyc files from the earlier standalone verification invocation: -B did not propagate to its Python subprocesses. Both are preserved with hashes/metadata under p12-verifier-cache-quarantine-1; exact911-file approved inventory is restored without changing source. Current P3d invoke already propagates PYTHONDONTWRITEBYTECODE=1, so historical child verification remains strict and mutation-checked.

Preflight audit corrected a new P3d identity-check bug before capture: literal raw-file SHA comparison would reject the genuine candidate git_sha label even for unchanged simulation. The checker now applies the standing tools/v8/identity.sh raw_digest rule (top-level git_sha plus only identity_removed_keys.txt entries proven empty/zero), then compares exact filename-to-canonical-digest maps. D26 outcome/why/colony exclusion lists are unchanged. Fourteen targeted tests pass, including missing/extra files, changed simulation values and nonzero removed fields; the historical BASE sample reproduces identity.sh digest. Patch76d7f0b79a4a6397b568d5e39eb22d4924768d2bed12973bd4dc2525f55291c0 and failed-first logs remain in p3d-baseline-label-fix. This restores the existing baseline identity contract; it does not relabel candidate output or claim new baseline execution.

The same audit added missing held-out terminal validation before scoring: both fresh arms must have strict integer probe_exit0 and their actual probe logs must contain DETERMINISM PASS with no FAIL. Seventeen successor tests pass, including22 invalid arm cases rejected before scorer invocation. Patch e0bf9059fd1f15565b8809800181ea7dcd101ddbcaefa18988adc93ca4bcf8b2 and red/green logs remain in p3d-heldout-terminal-fix. No held-out world was executed to develop these tests.

Exploratory P3d calibration round1 OFF actual strict gate passes:2031192/2041927 behavior matches (99.474271%),69categories reviewed from1218 retained examples, zero unexplained categories. Same-feature control, feature-OFF identity, finalized accounting and all8 live guard replays pass. Raw captures are p3d-exploratory-calibration-1, independent review/strict log p3d-calibration-review-1. ON and complete controller endpoint remain pending; this is staged-source exploration and cannot replace a postcommit final calibration/freeze.

Exploratory round1 completed both modes. ON strict gate passes1684600/1692967 (99.505779%),58categories/1136examples independently reviewed. No-place3290/1692967=0.1943% is the sole ON category above0.1%;24 retained examples show full evaluation, admission1, weighted-place0 and Idle1, preserving the documented D16 mismatch. Both mode checks pass original97% arithmetic, control identity and coverage. All16 compiled guard replays validate3171241 retained and563653 full person-ticks;13primitive cases/factory/replay pass. Pre/post Rust SHA remains11be938525756c6060d0ca8e84a95aa71f77d4fe55ab3a2132318157b75464de. Controller result SHA `c5ec25b3f67780de62dd86d08da9925edaec1236aa9338270cb7ab92697fea67` is deliberately CAPTURED_REVIEW_REQUIRED, preserving its initial pending-review failures; later independent reviewed PASS receipts are separate in p3d-calibration-review-1. No result is relabeled as a committed final capture.

Candidate2 admitted interleaved diagnostic completes8/8valid members with zero sleep events. N1000 tick2269.376→3410.741µs (1.502942×); old/mind1575.388→2704.208µs (1.716535×). N2000 tick6560.099µs. Both means meet C3 time ceilings diagnostically, but standing mean2220.46µs still fails, with mind budget1513.927µs after observed other work. Preserve p3d-diagnostic-pairs-2 unchanged; profile candidate2 before further source optimization. No commit, fresh-heldout access or acceptance claim follows.

Candidate2 production calibration reference (seeds0–7,48days,OFF/ON,story/neighbours on) completed both actual determinism runs exit0 at14:01 KST. Literal summary SHAs OFF a8e2fb5e943a6524ab28f9993c7a7754d91360da5d009d5643c06829bfed9181; ON9577c7950e570b7a462fb524a3478483e5cc6dbc45e1e44989a663c9972d4d39. This is a preserving-optimization reference within P3d, never a fresh-held-out or original-p12 equivalence claim. Candidate3 plan p3d-scheduled-third-plan.md precedes exact ordered-prefix memoization, checked active-row reuse, prebound shared metadata and removal of discarded claim capture. First new prefix fixture failed because a short compiled policy legitimately omitted a fact; retain p3d-third-core-tests.log. Test-only correction preserves all applicable cases and the unchanged original guard oracle. Runtime121 tests and corrected core57 tests pass; release, calibration comparison and paired performance remain pending.

Candidate3 external default-feature release source0521b267361483d8750b1da3278c3a5d1b97c33f889f492bb6e850a60b08b47b built successfully: perf2015832aa13650d52eef811f62ec14a4ed03e99ba04cde15b2066169505eec57, probe ca3bbd3fb062c8e86a2529857d36c7fd71ab0e8a3778b1881b9993787379b3ab. Strict clippy passes. Its actual48-day seeds0–7 OFF/ON calibration exits0 with DETERMINISM PASS and summaries literally equal candidate2 (no excluded keys); preservation receipt SHA256 9beadd398fde820d5c2ceda06d2f76d72069c3ae0ce1085299089d9686129a13. This confirms that bounded calibration, not all accepted original-p12 outputs or final shadow/heldout. Current high-load profiling is INVALID timing and never contributes to acceptance ratios.

Candidate4 external integration tests pass61core/125runtime. Initial strict clippy rejected a redundant u32→u32 cast in new scope test; preserve p3d-fourth-clippy.log, remove only thecast, corrected strict workspace/alltargets/mind_shadow check exits0 (p3d-fourth-clippy-fixed.log). Candidate3 diagnosticpairs launchedzero members becauseeveryadmissionload>=8; NOT_RUN receipt andown-controllerSIGTERMjournalretained. No invalid timing converted into a base. Current checkoutPython145testsPASS; expectednegativeF10 testprintsFAIL whileunittestterminalexit0, notanactualF10gateclaim.

Candidate4 default-feature release build exits0 with identical pre/post source dfb350f1c2a5ffc9338e95c53b428813c67c6265afc56d2460d41d7a599504a2; probe d9d3f55a79bf7ac3295219325fccf5f2e637c240bb9b9537b622835673d5f4be; perf bc10deea9125380839ac6b7a1e38496fed099c6a759152e14fd6b24203ec856f. Independent review p3d-fourth-independent-review.md found no concrete preservation blockers, with end-to-end calibration/shadow/timing still pending. The complete docs checker passes (p3d-docs-check-1418.log), including live system table and snapshot58getters/866keys. ROADMAP current row now states repair-in-progress under P1/P3d while preserving originalp8FAIL. At14:18 remote main/wip remain BASE9855bd082ad88a4c49296410f206f148d98c4df9; core-review remains61260ec8c245d88c6d12ef015de815dae6849017. No publication or deletion performed.

Candidate5 selector exact-prefix jump tests initially failed because a new fixture authored cadence3 in need, which the unchanged loader correctly forbids. Preserve p3d-fifth-core-tests.log. Fixture-only correction moves variable cadence into leisure and retains later idle cross-band cases; original loader/policy/production unchanged by that correction. Corrected core63tests and strict workspace/alltargets/mind_shadow clippy pass. Candidate5 build source43aeac4fd9b93026ff7fde187696d70d56cd7f8047ca5759793323247f141b46 is exploratory; actual calibration and valid timing remain pending. Candidate4 OFF48-day eight-seed summary and churn files literally matchcandidate2, with determinismPASS; ON stillrunning.

Candidate4 both48-day eight-seed modes completed DETERMINISM PASS and literal summary equality withcandidate2 at14:26 KST, no excludedkeys. ReceiptSHA256 36373e0729c01364f2034fa597aedb0a2c6a0dd2d6a418a44d49178f3ed78030. Twelve exact reviewed Rustpaths were integrated into checkout after checking candidate2beforehashes; complete747-file Rustinventory now matches archivedcandidate4 dfb350f1c2a5ffc9338e95c53b428813c67c6265afc56d2460d41d7a599504a2. Integration receipt p3d-fourth-integration-receipt.json, unrelatedwork untouched, no commit/finalshadow/freshheldout/performanceacceptance claimed.


Candidate5 completed both actual48-day seeds0–7 OFF/ON determinism runs and literal summary equality with candidate2 at14:31 KST, with no excluded keys. Preservation receipt SHA256 9e426a22cd7fd0c7061fcfecb2735b7c681fe4f52c14a323664ae6416ad9c186. Reviewed selector prefix-jump paths integrated after checking all candidate4 before-hashes; full Rust inventory now matches candidate5 43aeac4fd9b93026ff7fde187696d70d56cd7f8047ca5759793323247f141b46. Integration plan/receipt p3d-fifth-integration-{plan,receipt}.json retain exact paths and hashes. Candidate6 runtime126tests and strict clippy pass, release build succeeds, calibration pending. Candidate7 record-header cache focused18tests pass; clippy/build/calibration and memory assessment remain pending. No final acceptance or valid new timing is claimed.


Candidate6 completed actual OFF/ON48-day seeds0–7 determinism and literal summary preservation at14:39 KST, receipt SHA256 2e0782ea1d7c56a412449c581c1869a04f297b151ea6004cdfc07a135266c851. Running metadata now uses a checked row-aligned three-state column; old-map differential coverage and independent p3d-running-packed-review.md found no concrete semantic blocker. Integrated 4 reviewed paths after complete candidate5 before-hash checks; all 750 Rust inputs match 64b7780e02fcb3608c22964563a6c18a37ab1f83af7b37abcb594fde80813595. Candidate7 release builds successfully and calibration is running. Candidate8 row-order runtime128 tests and strict clippy pass; no candidate8 release or timing has run. Candidate8 and9 disjoint source slices are composed externally for tests; they are not checkout or accepted results.


Candidate7 record-header cache completed OFF/ON48-day seeds0–7 determinism and literal summary preservation at14:41 KST, receipt SHA256 bb5bfd68f3e8398655d9068e016bf82146c8daf292a29ec530ffab61b32a8ab2. Both checked constructors cache revision/agent/node from immutable validated bytes; serialization and strict parse/resolve stay unchanged. Focused18 tests, strict clippy and independent review pass. Three exact paths integrated after candidate6 inventory checks; complete750-file source e1b38164deba864fe21a2e5ff5a576587940a67352328df3fda4d95c1df333b3. Payload grows48→72bytes on64-bit (+24 per unique payload, +384/person for16 retained records; extra causal-only payloads also count). This tradeoff is provisional until unchanged memory gates pass. Profile7 top15 is retained in CORE_PERF with all timing INVALID. No final shadow/freshheldout or new admitted timing claimed.


Combined candidate9 source d7b3a7510b6e497a9236c48416cf962ff439421d5c1c2b9a77b0a16c25d23bf1 composes only the disjoint row-order and guard true-prefix slices on candidate7. Core69/runtime128 tests and strict workspace/all-targets/mind_shadow clippy pass; independent integration review found no concrete blocker. Release build pre/post source matches all752 Rust inputs; perf5966fe3bcdaa3098bd2d3050d5efd7fd1c8b743f5200a85b8c235d21b90a2d9d, probe c425b58ed9527db133a2b8e958fe4189bab026b858b9cc22b6f99f53c3d80b62. No candidate8 standalone release/timing was run; composition and per-slice before/after hashes remain external. Candidate9 actual OFF/ON calibration has started, followed by admitted interleaved diagnostics after owned CPU jobs finish. This is exploratory source, not final acceptance.


## D31 — Preserve authored cadence opportunities in the continuation shortcut (2026-10-27)

Authority: planner P-3d permits the generic continuing-action optimization, while P-2 freezes all157 authored nodes and scores. A read-only integration audit found every frozen node has `every_ticks: 1`, but custom leisure/idle policies may author longer periods. Full evaluation at `tick ≡ agent (mod8)` and node admission at `tick ≡ agent + random_key (mod p)` intersect only when `random_key ≡ 0 (mod gcd(8,p))`. A stable owner can therefore starve a phase-disjoint custom opportunity indefinitely. The earlier universal seven-tick opportunity bound was too broad: for period1 it bounds the next reconsideration of a continuously eligible opportunity, not selection, and transient opportunities can be missed. Historical measurements remain unchanged.

Before final freeze, use a conservative full-selector fallback for policies with non-unit authored cadence. This leaves the frozen157-node policy and phase8 behavior unchanged, while preserving ordinary custom-node admission semantics. Implement/test in an isolated candidate, retain original oracle evidence and explicit differential coverage, then prove the usual frozen-policy summary equality. No cadence or score in mind.ron may be tuned. This entry records a required correction in progress, not a passing implementation or new held-out result.

Candidate9 actual OFF/ON48-day seeds0–7 calibration completed determinism and literal summary equality with candidate2 at14:50 KST, no excluded keys; receipt SHA256 1b8beb0395d2298751ef2c257eb9e6a7f16a9b43168f4333402059b289d400b7. Four reviewed paths integrated after complete candidate7 before-hash validation; all752 Rust inputs match d7b3a7510b6e497a9236c48416cf962ff439421d5c1c2b9a77b0a16c25d23bf1. Source integration does not implement D31 yet. All owned CPU-heavy jobs finished before p3d-diagnostic-pairs-9 began admission waits; D31 external code-only work does not launch a build or probe. No new timing accepted yet.


Candidate9 paired diagnostic validates8/8 members and zero actual sleep events. N1000 tick2941.245→3877.269µs (1.318241×), old/mind2054.814→2937.261µs (1.429454×); standing2220.46µs still fails. N2000 tick7348.247µs. Raw results, power window, post-session battery/swap observations and adjudication remain in p3d-diagnostic-pairs-9. Do not discard the admitted result because BASE is slower than prior sessions. Before increasing generic phase8, test behavior-preserving tracked release thin-LTO/codegen-units1 on an isolated copy containing D31 correction. No dependency, fast-math, target-CPU or frozen-policy change; final default production/gate builds must use the same committed profile if adopted. Plan p3d-release-lto-plan.json preceded the Cargo.toml edit; source tests/build/calibration/performance are pending.


Candidate11 implements D31 and tracked release thin-LTO/codegen-units1. Core73 and broad runtime/save mind137 tests pass; strict clippy and default release build pass. Initial clippy manual_is_multiple_of failure remains recorded; only the test predicate spelling changed. Actual seeds0–7 ×48days OFF/ON determinism passes and both summaries remain byte-identical to candidate2, excluded keys []. Preservation receipt a1abaedd3b63f5b314a0b5d74663c428e78cdd851a617b16e98fcf71f04565c1; terminal evidence p3d-eleventh-cpu-terminal.json. Exact seven-path integration checked every candidate9 before-hash and every candidate11 after-hash; Rust source1f58f5097313d886a4e250d4673b03612448349caea021d5d58ea1cc7e365008. No policy/phase change, final acceptance or held-out claim. Candidate11 interleaved timing begins only after all owned CPU validation jobs terminate, with each repeat requiring load1<8.


Candidate11 admitted ABBA diagnostic (p3d-diagnostic-pairs-11): all8 members valid, zero sleep/wake events; no final acceptance. N1000 BASE tick2986.977 / old2083.586µs; mind tick3664.286 / mind2796.072µs, tick ratio1.226754 and mind/old1.341952. N2000 BASE tick6471.868 / old4811.196µs; mind tick6928.530 / mind5367.841µs, tick ratio1.070561 and mind/old1.115698. C3 means are below5/10ms diagnostically, but N1000 standing2220.46µs still fails. Observed other work868.213µs leaves mind budget1352.247µs. Both report-only targets remain missed. Retain all raw/load/build/power evidence; subsequent sampling is attribution-only INVALID timing.


Profile11 (sample b0ac46c29eecc31da8ec9f1a246dddf2711f6b35433341fb0a3342582da964e9, denominator12625) records the exact top15 in CORE_PERF before the next edits. All profile timings are INVALID despite startload7.09. Remaining costs motivate separate preserving slices: exclusive full person refresh; lazy food index materialization; bounded two-peer social buckets; shared initial forage stock facts. Each uses an external source11 copy, a written plan and differential/reference tests, then composition and actual summary equality before integration. No speculative aggregate speedup is claimed. Cross-tick guard memoization is deferred: it would add persistent ephemeral per-agent state, exact trace-replay obligations and uncertain hit rates for at most14.9% of sampled mind cost. Phase8 and157 frozen nodes remain unchanged.


Candidate12 external composition changes exactly10 paths on source11, with all4 reviewed preserving slices. Ordinary patch context checks failed only because independent hunks shared context; retained raw three-way output resolves the appended test-module overlap by preserving both cfg(test) modules, and all5 stock files merge cleanly. p3d-twelve-composition.json and p3d-twelve-independent-review.md bind exact hashes/no unlisted source changes. Core73/runtime-save148 tests, formatting and strict workspace/all-targets/mind_shadow clippy pass without fixes. Default release and actual calibration are pending; source757 inputs digest75498d34d3d919b053b4c0ed817bfd74097e8cc11f6e66e0bf7ab9163a6eefaa. Source11 remains in checkout until actual result preservation is proven.


Read-only continuation contingency rejected blanket External admission: that class includes combat/flight, and a current same-node job does not authenticate assignment identity. A Job-only change would need typed capability, exact stage target/assignment evidence and new trace fixtures, and would delay ordinary need-band preemption; it is behavior-changing P3d rather than preservation. No such change was made. Continue with candidate12 before considering wider scheduling behavior.


Candidate12 release and both actual48-day calibration modes completed at15:42 KST with DETERMINISM PASS and literal equality to candidate2, excluded keys []. Preservation receipt SHA256 59676cf5bcde13c5682cef49241dc185cf35b5f4c80968c9b52612ce150250f5. Exact10 reviewed paths integrated after verifying all source11 before-hashes; all757 Rust inputs now match75498d34d3d919b053b4c0ed817bfd74097e8cc11f6e66e0bf7ab9163a6eefaa. Perf a56a6ff630cbe1b9c2841ace06c031799f0fc67a824e62aed35ab22a18d41850, probe c74cf57e3d697eac79e1b650477f8109678223b8788d2d7b6c4933d9fe0dc567. Admission-gated interleaved diagnostic12 starts after all owned CPU jobs terminate; no final acceptance or fresh-heldout claim.


Candidate12 admitted ABBA session p3d-diagnostic-pairs-12 completed all8valid members with zero sleep/wake events. N1000 BASE tick2869.937 / old2003.518µs; candidate tick3412.961 / mind2578.813µs; tick ratio1.189211, mind/old1.287143. N2000 BASE tick6391.907 / old4770.125µs; candidate tick6626.950 / mind5100.572µs; tick ratio1.036772, mind/old1.069274. C3 scene means remain diagnostically under5/10ms; N1000 standing2220.46µs still fails. Observed other834.148µs leaves mind1386.312µs. This is not final acceptance.


## D32 — Continue on battery without changing acceptance (2026-10-27)

Authority: owner reply to the power-state question: “현재 배터리 상태로 계속 측정하세요”. Continue measuring on the present battery state; do not change power settings or discard earlier admitted results. The strict per-repeat load1<8 admission, recorded1/5/15 loads, same-session interleaving, standing bench/C3 ceilings and no-justification rule remain unchanged. Battery observations do not waive a failing absolute or paired bar.

A read-only successful-path-cache feasibility scan of original source2 seed0 OFF20-day replay (SHA25606cb6045f1eda65dd5a5cfcfc2915072b738f192285b49cec3034c7db6592f29) found278570rows,215380guardscanswithreads,214291continued,181051consecutive same-owner candidates and163900exactread-sequence matches. These are historical potential hits, not N1000 coverage, actual optimized hits or performance measurements. Prepare a bounded preserving exact-value guard-path prototype with unchanged stateless guard/reference and exact trace validation; do not substitute predicate-truth caching or change phase/policy.


## D33 — Revalidate exact successful guard paths before skipping traversal (2026-10-27)

Authority: planner P-3b preserving optimization priority, continuing within the existing P-3d candidate. Candidate12 still misses performance; source-only analysis and historical trace feasibility support a bounded exact-value cache experiment. The written p3d-guard-cache-design.md precedes implementation. Re-run all runtime ownership/history checks and original core structural preflight, then re-read the previous successful unique fact sequence in its original order. Exact match permits the same ContinueOwned result; first changed value stops speculative replay and seeds only the observed prefix into the unchanged traversal, without duplicate provider reads. Errors/unsupported/boolean validation retain their original position. Cache entries are immutable-program/agent/entity-generation/node/stage bound, consecutive-tick only, at most32 facts, pruned and cold after load. No serialized state, policy/score/phase change, false witnesses or new ownership capability. Original stateless guard and frozen reference remain independent trace oracles.

The maximum cached fact vector alone is512bytes/entry (~1.024MB at2000), plus metadata/collection overhead; after-run memory must be reported explicitly. This experimental source is not adopted until actual composed tests, warm/cold trace equality, literal calibration and valid paired timing establish its value. Predicate-truth caching and Job/External continuation remain deferred.


### Candidate13 preserving implementation checkpoint (16:14 KST)

The exact-path cache source review found no blocking read-prefix/error-order defect, but correctly required retaining the old plan-mismatch fallback rather than rebuilding a plan silently. The implementation now clears derived cache identity without changing that fallback. Opt-in WORLDSIM_MIND_GUARD_CACHE_DIAGNOSTIC=1 records aggregate actual hits/misses and vector capacities at Drop; ordinary timing leaves it unset. Allocator/map overhead is not claimed to equal the requested Vec capacity. Tests/build/calibration/timing remain separate prerequisites.

A second bounded preserving slice is authorized by the written p3d-primary-refresh-analysis.md: only a primary, noninvited visit may retain tick-start Hunger/Thirst/Sleep/Social/Temperament/Inventory/Job/Carry; refresh Position/AgentState/Child/Elder/Drafted/Fleeing/Activity/ActivityRest and keep live routine overlays. Replay/invited visits retain the original full light refresh. Independent source review found no immediate peer writes to retained fields among registered callbacks; malformed noncaptured duplicate AgentIds require refreshing secondary-pass fields. Differential real callback/invitation/replay tests must establish the caller boundary before adoption. No policy, phase, score or serialized-state changes are authorized by this slice.


Candidate13 external composition adds a third preserving slice under p3d-rich-upgrade-analysis.md: after a successful light refresh and read-only guard, fetch only Cold/Sick/Mood/BodyHealth/Memory/Lineage/Activity (22→7). Activity must be restored from ECS before the second unchanged routine overlay, preserving Person stamps and overflow order. The original exclusive full-refresh bodies remain cfg(test) oracles; the shared adapter refresh is unchanged. Primary patch applies cleanly over cache+rich with no conflict. Independent combined source review found no concrete blocker; merged context/system hashes are recorded in p3d-thirteen-independent-review.md. Actual combined tests/build/calibration and performance remain required.


Composed candidate13 passes core79 and runtime/save mind160 tests plus strict workspace/all-targets/mind_shadow clippy (p3d-thirteen-r4-*). Initial formatting and test-only FactError equality compile failures remain retained; test comparison now matches exact Context error values without changing production traits. A subsequent clippy failure was stale shared-target check metadata: old relative dep-info and older copied-source mtimes let the primary-only core metadata omit the new cache exports. Preserved fingerprints are p3d-thirteen-stale-check-metadata. Touching only external core/src/lib.rs changed no source bytes and forced recompilation; verbose r4 clippy explicitly reports Dirty sim-core and freshly checks all five workspace crates. No cache deletion or acceptance exemption. Default release, literal calibration and actual cache observation are in progress; checkout still source12. Final production build must use a fresh isolated target to avoid cross-tree artifact reuse.


Candidate13 actual default release and OFF/ON seeds0–7×48-day determinism/literal preservation pass, excluded keys []. Source c576841c54c6dc7cd4e52af8200e5b2084b17c6881e23d425947db8a316336a6, perf122ca7337b7348509534f99aeecfcf842222e97dfa5334899b17438424abea99, probe f03be2693a491e7f4305296752136776a789f9685d8e58b5066f92dca0ad683b. Exact14-path integration verifies all source12 before-hashes and all763 source13 inputs; source receipt p3d-thirteen-integration-receipt.json. Actual N1000 opt-in observation records2271404hits/1913213misses,1000entries,88bytes/entry and256000vector-capacity bytes; map allocator overhead is not separately measured. Whole-process after-build/after-run allocations5254157/17391273bytes, RSS13616/39904KiB. Every observation timing is INVALID_DIAGNOSTIC and supplies no bar result.

Before paired session13, actual power is AC55% charging (p3d-thirteen-timing-launch.json), replacing the earlier battery observation without an agent power-setting change. D32 still forbids discarding admitted runs or changing bars. Do not attribute cross-session absolute timing differences solely to optimization; use interleaved same-session BASE ratios and unchanged standing ceilings. All owned CPU validations ended before session13 began admission. Finalcommit/shadow/freshheldout/gate/publication remain pending.


Candidate13 admitted AC-power ABBA diagnostic p3d-diagnostic-pairs-13 completes8/8valid members, two pairs per scene, zero Sleep/Wake/DarkWake events (33Assertions are not sleep events). N1000 BASE tick2170.767 / old1511.197µs; candidate tick2427.484 / mind1772.188µs, tick ratio1.118261 and mind/old1.172705. N2000 BASE tick4803.512 / old3558.847µs; candidate tick4851.080 / mind3616.211µs, ratios1.009903/1.016119. Diagnostic C3 means are below5/10ms; standing N1000 mean2220.46µs still FAILS by207.024µs. Current other655.296µs leaves mind1565.164µs, requiring a further11.68% mind-cost reduction at these conditions. Both report-only targets remain missed. Earlier battery results remain valid and retained; do not attribute their absolute difference to this optimization. Receipt02cb6f9cd96ade5871fcd7b05cad8a46d586f52543be94ea044cb4ab2dbb0e60, power window0b4e00e80aadf53f14be7f5c10a4eb1e4189109d490a880d0992fe0aac26046b. No final acceptance. Sampling13 follows only after the paired controller and power audit terminate.


## D34 — Remove work after exact current-evidence checks (2026-10-27)

Authority: planner P-3b preserving-optimization priority within the already authorized P-3d implementation. Candidate13 remains below C3 scene means but fails N1000 standing mean by207.024µs. Profile13 top15 is recorded in CORE_PERF before edits (denominator12111): mind8726inclusive, person5941, full selector1407, cached guard1055; the eight-component hecs preparation has495self samples. These are attribution shares, not predicted savings.

Two bounded preserving candidates are authorized in external trees. First, an exact guard-cache hit must not reset/write unused scratch; a first mismatch resets once and seeds only the freshly verified equal prefix plus current changed value, retaining read/error order and original scan. Explicit reused-scratch/interleaved-policy tests are required. Second, a first noninvited primary may acknowledge an untouched initial snapshot without querying ECS only if capture provenance proves its secondary observations belong to the same entity. Any secondary alias must be marked monotonically without repairing earlier snapshots; aliases/unverified rows keep the eight-query helper, invitations/replay keep the original16. The private proof must not become a generic freshness claim across world mutations or context reuse. Preserve Person invalidation/error order and one-shot admission. No policy, score or phase change; actual composed tests/calibration and paired timing remain prerequisites.

Candidate14 external integration: private row richness metadata becomes two named booleans in the same vector allocation (2 versus1 byte per row); production-only capture grants one-shot eligibility, any alias or row-mutator attempt revokes it. Independent review found no blocker, with the no-intervening-peer-ECS-write caller precondition explicit. Cache-hit reused-scratch precheck initially passed9/10: the test assumed unused schema facts survived compilation, but compiled slots include referenced facts only. The revised fixture adds zero-weight score references, retaining all assertions and unchanged production bytes; original failure is kept in p3d-fourteen-cache-precheck.log. Full combined validation remains pending at this entry.

Candidate14 is adopted into the working tree after actual core83/runtime-save168 tests, strict clippy, default release and OFF/ON0–7×48-day determinism plus literal candidate2 summary equality pass. Source766 inputs efbd661b5f1fe4dad998129f4c41961702973b2077cbc047315d4bdedf6cc224; perf eec67878050361f852cd7254d27803adda5464a191dc7a92c7bd4b5e61a2e04d; probe e7b39b15929c1403bf1e444fd2e34364975b5a4633685be666a6bc5e7d825860. Seven phase exit/log hashes were independently checked before exact10-path integration. The earlier failed cache fixture remains preserved. Guard observation hits2271404/misses1913213 matches source13; timings are INVALID_DIAGNOSTIC. New paired session14 starts only after all CPU validation ends; final acceptance remains pending.

Candidate14 admitted paired session (8/8 valid, zero sleep/wake): N1000 BASE tick2068.862326/old1444.794260us; candidate tick2149.799653/mind1548.769798us. Tick ratio1.039121659, mind/old1.071965636. N2000 BASE tick4591.946875/old3423.419344us; candidate tick4032.952083/mind2946.250254us, ratios0.878266276/0.860616231. Standing N1000 mean2018.6*1.10=2220.46us leaves1619.430145us mind at observed other601.029855us: diagnostic mean margin70.660347us. Both C3 mean time ceilings and same-session1.10 ratios are met diagnostically; full step25 mean/p99/memory and final step45 receipt remain mandatory. Report-only0.5/half-share targets remain missed. Receipt 099539afcfae6e990cb975da7064573a202f3d2e5149b4199592059ed4d85049; empty power-window SHA e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 means no events in the covered interval, not missing log coverage. No final acceptance.
