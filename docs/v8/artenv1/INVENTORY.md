# ART-ENV1 인벤토리

시뮬레이션 동결: `origin/v8/wip = 3a3b236c125c0c25e9b74261be88536473fc01e9`. 아트 기준: `v8/art-prod1 = 159ef0f9a1cfb7ee9e6005e85d5388d677e07fa8`. 스키마 v5 전체와 실제 enum·collector·현재 renderer를 교차 확인했다. 이 문서는 **제작 전 커버리지와 제작/보류 일정**이며 신규 자산의 수락 또는 오너 승인을 뜻하지 않는다.

총 **129 semantic drawable rows**: REUSE 57, NEW 70, PARTIAL 2. 기존 05 PNG는 중복 없이 **35개**, 채택 사람은 **34 looks × 10 frames = 340 PNG**다. 한 PNG를 여러 행이 참조할 수 있으므로 행 수를 생산 자산 수로 더하지 않는다. NEW에는 생성할 이미지, 코드 overlay, 명시적으로 보류할 그림이 모두 들어 있다.

각 행의 sim code는 동결 코드 기준이다. `tile footprint`는 물리 점유와 그림 크기를 구별한다. 파일 경로는 기준 아트에서 존재를 확인했다. 새 경로를 미리 존재하는 것으로 꾸미지 않고 NEW로 남긴다. 실제 최종 파일·가족별 수락은 생산 manifest와 ENV1_REPORT에 연결해야 한다.

## 우선 제작 순서

1. Part1.1: log/stone straight strip+2×2 corner, 각각16 mask, 양 벽 문 closed/open, 조용한 packed/dug floor, bedroll.
2. Part1.2: rack empty/hide, pottery bench, grinding stone, firing hearth/unlit, clay full/depleted, ford, hill edge, dug face. knapping/wood bench/hut scale 재검토.
3. Part2: wolf/deer 각 S/E×5 whole-figure poses, dead 포함.
4. Part1.3: ItemKind 17종 중 누락 11종, clay pot empty/filled와 meat tint; 모든 종류 32px 검토.
5. Part1.4/4: snow1장, ground tone·night light·marks·rooms·sites·LOD는 코드/기하 처리. 초상은 Part3 시간 허용 시.

## 코드와 스키마의 차이 및 그리기 계약

- 스키마 머리는 v5지만 일부 표의 설명은 M2/M3 코드만 기재했다. 실제 `NodeKind`는 ClayBank=2, `Designation`은 ClayPit=4/Hunt=5, BlueprintKind는9/10/11, JobKind는9–16, RecipeKind는3–10까지 존재한다. 아래는 actual enum을 우선한다.
- 일반 작업장5종은 knapping/wood/drying/grinding/pottery다. HearthFire를 포함하면 `is_workshop()`은6종이다. H11은 worldgen 화로이며 player-placeable이 아니다.
- 문은 sim에 open/closed나 log/stone subtype이 없고 passable wall flag만 있다. 요청된 open/closed+재질은 리허설의 선언 시각 상태다. 기존 renderer의 door_h/door_v는 방향이다.
- `get_animals`는 살아 있는 animal만 반환한다. 사체는 persistent animal이 아니다. 현 renderer는 Meat item을 carcass sprite로 그리며, wolf/deer dead frame과 spoiled meat는 선언된 표현으로 명시한다.
- 현재 stack sprite bucket은≤8/≤16/>16이나 ENV1은 요구대로1/2–8/9+를 사용한다. 변경은 리허설에만 있으며 game code는 바꾸지 않는다.
- item quality/material, bill mode/suspension, room override는 상태다. 종류별 silhouette를 늘리지 않고 glyph/text를 쓴다. filled pot은 현 ItemStack의 별도 상태가 아니며 선언된 art variant다.
- ASCII는 모든 레이어를 복원할 수 없다. 지형/기본 zone/log/site/건물/나무/사람만 증거로 사용하며 M3–M5와 빈 glyph의 세부는 `declared` overlay로 명시한다.
- 아래 REUSE는 baseline path 존재에 대한 말이며 이번 pass의 재검토 PASS를 의미하지 않는다. NEW→보류는 커버리지 결손으로 유지한다.

## terrain

| ID | 이름 / sim code | tile footprint | 그릴 상태 | 기존 05 자산 또는 NEW | 일정·제약 |
|---|---|---|---|---|---|
| T00 | Plain<br>`TerrainType::Plain=0` | 1×1 | 기본 / day / night / winter; ground cap 적용 | `art/style05/env/ground/meadow.png`<br>`art/style05/env/ground/dark_grass.png` | 재사용·Part4 tone cap<br>두 grass 톤은 시각 변형이며 별도 sim biome가 아니다.<br>근거: `rust/crates/sim-core/src/material/terrain.rs:10`; `scripts/world/colony/terrain_layer.gd:115` |
| T01 | Forest<br>`TerrainType::Forest=1` | 1×1 | 기본 / day / night / winter; ground cap 적용 | `art/style05/env/ground/forest_floor.png` | 재사용·Part4 tone cap<br>실제 나무는 get_trees 별도 레이어.<br>근거: `rust/crates/sim-core/src/material/terrain.rs:10`; `scripts/world/colony/terrain_layer.gd:115` |
| T02 | Mountain<br>`TerrainType::Mountain=2` | 1×1 | 기본 / day / night / winter; ground cap 적용 | `art/style05/env/ground/hill_rock.png` | 재사용·Part4 tone cap<br>암석 표면 재사용만 가능; cliff/top 전용 실루엣은 T10에서 따로 미충족 처리.<br>근거: `rust/crates/sim-core/src/material/terrain.rs:10`; `scripts/world/colony/terrain_layer.gd:115` |
| T03 | Hill<br>`TerrainType::Hill=3` | 1×1 | 기본 / day / night / winter; ground cap 적용 | `art/style05/env/ground/hill_rock.png` | 재사용·Part4 tone cap<br>미굴착 hill; 가장자리와 굴착면은 T11/T12.<br>근거: `rust/crates/sim-core/src/material/terrain.rs:10`; `scripts/world/colony/terrain_layer.gd:115` |
| T04 | River<br>`TerrainType::River=4` | 1×1 | 기본 / day / night / winter; ground cap 적용 | `art/style05/env/ground/river_bank.png` | 재사용·Part4 tone cap<br>기존 강둑만 검증된 자산; 물 내부와 모든 접속형은 T13에서 따로 미충족 처리.<br>근거: `rust/crates/sim-core/src/material/terrain.rs:10`; `scripts/world/colony/terrain_layer.gd:115` |
| T05 | Coast<br>`TerrainType::Coast=5` | 1×1 | 기본 / day / night / winter; ground cap 적용 | **NEW** | NEW → 보류: 전용 해안 자산 없음. 필수 M3–M5 리허설 외 biome라 ENV1에서는 보류; river_bank를 해안 수락으로 세지 않는다.<br>전용 해안 자산 없음. 필수 M3–M5 리허설 외 biome라 ENV1에서는 보류; river_bank를 해안 수락으로 세지 않는다.<br>근거: `rust/crates/sim-core/src/material/terrain.rs:10`; `scripts/world/colony/terrain_layer.gd:115` |
| T06 | Desert<br>`TerrainType::Desert=6` | 1×1 | 기본 / day / night / winter; ground cap 적용 | **NEW** | NEW → 보류: 전용 사막 자산 없음. 필수 M3–M5 리허설 외 biome라 보류; packed earth는 사막 수락이 아니다.<br>전용 사막 자산 없음. 필수 M3–M5 리허설 외 biome라 보류; packed earth는 사막 수락이 아니다.<br>근거: `rust/crates/sim-core/src/material/terrain.rs:10`; `scripts/world/colony/terrain_layer.gd:115` |
| T07 | Tundra<br>`TerrainType::Tundra=7` | 1×1 | 기본 / day / night / winter; ground cap 적용 | **NEW** | NEW → 보류: 전용 툰드라 자산 없음. 겨울 눈 tone은 계절 처리이며 biome 자산을 대체하지 않아 보류.<br>전용 툰드라 자산 없음. 겨울 눈 tone은 계절 처리이며 biome 자산을 대체하지 않아 보류.<br>근거: `rust/crates/sim-core/src/material/terrain.rs:10`; `scripts/world/colony/terrain_layer.gd:115` |
| T08 | Swamp<br>`TerrainType::Swamp=8` | 1×1 | 기본 / day / night / winter; ground cap 적용 | **NEW** | NEW → 보류: 전용 늪 바닥 없음. reeds 보유만으로 지형 커버리지 충족 아님; 필수 리허설 외 biome라 보류.<br>전용 늪 바닥 없음. reeds 보유만으로 지형 커버리지 충족 아님; 필수 리허설 외 biome라 보류.<br>근거: `rust/crates/sim-core/src/material/terrain.rs:10`; `scripts/world/colony/terrain_layer.gd:115` |
| T09 | Cave<br>`TerrainType::Cave=9` | 1×1 | 기본 / day / night / winter; ground cap 적용 | **NEW** | NEW → 보류: 전용 동굴 지형 없음. dug rock은 M3 바닥이며 Cave 전체가 아님; 필수 리허설 외 biome라 보류.<br>전용 동굴 지형 없음. dug rock은 M3 바닥이며 Cave 전체가 아님; 필수 리허설 외 biome라 보류.<br>근거: `rust/crates/sim-core/src/material/terrain.rs:10`; `scripts/world/colony/terrain_layer.gd:115` |
| T10 | Mountain top / cliff<br>`TerrainType::Mountain=2, N/S 경계` | 1×1 | top / cliff / interior | **NEW** | NEW → 보류: 전용 절벽은 필수 제작 목록 외; hill_rock 재사용을 절벽 커버리지로 세지 않음<br>현재 renderer는 mountain_top_0..2, mountain_cliff를 분리한다.<br>근거: `scripts/world/colony/terrain_layer.gd:115` |
| T11 | Hill edge<br>`TerrainType::Hill=3와 이웃 지형` | 1×1 | N/E/S/W 경계와 연속 edge | **NEW** | NEW → Part1.2 hill edge<br>64px 타일 접속 검증; 내부 hill_rock과 별개.<br>근거: `scripts/world/colony/terrain_layer.gd:123` |
| T12 | Dug face<br>`get_space.dug_xs/ys와 Hill 경계` | 1×1 | N/E/S/W 노출 굴착면 | **NEW** | NEW → Part1.2 dug face<br>floor_rock 위 경계; bridge에 독립 face kind는 없다.<br>근거: `rust/crates/sim-bridge/src/ffi/space.rs:230` |
| T13 | River interior and bank joins<br>`TerrainType::River=4, N/E/S/W land mask` | 1×1 | deep / shallow / bank mask1..15 / four inner corners | **NEW** | NEW → 보류: river_bank 1장으로 전 접속형 수락을 주장하지 않음; 필수 ford는 B17 별도 제작<br>현재 renderer의 water/shallow/bank는 별개 시각 파생; map topology의 빈틈으로 인계.<br>근거: `scripts/world/colony/terrain_layer.gd:204` |
| T14 | Worn camp ground<br>`hearth 거리와 terrain의 시각 hash` | 1×1 | packed interior / grass edge | `art/style05/env/ground/packed_earth.png` | 재사용·Part4 tone cap<br>미시뮬레이션 마모 장식; PackedFloor와 다른 사용.<br>근거: `scripts/world/colony/terrain_layer.gd:158` |

## nature

| ID | 이름 / sim code | tile footprint | 그릴 상태 | 기존 05 자산 또는 NEW | 일정·제약 |
|---|---|---|---|---|---|
| N00 | oak standing tree<br>`get_trees; hash 0..4` | root 1×1; crown may overlap | small / middle / large; chop mark overlay | `art/style05/env/plants/oak_small.png`<br>`art/style05/env/plants/oak_large.png` | 재사용·중간은 동일 전체 sprite 기하 scale<br>수종·크기는 시뮬레이션 kind가 아닌 현재 renderer hash; root와 canopy 크기를 혼동하지 않음.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:10`; `scripts/world/colony/tree_layer.gd:21`; `scripts/world/colony/tree_layer.gd:172` |
| N01 | birch standing tree<br>`get_trees; hash 5..7` | root 1×1; crown may overlap | small / middle / large; chop mark overlay | `art/style05/env/plants/birch_small.png`<br>`art/style05/env/plants/birch_large.png` | 재사용·중간은 동일 전체 sprite 기하 scale<br>수종·크기는 시뮬레이션 kind가 아닌 현재 renderer hash; root와 canopy 크기를 혼동하지 않음.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:10`; `scripts/world/colony/tree_layer.gd:21`; `scripts/world/colony/tree_layer.gd:172` |
| N02 | pine standing tree<br>`get_trees; hash 8..9` | root 1×1; crown may overlap | small / middle / large; chop mark overlay | `art/style05/env/plants/pine_small.png`<br>`art/style05/env/plants/pine_large.png` | 재사용·중간은 동일 전체 sprite 기하 scale<br>수종·크기는 시뮬레이션 kind가 아닌 현재 renderer hash; root와 canopy 크기를 혼동하지 않음.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:10`; `scripts/world/colony/tree_layer.gd:21`; `scripts/world/colony/tree_layer.gd:172` |
| N03 | Tree stump<br>`get_trees 이전 read−현재 read` | 1×1 | 벌목 후 1440 tick / 만료 | `art/style05/env/misc/stump.png` | 재사용<br>stump collector 없음; 누적 read가 없으면 첫 화면의 그루터기는 알 수 없다.<br>근거: `scripts/world/colony/tree_layer.gd:136`; `docs/v8/snapshot_schema.md:530` |
| N04 | Berry bush / food source<br>`get_resource_snapshot.kinds=0` | source 1×1; art96×96 | amount>0 full / amount=0 bare | `art/style05/env/plants/berry_full.png`<br>`art/style05/env/plants/berry_bare.png` | 재사용<br>ASCII에 없는 source.<br>근거: `scripts/world/colony/tree_layer.gd:223`; `docs/v8/snapshot_schema.md:450` |
| N05 | Water source<br>`get_resource_snapshot.kinds=1` | 1×1 | available amount; no standalone sprite | `art/style05/env/ground/river_bank.png` | 재사용·ground와 결합<br>독립 물통·샘을 생성하지 않는다. 현재 nature layer도 별도 drawing 없음.<br>근거: `docs/v8/snapshot_schema.md:450`; `scripts/world/colony/tree_layer.gd:231` |
| N06 | Sleep source<br>`get_resource_snapshot.kinds=2` | 1×1 sim source | available / occupied | `art/style05/env/buildings/lean_to.png`<br>`art/style05/env/buildings/hut.png` | 재사용·건물/침상 표시에 결합<br>source 자체는 독립 물체 아님.<br>근거: `docs/v8/snapshot_schema.md:450`; `scripts/world/colony/tree_layer.gd:231` |
| N07 | Deadwood source<br>`get_resource_snapshot.kinds=3` | 1×1 | amount>0 / depleted(undrawn) | **NEW** | NEW → 보류: 자연 잔가지 고유 silhouette 미제작; 통나무 더미로 같다고 주장하지 않음<br>ASCII에 없음; 현재 renderer는 deadwood sprite.<br>근거: `scripts/world/colony/tree_layer.gd:234` |
| N08 | Boulder node<br>`NodeKind::Boulder=0` | 1×1 | left>0; exhausted removed | `art/style05/env/misc/boulder_small.png`<br>`art/style05/env/misc/boulder_large.png` | 재사용<br>Stone ground item과 node를 구분.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:17` |
| N09 | Flint outcrop node<br>`NodeKind::FlintOutcrop=1` | 1×1 | left>0; exhausted removed | **NEW** | NEW → 보류: flint_pile은 채굴 지층이 아닌 item; mandatory fixtures 후속 후보<br>현재 05에 고유 outcrop 없음.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:17`; `scripts/world/colony/overlay_layer.gd:42` |
| N10 | Clay bank node<br>`NodeKind::ClayBank=2` | 1×1 | full / depleted | **NEW** | NEW → Part1.2 clay bank full+depleted<br>left=0에서 node 제거; depleted는 요청된 리허설 시각 상태이지 지속 sim node 아님.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:17`; `rust/crates/sim-core/src/colony/orders.rs:46` |
| N11 | Reeds decoration<br>`no distinct sim code` | root 1×1; art96×96 | static | `art/style05/env/plants/reeds.png` | 재사용·선언 장식만<br>별도 simulation plant로 세지 않는다.<br>근거: `art/style05/env/plants/reeds.json` |

## built

| ID | 이름 / sim code | tile footprint | 그릴 상태 | 기존 05 자산 또는 NEW | 일정·제약 |
|---|---|---|---|---|---|
| B00 | Hearth<br>`BuildingKind::Hearth=0; BlueprintKind::HearthFire=11` | 1×1 | lit frame0/1 / unlit / firing pots | `art/style05/env/misc/hearth_0.png`<br>`art/style05/env/misc/hearth_1.png`<br>**NEW 상태/변형 있음** | NEW → Part1.2 firing pots + unlit; 기존 lit animation 재사용<br>두 기존 PNG는 모두 불이 켜진 frame. HearthFire는 workshop이지만 player-placeable 아님.<br>근거: `rust/crates/sim-core/src/components/building_kind.rs:6`; `rust/crates/sim-core/src/colony/orders.rs:112`; `scripts/world/wood_visuals.gd:35` |
| B01 | Shelter / lean-to<br>`BuildingKind::Shelter=1; SHELTER_BLUEPRINT_ID=1` | sim1×1; existing visual2×2 | site / finished / occupied | `art/style05/env/buildings/lean_to.png` | 재사용·Part1.2 site overlay / scale review<br>128² art와 1×1 sim footprint 차이를 명시; art를 물리 2×2로 해석하지 않음.<br>근거: `rust/crates/sim-core/src/components/building_kind.rs:39` |
| B02 | Legacy stockpile marker<br>`BuildingKind::Stockpile=2` | point1×1 | stored amount / empty | `art/style05/env/buildings/storage_pit.png` | 재사용 후보·현 colony에서는 숨김<br>현재 building_layer와 ASCII는 kind2를 그리지 않는다. storage_pit은 legacy marker 후보이며 stockpile zone 대체 아님.<br>근거: `rust/crates/sim-core/src/components/building_kind.rs:6`; `scripts/world/colony/building_layer.gd:22` |
| B03 | Hut<br>`BuildingKind::Hut=3; BlueprintKind::Hut=0` | 3×3; centre-keyed finished | site / finished / occupied | `art/style05/env/buildings/hut.png` | 재사용·Part1.1 벽 대비 scale / Part1.2 site overlay<br>site는 top-left, finished는 centre;192² 유지.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:112`; `scripts/world/colony/building_layer.gd:243` |
| B04 | knapping_spot<br>`BlueprintKind::KnappingSpot=1` | 1×1 | empty/inputs/active fine-or-normal blade; south work tile | `art/style05/env/buildings/knapping_spot.png` | 재사용·Part1.2 재검토<br>모든 placeable kind는 완성 sprite의 pale outline+25% wash로 site 파생; 별도 생성 없음.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:112`; `rust/crates/sim-core/src/colony/orders.rs:242`; `scripts/world/colony/building_layer.gd:154` |
| B05 | wood_bench<br>`BlueprintKind::WoodBench=2` | 2×1 | idle/axe/firewood/spear inputs+active; south work tile | `art/style05/env/buildings/woodworking_bench.png` | 재사용·Part1.2 재검토<br>모든 placeable kind는 완성 sprite의 pale outline+25% wash로 site 파생; 별도 생성 없음.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:112`; `rust/crates/sim-core/src/colony/orders.rs:242`; `scripts/world/colony/building_layer.gd:154` |
| B06 | drying_rack<br>`BlueprintKind::DryingRack=3` | 2×1 | empty/with hide; scrape/hide-wrap/fur-cloak bills | **NEW** | NEW → Part1.2 empty + hide<br>모든 placeable kind는 완성 sprite의 pale outline+25% wash로 site 파생; 별도 생성 없음.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:112`; `rust/crates/sim-core/src/colony/orders.rs:242`; `scripts/world/colony/building_layer.gd:154` |
| B07 | log_wall<br>`BlueprintKind::LogWall=4` | 1×1 | site/finished; masks 0..15 N1/E2/S4/W8 | **NEW** | NEW → Part1.1 straight strip + 2×2 corner → 16 cases<br>모든 placeable kind는 완성 sprite의 pale outline+25% wash로 site 파생; 별도 생성 없음.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:112`; `rust/crates/sim-core/src/colony/orders.rs:242`; `scripts/world/colony/building_layer.gd:154` |
| B08 | stone_wall<br>`BlueprintKind::StoneWall=5` | 1×1 | site/finished; masks 0..15 N1/E2/S4/W8 | **NEW** | NEW → Part1.1 straight strip + 2×2 corner → 16 cases<br>모든 placeable kind는 완성 sprite의 pale outline+25% wash로 site 파생; 별도 생성 없음.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:112`; `rust/crates/sim-core/src/colony/orders.rs:242`; `scripts/world/colony/building_layer.gd:154` |
| B09 | door<br>`BlueprintKind::Door=6` | 1×1 | horizontal/vertical; open/closed and material contexts | **NEW** | NEW → Part1.1 log+stone surrounds × closed/open<br>모든 placeable kind는 완성 sprite의 pale outline+25% wash로 site 파생; 별도 생성 없음. sim door는 passable wall flag만 제공; material/open-state 없음. 두 재질·open state는 선언된 rehearsal art 변형.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:112`; `rust/crates/sim-core/src/colony/orders.rs:242`; `scripts/world/colony/building_layer.gd:154` |
| B10 | packed_floor<br>`BlueprintKind::PackedFloor=7` | 1×1 | site/finished; seamless repeat | `art/style05/env/ground/packed_earth.png`<br>**NEW 상태/변형 있음** | NEW → Part1.1 quieter packed floor tile; source material reusable<br>모든 placeable kind는 완성 sprite의 pale outline+25% wash로 site 파생; 별도 생성 없음.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:112`; `rust/crates/sim-core/src/colony/orders.rs:242`; `scripts/world/colony/building_layer.gd:154` |
| B11 | bedroll<br>`BlueprintKind::Bedroll=8` | 1×1 | site/finished/assigned/occupied via person | **NEW** | NEW → Part1.1 bedroll<br>모든 placeable kind는 완성 sprite의 pale outline+25% wash로 site 파생; 별도 생성 없음.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:112`; `rust/crates/sim-core/src/colony/orders.rs:242`; `scripts/world/colony/building_layer.gd:154` |
| B12 | grinding_stone<br>`BlueprintKind::GrindingStone=9` | 1×1 | idle/stone+axe inputs/active; south work tile | **NEW** | NEW → Part1.2 grinding stone<br>모든 placeable kind는 완성 sprite의 pale outline+25% wash로 site 파생; 별도 생성 없음.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:112`; `rust/crates/sim-core/src/colony/orders.rs:242`; `scripts/world/colony/building_layer.gd:154` |
| B13 | pottery_bench<br>`BlueprintKind::PotteryBench=10` | 1×1 | idle/clay/raw pot/active; south work tile | **NEW** | NEW → Part1.2 pottery bench<br>모든 placeable kind는 완성 sprite의 pale outline+25% wash로 site 파생; 별도 생성 없음.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:112`; `rust/crates/sim-core/src/colony/orders.rs:242`; `scripts/world/colony/building_layer.gd:154` |
| B14 | Dug-rock floor<br>`get_space.floor_kinds=1 + dug_xs/ys` | 1×1 | dug / packed overwrite distinction / repeat | **NEW** | NEW → Part1.1 dug-rock floor<br>collect_space emits a dug tile in both dug and floors; draw one floor, not duplicate art.<br>근거: `rust/crates/sim-bridge/src/ffi/space.rs:256` |
| B15 | Storehouse roof coverage<br>`get_rooms.roofed + RoomRole=2; item_covered=1` | room tile set | roofed/unroofed; covered/uncovered stacks | **NEW** | NEW → Part1.2 code overlay; opaque roof sprite deferred to M-ART2<br>재고를 가리지 않는 pale region boundary/label. 실제 roof tile collector 없음.<br>근거: `docs/v8/snapshot_schema.md:578`; `scripts/world/colony/room_layer.gd:166` |
| B16 | Blueprint / construction state<br>`ConstructionSite; site_kinds0..10` | kind별 plan footprint | planned/delivering/work progress/finished | **NEW** | NEW → Part1.2 code overlay (generation 0)<br>각 B01/B03/B04..B13의 finished sprite25% wash+pale ink outline; materials/work progress separate. HearthFire11은 site 없음.<br>근거: `docs/v8/snapshot_schema.md:440`; `scripts/world/colony/overlay_layer.gd:191` |
| B17 | Ford<br>`get_space.ford_xs/ys; TileGrid::is_ford` | 1×1 connected river tiles | shallow crossing / along-river repeat | **NEW** | NEW → Part1.2 ford tiles<br>물 위, 다른 floor-level layer 아래.<br>근거: `rust/crates/sim-bridge/src/ffi/space.rs:253`; `scripts/world/colony/building_layer.gd:129` |

## item

| ID | 이름 / sim code | tile footprint | 그릴 상태 | 기존 05 자산 또는 NEW | 일정·제약 |
|---|---|---|---|---|---|
| I00 | Log<br>`ItemKind::Log=0` | ground1×1; tool may overhang | ground / carried / shelf; count1 / 2–8 / 9+; quality via text | `art/style05/env/items/log_pile_small.png`<br>`art/style05/env/items/log_pile_medium.png`<br>`art/style05/env/items/log_pile_large.png` | 재사용·Part1.3 count/32px review<br>수량 bucket은 전체 sprite의 기하 복제·배치로 파생 가능; 새 texture variant마다 생성 불필요. 32px silhouette review 필요. ENV1 bucket1/2–8/9+는 현 renderer≤8/≤16/>16과 다름; 의도적 rehearsal 분류.<br>근거: `rust/crates/sim-core/src/colony/items.rs:9`; `scripts/ui/economy_text.gd:34`; `scripts/world/colony/overlay_layer.gd:145` |
| I01 | Firewood<br>`ItemKind::Firewood=1` | ground1×1; tool may overhang | ground / carried / shelf; count1 / 2–8 / 9+; quality via text | **NEW** | NEW → Part1.3 Firewood<br>수량 bucket은 전체 sprite의 기하 복제·배치로 파생 가능; 새 texture variant마다 생성 불필요. 32px silhouette review 필요.<br>근거: `rust/crates/sim-core/src/colony/items.rs:9`; `scripts/ui/economy_text.gd:34`; `scripts/world/colony/overlay_layer.gd:145` |
| I02 | Stone<br>`ItemKind::Stone=2` | ground1×1; tool may overhang | ground / carried / shelf; count1 / 2–8 / 9+; quality via text | `art/style05/env/items/stone_pile.png` | 재사용·Part1.3 count/32px review<br>수량 bucket은 전체 sprite의 기하 복제·배치로 파생 가능; 새 texture variant마다 생성 불필요. 32px silhouette review 필요.<br>근거: `rust/crates/sim-core/src/colony/items.rs:9`; `scripts/ui/economy_text.gd:34`; `scripts/world/colony/overlay_layer.gd:145` |
| I03 | Flint<br>`ItemKind::Flint=3` | ground1×1; tool may overhang | ground / carried / shelf; count1 / 2–8 / 9+; quality via text | `art/style05/env/items/flint_pile.png` | 재사용·Part1.3 count/32px review<br>수량 bucket은 전체 sprite의 기하 복제·배치로 파생 가능; 새 texture variant마다 생성 불필요. 32px silhouette review 필요.<br>근거: `rust/crates/sim-core/src/colony/items.rs:9`; `scripts/ui/economy_text.gd:34`; `scripts/world/colony/overlay_layer.gd:145` |
| I04 | FlintBlade<br>`ItemKind::FlintBlade=4` | ground1×1; tool may overhang | ground / carried / shelf; count1 / 2–8 / 9+; quality via text; held tool/weapon | `art/style05/env/items/flint_blade.png` | 재사용·Part1.3 count/32px review<br>수량 bucket은 전체 sprite의 기하 복제·배치로 파생 가능; 새 texture variant마다 생성 불필요. 32px silhouette review 필요.<br>근거: `rust/crates/sim-core/src/colony/items.rs:9`; `scripts/ui/economy_text.gd:34`; `scripts/world/colony/overlay_layer.gd:145` |
| I05 | StoneAxe<br>`ItemKind::StoneAxe=5` | ground1×1; tool may overhang | ground / carried / shelf; count1 / 2–8 / 9+; quality via text; held tool/weapon | `art/style05/env/items/stone_axe.png` | 재사용·Part1.3 count/32px review<br>수량 bucket은 전체 sprite의 기하 복제·배치로 파생 가능; 새 texture variant마다 생성 불필요. 32px silhouette review 필요.<br>근거: `rust/crates/sim-core/src/colony/items.rs:9`; `scripts/ui/economy_text.gd:34`; `scripts/world/colony/overlay_layer.gd:145` |
| I06 | Hide<br>`ItemKind::Hide=6` | ground1×1; tool may overhang | ground / carried / shelf; count1 / 2–8 / 9+; quality via text | `art/style05/env/items/hide.png` | 재사용·Part1.3 count/32px review<br>수량 bucket은 전체 sprite의 기하 복제·배치로 파생 가능; 새 texture variant마다 생성 불필요. 32px silhouette review 필요.<br>근거: `rust/crates/sim-core/src/colony/items.rs:9`; `scripts/ui/economy_text.gd:34`; `scripts/world/colony/overlay_layer.gd:145` |
| I07 | Clay<br>`ItemKind::Clay=7` | ground1×1; tool may overhang | ground / carried / shelf; count1 / 2–8 / 9+; quality via text | **NEW** | NEW → Part1.3 Clay<br>수량 bucket은 전체 sprite의 기하 복제·배치로 파생 가능; 새 texture variant마다 생성 불필요. 32px silhouette review 필요.<br>근거: `rust/crates/sim-core/src/colony/items.rs:9`; `scripts/ui/economy_text.gd:34`; `scripts/world/colony/overlay_layer.gd:145` |
| I08 | RawPot<br>`ItemKind::RawPot=8` | ground1×1; tool may overhang | ground / carried / shelf; count1 / 2–8 / 9+; quality via text | **NEW** | NEW → Part1.3 RawPot<br>수량 bucket은 전체 sprite의 기하 복제·배치로 파생 가능; 새 texture variant마다 생성 불필요. 32px silhouette review 필요.<br>근거: `rust/crates/sim-core/src/colony/items.rs:9`; `scripts/ui/economy_text.gd:34`; `scripts/world/colony/overlay_layer.gd:145` |
| I09 | ClayPot<br>`ItemKind::ClayPot=9` | ground1×1; tool may overhang | ground / carried / shelf; count1 / 2–8 / 9+; quality via text; empty / filled | **NEW** | NEW → Part1.3 ClayPot<br>수량 bucket은 전체 sprite의 기하 복제·배치로 파생 가능; 새 texture variant마다 생성 불필요. 32px silhouette review 필요. ItemStack에 filled 상태 없음; filled는 art 요청의 선언 상태.<br>근거: `rust/crates/sim-core/src/colony/items.rs:9`; `scripts/ui/economy_text.gd:34`; `scripts/world/colony/overlay_layer.gd:145` |
| I10 | GroundAxe<br>`ItemKind::GroundAxe=10` | ground1×1; tool may overhang | ground / carried / shelf; count1 / 2–8 / 9+; quality via text; held tool/weapon | **NEW** | NEW → Part1.3 GroundAxe<br>수량 bucket은 전체 sprite의 기하 복제·배치로 파생 가능; 새 texture variant마다 생성 불필요. 32px silhouette review 필요.<br>근거: `rust/crates/sim-core/src/colony/items.rs:9`; `scripts/ui/economy_text.gd:34`; `scripts/world/colony/overlay_layer.gd:145` |
| I11 | Meat<br>`ItemKind::Meat=11` | ground1×1; tool may overhang | ground / carried / shelf; count1 / 2–8 / 9+; quality via text; fresh / spoiled global tint | **NEW** | NEW → Part1.3 Meat<br>수량 bucket은 전체 sprite의 기하 복제·배치로 파생 가능; 새 texture variant마다 생성 불필요. 32px silhouette review 필요. 현재 ground sprite는 carcass; sim은 spoil 시 meat lot/item을 없앤다. 지속 spoiled sprite는 선언한 보조 상태.<br>근거: `rust/crates/sim-core/src/colony/items.rs:9`; `scripts/ui/economy_text.gd:34`; `scripts/world/colony/overlay_layer.gd:145` |
| I12 | CuredHide<br>`ItemKind::CuredHide=12` | ground1×1; tool may overhang | ground / carried / shelf; count1 / 2–8 / 9+; quality via text | **NEW** | NEW → Part1.3 CuredHide<br>수량 bucket은 전체 sprite의 기하 복제·배치로 파생 가능; 새 texture variant마다 생성 불필요. 32px silhouette review 필요.<br>근거: `rust/crates/sim-core/src/colony/items.rs:9`; `scripts/ui/economy_text.gd:34`; `scripts/world/colony/overlay_layer.gd:145` |
| I13 | HideWrap<br>`ItemKind::HideWrap=13` | ground1×1; tool may overhang | ground / carried / shelf; count1 / 2–8 / 9+; quality via text; worn uses whole person look | **NEW** | NEW → Part1.3 HideWrap<br>수량 bucket은 전체 sprite의 기하 복제·배치로 파생 가능; 새 texture variant마다 생성 불필요. 32px silhouette review 필요.<br>근거: `rust/crates/sim-core/src/colony/items.rs:9`; `scripts/ui/economy_text.gd:34`; `scripts/world/colony/overlay_layer.gd:145` |
| I14 | FurCloak<br>`ItemKind::FurCloak=14` | ground1×1; tool may overhang | ground / carried / shelf; count1 / 2–8 / 9+; quality via text; worn uses whole person look | **NEW** | NEW → Part1.3 FurCloak<br>수량 bucket은 전체 sprite의 기하 복제·배치로 파생 가능; 새 texture variant마다 생성 불필요. 32px silhouette review 필요.<br>근거: `rust/crates/sim-core/src/colony/items.rs:9`; `scripts/ui/economy_text.gd:34`; `scripts/world/colony/overlay_layer.gd:145` |
| I15 | Spear<br>`ItemKind::Spear=15` | ground1×1; tool may overhang | ground / carried / shelf; count1 / 2–8 / 9+; quality via text; held tool/weapon | **NEW** | NEW → Part1.3 Spear<br>수량 bucket은 전체 sprite의 기하 복제·배치로 파생 가능; 새 texture variant마다 생성 불필요. 32px silhouette review 필요.<br>근거: `rust/crates/sim-core/src/colony/items.rs:9`; `scripts/ui/economy_text.gd:34`; `scripts/world/colony/overlay_layer.gd:145` |
| I16 | WolfHide<br>`ItemKind::WolfHide=16` | ground1×1; tool may overhang | ground / carried / shelf; count1 / 2–8 / 9+; quality via text | **NEW** | NEW → Part1.3 WolfHide<br>수량 bucket은 전체 sprite의 기하 복제·배치로 파생 가능; 새 texture variant마다 생성 불필요. 32px silhouette review 필요.<br>근거: `rust/crates/sim-core/src/colony/items.rs:9`; `scripts/ui/economy_text.gd:34`; `scripts/world/colony/overlay_layer.gd:145` |
| I17 | Gathered food basket<br>`Inventory.food / porter.kind=0; no ItemKind` | 1×1 | ground food cue / carried food | `art/style05/env/items/berry_basket.png` | 재사용<br>17 ItemKind의 추가 enum으로 세지 않음. legacy food resource/porter용.<br>근거: `docs/v8/snapshot_schema.md:259`; `art/style05/env/items/berry_basket.json` |

## designation

| ID | 이름 / sim code | tile footprint | 그릴 상태 | 기존 05 자산 또는 NEW | 일정·제약 |
|---|---|---|---|---|---|
| M00 | Chop<br>`Designation::Chop=0` | 1×1; Hunt follows AnimalId | designated / active / cancelled | **NEW** | NEW → Part1.2 code overlay (generation 0)<br>current marks 0; 고유 overlay glyph로 구별.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:59`; `scripts/world/colony/overlay_layer.gd:46` |
| M01 | Quarry<br>`Designation::Quarry=1` | 1×1; Hunt follows AnimalId | designated / active / cancelled | **NEW** | NEW → Part1.2 code overlay (generation 0)<br>current marks 1; 고유 overlay glyph로 구별.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:59`; `scripts/world/colony/overlay_layer.gd:46` |
| M02 | Dig<br>`Designation::Dig=2` | 1×1; Hunt follows AnimalId | designated / active / cancelled | **NEW** | NEW → Part1.2 code overlay (generation 0)<br>current marks 2; 고유 overlay glyph로 구별.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:59`; `scripts/world/colony/overlay_layer.gd:46` |
| M03 | Deconstruct<br>`Designation::Deconstruct=3` | 1×1; Hunt follows AnimalId | designated / active / cancelled | **NEW** | NEW → Part1.2 code overlay (generation 0)<br>current marks 3; 고유 overlay glyph로 구별.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:59`; `scripts/world/colony/overlay_layer.gd:46` |
| M04 | ClayPit<br>`Designation::ClayPit=4` | 1×1; Hunt follows AnimalId | designated / active / cancelled | **NEW** | NEW → Part1.2 code overlay (generation 0)<br>current marks 4; 고유 overlay glyph로 구별.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:59`; `scripts/world/colony/overlay_layer.gd:46` |
| M05 | Hunt<br>`Designation::Hunt=5` | 1×1; Hunt follows AnimalId | designated / active / cancelled | **NEW** | NEW → Part1.2 code overlay (generation 0)<br>pale ink tool glyph; Hunt는 tile designation이 아니라 get_animals.marked에 연결.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:59`; `scripts/world/colony/overlay_layer.gd:46` |
| M06 | Stockpile zone<br>`ZoneKind::Stockpile; zone_ids/zone_xs/zone_ys` | arbitrary tile set | empty/filled; open edges/corner stakes | **NEW** | NEW → Part1.2 code overlay (generation 0)<br>타일 wash+경계+stakes; storage_pit 한 장으로 zone을 대체하지 않음.<br>근거: `rust/crates/sim-core/src/colony/orders.rs:94`; `scripts/world/colony/overlay_layer.gd:163` |
| M07 | Hunt zone<br>`Wildlife.zones; get_animals.zones` | inclusive rectangle | shown/hidden; contains huntable deer | **NEW** | NEW → Part1.2 code overlay (generation 0)<br>황토색 border/wash, get_animals 기반.<br>근거: `rust/crates/sim-core/src/colony/wildlife.rs:131`; `scripts/world/colony/animal_layer.gd:160` |
| M08 | Placement preview<br>`get_place_preview.ok/reasons` | requested footprint | valid/invalid; line/hollow/fill | **NEW** | NEW → Part1.2 code overlay (generation 0)<br>pale finished-sprite ghost + valid/invalid ring; game code 설치는 M-ART2.<br>근거: `docs/v8/snapshot_schema.md:642` |

## room

| ID | 이름 / sim code | tile footprint | 그릴 상태 | 기존 05 자산 또는 NEW | 일정·제약 |
|---|---|---|---|---|---|
| R00 | Unassigned<br>`RoomRole::Unassigned=0` | irregular room tile set | auto/overridden; roofed/dug; border+label | **NEW** | NEW → Part1.2 code overlay (generation 0)<br>역할은 새 건물 모델 아님; 역할마다 구별된 얇은 경계색과 label.<br>근거: `rust/crates/sim-core/src/rooms.rs:33`; `scripts/world/colony/room_layer.gd:20` |
| R01 | Bedroom<br>`RoomRole::Bedroom=1` | irregular room tile set | auto/overridden; roofed/dug; border+label | **NEW** | NEW → Part1.2 code overlay (generation 0)<br>역할은 새 건물 모델 아님; 역할마다 구별된 얇은 경계색과 label.<br>근거: `rust/crates/sim-core/src/rooms.rs:33`; `scripts/world/colony/room_layer.gd:20` |
| R02 | Storehouse<br>`RoomRole::Storehouse=2` | irregular room tile set | auto/overridden; roofed/dug; border+label | **NEW** | NEW → Part1.2 code overlay (generation 0)<br>역할은 새 건물 모델 아님; 역할마다 구별된 얇은 경계색과 label.<br>근거: `rust/crates/sim-core/src/rooms.rs:33`; `scripts/world/colony/room_layer.gd:20` |
| R03 | Workshop<br>`RoomRole::Workshop=3` | irregular room tile set | auto/overridden; roofed/dug; border+label | **NEW** | NEW → Part1.2 code overlay (generation 0)<br>역할은 새 건물 모델 아님; 역할마다 구별된 얇은 경계색과 label.<br>근거: `rust/crates/sim-core/src/rooms.rs:33`; `scripts/world/colony/room_layer.gd:20` |
| R04 | Hall<br>`RoomRole::Hall=4` | irregular room tile set | auto/overridden; roofed/dug; border+label | **NEW** | NEW → Part1.2 code overlay (generation 0)<br>역할은 새 건물 모델 아님; 역할마다 구별된 얇은 경계색과 label.<br>근거: `rust/crates/sim-core/src/rooms.rs:33`; `scripts/world/colony/room_layer.gd:20` |

## animal

| ID | 이름 / sim code | tile footprint | 그릴 상태 | 기존 05 자산 또는 NEW | 일정·제약 |
|---|---|---|---|---|---|
| A00 | Wolf<br>`AnimalKind::Wolf=0` | occupied1×1; art96×80, dead96×64 | S/E; W flip; stand / walk / run / attack / dead | **NEW** | NEW → Part2 whole-figure sheet ×10 frames<br>어깨높이 28px; foot y72. carcass는 dead frame을 사용하되 persistent animal entity로 주장하지 않음.<br>근거: `rust/crates/sim-core/src/colony/wildlife.rs:13`; `rust/crates/sim-core/src/colony/wildlife.rs:40` |
| A01 | Deer<br>`AnimalKind::Deer=1` | occupied1×1; art96×80, dead96×64 | S/E; W flip; stand / walk / run(flee) / graze / dead | **NEW** | NEW → Part2 whole-figure sheet ×10 frames<br>어깨높이 40px; foot y72. carcass는 dead frame을 사용하되 persistent animal entity로 주장하지 않음.<br>근거: `rust/crates/sim-core/src/colony/wildlife.rs:13`; `rust/crates/sim-core/src/colony/wildlife.rs:40` |

## person

| ID | 이름 / sim code | tile footprint | 그릴 상태 | 기존 05 자산 또는 NEW | 일정·제약 |
|---|---|---|---|---|---|
| P00 | Adult<br>`get_agent_snapshot.life_stages=0` | anchor1×1; upright80×112 / sleep112×80 | idle/walk_a/work_a/carry/sleep ×S/E; W flip; N=S | `art/character/route_i/adult_M_loincloth_short/S/idle.png` | 재사용·Part4 adopted look selection<br>adult56px/child40px; 34 adopted look exact paths는 아래 roster.<br>근거: `docs/v8/snapshot_schema.md:140`; `docs/v8/artprod3/HANDOVER.md` |
| P01 | Child<br>`get_agent_snapshot.life_stages=1` | anchor1×1; upright80×112 / sleep112×80 | idle/walk_a/work_a/carry/sleep ×S/E; W flip; N=S | `art/character/route_i/child_M_loincloth_short/S/idle.png` | 재사용·Part4 adopted look selection<br>adult56px/child40px; 34 adopted look exact paths는 아래 roster.<br>근거: `docs/v8/snapshot_schema.md:140`; `docs/v8/artprod3/HANDOVER.md` |
| P02 | Elder<br>`get_agent_snapshot.life_stages=2` | anchor1×1; upright80×112 / sleep112×80 | idle/walk_a/work_a/carry/sleep ×S/E; W flip; N=S | `art/character/route_i/adult_M_loincloth_short/S/idle.png` | 재사용·Part4 adopted look selection<br>전용 elder art는 보류; adult fallback을 명시.<br>근거: `docs/v8/snapshot_schema.md:140`; `docs/v8/artprod3/HANDOVER.md` |
| P03 | Infant<br>`get_agent_snapshot.infants=1` | anchor1×1 | carried/sleep/idle distinction absent | **NEW** | NEW → 보류: 전용 infant 그림은 Part2 animal 범위 밖; child proxy는 명시<br>infant를 새 accepted look로 세지 않음.<br>근거: `docs/v8/snapshot_schema.md:157` |

## apparel

| ID | 이름 / sim code | tile footprint | 그릴 상태 | 기존 05 자산 또는 NEW | 일정·제약 |
|---|---|---|---|---|---|
| P04 | Loincloth<br>`no Apparel=0` | person full-figure | worn / S/E activity; all adopted looks | `art/character/route_i/adult_M_loincloth_short/S/idle.png` | 재사용·Part4 whole person only<br>의복 분해 합성 금지; 땅의 item은 I13/I14 별도.<br>근거: `rust/crates/sim-core/src/colony/people.rs:13`; `docs/v8/artprod3/HANDOVER.md` |
| P05 | HideWrap<br>`Top::HideWrap=1` | person full-figure | worn / S/E activity; all adopted looks | `art/character/route_i/adult_M_hide_wrap_short/S/idle.png` | 재사용·Part4 whole person only<br>의복 분해 합성 금지; 땅의 item은 I13/I14 별도.<br>근거: `rust/crates/sim-core/src/colony/people.rs:13`; `docs/v8/artprod3/HANDOVER.md` |
| P06 | FurCloak<br>`Top::FurCloak=2` | person full-figure | worn / S/E activity; all adopted looks | `art/character/route_i/adult_M_fur_cloak_short/S/idle.png` | 재사용·Part4 whole person only<br>의복 분해 합성 금지; 땅의 item은 I13/I14 별도.<br>근거: `rust/crates/sim-core/src/colony/people.rs:13`; `docs/v8/artprod3/HANDOVER.md` |

## job

| ID | 이름 / sim code | tile footprint | 그릴 상태 | 기존 05 자산 또는 NEW | 일정·제약 |
|---|---|---|---|---|---|
| J01 | Chop<br>`JobKind::Chop=1` | person anchor1×1 | work_a / movement / target | `art/character/route_i/adult_M_loincloth_short/S/work_a.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>이동중 idle/walk_a; 수면 sleep가 우선. chop은 도끼 work_a와 대응.<br>근거: `rust/crates/sim-core/src/colony/job.rs:88`; `rust/crates/sim-core/src/colony/job.rs:197` |
| J02 | Haul<br>`JobKind::Haul=2` | person anchor1×1 | carry / movement / target | `art/character/route_i/adult_M_loincloth_short/S/carry.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>이동중 idle/walk_a; 수면 sleep가 우선. carry frame은 통나무 pose이며 비통나무 내용물 일치 보장은 없음; 종류 glyph/label로 명시, dedicated carrying pose는 M-ART2 보류.<br>근거: `rust/crates/sim-core/src/colony/job.rs:88`; `rust/crates/sim-core/src/colony/job.rs:197` |
| J03 | Deliver<br>`JobKind::Deliver=3` | person anchor1×1 | carry / movement / target | `art/character/route_i/adult_M_loincloth_short/S/carry.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>이동중 idle/walk_a; 수면 sleep가 우선. carry frame은 통나무 pose이며 비통나무 내용물 일치 보장은 없음; 종류 glyph/label로 명시, dedicated carrying pose는 M-ART2 보류.<br>근거: `rust/crates/sim-core/src/colony/job.rs:88`; `rust/crates/sim-core/src/colony/job.rs:197` |
| J04 | Build<br>`JobKind::Build=4` | person anchor1×1 | work_a / movement / target | `art/character/route_i/adult_M_loincloth_short/S/work_a.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>이동중 idle/walk_a; 수면 sleep가 우선. 기존 work/idle 대체는 동작 의미의 개별 수락 아님; 비도끼 전용 pose 제작은 ENV1 범위 밖, activity glyph/label 필요.<br>근거: `rust/crates/sim-core/src/colony/job.rs:88`; `rust/crates/sim-core/src/colony/job.rs:197` |
| J05 | Quarry<br>`JobKind::Quarry=5` | person anchor1×1 | work_a / movement / target | `art/character/route_i/adult_M_loincloth_short/S/work_a.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>이동중 idle/walk_a; 수면 sleep가 우선. 기존 work/idle 대체는 동작 의미의 개별 수락 아님; 비도끼 전용 pose 제작은 ENV1 범위 밖, activity glyph/label 필요.<br>근거: `rust/crates/sim-core/src/colony/job.rs:88`; `rust/crates/sim-core/src/colony/job.rs:197` |
| J06 | Supply<br>`JobKind::Supply=6` | person anchor1×1 | carry / movement / target | `art/character/route_i/adult_M_loincloth_short/S/carry.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>이동중 idle/walk_a; 수면 sleep가 우선. carry frame은 통나무 pose이며 비통나무 내용물 일치 보장은 없음; 종류 glyph/label로 명시, dedicated carrying pose는 M-ART2 보류.<br>근거: `rust/crates/sim-core/src/colony/job.rs:88`; `rust/crates/sim-core/src/colony/job.rs:197` |
| J07 | Craft<br>`JobKind::Craft=7` | person anchor1×1 | work_a / movement / target | `art/character/route_i/adult_M_loincloth_short/S/work_a.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>이동중 idle/walk_a; 수면 sleep가 우선. 기존 work/idle 대체는 동작 의미의 개별 수락 아님; 비도끼 전용 pose 제작은 ENV1 범위 밖, activity glyph/label 필요.<br>근거: `rust/crates/sim-core/src/colony/job.rs:88`; `rust/crates/sim-core/src/colony/job.rs:197` |
| J08 | Gather<br>`JobKind::Gather=8` | person anchor1×1 | work_a / movement / target | `art/character/route_i/adult_M_loincloth_short/S/work_a.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>이동중 idle/walk_a; 수면 sleep가 우선. 기존 work/idle 대체는 동작 의미의 개별 수락 아님; 비도끼 전용 pose 제작은 ENV1 범위 밖, activity glyph/label 필요.<br>근거: `rust/crates/sim-core/src/colony/job.rs:88`; `rust/crates/sim-core/src/colony/job.rs:197` |
| J09 | Dig<br>`JobKind::Dig=9` | person anchor1×1 | work_a / movement / target | `art/character/route_i/adult_M_loincloth_short/S/work_a.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>이동중 idle/walk_a; 수면 sleep가 우선. 기존 work/idle 대체는 동작 의미의 개별 수락 아님; 비도끼 전용 pose 제작은 ENV1 범위 밖, activity glyph/label 필요.<br>근거: `rust/crates/sim-core/src/colony/job.rs:88`; `rust/crates/sim-core/src/colony/job.rs:197` |
| J10 | Deconstruct<br>`JobKind::Deconstruct=10` | person anchor1×1 | work_a / movement / target | `art/character/route_i/adult_M_loincloth_short/S/work_a.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>이동중 idle/walk_a; 수면 sleep가 우선. 기존 work/idle 대체는 동작 의미의 개별 수락 아님; 비도끼 전용 pose 제작은 ENV1 범위 밖, activity glyph/label 필요.<br>근거: `rust/crates/sim-core/src/colony/job.rs:88`; `rust/crates/sim-core/src/colony/job.rs:197` |
| J11 | Clear<br>`JobKind::Clear=11` | person anchor1×1 | carry / movement / target | `art/character/route_i/adult_M_loincloth_short/S/carry.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>이동중 idle/walk_a; 수면 sleep가 우선. carry frame은 통나무 pose이며 비통나무 내용물 일치 보장은 없음; 종류 glyph/label로 명시, dedicated carrying pose는 M-ART2 보류.<br>근거: `rust/crates/sim-core/src/colony/job.rs:88`; `rust/crates/sim-core/src/colony/job.rs:197` |
| J12 | DigClay<br>`JobKind::DigClay=12` | person anchor1×1 | work_a / movement / target | `art/character/route_i/adult_M_loincloth_short/S/work_a.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>이동중 idle/walk_a; 수면 sleep가 우선. 기존 work/idle 대체는 동작 의미의 개별 수락 아님; 비도끼 전용 pose 제작은 ENV1 범위 밖, activity glyph/label 필요.<br>근거: `rust/crates/sim-core/src/colony/job.rs:88`; `rust/crates/sim-core/src/colony/job.rs:197` |
| J13 | Teach<br>`JobKind::Teach=13` | person anchor1×1 | idle / movement / target | `art/character/route_i/adult_M_loincloth_short/S/idle.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>이동중 idle/walk_a; 수면 sleep가 우선. 기존 work/idle 대체는 동작 의미의 개별 수락 아님; 비도끼 전용 pose 제작은 ENV1 범위 밖, activity glyph/label 필요.<br>근거: `rust/crates/sim-core/src/colony/job.rs:88`; `rust/crates/sim-core/src/colony/job.rs:197` |
| J14 | Learn<br>`JobKind::Learn=14` | person anchor1×1 | idle / movement / target | `art/character/route_i/adult_M_loincloth_short/S/idle.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>이동중 idle/walk_a; 수면 sleep가 우선. 기존 work/idle 대체는 동작 의미의 개별 수락 아님; 비도끼 전용 pose 제작은 ENV1 범위 밖, activity glyph/label 필요.<br>근거: `rust/crates/sim-core/src/colony/job.rs:88`; `rust/crates/sim-core/src/colony/job.rs:197` |
| J15 | Hunt<br>`JobKind::Hunt=15` | person anchor1×1 | work_a / movement / target | `art/character/route_i/adult_M_loincloth_short/S/work_a.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>이동중 idle/walk_a; 수면 sleep가 우선. 기존 work/idle 대체는 동작 의미의 개별 수락 아님; 비도끼 전용 pose 제작은 ENV1 범위 밖, activity glyph/label 필요.<br>근거: `rust/crates/sim-core/src/colony/job.rs:88`; `rust/crates/sim-core/src/colony/job.rs:197` |
| J16 | Equip<br>`JobKind::Equip=16` | person anchor1×1 | idle / movement / target | `art/character/route_i/adult_M_loincloth_short/S/idle.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>이동중 idle/walk_a; 수면 sleep가 우선. 기존 work/idle 대체는 동작 의미의 개별 수락 아님; 비도끼 전용 pose 제작은 ENV1 범위 밖, activity glyph/label 필요.<br>근거: `rust/crates/sim-core/src/colony/job.rs:88`; `rust/crates/sim-core/src/colony/job.rs:197` |

## routine

| ID | 이름 / sim code | tile footprint | 그릴 상태 | 기존 05 자산 또는 NEW | 일정·제약 |
|---|---|---|---|---|---|
| Q00 | TendFire<br>`ActivityKind::TendFire=0` | person anchor1×1 | travel / work_a | `art/character/route_i/adult_M_loincloth_short/S/work_a.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>각 activity의 전용 포즈 없음; pose alias와 읽기 한계 공개. Play는 실제 child look 사용.<br>근거: `rust/crates/sim-core/src/components/activity.rs:8` |
| Q01 | Craft<br>`ActivityKind::Craft=1` | person anchor1×1 | travel / work_a | `art/character/route_i/adult_M_loincloth_short/S/work_a.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>각 activity의 전용 포즈 없음; pose alias와 읽기 한계 공개. Play는 실제 child look 사용.<br>근거: `rust/crates/sim-core/src/components/activity.rs:8` |
| Q02 | Repair<br>`ActivityKind::Repair=2` | person anchor1×1 | travel / work_a | `art/character/route_i/adult_M_loincloth_short/S/work_a.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>각 activity의 전용 포즈 없음; pose alias와 읽기 한계 공개. Play는 실제 child look 사용.<br>근거: `rust/crates/sim-core/src/components/activity.rs:8` |
| Q03 | HearthCircle<br>`ActivityKind::HearthCircle=3` | person anchor1×1 | travel / idle | `art/character/route_i/adult_M_loincloth_short/S/idle.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>각 activity의 전용 포즈 없음; pose alias와 읽기 한계 공개. Play는 실제 child look 사용.<br>근거: `rust/crates/sim-core/src/components/activity.rs:8` |
| Q04 | Explore<br>`ActivityKind::Explore=4` | person anchor1×1 | travel / walk_a | `art/character/route_i/adult_M_loincloth_short/S/walk_a.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>각 activity의 전용 포즈 없음; pose alias와 읽기 한계 공개. Play는 실제 child look 사용.<br>근거: `rust/crates/sim-core/src/components/activity.rs:8` |
| Q05 | Play<br>`ActivityKind::Play=5` | person anchor1×1 | travel / walk_a | `art/character/route_i/adult_M_loincloth_short/S/walk_a.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>각 activity의 전용 포즈 없음; pose alias와 읽기 한계 공개. Play는 실제 child look 사용.<br>근거: `rust/crates/sim-core/src/components/activity.rs:8` |
| Q06 | SitByFire<br>`ActivityKind::SitByFire=6` | person anchor1×1 | travel / idle | `art/character/route_i/adult_M_loincloth_short/S/idle.png` | 재사용·Part4 frame plan; NEW glyph는 Part1.2 code overlay<br>각 activity의 전용 포즈 없음; pose alias와 읽기 한계 공개. Play는 실제 child look 사용.<br>근거: `rust/crates/sim-core/src/components/activity.rs:8` |

## effect

| ID | 이름 / sim code | tile footprint | 그릴 상태 | 기존 05 자산 또는 NEW | 일정·제약 |
|---|---|---|---|---|---|
| E00 | Smoke<br>`hearth fire_lit; derived visual` | hearth tile; rising overlay | lit only / fade | `art/style05/env/misc/smoke_puff.png` | 재사용·Part4 conditional smoke<br>smoke entity/chronicle code 없음.<br>근거: `docs/v8/snapshot_schema.md`; `rust/crates/sim-core/src/chronicle.rs:20`; `scripts/world/colony/agent_layer.gd:384` |
| E01 | Hearth light<br>`fire_lit + time_of_day` | radius5 tiles | day0 / night restore value | **NEW** | NEW → Part1.4 + Part4 code light (generation0)<br>전역 -45% value 후 화로 반경 복원; 별도 physics 아님.<br>근거: `docs/v8/snapshot_schema.md`; `rust/crates/sim-core/src/chronicle.rs:20`; `scripts/world/colony/agent_layer.gd:384` |
| E02 | Snow<br>`season=3; ground only` | tileable1×1 | winter alpha≤0.60 | **NEW** | NEW → Part1.4 single snow texture<br>인물·물체는 ground snow transform 대상 아님.<br>근거: `docs/v8/snapshot_schema.md`; `rust/crates/sim-core/src/chronicle.rs:20`; `scripts/world/colony/agent_layer.gd:384` |
| E03 | Drought ground<br>`PressureKind::Drought=1` | ground tiles | grass hue→ochre | **NEW** | NEW → Part1.4 code tone; hard-stop drop allowed<br>재생성 없음; 선택 condition.<br>근거: `docs/v8/snapshot_schema.md`; `rust/crates/sim-core/src/chronicle.rs:20`; `scripts/world/colony/agent_layer.gd:384` |
| E04 | Cold snap<br>`PressureKind::ColdSnap=0` | ground/people status | winter/severity + cold cue | **NEW** | NEW → Part1.4 code tone; dedicated cold glyph deferred to M-ART2<br>별도 눈 texture 재생성 없음.<br>근거: `docs/v8/snapshot_schema.md`; `rust/crates/sim-core/src/chronicle.rs:20`; `scripts/world/colony/agent_layer.gd:384` |
| E05 | Beasts pressure<br>`PressureKind::Beasts=2` | wolf positions/banner | approach/attack/retreat | **NEW** | NEW → Part2 animals + Part1.2 code marker<br>늑대 A00 재사용; 별도 beast creature 없음.<br>근거: `docs/v8/snapshot_schema.md`; `rust/crates/sim-core/src/chronicle.rs:20`; `scripts/world/colony/agent_layer.gd:384` |
| E06 | Illness badge<br>`PressureKind::Illness=3; sick=1` | person badge | sick/recovered | **NEW** | NEW → Part1.2 code overlay; M-ART2 bridge integration deferred<br>현재 agent badge를 보존할 UI 의미; generated sick face는 없음.<br>근거: `docs/v8/snapshot_schema.md`; `rust/crates/sim-core/src/chronicle.rs:20`; `scripts/world/colony/agent_layer.gd:384` |
| E07 | Draft outline / target<br>`drafted=1; DraftOrder0..3` | person+target | hold/move/attack animal/attack person | **NEW** | NEW → Part1.2 code overlay; M-ART2 bridge integration deferred<br>외곽선/target line; 사람 sprite 재색칠 금지.<br>근거: `docs/v8/snapshot_schema.md`; `rust/crates/sim-core/src/chronicle.rs:20`; `scripts/world/colony/agent_layer.gd:384` |
| E08 | Mood band<br>`mood_bands=0..4` | person badge | none/low/normal/high/breakdown | **NEW** | NEW → Part1.2 code overlay; M-ART2 bridge integration deferred<br>별도 표정 variant는 범위 밖.<br>근거: `docs/v8/snapshot_schema.md`; `rust/crates/sim-core/src/chronicle.rs:20`; `scripts/world/colony/agent_layer.gd:384` |
| E09 | Death cue<br>`get_recent_deaths; ChronicleKind::Died=11` | death tile1×1 | reason/fade | **NEW** | NEW → 보류: 전용 인간 사체/죽음 효과는 ENV1 필수 asset 목록 밖<br>sleep frame을 dead human으로 수락하지 않는다.<br>근거: `docs/v8/snapshot_schema.md`; `rust/crates/sim-core/src/chronicle.rs:20`; `scripts/world/colony/agent_layer.gd:384` |
| E10 | Deer carcass<br>`ChronicleKind::Hunted=31; Meat11+Hide6` | dead art96×64 | fresh / removed | **NEW** | NEW → Part2 Deer dead frame reuse<br>실제 persistent carcass collector 없음; 선언된 death visualization.<br>근거: `docs/v8/snapshot_schema.md`; `rust/crates/sim-core/src/chronicle.rs:20`; `scripts/world/colony/agent_layer.gd:384` |
| E11 | Wolf carcass<br>`ChronicleKind::WolfKilled=35; WolfHide16` | dead art96×64 | fresh / removed | **NEW** | NEW → Part2 Wolf dead frame reuse<br>실제 persistent carcass collector 없음; 종류는 event/declared overlay에서만 알 수 있음.<br>근거: `docs/v8/snapshot_schema.md`; `rust/crates/sim-core/src/chronicle.rs:20`; `scripts/world/colony/agent_layer.gd:384` |
| E12 | Spoiled meat cue<br>`Wildlife.meat_lots expiry` | item1×1 | fresh / spoiled tint before removal | **NEW** | NEW → Part1.3 Meat global tint, generation0<br>spoiled persistent stack이 없으므로 선언된 state.<br>근거: `docs/v8/snapshot_schema.md`; `rust/crates/sim-core/src/chronicle.rs:20`; `scripts/world/colony/agent_layer.gd:384` |
| E13 | Construction progress<br>`site_delivered/needed; progress/required` | site footprint | delivery/work/complete | **NEW** | NEW → Part1.2 code overlay<br>B16 ghost 위 bar/text; 모든 kind에 같은 규칙.<br>근거: `docs/v8/snapshot_schema.md`; `rust/crates/sim-core/src/chronicle.rs:20`; `scripts/world/colony/agent_layer.gd:384` |
| E14 | Contact shadow<br>`display derived` | person28×8px | standing / sleeping anchor | **NEW** | NEW → Part4 code ellipse, generation0<br>PNG에 굽지 않음; HANDOVER foot anchor.<br>근거: `docs/v8/snapshot_schema.md`; `rust/crates/sim-core/src/chronicle.rs:20`; `scripts/world/colony/agent_layer.gd:384` |
| E15 | Person LOD marker<br>`zoom≤0.5` | 10px circle;0.25=6px | top colour+1px ink; activity at0.5 | **NEW** | NEW → Part4 code overlay<br>1/2 without LOD도 별도 렌더.<br>근거: `docs/v8/snapshot_schema.md`; `rust/crates/sim-core/src/chronicle.rs:20`; `scripts/world/colony/agent_layer.gd:384` |
| E16 | Animal LOD marker<br>`zoom≤0.5; animal kind` | triangle | wolf red/deer tan | **NEW** | NEW → Part4 code overlay<br>A00/A01을 대체하는 축소 표시.<br>근거: `docs/v8/snapshot_schema.md`; `rust/crates/sim-core/src/chronicle.rs:20`; `scripts/world/colony/agent_layer.gd:384` |
| E17 | Roof/indoor cue<br>`get_rooms.roofed; TileGrid.outdoor` | room tile set | roofed/unroofed | **NEW** | NEW → Part1.2 code overlay<br>B15 storehouse 외 bedroom/workshop/hall도 적용; opaque roof deferred.<br>근거: `docs/v8/snapshot_schema.md`; `rust/crates/sim-core/src/chronicle.rs:20`; `scripts/world/colony/agent_layer.gd:384` |
| E18 | Tool break cue<br>`ChronicleKind::ToolBroke=24` | person/item position | intact / tool removed | **NEW** | NEW → 보류: 전용 깨짐 effect는 ENV1 필수 목록 밖<br>current chronicle/UI로만 인계; broken item enum 없음.<br>근거: `docs/v8/snapshot_schema.md`; `rust/crates/sim-core/src/chronicle.rs:20`; `scripts/world/colony/agent_layer.gd:384` |
| E19 | Fight / quarrel cue<br>`ChronicleKind::Quarrel=19,Fight=32` | people position | talk conflict / hit | **NEW** | NEW → 보류: 전용 combat hit effect는 ENV1 범위 밖<br>늑대 attack A00은 별도 필수; 사람 combat pose는 generation되지 않음.<br>근거: `docs/v8/snapshot_schema.md`; `rust/crates/sim-core/src/chronicle.rs:20`; `scripts/world/colony/agent_layer.gd:384` |

## Exhaustive enum 대조

| enum | 실제 variants | row 대응 / 누락 방지 |
|---|---|---|
| `TerrainType` | 10: `Plain`, `Forest`, `Mountain`, `Hill`, `River`, `Coast`, `Desert`, `Tundra`, `Swamp`, `Cave` | T00–T09 |
| `ItemKind` | 17: `Log`, `Firewood`, `Stone`, `Flint`, `FlintBlade`, `StoneAxe`, `Hide`, `Clay`, `RawPot`, `ClayPot`, `GroundAxe`, `Meat`, `CuredHide`, `HideWrap`, `FurCloak`, `Spear`, `WolfHide` | I00–I16 |
| `BuildingKind` | 4: `Hearth`, `Shelter`, `Stockpile`, `Hut` | B00–B03 |
| `BlueprintKind` | 12: `Hut`, `KnappingSpot`, `WoodBench`, `DryingRack`, `LogWall`, `StoneWall`, `Door`, `PackedFloor`, `Bedroll`, `GrindingStone`, `PotteryBench`, `HearthFire` | B03(Hut0), B04–B13(1–10), B00(HearthFire11) |
| `NodeKind` | 3: `Boulder`, `FlintOutcrop`, `ClayBank` | N08–N10 |
| `Designation` | 6: `Chop`, `Quarry`, `Dig`, `Deconstruct`, `ClayPit`, `Hunt` | M00–M05 |
| `ZoneKind` | 1: `Stockpile` | M06 |
| `RoomRole` | 5: `Unassigned`, `Bedroom`, `Storehouse`, `Workshop`, `Hall` | R00–R04 |
| `AnimalKind` | 2: `Wolf`, `Deer` | A00–A01 |
| `AnimalState` | 6: `Wander`, `Flee`, `Rest`, `Hunt`, `Attack`, `Retreat` | 아래6-state pose map |
| `Top` | 2: `HideWrap`, `FurCloak` | P05–P06; no Apparel=0은P04 |
| `JobKind` | 16: `Chop`, `Haul`, `Deliver`, `Build`, `Quarry`, `Supply`, `Gather`, `Dig`, `Deconstruct`, `Clear`, `DigClay`, `Teach`, `Learn`, `Hunt`, `Equip`, `Craft` | J01–J16 |
| `ActivityKind` | 7: `TendFire`, `Craft`, `Repair`, `HearthCircle`, `Explore`, `Play`, `SitByFire` | Q00–Q06 |
| `RecipeKind` | 11: `KnapBlade`, `HaftAxe`, `SplitFirewood`, `KnapFineBlade`, `GrindAxe`, `FormPot`, `FirePot`, `HaftSpear`, `ScrapeHide`, `MakeHideWrap`, `MakeFurCloak` | 아래11-recipe route |
| `PressureKind` | 4: `ColdSnap`, `Drought`, `Beasts`, `Illness` | E04/E03/E05/E06 |
| `ChronicleKind` | 38: `Born`, `FoundingMember`, `Paired`, `CameOfAge`, `BecameElder`, `Departed`, `DepartureCancelled`, `Settled`, `Joined`, `LeftSettlement`, `Widowed`, `Died`, `SettlementFormed`, `FissionStarted`, `DaughterFounded`, `FissionCancelled`, `SettlementDissolved`, `PorterDeparted`, `PorterDelivered`, `Quarrel`, `VillagesEstranged`, `VillagesReconciled`, `HutBuilt`, `RoomFormed`, `ToolBroke`, `KnowledgeLearned`, `KnowledgeDiscovered`, `KnowledgeAtRisk`, `KnowledgeLost`, `PressureBegan`, `PressureEnded`, `Hunted`, `Fight`, `Breakdown`, `Drafted`, `WolfKilled`, `Sick`, `Recovered` | inventory_source.json의38-row chronicle_routes; 별도 효과는E00–E19 |

### 동물6개 상태의 pose 대응

| code/state | deer | wolf |
|---|---|---|
| 0 Wander | stand/graze/walk by movement | stand/walk by movement |
| 1 Flee | run | not normal species state |
| 2 Rest | stand (dedicated rest pose deferred) | not normal species state |
| 3 Hunt | not normal species state | walk/run by movement |
| 4 Attack | not normal species state | attack |
| 5 Retreat | not normal species state | run |

dead는7번째 sim state가 아니라 Part2가 요구하는 사체 art frame이다. N facing은 별도 제작 대상이 아니며 S/E/W의 표시 범위를 명시한다.

### 11개 recipe의 물체/상태 대응

| recipe | drawing row route |
|---|---|
| KnapBlade | B04+I03→I04 |
| HaftAxe | B05+I04+I00→I05 |
| SplitFirewood | B05+I00→I01 |
| KnapFineBlade | B04+I03→I04 (quality only) |
| GrindAxe | B12+I05+I02→I10 |
| FormPot | B13+I07→I08 |
| FirePot | B00+I08+I01→I09 |
| HaftSpear | B05+I04+I00→I15 |
| ScrapeHide | B06+I06+I04→I12 |
| MakeHideWrap | B06+I12→I13 |
| MakeFurCloak | B06+I16→I14 |

bill의 MakeN/MaintainN, suspended, active, input shelf는 위 물체들을 조합하는 상태다. idle/inputs/working/stopped를 구분하며 레시피마다 새 작업대 PNG를 요구하지 않는다.

## 채택 사람34룩의 정확한 원천

아래 각 directory에 `S/{idle,walk_a,work_a,carry,sleep}.png`와 `E/{idle,walk_a,work_a,carry,sleep}.png`가 모두 존재함을 확인했다. JSON에는340개 exact path가 있다. 의복은 whole person 안에 있어 부품 합성을 하지 않는다. Portrait가 만들어지면 이34행을 기준으로1개씩 생산한다.

| look | path | frames |
|---|---|---|
| adult_F_fur_cloak_bun | `art/character/route_i/adult_F_fur_cloak_bun/` | 10 |
| adult_F_fur_cloak_long_tied | `art/character/route_i/adult_F_fur_cloak_long_tied/` | 10 |
| adult_F_fur_cloak_short | `art/character/route_i/adult_F_fur_cloak_short/` | 10 |
| adult_F_hide_wrap_bun | `art/character/route_i/adult_F_hide_wrap_bun/` | 10 |
| adult_F_hide_wrap_long_tied | `art/character/route_i/adult_F_hide_wrap_long_tied/` | 10 |
| adult_F_hide_wrap_short | `art/character/route_i/adult_F_hide_wrap_short/` | 10 |
| adult_F_loincloth_bun | `art/character/route_i/adult_F_loincloth_bun/` | 10 |
| adult_F_loincloth_long_tied | `art/character/route_i/adult_F_loincloth_long_tied/` | 10 |
| adult_F_loincloth_short | `art/character/route_i/adult_F_loincloth_short/` | 10 |
| adult_M_fur_cloak_bun | `art/character/route_i/adult_M_fur_cloak_bun/` | 10 |
| adult_M_fur_cloak_short | `art/character/route_i/adult_M_fur_cloak_short/` | 10 |
| adult_M_hide_wrap_bun | `art/character/route_i/adult_M_hide_wrap_bun/` | 10 |
| adult_M_hide_wrap_short | `art/character/route_i/adult_M_hide_wrap_short/` | 10 |
| adult_M_loincloth_bun | `art/character/route_i/adult_M_loincloth_bun/` | 10 |
| adult_M_loincloth_long_tied | `art/character/route_i/adult_M_loincloth_long_tied/` | 10 |
| adult_M_loincloth_short | `art/character/route_i/adult_M_loincloth_short/` | 10 |
| child_F_fur_cloak_bun | `art/character/route_i/child_F_fur_cloak_bun/` | 10 |
| child_F_fur_cloak_long_tied | `art/character/route_i/child_F_fur_cloak_long_tied/` | 10 |
| child_F_fur_cloak_short | `art/character/route_i/child_F_fur_cloak_short/` | 10 |
| child_F_hide_wrap_bun | `art/character/route_i/child_F_hide_wrap_bun/` | 10 |
| child_F_hide_wrap_long_tied | `art/character/route_i/child_F_hide_wrap_long_tied/` | 10 |
| child_F_hide_wrap_short | `art/character/route_i/child_F_hide_wrap_short/` | 10 |
| child_F_loincloth_bun | `art/character/route_i/child_F_loincloth_bun/` | 10 |
| child_F_loincloth_long_tied | `art/character/route_i/child_F_loincloth_long_tied/` | 10 |
| child_F_loincloth_short | `art/character/route_i/child_F_loincloth_short/` | 10 |
| child_M_fur_cloak_bun | `art/character/route_i/child_M_fur_cloak_bun/` | 10 |
| child_M_fur_cloak_long_tied | `art/character/route_i/child_M_fur_cloak_long_tied/` | 10 |
| child_M_fur_cloak_short | `art/character/route_i/child_M_fur_cloak_short/` | 10 |
| child_M_hide_wrap_bun | `art/character/route_i/child_M_hide_wrap_bun/` | 10 |
| child_M_hide_wrap_long_tied | `art/character/route_i/child_M_hide_wrap_long_tied/` | 10 |
| child_M_hide_wrap_short | `art/character/route_i/child_M_hide_wrap_short/` | 10 |
| child_M_loincloth_bun | `art/character/route_i/child_M_loincloth_bun/` | 10 |
| child_M_loincloth_long_tied | `art/character/route_i/child_M_loincloth_long_tied/` | 10 |
| child_M_loincloth_short | `art/character/route_i/child_M_loincloth_short/` | 10 |

결손 `adult_M_fur_cloak_long_tied`→`adult_M_fur_cloak_bun`, `adult_M_hide_wrap_long_tied`→`adult_M_hide_wrap_bun`은 기존 채택 결정의 hair fallback이다. 이를 신규 성공 룩으로 세지 않는다. 전용 elder/infant/N-back/머리색/표정은 이번 제작 범위 밖이며 Part3 초상은 droppable이다.

## 검증 범위

- 동결 commit과 schema5,16개 enum의 count·variant를 파일에서 추출해 검증했다. 총144개 enum variant를 대조했으며 코드가 중복인 aliases와 payload는 새 종류로 세지 않았다.
- 35 baseline environment PNG와 인벤토리가 참조한 기존 경로의 존재를 확인했다. accepted person manifest34룩과340frame 존재도 확인했다. PNG 존재만으로 scale·seam·readability·judge 수락을 주장하지 않는다.
- `inventory_source.json`은 읽은 simulation source의 SHA-256, 전체 rows, enum variants, animal state map, recipe routes,38개 chronicle route,340개 person path를 담는다.
- game code·simulation tree는 읽기만 수행했다. 전용 biome/효과/infant 등의 보류는 전체 drawable coverage가 아직100%가 아님을 뜻한다. 최종 보고서에서 보류 수와 사유를 유지한다.
