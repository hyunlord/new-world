# M4 localization keys

Every key the M4 UI (Part 4, goal §9) adds or rewrites, with its Korean and English text. Sources: `localization/{ko,en}/{ui,buildings}.json`, compiled by `python3 tools/localization_compile.py --project-root .`.
Formats use `Locale.trf` placeholders (`{name}`). Korean is the owner-facing language; refusal reasons end in the noun form (`~음`) like the existing `UI_REJECT_*` texts. Node names are the `name_key`s of `rust/crates/sim-core/data/colony/knowledge.ron` (`scripts/ui/knowledge_text.gd` `NODE_KEYS` mirrors them).

Parameters: `{node}` is a node name; in Korean the UI appends the object particle 을/를 (chosen from the last Hangul syllable, `KnowledgeText.with_object`) where a key's Korean text has `{node}` right before a verb (`M4_WAIT_FMT`, `M4_COND_KNOWS_FMT`, `CHRONICLE_KNOWLEDGE_TAUGHT_FMT`, `_LEARNED_FMT`, `_DISCOVERED_FMT`), and the subject particle 이/가 to `{item}` in `CHRONICLE_TOOL_BROKE_FMT`; English gets the bare name. `{person}`, `{name}`, `{teacher}`, `{learner}`, `{partner}` are person display names (`person_names.gd`; in the chronicle a person link). `{percent}`, `{goal}`, `{threshold}` are whole percent (‰ / 10). `{era}` is one `M4_ERA_*` label; `{state}` one `M4_ENTRY_*`, `{source}` and `{channel}` one `M4_SOURCE_*`, `{status}` one `M4_STATUS_*`, `{lane}` one `M4_LANE_*`, `{when}` `M4_USED_TODAY` or `M4_DAYS_AGO_FMT`.

Word choices: sources 전승 (seeded: the founders' inherited craft), 구전 (oral), 관찰 (observed), 발견 (discovered), 도제 (apprenticed); entry states 숙련 / 배우는 중 / 잊어 감; statuses 미지 / 위험 / 알려짐 / 안정 / 소실; the era labels 구석기 / 신석기 전환기 / 신석기 시대.

## New keys (147)

| key | file | ko | en | used by |
|---|---|---|---|---|
| `KNOWLEDGE_FIRE_KEEPING` | ui | 불 다루기 | Fire keeping | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_FORAGING` | ui | 채집 | Foraging | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_WATER_FINDING` | ui | 물 찾기 | Water finding | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_FLINT_KNAPPING` | ui | 돌 깨기(타제) | Flint knapping | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_HAFTING` | ui | 자루 달기 | Hafting | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_FIREWOOD_SPLITTING` | ui | 장작 패기 | Firewood splitting | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_PRESSURE_FLAKING` | ui | 눌러떼기 | Pressure flaking | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_STONE_GRINDING` | ui | 간석기 | Stone grinding | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_HUT_BUILDING` | ui | 움집 짓기 | Hut building | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_TIMBER_WALL` | ui | 통나무 벽 | Timber walls | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_DOORWAY` | ui | 문 만들기 | Doorways | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_PACKED_FLOOR` | ui | 흙바닥 다지기 | Packed floors | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_BEDDING` | ui | 잠자리 만들기 | Bedding | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_STONE_WALLING` | ui | 돌담 쌓기 | Stone walling | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_STONE_QUARRYING` | ui | 채석 | Stone quarrying | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_DIGGING` | ui | 땅 파기 | Digging | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_CLAY_DIGGING` | ui | 점토 캐기 | Clay digging | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_CLAY_PREP` | ui | 점토 반죽 | Clay preparation | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_POT_FORMING` | ui | 토기 빚기 | Pot forming | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_POT_FIRING` | ui | 토기 굽기 | Pot firing | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_INSTRUCTION` | ui | 가르치기 | Instruction | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_STORYTELLING` | ui | 이야기하기 | Storytelling | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_TRACKING` | ui | 발자국 읽기 | Tracking | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `KNOWLEDGE_HIDE_SCRAPING` | ui | 가죽 손질 | Hide scraping | tech map, teach dialog, inspector 지식, chronicle, HUD |
| `M4_LANE_SURVIVAL` | ui | 생존 | Survival | tech map row, node detail |
| `M4_LANE_CRAFT` | ui | 제작 | Craft | tech map row, node detail |
| `M4_LANE_BUILD` | ui | 건설 | Building | tech map row, node detail |
| `M4_LANE_MINE` | ui | 채굴 | Mining | tech map row, node detail |
| `M4_LANE_POTTERY` | ui | 토기 | Pottery | tech map row, node detail |
| `M4_LANE_SOCIAL` | ui | 사회 | Social | tech map row, node detail |
| `M4_ERA_COL_PALEOLITHIC` | ui | 구석기 | Paleolithic | tech map column, node detail |
| `M4_ERA_COL_NEOLITHIC` | ui | 신석기 | Neolithic | tech map column, node detail |
| `M4_ERA_PALEOLITHIC` | ui | 구석기 | Paleolithic | HUD era line (era 0, progress < 25 %) |
| `M4_ERA_TRANSITION` | ui | 신석기 전환기 | Neolithic transition | HUD era line (era 1, 25–74 %) |
| `M4_ERA_NEOLITHIC` | ui | 신석기 시대 | Neolithic age | HUD era line (era 2, ≥ 75 %) |
| `M4_STATUS_UNKNOWN` | ui | 미지 | Unknown | tech map legend, card tooltip, node detail (status 0) |
| `M4_STATUS_AT_RISK` | ui | 위험 | At risk | tech map legend, card tooltip, node detail (status 1) |
| `M4_STATUS_KNOWN` | ui | 알려짐 | Known | tech map legend, card tooltip, node detail (status 2) |
| `M4_STATUS_STABLE` | ui | 안정 | Stable | tech map legend, card tooltip, node detail (status 3) |
| `M4_STATUS_LOST` | ui | 소실 | Lost | tech map legend, card tooltip, node detail (status 4) |
| `M4_SOURCE_SEEDED` | ui | 전승 | inherited | inspector 지식 entry (source 0: a founder's knowledge) |
| `M4_SOURCE_DISCOVERED` | ui | 발견 | discovered | inspector 지식 entry (source 1) |
| `M4_SOURCE_ORAL` | ui | 구전 | told | inspector 지식 entry (source 2), chronicle KnowledgeLearned channel |
| `M4_SOURCE_OBSERVED` | ui | 관찰 | watched | inspector 지식 entry (source 3), chronicle KnowledgeLearned channel |
| `M4_SOURCE_APPRENTICED` | ui | 도제 | apprenticed | inspector 지식 entry (source 4), chronicle KnowledgeLearned channel |
| `M4_ENTRY_SKILLED` | ui | 숙련 | skilled | inspector 지식 entry (state 0: at threshold) |
| `M4_ENTRY_LEARNING` | ui | 배우는 중 | learning | inspector 지식 entry (state 1: below, used lately) |
| `M4_ENTRY_FORGETTING` | ui | 잊어 감 | fading | inspector 지식 entry (state 2: unused past 6 days) |
| `M4_HUD_ERA_FMT` | ui | {era} {percent} % | {era} {percent} % | HUD era line beside the calendar |
| `M4_HUD_AT_RISK_FMT` | ui | ⚠ 기술 위험 {count} | ⚠ At risk {count} | HUD badge (opens the tech map) |
| `M4_WARN_AT_RISK` | ui | ⚠ {node}: {name}만 앎 | ⚠ {node}: only {name} knows | HUD warning line (first at-risk node and its one holder) |
| `M4_TECH_TITLE` | ui | 기술 지도 | Tech map | tech map title |
| `M4_TECH_HINT` | ui | 칸을 누르면 자세히 · T 닫기 | Click a card for details · T closes | tech map hint |
| `M4_TECH_MARKER_LEGEND` | ui | 시대 표지 | era marker | tech map legend (◆) |
| `M4_TECH_PRACTITIONERS_FMT` | ui | {count}명 | {count} people | tech map card (practitioners) |
| `M4_TECH_SELECT_HINT` | ui | 기술을 고르면 자세한 내용이 보임 | Pick a technology to see its detail | node detail (nothing chosen) |
| `M4_TECH_STATUS_FMT` | ui | {status} · 기준 {threshold} % · 아는 사람 {holders} · 쓰는 사람 {practitioners} | {status} · threshold {threshold} % · {holders} know it · {practitioners} use it | node detail |
| `M4_TECH_LANE_ERA_FMT` | ui | {lane} · {era} | {lane} · {era} | node detail |
| `M4_TECH_MARKER_FMT` | ui | 시대 표지 ◆{weight} | era marker ◆{weight} | node detail (marker weight > 0) |
| `M4_TECH_HOLDERS` | ui | 아는 사람 | Who knows it | node detail heading |
| `M4_TECH_NO_HOLDERS` | ui | 아무도 모름 | Nobody knows it | node detail |
| `M4_TECH_INSTRUCTOR` | ui | 가르치기 숙련 | skilled teacher | node detail holder, teach dialog teacher (knows instruction) |
| `M4_TECH_LEARNERS` | ui | 배울 수 있는 사람 | Who could learn it | node detail heading |
| `M4_TECH_MORE_FMT` | ui | 외 {count}명 | and {count} more | node detail learner list |
| `M4_TECH_DISCOVERY` | ui | 발견 조건 | Discovery conditions | node detail heading |
| `M4_TECH_NO_DISCOVERY` | ui | 발견 조건 없음 (처음부터 아는 기술) | No discovery (known from the start) | node detail |
| `M4_COND_STOCKPILE_FMT` | ui | {item} {amount}개 이상 쌓아 둠 | {amount} or more {item} stored | node detail condition (kind 0) |
| `M4_COND_SKILL_FMT` | ui | {skill} {amount} 이상인 사람 | someone with {skill} {amount} or more | node detail condition (kind 1) |
| `M4_COND_KNOWS_FMT` | ui | {node} {percent} % 이상 아는 사람 | someone who knows {node} to {percent} % | node detail condition (kind 2) |
| `M4_COND_CLAY_FMT` | ui | 캠프 {amount}칸 안에 점토 둑 | a clay bank within {amount} tiles of the camp | node detail condition (kind 3) |
| `M4_TECH_GATES` | ui | 이 기술이 있어야 하는 일 | What it is needed for | node detail heading |
| `M4_GATE_NONE` | ui | 막는 일 없음 | Nothing waits on it | node detail |
| `M4_GATE_RECIPE_FMT` | ui | 제작: {name} | Craft: {name} | node detail gate (recipe) |
| `M4_GATE_BLUEPRINT_FMT` | ui | 짓기: {name} | Build: {name} | node detail gate (blueprint) |
| `M4_GATE_WORKSHOP_FMT` | ui | 작업장: {name} | Workshop: {name} | node detail gate (workshop) |
| `M4_GATE_DESIGNATION_FMT` | ui | 표시: {name} | Mark: {name} | node detail gate (designation) |
| `M4_GATE_CHANNEL_FMT` | ui | 전수: {name} | Passing on: {name} | node detail gate (channel) |
| `M4_GATE_LORE_FMT` | ui | 생활: {name} | Daily life: {name} | node detail gate (lore) |
| `M4_CHANNEL_APPRENTICESHIP` | ui | 도제 가르치기 | apprenticeship | node detail gate (instruction) |
| `M4_CHANNEL_ORAL` | ui | 저녁 이야기 (+10 %) | evening telling (+10 %) | node detail gate (storytelling) |
| `M4_LORE_HEARTH` | ui | 모닥불 돌보기 | tending the fire | node detail gate (fire keeping) |
| `M4_LORE_FORAGE` | ui | 먹을거리 모으기 | gathering food | node detail gate (foraging) |
| `M4_LORE_WATER` | ui | 물 찾아 마시기 | finding water | node detail gate (water finding) |
| `M4_TECH_WAITING_FMT` | ui | 이 기술을 기다리는 일 {count}개 | {count} orders wait for it | node detail (get_knowledge_waits) |
| `M4_TECH_TEACH` | ui | 가르치기 | Teach | node detail button (opens the teach dialog) |
| `M4_PERCENT_FMT` | ui | {percent} % | {percent} % | proficiency next to a bar (node detail, teach dialog) |
| `M4_TEACH_TITLE` | ui | 가르치기 명령 (K) | Teaching order (K) | teach dialog title |
| `M4_TEACH_KNOWLEDGE` | ui | 기술 | Technology | teach dialog row |
| `M4_TEACH_TEACHER` | ui | 스승 | Teacher | teach dialog row |
| `M4_TEACH_LEARNER` | ui | 제자 | Learner | teach dialog row |
| `M4_TEACH_ORDER` | ui | 명령 | Order | teach dialog button |
| `M4_TEACH_NO_TEACHER` | ui | 가르칠 사람 없음 | Nobody can teach it | teach dialog status, toast |
| `M4_TEACH_NO_LEARNER` | ui | 배울 사람 없음 | Nobody can learn it | teach dialog status, toast, node detail |
| `M4_TEACH_ACTIVE` | ui | 가르치는 중인 일 | Teaching under way | teach dialog heading |
| `M4_TEACH_NONE` | ui | 가르치는 일 없음 | No teaching under way | teach dialog list |
| `M4_TEACH_ROW_FMT` | ui | {teacher} → {learner} · {node} | {teacher} → {learner} · {node} | teach dialog list row |
| `M4_TEACH_PROGRESS_FMT` | ui | {percent} / {goal} % | {percent} / {goal} % | teach dialog list row (learner's proficiency / threshold) |
| `M4_TEACH_SESSIONS_FMT` | ui | 수업 {count}번 | {count} lessons | teach dialog list row |
| `M4_TEACH_IN_SESSION` | ui | 수업 중 | in a lesson | teach dialog list row |
| `M4_TEACH_CANCEL` | ui | 취소 | Cancel | teach dialog list row button |
| `M4_TEACH_TOAST_FMT` | ui | 가르치기: {teacher} → {learner} · {node} | Teaching: {teacher} → {learner} · {node} | order toast (accepted) |
| `M4_TEACH_TOAST_CANCEL` | ui | 가르치기 취소함 | Teaching cancelled | order toast |
| `UI_REJECT_NO_PRACTITIONER` | ui | 그 기술을 쓸 줄 아는 사람이 아님 | Not a practitioner of it | order toast (reason no_practitioner) |
| `UI_REJECT_TOO_YOUNG` | ui | 너무 어림 (10살부터) | Too young (10 years or older) | order toast (reason too_young) |
| `M4_INSPECT_TAB_INFO` | ui | 정보 | Info | inspector tab |
| `M4_INSPECT_TAB_KNOWLEDGE` | ui | 지식 | Knowledge | inspector tab |
| `M4_INSPECT_KNOWLEDGE_NONE` | ui | 아는 기술 없음 | Knows no technology | inspector 지식 tab |
| `M4_INSPECT_ENTRY_FMT` | ui | {node} · {state} ({source}) | {node} · {state} ({source}) | inspector 지식 entry |
| `M4_INSPECT_USE_FMT` | ui | {percent} % · 마지막으로 씀: {when} | {percent} % · last used {when} | inspector 지식 entry (under the bar) |
| `M4_DAYS_AGO_FMT` | ui | {days}일 전 | {days} days ago | inspector 지식 entry (days_unused) |
| `M4_USED_TODAY` | ui | 오늘 | today | inspector 지식 entry (days_unused 0) |
| `M4_LESSON_TEACHING_FMT` | ui | 가르치는 중: {partner}에게 {node} | Teaching {partner}: {node} | inspector 지식 tab (lesson_role 0) |
| `M4_LESSON_LEARNING_FMT` | ui | 배우는 중: {partner}에게서 {node} | Learning from {partner}: {node} | inspector 지식 tab (lesson_role 1) |
| `UI_JOB_DIG_CLAY` | ui | 점토 캐는 중 | Digging clay | inspector job line (job 12) |
| `UI_JOB_TEACH` | ui | 가르치는 중 | Teaching | inspector job line (job 13) |
| `UI_JOB_LEARN` | ui | 배우는 중 | Learning | inspector job line (job 14) |
| `M4_PRIORITY_POTTERY` | ui | 토기 솜씨 | Pottery skill | priority table (read-only Pottery level column) |
| `SKILL_POTTERY` | ui | 토기 | Pottery | inspector skills, discovery conditions |
| `UI_ORDER_CLAY` | ui | 점토 캐기 | Clay pit | F7 submenu 2, node detail gate (clay_pit) |
| `UI_ORDER_TOAST_CLAY` | ui | 점토 둑 {count}곳 캐기 표시 | {count} clay banks marked for digging | order toast (cmd_designate_clay) |
| `UI_ORDER_HINT_CLAY` | ui | 끌어서 점토 둑에 캐기 표시 · Esc 끝 | Drag over clay banks to mark them · Esc ends | order bar hint (clay mode) |
| `UI_ORDER_HINT_DIG_MENU` | ui | 1 채석 · 2 점토 캐기 · 끌어서 표시 | 1 quarry · 2 clay pit · drag to mark | order bar hint (F7 submenu open) |
| `NODE_CLAY_BANK` | ui | 점토 둑 | Clay bank | map hover on a clay bank |
| `MATERIAL_CLAY` | ui | 점토 | clay | item names with a material |
| `ITEM_CLAY` | ui | 점토 | Clay | hover, shelf, conditions, HUD |
| `ITEM_RAW_POT` | ui | 빚은 토기 | Unfired pot | hover, shelf, conditions |
| `ITEM_CLAY_POT` | ui | 토기 | Clay pot | hover, shelf, conditions |
| `ITEM_GROUND_AXE` | ui | 간 돌도끼 | Ground axe | hover, shelf, inspector tool, chronicle ToolBroke |
| `ITEM_CLAY_FMT` | ui | 점토 | Clay | item with a material (material not shown) |
| `ITEM_RAW_POT_FMT` | ui | 빚은 토기 | Unfired pot | item with a material (material not shown) |
| `ITEM_CLAY_POT_FMT` | ui | 토기 | Clay pot | item with a material (material not shown) |
| `ITEM_GROUND_AXE_FMT` | ui | 간 돌도끼 ({material} 날) | Ground axe ({material} blade) | inspector tool, hover |
| `RECIPE_KNAP_FINE_BLADE` | ui | 고운 날 눌러떼기 | Pressure-flake a fine blade | workshop bills and recipe list, node detail gate |
| `RECIPE_GRIND_AXE` | ui | 돌도끼 갈기 | Grind an axe | workshop bills and recipe list, node detail gate |
| `RECIPE_FORM_POT` | ui | 토기 빚기 | Form a pot | workshop bills and recipe list, node detail gate |
| `RECIPE_FIRE_POT` | ui | 토기 굽기 | Fire a pot | workshop bills and recipe list (the hearth), node detail gate |
| `BUILDING_GRINDING_STONE` | buildings | 갈돌 | Grinding stone | build menu -, workshop panel, node detail gate |
| `BUILDING_POTTERY_BENCH` | buildings | 토기대 | Pottery bench | build menu 0, workshop panel, node detail gate |
| `BUILDING_HEARTH_FIRE` | buildings | 모닥불 (토기 굽기) | Hearth (pot firing) | workshop panel title of the hearth's fire_pot workshop |
| `M4_WAIT_FMT` | ui | 기다리는 중: {node} 아는 사람이 없음 | Waiting: nobody knows {node} | workshop bill, why? panel (site or mark) |
| `CHRONICLE_TOOL_BROKE_FMT` | ui | {person}의 {item} 부러졌다 ({uses}번 씀) | {person}'s {item} broke after {uses} uses | chronicle (ToolBroke, 24) |
| `CHRONICLE_KNOWLEDGE_TAUGHT_FMT` | ui | {teacher}가 {learner}에게 {node} 가르쳤다 | {teacher} taught {learner} {node} | chronicle (KnowledgeLearned with a teacher, 25) |
| `CHRONICLE_KNOWLEDGE_LEARNED_FMT` | ui | {learner}가 {node} 익혔다 ({channel}) | {learner} picked up {node} ({channel}) | chronicle (KnowledgeLearned without a teacher, 25) |
| `CHRONICLE_KNOWLEDGE_DISCOVERED_FMT` | ui | {person}가 {node} 발견했다! | {person} discovered {node}! | chronicle (KnowledgeDiscovered, 26) |
| `CHRONICLE_KNOWLEDGE_AT_RISK_FMT` | ui | ⚠ {node}: {person}만 안다 | ⚠ {node}: only {person} knows it | chronicle (KnowledgeAtRisk, 27) |
| `CHRONICLE_KNOWLEDGE_LOST_FMT` | ui | 기술 소실: {node} (마지막: {person}) | Knowledge lost: {node} (last: {person}) | chronicle (KnowledgeLost, 28), loss toast |
| `CHRONICLE_KNOWLEDGE_LOST_NOBODY_FMT` | ui | 기술 소실: {node} | Knowledge lost: {node} | chronicle and toast (KnowledgeLost without a holder) |

## Changed keys (3; text widened for M4)

| key | file | ko (was) | ko | en | used by |
|---|---|---|---|---|---|
| `UI_ORDER_HINT_BUILD_MENU` | ui | 1–9로 고르세요 · F3 닫기 | 1–9·0·-로 고르세요 · F3 닫기 | Pick with 1–9, 0, - · F3 closes | order bar hint (the build menu grew to 11: 0 토기대, - 갈돌) |
| `UI_ORDER_HINT_IDLE` | ui | F1–F4·F6–F8·F10 명령 · R 방 · P 우선순위 · Space 멈춤 · Home/H 캠프로 | F1–F4·F6–F8·F10 명령 · R 방 · P 우선순위 · T 기술 · K 가르치기 · Space 멈춤 · Home/H 캠프로 | F1–F4·F6–F8·F10 orders · R rooms · P priorities · T tech · K teach · Space pause · Home/H camp | order bar idle hint (T tech map, K teach) |
| `UI_ORDER_HINT_IDLE_MAC` | ui | fn + F1–F4·F6–F8·F10 명령 · R 방 · P 우선순위 · Space 멈춤 · Home/H 캠프로 | fn + F1–F4·F6–F8·F10 명령 · R 방 · P 우선순위 · T 기술 · K 가르치기 · Space 멈춤 · Home/H 캠프로 | fn + F1–F4·F6–F8·F10 orders · R rooms · P priorities · T tech · K teach · Space pause · Home/H camp | order bar idle hint (macOS) |
