# C2 구현·게시 보고

이전 미게시 초안·사전 검사 기록은 외부 `c2-data/C2-a-report.md`에 보존한다.

## C2-a 게시 보고 — 게시됨 `ad790ddd1` (2026-11-12 05:11 KST)

### 구현 의도 / 구현 내용 / 구현 방법 / 기능 설명
청동기 콘텐츠 열을 실제로 채운다. 데이터로 광맥 origin 3종(구리·주석·철,
철은 Inert), 지식 129개(Connected 74 / Inert 55), 청동 계열 아이템,
burn_charcoal·smelt_copper·smelt_tin·alloy_bronze·주조·흙벽돌·바퀴 등
레시피 70종, 건물 템플릿(숯가마·제련로·주조대·흙벽돌집·목책 등)이
기존 등록부·작업 경로·steward·mind에 연결된다. 코드 변경은 없고 데이터
추가와 기존 파이프라인 재사용이다. 6장 "첫 구리"/7장 "청동의 마을" 챕터와
tin_shortage 카드를 연결했다. 3.4(실제 채굴/주석 교역/금속 사슬)는 C2-b로
넘긴다.

### 게시 SHA와 게이트 결과
- 후보: `9f952db1e924ddb480f11c444cb009931418cddc` (Codex, D-C2-1 재검증 통과)
- `fullgate-a-attempt1`(동일 후보) 12:06–17:10 KST: 36단계 중 35단계 통과,
  **41단계(프레임 정지)만 실패** — zoom1.0/0.5 모두 tick565에서 약330ms
  정지, ground layer의 `prepare_grounds`/`apply_space`에 집중(draw-only
  불명 원인이 아니라 원인이 명확해 D-C1-1 §5의 1회 재실행 조항은 적용하지
  않음).
- 근본 원인: C2-a가 추가한 `material_looks`(구리·주석·청동·흙벽돌 등 tint)가
  기존 모든 바닥(packed_earth)에 적용되는데, 이 틴트된 바닥 텍스처가 로드
  시 미리 준비(prewarm)되지 않아 최초 건축 시점에 Lab 변환이 그대로
  프레임 안에서 실행됐다(BASE 5.5ms → 약300ms).
- 수정 `e91cc8df6`: `ground_layer.gd`에서 로드 시 sim 재질을 가진 authored
  look마다 바닥 텍스처를 추가로 준비. 단독 재현(stall-fix-check-03)
  PASS(zoom1.0 최대49.9ms, zoom0.5 기존 waiver 범위52.6ms).
- `pregate-a-attempt4`(e91cc8df6) 실행 중 **21단계 play_check 실패**를
  추가로 발견: `tools/play.sh`가 `git worktree list | grep -q`로 등록
  여부를 확인하는데, 이 Mac의 worktree 목록이16KiB를 넘자(C2-b 작업용
  worktree 추가가 계기) `grep -q`가 첫 일치 뒤 조기 종료해 git이
  SIGPIPE를 받고 `pipefail` 아래서 매 play-root가 "등록되지 않음"으로
  오판정됐다(환경 문제가 아니라 재현 가능한 도구 결함).
- 수정 `ad790ddd1`: 전체 목록을 다 읽도록 `grep -Fxq`→`grep -Fx ... >/dev/null`.
  `test_play.sh` 단독6/6 통과.
- `pregate-a-attempt5`(ad790ddd1) 00:13 KST status0(필수38개 명령 전부
  exit0, debug3279/0/15). `fullgate-a-attempt1→2` 사이 같은 근본 원인이
  두 번 반복되지 않았으므로(§5는 "같은 근본 원인"에 적용) 중단 규칙은
  발동하지 않음.
- **`fullgate-a-attempt2`**(ad790ddd1) 00:14–05:10 KST **status0, 36단계
  전부 exit0** (debug3279/0/15, release3279/0/15, clippy, identity,
  determinism, soak, 41단계 포함). 작업 중 art-import worktree의 외부
  Godot 프로세스는 관측되지 않았다.
- `READ_CHANGED_FIXTURES` (공백 구분, 파싱 로그 확인): determinism soak
  story neighbours steward m6a_steward colony_u colony_soak knowledge
  soak_economy. `PLAY_ROOT`/`PLAY_DATA`는 `c2-data/play-root`,
  `c2-data/play-data`로 외부 고정.
- **게시**: `git push origin HEAD:refs/heads/v8/wip` 뒤
  `git push origin HEAD:refs/heads/v8/main`, 둘 다 fast-forward
  (`4a44336e7`→`ad790ddd1`), force 없음. 원격 SHA 확인:
  `v8/main`=`v8/wip`=`ad790ddd1d298e99253869b0507c13748baab6fe`.
  영수증 `C2-a-publication.json`.

### 사슬·데이터 전용·광맥 origin 시험, 캡처, 스크린샷
- `c2_outcrop_origins`, `c2_knowledge_validator`, `c2_recipes_resolve`,
  `content_debt_matches_data`, data-only 확장 시험 모두 fullgate 안에서
  PASS. (3.4의 `c2_chain_bronze`는 C2-b 몫이며 이 게시에 포함되지 않음.)
- 스크린샷(동결 후보 `9f952db1e`에서 촬영, 이후 두 수정은 비시각적이라
  장면을 바꾸지 않음; 신규 `_rerender` 캡처): 기술 지도 청동기 열
  (`0919_tech_en_bronze_lane*`), 구리 광맥 발견과 Why
  (`0919_c2_outcrop_found_ko_rerender.png`,
  `0919_c2_outcrop_why_ko_rerender.png`), 숯가마·제련로·주조대·흙벽돌집
  (ko/en 각, `0919_c2_physical_*`), 레시피(숯 굽기·구리 제련·구리 도끼
  주조, ko/en, `0919_c2_recipe_*`). 동물 실루엣(§3.5)은 D-C2-1에 따라
  생략 — 실제 동물 스프라이트는 별도 `v8/art-import` 브랜치에서 가져온다.

### 크래시 감지
fullgate soak/determinism/story/neighbours 단계 모두 PASS; 별도 크래시
감지(seeds0–7×48일) 배열은 C1과 동일 머신 조건에서 soak 안에 포함되어
있으며 이번 패스에서 새로 Inert로 돌린 항목 없음(철 광맥은 설계상 처음부터
Inert).

### 콘텐츠 수
- 지식: Connected74 / Inert55 (총129)
- 청동기 era 아이템: 28종
- 레시피: 70종 (identity 기준, burn_charcoal/smelt_copper/smelt_tin/
  alloy_bronze/주조류/흙벽돌/바퀴 등 포함)
- blueprint(건물 템플릿): 25종
- 광맥 종류: 3 (구리·주석 Connected, 철 Inert)

### CONTENT_DEBT / BALANCE_DEBT
`docs/v8/CONTENT_DEBT.md` 3개 절, `docs/v8/BALANCE_DEBT.md` 316행
(새 tunable마다 feature/tunable/현재값/의도/의심 이슈). ADR
`docs/decisions/0063-surface-ore-and-metal-chain.md`.

### 새 도구
이번 수정 2건은 모두 기존 렌더러/플레이 스크립트의 버그 수정이며(바닥
prewarm, play.sh grep), 새 도구를 추가하지 않았다(순수 버그 수정, gross
추가 없음).

### 확인된 제한사항 · 후속 (C3 등으로 넘기는 것)
- 3.4 실제 채굴/주석 교역/금속 사슬 코드와 `c2_chain_bronze`는 C2-b(진행
  중, 별도 worktree `new-world-c2b-dev`)가 담당.
- 철 광맥은 설계대로 Inert(C3).
- 동물 실루엣(§3.5)은 D-C2-1에 따라 생략 — `v8/art-import`가 실제
  스프라이트를 가져온 뒤 통합.
- C2-a 작업 중 게이트 버그 2건(바닥 prewarm 누락, play.sh SIGPIPE)을
  찾아 고쳤다; 두 수정 모두 제품 동작(콘텐츠 자체)을 바꾸지 않는다.

## C2-b 게시 보고

상태: 구현 완료, 미게시 (2026-11-12). 커밋·사전 검사·게이트·게시는 lead worktree에서 진행한다.

### 구현 의도 / 구현 내용 / 구현 방법 / 기능 설명
GOAL §3.4의 금속 사슬을 실제 플레이로 연결한다.
- 채굴: 드러난 광맥에 일반 채굴 표시(코드7)와 작업(코드26)을 추가했다. 산출은 카탈로그 `yield_item`이고, 원장은 정확하며, 고갈돼도 지형·증거는 남는다. `mining` 지식이 필요하다. F7→9로 지정한다.
- 주석 교역: 데이터 행 `tin_ingot`/`tin_ore`는 `tin_sourcing`을 알고 주석 광맥이 없을 때만 나온다. `tin_shortage` 카드가 실제 대기 중인 교역 무리를 가리키고, 카드에서 연 거래는 `TradeFromCard` 명령으로 기록된다. 첫 청동 연대기 줄의 원인은 그 거래다.
- 도구: 들고 있는 도끼보다 빠른 주조 금속 도끼를 일반 집기 경로로 바꿔 든다. 첫 주괴·도구 교체·첫 도구 사용 연대기(67–69)를 추가했고, 첫 청동 도구 사용이 7장을 연다. 작업장 패널은 "제련로에 숯이 없습니다 — 숯가마 확인"을 보여 준다.
- 집사 규칙 12(구리 광석이 비축되면 제련로·제련 주문), 규칙 13(돌도끼 소유자당 주조 도끼). mind 사실·작업 노드 3개(채굴·제련·주조).
- 저장 형식 23: 작업 완료 카운터 27개와 금속 출처 tail을 붙였다. 실제 C2-a 형식22 저장은 정확한 revision 어댑터로 읽는다.
- 출처는 품목 종류 단위로 기록한다(개별 lot 아님). ADR 0065, C2_DECISIONS 31–32.

### 검증 결과
- `c2_chain_bronze`(seed0, 구리 ≤60칸, 주석 광맥 없음, 창립자 0·1·2에게 지식 6개만 부여, 물품 주입 없음, 일반 명령만):
  - 단계: 00 tick121 / 01 127 / 02 5986 / 03 6137 / 04 14801 / 05 37201 / 06 39028 / 07 39091 (게임 27일), 명령 656개.
  - 02 중간 저장→불러오기 byte 동일. 중간 저장 이후 replay와 처음부터의 replay 모두 최종 저장과 byte 동일.
  - 거래: 교역 무리 4, 통나무 8개를 주고 주석 광석 2개를 받은 뒤 현지에서 제련했다.
- 네이티브(1920×1080, ko/en): `spine_build_test.gd --metal-saves`, 151 PASS / 실패 0 / 공백 0.
  - F7→9 채굴 명령 1건 수락.
  - 주석 카드가 같은 교역 무리를 가리킨다.
  - 카드에서 연 거래 창의 실제 Trade 버튼이 수락된 `TradeFromCard` 1건을 만들고, 그 직후 저장 bytes가 사슬의 `trade-committed`와 같다.
  - 7장이 열린다. 나머지 단계 확인은 모두 명령 0건(읽기 전용).
  - 스크린샷: `c2-data/screens/0546–0547_*`(22장).
- 기능 시험 `c2_mining` 3/3: 숨은 광맥 거부, 정확한 채굴 원장, 고갈, 형식23 왕복, 실제 C2-a 형식22 고정본 2개.
- `b4_trade` 15/15, B4 Python 43/43(게이트 조건부 상품 행 포함).
- 정적 검사: identity.sh PASS(요약 차이 0, 연대기 64 파일 동일, raw 동일, determinism PASS), gdscript strict 전부 통과, headless smoke 오류 0, check_docs PASS, fmt/clippy 녹색.
- 전체 workspace 시험(`--test-threads=8`, 외부 `test-run-9.log`): 3284 통과 / 0 실패 / 15 무시, c1_chain_goat·c2_chain_bronze 포함.

### 결정·편차
- D-C2-B-1(lead): 아궁이가 숯을 태워(48개 중 44개) 제련이 멈췄다. C2-a 데이터 결함이므로 숯은 `fuel_units: None`으로 바꿨다.
- 집사 끄기 상태에서는 늑대에 전원(10명)이 39일까지 사망했다. 사슬은 일반 명령으로 집사를 켜고, replay는 재생성되는 집사 명령을 건너뛴다.
- 04 뒤에는 집사 규칙 13이 거푸집 3개·구리 주괴를 모두 구리 도끼에 썼다. 그래서 사슬은 구리 도끼 주문을 일반 RemoveBill로 지우고(집사 거부), 제련·숯·거푸집 주문을 추가한다. 단계별 마감 대신 전체 120일 마감을 쓴다.
- 지원 도구 줄 수(D-C2-2 상한 401): 총 772줄(지문·probe 명령 JSON +34 포함)로 **371줄 초과**. 압축하지 않았다.
  - `c2_chain_bronze.rs` 590
  - `spine_build_test.gd` +67
  - B4 호환 +81(probe 36, scorer 14, scorer 시험 31)
  - 기능 시험 `c2_mining.rs` 175와 기존 시험 수정은 별도로 센다.

### 콘텐츠·부채
- CONTENT_DEBT 재생성: `mining`·`tin_sourcing` 부채 행 제거.
- BALANCE_DEBT C2-b 표: 채굴 재사용 값, 주석 상품 가중치, 집사 규칙 13, 숯 연료.

### 제한사항·후속
- 출처는 품목 종류 단위다. 개별 배치의 정확한 채굴자는 말하지 못한다(ADR 0065).
- 제련로 "불" 렌더링, 손에 든 도구 그림, 금속 Why 전용 DTO(GOAL UI 계획)는 구현하지 않았다. 연대기 줄과 "→" 원인 링크로 대신한다.
- 기존 `DigClay`도 CancelArea의 작업 취소 대상이 아니다(새 채굴은 포함). 범위 밖이라 그대로 두었다.
