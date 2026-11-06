# Balance debt

D-FEATURE-FIRST (2026-11-06): these are authored starting values, not outcome acceptance bars. Do not tune against smoke seeds or start calibration/held-out loops. Technical conservation, replay, compatibility and collapse checks remain blocking.

## Stage 2 weaving

Knowledge capacity correction: spinning includes fibre preparation, leaving the existing32-bit mask at31bundled nodes and retaining bit31 for selected-pack content such as `test_hide_craft`. Legacy indices0–27 stay unchanged; spinning28, weaving29 and tailoring30 are appended. Discovery requires raw stalks rather than the gated process's fibre output, avoiding a prerequisite deadlock. This fixes content-family integration capacity; it is not an outcome-directed balance change.

| Feature | Tunable | Current value | Intended effect | Suspected issue |
|---|---|---|---|---|
| Retting | Input/output/work | 2 flax stalks → 2 fibre; 30 active ticks | One physical fibre preparation batch | Labour versus future systems uncalibrated |
| Retting | Passive duration/capacity/wet requirement | 2 qualifying days; 1 pending batch per pit; cardinal river required | A small, visible wet process that pauses when dry | River access and throughput may limit clothing supply |
| Spinning | Input/output/work/tool | 2 fibre → 2 thread; 60 ticks; 1 reusable spindle | Preserve physical tool ownership without consuming it or knife wear | Reusable tool lifetime is unbounded in this starting model |
| Weaving | Input/output/work | 2 thread → 1 cloth; 90 ticks | Make cloth from real spun thread | Shared loom congestion with tailoring uncalibrated |
| Tunic | Input/output/work | 2 cloth → 1 tunic; 120 ticks | Main full textile chain | Production capacity versus population uncalibrated |
| Wrap | Input/output/work | 1 cloth → 1 wrap; 60 ticks | Cheaper alternate garment | Relative usefulness versus tunic uncalibrated |
| Spindle | Input/output/work | 1 log → 1 spindle; 30 ticks | Accessible reusable tool on the existing wood bench | No autonomous textile tool target |
| All textile crafts | Skill/work/quality adjustment | Existing woodwork/craft; quality_add 0; no material term | Reuse mechanical work channels without a new skill enum | Woodworking proficiency also helps textiles |
| Retting pit | Footprint/material/build work | 1×1; 2 stone; 30 ticks | Dry accessible pit beside river with south work tile | Site accessibility can constrain use |
| Loom | Footprint/material/build work | 1×1; 4 logs; 90 ticks | One small shared weaving/tailoring station | Queue contention uncalibrated |
| Fibre/thread | Stack/carry/value | 30/10/2 each | Conventional physical intermediate inventory | Transport cost uncalibrated |
| Cloth | Stack/carry/value | 30/10/4 | Conventional inventory for two-garment recipes | Trade value uncalibrated |
| Spindle | Stack/carry/value | 10/2/3 | Small reusable workshop tool | Stocking and trade value uncalibrated |
| Tunic | Stack/carry/value | 1/1/8 | Preserve distinct garment condition | Extra hauling versus stackable material |
| Wrap | Stack/carry/value | 1/1/4 | Preserve distinct garment condition | Extra hauling versus stackable material |
| Tunic | Wear/cold/comfort/warmth rank/snap immunity | 48 worn days; cold600‰; mood+20‰; rank1; false | Modest durable clothing; fur stays better for cold | Comfort versus warmth preference uncalibrated |
| Wrap | Wear/cold/comfort/warmth rank/snap immunity | 24 worn days; cold700‰; mood+10‰; rank1; false | Shorter-lived inexpensive clothing | Same cold protection as hide wrap; replacement effort uncalibrated |
| Legacy garments | Wear/comfort | None/0 | Preserve hide/fur behaviour | Historical unlimited wear is intentionally retained |
| Textile knowledge | Practitioner threshold/difficulty/seeding | 300‰ / difficulty1 / unseeded for all three nodes | Discoverable, teachable work through existing channels | Natural adoption speed is not evaluated |
| Spinning and fibre preparation | Discovery | Cultivation prerequisite; stockpile2 raw flax stalks | Unlock retting, spindle and spinning before requiring their outputs | Knowledge loss can interrupt the chain; requires deliberate stockpiling |
| Weaving | Discovery/era weight | Spinning prerequisite; stockpile2 thread; weight2 | Actual woven craft practice fills the previous absent marker | No adoption target; total era denominator stays14 |
| Tailoring | Discovery | Weaving prerequisite; stockpile1 cloth | Garment recipes follow physical cloth production | Shared loom/teaching labour uncalibrated |
| Existing flax field | Yield/work | 6 stalks/tended tile; 60 till +30 sow +30 tend +60 harvest ticks | Existing Stage1 crop supplies the chain unchanged | Untended yield750‰, frost, fertility and labour reduce realised supply |

Starting arithmetic: one tunic consumes4stalks and480active processing ticks (2retting×30 +2spinning×60 +2weaving×90 +120tailoring). One wet batch takes30active ticks +2×1440qualifying ticks =2910ticks before output placement. Field work for N tunics is180×ceil(4N/6) ticks with tended yields, excluding hauling, tool/build work and knowledge acquisition. A wrap consumes2stalks and240active processing ticks. With the current24-day year, tunic replacement is N/2garments/year and wrap replacement is Ngarments/year: both require2Nstalks and240Nactive processing ticks/year in steady state, plus field work180×ceil(2N/6). Initial outfitting and blocked output time are extra. Cold600/700‰ scales a qualifying cold bite, not the chance that fuel or shelter runs out; neither linen garment grants snap-work immunity. These are arithmetic assumptions, not performance or balance guarantees.

## Stage 3 temperature and dwellings

Initial authored values are in `rust/crates/sim-core/data/colony/thermal.ron`; the model and reference arithmetic are in [S3_DESIGN](m6b/S3_DESIGN.md). These are uncalibrated starting values. No natural comfort percentage, cold-death improvement or held-out outcome is a publication bar.

| Feature | Tunable | Current value | Intended effect | Suspected issue |
|---|---|---|---|---|
| Outdoor climate | Season bases | 120,220,100,−30 deci-degrees | Distinguish seasons | Regional variation absent |
| Outdoor climate | Hour offsets | −20×6,−10,0,10,20,30,40,40,40,30,20,10,0,−10,−20×5 | Integer day/night swing | Smoothness and winter daytime cost uncalibrated |
| Cold snap | Temperature offset | −50 deci-degrees | Physical cold pressure | Extreme-weather severity versus later shelter systems uncalibrated |
| Felt cold | Threshold | 50 deci-degrees | Derive colony Cold from place plus worn insulation | Clothing/food/exposure interactions require later balance |
| Room storage | Capacity per tile | 750 | Slow hourly temperature response | Large-room warm-up time uncalibrated |
| Room envelope | Roof/floor loss per tile | 100 | Nonzero loss for enclosed area | No distinct roof/floor material insulation yet |
| Material envelope | Insulation permille | oak900,pine880,birch870,granite700,limestone750,flint650,chert650 | Actual wall material changes heat retention | Thermal values are gameplay choices, not physical measurements |
| Natural boundary | Loss per edge | 300 | Explicit terrain enclosure fallback | Terrain variants share one initial value |
| Doors | Closed/open loss | 250/9000 | Physical open doors lose more heat | Traffic can dominate small-room heat loss |
| Doors | Passage duration | 2 ticks | Actual passage opens briefly | Traffic/animation timing uncalibrated; overlapping intervals are a union |
| Paid primary hearth | Rated heat | 450000 per hourly integration | Warm only its actual containing room after positive full fuel payment | One heater per settlement; no separate room heaters or radiant outdoor benefit |
| Occupants | Adult/child heat | 4500/2500 | Actual bodies contribute heat | Occupancy timing is sampled at the hourly boundary |
| Apparel | Felt insulation | hide20,fur50,linen tunic30,linen wrap20 deci-degrees | Existing physical garments affect felt temperature | Protection values are not inferred from old cold_permille |
| Dwelling stages | Minimum beds/floor coverage | 1 bed;0/1000/1000/1000permille | Real shelter then complete eligible floor | Permanently occupied fixture/work cells cannot accept floors; denominator must reflect legal geometry |
| Warm dwelling | Paid heat/warm occupants | Required for ranks3/4 | Warmth must be physically supported and observed | Sparse/absent occupancy does not prove warm housing |
| Provisioned dwelling | Food/distance | 1 adult-day; navigation distance8 | Reachable real food near the dwelling | Shared food and resident demand require later balance |
| Dwelling hysteresis | Continuous deficiency delay | 2880ticks for all stages | Prevent immediate downgrade on brief shortages | Two-day delay is an initial choice; roof loss still removes physical shelter immediately |
| Dwelling mood | Stage effects | +10/+20/+30/+40permille | Reflect actual dwelling quality once per resident | M7 debt: stage mood interpretation remains code; interaction with other mood sources uncalibrated |
| Physical refuge | House.hearth_refuge_width | Some(3), defaultNone | Start at3×3minimum actual interior and select the smallest legal bounded enclosure containing the actual primary hearth | Initial shelters can require a3×4interior; retained terrain may shape the actual component, and ordinary construction/access/fuel still constrain availability |
| Heating service | Daytime fuel rate | Existing winter fuel rate and active-pressure multiplier | Pay once for an occupied cold primary-hearth room outside the normal burn window | Continuous service increases physical fuel demand; forecast must cover the same window |
| Refuge need | Cold-entry critical need boundary | Existing750 tenths; existing owners yield to actionable needs at the survival healing prerequisite | Reach shelter, then complete actual eating/drinking needed for healing | Travel and recovery interaction remains uncalibrated; no need or health is reset |
| Refuge recovery | Admission/release | Cold injury or warm injured idle resumption; existing SAFE_NEED_CEILING50 for both needs; release at full health | Use actual damage and existing healing as hysteresis; finish ordinary need actions | Full recovery can occupy room seats for long periods; essential fuel service remains physical work |

A real scripted one-hearth scene and the technical collapse floor are required. Smoke-01 failed all8 worlds before physical refuge construction, warmth-seeking and daytime heating were connected; S3-INTEGRATION-1 records that implementation correction without adjusting heat coefficients or exposure damage. No room-to-room heat exchange, heater-capacity tuning or natural population comfort claim is included.


Stage3 **M6B-S3-SAFETY (unpublished)**: `f24a7d124` smoke-04 fails seven worlds to extinction, KnowledgeLost163 versus BASE0/allowed2. Actual route/ownership/damage traces establish prolonged Cold on valid shortest trips and Cold inside some fully paid rooms. The initial values above are not safe for publication. This is a publication blocker under D-FEATURE-FIRST, not a waived improvement bar. No values were tuned; further model/balance work must preserve the failed evidence and re-establish technical safety. See M6B_DECISIONS S3-REVIEW-1.
