# C1 decisions

## C1-D1 — Publication and evidence boundaries

Work occurs only on `v8/c1` in `new-world-c1-dev`; BASE is
`d30dbfc5b8c6d68973e1c13fdd291741db892998`. Raw evidence lives in the external
`c1-data` directory. C1-a precedes C1-b. Remote publication fast-forwards `v8/wip`
then `v8/main`; it never checks out or changes the protected predecessor trees.
Both gate runs explicitly set `PLAY_ROOT` and `PLAY_DATA` under `c1-data`.
§5's repeated-same-gate-failure stop, one Inert crash fallback and 48-hour limit
remain binding. Historical failed or aborted evidence is retained.

## C1-D2 — Capacity and genuine predecessor bytes

The requested 39 appended knowledge nodes plus 43 existing nodes exceed64.
Widening knowledge capacity is necessary infrastructure for the requested data,
not a balance change. Historical ordinals remain stable. BASE was archived and
compiled before product changes; its actual format19 save at tick1440 is the
predecessor test input. Actual BASE19 loading, byte-identical format20 resave and twin1440-tick
continuation passed; boundary tests cover the complete prior u64 range and
new ids0/31/32/63/64/95/96/127. Evidence: external
`c1-data/logs/knowledge-width-{boundaries1,high1,save19-1}.log`.

## C1-D3 — Supported dependencies and staged livestock

Berry dye uses `weaving` and `foraging`, not unsupported ochre production.
Bow crafting uses `cordage` and `hafting`, not an absent bone-tool recipe.
Animal taming uses `tracking` and `cordage`, not Inert trapping; C1-a connects
the physical pen and retains capture debt until C1-b. Dairy discovery depends
on actual milk stock and pottery practice, not an unreachable Inert herding
node. No physical milk is granted by discovery. Tool efficiency, armour defence,
basket carrying and healing remain debt where the actual runtime has no hook.

## C1-D4 — Physical template geometry

Oven_house and smokehouse use4×4 instead of the brief's illustrative3×3 because
the existing station needs a separate south work tile. Granary retains4×4 and
pen5×5. Open pens must not grant roof shelter. Fence blocks livestock by an
authored capability while retaining people-passable physical construction.

## C1-D5 — No calibration

Smoke seeds0–7×48days stewardON/no orders detect collapses and panics. They are
not a tuning set. Wild yields, crop seasons/work, knowledge discovery, trade seed
stacks and later livestock quantities stay commented data and BALANCE_DEBT rows.

## C1-D6 — Focused integration failures are preserved

Before the first publication gate, focused tests exposed colony world metadata
that leaves world_variety empty: assuming neutral at that point defeated the
origin feature. Habitat admission now falls back to the existing seed-derived
WorldVariety draw without mutating baseline state. The historical five-plant
census retains its frozen arrays against archived predecessor plant/crop data;
current C1 worlds are independently tested for their actual regional content.

Widening internal masks also requires JSON to expose low/high words rather than
serialize a u128 number through serde_json::Value. Zero high words are omitted
from historical probe rows. These are implementation corrections, not outcome
tuning. Failed focused logs remain in c1-data/logs; the later full gate is recorded below.

## Integration review: regional seed clues and staged dairy debt

A traded seed originally did not break the absent-origin discovery cycle: crop work required its knowledge while discovery required harvested crop food. The five new seed items now name their crop node as an existing artifact/eureka clue. Physical trade and cultivation prerequisites remain necessary; no knowledge grant, injected food or new discovery probability was added. The origin integration fixture must demonstrate the actual trade-to-discovery path.

Milk/wool and dairy now explicitly retain C1-b supply debt. Their authored food/recipe identities are connected, but C1-a has no milking/shearing producer; dairy's milk-stockpile discovery cannot naturally run until that source exists. Wool also retains C2 processing debt. These debt annotations change no simulation behavior.

## D7 — one authorized Inert fallback after a completed RED smoke

The completed candidate `crash-head-a2` returned alive `[10,10,11,14,10,0,13,10]`
and KnowledgeLost `[0,0,0,0,0,25,0,0]` against the original BASE's
`[10,12,11,14,10,13,12,14]` and all-zero losses. Seed5 lost ten people to
starvation. Separate seed2 paired construction diagnostics proved severe labor
interference from resident boars; they do not alone prove every cause of seed5's
collapse. Under GOAL §5, `wild_boar` alone becomes Inert, with its authored
population, origin and numeric parameters preserved. No second item switch or
outcome-directed repair loop is authorized by this decision. [#694](https://github.com/hyunlord/new-world/issues/694)
tracks reactivation in a later balance pass.

Animals use the existing Connected/Inert content status with Connected default,
mandatory Inert debt and a generic natural-population admission check. This is
content activation plumbing within AnimalSystem, not another system. Existing
saved animals remain representable. The generated debt ledger lists the withheld
species. First probe attempt's JSON panic is separately retained as a technical
failure, not treated as a completed crash-detector outcome.

Post-switch `crash-head-a3` completed exit0: alive `[10,13,11,14,10,12,12,12]`,
KnowledgeLost all zero. The paired collapse floor passed without any further
item switch or numerical change. Exact binary and summary hashes are in its
external receipt. Full frozen-source publication verification remains required.

## D8 — Preserve the first full gate failures

Frozen e0578430641a8c3ada03b81f53a67ef27f5b9c9e completed its first full gate
with47/49 steps passing. Step41 exceeded the existing60ms post-warmup ceiling
once at zoom1.0:61.646ms, displayed tick361. StewardSystem consumed33.286ms
of37.063ms native processing; this does not account for the entire frame.
Repeated template expansion inside the measured house-search path is a bounded
optimization candidate. Its timing benefit is not yet established; preserve
candidate order, visit/score bounds, geometry, occupancy and navigation behavior.
No threshold, waiver, authored quantity or simulation policy changes are allowed.

Step46's Why EN run failed7 checks and completed49 of the expected64 checks.
The missing visible minimap caused15 checks to be skipped; an open priority panel
obscured map input and the observed language was Korean. The initiating cause
remains unproven. One unchanged isolated EN/KO reproduction passed64 checks each
with correct language. This is non-reproduction, not a fix or replacement for the
failed full gate. Do not reduce coverage, hide the obstruction or invent an
external-input explanation. Both original failures remain counted under GOAL §5.

External evidence: fullgate-a-attempt1, stall-a1-failure-extract.json,
stall-a1-diagnosis.md and why-a1-diagnostic. Postrun binding confirmed all3058
source pins and the gate configuration unchanged. Original executable hashes
are retained in a1-frozen-bin for behavior comparisons. Publication remains pending.

The follow-up implementation caches four immutable rotated plans and their first
door offsets once per house-planning call, borrowing the selected plan. Existing
pre-change template9/house12 tests passed; after the change template10/house12
passed, including exact original-algorithm comparison for square, rectangular,
custom-door, missing-door and invalid templates. This proves the focused contract,
not the full identity or frame ceiling; both remain required in the next gate.

## D9 — D-C1-1 distinguishes draw-only stalls from the fixed house search

The second gate preserved the native template-search improvement, but step41
recorded an83.751ms frame at zoom0.5, tick1946 with zero simulation ticks.
Its process-to-pre-draw interval was4926µs and pre-to-post-draw78646µs.
At that frame building sprite creation/free/texture-request counts were zero,
terrain bake/compose/upload counts were zero, and nearby draw/node/object counts
were unchanged. The previous building texture request was over12seconds earlier.
No image was captured at the hitch. Existing counters do not exhaustively cover
procedural textures, font glyphs or driver activity, so its root cause is unknown.

The bounded resource investigation found no new imported C1 art bindings.
New grazers reuse category fallback frames and are hidden at half-scale; plants
share existing phase textures. Item/template procedural caches are outside the
manifest-only warmup, but there is no evidence they first appeared at the hitch.
D-C1-1 authorizes extending the existing warmup only for an attributable resource;
no speculative renderer change or threshold adjustment is made.

A2 step28 was a launcher defect: comma-joined READ_CHANGED_FIXTURES was read by a
whitespace parser. The external launcher now derives the same ten-name space
list and asserts/prints exact equality with the frozen config before launch.
A3 must run from step1 with external PLAY_ROOT/PLAY_DATA. Step46 must pass.
Only if step41 alone fails with an unattributed draw-only hitch may one isolated
same-binary rerun qualify under D-C1-1; a second failure stops the pass. Retain
both runs and, if that exception is used, record the debt and planner issue note.
Historical A1/A2 failures and the prior literal §5 stop remain preserved.

## D10 — C1-a publication is a separate completed boundary

A3 completed49/49 PASS, status0, with the existing step41 waiver and no isolated
rerun. At2026-11-09 02:41:59 KST, `v8/wip` and `v8/main` both fast-forwarded to
`094fea092f467a7c82764165ad0d2f9e74b806e3`. The external
`C1-a-publication.json` records both remote results. This closes the pending A
publication statements above; it does not erase A1/A2 or establish any B result.

## D11 — Livestock content selects mechanics; pens supply physical capacity

C1-b selects capture knowledge, age/sex eligibility, products, tool costs and art
through tameable animal data and existing categories. No animal-name branch or
new content enum is required; renamed data-only content must exercise the same
mechanics. The generic steward requests a known template with
`livestock_capacity`, then designates an eligible wild kid through ordinary
commands. It neither names a pen template nor grants knowledge or supplies.

A completed pen requires its actual fence/door layout. Capacity6 and9 physical
grazing cells are distinct: the fixed regrowth/feed values support3 heads per day
without stored fodder. Daily order is regrowth, stable-ID feeding, hunger/readiness,
deaths, then births. Capture, milk and explicit slaughter use actual workers,
tools and atomic physical output placement. Authored values are starting design
values recorded in BALANCE_DEBT, not outcome-tuned acceptance targets. This B
decision supersedes D3's staged dairy debt: discovery uses actual capture practice
and pot-firing knowledge, avoiding a prerequisite on its own milk output.

## D12 — Livestock terminal facts have immutable typed witnesses

Captured, born, milked, slaughtered and died events retain typed witnesses keyed
by their actual Chronicle append index. `CauseRef::Livestock` and its matching
link validate that witness, event and tick rather than forming an Entry self-loop.
Capture/slaughter intent provenance requires the exact accepted command target
and steward wrapper; automatic milking uses the animal's actual creation event.
Birth/death do not invent human workers or command origins. This preserves the
distinction between an accepted request and a completed physical outcome.

## D13 — Preserve genuine format20 state and exact historical mind registries

The B wire boundary is format21: append the livestock tail while preserving the
frozen format20 body's field order. Skills completion counters widen23→26 by
appending three slots; old counters and upper knowledge-mask bits must survive
without a second conversion. The published A executable produced genuine
ordinary/rich format20 files; the frozen A3 visibility run supplied a third
completed-building save, with hashes in `published-a-legacy20/receipt.json`;
current-writer fixtures cannot substitute for those inputs.

The genuine rich-save test exposed a real historical mind-loader failure:
`unknown fact "job_food_transport"` (`b-genuine20-tests-2.log`,0 passed/1 failed).
Adding even an unused fact changes the compiled dictionary/revision. The
immediate predecessor must retain `harvest_window`, `job_food_transport`,
`harvest_storage_available`, `winter_stock_duty` and `water_target_available`,
excluding only the newly appended `job_livestock`. The older format13 registry
is a separate explicit case. Every admitted schema must still match the complete
saved dictionary and revision; unknown schemas remain errors. The explicit
pre-livestock reconstruction passed all three genuine fixtures, byte-stable21
resave and60-tick twins (`b-genuine20-tests-4.log`,1 PASS); the full systems lib
suite passed813 tests (`b-systems-lib-tests-2.log`). These focused results do not
claim the B publication gate or end-to-end chain passed.

## D14 — Fix the natural source before executing the physical chain

The B scene fixes its source by the first qualifying seed in0..63: reachable open
highland within40 with at least20 cells, natural opposite-sex goat kids, and clay
within24. Source admission precedes outcomes; no age slack, replacement spawn,
RNG edit or intermediate supply is added after observing the chain. Only the
declared animal-taming knowledge goes to the two adult founders. Clay, fire,
pots, flax, cordage and pen materials follow ordinary physical work.

The bounded scene must show pen, capture, two adults, birth, subsequent fresh milk
and explicit slaughter/meat/hide, with a mid-chain save/twin continuation and full
command replay. These are required assertions, not completed results in this
decision record. B chain, full gate and publication remain unclaimed here.

The first admission fixed seed5 and sources10/11; source profiles and hashes are
retained in `b-chain-admission-lock.json`. Run1 stopped on a redundant priority
command; run2 built the real pen but the original sources matured before capture.
Ordinary workflow repair removes automatic bill contention, schedules physical
collection/crafting/building and captures the earlier-expiring source first.
The same source set, age/sex, authored odds and biological clocks remain fixed.

Run3 exposed the existing batch-job accounting: two physical products on one
`Make(2)` bill complete one Craft job, so they cannot prove two completed Craft
jobs. Run4 used two ordinary `Make(1)` bills and proved both completions, but
later preparation outlasted the original sources and a stale person reference
failed after a natural death. These failed scenes remain preserved. They are
workflow failures, not a reason to weaken discovery, grant cord, replace sources
or change their age. Final chain success remains pending.
