# M5 localization keys

Every key the M5 UI (Part 5, goal §3.5 and §10) adds or rewrites, with its Korean and English text. Source: `localization/{ko,en}/ui.json`, compiled by `python3 tools/localization_compile.py --project-root .`.
Formats use `Locale.trf` placeholders (`{name}`). Korean is the owner-facing language; reasons end in the noun form like the existing `UI_REJECT_*` texts.

Parameters: `{person}`, `{name}`, `{a}`, `{b}`, `{other}` are person display names (`person_names.gd`; in the chronicle a person link); as in M4 the Korean subject particle after a name is written 가. `{pressure}` is one `M5_PRESSURE_*`, `{cause}` one `M5_QUARREL_*`, `{mods}` up to three `M5_MOD_*` words joined by ", ", `{band}` one `M5_MOOD_*`, `{top}` one `M5_TOP_*`, `{order}` one of `M5_DRAFT_HOLD` / `_MOVE_FMT` / `_ATTACK_FMT`, `{target}` an `M5_ANIMAL_*` word or a person name, `{weapon}` an item name or `M5_WEAPON_BARE`, `{animal}` and `{state}` the `M5_ANIMAL_*` words. `{percent}` is −(1000 − `top_cold_permille`) / 10; `{day}` is get_story `days_of`; `{wood}` and `{need}` are `firewood` and `firewood_day_need`; `{hours}` the illness's hours left; `{hp}` hit points 0–100.

Word choices: the pressures 한파 / 가뭄 / 늑대 무리 / 병; the mood bands 우울 / 보통 / 흡족 / 무너짐 (a breakdown); the tops 허리 두르개 (no top: the art's loincloth) / 가죽 랩 / 털 망토; drafting is 소집, releasing 소집 해제; the quarrel causes read as reasons (배고파서, 추워서, 밖에서 자서, 슬퍼서, 시련 때문에, 사이가 나빠서), the modifier words as states (배고픔, 춥게 잠 …); code 100 (the relation itself) is "사이가 나빠서" both as a modifier and as a quarrel cause.

## New keys (128)

| key | file | ko | en | used by |
|---|---|---|---|---|
| `M5_ITEM_MEAT` | ui | 고기 | Meat | item names (overlay hover, workshop, inspector carry, HUD) |
| `M5_ITEM_CURED_HIDE` | ui | 손질한 가죽 | Cured hide | item names |
| `M5_ITEM_HIDE_WRAP` | ui | 가죽 랩 | Hide wrap | item names |
| `M5_ITEM_FUR_CLOAK` | ui | 털 망토 | Fur cloak | item names |
| `M5_ITEM_SPEAR` | ui | 창 | Spear | item names, drafted list (weapon) |
| `M5_ITEM_WOLF_HIDE` | ui | 늑대 가죽 | Wolf hide | item names |
| `M5_RECIPE_HAFT_SPEAR` | ui | 창 만들기 | Haft a spear | workshop panel (recipe 7) |
| `M5_RECIPE_SCRAPE_HIDE` | ui | 가죽 손질 | Scrape a hide | workshop panel (recipe 8) |
| `M5_RECIPE_MAKE_HIDE_WRAP` | ui | 가죽 랩 만들기 | Make a hide wrap | workshop panel (recipe 9) |
| `M5_RECIPE_MAKE_FUR_CLOAK` | ui | 털 망토 만들기 | Make a fur cloak | workshop panel (recipe 10) |
| `M5_WORK_HUNT` | ui | 사냥 | Hunt | priority table column (work type hunt) |
| `M5_SKILL_HUNTING` | ui | 사냥 솜씨 | Hunting | inspector skills (skill hunting) |
| `M5_REJECT_NOT_DRAFTED` | ui | 소집되지 않은 사람 | not drafted | order toasts (reason not_drafted) |
| `M5_JOB_HUNT` | ui | 사냥 | Hunting | inspector job line (job 15) |
| `M5_JOB_EQUIP` | ui | 옷 입기 | Getting dressed | inspector job line (job 16) |
| `M5_ORDER_HUNT` | ui | 사냥 (사슴) | Hunt (deer) | F7 submenu 3; tech map gate (designation hunt) |
| `M5_ORDER_HUNT_ZONE` | ui | 사냥 구역 | Hunt zone | F7 submenu 4 |
| `M5_HINT_HUNT` | ui | 사슴을 클릭해 사냥 표시 · Esc 끝 | Click a deer to mark it for hunting · Esc ends | order bar hint (HUNT mode) |
| `M5_HINT_HUNT_ZONE` | ui | 끌어서 사냥 구역 정하기 · Esc 끝 | Drag to set a hunt zone · Esc ends | order bar hint (HUNT_ZONE mode) |
| `M5_TOAST_HUNT_MARK` | ui | 사슴 사냥 표시 | Deer marked for hunting | toast (cmd_designate_hunt accepted) |
| `M5_TOAST_HUNT_ZONE` | ui | 사냥 구역을 정함 | Hunt zone set | toast (cmd_add_hunt_zone accepted) |
| `M5_TOAST_NO_DEER` | ui | 거기에 사슴이 없음 | No deer there | toast (HUNT click off a deer) |
| `M5_TOAST_DRAFTED_FMT` | ui | {name} 소집 | {name} drafted | toast (D) |
| `M5_TOAST_UNDRAFTED_FMT` | ui | {name} 소집 해제 | {name} released | toast (D on a drafted person) |
| `M5_TOAST_MOVE_FMT` | ui | {count}명 이동 | {count} moving | toast (right click on a tile) |
| `M5_TOAST_ATTACK_FMT` | ui | {count}명 공격: {target} | {count} attacking: {target} | toast (right click on an animal or a person) |
| `M5_TOAST_UNDRAFT_ALL_FMT` | ui | {count}명 소집 해제 | {count} released | toast (모두 해제) |
| `M5_TOAST_SELF_UNDRAFT_FMT` | ui | {person}: 너무 지쳐 소집에서 빠짐 | {person} left the draft, worn out | toast (Drafted, cause 2) |
| `M5_DRAFT_TITLE_FMT` | ui | 소집 {count}명 | Drafted: {count} | drafted list title |
| `M5_DRAFT_ROW_FMT` | ui | {name} · 몸 {hp} · {order} · {weapon} | {name} · hp {hp} · {order} · {weapon} | drafted list row |
| `M5_DRAFT_UNDRAFT_ALL` | ui | 모두 해제 | Release all | drafted list button (cmd_undraft_all) |
| `M5_DRAFT_HOLD` | ui | 대기 | holding | draft order 0 (drafted list, inspector) |
| `M5_DRAFT_MOVE_FMT` | ui | ({x}, {y})로 이동 | moving to ({x}, {y}) | draft order 1 |
| `M5_DRAFT_ATTACK_FMT` | ui | {target} 공격 | attacking {target} | draft orders 2 and 3 |
| `M5_WEAPON_BARE` | ui | 맨손 | bare hands | drafted list (no weapon) |
| `M5_INSPECT_DRAFT_FMT` | ui | 소집됨 · {order} | Drafted · {order} | inspector draft line |
| `M5_ANIMAL_WOLF` | ui | 늑대 | Wolf | animal hover, toasts, draft orders |
| `M5_ANIMAL_DEER` | ui | 사슴 | Deer | animal hover, toasts, draft orders |
| `M5_ANIMAL_ANY` | ui | 짐승 | an animal | draft order 2 when the animal is gone |
| `M5_ANIMAL_WANDER` | ui | 돌아다님 | wandering | animal hover (state 0) |
| `M5_ANIMAL_FLEE` | ui | 달아남 | fleeing | animal hover (state 1) |
| `M5_ANIMAL_REST` | ui | 쉼 | resting | animal hover (state 2) |
| `M5_ANIMAL_HUNT` | ui | 사냥감을 쫓음 | hunting | animal hover (state 3) |
| `M5_ANIMAL_ATTACK` | ui | 공격 중 | attacking | animal hover (state 4) |
| `M5_ANIMAL_RETREAT` | ui | 물러남 | retreating | animal hover (state 5) |
| `M5_ANIMAL_HOVER_FMT` | ui | {animal} · {state} · {hp} | {animal} · {state} · {hp} | animal hover label |
| `M5_PRESSURE_COLD_SNAP` | ui | 한파 | Cold snap | chronicle, why? chain |
| `M5_PRESSURE_DROUGHT` | ui | 가뭄 | Drought | chronicle, why? chain |
| `M5_PRESSURE_BEASTS` | ui | 늑대 무리 | Wolf pack | chronicle, why? chain |
| `M5_PRESSURE_ILLNESS` | ui | 병 | Illness | chronicle, why? chain |
| `M5_BANNER_WOLVES_FMT` | ui | 늑대 무리가 다가온다 ({count}) | A wolf pack is coming ({count}) | HUD banner (wolves on the map) |
| `M5_BANNER_COLD_FMT` | ui | 한파 {day}일째 · 장작 {wood} (하루 {need} 필요) | Cold snap, day {day} · firewood {wood} ({need} a day needed) | HUD banner (cold snap) |
| `M5_BANNER_DROUGHT_FMT` | ui | 가뭄 {day}일째 | Drought, day {day} | HUD banner (drought) |
| `M5_BANNER_ILLNESS_FMT` | ui | 병 {day}일째 · 아픈 사람 {count} | Illness, day {day} · {count} sick | HUD banner (illness) |
| `M5_MOOD_NONE` | ui | 없음 | none | mood band 0 |
| `M5_MOOD_LOW` | ui | 우울 | low | mood band 1 |
| `M5_MOOD_NORMAL` | ui | 보통 | even | mood band 2 |
| `M5_MOOD_CONTENT` | ui | 흡족 | content | mood band 3 |
| `M5_MOOD_BREAKDOWN` | ui | 무너짐 | breaking down | mood band 4 |
| `M5_MOOD_LEGEND` | ui | 기분 (M): 파랑 우울 · 노랑 흡족 · 빨강 무너짐 | Mood (M): blue low · yellow content · red breaking down | mood overlay legend |
| `M5_INSPECT_MOOD_FMT` | ui | 기분: {band} ({value}) — {mods} | Mood: {band} ({value}) — {mods} | inspector mood line |
| `M5_INSPECT_MOOD_PLAIN_FMT` | ui | 기분: {band} ({value}) | Mood: {band} ({value}) | inspector mood line (no modifier) |
| `M5_MOD_SLEPT_ROOFED` | ui | 지붕 아래서 잠 | slept under a roof | mood modifier 0 |
| `M5_MOD_SLEPT_OUTSIDE` | ui | 밖에서 잠 | slept outside | mood modifier 1 |
| `M5_MOD_FED` | ui | 배부름 | fed | mood modifier 2 |
| `M5_MOD_HUNGRY` | ui | 배고픔 | hungry | mood modifier 3 |
| `M5_MOD_THIRSTY` | ui | 목마름 | thirsty | mood modifier 4 |
| `M5_MOD_COLD` | ui | 춥게 잠 | cold night | mood modifier 5 |
| `M5_MOD_QUARRELLED` | ui | 다툼 | quarrelled | mood modifier 6 |
| `M5_MOD_LEARNED` | ui | 배움 | learned | mood modifier 7 |
| `M5_MOD_DISCOVERED` | ui | 발견 | discovered | mood modifier 8 |
| `M5_MOD_GRIEF` | ui | 가족을 잃음 | grief | mood modifier 9 |
| `M5_MOD_SICK` | ui | 아픔 | sick | mood modifier 10 |
| `M5_MOD_CRAFTED` | ui | 솜씨를 부림 | crafted | mood modifier 11 |
| `M5_MOD_DRAFTED` | ui | 소집됨 | drafted | mood modifier 12 |
| `M5_MOD_PRESSURE` | ui | 시련 | hard times | mood modifier 13 |
| `M5_MOD_RELATION` | ui | 사이가 나빠서 | bad blood | why? chain (code 100), modifier name relation |
| `M5_TOP_LOINCLOTH` | ui | 허리 두르개 | loincloth | inspector apparel line (top 0) |
| `M5_TOP_HIDE_WRAP` | ui | 가죽 랩 | hide wrap | inspector apparel line (top 1) |
| `M5_TOP_FUR_CLOAK` | ui | 털 망토 | fur cloak | inspector apparel line (top 2) |
| `M5_INSPECT_TOP_FMT` | ui | 옷: {top} (추위 −{percent} %) | Wears: {top} (cold −{percent} %) | inspector apparel line |
| `M5_INSPECT_TOP_PLAIN_FMT` | ui | 옷: {top} | Wears: {top} | inspector apparel line (loincloth) |
| `M5_DRESS_ORDER` | ui | 모두 가죽 옷 | Hides for all | priority table header toggle |
| `M5_TOAST_DRESS_ON` | ui | 모두 가죽 옷: 켬 | Hides for all: on | toast (cmd_set_dress_order on) |
| `M5_TOAST_DRESS_OFF` | ui | 모두 가죽 옷: 끔 | Hides for all: off | toast (cmd_set_dress_order off) |
| `M5_INSPECT_SICK_FMT` | ui | 아픔 · {hours}시간 남음 | Sick · {hours} h left | inspector illness line |
| `M5_INSPECT_HP_FMT` | ui | 몸 {hp}/100 | Health {hp}/100 | inspector hp line (below 100) |
| `M5_INSPECT_TAB_STORY` | ui | 이야기 | Story | inspector tab |
| `M5_INSPECT_STORY_NONE` | ui | 아직 이야기가 없음 | No story yet | inspector 이야기 tab (empty) |
| `M5_CHRON_BEGAN_COLD` | ui | 한파가 닥쳤다 | A cold snap set in | chronicle (PressureBegan, cold snap) |
| `M5_CHRON_BEGAN_DROUGHT` | ui | 가뭄이 들었다 | A drought began | chronicle (PressureBegan, drought) |
| `M5_CHRON_BEGAN_BEASTS_FMT` | ui | 늑대 {count}마리가 나타났다 | {count} wolves appeared | chronicle (PressureBegan, beasts) |
| `M5_CHRON_BEGAN_ILLNESS_FMT` | ui | {person}가 앓기 시작하며 병이 돌았다 | Illness came, starting with {person} | chronicle (PressureBegan, illness with a subject) |
| `M5_CHRON_BEGAN_ILLNESS` | ui | 병이 돌기 시작했다 | Illness came | chronicle (PressureBegan, illness) |
| `M5_CHRON_ENDED_FMT` | ui | {pressure} 지나갔다 · 죽은 사람 {deaths} | {pressure} passed · {deaths} dead | chronicle (PressureEnded; Korean appends 이/가 to the pressure) |
| `M5_CHRON_HUNTED_DEER_FMT` | ui | {person}가 사슴을 잡았다 (고기 {meat}) | {person} killed a deer ({meat} meat) | chronicle (Hunted, deer) |
| `M5_CHRON_HUNTED_WOLF_FMT` | ui | {person}가 늑대를 잡았다 | {person} killed a wolf | chronicle (Hunted, wolf) |
| `M5_CHRON_QUARREL_FMT` | ui | {a}가 {b}와 다퉜다 — {cause} | {a} quarrelled with {b} — {cause} | chronicle (M5 Quarrel) |
| `M5_CHRON_FIGHT_FMT` | ui | {a}가 {b}와 주먹다짐을 했다 — {cause} | {a} came to blows with {b} — {cause} | chronicle (Fight) |
| `M5_CHRON_BREAKDOWN_FMT` | ui | {person}가 무너졌다 — {mods} | {person} broke down — {mods} | chronicle (Breakdown) |
| `M5_CHRON_BREAKDOWN_PLAIN_FMT` | ui | {person}가 무너졌다 | {person} broke down | chronicle (Breakdown, no modifier) |
| `M5_CHRON_DRAFTED_FMT` | ui | {person}가 소집되었다 | {person} was drafted | chronicle (Drafted, cause 0) |
| `M5_CHRON_UNDRAFTED_FMT` | ui | {person}가 소집에서 풀려났다 | {person} was released from the draft | chronicle (Drafted, cause 1) |
| `M5_CHRON_SELF_UNDRAFTED_FMT` | ui | {person}가 너무 지쳐 소집에서 빠졌다 | {person} left the draft, worn out | chronicle (Drafted, cause 2) |
| `M5_CHRON_WOLF_KILLED_FMT` | ui | {person}가 늑대를 쓰러뜨렸다 (남은 늑대 {left}) | {person} brought down a wolf ({left} left) | chronicle (WolfKilled) |
| `M5_CHRON_WOLF_DIED_FMT` | ui | 늑대가 쓰러졌다 (남은 늑대 {left}) | A wolf fell ({left} left) | chronicle (WolfKilled, no person) |
| `M5_CHRON_SICK_FMT` | ui | {person}가 병에 걸렸다 | {person} fell ill | chronicle (Sick) |
| `M5_CHRON_SICK_FROM_FMT` | ui | {person}가 {other}에게서 병이 옮았다 | {person} caught the illness from {other} | chronicle (Sick, passed on) |
| `M5_CHRON_RECOVERED_FMT` | ui | {person}가 병에서 나았다 | {person} recovered | chronicle (Recovered) |
| `M5_QUARREL_HUNGER` | ui | 배고파서 | hunger | quarrel cause 0 |
| `M5_QUARREL_COLD` | ui | 추워서 | the cold | quarrel cause 1 |
| `M5_QUARREL_SLEEP` | ui | 밖에서 자서 | a night outside | quarrel cause 2 |
| `M5_QUARREL_GRIEF` | ui | 슬퍼서 | grief | quarrel cause 3 |
| `M5_QUARREL_PRESSURE` | ui | 시련 때문에 | hard times | quarrel cause 4 |
| `M5_QUARREL_RELATION` | ui | 사이가 나빠서 | bad blood | quarrel cause 5 |
| `M5_CHRON_WHY` | ui | → 왜? | → why? | chronicle and 이야기 tab link |
| `M5_WHY_TITLE` | ui | 왜? | Why? | why? panel title |
| `M5_WHY_MODIFIER_FMT` | ui | {name} | {name} | why? chain (a mood modifier) |
| `M5_WHY_PRESSURE_FMT` | ui | {name} 때문 | because of the {name} | why? chain (a pressure) |
| `M5_WHY_NONE` | ui | 더 거슬러 올라갈 원인이 없음 | No earlier cause | why? panel (a chain of one) |
| `M5_FILTER_ALL` | ui | 전체 | All | chronicle filter |
| `M5_FILTER_PRESSURE` | ui | 압력 | Pressure | chronicle filter |
| `M5_FILTER_HUNT` | ui | 사냥·늑대 | Hunts · wolves | chronicle filter |
| `M5_FILTER_MOOD` | ui | 다툼·기분 | Quarrels · mood | chronicle filter |
| `M5_FILTER_KNOWLEDGE` | ui | 지식 | Knowledge | chronicle filter |
| `M5_FILTER_LIFE` | ui | 삶 | Life | chronicle filter |
| `M5_FILTER_PERSON` | ui | 고른 사람만 | Selected person | chronicle filter toggle |
| `M5_YEAR_LINE_FMT` | ui | {year}년 차 정리 · 출생 {births} · 사망 {deaths} · 압력 {pressures} · 발견 {discoveries} · 다툼 {quarrels} · 사슴 {hunted} · 늑대 {wolves} | Year {year} · births {births} · deaths {deaths} · pressures {pressures} · discoveries {discoveries} · quarrels {quarrels} · deer {hunted} · wolves {wolves} | chronicle year summary |

## Changed keys (3; text widened for M5)

| key | file | ko (was) | ko | en | used by |
|---|---|---|---|---|---|
| `UI_ORDER_HINT_IDLE` | ui | F1–F4·F6–F8·F10 명령 · R 방 · P 우선순위 · T 기술 · K 가르치기 · Space 멈춤 · Home/H 캠프로 | F1–F4·F6–F8·F10 명령 · R 방 · P 우선순위 · T 기술 · K 가르치기 · D 소집 · M 기분 · Space 멈춤 · Home/H 캠프로 | F1–F4·F6–F8·F10 orders · R rooms · P priorities · T tech · K teach · D draft · M mood · Space pause · Home/H camp | order bar idle hint (D draft, M mood) |
| `UI_ORDER_HINT_IDLE_MAC` | ui | fn + F1–F4·F6–F8·F10 명령 · R 방 · P 우선순위 · T 기술 · K 가르치기 · Space 멈춤 · Home/H 캠프로 | fn + F1–F4·F6–F8·F10 명령 · R 방 · P 우선순위 · T 기술 · K 가르치기 · D 소집 · M 기분 · Space 멈춤 · Home/H 캠프로 | fn + F1–F4·F6–F8·F10 orders · R rooms · P priorities · T tech · K teach · D draft · M mood · Space pause · Home/H camp | order bar idle hint (macOS) |
| `UI_ORDER_HINT_DIG_MENU` | ui | 1 채석 · 2 점토 캐기 · 끌어서 표시 | 1 채석 · 2 점토 캐기 · 3 사냥 · 4 사냥 구역 | 1 quarry · 2 clay pit · 3 hunt · 4 hunt zone | order bar hint (F7 submenu of four) |
