## 구현 완료 보고 — V8 M1

### 구현 의도
플레이어가 처음으로 정착지에 지시를 내린다. 나무 베기·비축 구역·오두막 청사진을 기록되는 명령으로 내리면, 사람들이 생존을 해치지 않는 선에서 일감으로 받아 통나무를 베고 나르고 쌓아 오두막을 짓는다. 이것을 192×192 colony 세계와 새 탑다운 화면에서 눈으로 볼 수 있게 한다.

### V8-0 화면 검증 (Part 0)
`tools/v8/verify_v8_0.gd`를 창 모드(opengl3, 1920×1080)로 V7 게임(`v7-final-art` = `b1ca032f`)과 V8-0(`24e16677`)에 똑같이 돌렸다. 시드 20261010, 틱 1,440과 4,320에서 확인했다. 자세한 내용은 `docs/v8/m1/V8_0_SCREEN_CHECK.md`에 있다.

| # | 확인 | 기준 V7 값 | V8-0 값 | 판정 |
|---|---|---|---|---|
| S1 | 신의 손 제거 | 노드 3개 (`UI/GodHandPanel`, `GodHandEffects`) | 0개 (시작·끝) | 통과 |
| S2 | 오버레이 제거 | 시작 시 15개 | 0개 (노드 1,288 / 1,332개 검사) | 통과 |
| S3 | 같은 세계 | t1440: 인구 26, 거처 4, 식량 15, 장작 31 / t4320: 29, 4, 34, 25 | 값이 같고, 사람 id·타일 목록 해시도 같다 | 통과 |
| S4 | 사람 클릭 | 상세 창: "유루게 · #8", "어른 · 나이 7일", "1번 마을" | 같음 | 통과 |
| S5 | 타일 클릭 | (22,19)에서 '왜?' 패널, 사건 2개 | 같음 | 통과 |
| S6 | 연대기 | `C` 키, t4320에 32개 중 6개 표시 | 같음 | 통과 |
| S7 | 콘솔 | 오류 0 | 0 | 통과 |
| S8 | FPS (3배속 120초) | 102.37 / 102.42 | 111.00 / 109.98 | 통과 (×1.08 / ×1.07, 다른 부하가 있던 측정) |

- 스크린샷과 원본 JSON(커밋 안 함): `/Users/rexxa/work/m1-data/v8_0_check/` (`v8_*.png`, `v7_*.png`, `pair2/`).
- 문서 Q3(브리지 읽기 추가)·Q7(시간 척도) 재검증:
  - 1차(Part 0): 내용은 맞았고 빈 곳 26개 → 고쳤다 (D7).
  - 2차(Part 6, 12틱/초 반영 뒤): 두 답 모두 맞았고 빈 곳·모순 31개를 찾았다. 그중 V8-0 기록 문서와 VISION(오너 원문)을 뺀 나머지를 고쳤다 (D28). 두 번까지라는 제한 안에서 끝냈다.
- `.uid` 정리:
  - `*.uid` 무시 규칙을 지우고 `.uid` 42개를 커밋했다. 짝 잃은 12개는 저장소 밖으로 옮겼다 (D2–D4).
  - `check_docs.sh` 검사 8과 게이트 20단계가 커밋된 UID를 확인한다.
  - `.gitignore`는 `docs/` 아래 증거 이미지만 무시한다 (`docs/images/`는 예외). 오너의 로컬 `/graft/` 줄은 스테이지하지 않았다.

### 구현 내용
- **새 데이터** (`rust/crates/sim-core/src/colony/`):
  - 통나무 `ItemKind::Log`, 땅의 더미 `ItemStack`(최대 25), 운반 `Carry`(최대 5), 장부 `ItemLedger`;
  - 나무 `Tree`(통나무 3–5개), 베기 표시 `Designation`, 비축 구역 `Zone`;
  - 오두막 청사진 `ColonySite` + `MaterialNeeds`(3×3, 통나무 12, 진행 240, 침대 4);
  - 예약 `ReserveKey`(Tree/Stack/Cell/Site).
  - 모든 상태는 `SimResources::colony` 하나에 있다.
- **명령** (`sim-systems/src/commands.rs`): 베기 지정(사각형 / 타일 목록), 영역 취소, 비축 구역 추가, 구역 지우기, 청사진 놓기. `submit`이 바로 적용하고 결과 `{accepted, count, reason}`를 돌려주며, 다음 틱 번호로 `command_log`에 기록한다 (ADR 0012).
- **일감** (`sim-systems/src/runtime/jobs/`):
  - `JobAssignSystem`(124)이 Build/Deliver > Chop > Haul 순으로 가장 가까운 대상을 예약해 준다.
  - `JobExecuteSystem`(127)이 걷기 → 일하기 → 나르기 → 내려놓기를 진행한다.
  - `cancel.rs`가 취소·완료·사망 때 예약을 풀고 들고 있던 것을 내려놓는다.
  - `invariants.rs`가 장부·예약·더미 불변식을 확인한다.
  - colony에서는 `ConstructionSystem`이 현장이 사라진(취소·완공) V7 `Seeking { ConstructionSite }` 사람을 Idle로 돌린다 (리뷰 H1, D29·D30).
- **colony 세계** (ADR 0014):
  - `WorldRules::colony(seed)`, `WorldGenConfig::colony(seed)`: 192×192, 창설자 10명, 나무 약 1만 그루, 덤불 96, 마른 나무 160.
  - 게임(`build_game_engine`)은 colony이고, `baseline`은 V7 그대로 동결되어 identity로 지킨다.
- **시간 속도** (D22, ADR 0004 틱 정의): 1배속 초당 12틱(하루 2분), 프레임당 최대 8틱. 1–4 키는 1–4배속(초당 12–48틱), P는 일시정지.
- **새 화면** (`scripts/world/colony/`, 노드 `ColonyView`):
  - 지형·나무·건물은 `TileMapLayer`이고, 주문 표시(베기 X, 구역, 청사진, 통나무 더미)와 사람(그림자, 가족 옷 색, 운반 통나무, 도구)을 그린다.
  - `order_tool.gd`가 드래그·클릭을 `cmd_*` 호출로 바꾼다. 아래 `order_bar.gd`(F1–F5)와 결과 알림이 있다.
  - 임시 타일셋은 코드로 그렸다: `tools/v8/gen_temp_tiles.py` → `assets/v8/temp/tiles.png`.
  - V7 렌더러 21개를 지웠다.
- **브리지**:
  - 추가: `cmd_designate_chop`, `cmd_designate_chop_tiles`, `cmd_cancel_area`, `cmd_add_stockpile`, `cmd_remove_zone_area`, `cmd_place_blueprint`, `can_place_blueprint`, `get_trees`, `get_colony_overlay`, `get_colony_versions`. 사람 스냅샷에 `jobs`, `carry_kinds`, `carry_counts` 열이 생겼다.
  - 삭제: `art_declump`, `art_walker_offsets`.
  - 목록: `docs/v8/bridge_api_m1.txt`.

### 구현 방법
- **명령 기록과 재생:**
  - 명령은 GDScript가 상태를 직접 쓰는 길이 아니다. `commands::submit` 하나로만 들어가고 `CommandRecord{tick, command, result}`로 남는다.
  - probe는 `--commands`로 스크립트를 읽고 `--commands-out`으로 적용된 기록을 쓴다. 그 기록을 다시 넣으면 `summary.json`이 바이트 단위로 같다 (게이트 19단계 `replay`, 단위 테스트).
- **예약과 취소:**
  - 일감 하나는 단계별로 열쇠를 예약한다 (`Job::keys`: 베기 = 나무, 나르기 = 더미 + 칸, 짓기 = 현장).
  - 끝날 때는 언제나 `jobs::cancel::{finish, cancel, release_job, on_death}`를 거쳐 예약을 푼다.
  - 취소 이유: 욕구(Needs), 대상 사라짐, 명령, 막힘(240틱), 중단, 사망. 들고 있던 통나무는 그 자리에 놓인다(병합, 넘치면 주변으로).
- **결정 시스템 접점** (`rust/crates/sim-systems/src/runtime/decision/system.rs`, +26 −1줄, D10·D20):
  - ~L832: `jobs::holders`;
  - ~L1127: `hold_for_decision` (일하는 사람은 잠시 Idle로 판단);
  - ~L1453: colony에서 V7 기억 편향 건설 갈래 끔 (D20);
  - ~L1774: 여가 갈래 건너뜀;
  - ~L1999, ~L2134, ~L2638: `TargetKind::Job` 갈래 3개 (컴파일러가 강제);
  - ~L2661: `settle_after_decision`;
  - ~L2968: 후처리에서 일감 목표 유지.
  - 판단이 "할 일 없음 / 식사 대기 / 지금 서 있는 colony 현장 짓기"이면 일감을 지킨다. 그 밖의 생존·사회 선택이면 일감을 내려놓는다 (ADR 0013).
  - 지은 사람이 완공 순간 이미 다른 행동으로 바뀌었으면 그 상태를 건드리지 않고 일감만 끝낸다 (`finish_in_place`, 리뷰 L1).
- **통나무 장부:**
  - 생성 = 땅 + 운반 + 현장 납품 + 소비. 디버그 빌드에서는 매 틱 `JobExecuteSystem`이 확인하고, 테스트와 평가는 매 틱 검사한다.
  - 평가 32 시드 × 10일 = 460,800틱에서 위반 0.
- **보정** (D23, 시드 0–7):
  - 0차: U1–U3은 맞았지만 시드마다 1명이 탈수로 죽었다. 원인은 V7 기억 편향 건설 갈래가 없는 현장을 영원히 찾은 것이다 → colony에서 끔 (D20).
  - 1차: 8/8 시드 모두 사망 0. 수치는 하나도 바꾸지 않았다 (6번 중 2번 사용).

### 기능 설명
- **오너가 하는 것:**
  - F1 베기: 숲을 드래그하면 노란 X가 표시된다.
  - F2 비축 구역: 드래그하면 노란 테두리와 칠이 생긴다.
  - F3 오두막: 마우스를 따라 3×3 그림자가 다니고, 놓을 수 있으면 초록, 없으면 빨강이다. 클릭하면 "통나무 0/12"가 나온다.
  - F4 구역 지우기, F5 취소(드래그).
  - 1–4 속도, P 일시정지, 휠·`-`·`=` 확대(1–4배), WASD·화살표 이동, Home/H 캠프로 이동.
- **사람들이 하는 것:**
  - 한가하면 일감을 받는다: 현장에 통나무 나르기·짓기 > 베기 > 비축 구역으로 나르기.
  - 배고프거나 목마르거나 졸리면 일을 내려놓고 생존을 챙긴다. 들고 있던 통나무는 그 자리에 둔다.
  - 오두막이 완성되면 연대기에 "오두막이 완성됨"이 뜨고, 밤에 4명이 거기서 잔다.
- **화면에서 보이는 것:**
  - 나무가 베이면 바닥이 숲 바닥으로 바뀌고 통나무 더미가 생긴다.
  - 나르는 사람 머리 위에 밝은 통나무 묶음, 베는 사람에게 도끼가 보인다.
  - 위 막대: 거처(오두막 포함), 건설 중(청사진 포함), 비축 식량, 장작, 통나무(땅의 더미), 야생 열매. 시계는 새벽·낮·저녁·밤.
  - 주문하면 아래에 "나무 16그루 베기 표시" 같은 알림이 뜨고, 청사진 위에 "통나무 d/12 · 작업 p%"와 막대 두 개가 보인다.
  - 사람을 클릭하면 "지금: …", "들고 있음: 통나무 n"이 보인다.

### 평가 결과 (시드 100–131)
보정에 쓰지 않은 새 시드 32개로 평가했다. 공식 결과는 Part 6 리뷰 수정 뒤 최종 시뮬레이션 코드(`8626fdc9`)로 다시 돌린 값이다 (`$DATA/eval2/`). Part 5의 첫 실행도 같은 판정이었다. U-스크립트(`tools/v8/u_script.json`)는 06:00에 다음 세 주문을 낸다: 가까운 나무 15그루 베기, 화덕 동쪽 4×4 비축 구역, 화덕 남쪽 오두막. 시드별 값은 `docs/v8/m1/M1_EVALUATION.md` 부록에 있다.

| # | 기준 | 결과 | 판정 |
|---|---|---|---|
| U1 | 놓은 뒤 3일 안에 오두막 완성 ≥ 28/32 | 32/32 (369–1,220틱, 중앙값 456) | 통과 |
| U2 | 4일 안에 15그루 모두 벰 ≥ 28/32 | 32/32 (345–1,473틱) | 통과 |
| U3 | 5일째 모든 통나무가 구역·운반·납품 중 ≥ 28/32 | 32/32 (구역 밖 0) | 통과 |
| U4 | 매 틱 장부 정확 32/32 | 32/32 (460,800틱 위반 0) | 통과 |
| U5 | 매 틱 예약 불변식 32/32, 막힘 취소 0 ≥ 30/32 | 32/32, 32/32 | 통과 |
| U6 | 10일 동안 굶주림·탈수·추위 사망 0 (스크립트 있음 / 없음) | 32/32, 32/32 (사망 0) | 통과 |
| U7 | colony 틱 평균 ≤ 0.6 ms, p99 ≤ 4 ms; 2배 확대 ≥ 60 FPS | 0.0727 ms, 0.107 ms; 120.00 FPS (1배 확대 119.92) | 통과 |
| U8 | baseline identity 그대로 | 통과 (p1, p5, p6; 최종 게이트 10단계: 228,748값 차이 0, 연대기·원시 표본 같음) | 통과 |
| U9 | GUI 테스트 처음부터 끝까지 | 15/15 (최종 화면·코드, 오두막 틱 718) | 통과 |

- 결정론: 32 시드 모두 스크립트 유무와 관계없이 `DETERMINISM PASS`.
- 눈에 띄는 점: 모든 시드에서 10일째 29명이다 (출생 19). V7 달력의 출산이 그대로 남아 있기 때문이며, M1.5에서 다시 맞춘다.

### 성능
- **colony 틱:** 시드 100–103, 10일, 스크립트, 3회 반복의 중앙값으로 평균 0.0727 ms, p99 0.107 ms (반복별 0.0722 / 0.0727 / 0.0730). 부하 5.1–6.0이라 경합이 있던 측정이다.
- **baseline 대비:** baseline 자연 세계 평균은 0.2513 ms다 (`M1_BASE_NUMBERS.md`, 18–40명). colony는 10–29명이라 같은 조건의 비교가 아니다.
- **Godot FPS** (릴리스 라이브러리를 디버그 경로에 복사해 측정하고 되돌림, 4배속 120초씩, 최종 화면): 2배 확대 120.00, 1배 확대 119.92. 120 Hz 화면 상한에 걸렸으므로 여유분은 이 방법으로 잴 수 없다. 나무 임포스터(9.5)는 필요 없었다.
- **게이트 perf 단계** (baseline 자연 세계, 기준 `24e16677` 같은 기계): 평균 0.2513 → 0.2475 ms (×0.985), p99 0.5092 → 0.4945 ms (×0.971), quick 0.3644 → 0.3552 ms (×0.975). +10 % 넘는 항목 없음. 반복 1의 지문이 같아 같은 세계를 쟀다.

### 변경된 파일 목록
- `9d9c2c02` (p0): 목표 문서, 참조 목업 `docs/images/v8_target_view_b.png`, `.uid` 42개 커밋, `.gitignore`, `check_docs_helpers.py uid`.
- `4944b5a0` (p0): `docs/v8/m1/V8_0_SCREEN_CHECK.md`, `M1_BASE_NUMBERS.md`, `tools/v8/verify_v8_0.gd`, ARCHITECTURE·스킬 문서 1차 수정.
- `ca69b86b` (p1–p3):
  - `sim-core/src/colony/*`, `rules/colony.rs`;
  - `sim-systems/src/commands.rs`, `runtime/jobs/*`, `worldgen/colony.rs`;
  - 결정 시스템 접점, `construction_system.rs`, `beds.rs`;
  - `sim-bridge/src/ffi/colony.rs`, `node.rs`;
  - `sim-test/src/probe/colony.rs`, 테스트 `colony_*.rs`, `engine_colony_ffi.rs`;
  - `tools/v8/compare_chronicles.py`, `u_script.json`, `docs/v8/bridge_api_m1.txt`.
- `7e712860` (p4):
  - `scripts/world/colony/*.gd`, `scripts/ui/hud/order_bar.gd`, `camera_controller.gd`;
  - 현지화 키, `tools/v8/gen_temp_tiles.py`, `assets/v8/temp/tiles.*`, `tools/v8/m1_gui_test.gd`;
  - V7 렌더러 21개와 `art_kernels.rs` 삭제.
- `3b9d4520` (p5): `docs/v8/m1/M1_EVALUATION.md`, ADR 0012–0014, ADR 0004 갱신, perf `--commands`.
- `6361ac9f` (p6): 게이트 17–20단계, `inventory.py`, CHANGELOG·ROADMAP·HISTORY·README·AGENTS.
- `4fc503d2` (p6): ARCHITECTURE (명령·일감·통나무 흐름, colony 화면 구성, 시스템 표).
- `2293b37c` (p6): 스킬 (명령 추가, 일감 종류 추가, 접점·장부 규칙, 합성 입력, colony 테스트 패턴, 교훈).
- `acefa4d7` (p6): 문서 Q3·Q7 2차 수정 (colony 읽기, 시간 척도 세부, 커밋되는 `.uid`, archive의 M1 절).
- `8626fdc9` (p6): 코드 리뷰 1차 수정 (H1 현장 잃은 사람 풀기, L1 `finish_in_place`, L2 assert, 8 시드 × 10일 불변식 테스트).
- `f742c552` (p6): 플레이어 리뷰 수정 (위 막대 수, 운반 묶음, 옷 색, 아이 눈, 진행 막대, 알림 문구, 저녁, Home/H, 마른 나무).
- `fda64089` (p6): CHANGELOG·README·ARCHITECTURE에 리뷰 수정 반영.
- `659191e5` (p6): 코드 리뷰 2차 (완공 경로 테스트, shelter_planning 조건), 평가 재실행.
- `6ea19569` (p6): U7 FPS·U9 GUI 테스트를 최종 코드로 재실행.
- 이 보고서 커밋: 아래 Gate 결과 다음에 `v8/main`을 한 번 더 빨리 감기한다 (최종 메시지의 해시 참고).

### 리뷰 결과
- **코드 리뷰 1차** (`$DATA/review/code_review_round1.md`): 높음 1, 중간 2, 낮음 7. 장부 구멍·예약 누수·비결정성·baseline 누출은 없었다.
  - H1(높음, 재현됨): colony 청사진을 취소하거나 다른 사람이 완공한 순간, 그 위에서 V7 `Seeking { ConstructionSite }`에 들어간 사람이 영원히 멈춰 결국 목말라 죽는다. → `ConstructionSystem`이 colony에서 그런 사람을 Idle로 돌린다. 취소·완공 두 경로 모두 테스트가 있고, 수정을 끄면 두 테스트 모두 실패한다.
  - M1(중간): 목표 13장의 "8 시드 × 10일 불변식" 테스트가 없었다 → `colony_invariants_hold_over_8_seeds_x_10_days`를 추가했다.
  - M2(중간): D20 한 줄(`decision/system.rs` ~L1453)은 4.3이 허용한 두 목적 밖이다 → 그대로 두고 오너 결정으로 올렸다.
  - L1 수정, L2 일부 수정(디버그 assert), L3–L7은 기록만 했다 (D29).
- **코드 리뷰 2차** (`$DATA/review/code_review_round2.md`): 높음 없음. 완공 경로 테스트 추가(M-A), 지키는 조건 고정(L-a), shelter_planning 조건(L-b)을 고쳤다. 나머지(L-c–L-g)는 기록했다 (D30). 두 번까지라는 제한 안에서 끝냈다.
- **플레이어 눈 리뷰** (`$DATA/review/player_review_round1.md`) 답:
  - 2배에서 사람과 땅·물건 구분: 대체로 된다(윤곽선과 그림자 덕분). 다만 옷 색이 땅과 밝기가 같았다 → 밝은 옷 색으로 고쳤다.
  - 누가 무엇을 나르는지: 안 보였다(2 px 갈색 줄) → 머리 위 밝은 통나무 묶음으로 고쳤다. 도끼는 겨우 보이고, 망치는 확인하지 못했다.
  - 무엇이 주문됐는지: 베기 X, 비축 구역, 청사진 그림자는 잘 보인다. 놓인 청사진의 진행 막대가 1 px였다 → 라벨 아래 2 px 막대 두 개와 % 표시로 고쳤다.
  - 새 플레이어가 처음 모를 것: 위 막대의 "식량 0"과 "열매 872"(지도 전체 야생 열매) → "비축 식량", "야생 열매"로 이름을 바꿨다. 오두막을 거처에 넣고 청사진을 건설 중에 넣었다.
  - 그 밖에 아이 눈, 알림 문구, 저녁 표시, Home/H, 마른 나무를 고쳤다 (`$DATA/review/player_fixes_round1.md`). GUI 테스트 15/15, strict check, smoke 0으로 확인했다.
  - 남은 것(아트·디자인 필요): 거처 그림(상자처럼 보임), 숲이 사람보다 눈에 띔, 아래 막대 아이콘, 지도 위 사건 표시(연대기 로그), 겹친 사람, 베기 X 색.

### 확인된 제한사항
- `get_construction_snapshot`은 `Position`이 있는 V7 현장만 센다. colony 청사진은 `get_colony_overlay`로 읽는다.
- 경로 찾기가 없다. 사람은 직선으로 걷고 막히면 240틱 뒤 취소된다 (평가에서 막힘 0).
- 타일셋은 임시이고 코드로 그렸다. 사람·나무·오두막 그림은 아트 패스에서 바꾼다.
- 남은 V7 코드:
  - 1칸짜리 V7 거처와 V7 건설 현장 경로가 그대로 있다 (colony에서는 shelter_planning이 꺼져 있다).
  - `scripts/world/` 아래 패널용 도우미(`art_hash.gd`, `art_sprite_library.gd`, `porter_presentation.gd`, `wood_visuals.gd`)가 남아 있다.
  - 출생·나이·달력은 V7 그대로다.
- 일감 교체가 잦다: 시드당 일감 약 150개를 받아 약 32개를 끝냈다. 나머지는 생존 때문에 내려놓았다가 다시 받았다. 걷는 시간이 낭비되지만 예약이 남은 적은 없다.
- 1,200줄이 넘는 파일 (실패는 아니고 보고만 한다):

  | 파일 | 줄 |
  |---|---|
  | `rust/crates/sim-systems/src/runtime/decision/system.rs` | 3,487 |
  | `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs` | 1,508 |
  | `rust/crates/sim-bridge/src/ffi/snapshot.rs` | 1,416 |
  | `rust/crates/sim-test/src/perf.rs` | 1,390 |
  | `rust/crates/sim-engine/src/lib.rs` | 1,312 |
- 드롭한 항목: 없음 (9.6 사람 상세의 일감 줄, 8.4 연대기, U7 4배 확대 FPS, 9.5 임포스터 모두 유지 또는 불필요).

### 오너 결정 필요
1. **다음 순서:** M1.5 달력을 M2보다 먼저 하기를 권고한다. 지금은 V7 달력의 출산 때문에 10일 만에 10명 → 29명이 되어, 일감·식량 보정이 달력에 묶여 있다 (ROADMAP "다음").
2. **키 충돌 (D24):** 목표 9.6은 1–5 = 주문 모드, 14.2는 1–4 = 속도라고 한다. 속도를 1–4에 두고 주문을 F1–F5로 옮겼다. 바꾸려면 알려 달라.
3. **결정 시스템의 D20 한 줄:** 4.3이 허용한 두 목적(일감 지키기, jobs/로 취소) 밖의 한 줄이다. H1 수정이 있어도 이 줄을 지우면 기억 편향 갈래가 Idle↔Seeking을 오가며 시간을 버린다 (치명적이지는 않음, 리뷰 2차 L-c). M1에서는 남기고, M2에서 지울지(갈래 자체를 colony에 맞게 다시 쓰기) 결정이 필요하다.
4. **리뷰가 남긴 화면 문제** (아트·디자인 필요):
   - 거처 그림이 상자처럼 보이고, 그 위에 선 사람이 액자처럼 보인다;
   - 숲이 사람보다 눈에 띈다 (VISION의 사람 > 물건 > 바닥 > 땅 순서와 반대);
   - 아래 막대에 아이콘이 없다;
   - 지도 위에서 사건(출생 등)이 보이지 않는다 (연대기 로그 패널이 필요);
   - 겹친 사람이 덩어리로 보인다.
5. **출생 폭증:** V7 달력의 출산 때문에 10일 만에 29명이 된다. M1.5에서 해 단위로 다시 맞출 때 목표 인구 곡선을 정해 주면 좋다.
6. **V7 그림:** 쓰이지 않는 V7 사람·건물 스프라이트와 `palette_swap` 셰이더를 지울지 아트 패스로 넘길지.

### Gate 결과
`bash tools/v8/gate.sh /Users/rexxa/work/m1-data/gate/final --base-data /Users/rexxa/work/m1-data/gatebase`를 커밋 `6ea19569`에서 돌렸다. 도중에 고친 것은 없고 `dirty 0`이며, 2026-10-21 00:02–00:56 KST에 20단계가 모두 통과했다. 그 뒤 `v8/main`을 `6ea19569`로 빨리 감기했다. 기준 데이터는 `24e16677`의 identity와 perf다 (`gatebase`는 `$DATA/base/identity/*`와 `$DATA/base/perf`의 심볼릭 링크).

| # | 단계 | 결과 |
|---|---|---|
| 1 | fmt | 통과 |
| 2 | bridge_build | 통과 |
| 3 | godot_import | 통과 |
| 4 | test_debug | 통과 (1,782개 통과, 실패 0) |
| 5 | test_release | 통과 (1,782개 통과, 실패 0) |
| 6 | clippy | 통과 |
| 7 | forbidden_math | 통과 |
| 8 | gdscript_strict | 통과 |
| 9 | smoke | 통과 (오류·raw key 0) |
| 10 | identity | 통과 |
| 11 | determinism | 통과 |
| 12 | perf (보고만) | OK (+10 % 넘는 평균 없음, ×0.985) |
| 13 | bridge_api (`docs/v8/bridge_api_m1.txt`) | 통과 |
| 14 | docs | 통과 |
| 15 | inventory_closure | 통과 |
| 16 | python_unittest | 통과 |
| 17 | m1_tests | 통과 (36개 + `Locale.trf` 테스트) |
| 18 | colony_determinism | 통과 |
| 19 | replay | 통과 |
| 20 | uid_check | 통과 |

로그: `/Users/rexxa/work/m1-data/gate/final/` (`gate.txt`, 단계별 `*.log`).
