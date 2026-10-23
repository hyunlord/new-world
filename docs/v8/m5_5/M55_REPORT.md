## 구현 완료 보고 — M5.5

### 구현 의도

M5.5는 마을 밖에 이웃 둘을 두는 패스다. 화면 밖의 이웃이 지식을 가진 이주민을 보내고, 계절마다 교역 무리를 보낸다. 교역 무리의 처음 보는 물건은 마을이 모르는 기술의 단서(유레카)가 되고, 그들은 마을의 남는 날(부싯돌 날)을 사 간다. 함께 M5 보고서의 오너 결정 여섯 가지를 고쳤다: 소집 안 한 사람은 늑대를 피해 도망하고, 압력은 겹치지 않으며, 사냥꾼은 고기를 바로 들고 오고, 가죽 옷은 가죽 1장, 칼은 4번이고, 기분과 다툼은 느껴지게 보정했으며, 다툼의 원인은 실패할 수 있는 기록을 가리킨다. 계약은 `docs/v8/m5_5/M55_GOAL.md`, 결정은 `docs/v8/m5_5/M55_DECISIONS.md` D1–D29, 평가는 `docs/v8/m5_5/M55_EVALUATION.md`. 패스 도중(10-24 02:30) 받은 오너 지시 — 모든 구조는 확장 가능해야 하고 내용은 데이터에 둔다 — 도 적용했다(D28–D29, 아래 "확장성").

### Part 0 기록 (ADR 0016 개정, 아트 상태, M5 결정 기록)

- **ADR 0016 개정 (D3):** 사람당·칸당 메모리 상한은 할당자 계수기로 판정한다(결정적). RSS는 바깥 한도로 남는다: 벤치가 빌드만 하는 프로세스를 5번씩(N=200, N=2,000, 64²) 돌려 반복별 기울기의 중앙값이 사람당 5,120 B, 칸당 128 B 이하여야 한다(`rss_outer`, 상한 `rss_per_agent_le_5120`, `rss_per_tile_le_128`). 빌드만 한 실행이 없으면 `n/a`이고 검사는 실패한다. "게이트가 도는 동안 계기를 바꾸지 않는다"를 ADR과 pitfalls 스킬에 적었다.
- **아트 상태 (D2, D27):** ART-ENV1(`v8/art-env1` `429a1800`)은 PARTIAL로 끝났다(오너: 구조물이 부자연스럽다). ART-STRUCT(`v8/art-struct` `b6baa73f`)는 오너 판단을 기다린다(보고서: 통나무 벽 윗면 이음새 3회 모두 실패). 아트 브랜치는 합치지도 건드리지도 않았다. 사람은 경로 I.
- **M5 오너 결정 (D4):** 압력 겹침 없음, 소집 안 한 사람 도망, 사냥꾼이 고기 운반, 칼 4번, 가죽 옷 1장, 기분·다툼 보정 허용(§3.1.6 한도 안), 사슴 그림은 예·늑대 그림은 보류.

### 보정 (§3.1 일곱 항목 — 각각 전후 수치)

| 항목 | M5 (전) | M5.5 (후) |
|---|---|---|
| 1. 압력 겹침 없음 | 시드 0–7 × 48일 일정에서 병 15번이 큰 압력 위에 겹침; 804·814는 한파+병+장작 0으로 거의 전멸 | 같은 일정에서 병 14번, 큰 압력 중·후 2일 안에 시작한 것 0 (`colony_story.rs`); 평가 F1a 32/32 |
| 2. 소집이 의미 있음 | 소집 안 해도 늑대 무리 100개를 모두 이김(사망 3) | 평가: 소집 시 늑대 사망 0인 세계 32/32, 소집 안 하면 50 hp 미만이나 사망이 있는 세계 29/32 (사망 19명, 기록 실행); 늑대 8칸 안의 사람이 30틱 안에 멀어짐 0.903 |
| 3. 고기 운반 | 고기가 창고에 오기 전에 썩어 F4 27/32 | 잡은 자리에서 바로 운반 (`haul_home`); 평가 F3 26/32 — 실패 6곳은 비축 칸이 통나무로 차서 고기 둘 칸이 없음 (D24) |
| 4. 가죽 옷 = 손질 가죽 1장 | 2장, 30일까지 4명 입은 세계 19/32 | 1장; 30일 3명·48일 5명 입은 세계 31/32 (교역 가죽 포함) |
| 5. 칼 = 4번 | 8번, 세계당 사용 6–22번이라 소모 0–1개 | 4번 (`colony_hides.rs`: 네 번째에 부러짐) |
| 6. 기분·다툼이 느껴짐 | 압력이 있어도 실의 있는 세계 3/32, 다툼 4번 미만 세계 7/32 | 시드 0–7(r2): 실의 1.12/세계(이야기꾼 끔 0), 다툼 6.8/세계-해; 평가: 실의 있는 세계 24/32(목표 ≥ 8), 다툼 6.80/세계-해 — 단 무너지는 두 세계(903 6번, 906 8번)가 상한 3을 넘음 |
| 7. 다툼 원인이 실패할 수 있음 | 원인 사슬 197개 중 32개만 연대기 항목에 닿음 | 원인이 구체적 기록(필요 표본·죽음·압력·같은 두 사람의 이전 다툼)을 가리키고 `mood::check_cause`가 검사: 평가에서 435개 중 418개 해결(0.961); 리뷰 후 기록이 원인과 맞는지도 검사 |

### 구현 내용 (이웃·이주·교역·유레카·도망, UI — 파일별)

- **이웃** `sim-core/src/colony/neighbours.rs`(이웃·파도·교역 무리·장부 기록), `sim-core/src/rules/neighbours.rs`(`NeighbourRule::m5_5()`, 값은 `data/colony/neighbours.ron`), `sim-systems/src/runtime/neighbours/mod.rs`(`NeighbourSystem` 174, 하루 한 번: 생성, 파도 1·2, 계절 굴림, 교역 날짜), `visitors.rs`(`VisitorSystem` 175, 3틱마다: 걷기·기다리기·떠나기), `join.rs`(받아들인 이주민 생성), `trade.rs`(물건 표, 값, 물물교환, 단서).
- **이주:** 명령 `AcceptMigrants`/`DeclineMigrants`, 지식 출처 `Migrant`(견습 바닥), 50 %가 가죽 랩, 미리 잡아 둔 AgentId(대화창 이름 = 마을 사람 이름).
- **교역:** 명령 `Trade`, 일 `JobKind::Trade`(준 물건을 무리 자리로 나름), 장부 `traded_in`/`traded_out`, 값표 `sim-core/data/colony/values.ron`; 탐사 명령 `DebugGive`.
- **유레카:** `sim-systems/src/runtime/knowledge/discover.rs`(이웃 규칙 아래에서만; 단서 노드 ×3, 비축 조건 충족; 단서가 결정한 발견만 `Eureka`).
- **도망:** `sim-systems/src/runtime/wildlife/flee.rs`(`FleeSystem` 123, 3틱마다), `jobs/seam.rs` +7줄, `draft.rs`(소집하면 도망 끝), `hunt.rs`(반격 절반, 고기 운반), `wildlife/mod.rs`(쫓는 늑대 2칸 걸음, 시체 목록).
- **기분·다툼:** `rules/mood.rs`(가중치·다툼 상수·`cause_records`), `mood/mod.rs`(`cause_record`, `check_cause`), `mood/quarrel.rs`, `chronicle.rs`(`NeedSample`, 종류 38–44).
- **브리지** `sim-bridge/src/ffi/neighbours.rs`: `get_neighbours`, `get_wave`, `get_caravan`, `cmd_accept_migrants`, `cmd_decline_migrants`, `cmd_trade`, 스냅숏 `fleeing`; 연간 정리에 이주·거래·단서(`docs/v8/bridge_api_m5_5.txt` 97개, 스키마 6).
- **UI:** `scripts/ui/panels/migrant_panel.gd`(이주민 창, 1배속이면 멈춤), `trade_panel.gd`(거래 창, 값 막대, "처음 보는 물건 — 단서"), `neighbours_panel.gd`(N), `scripts/world/colony/visitor_layer.gd`(지도 위 방문자), `scripts/ui/neighbour_text.gd`; 고친 파일 `story_banner.gd`(방문자 배너), `tech_panel.gd`(소문 줄, ★), `agent_layer.gd`("!"), `agent_inspector_panel.gd`("도망 중: 늑대"), `chronicle_text.gd`·`chronicle_panel.gd`(종류 38–44, 해 정리), `sim_reads.gd`, `hud_topbar.gd`, `order_tool.gd`, `camera_controller.gd`; 현지화 키 96개(`docs/v8/m5_5/M55_LOCALE.md`).
- **측정:** `sim-test/src/probe/m55.rs`(`m55` 블록), `--neighbours on|off`, `tools/v8/m55_u_script.json`·`m55_u_nodraft_script.json`, `docs/v8/m5_5/m55_score.py`, 벤치 줄 `n1000_wolves`, 게이트 30단계 `neighbours`.

- **확장성 (오너 지시 2026-10-24, D28–D29):** M5.5의 수치와 내용은 `sim-core/data/colony/`의 RON 파일에 있다 — `neighbours.ron`(이웃 규칙 전부: 파도·계절 굴림·이주민 지식 규칙·교역·물물교환·유레카 ×3, 이름 키 12개, 교역 물건 표(물건 + 태그 `lacked`/`staple` + 더미 크기, 뽑기 순서), 반드시 한 이웃이 아는 노드(토기 굽기), 이주민의 옷), `story.ron`(압력 수치), `wildlife.ron`(동물·사냥·도망), `mood.ron`(기분·다툼 보정 값), `values.ron` 2판(값, `made_by`, `clue_nodes`, 태그 `carcass`). 코드는 물건을 이름이 아니라 태그로 고르고(`ARTIFACTS`/`STAPLES`/물건 이름 맞춤과 `M5_5` 상수를 모두 지움), 파일은 엄격히 읽는다(판 번호, 모르는 필드 거절; `rules::data`). 데이터의 이름이 모두 풀리는지 시험이 확인하고, 깨진 파일은 그 기능을 끈다(멈추지 않음). 브리지는 이웃 이름을 로캘 키 문자열로 보내고(`name_keys`), 이주민 옷을 코드로 보낸다(`tops`). 행동이 바뀌지 않았음: 데이터로 옮긴 빌드로 시험 세계 평가를 세 번 다시 돌려 `32948f2d`와 다섯 실행 모두 값 하나도 다르지 않았다(예: 압력 실행 196,975값, 차이 0).

### 구현 방법 (이웃 생성, 파도 확률식, 가치표와 물물교환 규칙, 유레카, 도망 접점 줄 수)

- **이웃 생성 (D17):** 첫날 `STREAM_NEIGHBOURS`로 둘: 서로 다른 가장자리, 거리 2–5일, 인구 20–60, 태도 40–60, 구석기 전부 + 신석기 7개 중 열쇠 섞기의 앞 2–4개. 둘 다 토기 굽기를 모르면 첫 이웃의 마지막 노드를 토기 굽기로 바꾼다(D23). 인구는 하루 ±1 %만 흔들린다.
- **파도 (D18):** 1번 8–12일(2–4명), 2번 20–26일(2–6명), 그 뒤 계절마다 2–5일에 한 번 굴림: 확률 ‰ = min(800, 50 × 남는 식량 일수 + 비축 값/50 + 태도 × 2), 1–4명, 사람 40명 이상이면 없음. 태도가 높은 이웃에서 온다. 가장자리에서 모닥불까지 걸어와 하루 기다린다; 걸어오는 중에 받아들이면 도착해서 합류한다(리뷰 F3). 거절·무응답이면 떠나고 태도 −5.
- **이주민 지식:** 이웃의 구석기 노드는 450–700 ‰, 신석기 노드는 50 % 확률로 420–650 ‰(문턱 400 이상이면 숙련), 출처 `Migrant`.
- **교역 (D19):** 계절마다 2–5일 중 계획된 큰 압력이 없는 날(D23), 큰 압력 중이면 끝+1일로 미룸, 6일 전엔 없음, 태도 30 이상인 이웃(둘 다면 번갈아). 물건 6–10더미: 이웃이 알고 마을이 모르는 노드의 물건(토기·간 도끼·손질 가죽·창·가죽 랩·털옷) 무게 3, 기본 물건(고기·부싯돌·점토·가죽·장작) 무게 1, 품질 30–70.
- **값과 물물교환:** 값 = 기본값 × (500 + 5 × 품질) / 1,000 (반올림, 최소 1). 받아들이는 조건: 준 값 × 0.8 ≥ 받는 값 × 1.2. 받은 물건은 무리 자리에 놓이고(`traded_in`), 준 물건은 운반꾼이 무리 자리로 나르면 나간다(`traded_out`); 준 물건은 운반꾼이 집을 때까지 빚으로 남아 다시 줄 수 없다(리뷰 F1). 거래 태도 +3, 선물은 값 10당 +1. 장부: 만듦 + 들어옴 = 땅 + 운반 + 손 + 배달 + 선반 + 소비 + 나감.
- **유레카 (D20, D25, D26):** 마을이 가진 물건 중 그것을 만드는 노드를 아무도 못 하는 것이 단서다. 이웃 규칙 아래에서만, 그 노드의 비축 조건이 충족된 것으로 치고 같은 열쇠 뽑기가 0, 1, 2에서 이긴다(×3). 단서가 결정한 발견만 `Eureka`로 센다. 증거: 시드 0–7, 이웃 켬, 2일째 토기 하나 → 토기 빚기 첫 발견 중앙값 11.4일(없으면 13.5), 토기 굽기 15.2일(22.0).
- **도망 접점:** `decision/system.rs` **+0줄**(예산 +20), `jobs/seam.rs` +7줄(도망 중인 사람을 결정이 건너뜀). `FleeSystem`은 우선순위 123(결정 125 전), 늑대(≤ 4) 주변 청크만 본다; 피난처는 늑대보다 먼저 닿는 지붕 방, 아니면 모닥불, 아니면 늑대 반대쪽 8칸.

### 성능 계획과 실제

계획(§4): M5.5 시스템 ≤ 60 µs/틱(N=1,000), `FleeSystem` ≤ 15 µs/틱(무리 있을 때), 할당/사람 ≤ M5 + 32 B.

조용한 교차 세션(01:12–01:17 KST, 1분 부하 4.9–7.3; M5 `cb58a801`과 M5.5 `32948f2d`를 번갈아 3라운드, `$DATA/bench/`):

| 장면 | M5 틱 평균 µs | M5.5 | 비 | p99 비 | M5.5 시스템 µs/틱 |
|---|---|---|---|---|---|
| n200 | 442.6 | 433.9 | ×0.980 | ×0.973 | 0.32 |
| n1000 | 2,163.7 | 2,097.3 | ×0.969 | ×0.951 | 0.50 |
| n2000 | 4,736.4 | 4,824.4 | ×1.019 | ×1.023 | 0.63 |
| n200_script | 472.8 | 548.1 | ×1.159 | ×1.518 | 0.48 |
| n200_side64 | 269.0 | 280.8 | ×1.044 | ×1.102 | 0.34 |
| n1000_wolves (새) | — | 2,039.9 | — | — | **10.6** (도망 10.6), 살아남음 998/1,000 |

- n200_script의 ×1.159는 같은 세계(지문 `bd38af8089e5f72b` 동일)에서 결정 시스템이 +45 µs였다. 이 장면만 따로 5번씩 번갈아 다시 재니 중앙값 522.9 → 532.6 µs(×1.019), 결정 시스템도 같다 — 세션의 ×1.159는 재현되지 않는 잡음이다(같은 시각 다른 작업의 부하가 15까지 오름).
- 할당/사람 1,871.37 → 1,871.37 B(+0 B), 할당/칸 63.1 B; RSS 바깥 한도: 사람당 중앙값 2,230 B(≤ 5,120), 칸당 100.0 B(≤ 128), 15쌍; C3 상한 6/6 통과.
- 이 세션은 `32948f2d`로 쟀다. 그 뒤 바뀐 것은 데이터를 시작할 때 한 번 읽는 것뿐이고 평가 결과는 값 하나까지 같았다; 최종 커밋의 틱 값은 게이트의 bench 단계가 잰다(아래).
- **최종 커밋 `ae443aa4`의 게이트 bench**(조용함, 1분 부하 6.44; M5 장부 줄 대비): 틱 평균 n200 ×0.996, n1000 ×1.011, n2000 ×1.012, n200_script ×1.003, n200_side64 ×1.015; p99 ×0.989–×1.023; 할당/사람 1,871 → 1,871 B; RSS 바깥 한도 사람당 2,175 B·칸당 82 B; C3 6/6; 늑대 줄의 M5.5 시스템 **9.5 µs/틱**(도망 9.5, 계획 ≤ 15·예산 ≤ 60). 장부에 "M5.5" 줄 6개를 게이트 기록으로 붙였다.
- identity·replay·determinism은 게이트에서(아래).

### 평가 결과 (F1–F11 표 + 시드별 부록, 대조군 셋과 비교)

시험 세계 900–931, 48일, 두 번 측정: 기록(`c727cd45`)과 리뷰 수정 뒤(`32948f2d`). 시험 세계로는 아무것도 맞추지 않았다.

| # | 막대 | 기록 | 리뷰 뒤 | 판정 |
|---|---|---|---|---|
| F1 | 겹침 없음 32 · 비늑대 압력 사망 ≤ 2 (≥ 30) · 절반 잃은 세계 없음 32 | 32 · 32 · 31 | 32 · 32 · 31 (906) | **미달** |
| F2 | 소집 늑대 사망 0 ≥ 28 · 소집 없이 사망/50 hp 미만 ≥ 20 · 30틱 안 멀어짐 ≥ 80 % | 32 · 29 · 0.901 | 32 · 29 · 0.903 | **통과** |
| F3 | 사슴 ≥ 6, 먹은 고기 ≥ 12, 밖에서 썩은 고기 ≤ 10 % (≥ 28) | 25 | 26 | **미달** |
| F4 | 30일 3명·48일 5명 입음, 추위 비 ≤ 0.8 (≥ 24) | 29 | 30 | **통과(등록 규칙)** — 추위 비는 30곳에서 측정 불가(둘 다 0), 측정된 2곳은 0.816·0.813로 미달 |
| F5 | 실의 있는 세계 ≥ 8이고 모든 세계 ≤ 3 · 이야기꾼 끔 ≤ 1 (≥ 28) | 23곳, 최대 8 · 32 | 24곳, 최대 8 · 32 | **미달** (903 6번, 906 8번) |
| F6 | 다툼 ≥ 4 (≥ 28), 싸움 ≤ 다툼/4, 다툼 사망 0; 원인 해결 ≥ 90 % | 29 · 0.962 | 30 · 0.961 | **통과** (싸움은 0번) |
| F7 | 파도 1·2 제때 32 · 48일 인구 16–40 ≥ 30 · 이주민이 신석기 노드 ≥ 20 | 32 · 28 · 32 | 32 · 28 · 32 | **미달** (906 3명, 920·922·928 15명) |
| F8 | 교역 ≥ 6번 ≥ 28 · 처음 보는 물건 구매 ≥ 28; 유레카 보고 | 32 · 26 · 55 | 32 · 26 · 33 | **미달** |
| F9 | 37–48일 날 ≥ 5, 최고 석공 ≥ 6, 품질 차 ≥ 8 (≥ 24) | 23 | 24 | **통과**(막대에 딱) |
| F10 | 출생/여성-해 0.20–0.30 · 모든 실행 장부·예약 정확 | 0.299 · 32/32 | 0.299 · 32/32 | **통과** |
| F11 | M5.5 시스템 ≤ 60 µs, 할당 ≤ M5+32 B, RSS 한도, C3, M5 대비 회귀, identity, replay | — | 9.5 µs(늑대 줄) · +0 B · 2,175 B/사람·82 B/칸 · 6/6 · ×0.989–×1.023 · 통과 · 통과 | **통과** |

**리뷰 뒤 실패 시드:** F1c 906 · F3 903, 906, 908, 926, 927, 929 · F4 906, 920 · F5a 903(6), 906(8) · F6a 906, 922 · F7b 906, 920, 922, 928 · F8b 905, 911, 912, 926, 927, 928 · F9 902, 903, 905, 906, 914, 918, 923, 926. 부록: `$DATA/eval/fixed/appendix.json`, `$DATA/eval/record/appendix.json`.

**대조군 셋과 비교:** 소집 없음 — 50 hp 미만 세계 29곳·사망 19명(소집 1곳·0명; 물린 수는 같음). 이야기꾼 끔 — 실의 ≤ 1인 세계 32/32. 이웃 없음 — 토기 빚기 첫 발견 중앙값 16.7일(이웃 있음 13.2), 간 도끼 13.2(12.2), 유레카 0(33). 토기 굽기는 이웃이 있으면 16곳이 아예 발견할 필요가 없다(이주민이 가져온다는 가설; 세계별로 추적하지 않음).

### 보정 기록 (라운드별 바꾼 수치와 이유)

시드 0–7, 48일, 3라운드(한도 8) — D15:
- **r0**(M5 기분 값): 실의 0.25/세계, 다툼 14.8/세계-해, 소집 없는 세계가 한 번도 물리지 않음(F2b 0/8: 도망이 늑대 걸음과 같아 늘 빠져나감), 도망 성공 0.769(늑대 너머 방으로 뜀), 교역 5/8(날짜가 큰 압력에 밀려 계절 밖으로).
- **r1:** 다툼 기본 2 → 1 ‰, 관계 나눔수 100 → 200; 압력 −40 → −80, 병 −80 → −160(×2 한도); 실의 문턱 100 → 120; 쫓는 늑대는 2칸 걸음; 늑대보다 먼저 닿는 피난처만; 교역 날짜는 계획된 큰 압력을 피함; 한 이웃은 토기 굽기를 앎 → 실의 2.38, 다툼 8.8, F2b 8/8, 도망 0.943, 교역 8/8.
- **r2:** 실의 문턱 120 → 105, 다툼 기분 나눔수 25 → 40 → 실의 1.12(끔 0), 다툼 6.8, 원인 해결 0.972. 남은 미달 F8b 6/8 — 시드 0·3은 처음 보는 물건이 한 번도 나오지 않음(D24에서 원인 확인: 둘째 무리부터 마을이 24개 노드를 모두 안다).

### 변경된 파일 목록 (커밋별)

- `b7b69e4b` (p0): 목표, `M55_DECISIONS.md`, ADR 0016 개정·0018 아트 기록·0027–0030 개정 초안·0031·0032 초안, ROADMAP·HISTORY, `sim-test/src/bench.rs`(RSS 바깥 한도), `tools/v8/bench_ledger.py`, `tools/v8/gate.sh`.
- `9261bd88` (p1): 보정 — `rules/story.rs`·`wildlife.rs`·`mood.rs`·`neighbours.rs`·`colony.rs`, `colony/people.rs`(`Fleeing`)·`wildlife.rs`·`neighbours.rs`·`values.rs`·`items.rs`·`knowledge.rs`·`job.rs`·`command.rs`, `chronicle.rs`, `rng.rs`, `data/colony/recipes.ron`·`values.ron`, `runtime/story/mod.rs`, `wildlife/flee.rs`·`mod.rs`·`fight.rs`·`draft.rs`, `jobs/hunt.rs`·`assign.rs`·`execute.rs`·`invariants.rs`·`seam.rs`, `mood/mod.rs`·`quarrel.rs`, `neighbours/*`(꺼진 채), `save/*`(v6), 탐사 `probe/colony.rs`, 시험(`colony_story.rs`, `colony_wildlife.rs`, `colony_hides.rs`, `colony_mood.rs`) — 51개 파일.
- `2b223606` (p2, p3): 이웃·이주·교역·유레카 켬 — `rules/colony.rs`, `commands.rs`, `command.rs`, `knowledge/discover.rs`, `neighbours/trade.rs`, `probe/colony.rs`·`m55.rs`, `colony_neighbours.rs`, `m55_score.py`, 스크립트 둘.
- `c727cd45` (보정, p4 브리지): `rules/mood.rs`·`wildlife.rs`, `neighbours/*`, `flee.rs`, `wildlife/mod.rs`, `sim-bridge/src/ffi/neighbours.rs`·`node.rs`·`story.rs`·`mod.rs`, `engine_neighbours_ffi.rs`, `bench.rs`(늑대 줄), `gate.sh`(30단계), `bridge_api_m5_5.txt`, `snapshot_schema.md`, ARCHITECTURE, CHANGELOG.
- `e13a41ce` (p4): UI 32개 파일(위 "구현 내용").
- `32948f2d` (p6 리뷰): `rules/mood.rs`, `jobs/assign.rs`·`execute.rs`, `knowledge/discover.rs`, `mood/mod.rs`·`quarrel.rs`, `neighbours/join.rs`·`trade.rs`·`visitors.rs`, 시험 `colony_neighbours.rs`·`colony_mood.rs`, D24·D25.
- `64cff209` (p5, p6 문서): `M55_EVALUATION.md`, ADR 0027–0032 확정, ARCHITECTURE, CHANGELOG, ROADMAP, HISTORY, README, AGENTS.md, 스킬 넷, 접촉 시트(`docs/images/m55_contact_sheet.png`, M5 시트 지움), D26·D27.
- `ae443aa4` (p6 확장성): `sim-core/data/colony/neighbours.ron`·`story.ron`·`wildlife.ron`·`mood.ron`(새), `values.ron`(2판), `sim-core/src/rules/data.rs`(새, 엄격 읽기)·`neighbours.rs`·`story.rs`·`wildlife.rs`·`mood.rs`·`colony.rs`·`mod.rs`, `colony/values.rs`, `sim-systems/src/runtime/neighbours/mod.rs`·`trade.rs`·`join.rs`, `jobs/hunt.rs`, `wildlife/mod.rs`, `sim-bridge/src/ffi/neighbours.rs`, GDScript `neighbour_text.gd`·`migrant_panel.gd`·`neighbours_panel.gd`·`m55_ui_test.gd`, 시험 넷, 스키마, ADR 0031·0032, ARCHITECTURE, CHANGELOG, HISTORY(오너 지시), 아키텍처 스킬, D28·D29.
- (보고 커밋): `M55_REPORT.md`, `M55_EVALUATION.md` F11, `PERF_LEDGER.md`.

### 리뷰 결과

- **코드 리뷰**(새 서브에이전트, `$DATA/review/code_review.md`, 재현 시험 6개): 큰 문제 2, 작은 문제 7, 사소한 것 1.
  - F1(큰): 교역에 준 물건을 운반꾼이 배정되는 순간 빚에서 빼서, 아직 더미에 있는 같은 날 12개를 두 번 줄 수 있었다 → 집을 때 뺀다.
  - F2(큰): 유레카의 "비축 조건 충족"이 이웃 규칙이 꺼져도 작동했고, D20의 증거가 바로 그 경로를 쟀다 → 규칙 아래에서만; 증거를 이웃 켬으로 다시 냄(D26).
  - F3: 걸어오는 중에 받아들인 이주민이 지도 가장자리(모닥불에서 72칸)에 생김 → 도착해서 합류.
  - F4: `check_cause`가 원인에 맞지 않는 기록도 통과시킴 → 기록 종류와 같은 두 사람을 검사.
  - F5: 단서가 결정하지 않은 발견도 유레카로 셈 → 단서가 결정한 것만.
  - F6: M5 기분 규칙에서도 M5.5 원인 표본을 씀 → `cause_records`.
  - F7: 만남 자리가 없으면 방문자가 멈춤 → 그 자리에서 기다리고 떠남.
  - F8: 같은 칸의 다른 종류 빚이 서로 막음 → 종류로 맞춤.
  - F9: 미룬 교역이 큰 압력 끝 +1일 — 목표 문구대로 유지(결정).
  - F10: 사소한 것 넷 — `FleeSystem`·`BillSystem` 우선순위 공유(등록 순서로 결정적, ARCHITECTURE에 적음), D17 문구 고침 등.
  - 수정마다 고치기 전 코드에서 실패하는 시험을 붙였다(9개 중 8개가 옛 코드에서 실패; ×3 시험은 증거용).
  - 깨끗했던 것: 도망(무리 4마리 × 6세계 × 900틱, 도망 156번, 문제 0, 도망 중 저장/불러오기 6번이 같은 결과), 이주민 합류 경로, 장부, 유레카 결정성, 교역 일정.
- **확장성 리뷰**(오너 지시의 Part 6 검사, 새 서브에이전트, `$DATA/review/extensibility_review.md`): 반드시 고칠 것 7개 — 빚 목록 없음, 교역 더미 크기가 코드 상수, 이주민 창이 가죽 랩을 하드코딩, RON의 오타·판 번호를 조용히 받아들임, 풀리지 않는 이름을 조용히 버림, 깨진 `values.ron`이 꺼지지 않고 값 1로 돎, 시험 전용 `pottery_mask`가 제품 코드에 있음 — 모두 고쳤다(D29). 구조적 제한 약 18개는 M-EXT 빚으로 D29에 적었다(이웃 정확히 둘, 정해진 파도 둘, 시대(구석기/신석기)로 짠 이주민 지식, `Source::Migrant`, 태그 `lacked`/`staple`의 고정 의미, 이름 붙은 기분 가중치, 늑대에게서만 도망, `include_str!`이라 모드 덮어쓰기 경로 없음, GDScript의 연대기 번호 표 등). 새 enum 중 내용 enum은 `Source::Migrant`(경계)뿐이고 나머지(`Edge`, `VisitState`, `WaveAnswer`, 명령·거절 이유, `JobKind::Trade`, 연대기 종류 38–44 등)는 구조적이다.
- **숫자 리뷰**(`$DATA/review/numbers_review.md`): F2, F4, F5, F6, F9를 원자료에서 자기 코드로 다시 계산 — 모든 개수와 실패 시드가 일치. 지적:
  - F4 추위 비는 대부분 측정되지 않음(0 ≤ 0.8 × 0).
  - F9의 전제(초반 석공 수준 ~1)가 틀림: 2–10일에 이미 6 이상.
  - F2c는 사건 단위이고 97 %가 소집 없음 실행에서 나옴.
  - F6은 싸움이 0번이라 싸움 조건이 공허함.
  - 906은 F1b가 못 잡음.

  모두 평가 문서와 이 보고서에 반영했다.

### 확인된 제한사항

- **906(추운 세계):** 13/16명이 죽는다(노출 11). 이야기꾼 끔·이웃 없음·M5 빌드에서도 같다 — M5.5 때문이 아니라 세계 자체.
- **비축 칸이 차면 고기가 썩는다:** 스크립트가 통나무를 계속 베어 72칸이 31일쯤 찬다. 고기 운반은 "음식을 받는 칸"이 있어야 한다.
- **이주민과 처음 보는 물건의 충돌:** 모든 파도를 받아들이면 둘째 교역 무리부터 마을이 모든 노드를 알아 살 단서가 없다(F7c ↔ F8b).
- **F9:** 막대의 전제가 틀려 품질 차 8이 대부분 불가능; 리뷰 뒤 24/32는 막대에 딱이다.
- **F4:** 집에서 자는 사람은 춥지 않아, 옷의 추위 효과를 이 설계로는 잴 수 없다.
- **교역 빚:** 집은 뒤 취소된 짐은 탕감되고, 다른 일이 빚진 물건을 쓸 수 있으며, 태도는 거래할 때 오른다.
- **인구 상한 40:** 파도가 떠날 때만 검사한다.
- **유레카 연대기:** 한 발견에 "발견했다"와 "토기를 보고 … (단서)" 두 줄이 같은 날 보인다.
- **"!" 겹침:** 고른 사람의 "!"가 선택 상자와 겹쳐 둘로 보인다(접촉 시트 4번).
- **싸움:** 보정 값에서 다툼이 싸움으로 번지지 않는다(0번).
- **확장성 빚:** D29의 M-EXT 목록 — 구조(이웃 둘, 파도 둘, 시대 구조, 태그 의미, 닫힌 `ItemKind`, 모드 덮어쓰기 경로 없음)는 아직 코드에 있다.

### 오너 결정 필요 (다음: M-ART2 vs M6; 인구 상한; 교역 빈도; 이주 거절의 대가)

0. **확장성 계약:** ADR 0033은 지시대로 M-ART2의 Part 0에서 쓴다. M-EXT 빚 목록(D29) 중 무엇을 먼저 풀지 — 특히 닫힌 `ItemKind`(데이터로 물건을 더할 수 없음)와 `include_str!`(모드 덮어쓰기 경로 없음).

1. **다음 패스:** ART-STRUCT가 오너 판단 대기 → ROADMAP "다음"은 M6 준비로 두었다(D27). ART-STRUCT를 승인하면 M-ART2(경로 I 사람, 64픽셀 카메라, 생성 환경, 한 투영 구조물)가 먼저.
2. **인구 상한:** 40은 파도가 떠날 때만 본다; 받아들이는 순간에도 막을지, 상한을 바꿀지.
3. **교역 빈도와 물건:** 계절당 한 번(2년에 6–7번). 이주민이 기술을 다 가져오면 단서가 사라진다 — 무리가 이웃이 아는 물건을 더 자주 가져오게 할지, 이주민 지식을 줄일지(F8b).
4. **이주 거절의 대가:** 지금은 태도 −5뿐. 더 무겁게(교역 중단 등) 할지.
5. **고기와 비축 칸:** 칸이 없으면 모닥불 곁에 고기 더미를 두게 할지(F3).
6. **F4·F9 막대:** 옷의 효과를 밤 추위 대신 무엇으로 잴지; 날 품질 막대를 실제 숙련 속도에 맞게 다시 쓸지.

### Gate 결과

`bash tools/v8/gate.sh $DATA/gate2 --base-data $DATA/base/identity` (`BENCH_PASS=M5.5`), 깨끗한 커밋 `ae443aa4`에서 30단계 모두 통과(03:53–05:24 KST, `status 0`, `dirty 0`). 첫 게이트(`64cff209`)는 1–9단계 통과 뒤 오너의 확장성 지시로 멈췄다(코드가 바뀌므로; D28).

```
head ae443aa4a498680ef56211faa6b993533019a6d2
dirty 0
started 2026-10-24 03:53:46 KST
 1 fmt                exit   0      1s  started 03:53:46
 2 bridge_build       exit   0      0s  started 03:53:47
 3 godot_import       exit   0      7s  started 03:53:47
 4 test_debug         exit   0   3790s  started 03:53:54
   test_debug passed 2012 failed 0 ignored 0
 5 test_release       exit   0    193s  started 04:57:04
   test_release passed 2012 failed 0 ignored 0
 6 clippy             exit   0      0s  started 05:00:17
 7 forbidden_math     exit   0      1s  started 05:00:17
 8 gdscript_strict    exit   0      9s  started 05:00:18
 9 smoke              exit   0     15s  started 05:00:27
10 identity           exit   0    469s  started 05:00:42
11 determinism        exit   0    129s  started 05:08:31
12 perf               exit   0    534s  started 05:10:40
   perf: OK (no mean above +10 %)
13 bridge_api         exit   0      0s  started 05:19:34
14 docs               exit   0      1s  started 05:19:34
15 inventory_closure  exit   0      0s  started 05:19:35
16 python_unittest    exit   0      1s  started 05:19:35
17 m1_tests           exit   0     46s  started 05:19:36
   m1_tests passed 58 failed 0
18 colony_determinism exit   0      3s  started 05:20:22
19 replay             exit   0      2s  started 05:20:25
20 uid_check          exit   0      0s  started 05:20:27
21 play_check         exit   0     13s  started 05:20:27
   play_check passed 6 of 6
22 schema_docs        exit   0      0s  started 05:20:40
23 renderer_agreement exit   0     21s  started 05:20:40
24 save_roundtrip     exit   0     11s  started 05:21:01
25 bench              exit   0    130s  started 05:21:12
   admission: quiet (1-min load 6.44 < 8 after waiting 0 s)
   n200 tick mean (µs): 415.9 -> 414.2 (x0.996) ok
   n200 p99 (µs): 846.0 -> 842.0 (x0.995) ok
   n1000 tick mean (µs): 1957.9 -> 1980.1 (x1.011) ok
   n1000 p99 (µs): 4958.0 -> 5021.0 (x1.013) ok
   n2000 tick mean (µs): 4445.5 -> 4500.7 (x1.012) ok
   n2000 p99 (µs): 10787.0 -> 10673.0 (x0.989) ok
   n200_script tick mean (µs): 444.3 -> 445.6 (x1.003) ok
   n200_script p99 (µs): 902.0 -> 901.0 (x0.999) ok
   n200_side64 tick mean (µs): 253.6 -> 257.5 (x1.015) ok
   n200_side64 p99 (µs): 487.0 -> 498.0 (x1.023) ok
   alloc/agent: 1871 -> 1871 B (x1.000) ok
   RSS/agent: 2166 -> 2166 B (x1.000) information only
   rss outer bound: per agent 2175.431111111111 B (<= 5120.0), per tile 82.0 B (<= 128.0), median of 5 build-only repeats
   ceiling bytes_per_agent_le_4096: PASS
   ceiling bytes_per_tile_le_96: PASS
   ceiling n1000_mean_le_5ms: PASS
   ceiling n2000_mean_le_10ms: PASS
   ceiling rss_per_agent_le_5120: PASS
   ceiling rss_per_tile_le_128: PASS
   bench: PASS
26 soak               exit   0     10s  started 05:23:22
   SOAK PASS (4 seeds x 20 days: ledger exact, reservations exact, 0 deaths, 0 KnowledgeLost, determinism)
27 nav_bench          exit   0      9s  started 05:23:32
   nav_bench: N=1000 movement 131.0 µs/tick (budget 600) WITHIN | tick 2190.0 µs | requests/100 ticks 3031.9 | memo hits 84.9 % | mean expanded 182.1 | unreachable 0 | load 3.04 3.29 3.91 -> 2.87 3.25 3.88
28 knowledge          exit   0      5s  started 05:23:41
   test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.14s
   F10: 8/8 PASS
29 story              exit   0     18s  started 05:23:46
   STORY PASS (2 seeds: F1 schedule, ledger and reservations exact)
30 neighbours         exit   0     29s  started 05:24:04
   NEIGHBOURS PASS (2 seeds: two neighbours, waves 1 and 2 in their windows and accepted, a trade, ledger and reservations exact)
finished 2026-10-24 05:24:33 KST status 0
```

`v8/main`은 `cb58a801` → `ae443aa4`로 빨리 감기(fast-forward)했다. 이 보고서와 장부·평가 문서는 그 위의 문서 커밋이다(코드는 `ae443aa4`와 같음).

git ls-remote 결과

```
ae443aa4a498680ef56211faa6b993533019a6d2	refs/heads/v8/main
ae443aa4a498680ef56211faa6b993533019a6d2	refs/heads/v8/wip
```
(게이트 통과 직후. 보고 커밋을 올린 뒤의 결과는 최종 메시지 끝에 덧붙인다.)
