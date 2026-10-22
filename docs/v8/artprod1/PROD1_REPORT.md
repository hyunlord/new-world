# ART-PROD1 최종 보고서 — FAIL / route I 미채택

2026-10-23. 범위: `v8/art-prod1`, 기준 `9122b9b1c5e4c28d8dd8ff8a0413bd85963a225d`.
목표에 정한 초기 생산과 한 번의 교정·재리뷰를 종료했다. **96회 생성, 13/36룩 수락(성인 13/18, 어린이 0/18), 130프레임**이다. S1·R1·P1이 실패했으므로 표준 채택·역사 자산 archive 이동·M-ART2 설치·merge를 하지 않는다. `TEMPLATE_V3`와 `route_i`는 후보/기계검사 수락 증거로 남긴다.

## 1. 판정과 분모

| 항목 | 결과 | 기준/판정 |
|---|---|---|
| S1 | 전체 13/36, 성인 13/18, 어린이 0/18 | ≥32/36 및 성인 ≥17/18: **FAIL** |
| S2 | 전체 소비 96/13 = **7.385회/수락룩** | ≤2.0: **FAIL** |
| S2 보조 | 수락된 13룩에 소비한 호출만 27/13 = **2.077회** | 탈락룩 비용을 뺀 값도 미달 |
| J1 | 수락 프레임 원본 패널 numeric judge **130/130** | **PASS**, 모델 의미판정과 별개 |
| R1 | 인증 랩 구성 불가; 최종 진단 점수도 속성/성별연령 미달 | **FAIL**, 진단을 인증으로 대체하지 않음 |
| T1 | medium 3.667, line 4.000, light 3.167, palette 4.000; 총평균 **3.708/5** | 보고 완료, 권장 4.0 미달 |
| P1 | 200명·60.0142초, **59.269306 FPS** | ≥60: **FAIL**, 반올림 승격 없음 |
| K1 | 96회, 수락률 13/36=36.1%; 1 top 추가는 12시트·최대36회 | 추가 top의 수율·실제 비용은 미측정 |
| D1 | 절단 **910쌍**, 정규화 **860쌍** 바이트 동일 | **PASS(생성된 출력의 재현성)**; 결손 100개는 성공 제외 |

[호출 전체 기록](PROD1_GENERATION_LOG.json), [최종 세트 검사](SET_CHECK_FINAL.json), [최종 계측 수치](INSTRUMENT_COUNTS_FINAL.json), [군중 결과](CROWD_RESULT.json).

## 2. 계측기 수정과 TRIAL5 재집계

전체 시트 대신 각 인물의 원본 해상도 alpha crop + 8% padding에 동결된 person judge v2를 실행했다. calibration SHA는 `7218ac77979065ec6398c77cb6156bce5bd523979ed48732945f2ab642a5d5fd`로 유지했다. whole-sheet 경로는 legacy로 보존했다. 구멍 규칙은 whole-figure 검사에서 제거했고, 연결성·접지·캔버스·idle 높이·sleep 폭과 축을 검사한다. 2×5 소유권 분할과 공통 배율을 적용했다.

TRIAL5 72개 생산 try2 look×facing 재집계: 구조 전체 **2/72**, 패널 **342/360**, 패널 전체 통과 **55/72**, 구조·패널 교집합 **2/72**. 자세한 분모와 실패는 [재집계](TRIAL5_RECOUNT.md)에 있다. 이 72개는 실제 리뷰 round1의 혼합 try1/try2 선택과 같지 않다. 역사 자산은 승격하지 않았다.

**교정 제약:** 목표에 서술된 Part0 I-1 5자세 시트는 역사 기록에 없다. 실제 I-1 단일 인물은 1/1 통과, 사전 지정한 가장 가까운 실제 생산 5자세 시트는 4/5 통과(work_a stroke_q90 실패)다. 존재하지 않는 5패널 교정을 통과했다고 주장하지 않는다. [Part0 계측](PART0_INSTRUMENTS.md), [결정 D03·D06·D09](PROD1_DECISIONS.md)에 이 불일치를 유지했다.

## 3. 세트·시도와 룩별 실패

초기 95회, 리뷰 교정 1회로 총 96회다. 각 룩 최대3회, 전체108회 제한을 지켰으며 최초 10/10에서 멈췄다. 초기 성인남 19회/6룩, 성인여 22회/7룩, 어린이남 27회/0룩, 어린이여 27회/0룩이다. 여성 성인 허리천 short 교정 1회는 실패해 기존 try2를 유지했다. 미사용 12회는 한도 소진 룩이나 추가 리뷰 라운드에 전용하지 않는다.

다음 표의 원인은 **최선 시도** 기준이다. 모든 다른 시도·원문 실패는 호출 JSON에 남았다. “미리뷰”는 의미상 합격을 뜻하지 않는다. 수락 13룩 중 고정 진단 표본에 포함된 수락룩은 3개뿐이다.

| look | 호출 | 게시 | 최선 try / 통과 | 최선 기계검사 실패 | 두 리뷰 라운드 오독 필드 |
|---|---:|---|---|---|---|
| adult_M_loincloth_short | 3 | 수락 | 3 / 10/10 | 없음 | 미리뷰 |
| adult_M_loincloth_long_tied | 1 | 수락 | 1 / 10/10 | 없음 | 미리뷰; 생성자 관찰 top 불일치 |
| adult_M_loincloth_bun | 3 | 누락 | 1 / 9/10 | S/sleep: canvas 정규화 | top |
| adult_M_hide_wrap_short | 2 | 수락 | 2 / 10/10 | 없음 | top |
| adult_M_hide_wrap_long_tied | 1 | 수락 | 1 / 10/10 | 없음 | 미리뷰 |
| adult_M_hide_wrap_bun | 1 | 수락 | 1 / 10/10 | 없음 | 미리뷰 |
| adult_M_fur_cloak_short | 3 | 누락 | 1 / 9/10 | S/sleep: canvas 정규화 | 미리뷰 |
| adult_M_fur_cloak_long_tied | 3 | 누락 | 1 / 9/10 | E/work_a: judge:stroke_q90 | 표본 내 오독 없음 |
| adult_M_fur_cloak_bun | 2 | 수락 | 2 / 10/10 | 없음 | 미리뷰 |
| adult_F_loincloth_short | 3 | 수락 | 2 / 10/10 | 없음 | sex, top |
| adult_F_loincloth_long_tied | 2 | 수락 | 2 / 10/10 | 없음 | 미리뷰 |
| adult_F_loincloth_bun | 3 | 수락 | 3 / 10/10 | 없음 | 미리뷰 |
| adult_F_hide_wrap_short | 2 | 수락 | 2 / 10/10 | 없음 | 미리뷰 |
| adult_F_hide_wrap_long_tied | 2 | 수락 | 2 / 10/10 | 없음 | 미리뷰 |
| adult_F_hide_wrap_bun | 3 | 누락 | 1 / 9/10 | S/sleep: canvas 정규화 | hair, sex |
| adult_F_fur_cloak_short | 3 | 수락 | 3 / 10/10 | 없음 | 미리뷰 |
| adult_F_fur_cloak_long_tied | 2 | 수락 | 2 / 10/10 | 없음 | 표본 내 오독 없음 |
| adult_F_fur_cloak_bun | 3 | 누락 | 1 / 9/10 | S/sleep: canvas 정규화 | 미리뷰 |
| child_M_loincloth_short | 3 | 누락 | 1 / 8/10 | S/sleep: canvas 정규화; E/sleep: sleep 폭 | 미리뷰 |
| child_M_loincloth_long_tied | 3 | 누락 | 1 / 7/10 | S/sleep: sleep 폭; E/idle: judge:stroke_q25; E/sleep: sleep 폭 | 미리뷰 |
| child_M_loincloth_bun | 3 | 누락 | 1 / 8/10 | S/sleep: sleep 폭; E/sleep: sleep 폭 | sex, top |
| child_M_hide_wrap_short | 3 | 누락 | 1 / 8/10 | S/sleep: canvas 정규화; E/sleep: sleep 폭 | 미리뷰 |
| child_M_hide_wrap_long_tied | 3 | 누락 | 1 / 8/10 | S/sleep: canvas 정규화; E/sleep: sleep 폭 | hair, sex, top |
| child_M_hide_wrap_bun | 3 | 누락 | 1 / 8/10 | S/sleep: canvas 정규화; E/sleep: sleep 폭 | 미리뷰 |
| child_M_fur_cloak_short | 3 | 누락 | 1 / 8/10 | S/sleep: canvas 정규화; E/sleep: sleep 폭 | age, hair, sex, top |
| child_M_fur_cloak_long_tied | 3 | 누락 | 1 / 8/10 | S/sleep: sleep 폭; E/sleep: sleep 폭 | 미리뷰 |
| child_M_fur_cloak_bun | 3 | 누락 | 1 / 8/10 | S/sleep: canvas 정규화; E/sleep: sleep 폭 | 미리뷰 |
| child_F_loincloth_short | 3 | 누락 | 1 / 8/10 | S/sleep: canvas 정규화; E/sleep: sleep 폭 | 미리뷰 |
| child_F_loincloth_long_tied | 3 | 누락 | 1 / 8/10 | S/sleep: canvas 정규화; E/sleep: sleep 폭 | sex, top |
| child_F_loincloth_bun | 3 | 누락 | 1 / 8/10 | S/sleep: canvas 정규화; E/sleep: sleep 폭 | 미리뷰 |
| child_F_hide_wrap_short | 3 | 누락 | 1 / 8/10 | S/sleep: canvas 정규화; E/sleep: sleep 폭 | sex, top |
| child_F_hide_wrap_long_tied | 3 | 누락 | 1 / 8/10 | S/sleep: canvas 정규화; E/sleep: sleep 폭 | 미리뷰 |
| child_F_hide_wrap_bun | 3 | 누락 | 1 / 8/10 | S/sleep: canvas 정규화; E/sleep: sleep 폭 | 미리뷰 |
| child_F_fur_cloak_short | 3 | 누락 | 1 / 9/10 | S/sleep: canvas 정규화 | 미리뷰 |
| child_F_fur_cloak_long_tied | 3 | 누락 | 1 / 8/10 | S/sleep: canvas 정규화; E/sleep: sleep 폭 | 미리뷰 |
| child_F_fur_cloak_bun | 3 | 누락 | 1 / 0/10 | S/idle: grid; S/walk_a: grid; S/work_a: grid; S/carry: grid; S/sleep: grid; E/idle: grid; E/walk_a: grid; E/work_a: grid; E/carry: grid; E/sleep: grid | activity, age, hair, item, sex, top |

[36룩 보드](../../../art/character/boards/route_i_sheet_2x.png), [4룩 전체 자세](../../../art/character/boards/route_i_states_2x.png), [A/B/P+PROD1 비교](../../../art/character/boards/compare_ABPT2_2x.png). 누락은 MISSING으로 표시했다. 비교는 역사 여섯 사례에 PROD1 마지막 열을 붙인 것이며, TRIAL5 열을 추가했다는 뜻이 아니다. N은 S 대체, 역사 성별·연령은 완전 일치하지 않는다.

## 4. 절단·정규화와 구조·패널 판정

80×112, 접지 y104, x40; sleep112×80, y72, x56. S idle 기준 한 배율을 열 자세 모두에 적용했다. 수락 배율은 **0.126984–0.141414**, 수락 130프레임 회전은 모두 **0°**다. 픽셀 수술·픽셀 삭제·개별 자세 축소·자산 색 양자화·톤 보정은 없다. alpha0 trim과 uniform resize, 필요한 방향 회전만 허용했다. GIF는 리뷰용 파생물이며 수락 PNG를 대체하지 않는다.

96시트의 예정960프레임 중 5시트는 grid 단계에서 실패했다. judge는 실제 분리된910패널 중891 통과, 구조는784/960, 구조·judge 동시767/960이다. 최종 게시13룩은130/130 모두 원본·해시·배율·회전·픽셀·구조·패널·필수 MANIFEST 독립 재검사를 통과했다. 그러나 세트 표본은 **26/48 고유 look×facing**뿐이므로 check_set 전체 결과는 FAIL이다.

수면의 희미한 alpha>0 꼬리는 실제 생성 픽셀이므로 지우지 않았다. 95시트 민감도 검사에서 분리 가능한180수면 패널 중30개는 반대180°에서 캔버스에 들어갔으나 검출 머리가 오른쪽으로 바뀌었다. 이 가운데14개는 구조도 맞지만 head-left와 충돌한다. canonical head-left만 사용하며 “어느 방향도 안 맞을 때만 실패”의 일반 fallback은 구현하지 않은 제한이다. 마지막 교정시트는 이 민감도 조사에 포함하지 않았다. 판정 도구·임계값을 생산 중 느슨하게 바꾸지 않았다.

D1은96원본을 고정하고 별도 Python 프로세스 두 번에서 절단·정규화 PNG와 변환/소유권 기록을 비교했다. 50개는 분할 실패,50개는 정규화 실패로 출력 부재이며 재현성 성공 출력 수에 넣지 않았다. 수락130쌍 및 탈락 후보730쌍이 정규화 동일성 검사를 통과했다. 원시 verifier의 INTERIM 표시는 최종 cohort 감사와 분리되어 있으며 최종 감사가96회 전체와 소스 불변성을 확인했다.

## 5. 랩·리뷰와 한 번의 교정

수락 어린이0명 때문에 ≥2children 조건의 인증 랩은 **구성 실패**다. 대신 고정12룩의 최선 시도를 진단 랩으로 만들고, 정규화 결과가 없는16/120프레임은 MISSING으로 표시했다. 모든 metadata·GIF footer·리뷰 점수는 `formal_R1_eligible=false`다.

각 라운드에서 실제 Chromium GIF 재생24개를3초·8Hz로 캡처했다(576스크린샷/라운드, 브라우저 오류0). 캡처된 시간순 접촉판48장만, 정답·코드·이전 리뷰 접근을 막은 새로운 에이전트2명에게 제공했다. 네 명 모두48장을 직접 열었고 원문24행씩을 보존했다. 사람 대상 연구가 아니라 독립 모델 리뷰다. 분류 선택지는 동일하게 주었으며 unknown은 오답, 기존 TRIAL5 매핑·기준은 동결했다.

| 라운드 | 리뷰어 | 배율 | 활동 | item+top+hair | sex+age | 종합 |
|---|---|---|---|---|---|---|
| 0 | A | 1× | 11/12 | 25/36 | 14/24 | FAIL |
| 0 | A | 2× | 11/12 | 28/36 | 15/24 | FAIL |
| 0 | B | 1× | 11/12 | 24/36 | 15/24 | FAIL |
| 0 | B | 2× | 11/12 | 27/36 | 15/24 | FAIL |
| 1 | A | 1× | 11/12 | 25/36 | 15/24 | FAIL |
| 1 | A | 2× | 11/12 | 28/36 | 16/24 | FAIL |
| 1 | B | 1× | 11/12 | 26/36 | 16/24 | FAIL |
| 1 | B | 2× | 11/12 | 27/36 | 16/24 | FAIL |

기준: 활동1×≥8/12,2×≥10/12; 속성≥30/36; 성별연령≥20/24. 수치가 통과하더라도 진단 표본은 인증 R1이 될 수 없다.

원문과 판정: [0A](reviews/round0/a.md), [0B](reviews/round0/b.md), [0점수](reviews/round0/scores.json), [1A](reviews/round1/a.md), [1B](reviews/round1/b.md), [1점수](reviews/round1/scores.json). 공통 관찰은 완전 누락 p1_08, 한 방향 누락 p1_05, 통나무의 머리/몸 가림, 어린이 성별 불명확, 단순 두 자세의 급격한 전환이다. 최종 원문은 별도 링크대로 유지하며 서로 다른 판독을 합쳐 정답으로 만들지 않았다.

첫 리뷰 오독9룩 중 남은 한도가 있는 `adult_F_loincloth_short`만 sex/top cue를 강화해 try3 호출했다. S sleep clipping과 E idle stroke_q25로8/10 실패, try2 유지. 두 라운드24GIF 및12선택 시도는 동일해 두 번째 점수 변화는 개선 효과가 아니라 리뷰어 변동이다. [교정 계획](REVIEW_RETRY_PLAN.json), [동일성](REVIEW_ROUND_IDENTITY.json). **세 번째 교정·리뷰 없음.**

## 6. 스타일 점수와 화면 검증

같은 person-family 기준 이미지 full/upper를 직접 열고, 고정6스틸(01,02,03,06,07,09)을 부모 모델이1×에서 관찰했다. 잉크 윤곽·흙색은 유지되지만 어린이 크기에서 수채 질감과 조명 방향이 약해졌다. medium3.667/line4/light3.167/palette4, 평균3.708이다. [개별 점수·설명·해시](STYLE_SCORES.json). 최종6스틸도 초기와 바이트 동일했다. 외부 시각 API나 인간 심미 승인을 주장하지 않는다.

실제 브라우저 idle 영역과 원본 GIF 프레임 비교는 **0/55,552픽셀 차이**였다. [픽셀 결과](VISUAL_NATIVE_PIXEL_DIFF.json), [UI 검사 A](visual_qa_A.md), [UI 검사 B](visual_qa_B.md). 375/768/1280px 화면을 확인했으며 작은 화면2×는 native448px 이미지를 내부 스크롤한다. 주변 ground 선택기는 불투명한 합성 GIF 뒤에 가려 시각 변화가 없는 기존 낮은 우선순위 제한이 있다. 두 요구 지면은 GIF 내부에 모두 보인다. UI 표시 검사는 인물 자산 승인을 대신하지 않는다.

## 7. 군중 FPS와 비용

13개 수락 성인 룩을 반복해200명,1×,2240×1120 Canvas2D에서60.0142초 동안 그렸다. 3,558프레임·711,600 sprite draw, rAF 평균59.269306FPS, 프레임 간격p50/p95=16.7ms,p99=16.8ms. 시작/종료 화면에서20×10배치를 확인했고 page error0이다. **P1 FAIL**이며 60으로 반올림하지 않는다.

이는 headless Chromium 제출 프레임 계측이며 Godot·시뮬레이션·물리 디스플레이 scanout 성능이 아니다. 호스트의 병행 랩 캡처 부하도 있었다. 교정 후 PNG·sidecar260개의 해시가 같아 최초 측정이 최종 입력에도 적용된다. [입력 동일성](CROWD_FINAL_IDENTITY.json). 비용은 builtin image tool96회로 보고하며 금액/크레딧은 측정하지 않았다. 새 top12시트의 상한36회는 계획 제약일 뿐 실제 수율 검증이 아니다.

## 8. 검증·변경 파일·제한사항

최종 art 파이프라인/랩 테스트 **64개 통과**, 변경 Python Ruff 통과, 랩 Pyright **0 errors/0 warnings**, docs check와 git diff check 통과. 게임 코드와 bridge를 바꾸지 않았고 전체21단계 게임 gate·Godot 실행·장치 플레이는 수행하지 않았다. 이를 게임 성능/동작 승인으로 해석하지 않는다.

주요 변경은 `art/pipeline5/`의 panel judge·whole checks·sheet geometry·process/check_set/recount·테스트, `art/whole_figure_lab/prod1*`의 랩/보드/브라우저/스코어/진단, `art/character/TEMPLATE_V3.md`, 수락130PNG·sidecar·manifest, 비교/검토 보드와24GIF, 본 문서 묶음이다. 잘못 적용한 whole-sheet 측정과 layered hole 규칙을 수락 경로에서 제거했고 공통 배율과 단일 시트 원본으로 S/E 출처를 통합했다. 남은 위험은 교정 원본 불일치, 반복 alpha 꼬리, 어린이 sleep 폭, 실제 의미 가독성, 전체 population 미달이다.

허용 범위를 지켜 전역 CHANGELOG/ROADMAP/HISTORY/ADR·게임 코드 대신 이 패스 문서와 [결정 기록](PROD1_DECISIONS.md)에 기록했다. 기존 `.gitignore`, `.ignore`, `.omo/`, `docs/art/`를 건드리거나 stage하지 않았다. 모든 신규 커밋 파일은1.5MB 이하이고 원본/캡처/로그는 외부에 보존했다.

원시 증거 루트: `/Users/rexxa/work/artprod1-data/`. `generation/` 호출·원본·prompt, `production/` 판정, `round0/`·`round1/` 실제 캡처/블라인드 패킷, `d1-final/` 두 번의 실제 처리 결과, `trial5-recount/run-20261023-02/` 역사 재집계. [검증 인덱스](PROD1_VERIFICATION.json)에 해시와 완료 범위를 기록했다.

## 9. 오너 결정 필요 — 새 작업으로만 진행

이 패스에서 표준 채택/설치는 승인하지 않는다. 향후 M-ART2 범위(64px 카메라에서 후보 세트를 설치할지), N 방향의 실제 제작 또는 S 대체, 노인·머리색 변형, 어린이 수면 규격과 성별 구별 목표는 별도 결정이 필요하다. 이 보고를 이유로 실험이나 생성 라운드를 자동 연장하지 않는다.
