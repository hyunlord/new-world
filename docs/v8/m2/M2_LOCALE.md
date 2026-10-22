# M2 localization keys

Every key the M2 UI (Part 5) adds or rewrites, with its Korean and English text. Sources: `localization/{ko,en}/{ui,buildings}.json`, compiled by `python3 tools/localization_compile.py --project-root .`.
Formats use `Locale.trf` placeholders (`{name}`).

## New keys (90)

| key | file | ko | en | used by |
|---|---|---|---|---|
| `ITEM_FIREWOOD` | ui | 장작 | Firewood | item name |
| `ITEM_STONE` | ui | 돌 | Stone | item name |
| `ITEM_FLINT` | ui | 부싯돌 | Flint | item name |
| `ITEM_FLINT_BLADE` | ui | 부싯돌 날 | Flint blade | item name |
| `ITEM_STONE_AXE` | ui | 돌도끼 | Stone axe | item name |
| `ITEM_HIDE` | ui | 가죽 | Hide | item name |
| `ITEM_LOG_FMT` | ui | {material} 통나무 | {material} log | item with material |
| `ITEM_FIREWOOD_FMT` | ui | {material} 장작 | {material} firewood | item with material |
| `ITEM_STONE_FMT` | ui | {material} 돌 | {material} stone | item with material |
| `ITEM_FLINT_FMT` | ui | {material} | {material} | item with material (the material is the item) |
| `ITEM_FLINT_BLADE_FMT` | ui | {material} 날 | {material} blade | item with material |
| `ITEM_STONE_AXE_FMT` | ui | 돌도끼 ({material} 날) | stone axe ({material} blade) | item with material |
| `ITEM_HIDE_FMT` | ui | {material} 가죽 | {material} hide | item with material |
| `MATERIAL_FLINT` | ui | 부싯돌 | flint | material |
| `MATERIAL_CHERT` | ui | 처트 | chert | material |
| `MATERIAL_GRANITE` | ui | 화강암 | granite | material |
| `MATERIAL_LIMESTONE` | ui | 석회암 | limestone | material |
| `MATERIAL_OAK` | ui | 참나무 | oak | material |
| `NODE_BOULDER` | ui | 바위 | Boulder | overlay hover |
| `NODE_FLINT_OUTCROP` | ui | 부싯돌 노두 | Flint outcrop | overlay hover |
| `UI_NODE_LEFT_FMT` | ui | {name} · {item} {left}개 남음 | {name} · {left} {item} left | overlay hover |
| `UI_STACK_FMT` | ui | {item} ×{count} · 품질 {quality} | {item} ×{count} · quality {quality} | overlay hover, inspector carry |
| `BUILDING_KNAPPING_SPOT` | buildings | 돌깎이 자리 | Knapping spot | build menu, workshop panel |
| `BUILDING_WOOD_BENCH` | buildings | 목공대 | Wood bench | build menu, workshop panel |
| `BUILDING_DRYING_RACK` | buildings | 건조대 | Drying rack | build menu, workshop panel |
| `RECIPE_KNAP_BLADE` | ui | 부싯돌 날 깎기 | Knap a flint blade | workshop panel |
| `RECIPE_HAFT_AXE` | ui | 돌도끼 만들기 | Haft a stone axe | workshop panel |
| `RECIPE_SPLIT_FIREWOOD` | ui | 장작 패기 | Split firewood | workshop panel |
| `WORK_CRAFT` | ui | 제작 | Craft | priority table |
| `WORK_BUILD` | ui | 건축 | Build | priority table |
| `WORK_QUARRY` | ui | 채석 | Quarry | priority table |
| `WORK_CHOP` | ui | 벌목 | Chop | priority table |
| `WORK_SPLIT` | ui | 장작 | Firewood | priority table |
| `WORK_HAUL` | ui | 운반 | Haul | priority table |
| `SKILL_KNAP` | ui | 돌 다듬기 | Knapping | inspector |
| `SKILL_WOODWORK` | ui | 목공 | Woodwork | inspector |
| `SKILL_CHOP` | ui | 벌목 | Chopping | inspector |
| `SKILL_QUARRY` | ui | 채석 | Quarrying | inspector |
| `SKILL_BUILD` | ui | 건축 | Building | inspector |
| `SKILL_TIER_0` | ui | 초보 | Novice | inspector |
| `SKILL_TIER_1` | ui | 견습 | Apprentice | inspector |
| `SKILL_TIER_2` | ui | 숙련 | Skilled | inspector |
| `SKILL_TIER_3` | ui | 달인 | Master | inspector |
| `BILL_MODE_MAKE_FMT` | ui | 만들기 {count} | Make {count} | workshop panel |
| `BILL_MODE_MAINTAIN_FMT` | ui | 유지 {count} | Keep {count} | workshop panel |
| `BILL_AUTO` | ui | (자동) | (auto) | workshop panel |
| `BILL_DONE_FMT` | ui | {done}개 만듦 · {left}개 남음 | {done} made · {left} to go | workshop panel (make) |
| `BILL_MADE_FMT` | ui | {done}개 만듦 | {done} made | workshop panel (maintain) |
| `BILL_ACTIVE` | ui | 작업 중 | in work | workshop panel |
| `BILL_SUSPENDED` | ui | 멈춤 | suspended | workshop panel |
| `BILL_SUSPEND` | ui | 멈추기 | Suspend | workshop panel button |
| `BILL_RESUME` | ui | 다시 | Resume | workshop panel button |
| `BILL_REMOVE` | ui | 삭제 | Delete | workshop panel button |
| `BILL_ADD` | ui | 추가 | Add | workshop panel button |
| `BILL_TO_MAKE` | ui | 만들기로 | To make | workshop panel button |
| `BILL_TO_MAINTAIN` | ui | 유지로 | To keep | workshop panel button |
| `BILL_NONE` | ui | 주문 없음 | No bills | workshop panel |
| `UI_WORKSHOP_BILLS` | ui | 주문 | Bills | workshop panel |
| `UI_WORKSHOP_SHELF` | ui | 선반 | Shelf | workshop panel |
| `UI_WORKSHOP_NO_RECIPES` | ui | 이 작업장은 아직 만들 것이 없음 | Nothing to make here yet | workshop panel (drying rack) |
| `UI_WORKSHOP_PICK_RECIPE` | ui | 무엇을 만들까요? | What to make? | workshop panel add list |
| `UI_CLOSE` | ui | 닫기 | Close | workshop and priority panels |
| `UI_ORDER_BUILD` | ui | 짓기 | Build | order bar F3 |
| `UI_ORDER_QUARRY` | ui | 채석 | Quarry | order bar F7 |
| `UI_ORDER_TOAST_QUARRY` | ui | 바위·노두 {count}개 채석 표시 | {count} rocks marked for quarrying | order toast |
| `UI_ORDER_TOAST_BUILD_FMT` | ui | {name} 청사진 놓음 | {name} blueprint placed | order toast |
| `UI_ORDER_TOAST_BILL` | ui | 주문을 바꿈 | Bill changed | workshop panel toast |
| `UI_ORDER_TOAST_PRIORITY` | ui | 우선순위를 바꿈 | Priority changed | priority panel toast |
| `UI_ORDER_HINT_BUILD_MENU` | ui | 1–4로 고르세요 · F3 닫기 | Pick with 1–4 · F3 closes | order bar hint |
| `UI_ORDER_HINT_PLACE_FMT` | ui | 클릭해서 놓으세요 · {name} | Click to place · {name} | order bar hint |
| `UI_BLUEPRINT_MATERIALS` | ui | 재료 {delivered}/{needed} | {delivered}/{needed} materials | workshop blueprint label |
| `UI_REJECT_NO_COLONY` | ui | 마을이 없음 | No colony | order toast |
| `UI_REJECT_NOT_YET` | ui | 아직 할 수 없음 | Not yet | order toast |
| `UI_REJECT_WRONG_WORKSHOP` | ui | 이 작업장에서는 만들 수 없음 | Not made at this workshop | order toast |
| `UI_REJECT_UNKNOWN` | ui | 알 수 없는 명령 | Unknown order | order toast |
| `UI_PRIORITY_TITLE` | ui | 작업 우선순위 | Work priorities | priority panel |
| `UI_PRIORITY_RESET` | ui | 모두 기본 | All default | priority panel |
| `UI_PRIORITY_HINT` | ui | 칸을 누르면 바뀜: 1 먼저 … 4 나중, 0 안 함 · P 닫기 | Click a cell: 1 first … 4 last, 0 never · P closes | priority panel |
| `UI_PRIORITY_NAME` | ui | 이름 | Name | priority panel |
| `UI_PRIORITY_EMPTY` | ui | 어른이 없음 | No adults | priority panel |
| `UI_PRIORITY_ELDER` | ui | 노인 | elder | priority panel row |
| `UI_INSPECT_SKILLS` | ui | 기술 | Skills | inspector |
| `UI_INSPECT_SKILL_FMT` | ui | {skill} {level} · {tier} | {skill} {level} · {tier} | inspector |
| `UI_INSPECT_TOOL` | ui | 도구 | Tool | inspector |
| `UI_INSPECT_TOOL_FMT` | ui | {item} · 품질 {quality} | {item} · quality {quality} | inspector |
| `UI_INSPECT_NOTHING` | ui | 없음 | none | inspector |
| `UI_JOB_QUARRY` | ui | 돌 캐는 중 | Quarrying | inspector job line |
| `UI_JOB_SUPPLY` | ui | 작업장에 재료 가져가는 중 | Supplying a workshop | inspector job line |
| `UI_JOB_CRAFT` | ui | 만드는 중 | Crafting | inspector job line |
| `UI_JOB_GATHER` | ui | 땔감 줍는 중 | Gathering deadwood | inspector job line |

## Changed keys (5; text widened for M2)

| key | file | ko | en | used by |
|---|---|---|---|---|
| `UI_JOB_HAUL` | ui | 물건 나르는 중 | Hauling | inspector job line |
| `UI_JOB_DELIVER` | ui | 청사진에 재료 가져가는 중 | Bringing materials to a blueprint | inspector job line |
| `UI_JOB_BUILD` | ui | 짓는 중 | Building | inspector job line |
| `UI_ORDER_HINT_IDLE` | ui | F1–F4·F6·F7 명령 · P 우선순위 · Space 멈춤 · Home/H 캠프로 | F1–F4·F6·F7 orders · P priorities · Space pause · Home/H camp | order bar idle hint |
| `UI_ORDER_HINT_IDLE_MAC` | ui | fn + F1–F4·F6·F7 명령 · P 우선순위 · Space 멈춤 · Home/H 캠프로 | fn + F1–F4·F6·F7 orders · P priorities · Space pause · Home/H camp | order bar idle hint |
