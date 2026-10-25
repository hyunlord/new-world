# 교훈 — 실패 기록을 다음 검증의 조건으로 남기기

> Stage C2 교훈 통합, 2026-10-25. B gate1 `60f3371691b398e16a9d3675ea3e36b058559616`의 실패와 D45 수정, B gate2 `5e20c65a1d52ba2a96a6e524614d3a7679f31a44`의 36/36 status0 및 v8/wip·v8/main 게시 영수증을 구분해 반영했다. 이 문서와 함께 추가한 skill 예방 규칙을 적용한다. 이번 편집은 기록 검토이며 시험 재실행이 아니다. C 전체 gate·게시·물리 입력·오너 수락 완료를 선언하지 않는다. 실패 원문과 후속 검증을 함께 보존하며 절대경로 링크는 저장소 밖의 로컬 원자료다.

## 1. 비교 기준도 측정 대상이다 — 혼잡한 벤치마크 원장

M3/M4에서는 부하 18.6–24.7에서 얻은 M2/M3 수치를 기준으로 삼아 퇴행이 가려졌다. 조용한 세션에서 번갈아 재측정하자 64² 장면의 평균은 ×1.156, p99는 ×1.319였다. 원장의 과거 열을 지우지 않고 `contended, not a base`로 표시하고 비교 대상에서 제외한 이유다.

예방은 같은 장비·프로필·절차·부하에서 기준과 후보를 교차 측정하고 부하를 함께 보존하는 것이다. [worldsim-testing의 Perf 및 Benchmark 규칙](../.claude/skills/worldsim-testing/SKILL.md#perf), [worldsim-pitfalls의 혼잡 원장 교훈](../.claude/skills/worldsim-pitfalls/SKILL.md), [M4 D3](v8/m4/M4_DECISIONS.md#d3--the-ledger-rule-and-the-quiet-m3-re-measure-goal-36)가 근거다. 원장 도구 수정은 과거 성능 실패를 PASS로 바꾸지 않는다. 원문이 가리키는 `$DATA/bench/RUN.txt` 원자료는 이번 문서 작업에서 다시 검증하지 않았다.

## 2. 게이트 결과는 그때 검사한 커밋에만 붙인다 — M-ART2 D19 동시 세션

[M-ART2.1 목표의 시작 조건](v8/m_art2_1/MART21_GOAL.md)은 첫 세션이 게이트를 돌리는 동안 두 번째 세션이 D19를 만들었고, **검사된 커밋 자체는 깨끗했지만** `a1c20220`은 검사되지 않았다고 기록한다. 이를 “원래 게이트가 혼합 트리를 검사했다”로 바꿔 적으면 사실과 달라진다. 뒤의 격리 검사도 33/34, bench 실패였으므로 그 중간 결과를 전체 통과로 인용할 수 없다.

예방은 [worldsim-pitfalls의 별도 worktree 원칙](../.claude/skills/worldsim-pitfalls/SKILL.md), [worldsim-testing의 Gate and determinism](../.claude/skills/worldsim-testing/SKILL.md#gate-and-determinism), [AGENTS의 main 승격 조건](../AGENTS.md#branches-and-pushing)이다. [M-ART2.1 D1](v8/m_art2_1/MART21_DECISIONS.md#d1--intake-part-0-2026-10-24-172x-kst)의 시작 점검과 격리 검사 기록을 함께 읽는다. 파일 소유권을 나눠 공동 작업하더라도 최종 검사 동안의 소스 동결과 결과의 커밋 귀속을 생략할 근거가 되지는 않는다.

## 3. 실행 명령은 현재 허용된 checkout과 target을 함께 전달한다 — 잘못된 play 명령

2026-10-24 잘못된 play 명령 사건은 [MUX C2 원래 요구](v8/mux/MUX_GOAL.md#c2-issue-registry-and-github)에 명시되어 있다. 그러나 이번 조사에서는 원래 잘못된 명령, 실행 출력, 대상 ref와 정확한 피해 범위를 확인하지 못했다. **보고된 사건 / 원본 증거 미확보**로 남기며 인접한 launcher 결함을 이 사건의 원인으로 대체하지 않는다.

예방 근거는 [PLAY2 D1·D7의 허용 checkout 및 금지된 Desktop 대상 규칙](v8/play2/PLAY2_DECISIONS.md), [MUX 실행 경계](v8/mux/MUX_GOAL.md)다. 명령을 안내할 때 현재 계약의 경로·버전·인자를 그대로 확인한다. PLAY2 D5는 `.ron`까지 포함한 의존성 판단을 Cargo에 맡겼으므로, 오래된 skill의 mtime 설명을 새 권장 절차로 복사하지 않는다. 후속 launcher 개선은 사건 원본의 복원이나 과거 피해 해소 증거가 아니다. PLAY2 원문 보존 링크: [67a6232a의 결정 기록](https://github.com/hyunlord/new-world/blob/67a6232ab41a4bf5051b11303bcac97543b111f0/docs/v8/play2/PLAY2_DECISIONS.md).

[worldsim-godot의 실행 경로·bridge 재빌드 규칙](../.claude/skills/worldsim-godot/SKILL.md)도 현재 계약과 함께 확인한다.

## 4. 시작 점검은 허용된 목표 파일과 외부 변경을 구분한다 — PLAY2 goal-file stop

이 사건 역시 [MUX C2](v8/mux/MUX_GOAL.md#c2-issue-registry-and-github)가 요구한 이력이다. 원래 중단 대화·dirty 출력·문제가 된 파일은 확보하지 못했다. 따라서 특정 파일 때문에 특정 규칙이 발동했다고 단정하지 않는다.

예방에 참고할 실제 선례는 [M-ART2.1 D1](v8/m_art2_1/MART21_DECISIONS.md#d1--intake-part-0-2026-10-24-172x-kst)이 자기 목표 파일과 명시적으로 허용된 기존 파일을 구분하여 시작 조건을 확인한 기록이다. [AGENTS의 명시 경로 staging·로컬 상태 보존](../AGENTS.md#staging-and-the-working-tree), [PLAY2 D2의 승인된 계획 계속 수행 및 D4의 dirty worktree 보호](v8/play2/PLAY2_DECISIONS.md)를 함께 적용한다. 현재 계약의 좁은 예외를 반영하되 알 수 없는 변경을 무시하거나 지우지는 않는다. PLAY2 D11의 SIGPIPE 때문에 중단한 게이트는 별개 사건이다. 이 항목도 **원본 증거 미확보**이며 수정 완료로 분류하지 않는다.

## 5. 측정 도구를 고치면 검증을 다시 시작한다 — M5 D19

[M5 D19](v8/m5/M5_DECISIONS.md#d19--the-memory-ceilings-judge-the-allocator-counter)는 동일 바이너리의 RSS가 83.0–117.5 B/tile로 흔들렸고 allocator 값은 63.1로 일정했음을 기록한다. 첫 게이트 `03115c93`을 세 단계 뒤 중단하고 측정 기준을 바꿨다는 사실도 남아 있다. 도구 변경의 이유가 타당하더라도 중단한 검사는 변경 후 소스를 증명하지 못한다.

예방은 [worldsim-pitfalls의 M5 D19 행](../.claude/skills/worldsim-pitfalls/SKILL.md), [ADR0016](decisions/0016-simulation-contract.md), [worldsim-testing의 전체 게이트 절차](../.claude/skills/worldsim-testing/SKILL.md)다. 중단을 기록하고 도구·계약 변경을 별도 커밋으로 남긴 뒤 처음부터 재검사한다. 실제 의미가 있는 RSS 외곽 한계도 별도로 유지한다. 여기서는 후속 최종 게이트를 다시 확인하지 않았으므로 중단 기록으로 종결을 추론하지 않는다.

## 6. 평가 기준을 등록하기 전에 공급·소비·측정 창의 산술을 확인한다

실패 이름을 모두 F7/F8로 통일하면 다른 원인을 잃는다.

- [M2 E7](v8/m2/M2_REPORT.md): 품질 증가 기준 0/32. 칼 여섯 개가 1.4일 안에 만들어져 비교 창에 지속 작업이 없었다.
- [M3 F7](v8/m3/M3_REPORT.md): 원래 씨앗 1/32, 두 번째 집합 0/32. 주문은 day14까지 채워졌고 이후 칼 소비 경로가 없었다.
- [M4 F8](v8/m4/M4_REPORT.md): 0/32. 31/32에서 day24 이후 칼을 만들지 않았으며 도끼 마모만으로 생산이 유지되지 않았다.
- [M5 F8](v8/m5/M5_REPORT.md): 0/32. 칼 열 개 소비에 필요한 80회와 실제 6–22회 사용이 맞지 않았다.
- [M5.5 F7/F8 및 F9](v8/m5_5/M55_DECISIONS.md): 인구 28/32 대 기준30, 미지 물품 구매26/32 대 기준28. 이주자가 지식을 가져와 미지 물품 후보가 사라지는 충돌과, 초반 기술 수준을 낮게 가정해 품질 증가 여지가 부족해진 F9는 별도 원인이다.

예방은 [worldsim-testing의 J: Controls and the arithmetic before a bar](../.claude/skills/worldsim-testing/SKILL.md#j-controls-and-the-arithmetic-before-a-bar-m55)다. 제품 결과뿐 아니라 전제·측정 가능 세계 수·관측 창을 calibration에서 확인하고, held-out 이전에 기준과 대조군을 등록한다. 이미 실패한 기준을 낮추거나 0 대 0을 성공으로 세어 과거 실패를 없애지 않는다. 모든 실패가 기획 오류라는 포괄 판단도 하지 않는다.

## 7. “읽힌 데이터”가 아니라 실제 소비 경로를 검증한다

**ISS-269·270·271·276·277·280.** RON 고정 길이 배열 표현 불일치, 존재하지 않는 시험 API와 잘못된 `extra_days=0`은 실제 타입·유효 데이터 계약을 먼저 읽어야 하는 이유다. 컴파일 또는 검증 실패를 동작 회귀 증거로 오인하지 않는다. 칼 selector가 `knife_uses` 대신 `wildlife.knife_uses`를 요청해 기본8회로 돌아갔을 때 넓은 ON fixture는 그대로 통과했다. 설정2회에서 실제 일반 제작 경로의 파손·다음 제작 차단을 보는 부정대조가 결함을 드러냈다. `slab_door`가 벽으로 보인 문제도 이름 목록 추가가 아니라 기계적 문 속성 전달로 고쳤다.

새 가뭄 데이터는 0·1000 허용,1001 거부 경계를 검증했지만 직접 지정한 기존 `StoryRule`의1000 초과 경로는 **ISS-277 OPEN**, 정적 발견이며 실행 재현은 하지 않았다. 새 입력 검증의 성공을 모든 기존 입력 경로의 수정으로 확대하지 않는다.

예방 근거는 [AGENTS의 kinds are data 계약](../AGENTS.md), [worldsim-architecture의 tunable·permille 규칙](../.claude/skills/worldsim-architecture/SKILL.md), [worldsim-testing의 실제 경로·칼 설정 경계 교훈](../.claude/skills/worldsim-testing/SKILL.md), [MUX D36·D37](v8/mux/MUX_DECISIONS.md)이다. [실제 제작 회귀](../rust/crates/sim-test/tests/recipe_registry_runtime.rs), [칼 수정 전 실패](</Users/rexxa/work/mux-data/stage-b/cutter-negative.log>), [선택 팩 첫 실패](</Users/rexxa/work/mux-data/stage-b/data-only-rust-first.log>)와 [이슈269–280](ISSUES.md)을 연결해 보존한다. focused 시험과 전체 identity/OFF/ON 안전망은 둘 다 필요하다.

## 8. 폭 변경과 저장 형식 변경은 끝까지 검증한다

**ISS-272·278·281.** Rust 쪽 ID를 넓혀도 Godot 시험의 Packed 배열과 합성 story 형식이 옛 계약이면 일부 검사가 중단된다. UI-first는 exit0·261검사에도 SCRIPT ERROR2개가 있었고, 구 M5 시험은24검사·4실패·SCRIPT ERROR5개였다. 체크 수나 종료코드만으로 통과를 선언하지 않는다.

저장 형식 부정대조도 같은 원칙이다. 잘못된 폭의 decoder가 반드시 오류를 내는 것은 아니다. 다른 값으로 성공할 수도 있다. ISS278의205통과·1실패는 이 잘못된 시험 가정을 발견한 기록이다. 올바른 frozen reader의 완전 동등성은 유지하고, 잘못된 reader가 성공하면 결과가 비동등함을 요구한다. 형식6/7/승인된8을 각 고정 표현으로 읽고 직접 변환하는 계약을 넓은 “이전 버전 허용”으로 약화하지 않는다.

예방은 [worldsim-godot의 packed type 확인·completion sentinel 및 stderr 검사](../.claude/skills/worldsim-godot/SKILL.md), [MUX D37의 frozen reader·old8 allowlist 규칙](v8/mux/MUX_DECISIONS.md), [worldsim-testing의 반증 가능한 검사](../.claude/skills/worldsim-testing/SKILL.md#j-controls-and-the-arithmetic-before-a-bar-m55)다. [저장 재시험19통과](</Users/rexxa/work/mux-data/stage-b/ecology-save-second.log>)와 [독립 저장 검토](</Users/rexxa/work/mux-data/stage-b/ecology-save-audit.md>)는 범위가 다른 근거다. 컴파일러를 통과시키기 위해 ID를 잘라 내거나 과거 인과 이력을 재생성하지 않는다.

## 9. 시험의 성공 표시는 실제 완료와 상태 출처를 증명해야 한다

**ISS-273·279.** 합성 창고 사실이 다음 실제 화면까지 남을 수 있었고, 선택 팩 bootstrap은 `Array[String]` 예외 뒤에도 성공처럼 보였다. 전자는 합성 검사 뒤 실제 bridge 사실을 복원하고 캡처 직전 출처를 기록했으며, 후자는 명시적인 `ok:true` 완료 receipt를 요구했다. 과거 경고 차이의 wall-clock polling 위상은 당시 기록이 없으므로 인과 설명은 추정으로 남는다.

예방은 [worldsim-godot의 exit0 함정·실제 WorldSimNode·완료 sentinel 규칙](../.claude/skills/worldsim-godot/SKILL.md), [worldsim-testing의 fixture 스스로 값을 설정하는 녹색 시험 경고](../.claude/skills/worldsim-testing/SKILL.md), [MUX D36의 실제 facts 복원·poll timer 보존](v8/mux/MUX_DECISIONS.md)이다. [첫 bootstrap 오류](</Users/rexxa/work/mux-data/stage-b/data-only-gate-first/ui.log>)와 [core 검증 기록](v8/mux/B_CORE_VERIFICATION.md)을 남긴다. 검증 수 증가만으로 오류·예외·미완료를 덮지 않는다.

## 10. 기능 통과, 새 회귀 없음, 절대 시각 품질은 별도 판정이다

**ISS-274·275·282.** 첫 확장 팩 창은 기능36/0이어도 긴 경고 제목이 수량을 가리고 미니맵이 비어 시각 REVISE였다. 이후40/0과 제목·수량폭·실제 미니맵 캡처의 scoped PASS를 받았어도 기존 한국어 단어 분리와 신생아 경고 잘림은 해결되지 않았다. 특히 이전 PNG와 동일하다는 증거는 “새 회귀 없음”이지 “좋은 화면”이라는 증거가 아니다.

예방은 [worldsim-godot의 실제 windowed shot 확인·Hangul 단어 보존 교훈](../.claude/skills/worldsim-godot/SKILL.md), [verification-before-completion의 주장별 직접 검증](../.claude/skills/verification-before-completion/SKILL.md), [MUX A11의 화면 증거](v8/mux/MUX_GOAL.md#a11-stage-a-verification-and-publish)다. [첫 시각 실패](</Users/rexxa/work/mux-data/stage-b/data-only-window-first-visual.json>), [후속 scoped 판정](</Users/rexxa/work/mux-data/stage-b/data-only-window-second-visual.json>), [9장 scale 판정](</Users/rexxa/work/mux-data/stage-b/ecology-scale-visual.json>)을 함께 보존한다. ISS274/275는 **OPEN / 절대 REVISE**다. 합성 입력은 물리 트랙패드 검증이 아니며 오너 수락도 아직 대신하지 않는다.

## 11. 설정 격리는 장면을 만들기 전의 실행 조건이다

**ISS-283**, 앞선 D12·D31 교훈의 재발. M5 보조 시험 두 번에서 격리 환경 변수를 빠뜨렸고 공유 settings 파일의 갱신시각이 실행과 겹쳤다. 실행 전 사본이 없어 어떤 내용이 바뀌었는지, 무엇으로 복원해야 하는지는 알 수 없다. 사후 파일을 보존하고 임의 복원을 하지 않은 것은 증거 보존이지 사고 해결이 아니다.

예방 근거는 [MUX D12·D31의 외부 설정 경로와 명시적 standalone 환경 변수 규칙](v8/mux/MUX_DECISIONS.md), [m5_ui_test의 장면 생성 전 guard](../tools/v8/m5_ui_test.gd), [AGENTS의 타인 작업 보존](../AGENTS.md#staging-and-the-working-tree)다. 외부 절대 경로가 없으면 시작을 거부하는 부정대조와 격리43/0 실행은 뒤의 무오염을 확인한다. [사고 사후 기록](</Users/rexxa/work/mux-data/stage-b/m5-settings-omission.json>), [격리 실행 전](</Users/rexxa/work/mux-data/stage-b/m5-settings-before-isolated.json>), [격리 실행 후](</Users/rexxa/work/mux-data/stage-b/m5-settings-after-isolated.json>)에서 SHA·mtime 불변을 확인하되, 이 비교의 ‘전’은 **사고 전이 아니라 후속 격리 실행 전**이다. ISS283은 **OPEN**, 앞선 변경범위·복원은 여전히 불명이다.

## 12. 도구 부재를 clean으로 쓰지 않고, 구조 이동도 실제 lint로 확인한다

**ISS-284·285.** B6에서 LSP status와 diagnostics가 `Transport closed`로 끝났다. compiler·clippy·회귀 시험으로 검증을 계속할 수 있지만 LSP 진단을 얻었다고 쓰지는 않는다. 이는 **ISS284 OPEN / LSP NOT_AVAILABLE**이며 제품 코드 검증과 도구 서버 복구의 범위를 분리한다.

자원 collector를 옮긴 첫 clippy는 시험 모듈 뒤의 production 항목 때문에 실패했다. 시험 본문을 바꾸지 않고 파일 끝으로 옮긴 뒤 strict clippy·bridge51·fmt를 다시 확인했다. 예방은 [worldsim-pitfalls의 B6 시험 모듈 배치 교훈](../.claude/skills/worldsim-pitfalls/SKILL.md), [verification-before-completion의 lint·컴파일·시험 구분](../.claude/skills/verification-before-completion/SKILL.md), [MUX D38의 기존 경계 보존·focused 시험 뒤 안전망](v8/mux/MUX_DECISIONS.md)이다. [LSP 실제 응답](</Users/rexxa/work/mux-data/stage-b/b6-lsp-unavailable.json>), [첫 clippy 실패](</Users/rexxa/work/mux-data/stage-b/b6-clippy-first.log>), [두 번째 clippy](</Users/rexxa/work/mux-data/stage-b/b6-clippy-second.log>), [B6 검증 문서](v8/mux/B6_VERIFICATION.md)를 분리해 링크한다. 작은 이동이라고 먼저 무동작변경을 선언하지 않는다. 후속 커밋된 [B6 최종 안전망 기록](v8/mux/B6_VERIFICATION.md#최종-동작-안전망)은 원본 identity, OFF9개 및 Stage A ON8세계의 동일성과 제품2087경로 hash 불변을 확인했다. 이는 당시 LSP 부재를 해소하거나 전체 B 게이트를 대신하지 않는다.

## 13. 시험 식별자와 외부 schema는 실제 초기화·lint 경계에서 고정한다

**ISS-286·287.** 새 엔진 allocator가0부터 시작하는데 fixture가1부터라고 가정해 첫 실행의288 cold invocation·32 craft가 무효가 됐다. 첫 로그의 실제0/기대1과1통과·1실패를 보존하고, 공개 allocator로0을 예약한 뒤 사전등록 Agent1–8을 유지했다. 두 번째 scoped 시험2개와 cold2304행·paid craft32개가 VALID여도 앞선 무효 결과는 지우지 않는다. 독립 scorer35시험 및 calibration의1,773 primary craft 재계산은 관측 계약의 근거이며 held-out 수용은 아니다.

모듈 선언 순서는 첫 fmt를, 동치 boolean 표현과 내부 BPT/BPTH 이름은 strict clippy3진단을 발생시켰다. 선언 정렬, `is_none_or`, 내부 `Bpt`/`Bpth`와 명시적 serde 외부명 보존으로 고쳤다. 이름 정리 때 외부 JSON vocabulary까지 바꾸지 않는다. 실패 후 재검사를 별도로 기록하며 비어 있는 fmt 로그만으로 exit0을 독립 확인했다고 쓰지 않는다.

근거: [ISSUES 286/287](ISSUES.md), [첫 fixture 실패](</Users/rexxa/work/mux-data/stage-b/b4/o-controlled-first.log>), [두 번째 fixture](</Users/rexxa/work/mux-data/stage-b/b4/o-controlled-second.log>), [첫 fmt](</Users/rexxa/work/mux-data/stage-b/b4/o-fmt-first.log>), [첫 clippy](</Users/rexxa/work/mux-data/stage-b/b4/o-clippy-first.log>), [두 번째 clippy](</Users/rexxa/work/mux-data/stage-b/b4/o-clippy-second.log>), [관측7시험](</Users/rexxa/work/mux-data/stage-b/b4/o-probe-tests-second.log>), [독립 review](</Users/rexxa/work/mux-data/stage-b/b4/o-observer-review.md>), [O 검증 초안](</Users/rexxa/work/mux-data/stage-b/b4/o-verification-draft.md>). 예방은 fixture가 실제 allocator/ordinary payment를 통과한 뒤 의미 조건을 확인하는 것, 그리고 최종 동결 소스의 identity/OFF/ON을 별도로 확인하는 것이다. 후속 [O 검증 기록](v8/mux/B4_O_VERIFICATION.md)은 identity/OFF9개/원래 no-script ON 및 실제 제작 후 저장의 범위별 동일성을 확인했다. 초안 당시의 safety PENDING은 현재 상태가 아니다. 수동 post-finish decay/death는 자연 scheduler coincidence가 아니며 제어 finish 품질은 작업 속도 증거가 아니다.

예방 규칙은 [worldsim-testing의 실제 fixture·반증 가능한 시험 원칙](../.claude/skills/worldsim-testing/SKILL.md)과 [verification-before-completion의 lint/시험별 증거 확인](../.claude/skills/verification-before-completion/SKILL.md)이다. 외부 schema 보존은 이 사건의 계약이며 일반 lint 통과만으로 확인되지 않는다.

## 14. 동일성 비교는 입력 대본까지 같아야 하며 Cargo 작업 경로를 명시한다

**ISS-288·289.** O 최초 ON 비교는 원래 플레이어 명령 없는 기준에 M55-U `--commands`를 추가해30882경로 차이를 만들었다. 차이를 제품 회귀나 golden 갱신 근거로 삼지 않고 실제 argv/script/null과 첫 tick360 명령을 독립 확인했다. 소스·golden 변경 없이 원래 no-script ON2로 돌아가 요약·8개 command 로그의 literal 동일성을 확인했다. [첫 실패](</Users/rexxa/work/mux-data/stage-b/b4/o-on1-failed-receipt.json>), [입력 감사](</Users/rexxa/work/mux-data/stage-b/b4/o-on1-input-review.md>), [ON2](</Users/rexxa/work/mux-data/stage-b/b4/o-on2-receipt.json>)를 함께 보존한다.

첫 release 빌드는 저장소 루트에서 Cargo.toml을 찾지 못했다. `rust/`에서 재실행6.34초, 최종 재빌드6.19초로 완료했다. [최초 cwd 오류](</Users/rexxa/work/mux-data/stage-b/b4/o-probe-build-first.log>)와 [최종 빌드](</Users/rexxa/work/mux-data/stage-b/b4/o-probe-build-final.log>)를 구분한다. 예방은 실행 manifest에 cwd·argv·script SHA/null·binary SHA를 명시하고 비교 전에 기준 입력과 일치시키는 것이다.

O 커밋 `f0d5140a257628a6aeb470deb12ea8c6787fa286` 이후 scoped identity/9OFF/원래ON2는 PASS다. [O 최종 기록](v8/mux/B4_O_VERIFICATION.md)이 그 범위를 명시한다. 뒤의 P/T/H 및 held-out은 별도 [B4 최종 보고](v8/mux/B4_HELDOUT_VERIFICATION.md)에 근거하며 O 검증에서 추론하지 않는다. 전체 Stage B gate·게시로 확대하지 않는다.

예방 규칙은 [worldsim-testing의 identity·등록 대본·명령 재생 절차](../.claude/skills/worldsim-testing/SKILL.md)와 [AGENTS의 검사된 커밋 승격 조건](../AGENTS.md#branches-and-pushing)이다. cwd·argv·대본을 기록하는 실행 manifest는 이 비교를 재현하기 위한 구체적 적용이다.

## 15. 실제 정착지 fixture와 문맥별 거절 설명을 구분한다

**ISS-290·291.** tick0 세계에 settlement가 있을 것이라 가정한 P fixture는 컴파일 후10시험이 모두 실패했다. 실제 hearth의 Settlement/member_agents를 명시한 뒤 B0에서 정상3통과/정책실패7로 나뉘었다. membership/grants assertion을 약화하지 않고 준비 조건을 고친 것이므로 최초 실패와 두 번째 red split을 모두 남긴다. [첫 실행](</Users/rexxa/work/mux-data/stage-b/b4/p-tests-before-first.log>), [두 번째 B0 실행](</Users/rexxa/work/mux-data/stage-b/b4/p-tests-before-second.log>).

P source reviewer는 인구 초과 거절이 기존 Blocked 코드로 반환되고 migrant dialog가 “Something is in the way”/“무언가 막고 있음”을 표시하는 문구 공백을 확인했다. 거절을 성공으로 표시하거나 창을 닫는 결함은 아니며 **ISS291 OPEN/design** 후속 작업이다. O pinned UI는 기존 문자열만 입증한다. P는 후속 커밋 `bbae211b3ee8cf8651cbe43d1014831b8a606001`과 [P 검증 기록](v8/mux/B4_P_VERIFICATION.md)으로 고정됐지만 이 문맥별 번역 공백의 실제 UI 수정·검증은 주장하지 않는다. 수용 명령 문맥의 전용 번역을 검토하되 배치에도 쓰이는 전역 blocked 문구를 덮어쓰지 않는다. [P source review](</Users/rexxa/work/mux-data/stage-b/b4/p-independent-review.md>), [O migrant dialog](https://github.com/hyunlord/new-world/blob/f0d5140a257628a6aeb470deb12ea8c6787fa286/scripts/ui/panels/migrant_panel.gd#L234).

P의 후속 안전망과 경계 시험은 [P 검증 문서](v8/mux/B4_P_VERIFICATION.md)에 기록됐다. 자연 보정에서는 정원 경계가 발생하지 않았으므로 경계 시험을 자연 개선으로 바꾸지 않는다.

예방 규칙은 [worldsim-testing의 실제 settlement 사전조건 교훈](../.claude/skills/worldsim-testing/SKILL.md)과 [worldsim-godot의 사용자 문구 Locale 규칙](../.claude/skills/worldsim-godot/SKILL.md)이다. 전역 거절 문구 수정 전에 호출 문맥을 구분한다.

## 16. Copy 값과 실행별 provenance는 의미에 맞게 비교한다

ISS292: WorldRules가 Copy인 시험의 clone 호출로 strict clippy가 실패했다. 직접 값 복사로 고친 뒤 target15/fmt/clippy 재검증 통과라는 T registry 기록을 반영하며 최초 [로컬 외부 원자료](</Users/rexxa/work/mux-data/stage-b/b4/t-clippy-first.log>)를 보존한다.

ISS293: whole-row 비교는 commit-linked manifest 식별자가 바뀌면 정상 provenance 변경도 실패시킨다. 보존된 첫 실패 script의38행 비교가 craft32에서 실패했고 cold2304는 전체 동일했다. craft32의 유일한 차이는 arithmetic_manifest_sha256이며 자기 manifest와 일치하는지는 별도로 검증한다. numeric/category 데이터·scorer·정책·raw 출력은 고치지 않았다. [로컬 외부 원자료](</Users/rexxa/work/mux-data/stage-b/b4/t-controlled-paired-equality.json>)과 첫 script/raw file SHA를 registry JSON에 보존했다. 이 문서 작업은 산출물 읽기이며 시험 재실행이 아니다.

P `bbae211b3ee8cf8651cbe43d1014831b8a606001`, T `54a55a421361d9c24268e1b3a6a5d2c2d1c6c211`은 후속 고정 커밋이다. ISS291 OPEN/design과 UI runtime 미검증은 그대로다. [T 검증 기록](v8/mux/B4_T_VERIFICATION.md)의 focused·identity·9 OFF·보정 근거를 사용한다. 전체 게이트와 후속 held-out은 별도 증거다.

예방 규칙은 [worldsim-testing의 manifest provenance와 산술 행 구분 교훈](../.claude/skills/worldsim-testing/SKILL.md)과 [verification-before-completion의 최종 lint 증거 확인](../.claude/skills/verification-before-completion/SKILL.md)이다.

## 17. 조회 실패와 실행 전 parser 검토를 제품 실행 실패와 구분한다

ISS294: workdir의 선행 slash 누락은 read-only tail 프로세스 생성 전에 거절됐다. 절대경로 재조회로 해결했고 진행 중 회귀·소스·원문 로그를 변경하지 않았다. [로컬 외부 원자료](</Users/rexxa/work/mux-data/stage-b/b4/h-root-path-error.json>)의 잘못된 경로와 오류 원문을 유지한다.

ISS295: 제어 제작 JSON은 event 안이 아니라 최상위에 arithmetic_manifest_sha256을 둔다. 기존 산출물의 실제 형태를 먼저 읽고 parser를 맞춘다. 실행 전 발견된 첫 runner/notes를 보존했고 B0/BP/BPT calibration parser·paired 비교 PASS 및 raw 불변 receipt를 확인했다. 이는 held-out 실행 성공이 아니다. receipt의 preflight/execute/heldout/full source hash audit/typecheck/subprocess orchestration NOT_RUN 한계를 유지한다.

H는 후속 `bdbd4fb2ea9c5b446648778ea7b0b2da14635049`로 고정됐다. [H 검증 기록](v8/mux/B4_H_VERIFICATION.md)은 최종23개 통제 시험과 identity·OFF 안전망, 보정의 범위·차이를 보존한다. [최종 B4 held-out](v8/mux/B4_HELDOUT_VERIFICATION.md)은 등록 범위 VALID/PASS이지만 자연 운반 terminal은 UNRESOLVED다. 당시 미커밋·캡처 진행 상태를 현재 상태로 복사하지 않으며 전체 B gate와 구분한다.

예방 근거는 [verification-before-completion의 실제 출력·종료상태 확인](../.claude/skills/verification-before-completion/SKILL.md)과 [worldsim-testing의 관측 계약·산술 대조 원칙](../.claude/skills/worldsim-testing/SKILL.md)이다. 조회 실패는 실행 실패가 아니며, 사전 parser 검토를 실행 완료로 적지 않는다.

## 18. 전역 초기화가 필요한 시험은 자신의 준비 단계에서 초기화한다 — ISS301

B5 첫 기존 steward 회귀에서 typed parser 시험이 다른 시험의 catalog 초기화 순서에 의존했다. 첫 suite는 8통과/1실패였고 새 프로세스 단독 실행도 0통과/1실패였다. fixture에 명시적 Bundled bootstrap을 넣은 뒤 단독1통과, 후속 steward9통과/1ignored를 확인했다. production parser가 몰래 전역 catalog를 초기화하도록 바꾸지 않았으며 미초기화 파서의 거절 시험도 유지했다.

예방 규칙: [worldsim-testing 예방 규칙](../.claude/skills/worldsim-testing/SKILL.md)의 각 parser test 명시적 boot·fresh-process 단독 재현·unbooted negative 유지 규칙. 근거: [문서 근거](ISSUES.md), [문서 근거](v8/mux/B5_VERIFICATION.md) 기존 회귀 절, [MUX 결정](v8/mux/MUX_DECISIONS.md) D44 후속, 외부 [로컬 외부 근거](</Users/rexxa/work/mux-data/stage-b/b5/regression-first/steward-isolated-before.log>)와 [단독 후속 로그](</Users/rexxa/work/mux-data/stage-b/b5/regression-first/steward-isolated-after.log>). **기존 시험 준비 결함이며 새 flora 제품 결함으로 바꾸어 서술하지 않는다.**

## 19. 대상 데이터와 실제 보이는 픽셀은 서로 다른 증거다 — ISS302

첫 창 캡처는 로딩 후 지연된 camp 중심 이동 때문에 대상이 화면 밖(-5696,-2020)에 있었다. 첫 focus 수정도 이전 ¼zoom에서 clamp한 뒤 1zoom으로 바꾸어 실패했다. 요청 zoom을 먼저 적용하고 중심을 잡으며 실제 HUD 안쪽 촬영 영역을 사용한 `visual-focused2`는 자동 검사 failures0이었지만 주변 수관이 원래 그루터기와 벽을 가렸다. 추가 일반 벌목900tick만 한 후속도 원목 더미가 그루터기를 가렸다. D44에 기록한 일반 stockpile·운반 후속에서 원목이 이동했고 원래 그루터기가 드러났다.

최종 실제 창54검사0실패·PNG6개 검토는 노출된 그루터기와 pine/oak 벽 대비의 제한된 PASS다. 구현 담당자의 재검토를 독립 오너 승인으로 부르지 않으며, 밀집 수관의 standing 장면을 단독 종 실루엣 판독 PASS로 주장하지 않는다. 초기 실패와 가림 캡처는 모두 남긴다.

예방 규칙: [worldsim-testing 예방 규칙](../.claude/skills/worldsim-testing/SKILL.md)의 지연 centering/zoom 순서·실제 HUD 경계·visible 객체와 픽셀 구분·일반 명령 후속의 명시 규칙. 근거: [문서 근거](ISSUES.md), [B5 검증](v8/mux/B5_VERIFICATION.md) 실제 창 후속, [실제 캡처 harness](../tools/v8/b5_tree_visual.gd), 외부 [로컬 외부 근거](</Users/rexxa/work/mux-data/stage-b/b5/window-final-receipt.json>), [로컬 최종 픽셀 검토](</Users/rexxa/work/mux-data/stage-b/b5/b5-window-review-final.md>) 및 [JSON](</Users/rexxa/work/mux-data/stage-b/b5/b5-window-review-final.json>). 촬영 harness 수정과 제품 카메라 변경을 혼동하지 않는다.

## 20. ignored lockfile도 빌드 입력이다 — ISS304

최초 B5 제품2123파일 freeze는 ignored `rust/Cargo.lock`을 포함하지 않았고 첫 빌드 전 lock hash는 알 수 없다. 이후 lock hash 전후 일치 및 `cargo --locked` release 재빌드 종료0, probe/perf binary hash가 기존 freeze와 동일함을 확인했다. 이는 재현 확인이며 최초 사전 pin 공백을 소급해서 없앤 것이 아니다. 추가 scene/shader/localization/gdextension 입력의 사후 관측도 최초 입력 freeze와 별개다.

예방 규칙: [worldsim-testing 예방 규칙](../.claude/skills/worldsim-testing/SKILL.md)의 ignored lock 사전/사후 hash·`--locked`·binary 결합·이전 공백 보존. 이전 사례 규칙도 [worldsim-pitfalls 예방 규칙](../.claude/skills/worldsim-pitfalls/SKILL.md)에 있다. 근거: [문서 근거](ISSUES.md), [B5 검증](v8/mux/B5_VERIFICATION.md) 최종 OFF 안전망, 외부 [로컬 외부 근거](</Users/rexxa/work/mux-data/stage-b/b5/locked-rebuild-receipt.json>). 의존성 provenance 전체가 처음부터 완전했다고 쓰지 않는다.

## 21. 커밋 문자열은 실제 object에서 검증한다 — ISS303

B1 review/performance runbook의 Stage A SHA에 `b94`가 중복되어 43자가 되었다. 실제 `git rev-parse` 결과는 40자 `ca9f531ca6ef700ca223d99dba14d834b94b45da`다. 저장소 설명은 수정하고 외부 원래 초안은 보존했다. 비교 대상은 오타 문자열이 아니라 실제 pin이어야 한다.

예방 규칙은 현재 설치된 [Commit pin resolution](../.claude/skills/worldsim-testing/SKILL.md#commit-pin-resolution)과 [Evidence identity and terminal states](../.claude/skills/worldsim-pitfalls/SKILL.md#evidence-identity-and-terminal-states)이다. pin을 로컬 commit object에서 얻고 문서에 복사한 40자리 값도 다시 resolve해 비교한다. 로컬 객체 검증은 원격 게시 검증이 아니다. [ISS303](ISSUES.md), [코드 검토](v8/review/CODE_REVIEW_2026-10.md), 외부 [원래 runbook](</Users/rexxa/work/mux-data/stage-b/b1-final-performance-runbook.md>)을 보존한다.

## 22. 시작 관측을 완료로 바꾸지 않는다 — ISS300/B4

H calibration의 107건 시작(stockpile79/hearth28)은 terminal이 모두 UNRESOLVED였다. held-out도 네 arm 시작368/368/372/472건의 terminal이 전부 UNRESOLVED다. BPTH472는 stockpile373/hearth99이며 472번 배달 성공이라는 뜻이 아니다. 별도의 통제된23시험과 자연 실행의 완료 연결 부족을 구분한다. B4 VALID/PASS는 사전 등록한 평가의 유효성·기준 충족이며 모든 정책의 개선이나 인과 효과를 뜻하지 않는다. natural cold 기회가 없어 N/A인 행을 성공으로 세지 않는다.

실제 계약은 [사전등록](v8/mux/B4_PREREGISTRATION.md), [관측 schema](v8/mux/B4_OBSERVER_SCHEMA.md), [최종 검증](v8/mux/B4_HELDOUT_VERIFICATION.md), D39/D42/D43이다. 현재 설치된 [Terminal observation evidence](../.claude/skills/worldsim-testing/SKILL.md#terminal-observation-evidence)와 [Evidence identity and terminal states](../.claude/skills/worldsim-pitfalls/SKILL.md#evidence-identity-and-terminal-states)는 시작과 확인된 종료를 분리하고 UNRESOLVED/N/A를 성공으로 세지 않도록 명시한다. frozen B4 원시 자료·평가기·정책을 소급 수정하지 않는다.

## 23. 낡은 시험 준비를 제품의 자동 보정으로 감추지 않는다 — D45

clean `60f3371691b398e16a9d3675ea3e36b058559616`의 B gate1은 **31/36 FAIL**이었다. 재료 수5 기대, synthetic tree identity 누락, 예전 style 이름, M5.5 API 목록, Locale bootstrap 누락을 시험5경로에서만 수정했다. 배치·내구도120·Chronicle·raw-column/category·locale 문구 계약은 유지했다. 후속 focused Rust3·Python133·fmt·명시적 bridge build·Locale 실행/strict·workspace all-target clippy는 통과했다. 이 focused 기록 자체는 full gate2 증거가 아니다. 이후 별도의 [B gate2 영수증](</Users/rexxa/work/mux-data/gate-stage-b-2/receipt.json>)과 [전체 로그](</Users/rexxa/work/mux-data/gate-stage-b-2/gate.txt>)는 clean `5e20c65a1d52ba2a96a6e524614d3a7679f31a44`에서 18:49:48–19:54:54 KST, 36/36 status0을 기록했다. [게시 영수증](</Users/rexxa/work/mux-data/stage-b-publication.json>)은 19:56:18 KST에 v8/wip·v8/main이 같은 SHA임을 확인했다. 이 후속 통과·게시는 첫 실패를 지우지 않으며 C 전체 gate 결과가 아니다.

실제 예방 규칙은 [Registry migration fixture audit](../.claude/skills/worldsim-testing/SKILL.md#registry-migration-fixture-audit), 근거는 [D45](v8/mux/MUX_DECISIONS.md), [수정 검증](v8/mux/B_GATE_REPAIR_VERIFICATION.md), ISS305/306/308/309/310이다. 최초 gate 실패를 보존하고 원본2123 freeze 중 시험5개만 달라진 사실과 옛 OFF/ON 바이너리 귀속을 분리한다.

## 24. 빈 실패 목록은 실행 완료 증거가 아니다 — ISS307

외부 C safety wrapper의 실행 전 초안은 KeyboardInterrupt/SystemExit가 일반 exception 처리를 벗어나면 finally에서 실패 목록이 비었다는 이유로 PASS를 쓸 수 있었다. 결함이 있던 원래 초안으로 실제 capture는 실행되지 않았다. 수정은 completed=false에서 시작해 정확히20개의 literal 동등 비교를 마친 뒤만 true로 하고, finally에서도 완료·개수·동등성을 모두 요구한다. 인터럽트는 전파하며 종료 I/O 중 영수증이 없으면 미완료다.

수정 근거는 [외부 원본·후속 source review](</Users/rexxa/work/mux-data/stage-c-prep-1/run-c-safety-source-review.md>)의 실제 guard를 추출한 순수6경우 검사다. 이것은 전체 wrapper/preflight/capture 실행 증거가 아니다. 현재 설치된 [Completion receipts survive interruption](../.claude/skills/worldsim-testing/SKILL.md#completion-receipts-survive-interruption) 예방 규칙을 적용한다. 새 제품 버그나 실행된 안전망 실패로 바꾸어 쓰지 않는다.

## 25. 제목 규약 위반을 고치려고 측정된 커밋을 다시 쓰지 않는다 — ISS311

B p9 `3c39dcbe0039c98479c6294c09b599ed009650b0`의 실제 제목은 `Keep tree material identity consistent from world generation to built walls`, p10 `60f3371691b398e16a9d3675ea3e36b058559616`은 `Bind Stage B reports to the measured tree-material commit`이었다. 둘 다 [목표의 Git 계약](v8/mux/MUX_GOAL.md)의 `v8-mux(B pN): …` 접두사를 빠뜨렸다. p11 `5e20c65a1d52ba2a96a6e524614d3a7679f31a44`는 `v8-mux(B p11): Keep migrated contracts covered by valid test fixtures`로 규약을 지켰다.

ISS311은 역사적 process 위반으로 won't fix이며 오너에게 보고했다. 원래 SHA와 연결된 측정/gate1 증거를 보존하고 명시적인 rewrite 승인 없이 amend/rebase하지 않는다. 새 정상 제목은 과거 위반의 소급 해소가 아니다. 예방은 현재 설치된 [Stage commit subject validation](../.claude/skills/worldsim-testing/SKILL.md#stage-commit-subject-validation): 커밋 전 정확한 subject 첫 줄을 정규식과 현재 단계/번호로 검증하고 커밋 후 실제 제목도 확인한다.

## 편집·증거 한계

1–6절은 C2가 요구한 여섯 역사 사건을 유지한다. 잘못된 play 명령과 goal-file 중단의 원본 증거는 미확보이며, ISS283 사고 전 설정 원본도 알 수 없다. ISS284 LSP NOT_AVAILABLE, ISS291 문맥별 거절 문구, ISS300 자연 운반 종료 연결 공백을 그대로 둔다. 21·22·24·25절의 전용 예방 규칙은 worldsim-testing과 worldsim-pitfalls에 함께 설치된 문안을 따른다.

각 검증 문서의 결과는 커밋된 기록을 인용한 것이며 이번 편집에서 재실행한 결과가 아니다. B6/O/P/T/H의 후속 기록과 B gate2 36/36·게시 영수증을 반영했다. C1 삭제 후 안전망은93a99ce16에서 원본 identity 및20 literal 비교를 완료했고, C 전체 gate·게시·물리 입력·오너 수락은 완료로 기입하지 않았다. 당시 정본 누적 수는 이슈 원장에 보존하고 여기에는 서로 다른 시점의 수를 현재 합계처럼 나열하지 않는다.

C wrapper 후속 실행: 2026-10-25 20:22:05 KST completed=true·20/20동일·failures=[]·exit0을 확인했다([C 검증](v8/mux/C_CLEANUP_VERIFICATION.md)). 순수 guard 검사, 실제 capture, 전체 gate는 각각 다른 증거다.

## 26. 생성 성공과 목록 반영은 같은 순간이 아니다 — ISS312

GitHub 등록 첫 두 번은 실제#513/#514가 생성됐지만 즉시 목록에 나타나지 않아 안전 중단했다. pending 기록을 지우거나 새 journal로 재시도하지 않았다. 반환 canonical URL·번호를 영속화하고 직접 GET으로 검증하도록 보완했으며 불확실한 응답에는 제한된 읽기 재시도만 허용했다. 원본 재현 포함12 mock 그룹과 실제105건 등록·독립 전체 대조를 구분한다. 기존#59는 내용·수정 시각 모두 불변이다. [검증](v8/mux/C_GITHUB_VERIFICATION.md), [예방 규칙](../.claude/skills/worldsim-testing/SKILL.md#external-publication-read-after-write-verification).


## 27. 조용한 진입과 짧은 tail 비교만으로 원인을 확정하지 않는다 — ISS314

C gate2는 부하6.23으로 진입하고7.00으로 종료했지만 두 p99 기준에 실패했다. 인접 단계의 부하8.06을 그 실패의 원인으로 전용하지 않는다. 원본35/36 실패와 고정 여섯 교차 실행을 모두 보존하고, 첫 비교의10.48 부하·서로 다른 빌드 정체성·tail 변동을 명시한다. 진단은 전체 gate나 원인 증명이 아니다. 기준·장부를 바꾸지 않고 최종 문서 HEAD의 새 전체 gate를 별도로 기록한다. 설치 규칙: [Bounded benchmark diagnosis](../.claude/skills/worldsim-testing/SKILL.md#bounded-benchmark-diagnosis); 근거: [C gate2 진단](v8/mux/C_GATE2_DIAGNOSTIC.md).
