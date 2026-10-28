## 구현 완료 보고 — M-UX2.1

**검증 진행 중 · IN_PROGRESS · NOT_PUBLISHED.** 오너 지정 제목이며 전체 완료 선언이 아니다. 이 초안은 로컬 `b0401ca2e40de0ee972d451b3c4c1ecf0c1fa816`의 완료된 영수증만 반영한다. 2026-10-28 재개 기준은 게시 M-CORE `3814828c06d1a7a211b13b54368e4e0a488cc0e3`, 브랜치는 `v8/mux2-1b`다. 원래 브랜치와 [2026-10-26 차단 보고](MUX21_BLOCKED_20261026.md)는 보존했다.

### 원인 (빌드 3개 비교, 이분 탐색으로 찾은 커밋, 프로파일 상위 5개)

조용한 호스트에서 교차 순서로 세 빌드의 ¼× 비교12개를 완료했다. 동일 하네스·release bridge·1920×1080·vsync off·30초 준비+60초 이상 렌더 후 프레임을 사용했다. 무효 실행은 제외하고 원본을 보존했다. N은 초기 요청 인구이며 N10의 실제 관측 범위는10–11, 나머지는 요청 인구와 같았다. [비교 영수증](/Users/rexxa/work/mux21-data/resume-20261028/comparison-summary.json).

| 초기 N | M-ART2.1 `3a51636d` | M-UX `0207b497` | 게시 BASE `3814828c` |
|---:|---:|---:|---:|
| 10 (실제10–11) | 453.120 | 338.924 | 317.107 |
| 40 | 490.763 | 374.202 | 267.864 |
| 200 | 339.207 | 286.806 | 34.948 |
| 1,000 | 153.656 | 139.582 | 0.350 |

M-UX까지 유지되던 성능이 M-UX2의 표식 배치 경로에서 급락한다. `1c8d2979e`의 매 프레임 `Layout.place()`와 겹침 탐색이 원인 경로이며, 이 세 빌드 비교와 아래 조용한 BASE 진단이 같은 경로를 지목하므로 재개 지시에 따라 추가 이분 탐색은 생략했다. 단일 커밋을 새 이분 탐색으로 측정했다는 주장은 하지 않는다.

BASE N200·¼× 진단은60.004403초/2,106표본. 부하1/5/15는 시작5.72/5.07/6.28, 종료6.11/5.51/6.33이며 다른 Godot·cargo 없음 기록을 보존했다. 이전 혼잡 진단25.053ms와 같은 병목이 조용한 조건에서도 재현됐다. 아래 구간은 중첩 합산하지 않는다. [프로파일](/Users/rexxa/work/mux21-data/resume-20261028/base-profile-quiet/profile.json).

| 주요 구간 | 평균 ms/프레임 |
|---|---:|
| 표식 갱신 | 24.570 |
| 사람 상태 동기화 | 0.316 |
| 지형 처리 | 0.181 |
| 사람 정리·배치 준비 | 0.139 |
| 사람 파라미터·스냅숏 | 0.101 |

draw calls335.731, objects drawn1384.066, process32.592ms. 계측 진단은 FPS 수용 실행과 구분한다.

### 고친 것

- 기존 MultiMesh 배치와 캐시된 링 텍스처를 유지하고, 정확한 자유 구간 탐색·변하지 않은 배치 결과 재사용으로 매 프레임 전수 배치를 제거했다. 마커14px·밝은 링·가구 외곽 링·어두운 윤곽·선택/징집 표시를 유지한다.
- p9에서 스프라이트 보간 위치가 마커 배치를 계속 무효화하던 경로를 고쳤다. 안정된 타일 목표를 배치 입력으로 사용한다.
- Chronicle 기본 목록은 `assets/ui/chronicle_feed.json`의 kind visibility로 필터링한다. 숨긴 행은 목록 용량을 차지하지 않고, 개인 기록과 왜?는 원래 데이터를 읽는다. 시뮬레이션 기록은 바꾸지 않았다.
- 방어 알림은 일반 steward sequence와 별개인 chronicle cursor로 조회한다. 한국어 합성 문구의 기존 단어 경계 처리를 유지한다.
- 오너 승인 어댑터는 원래 수용된 M-CORE 검증 chain 상속과 새 검증을 구분한다. 기존 Rust·lock·역사 검증기 의존성을 고정하고 새381 BASE 성능 비교에는 P7 예외를 적용하지 않는다. 원래 외부 작업트리 검증기는 **NOT_RERUN**이며 원시 실패는 보존한다.
- p10은 저장 직후 직접/캐시 시계를 대조해 캡처의 seed/tick 상태를 증명한다. 캡처 도구 수정이며 게임 규칙 수정이 아니다.

### FPS 전후 표 (인구 10/40/200/1,000 × ¼×·½×·1×, 막대 판정)

후 값은 b0401ca2e의 완료된 VALID 영수증만 사용한다. ¼× 전 값은 이번 게시 BASE 비교, ½×·1× 전 값은 원래 오너 측정의 역사 기록이다. **필수 FPS7막대 모두 VALID/PASS**다. GPU·자연 재굽기·전체 gate 수용과는 별개다. N40 첫 반복은 종료부하9.81로 INVALID이며 결과에서 제외했다.15분 대기 후 유효한 a2만 아래에 사용한다.

| 초기 인구 | ¼× 전 → 후 / 기준 | ½× 전 → 후 / 기준 | 1× 전 → 후 / 기준 |
|---:|---|---|---|
| 10 | 317.107 → **377.513 PASS** / ≥144 (실제10–11) | — → NOT_RUN | — → NOT_RUN |
| 40 | 267.864 → **381.968 PASS** / ≥144 | — → NOT_RUN | — → NOT_RUN |
| 100 (추가 필수) | NOT_RUN | 164.9 → **179.936 PASS** / ≥150 | NOT_RUN |
| 200 | 34.948 → **246.246 PASS** / ≥120 | 119.3 → **123.757 PASS** / ≥105 | 225.8 → **244.102 PASS** / ≥200 |
| 1,000 | 0.350 → **115.303 PASS** / ≥100 | 35.98 → NOT_RUN (기존 실패 보존) | — → NOT_RUN |

완료 근거: ¼× [N10](/Users/rexxa/work/mux21-data/resume-20261028/comparison/candidate-n10-z0.25-a1/receipt.json), [N40 a2](/Users/rexxa/work/mux21-data/resume-20261028/comparison/candidate-n40-z0.25-a2/receipt.json), [N200](/Users/rexxa/work/mux21-data/resume-20261028/comparison/candidate-n200-z0.25-a1/receipt.json), [N1000](/Users/rexxa/work/mux21-data/resume-20261028/comparison/candidate-n1000-z0.25-a1/receipt.json); ½× [N100](/Users/rexxa/work/mux21-data/resume-20261028/comparison/candidate-n100-z0.5-a1/receipt.json), [N200](/Users/rexxa/work/mux21-data/resume-20261028/comparison/candidate-n200-z0.5-a1/receipt.json); 1× [N200](/Users/rexxa/work/mux21-data/resume-20261028/comparison/candidate-n200-z1-a1/receipt.json). 원본1/5/15 부하·프로세스·입력 불변 기록은 각 실행 디렉터리에 보존한다. N10은 정확히10명 고정 실험이 아니며 실제10–11, 나머지6회는 요청 인구와 같다. 필수 막대가 없는 조합의 NOT_RUN은 미측정을 명시하며 추정값을 채우지 않는다.

### 가시성 재확인

최종 HEAD의 GPU 낮/밤140px 대비·선택/징집 링 수용 재검사는 **PENDING**. 표식 회귀 도구의 실제 창 검사와 개별 방어 캡처는 전체 GPU 수용 검사를 대체하지 않는다. 자연 ¼× 재굽기는 seed0/day3 저장에서30초 준비+60초 관측, 시작/종료1분 부하2.81→2.70의 유효 실행이다. 측정 구간의17재굽기/16프레임을 `(reset_epoch, frame)`별로 합산한 최대는 **10.493ms/프레임**, 새 굽기는0건이다. **4ms 목표 미달**을 그대로 남기며 FPS 통과로 덮지 않는다. 준비 구간은 합계에서 제외했다. [프레임별 원시 집계](/Users/rexxa/work/mux21-data/resume-20261028/acceptance-summary.json).

### 플레이 대신 확인한 화면 (정보창 이유, 연대기, 방어)

오너 플레이는 대신 수행하지 않았다. 다음은 실제 UI 캡처와 자동 관측이며 최종 a5의16장을 두 독립 검토자가 확인했다. 이유6/6·왜?6/6·토스트2/2가 읽히며 잘리지 않는다. [시각 검토1](/Users/rexxa/work/mux21-data/resume-20261028/final-a5-visual-review-1.json), [시각 검토2](/Users/rexxa/work/mux21-data/resume-20261028/final-a5-visual-review-2.json). [자동 근거 요약](/Users/rexxa/work/mux21-data/resume-20261028/play-final-summary.json)은5개 실행 모두 automated_pass=true, 오류0을 기록한다. 이전 실패와 a1 검토도 별도로 보존한다.

| 상황 | KO 배율1.0 | KO 배율1.25 | 자동 관측 |
|---|---|---|---|
| 일하는 성인 | [정보창](/Users/rexxa/work/mux21-data/resume-20261028/play-final-work-a5/work-1.0-person.png) · [왜?](/Users/rexxa/work/mux21-data/resume-20261028/play-final-work-a5/work-1.0-why.png) | [정보창](/Users/rexxa/work/mux21-data/resume-20261028/play-final-work-a5/work-1.25-person.png) · [왜?](/Users/rexxa/work/mux21-data/resume-20261028/play-final-work-a5/work-1.25-why.png) | 이유3줄/표시3줄, fit=true |
| 배고픈 사람 | [정보창](/Users/rexxa/work/mux21-data/resume-20261028/play-final-hunger-a5/hunger-1.0-person.png) · [왜?](/Users/rexxa/work/mux21-data/resume-20261028/play-final-hunger-a5/hunger-1.0-why.png) | [정보창](/Users/rexxa/work/mux21-data/resume-20261028/play-final-hunger-a5/hunger-1.25-person.png) · [왜?](/Users/rexxa/work/mux21-data/resume-20261028/play-final-hunger-a5/hunger-1.25-why.png) | 이유3줄/표시3줄, fit=true |
| 늑대에게서 피하는 사람 | [정보창](/Users/rexxa/work/mux21-data/resume-20261028/play-final-flee-a5/flee-1.0-person.png) · [왜?](/Users/rexxa/work/mux21-data/resume-20261028/play-final-flee-a5/flee-1.0-why.png) | [정보창](/Users/rexxa/work/mux21-data/resume-20261028/play-final-flee-a5/flee-1.25-person.png) · [왜?](/Users/rexxa/work/mux21-data/resume-20261028/play-final-flee-a5/flee-1.25-why.png) | 이유2줄/표시2줄, fit=true |

실제 표시 문구는 “지금: 일하기 · 이유: 맡은 작업 진행 중 · 위험에 처함: 아니요”, “지금: 식사 · 이유: 새 활동 가능 · 배고픔을 해소할 수 있음”, “지금: 안전한 곳으로 피하기 · 이유: 위험에 처함”이다. 요청의 “지금: … — 왜: …” 의미를 기존 M-CORE의 실제 표현으로 확인했으며 새 문구로 바꿨다고 주장하지 않는다. 여섯 왜? 버튼 모두 실제 입력 down/pressed/up과 팝업 표시를 기록했다.

Chronicle은 seed0/tick4320, 경과3일의 원시111행을 세었다. kind별1:10,12:1,23:1,26:1,45:60,47(mind_choice):38이다. 전후 원시 행 canonical JSON SHA256은 같은 `38d781859829766ee688237e2e4bce9c472dab95726ca9e7030b396cbac8c0a4`다. 기본 목록60행에서는47을 제외하고45:58/26:1/23:1을 유지한다. 전체 evidence 파일이 바이트 동일하다는 뜻은 아니다.

- 배율1.0: [필터 전](/Users/rexxa/work/mux21-data/resume-20261028/chronicle-before-a1/chronicle-day3-1.0.png) → [필터 후](/Users/rexxa/work/mux21-data/resume-20261028/chronicle-final-a5/chronicle-day3-1.0.png).
- 배율1.25: [필터 전](/Users/rexxa/work/mux21-data/resume-20261028/chronicle-before-a1/chronicle-day3-1.25.png) → [필터 후](/Users/rexxa/work/mux21-data/resume-20261028/chronicle-final-a5/chronicle-day3-1.25.png).
- 방어 [1.0](/Users/rexxa/work/mux21-data/resume-20261028/play-final-defence-a5/defence-1.0.png) · [1.25](/Users/rexxa/work/mux21-data/resume-20261028/play-final-defence-a5/defence-1.25.png): tick18722, 징집2명·drafted_mark=true와 “마을이 스스로: 늑대를 막으려고 2명을 불렀어요” 토스트를 기록했다. 원래 방어 event의 모든 필드가 같고 두 사람의 붉은 스프라이트 윤곽을 시각 확인했다. 이1× 장면에서 별도 원형 지면 링을 확인했다고 주장하지 않는다.

¼× 추가 방어 캡처 [1.0](/Users/rexxa/work/mux21-data/resume-20261028/quarter-defence-a1/defence-quarter-1.0.png) · [1.25](/Users/rexxa/work/mux21-data/resume-20261028/quarter-defence-a1/defence-quarter-1.25.png)는 같은 실제 event를 재현해 토스트와 두 붉은 원형 소집 링을 함께 보인다. [선택+소집](/Users/rexxa/work/mux21-data/resume-20261028/quarter-defence-a1/defence-quarter-selected-1.25.png) · [선택 이동](/Users/rexxa/work/mux21-data/resume-20261028/quarter-defence-a1/defence-quarter-selection-moved-1.25.png)에서 실제 클릭에 따른 선택 이동·소집 유지12검사가 통과했다. 외부 캡처 어댑터는 원래 방어 절차의 카메라 배율과 추가 입력 관측만 바꿨다. [6장 시각 검토](/Users/rexxa/work/mux21-data/resume-20261028/quarter-defence-visual.json)도 링·토스트를 확인했고,1.25의 경고/소집 명단과 정보창 제목/초상 겹침은 남았다.

추가 `test_core_mind_ui.gd` 실제 창 검사도 실패0: 배고픔·위협 전환 OBSERVED, 데이터만으로 필터 반전, 기본 목록 숨김, 실제 개인 critical 기록·이야기·왜? 후보 보존을 확인했다. [로그](/Users/rexxa/work/mux21-data/resume-20261028/core-mind-ui-a1.log). 이 추가 도구의 자체 screenshot visual_verdict는 PENDING_REVIEW이며 요청된 시각 수락은 위의 a5/¼× 별도 검토에 귀속한다.

### 닫은 GitHub 이슈 / 새로 연 이슈

닫은 이슈 없음. ISS-343 [#638](https://github.com/hyunlord/new-world/issues/638) OPEN 유지. 새 ISS-344 방어 토스트 [#655](https://github.com/hyunlord/new-world/issues/655), ISS-345 후속 UI gate 어댑터 [#656](https://github.com/hyunlord/new-world/issues/656)는 수정 커밋이 로컬 미게시이며 전체 검증·게시 대기로 **OPEN**이다.

잔여 ISS-346 [#657](https://github.com/hyunlord/new-world/issues/657)은 KO UI1.25 늑대 경고와 정보창 이름·제목의 겹침을 기록한 **OPEN·미수정** 이슈다. 이유 본문과 왜?는 가리지 않는다.

자연 재굽기 목표 미달은 신규 ISS-347 [#658](https://github.com/hyunlord/new-world/issues/658) OPEN으로 남겼다. 기존 계절 전환 ISS-201/#576과 광범위 지연 ISS-030/#515에 원인을 단정해 합치지 않았다.

### 변경된 파일 목록 (커밋별)

아래 SHA는 로컬 커밋이며 아직 공개 링크를 주장하지 않는다. p0–p6의 목표/도구/재개 기록은 [결정 기록](MUX21_DECISIONS.md)과 보존 보고서에 남겼다.

| 커밋 | 변경 파일/범위 |
|---|---|
| p7 `f7c91a6e4` | `marker_layout.gd`, `lod_markers.gd`; `chronicle_panel.gd`, `agent_inspector_panel.gd`, `mux_hud.gd`; `assets/ui/chronicle_feed.json`; layout/play/toast 시험·`test_core_mind_ui.gd`; ADR0048/0049·결정 기록 |
| p8 `57f6e7881` | `tools/v8/core_ui_successor*.py`, 어댑터 시험2개, `core_gate_checks.sh`, `gate.sh`; ADR0050 |
| p9 `cf2bbafd7` | `agent_layer.gd`, `lod_markers.gd`, `marker_layout.gd`; layout/play 캡처 도구; ADR0051·결정 기록 |
| p10 `b0401ca2e` | `tools/v8/mux21_play_capture.gd` |
| p11 `e0f257869` | 보고서·과거 차단 보고 보존·HISTORY/ROADMAP/CHANGELOG·ISSUES·연락 그림·Godot 교훈 |
| p12 `c5ef23bd8` | 후속 어댑터 전체 Git 모드/링크 객체 증명·봉인·회귀시험·결정 기록 |

### 확인된 제한사항

전체 `rust/` tree는 게시 BASE381 및 기존 측정826과 동일하다(`c942d93fcd52bea5d7bfed2fb7e9c82de87b7a99`). 소스 동일성은 새 전체 identity/fixture gate 통과를 뜻하지 않는다. debug/release bridge 재빌드 완료 근거와 실제 캡처의 bridge 해시는 외부 증거에 보존한다.

도주1.25의 늑대 경고가 인물 이름을 덮지만 이유와 왜?는 가리지 않는다(ISS-346/#657 OPEN·미수정). 일부1.0 캡처의 작은 지도는 빈 화면이다. UI 설정1.25는 기존 화면 맞춤 상한으로 실제 약1.2233이 될 수 있다. 물리 트랙패드/오너 플레이 수용은 이 캡처가 대신하지 않는다.

필수 FPS7막대는 통과했으며 GPU 가시성, 전체46단계 gate, 원격 fast-forward 게시·이슈 폐쇄가 남았다. 이전 혼잡 실행과 잘못된 캡처·보간 후보 실패는 보존한다. 역사 M-CORE side64 FAIL은 어댑터가 지우지 않으며 새 비교는 예외 없이 통과해야 한다. 원래 워크트리와 외부 도구 `.gitignore`/`.ignore` 변경은 보존하고 임의 제거하지 않는다.

### Gate 결과

**현재 HEAD 전체 gate: NOT_RUN. 게시: NOT_PUBLISHED.** 집중 검사 근거는 p7 알림22검사, p8 어댑터22단위시험·shell syntax, 실제 MultiMesh 배치244검사, a5 UI 캡처와 추가 mind UI 자동 검사다. 이 결과는 전체 gate를 대신하지 않는다. p11 전 문서 검사 PASS, 시스템 표 재생성 diff0. 봉인 사전검증의 기존 skill symlink 처리 오류를 p12에서 고쳤으며23시험·실제 Git inventory·역사 pin 검증 및 독립 검토 PASS다. 실패한 사전 봉인은 보존했다. 최종 HEAD의 전체 gate는 후속 기록이 필요하다. 원격이 앞서면 충돌 없는 rebase 뒤 전체 재게이트 규칙을 적용한다.

### 오너가 할 일 (실행 한 줄: /Users/rexxa/work/new-world-play2-dev/tools/play.sh main)

현재 명령은 게시된 버전을 실행한다. 이 M-UX2.1 후보는 아직 게시되지 않았다.

```bash
/Users/rexxa/work/new-world-play2-dev/tools/play.sh main
```

`git ls-remote` 최종 게시 후 확인: **PENDING**. 재개 때 확인한 BASE381을 현재 후보 게시 증거로 대신하지 않는다.
