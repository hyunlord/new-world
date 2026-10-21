## 구현 완료 보고 — M1-ART

### 구현 의도
오너가 M1을 해 보고 "그림이 너무 조잡해서 게임을 판단할 수 없다"고 했다(2026-10-21). 팔레트는 목업과 같았지만 그 위의
그림은 전부 임시였다: 숲은 타일마다 같은 원이라 초록 점으로 보였고 사람을 가렸고, 오두막은 회색 네모, 거처는 상자,
통나무 34개는 작은 네모 넷, 사람은 테두리 없는 16픽셀 막대였다. 이번 패스는 M1에 이미 있는 것(땅, 나무, 오두막, 통나무,
비축 구역, 화덕, 사람)을 목업처럼 그리는 것이다. 시뮬레이션은 바꾸지 않았다(`rust/` 변경 0줄).

### 구현 내용 (시트·레이어·UI·버그, 파일별)
- **시트** `tools/v8/gen_tiles.py` → `assets/v8/tiles.png`(512×304) + `tiles.json`: 코드로 그린 시트 하나. 풀 두 무리
  (각 3종), 숲 바닥 4, 밟힌 흙 4 + 풀 디더 가장자리 2, 언덕 3, 바위 3 + 밝은 윗면 3 + 절벽 1, 깊은 물 4, 얕은 물 4,
  둑 15(4비트 마스크) + 안쪽 모서리 4, 비축 바닥·테두리·말뚝; 나무 3종(참나무·자작나무·소나무)×3크기 24×24와 그림자,
  그루터기; 통나무 더미 3크기; 오두막 48×48(통나무 벽 두 겹, 나이테, 남쪽 문, 흙바닥, 잠자리 4, 화덕돌), 청사진(파란
  빗금 벽·문·가구 윤곽, 점선), 배치용 회색 유령, 가죽 지붕 움막 16×20, 화덕 2프레임; 사람(서기·걷기·잠 프레임, 어른·아이,
  피부·머리·옷 회색 레이어 + 윤곽 레이어 + 후광 레이어), 어깨 통나무(뒤 2개·앞 1개), 도끼·망치, 도끼 모양 베기 표시,
  선택·마우스 위 테두리, 명령 막대 아이콘 5개, 사람 팔레트. 테스트 `tools/v8/test_gen_tiles.py` 23개.
- **땅** `scripts/world/colony/terrain_layer.gd`: 월드 시드 값 노이즈로 풀 무리(경계는 해시로 들쭉날쭉), 나무와 그
  이웃 8칸 아래 숲 바닥, 캠프 공터를 노이즈로 깎고 풀 쪽으로 돌린 디더 가장자리, 물 오토타일(둑 → 얕은 물 → 깊은 물),
  바위 윗면과 절벽. 월드가 바뀔 때만 다시 그린다.
- **나무** `tree_layer.gd`: 16×16타일 청크 노드가 24×24 수관을 y순으로 겹쳐 그린다(남쪽이 북쪽을 덮음), 종·크기·±2px
  흔들림은 (x, y) 키 해시, 그림자는 2–4배에서만. 벤 나무 자리에 하루 동안 그루터기(버전 차이로 판단). 사람 레이어
  아래.
- **오버레이** `overlay_layer.gd`: 땅 높이(z 1) — 다진 흙 비축 바닥·테두리·말뚝, 개수별 통나무 더미, 청사진과 배달된
  통나무 더미; 위(z 4) — 베기 표시, 어두운 상자 위 "통나무 d/12 · 작업 p%" 라벨, 3px 막대(배달 중 호박색 → 건설 중
  초록), 마우스 위 더미 개수.
- **건물** `building_layer.gd`: 새 오두막, 움막, 0.4초마다 바뀌는 화덕 불.
- **사람** `agent_layer.gd`: 레이어 사람, 걸을 때 다리 번갈아, 서쪽으로 가면 좌우 반전, 어깨 통나무, 일할 때 도구,
  숲 속·밤에(불 밖) 밝은 후광, 밤에 화덕·오두막 근처는 따뜻한 색, 잠자는 사람은 누운 자세로 오두막 잠자리에
  (같은 칸에 여럿이면 펼쳐서).
- **기타** `daylight_controller.gd`: 화덕 빛이 반 칸 어긋나 있던 것을 화덕 중앙으로, 밤 곡선 공유. `order_tool.gd`: 배치
  유령, 영역 모드의 마우스 위 테두리.
- **UI·버그** `scripts/ui/panels/chronicle_panel.gd`: `NOTABLE`에 22(HutBuilt) 추가 → 연대기에 "1일째 · 미유나 — 1번
  마을에 오두막이 완성됨"이 보인다(캡처 P4). `UI_CAUSAL_PANEL_PLACEHOLDER`(en/ko)에서 "(γ-2-β)" 삭제.
  `scripts/ui/hud/order_bar.gd`: 버튼마다 12×12 아이콘(2배), 활성 모드 금색, macOS에서 "fn + F1–F5 …".
- **도구** `tools/v8/capture.gd`(샷마다 화면 위치 덤프, 아침 샷 `m1_10`), `contact_sheet.py`(`--columns`, 빈 칸),
  `m1_gui_test.gd`(`--no-vsync`), `inventory.py`(새 이름).

### 구현 방법 (노이즈 군집, 오토타일, 캐노피 겹침, 사람 레이어, 측정 방법)
- **노이즈 군집:** 목업의 `value_noise`를 GDScript로 옮겼다(`view.gd` `value_noise`/`cluster_noise`). 난수 대신
  (시드, 격자점) 정수 해시, smoothstep 보간, 0.65×(9칸) + 0.35×(4칸). 임계값 ±0.035 안에서는 타일 해시로 골라 경계를
  들쭉날쭉하게 했다. 타일마다 무작위 색은 쓰지 않는다(규칙 4).
- **오토타일:** 물 칸의 땅 이웃 4비트 마스크(N1 E2 S4 W8) → `bank_<mask>`; 마스크 0인데 대각선이 땅이면
  `bank_in_*`; 2칸 안에 땅이면 얕은 물; 나머지 깊은 물. 공터 가장자리는 "북쪽이 풀"인 타일 하나를 TileMapLayer의 실행
  시간 변환 플래그(`TRANSPOSE`, `FLIP_H`, `FLIP_V`)로 네 방향으로 돌렸다.
- **캐노피 겹침:** 타일맵은 칸을 넘는 그림을 못 그리므로, 트리 레이어 아래 청크 노드가 `draw_texture_rect_region`으로
  그린다. 청크는 행 순서로 만들어 남쪽 청크가 위에 그려지고, 청크 안은 (y, x) 정렬. 1배 확대에서는 그림자만 뺀다
  (16×16 대체 그림은 숲을 다시 점으로 만들어서 버렸다 — DECISIONS D9).
- **사람 레이어:** 회색 피부·머리·옷 레이어를 사람마다 `modulate`로 물들이고, 윤곽·눈·허리띠는 물들이지 않는 윤곽
  레이어에, 후광은 실루엣 바깥 1px 고리 레이어에. 프레임은 ASCII 표로 그려서 레이어로 나눈다(`figure_layers`).
  "잠"은 스냅숏 `states`=3(소비 중) & `target_kinds`=3(잠)으로 판단해 Rust 열을 추가하지 않았다(D1).
- **측정 방법:** 캡처 도구가 샷마다 사람·건물·더미·구역의 화면 좌표를 덤프하고, 그 좌표로 블라인드 조각을 자르고
  사람 찾기 결과를 맞췄다(반경 ¾칸, 일대일). 블라인드·찾기는 매번 새 하위 에이전트, 이미지만, 답은 파일로. FPS는 수직
  동기를 끈 채(`--no-vsync`) 같은 릴리스 라이브러리로 기준(`bba3110e` 워크트리)과 이후를 번갈아 쟀다.

### 검사 결과 (8.1 시트 경로, 8.2 블라인드 10문항 답과 점수, 8.3 사람 찾기 비율, 8.4 체크리스트 표, 8.5 FPS 전후, 8.6 identity)
**8.1 시트:** `docs/images/m1art_contact_sheet.png` (P1, P3, P4, P5, P6, P7, P9 각각 이전 | 이후, P3 줄 오른쪽에
목업 지도 부분; 매니페스트 `docs/v8/m1art/contact_sheet_m1art.json`). 이전 = `bba3110e`와 같은 코드(`a4a7eaf4`),
이후 = `086e395a`의 최종 캡처, 같은 시드 20261010·같은 틱. `docs/images/` 6장.

**8.2 블라인드 읽기 — 기준(≥ 8/10) 미달: 1차 6/10, 수정 1회 뒤 2차 7/10.** 엄격하게 채점(첫 답이 맞아야 함).

| 조각 | 정답 | 1차 답 (그대로) | 1차 | 2차 답 (그대로) | 2차 |
|---|---|---|---|---|---|
| 01 | 아이 | a villager (man in cream/brown striped shirt) standing idle | ✗ | a villager standing idle on bare dirt | ✗ |
| 02 | 화덕 | a campfire in a stone ring (lit fire) | ✓ | a campfire in a stone ring (fire pit) | ✓ |
| 03 | 밤에 자는 사람 | an unlit stone hearth (fire pit) inside a hut, between beds | ✗ | an unlit hearth (fire pit) inside a hut, with sleeping mats around it | ✗ |
| 04 | 맨땅의 통나무 더미 | a pile of chopped logs (woodpile) among tree stumps | ✓ | a small pile of cut logs among tree stumps (felled trees) | ✓ |
| 05 | 오두막 | a stone hearth with glowing embers in the middle of a hut (four beds around it) | ✗ (조각 설계 결함: 3×3 오두막 조각의 "가운데"는 화덕돌) | a wooden hut seen from above: four beds around a central hearth, door at the bottom | ✓ |
| 06 | 나무 베는 사람 | a villager (man in red) chopping a tree with a stone axe | ✓ | a man with an axe chopping a tree (tree marked for felling) | ✓ |
| 07 | 거처(움막) | a hut's thatched/hide entrance (tent-like lean-to on the hut wall) | ✓ (lean-to) | a woven basket / storage sack outside a hut wall (2순위: a hide tent) | ✗ |
| 08 | 통나무 나르는 사람 | a villager carrying a log | ✓ | a villager carrying logs (hauling wood) | ✓ |
| 09 | 청사진 | a designated zone / area outline (2순위: building plan / blueprint footprint) | ✗ | a planned hut (blueprint / ghost outline of a building to be built) | ✓ |
| 10 | 통나무 비축 | a stockpile of logs (stacked wood) | ✓ | a stockpile of logs (woodpile) | ✓ |

수정 1회(D19): 아이를 큰 머리·작은 몸 8×10으로, 잠 프레임을 머리+담요로, 청사진에 빗금 벽·문·가구 윤곽, 청사진
조각 중심 한 칸 보정, 질문을 "조각이 가리키는 주된 것"으로. 청사진·오두막은 고쳐졌고, **아이·잠자는 사람·움막은
2차에서도 틀렸다**(움막은 1차에 맞았다가 2차에 "바구니"). 목표는 수정 1회만 허용하므로 여기서 멈추고 미달로 보고한다.
1·2차 답 원본: `/Users/rexxa/work/m1art-data/blind1/answers.md`, `blind2/answers.md`, 정답 키 `blind*/key.json`.

**8.3 사람 찾기 — 기준 통과.** 1차(수정 전 그림): 2배 9/10 = 90 %(놓친 1명은 아래 명령 막대 밑, y 1052), 1배 10/10 =
100 %, 오표시 0. 2차(최종 그림, 새 에이전트): 2배 10/10 = 100 %, 1배 10/10 = 100 %, 오표시 0. 기준 2배 ≥ 90 %,
1배 ≥ 70 %.

**8.4 목업 비교:** `docs/v8/m1art/CHECKLIST.md` — 25행 모두 "있음", M1에 없는 6행(밭, 동굴, 마을 오두막 가구, 옆
패널 내용, 작업장·다른 물건, 동물)은 "M2+". 단, "있음"은 그려져 있다는 뜻이고, 8.2에서 아이·잠·움막은 낯선 사람이
알아보기에 아직 부족했다. 노인 머리는 샷에 노인이 없어 코드 경로로만 확인.

**8.5 FPS (수직 동기 끔, 4배속, 줌마다 120초, 같은 릴리스 라이브러리 SHA `85c47662`):**

| 회차 | 2배 이전 | 2배 이후 | 1배 이전 | 1배 이후 | 1분 부하(이전 / 이후) |
|---|---|---|---|---|---|
| 1 (유효) | 528.1 | 638.5 | 405.9 | 503.6 | 13.6 / 19.9 |
| 2 (무효, D18) | 112.4 | 멈춤 2회 | 92.8 | — | 7.2 / — |

1회차: 이후 ≥ 60 FPS(양쪽 줌), 이후/이전 = 1.21(2배), 1.24(1배) ≥ 0.9 → **통과**. 2회차는 이전 실행이 6분짜리 일에
약 2시간이 걸렸고 이후 실행은 프레임 대기에서 두 번 멈춰서(창이 정상적으로 그려지지 않음) 쓰지 않았다. 따라서 근거는
번갈아 잰 1회 한 쌍뿐이다. 측정 커밋은 `5cf736c6`+작업 트리(수정 회차 전 스프라이트; 노드·쿼드 수는 같다).

**독립 검토(9장):** 새 하위 에이전트가 diff를 직접 확인 — `rust/` 변경 없음; 매 프레임 작업 중 지도·나무 수에
비례해 새로 늘어난 것은 없음. 높음 1건(화덕이 등록되기 전 지형 전체를 매 프레임 다시 그림, 기존 구조를 이번 패스가
무겁게 만듦)과 중간 1건(잠자는 사람 처리)을 고쳤다(D20). 사람 수에 비례하는 중간 항목 2건은 10–30명 규모라 그대로
뒀다. 보고서: `/Users/rexxa/work/m1art-data/review/REVIEW.md`.

**8.6 시뮬레이션 불변 — 통과:** `git diff --stat bba3110e..HEAD -- rust/` = 비어 있음. 게이트(`2f701621`)에서
identity(10단계, 32세계×120일) 통과, determinism 통과, colony_determinism·replay 통과.

### 변경된 파일 목록 (커밋별)
- `a4a7eaf4` p0: `docs/v8/m1art/ART_GOAL.md`.
- `cebdeb97` p0: `docs/v8/m1art/CHECKLIST.md`, `ART_DECISIONS.md`.
- `8fef7d42` p1: `tools/v8/gen_tiles.py`(← `gen_temp_tiles.py`), `tools/v8/test_gen_tiles.py`(← `test_gen_temp_tiles.py`),
  `assets/v8/tiles.png`, `tiles.json`, `tools/v8/inventory.py`, `docs/v8/V8_0_INVENTORY.md`, `ART_DECISIONS.md`.
- `975325b7` p2: `scripts/world/colony/{view,terrain_layer,tree_layer,overlay_layer,building_layer,agent_layer,order_tool}.gd`,
  `scripts/world/daylight_controller.gd`, `tools/v8/capture.gd`, `tools/v8/gen_tiles.py`, `test_gen_tiles.py`,
  `assets/v8/tiles.*`, `assets/v8/temp/` 삭제, `ART_DECISIONS.md`.
- `5cf736c6` p3: `scripts/ui/panels/chronicle_panel.gd`, `scripts/ui/hud/order_bar.gd`, `localization/{en,ko}/ui.json`,
  `localization/compiled/{en,ko}.json`, `localization/key_registry.json`.
- `086e395a` p4: `tools/v8/gen_tiles.py`, `test_gen_tiles.py`, `assets/v8/tiles.png`, `scripts/world/colony/terrain_layer.gd`,
  `tools/v8/contact_sheet.py`, `test_contact_sheet.py`, `tools/v8/m1_gui_test.gd`, `docs/images/m1art_contact_sheet.png`
  (`m1_contact_sheet.png` 삭제), `docs/v8/m1art/contact_sheet_m1art.json`, `CHECKLIST.md`, `ART_DECISIONS.md`.
- p5 (문서): `CHANGELOG.md`, `README.md`, `AGENTS.md`, `docs/ROADMAP.md`, `docs/HISTORY.md`, `docs/ARCHITECTURE.md`,
  `docs/decisions/0015-colony-art-one-code-drawn-sheet.md`, `.claude/skills/worldsim-godot/SKILL.md`,
  `references/rendering-m1.md`, `.claude/skills/worldsim-pitfalls/SKILL.md`, `.claude/skills/worldsim-docs-upkeep/SKILL.md`,
  `docs/v8/m1art/ART_REPORT.md`.

새 현지화 키 1개: `UI_ORDER_HINT_IDLE_MAC` — ko "fn + F1–F5 명령 · Home/H 캠프로", en "fn + F1–F5 orders · Home/H camp".

### 제한사항 (드롭한 것, 아직 임시인 것, M2에서 그릴 것)
- 드롭 없음: 6.6 화덕 깜빡임, 6.5 잠 자세, 5.4 나무 3종, 7.3 명령 막대 아이콘 모두 했다.
- **블라인드 읽기 미달(7/10):** 아이는 여전히 "작은 어른"으로, 밤의 잠자는 사람은 오두막 화덕·잠자리 무늬에 묻히고,
  움막은 "바구니"로 읽혔다. 다음에 그릴 것: 아이는 어른 옆에서 더 작게(혹은 머리 색·옷 모양 차이), 잠자는 사람은
  밤에도 보이는 담요 색·머리 대비, 움막은 기둥과 입구를 키운 측면감.
- 1배 확대에서는 나무 그림자를 그리지 않는다(16×16 대체 그림은 쓰지 않음, D9).
- 청사진 라벨의 "작업 n%"와 초록 막대는 건설 중에 늦게 갱신될 수 있다(건설 진행이 오버레이 버전을 올리지 않음;
  이전부터 있던 동작, 시뮬레이션 쪽 수정이라 이번 범위 밖).
- 같은 칸에 자는 여럿은 그림에서만 펼친다(시뮬레이션은 한 칸).
- FPS 근거는 한 쌍(1회차)뿐이다(2회차 무효).
- M2+: 밭, 동굴과 바위 방, 작업장과 다른 물건(돌·부싯돌·열매·가죽), 동물, 가구 있는 마을 오두막, 옆 패널 내용.

### 오너 결정 필요
1. 블라인드 기준 미달(아이·잠자는 사람·움막)을 다음 패스(M1.5 전 짧은 수정) 에서 고칠지, M6 아트 본 생산까지 둘지.
2. 1배 원경에서 나무 그림자를 빼는 것(D9)을 받아들일지.
3. 다음 순서: M1.5 달력(권고 유지) 또는 M2.

### Gate 결과
`bash tools/v8/gate.sh /Users/rexxa/work/m1art-data/gate1` — 커밋 `2f7016216ecad63b37777a49b003e7d9efca11f1`, dirty 0,
2026-10-21 19:26:28 → 20:23:12, **status 0, 21단계 모두 exit 0**:
fmt · bridge_build · godot_import · test_debug(1782 통과, 0 실패) · test_release(1782 통과, 0 실패) · clippy ·
forbidden_math · gdscript_strict · smoke · identity · determinism · perf(평균 +10 % 초과 없음) · bridge_api · docs ·
inventory_closure · python_unittest · m1_tests(36 통과) · colony_determinism · replay · uid_check · play_check(6/6).
`v8/main`을 `2f701621`로 빨리감기했다. 이 보고서 커밋은 문서만 바꾼다(ROADMAP·HISTORY의 게이트 커밋 해시와 이 절).

git ls-remote 결과
```
2f7016216ecad63b37777a49b003e7d9efca11f1	refs/heads/v8/main   (게이트 통과 커밋)
(같은 해시)	refs/heads/v8/wip, v8/main — 이 보고서 커밋(문서만, 게이트 커밋 위 1개; docs 검사 통과 후 빨리감기)
```
