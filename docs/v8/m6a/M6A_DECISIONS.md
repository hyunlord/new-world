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
