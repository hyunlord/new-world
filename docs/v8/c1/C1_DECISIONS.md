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
