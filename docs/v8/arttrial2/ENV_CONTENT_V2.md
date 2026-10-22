# Original environment cohort: content audit V2

Status: **38 PASS / 0 FAIL** for user-goal content identity only. No asset is promoted by this document.

Scope: the 38 original raw generations mapped by `ENV_RESCORE_V2.json`, not the later calm ground replacements or derived ghost sprites. Inspected `art/style05/boards/environment_first_attempt.png` and full-resolution originals for ambiguous species, counts, tools, openings and animation correspondence. Required content comes from the user goal in `TRIAL2_GOAL.md`. Agent-authored generation prompt details are observations only: extra exact log/stone counts and a specific bench tool cannot create a new rejection criterion. South means image bottom.

PASS means the user-requested asset identity is visibly supported. This audit does not redefine preregistered C2 (numeric AND model); content usability is reported separately. FAIL also covers a required feature that cannot be established visually; it does not claim unseen content is absent. Style scores, camera-quality scores, transparency, native-size readability, collision/path access, exact pixel animation identity and floor seams remain separate gates. Ground rows establish material identity only, not calmness or seamless tiling. Generic wetland reeds are not held to an unstated botanical species.

| Family | Asset | Content | Observed evidence / concrete issue |
|---|---|---|---|
| buildings | drying_rack | PASS | Two lashed end supports and horizontal drying rods carry hide/meat strips; south face is accessible. |
| buildings | hut | PASS | Roofless square enclosure, four bedrolls, central hearth and explicit opening through the south wall. |
| buildings | knapping_spot | PASS | Low oval anvil with flint core, antler hammer and three loose chips; south working side remains clear. |
| buildings | lean_to | PASS | Single rectangular sloping bark roof; open south underside between supports, not an A-frame tent. |
| buildings | storage_pit | PASS | Round wicker-lined pit with sack and lid resting at north; south approach is unobstructed. |
| buildings | woodworking_bench | PASS | Wide split-log bench on two trestles with worked branch and stone tool; open south working side. |
| ground | dark_grass | PASS | Dense dark green grass/vegetation surface, distinct from bare earth. |
| ground | forest_floor | PASS | Brown leaf litter and organic debris identify woodland floor. |
| ground | hill_rock | PASS | Exposed pale irregular rock and sparse vegetation identify rocky terrain. |
| ground | meadow | PASS | Green grass with earth showing between patches identifies meadow. |
| ground | packed_earth | PASS | Predominantly tan bare compact earth with small stones. |
| ground | river_bank | PASS | Dense rounded grey pebbles identify stony bank material; no water body required. |
| items | berry_basket | PASS | Wicker basket filled with red and dark berries. |
| items | firewood_bundle | PASS | Split wood gathered under one central fiber binding; multiple turns do not imply multiple separate cords. |
| items | flint_blade | PASS | Single narrow knapped stone blade without a handle. |
| items | flint_pile | PASS | Five distinguishable angular flint nodules. |
| items | hide | PASS | Single spread animal hide with irregular limb outline and pale turned underside. |
| items | log_pile_large | PASS | Visibly largest of the three log-pile sizes; stacked rough bark logs. |
| items | log_pile_medium | PASS | Seven distinguishable logs/end faces in stacked group. |
| items | log_pile_small | PASS | Three distinguishable parallel logs. |
| items | stone_axe | PASS | Wood handle with bound grey stone axe head. |
| items | stone_pile | PASS | Six distinguishable rounded irregular stones. |
| misc | boulder_large | PASS | Broad single irregular rock mass; larger form is distinguishable on board. |
| misc | boulder_small | PASS | Single low rounded rock mass, smaller than large variant on board. |
| misc | deadwood | PASS | One leafless broken branch with one main Y fork; short broken stubs are not additional full forks. |
| misc | hearth_0 | PASS | Stone-ring hearth with crossed fuel wood and bright flame; no smoke. |
| misc | hearth_1 | PASS | Alternate flame pose with visually matching stone/log layout; recognizable second hearth frame. |
| misc | smoke_puff | PASS | Three main overlapping curled smoke masses, with small detached wisps; no fire or ground. These are smoke, not a solid object. |
| misc | stump | PASS | Exposed concentric growth rings, bark rim and two principal cut root projections; no axe. |
| plants | berry_bare | PASS | Leafy bush without visible fruit clusters; minor buds/stem tips do not read as ripe berries. |
| plants | berry_full | PASS | Leafy bush with obvious red/burgundy and dark blue-black fruit clusters. |
| plants | birch_large | PASS | White bark with dark marks and toothed angular foliage distinguish birch; larger branching canopy. |
| plants | birch_small | PASS | White bark with dark bands and toothed angular foliage distinguish birch; compact canopy. |
| plants | oak_large | PASS | Broad branching canopy with clearly rounded lobed oak leaves. |
| plants | oak_small | PASS | Compact canopy with clearly rounded lobed oak leaves. |
| plants | pine_large | PASS | Woody radial branches with needle sprays and cones; conifer identity is explicit. |
| plants | pine_small | PASS | Full-resolution image shows radial needle sprays on woody branches; thumbnail resemblance to a palm is insufficient to reject it. |
| plants | reeds | PASS | Compact wetland blade clump with brown seed heads; fulfills requested generic wetland plant identity. |

## Prompt-fidelity observations (not user-goal failures)

- `log_pile_large`: ten distinguishable log ends rather than the agent prompt twelve; the user requested three pile sizes, which remain distinguishable.
- `hearth_0`: more than three wood members; twelve ring stones are not cleanly countable among smaller stones. These quantities were agent prompt additions.
- `hearth_1`: inherits those prompt-only quantities; flame changes while ring/log layout visually remains stable.
- `woodworking_bench`: tool reads as a rounded stone mallet rather than an adze. The user requested the bench and south working side, both present.

## Immutable reviewed inputs

Raw paths and SHA-256 below identify the exact bytes reviewed. Prompt sidecars reside alongside each family input as `<name>_try1.json`. Updating this audit requires regenerating its evidence hash in the external semantic ledger.

| Asset | Raw input | SHA-256 |
|---|---|---|
| drying_rack | [drying_rack_try1_raw.png](/Users/rexxa/work/arttrial2-data/buildings/drying_rack_try1_raw.png) | `9260e2ecddf023899e47a7d6372f3b415b58e2dfd75efe80120df6dc8796ad04` |
| hut | [hut_try1_raw.png](/Users/rexxa/work/arttrial2-data/buildings/hut_try1_raw.png) | `7c9245285c94b2197a37c7197d3a2c6362832281274699f12dab1edc478e349c` |
| knapping_spot | [knapping_spot_try1_raw.png](/Users/rexxa/work/arttrial2-data/buildings/knapping_spot_try1_raw.png) | `5366e8a986fef2c7be0265dda2173aa59813c94625f83de53892a7d7353668b6` |
| lean_to | [lean_to_try1_raw.png](/Users/rexxa/work/arttrial2-data/buildings/lean_to_try1_raw.png) | `da16377401004d8bbe94e214269779424be80812f4e98c1ca46b747a1f76a860` |
| storage_pit | [storage_pit_try1_raw.png](/Users/rexxa/work/arttrial2-data/buildings/storage_pit_try1_raw.png) | `7329f70b059fa51357a73879719b037304eebf6dd1f6d32002efd159346c6846` |
| woodworking_bench | [woodworking_bench_try1_raw.png](/Users/rexxa/work/arttrial2-data/buildings/woodworking_bench_try1_raw.png) | `ef89981eecd35eedb7f7346aa0ef74806810837bb4048af7dc12693bc8045c95` |
| dark_grass | [dark_grass_try1.png](/Users/rexxa/work/arttrial2-data/ground/dark_grass_try1.png) | `d75ccdcf7daeb3fa3e7a5ef7c266b5555677fdd342c4c14c08f1474eb3bd5877` |
| forest_floor | [forest_floor_try1.png](/Users/rexxa/work/arttrial2-data/ground/forest_floor_try1.png) | `2c9b89f62288521acca421e6cdcda9e507bc28f9ac63cc5631c353b9d5d3bdf3` |
| hill_rock | [hill_rock_try1.png](/Users/rexxa/work/arttrial2-data/ground/hill_rock_try1.png) | `b5adda31a55d978afa0ae7dd1964dd721416733aa253f3a586fc582427aa9387` |
| meadow | [meadow_try1.png](/Users/rexxa/work/arttrial2-data/ground/meadow_try1.png) | `b3210a96d3fa952fce70812bc4bae6cd57167b354d152cb0c13080f06f6e4e69` |
| packed_earth | [packed_earth_try1.png](/Users/rexxa/work/arttrial2-data/ground/packed_earth_try1.png) | `a113faed21b7d7c2119becce8bed398e270ba54814fcf4c0679bacf3238d3a07` |
| river_bank | [river_bank_try1.png](/Users/rexxa/work/arttrial2-data/ground/river_bank_try1.png) | `dee0e187becb221cda21be6dd5bca108a8d22657d6ab5d1d6e23e262eff6ccfa` |
| berry_basket | [berry_basket_try1_raw.png](/Users/rexxa/work/arttrial2-data/items/berry_basket_try1_raw.png) | `c2c9c4295bc441e13b2eb44e37eaa27e6f674482316aa211e66ffd0e74227d7e` |
| firewood_bundle | [firewood_bundle_try1_raw.png](/Users/rexxa/work/arttrial2-data/items/firewood_bundle_try1_raw.png) | `b79067ceb3f1bf3ea5614abde41a8268148b875141cf2a2c9d9aa9ccc5dbbb24` |
| flint_blade | [flint_blade_try1_raw.png](/Users/rexxa/work/arttrial2-data/items/flint_blade_try1_raw.png) | `72327fc0c0977e3fe82e289dd512282f203102f1c90ca2c295fa8e498ef6c902` |
| flint_pile | [flint_pile_try1_raw.png](/Users/rexxa/work/arttrial2-data/items/flint_pile_try1_raw.png) | `9282863c3d914dfc90f53be509dd524ce94cdf2b7947318e4669bad9cd6b930f` |
| hide | [hide_try1_raw.png](/Users/rexxa/work/arttrial2-data/items/hide_try1_raw.png) | `caa87d1d2a2341b252848aa857b46c74c84cc9df190f114c526a5700f83d1c09` |
| log_pile_large | [log_pile_large_try1_raw.png](/Users/rexxa/work/arttrial2-data/items/log_pile_large_try1_raw.png) | `86b7eb2f5a13f7bfa062e1f7b1d1dc19554e519b0cc4867741199efff3084de3` |
| log_pile_medium | [log_pile_medium_try1_raw.png](/Users/rexxa/work/arttrial2-data/items/log_pile_medium_try1_raw.png) | `4eac3a54f3c36c345a4354b9595664ca2a4879560ecdb108ad8b90d728bd21da` |
| log_pile_small | [log_pile_small_try1_raw.png](/Users/rexxa/work/arttrial2-data/items/log_pile_small_try1_raw.png) | `171f434320e7882c8d10e8747609bc57f5c2cb63154c164285cb42355f7ce03e` |
| stone_axe | [stone_axe_try1_raw.png](/Users/rexxa/work/arttrial2-data/items/stone_axe_try1_raw.png) | `4b3036e4b09a885faf52a4a2652720a9d84f137db2714496e75446ed58a0a949` |
| stone_pile | [stone_pile_try1_raw.png](/Users/rexxa/work/arttrial2-data/items/stone_pile_try1_raw.png) | `9312d5d8c0968bf43b79777a45653015e678a4851cce4d47edd567afc98f3f57` |
| boulder_large | [boulder_large_try1_raw.png](/Users/rexxa/work/arttrial2-data/misc/boulder_large_try1_raw.png) | `3f8fa6707dc3b0520cbee786be7404b82463e7aefd10dbac5f5f5eef932cb3e4` |
| boulder_small | [boulder_small_try1_raw.png](/Users/rexxa/work/arttrial2-data/misc/boulder_small_try1_raw.png) | `a21c391bb48be9e4d444b44c5a79c60ca72aa20994b21f012f2c70e0dc973b66` |
| deadwood | [deadwood_try1_raw.png](/Users/rexxa/work/arttrial2-data/misc/deadwood_try1_raw.png) | `f70402a6330d49f80c02734fcca0aea868717cc5f2eed685b1bf7ece90df61da` |
| hearth_0 | [hearth_0_try1_raw.png](/Users/rexxa/work/arttrial2-data/misc/hearth_0_try1_raw.png) | `6464701d4ee34ffe19b9490357a5a58170c9c4decdaf060d34d255b35c2ff1ea` |
| hearth_1 | [hearth_1_try1_raw.png](/Users/rexxa/work/arttrial2-data/misc/hearth_1_try1_raw.png) | `ac22e3c934dc94b34e54e24ebec34bda137914ee8ab9bf18addf5c18b35068ee` |
| smoke_puff | [smoke_puff_try1_raw.png](/Users/rexxa/work/arttrial2-data/misc/smoke_puff_try1_raw.png) | `1767524682b0e5e8feb98000336dc7b7096ce0de70e59c8894519b07bf3fed67` |
| stump | [stump_try1_raw.png](/Users/rexxa/work/arttrial2-data/misc/stump_try1_raw.png) | `b228b0c685b03f7c395579ab4e094703a56ae3f68523033e0310341594f8e0ba` |
| berry_bare | [berry_bare_try1_raw.png](/Users/rexxa/work/arttrial2-data/plants/berry_bare_try1_raw.png) | `2b97bfe15775d055c744c5085a1a32d342663355a97c5083379caf3757faf7fd` |
| berry_full | [berry_full_try1_raw.png](/Users/rexxa/work/arttrial2-data/plants/berry_full_try1_raw.png) | `733ea5343add84d78bf95fd06e8ef57e77391524b65295b4d56cf4dca128ecbe` |
| birch_large | [birch_large_try1_raw.png](/Users/rexxa/work/arttrial2-data/plants/birch_large_try1_raw.png) | `127de65cf9c87bc6955d6065c274a4e9cc00c91ff2bc40dabbe629459bef7099` |
| birch_small | [birch_small_try1_raw.png](/Users/rexxa/work/arttrial2-data/plants/birch_small_try1_raw.png) | `f74c0f2b80f8eb51f8cc8c7fd6756c9ff9b2441a1d311112d24f7214ec32d7af` |
| oak_large | [oak_large_try1_raw.png](/Users/rexxa/work/arttrial2-data/plants/oak_large_try1_raw.png) | `84d9d90936d06762f40f64bb349d0b45475bc069c3dbb901939cb4a6c3965d7b` |
| oak_small | [oak_small_try1_raw.png](/Users/rexxa/work/arttrial2-data/plants/oak_small_try1_raw.png) | `a96bf42e2b5c30b35db8d9744ef4158e698406fbb677cfa4cea53e3d66c236a5` |
| pine_large | [pine_large_try1_raw.png](/Users/rexxa/work/arttrial2-data/plants/pine_large_try1_raw.png) | `0c1171b148c8f9d1235173bff8821de1167150b4f32687789b5c46c1b52d2e2c` |
| pine_small | [pine_small_try1_raw.png](/Users/rexxa/work/arttrial2-data/plants/pine_small_try1_raw.png) | `d4574cbfc5d71735022cf05448a1ced6ec1006752d03f3c72e2c3923c0e91194` |
| reeds | [reeds_try1_raw.png](/Users/rexxa/work/arttrial2-data/plants/reeds_try1_raw.png) | `18997778bade3735c5aedbd754e9cd0e5cabe6110a33b9f6f18f2343defe7430` |
