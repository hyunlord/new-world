# M6a food-chain design and arithmetic

Status: pre-product implementation design, 2026-10-29. BASE
`5b0c88812374b87e7854cc9203f144ad0ccd9009`; runtime sources unchanged through
`cb3f79cc69df10ee05b10d5a5330e0226f67fc68`. This document commits Part 1
before product code. The census is actual tick-zero worldgen evidence;
food supply and spoilage scenarios are conditional arithmetic, not gameplay
or calibration results. [ADR 0052](../../decisions/0052-item-food-chain.md)
records the architecture. All owner acceptance counts remain unchanged.

## Current units and demand

Adult hunger growth `0.05/tick` is in
`rust/crates/sim-systems/src/worldgen/camp.rs:134` and
`runtime/agent/maturation.rs:56`; child growth `0.03/tick` is in
`rust/crates/sim-core/src/rules/baseline.rs:114`. HungerDecay applies it each
tick (`runtime/needs/hunger_decay.rs:57`). The colony calendar is 1,440 ticks
per day, six days per season, 24 per year (`rules/calendar.rs:34`). Existing
food consumes one counter unit and relieves 30 hunger points
(`runtime/agent_actions/consume.rs:31`, `agent_actions/needs.rs:152`);
automatic stockpile meals are a second legacy consumer and will be retired
only in colony worlds. `settlement/food_economy.rs:116` independently uses
2.4 units per adult-day.

Warm adult demand is 72 hunger/day = 12/5 food units/day = 57.6/year.
Child demand is 43.2 hunger/day = 36/25 units/day. Winter acceptance need is
`ceil(72 * living_adults / 5)`, rounded once per settlement: seven adults
need 101 units; ten need 144. Planning also counts children's demand; the
adult acceptance formula does not excuse starving children. Cold multiplies
growth by 1500 permille (`rules/baseline.rs:156`): 480 cold night ticks would
require 2.8 adult units/day, continuous cold 3.6. These are conditional
bounds, not observed winter exposure. Nominal 3 food/day in neighbour policy
is a conservative policy divisor, not the actual hunger constant.

FoodDef units retain the legacy scale. Meat presently has two units
(`sim-core/data/colony/items.ron:230`). Direct item consumption records the
whole item's units plus actual relief/wasted relief; inventory conservation
never silently subtracts only useful satiety. Cooked portions are units1
with batch counts in recipes so existing meal thresholds need not change.

## Census and deterministic placement contract

Ordinary final colony worldgen was executed twice for seeds0–7 at tick0.
Each map is192×192 with ten adult founders. Both runs exited0 and match
byte-for-byte:294,912 unique tiles,8 worlds,294,921 records,0 simulation ticks.
Input SHA256: `6d52f6351fdc99d6d6d6bd4843ca092b1b4bb62a478f39ad276e28cefe77c322`.
The census includes final nav connectivity after camp opening/fords and all
placement blockers. Existing12 camp bushes are not counted as new plants.
Radius16 is Chebyshev (`rules/baseline.rs:45`); counts require a standing
path from the actual camp. Whole-map placement may include other regions,
but those plants do not inflate the local supply arithmetic.

For every tile in y,x order, filter species by authored terrain/elevation
and reject physical blockers. All initial species admit plain/forest,
elevation0..255, passable tiles without buildings, trees, resource nodes,
ground items, space sites, walls, water or old wood sources. Old food is not
a blocker because colony food maps are being replaced. Species are sorted
by canonical ID; with eligible weights w and densities d, density is
`sum(w*d)/sum(w)`. Draw `keyed_below(seed, 0x504C414E54535631, x,y,0,0,1000)`
for occupancy; if below density, draw slot1 against sum(w) for cumulative
species selection. No seed-specific minimum, top-up or camp exception.
All initial densities are200permille and weights are berry3,hazel2,
flax1,grain3,root4. Product placement must match every frozen census row,
including all off-radius plants, before calibration.

## Derived supply and storage scenarios

Census seed0–7 validated by root and SHA-checked by evaluator: `6d52f6351fdc99d6d6d6bd4843ca092b1b4bb62a478f39ad276e28cefe77c322`. All eight have ten adult founders. Placement v1 remains unchanged. No product code, Cargo build or simulation was executed in this evaluator task. The Python RNG is a direct integer translation of `rng.rs:201–254`; mix(0) reference and bounded-draw checks passed; executed Rust/Python parity is NOT_RUN.

## Fixed candidate v2 knobs

- Placement: plain/forest, elevation0..255, passable, no building/tree/node/ground-item/space-site/wall/water/oldwood; old food is replaced, so not excluded. No camp-distance exclusion or seed topups. All species density200 permille; weighted-average admitted density; weights berry3,hazel2,flax1,grain3,root4. Canonical ID order; keyed stream0x504C414E54535631; subject=x,tick=y,replicate0; slot0 bound1000 density, slot1 bound13 choice.
- Yields per opportunity: berry4 on year-days6,8,10,12,14,16; hazel8 on12; grain6 on9; root5 on0,2,4,6,8,10,12,14,16; flax4 on6, not food. No banked missed harvest. Full raw grain becomes3 one-unit porridge portions per2 grain.
- Whole cooked items each units1; recipes: meat1→roast_meat3, grain2→porridge3, meat1+root1→stew4, roots2→roasted_roots3. Each item consumed is one completed meal event; batches are not meals. Root group retained; category meal cannot fabricate dietary diversity.
- Root raw lifetimes [ground,stockpile,roofed,pot]=[6,10,14,18] days. Berry[2,3,5,8], hazel/grain[12,18,24,30], rawmeat[2,3,4,6], cooked[2,3,4,6], dried[12,18,24,30]. All-class models assume unlimited actual capacity in that class; they do not prove any pot exists. Root recipe60 ticks/batch; harvest60 base /1500permille knowledge multiplier=40 ticks. Haul/path travel remains UNKNOWN.
- Demand:0.05×1440=72 hunger/adult-day; each one-unit portion relieves30 if hunger≥30; nominal2.4 portions/day,57.6/year,14.4/winter. Owner winter bar=ceil72A/5, ten adults144 edible stored units. No threshold changes or partial-item mechanic.

## Measured placement; derived annual production bounds

Counts below are reachable standing tiles inside radius16. Seasonal totals include edible raw units PLUS potential cooked grain units. Grain contribution is separated; it is not raw-edible stock.

|Seed|Berry|Hazel|Grain|Root|Flax|Spring|Summer incl grain|Autumn|Annual incl grain|Grain potential|Annual raw-edible|
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
|0|30|40|27|50|16|750|1353|1430|3533|243|3290|
|1|14|7|24|31|4|465|849|689|2003|216|1787|
|2|38|19|26|28|10|420|1110|1028|2558|234|2324|
|3|34|22|31|41|14|615|1302|1199|3116|279|2837|
|4|27|13|21|29|7|435|948|863|2246|189|2057|
|5|13|3|8|13|2|195|423|375|993|72|921|
|6|30|12|24|31|12|465|1041|921|2427|216|2211|
|7|51|13|30|34|8|510|1392|1226|3128|270|2858|

Min / mean counts: berry_bush 13 / 29.625, hazel 3 / 16.125, wild_flax 2 / 9.125, wild_grain 8 / 23.875, wild_root 13 / 32.125.

Preserved v1 failure: seed5 root13×3×3=117 spring units against144 warm demand, deficit27 even with instantaneous work. V2 root13×5×3=195, margin51 (35.4% of warm demand); need73.85% harvest realization if eaten raw. If Cold all480 nominal night ticks: need168, margin27; continuously Cold need216, deficit21. These are conditional arithmetic, not cold observations.

Grain potential alone spans72–279 units/year against576 warm annual units:12.5–48.44% satiety share, insufficient to derive50% cooked meals. Root cooking makes a food-supply-feasible cooked route available; actual meal share remains an evaluation outcome.

## All-harvest cohort stress scenario

Every reachable plant harvested immediately at every opportunity; zero work/haul delay, initial population frozen, no cold/pressure/trade/animal food. Fractional25ths bookkeeping models ideal demand, NOT a proposed fractional-item mechanic. Grain is kept raw until just-in-time cooking; its9 potential units/plant retain raw-grain lifetime. Expiry precedes daily intake. Percent loss is potential satiety units lost / produced in this model. All stock start snapshots below are after expiry and before consumption at days42/66. They include convertible grain potential and thus are NOT owner edible-store-bar results.

|Seed|Storage|72d loss %|72d unmet units|Winter2 stock potential|Winter3 stock potential|
|---:|---|---:|---:|---:|---:|
|0|ground|81.13|0|1063|1063|
|0|stockpile|74.12|0|1683|1683|
|0|roofed_store|69.40|0|2303|2303|
|0|pot|63.55|0|2923|2923|
|1|ground|71.11|0|582|582|
|1|stockpile|62.36|0|948|948|
|1|roofed_store|57.20|0|1314|1314|
|1|pot|51.11|0|1680|1680|
|2|ground|76.13|0|666|666|
|2|stockpile|69.43|0|1098|1098|
|2|roofed_store|65.78|0|1530|1530|
|2|pot|60.15|0|1962|1962|
|3|ground|80.15|0|865|865|
|3|stockpile|72.77|0|1411|1411|
|3|roofed_store|68.39|0|1957|1957|
|3|pot|62.55|0|2503|2503|
|4|ground|73.52|0|583|583|
|4|stockpile|66.41|0|981|981|
|4|roofed_store|62.11|0|1379|1379|
|4|pot|56.20|0|1777|1777|
|5|ground|44.41|72|226|226|
|5|stockpile|36.02|0|408|408|
|5|roofed_store|31.65|0|590|590|
|5|pot|25.55|0|772|772|
|6|ground|75.61|0|622|622|
|6|stockpile|68.38|0|1052|1052|
|6|roofed_store|64.13|0|1482|1482|
|6|pot|58.22|0|1912|1912|
|7|ground|80.99|0|714|714|
|7|stockpile|74.49|0|1258|1258|
|7|roofed_store|70.87|0|1802|1802|
|7|pot|65.07|0|2346|2346|

All scenario food balance identities passed. Full seasonal/year/cohort numeric values are in result-v2.json. Overharvesting can fail the30% spoilage criterion even with generous supply; the planner must stop harvest when demand/reserve are satisfied.

## Demand-targeted root-only feasibility scenarios

Whole integer roots and whole cooked portions. Ten fixed warm adults consume24 portions/day =8 batches/day =16 roots/day. Earliest-expiry raw cohorts; cook only same-day demand, no mass precooking. No other species harvested: intentionally proves only food/cooking feasibility, fails dietary variety. The two scenarios below use identical placement/yield/lifetime values.

1. Consumption-only target: replenish two days of raw demand; autumn target carries remaining demand to next spring, accounting for remaining per-plant harvest opportunities. Raw reserves near96 at winter can cover144 cooked portions but FAIL the owner144 edible-unit stock bar.
2. Owner-bar target: from summer day8, target enough current stock that future scheduled harvest capacity plus stock covers remaining prewinter raw consumption and144 raw units at day18. Formula at ripe day d∈[8,16]: max(32,(18-d)×16+144−future_harvest_opportunities×root_count×5). Harvest min(root_count,ceil(max(0,target−raw_stock)/5)) plants. Same rule repeats each year, no seed branches.

|Scenario|Storage days|Seeds|72d spoiled raw / harvested raw|Loss %|Unmet portions|Winter2 raw units|Winter3 raw units|
|---|---:|---|---|---:|---:|---:|---:|
|consumption|6|0,1,2,3,4,5,6,7|99/1155|8.57|144|97|97|
|consumption|10|0,1,2,3,4,5,6,7|0/1155|0.00|0|98|99|
|consumption|14|0,1,2,3,4,5,6,7|0/1155|0.00|0|98|99|
|consumption|18|0,1,2,3,4,5,6,7|0/1155|0.00|0|98|99|
|owner-bar|6|0,1,2,3,4,6,7|249/1305|19.08|144|147|147|
|owner-bar|6|5|249/1305|19.08|144|130|130|
|owner-bar|10|0,1,2,3,4,5,6,7|35/1235|2.83|0|144|144|
|owner-bar|14|0,1,2,3,4,5,6,7|0/1200|0.00|0|148|144|
|owner-bar|18|0,1,2,3,4,5,6,7|0/1200|0.00|0|148|144|

Only the owner-bar scenario tests nominal raw edible stores directly; stockpile10 days, roof14 and pot18 meet modeled winter2/3 stores and have zero unmet portions. Ground6 days still loses winter food despite an adequate start snapshot. No actual alive/meal/diversity bars are claimed.

## Labor and rejected oversized meals

Harvest60 ticks with foraging1500permille→40 ticks/job; each successful job yields5 roots. Demand-only lower steady labor:16 roots/day /5 =3.2 harvest jobs/day→128 ticks/day;8 cook batches/day×60=480 ticks/day; sum608 ticks/day=0.4222 worker-days before travel/hauling/interruptions/knowledge gates. Integer batching and reserve construction require more on some days. Actual modeled totals follow.

|Owner-bar storage|72d harvest jobs|Harvest ticks|Cook batches|Cook ticks|Total worker-days excluding hauling|Peak daily harvest+craft ticks|
|---:|---:|---:|---:|---:|---:|---:|
|6|261|10440|528|31680|29.250|1000|
|10|247|9880|576|34560|30.861|1000|
|14|240|9600|576|34560|30.667|1000|
|18|240|9600|576|34560|30.667|1000|

Rejected cooked units3: at hunger50, nominal90 relief wastes40 and requires4.32 food units/adult-day; at meal threshold30 wastes60 and requires7.2. Units2 at50 waste10; at30 waste30. One-unit cooked portions avoid these specific threshold-clamp losses without changing behavior thresholds. If actual eating occurs below30, even units1 can waste; actual relief/consumed-unit telemetry must preserve that distinction. Demand calculations are steady-state lower bounds until observed, not guaranteed completed meal counts.

## Scope and verification limits

All numeric losses above are deterministic conditional model results, not estimated natural gameplay probabilities. Daily boundary order, finite work, pathfinding, interruptions, per-agent hunger, knowledge, actual cooking fire, room/pot capacity, population changes and pressure remain unmodeled. Root-only scenario provides one group, never diversity success. Need frozen real eating-event telemetry, actual stored edible-vs-convertible units and exact spoil/recipe ledgers before calibration. Policy acceptance counts30/32,28/32,26/32 remain owner policy, not mathematically inferred probabilities.

Artifacts: contract.json and result-v1.json retain v1; result-v2.json retains first v2 model; result-v2-ownerbar.json records the stronger reserve scenario; plants-v1.jsonl contains all selected tiles. Unbuilt Rust evaluator draft remains preserved. Decimal stream draft was replaced before first evaluation by unambiguous hex identical to the approved contract; no measured run used the decimal draft.

## Frozen acceptance definitions before calibration

Calendar indices are zero-based. Winters2/3 are days[42,48) and[66,72).
Winter snapshots are after that day's expiry and before consumption.

| Bar | Required definition and derivation |
|---|---|
| Alive | At least30/32 worlds have a living village at the day72 boundary. This93.75% count is owner acceptance, not a probability derived from food arithmetic. |
| Hunger-free winters | At least28/32 worlds have no hunger deaths in either winter2 or3. Dead villages cannot pass vacuously. The87.5% target tests whether actual work/pressure fulfills the supply model. |
| Stores | At least26 of the original32 meet `ceil(72A/5)` edible, fresh, reachable stored units at BOTH day42 and66. Also report successes/alive separately for each boundary. Raw inedible grain and future conversion are not edible stores. |
| Cooked share | At least26/32 have completed cooked-item meal events / all completed item meal events >=50% over days[24,72). Zero meals fails. A batch is not one meal. Root/grain conversions make the target feasible; only real events prove it. |
| Diversity | At day36, use the trailing three days[33,36), wholly within year2 summer. For every living adult count distinct ingredient groups from successful meals. Lower median must be>=3 in26/32. No adults fails. Report all six summer daily windows as diagnostics; never choose the best window after observation. Category meal does not count. |
| Spoilage | In26/32, spoiled food units <=30% of harvested food units over[0,72). Count each raw harvest once; transformation gain has a separate ledger term and never inflates harvest denominator. Raw grain units are its declared ingredient units despite inedibility; also publish edible and convertible subtotals. Zero harvest fails. Without net cooking gain, ten adults require at least ceil(576/.7)=823 gross units/year before growth/work loss. This is an illustrative no-conversion bound, not a necessary raw-harvest bound for recipes with separately counted gain. Seed5's993 potential ceiling and targeted storage model support feasibility only. |
| Existing survival | Day1 stockpile, beds by day10, wolf deaths<=1 lose no more than one passing world against BASE on the same32 seeds. Publish exact inherited metric predicates. Ledger exact remains32/32, independently mandatory. |
| Performance | Every standing scene <=+10% with C3 ceilings passed, admitted interleaved repeats at load1<8 with1/5/15 recorded. No historical exception applies. |

These bars are frozen before calibration. Initial policy/data may be adjusted
only within at most four registered seed0–7 cohort rounds, with explicit
arithmetic and preserved failures. Freeze final policy/data, observer and
scorer before held-out1296–1327. Do not tune on held-out results.

## Food schema and policy choices

Schema1 plants carry stable IDs/name keys, category/tags, terrain/elevation,
weight/density, four phases, data ripe-start/end offsets, yielded item/count,
regrow days, harvest work and optional knowledge speed/gate. Unknown species
use category presentation fallback. Registry validation rejects dangling
items/knowledge, invalid phases/windows and unknown fields.

Food data carries units, nutritional groups, raw edibility, cooked preference,
sickness permille and four storage lifetimes. Initial cooked recipes have
units1 outputs: meat1->roast3; grain2->porridge3; meat1+root1->stew4;
roots2->roasted_roots3. Add fruit_porridge from grain2+berries1->4 portions,
groups grain+fruit, same knowledge-or-pot gate as porridge. This makes three
actual ingredient groups available with roots during the last three summer
days despite cooked-first choice; it is not a fabricated preparation group.
At the smallest census grain8 yields48 grains and berry13 yields ample fruit:
24 possible fruit-porridge batches produce96 portions, versus72 nominal meals
for ten adults over three days. This is an availability upper bound, not an
observed distribution across people. All cook recipes initially work60ticks.

Cooking knowledge is seeded to at least two founders through data, requires
fire_keeping for discovery; grain_parching requires foraging; food_drying is
an authored paleolithic node. All remain within32-bit knowledge capacity;
no farming/weaving era marker is added. Exact seeding/discovery values are
recorded in the initial data commit and frozen before the first cohort.

Harvest uses the existing generic job lifecycle,60 base work ticks and1500
permille foraging speed. Priority selects urgent edible supply then missing
groups and durable winter reserves; stop designating when actual usable
stores plus reserved production meet the authored target. Winter reserve
planning starts in summer, not on the last autumn day. Cooking maintains
small demand-limited batches instead of making all ingredients spoil early.
Every target/weight lives in data and respects player-edited bills/priorities.

## Persistence and ownership contract

Keep existing serialized item,carry,workshop and colony structures intact.
Introduce an explicitly serialized format13 FoodState sidecar, skipped in
SimResources' legacy wire tree. It contains tile-indexed plants; per-owner
birth dates; real pot vessels/contents; recent meals/groups; food tallies and
once-only causal events. Owners are ground tile, carried AgentId, stable
workshop/spec key or pot/spec key. Shelf vector indices are not stable IDs.
Splits preserve birth; merges choose oldest birth. Expiry precedes transfer
and intake. Missing metadata is an invariant failure, never a freshness reset.

All food movements use one take/put/transfer API with explicit leftovers and
no silent item loss. Job cancellation/death, workshop shelves, wildlife,
trade and consumption share it. Whole-map-full cases retain ownership or
return failure; they cannot rely on a debug-only assertion. Existing item
ledger counts pot vessels and contents exactly once; sidecar totals split
harvest, meal, spoil, craft gain/loss and migration. Satiety identity is
initial+harvest+imports+conversion_gain = eaten+spoiled+exports+present.

Pots are real stationary containers while filled, capacity12 physical items
per vessel from item data. Deploy an actual pot, remove it from ground/carry,
record its vessel owner, then fill via transfers. Removing a filled pot
spills contents with unchanged ages. Roofed store requires an actual
storehouse role; ordinary carried food uses ground lifetime. Spoiled food
vanishes with a ledger term; no rot item in M6a.

Pin legacy bundled catalogue revision
`catalog-v1-fnv1a64:1e4d7fded25139b4:45408` and all historical ID maps before
any data change. Formats6–12 baseline continuation remains supported with
empty FoodState. Freeze the exact v12 body decoder; do not deserialize new
fields under an old version. Existing old-colony continuation tests also
remain supported: convert each finite abstract reserve/carried Food unit to
one units1 legacy_food item, using safe chunks and an explicit migration
term. Existing carried jobs keep their item ownership; abstract cargo
converts to ground at its owner. No fabricated meal history. Existing physical food on ground, in Carry and on
workshop shelves also receives metadata. Sort owners deterministically and
assign old global meat-lot quantities oldest first; a merged stack takes its
oldest assigned birth day. Missing historical age cannot imply freshness:
unmatched pieces are conservatively expired during migration with explicit
spoil/consumed ledger entries, retaining their quantities in the audit.
Counter-to-legacy_food conversion is separately declared at the saved day
because that old representation carried no item age; it is not an ordinary
transfer or harvested food. Clear migrated meat lots only after exact quantity
reconciliation. Generate plants
once from the saved seed at the saved day/phase; preserve all known legacy
system state and explicitly insert only the new system slots. Unknown
catalogue revisions still fail before typed decoding.

## Legacy migration decisions

Colony births no longer withdraw an unchecked ten-unit abstract food tax;
actual child hunger and planning remain. Colony fission stays disabled.
Legacy Food aid/porter proposals become baseline-only; preserve wood aid,
relations and the existing item-based caravan trade. The single-settlement
colony does not gain an unrequested second food-cargo authority. Record this
explicit limitation; future inter-settlement food aid must carry real items.
No periodic clearing of counters hides leaks: fixtures assert no colony
legacy food maps/reserves/cargo after initialization, migration and transfers.

The full pre-code inventory follows, including every direct production site
and all142 food-related authored mind nodes. File:line references identify
the frozen preimplementation source; migration completion will be audited
against this table rather than deleting it as code moves.

## Full production food inventory

Scope: current pre-implementation source in `/Users/rexxa/work/new-world-m6a-dev`; evidence snapshot HEAD `a3106c9d727ef10dd6f29cb2a5fe5d493e582565`. No builds/tests performed. Paths below start at `rust/crates/` unless stated otherwise. Full audit: [inventory.md](/Users/rexxa/work/m6a-data/20261029/inventory.md).

Verdicts apply to the food portion only: **replace** colony path with plant/item state; **keep baseline** preserves old executable/serialized behavior; **delete** retires colony legacy policy/conversion. Shared water/wood/sleep mechanics remain. Every row below declares world and verdict.

## Production file/symbol responsibilities

| Source | World / verdict | Responsibility and required migration |
|---|---|---|
| `sim-engine/src/lib.rs:428,449,795,798,885,918` | shared / keep for baseline only | Authoritative food maps, initialization and setters. Colony must no longer seed/use them. Retain serialized baseline layout or version it explicitly. |
| `sim-engine/src/legacy_v7.rs:105,126,346,349` | baseline compatibility / keep for baseline only | Frozen historical resources and conversion copy both food maps; do not reinterpret old bytes as a new layout. |
| `sim-systems/src/worldgen/camp.rs:297,551; worldgen/colony.rs:588` | shared camp + colony extension / replace colony; keep baseline | Finite bushes and caps seeded by camp/regional generation. Replace plant placement; terrain/tree/shelter occupancy checks at camp:326,491,529,562,599 and colony:113,484,575,604,716 must check plant occupancy. |
| `sim-bridge/src/ffi/engine_build.rs:244` | shared fixture constructor / keep for baseline only | Legacy finite-food scene constructor; preserve supported legacy scene behavior. |
| `sim-systems/src/runtime/resource_regen/mod.rs:413,431,444,588,607` | shared / replace colony; keep baseline | Food regen snapshot, seasonal slowing/tally and depletion observation. Water/sleep regen remains. Colony plants own phase/day updates; legacy food regen channel must not refill colony items. |
| `sim-systems/src/runtime/agent_actions/consume.rs:12,24,31,33,37` | shared / keep for baseline only | Direct bush consumption and sentinel decrement semantics. Colony eat must call item consumption instead; baseline decision uses this same function. |
| `sim-systems/src/runtime/agent_actions/needs.rs:27; agent_actions/forage.rs:35,41,85,89` | shared / replace colony; keep baseline | NeedAvailability food flag and nearest/eligible bush mechanics use old maps. Colony must get edible item availability; harvest uses plant jobs. |
| `sim-systems/src/runtime/agent/movement.rs:366,517,568` | shared / replace colony; keep baseline | Resource pursuit availability and carried abstract-food return-home path. Item hands/haul must replace colony Inventory::Food routing; preserve water/sleep and baseline movement. |
| `sim-systems/src/runtime/settlement/food_economy.rs:20,77,85,91,104,124,148` | shared / replace colony; keep baseline | Stock sum = abstract settlement reserve plus member Inventory Food. Supply/carrying-capacity/stock-target estimators depend on source caps and continuous regen. Replace colony store sum and seasonal harvest arithmetic together. |
| `sim-systems/src/runtime/settlement/stockpile_deposit.rs:134,144,160` | shared / replace colony; keep baseline | Drain Inventory Food into abstract settlement reserve. Leave wood path intact. No colony guard in current tick. |
| `sim-systems/src/runtime/settlement/stockpile_consume.rs:188` | shared / replace colony; keep baseline | Automatic meal/relief withdrawals outside mind. No colony guard in current tick. Must disable legacy food feeding in colony to avoid double consumption or hidden meals. |
| `sim-systems/src/runtime/settlement/settlement_system.rs:958,984,993,996,1254` | shared / replace colony; keep baseline | Birth/growth food gates and per-birth withdrawal. Replace counter/supply observations; define item-backed birth cost or retire colony abstract withdrawal explicitly. |
| `sim-systems/src/runtime/settlement/fission.rs:271,333,420,706,979,991` | shared / replace colony; keep baseline | Site occupancy; abort/arrival stores; migration food reserve and departure withdrawal. Migrating group food must be item-backed or colony rules must explicitly avoid this legacy channel. |
| `sim-systems/src/runtime/settlement/relations.rs:358,416,503,611,686,705,828,876,885` | shared / replace colony; keep baseline | Food aid/trade/porter paths use generic kind.resource() aliases, not literal Food at each mutation. Delivery/return stores and departure withdrawal/overflow are real food writers. Do not miss by literal-only grep. |
| `sim-core/src/components/porter.rs:29; components/settlement.rs:177,190` | shared / keep for baseline only for food semantics | PorterKind::Food maps to ResourceKind::Food; generic Settlement store/withdraw helpers are used by both food and wood. |
| `sim-systems/src/runtime/settlement/shelter_planning.rs:101,102` | shared / replace colony; keep baseline | Building placement excludes old food maps. Update colony plant occupancy exclusion. |
| `sim-systems/src/runtime/wildlife/mod.rs:750,773,808,911` | colony / delete legacy conversion; replace with item eating | feed_larder destroys grazer_meat stacks, records consumption and stores abstract Food without a person eating. Must remove colony automatic conversion when food items become meals. Existing meat spoil-lot handling also needs unified stack spoilage. |
| `sim-systems/src/runtime/neighbours/mod.rs:264` | colony / replace | Food condition for visitor/newcomer behavior reads abstract reserve; must read edible item units. |
| `sim-systems/src/runtime/steward/planning.rs:310` | colony / replace | Stock-in-hand drives steward food planning. Replace source with edible units and winter need. |
| `sim-bridge/src/ffi/ui_facts/economy.rs:32; reads_settlement.rs:113; snapshot.rs:270` | shared read facade / replace colony; keep baseline | Economy days/food, settlement reserve and carried food bridge rows expose old units. Keep documented baseline fields while projecting colony items consistently. |
| `sim-bridge/src/ffi/snapshot/resources.rs:46,61,63,80,81` | shared read facade / replace colony; keep baseline | Draw resource tile rows, including depleted capped bushes. Colony needs plant phases instead; water and sleep rows unchanged. |
| `sim-systems/src/runtime/mind/context.rs:205,299,395,425,620,730,796,820,840,846,918,939,1135,1202` | colony / replace | Snapshots of Inventory Food, food spatial index, settlement target scopes/camp-stocked flags, stock target, food invalidation. All must observe items/plants, with same-tick invalidation on every consuming/producing effect. |
| `sim-systems/src/runtime/mind/facts.rs:361,374,389,404,412,435,438,445,668,697,709,718,726,761,778,1160,1180,1184,1214` | colony / replace or delete old forage facts | Availability/actionability/camp stock/carry/hoard/targets and target resolution. Preserve pure fact-provider boundary. Hunger measurement/memory remains; only food source semantics change. |
| `sim-systems/src/runtime/mind/behaviours/mod.rs:177,223,257,264` | colony / replace eat+hearth_meal; delete legacy forage+gather policy | Registry binds eat to Food, hearth_meal to idle callbacks, forage/gather to GatherFood. Hearth meal currently waits for automatic stockpile relief, not own item consumption. |
| `sim-systems/src/runtime/mind/behaviours/needs.rs:18,36,64,68,78,80,122` | colony / replace | Target validation, consume_food call, successful meal provenance and dirty domains. Preserve arrival-before-consumption, successful-only provenance and interruption cleanup. |
| `sim-systems/src/runtime/mind/behaviours/gather.rs:25,33,43,69,109,207,209,212,218,223` | colony / delete legacy gather; replace by harvest jobs | Bush gathering consumes counter into abstract Inventory and preserves infinite sentinel. New harvest should yield ground item stacks; no infinite colony food. |
| `sim-systems/src/runtime/mind/behaviours/prepare.rs:21,28,33,38,77,104,114; behaviours/postpass.rs:75,84,183,232,243,244` | colony / replace | Forage plans/claims and retarget/deferred postpass are old-bush dependent. Retire associated food branches, preserve unrelated social/routine/firewood paths. |
| `sim-systems/src/runtime/mind/system.rs:650` | colony / replace | FoodTile effect-domain updates shared tick context after effects. Item stack changes need corresponding food invalidation before later agent selection. |
| `sim-systems/src/lib.rs:337,343,362,373,374,375,460,466,467` | shared / replace registration carefully | World dispatch separates decision only; common registration still includes regen/deposit/consume. Retain baseline system names, order and serialized state. |
| `sim-systems/src/save/mod.rs:73,260,395,402,405,410,419,426,438,441,447,495` | shared / replace versioned save format; keep legacy codecs | FORMAT_VERSION=12; decoded resources/content/mind validation then world build, saved-policy-aware registration, legacy system-slot migration and install_saved. New food state requires format bump and explicit decode migration/refusal, plus system-state-list migration if adding systems. |


## Exact direct production sites

Supplement to the semantic rows: complete executable/declaration reference sites for `food_tiles`, `food_source_max`, `set_food_tile`, `set_food_source_max`, `stock_in_hand`, `ResourceKind::Food`, and food-relevant infinite-source branches. Imports, comments, tests and unrelated infinite water/sleep uses excluded. Lines on field declarations/serialization/init are included because they establish state ownership.

| File | Sites | Classification / verdict |
|---|---|---|
| `sim-bridge/src/ffi/engine_build.rs` | 216,243,244 | shared / replace colony; keep baseline |
| `sim-bridge/src/ffi/reads_settlement.rs` | 113 | shared / replace colony; keep baseline |
| `sim-bridge/src/ffi/snapshot/resources.rs` | 46,61,63,80,81,224,225,226,247,253 | shared / replace colony; keep baseline |
| `sim-bridge/src/ffi/snapshot.rs` | 270 | shared / replace colony; keep baseline |
| `sim-bridge/src/ffi/ui_facts/economy.rs` | 32 | shared / replace colony; keep baseline |
| `sim-core/src/components/porter.rs` | 29 | shared / replace colony; keep baseline |
| `sim-engine/src/legacy_v7.rs` | 105,126,346,349 | baseline / keep baseline |
| `sim-engine/src/lib.rs` | 290,428,449,795,798,885,887,889,918,920,922 | shared / replace colony; keep baseline |
| `sim-systems/src/runtime/agent/movement.rs` | 366,517,568 | shared / replace colony; keep baseline |
| `sim-systems/src/runtime/agent_actions/consume.rs` | 24,31,33,37 | shared / replace colony; keep baseline |
| `sim-systems/src/runtime/agent_actions/forage.rs` | 35,41,85,89 | shared / replace colony; keep baseline |
| `sim-systems/src/runtime/agent_actions/needs.rs` | 27 | shared / replace colony; keep baseline |
| `sim-systems/src/runtime/baseline/decision/system.rs` | 219,390,548,667,969,1018,1081,1100,1177,1205,2110,2121,2221,2239,2863,2870,3170,3337,3343,3350,3361,3373,3647,3664,3793,3807,3818,3937 | baseline / keep baseline |
| `sim-systems/src/runtime/mind/behaviours/gather.rs` | 52,109,207,209,212,218,223 | colony / delete legacy gather |
| `sim-systems/src/runtime/mind/behaviours/needs.rs` | 36 | colony / replace |
| `sim-systems/src/runtime/mind/behaviours/postpass.rs` | 75,84,183,232,243,244 | colony / replace |
| `sim-systems/src/runtime/mind/behaviours/prepare.rs` | 33 | colony / replace |
| `sim-systems/src/runtime/mind/context.rs` | 205,299,395,425,620,796,820,840,846,918,939,1135,1202 | colony / replace |
| `sim-systems/src/runtime/mind/facts.rs` | 404,405,423,710,1164,1168 | colony / replace |
| `sim-systems/src/runtime/mind/system.rs` | 650 | colony / replace |
| `sim-systems/src/runtime/neighbours/mod.rs` | 264 | colony / replace |
| `sim-systems/src/runtime/resource_regen/mod.rs` | 159,413,431,444,588,607 | shared / replace colony; keep baseline |
| `sim-systems/src/runtime/settlement/fission.rs` | 271,333,420,706,979,991 | shared / replace colony; keep baseline |
| `sim-systems/src/runtime/settlement/food_economy.rs` | 20,77,85,91,148 | shared / replace colony; keep baseline |
| `sim-systems/src/runtime/settlement/relations.rs` | 504 | shared / replace colony; keep baseline |
| `sim-systems/src/runtime/settlement/settlement_system.rs` | 958,996,1254 | shared / replace colony; keep baseline |
| `sim-systems/src/runtime/settlement/shelter_planning.rs` | 101,102 | shared / replace colony; keep baseline |
| `sim-systems/src/runtime/settlement/stockpile_consume.rs` | 188 | shared / replace colony; keep baseline |
| `sim-systems/src/runtime/settlement/stockpile_deposit.rs` | 134,144,160 | shared / replace colony; keep baseline |
| `sim-systems/src/runtime/steward/planning.rs` | 310 | colony / replace |
| `sim-systems/src/runtime/wildlife/mod.rs` | 808 | colony / delete conversion |
| `sim-systems/src/worldgen/camp.rs` | 296,297,326,327,491,529,550,551,562,563,599,600 | shared / replace colony; keep baseline |
| `sim-systems/src/worldgen/colony.rs` | 113,114,484,485,575,587,588,604,605,716 | colony / replace |

The shared registration calls `ResourceRegenSystem`, `StockpileDepositSystem`, `StockpileConsumeSystem` in both worlds (`sim-systems/src/lib.rs:362,373-375`). Only decision selection differs at `:466-467`. Neither stockpile tick has a colony early return. Camp seeding is likewise shared (`worldgen/mod.rs:234-248`, `camp.rs:239`); only colony extras are gated at `worldgen/mod.rs:251`. Never globally delete these helpers or change baseline call order.

## Food fact and target inventory

All entries are colony mind observations. File: `sim-systems/src/runtime/mind/facts.rs`. Hunger numeric/memory facts stay observations with existing units; replace their food availability dependencies, not the selector. Unused registered facts are included.

| Fact/target group and exact declaration sites | Verdict |
|---|---|
| `routine_hearth_circle_preference`:259; `hunger_actionable`:361; `stocked_camp_in_radius`:372; `camp_stocked`:389; `within_camp_radius`:392; `stock_in_hand`:404; `stock_hoard_order`:412; `stock_shortfall`:435; `food_carry_loaded`:445; `hearth_available`:469; `inventory_food`:668; `food_available`:697; `at_food`:709; `target_food`:718; `hearth_lit`:731; `routine_hearth_circle_available`:773 | replace item-backed food observations; retain pure provider |
| `hunger_tenths`:289; `hunger_threshold_tenths`:298; `hunger_excess_tenths`:348; `hunger`:504; `hunger_threshold`:510; `hunger_critical_order`:536; `hungry`:555; `memory_hunger`:785; `recallable_hunger`:790 | retain hunger measurement/memory |
| `forage_window`:438; `preempt_gathering_enabled`:533; `gather_available`:706; `target_gather`:727; `running_foraging`:761; `forage_available`:778 | delete legacy gather policy or replace with job observation |
| Targets: `food`:1160-1170; `gather_food`:1180-1182; `forage_bush`:1184-1186; `forage_candidate`:1214 | replace food target; delete legacy gather/forage targets in favor of plant jobs |

Lifecycle registration: `behaviours/mod.rs:177-184` eat → needs callbacks; `:223-228` hearth_meal → owned idle callbacks (actual feeding is stockpile relief); `:257-269` forage/gather → gathering callbacks. `needs.rs:64-80` must preserve successful-only meal provenance and invalidation. `context.rs:918` + `system.rs:650` coordinate same-tick food changes. `postpass.rs` handles deferred targets; do not leave its old food-map branches reachable.

## Complete food-related node IDs grouped by effect

File: `sim-core/data/colony/mind.ron`. Each entry is `line:id`. A node is included if it uses a food/hunger/camp/stock/forage/gather fact or one of the four direct food behaviours. Broad guards affect nonfood behaviours, so all those IDs are retained here. All rows are colony; verdict belongs to the whole row.

| Effect / guards / verdict | Complete IDs |
|---|---|
| `socialise` (40); retain effect; replace food-availability/carry/camp guards | 9:finish_social_need_hunger; 336:critical_hunger_idle_hunger_memory_social; 394:critical_hunger_idle_thirst_memory_social; 434:critical_hunger_idle_fatigue_memory_social; 456:critical_hunger_idle_social; 536:critical_thirst_idle_hunger_memory_social; 597:critical_thirst_idle_thirst_memory_social; 639:critical_thirst_idle_fatigue_memory_social; 662:critical_thirst_idle_social; 740:camp_critical_water_idle_memory_social; 812:critical_hunger_routine_hunger_memory_social; 867:critical_hunger_routine_thirst_memory_social; 905:critical_hunger_routine_fatigue_memory_social; 926:critical_hunger_routine_social; 1002:critical_thirst_routine_hunger_memory_social; 1060:critical_thirst_routine_thirst_memory_social; 1100:critical_thirst_routine_fatigue_memory_social; 1122:critical_thirst_routine_social; 1196:camp_critical_water_routine_memory_social; 1264:critical_hunger_job_hunger_memory_social; 1316:critical_hunger_job_thirst_memory_social; 1352:critical_hunger_job_fatigue_memory_social; 1372:critical_hunger_job_social; 1444:critical_thirst_job_hunger_memory_social; 1499:critical_thirst_job_thirst_memory_social; 1537:critical_thirst_job_fatigue_memory_social; 1558:critical_thirst_job_social; 1628:camp_critical_water_job_memory_social; 1789:need_idle_hunger_memory_social; 1841:need_idle_thirst_memory_social; 1877:need_idle_fatigue_memory_social; 1897:need_idle_social; 1996:need_routine_hunger_memory_social; 2045:need_routine_thirst_memory_social; 2079:need_routine_fatigue_memory_social; 2098:need_routine_social; 2191:need_job_hunger_memory_social; 2237:need_job_thirst_memory_social; 2269:need_job_fatigue_memory_social; 2287:need_job_social |
| `eat` (17); replace item consumption/selection | 57:finish_food_crossover_thirst_source_missing; 75:finish_food_crossover_thirst_below_limit; 94:finish_food_crossover_fatigue_source_missing; 113:finish_food_crossover_fatigue_below_limit; 281:critical_hunger_idle_hunger; 478:critical_thirst_idle_hunger; 702:camp_critical_water_idle_memory_hunger; 760:critical_hunger_routine_hunger; 947:critical_thirst_routine_hunger; 1160:camp_critical_water_routine_memory_hunger; 1215:critical_hunger_job_hunger; 1392:critical_thirst_job_hunger; 1594:camp_critical_water_job_memory_hunger; 1740:need_idle_hunger; 1950:need_routine_hunger; 2148:need_job_hunger; 2305:continue_food |
| `drink` (25); retain effect; replace food-availability/carry/camp guards | 133:finish_water_crossover_hunger_source_missing; 150:finish_water_crossover_hunger_below_limit; 168:finish_water_crossover_fatigue_source_missing; 187:finish_water_crossover_fatigue_below_limit; 298:critical_hunger_idle_hunger_memory_thirst; 356:critical_hunger_idle_thirst; 496:critical_thirst_idle_hunger_memory_thirst; 557:critical_thirst_idle_thirst; 685:camp_critical_water_idle; 776:critical_hunger_routine_hunger_memory_thirst; 831:critical_hunger_routine_thirst; 964:critical_thirst_routine_hunger_memory_thirst; 1022:critical_thirst_routine_thirst; 1144:camp_critical_water_routine; 1230:critical_hunger_job_hunger_memory_thirst; 1282:critical_hunger_job_thirst; 1408:critical_thirst_job_hunger_memory_thirst; 1463:critical_thirst_job_thirst; 1579:camp_critical_water_job; 1755:need_idle_hunger_memory_thirst; 1807:need_idle_thirst; 1964:need_routine_hunger_memory_thirst; 2013:need_routine_thirst; 2161:need_job_hunger_memory_thirst; 2207:need_job_thirst |
| `forage` (6); delete legacy nodes; harvest via jobs | 207:finish_forage_critical_hunger; 223:finish_forage_critical_thirst; 2353:continue_forage; 2379:forage_idle; 2422:forage_routine; 2463:forage_job |
| `gather` (6); delete legacy nodes; harvest via jobs | 239:finish_gather_critical_hunger; 255:finish_gather_critical_thirst; 2366:continue_gather; 2401:gather_idle; 2443:gather_routine; 2483:gather_job |
| `sleep` (30); retain effect; replace food-availability/carry/camp guards | 317:critical_hunger_idle_hunger_memory_fatigue; 374:critical_hunger_idle_thirst_memory_fatigue; 415:critical_hunger_idle_fatigue; 516:critical_thirst_idle_hunger_memory_fatigue; 576:critical_thirst_idle_thirst_memory_fatigue; 619:critical_thirst_idle_fatigue; 721:camp_critical_water_idle_memory_fatigue; 794:critical_hunger_routine_hunger_memory_fatigue; 848:critical_hunger_routine_thirst_memory_fatigue; 887:critical_hunger_routine_fatigue; 983:critical_thirst_routine_hunger_memory_fatigue; 1040:critical_thirst_routine_thirst_memory_fatigue; 1081:critical_thirst_routine_fatigue; 1178:camp_critical_water_routine_memory_fatigue; 1247:critical_hunger_job_hunger_memory_fatigue; 1298:critical_hunger_job_thirst_memory_fatigue; 1335:critical_hunger_job_fatigue; 1426:critical_thirst_job_hunger_memory_fatigue; 1480:critical_thirst_job_thirst_memory_fatigue; 1519:critical_thirst_job_fatigue; 1611:camp_critical_water_job_memory_fatigue; 1772:need_idle_hunger_memory_fatigue; 1823:need_idle_thirst_memory_fatigue; 1860:need_idle_fatigue; 1980:need_routine_hunger_memory_fatigue; 2028:need_routine_thirst_memory_fatigue; 2063:need_routine_fatigue; 2176:need_job_hunger_memory_fatigue; 2221:need_job_thirst_memory_fatigue; 2254:need_job_fatigue |
| `do_job` (5); retain effect; replace food-availability/carry/camp guards | 1657:job_finish_craft; 1672:job_finish_delivery_leg; 1688:job_below_yield_margin; 2117:meal_job_no_water; 2132:meal_job_water_not_critical |
| `hearth_meal` (4); replace item consumption/selection | 1705:meal_idle_no_water; 1722:meal_idle_water_not_critical; 1917:meal_routine_no_water; 1933:meal_routine_water_not_critical |
| `routine_tend_fire` (1); retain effect; replace food-availability/carry/camp guards | 2527:enter_routine_tend_fire |
| `routine_craft` (1); retain effect; replace food-availability/carry/camp guards | 2559:enter_routine_craft |
| `routine_repair` (1); retain effect; replace food-availability/carry/camp guards | 2591:enter_routine_repair |
| `routine_hearth_circle` (1); retain effect; replace food-availability/carry/camp guards | 2623:enter_routine_hearth_circle |
| `routine_explore` (1); retain effect; replace food-availability/carry/camp guards | 2655:enter_routine_explore |
| `routine_play` (1); retain effect; replace food-availability/carry/camp guards | 2687:enter_routine_play |
| `routine_sit_by_fire` (1); retain effect; replace food-availability/carry/camp guards | 2719:enter_routine_sit_by_fire |
| `routine_no_place` (1); retain effect; replace food-availability/carry/camp guards | 2738:routine_no_place |
| `wander` (1); retain effect; replace food-availability/carry/camp guards | 2757:wander |

## Coverage assertion and exceptions

- All 157 authored nodes inspected; 142 food-related nodes listed, including direct eat=17, hearth_meal=4, forage=6, gather=6. Food-relevant registered fact declarations and target resolver branches are listed even when not used by policy.
- Exact-site supplement contains 159 production declaration/use sites across the seven substrate/inventory search terms. Semantic map additionally follows generic `Settlement::store/withdraw`, `PorterKind::resource`, food-economy callers, wildlife conversion, migration, save/registration and bridge consumers that literal searches miss.
- Raw audit preserves reproducible overlapping query counts: direct 874 lines/107 files; indirect 279/73; food mind 953/48; migration/registration 153/37. This supplement removes docs, imports, comments and test bodies; those raw counts are not production-site counts.
- Sentinel exceptions: RESOURCE_SOURCE_INFINITE for water/sleep remains; generic regen sentinel guard remains for baseline channels. Building sleep registration and unrelated wood branches are intentionally excluded. Generic Inventory and Settlement containers remain shared; only Food use is replaced in colony.
- Coverage is a source-audit assertion over this snapshot, not a dynamic call-graph proof or gate result. Alias mutation paths were semantically inspected; future implementation must re-run the direct and generic-store searches. File line numbers can drift with concurrent edits.
- Save format is 12 at this snapshot; format bump plus explicit migration/refusal and system-list compatibility are required. Baseline historical codecs/fixtures are preserved. Root owns identity, runtime, ledger, save/load, data-only and full-gate evidence.

## External evidence hashes

- `census-validation.json`: `86af76c13a81ff479cc81cd1bcff7a00046a355932006a77c7e226882169f04c`.
- `base-catalog-manifest.json`: `eddc973ee61f8b9f90e8524955c7f3568bfbbd8a83acf2c25a4b8dacfb3454a1`.
- `placement/contract.json`: `70584a752e6b55c0acc022bb94ecfad85614862b0554b42e3be83d2ebec2e6f8`.
- `placement/evaluate.py`: `7255e4b03f593d0ab00c91c860ecf6309e06e96fee5ae46987fb21b8eb3ff063`.
- `placement/evaluate_v2.py`: `6986405398d4d27880551ab85000f6393220876d20909da55f7c670894088eb3`.
- `placement/result-v1.json`: `abaae74381e3f9c91c82d999de2b235a392f09f716914245e6b529c69618d5fc`.
- `placement/result-v2.json`: `2a60bb2610a1c951871ce8929e7d7b3d7f173aa6720aa590db30aa3396f972c5`.
- `placement/result-v2-ownerbar.json`: `1ebdae64fd6f13be99c3d912d2fe4a15868ee82eeca0fd83bbea90b8afd1358e`.
- `inventory-for-design.md`: `d7ab44dfd5886c3239327b9f3c6f6d2b6b09655fc8c25dbaf9bdd3520e89942b`.

Raw evidence root: `/Users/rexxa/work/m6a-data/20261029`. Product placement parity, mechanism/runtime tests, calibration, held-outs, performance and the final gate are NOT_RUN at this design commit.
