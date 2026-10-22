# M3 localization keys

Every key the M3 UI (Part 4) adds or rewrites, with its Korean and English text. Sources: `localization/{ko,en}/{ui,buildings}.json`, compiled by `python3 tools/localization_compile.py --project-root .`.
Formats use `Locale.trf` placeholders (`{name}`). Korean is the owner-facing language; refusal reasons end in the noun form (`~음`) like the existing `UI_REJECT_*` texts. Existing keys are reused where they fit: `UI_REJECT_BLOCKED` / `_WATER` / `_OUT_OF_MAP` / `_NOTHING` / `_NOT_YET` / `_UNKNOWN` (refusals), `UI_ORDER_HINT_AREA` (dig and deconstruct drags), `UI_ORDER_HINT_PLACE_FMT` (door and bedroll: one click), `UI_BLUEPRINT_MATERIALS` / `UI_BLUEPRINT_PROGRESS_FMT` (site labels), `UI_CLOSE` (room panel), `QUALITY_BAND_*` / `UI_QUALITY_FMT` (quality words, added in Part 0).

Parameters: `{role}` is a `ROOM_ROLE_*` text; `{roof}` is `UI_ROOM_ROOFED` or `UI_ROOM_OPEN` (or `UI_ROOM_DUG` for a dug pocket); `{effect}` is one `UI_ROOM_EFFECT_*` text (or several joined with " · "); `UI_INSPECT_BED_FMT`'s `{room}` is the room's role text and `{id}` its number; `UI_ORDER_HINT_DRAG_FMT`'s `{line}` / `{rect}` are `UI_HINT_DRAG_LINE` and `UI_HINT_DRAG_RECT` (walls) or `UI_HINT_DRAG_RECT_FILLED` (floors).

## New keys (61)

| key | file | ko | en | used by |
|---|---|---|---|---|
| `BUILDING_LOG_WALL` | buildings | 통나무 벽 | Log wall | build menu 5, place hint, hover |
| `BUILDING_DOOR` | buildings | 문 | Door | build menu 6, place hint, hover |
| `BUILDING_PACKED_FLOOR` | buildings | 다진 흙바닥 | Packed floor | build menu 7, place hint, hover |
| `BUILDING_BEDROLL` | buildings | 침상 | Bedroll | build menu 8, place hint, hover |
| `BUILDING_STONE_WALL` | buildings | 돌담 | Stone wall | build menu 9 (if 7.5 kept), place hint, hover |
| `UI_ORDER_DIG` | ui | 파기 | Dig | order bar F8 |
| `UI_ORDER_DECONSTRUCT` | ui | 철거 | Deconstruct | order bar F10 |
| `UI_HINT_DRAG_LINE` | ui | 드래그: 줄 | Drag: line | order bar hint (walls, floors) |
| `UI_HINT_DRAG_RECT` | ui | Shift+드래그: 네모(속 빔) | Shift+drag: rectangle (hollow) | order bar hint (walls) |
| `UI_HINT_DRAG_RECT_FILLED` | ui | Shift+드래그: 네모(꽉 채움) | Shift+drag: rectangle (filled) | order bar hint (floors) |
| `UI_ORDER_HINT_DRAG_FMT` | ui | {name} · {line} · {rect} | {name} · {line} · {rect} | order bar hint while placing a wall/floor (line, rect = the two hints above) |
| `UI_ORDER_TOAST_DIG` | ui | {count}칸 파기 표시 | {count} tiles marked for digging | order toast |
| `UI_ORDER_TOAST_DECONSTRUCT` | ui | {count}칸 철거 표시 | {count} tiles marked for deconstruction | order toast |
| `UI_ORDER_TOAST_PLACE_MANY_FMT` | ui | {name} 청사진 {count}칸 놓음 | {count} {name} blueprints placed | order toast (line/rect drag) |
| `UI_ORDER_TOAST_REFUSED_FMT` | ui | {count}칸은 안 됨 — {reason} | {count} tiles refused — {reason} | order toast (red ghost tiles) |
| `UI_ORDER_TOAST_ROOM_ROLE` | ui | 방 역할을 바꿈 | Room role changed | room panel toast |
| `UI_ROOMS_TOGGLE` | ui | 방 보기 R | Rooms R | order bar / room overlay toggle |
| `ROOM_ROLE_BEDROOM` | ui | 침실 | Bedroom | room overlay, room panel, inspector, chronicle |
| `ROOM_ROLE_STOREHOUSE` | ui | 창고 | Storehouse | room overlay, room panel, inspector, chronicle |
| `ROOM_ROLE_WORKSHOP` | ui | 작업장 | Workshop | room overlay, room panel, inspector, chronicle |
| `ROOM_ROLE_HALL` | ui | 홀 | Hall | room overlay, room panel, inspector, chronicle |
| `ROOM_ROLE_UNASSIGNED` | ui | 미정 | Unassigned | room overlay, room panel, inspector, chronicle |
| `ROOM_ROLE_AUTO` | ui | 자동 | Auto | room panel role dropdown (clear the override) |
| `UI_ROOM_LABEL_FMT` | ui | {role} · {tiles}칸 · 침대 {beds} | {role} · {tiles} tiles · {beds} beds | room overlay label (beds > 0) |
| `UI_ROOM_LABEL_NOBED_FMT` | ui | {role} · {tiles}칸 | {role} · {tiles} tiles | room overlay label (no beds) |
| `UI_ROOM_NAME_FMT` | ui | {role} {id} | {role} {id} | room panel heading |
| `UI_ROOM_PANEL_TITLE` | ui | 방 | Room | room panel |
| `UI_ROOM_ROLE` | ui | 역할 | Role | room panel |
| `UI_ROOM_TILES_FMT` | ui | {tiles}칸 | {tiles} tiles | room panel |
| `UI_ROOM_BEDS_FMT` | ui | 침대 {beds} | {beds} beds | room panel |
| `UI_ROOM_DOORS_FMT` | ui | 문 {doors} | {doors} doors | room panel |
| `UI_ROOM_NO_DOOR` | ui | 문이 없어 들어갈 수 없음 | No door: nobody can get in | room panel (enclosed, unreachable) |
| `UI_ROOM_COVERED_FMT` | ui | 지붕 아래 물건 {count}개 | {count} items under the roof | room panel |
| `UI_ROOM_ROOFED` | ui | 지붕 있음 | roofed | room panel, why line, inspector {roof} |
| `UI_ROOM_OPEN` | ui | 지붕 없음 | open | room panel, why line, inspector {roof} |
| `UI_ROOM_DUG` | ui | 판 굴 | dug pocket | room panel, why line |
| `UI_ROOM_OVERRIDE` | ui | (역할 고정) | (role fixed) | room panel, room overlay label |
| `UI_ROOM_WHY_FMT` | ui | {role} · {roof} · 침대 {beds} · {effect} | {role} · {roof} · {beds} beds · {effect} | causal (why?) panel on a room tile |
| `UI_ROOM_EFFECT_COLD` | ui | 추위 보호 | shelter from the cold | why line (roofed bedroom) |
| `UI_ROOM_EFFECT_SLEEP` | ui | 잠이 빨리 회복 | faster sleep | why line (roofed bedroom) |
| `UI_ROOM_EFFECT_STORE` | ui | 물건 보관 | keeps goods covered | why line (storehouse) |
| `UI_ROOM_EFFECT_CRAFT` | ui | 작업이 빨라짐 | faster crafting | why line (workshop room, 8.5) |
| `UI_ROOM_EFFECT_GATHER` | ui | 저녁 모임 자리 | evening gathering place | why line (hall) |
| `UI_ROOM_EFFECT_NONE` | ui | 효과 없음 | no effect | why line (unassigned or open) |
| `UI_ITEM_COVERED` | ui | 지붕 아래 | under a roof | overlay hover on an item stack in a storehouse |
| `UI_INSPECT_BED_FMT` | ui | 잠자리: {room} {id} ({roof}) | Bed: {room} {id} ({roof}) | inspector (person) |
| `UI_INSPECT_BED_HUT` | ui | 잠자리: 오두막 | Bed: hut | inspector (person with a hut bed) |
| `UI_INSPECT_BED_NONE` | ui | 잠자리: 없음 | Bed: none | inspector (person) |
| `UI_INSPECT_ROOM_FMT` | ui | 방: {role} {id} | Room: {role} {id} | inspector / hover (item in a room) |
| `UI_REJECT_PERSON` | ui | 사람이 서 있음 | Someone is standing there | order toast |
| `UI_REJECT_HILL` | ui | 언덕은 먼저 파야 함 | Dig the hill first | order toast |
| `UI_REJECT_EDGE` | ui | 지도 가장자리 | At the map edge | order toast |
| `UI_REJECT_BUILT` | ui | 이미 지어져 있음 | Already built | order toast |
| `UI_REJECT_NOT_HILL` | ui | 언덕이 아님 | Not a hill | order toast (dig) |
| `UI_REJECT_UNREACHABLE` | ui | 닿을 수 없음 | Out of reach | order toast (dig without a passable neighbour) |
| `UI_JOB_DIG` | ui | 파는 중 | Digging | inspector job line |
| `UI_JOB_DECONSTRUCT` | ui | 철거하는 중 | Deconstructing | inspector job line |
| `UI_JOB_CLEAR` | ui | 치우는 중 | Clearing a site | inspector job line (haul-away) |
| `CHRONICLE_ROOM_FORMED` | ui | 새 방이 생김 — {role} | A new room: {role} | chronicle (RoomFormed) |
| `WORK_MINE` | ui | 채굴 | Mine | priority table |
| `SKILL_MINING` | ui | 채굴 | Mining | inspector skills |

## Changed keys (3; text widened for M3)

| key | file | ko (was) | ko | en | used by |
|---|---|---|---|---|---|
| `UI_ORDER_HINT_BUILD_MENU` | ui | 1–4로 고르세요 · F3 닫기 | 1–9로 고르세요 · F3 닫기 | Pick with 1–9 · F3 closes | order bar hint (build menu grew to 9 entries) |
| `UI_ORDER_HINT_IDLE` | ui | F1–F4·F6·F7 명령 · P 우선순위 · Space 멈춤 · Home/H 캠프로 | F1–F4·F6–F8·F10 명령 · R 방 · P 우선순위 · Space 멈춤 · Home/H 캠프로 | F1–F4·F6–F8·F10 orders · R rooms · P priorities · Space pause · Home/H camp | order bar idle hint (F8 dig, F10 deconstruct, R rooms) |
| `UI_ORDER_HINT_IDLE_MAC` | ui | fn + F1–F4·F6·F7 명령 · P 우선순위 · Space 멈춤 · Home/H 캠프로 | fn + F1–F4·F6–F8·F10 명령 · R 방 · P 우선순위 · Space 멈춤 · Home/H 캠프로 | fn + F1–F4·F6–F8·F10 orders · R rooms · P priorities · Space pause · Home/H camp | order bar idle hint (macOS) |

The two idle hints name F8 (파기), F10 (철거) and R (방 보기) before the GDScript binds them; Part 4 binds them (goal 9.2, 9.3) and README's key table follows.

## Keys added in Part 4 (2; UI wiring)

| key | file | ko | en | used by |
|---|---|---|---|---|
| `UI_ROOM_SLEEPERS_FMT` | ui | 자는 사람 {count} | {count} asleep | room panel (`get_room_detail` `sleepers`) |
| `UI_INSPECT_BED_TILE_FMT` | ui | 잠자리: 방 밖 ({x}, {y}) | Bed: outside a room ({x}, {y}) | inspector (a bed that is in no room and not a hut's, e.g. a bedroll in the open) |

Part 4 also binds the keys above: F8 파기, F10 철거, R 방 보기 (the order bar's "방 보기 R" button), the build menu's 1–9.
