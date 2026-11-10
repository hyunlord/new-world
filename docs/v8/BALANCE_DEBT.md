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

## C1-a neolithic content (2026-11-08)

These are the commented starting values in `rust/crates/sim-core/data/colony/{knowledge,items,recipes,plants,crops,animals,neighbours,blueprints,buildings}.ron`. No smoke outcome, calibration or held-out result was used to tune them. `Connected` describes the implemented effect, not balance acceptance; `CONTENT_DEBT.md` records absent effects separately. Milk and wool identities do not create a C1-a livestock supply. Inert medicine, rotation and social nodes have no gameplay effect.

### Physical recipes

All new recipes use `work: craft`, `quality_add: 0`, `material_term: None`, `tool: None`, `auto_bill_role: None`; no automatic production target is added. Work counts are active ticks. Pottery/woodwork/knap skill reuse and station contention are unbalanced. Explicit knowledge requirements retain closed recipes when their node is Inert.

| Feature | Tunable/current value | Intended effect | Suspected issue |
| --- | --- | --- | --- |
| grind_flour | [("wild_grain", 2)] → ("flour", 1); 60 ticks; grinding_stone/woodwork; knowledge ["querns"] | Physical input consumption and output production | Supply and labour return unbalanced |
| bake_bread | [("flour", 2), ("firewood", 1)] → ("bread", 3); 90 ticks; clay_oven/pottery; knowledge ["oven"] | Physical input consumption and output production | Supply and labour return unbalanced |
| brew_beer | [("barley", 3)] → ("beer", 2); 90 ticks; pottery_bench/pottery; knowledge ["fermentation"] | Physical input consumption and output production | Supply and labour return unbalanced |
| ferment_milk | [("milk", 2)] → ("cheese", 1); 90 ticks; pottery_bench/pottery; knowledge ["dairy"] | Physical input consumption and output production | C1-a has no livestock milk producer; supply follows C1-b |
| smoke_meat | [("meat", 2), ("firewood", 1)] → ("smoked_meat", 2); 90 ticks; smoking_rack/pottery; knowledge ["food_preservation"] | Physical input consumption and output production | Supply and labour return unbalanced |
| weave_basket | [("cord", 3)] → ("basket", 1); 60 ticks; wood_bench/woodwork; knowledge ["basketry"] | Physical input consumption and output production | Carry bonus absent; M-BAL |
| make_hoe | [("flint_blade", 1), ("log", 1), ("cord", 1)] → ("hoe", 1); 90 ticks; wood_bench/woodwork; knowledge ["hoe"] | Physical input consumption and output production | Crafting only; tool efficiency absent; M-BAL |
| make_sickle | [("flint_blade", 1), ("log", 1), ("cord", 1)] → ("sickle", 1); 90 ticks; wood_bench/woodwork; knowledge ["sickle"] | Physical input consumption and output production | Crafting only; tool efficiency absent; M-BAL |
| make_adze | [("flint_blade", 1), ("log", 1), ("cord", 1)] → ("adze", 1); 90 ticks; wood_bench/woodwork; knowledge ["adze"] | Physical input consumption and output production | Crafting only; tool efficiency absent; M-BAL |
| make_bow | [("log", 1), ("cord", 2)] → ("bow", 1); 90 ticks; wood_bench/woodwork; knowledge ["bow"] | Physical input consumption and output production | Crafting only; ranged combat/ammunition absent; M9 |
| make_arrows | [("flint", 1), ("log", 1)] → ("arrow", 5); 60 ticks; knapping_spot/knap; knowledge ["bow"] | Physical input consumption and output production | Crafting only; ranged combat/ammunition absent; M9 |
| dye_tunic | [("linen_tunic", 1), ("berries", 2)] → ("dyed_tunic", 1); 60 ticks; pottery_bench/pottery; knowledge ["dyeing"] | Physical input consumption and output production | Supply and labour return unbalanced |
| make_leather_armor | [("hide", 3), ("cord", 2)] → ("leather_armor", 1); 120 ticks; drying_rack/knap; knowledge ["leather_armor"] | Physical input consumption and output production | Supply and labour return unbalanced |
| prepare_poultice | [("herbs", 2)] → ("poultice", 1); 45 ticks; pottery_bench/pottery; knowledge ["herbalism"] | Physical input consumption and output production | Inert herbalism: no healing or natural herb supply; M9 |
| save_barley_seed | [("barley", 2)] → ("barley_seed", 2); 45 ticks; wood_bench/woodwork; knowledge ["barley"] | Physical input consumption and output production | Supply and labour return unbalanced |
| save_rye_seed | [("rye", 2)] → ("rye_seed", 2); 45 ticks; wood_bench/woodwork; knowledge ["rye"] | Physical input consumption and output production | Supply and labour return unbalanced |
| save_lentil_seed | [("lentil", 2)] → ("lentil_seed", 2); 45 ticks; wood_bench/woodwork; knowledge ["lentil"] | Physical input consumption and output production | Supply and labour return unbalanced |
| save_pea_seed | [("pea", 2)] → ("pea_seed", 2); 45 ticks; wood_bench/woodwork; knowledge ["pea"] | Physical input consumption and output production | Supply and labour return unbalanced |
| save_millet_seed | [("millet", 2)] → ("millet_seed", 2); 45 ticks; wood_bench/woodwork; knowledge ["millet"] | Physical input consumption and output production | Supply and labour return unbalanced |
| make_sling | [("cord", 2), ("hide", 1)] → ("sling", 1); 60 ticks; wood_bench/woodwork; knowledge ["sling"] | Physical input consumption and output production | Crafting only; ranged combat/ammunition absent; M9 |
| grind_barley | [("barley", 2)] → ("flour", 1); 60 ticks; grinding_stone/woodwork; knowledge ["querns"] | Physical input consumption and output production | Supply and labour return unbalanced |
| grind_rye | [("rye", 2)] → ("flour", 1); 60 ticks; grinding_stone/woodwork; knowledge ["querns"] | Physical input consumption and output production | Supply and labour return unbalanced |
| grind_millet | [("millet", 2)] → ("flour", 1); 60 ticks; grinding_stone/woodwork; knowledge ["querns"] | Physical input consumption and output production | Supply and labour return unbalanced |

### Item capacities, value and food/apparel effects

Item quantities below are `stack_max/carry/value`; food lifetime tuples are `ground/stockpile/roofed_store/pot[/granary]` days. All new foods have sickness chance 0, no rule overrides and no equipment damage/cutter/chop capability. Values affect trade; transport throughput and item abundance are unbalanced. Food groups, edibility and preference are included because they affect actual consumption and grain-store admission.

| Feature | Tunable/current value | Intended effect | Suspected issue |
| --- | --- | --- | --- |
| flour | 30/10/3; food units=1; groups=["grain"]; raw/cooked=false/false; preference=0; days=ground: 8, stockpile: 12, roofed_store: 16, pot: 24, granary: Some(32 | Physical inventory and food consumption | Transport, trade value and supply/use balance unevaluated |
| bread | 30/10/4; food units=2; groups=["grain"]; raw/cooked=true/true; preference=100; days=ground: 3, stockpile: 4, roofed_store: 6, pot: 8, granary: Some(12 | Physical inventory and food consumption | Transport, trade value and supply/use balance unevaluated |
| beer | 20/6/4; food units=1; groups=["grain"]; raw/cooked=true/true; preference=50; days=ground: 4, stockpile: 6, roofed_store: 8, pot: 12, granary: Some(16 | Physical inventory and food consumption | Transport, trade value and supply/use balance unevaluated |
| milk | 20/6/3; food units=1; groups=["dairy"]; raw/cooked=true/false; preference=0; days=ground: 1, stockpile: 2, roofed_store: 3, pot: 4 | Physical inventory and food consumption | C1-a supply absent; C1-b physical milk producer required |
| cheese | 20/6/5; food units=2; groups=["dairy"]; raw/cooked=true/true; preference=100; days=ground: 6, stockpile: 10, roofed_store: 14, pot: 18 | Physical inventory and food consumption | Transport, trade value and supply/use balance unevaluated |
| smoked_meat | 30/10/5; food units=2; groups=["meat"]; raw/cooked=true/true; preference=100; days=ground: 12, stockpile: 18, roofed_store: 24, pot: 30 | Physical inventory and food consumption | Transport, trade value and supply/use balance unevaluated |
| hay | 30/10/1; nonfood | Physical inventory / authored capability | M-BAL: harvested fodder and supplementary feed hook; livestock grazes living grass. |
| hoe | 5/2/12; nonfood | Physical inventory / authored capability | tool efficiency hook — M-BAL |
| sickle | 5/2/12; nonfood | Physical inventory / authored capability | tool efficiency hook — M-BAL |
| adze | 5/2/14; nonfood | Physical inventory / authored capability | tool efficiency hook — M-BAL |
| bow | 5/2/16; nonfood | Physical inventory / authored capability | M9: ranged combat and ammunition consumption; bow crafting is available. |
| arrow | 30/10/2; nonfood | Physical inventory / authored capability | M9: ranged ammunition consumption; arrow crafting is available. |
| dyed_tunic | 5/1/20; apparel legacy_top_code: 5, warmth_rank: 1, cold_permille: 800, cold_snap_work_immune: false, wear_days: Some(24), comfort_permille: 20 | Physical inventory / authored capability | Transport, trade value and supply/use balance unevaluated |
| leather_armor | 5/1/24; apparel legacy_top_code: 6, warmth_rank: 1, cold_permille: 700, cold_snap_work_immune: false, wear_days: Some(36), comfort_permille: 0 | Physical inventory / authored capability | M9: armour damage mitigation; the existing apparel warmth slot is connected. |
| herbs | 30/10/3; nonfood | Physical inventory / authored capability | M9: wild herb source and medicinal consumption are not connected. |
| poultice | 20/5/5; nonfood | Physical inventory / authored capability | M9: sickness has no medicinal-consumable hook; the authored recipe requires herbalism. |
| barley | 30/10/2; food units=1; groups=["grain"]; raw/cooked=false/false; preference=0; days=ground: 12, stockpile: 18, roofed_store: 24, pot: 30, granary: Some(48 | Physical inventory and food consumption | Transport, trade value and supply/use balance unevaluated |
| barley_seed | 30/10/1; nonfood | Physical inventory / authored capability | Transport, trade value and supply/use balance unevaluated |
| rye | 30/10/2; food units=1; groups=["grain"]; raw/cooked=false/false; preference=0; days=ground: 12, stockpile: 18, roofed_store: 24, pot: 30, granary: Some(48 | Physical inventory and food consumption | Transport, trade value and supply/use balance unevaluated |
| rye_seed | 30/10/1; nonfood | Physical inventory / authored capability | Transport, trade value and supply/use balance unevaluated |
| lentil | 30/10/2; food units=1; groups=["legume"]; raw/cooked=true/false; preference=0; days=ground: 12, stockpile: 18, roofed_store: 24, pot: 30 | Physical inventory and food consumption | Transport, trade value and supply/use balance unevaluated |
| lentil_seed | 30/10/1; nonfood | Physical inventory / authored capability | Transport, trade value and supply/use balance unevaluated |
| pea | 30/10/2; food units=1; groups=["legume"]; raw/cooked=true/false; preference=0; days=ground: 6, stockpile: 10, roofed_store: 14, pot: 18 | Physical inventory and food consumption | Transport, trade value and supply/use balance unevaluated |
| pea_seed | 30/10/1; nonfood | Physical inventory / authored capability | Transport, trade value and supply/use balance unevaluated |
| millet | 30/10/2; food units=1; groups=["grain"]; raw/cooked=false/false; preference=0; days=ground: 12, stockpile: 18, roofed_store: 24, pot: 30, granary: Some(48 | Physical inventory and food consumption | Transport, trade value and supply/use balance unevaluated |
| millet_seed | 30/10/1; nonfood | Physical inventory / authored capability | Transport, trade value and supply/use balance unevaluated |
| apple | 30/10/2; food units=1; groups=["fruit"]; raw/cooked=true/false; preference=0; days=ground: 4, stockpile: 6, roofed_store: 10, pot: 14 | Physical inventory and food consumption | Transport, trade value and supply/use balance unevaluated |
| goat_hide | 10/5/5; nonfood | Physical inventory / authored capability | Transport, trade value and supply/use balance unevaluated |
| wool | 30/10/3; nonfood | Physical inventory / authored capability | C1-b: sheep wool production is not connected; C2: wool spinning and woollen garments. |

| Historical item changed | Tunable | Current value | Intended effect | Suspected issue |
| --- | --- | --- | --- | --- |
| wild_grain; porridge | Granary lifetime | 48; 8 days (roofed 24; 4) | Double roofed lifetime for admitted grain food | Cooked grain and beer also receive grain-group protection; preservation strength unbalanced |
| basket; sling | stack/carry/value (unchanged) | 20/5/4 each | Existing identities now craftable | Basket carry bonus M-BAL; sling ranged effect M9 |

### Habitats, harvests and wildlife

| Feature | Tunable/data | Current value | Intended effect | Suspected issue / owner |
|---|---|---|---|---|
| C1 wild crops/apple | plants.ron placement.weight / density_permille | 1 / 200 each | Existing per-tile flora weighting with additional regional species | Additional species dilute prior wild grain/berries; M-BAL |
| C1 wild crops/apple | stages / ripe_start_day / yield_count / regrow_days | Growing,Ripe,Ripe,Dormant / 0 / 6 / 0 | Seasonal harvest supplies food and seed-saving input | Single harvest and broad season window unbalanced; M-BAL |
| C1 wild crops/apple | harvest_ticks / knowledge_speed_permille | 60 / 1500 | Existing gathering work scale | Labour/food return unbalanced; M-BAL |
| C1 wild species | origin climate/terrain/elevation | Bible bands; plain/forest; goat/sheep/rye >=160 | Regional physical source availability | River/dry/edge labels approximated by current ground classes; no new biome subsystem |
| C1 resident wildlife | population.initial / respawn_days / herd_tiles | 2..4 / 8 / 12 | Small deterministic herds with replacement | Hunting supply and capture supply unbalanced; M-BAL |
| C1 grazers | hp / flee_tiles / flee_speed / flee_steps / rest_steps / home_tiles / min_hearth_distance | 30 / 6 / 2 / 4 / 40 / 12 / 20 | Reuse existing flee/herd behaviour | Highland access/flight and hearth separation unbalanced; M-BAL |
| C1 boar | hp / sight / damage / chase_tiles / hit_ticks | 30 / 3 / 2 / 1 / 30 | Dangerous but short-range resident predator | Danger/hunting return unbalanced; M-BAL |
| C1 wildlife loot | meat with cutter / without / hide | 3 / 2 / 1 | Existing hunt outputs | Meat/hide abundance unbalanced; M-BAL |
| C1 new crops | seeds_per_tile / yields food,seed / growth_days | 2 / 10,2 / 4 | Physical seed renewal through field work | Yield/growth unbalanced; M-BAL |
| C1 new crops | sow_window / frost_sensitive | Spring [0,3); true except rye false | Existing seasonal cultivation | Climate effect beyond origins unbalanced; M-BAL |
| C1 new crops | till/sow/tend/harvest ticks | 60/30/30/60 | Existing labour budget | Return per labour unbalanced; M-BAL |
| C1 new crops | untended / initial,min fertility / cost / fallow recovery | 750 / 1000,400 / 200 / 10 | Existing soil exhaustion and tending | Rotation incentives unbalanced; M-BAL |

Exact new origin filters (inclusive elevation; absent elevation means unrestricted): wild_barley all climates/plain; wild_rye Neutral+Harsh/plain/160–255; wild_lentil+wild_pea Mild+Neutral/plain; wild_millet Mild/plain; wild_apple Neutral+Harsh/forest. Wild goat/sheep all climates/plain+forest/160–255; wild boar Mild+Neutral/forest; reindeer Harsh/plain. New plant placement elevation remains 0–255 and intersects origin; ripe_end_day=None. Existing plants retain all climates/plain+forest and their original placement values; wolf/deer origins are unrestricted. This is regional scarcity, not an assurance that every world contains every resource.

| Feature | Tunable | Current value | Intended effect | Suspected issue |
| --- | --- | --- | --- | --- |
| Imported barley/rye/lentil/pea/millet seeds | Neighbour staple offer count | 2–6 each | Physical trade route where wild sources are absent | Offer dilution and trade-dependent adoption unbalanced |
| Wild goat hide | Loot identity | goat_hide; other new animals hide | Distinct physical species product | Goat hide downstream processing must be authored; no automatic substitution |

### Buildings and storage

| Feature | Tunable | Current value | Intended effect | Suspected issue |
| --- | --- | --- | --- | --- |
| fence | Cost/work/footprint | 1 log / 30 ticks / 1×1 | Human-passable enclosure; blocks_livestock=true; supports_roof=false | Livestock movement enforcement follows C1-b; no shelter from fence ring |
| clay_oven | Cost/work/footprint/tag | 4 clay+2 stone / 90 ticks / 1×1 / baking | Physical bread workshop | Clay access and workload unbalanced |
| smoking_rack | Cost/work/footprint/tag | 2 log+1 cord / 60 ticks / 1×1 / smoking | Physical preservation workshop | Fuel and preservation return unbalanced |
| granary | Geometry/material/function | 4×4 ring+south door+packed interior; 23 logs; grain storage; 0 beds; Roofed | Filtered grain store with item-authored lifetimes | Construction cost versus preservation benefit unbalanced |
| pen | Geometry/material/capacity | 5×5; 15 fences+south door; 16 logs; 6 heads; 0 beds; Open | Physical future livestock enclosure | Capacity is definition only in C1-a; capture/feed/breeding C1-b |
| smokehouse | Geometry/material/function | 4×4 ring+door+floor+smoking_rack; 25 logs+1 cord; 0 beds; Roofed | Enclosed smoking workplace | Station south access requires 4×4; travel/queue unbalanced |
| oven_house | Geometry/material/function | 4×4 ring+door+floor+clay_oven; 23 logs+4 clay+2 stone; 0 beds; Roofed | Enclosed baking workplace | Station south access requires 4×4; travel/queue unbalanced |

Existing wall/door/floor per-piece work costs are reused. Granary lifetime is exactly 2× roofed lifetime for authored grain-group foods only: wild_grain/barley/rye/millet48, flour32, bread12, beer16, porridge8. Legumes and other food retain ordinary roofed lifetimes; physical room loss removes the granary association. Well remains Inert C2 because physical water extraction/delivery is absent.

### Knowledge thresholds and discovery

출처: `rust/crates/sim-core/data/colony/knowledge.ron`. 아래 값은 시작 설계값이며 관측 결과에 맞춘 조정이나 자연 채택률 보장이 아니다. 신규 39개 모두 `seed: None`, era `neolithic`; threshold 단위는 permille. Inert 발견 조건은 저작 정보이며 런타임 발견·게이트는 비활성이다.

| feature | tunable/current value | intended effect | suspected issue / debt |
| --- | --- | --- | --- |
| animal_taming (Connected) | tier=1; threshold=300; difficulty=1; marker_weight=2; prereqs=["tracking", "cordage"]; discover=[Practice(job_kind: "hunt", count: 2), Stockpile("cord", 1)] | gates=[] | C1-b: pen construction is connected; capture and livestock ownership are not active in C1-a.; 자연 발견·전승 빈도 미균형조정 |
| herding (Inert) | tier=2; threshold=300; difficulty=1; marker_weight=0; prereqs=["animal_taming"]; discover=[Knows("animal_taming", 300), Stockpile("hay", 3)] | gates=[] | C1-b: livestock grazing, breeding and milk collection are not active in C1-a.; 자연 발견·전승 빈도 미균형조정 |
| dairy (Connected) | tier=3; threshold=300; difficulty=2; marker_weight=0; prereqs=["pot_firing"]; discover=[Practice(job_kind: "craft", count: 3), Stockpile("milk", 2)] | gates=[Recipe("ferment_milk")] | C1-b: cheese recipe connected; livestock milk supply and milk-based natural discovery absent in C1-a; 자연 발견·전승 빈도 미균형조정 |
| dog_taming (Inert) | tier=1; threshold=300; difficulty=1; marker_weight=0; prereqs=["trapping"]; discover=[Knows("trapping", 300), Stockpile("meat", 2)] | gates=[] | C2: companion dogs and their hunting/guarding effects require a dedicated driver.; 자연 발견·전승 빈도 미균형조정 |
| selective_breeding (Inert) | tier=3; threshold=300; difficulty=2; marker_weight=0; prereqs=["herding"]; discover=[Knows("herding", 300), Stockpile("hay", 3)] | gates=[] | M-BAL: inherited livestock traits and selection-based production changes are absent.; 자연 발견·전승 빈도 미균형조정 |
| querns (Connected) | tier=1; threshold=300; difficulty=1; marker_weight=0; prereqs=["stone_grinding", "cultivation"]; discover=[Practice(job_kind: "craft", count: 2), Stockpile("wild_grain", 2)] | gates=[Recipe("grind_flour"), Recipe("grind_barley"), Recipe("grind_rye"), Recipe("grind_millet")] | 없음; 자연 발견·전승 빈도 미균형조정 |
| oven (Connected) | tier=2; threshold=300; difficulty=1; marker_weight=0; prereqs=["querns", "clay_prep"]; discover=[Practice(job_kind: "craft", count: 3), Stockpile("flour", 2)] | gates=[Workshop("clay_oven"), Recipe("bake_bread")] | 없음; 자연 발견·전승 빈도 미균형조정 |
| fermentation (Connected) | tier=2; threshold=300; difficulty=1; marker_weight=0; prereqs=["pot_firing"]; discover=[Practice(job_kind: "craft", count: 3), Stockpile("barley", 3)] | gates=[Recipe("brew_beer")] | 없음; 자연 발견·전승 빈도 미균형조정 |
| granary (Connected) | tier=2; threshold=300; difficulty=1; marker_weight=0; prereqs=["cultivation", "packed_floor"]; discover=[Practice(job_kind: "build", count: 3), Stockpile("wild_grain", 4)] | gates=[] | 없음; 자연 발견·전승 빈도 미균형조정 |
| food_preservation (Connected) | tier=2; threshold=300; difficulty=1; marker_weight=0; prereqs=["food_drying"]; discover=[Practice(job_kind: "craft", count: 3), Stockpile("meat", 2)] | gates=[Workshop("smoking_rack"), Recipe("smoke_meat")] | 없음; 자연 발견·전승 빈도 미균형조정 |
| barley (Connected) | tier=1; threshold=300; difficulty=1; marker_weight=0; prereqs=["cultivation"]; discover=[Knows("cultivation", 300), Stockpile("barley", 2)] | gates=[Recipe("save_barley_seed")] | 없음; 자연 발견·전승 빈도 미균형조정 |
| lentil (Connected) | tier=2; threshold=300; difficulty=1; marker_weight=0; prereqs=["cultivation"]; discover=[Knows("cultivation", 300), Stockpile("lentil", 2)] | gates=[Recipe("save_lentil_seed")] | 없음; 자연 발견·전승 빈도 미균형조정 |
| pea (Connected) | tier=2; threshold=300; difficulty=1; marker_weight=0; prereqs=["cultivation"]; discover=[Knows("cultivation", 300), Stockpile("pea", 2)] | gates=[Recipe("save_pea_seed")] | 없음; 자연 발견·전승 빈도 미균형조정 |
| millet (Connected) | tier=1; threshold=300; difficulty=1; marker_weight=0; prereqs=["cultivation"]; discover=[Knows("cultivation", 300), Stockpile("millet", 2)] | gates=[Recipe("save_millet_seed")] | 없음; 자연 발견·전승 빈도 미균형조정 |
| rye (Connected) | tier=1; threshold=300; difficulty=1; marker_weight=0; prereqs=["cultivation"]; discover=[Knows("cultivation", 300), Stockpile("rye", 2)] | gates=[Recipe("save_rye_seed")] | 없음; 자연 발견·전승 빈도 미균형조정 |
| orchard (Inert) | tier=3; threshold=300; difficulty=2; marker_weight=0; prereqs=["cultivation"]; discover=[Knows("cultivation", 300), Stockpile("apple", 2)] | gates=[] | C2: perennial planting and orchard regrowth are absent; wild apples can be harvested.; 자연 발견·전승 빈도 미균형조정 |
| crop_rotation (Inert) | tier=3; threshold=300; difficulty=2; marker_weight=0; prereqs=["barley", "lentil"]; discover=[Knows("barley", 300), Knows("lentil", 300)] | gates=[] | M-BAL: fallow recovery exists, but crop-history rotation has no fertility hook.; 자연 발견·전승 빈도 미균형조정 |
| hoe (Connected) | tier=1; threshold=300; difficulty=1; marker_weight=0; prereqs=["stone_grinding", "hafting"]; discover=[Practice(job_kind: "craft", count: 2), Stockpile("flint_blade", 1)] | gates=[Recipe("make_hoe")] | M-BAL: tool efficiency hook — craftable hoe does not change tilling time.; 자연 발견·전승 빈도 미균형조정 |
| sickle (Connected) | tier=2; threshold=300; difficulty=1; marker_weight=0; prereqs=["pressure_flaking", "hafting"]; discover=[Practice(job_kind: "craft", count: 2), Stockpile("flint_blade", 1)] | gates=[Recipe("make_sickle")] | M-BAL: tool efficiency hook — craftable sickle does not change harvest time.; 자연 발견·전승 빈도 미균형조정 |
| adze (Connected) | tier=2; threshold=300; difficulty=1; marker_weight=0; prereqs=["stone_grinding"]; discover=[Practice(job_kind: "craft", count: 2), Stockpile("flint_blade", 1)] | gates=[Recipe("make_adze")] | M-BAL: tool efficiency hook — craftable adze does not change carpentry quality.; 자연 발견·전승 빈도 미균형조정 |
| wattle_daub (Inert) | tier=1; threshold=300; difficulty=1; marker_weight=0; prereqs=["timber_wall", "clay_prep"]; discover=[Practice(job_kind: "build", count: 2), Stockpile("clay", 2)] | gates=[] | C2: a produced wall material and construction recipe are required before material-tag walls connect.; 자연 발견·전승 빈도 미균형조정 |
| thatch (Inert) | tier=1; threshold=300; difficulty=1; marker_weight=0; prereqs=["cordage"]; discover=[Knows("cordage", 300), Stockpile("hay", 3)] | gates=[] | C2: authored thatch roof pieces and their rain/insulation material effects are absent.; 자연 발견·전승 빈도 미균형조정 |
| mud_brick (Inert) | tier=3; threshold=300; difficulty=2; marker_weight=0; prereqs=["clay_prep", "oven"]; discover=[Practice(job_kind: "craft", count: 3), Stockpile("clay", 4)] | gates=[] | C2: brick production, brick items and buildable brick walls are absent.; 자연 발견·전승 빈도 미균형조정 |
| well (Inert) | tier=2; threshold=300; difficulty=1; marker_weight=0; prereqs=["digging", "stone_walling"]; discover=[Practice(job_kind: "dig", count: 2), Stockpile("stone", 4)] | gates=[] | C2: water is not an item; wells require physical extraction and delivery semantics.; 자연 발견·전승 빈도 미균형조정 |
| fence (Connected) | tier=1; threshold=300; difficulty=1; marker_weight=0; prereqs=["cordage", "timber_wall"]; discover=[Practice(job_kind: "build", count: 2), Stockpile("log", 2)] | gates=[Blueprint("fence")] | 없음; 자연 발견·전승 빈도 미균형조정 |
| megalith (Inert) | tier=4; threshold=300; difficulty=2; marker_weight=0; prereqs=["stone_quarrying", "cordage"]; discover=[Practice(job_kind: "build", count: 3), Stockpile("stone", 8)] | gates=[] | M8: communal megalith construction and ritual function are absent.; 자연 발견·전승 빈도 미균형조정 |
| dyeing (Connected) | tier=3; threshold=300; difficulty=2; marker_weight=0; prereqs=["weaving", "foraging"]; discover=[Practice(job_kind: "craft", count: 3), Stockpile("berries", 2)] | gates=[Recipe("dye_tunic")] | 없음; 자연 발견·전승 빈도 미균형조정 |
| leather_armor (Connected) | tier=2; threshold=300; difficulty=1; marker_weight=0; prereqs=["hide_working"]; discover=[Knows("hide_working", 300), Stockpile("hide", 3)] | gates=[Recipe("make_leather_armor")] | 없음; 자연 발견·전승 빈도 미균형조정 |
| bow (Connected) | tier=2; threshold=300; difficulty=1; marker_weight=0; prereqs=["cordage", "hafting"]; discover=[Practice(job_kind: "craft", count: 3), Stockpile("cord", 2)] | gates=[Recipe("make_bow"), Recipe("make_arrows")] | M9: bow and arrows are craftable; ranged combat and hunting efficiency hooks are absent.; 자연 발견·전승 빈도 미균형조정 |
| dugout_canoe (Inert) | tier=3; threshold=300; difficulty=2; marker_weight=0; prereqs=["adze"]; discover=[Practice(job_kind: "craft", count: 3), Stockpile("log", 3)] | gates=[] | C2: canoe construction, boarding and water traversal are absent.; 자연 발견·전승 빈도 미균형조정 |
| barter (Inert) | tier=1; threshold=300; difficulty=1; marker_weight=0; prereqs=["storytelling"]; discover=[Knows("storytelling", 300), Practice(job_kind: "craft", count: 2)] | gates=[] | M7: ordinary trade already works; a barter-knowledge hook for expanded trade cards is absent.; 자연 발견·전승 빈도 미균형조정 |
| tally (Inert) | tier=2; threshold=300; difficulty=1; marker_weight=0; prereqs=["storytelling"]; discover=[Knows("storytelling", 300), Stockpile("log", 2)] | gates=[] | M7: tally sticks and knowledge-gated inventory records are absent.; 자연 발견·전승 빈도 미균형조정 |
| lunar_calendar (Inert) | tier=3; threshold=300; difficulty=2; marker_weight=0; prereqs=["tally", "weather_lore"]; discover=[Knows("tally", 300), Knows("weather_lore", 300)] | gates=[] | M7: lunar observations and knowledge-based seasonal forecasts are absent.; 자연 발견·전승 빈도 미균형조정 |
| ritual (Inert) | tier=1; threshold=300; difficulty=1; marker_weight=0; prereqs=["ochre", "storytelling"]; discover=[Knows("storytelling", 300), Practice(job_kind: "build", count: 2)] | gates=[] | M8: collective rites, ritual places and feast effects are absent.; 자연 발견·전승 빈도 미균형조정 |
| elder_council (Inert) | tier=2; threshold=300; difficulty=1; marker_weight=0; prereqs=["ritual"]; discover=[Knows("ritual", 300), Knows("storytelling", 300)] | gates=[] | M8: elder deliberation and its steward decision effects are absent.; 자연 발견·전승 빈도 미균형조정 |
| kinship (Inert) | tier=1; threshold=300; difficulty=1; marker_weight=0; prereqs=["storytelling"]; discover=[Knows("storytelling", 300), Practice(job_kind: "build", count: 2)] | gates=[] | M8: household kin groups and their collective decision semantics are absent.; 자연 발견·전승 빈도 미균형조정 |
| taboo (Inert) | tier=3; threshold=300; difficulty=2; marker_weight=0; prereqs=["ritual"]; discover=[Knows("ritual", 300), Knows("storytelling", 300)] | gates=[] | M7: shared food/action prohibitions and their enforcement are absent.; 자연 발견·전승 빈도 미균형조정 |
| herbalism (Inert) | tier=2; threshold=300; difficulty=1; marker_weight=0; prereqs=["foraging"]; discover=[Knows("foraging", 300), Stockpile("herbs", 2)] | gates=[Recipe("prepare_poultice")] | M9: illness recovery has no consumable medicine hook; herbs and poultice are authored without healing.; 자연 발견·전승 빈도 미균형조정 |
| bone_setting (Inert) | tier=3; threshold=300; difficulty=2; marker_weight=0; prereqs=["herbalism"]; discover=[Knows("herbalism", 300), Stockpile("cord", 2)] | gates=[] | M9: wound treatment jobs, splints and injury recovery effects are absent.; 자연 발견·전승 빈도 미균형조정 |

기존 노드 변경: basketry Connected (cordage300+cord3 → weave_basket, carry bonus M-BAL); sling Connected (cordage300+stone3 → make_sling, ranged M9); hide_working Connected (craft3+hide_scraping300+hide3 → make_leather_armor, leather-thong C2). 세 노드 모두 threshold300/difficulty1/marker0/seedNone. trapping은 Inert C2로 유지한다.

### Data-comment coverage and limits

The `knowledge.ron` C1 block explains threshold300, ordinary/advanced difficulty1/2, physical discovery witnesses and no founder seeding; per-node debt states missing effects. The `items.ron` C1 header covers capacities, value, nutrition and storage lifetimes, with separate tool/apparel/seed comments. The `recipes.ron` C1 header covers ingredient ratios and45–120 active ticks, explicit knowledge requirements and physical seed saving. Each added plant, crop and animal block carries its starting-value/M-BAL comment. `neighbours.ron` explains2–6 physical seed imports. `blueprints.ron` labels costs/work as starting values; `buildings.ron` comments explain2×grain life, capacity6 and station geometry. No numerical value was changed during this consolidation. Material/art names are presentation/category references rather than new efficacy tunables; C1-a adds no material efficacy table.

The 39-node table includes Inert authoring values so future activation remains reviewable; it does not claim those discovery predicates run today. The old Stage2 mask/capacity description above remains a dated historical record; C1 expands practitioner storage separately, without treating bit width as a balance parameter.

Imported barley/rye/lentil/pea/millet seeds now carry their matching crop knowledge in `clue_nodes`, using the existing physical-artifact eureka path. Review found a circular prerequisite in regions without the wild crop: sowing needs crop knowledge, but ordinary discovery needs its harvested food. A physically held traded seed now supplies a discovery clue without injecting knowledge or intermediate food, while cultivation prerequisites and the existing eureka probability remain unchanged. Local seed saving still requires the crop node. No new numerical tunable was introduced; trade frequency and discovery latency remain unbalanced.

C1-a focused diagnosis, seed2: resident boars caused22,005 founder-flight ticks during a4,320-tick hut fixture; all12logs arrived but construction reached92/240. A diagnostic arm suppressing only the added predators completed construction in282ticks with240work and no flight. The housing regression now explicitly isolates construction/sleep from threats, retaining its original deadline and four-bed assertions. This does not accept normal-world labor availability: boar pressure remains a balance concern, with all authored numbers unchanged and candidate crash-detector acceptance still separate. Raw paired diagnostics are `c1-data/logs/hut-trace-a2.log` and `hut-isolation-a3.log`.

### C1-a completed red crash detector and one Inert fallback

The paired48-day colony smoke used seeds0–7, steward ON and no player orders. Completed evidence is preserved outside the repository at `c1-data/crash-base-r2` and `c1-data/crash-head-a2`. These arrays are in seed order0–7:

| Arm | alive_end | KnowledgeLost event counts |
| --- | --- | --- |
| BASE | [10,12,11,14,10,13,12,14] | [0,0,0,0,0,0,0,0] |
| Candidate before fallback | [10,10,11,14,10,0,13,10] | [0,0,0,0,0,25,0,0] |

The completed candidate is RED: seed5 is extinct while BASE retains13 people; total KnowledgeLost25 exceeds2×BASE0+2. Root identified resident-boar labour disruption followed by starvation and applied the GOAL§5 single allowed content fallback: `wild_boar` becomes Inert with explicit M-BAL debt. Its population, habitat, damage, chase, timing and loot values above remain unchanged and authored but inactive. This is the authorized content deactivation, not numerical outcome tuning. The red result remains evidence; this note does not claim a post-fallback smoke pass. No calibration, held-out evaluation or repair loop is authorized by this result. Publication still requires the technical gate and issue/report handling owned by the root executor.


## C1-b livestock starting policy (fixed before chain/smoke outcomes)

These are design values authored in `animals.ron`, not acceptance thresholds or tuned outcomes. Existing wild populations, respawn and origins remain unchanged; wild_boar stays Inert under ISS364. Wool collection/processing remains C2 debt; hay supplementation remains M-BAL debt.

| Feature | Tunable/current value | Intended effect | Suspected issue |
| --- | --- | --- | --- |
| Tameable sources | goat/sheep/reindeer, Grazer policy; initial keyed age 0..29 days; kid/adult boundary30 | Copy natural age/sex and accept only existing wild kids | Source window and natural replenishment unbalanced; no scene age/sex override |
| Capture | range40tiles; work60ticks; success700permille; retry1day; one selected physical tool consumed on success only | Practitioner walks to real source and commits one logged capture | Travel time, tether cost and retry throughput unbalanced |
| Pen | authored hard capacity6; living forage3distinct interior tiles/head/day; one unit/cell; regrow1day | Separate hard capacity from physical forage support | A 5×5 enclosure has9interior cells, supporting3daily fed heads; overstock and damaged fences diagnose rather than gain free feed |
| Feeding/death | starvation after3unfed days; max age1800days; stable-ID daily feeding | Deterministic competition and physical natural death | No fodder supplement or transport mechanic this pass |
| Breeding | opposite-sex same-species adults; interval20days from capture/previous birth; free hard capacity; both fed | Real kid with immutable parent witnesses | No new20day delay after adulthood; population/feed pressure unbalanced |
| Milk | adult fed female; interval1day; work45ticks; one authored milk item; dairy practitioner | Physical milk commit at real gate; readiness retained on blocked output | No lactation/supplementary feed mechanic; production/spoilage balance deferred |
| Slaughter | explicit logged intent; work90ticks; selected meat3+hide1 atomically | Durable cause plus exact mixed-output ledger | Meat/hide value and labour balance deferred |
| Dairy discovery | one completed capture plus pot_firing400; prerequisites animal_taming+pot_firing; threshold300, difficulty2 unchanged | Break circular discovery that formerly required unavailable milk stock | Ordinary fired-pot practice and diffusion must maintain knowledge; no dairy grant |
| Herding discovery | animal_taming300 plus one completed capture; threshold300, difficulty1 unchanged | Natural learning after actual capture; connected grazing/breeding | Knowledge social/adoption balance deferred |
| Steward livestock | authored rule11, Food scope,360tick successful-action cooldown | Ordinary pen request followed by ordinary capture designation | Never a second job scheduler; path/source/tool/knowledge admission shared with jobs |
| Colony mind | one work-band livestock owner node, score0/hysteresis0/everytick; existing do_job lifecycle | Explain capture/milk ownership without replacing needs priority | Needs and duty bands retain their authored precedence |


## C2-a bronze content starting values

All values are authored design inputs, not smoke/calibration acceptance. Recipe skill reuse, inventory handling and workshop contention are unbalanced. All recipes use craft, quality_add0, material_termNone and no automatic bill.

| Feature | Tunable/current value | Intended effect | Suspected issue |
| --- | --- | --- | --- |
| burn_charcoal | [("log", 4)], output: ("charcoal", 3), 120, skill: "pottery", work: "craft", quality_add: 0, material_term: None, | Physical transformation at its authored station | Supply, skill reuse and throughput unbalanced |
| smelt_copper | [("copper_ore", 2), ("charcoal", 2)], output: ("copper_ingot", 1), 120, skill: "pottery", work: "craft", quality_add: 0, material_term: None, | Physical transformation at its authored station | Supply, skill reuse and throughput unbalanced |
| smelt_tin | [("tin_ore", 2), ("charcoal", 2)], output: ("tin_ingot", 1), 120, skill: "pottery", work: "craft", quality_add: 0, material_term: None, | Physical transformation at its authored station | Supply, skill reuse and throughput unbalanced |
| alloy_bronze | [("copper_ingot", 3), ("tin_ingot", 1), ("charcoal", 2)], output: ("bronze_ingot", 3), 120, skill: "pottery", work: "craft", quality_add: 0, material_term: None, | Physical transformation at its authored station | Supply, skill reuse and throughput unbalanced |
| make_mould | [("clay", 2)], output: ("clay_mould", 1), 60, skill: "pottery", work: "craft", quality_add: 0, material_term: None, | Physical transformation at its authored station | Supply, skill reuse and throughput unbalanced |
| cast_copper_axe | [("copper_ingot", 1), ("clay_mould", 1), ("log", 1)], output: ("copper_axe", 1), 120, skill: "pottery", work: "craft", quality_add: 0, material_term: None, | Physical transformation at its authored station | Supply, skill reuse and throughput unbalanced |
| cast_bronze_axe | [("bronze_ingot", 1), ("clay_mould", 1), ("log", 1)], output: ("bronze_axe", 1), 120, skill: "pottery", work: "craft", quality_add: 0, material_term: None, | Physical transformation at its authored station | Supply, skill reuse and throughput unbalanced |
| cast_bronze_sickle | [("bronze_ingot", 1), ("clay_mould", 1), ("log", 1)], output: ("bronze_sickle", 1), 120, skill: "pottery", work: "craft", quality_add: 0, material_term: None, | Physical transformation at its authored station | Supply, skill reuse and throughput unbalanced |
| cast_bronze_hoe | [("bronze_ingot", 1), ("clay_mould", 1), ("log", 1)], output: ("bronze_hoe", 1), 120, skill: "pottery", work: "craft", quality_add: 0, material_term: None, | Physical transformation at its authored station | Supply, skill reuse and throughput unbalanced |
| cast_bronze_spear | [("bronze_ingot", 1), ("clay_mould", 1), ("log", 1)], output: ("bronze_spear", 1), 120, skill: "pottery", work: "craft", quality_add: 0, material_term: None, | Physical transformation at its authored station | Supply, skill reuse and throughput unbalanced |
| cast_bronze_dagger | [("bronze_ingot", 1), ("clay_mould", 1), ("log", 1)], output: ("bronze_dagger", 1), 120, skill: "pottery", work: "craft", quality_add: 0, material_term: None, | Physical transformation at its authored station | Supply, skill reuse and throughput unbalanced |
| hammer_awl | [("copper_ingot", 1)], output: ("copper_awl", 2), 60, skill: "woodwork", work: "craft", quality_add: 0, material_term: None, | Physical transformation at its authored station | Supply, skill reuse and throughput unbalanced |
| hammer_needle | [("copper_ingot", 1)], output: ("copper_needle", 2), 60, skill: "woodwork", work: "craft", quality_add: 0, material_term: None, | Physical transformation at its authored station | Supply, skill reuse and throughput unbalanced |
| spin_wool | [("wool", 2)], output: ("wool_yarn", 2), 60, skill: "woodwork", work: "craft", quality_add: 0, material_term: None, | Physical transformation at its authored station | Supply, skill reuse and throughput unbalanced |
| weave_wool | [("wool_yarn", 2)], output: ("wool_cloth", 1), 90, skill: "woodwork", work: "craft", quality_add: 0, material_term: None, | Physical transformation at its authored station | Supply, skill reuse and throughput unbalanced |
| sew_wool_tunic | [("wool_cloth", 2)], output: ("wool_tunic", 1), 120, skill: "woodwork", work: "craft", quality_add: 0, material_term: None, | Physical transformation at its authored station | Supply, skill reuse and throughput unbalanced |
| make_mud_brick | [("clay", 2), ("flax_stalks", 1)], output: ("mud_brick", 2), 60, skill: "pottery", work: "craft", quality_add: 0, material_term: None, | Physical transformation at its authored station | Supply, skill reuse and throughput unbalanced |
| burn_lime | [("limestone", 2), ("charcoal", 2)], output: ("lime", 1), 120, skill: "pottery", work: "craft", quality_add: 0, material_term: None, | Physical transformation at its authored station | Supply, skill reuse and throughput unbalanced |
| make_wheel | [("log", 2)], output: ("wheel", 1), 90, skill: "woodwork", work: "craft", quality_add: 0, material_term: None, | Physical transformation at its authored station | Supply, skill reuse and throughput unbalanced |
| make_composite_bow | [("log", 1), ("cord", 2), ("cured_hide", 1)], output: ("composite_bow", 1), 120, skill: "woodwork", work: "craft", quality_add: 0, material_term: None, | Physical transformation at its authored station | Supply, skill reuse and throughput unbalanced |

| Item | Stack / carry / value and mechanics | Intended effect | Suspected issue |
| --- | --- | --- | --- |
| charcoal | 30/10/4; fuel_units=Some(1) | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| copper_ore | 30/10/5; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| tin_ore | 30/10/6; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| iron_ore | 30/10/5; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| copper_ingot | 30/10/12; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| tin_ingot | 30/10/14; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| bronze_ingot | 30/10/18; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| clay_mould | 30/10/4; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| copper_awl | 5/2/10; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| copper_needle | 5/2/10; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| copper_axe | 5/2/26; chop=Some((quality_permille: (400, 390, 380, 370, 360, 350, 340, 330, 320, 310, 300), wear_rule: None, wear_default: 240, break_return: None, preserve_return_material: false)); weapon=Some((damage: 11, damage_rule: None, wear_rule: None, wear_default: 0, draft_rank: Some(4), hunt_rank: None, hunt_hit_bonus: false, break_tally: None)) | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| bronze_axe | 5/2/32; chop=Some((quality_permille: (350, 340, 330, 320, 310, 300, 290, 280, 270, 260, 250), wear_rule: None, wear_default: 320, break_return: None, preserve_return_material: false)); weapon=Some((damage: 13, damage_rule: None, wear_rule: None, wear_default: 0, draft_rank: Some(5), hunt_rank: None, hunt_hit_bonus: false, break_tally: None)) | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| bronze_sickle | 5/2/26; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| bronze_hoe | 5/2/26; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| bronze_spear | 5/2/30; weapon=Some((damage: 16, damage_rule: None, wear_rule: None, wear_default: 60, draft_rank: Some(6), hunt_rank: Some(2), hunt_hit_bonus: true, break_tally: None)) | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| bronze_dagger | 5/2/28; weapon=Some((damage: 13, damage_rule: None, wear_rule: None, wear_default: 48, draft_rank: Some(5), hunt_rank: None, hunt_hit_bonus: false, break_tally: None)); cutter=Some((wear_rule: None, wear_default: 16, break_tally: None)) | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| honey | 30/10/5; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| beeswax | 30/10/5; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| wool_yarn | 30/10/4; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| wool_cloth | 30/10/8; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| wool_tunic | 5/1/20; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| wheel | 30/10/12; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| mud_brick | 30/10/3; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| lime | 30/10/4; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| seal | 30/10/8; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| tablet | 30/10/5; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| glass_bead | 30/10/8; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| composite_bow | 5/2/24; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |
| limestone | 30/10/2; no extra mechanic | Authored physical inventory/tool tier | No balance claim; Inert entries remain unavailable |

| Piece | Footprint / needs / work | Intended effect | Suspected issue |
| --- | --- | --- | --- |
| charcoal_clamp | (2, 2); [(item: "clay", count: 4, count_rule: None), (item: "stone", count: 2, count_rule: None)]; 120ticks | Existing construction and work-tile mechanics | Placement and labour unbalanced |
| smelter | (1, 1); [(item: "clay", count: 6, count_rule: None), (item: "stone", count: 2, count_rule: None)]; 120ticks | Existing construction and work-tile mechanics | Placement and labour unbalanced |
| casting_bench | (2, 1); [(item: "log", count: 4, count_rule: None), (item: "stone", count: 2, count_rule: None)]; 120ticks | Existing construction and work-tile mechanics | Placement and labour unbalanced |
| mud_brick_wall | (1, 1); [(item: "mud_brick", count: 3, count_rule: None)]; 60ticks | Existing construction and work-tile mechanics | Placement and labour unbalanced |
| palisade | (1, 1); [(item: "log", count: 3, count_rule: None)]; 60ticks | Existing construction and work-tile mechanics | Placement and labour unbalanced |
| kiln | (1, 1); [(item: "clay", count: 6, count_rule: None), (item: "stone", count: 2, count_rule: None)]; 120ticks | Existing construction and work-tile mechanics | Placement and labour unbalanced |
| stone_masonry_wall | (1, 1); [(item: "stone", count: 4, count_rule: None)]; 90ticks | Existing construction and work-tile mechanics | Placement and labour unbalanced |

| Template | Current footprint, tier and materials | Intended effect | Suspected issue |
| --- | --- | --- | --- |
| charcoal_clamp | 3×3/tier1; clay4+stone2; Open | 2×2 earthen station with a south work tile | No fire/temperature subsystem |
| smelter | 4×4/tier2; log23+clay6+stone2; Roofed | Enclosed smelting station | Station and stock traffic unbalanced |
| casting_bench | 4×4/tier3; log27+stone2; Roofed | Enclosed casting bench | Station and stock traffic unbalanced |
| mud_brick_house | 4×4/tier4; mud_brick33+log3; 2beds | Replace11 log walls and retain floor/door/beds | Existing roof warmth only; no new thermal mechanics |
| kiln | 3×3/tier2; clay6+stone2; Open | Existing firing recipe compatibility | No temperature curve |
| bee_skep | 1×1/tier2; Inert | Visible authored future utility | M-GROW dual food/nonfood hive timer absent |
| wool garment art | top_code7→wrap silhouette; RGB192/180/155 | Distinct clothing colour without a new asset | Placeholder visual only |

### C2-a geological and livestock starting policy

Fixed before the64-seed origin census and smoke outcomes; no value below is a
balance acceptance threshold. The original outcrop data/hash is retained in
external c2-data/outcrops-initial-authored.ron.

| Feature | Tunable/current value | Intended effect | Suspected issue |
| --- | --- | --- | --- |
| Copper origins | all3climates; hill/mountain; no extra elevation bound; presence1000‰;1–3clusters,1–3tiles; camp radius60 | Physical nearby highland copper | Coverage is measured once on seeds0–63; no post-outcome tuning |
| Tin origins | same habitat/radius; independent presence500‰;0–1cluster,1–2tiles | About25% intended tin worlds; trade otherwise | Actual64-seed percentage is evidence, not a tuning target |
| Iron origins | Inert/C3; presence1000‰;1–2clusters,1–3tiles, radius60 | Reserve authored iron family | No active iron deposits or production |
| Deposit quantity |24units/tile for each authored family | Finite physical extraction capacity | Mining yield/labour balance deferred to M-BAL |
| Discovery | practitioner radius4; completed quarry radius1; every8ticks at offset3 | Real knowledge or quarry cause reveals terrain | Visibility/travel latency unbalanced; no knowledge granted by reveal |
| Aurochs source | plain terrain;2–4animals, respawn8days, radius12 | Natural cattle source through existing capture | Habitat/feed competition unbalanced |
| Cattle livestock | reused C1 capture700‰/60ticks/retry1day; grazing3cells/day; starvation3days; breed20days; kid30days; max age1800days | Same physical capture, feeding and parent witnesses | No new husbandry efficacy or supplementary fodder |
| Sheep wool |1wool/4days,45ticks; adult fed either sex; wool_spinning practitioner | Existing recurring product work | Cooldown independent of milk; throughput and garment warmth unbalanced |
| Animal art | shared tile raster; authored body/leg/head/horn/antler/tail; juvenile600‰ | Distinguishable species and small offspring | Presentation only; exact per-species values live in animals.ron |
| Material art | render_params.material_looks Lab deltas and existing categories | Copper/tin/bronze and earth distinguishable in shared placeholders | Presentation only; does not change material efficacy |

# C2-a 지식 수치 기록 — BALANCE_DEBT 병합용

2026-11-10. 현재 `knowledge.ron`을 읽어 추출했다. 새 47개와 기존 승격·표지 변경 2개를 모두 포함한다. 수치 조정, 자연 채택률 평가, held-out 재실행은 하지 않았다. 원본은 `rust/crates/sim-core/data/colony/knowledge.ron`이며 표의 ID는 영구 배열 인덱스이다.

## 의미·의도·확인할 문제

- `threshold`는 개인이 실천자로 인정되는 숙련도(‰), `difficulty`는 기존 발견 확률 등급, `seed`는 창립자 초기 보유, `marker_weight`는 해당 시대 진척 가중치다. `discover` 조건은 모두 AND이며 prerequisite 보유 검사와 함께 적용된다.
- 새 47개는 threshold 300 / seed None이다. 난이도는 tier 1→1, 2→2, 3·4→3을 초기 설계값으로 택했다. 목적은 선행 지식과 실제 작업·재고 뒤에 연결 기능을 발견하도록 하는 것이다. 실제 도달 속도·시대 진입 시점은 아직 균형 검증하지 않았다.
- `Practice`는 완료된 직업 누계, `Stockpile`은 실제 재고, `Knows`는 개인 숙련, `Trades`는 실제 성공 교환 누계를 읽는다. 의심할 문제는 누계가 많은 기존 마을에서 빠르게 연속 발견되거나 희귀 투입 재고 때문에 발견이 늦어질 가능성이다. 이 패스는 결과 기반 조정을 하지 않는다.
- Inert 노드의 발견 조건은 미래 연결용 저작 데이터이며 현재 런타임에서 발견·실천·시대 표지를 만들지 않는다. 이를 현재 작동하는 기능으로 해석하지 않는다.
- 청동 표지 합계 11: mud_brick 1, copper_smelting 3, mould_casting 2, bronze_alloy 3, wheel 2. 기존 신석기 표지 합계 16은 유지된다. wheel은 제작 아이템만 연결되며 운반 효과는 M-GROW 부채이다.

## 새 47개 및 승격·표지 변경 2개

| 인덱스 / 노드 | 상태 / 시대 / tier | threshold | difficulty | seed | marker_weight | prereqs | discover 현재 값 |
|---|---|---:|---:|---|---:|---|---|
| 65 `mud_brick` (L274) | Connected / bronze / 3 | 300 | 2 | None | 1 | `["clay_prep", "oven"]` | `[Practice(job_kind: "craft", count: 3), Stockpile("clay", 4)]` |
| 73 `barter` (L313) | Connected / neolithic / 1 | 300 | 1 | None | 0 | `["storytelling"]` | `[Knows("storytelling", 300), Trades(1)]` |
| 82 `charcoal` (L362) | Connected / bronze / 1 | 300 | 1 | None | 0 | `["fire_keeping", "digging"]` | `[Practice(job_kind: "craft", count: 2), Stockpile("log", 4)]` |
| 83 `prospecting` (L366) | Connected / bronze / 1 | 300 | 1 | None | 0 | `["stone_quarrying"]` | `[Practice(job_kind: "quarry", count: 2), Stockpile("stone", 4)]` |
| 84 `mining` (L370) | Connected / bronze / 1 | 300 | 1 | None | 0 | `["prospecting", "pressure_flaking"]` | `[Practice(job_kind: "quarry", count: 3), Knows("prospecting", 300)]` |
| 85 `copper_smelting` (L374) | Connected / bronze / 2 | 300 | 2 | None | 3 | `["charcoal", "mining", "pot_firing"]` | `[Practice(job_kind: "craft", count: 3), Stockpile("copper_ore", 2)]` |
| 86 `cold_hammering` (L378) | Connected / bronze / 2 | 300 | 2 | None | 0 | `["copper_smelting"]` | `[Practice(job_kind: "craft", count: 3), Stockpile("copper_ingot", 1)]` |
| 87 `mould_casting` (L382) | Connected / bronze / 3 | 300 | 3 | None | 2 | `["copper_smelting", "clay_prep"]` | `[Practice(job_kind: "craft", count: 4), Stockpile("copper_ingot", 1), Stockpile("clay", 2)]` |
| 88 `tin_sourcing` (L386) | Connected / bronze / 3 | 300 | 3 | None | 0 | `["prospecting", "barter"]` | `[Trades(1), Knows("prospecting", 300)]` |
| 89 `bronze_alloy` (L391) | Connected / bronze / 4 | 300 | 3 | None | 3 | `["mould_casting", "tin_sourcing"]` | `[Practice(job_kind: "craft", count: 5), Stockpile("copper_ingot", 3)]` |
| 90 `annealing` (L395) | Inert / bronze / 4 | 300 | 3 | None | 0 | `["bronze_alloy"]` | `[Practice(job_kind: "craft", count: 5), Knows("bronze_alloy", 300)]` |
| 91 `lost_wax` (L399) | Inert / bronze / 4 | 300 | 3 | None | 0 | `["bronze_alloy", "beekeeping"]` | `[Practice(job_kind: "craft", count: 5), Knows("bronze_alloy", 300)]` |
| 92 `beekeeping` (L403) | Inert / bronze / 2 | 300 | 2 | None | 0 | `["foraging"]` | `[Practice(job_kind: "harvest", count: 3), Knows("foraging", 300)]` |
| 93 `wheel` (L407) | Connected / bronze / 2 | 300 | 2 | None | 2 | `["adze"]` | `[Practice(job_kind: "craft", count: 3), Stockpile("log", 2)]` |
| 94 `cart` (L411) | Inert / bronze / 3 | 300 | 3 | None | 0 | `["wheel", "herding"]` | `[Practice(job_kind: "craft", count: 4), Knows("wheel", 300)]` |
| 95 `cattle` (L415) | Connected / bronze / 2 | 300 | 2 | None | 0 | `["herding"]` | `[Practice(job_kind: "capture", count: 2), Knows("herding", 300)]` |
| 96 `plough` (L419) | Inert / bronze / 3 | 300 | 3 | None | 0 | `["cattle", "adze"]` | `[Practice(job_kind: "sow_field", count: 3), Knows("cattle", 300)]` |
| 97 `irrigation` (L423) | Inert / bronze / 3 | 300 | 3 | None | 0 | `["digging", "cultivation"]` | `[Practice(job_kind: "dig", count: 3), Knows("cultivation", 300)]` |
| 98 `terrace` (L427) | Inert / bronze / 4 | 300 | 3 | None | 0 | `["irrigation", "stone_walling"]` | `[Practice(job_kind: "build", count: 4), Knows("irrigation", 300)]` |
| 99 `wool_spinning` (L431) | Connected / bronze / 2 | 300 | 2 | None | 0 | `["herding", "spinning"]` | `[Practice(job_kind: "craft", count: 3), Knows("spinning", 300)]` |
| 100 `mud_brick_house` (L435) | Connected / bronze / 1 | 300 | 1 | None | 0 | `["mud_brick"]` | `[Practice(job_kind: "build", count: 2), Knows("mud_brick", 300)]` |
| 101 `lime_plaster` (L439) | Inert / bronze / 2 | 300 | 2 | None | 0 | `["oven", "stone_quarrying"]` | `[Practice(job_kind: "craft", count: 3), Knows("oven", 300)]` |
| 102 `stone_masonry` (L443) | Connected / bronze / 3 | 300 | 3 | None | 0 | `["stone_walling", "bronze_alloy"]` | `[Practice(job_kind: "build", count: 4), Stockpile("stone", 8)]` |
| 103 `chief_hall` (L447) | Inert / bronze / 2 | 300 | 2 | None | 0 | `["elder_council", "mud_brick_house"]` | `[Practice(job_kind: "build", count: 3), Knows("elder_council", 300)]` |
| 104 `chieftainship` (L451) | Inert / bronze / 3 | 300 | 3 | None | 0 | `["chief_hall"]` | `[Knows("chief_hall", 300), Practice(job_kind: "build", count: 4)]` |
| 105 `shrine` (L455) | Inert / bronze / 2 | 300 | 2 | None | 0 | `["ritual", "mud_brick"]` | `[Practice(job_kind: "build", count: 3), Knows("ritual", 300)]` |
| 106 `marriage_alliance` (L459) | Inert / bronze / 3 | 300 | 3 | None | 0 | `["kinship", "barter"]` | `[Trades(2), Knows("kinship", 300)]` |
| 107 `palisade` (L463) | Connected / bronze / 1 | 300 | 1 | None | 0 | `["timber_wall"]` | `[Practice(job_kind: "build", count: 2), Stockpile("log", 4)]` |
| 108 `ditch` (L467) | Inert / bronze / 2 | 300 | 2 | None | 0 | `["digging"]` | `[Practice(job_kind: "dig", count: 3), Knows("digging", 300)]` |
| 109 `bronze_weapons` (L471) | Connected / bronze / 4 | 300 | 3 | None | 0 | `["bronze_alloy"]` | `[Practice(job_kind: "craft", count: 5), Stockpile("bronze_ingot", 1)]` |
| 110 `composite_bow` (L475) | Connected / bronze / 3 | 300 | 3 | None | 0 | `["bow", "hide_working"]` | `[Practice(job_kind: "craft", count: 4), Knows("bow", 300)]` |
| 111 `caravan` (L479) | Inert / bronze / 3 | 300 | 3 | None | 0 | `["cart", "barter"]` | `[Trades(2), Knows("cart", 300)]` |
| 112 `trading_post` (L483) | Inert / bronze / 2 | 300 | 2 | None | 0 | `["barter", "granary"]` | `[Trades(2), Practice(job_kind: "build", count: 3)]` |
| 113 `seal_stamp` (L487) | Inert / bronze / 2 | 300 | 2 | None | 0 | `["tally", "clay_prep"]` | `[Practice(job_kind: "craft", count: 3), Knows("tally", 300)]` |
| 114 `proto_writing` (L491) | Inert / bronze / 3 | 300 | 3 | None | 0 | `["seal_stamp"]` | `[Practice(job_kind: "craft", count: 4), Knows("seal_stamp", 300)]` |
| 115 `standard_weights` (L495) | Inert / bronze / 3 | 300 | 3 | None | 0 | `["tally"]` | `[Trades(2), Knows("tally", 300)]` |
| 116 `solar_calendar` (L499) | Inert / bronze / 3 | 300 | 3 | None | 0 | `["lunar_calendar"]` | `[Practice(job_kind: "sow_field", count: 3), Knows("lunar_calendar", 300)]` |
| 117 `reed_boat` (L503) | Inert / bronze / 3 | 300 | 3 | None | 0 | `["cordage", "weaving"]` | `[Practice(job_kind: "craft", count: 4), Knows("weaving", 300)]` |
| 118 `sail` (L507) | Inert / bronze / 3 | 300 | 3 | None | 0 | `["cordage", "weaving"]` | `[Practice(job_kind: "craft", count: 4), Knows("weaving", 300)]` |
| 119 `wine` (L511) | Inert / bronze / 3 | 300 | 3 | None | 0 | `["fermentation", "orchard"]` | `[Practice(job_kind: "craft", count: 4), Knows("fermentation", 300)]` |
| 120 `olive_press` (L515) | Inert / bronze / 3 | 300 | 3 | None | 0 | `["fermentation", "orchard"]` | `[Practice(job_kind: "craft", count: 4), Knows("orchard", 300)]` |
| 121 `glass_beads` (L519) | Inert / bronze / 4 | 300 | 3 | None | 0 | `["bronze_alloy"]` | `[Practice(job_kind: "craft", count: 5), Knows("bronze_alloy", 300)]` |
| 122 `faience` (L523) | Inert / bronze / 4 | 300 | 3 | None | 0 | `["bronze_alloy"]` | `[Practice(job_kind: "craft", count: 5), Knows("bronze_alloy", 300)]` |
| 123 `trepanation` (L527) | Inert / bronze / 4 | 300 | 3 | None | 0 | `["bone_setting", "bronze_weapons"]` | `[Knows("bone_setting", 300), Knows("bronze_weapons", 300)]` |
| 124 `surgery` (L531) | Inert / bronze / 4 | 300 | 3 | None | 0 | `["bone_setting", "bronze_weapons"]` | `[Knows("bone_setting", 300), Knows("bronze_weapons", 300)]` |
| 125 `drum` (L535) | Inert / bronze / 1 | 300 | 1 | None | 0 | `["ritual"]` | `[Practice(job_kind: "craft", count: 2), Knows("ritual", 300)]` |
| 126 `flute` (L539) | Inert / bronze / 1 | 300 | 1 | None | 0 | `["ritual"]` | `[Practice(job_kind: "craft", count: 2), Knows("ritual", 300)]` |
| 127 `mask` (L543) | Inert / bronze / 2 | 300 | 2 | None | 0 | `["ritual"]` | `[Practice(job_kind: "craft", count: 3), Knows("ritual", 300)]` |
| 128 `tattoo` (L547) | Inert / bronze / 2 | 300 | 2 | None | 0 | `["ritual"]` | `[Practice(job_kind: "craft", count: 3), Knows("ritual", 300)]` |

## 그 밖의 기존 노드 변경

- `pot_firing` (인덱스 19, L74): kiln Workshop gate 추가만 있다. 기존 수치 threshold 400, difficulty 2, seed None, marker_weight 3, discover `[Stockpile("raw_pot", 2)]`는 BASE와 같다.
- `mud_brick`: 기존 ID 65와 threshold 300, difficulty 2, seed None, 발견 craft 3회 + clay 4는 유지한다. era neolithic→bronze, Inert→Connected, marker_weight 0→1, 실제 make_mud_brick·mud_brick_wall gate를 추가했다.
- `barter`: 기존 ID 73와 threshold 300, difficulty 1, seed None, marker_weight 0을 유지한다. Inert→Connected이며 발견 조건 중 `Practice(job_kind: "craft", count: 2)`를 `Trades(1)`로 바꾸어 실제 거래 증거를 연결했다. Knows(storytelling, 300)은 유지한다.
- `spinning`: 데이터 수치·gate 변경 없음. Bible의 flax_retting은 기존 ret_flax/retting_pit gate로 이미 연결되므로 별도 중복 노드를 만들지 않는 주석과 회귀 단언만 추가했다.

## 주석 범위와 잔여 검증

- 파일 상단 L5–17은 threshold·difficulty·discover·marker·seed 의미와 Trades 성공 교환을 설명한다. L169–171 C1 주석은 기존 승격 대상의 300‰·난이도·작업 누계 초기값 의도를 설명한다. L358–361 C2 주석은 초기 설계값임을 밝히고 threshold 300‰, difficulty=tier(최대 3), seed None, 완료 작업 2–5회, 실물 재고 1–8개, 청동 표지별 가중치를 설명한다.
- 개별 노드의 debt 문자열은 기능이 빠진 이유와 C3/M8/M9/M-GROW 등의 후속 패스를 명시한다. 모든 새 노드에 개별 수치별 근거 주석이 있는 것은 아니다. 공통 난이도 tier 규칙과 표지 수치의 초기 설계 의도는 데이터 공통 주석과 이 기록에 명시되어 있다. seed None·threshold 300·5개 표지의 실제 값은 해당 회귀 시험에서도 확인한다.
- 광맥·레시피·동물 수치는 각 담당 표에 포함하며 이 표가 중복 소유하지 않는다. 미연결 실제 채광·주석 요청·청동 도구 사용 사실은 C2-b 인계 항목이다.
- 추출 확인: 전체 129, 새 47 = Connected 16 / Inert 31, 이 표 49행. BASE 대비 기존 데이터 변경은 pot_firing·mud_brick·barter 3개뿐임을 비교했다. 모든 신규 발견 조건이 비어 있지 않음은 c2_knowledge 시험에서 단언한다.

## Freeze 전 인과 순환 교정

`bronze_alloy`에서 `Stockpile("tin_ingot", 1)` 발견 조건만 제거했다. 첫 주석 요청 카드가 bronze_alloy 보유를 요구하므로, 주석을 먼저 요구하면 카드로 첫 주석을 얻는 경로가 순환하기 때문이다. craft 5회·copper_ingot 3개, Bible의 mould_casting+tin_sourcing 선행, threshold/difficulty/seed/marker는 그대로다. 실제 합금 레시피의 주석 투입도 유지한다. 데이터 주석 및 `D-C2-KNOWLEDGE.md`에 근거를 남겼다. 조건 회귀의 수정 전 실패 로그는 `a-knowledge-regression-red.log`이며 채택률·생존 결과에 맞춘 수치 조정이 아니다.

### C2-a complete presentation and cattle parameter inventory

D-C2-1 defers custom species geometry to the separate art-import branch.
C2 retains category/manifest art and the following unchanged biological data.

| Cattle policy | Current value | Intended effect | Suspected issue |
| --- | --- | --- | --- |
| Source and escape | all3climates, plain, elevationNone; population2–4, respawn8days, herd12; hp30; flee true, distance6, speed2, steps4, rest40, home12, min hearth20 | Ordinary physical grazer source | Starting C1-style values, no habitat balance |
| Capture and age | cattle+cord; range40, work60, success700‰, retry1day; initial age0–29, adult30, breed20days, max age1800 | Existing livestock lifecycle | No new cattle-specific efficacy |
| Feed and products | graze3cells/day, starve3days, regrow1day; milk1/day with dairy, work45; slaughter90; meat3, hide1; wild meat3withcutter/2without, hide1 | Existing exact physical products | Food and labour competition unbalanced |

| Material look | Current Lab delta / category / stable code | Intended effect | Suspected issue |
| --- | --- | --- | --- |
| copper_ore | `{"tint":{"dL":-8,"da":-12,"db":4}}` | Shared placeholder colour | Presentation only, no material efficacy |
| tin_ore | `{"tint":{"dL":-9,"da":-2,"db":-10}}` | Shared placeholder colour | Presentation only, no material efficacy |
| iron_ore | `{"tint":{"dL":-18,"da":4,"db":2}}` | Shared placeholder colour | Presentation only, no material efficacy |
| copper | `{"category":"Mineral","sim_id":4142247758,"tint":{"dL":-5,"da":24,"db":25}}` | Shared placeholder colour | Presentation only, no material efficacy |
| tin | `{"category":"Mineral","sim_id":193506896,"tint":{"dL":8,"da":-2,"db":-8}}` | Shared placeholder colour | Presentation only, no material efficacy |
| bronze | `{"category":"Mineral","sim_id":4106632693,"tint":{"dL":-8,"da":10,"db":29}}` | Shared placeholder colour | Presentation only, no material efficacy |
| mud_brick | `{"category":"Mineral","sim_id":2075554421,"tint":{"dL":-15,"da":10,"db":17}}` | Shared placeholder colour | Presentation only, no material efficacy |
| packed_earth | `{"category":"Mineral","sim_id":3087221664,"tint":{"dL":-22,"da":5,"db":12}}` | Shared placeholder colour | Presentation only, no material efficacy |
