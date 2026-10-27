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
