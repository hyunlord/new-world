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
