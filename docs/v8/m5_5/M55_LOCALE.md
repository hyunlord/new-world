# M5.5 localization keys

Every key the M5.5 UI (Part 4, goal §9) adds, with its Korean and English text. Source: `localization/{ko,en}/ui.json`, compiled by `python3 tools/localization_compile.py --project-root .`.
Formats use `Locale.trf` placeholders (`{name}`). Korean is the owner-facing language.

Parameters: `{neighbour}` is a neighbour name (`NEIGHBOUR_NAME_<n>` of get_neighbours `name_keys`; in the tech map line it carries the Korean subject particle 이/가, `KnowledgeText.with_subject`); `{person}` and `{name}` are person display names (`person_names.gd`; in the chronicle a person link; as in M4/M5 the Korean subject particle after a name is written 가); `{node}` a knowledge node name (with 을/를 in the eureka line), `{item}` an item name (`EconomyText.item`); `{count}` people or items; `{hours}` whole hours to a wave's `wave_deadline` or a caravan's `caravan_leaves` (rounded up); `{value}` a value at 100 % (Σ unit value × count from get_caravan), `{rate}` the ‰ as a factor (0.8, 1.2), `{weighted}` value × rate, `{short}` the weighted shortfall; `{give}`/`{take}` the weighted values of a trade (the Traded row's value = give << 16 | take); `{day}` a display day; `{dir}` one `M55_DIR_*`; `{word}` one `M55_ATT_*`.

Word choices: neighbours are 이웃, a migrant wave 이주민, a caravan 교역 무리; accepting is 받아들이기, declining 돌려보내기; a practised node (know_practice 1) is 숙련; the apparel reuses M5's 가죽 랩 / 허리 두르개 (`M5_TOP_*`); an artifact no colonist can make is "처음 보는 물건 — 단서" and the eureka glyph is ★ (◆ is already the era marker); the neighbour names are short place-like group names (붉은바위, 갈대물, …) without the word 무리, which is kept for the traders.

## New keys (96)

| key | file | ko | en | used by |
|---|---|---|---|---|
| `NEIGHBOUR_NAME_0` | ui | 붉은바위 | Red Rock | neighbour name (`name_keys` 0): N panel, dialogs, banner, chronicle, tech map |
| `NEIGHBOUR_NAME_1` | ui | 갈대물 | Reedwater | neighbour name (`name_keys` 1): N panel, dialogs, banner, chronicle, tech map |
| `NEIGHBOUR_NAME_2` | ui | 곰골 | Bear Hollow | neighbour name (`name_keys` 2): N panel, dialogs, banner, chronicle, tech map |
| `NEIGHBOUR_NAME_3` | ui | 매봉 | Hawk Peak | neighbour name (`name_keys` 3): N panel, dialogs, banner, chronicle, tech map |
| `NEIGHBOUR_NAME_4` | ui | 너른들 | Wide Field | neighbour name (`name_keys` 4): N panel, dialogs, banner, chronicle, tech map |
| `NEIGHBOUR_NAME_5` | ui | 흰여울 | White Rapids | neighbour name (`name_keys` 5): N panel, dialogs, banner, chronicle, tech map |
| `NEIGHBOUR_NAME_6` | ui | 검은숲 | Black Wood | neighbour name (`name_keys` 6): N panel, dialogs, banner, chronicle, tech map |
| `NEIGHBOUR_NAME_7` | ui | 사슴목 | Deer Pass | neighbour name (`name_keys` 7): N panel, dialogs, banner, chronicle, tech map |
| `NEIGHBOUR_NAME_8` | ui | 조개무지 | Shell Mound | neighbour name (`name_keys` 8): N panel, dialogs, banner, chronicle, tech map |
| `NEIGHBOUR_NAME_9` | ui | 버들내 | Willow Creek | neighbour name (`name_keys` 9): N panel, dialogs, banner, chronicle, tech map |
| `NEIGHBOUR_NAME_10` | ui | 돌밭 | Stone Field | neighbour name (`name_keys` 10): N panel, dialogs, banner, chronicle, tech map |
| `NEIGHBOUR_NAME_11` | ui | 늑대재 | Wolf Ridge | neighbour name (`name_keys` 11): N panel, dialogs, banner, chronicle, tech map |
| `M55_NB_SOMEONE` | ui | 이웃 | a neighbour | a neighbour id never seen in a get_neighbours read |
| `M55_DIR_NORTH` | ui | 북 | north | neighbour edge 0 (N panel) |
| `M55_DIR_EAST` | ui | 동 | east | neighbour edge 1 |
| `M55_DIR_SOUTH` | ui | 남 | south | neighbour edge 2 |
| `M55_DIR_WEST` | ui | 서 | west | neighbour edge 3 |
| `M55_ATT_0` | ui | 적대 | hostile | attitude 0–19 (N panel) |
| `M55_ATT_1` | ui | 냉담 | cold | attitude 20–39 |
| `M55_ATT_2` | ui | 무심 | indifferent | attitude 40–59 |
| `M55_ATT_3` | ui | 우호 | friendly | attitude 60–79 |
| `M55_ATT_4` | ui | 친밀 | close | attitude 80–100 |
| `M55_SEX_FEMALE` | ui | 여자 | woman | migrant dialog (female 1) |
| `M55_SEX_MALE` | ui | 남자 | man | migrant dialog (female 0) |
| `M55_NB_TITLE` | ui | 이웃 | Neighbours | N panel title |
| `M55_NB_HINT` | ui | N 또는 Esc로 닫기 | N or Esc closes | N panel hint |
| `M55_NB_DIR_FMT` | ui | {dir}쪽 · {days}일 거리 | to the {dir} · {days} days away | N panel (edge, distance_days) |
| `M55_NB_POP_FMT` | ui | 인구 {count}명 | {count} people | N panel (populations) |
| `M55_NB_ATTITUDE_FMT` | ui | 태도 {value} · {word} | attitude {value} · {word} | N panel attitude line over its bar |
| `M55_NB_CONTACT_FMT` | ui | 마지막 만남: {day}일째 | last contact: day {day} | N panel (last_contact ≥ 0) |
| `M55_NB_CONTACT_NONE` | ui | 아직 만난 적 없음 | never met yet | N panel (last_contact −1) |
| `M55_NB_RUMOUR_HEAD` | ui | 소문으로 아는 신석기 기술 | Neolithic knowledge by rumour | N panel heading over the rumoured nodes |
| `M55_NB_RUMOUR_NONE` | ui | 들리는 소문 없음 | no rumours | N panel (rumoured 0) |
| `M55_NB_TALLY_FMT` | ui | 합류한 이주민 {migrants} · 거래 {trades} · 선물 {gifts} · 단서로 발견 {eurekas} | migrants joined {migrants} · trades {trades} · gifts {gifts} · found by a clue {eurekas} | N panel tallies |
| `M55_NB_NONE` | ui | 이 세계에는 이웃이 없다 | This world has no neighbours | N panel without neighbours |
| `M55_NB_VISIT_WAVE_FMT` | ui | 이주민 {count}명이 와 있다 | {count} migrants are here | N panel: a wave of this neighbour waiting |
| `M55_NB_VISIT_CARAVAN` | ui | 교역 무리가 와 있다 | traders are here | N panel: a caravan of this neighbour trading |
| `M55_MIGRANT_TITLE_FMT` | ui | {neighbour}에서 이주민 {count}명이 왔다 | {count} migrants from {neighbour} | migrant dialog title |
| `M55_MIGRANT_ASK` | ui | 받아들이면 우리 마을 사람이 된다. 답하지 않으면 하루 뒤 돌아간다. | Taken in, they join the colony. Left unanswered, they leave after a day. | migrant dialog line under the title |
| `M55_MIGRANT_LEFT_FMT` | ui | 남은 시간 {hours}시간 | {hours} h left | migrant dialog countdown (wave deadline) |
| `M55_MIGRANT_PERSON_FMT` | ui | {name} · {sex} · {age}세 | {name} · {sex} · age {age} | migrant dialog: one person |
| `M55_MIGRANT_KNOW_FMT` | ui | {node} {permille}‰ | {node} {permille}‰ | migrant dialog: one of the top 3 nodes |
| `M55_MIGRANT_SKILLED` | ui | 숙련 | practised | migrant dialog: know_practice 1 |
| `M55_MIGRANT_KNOWS_NONE` | ui | 아는 기술 없음 | no knowledge to speak of | migrant dialog: no knowledge rows |
| `M55_MIGRANT_APPAREL_FMT` | ui | 옷: {top} | wears: {top} | migrant dialog: wrap 1 → 가죽 랩, else 허리 두르개 |
| `M55_MIGRANT_ACCEPT` | ui | 받아들이기 | Take them in | migrant dialog button (cmd_accept_migrants) |
| `M55_MIGRANT_DECLINE` | ui | 돌려보내기 | Send them away | migrant dialog button (cmd_decline_migrants) |
| `M55_MIGRANT_LATER` | ui | 나중에 | Later | migrant dialog button (closes; the banner reopens it) |
| `M55_MIGRANT_GONE` | ui | 이주민이 이미 떠났거나 답을 받았다 | The migrants have left or been answered | migrant dialog when the wave is no longer waiting |
| `M55_TOAST_ACCEPTED_FMT` | ui | 이주민 {count}명을 받아들였다 | Took in {count} migrants | toast (cmd_accept_migrants accepted) |
| `M55_TOAST_DECLINED` | ui | 이주민을 돌려보냈다 | Sent the migrants away | toast (cmd_decline_migrants accepted) |
| `M55_BANNER_WAVE_FMT` | ui | 이주민 {count}명이 모닥불에서 기다린다 · {hours}시간 남음 | {count} migrants wait at the hearth · {hours} h left | HUD banner while a wave waits (click reopens the dialog) |
| `M55_BANNER_WAVE_WALKING_FMT` | ui | {neighbour}에서 이주민 {count}명이 오고 있다 | {count} migrants are coming from {neighbour} | HUD banner while a wave walks in |
| `M55_BANNER_CARAVAN_FMT` | ui | {neighbour} 교역 무리 · {hours}시간 뒤 떠남 — 눌러서 거래 | Traders from {neighbour} · leave in {hours} h — click to trade | HUD banner while a caravan trades (click opens the trade dialog) |
| `M55_BANNER_CARAVAN_WALKING_FMT` | ui | {neighbour} 교역 무리가 오고 있다 | Traders from {neighbour} are coming | HUD banner while a caravan walks in |
| `M55_HOVER_WAVE_FMT` | ui | 이주민 {count}명 · {neighbour} | {count} migrants · {neighbour} | visitor layer hover (a wave) |
| `M55_HOVER_CARAVAN_FMT` | ui | 교역 무리 · {neighbour} | traders · {neighbour} | visitor layer hover (a caravan) |
| `M55_TRADE_TITLE_FMT` | ui | {neighbour} 교역 무리 | Traders from {neighbour} | trade dialog title |
| `M55_TRADE_LEAVES_FMT` | ui | {hours}시간 뒤 떠남 · 거래 {trades}번 | leave in {hours} h · {trades} trades | trade dialog line under the title |
| `M55_TRADE_RULE_FMT` | ui | 그들은 우리 물건을 값의 {buy}배로 사고, 자기 물건을 {sell}배로 판다 | They buy our goods at {buy}× their worth and sell theirs at {sell}× | trade dialog rule line (buy/sell ‰) |
| `M55_TRADE_OURS` | ui | 우리 비축 | Our stores | trade dialog left column |
| `M55_TRADE_THEIRS` | ui | 그들의 물건 | Their goods | trade dialog right column |
| `M55_TRADE_ROW_FMT` | ui | {item} · {quality} · {count}개 · 값 {value} | {item} · {quality} · ×{count} · worth {value} | trade dialog row (stock or goods) |
| `M55_TRADE_UNKNOWN` | ui | 처음 보는 물건 — 단서 | never seen before — a clue | trade dialog: goods_unknown 1 |
| `M55_TRADE_ALL` | ui | 모두 | All | trade dialog row button (pick every item of the row) |
| `M55_TRADE_NO_STOCK` | ui | 비축 구역에 쌓인 물건이 없다 (비축한 것만 줄 수 있다) | Nothing in a stockpile (only stockpiled items can be given) | trade dialog: no stockpiled stacks |
| `M55_TRADE_NO_GOODS` | ui | 남은 물건이 없다 | No goods left | trade dialog: no goods |
| `M55_TRADE_GIVE_FMT` | ui | 주는 값 {value} × {rate} = {weighted} | we give {value} × {rate} = {weighted} | balance bar: give side |
| `M55_TRADE_TAKE_FMT` | ui | 받는 값 {value} × {rate} = {weighted} | we take {value} × {rate} = {weighted} | balance bar: take side |
| `M55_TRADE_BALANCE_OK` | ui | 그들이 받아들일 만하다 | They would accept this | balance line (give × buy ≥ take × sell) |
| `M55_TRADE_BALANCE_SHORT_FMT` | ui | {short}만큼 모자라다 | {short} short | balance line (the shortfall in weighted value) |
| `M55_TRADE_PICK_HINT` | ui | 주거나 받을 물건을 고르시오 | Pick what to give and what to take | balance line with nothing picked |
| `M55_TRADE_DO` | ui | 거래 | Trade | trade dialog button (cmd_trade) |
| `M55_TRADE_GIFT` | ui | 선물 | Gift | trade dialog button (cmd_trade with an empty take) |
| `M55_TRADE_CLOSE` | ui | 닫기 | Close | trade dialog button |
| `M55_TRADE_REFUSED_FMT` | ui | 거절당함: {reason} | Refused: {reason} | trade dialog status (a refused cmd_trade) |
| `M55_TRADE_GONE` | ui | 교역 무리가 떠났다 | The traders have left | trade dialog when the caravan is no longer trading |
| `M55_TOAST_TRADED_FMT` | ui | 거래함 · 준 값 {give} · 받은 값 {take} | Traded · gave {give} · took {take} | toast and dialog status (cmd_trade accepted) |
| `M55_TOAST_GIFT_FMT` | ui | 선물함 · 값 {give} | Gift given · worth {give} | toast and dialog status (a gift accepted) |
| `M55_REJECT_NO_VISITOR` | ui | 손님이 이미 떠남 | the visitors are gone | order reason no_visitor |
| `M55_REJECT_UNBALANCED` | ui | 값이 맞지 않음 | the values do not balance | order reason unbalanced |
| `M55_TECH_RUMOUR_FMT` | ui | 이웃 {neighbour} 앎 (소문) | neighbour {neighbour} knows it (rumour) | tech map node detail, one per neighbour knowing the node |
| `M55_TECH_EUREKA_FMT` | ui | ★ 단서: {item} — 발견 확률 ×3 | ★ clue: {item} — discovery chance ×3 | tech map node detail for a clue node |
| `M55_TECH_EUREKA_LEGEND` | ui | 단서 (유레카) | clue (eureka) | tech map legend after ★ |
| `M55_INSPECT_FLEE` | ui | 도망 중: 늑대 | Fleeing: wolves | inspector line (get_agent_m5 fleeing) |
| `M55_CHRON_MIGRANTS_ARRIVED_FMT` | ui | {neighbour}에서 이주민 {count}명이 왔다 | {count} migrants arrived from {neighbour} | chronicle kind 38 |
| `M55_CHRON_MIGRANTS_JOINED_FMT` | ui | {neighbour}에서 온 {count}명이 마을 사람이 되었다 ({person} 등) | {count} people from {neighbour} joined the colony ({person} and others) | chronicle kind 39 (count ≥ 2) |
| `M55_CHRON_MIGRANT_JOINED_ONE_FMT` | ui | {neighbour}에서 온 {person}가 마을 사람이 되었다 | {person} from {neighbour} joined the colony | chronicle kind 39 (count 1) |
| `M55_CHRON_MIGRANTS_DECLINED_FMT` | ui | {neighbour}의 이주민 {count}명을 돌려보냈다 | Sent {count} migrants from {neighbour} away | chronicle kind 40 (reason 0) |
| `M55_CHRON_MIGRANTS_UNANSWERED_FMT` | ui | 답이 없자 {neighbour}의 이주민 {count}명이 돌아갔다 | Unanswered, {count} migrants from {neighbour} went home | chronicle kind 40 (reason 1) |
| `M55_CHRON_TRADER_ARRIVED_FMT` | ui | {neighbour} 교역 무리가 왔다 | Traders from {neighbour} arrived | chronicle kind 41 |
| `M55_CHRON_TRADED_FMT` | ui | {neighbour} 교역 무리와 거래했다 (준 값 {give} · 받은 값 {take}) | Traded with {neighbour} (gave {give} · took {take}) | chronicle kind 42 |
| `M55_CHRON_GIFT_FMT` | ui | {neighbour}에 선물을 주었다 (값 {give}) | Gave {neighbour} a gift (worth {give}) | chronicle kind 42 with take value 0 |
| `M55_CHRON_TRADER_LEFT_FMT` | ui | {neighbour} 교역 무리가 떠났다 | Traders from {neighbour} left | chronicle kind 43 |
| `M55_CHRON_EUREKA_FMT` | ui | {person}가 {item} 보고 {node} 발견했다! (단서) | {person} discovered {node} from a clue: {item}! | chronicle kind 44 |
| `M55_YEAR_EXTRA_FMT` | ui |  · 이주 {migrants} · 거래 {trades} · 단서 {eurekas} |  · migrants {migrants} · trades {trades} · clues {eurekas} | year summary, appended to M5_YEAR_LINE_FMT |

Reused M5 keys (unchanged): `M5_TOP_HIDE_WRAP`, `M5_TOP_LOINCLOTH` (migrant apparel), `M5_YEAR_LINE_FMT` (the year line the M5.5 tallies are appended to), `M4_TECH_*` (tech map), the `ITEM_*` / `M5_ITEM_*` item names.
