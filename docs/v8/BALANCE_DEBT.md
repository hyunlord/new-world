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
