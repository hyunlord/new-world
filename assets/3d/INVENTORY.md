# M3D-ASSETS content inventory

Existing-model baseline: `origin/v8/3d` = `a0260ce9a91e8328ae855619a1d0a89d23fdc535`.
Provisional content: baseline plus local `v8/c3` = `cfeb57c9e1906eedd4e5d14447ba4faca20c4047` additions.

**Source warning:** `origin/v8/c3` does not exist on the remote (fetch and ls-remote checked 2026-11-15). Local C3 is recorded separately; this is not a claim to have inventoried the missing remote ref. Regenerate/reconcile when the owner supplies the authoritative ref.

Existing means a GLB in the frozen M3D-1 manifest. Shared/generic mappings and procedural renderer profiles are explicitly distinguished from dedicated models. This is the before-work inventory; newly generated assets are delivered separately.

## buildings (25)

Source: `rust/crates/sim-core/data/colony/buildings.ron`.

| Code | Identity | Source | Existing model / representation | Gap |
|---|---|---|---|---|
| - | `hut` | M3D-1 | — | dedicated model missing |
| - | `log_house` | M3D-1 | — | dedicated model missing |
| - | `stone_house` | M3D-1 | — | dedicated model missing |
| - | `workshop_shed` | M3D-1 | — | dedicated model missing |
| - | `storehouse` | M3D-1 | — | dedicated model missing |
| - | `drying_shed` | M3D-1 | — | dedicated model missing |
| - | `granary` | M3D-1 | — | dedicated model missing |
| - | `pen` | M3D-1 | — | dedicated model missing |
| - | `well` | M3D-1 | well_body | shared anchor only; template assembled at runtime |
| - | `smokehouse` | M3D-1 | — | dedicated model missing |
| - | `kiln` | M3D-1 | kiln | none (existing dedicated GLB) |
| - | `mud_brick_house` | M3D-1 | — | dedicated model missing |
| - | `charcoal_clamp` | M3D-1 | charcoal_clamp | none (existing dedicated GLB) |
| - | `smelter` | M3D-1 | smelter | none (existing dedicated GLB) |
| - | `casting_bench` | M3D-1 | casting_bench | none (existing dedicated GLB) |
| - | `bloomery` | M3D-1 | — | dedicated model missing |
| - | `forge` | M3D-1 | — | dedicated model missing |
| - | `shrine` | M3D-1 | — | dedicated model missing |
| - | `chief_hall` | M3D-1 | — | dedicated model missing |
| - | `trading_post` | M3D-1 | — | dedicated model missing |
| - | `market` | M3D-1 | — | dedicated model missing |
| - | `warehouse` | M3D-1 | — | dedicated model missing |
| - | `oven_house` | M3D-1 | — | dedicated model missing |
| - | `bee_skep` | M3D-1 | bee_skep | none (existing dedicated GLB) |
| - | `cut_stone_house` | local C3 only | — | dedicated model missing |

## blueprints (33)

Source: `rust/crates/sim-core/data/colony/blueprints.ron`.

| Code | Identity | Source | Existing model / representation | Gap |
|---|---|---|---|---|
| 0 | `hut` | M3D-1 | — | dedicated model missing |
| 1 | `knapping_spot` | M3D-1 | knapping_spot | none (existing dedicated GLB) |
| 2 | `wood_bench` | M3D-1 | wood_bench | none (existing dedicated GLB) |
| 3 | `drying_rack` | M3D-1 | drying_rack | none (existing dedicated GLB) |
| 4 | `log_wall` | M3D-1 | runtime procedural profile | standalone GLB / construction variants missing |
| 5 | `stone_wall` | M3D-1 | runtime procedural profile | standalone GLB / construction variants missing |
| 6 | `door` | M3D-1 | door | none (existing dedicated GLB) |
| 7 | `packed_floor` | M3D-1 | runtime procedural profile | standalone GLB / construction variants missing |
| 8 | `bedroll` | M3D-1 | bedroll | none (existing dedicated GLB) |
| 9 | `grinding_stone` | M3D-1 | grinding_stone | none (existing dedicated GLB) |
| 10 | `pottery_bench` | M3D-1 | pottery_bench | none (existing dedicated GLB) |
| 11 | `hearth_fire` | M3D-1 | hearth | shared existing station model |
| 12 | `food_drying_rack` | M3D-1 | drying_rack | shared existing station model |
| 13 | `retting_pit` | M3D-1 | retting_pit | none (existing dedicated GLB) |
| 14 | `loom` | M3D-1 | loom | none (existing dedicated GLB) |
| 15 | `fence` | M3D-1 | runtime procedural profile | standalone GLB / construction variants missing |
| 16 | `clay_oven` | M3D-1 | clay_oven | none (existing dedicated GLB) |
| 17 | `smoking_rack` | M3D-1 | drying_rack | shared existing station model |
| 18 | `charcoal_clamp` | M3D-1 | charcoal_clamp | none (existing dedicated GLB) |
| 19 | `smelter` | M3D-1 | smelter | none (existing dedicated GLB) |
| 20 | `casting_bench` | M3D-1 | casting_bench | none (existing dedicated GLB) |
| 21 | `mud_brick_wall` | M3D-1 | runtime procedural profile | standalone GLB / construction variants missing |
| 22 | `palisade` | M3D-1 | runtime procedural profile | standalone GLB / construction variants missing |
| 23 | `kiln` | M3D-1 | kiln | none (existing dedicated GLB) |
| 24 | `stone_masonry_wall` | M3D-1 | runtime procedural profile | standalone GLB / construction variants missing |
| 25 | `bloomery` | local C3 only | — | dedicated model missing |
| 26 | `forge` | local C3 only | — | dedicated model missing |
| 27 | `anvil` | local C3 only | anvil | none (existing dedicated GLB) |
| 28 | `mason_bench` | local C3 only | — | dedicated model missing |
| 29 | `cut_stone_wall` | local C3 only | — | dedicated model missing |
| 30 | `town_wall` | local C3 only | — | dedicated model missing |
| 31 | `town_gate` | local C3 only | — | dedicated model missing |
| 32 | `paved_road` | local C3 only | — | dedicated model missing |

## items (119)

Source: `rust/crates/sim-core/data/colony/items.ron`.

| Code | Identity | Source | Existing model / representation | Gap |
|---|---|---|---|---|
| 0 | `log` | M3D-1 | log_pile_s, log_pile_m, log_pile_l | shared silhouette; per-code/stack variants incomplete |
| 1 | `firewood` | M3D-1 | firewood | none (existing dedicated GLB) |
| 2 | `stone` | M3D-1 | stone_pile | shared silhouette; per-code/stack variants incomplete |
| 3 | `flint` | M3D-1 | flint_pile | shared silhouette; per-code/stack variants incomplete |
| 4 | `flint_blade` | M3D-1 | tool | shared silhouette; per-code/stack variants incomplete |
| 5 | `stone_axe` | M3D-1 | tool | shared silhouette; per-code/stack variants incomplete |
| 6 | `hide` | M3D-1 | hide | none (existing dedicated GLB) |
| 7 | `clay` | M3D-1 | — | dedicated model missing |
| 8 | `raw_pot` | M3D-1 | pot | shared silhouette; per-code/stack variants incomplete |
| 9 | `clay_pot` | M3D-1 | — | dedicated model missing |
| 10 | `ground_axe` | M3D-1 | — | dedicated model missing |
| 11 | `meat` | M3D-1 | meat | none (existing dedicated GLB) |
| 12 | `cured_hide` | M3D-1 | — | dedicated model missing |
| 13 | `hide_wrap` | M3D-1 | — | dedicated model missing |
| 14 | `fur_cloak` | M3D-1 | — | dedicated model missing |
| 15 | `spear` | M3D-1 | tool | shared silhouette; per-code/stack variants incomplete |
| 16 | `wolf_hide` | M3D-1 | — | dedicated model missing |
| 17 | `berries` | M3D-1 | — | dedicated model missing |
| 18 | `hazelnuts` | M3D-1 | — | dedicated model missing |
| 19 | `wild_grain` | M3D-1 | — | dedicated model missing |
| 20 | `roots` | M3D-1 | — | dedicated model missing |
| 21 | `flax_stalks` | M3D-1 | — | dedicated model missing |
| 22 | `roast_meat` | M3D-1 | — | dedicated model missing |
| 23 | `porridge` | M3D-1 | — | dedicated model missing |
| 24 | `stew` | M3D-1 | — | dedicated model missing |
| 25 | `dried_meat` | M3D-1 | — | dedicated model missing |
| 26 | `dried_berries` | M3D-1 | — | dedicated model missing |
| 27 | `roasted_roots` | M3D-1 | — | dedicated model missing |
| 28 | `fruit_porridge` | M3D-1 | — | dedicated model missing |
| 29 | `legacy_food` | M3D-1 | — | dedicated model missing |
| 30 | `flax_seed` | M3D-1 | — | dedicated model missing |
| 31 | `emmer_seed` | M3D-1 | — | dedicated model missing |
| 32 | `flax_fibre` | M3D-1 | — | dedicated model missing |
| 33 | `linen_thread` | M3D-1 | — | dedicated model missing |
| 34 | `linen_cloth` | M3D-1 | — | dedicated model missing |
| 35 | `hand_spindle` | M3D-1 | — | dedicated model missing |
| 36 | `linen_tunic` | M3D-1 | — | dedicated model missing |
| 37 | `linen_wrap` | M3D-1 | — | dedicated model missing |
| 38 | `cord` | M3D-1 | — | dedicated model missing |
| 39 | `basket` | M3D-1 | basket | none (existing dedicated GLB) |
| 40 | `sling` | M3D-1 | — | dedicated model missing |
| 41 | `bone_needle` | M3D-1 | — | dedicated model missing |
| 42 | `fish` | M3D-1 | — | dedicated model missing |
| 43 | `flour` | M3D-1 | — | dedicated model missing |
| 44 | `bread` | M3D-1 | — | dedicated model missing |
| 45 | `beer` | M3D-1 | — | dedicated model missing |
| 46 | `milk` | M3D-1 | — | dedicated model missing |
| 47 | `cheese` | M3D-1 | — | dedicated model missing |
| 48 | `smoked_meat` | M3D-1 | — | dedicated model missing |
| 49 | `hay` | M3D-1 | — | dedicated model missing |
| 50 | `hoe` | M3D-1 | — | dedicated model missing |
| 51 | `sickle` | M3D-1 | — | dedicated model missing |
| 52 | `adze` | M3D-1 | — | dedicated model missing |
| 53 | `bow` | M3D-1 | — | dedicated model missing |
| 54 | `arrow` | M3D-1 | — | dedicated model missing |
| 55 | `dyed_tunic` | M3D-1 | — | dedicated model missing |
| 56 | `leather_armor` | M3D-1 | — | dedicated model missing |
| 57 | `herbs` | M3D-1 | — | dedicated model missing |
| 58 | `poultice` | M3D-1 | — | dedicated model missing |
| 59 | `barley` | M3D-1 | — | dedicated model missing |
| 60 | `barley_seed` | M3D-1 | — | dedicated model missing |
| 61 | `rye` | M3D-1 | — | dedicated model missing |
| 62 | `rye_seed` | M3D-1 | — | dedicated model missing |
| 63 | `lentil` | M3D-1 | — | dedicated model missing |
| 64 | `lentil_seed` | M3D-1 | — | dedicated model missing |
| 65 | `pea` | M3D-1 | — | dedicated model missing |
| 66 | `pea_seed` | M3D-1 | — | dedicated model missing |
| 67 | `millet` | M3D-1 | — | dedicated model missing |
| 68 | `millet_seed` | M3D-1 | — | dedicated model missing |
| 69 | `apple` | M3D-1 | — | dedicated model missing |
| 70 | `goat_hide` | M3D-1 | — | dedicated model missing |
| 71 | `wool` | M3D-1 | — | dedicated model missing |
| 72 | `charcoal` | M3D-1 | — | dedicated model missing |
| 73 | `copper_ore` | M3D-1 | — | dedicated model missing |
| 74 | `tin_ore` | M3D-1 | — | dedicated model missing |
| 75 | `iron_ore` | M3D-1 | — | dedicated model missing |
| 76 | `copper_ingot` | M3D-1 | — | dedicated model missing |
| 77 | `tin_ingot` | M3D-1 | — | dedicated model missing |
| 78 | `bronze_ingot` | M3D-1 | — | dedicated model missing |
| 79 | `clay_mould` | M3D-1 | — | dedicated model missing |
| 80 | `copper_awl` | M3D-1 | — | dedicated model missing |
| 81 | `copper_needle` | M3D-1 | — | dedicated model missing |
| 82 | `copper_axe` | M3D-1 | — | dedicated model missing |
| 83 | `bronze_axe` | M3D-1 | — | dedicated model missing |
| 84 | `bronze_sickle` | M3D-1 | — | dedicated model missing |
| 85 | `bronze_hoe` | M3D-1 | — | dedicated model missing |
| 86 | `bronze_spear` | M3D-1 | — | dedicated model missing |
| 87 | `bronze_dagger` | M3D-1 | — | dedicated model missing |
| 88 | `honey` | M3D-1 | — | dedicated model missing |
| 89 | `beeswax` | M3D-1 | — | dedicated model missing |
| 90 | `wool_yarn` | M3D-1 | — | dedicated model missing |
| 91 | `wool_cloth` | M3D-1 | — | dedicated model missing |
| 92 | `wool_tunic` | M3D-1 | — | dedicated model missing |
| 93 | `wheel` | M3D-1 | — | dedicated model missing |
| 94 | `mud_brick` | M3D-1 | — | dedicated model missing |
| 95 | `lime` | M3D-1 | — | dedicated model missing |
| 96 | `seal` | M3D-1 | — | dedicated model missing |
| 97 | `tablet` | M3D-1 | — | dedicated model missing |
| 98 | `glass_bead` | M3D-1 | — | dedicated model missing |
| 99 | `composite_bow` | M3D-1 | — | dedicated model missing |
| 100 | `limestone` | M3D-1 | — | dedicated model missing |
| 101 | `bog_iron` | local C3 only | — | dedicated model missing |
| 102 | `iron_bloom` | local C3 only | — | dedicated model missing |
| 103 | `wrought_iron` | local C3 only | — | dedicated model missing |
| 104 | `iron_axe` | local C3 only | — | dedicated model missing |
| 105 | `iron_hoe` | local C3 only | — | dedicated model missing |
| 106 | `iron_sickle` | local C3 only | — | dedicated model missing |
| 107 | `iron_saw` | local C3 only | — | dedicated model missing |
| 108 | `iron_knife` | local C3 only | — | dedicated model missing |
| 109 | `iron_adze` | local C3 only | — | dedicated model missing |
| 110 | `iron_spear` | local C3 only | — | dedicated model missing |
| 111 | `iron_sword` | local C3 only | — | dedicated model missing |
| 112 | `iron_helmet` | local C3 only | — | dedicated model missing |
| 113 | `iron_shield_boss` | local C3 only | — | dedicated model missing |
| 114 | `nails` | local C3 only | — | dedicated model missing |
| 115 | `cut_stone` | local C3 only | — | dedicated model missing |
| 116 | `coin` | local C3 only | — | dedicated model missing |
| 117 | `book` | local C3 only | — | dedicated model missing |
| 118 | `steel_edge` | local C3 only | — | dedicated model missing |

## outcrops (4)

Source: `rust/crates/sim-core/data/colony/outcrops.ron`.

| Code | Identity | Source | Existing model / representation | Gap |
|---|---|---|---|---|
| 1 | `copper_outcrop` | M3D-1 | ore_outcrop (shared/tinted) | dedicated material variant missing |
| 2 | `tin_outcrop` | M3D-1 | ore_outcrop (shared/tinted) | dedicated material variant missing |
| 3 | `iron_outcrop` | M3D-1 | ore_outcrop (shared/tinted) | dedicated material variant missing |
| 4 | `bog_iron_outcrop` | local C3 only | ore_outcrop (shared/tinted) | dedicated material variant missing |

## plants (11)

Source: `rust/crates/sim-core/data/colony/plants.ron`.

| Code | Identity | Source | Existing model / representation | Gap |
|---|---|---|---|---|
| 0 | `berry_bush` | M3D-1 | berry_bush | none (existing dedicated GLB) |
| 1 | `hazel` | M3D-1 | — | dedicated model missing |
| 2 | `wild_flax` | M3D-1 | — | dedicated model missing |
| 3 | `wild_grain` | M3D-1 | — | dedicated model missing |
| 4 | `wild_root` | M3D-1 | — | dedicated model missing |
| 5 | `wild_barley` | M3D-1 | — | dedicated model missing |
| 6 | `wild_rye` | M3D-1 | — | dedicated model missing |
| 7 | `wild_lentil` | M3D-1 | — | dedicated model missing |
| 8 | `wild_pea` | M3D-1 | — | dedicated model missing |
| 9 | `wild_millet` | M3D-1 | — | dedicated model missing |
| 10 | `wild_apple` | M3D-1 | — | dedicated model missing |

## crops (7)

Source: `rust/crates/sim-core/data/colony/crops.ron`.

| Code | Identity | Source | Existing model / representation | Gap |
|---|---|---|---|---|
| 0 | `emmer` | M3D-1 | — | dedicated model missing |
| 1 | `flax` | M3D-1 | — | dedicated model missing |
| 2 | `barley` | M3D-1 | — | dedicated model missing |
| 3 | `rye` | M3D-1 | — | dedicated model missing |
| 4 | `lentil` | M3D-1 | — | dedicated model missing |
| 5 | `pea` | M3D-1 | — | dedicated model missing |
| 6 | `millet` | M3D-1 | — | dedicated model missing |

## animals (7)

Source: `rust/crates/sim-core/data/colony/animals.ron`.

| Code | Identity | Source | Existing model / representation | Gap |
|---|---|---|---|---|
| 0 | `wolf` | M3D-1 | wolf | none (existing dedicated GLB) |
| 1 | `deer` | M3D-1 | deer | none (existing dedicated GLB) |
| 2 | `wild_goat` | M3D-1 | wild_goat | none (existing dedicated GLB) |
| 3 | `wild_sheep` | M3D-1 | wild_sheep | none (existing dedicated GLB) |
| 4 | `wild_boar` | M3D-1 | wild_boar | none (existing dedicated GLB) |
| 5 | `reindeer` | M3D-1 | reindeer | none (existing dedicated GLB) |
| 6 | `aurochs` | M3D-1 | aurochs | none (existing dedicated GLB) |

## Tree species (3)

Source: `rust/crates/sim-core/data/colony/flora.ron`; all three identities agree in both refs.

| Identity | Existing models | Gap |
|---|---|---|
| `oak` | tree_oak_a, tree_oak_b, tree_oak_c | none |
| `pine` | tree_pine_a, tree_pine_b, tree_pine_c | none |
| `birch` | tree_birch_a, tree_birch_b | none |

## Construction styles / floors / roads

Source: `assets/v8art/materials/construction_styles.json` at local C3. Blueprint-specific C3 road/wall entries are also listed above.

| Style | Kind | Existing representation | Gap |
|---|---|---|---|
| `log` | wall | runtime log profile | no dedicated style GLB |
| `drystone` | wall | runtime block profile | no dedicated style GLB |
| `packed_earth` | floor | runtime block profile | no dedicated style GLB |
| `slab_door` | door | door GLB | no dedicated style GLB |
| `inset_door` | door | door GLB | no dedicated style GLB |
| `mud_brick` | wall | runtime block profile | no dedicated style GLB |
| `fired_brick` | wall | runtime block profile | no dedicated style GLB |

## Blueprint-admitted wall / floor materials

Source: `blueprints.ron` `materials` registry; actual use is constrained by each blueprint. Material color alone is not a dedicated model.

| Material | Existing tint |
|---|---|
| `granite` | explicit binding |
| `limestone` | category fallback |
| `flint` | explicit binding |
| `chert` | category fallback |
| `oak` | explicit binding |
| `pine` | explicit binding |
| `birch` | explicit binding |
| `clay` | explicit binding |
| `packed_earth` | explicit binding |
| `copper_ore` | explicit binding |
| `tin_ore` | explicit binding |
| `iron_ore` | explicit binding |
| `copper` | explicit binding |
| `tin` | explicit binding |
| `bronze` | explicit binding |
| `mud_brick` | explicit binding |
| `bog_iron` | category fallback |
| `iron` | category fallback |
| `cut_stone` | category fallback |

## Livestock and people

Livestock uses the same species IDs as wild animals: `wild_goat`, `wild_sheep`, `aurochs`; no separate livestock species identity exists in the animal registry. Existing GLBs are procedural M3D-1 placeholders, not TRELLIS outputs. People have one existing `villager` with shared clips; dedicated age/sex/era AI variants are absent.

## Totals

buildings: 25, blueprints: 33, items: 119, outcrops: 4, plants: 11, crops: 7, animals: 7.

Frozen baseline manifest: 62 models. Count variants exist for logs (`log_pile_s/m/l`); other per-code stack coverage is incomplete.
