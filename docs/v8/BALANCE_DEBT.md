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
