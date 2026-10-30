# M6a decisions

## D1 — Authorized scope and clock (2026-10-29)

Authority: the owner’s unattended M6a goal, committed verbatim as
[M6A_GOAL.md](M6A_GOAL.md). This replaces the completed M-UX2.1 task;
its historical step-41 waiver does not extend to M6a.

Start: 2026-10-29 07:38:23 KST. Target: 2026-10-30 03:38:23 KST.
Hard stop: 2026-10-30 13:38:23 KST (30 hours). Evidence and the clock
register are outside the repository at `/Users/rexxa/work/m6a-data/20261029`.
Questions are resolved conservatively or recorded in the final report,
not sent to the unattended owner. No scope has been dropped.

BASE is `5b0c88812374b87e7854cc9203f144ad0ccd9009`, obtained after the
required fetch and exact remote-report existence check. The registered
worktree is `/Users/rexxa/work/new-world-m6a-dev`, branch `v8/m6a`.
No other worktree is a build, edit or runtime surface for this pass.

## D2 — Documentation and design precede product code

`a3106c9d727ef10dd6f29cb2a5fe5d493e582565` records the immutable goal.
`cb3f79cc69df10ee05b10d5a5330e0226f67fc68` adopts roadmap v2, preserves
complete v1, and records the coordination rules. The documentation check
passed. The generated system-table check invokes Cargo; it was allowed to
finish before the separate census build. Builds and captures are serialized.

The Part 1 census is a zero-tick design instrument, not a calibration round.
The two independently executed runs cover seeds 0–7, eight 192×192 maps,
294,912 unique tiles and ten founders per world. Both exited 0 and had
complete terminal records; their bytes are identical (SHA-256
`6d52f6351fdc99d6d6d6bd4843ca092b1b4bb62a478f39ad276e28cefe77c322`).
Source, dependency locks and executable hashes matched before and after.
The census records ordinary final worldgen terrain, blockers and reachability;
it does not claim new plants exist in the product yet. The proposed placement
model will be checked against actual runtime placement after implementation.

## D3 — Acceptance and provenance

Calibration remains seeds 0–7, at most four logged 72-day cohort rounds.
No cohort round or held-out run has started. Held-out seeds 1296–1327 stay
unseen until policy/data and observer definitions are frozen. Failed and
partial runs remain evidence. Physical supply arithmetic does not establish
a probability of survival: the owner’s success-count bars remain acceptance
requirements, with the actual denominators and observation windows explicit.

Only the current full gate plus new food tests, mandatory baseline identity,
food acceptance and clean-load performance evidence can authorize publication.
No inherited M-CORE performance exception or M-UX2 stall waiver applies.
The UI-successor source-identity adapter cannot certify simulation changes;
a separate explicit simulation-successor validation contract is required,
with original adapters and failures preserved.

## D4 — Pre-code arithmetic revisions and migration review

The first placement model is preserved: seed5 has thirteen reachable roots,
three units per harvest and three spring harvests,117 units against144 nominal
warm adult demand. Before any cohort calibration, root yield becomes five
(195 spring units), stockpile lifetime ten days, and cooking produces whole
one-unit portions. Roots2->3 portions and fruit/grain cooking provide authored
plant-based cooked paths. Full harvesting wastes food, so design targets
usable demand and a real144-unit winter reserve rather than all available yield.

Independent checks verified32 storage scenario balances,2,304 targeted daily
rows, all seasonal production formulas, and the1235 harvested /35 spoiled root
model at stockpile lifetime10. Its44,440 work ticks exclude hauling and finite
execution; it is not runtime or diversity acceptance. The no-gain823-unit gross
bound does not pretend cooking's conversion gain is raw harvest.

The pre-code review checked all nine artifact hashes and required explicit
old physical-stack age migration. The design now preserves known oldest ages
and records conservative expiry of unmatched unknown-age food. No blanket
old-colony save rejection is allowed: existing format11/12 continuation tests
must pass. Actual implementation and full-gate proof remain pending.

## D5 — Initial implementation policy, before calibration

No calibration or held-out run has started. Cooking and grain-parching
knowledge seed two founders each. Raw meat sickness is100permille; drying
uses60 work ticks followed by two calendar days without holding a worker.
The separate food drying rack preserves the historical hide rack and all
existing shortcut slots; its build-menu entry has no keyboard shortcut.
Diet thoughts use +60 for at least three actual ingredient groups and -40
for exactly one group in the last three days. These two code-table mood
variants are explicit M7 content-data debt; the meal history remains data-driven.

Food choice exposes factual capabilities `food_cooked_available`,
`food_novel_available`, and `food_cooked_novel_available`, plus corresponding
physical-owner target bindings. Authored node offsets +3/+2/+1 choose cooked
then unseen-group food; within each capability, distance precedes the
data-defined item preference tie-breaker. Abstract colony forage/gather
production nodes are removed; the existing work-priority array is preserved
and harvest is a Haul-priority job. Baseline policy and enum encodings remain
separate requirements, not inferred from these changes.

Food conservation includes explicit `lost_units` for an item that cannot be
placed when its carrier dies on a completely full map. It is neither a meal
nor spoilage. Cancellation retains the carrier's actual item and age instead.
Drying consumes inputs at work completion and creates outputs only when due;
pending output is not physical stock, but counts against a Maintain bill.
Pots are real vessels counted once as held items, with contents counted once
as stored items. Common settlement observers assign public stores to the
nearest reachable formation tile, breaking distance ties by settlement ID,
so multiple settlements cannot double-count the same food.

Initial mechanism verification is recorded externally in
`/Users/rexxa/work/m6a-data/20261029`: content5/5, migration4/4,
save-continuation2/2, storage14/14, plants4/4, legacy-supply isolation3/3,
and food-job transitions5/5 passed their named test runs. These are bounded
mechanism results, not survival, performance, visual or full-gate acceptance.
Preserved failures include duplicate build shortcut, RON fixed-array parsing,
a missing settlement in a migration fixture, and a fixture that expected a
fractional seasonal regeneration pulse to immediately produce a whole unit.

### Benchmark food gift representation (M6a integration decision, 2026-10-29)

The standing benchmark, navigation benchmark and fed stress runner retain the
published BASE gift schedule: after the first settlement-bearing tick, then at
06:00, supply three nominal food units per living person to the first settlement.
Natural and raw stress scenes receive no gifts. BASE binaries and baseline-mode
counter behavior are unchanged. The candidate colony helper converts this
external benchmark input to real `legacy_food` pieces: one unit per piece,
raw edible, uncooked, no ingredient groups and no sickness bonus or penalty.
A piece supplies the same nominal 30 hunger relief as one BASE reserve unit;
actual relief still uses the ordinary hunger clamp.

Candidate gifts use atomic physical placement around the settlement formation
tile and the normal item ledger with explicit initial-supply accounting
(`FoodSource::Debug`). They do not claim harvest, migration, cooking or prior
meal history. Placement or content-contract failure rejects the run instead of
silently dropping supply. Gift creation occurs after the timed tick, only at the
existing gift cadence; creation is included in overall wall time but not in the
reported tick mean/p99. Subsequent ordinary simulation work on that food is
included in the tick timings.

This is equivalence of supplied nominal units and cadence, not a claim of equal
consumed satiety or equal outcomes. Physical reach, hauling and the authored
one-day `legacy_food` expiry remain intended M6a behavior; BASE's abstract
reserve is not retroactively made physical or perishable. No BASE instrumentation
or data change is introduced. This decision is input provenance only and does
not establish a performance pass; fresh admitted paired measurements and the
standing ratios/C3 remain mandatory.

## D6 — Integration failures before calibration (2026-10-29)

The fresh-process apple fixture twice placed and ripened its data-added plant
but never assigned harvest during six days. Retained reports are
`/var/folders/0w/j63n8j5s46j4dl79hrwn0m2w0000gn/T/worldsim-m6a-apple-13251`
and `worldsim-m6a-apple-34543` in the same parent directory. The latter records
all ten people above hunger51 at tick961 with reachable ripe harvest candidates
and no job. The generic assignment veto treated hunger as actionable even when
no edible physical food existed, contradicting the authored mind predicate.
Correct the factual admission seam, preserve thirst/sleep/combat checks and
re-run the unchanged actual food-chain assertions; do not inject bootstrap food
or move the hunger threshold to make the fixture pass.

The existing mind-owner tests also exposed stale abstract-food fixture inputs
and a real one-piece completion regression. Preserve their job ownership and
choice completion assertions while using physical food; one consumed piece
finishes the action and the next authored choice decides whether to eat again.

Bounded verification so far: integration39/39 (`m6a-rust-test-4.log`), content7/7
(`m6a-content-test-5.log`), neutral benchmark gifts3/3
(`m6a-perf-gift-tests-1.log`), Python evidence tooling28/28
(`m6a-python-test-3.log`), and headless smoke with zero errors. These are not
final-commit acceptance. The system-food suite retains35PASS/3FAIL pending the
owner-fixture repair. Interrupted integration compiles against in-flight files
are retained as failures, not reported as successful runs. No calibration
round, held-out evaluation or candidate performance measurement has started.

### Pre-calibration dependency correctness repairs (2026-10-29)

Read-only review found three acquisition cycles before calibration: the food
planner required a pot already on a cooking shelf before creating the bill that
would fetch it; vessel recipe input planning waited for recipe knowledge whose
discovery required clay stock; one leftover inedible grain piece satisfied the
stock's group-presence check while remaining below a recipe's input count.

Planning now admits reachable supplyable vessels while actual crafting still
requires a local vessel. One ground vessel per cooking workshop is retained for
supply; after arrival, spare pots remain deployable as food storage. Optional
pot alternatives follow the recipe's knowledge conditions. Discovery-material
demand is read from the registered recipe knowledge prerequisites and their
Stockpile conditions, using ordinary knowledge-gated node designations. Clay
banks receive DesignateClay, not the ineffective quarry command. No knowledge
is granted by planning. Ingredient harvest demand includes one missing batch
of a currently usable recipe, and subtracts already pending harvests; raw grain
continues to count as zero edible reserve units.

These are pre-calibration dependency repairs. No production constants, yields,
spoilage lifetimes, owner arithmetic, acceptance thresholds or policy horizons
were tuned, and no calibration round or performance pass is claimed here.

### Shared supply observers and custom provisioned colonies (2026-10-29)

Colony seasonal supply now means ideal raw-edible harvest capacity from actual
reachable planted species in the requested settlement catchment. Authored ripe
windows, whole yields and regrowth intervals determine opportunities; continuous
ripe windows do not get a free yield at each season boundary. Grain requiring
cooking contributes zero to this conservative raw capacity, and capacity is
neither physical stock nor a promise of meals or survival. Legacy seasonal
regeneration multipliers are not applied a second time. The four authored phase
slots are projected against the configured calendar; absent extra season slots
have zero plant capacity and annual means use the configured season count.

New capacity arithmetic uses integer whole units. Shared catchments divide at
one-millionth-unit precision, rounding each plant's share down conservatively;
only the existing food-economy f64 API projects the cached integer totals to
units/day. A serde-skipped default-empty resource cache keys navigation version
and readiness, settlement position/radius, calendar, active sharing geometry
and plant count. Plant locations/species and the process-selected content
registry are immutable after worldgen/load; daily yields and food ownership
changes do not invalidate theoretical capacity. Loads start with an empty
cache. A new runtime plant-replacement mechanic must explicitly invalidate or
extend this key; equal-count species replacement is not an existing operation.

Custom colony Provisioned birth rules observe fresh reachable stored edible
items for their reserve requirement. Their effective synthetic food tax is
zero, consistent with the earlier decision retiring colony birth withdrawal;
actual newborn hunger remains. An abstract Food counter cannot satisfy a
positive reserve gate. Baseline source arithmetic, gates and withdrawal remain
on their original branch. These are observer/inventory correctness repairs,
not policy calibration or acceptance results.

### Integration verification and real-renderer plant anchors (2026-10-29)

The pre-commit integration runs preserve failures separately from reruns in
the external `20261029` evidence directory. Legacy meat-age and hunger-job
fixtures now create actual accounted food with age metadata. They retain the
original conservation, cancellation, reservation and save/replay assertions;
an unowned synthetic Food counter no longer makes a colony meal actionable.
The supply-cache fixture separately checks a blocked plant tile that remains
harvestable from a neighbour and a fully inaccessible plant. A single blocked
goal is not an inaccessible harvest under the existing navigation contract.

The initial plant-art structural run reported 47 checks with four native-anchor
failures under Godot's headless dummy renderer. That backend does not retain
MultiMesh transforms. The tests keep the native-size/anchor assertions and add
actual/expected transform diagnostics; step 48 now uses the real OpenGL renderer
in a bounded windowed process. Structural execution does not by itself claim
visual acceptance. The original failed receipt remains evidence. These runs
are mechanism checks, not calibration, held-out results or performance bars.

The windowed plant-art rerun passed all 51 checks including capture creation.
The first real-scene food capture separately failed workshop inspection entry
and stopped before winter/pot coverage. Meal and ground-food explanations at
UI scales 1.0/1.25 were readable in two independent image reviews; plant-phase
screenshots did not identify their target clearly enough. The capture harness
now records an unoccluded real workshop click point and plant viewport bounds
with raw image crops. Missing scenes remain unverified until recaptured; the
original failed run is retained. A duplicated item quantity line was removed
when the same physical food getter already supplies that row.

### Pre-calibration boundary corrections (2026-10-29)

A completed hunt cannot resurrect its already-killed animal when no ground
space admits the meat. Its actual yielded batch is therefore booked as harvest
followed by explicit loss, with no phantom owner or age. `Hunted.value` reports
retained meat and is zero for this failure; ordinary successful placement and
baseline operations remain unchanged. Packed-map and ordinary-kill assertions
cover conservation and repeated-completion idempotence.

The shared colony daily-demand projection now includes the currently applied
Cold hunger multiplier, matching the existing hunger-decay operation. Warm
children still use their own growth rate and baseline arithmetic is preserved.
This corrects stock targets and custom provisioned-birth reserve admission;
it does not change the frozen warm-adult winter acceptance formula or tune
steward policy. Regression tests and final identity remain required evidence.

### Whole-workspace migration failures before calibration (2026-10-29)

`workspace-test-4.log` preserves the first complete debug integration attempt
with the current food chain. Its failures are not calibration results. Three
catalog fixtures still expected the old knowledge, steward-rule and recipe
sets; their explicit expectations are extended for the authored additions,
while retaining the old entries and original behavioral assertions.

The no-orders colony viability and mood hand-over failures exposed a real
missing route: colony food counters are retired, but a person with the steward
off could neither eat a plant directly nor start an undesignated harvest job.
The existing no-orders survival bars remain binding. The repair must use an
authored needs decision and ordinary reserved harvest work that creates actual
items; free food, relaxed survival assertions and implicit steward enablement
are rejected. Trading and housing failures remain unresolved until reruns show
whether they share this starvation cause.

The replay writer emitted `designate_harvest` and `deploy_food_pot`, but the
reader omitted their concrete forms. Both player and steward-source roundtrip
checks now cover these commands, including rejection of missing coordinates.
The drought test also revealed that daily physical plant regrowth had lost its
pressure input. Its replacement must test real plant yields under a real
pressure, retaining the old baseline resource-regeneration protection rather
than restoring a colony abstract-food authority.

Original BASE build provenance was recovered from contemporaneous terminal
records: release build session 36751 exited 0, and the subsequent frozen copies
recorded docs-only source `cb3f79cc69df10ee05b10d5a5330e0226f67fc68` with an empty
runtime delta from BASE. External `base-build-provenance-recovery.md` records
exact calls and hashes; `base-build-receipt.json` binds the original binaries
and unchanged log. This is recovered provenance, not a newly executed build or
an acceptance result. No compiler/environment history was invented.

The initial build-validator attempt also exposed an impossible lockfile
assumption: `rust/Cargo.lock` is deliberately ignored and has no committed
blob. Build evidence now requires an external immutable lock-file reference,
its exact recorded hash and, for the candidate, equality to the live build
lock. Both gate and paired-performance validation reject missing, tampered or
worktree-local lock evidence. Original BASE bytes remain preserved. This fixes
the provenance contract without removing its dependency-lock check.

The physical drought clock now counts completed growth intervals at boundaries
`start_day + 1` through `end_day`, inclusive. This matches FoodDaily priority
119 preceding Storyteller priority 170, without changing either system order
or activating a planned pressure early. With the existing 400 permille rate,
five completed active intervals earn two growth days; the other three postpone
a finite depleted recurring-food deadline. First seasonal yields, standing
units, nonfood plants and non-regrowing sentinels are unchanged. Actual
scheduled-pressure tests cover clock offsets 0 and 480, and a save/load test
covers delayed growth continuation. The focused plant suite passed 8/8 and
the story suite 11/11; these are pre-commit mechanism evidence.

The pre-fix ordinary seed-3 colony probe (steward off, no commands, six days)
recorded ten starvation deaths and zero survivors despite 6,190 ripe units at
the deaths. After the autonomous route was added, the previously failing mood
morning hand-over/save test and all sixteen neighbour/trade tests passed.
A separate original breakdown test caught the new route bypassing work
admission. Both the authored node and the execution boundary now reject that
state; priority-zero hauling, active ownership, flight/draft and combat are
also rechecked. Rerunning those guards and the original survival bars remains
required before the behavior commit.


### Authored hunger acquisition while ordinary work is owned (2026-10-29)

`housing-diagnostic-3.log` confirms the remaining acquisition starvation:
by tick 740 an adult had hunger 80 while chopping, with 1,232 ripe raw-edible
plants, an eligible reachable plant within seven tiles and 13 Harvest marks,
but no edible owned food and zero harvested units. At the first starvation
death the person still owned work at hunger 100. This is a work/needs admission
failure, not evidence for changing yields, hunger thresholds or survival bars.

`need_job_harvest_food` is an authored need-band node using the existing
hunger threshold and acquisition score. The new Bool `job_harvesting` observes
the current Job kind and excludes an already active Harvest from replacement.
The node requires an adult outside breakdown/flight/draft, owned ordinary work,
no reachable edible owner and an eligible physical plant. `harvest_food` now
handles generic Job adoption by entering the same validated start transaction;
it only continues an already actual Harvest unchanged. JobAssign policy and
unrelated drinking policy are unchanged.

The transaction validates live plant/reservation, navigation, knowledge and
work admission first. Any carried food or nonfood uses the existing atomic
ground-drop planner; food retains its born day, or already expired food is
explicitly spoiled. Failed placement leaves the old Job, load, age,
reservation and seek destination intact. A successful drop permits normal
Needs cancellation, preserving the original work designation, then reserves
and starts the real Harvest. Deferred mind interruption cleanup cannot cancel
the replacement job. Player cancellation/veto, priority-zero Haul and breakdown
admission remain enforced. No food is created until ordinary harvest work
finishes. This predicts positive actual harvesting in the formerly work-bound
housing fixture; it is not a passing runtime claim.

Focused tests were written before this fix for ordinary-work transfer exactly
once, existing Harvest continuation, original designation retention, preserved
carried-grain age and conservation, and a packed map retaining the old load and
job. Root owns execution of those tests and the unchanged integration bars.

The pregnancy serialization diagnostic retained exit 101 in
`pregnancy-diagnostic-1.log`: the first mother (4) conceived at tick 8,880,
was due at 34,800, and died of starvation at 27,873. All inhabitants were dead
by 28,121 (six starvation, four combat deaths), before the due-birth checkpoint.
This failure preceded save restoration. The serialization fixture now enables
ordinary steward winter planning identically in all four engines. Natural
pregnancy and queued-birth predicates, 40/60-day bounds, birth-count assertions
and post-load full-state twins remain unchanged. No pregnancies, food or births
are injected; the rerun is pending.

The separate scripted seed-5 diagnostic found one dehydration death (person 3,
tick 2,022), not starvation: three commands were accepted, the hut completed at
780, and all fifteen trees at 700. The focused unchanged-product reproduction
also failed at that death. Its trace shows the person stationary at `(97,106)`
while seeking water, without a job or carried item. This is tracked separately
from housing work starvation; no survival threshold or test is relaxed.

The second seed-5 trace rejects the food-priority hypothesis. At tick 781 the
person is Idle with the completed Build job and its construction SeekTarget;
at 782 the job is gone and the FSM is Seeking Water, but the same non-water
tile remains attached. The isolated physical-food-to-water handoff test passes.
The repair validates the final colony water/sleep destination against its actual
resource and assigned bed, applies a valid newly selected destination, and
retargets a stale construction destination. It does not change baseline target
handling or the food selection score. The unchanged three-day survival bar and
an actual-water-target assertion are now in the focused regression; the full
integration rerun is pending.

The first work-transfer integration run retained three failing targets: housing,
no-orders survival (seed 0: ten deaths), and pregnancy setup (ten starvation
deaths before any pregnancy). Its eight-seed scripted-world invariants and the
two water-handoff tests passed. The housing trace exposed a second acquisition
boundary: a Harvest job existed, but its deferred destination was discarded by
the job-owned postpass. Ordinary job movement only tracks progress; it does not
reconstruct that missing target. Harvest now installs its owned Job and
SeekTarget together, as ordinary assignment does. A new remote-plant regression
uses actual movement, mind and job execution until conserved food is produced;
no post-start relocation is used. The focused harvest/cache suite passed 12/12
in `autonomous-harvest-test-6.log`. Survival integration remains pending.

The corrected owned destination now passes both unchanged housing fixtures
(`housing-repair-test-1.log`: 2/2). The flat and terrain villages complete
roofed assigned bedrooms and retain ten assigned beds. This is focused
integration evidence; no-orders survival, natural pregnancy/save roundtrips,
and the complete workspace are still being rechecked together.

Clippy exposed only mechanical follow-ups: three redundant u64 casts, a
Copy value cloned in a test, and a production collector placed after its test
module. Removing the redundant operations and moving the unchanged collector
before the tests passed all-target workspace Clippy (`clippy-7.log`). This is
not baseline identity evidence; the final identity run remains mandatory.

### Pre-calibration evidence review (2026-10-29)

No calibration rounds or held-out worlds had run when independent review found
two evidence-consistency defects. Cohorts inherited selected-content/diagnostic
environment overrides without binding those external inputs. They now reject
such overrides before creating output or consuming a calibration round, and
explicitly pass the admitted environment to the probe. The two legacy save-test
locator variables are removed from the child environment; they cannot affect
simulation inputs. The source SHA label remains available for provenance.

Spoilage scoring previously used the last food-flow row before the end of the
window, without proving that row was the latest or agreed with daily ledgers.
The authoritative endpoint is now the already required pre-expiry day-72
snapshot at tick 103,200, matching the frozen `[0,72)` window. Flow ordering,
boundary tallies and final tallies must agree. Thresholds and metric windows are
unchanged. Corrupted, omitted and reordered evidence cannot improve the score.

Calibration has an explicit eight-world observation command using the same
per-world definitions, reporting `OBSERVED` and `NOT_EVALUATED` for acceptance.
It does not relabel calibration receipts as held-out evidence or scale down the
32-world acceptance bars. All M6a Python tooling tests pass (39 tests in
`m6a-python-test-10.log`); this is tooling evidence, not a cohort result.

A separate product review found stored raw ingredients could become stranded:
Haul puts food into actual pots and steward counts it, but ordinary cooking
Supply only searches ground stacks. Pot-only grain therefore stops replacement
harvest demand without becoming runnable cooking input. Regression fixtures
are prepared; the pre-fix runtime and product correction are pending. No food
quantities or acceptance bars were changed to compensate.

The whole-workspace rerun passes no-orders ten-day survival, the scripted
8-seed world invariants, and both housing fixtures. Its pregnancy serialization
fixture still fails before restoration: mother 5 conceived at 20,400 (first
observed 20,401), was due at 46,320, and died in combat at 40,764. Five other
people starved during the first winter at ticks 34,322–34,576. At tick 86,400
four people remain, but no birth was queued. The six other save/load tests
pass. Preserve `workspace-test-5.log`; this is neither a save-state mismatch nor
proof that the pending pot-supply correction resolves winter survival.

The pot-only cooking regression reproduces the defect on unchanged product:
`pot-supply-regression-before.log` exits 101 with available grain 0 instead of
2; both atomic failure cases pass. The corrected product passes all three in
`pot-supply-regression-after-1.log`: an ordinary reserved worker walks to the
pot, carries the original-age grain, supplies the shelf and cooks exactly once.
Expired-on-carry and full-carry rejection preserve the full food state, job,
reservation and mutation versions. Bills count unreserved portable pot pieces;
assignment and execution use lazy spatial/tile indexes with live content checks.
Ground ingredients retain precedence. No Job/save schema or content tuning was
needed. Existing age semantics still require the batch to survive its current
pot class and the ground-lifetime carrying leg; storage does not refresh birth.
This focused pass does not resolve the open natural-pregnancy survival result.

The release pregnancy rerun after the pot-supply fix still fails with the exact
same conception/death timeline (`pregnancy-post-pot-release-1.log`). The fix
addresses a separate real cooking defect, not this winter failure. A seed-1
30-day observational probe (`seed1-winter-diagnostic-1`) reports zero invariant
violations but zero stored food on every daily snapshot: day 18 has 367 physical
units, and day 21 has only 41 after substantial ground spoilage. Cooking did
occur (152 cooked meals in the final tally). Before changing policy or yields,
inspect stockpile occupancy and actual worker assignments to distinguish full
storage from harvest jobs starving haul work. This diagnostic does not consume
or replace a calibration round; calibration remains 0/4.

Independent tile-level observation confirms the capacity obstruction:
`seed1-storage-diagnostic-1` has 16/16 cells occupied by firewood on day 10;
day 18 has 14 firewood stacks, one hide stack and one log stack. Every loose
food stack has zero compatible unreserved destination cells (41 stacks on day
10, 42 on days 14 and 18). Harvest designations are zero at these boundaries;
the sampled midnight jobs cannot establish daytime scheduling shares. The
existing stockpile planner stops permanently after creating its first zone,
while FoodPlan requests food but never storage capacity. Add a focused capacity
regression before correcting this missing planning step. Preserve physical
harvest → haul → storage: new capacity must use empty reachable cells, not
relabel food already lying at its harvest location as stored. Existing ordinary
stockpile commands, player vetoes and Food scope remain authoritative.

The capacity regression passes after the ordinary empty-cell extension. The
same seed now physically stores roots, grain, cooked food and dried food:
day 18 has 65 stockpile cells and 155 edible ground-stack units in those cells.
The exact saved-state classifier also counts one workshop-shelf root at a
zoned work tile, making runtime stored units 156. However, all ten
people subsequently starve (ticks 32,050–33,373); the release save suite remains
6 PASS / 1 FAIL. Capacity was a real defect, not a sufficient survival repair.
The post-fix 156 stored plus 8 other edible units exceed warm winter demand
144 by only 20, while another 99 units spoil before day 25. Preserve both runs.
Winter eligibility currently counts freshness only at winter entry, and the
harvest-only `winter_keep_days` parameter is not used by reserve accounting.
Formal calibration is still 0/4: record the initial policy in round 1 before
adopting any proposed winter-horizon policy/data change in a later logged round.

Review also found physical storage allocation wrongly required a raw-edible
ground-food bucket. The grain-only regression fails with zero cells instead of
one; separating physical ingredient slots from optional edible reserve slots
passes all five capacity tests (`storage-capacity-regression-after-2.log`).
Raw grain remains inedible and receives no invented satiety credit.

The real windowed UI capture exposed a separate presentation defect: explicit
new food kinds in `chronicle_feed.json` were still rejected by the legacy
`NOTABLE` list. Explicit data entries now admit supported kinds, while legacy
defaults and the data-defined mind-choice exclusion remain intact. Detached
feed tests pass; the full capture must still be repeated. A reviewer initially
misread the 1.25-scale plant crops as empty terrain. Individual image inspection
and identical capture-3/4 crop hashes refute that diagnosis: the plants are
present. Preserve this correction in the external visual review, not an invalid
crop classification. The added per-scale target recomputation and pixel checks
strengthen evidence collection; they do not demonstrate a prior crop defect.
Capture 3 still has 15 recorded failures, including the real chronicle omission.


### Initial candidate verification, 2026-10-29 06:33 UTC

`candidate-identity-2.log` exits 0: 228,748 shared simulation values with zero
differences or one-sided paths, 64 chronicle files (19,683 entries) equal,
64 raw sample files equal, and seed 0–3 determinism PASS. This checks the
working implementation before its first product commit, not final acceptance.

Capture 4 retains four failures and incomplete coverage. Its phase-0 stem is
visible: source RGB (119,116,97) multiplied by the observed night canvas tint
(.473,.517,.528) predicts (56.287,59.972,51.216), matching screen (56,60,51).
The capture oracle now applies the observed tint with unchanged sample-count,
match-fraction and tolerance thresholds. It must be rerun; missing why/pot
captures remain missing. Two independent visual reviews confirm the existing
food text fits both scales. They found an incorrect Korean object particle and
a duplicate eater name; reuse the existing generic particle formatter and mark
the cooked-meal sentence as already named. Locale compilation passes; runtime
text regression and a fresh full capture remain pending at this record.


Capture 5 exits 1 with nine failures: the phase-3 pixel check failed and eight
later scenarios were not reached. Source stem RGB (119,112,79) under global
night tint predicts (56,58,42), but the eleven actual samples range from
(106,101,71) to (108,103,72), brightening toward the visible hearth. The product
uses additive local hearth lights; global tint alone is not a complete oracle.
Exact light IDs were not recorded in capture 5. Admit future phase colour
proofs only in natural full daylight with no enabled positive-energy local
lights, recording that admission and retaining every original pixel threshold.
Do not alter the simulation clock or product lighting. Missing cases remain
missing; a subsequent capture is required. Strict scripts/harness checks and
`test_locale_trf.gd` passed before capture 5, as did full docs check 7.


### Formal calibration round 1, source dba478e54

All eight seed 0–7 worlds completed the prescribed 72 days with runner exit 0;
the append-only register records one of four rounds consumed. The source and
frozen release probe remained unchanged. `calibration-1-score.json` is OBSERVED,
NOT_EVALUATED for acceptance: alive 0/8, winter stores/cooked share/diversity
0/8, spoilage 2/8, ledger 8/8, day-1 storage/day-10 beds 8/8, wolf-death bar
6/8. No held-out worlds were evaluated. Seed 7's ten deaths are dehydration;
seed 6 has one starvation death followed by nine dehydration deaths. These
must not be attributed to food storage without mechanism evidence.

The next-round investigation retains three separate hypotheses: winter
eligibility ignores the intended keep horizon; one-day cooking targets also
cap long-life preservation; daytime fire is unavailable to recipes requiring
a lit hearth. Existing fire only burns in the night window, while planning
suspends unavailable recipes. Any correction must retain actual fuel withdrawal,
actual craft requirements, physical haul and original food ages, with baseline
identity and focused regressions before a separate logged second round.

### Round-2 mechanism verification before freeze

ADR 0053 records the separate behavior-changing candidate. The ten focused
test groups in `round2-focused-after1-terminal.json` all returned zero, including
food owner retention/invalidation, horizon and preservation, real fuel-backed
heat, bills/storage, save round trips and food fingerprints. Initial failing
tests and corrected fixture preconditions remain in the external evidence.

The observation-only natural seed-7 six-day run (`water-diagnostic-2.jsonl`,
terminal 0) ends at tick 8640 with ten living people and zero deaths. The prior
same-scenario diagnostic had ten deaths; doorway food-target oscillation is no
longer observed. Other round-2 mechanisms are present in this run, so this is
not an isolated attribution of every changed trajectory to retention. No food,
water, knowledge or navigation state was injected. The temporary observer was
moved outside the repository after execution. Formal calibration remains 1/4;
held-out, performance and full gate are still not evaluated.

Independent review also found preservation budgeting accepting a second rack
while bill application selected only the first, player-paused rack. A focused
ordinary-command regression must prove selection of the eligible second rack
without editing the first player's bill before freezing round 2.

That regression failed with an empty command list, then passed after the budget
retained the eligible workshop ID for application. A separate all-paused test
keeps the player veto from being bypassed by constructing another rack. Full
systems unit tests pass 470/470 (`systems-round2-test-1.log`); all-target Clippy
passes (`clippy-round2-3.log`). The first two Clippy attempts failed in the
temporary capture fixture and remain recorded. Retained carried-food eligibility
also uses the known entity and one physical stack read instead of two actor
scans; existing retention and invalidation regressions remain green.

### Authentic pot capture fixture and focused evidence

`pot-fixture-1` is a separate seed-0 ordinary-player scenario, never a calibration
world: stockpile, quarry, workshop, blade bill, an actual caravan trade and pot
deployment are recorded commands. At tick 31980 the actual trade exchanged
17 units of value for a quality-45 pot valued at 9. At tick 47700, pot 1 at
(88,99) held six grain and five roots; food validation passed and the save was
written. Root checked every captured source hash unchanged. The temporary
observer was moved outside the repository after terminal exit 0.

`ui-pot-round2-1` completes only focused pot coverage with zero failures and
both scales, not the whole capture suite. Root and an independent reviewer see
the items and remaining pot-storage days; at 1.25 the final paragraph needs
scrolling. The harness now requires actual wheel input and a fully enclosed
final label before a supplementary bottom capture. A brief capture delay ended
after bringing the owned Godot window to the foreground; the process sample and
original logs are retained, without claiming a product stall repair.

### Round2 focused UI follow-through and baseline identity

The third baseline identity run completed with exit0:228,748 shared values,0 differences/one-sided paths;64 chronicle files/19,683 entries and64 raw sample files equal under the identity tool's declared normalization; determinism seeds0–3 PASS. This binds the uncommitted round2 runtime used by that run, not a future full gate or colony survival acceptance. Evidence: `/Users/rexxa/work/m6a-data/20261029/candidate-identity-3.log`.

The actual pot follow-up `ui-pot-round2-2` completed with0 failures. At1.0 the explanation was already visible; at1.25 real mouse-wheel input moved scroll0→37 and enclosed the entire final paragraph. Root inspected `pot-food-why-bottom-1.25.png`: storage class/remaining days and the age-preservation explanation are readable, and the source inspector remains separate. Focused completion is not full-suite completion; the receipt explicitly retains `full_suite_complete=false`.

The first hunger capture fixture failed honestly: seed0 with one ordinary `SetSteward { on:false, scope:All }` command produced no HungerFoodChain through display day12. The entire steward was disabled; this is UI-only evidence, not a calibration world, not a steward-ON survival result, and not evidence that defence stayed active. Preserve `hunger-fixture-1/receipt.json` and its failed log; no event/state injection or successful coverage is claimed.

The previously failing natural pregnancy/due-birth save fixture now passes on round2 runtime: release test `save_load_roundtrip_through_a_pregnancy_and_a_due_birth`,1 passed/0 failed,26.46s after compilation. Its ordinary steward command,40/60-day bounds and save-continuity assertions are unchanged from p3. Preserve prior failures. This is evidence for that fixture, not72-day calibration acceptance (`pregnancy-round2-1.log`).

### Calibration2 and valid early performance failure

Candidate `de25dd74ed894365b4a59318fa4318056cb71b6d` completed the second registered8-seed×72-day calibration. Counts: alive8/8, no hunger deaths in winters2/3 8/8, diversity8/8, spoilage8/8, ledger8/8 and common stockpile/bed/wolf rows8/8; cooked share7/8 and both-winter stores7/8. Seed3 cooked563/1208=46.606%; seed5 winter66 stored104/130, with56 edible units outside storage. These residuals are retained; no new policy tuning follows them. This is calibration observation, not held-out acceptance.

The same p4 frozen binaries completed a valid interleaved BASE comparison (`perf-round2-session-1`), exit1. Ten mean/p99 ratios exceed1.10 and side64 absolute after-build allocation is1.135567×BASE; all six C3 ceilings pass. Keep every raw repeat and the failed receipt. Existing profiles attribute most added time to JobAssign, while FoodDaily runs twice over2880 measured ticks and costs≤0.086us/tick. No timing waiver or threshold change is authorized.

The behavior-preserving follow-up checks state/combat eligibility before costly need predicates and uses an allocation-free existence scan for boolean food availability. Actual nearest food ranking remains unchanged. Two tests compare the frozen old nearest-presence semantics across ground/pot/carry, expiry/depletion/replacement and navigation changes; valid precheck2/2 and post-change systems472/472 plus full all-target Clippy pass. The first test-name filter selected0 tests and is not verification. Full trajectory equality and fresh performance are still pending; source-level reasoning alone does not prove either.

### Actual hunger-chain UI and the selected player restriction

The UI-only seed0 fixture3 uses ordinary All steward until displayday6, then Housing and one sorted batch setting Haul/Hunt priority0 for existing adults. It does not alter needs, health, events or knowledge. Later adults retain normal work. An actual starvation at tick10031 produced the recorded chain; physical food and saved/load state validate. This deliberately restricted supply is not a natural steward-ON survival result or calibration round.

The first focused why capture selected the root observation only; its harness PASS did not prove the full chain. Preserve it as partial evidence. The strengthened capture selects the actual stores entry and checks three linked sentences at both scales: stored food0→reachable ripe food10159→ripe plants observed, with the historical reason for not harvesting explicitly unknown. `ui-hunger-round2-2` completes with0 failures, and root inspected the full1.25 text. It does not manufacture a historical explanation. The12-shot contact sheet combines these disclosed UI cases and does not substitute for held-out, performance, full gate or owner play.

### Allocation attribution before the next frozen candidate

The release no-tick census (`food-build-memory-before-1.log`, exit0) observed414 plants and no food ages, pots, meals, targets or preservation batches in side64. Both retired food maps retain capacity224, approximately6.7KB combined: insufficient to remove the56,198-byte excess over the existing10% limit. BASE already initializes navigation and region labels during colony generation; the plant population's ensure is not a new dense navigation allocation. No plant state or legacy placement inputs are removed to meet the bar.

Source inspection found the bundled196,204-byte mind policy owned twice: the compiled program's retained source and the saved MindState source. The proposed repair borrows the static bundled text in the compiled program, retains dynamic policies as owned text, and leaves saved bytes unchanged. A focused pointer/lifetime regression must fail under the forwarding-copy implementation before changing ownership. The expected saving is a hypothesis until the same census and the full valid paired benchmark measure it. No policy/data tuning or performance waiver is introduced.

The static-source forwarding test failed exactly on pointer equality while the dynamic lifetime test passed. After the Cow change, all474 sim-systems tests and workspace/all-target Clippy pass. The identical release census then measured side64 live allocation1,798,188→1,601,984 and side1924,052,564→3,856,360: exactly196,204 bytes saved in each, with the same plant counts, available units and other census fields. The temporary observer is external again. This controlled no-tick observation confirms the allocation mechanism, not the standing paired performance bar. Fresh baseline identity and full colony trajectory comparison remain separate obligations.

The fourth baseline identity run completed with exit0 on the optimized source:228,748 shared values with zero differing/one-sided paths,64 chronicle files/19,683 entries and64 raw files equal under the existing declared normalization, and determinism seeds0–3 PASS. The log labels the unchanged parent HEAD while the tested optimization was uncommitted; do not misrepresent it as a clean-head final gate. New paired performance and complete colony outputs remain pending.

### P5 measured failure and sampled loose-haul repair

The committed p5 `9936a8fd26be15d171afccf4000c3ebf113326dc` completed33 valid interleaved pairs against published BASE. Its receipt and independent raw recheck retain FAIL for9 mean/p99 ratios; allocator ratios and all6 C3 ceilings pass. No exception or calibration round was consumed. Evidence: `perf-p5-session-1`, `perf-p5-raw-recheck.json` under the external20261029 evidence directory.

A single admitted10-second CPU sample of the frozen p5 wolves scene completed with both sampler and benchmark exit0. It captured7435 main-thread samples,2059 inclusive under `loose_haul`; instrumented timings are not acceptance evidence. The initial regional food-availability cache and filtered mind-existence proposals were deferred because the observed hotspot was loose-haul nearest search. Preserve those proposals as unexecuted external notes, not claimed optimizations.

The haul source predicate narrows when no destination exists anywhere for an ItemSpec, but its prior `seen` witness survived. A later source miss consequently skipped the existing global absence proof and repeated the ground search for subsequent workers. The narrow repair clears only both Loose/LooseFuel positive witnesses after the existing failed-destination transition. Proven empty flags, per-region/per-tick lifetime, nearest ordering, worker radius, quantities, reservations and covered-storage preference remain unchanged. A radius miss alone never establishes global emptiness. Reservation-only exhaustion without a blocked-spec transition remains a potential cost.

The actual-function regression failed before the repair on the missing absence proof; two frozen-p5 output-oracle tests passed, covering a distant valid source, fuel/nonfuel independence, reserved cells and separate navigation regions. The product change adds two witness resets, without a new cache or authored-policy change. Post-change systems/lint, identity, colony trajectory equality and fresh paired performance are separate obligations; no performance improvement is claimed from the regression alone. Evidence: `p6-haul-before.log`, `p6-haul-plan.md`, `p6-haul-review.md`.

Post-repair sim-systems completed477/477 tests and workspace/all-target Clippy passed (`p6-systems-after.log`, `p6-clippy-after.log`). This includes the previously failing absence-proof regression and unchanged frozen output oracles. Real fuel-haul integrations, baseline identity, colony equality and the new benchmark remain separate pending checks at this record.

Existing real fuel-haul integrations subsequently passed11/11 and fuel/sleep integrations6/6 (`p6-haul-integration.log`). They retain successful hauling, player priorities, fatigue interruption and actual sleep/recovery coverage. Baseline identity5 is running; its future result is not implied by these tests.

The user-interrupted turn removed identity5's execution handle and process; its log ends during compilation without a terminal success. It is INTERRUPTED, never PASS. Root confirmed no remaining Cargo/rustc/probe before starting identity6 in a fresh output directory; previous partial evidence is retained.

Identity6 completed with exit0:228,748 shared values with zero differences/one-sided paths;64 chronicle files/19,683 entries and64 raw files match under the standing normalization, and determinism seeds0–3 PASS (`candidate-identity-6.log`). The log labels parentp5 while the two-line repair was uncommitted; this is baseline identity evidence for that working source, not final exact-head gate or colony trajectory equality.

P6 immutable-head fresh performance completed33 validpairs, preserving terminalFAIL and independent raw-recheckFAIL: only n200_side64/tick_mean_us exceeds1.10 (1.119897). All other timing ratios, allocation ratios and six C3 ceilings PASS. Side64 median BASE345.179861/new386.565972us/tick; the observed ceiling gap is6.868125us, not a predicted optimization benefit. Matched raw populations remain200 and p5/p6candidatefingerprints match. Mind accounts for+40.3273us of total+41.3861; JobAssign delta+0.3946. Diagnostic samples are not acceptance data and cannot distinguish call frequency from per-call cost. No cache-cap, cadence, policy or unsupported-provider change is justified yet. Preserve p5/p6receipts and defer calibration3 until performance repair is verified.

### P7 mandatory causal-evidence audit
The earlier starvation UI fixture proves readable linked current observations, but its historical nonharvest reason remains UNKNOWN. Re-reading goal2.7/ownercheck5 makes this incomplete mandatory scope, not a completed historical-cause explanation. Existing counters, plant state and generic cancellation do not reconstruct that history. A future evidence-only feature must record factual acquisition/job outcomes at their actual boundaries, retain bounded timestamps and physical-flow scope, and link those records through the existing why view. Do not fabricate global causality from one worker or infer historical failure from death-time ripe units. This is not implemented yet; it blocks completion/publication, not ongoing repair work. Any resulting evidence/save/fingerprint change must be a separate predicted feature commit and measured calibration result, never silently normalized to pass the prior preservation comparison.

### P7 predicate-equivalent continuation proof
The admitted existing counters show similar corecall counts851145(BASE)/845240(p6), but hits531605→329871 and misses319540→515369. A separately declared instrumented p6side64 diagnostic found first-changed hunger_tenths147795/thirst_tenths118286, versus seeking4383 and other337; cached path lengths8–15 disprove the proposed cap32 explanation for these misses. Its timings are excluded and temporary source was restored byte-exact. Immutable per-policy sorted transition boundaries now permit changed signed values only when every authored predicate remains equivalent. Bool validation/exactness, unsupported/errors, first-read order, policy/entity/stage/tick identity, eight-tick phase and MAX32/per-agent payload remain. Updated equivalent prefix values are current before a later ordinary-scan fallback. Extra memory is per-plan and must pass the allocation bars.

The new regression failed1/3 on the old exact-value hit rule, with other2 tests passing. Post-change sim-core484 unit tests plus integrations PASS. First sim-systems run476PASS/1FAIL was an obsolete numeric-miss hit expectation; its stateless result and full read/value trace still agreed. Only that cost expectation changed; second run477/477 and workspace/all-targetClippy PASS. Independent review found no material defect. Baseline identity7, full calibration trajectory equality and fresh paired performance remain separate obligations. Evidence: p7-equivalence-red.log, p7-core-green.log, p7-systems-green.log, p7-systems-green-2.log, p7-clippy.log, p7-cache-code-review.md.

P7 baseline identity7 completed exit0:228,748 shared values, zero differences/one-sided paths;64 chronicle files/19,683 entries,64 normalized raw sample files and determinism seeds0–3 PASS. The logged e156ae29 parent SHA includes uncommitted P7 source; this is not final clean-head gate evidence. Fresh paired performance and exact calibration2→3 colony outputs remain required. Original identity logs are preserved externally at candidate-identity-7.log.

### P7 acceptance and P8 historical evidence boundary
P7 source5857cdc4f89c24d4a4d5f2ad2f55703c937572c3 completed fresh same-session BASE performance:33/33 valid pairs, all six mean/p99 ratios <=1.10, allocator ratios and C3 PASS; independent raw recheck PASS. Mean/p99: n200 .972124/.960710; n1000 .947506/.827639; n2000 .741673/.861640; script1.044607/1.037530; side64 1.051704/.968085; wolves1.062197/1.037320. Preserve earlier p5/p6 failures. Authentic build/perf receipts remain under candidate-p7-binaries and perf-p7-session-1.
Calibration3 completed exit0 under original deadline: all24 files across8 worlds byte-identical to calibration2, all8 fingerprints equal, no normalizations. Register now3/4 used. These are calibration observations, not heldout acceptance. Cooked and both-winter stores remain7/8; other food/common bars8/8. The exact comparison proves the measured preserving changes did not change these full recorded trajectories.

P8 is a separate intended evidence/schema change required by goal2.7. Record real failed harvest admission/query/completion/cancellation, actual first cooked intake and first preserved placement; retain timestamped personal receipts and explicitly colony-wide flow intervals. Successful meals/harvests retire stale acquisition episodes. No inferred historical reason from current priorities, no per-tick plant scan, no authored-policy change. Append FoodEvidence after the twelve existing FoodState fields; format14 uses an exact legacy13 adapter. Preserve original13 fixtures and old UNKNOWN captures.
New FoodOperation53 entries intentionally change Chronicle ordinals/causal links, serialized evidence, full-state fingerprints and story-yield counts. Fingerprint includes the complete FoodState through food_fingerprint::append, and MindState separately; do not claim it excludes evidence or silently normalize P7. Compare P8 observed physical outcomes/flows/meals separately and audit old-event referents through explicit ordinal mapping. Cohort outputs do not prove every internal action unchanged. New history must be validated through real operation tests, malformed-save tests and actual ordinary-command starvation/milestone captures before scope completion. Final calibration4, freeze, heldout and full gate remain pending; no extra owner approval is required for this authorized correction.


### P8 integrated evidence and compatibility verification

The integrated uncommitted P8 source records actual historical acquisition failures
and colony-flow intervals, plus completed intake/storage parents; it keeps authored
policy and food decisions unchanged. FoodEvidence is appended after winter_plans;
format14 has an exact format13 decoder and no <=12 remigration for physical saves.
ADR0054 records the evidence and compatibility contract. Kind53 remains reachable
through Why but hidden from the owner feed by data. Missing historical reasons are
not filled from present-day snapshots, and old UNKNOWN captures remain preserved.

`p8-integration-green-1.log` reports497 passed,0 failed,1 ignored. The separately
required authentic old13 fixture test passed1/1 in
`p8-original13-save-test-3.log`, preserving complete old food encoding plus the new
empty tail, original system slots, and format14 evidence roundtrip. These are
uncommitted-source checks, not final-head acceptance. Earlier standalone attempts
failed first for missing catalog bootstrap and then for a fabricated cursor using
the engine clock rather than the observer's resource clock. The final test uses
explicit bundled bootstrap and the real observe_day boundary; preserve all failures.

The initial locale assertions failed because Korean layout inserts U+2060 word
joiners. Compare visible semantic text with only that known layout character
removed; preserve production text and all required content/unknown checks. This
normalization does not prove wrapping or replace captures. Source examples are
`tools/v8/test_locale_trf.gd` and `tools/v8/test_core_mind_ui.gd`.

P8 deliberately changes serialized evidence, complete FoodState fingerprints,
Chronicle ordinals/causal links and story counts. P7 calibration2/3 byte equality
remains valid only for its own source. Physical-outcome/flow/meal comparisons and
explicit old-event ordinal mapping are separate obligations; cohort output alone
does not prove every internal action unchanged. Fresh P8 performance, calibration4,
freeze, held-out and the full gate remain pending; no publication or acceptance is
claimed by these focused checks.

### P8 broad regression findings and test-contract correction

The broad workspace run passed2,197 tests in121 completed binaries before a food-mind test failed; this is not a complete workspace PASS. Its stale fixture advanced three days without movement, then expected reranking away from an admitted fresh root stack. Runtime trace confirmed position(90,100), retained roots(87,97), SeekingFood and zero meals. The provider/lifecycle match P7 and implement P4's documented destination retention; this custom policy never reads the new plant-query observer. Preserve the failure and diagnostic compile error. The corrected test makes independent new selections at0,window−1,window, retaining exact item/count assertions and actual arrival/consumption. A separate regression asserts an in-flight owner survives history expiry until arrival. Product policy and retention code remain unchanged. Both tests pass.

Two additional current-writer integration tests expected format13 despite calling save::save. Their actual14-vs13 failures were captured with no-fail-fast; only current-write expectations/names now use14. Authentic13 fixture assertions remain13. Focused three-target rerun passed11/11. Core kind-count assertions now independently pin FoodOperation53/name and54 total while preserving all earlier ordinal checks. Full final-head gate remains required.

### P8 pre-commit identity (2026-10-29)

The supervised identity8 run completed with exit 0: 228,748 shared summary values, zero differences or one-sided paths; 64 Chronicle files/19,683 entries equal apart from git_sha; 64 raw files equal under the existing documented normalization; determinism seeds 0–3 PASS. Source was the staged P8 tree atop P7, not a clean-head final gate. Evidence: `/Users/rexxa/work/m6a-data/20261029/p8-identity8-launch/child.log` and its original terminal receipt. P8 runtime performance, actual UI capture, calibration4, held-out and the complete final gate remain pending.


### P9 capture wording and evidence boundaries (2026-10-30)

P8 was committed as `e2ba280f345202b7f334b213ea2c13e2de828c46`.
`candidate-p8-binaries/build-receipt.json` binds its authentic release build
(exit0,49.22s). Identity8 and full documentation checks passed; identity8 remains
explicitly a pre-commit staged-source run, not a clean final-head full gate.

The ordinary-command hunger fixture reaches tick10031/day7. P8's focused hunger
receipt completed with zero metadata failures, but direct inspection of32 PNGs
was REVISE: an unconditional historical-unknown statement contradicted the actual
personal hauling-disabled receipt. Preserve `p8-ui-hunger-qa-b.md` and its images.
P9 qualifies the limitation of the death-time observation itself, without erasing
the actual historical reason or merging personal history with colony-wide flow.
Direct review of all32 recaptured KO/EN images at scales1.0/1.25 is PASS for this
focused panel (`p9-ui-hunger-qa-b.md`); metadata alone is not visual acceptance.

Keep the full-capture failures separate: P8 failed four obsolete expectations that
kind53 should appear in the owner feed, plus six cases missing at its early stop.
P9 full1 ran30days and captured13/15 cases, still missing hunger Why and pot. Its dried
image obscured by a modal is rejected as visual evidence even if a screenshot
file exists. The original rejection remains historical evidence; the completed full2 follow-up below supersedes the pending recapture action, not that failure classification.

These are wording/capture follow-ups, not changes to authored policy or acceptance
bars. P7's33/33 valid performance pairs remain PASS and calibration3/4 remains
consumed. P9 performance, calibration4, frozen held-out and full gate are NOT_RUN.
There is no P9 pre-commit SHA or publication claim. Earlier failures and UNKNOWN
captures retain their original historical classification.

### P9 native mind-label regression

The first genuine native run observed both hunger and threat transitions and failed exactly two KO/EN checks for `food_novel_available`. Its emitted `MIND_FACT_FOOD_NOVEL_AVAILABLE` key was absent. Auditing all six new providers also found the cooked and cooked-novel availability labels missing in both locales. Add all three labels; novelty explicitly means a food group not eaten in the last three calendar days, not first-ever item discovery. Preserve `p9-core-ui-native-1.log` and its captures. The identical native input procedure is rerun; do not replace it with the passing headless locale check. No provider, fact value, target policy or authored decision changed.

Native UI rerun2 completed exit0 with both hunger and threat transitions observed and zero failed checks. Read-only review of its actual18 PNGs passed the scoped person/Why/visible-Chronicle review, with no fact-key leak. Existing transient toasts can retain their old language after a locale switch; that incidental limitation is not presented as fixed. The focused hunger review independently inspected32 PNGs; its KO1.25 before/after image diff has equal1920×1080 dimensions, intact alpha and11,690 changed pixels, used only to locate the wording change rather than as a pass threshold.

For food captures the harness now presses the real migrant Later button before observation/each screenshot, asserts the panel disappears, and records unchanged tick and actual wave state. It never accepts or declines migrants, hides the panel directly, or injects a decision. The full2 dismissals at tick16140/wave1 and tick31980/wave2 preserve both tick and wave. Terminal evidence and independent QA-A/QA-B now confirm the dried-chain images are unobscured; full2 still has the two missing natural cases recorded below.

Full capture2 completed30days with exactly the two missing natural cases (hunger Why and pot), retaining exit1/complete=false. All13 observed cases and every migrant Later no-simulation-change guard passed; never relabel that run as a full-suite PASS. Separate final-source hunger2 completed with zero failures using the disclosed ordinary-command P8 save; focused pot1 completed with zero failures using the authenticated earlier physical format13 save through the current decoder. Pot UI/migration evidence is not a new natural cohort or evidence of a new pot trade in P9. The current12-shot contact sheet comes only from these final-source captures and labels plant art as placeholders. Independent final image review and the final acceptance gates remain distinct.


### P9 final capture review closure and identity9 (2026-10-30)

`p9-ui-full-2/evidence.json` is terminal exit1, `complete=false`, failures2: natural hunger Why and pot-food were not observed in30days. All46 PNGs were reviewed; `p9-ui-full-2-qa-a.md` and `p9-ui-full-2-qa-b.md` pass only the13 observed cases. Do not rename this receipt a natural full-suite PASS. Placeholder plant art remains placeholder art.

`p9-ui-hunger-2/evidence.json` has32 PNGs, focused complete=true and zero failures; direct QA-B is `p9-ui-hunger-2-qa-b.md`. `p9-ui-pot-1/evidence.json` has3 PNGs, focused complete=true and zero failures. The pot fixture is an authenticated historical format13 physical save: current UI/decoder evidence only, not a new natural P9 cohort or pot-trade result. `p9-core-ui-native-2/evidence.json` has18 PNGs and zero failures; its independent visual review remains scoped to the actual person/Why/Chronicle surfaces.

Product `res://` input hashes agree across the composite captures. Fixture absolute paths legitimately differ and must remain explicit provenance rather than being normalized into identical fixtures. The completed [composite receipt](</Users/rexxa/work/m6a-data/20261029/p9-ui-composite-coverage.json>) verifies exactly27 matching product `res://` hashes against live source,81 PNG hashes and the15-case union. The first external whole-input-map comparison failed because fixture absolute paths differ; raw receipts remain unchanged. This scoped composite does not clear full2's two missing-case failures.

`/Users/rexxa/work/m6a-data/20261029/p9-identity9-launch/child.log` completed all four checks:228,748 shared summary values with zero differing/one-sided paths;64 Chronicle files/19,683 entries equal apart from git_sha;64 raw sample files equal under the existing declared normalization; determinism seeds0–3 PASS. This tested the uncommitted candidate and is not a clean final-commit gate. Owner play, P9 performance, calibration4, held-out and the final full gate remain NOT_RUN. No acceptance or publication is declared.

### P10 BASE72 observation adapter; frozen product and failed evidence preserved

P9 is `5c7e05e677cebda16fea5825f88d83561acd9d2d`. Its authentic release/debug
builds passed; early performance completed33/33 admitted pairs, zero invalid,
all ratios and C3 PASS. Calibration4 completed all8 seeds with exit0 and closed
round4 in the original register. No fifth round is permitted. The first external
calibration3→4 comparator failed on existing Pressure/Modifier cause variants;
its failure remains intact. The separate v2 comparator passed26 negative/positive
checks, all14 previous checks and8/8 observed comparisons. Event bytes and
physical summaries match; new FoodOperation parents and raw fingerprints remain
explicit differences. This is not proof of all internal state/actions identity.

The original P9 freeze precedes held-out evaluation. Candidate1296–1327 completed
32/32 worlds×72days with unchanged source. BASE's original observer invocation
was rejected before simulation in all32 attempts because its CLI requires48days.
`p9-heldout-base/receipt.json` and the failed wrapper remain unchanged; no BASE
comparison, complete held-out acceptance or full-gate PASS is claimed from them.

The goal requires a72-day BASE comparison and permits diagnosed tool corrections;
this correction changes no product, policy, calibration budget or acceptance bar.
Keep the authentic original BASE probe/perf for existing performance/story uses.
A distinct BASE-library observer adapter validates the original48-day argument
contract and extends only the requested horizon to72. Its own source/compiler/link
inputs and qualifications must be authenticated explicitly; it is never described
as an unchanged BASE executable. The BASE library/observer sources are unchanged.
Static review found no48-day truncation of stockpile/beds/wolf counters, but the
legacy `alive_at_day48` field actually describes final survival and must not be
used as day48 evidence from a72-day run. Runtime acceptance additionally requires
start_tick0, end_tick/observed_ticks103680 and clean contiguous-observer checks.

The narrow tool-successor proof pins P9's original committed input inventory,
calibration4, freeze and candidate held-out receipts. Only the seven explicitly
reviewed adapter/validator/gate paths may differ inside product/input scopes;
all existing observers/scorers and their dependencies remain unchanged. Both old
and new source hashes are recorded, never equated or overwritten. Candidate
held-out is not repeated or relabelled, and no tuning follows its evaluation.
Qualification, completed BASE72, fresh final-head performance and the complete
48-step gate remain required before publication. Original deadline1793335103
and the prohibition on inherited M-UX2 stall waivers remain in force.

The first isolated BASE library build failed because the bridge embeds
`assets/ui/content_presentation.json`; archiving Rust alone omitted a real compile
input. Preserve `base72-library-launch` and the original compiler error. The
second build includes every BASE INPUT_ROOTS blob plus the authentic untracked
Cargo.lock and passed. All source bytes before/after, compiler invocations,
Cargo artifacts, link-search inputs and the distinct entry build are retained in
`base72-adapter/manifest.json`; the manifest verifier passed against the actual
artifacts. Its runtime metadata identifies BASE-library provenance, while the
outer receipt identifies the adapter and actual execution HEAD.

The three parser regression tests passed. Seed0 qualification48 is byte-exact
against the original BASE observer; qualification72 matches every deterministic
field of the original ordinary72 run after excluding only the adapter's added
`seeds[*].mux2` observation subtree. Neither result is a candidate calibration.
The unchanged observer's tests plus five explicit horizon tests passed, including
contiguous ticks69120/103679, late predator receipt, duplicate and missing ticks.
The synthetic observer tests are unit evidence, not simulated survival outcomes.
All58 M6a Python tests passed, including the new source-equivalence and malformed
coverage tests. These checks do not substitute for the required BASE held-out,
fresh final-head performance or full gate.

Independent review required binding each recorded Cargo invocation's compiler,
source, extern libraries, native/dependency search directories and generated
outputs to the authenticated inventory, not merely accepting its exit0. The
strengthened validator initially rejected Rust's legitimate implicit `proc_macro`
extern; preserve `manifest-check-2.log`. Resolve only that builtin through the
pinned target sysroot library, require its hash in the frozen inventory and keep
all unknown bare externs rejected. The actual manifest then passed
`manifest-check-3.log`; all62 M6a Python tests passed. No library, executable,
qualification, frozen policy or original cohort changed during this repair.

### Frozen held-out evaluation fails two unchanged bars (2026-10-30 KST)

P10 `bd799460c25a7019dd5d3f30296587c96674615b` completed the distinct BASE
adapter cohort1296–1327×72days with32/32 valid worlds, zero execution/observer
errors and unchanged source. Full artifact validation passed. The original P9
candidate cohort remains byte-for-byte retained; it was not rerun.

The unchanged scorer produced `p10-heldout-score.json`, exit1/statusFAIL:
alive29/32 against30 required; cooked-share24/32 against26 required. Hunger-free
winters29/28 required, stores28/26, diversity30/26, spoilage31/26 and ledger32/32
pass. Common rows are candidate/BASE: stockpile32/32, beds31/31, wolves31/29,
ledger32/32. At winter starts day42/day66, living denominators are31/31 and
stores pass31/28 respectively. These observations cannot be promoted to a gate
or publication PASS.

The no-tuning-after-heldout rule and four-round limit remain binding. Do not
change policy, data, product behavior, scorer thresholds or seed registration
to clear these failures. Independently audit the failing arithmetic against raw
observations; only a demonstrated instrumentation error could justify a tool
correction. Continue the required reference story and remaining validation on
frozen product bytes, retain any red gate, and do not publish while either bar
fails. An outcome-driven product follow-up requires a separately authorized
future evaluation protocol; it is not a fifth round in this pass.

Independent raw audit `p10-heldout-failure-audit.md/json` verified all96 candidate files and recomputed the two failing bars without invoking the scorer. Extinct seeds are1300,1305,1307. Cooked-share failures are1300,1305,1306,1307,1312,1315,1318,1319; seed1306 is651/1304, strictly below50%, and1305 has no qualifying meals. No scorer defect was found. Other bars retain the original scorer results, not a claim of independent full rescoring. Registry-created #670 (plant art) and #671 (frozen acceptance failure) were read back with exact title/body and OPEN state; retain temporary M6A-1/M6A-2 because publication is blocked.

### Reference story comparison completed on frozen product

`p10-story-launch/terminal.json` completed exit0 without interruption or signals. Candidate and original BASE each cover32worlds×72days,96 complete world-years. Candidate/BASE totals: entries57155/32173, deep entries86/7, distinct deep paths19/7, people with3 entries991/990, missing chains115/176, cycles0/0. Path/person totals sum per-world/year counts, not globally unique identities. Original arm executables and source inventories were checked before/after; no BASE72 adapter was used here. Reference only, no numeric acceptance bar, no change to held-out FAIL.

### P12: retain the red gate and isolate historical test environments

P11 `3191890c1b5d829b126d27cd5fd885c82d83c246` final-gate-1 completed **43/48 PASS**, with steps16,26,39,41,45 FAIL. Original logs and `p11-full-gate-summary.json` remain unchanged. Step16 had three actual assertion failures out of471: inherited M6A configuration selected the real M6A dispatch instead of two legacy fixtures and one UI fixture. The selector was nonempty M6A_GATE_CONFIG, not the deadline alone.

P12 changes exactly four tools: `tools/v8/test_core_shadow_freeze.py`, `tools/v8/test_core_ui_successor_dispatch.py`, `tools/v8/m6a_observer_successor.py`, and `tools/v8/test_m6a_observer_successor.py`. Only fixture subprocess environment copies discard foreign configuration; parent environment, real dispatch precedence, deadlines and all original assertions remain intact. Proof v1 remains default with its exact seven-path contract. Explicit v2 (`m6a_observer_test_successor_v2`) binds P10 predecessor `bd799460c25a7019dd5d3f30296587c96674615b`, exact nine P9-to-successor and four P10-to-successor paths, exact before/after test hashes and unchanged file identities/modes. Frozen product, policy, scorer, runner, candidate, register and deadline bindings remain mandatory.

Actual `p12-focused.log`:40 tests PASS, PARENT_ENVIRONMENT_UNCHANGED. Actual `p12-python-all.log`:473 tests PASS (38.307s). `p12-applied-review.md` reports no blocking defect but was written by the patch author, not an independent-author reviewer; A separate non-author review (`p12-independent-review.md`) also found no blocking defect. `p12-static.json` records AST/compile PASS for all four tools; LSP diagnostics were UNAVAILABLE. These checks repair the test/proof boundary only, not product acceptance. The original step16 FAIL is not retroactively replaced.

The unchanged BASE72 validator requires exact current-source identity. After committing the reviewed successor, generate a NEW explicit v2 proof/config and a fresh BASE-only cohort with the identical authenticated BASE library/adapter inputs, seeds1296–1327 and72-day protocol. Keep original BASE receipts and candidate cohort untouched; do not relabel old execution hashes. This is not candidate rerunning or a fifth calibration. New proof, fresh BASE cohort and full-gate-2 from step1 are **NOT_RUN**. Product/policy/scorer/candidate/calibration changes, post-heldout tuning and threshold waivers are prohibited; the four-round cap and original deadline1793335103 remain binding.

Step25 passed33 valid performance pairs/66 ADMITTED invocations/0 invalid pairs and all six C3 bars. Retain one pre-admission NOT_RUN and its15-minute wait; it was not an invalid measured pair.

Step26 is a valid standing-soak FAIL: seeds0–3 starvation9/9/9/10, survivors1/1/1/0, KnowledgeLost0/2/1/0. All ledger/reservation violation ticks are0 and activity requirements pass. The steward-OFF scripted fixture lacks explicit food preparation commands, but autonomous harvesting exists; no specific starvation mechanism or causal link from starvation to knowledge loss is proven. Do not dismiss an older standing fixture as invalid or change it to clear the result. M6A-3/#672 is OPEN after create/readback.

Step39 is a valid strict visibility FAIL: day1_noon_half agent9/entity4294967305, owned235, qualifying45<60, unstable0, unclipped; other seven scenes PASS. The historical top steward action control overlaps the actor. Unchanged layout/oracle paths do not prove a new M6a layout regression or make this a capture defect. No occlusion censoring, camera rearrangement or threshold reduction is allowed. M6A-4/#673 is OPEN after create/readback.

Step41 has a complete receipt and no capture errors, but strict FAIL: at1× elapsed34.469124s,52.771ms, tick361, ticks_advanced=1. There is one post-warmup frame over50ms at1× and none at0.5× (maximum49.539ms). **NO M6a waiver** applies; the historical M-UX2.1 exception is not inherited. Step45 retains frozen held-out FAIL (survival29/32 vs30; cooked-share24/32 vs26). No publication or issue closure follows. Sources: `/Users/rexxa/work/m6a-data/20261029/p11-full-gate-summary.json`, `p11-stall-summary.json`, `p11-soak-failure-review.md`, `p11-visibility-failure-review.md`, `p12-focused.log`, `p12-python-all.log`.

### P12 final gate2 completion; P13 documentation only

P12 `a9ae9c87cfc322ff9ef06db7c3de470f799a7be3` completed full-gate-2 exit1 without timeout or signals:44/48 PASS,26/39/41/45 FAIL. Gate SHA256 is `fa08d81b1da9c994bbb5d6a9f22b52c6e50672fce29879510fa4cd13b74b74ec`. The earlier P11 43/48 and step16 fixture failure remain historical facts; P12 step16 now passes. Sources: `p12-full-gate-summary.json`, exact48-row report table.

The new explicit v2 proof and fresh identical-protocol BASE32 cohort completed. Strict cohort validation passes and64 deterministic summary/churn files match the original BASE bytes; timing is excluded and not performance evidence (`p12-base-validation.json`). Frozen candidate and scorer remain unchanged; `p12-heldout-score.json` repeats survival29/30-required and cooked-share24/26-required FAIL. No fifth calibration, tuning, reseeding or waiver.

Current visibility FAIL is52/60 qualifying pixels, owned210, bounds[792,132,16,32], agent9/entity4294967305, unstable0, valid/unclipped, inputs unchanged; seven other scenes pass. Keep prior45/60 separately. Root viewed toolbar partial overlap; neither invalid capture nor a new layout regression is proven (`p12-visibility-summary.json`). Current stall is complete/errors[] but strict FAIL:1× elapsed34.448181s,52.056ms,tick361,advanced1; half zoom max49.157ms with no frame over50. Prior52.771ms remains historical. NO M6a waiver (`p12-stall-summary.json`). Standing soak also remains FAIL.

P12 performance passes33 valid pairs/66 ADMITTED/0 invalid/0 NOT_RUN, all6 C3, no waiver (`p12-perf-summary.json`); do not import P11's one NOT_RUN/wait into this session. All four registry issues #670–673 remain OPEN, no publication or closure. Remote main/wip were read back as BASE `5b0c88812374b87e7854cc9203f144ad0ccd9009`.

The goal is not complete. Subsequent P13 is documentation only, NOT_FULL_GATED, not a new product or a claim its own future SHA was tested. Keep product P9, gate P12 and documentation successor distinct. Future product work needs a separately authorized evaluation protocol; this record requests no waiver or extra calibration. Owner play remains NOT_RUN and the main play command runs published BASE.

P13 GitHub reconciliation: registry issues670–673 were read back OPEN. Added only final evidence comments671/6512553876,672/6512556023,673/6512558060 and verified exact bodies; original issue title/body/state/labels/assignees/milestone remain unchanged. No closure, renumbering or publication. Receipt: `p13-github-final-readback.json`.

## Repair round R1 — owner decision 2026-10-30 11:20 KST

Source: planner/owner message in this thread, dated 2026-10-30 11:20 KST. This explicitly supersedes the earlier prohibition on further defect repair and the earlier NO M6a stall waiver; historical failures and original frozen evidence remain unchanged. Runtime receipt began at 2026-10-30 02:18:51 UTC (11:18:51 KST), slightly before the note's stated time. Conservatively use that actual start: target 12:18:51 UTC /21:18:51 KST, hard stop 16:18:51 UTC /2026-10-31 01:18:51 KST, with no reset on continuation.

0. First action completed: pushed `v8/m6a:refs/heads/v8/m6a-review` at `e8d5c1668c0649d21a75f463759043e0af627e53`; readback matches. Push this review ref after every repair commit. It is not publication to wip/main; those remain BASE `5b0c88812374b87e7854cc9203f144ad0ccd9009`.
1. Treat starvation as an economy defect, not a near miss: old soak has9–10 hunger deaths/run and KnowledgeLost1–2; retired held-out lost3 villages and3 worlds had winter hunger deaths. Defect repair is authorized; tuning numbers against the old held-out is not.
2. Before product changes, write `M6A_R1_DIAGNOSIS.md`: every soak/failed-world hunger death with last meal, stores by kind, reachable ripe plants, harvest/cook/haul jobs and holders, last MindChoice/blocked candidates, steward targets, and evidence-based root cause. Diagnose every world's raw/cooked meal counts, hearth bills, cook jobs/workers, pot/grain availability. Diagnose half-zoom52/60 visibility with before/after crops (plants, ground tint, draw order and other evidenced causes). Compare Part1 predicted harvest/year and winter need with measured seeds0–7; explain every gap above20%. Missing telemetry is missing evidence, not a guessed cause.
3. Repair each confirmed cause in its own commit with a failing-first regression. A soak-like scenario must have0 hunger deaths and0 KnowledgeLost. Allowed domains: harvesting, hauling, reservations, eating choice, cooking bills, steward planning, rendering order. §3 bars stay unchanged. Plant yields/densities may change only if the arithmetic diagnosis proves a design error; write corrected arithmetic and affected numbers first.
4. Calibration seeds0–7 are reopened for at most2 logged repair rounds, then freeze. Reserve fresh held-out1328–1359 exclusively for final evaluation:72days, steward on, no orders, same bars; BASE5b0c8881 on identical seeds supplies every comparison row, including72-day survival. Retire1296–1327 from evaluation; inspection/replay is diagnosis only. All soak runs require0 hunger deaths/KnowledgeLost; half-zoom visibility still requires≥60px.
5. R1 stall waiver is authorized only if every other step passes, each zoom has≤2 evaluation frames>50ms and none>60ms, and capture is complete. Record use/non-use and all exceptional frames; old P12 failure is not retroactively a passing gate.
6. Run the full gate from step1 on the final commit. Publish only when all required conditions hold, via fast-forward of BASE to v8/wip then v8/main; if main advanced, apply the rebase/re-gate rule. Update#649's document link; close#671/#672/#673 only if fixed with public commit links;#670 stays open. Korean addendum includes before/after death causes, cooked causes, visibility, fresh held-out with BASE, soak, full gate table, published SHA and ls-remote.
7. Stop without publication if after2 repair calibration rounds soak still starves, or diagnosis reveals a design problem requiring the owner (for example insufficient plant density to feed10 adults through winter); report measured numbers. Preserve original goal/artifacts; this dated record is the protocol amendment, not an edit to the original goal.

Execution sequence: diagnosis and missing-observation inventory → root-cause regression/repair commits → at most2 calibration rounds → freeze and fresh paired cohorts → exact-head full gate/waiver audit → publication and issue reconciliation. Current phase: diagnosis; no R1 product edits, calibration or fresh held-out runs yet. Raw artifacts: `/Users/rexxa/work/m6a-data/20261030-r1/`.

### R1 repair A — actual meal admission (P16)

Diagnosis was committed and review-pushed as P15 `d60ac7c9a` before product edits. All66 hunger deaths and36 original replay identities are recorded there; the additional32 heat replays also match. Four unfiltered colony hearth-meal nodes admitted workshop shelf reserves without an edible target. Add the existing factual `food_available` guard to those four nodes; keep stock accounting, scores, food ownership and filtered meal nodes unchanged. No baseline policy, plant values or acceptance bars change.

Failing-first bundled-policy regressions: `meal-admission-red-5.log` has2FAIL/1PASS and the actual targetless `meal_idle_no_water` winner; the shelf counts2units and harvest knowledge admission is valid. Earlier invalid fixture setup (missing storage zone) is preserved separately, not counted as reproducing the defect. `meal-admission-green-1.log` passes3/3, including both water states, actual physical harvest then meal with shelf2retained, routine ownership and edible-ground positive control. This establishes the admission repair, not soak survival or final evaluation. R1 calibration remains0/2; fresh held-out NOT_RUN.

### R1 repair C — propagate existing cooked demand into ingredients (P17)

The cooking bill already targets the authored daily cooked budget, but ingredient procurement requested only one recipe batch. Reuse the same daily-target arithmetic and project the remaining output deficit through whole recipe batches. Procurement admits capable recipes without requiring their inputs to exist first; actual bill availability keeps its current supplied subset. Sum shared input demand, subtract fresh physical inputs once, and reserve still-unconsumed inputs of ToSource crafts whose outputs already count in Maintain. Working crafts already consumed their inputs. Preserve preservation's separate existing budget, knowledge/pot/heat capability and player protections.

No new seasonal cooking target, cooked-share knob, plant yield/density or recipe values were introduced. The broader winter-menu allocation question remains explicitly separate from this internal target mismatch. `cooking-procurement-red.log` reproduces6 intended failures, including2grain requested versus16 needed for the existing24portion target. `cooking-procurement-green.log` passes24 steward-food tests, including6 new cases for zero demand, existing output/input, shared input, pending stages and available recipe budget. Workspace all-target clippy passes (`repairs-ac-clippy.log`). These are scoped mechanics results, not proof of final cooked-share or starvation bars; calibration remains0/2.

### R1 evaluation adapter — fresh protocol and authenticated stall exception (P18)

The dated R1 contract has its own discriminator, seed allocation1328–1359, two started-round cap, exact current-source/binary/scorer freeze and fixed hard deadline1793377131. Legacy protocols and their failed records remain valid under their original rules; R1 does not extend the old observer-successor product exception. The qualified BASE72 adapter is reused with its original library/build/identity evidence and the fresh seed range. Every score row has a BASE column:72-day survival and common metrics are measured; missing seasonal telemetry is NOT_OBSERVED; absent physical-food concepts are NOT_SUPPORTED, never fabricated zero/PASS. All candidate bar predicates/numbers stay unchanged.

Independent review found that an incomplete calibration receipt could be frozen. The preserved counterexamples now reject: empty worlds, wrong horizon, changed source, failed world, absent required files and altered hashes. Freeze validates the final calibration receipt; final gate validates every successful round, while retaining failed prior starts within the two-round limit. `protocol-independent-review.md` records the original P2 and resolution,19 independent tests; `protocol-completion-final-green.log` passes92 tests.

Step41 uses the same producer and strict timing computation. A separate R1 receipt path authenticates complete raw strictFAIL/exit1 before sealing all source/binary/GPU and artifact hashes; raw FAIL is not rewritten. It records every evaluation frame>50ms, its tick and ticks_advanced. Eligibility is at most2 such frames per zoom, none>60ms; the existing>33ms/minute bar and capture/instrumentation checks remain. Final authorization separately requires every other gate step to pass, with an empty waiver list on strict PASS. Old P12's incomplete outer receipt is not retroactively accepted. Stall adapter focused tests/typecheck passed; actual R1 stall remains NOT_RUN, waiver use UNDETERMINED until the full gate.

### R1 adapter type boundaries and isolated counterexamples (P19)

A follow-up static audit found introduced Python type errors at JSON, timeout and waiver boundaries. Explicit typed containers, CLI arguments and timeout arguments remove all newly introduced diagnostics without suppression or changing score predicates. Comparing the same existing modules against P17 gives21errors/195warnings before and19errors/178warnings after; these retained diagnostics are not a clean typecheck claim. Evidence: `protocol-typecheck-comparison.md` and both diagnostic JSON files.

The freeze regression now first proves a complete valid fixture can freeze, then independently asserts each intended failure message; missing event/chronicle fixtures can no longer mask a failed-world check. `protocol-typefix-tests.log` passes92 tests. External launch scripts are prepared but not executed; calibration remains0/2 and fresh held-out NOT_RUN.

### R1 repair B — complete the physical stock transaction (P20)

M6a replaced legacy colony forage/gather execution but omitted the six autonomous stock-work admissions retained in BASE. Restore their existing work-band scores, adult/need/inventory/stock-target guards using physical Harvest jobs. The factual `harvest_window` reads the existing authored forage clock; `job_food_transport` observes current Haul/Supply food metadata. Neither adds a numeric policy. Existing duty-protected jobs, urgent eating, player priorities and disabled hauling remain authoritative.

Harvest now returns its actual ground-placement receipt. On completion, an eligible worker reserves an ordinary Haul of actual produced food to an existing reachable, unreserved stock cell with room; ordinary transfer code preserves age and accounting. Missing/blocked storage or disabled hauling leaves conserved loose output. Nonfood harvesting keeps ordinary completion. A separately reproduced routine boundary admits a pending Activity state only after the authoritative routine transaction has already interrupted that activity.

Failing-first evidence: `harvest-stock-red-1.log` has2expected failures/1fallbackPASS; `harvest-stock-boundary-red-1.log` reproduces the interrupted-routine start failure. `harvest-stock-green-5.log` passes4tests, including a persistent bundled Mind decision, actual Harvest→Haul→storage, preserved old Chop duty and no-storage/disabled-Haul fallback. An initial test incorrectly expected stock work to preempt protected Chop; its failure is retained, and the test now preserves inherited duty priority rather than changing policy. Final `harvest-stock-green-6.log` passes7/7, including reserved destination, actual spill placement and authored hoard-cap. The first reserved-destination fixture omitted its owning Job and correctly failed the invariant; that invalid-fixture log is retained separately. Related harvest tests11/11 and meal-admission tests3/3 pass; workspace all-target clippy with warnings denied passes (`harvest-stock-clippy.log`).

`harvest-stock-independent-review.md` independently reviewed the exact working-file hashes and found no blocker; it left actual soak and the subsequently completed boundary checks pending at review time. Whole-economy survival, cooking share and KnowledgeLost are not established by these unit tests. Calibration remains0/2; fresh held-out remains unopened.

### R1 visibility repair — preserve people beneath floating Actions (P21)

The same retained P12 fixture reproduced52/60 idle and45/60 work with zero unstable pixels at load3.27/5.71/7.13, no other Godot/cargo/rustc and the recorded P17 debug bridge. Two Actions-only theme variations preserve the measured Button4px and OptionButton8px margins, RGB, fonts, geometry, focus and disabled styles; only normal/hover/pressed background alpha changes0.60→0.45. No plant, camera, person shader, rendering metric or acceptance threshold changes.

At independently admitted load6.52/6.37/7.30, the identical scene/camera/person/poses reaches89/98 qualifying pixels with zero unstable pixels. The unchanged eight-scene instrument passes8/8scenes and80/80person rows on the retained fixtures; half minima89/80/119/85 and quarter318/384/318/384. KO noon/night×UI1.0/1.25 interaction matrix captures20screens, no overlap errors; root directly viewed noon full and night1.25inspector. Existing why-input checks pass42/42. Scoped GDScript strict parse/warnings and60FFI bindings pass.

Evidence: `actions-red1/`, `actions-green1/`, `actions-matrix1/`, `actions-eight-scenes1.log`, `actions-why1.log` and individual admission receipts under the R1 evidence directory. These are scoped UI regression results using the explicitly identified P17 bridge, not final repaired-world acceptance. The full gate must freshly build the final bridge and generate its own current-policy scenes. Initial load10.314/9.064/9.291 was NOT_RUN and waited one15minute window; it is not a measured failure. Plant placeholder art remains#670OPEN.

### R1 round1 result and bounded next repairs (P22 diagnosis)

Round1 onP21 completes all8 calibration worlds; survival/cooked/winter hunger/diversity/ledger all8, stores7, spoilage0. Unchanged soak is valid FAIL: starvation0/10/0/0, KnowledgeLost0all, determinismPASS. Sources remain unchanged, original register contains exactly one started/finished pair. No held-out execution or publication.

The fingerprint-matched seed1 replay proves stock harvesting can repeat without deliverable storage, creating194 loose pieces bytick360. Calibration separately proves avoidable mixing of new food into older stock reduces its lifetime. Preserve authored oldest-age merging, food lifetimes, scores, targets, yields and densities. Authorize bounded acquisition admission plus age-aware haul destination/owner allocation using existing actual capacity; selecting a farther clean storage cell is accepted to avoid destroying freshness, retaining normal distance/tie/covered preference within that pool. If no clean capacity exists, preserve existing mixed-age fallback rather than reset age or discard food. No global reroute/teleport, schema change or numerical tuning. Record each defect with its own regression and commit.

These mechanisms do not yet prove repaired survival. One R1 calibration remains; if the second round still starves or a design limitation requires the owner, stop without publication as directed.

### R1 repair D — avoidable storage-age mixing (P23)

Use existing physical age and capacity to prefer empty or same-production-day storage owners. Ordinary loose/carcass/restock hauling and harvest handoff share this preference; preserve spatial distance/ties and covered-store preference within the clean pool. Arrival fills clean pot/ground owners before existing mixed fallback. If no clean room exists, retain oldest-date merging. Do not change food ages, lifetimes, yields, densities, target numbers, reservations, generic ground placement or save schema. This removes the duplicate harvest destination selector.

Failing-first `age-haul-red-2.log` reproduces both nearest-old-stack selection and old-pot-before-empty-ground mixing. `age-haul-green-2.log` passes4 cases and `age-arrival-green-1.log` passes2, including real movement/delivery/expiry, harvest handoff, inverse pot/ground allocation, mixed fallback, missing age, reservations/reachability and nonfood control. Existing harvest-stock tests pass7/7. A separate review found no blocker; its initial ItemSpec memo concern was retracted because generic capacity is a superset, not an age-specific negative cache. Final performance and economy remain unproven; broader library/clippy checks follow with the separately owned admission repair.

### R1 repair E — acquire only deliverable stock (P24)

Six proactive physical-stock entries now require a real reachable, unreserved compatible storage destination and room, positive item carry capacity and enabled Haul priority. Reuse the shared indexed physical-storage selector; refresh its colony overlay once per Mind tick. This prevents repeated nonurgent harvesting before the scripted stockpile exists and prevents a rejected high-score action from blocking alternative work when hauling is disabled. Urgent hunger harvesting keeps its separate no-storage path. No score, target, cap, plant value or continuation cadence changes.

`harvest-admission-red-1.log` reproduces3 intended admission failures with the urgent control passing. Final `harvest-admission-green-2.log` passes5/5 and existing harvest-stock tests7/7. Intermediate transition-test failures are retained: the fixture wrongly expected immediate full selection at360; the existing absolute phase for agent7 is367. Exact runtime witness proves continuation retained wander between phases. Final regression advances ordinary movement/Mind/jobs through that boundary; no selector, checks or cadence were weakened to make it pass. The provider's unsupported-continuation status does not force a work-challenger scan between full phases. Full library/clippy and final economy evaluation remain separate.

The first full library run after these repairs reports522PASS/2FAIL/1ignored. Both failures are synthetic positive meal fixtures that default omitted facts to0: the water-boundary case chose `need_idle_hunger` instead of `meal_idle_water_not_critical`, and retained-job case chose `wander` instead of `meal_idle_no_water`. P16 intentionally requires actual `food_available`; add that input only to these positive fixtures. Preserve expected choices, exact thirst boundary, default-zero helper and actual shelf-only negative regressions. This is a test-input correction, not removal of the meal guard or a new economy change. `age-admission-all-lib.log` and `meal-factual-fixtures-pre-edit-red.log` retain the failures.

After the positive-fixture correction, all sim-systems library tests pass524 with1 pre-existing ignored (`age-admission-all-lib-2.log`); the focused facts suite passes13/13. Full docs including the generated system table passed before the final fixture-only correction; `p24-docs-static.log` retains the subsequent static pass. The separate harvest-output age audit identifies a possible seam without actual calibration event attribution, so it does not authorize an additional speculative repair.

Final integrated workspace all-target clippy with warnings denied and cargo fmt check pass (`age-admission-clippy.log`, `age-admission-fmt-check.log`). Independent read-only P23/P24 review finds no confirmed blocker. Remaining limits: admission uses the current nearest edible target rather than searching a different food kind when its storage is full, and extra storage queries need measured gate validation. No alternate-target policy or performance claim is added.

### R1 final calibration and freeze boundary (P25)

Round2 completes on P24 `b2fab234210a0ca301ee6b655aed816562501c98`, probe SHA256 `a1f88fe0e49c66e0e22c1f1017dd5f122f3786e7549c77a166f78a2f41f4436c`. Source/lock/binary provenance is complete and unchanged. The original register now contains exactly2 started/finished pairs; no third calibration or numerical tuning is authorized.

Unchanged soak passes4×20days with determinism, exact ledger/reservations, all40 adults alive,0deaths and0KnowledgeLost. Workshops2all; crafts75/74/76/77 and trees150all prove active economies. Prior9/9/9/10 and round1 0/10/0/0 starvation remain recorded failures. This repairs the standing soak result, not every possible world.

Calibration72days: alive/cooked share/hunger-free winters/diversity/stores/ledger/stockpile/beds/wolf all8/8. **Spoilage remains0/8**, ratios30.94–51.33%, against the unchanged≤30% per-world bar. Calibration is NOT_EVALUATED for acceptance; it cannot be relabeled a passing evaluation. The owner's post-round2 starvation stop does not trigger because all soak runs pass; no supply-impossibility diagnosis has been established. Freeze now and evaluate fresh1328–1359 with identical BASE seeds, without tuning on either retired or fresh outcomes. Publication still requires every unchanged held-out bar and the full gate.

|seed|cooked/all meal events|harvest units|spoiled units|spoil ratio|
|---|---:|---:|---:|---:|
|0|808/1105|2663|1185|44.50%|
|1|1015/1234|2893|1485|51.33%|
|2|799/1205|2943|1399|47.54%|
|3|737/1233|2522|1002|39.73%|
|4|875/1161|2768|1364|49.28%|
|5|784/1271|2175|673|30.94%|
|6|954/1239|2939|1472|50.09%|
|7|919/1255|3048|1550|50.85%|

Raw evidence: `candidate-p24-binaries/build-receipt.json`, `calibration-register.jsonl`, `calibration-round-2/{terminal.json,calibration-score.json,soak.log,soak/summary.json}` under the R1 directory. P25 is documentation only before fresh evaluation; runtime/tool inputs remain P24-identical. Fresh held-out, full gate and actual stall waiver use are NOT_RUN at this checkpoint.

### R1 frozen fresh evaluation and final-gate checkpoint (P26)

Under the owner note2026-10-30 11:20KST, both calibration rounds are exhausted and the P24 product remains frozen. Fresh1328–1359×72days candidate and qualified same-seed BASE receipts complete with unchanged sources; independent authentication and scorer recomputation agree. Candidate survival30/32 versus BASE31/32. The sole failed acceptance predicate is spoilage3/32(required26); all nine other predicates pass. Keep BASE unsupported/unobserved values explicit, never fabricate zero. No third calibration, reseeding, target/yield/density/lifetime or bar change. Publication is ineligible.

Fresh chronology records14 hunger deaths across1345(6) and1347(8). Exact jobs/holders, blocked candidates and individual access are not available: one death has9560 reachable ripe units; five retain4 recorded stored raw-edible units; eight have neither recorded stored units nor ripe plants. This is not proof of world-wide food absence, density insufficiency or a completed causal diagnosis. Preserve the original66-death diagnosis separately. See `r1-fresh-death-summary.md`, `heldout-final-score.json`, `r1-paired-evaluation-audit.md` in the R1 evidence directory.

Story reference comparison completes32worlds×3years per arm with source unchanged, exit0 and no timeout. Candidate/BASE entries49058/32644, deep42/7, distinct-deep-chain world/year sums9/7, people-with-three world/year sums955/990, missing80/152, cyclic0/0. These are reference counts, not a numerical acceptance bar. The final seal independently validates complete years and authenticated report inputs.

The contact sheet now records actual P21 UI before/after crops and KO scale/time views, explicitly limited to retained fixtures and the P17 bridge. Preserve the old sheet externally; final fresh bridge/world visibility is still NOT_RUN. P26 is documentation-only and must run the complete48-step gate from1. The R1 step41 waiver is not eligible while another required predicate fails, irrespective of frame-level eligibility; actual stall frames remain pending. Review branch only; no wip/main publication or issue closure.
