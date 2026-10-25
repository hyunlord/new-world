# M-UX Stage A 구현·검증 보고서

**기술 게이트 PASS: p12 `f38d488f4`, 35/35 단계, status 0. A8 수용 기준은 FAILED다.** 2026-10-25 09:07:08 KST에 두 번째 전체 게이트가 끝났으며 debug/release는 각각 **2,060 PASS / 0 FAIL / 1 ignored**다. 첫 게이트의 Chronicle 테스트·문서 목록 실패는 보존하고 최소 수정 후 재검증했다. 이 보고서 작성 시점에는 보고서 갱신을 포함한 최종 커밋의 gate3와 게시가 **PENDING / NOT_PUBLISHED**다. 최종 게시에는 그 정확한 커밋의 gate status 0과 두 원격 SHA 일치가 필요하다.

UI14는 268검사·실제 8장 검토92점, 배율5는 220검사·9장 검토91점으로 통과했다. 그러나 물리 입력은 **NOT_VERIFIED**이며, 집사는 보정1–5 및 held-out의 침상30/32·장작21/32·생존22/32 기준을 충족하지 못했다. 정책은 동결했고 평가 후 보정하지 않는다. D29/D34 이후에도 정상 debug 플레이에 반복73–99ms 지연이 남아 **ISS030 OPEN**이다. 기술 게이트 통과는 기준 면제·사용자 승인·FPS 불변·전체 목표 달성을 뜻하지 않는다.

개별 UI·측정 로그는 실행 당시 소스의 증거이며, 서로 다른 실행을 하나의 최종 검사로 합산하지 않는다. 완료된 전체 게이트 영수증과 이전 실패·구성요소 검사를 아래에서 구별한다.

## 대상과 증거 경계

- 작업 경로: `/Users/rexxa/work/new-world-mux-dev`, 브랜치 `v8/mux`.
- gate2 검증 소스: `f38d488f43156145a44898671e77b26b3ee6c496` (p12). D29 `91f029e9e`, D34 `78af5e245`, D35 최소 수리까지 포함한 깨끗한 HEAD다. 이 문서 갱신을 포함한 후속 커밋의 gate3는 별도이며, 아래 gate2 결과를 그 커밋의 결과로 전용하지 않는다.
- 요구사항: [MUX_GOAL.md](MUX_GOAL.md). 선택과 실패 이력: [MUX_DECISIONS.md](MUX_DECISIONS.md), 특히 D3–D7, D9–D35.
- 원시 로그·캡처·동결 바이너리는 `/Users/rexxa/work/mux-data/`에 보관한다. 다른 체크아웃에서 실행한 결과를 이 패스의 검증으로 사용하지 않는다.
- 문서 준비 시점(2026-10-25 09:07 KST 이후)의 게시 상태: **NOT_PUBLISHED**. 최종 exact-HEAD 게이트와 게시 영수증은 `/Users/rexxa/work/mux-data/gate-stage-a-3/` 및 `stage-a-publication.json`에 별도로 기록한다. 아직 생성되지 않은 영수증을 완료 근거로 사용하지 않는다.

## A1–A10 구현 현황

| 요구 | 현재 구현 | 검증 범위와 남은 일 |
|---|---|---|
| A1 카메라 | 두 손가락 이동·핀치, 커서 기준 휠 확대, 왼쪽/오른쪽 드래그, 기존 키보드 이동, 화면에 맞춘 초기 창, 전체 화면 전환 | 창 기반 합성 이벤트 검사 통과. 동일 이벤트의 중복 전달을 source identity로 제거한 D14 회귀 포함. 물리 트랙패드 감각·OS 전달은 미검증 |
| A2 화면 조작 버튼 | 속도·일시정지, 확대/축소, 캠프로 이동, 단축키 툴팁 | 통합 UI 검사에서 실제 버튼 경로 확인 |
| A3 미니맵 | 지형·방·사람·동물·카메라 영역, 클릭/드래그 이동, 접기, 낮은 빈도 갱신 | 구성요소38개 통과. 전용 ui-perf-2 정지 tick1 장면 상한 **345µs**로500µs 이내; UI14 **545µs**는 불확정 이력이며 전체 게임 비용 증거가 아님 |
| A4 도움말 | `assets/ui/controls.json` 기반 조작표, 도움말 창, 버튼 툴팁 | 통합 및 배율 검사 통과. 기존 패널이 실제로 생성된 뒤 툴팁 확인 |
| A5 알림 | 데이터 기반 17종, 수량·설명·해결법, 원인 이동/선택·순환, 해제 후 재발 표시, 최대 2 Hz 평가 | 알림 254개 통과. D32는 전역 사용 가능 장작에 모든 정착지의 일일 필요량 합계 × 겨울 일수를 비교한다. 벌크 읽기 `get_ui_facts`와 UI 변환기를 사용하며 음식 필요량과 전역 부패 로트 의미를 보존 |
| A6 목표 | 첫해·시대 목표 패널, 실제 시뮬레이션 사실에 따른 표시 | 학습/목표 구성요소 검사 포함. 첫겨울 완료는 보존된 사망 날짜로 판정해 이후 사망으로 과거 성취가 해제되지 않음 |
| A7 학습 도우미 | 단계별 카드·강조, 다음/건너뛰기/끄기, 사용자 설정 저장, 실제 조작과 상태로 진행 | 최신 구성요소 128개 통과. 자동 명령의 승인 출처와 목표 완료를 구별. D28은 실제 패널 열림을 즉시 저장하고 상태 읽기는 2 Hz 유지; 통합 회귀 23개 통과 |
| A8 마을 스스로 | 데이터 규칙 6종, 일반 명령 경로·로그, 켜기/범위, 취소 보호·재생·저장, 출처가 있는 연대기와 토스트 | 단위·통합·이전 저장 마이그레이션 검사 있음. 보정 round1–round5 및 held-out ON FAIL; **A8 수용 FAILED**. 이주민 응답·자동 징집은 하지 않음 |
| A9 설정 | 가장자리 이동, 반전, UI 1.0/1.25/1.5배, 도우미, 새 게임 스튜어드 기본값 | 배율 검사 220개 통과. 기본값은 새 게임 경계에만 적용하고 로드한 저장의 설정을 덮어쓰지 않음(D11) |
| A10 렌더러 | 물을 땅보다 어둡고 푸르게 조정, 해안 반경·눈·나무/벽 주변 눈 마스크 구현 | 실패 이력 68→79→90(당시 수치 FAIL)을 보존. comparison6 시각 90 PASS, D31 이후 실제 GPU run4 실패 0; 최종 소스 게이트와 사용자 승인은 별개 |

현재 A6/A9를 포함해 드롭한 Stage A 항목은 없다. 위 표의 “구현”은 최종 런타임·시각 승인 상태와 별개다.

## 실행된 검사

아래는 개별 실행의 결과다. 같은 검사군의 반복 실행 수를 합산하지 않는다.

| 검사 | 확인된 결과 | 증거 |
|---|---|---|
| 통합 창 UI run14 | **268 PASS / 0 FAIL**, `synthetic windowed`; 알림·언어·원인 패널 회귀 포함, 시각 8장 **92/100 PASS** | `/Users/rexxa/work/mux-data/a-ui-test-14.log`, `ui-run14/mux-ui-results.json`, `ui-visual-run14.md` |
| UI 배율 run5 | **220 PASS / 0 FAIL**, 1.0/1.25/1.5배; 9장 검토 **91/100 PASS** | `/Users/rexxa/work/mux-data/a-scale-test-5.log`, `scale-run5/mux-scale-results.json`, `scale-visual-run5.md` |
| 알림 | **254 PASS / 0 FAIL**, D32 다중 정착지 전역 장작 경계 포함 | `/Users/rexxa/work/mux-data/a-alerts-test-3.log` |
| 학습·목표·설정 | **128 PASS / 0 FAIL**, D31 격리와 D32 겨울 전체 필요량 회귀 포함 | `/Users/rexxa/work/mux-data/a-learning-test-9.log` |
| 미니맵 | **38 PASS / 0 FAIL** | `/Users/rexxa/work/mux-data/a-minimap-test-3.log` |
| 기존 패널 배치 run2 | **23 PASS / 0 FAIL** | `/Users/rexxa/work/mux-data/a-legacy-test-2.log` |
| 카메라 단독 run5 | **27 PASS / 0 FAIL** (PASS 행 27개), `MUX_CAMERA_DONE failures=0` | `/Users/rexxa/work/mux-data/a-camera-test-5.log` |
| UI 사실·스튜어드 브리지 Rust run2 | **9 PASS / 0 FAIL**; 관측 후 실행 동일성, 첫겨울 사망 날짜, 스튜어드 명령 경계 검증 포함 | `/Users/rexxa/work/mux-data/a-ui-facts-test-2.log` |
| D27 스튜어드 통합 | **9 PASS / 0 FAIL / 1 ignored**. v6 fixture는 이 실행에서 제외됐고 아래 별도 실행으로 확인 | `/Users/rexxa/work/mux-data/steward-d27-tests.txt` |
| D27 스튜어드 단위 | **10 PASS / 0 FAIL**; 소유 출처·플레이어 우선권·데이터 역할·계획 회귀 | `/Users/rexxa/work/mux-data/steward-d27-unit.txt` |
| 실제 v6 저장 마이그레이션 | **1 PASS**, OFF 이행 및 재저장 후 계속 실행 동일성 | `/Users/rexxa/work/mux-data/a-v6-test-3.log` |
| GDScript strict run5 | **67 파일 / FFI 57 호출 PASS**; 최종 exact-HEAD 전체 게이트와 별개 | `/Users/rexxa/work/mux-data/a-gdscript-strict-5.log` |
| D33 소스 검토 | 범위 내 actionable finding 없음; 런타임·시각 판정과 별개 | `/Users/rexxa/work/mux-data/d33-code-review.md` |
| D34 profile1 scoped tests | 단위 **10 PASS**, 통합 **9 PASS / 1 ignored**, 명시적 v6 migration **1 PASS** | `/Users/rexxa/work/mux-data/profile1/{steward-unit,steward-integration,v6-migration}.log` |
| D34 fresh OFF safety | **PASS**:9 literal 요약·228748값 차이0·64상세파일 동일(git_sha 제외) | `/Users/rexxa/work/mux-data/profile1/safety-compare.log` |
| 기준선 perf | **exit 0**, 2026-10-25 06:55:07 KST 완료; 후보 비교·게임 FPS 아님 | `/Users/rexxa/work/mux-data/baseline/identity/perf/perf_summary.json` |
| 전체 sim-test (D34 이전) | **exit 0; 1403 PASS / 0 FAIL / 1 ignored**, 102 result summaries; ignored는 별도 v6 실행과 구별 | `/Users/rexxa/work/mux-data/a-sim-test-full-1.log`, [D29 검증](D29_VERIFICATION.md) |
| 활성 debug 창 run2 | **exit 0**, ON/OFF 각 1441 tick; ON 평균 2422.547 µs·최대 500592 µs, OFF 평균 1634.375 µs | `/Users/rexxa/work/mux-data/active-perf-2/mux-active-perf.json`, `d29-active-comparison.json` |
| 문서 검사 run4 | **PASS**, 스냅샷 56 getters / 789 keys | `/Users/rexxa/work/mux-data/a-docs-check-4.log` |
| 렌더러 마스크 계약 | `MUX_RENDERER_DONE failures=0` | `/Users/rexxa/work/mux-data/a-renderer-test-1.log` |
| D31 이후 지면 실제 GPU run4 | **failures=0**, 골든 변경 없음; comparison6 시각 **90/100 PASS**는 별도 기존 증거 | `/Users/rexxa/work/mux-data/a-ground-rendered-4.log`, `renderer-comparison6/` |
| 지면 수치/골든 갱신 실행 | `MART2_GROUND_DONE failures=0`; rendered_pixels는 **SKIP** | `/Users/rexxa/work/mux-data/a-ground-golden-refresh.log` |

미니맵 run5 측정은 `conservative_frame_usec=455`, `max_frame_usec=269`, `max_draw_usec=186`, 6회 갱신이다. 전체 게임 규모별 성능 보증은 아니다. 지면 검사의 최소 야간 명도 차이 0.1112는 기준 0.10을 넘지만, 해안과 눈의 시각적 자연스러움을 증명하지 않는다.

최신 UI14의 미니맵 보수적 상한은 **545µs = process 최대311 + draw 최대234**다. 서로 다른 프레임의 최대값 합계이므로 실제 단일 프레임이500µs를 넘었다는 증거는 아니지만, 이 실행으로 A3의500µs 이내를 입증할 수도 없다. 이전455µs와 전용 정지 장면381µs는 각 실행 범위에만 유효하다. 새 브리지 빌드 후 전용 `ui-perf-2/mux-ui-perf.json` 실행은 exit0, stable=true, 각 phase600표본으로 완료됐다. 시작·워밍업을 포함한 보수적 상한은 **345µs = process272 + draw73**이고 lifetime budget=true다. 이는 정지 tick1 장면에서만500µs 이내를 입증하며 UI14 불확정 결과나 전체 게임/FPS 판정을 대체하지 않는다. 초과 시 임의로 두 최대값을 빼거나 유리한 구간만 고르지 않는다.

## 스튜어드 보정 round1 — 엄격 기준 FAIL

동결 조건은 시드 **0–7**, **48일**, 스튜어드 ON, 플레이어 명령 스크립트 없음이다. 최대 6회 중 첫 보정이며, 저장된 실패 결과를 유지한다. 당시 미사용이던 시드 **1000–1031**로 규칙을 맞추지 않는 조건이었다.

| 지표 | 결과 | 보정 합격 기준 | 판정 |
|---|---:|---:|---|
| day1 비축 구역 | 8/8 | 8/8 | PASS |
| day10 벽 있는 집·전원 침상 | 8/8 | 8/8 | PASS |
| 첫겨울 추위 사망 0 | 8/8 | ≥7/8 | PASS |
| 한파 중 사용 가능한 장작이 0이 되지 않음 | **4/8** | ≥6/8 | **FAIL** |
| day48 마을 생존 | **4/8** | 8/8 | **FAIL** |
| 물품 원장 정확성 | 8/8 | 8/8 | PASS |

증거: `/Users/rexxa/work/mux-data/calibration/round1/score.json`, `probe/summary.json`, 동결 `steward.ron`, `source_sha256.json`, `binary_sha256.txt`.

사망 73명은 모두 combat이며 굶주림·탈수·노출·노화 사망은 0이다. 시드 1/2/3/6이 멸절했고, 각 beast 사건의 사망 합계가 해당 시드의 combat 합계와 일치한다. 모든 beast 기록의 drafted는 0이다. 중첩된 illness pressure의 `deaths`는 시간 구간 집계이므로 질병 사망 원인으로 해석하지 않는다.

장작 실패 시드 1/2/3/5의 한파 중 0 tick은 각각 **480/720/1800/360**이다. 시드 1/2/3은 일별 성인 수가 각각 day35/day29/day35에 0이 된 뒤 늦은 한파에서 비축이 소진됐다. 시드 5는 day48에 지상 장작 90개가 남았으나 비축 구역·정착지 장작은 모두 0이었다. 이는 사용 가능한 위치로 운반하는 문제를 가리키지만, 원시 좌표·행동 추적 없이 특정 작업 실패까지 확정하지 않는다. 정확한 부족 시작/종료 tick은 저장되지 않았고 누적 tick과 일별 표본만 있다.

D15의 후속 후보는 기존 비축 구역과 작업 공간을 지붕 아래로 모으는 방향이다. 늑대 규칙 변경, 자동 징집, 판정 기준 완화는 하지 않는다. 이 가설의 round2 결과는 아래와 같이 실패로 보존한다. 당시 32개 시드 평가는 미실행이었다. 현재 완료 결과는 아래 held-out 절에 기록한다.

## 2026-10-25 보정 round2 — 엄격 기준 FAIL

시드 0–7 ×48일, ON·플레이어 명령 없음 조건을 유지했다. day1 비축·day10 전원 침상·첫겨울 추위 사망 0·원장 정확성은 각각 **8/8 PASS**지만, 한파 장작은 **3/8 (필요 ≥6/8)**, day48 생존은 **3/8 (필요 8/8)**로 FAIL이다. 증거: `/Users/rexxa/work/mux-data/calibration/round2/score.json`, `probe/summary.json`, `registration.json`, 동결 소스·바이너리 해시.

사망은 round1과 같은 **73명, 모두 combat**이다. 방은 커졌지만 8개 시드 모두 비축 구역의 지붕 덮임은 0이며, 생존 마을은 4개에서 3개로 줄었다. 침상 확보만으로 야외 작업·이동 중 늑대 노출을 막았다고 볼 수 없다. D19의 후속 가설은 실제 비축·작업 공간을 덮는 배치와 위협 당시 행동을 확인해야 하며, 아직 생존 기준 충족의 증거가 아니다. 늑대 변경·자동 징집은 하지 않는다.

장작 판정은 **관측 후 성공 3개(1/6/7), 실제 고갈 3개(2/3/5), 한파 미관측 2개(0/4)**다. 고갈 tick은 각각 720/1800/1740이다. 미관측 시드는 `cold_snap_observed_ticks=0`, `snaps=[]`이고 skipped pressure가 round1의 4에서 5로 늘었다. 늦은 질병이 지속된 기록과, 환자가 많으면 major pressure를 미루다가 계절 창 끝에서 버리는 기존 규칙은 한파 누락 원인을 뒷받침한다. 다만 일별 환자·보류 사유 추적이 없어 정확한 누락 경로는 확정하지 않는다. **미관측을 성공으로 세지 않고 FAIL로 유지하며 분모 8/32, 기준과 사건 일정을 바꾸지 않는다.** 강제 한파도 추가하지 않는다.

## 보정 round3·round4 — 동결 실패 결과

모두 시드 0–7 ×48일, ON·플레이어 명령 없음이다. `/Users/rexxa/work/mux-data/calibration/round3/`와 `round4/`의 `registration.json`, `score.json`, `probe/summary.json`, 소스·바이너리 해시를 보존한다. round4 종료 당시 최대 6회 중 4회 사용했고 시드 1000–1031은 미사용이었다. 이후 round5와 동결 후 held-out 결과는 아래에 분리한다.

| 보정 | 비축 / 침상 / 추위 사망 0 / 원장 | 한파 장작 (필요 ≥6/8) | day48 생존 (필요 8/8) | 판정 |
|---|---|---|---|---|
| round3 | 각각 8/8 | **3/8** | **5/8** | **FAIL** |
| round4 | 각각 8/8 | **5/8** | **4/8** | **FAIL** |

Round3: 사망 61명은 모두 combat, 생존 주민 26명이다. 장작은 관측 성공 0/6/7, 고갈 1/3/5(540/1800/540 tick), 미관측 2/4로 분리한다. 8개 시드 모두 첫 지붕·최종 배치의 비축 구역과 지붕 교집합이 0이고 작업 위치도 모두 야외여서 공동 작업·비축 보호 효과를 입증하지 못했다. 사망 직전 61건 모두 야외·Fleeing 상태이며 도달 가능한 지붕까지 2–36칸이었지만, 이 관측만으로 도주 목적지 선택의 원인을 확정하지 않는다.

Round4: 생존 시드는 1/4/5/7, 주민은 17명이고 사망 69명은 모두 combat이다. 한파는 8개 시드 모두 관측했으며, 고갈은 0/2/3(480/720/1800 tick), 나머지 5개는 관측 후 성공이다. 미관측을 성공으로 바꾸거나 분모를 줄인 결과가 아니다. 최종 배치에서 시드 4를 제외한 7개 시드는 비축 구역 16/16칸이 지붕 아래에 있고, 모든 시드의 작업 위치 3/3개가 덮였다. 그러나 공동 방 역할은 Workshop이며 모든 시드의 `bedroom_sleepers`는 0이다. 기존 `stock_covered_end`/`stock_share_end`는 여전히 0이므로 이 지표와 좌표 기반 지붕 교집합을 같은 값으로 표현하지 않는다.

D27은 round5 전에 등록한 정책 가설이다. 전적으로 스튜어드가 지은 침상·경계의 출처와 완성을 확인한 뒤 기존 House 규칙 안에서 일반 `SetRoomRole` 명령으로 데이터의 completion_role을 적용한다. 플레이어의 유효한 역할 선택은 우선하며, Bedroom 선호·수면·기분·질병 전파와 Workshop 제작 시간 혜택 상실은 실제 행동 변화다. 단순 이름 변경이나 생존 개선으로 미리 보고하지 않는다. 늑대·도주·징집·자원·비축 위치·판정 기준은 유지하고, 당시 남은 보정은 최대 2회였다. D27 검증·round5와 D30 동결은 아래와 같다.

## D27 검증·round5 실패·D30 정책 동결

D27은 완성된 경계·문·침상의 스튜어드 소유 출처를 모두 확인한 뒤 일반 역할 명령을 적용한다. 최종 단위 10개·통합 9개와 별도 실제 v6 마이그레이션 검사는 통과했다. 통제된 Rust 건설 fixture는 tick3587에 지붕 아래 비축 16칸·침상 10개, **tick5027에 주민 10명 전원의 지붕 있는 Bedroom 침상 배정**을 확인했다. 이는 통제된 fixture이며, **실제 Main 창 UI9의 tick2880 완성 집**과 서로 다른 실행이다. 어느 쪽도 48일 생존 증거로 대체하지 않는다.

Round5는 시드 0–7 ×48일, ON·플레이어 명령 없음의 다섯 번째 동결 보정이다. 비축·침상·추위 사망 0·원장 각각 **8/8 PASS**, 한파 장작 **3/8(필요 6)**, day48 생존 **4/8(필요 8)**로 **FAIL**이다. 증거는 `/Users/rexxa/work/mux-data/calibration/round5/{registration.json,score.json,probe/summary.json}`와 동결 소스·바이너리 해시다. Bedroom 수면은 관측됐지만 생존 기준 달성을 입증하지 못했다.

D30은 근거 없는 여섯 번째 정책 보정을 하지 않고 D27 정책을 round5 소스/바이너리로 동결한다. **최대 6회 중 5회 사용**했으며, 아래 held-out 결과를 본 뒤 정책을 조정하지 않는다. 이는 수용 기준 면제가 아니다.

## 동결 후 held-out ON 평가 — 완료 / FAIL

기존 미사용 시드 **1000–1031 ×48일**, ON·플레이어 명령 없음으로 평가했다. 이제 이 시드를 미사용이라고 표현하지 않는다. 증거: `/Users/rexxa/work/mux-data/heldout/{registration.json,score.json,on/summary.json}`.

| 지표 | 성공 / 전체 | 원래 필요 성공 수 | 판정 |
|---|---:|---:|---|
| day1 비축 구역 | 32/32 | 32 | PASS |
| day10 벽 있는 집·전원 침상 | **30/32** | 32 | **FAIL** |
| 첫겨울 추위 사망 0 | 31/32 | 28 | PASS |
| 한파 장작 보호 | **21/32** | 24 | **FAIL** |
| day48 마을 생존 | **22/32** | 32 | **FAIL** |
| 물품 원장 정확성 | 32/32 | 32 | PASS |

**A8 수용은 FAILED다.** 분모·기준을 바꾸거나 실패를 기술 검사 통과로 상쇄하지 않는다. paired original-baseline/candidate OFF 32시드 요약은 literal byte 동일이다 (`heldout/off-identity.txt`). 이 OFF 통과는 ON 수용 실패와 별도다.

## OFF 안전망 — 중간 소스만 통과

동결 round3 후보(candidate-a3) 바이너리와 불변 기준 바이너리를 비교한 결과, baseline identity는 공통 시뮬레이션 값 **228,748개에서 차이 0**, 상세 파일 **64개**가 `git_sha` 외 필드 모두 같았다. colony 요약 **9개**도 literal byte 비교를 통과했다.

증거: `/Users/rexxa/work/mux-data/candidate-a3-compare.log`, `baseline/`, `candidate-a3/`. 이전 candidate-a1/a2 통과 기록도 보존한다. 범위는 soak, story, neighbours, knowledge F10, M5.5 pressure/nodraft/storyoff/noneighbours/quiet 고정 fixture다.

**candidate-a4도 PASS**다: 공통 값 228,748개 차이 0, 상세 64개 파일은 git_sha 외 동일, 9개 colony 요약 literal byte 동일 (`candidate-a4-compare.log`). D24/D25/D26을 포함하지만 이후 D27 및 최종 소스를 포함하지 않는다. **candidate-a5와 D29 candidate-opt1도 PASS**다: 각각 공통 값 228,748개 차이 0, 상세 파일 64개는 git_sha 외 동일, colony 요약 9개 literal byte 동일 (`candidate-a5-compare.log`, `candidate-opt1-compare.log`). paired held-out OFF 32시드 요약도 동일 SHA256 `3e94f1229ab97b2f1a303c6285756c83cac504498a5cc12b6912a470193172dc`다 (`heldout/off-identity.txt`). 이들은 각 동결 실행의 증거이며 최종 exact-HEAD 전체 게이트를 대신하지 않는다. 이 고정 fixture 통과는 요구사항의 동일 held-out 시드 1000–1031 OFF 재현 검사를 대신하지 않는다.

## 화면 증거와 시각 승인

| 캡처 | 의미 | 경계 |
|---|---|---|
| `ui-run14/hud.png` | 최신 통합 HUD | 합성 창 실행 당시 화면 |
| `ui-run14/help.png` | 도움말 | 물리 키보드/마우스 수용 판정 아님 |
| `ui-run14/lesson.png` | 학습 카드 | 장기 플레이 완료 증거 아님 |
| `ui-run14/alerts-synthetic-causes.png` | 시험용 알림 원인·이동 fixture | 자연 발생한 모든 17종 상황의 기록 아님 |
| `ui-run14/steward-housing-sites-tick-361.png` | 실제 스튜어드 공사 진행 | 완성 집과 별개 |
| `ui-run14/steward-completed-house-tick-2880.png` | 실제 명령으로 완성한 집 | 지붕 방·침상 및 배정의 단일 월드 증거; Rust 통제 fixture tick5027과 별개 |
| `ui-run14/causal-alert-target.png` | 실제 열린 원인 기록 패널 | 빈 기록 설명·배치 확인; 모든 원인 이력의 검증 아님 |
| `ui-run14/english-controls.png` | 영어 알림·명령줄·활성 건설 안내 | 모든 드롭다운의 시각 검증 아님 |
| `scale-run5/scale-{100,125,150}-*.png` | 배율별 HUD·도움말·건설 메뉴 9장 | 검토 91/100; 모든 기존 패널의 1.5배 검증 아님 |
| `renderer-comparison6/*.png` | 주야간·1×/¼× 20장 직접 검토 | **90/100 PASS**; 눈 벽 fixture는 render-only이며 실제 집 증거와 별개 |

위 경로의 공통 접두사는 `/Users/rexxa/work/mux-data/`다. 이전 UI run1의 겹침·잘림 실패, 배율 run1의 67점 REVISE, scale-run3의 92·91점과 UI9의 92점을 보존한다. UI10은 빈 동작 토스트로 **88 REVISE**, UI11은 영어 전환 뒤 기존 명령줄의 한국어 잔류·알림 제목 잘림으로 **89 REVISE**였다. UI12/13은 각각 268검사 중 fit 1개 FAIL 및 **89 REVISE**이며 제목 폭은 308 px 공간에 각각 316/312 px였다. UI14는 실제 생산 글꼴에서 “Newborn: no bed” 274/308 px로 맞고 의미와 수량이 보인다. 기준·임계값은 바꾸지 않았다.

최신 UI14 8장 직접 검토는 **92 PASS**(`ui-visual-run14.md`), scale5 9장은 **91 PASS**(`scale-visual-run5.md`)다. 완전한 동작 토스트, 번역된 명령줄·활성 안내와 원인 패널의 가독성·비겹침을 확인했다. UI14 검토자는 이전 도우미·툴팁·D28·D32 작성에 참여했으며 D33 및 현재 배치 통합 작성자는 아니다. 물리 입력 수용·사용자 승인·장기 생존을 뜻하지 않는다. 일부 한국어 단어 줄바꿈과 조밀한 단축키 줄은 비차단 한계다.

D33 부정 대조는 기존 코드에서 **193검사 중 18 FAIL**(`d33-negative.log`), 별도 알림 라벨 대조는 **6검사 중 1 FAIL**(`d33-label-negative.log`)을 보존한다. 이후 통과가 이 실패 기록을 지우지 않는다.

렌더러의 **68→79→90(당시 수치 FAIL)** 실패 이력은 유지한다. 이후 D21 comparison6는 20장 직접 검토에서 **90/100 PASS**, CPU·실제 GPU run3도 **failures=0**이다. 바위 강둑 일부의 직선 단차, ¼배에서 희미한 눈·나무 밑 눈더미, 약간 직사각형인 벽 눈 테두리, 거의 검게 보이는 ¼배 야간 물은 남은 한계다. 근거: `a-ground-rendered-3.log`, `renderer-comparison6/`, `.omx/state/mux/ralph-progress.json`. 수치·캡처 통과를 사용자 승인이나 최종 소스 전체 게이트로 확대하지 않는다.

`ui-perf-1/mux-ui-perf.json`은 tick1에서 일시정지한 장면의 UI/미니맵 비용이다. 실제 `frame_post_draw`를 썼어도 활성 스튜어드의 tick0·360 경계 비용이나 정상 속도 전체 게임 FPS를 입증하지 않는다. 별도 `active-perf-1/mux-active-perf.json`은 debug 브리지에서 ON/OFF 각각 1441 tick을 측정했다. ON advance 평균 3762.853 µs·최대 **1,048,501 µs**, OFF 평균 1830.508 µs·최대 11,158 µs다. ON tick0/360/720/1080/1440은 1048501/515069/890300/458772/17526 µs로, 시작 시점뿐 아니라 반복 경계에서도 큰 지연이 있다. frame_post_draw 평균 ON/OFF 11.035/8.683 ms·최대 1506.759/460.422 ms는 렌더링·시작 비용도 포함한다. 정상 속도 전체 게임 FPS 합격이나 전용 시스템 비용을 뜻하지 않는다.

D29는 기존 순수 tuple 키의 `sort_by_key`를 `sort_by_cached_key`로 바꾸는 제한된 최적화다. 후보·순서·정책은 바꾸지 않는다. **8시드 ×48일 ON 전체 요약(원본 round5와도 동일), 8개 실제 명령 로그, seed5의 69120-tick 전체 저장이 변경 전후 literal byte 동일**이며 candidate-opt1 OFF 안전망도 통과했다. D29 p9는 `91f029e9e`로 커밋됐다. 프로파일 변경 전 전체 sim-test는 exit 0, 1403 passed / 0 failed / 1 ignored, 102 result summaries로 완료됐다. ignored 항목은 이 실행의 PASS로 세지 않으며 별도 v6 증거를 유지한다. [D29 검증 기록](D29_VERIFICATION.md)을 참조한다.

일반 debug 브리지를 다시 빌드한 active-perf-2는 exit 0으로 ON/OFF 각 1441 tick을 완료했다. advance-call 평균은 ON **2422.547 µs**, OFF **1634.375 µs**, 최대는 ON **500592 µs**, OFF **10872 µs**다. ON tick0/360/720/1080/1440 경계는 **500592/238384/320699/182202/16250 µs**였다. `d29-active-comparison.json`에 이전 측정과 함께 보존한다. 순차 개발 진단이며 조용한 환경의 교차 release 비교가 아니므로 보정된 속도 향상률은 주장하지 않는다. 큰 지연이 남고 1 tick/frame 계측이므로 정상 게임 FPS 수용도 아니다. 이후 실제 자동 누적기 gameplay run1과 D34 profile1 gameplay run2는 완료됐다. 아래 D34 증거와 구별한다. 이 측정 당시에는 최종 게이트·게시가 남아 있었다. 이후 p12 gate2 결과는 아래에 기록한다.

D34 profile1은 `rust/Cargo.toml`에서 sim-systems 패키지의 dev opt-level만1로 변경했다(p10 `78af5e245`). 컴파일러 감사는 debug assertions·기본 overflow checks 유지를 확인했다. 673개 소스 해시 확인 및 ON/OFF tick1441 debug 저장 두 개의 literal byte 동일성을 기록했다. 이는 전 시드 안전성이나 새 프로파일 전체 테스트 통과가 아니다.

실제 자동 게임플레이 4개 phase가 완료됐다. ON 1x/4x는 **11.949/47.173 TPS**, 평균 **118.216/114.380 FPS**, 시작 구간을 포함한 최대 프레임 간격 **649.902/646.191 ms**다. OFF 평균은 **119.103/117.776 FPS**, 최대 **447.113/444.356 ms**다. ON 반복 계획 경계는 약 **73–99 ms**이며 큰 지연이 남는다. **ISS030 OPEN**; FPS 불변·보정된 속도 향상·지연 해결을 주장하지 않는다. 순차 debug 관측이며 대규모·release 수용 결과가 아니다. 상세와 원시 경로: [D34 검증 기록](D34_VERIFICATION.md). 새 profile1 단위10·통합9 PASS/1 ignored 및 별도 v6 마이그레이션1 PASS를 확인했다. fresh OFF safety는9개 요약 literal 동일·228748값 차이0·상세64파일 동일(git_sha 제외)로 PASS다. 이전 전체 1403/0/1 테스트는 프로파일 변경 전 결과다.

D31(`d0c7f2b04`)은 기본 설정 경로에만 선택적 `WORLDSIM_TEST_SETTINGS_DIR` 격리를 적용하고 명시적 경로 우선권과 일반 게임의 ON 기본값을 유지한다. 과거 렌더 fixture는 일반 명령으로 OFF를 명시한다. 격리된 부정 대조에서 tick360 명령 결과 불일치를 재현한 후, 수정한 렌더 일치 검사는 tick900의 사람/외형 10개를 통과했다. 지면·사람·나무 fixture와 GPU 지면 run4는 실패 0, 학습/설정은 115개 통과다. 사용자 설정 격리를 전체 user:// 파일 격리나 새 정책 변경으로 표현하지 않는다.

D32(`f1532878b`)는 목표와 알림 모두에서 전역 사용 가능 장작을 정착지별 양수 일일 필요량의 합계 × 겨울 일수와 비교한다. 하루치만 있거나 전체 필요량보다 1개 부족한 경우를 통과시키지 않는다. 최신 학습 128개·알림 254개 검사는 이 경계 회귀를 포함한다. D33(`c20e1fdee`)의 알림·언어·원인 패널 수정은 위 UI14·scale5·strict5와 별도 소스 검토로 확인했다.

현재 문서 화면은 [UI14 HUD 8장](https://github.com/hyunlord/new-world/blob/ca9f531ca6ef700ca223d99dba14d834b94b45da/docs/images/mux_contact_sheet.png)과 [렌더러 비교 12장](../../images/mux_compare_sheet.png)이다. 렌더러 시트 마지막 4장은 실제 건설이 아닌 render-only 눈 fixture로 명시했다. 이전 [mart21 화면 모음](https://github.com/hyunlord/new-world/blob/67a6232ab41a4bf5051b11303bcac97543b111f0/docs/images/mart21_contact_sheet.png)은 기준 커밋 `67a6232a`에 보존한다.

## 게시 조건과 남은 수용 한계

1. 보고서 갱신을 포함한 최종 커밋에서 gate3를 실행하고, status 0인 그 HEAD만 `v8/wip` 다음 `v8/main`에 fast-forward로 게시한다. 거절되면 강제 갱신하지 않는다. 현재 완료 영수증은 p12 gate2이며 게시 결과가 아니다.
2. round1–5와 held-out 실패, 원래 기준, 동결 소스·바이너리를 보존한다. 정책을 더 보정하거나 A8 수용을 선언하지 않는다.
3. D29 ON/OFF 동등성, D34 실제 debug 저장·별도 scoped tests·OFF 안전망 및 이전1.05초 지연을 보존한다. 최신 정상 플레이의650/646ms 최대 프레임(시작 포함)과 반복73–99ms 지연을 정지 UI 비용으로 대체하지 않는다.
4. comparison6·scale5·UI14의 실제 화면과 물리 입력 NOT_VERIFIED를 구별한다. B1 소스 검토 BR-TR03은 UI의 roofed-only 시험 조건이 probe의 roofed AND !dug AND doors-nonempty보다 약함을 확인했다. 실제 집 캡처가 실패했다는 판정은 아니며, dug/doorless 부정 대조는 아직 실행하지 않았다. 정확히48일인 동결 A8 결과에는 별도 BR-TR05(48일 이후 최종 생존 표본)의 문제가 적용되지 않는다.
5. Stage B 제품 수정은 Stage A 게시와 원격 SHA 확인 이후 시작한다. 검토·레지스트리 전환·Stage C 기록 작업은 아직 완료되지 않았다.

기준선 perf 실행은 2026-10-25 06:55:07 KST에 exit0으로 완료됐다. natural median mean0.253830 ms·p990.511 ms, quick mean0.383802 ms, stress_v26 mean0.369900 ms이며 이는 기준선 자료다. 후보 대비 속도 향상률을 뜻하지 않는다. AF01 전용 `ui-perf-2`는 exit0으로 완료됐고 정지 tick1 장면에서345µs 상한을 확인했다. UI14의545µs 상한 불확실성은 해당 실행 이력으로 보존한다.

## 첫 전체 게이트 — 실패 기록 (2026-10-25 08:03:24 KST)

깨끗한 `6b0f572e4afa0a5d642793b8201386a81838239c`에서35단계를 모두 실행했고 **status1, 32/35단계 exit0**이다. debug/release는 각각2059통과·1실패·1제외이며 같은 Chronicle 종류 수 기대값45(실제46)가 실패했다. 문서 목록 검사는 DESIGN.md 한 경로가 누락됐다. 이 두 원인을 D35에서 최소 수정하며 최초 실패 로그를 보존한다. 이 첫 게이트 종료 당시에는 새 커밋의 전체 게이트와 게시 결과가 없었다. 후속 gate2 결과는 아래에 기록한다.

나머지 결과: identity228748값 차이0, Chronicle64파일은git_sha를 제외하고 일치(19683항목); raw sample64파일은 기존 비교 규칙(git_sha와 제거된0값 키5개)을 적용해 일치했다. 원시 파일 전체의 무조건 바이트 동일을 뜻하지 않는다. 결정성, M1테스트58, 런처6/6, 렌더러 일치, 저장 왕복, 경제soak4시드×20일(사망0·KnowledgeLost0), F10 8/8, 이야기·이웃 각각2시드×48일, 해석기39재질/352수목 차이0, 렌더러45검사, 집사2시드×24일 결정성·원장·예약 검사는 통과했다. 집사 fixture는 A8 held-out 수용 실패를 대체하지 않는다.

자연 장면3회 중앙평균은 기준선0.253829912ms→후보0.253133183ms로+10% 초과가 없었다. 표준 bench 시간·할당·RSS 상한도 통과했다. nav N1000 이동129.0µs/tick은600µs 보고 예산 이내다. 이들은 release 측정이며 기존 debug 게임플레이의 반복 지연과 FPS 제한을 해소한 증거가 아니다. 원시 근거: `/Users/rexxa/work/mux-data/gate-stage-a-1/gate.txt`와 같은 디렉터리의 단계별 로그·perf/bench 자료.

D35 최소 수정 후 동일 Chronicle 테스트는 debug/release 각각1통과·0실패이며 fmt도 통과했다. 문서 목록은2221경로·1944행·누락0건이고 새 행과 규칙의 정확한 일치도 확인했다. 이는 당시 범위 검사 결과다. 이후 아래 gate2가 완료됐다.


## 두 번째 전체 게이트 — 기술 PASS

| 항목 | 확인된 결과 |
|---|---|
| 소스 | p12 `f38d488f43156145a44898671e77b26b3ee6c496`, 시작 dirty 0 |
| 종료 | 2026-10-25 09:07:08 KST, 프로세스 exit0·gate status0 |
| 단계 | **35/35 exit0**; perf/nav 보고 항목도 결과를 별도로 보존 |
| Rust workspace | debug **2060/0/1**, release **2060/0/1** (PASS/FAIL/ignored); ignored v6 fixture는 기존 별도 실행 증거와 구별 |
| 첫 실패 재검증 | sim-core 전체355 unit PASS(Chronicle 회귀 포함), 문서 목록2221경로·1944행·누락0 |
| 정적·실행 검사 | fmt, bridge build/import, clippy, forbidden math, strict, smoke, bridge API, docs, Python, M1 58검사, 런처6/6, schema, 렌더러 일치·저장 왕복 통과 |
| 동일성·결정성 | 228748값 차이0·편측0; Chronicle64파일/19683항목(git_sha 제외), raw sample64파일(기존 git_sha·제거된0값 키 규칙), baseline/colony 결정성·명령 재생 통과 |
| 장기 fixture | soak4×20일 원장·예약·사망0·KnowledgeLost0; F10 8/8; story/neighbours 각2×48일; steward2×24일 결정성·원장·예약 통과 |
| 아트 | manifest, resolver39재질/352수목 차이0, extensibility grep, renderer45검사 통과 |
| 성능 | 자연 장면3회 중앙평균 **0.253345ms**, 기준선0.253830ms 대비 +10% 초과 없음. quick 평균0.360378ms; 표준 bench 시간·할당·RSS 상한 통과. nav N1000 이동128µs/tick, 보고 예산600µs 이내 |

원본: `/Users/rexxa/work/mux-data/gate-stage-a-2/gate.txt`, 단계별 로그와 `receipt.json`. gate.txt SHA256은 `d38ff003671c2b15eefd21dd7d63d68d172dce7e597783d2fba987f0d0f9ed98`다. 단계35개와 종료 상태를 영수증에서 직접 확인했다. release 성능·24일 fixture 결과는 debug 지연·전체 UI 예산·A8의48일 held-out 수용을 대체하지 않는다.
