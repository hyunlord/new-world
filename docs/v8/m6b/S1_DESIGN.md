# Stage 1 — farming and bounded stewardship

Status: first implementation design, 2026-11-04; BASE seeds 0–7 measurements completed with actual exit 0 each. Calibration values remain revisable before held-out freeze. No gameplay implementation or held-out evaluation is claimed. Owner D-S0-2 stopped the generic-gate work; use the published M-LOOP/M-READ gate. BASE0 is `2f6993009ddb600e9249c28f9768a1a5252c8821`. The existing m6a observer at `509ce99d1` differs in its accepted horizon only (72→96); simulation/data bytes match BASE0. Raw measurement stays in `/Users/rexxa/work/m6b-data/stage1/`.

## Food and labour arithmetic

Calendar:1,440ticks/day,6days/season,24days/year. Warm adult consumption2.4food units/day; child1.44; existing cold multiplier1.5 must be included in actual population demand.

| Warm adults | Daily food | Annual food | Ideal full-food emmer tiles | Ideal30% tiles |
|---:|---:|---:|---:|---:|
|10|24|576|39|12|
|20|48|1,152|77|24|
|40|96|2,304|154|47|

Provisional emmer:12grain/tile/annual harvest,2grain retained as next seed;10grain funds five real2→3 porridge batches, at most15edible units before spoilage or missed work. The table uses ceil(annual need/15) and ceil(0.3×annual need/15); it is a theoretical upper yield, not measured intake. Actual automatic area derives from demand, observed dependable wild supply and calibrated effective net crop yield, then has a hard1.25×area ceiling including accepted uncompleted field orders.

Provisional per-tile work: till60ticks+sow30+tend30+harvest60=180ticks (3worker-hours), plus actual seed/harvest hauling and five recipe batches.10people at39tiles require117field hours/year before hauling/cooking, versus1,920hours at8hours/person/day. Job delivery and hunger/cold interruptions are measured, not assumed available. Initial30% area12tiles costs36field hours before those overheads.

## Required BASE numbers before policy changes

Run seeds0–7 for96days, steward/defence/story/neighbours ON, no player orders. Existing m6a daily snapshots and per-kind cumulative harvested tallies give food plant production. Derive plant yield kinds from the content registry: current plant-food kinds and animal meat are disjoint. Exclude meat from wild plant totals. Nonfood flax is absent from food tallies and must not be reported as zero. Convertible grain is not raw-edible food or measured meals.

For each seed and display-year [0,24),[24,48),[48,72),[72,96), report plant pieces by kind, raw-edible and convertible nominal units, total cooked/eaten/expired units, alive and knowledge-loss counts. Record actual observed coverage and partial boundary days. Wild-origin eaten supply remains unmeasured until origin tracking qualifies; it must not be inferred as all nonfield food. Calibration output and tables will be linked here before gameplay changes.

The #674 numerical diagnosis compares usable stores plus accepted pending production against actual planner targets, recording first overage, peak and consecutive days. Existing harvest planner permits diversity/ingredient orders after both immediate/winter gaps reach zero; demonstrate the quantity before correcting that admission. A completed harvest's indivisible quantum is distinguished from repeated surplus orders.

## First gameplay slice: production and storage (#674/#684)

Use one budget for usable food plus accepted pending harvest/cooking output. Cancelled/rejected/unreachable work is not a reserve. Designation plus assigned job is one commitment. Proactive diversity/ingredient selection cannot bypass a full budget; urgent individual hunger and explicit player orders retain their ordinary paths. Future standing crops are not currently edible stores.

Build an actual roofed Store through existing House/blueprint/ownership paths before proactive surplus harvest. Complete a bedless Store explicitly; do not manufacture rooms, require phantom beds or override player roles. Immediate meals remain obtainable while construction completes. Ordinary hauling must put food under the completed roof with compatible capacity.

Separate log construction demand from executable fuel conversion.1log→4firewood at30base work ticks. Nominal winter burn13hours×3=39firewood/day;6days need234, rounded59batches→236fuel/1,770work ticks. A1.25buffer would need74batches→296fuel/2,220ticks; this is a candidate calculation, not an adopted answer to cold snaps. Existing LOOP held-out failed snap safety, day5/day24logs and total knowledge loss; do not reuse those seeds.

Tools-only survival means actual tool bill actions only: the current Bills arm's unconditional winter-fuel append must be separated. No grace delay for survival tools. Teaching stays immediate and uses real teacher/order lifecycle; the card displays progress and offers explicit commands, with passive expiry.

## Cultivation mechanism and content

Data defines emmer/flax identities, seed input, outputs, planting windows, growth days, work ticks and fertility. No runtime content-name switches. A third crop loaded only through data must work. `cultivation` is neolithic, with foraging/grain_parching prerequisites and real discovery/practice/observation channels.

Field tiles are a sorted colony-only map. Phases: untilled, tilled, sown, growing, ripe, harvested, fallow. Fertility is integer and persists through erase/repaint; harvest depletes it and fallow recovers it. Erasing designation does not reset soil or destroy an established crop. Calendar updates are daily; jobs use existing navigation/reservation/do_job/cancellation.

Sowing consumes actual seeds once at completion. Emmer may use retained grain; flax requires a data-defined seed bootstrap from the existing wild plant chain. Harvest output is atomic or remains durable pending output on capacity failure. No retry duplicates or synthetic seed refunds. Crop growth, seed debit, fertility and origin metadata survive save/load; append command/job variants and migrate the exact old format15 body.

Steward sowing uses accepted area plus existing fields and seed availability. Seed reserves protect next sowing without trapping emergency food or explicit player commands. Field allocation must have a real labour/seed bootstrap and must not be permanently suppressed by full wild-food stores. Real work supplies cultivation practice; the farming era marker has weight3. With weaving still absent, the theoretical existing14weight denominator permits floor(12/14×1000)=857permille; real progression still needs practitioners.

## Product surfaces

Paint field zones through the existing map order input and validated commands. Draw read-only state as furrows, sprouts, ripe tint and stubble; seasonal tint and code-drawn crops are placeholders with an art follow-up issue. Bridge reads expose actual accepted fields, crop, phase and fertility.

Add sowing_window and harvest_ready cards, diagnosis for missed window/no seed/exhausted soil/no farmer, need_seed request, and first-sown/first-harvest chapter4 conditions. Record actual first field, first harvest and crop failure causes in chronicle. Do not unlock later weaving conditions prematurely. Card validation must retain zero rejected issued commands.

## Measurement and freeze

Field intake share must count actual eaten units in year3 [48,72). Preserve field contribution through splits/merges, cooking and storage; use bounded integer attribution and retain uncertainty rather than credit all mixed food as field-grown. This is one concrete missing game metric, not permission for a new observer framework.

#675 counts a day only when, at an actual expiry event in that settlement, the same living person is awake, above their seek threshold, and has no reachable edible food. Sleeping hunger, inedible grain, another settlement's stock and food only in an unreachable region are negative controls. Adopt the bar only if paired natural calibration discriminates; zero/zero remains diagnostic.

Freeze code/data/bars after calibration0–7×96days and before held-out1520–1551. Preserve spent1392–1519/1680–1743. Stage2 holds1552–1583; Stage3 holds1584–1615. Farming defaults to derive from measurements: BASE-relative survival within one world; years2–4 winter-hunger-death-free≥28/32; field sown byyear2≥28/32; year3 field intake≥30% in≥24/32; >1.5×foodtarget never lasts>6consecutive days in≥28/32; exact ledger32/32.

Owner #684 requirements: cold-snap firewood never zero among exposed worlds, compared with BASE; report shared and one-sided exposures so lost exposure is not a success. Freeze exposure scoring before held-out. Report loose logs atday5/day24 and construction/fuel demand separately. KnowledgeLost≤BASE+1 per world in≥30/32 and total≤BASE+3. Do not replace total with only per-world success.

Use the working legacy full gate for publication, retaining intended changed fixtures explicitly. New tooling must address an observed failure and its line count belongs in the report. The current measurement change adds zero tools lines. Report runtime/gameplay, visual, performance and human acceptance separately.

## First calibration configuration and attribution

Use a replacement crop floor of 30% of annual demand even when wild production covers demand: agricultural adoption replaces a share of gathered food rather than waiting for a famine. The initial effective net yield is 12 edible units/tile/year (15 theoretical after seed retention, discounted 20% for delivery/spoilage); this is a calibration assumption, not a measured field result. Ten/twenty/forty warm adults need 15/29/58 planned tiles, capped at 19/37/73 including accepted outstanding zones. Cold and children use actual demand arithmetic. No field-food reserve is credited until physical output exists. Available physical seeds constrain sowing, not the planned target; successful harvest retains the next cycle's two seeds per tile before proactive cooking, without blocking emergency intake.

Emmer uses two wild-grain pieces per sowing and produces twelve grain pieces when tended. Flax uses two physical flax seeds and produces six stalks plus four seeds. Its bootstrap is a data recipe consuming two wild flax stalks to produce four seeds in 30 ticks at an existing compatible workshop. No free starting seeds are injected. Both plant in spring days [0,3); growth takes four days for emmer and five for flax. Work per tile is 60 till, 30 sow, 30 tend, 60 harvest ticks. Untended output is 750 permille of each authored output (floor), representing real omitted work. Soil starts at 1000, requires at least 400 to sow, loses 200 per harvest, and regains 10 per fallow day up to 1000. Soil gates sowing rather than introducing a second hidden yield multiplier. These are data fields, not crop-name switches.

Fields persist an active designation and Auto/Now/Paused work mode. Erase sets active=false and cancels jobs but retains established crops and soil. Explicit Now resumes and prioritizes a feasible real job; it cannot bypass planting windows, knowledge, seed, navigation or work duration. Later pauses; steward returns Auto. A completed one-shot request returns to Auto. Tending is a real appended job, not a visual state.

For #674, the source-level counterexample has one adult and a five-unit reserve: five usable roots already satisfy the target, yet the diversity arm admits four berries, committing nine units (1.8× target). Three stored units plus an accepted five-unit root harvest already commit eight, yet the same arm can add four berries to reach twelve. The repair keeps one indivisible harvest quantum for a real shortage and preserves a distinct durable-winter horizon; it does not treat short-lived stock as winter security.

Field-origin attribution uses integer fractions bounded outward at one million parts. Pure field harvest is [1000000,1000000]; old-world/wild/traded food is [0,0]. Splitting preserves the fraction; merging equal items uses count-weighted fractions with floor(lower)/ceil(upper). A recipe uses nominal food-input-unit weights, and all its output carries that bounded fraction, including passive drying and capacity retries. This defines proportional attribution through mixing, not fictional physical grain identities. Only actual completed meals add to per-calendar-year total units and field lower/upper totals. The 30% bar compares the lower bound against total eaten units. Seed consumption is a separate persisted food sink, never eating or cooking. All origin state is in the version-16 cultivation tail; version-15 migration defaults its existing food to nonfield.

## Measured BASE wild production before implementation

All eight 96-day runs exited 0 on pinned probe `f44e3d5561d40a879879d00d1f7226255a10457e243816084071e409503137be` from `509ce99d1`. Raw output is 69,955,726 bytes outside the repository. Each cell below is raw-edible plant units / convertible raw-grain pieces. Meat is excluded. These are harvested production, not eaten supply.

| Seed | Year 1 | Year 2 | Year 3 | Year 4 |
|---:|---:|---:|---:|---:|
| 0 | 790 / 24 | 880 / 24 | 871 / 30 | 665 / 30 |
| 1 | 890 / 24 | 921 / 30 | 906 / 24 | 1059 / 30 |
| 2 | 852 / 30 | 963 / 30 | 976 / 30 | 770 / 24 |
| 3 | 917 / 30 | 914 / 30 | 551 / 24 | 1043 / 36 |
| 4 | 727 / 30 | 957 / 24 | 944 / 30 | 998 / 30 |
| 5 | 593 / 24 | 864 / 24 | 604 / 18 | 592 / 24 |
| 6 | 744 / 30 | 1006 / 30 | 1065 / 24 | 1183 / 24 |
| 7 | 856 / 30 | 1007 / 30 | 1079 / 30 | 1058 / 30 |

Source: `/Users/rexxa/work/m6b-data/stage1/base-calibration-20261104/display-year-plant-food.csv`; per-kind production, eaten, expired, alive and knowledge losses are in that table. Daily boundaries and terminal totals remain alongside it. Display-year snapshots close at days 24/48/72/96; the initial clock offset and final 480-tick tail mean these are explicitly display-year production windows, not falsely exact [0,24) elapsed-time windows. Only seed 5 loses knowledge (one event in year 3). The corrected #675 predicate is not measured by this old observer.

Observed plant output is 551–1183 units per display-year, already comparable to or above ten adults' 576 warm annual demand. Therefore a deficit-only field rule would often plan zero fields and cannot meet the owner's adoption/intake bars. The 30% replacement floor above is necessary; it does not increase the edible-store target. Actual wild supply eaten remains unknown until attribution exists. Existing overproduction and spoilage make increasing total gathering an unjustified response.
