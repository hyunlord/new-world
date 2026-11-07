# WorldSim 콘텐츠 바이블 v1.1 — 문명 발전 콘텐츠 계획 (2026-11-07, 오너 결정 반영)

> 오너 결정(2026-11-07 04:52–05:10): 기술·건물·아이템·시대·국가의 발전을 **콘텐츠(데이터)로 먼저 넓게 만들어 두고**, 세부 로직과 연결은 그 뒤에 맞춘다. 박물관이어도 전시품이 데이터면 괜찮다.
> 이 문서는 그 콘텐츠를 **무엇으로, 얼마나, 어떤 순서로** 만들지 정한다. 실제 데이터 작성은 M-SPINE → C1 → C2 → C3 패스가 한다.
> 출처: V7 기술 목록 169개(`v7-final-sim` 태그의 `localization/ko/tech.json`), 테크트리 4자 합성, 콘텐츠 스케일 연구(물질 15→35→60→100종, 오브젝트 150종), 자율 문명 창발 설계(시저 3 주거 진화, 욕망의 길), 비주얼 딥리서치(시대별 시각 신호).

## 0. 원칙

1. **콘텐츠는 데이터, 행동은 코드**(ADR 0033). 기술·건물·아이템·조리법·작물·가축·시대는 전부 `sim-core/data/colony/*.ron`에 쓴다. 새 항목을 더할 때 코드가 필요 없어야 하고, 가족마다 데이터만으로 추가하는 시험이 있다.
2. **박물관 허용.** 아직 작동하지 않는 콘텐츠는 기술 지도와 건물 패널에 **회색(미연결)**으로 보인다. 숨기지 않는다. `docs/v8/CONTENT_DEBT.md`에 "무엇이 아직 안 도는지"를 항목별로 적는다.
3. **시대마다 작동 사슬 하나.** 새 동작(코드)이 필요한 사슬은 시대당 하나만 실제로 돌린다. 기존 장치(작업대에서 조리법으로 만들기, 짓기, 구역)만 쓰는 콘텐츠는 데이터를 쓰는 순간 바로 작동한다.
4. **균형은 나중.** 수치는 시작값일 뿐이고 `BALANCE_DEBT.md`에 쌓는다(D-FEATURE-FIRST).
5. **시대는 선언이 아니라 관측.** 표지 기술의 보급률 가중평균이 진척도다. 25 % 이상 "전환기", 75 % 이상 "시대". 전승자가 죽으면 되돌아갈 수 있다(V8 방향 §4.3, 이미 구현).
6. **아트는 임시.** 새 아이템·건물은 코드로 그린 자리표시(모양 + 재료 색조)로 시작하고, 아트 패스에서 교체한다. 자리표시도 시대·재료를 구분할 수 있어야 한다.
7. **발견 방식은 기존 것을 쓴다.** 실천 중 유레카, 이주민·교역·관찰 전파, 가르치기, 최소 실천자와 소실(M4). 새 기술은 선행 기술을 가진 사람이 관련 일을 하다가 발견한다.
8. **모든 결과와 과정에 인과가 있다** (오너 2026-11-07: "어떤 결과와 과정이든 인과가 있어야 한다"가 이 시뮬레이션의 목적). 콘텐츠에도 같은 규칙을 건다:
   - 어떤 콘텐츠도 명령으로 나타나지 않는다. 기술은 누군가가 선행 기술로 관련 일을 하다가 발견했기 때문에, 작물은 그 야생종이 지도에 있었고 누군가 씨를 모았기 때문에, 가축은 그 야생 동물이 지도에 있었고 누군가 새끼를 잡았기 때문에, 건물은 요청·구역·집사 결정이 있었기 때문에, 시대는 표지 기술의 보급률 때문에 나타난다. 모든 항목의 데이터에 `origin`(나타나는 조건)이 있고, 조건을 만족하는 사건이 연대기에 원인 링크와 함께 남는다.
   - 따라서 **기후·지형이 콘텐츠를 고른다.** 그 세계에 없는 야생종에서 나오는 작물·가축은 그 세계에서는 나타나지 않는다(교역으로 들어올 수는 있고, 그 경우 원인은 교역이다).
   - 박물관(회색) 항목도 `origin`을 갖는다. 연결 패스는 원인 사슬을 잇는 작업이지 항목을 켜는 작업이 아니다.
   - 검증: 새 콘텐츠 가족마다 "첫 등장의 왜?"가 사슬 깊이 ≥ 2로 열리는 시험(예: 첫 청동 도끼 ← 주조 ← 구리 주괴 ← 제련 ← 광석·숯 ← 채굴·광맥 찾기).

## 1. 시대 사다리

| 시대 | id | 표지 기술 (가중치) | 전환기 느낌 | 이 시대의 "작동 사슬" |
|---|---|---|---|---|
| 구석기 | `paleolithic` | 시작 시대. 표지 없음 | 불·타제 석기·채집·사냥·오두막 | (있음: 베기·사냥·짓기·토기) |
| 신석기 | `neolithic` | 간석기 3, 토기 소성 3, 농경 3, 정착 주거 2, 직조 2, **가축 2** | 밭·토기·천 옷·가축 우리·곡식 창고·거석 | **가축**: 야생 염소 포획 → 우리 → 번식 → 젖·고기·가죽 |
| 청동기 | `bronze` | 구리 제련 3, 합금 3, 거푸집 주조 2, 바퀴 2, 눈금 셈 1, 흙벽돌 1 | 광산·제련로·청동 도구·수레·흙벽돌 집·족장의 집·셈 | **금속**: 지표 광맥 → 채굴 → 숯 → 제련로 → 주괴 → 주조 → 청동 도끼 |
| 철기 | `iron` | 철 제련 3, 단조 3, 쟁기 2, 포장 도로 1, 시장 1, 성문법 1 | 대장간·철 도구·길·시장·성벽·법 | **철**: 철광 → 괴철로 → 대장간 단조 → 철 도구 |
| 고전기 | `classical` | (후속) 석조 건축, 문자 기록, 화폐, 상비군 | 신전·광장·수로·도서관 | 후속 |

- 순동(동기)은 청동기 안의 1단계(tier)로 둔다. 시대를 따로 두면 표지가 너무 적다.
- 각 시대의 "장(chapter)"은 M-LOOP의 `chapters.json`에 이어 붙인다: 4장 "첫 밭과 천 옷"(있음) → 5장 "우리와 가축" → 6장 "첫 구리" → 7장 "청동의 마을" → 8장 "철과 길" → 9장 "읍이 되다".

## 2. 카테고리 레인 (기술 지도의 세로축)

| 레인 | id | 내용 |
|---|---|---|
| 식량·채집 | `food` | 채집, 요리, 보존, 빵, 발효, 유제품 |
| 사냥·어로·가축 | `animals` | 사냥 도구, 함정, 낚시, 길들이기, 목축, 선택 교배 |
| 농경 | `farming` | 재배, 작물 종류, 윤작, 쟁기, 관개, 과수 |
| 석기·도구 | `tools` | 타제·간석기, 자루, 뼈·나무 도구, 바구니, 밧줄 |
| 건설·주거 | `building` | 오두막, 벽·바닥·문, 흙벽돌, 석조, 지붕, 거석, 우물, 길 |
| 불·토기·금속 | `heat` | 불 다루기, 숯, 토기, 가마, 제련, 합금, 주조, 단조 |
| 섬유·가죽·옷 | `textile` | 가죽 가공, 실 잣기, 직조, 바느질, 염색 |
| 사회·의례·법 | `society` | 이야기, 의례, 원로 회의, 수장, 금기, 혼인 동맹, 법, 조세 |
| 기록·셈·역법 | `record` | 눈금 셈, 음력, 인장, 문자, 도량형 |
| 교통·교역·군사 | `trade` | 뗏목·카누, 바퀴·수레, 교역로, 시장, 목책, 성벽, 무기·갑옷 |

## 3. 데이터 스키마 (M-SPINE이 만든다)

**기술** `knowledge.ron` (기존 31개 확장)
- `id`, `name_key`, `era`, `lane`, `tier`(시대 안 1–4), `prereq`(필수 목록), `soft_prereq`(있으면 발견 빨라짐), `discovery`(실천 활동 태그, 유레카 가중치, 전파 채널), `min_practitioners`, `grace_days`
- `origin`: 이 항목이 세계에 나타나는 조건 — 기술은 `practice`(어떤 일을 몇 번) + `prereq`, 작물·가축은 `wild_source`(야생종 id) + `climate[]` + `terrain[]`, 건물은 `requires_knowledge` + `trigger`(요청·구역·집사·명령), 시대는 표지 보급률. 원칙 8.
- `unlocks`: `recipes[]`, `buildings[]`, `items[]`, `zones[]`, `jobs[]`, `actions[]`, `modifiers{}`
- `marker`(시대, 가중치) 또는 없음
- `status`: `connected` | `inert`(데이터는 있으나 unlock 대상이 아직 코드에 없음)
- `ui`(x, y)는 시대 칼럼 × 레인으로 자동 배치하고, 필요한 것만 손으로 보정

**건물** `buildings.ron` (M-GROW 1번 항목, M-SPINE으로 당김)
- `id`, `name_key`, `era`, `category`(주거·저장·생산·사회·방어·기반), `footprint`(w×h), `pieces[]`(벽·바닥·문·지붕 조각과 재료 태그), `stations[]`(작업대), `requires_knowledge[]`, `materials{}`, `function`(잠자리 수, 저장 종류·용량, 조리법 집합, 열, 모임), `upgrade_to`(다음 단계), `zone_kind`(어떤 구역에서 자동으로 지어지나), `desirability`(주변 매력도 ±), `art`(자리표시 모양·재료 색)

**아이템** `items.ron` (기존 확장): `category`(재료·도구·무기·옷·음식·그릇·귀중품·기록물), `material_tags`, `tool_kind`와 `efficiency`, `durability`, `apparel`, `food`, `value`

**조리법** `recipes.ron`: `station`, `inputs(태그·속성 조건)`, `outputs`, `work_ticks`, `requires_knowledge`

**작물** `crops.ron`(있음), **가축** `animals.ron`(기존 야생동물 확장): `tameable`, `pen_building`, `products{}`(젖·털·고기·가죽·알), `breeding`, `feed`, 그리고 `origin.wild_source`·`origin.climate[]`·`origin.terrain[]` (§4.5)

**시대** `eras.ron`: 순서, 표지와 가중치, 전환기·시대 문턱, 장(chapter) 연결

## 4. 시대별 콘텐츠

### 4.1 구석기 (`paleolithic`) — 있음 31개 + 추가 12개
기존 31개(불 다루기, 채집, 물 찾기, 타제 석기, 자루, 장작 쪼개기, 눌러떼기, 간석기, 오두막, 통나무 벽, 문, 다진 바닥, 잠자리, 돌벽, 채석, 파기, 점토 캐기·다루기, 토기 빚기·소성, 가르치기, 이야기, 추적, 가죽 벗기기, 요리, 곡물 볶기, 말리기, 재배, 실 잣기, 직조, 바느질)은 그대로 두고 레인·시대·tier만 붙인다. 간석기·토기·재배·직조는 **신석기 표지**이므로 era를 `neolithic`으로 옮긴다.

추가 기술(V7 목록에서 채택):

| id | 이름 | 레인 | 선행 | 여는 것 |
|---|---|---|---|---|
| `cordage` | 끈 꼬기 | tools | foraging | 아이템 끈, 바구니·함정 선행 |
| `basketry` | 바구니 짜기 | tools | cordage | 아이템 바구니(운반량 +) |
| `trapping` | 함정 사냥 | animals | tracking, cordage | 작업 덫 놓기, 소형 짐승 고기 |
| `spear` | 창 | tools | hafting | 아이템 창(있음) 정식 연결 |
| `sling` | 투석구 | animals | cordage | 아이템 투석구 |
| `atlatl` | 투창기 | animals | spear | 사냥 효율 |
| `bone_tools` | 뼈 도구 | tools | flint_knapping | 바늘·송곳(바느질 선행) |
| `fishing` | 낚시·어살 | animals | cordage | 작업 낚시(물가), 물고기 |
| `nut_processing` | 견과 가공 | food | foraging | 개암 저장 조리법 |
| `hide_working` | 가죽 가공 | textile | hide_scraping | 가죽 옷·가죽 끈 |
| `ochre` | 황토 채색 | society | — | 의례 터 장식(사회 레인 첫 노드) |
| `weather_lore` | 날씨 읽기 | food | — | 한파·가뭄 예보 카드 |

구석기 건물(자리표시): 모닥불(있음: 화덕), 오두막(있음), 바람막이, 말리는 틀(있음), 덫. 아이템: 끈, 바구니, 투석구, 투창기, 바늘, 가죽 옷(있음), 물고기, 견과.

### 4.2 신석기 (`neolithic`) — 목표 45개 (이관 5 + 신규 40)

| id | 이름 | 레인 | tier | 선행 | 여는 것 | 상태 |
|---|---|---|---|---|---|---|
| `stone_grinding`(이관) | 간석기 | tools | 1 | pressure_flaking | 간 돌도끼(효율 ↑) | connected |
| `pot_firing`(이관) | 토기 소성 | heat | 1 | pot_forming | 항아리(있음) | connected |
| `cultivation`(이관) | 재배 | farming | 1 | foraging, grain_parching | 밭(있음) | connected |
| `weaving`(이관) | 직조 | textile | 1 | spinning | 베틀(있음) | connected |
| `tailoring`(이관) | 바느질 | textile | 1 | weaving, bone_tools | 아마 옷(있음) | connected |
| `animal_taming` | 길들이기 | animals | 1 | trapping | 작업 포획(염소·돼지 새끼), **우리** | **C1 작동 사슬** |
| `herding` | 목축 | animals | 2 | animal_taming | 방목 구역, 젖·털 | C1 |
| `dairy` | 젖 가공 | food | 3 | herding, pot_firing | 조리법 발효유·치즈 | 데이터만으로 작동 |
| `dog_taming` | 개 길들이기 | animals | 1 | trapping | 사냥·경계 보정 | inert |
| `selective_breeding` | 선택 교배 | animals | 3 | herding | 가축 생산량 ↑ | inert |
| `querns` | 맷돌 | food | 1 | stone_grinding, cultivation | 아이템 가루, 건물 맷돌 | 작동(조리법) |
| `oven` | 화덕 가마 | heat | 2 | querns, clay_prep | 건물 화덕 가마, 조리법 빵 | 작동 |
| `fermentation` | 발효 | food | 2 | pot_firing | 조리법 맥주·발효 채소 | 작동 |
| `granary` | 곡식 창고 | building | 2 | cultivation, packed_floor | 건물 곡식 창고(썩음 ↓) | 작동(저장 종류) |
| `food_preservation` | 염장·훈제 | food | 2 | food_drying | 훈제소, 훈제 고기 | 작동 |
| `barley`·`lentil`·`pea`·`millet` | 보리·렌틸·완두·기장 | farming | 1–2 | cultivation | 작물 4종 | 작동(crops) |
| `orchard` | 과수 | farming | 3 | cultivation | 사과나무 심기 | inert(다년생) |
| `crop_rotation` | 돌려짓기 | farming | 3 | barley, lentil | 지력 회복 ↑ | 작동(soil) |
| `hoe` | 괭이 | tools | 1 | stone_grinding, hafting | 아이템 괭이(갈기 ↑) | 작동 |
| `sickle` | 낫 | tools | 2 | pressure_flaking, hafting | 아이템 낫(수확 ↑) | 작동 |
| `adze` | 자귀 | tools | 2 | stone_grinding | 목공 품질 | 작동 |
| `wattle_daub` | 가지 엮음 미장 | building | 1 | timber_wall, clay_prep | 벽 종류(단열 중간) | 작동(재료) |
| `thatch` | 초가 지붕 | building | 1 | cordage | 지붕 조각(비·단열) | inert(지붕 체계) |
| `mud_brick` | 흙벽돌 | building | 3 | clay_prep, oven | 벽돌, 벽돌 벽 | 작동(재료)·청동기 표지 |
| `well` | 우물 | building | 2 | digging, stone_walling | 건물 우물(물 찾기 불필요) | C1 선택 |
| `fence` | 울타리 | building | 1 | cordage, timber_wall | 울타리 조각, 우리 | C1 |
| `megalith` | 거석 세우기 | building | 4 | stone_quarrying, cordage | 건물 거석(의례 터) | inert |
| `basket_loom`→`loom`(있음) | | | | | | |
| `dyeing` | 염색 | textile | 3 | weaving, ochre | 색 옷(가구 색) | 작동(appearance) |
| `leather_armor` | 가죽 갑옷 | trade | 2 | hide_working | 아이템 갑옷(방어) | 작동(apparel) |
| `bow` | 활과 화살 | trade | 2 | cordage, bone_tools | 아이템 활(사냥·전투) | 작동 |
| `dugout_canoe` | 통나무 카누 | trade | 3 | adze | 아이템 카누(강 건너기) | inert |
| `barter` | 물물교환 | trade | 1 | storytelling | 교역 카드 확장(있음) | 작동 |
| `tally` | 눈금 셈 | record | 2 | storytelling | 아이템 눈금 막대, 창고 장부 | inert·청동기 표지 |
| `lunar_calendar` | 달력 | record | 3 | tally, weather_lore | 계절 예보 정확 | inert |
| `ritual` | 의례 | society | 1 | ochre, storytelling | 건물 의례 터, 잔치 사건 | inert(M8) |
| `elder_council` | 원로 회의 | society | 2 | ritual | 집사 결정에 원로 의견 | inert(M8) |
| `kinship` | 혈연 집단 | society | 1 | storytelling | 가구 묶음 표시 | inert |
| `taboo` | 금기 | society | 3 | ritual | 특정 음식·행동 금지 | inert(M7) |
| `herbalism` | 약초 | food | 2 | foraging | 약초 아이템, 병 회복 | 작동(sick) |
| `bone_setting` | 접골 | food | 3 | herbalism | 상처 치료 | inert(M9) |

신석기 건물(목록): 우리, 방목 구역(구역), 곡식 창고, 맷돌, 화덕 가마, 훈제소, 우물, 울타리, 통나무집(오두막 2단계), 흙벽돌집(3단계), 의례 터, 거석, 창고(지붕, 있음). 아이템: 괭이·낫·자귀·활·가루·빵·맥주·치즈·훈제 고기·약초·눈금 막대·색 옷·가죽 갑옷·카누. 가축: 개·염소·양·돼지. 작물: 에머밀(있음)·아마(있음)·보리·렌틸·완두·기장·사과.

### 4.3 청동기 (`bronze`) — 목표 55개

| id | 이름 | 레인 | tier | 선행 | 여는 것 | 상태 |
|---|---|---|---|---|---|---|
| `charcoal` | 숯 굽기 | heat | 1 | fire_keeping, digging | 건물 숯가마, 아이템 숯 | **C2 사슬** |
| `prospecting` | 광맥 찾기 | heat | 1 | stone_quarrying | 지표 광맥 표시(구리·주석·철) | C2 |
| `mining` | 채굴 | tools | 1 | prospecting, pressure_flaking | 구역 채굴, 아이템 광석 | C2 |
| `copper_smelting` | 구리 제련 | heat | 2 | charcoal, mining, pot_firing | 건물 제련로, 구리 주괴 | C2·표지 |
| `cold_hammering` | 냉간 단조 | heat | 2 | copper_smelting | 구리 바늘·송곳 | 작동 |
| `mould_casting` | 거푸집 주조 | heat | 3 | copper_smelting, clay_prep | 건물 주조대, 구리 도끼 | C2·표지 |
| `tin_sourcing` | 주석 구하기 | trade | 3 | prospecting, barter | 교역품 주석 | C2 |
| `bronze_alloy` | 청동 합금 | heat | 4 | mould_casting, tin_sourcing | 청동 주괴·도끼·낫·창 | C2·표지 |
| `annealing` | 풀림 | heat | 4 | bronze_alloy | 도구 내구 ↑ | inert |
| `lost_wax` | 밀랍 주조 | heat | 4 | bronze_alloy, beekeeping | 장식품·조각 | inert |
| `beekeeping` | 양봉 | food | 2 | foraging | 벌통, 꿀·밀랍 | 작동 |
| `wheel` | 바퀴 | trade | 2 | adze | 아이템 바퀴 | 표지 |
| `cart` | 수레 | trade | 3 | wheel, herding | 운반량 ×4(소·당나귀) | inert |
| `cattle` | 소 기르기 | animals | 2 | herding | 가축 소, 젖·가죽·견인 | 작동(animals) |
| `plough` | 쟁기 | farming | 3 | cattle, adze | 밭 갈기 속도 ×3 | inert·철기 표지 선행 |
| `irrigation` | 관개 | farming | 3 | digging, cultivation | 구역 수로, 가뭄 저항 | inert |
| `terrace` | 계단밭 | farming | 4 | irrigation, stone_walling | 경사지 밭 | inert |
| `flax_retting`(있음) | | | | | | |
| `wool_spinning` | 양털 실 | textile | 2 | herding, spinning | 양털 실·모직 옷 | 작동 |
| `mud_brick_house` | 흙벽돌 집 | building | 1 | mud_brick | 건물 흙벽돌 집(4단계) | 작동 |
| `lime_plaster` | 석회 미장 | building | 2 | oven, stone_quarrying | 벽 마감(단열·미관) | 작동(재료) |
| `stone_masonry` | 석조 쌓기 | building | 3 | stone_walling, bronze_alloy | 석조 벽·창고 | 작동(재료) |
| `chief_hall` | 족장의 집 | society | 2 | elder_council, mud_brick_house | 건물 족장의 집 | inert(M8) |
| `chieftainship` | 수장제 | society | 3 | chief_hall | 집사 → 족장(이름 있는 사람) | inert(M8) |
| `shrine` | 사당 | society | 2 | ritual, mud_brick | 건물 사당 | inert |
| `marriage_alliance` | 혼인 동맹 | society | 3 | kinship, barter | 이웃 카드 | inert(M8) |
| `palisade` | 목책 | trade | 1 | timber_wall | 조각 목책, 문루 | 작동(조각) |
| `ditch` | 해자 | trade | 2 | digging | 조각 해자 | inert(지형) |
| `bronze_weapons` | 청동 무기 | trade | 4 | bronze_alloy | 청동 창·단검 | 작동(item) |
| `composite_bow` | 복합궁 | trade | 3 | bow, hide_working | 활 ↑ | 작동 |
| `caravan` | 대상 교역 | trade | 3 | cart, barter | 교역 규모 ↑, 교역소 | inert |
| `trading_post` | 교역소 | trade | 2 | barter, granary | 건물 교역소 | inert |
| `seal_stamp` | 인장 | record | 2 | tally, clay_prep | 아이템 인장(소유 표시) | inert |
| `proto_writing` | 그림 문자 | record | 3 | seal_stamp | 기록판(지식 소실 ↓) | inert·중요 |
| `standard_weights` | 도량형 | record | 3 | tally | 교역 공정 | inert |
| `solar_calendar` | 태양력 | record | 3 | lunar_calendar | 파종 시기 알림 | inert |
| `reed_boat`·`sail` | 갈대 배·돛 | trade | 3 | cordage, weaving | 강 교역 | inert |
| `wine`·`olive_press` | 포도주·기름 | food | 3 | fermentation, orchard | 조리법 | inert(기후 조건) |
| `glass_beads`·`faience` | 유리 구슬 | heat | 4 | bronze_alloy | 귀중품 | inert |
| `trepanation`·`surgery` | 외과 | food | 4 | bone_setting, bronze_weapons | 치료 | inert(M9) |
| `drum`·`flute`·`mask`·`tattoo` | 북·피리·가면·문신 | society | 1–2 | ritual | 의례 효과 | inert |

청동기 건물: 숯가마, 제련로, 주조대, 광산 입구(구역), 벌통, 흙벽돌 집, 석조 창고, 족장의 집, 사당, 교역소, 목책·문루, 수레 보관소. 아이템: 숯, 광석 3종, 주괴 3종, 구리·청동 도구 6종, 청동 무기 3종, 바퀴·수레, 꿀·밀랍, 모직 옷, 인장, 기록판, 유리 구슬, 석회.

### 4.4 철기 (`iron`) — 목표 50개

| id | 이름 | 레인 | tier | 선행 | 여는 것 | 상태 |
|---|---|---|---|---|---|---|
| `iron_smelting` | 철 제련 | heat | 1 | copper_smelting, charcoal | 건물 괴철로, 철 덩이 | **C3 사슬**·표지 |
| `forging` | 단조 | heat | 2 | iron_smelting | 건물 대장간, 철 도구 | C3·표지 |
| `iron_tools` | 철 도구 | tools | 2 | forging | 철 도끼·괭이·낫·톱 | C3 |
| `iron_weapons` | 철 무기 | trade | 3 | forging | 철 창·검·투구 | 작동(item) |
| `steel_edge` | 강철 날 | heat | 4 | forging | 내구 ↑ | inert |
| `iron_plough` | 철 쟁기 | farming | 2 | plough, iron_tools | 밭 ×5 | inert·표지 |
| `horse` | 말 기르기 | animals | 2 | cattle | 가축 말, 수레·기병 | inert |
| `paved_road` | 포장 도로 | building | 2 | stone_masonry | 조각 포장 길(이동 ↑) | 작동(조각)·표지 |
| `bridge` | 다리 | building | 3 | paved_road, adze | 조각 다리(강) | inert |
| `aqueduct`·`reservoir` | 수로·저수지 | building | 4 | irrigation, stone_masonry | 물 공급 | inert |
| `stone_house` | 석조 집 | building | 2 | stone_masonry | 집 5단계 | 작동 |
| `town_wall`·`gate`·`tower` | 성벽·성문·망루 | trade | 2–3 | stone_masonry, palisade | 방어 조각 | 작동(조각) |
| `market` | 시장 | trade | 2 | trading_post, standard_weights | 건물 시장(교역·분배) | inert·표지 |
| `coinage` | 화폐 | trade | 3 | market, bronze_alloy | 아이템 주화 | inert |
| `warehouse` | 큰 창고 | building | 2 | granary, stone_masonry | 저장 ×4 | 작동 |
| `codified_law` | 성문법 | society | 3 | proto_writing, elder_council | 다툼 해결 규칙, 재판 | inert(M8)·표지 |
| `taxation` | 조세 | society | 3 | codified_law, tally | 공동 창고 몫 | inert(M-NATION) |
| `bureaucracy` | 관료 | society | 4 | taxation, writing | 집사 역할 분화 | inert |
| `writing` | 문자 | record | 2 | proto_writing | 기록물(지식 영구 보존) | inert·중요 |
| `school` | 배움터 | record | 3 | writing, instruction | 건물 배움터(가르치기 ×3) | inert |
| `map_making` | 지도 | record | 3 | writing | 이웃 지도 | inert(M-NATION) |
| `militia`·`shield_wall`·`siege_ladder` | 민병·방진·공성 | trade | 2–4 | iron_weapons, chieftainship | 전투 규칙 | inert(M-NATION) |
| `polytheism`·`temple` | 다신교·신전 | society | 2–3 | shrine, stone_masonry | 건물 신전 | inert |
| `geometry`·`sundial` | 기하·해시계 | record | 3–4 | writing, standard_weights | 건설 정확 | inert |
| `rotary_quern`·`watermill` | 회전 맷돌·물레방아 | food | 2–4 | iron_tools | 가루 ×3 | inert |
| `silk`·`dye_purple` | 양잠·자주 염색 | textile | 4 | dyeing | 귀중품 | inert |
| `falconry`·`veterinary` | 매사냥·수의 | animals | 3 | horse | — | inert |

철기 건물: 괴철로, 대장간, 석조 집, 큰 창고, 시장, 배움터, 신전, 성벽·성문·망루, 다리, 포장 길, 수로. 아이템: 철 덩이, 철 도구 6종, 철 무기 4종, 주화, 책·기록물, 강철 날.

### 4.5 가축·작물은 기후와 지형이 고른다 (오너 결정 2026-11-07)

세계는 지금도 시드에서 기후를 뽑는다(`variety.rs`: 온화 / 보통 / 혹독 겨울 세 띠, 각 ⅓)이고, 재료 체계는 지형 10종(`TerrainType`: 평원·숲·산·구릉·강·해안·사막·툰드라·늪·동굴)마다 `natural_in`을 갖는다. 작물·가축은 여기에 얹는다:

- **야생종이 먼저.** `plants.ron`·`animals.ron`의 야생종에 `climate[]`·`terrain[]`을 붙여 그 세계에 실제로 자라는/사는 것만 생성한다. 지금 5종(열매 덤불·개암·야생 아마·야생 곡물·야생 뿌리)과 2종(사슴·늑대)에 추가한다.
- **길들인 종은 야생종에서만.** `crops.ron`·가축의 `origin.wild_source`가 그 세계에 없으면 그 작물·가축은 나타나지 않는다. 교역으로 씨앗·새끼가 들어오는 길만 열어 둔다(원인 = 교역 카드).
- **기후 띠는 수확량·번식률 수정자**이지 금지가 아니다. 단, 아래 표의 "—"는 그 띠에서 야생종이 생성되지 않음을 뜻한다.

| 종 | 야생종 | 온화 | 보통 | 혹독 | 지형 | 시대 |
|---|---|---|---|---|---|---|
| 에머밀(있음) | 야생 곡물 | ○ | ○ | △ | 평원·구릉 | 신석기 |
| 보리 | 야생 보리(신규) | ○ | ○ | ○ | 평원·구릉 | 신석기 |
| 호밀·귀리 | 야생 호밀(신규) | △ | ○ | ○ | 구릉·툰드라 가장자리 | 신석기 (혹독 띠의 주곡) |
| 렌틸·완두 | 야생 콩(신규) | ○ | ○ | — | 평원·강 | 신석기 |
| 기장 | 야생 기장(신규) | ○ | △ | — | 평원(건조) | 신석기 |
| 아마(있음) | 야생 아마 | ○ | ○ | ○ | 강·평원 | 신석기 |
| 사과 | 야생 사과(신규) | △ | ○ | ○ | 숲 가장자리·구릉 | 신석기 |
| 포도·올리브 | 야생 포도·올리브(신규) | ○ | — | — | 구릉·해안 | 청동기 |
| 대마 | 야생 대마(신규) | ○ | ○ | △ | 강 | 청동기 |
| 개 | 늑대(있음) | ○ | ○ | ○ | 어디든 | 신석기 |
| 염소 | 야생 염소(신규) | ○ | ○ | ○ | 구릉·산 | 신석기 — **C1 작동 사슬** |
| 양 | 야생 양(신규) | ○ | ○ | ○ | 구릉·산 | 신석기 |
| 돼지 | 멧돼지(신규) | ○ | ○ | △ | 숲·늪 | 신석기 |
| 소 | 오록스(신규) | ○ | ○ | △ | 평원 | 청동기 |
| 말 | 야생 말(신규) | ○ | ○ | △ | 평원 | 철기 |
| 순록 | 순록(신규) | — | — | ○ | 툰드라 | 신석기 (혹독 띠의 가축) |
| 닭 | 들닭(신규) | ○ | △ | — | 숲 | 청동기 |
| 벌 | 야생 벌집(신규) | ○ | ○ | △ | 숲·평원 | 청동기 |

혹독 띠의 세계는 보리·호밀·염소·양·순록으로, 온화 띠는 거기에 포도·올리브·기장·닭을 더해 간다. 같은 신석기라도 세계마다 다른 밭과 우리가 서는 게 의도다. 지도에 구릉·산이 없는 세계는 염소·양이 없으므로 C1 사슬 시험은 "구릉이 있는 고정 시드"로 돌린다.

### 4.6 수량 요약

| | 구석기 | 신석기 | 청동기 | 철기 | 합계 |
|---|---|---|---|---|---|
| 기술 | 31+12 = 43 (표지 5개 신석기로 이관 → 38) | 45 | 55 | 50 | **~190** |
| 건물 | 6 | 13 | 12 | 11 | **~42** (+ 조각·단계 변형으로 체감 70+) |
| 아이템 | 20 | 25 | 35 | 25 | **~105** (+ 재료 변형) |
| 가축·작물 | 0 / 2 | 4 / 7 | 1 / 2 | 1 / 1 | 6 / 12 |
| 시대마다 작동 사슬 | 있음 | 가축 | 금속 | 철 | |

## 5. 작성·연결 순서

| 패스 | 만드는 것 | 실제로 돌리는 것 | 화면 |
|---|---|---|---|
| **M-SPINE** | `eras.ron`, 기술 스키마 확장과 레인·tier 부여, `buildings.ron`과 건물 체계(한 번에 짓기, 구역에서 자라기, 단계), 자리표시 아트 규칙, `CONTENT_DEBT.md`, 회색 표시 | 건물 하나를 고르면 조각이 깔리고 지어짐; 주거 구역에서 집이 자동으로 섬 | 기술 지도 = 시대 칼럼 × 레인, 회색 노드; 건물 패널 |
| **C1 신석기** | §4.2 전체 데이터 | 가축 사슬(포획 → 우리 → 번식 → 산물) + 조리법만 쓰는 것들 전부 | 5장 "우리와 가축" |
| **C2 청동기** | §4.3 전체 데이터 | 금속 사슬(광맥 → 채굴 → 숯 → 제련 → 주조 → 청동 도끼) | 6–7장, 시대 재료 물결의 첫 단계(흙벽돌) |
| **C3 철기** | §4.4 전체 데이터 | 철 사슬(철광 → 괴철로 → 대장간 → 철 도구) + 포장 길 조각 | 8–9장 |
| M-GROW | 깊이감 있는 2D, 닳은 길, 자라는 집, 재료 물결, 연기·더미 | — | 첫인상 |
| M-NATION | 화면 밖 정착지, 공물·동맹·전쟁 카드, 지도 | 표현판 | — |
| 이후 | 회색 항목을 묶어서 연결(관개·쟁기·문자·시장·법·의례) — 대부분 M7·M8·M-BAL·M11과 함께 | | |

## 6. 보이는 방식

- **기술 지도:** 가로 = 시대(구석기 → 철기), 세로 = 레인 10줄. 노드 색: 밝음 = 이 마을이 안정적으로 가짐, 테두리 = 1명뿐(위험), 어둠 = 모름, **회색 빗금 = 콘텐츠는 있으나 아직 작동하지 않음**. 선택하면 선행·후행 사슬만 강조. 시대 칼럼 위에 표지 게이지(M6b 단계 3).
- **건물 패널:** 지금 있는 건물 / 지을 수 있는 건물 / 다음에 열릴 건물(조건: 기술·재료). 회색은 같은 규칙.
- **시대 전환:** 25 %에서 "신석기 전환기" 알림과 장(chapter) 열림, 75 %에서 "신석기 시대" + 연대기 + 집 재료 물결(M-GROW).
- **자리표시 아트:** 건물은 발자국 크기의 지붕 모양(둥근 = 구석기·신석기, 각진 = 청동기 이후) + 재료 색조(짚·통나무·흙벽돌·돌) + 기능 아이콘 1개. 아이템은 모양 8종(덩이·막대·날·그릇·천·곡물·광석·주괴) × 재료 색조.

## 7. 이번 판에서 뺀 것

고전기 이후(석조 공공건축·화폐 경제·상비군), 다문명 양식 DNA, 지하 자원 층(광맥은 지표 노출로 시작), 바다·항해(강 뗏목까지만), 종교의 실제 효과(M7·M8 뒤), 질병·의학의 실제 효과(M9 뒤).

## 8. 오너 결정 (2026-11-07 05:31)

1. 시대는 넷(구석기·신석기·청동기·철기). 순동은 청동기 1단계.
2. 신석기 작동 사슬은 가축. 빵·맥주·치즈 등 조리법만 쓰는 것은 데이터로 함께 넣는다.
3. 사회·기록 레인은 지금 데이터로 넣고 전부 회색.
4. 가축·작물은 기후·지형이 고른다 (§4.5).
5. **인과 원칙** (§0-8): 모든 결과와 과정에 원인이 있어야 하며, 콘텐츠도 `origin`을 갖는다.


---
_Generated by [Claude Code](https://claude.ai/code)_
