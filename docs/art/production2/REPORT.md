## 구현 완료 보고

**기술 게이트 25/25 PASS. A1·A5·A11은 FAIL이며 시각적 수용 완료가 아니다.** 최신 시뮬레이션 임시 호환성도 통과했다. 게시 커밋의 전체 재검증·원격 확인 영수증은 아래 경로와 PR #511 Art Pass 2 항목에 기록한다.

### 구현 의도

기존 생성물 대부분이 크기·색상 기준으로 배제되어 대체 그림에 의존하던 화면을, 측정한 픽셀 격자에 맞춘 축소와 독립 시각 검토로 회수하고 읽기 쉬운 주민·숲·HUD로 개선하려 했다.

### 구현 내용

| Part | 구현/주요 파일 |
|---|---|
| 0 | Pass 3 병합, 기준선·원본 42개 검증, 공유 CPU/창 lock |
| 1 | `pixel_normalize_v4.py`, `v4_contracts.py`: 격자 측정·맞춤 축소·가드 |
| 2 | `salvage_v4.py`, `salvage_review.py`, `salvage_verdicts.py`: 256슬롯 두 번 검토·해시 보존 |
| 3 | `fill_v4.py`, `merge_fill_selection.py`, `install_v4.py`, `pack_atlas.py`: 6시트 검토·물리ID 매핑·설치·억제 |
| 4 | `agent_activity.gd`, `agent_renderer.gd`, `art_sprite_library.gd`: 체형/방향/활동 선택·아이콘·노인 |
| 5 | `nature_layout.gd`, `nature_canopy.gd`, `nature_renderer.gd`, `art_season_tint.gd`, `terrain_renderer.gd`: 군집/빈터/반투명/계절 |
| 6 | HUD 두 패널·`death_viz_renderer.gd`·locale: 원인별 죽음·노환 합계·계절 날짜·9개 키 |
| 7 | windowed 94장+calibration 4장, QA/블라인드/비교판/GIF, 최신 green 임시 호환성 |
| 8 | 최종 기술 gate 25/25, 한국어 보고서·근거·범위/용량 감사. 게시 HEAD는 전체 gate 재실행 후 green으로 fast-forward하며 PR은 draft 유지 |

### 구현 방법

v4는 격자 주기·위상을 측정하여 성인28/아이21 native로 맞춘다. 그룹별 배율·동일 물리 attempt를 유지하고 색차>6 경고/>12 거절, 팔레트·alpha·기하·발·베리·바구니 가드를 적용한다. outline 전후 크기를 분리한다. 검토는 라벨/해시를 먼저 저장하고 두 번까지 수행했다.

빈칸은 동일 체형의 동서 반사→procedural→정면→idle+아이콘 순으로 처리한다. 파생/fallback은 accepted가 아니며 carry 밖 바구니와 D3 gatherer catchall을 금지한다. 억제 tombstone은 legacy 재등장을 막는다.

숲은 6타일 bilinear noise/smoothstep, gain0.65를 사용한다. 건물·공사장·화덕·정착점·베리 덤불의 Chebyshev4를 비운다. 보간 뒤 머리(발−20px)를 가리는 앞쪽 나무만 alpha0.45로 만든다. 계절 RGB는 지면/자연물에만 적용한다.

선택적 키가 없으면 기존 화면을 유지한다. 사망은 `(tick,x,y,reason)`으로 120tick 중복 제거하며 feed만료와 독립적이고 rollback/reset을 처리한다. 실제 bridge의 season0-based/day_of_season1-based를 반영해 날짜·첫날6시간 tint를 수정하고 전후 회귀를 확인했다.

### 되살린 결과

원본256+보충37=293개를 상호 배타적으로 집계했다. 아래 물리 fill 표와 중복 합산하지 않는다.

| 시트/추가 대상 | 원본 회수 | 파생 | 신규 생성 | fallback | procedural | 그리지 않음 | 합계 |
|---|---:|---:|---:|---:|---:|---:|---:|
| V_a1 | 2 | 0 | 1 | 0 | 13 | 0 | 16 |
| V_a2 | 1 | 1 | 0 | 8 | 6 | 0 | 16 |
| V_a3 | 4 | 0 | 5 | 2 | 5 | 0 | 16 |
| V_b1 | 10 | 2 | 4 | 0 | 0 | 0 | 16 |
| V_b2 | 4 | 0 | 0 | 8 | 4 | 0 | 16 |
| V_b3 | 14 | 0 | 0 | 0 | 2 | 0 | 16 |
| V_c1 | 4 | 0 | 0 | 0 | 12 | 0 | 16 |
| V_c2 | 6 | 0 | 0 | 0 | 10 | 0 | 16 |
| S1 | 16 | 0 | 0 | 0 | 0 | 0 | 16 |
| S2 | 13 | 0 | 0 | 0 | 0 | 3 | 16 |
| N1 | 15 | 0 | 0 | 0 | 0 | 1 | 16 |
| N2 | 14 | 0 | 0 | 0 | 0 | 2 | 16 |
| N3 | 10 | 0 | 0 | 0 | 0 | 6 | 16 |
| F1 | 8 | 0 | 0 | 0 | 0 | 8 | 16 |
| F2 | 9 | 0 | 0 | 0 | 0 | 7 | 16 |
| U1 | 11 | 0 | 0 | 5 | 0 | 0 | 16 |
| P2V_a_N_drink | 0 | 0 | 2 | 0 | 0 | 0 | 2 |
| P2V_b_S_drink | 0 | 0 | 1 | 0 | 1 | 0 | 2 |
| P2V_b_E_eat | 0 | 0 | 1 | 0 | 1 | 0 | 2 |
| P2V_b_N_eat | 0 | 0 | 1 | 0 | 1 | 0 | 2 |
| P2V_b_N_sit | 0 | 0 | 1 | 0 | 0 | 0 | 1 |
| P2V_c_S_carry | 0 | 0 | 0 | 4 | 0 | 0 | 4 |
| P2V_a_N_build | 0 | 0 | 0 | 0 | 2 | 0 | 2 |
| P2V_b_N_gather | 0 | 0 | 0 | 0 | 2 | 0 | 2 |
| P2V_b_N_build | 0 | 0 | 0 | 0 | 2 | 0 | 2 |
| P2V_b_N_drink | 0 | 0 | 0 | 0 | 2 | 0 | 2 |
| P2V_b_S_asleep | 0 | 0 | 0 | 0 | 1 | 0 | 1 |
| P2V_b_N_asleep | 0 | 0 | 0 | 0 | 1 | 0 | 1 |
| P2V_c_E_carry | 0 | 0 | 0 | 4 | 0 | 0 | 4 |
| P2V_c_S_gather | 0 | 0 | 0 | 0 | 2 | 0 | 2 |
| P2V_c_E_gather | 0 | 0 | 0 | 0 | 2 | 0 | 2 |
| P2V_c_S_build | 0 | 0 | 0 | 0 | 2 | 0 | 2 |
| P2V_c_E_build | 0 | 0 | 0 | 0 | 2 | 0 | 2 |
| P2V_c_E_drink | 0 | 0 | 0 | 0 | 2 | 0 | 2 |
| **합계** | **141** | **3** | **16** | **31** | **75** | **27** | **293** |

합계는 **원본 회수 141 + 파생 3 + 신규 생성 16 + fallback 31 + procedural 75 = drawable 266**, 억제 27을 더해 293이다. accepted는 원본 회수와 신규 생성만 합친 **157**이다. fallback은 기존 기능 아이콘 유지 5개와 idle_icon 26개다. atlas 총 282개에는 이 266개 외 보존한 D3 참조 16개가 포함된다.

원본 선택143 accepted+4 derived에서 accepted2개 교체·derived1개 제외 후 회수141+파생3이 남았다. 설치 계획과 manifest의 266개 출처/상태/ID가 일치한다.

| 신규 물리 시트 | 시각 OK | 시각 거절 | 가드+검토 승인 및 설치 |
|---|---:|---:|---:|
| P2_fill_01 | 14 | 2 | 4 |
| P2_fill_02 | 13 | 3 | 4 |
| P2_fill_03 | 0 | 16 | 0 |
| P2_fill_04 | 13 | 3 | 4 |
| P2_fill_05 | 7 | 9 | 4 |
| P2_fill_06 | 0 | 16 | 0 |
| 합계 | 47 | 49 | 16 |

원본 최종 시각 거절 41개: artefact 28, region_map_wrong 11, broken_anatomy 1, detail_destroyed_by_fit 1. 원본 미충족 109개는 이 41개와 coherent guard-passing 후보 없음 54개, pending_cycle_completion 14개다. 신규 최종 시각 거절 49개: region_map_wrong 34, artefact 14, wrong_pose 1. 시각 OK는 기술 승인과 동일하지 않다.

생성은 **6/18회** 사용했다. 체크포인트에서 신규 충족 **15/96=15.625%**로 50% 미만이므로 중단했고 남은 12회를 쓰지 않았다. 설치 신규 16개 중 `V_a3_09`는 이미 승인된 원본의 교체여서 체크포인트 신규 충족에 포함되지 않는다. 축척 탈출 조건은 해당 주민 0/128로 성립하지 않아 성인 28/아이 21을 유지했다.

근거: [final_selection.json](final_selection.json), [installation_selection.json](installation_selection.json), [install_plan.json](install_plan.json), [최종 원본 판정](review/round2/verdicts.json), [fill 체크포인트](fills/checkpoint6.json), [호출 원장](generation_calls.json). 신규 원장/판정은 `fills/P2_fill_XX_attempt_1.json`, `fills/review/P2_fill_XX/round2/verdicts.json`, `fills/adjudication_XX_round2.json`(XX=01…06)에 있다.

### 실제 게임 화면 검증

[ingame_qa.json](ingame_qa.json)의 측정값이다. art94/94장, 별도calibration4장. art 시작SHA `37fc6ca004e49df29293ed5ea29c2c1667a85b2e`/dirty15, calibration SHA `5433649aa123e0b1e84ec031efdf5a0b3123af52`/dirty1이다. 캡처 중 도구를 커밋했지만 렌더링 소스는 바뀌지 않았다는 DECISIONS 기록을 따른다. 최종 clean HEAD 캡처는 아니다. [capture_source_validity.json](capture_source_validity.json)은 art/calibration/compat 창 로그 모두 SCRIPT ERROR/ERROR 0을 확인한다. 이후 runtime 변경은 계절 소비자2개와 테스트2개뿐이며 pinned Pass3에는 해당 키가 없어 초기 캡처 동작은 유지된다.

| 검사 | 결과 | 측정 및 기준 |
|---|---|---|
| A1 | FAIL | 94장 중 calibration z4/z6 색상 61개로 64 미달; 표준편차 0.0645/0.0967 |
| A2 | PASS | 4×/6× 원본 checker 혼합 픽셀 0 |
| A3 | PASS | drawn rect 1,948개, 잘못된 크기 0; 성인 S-idle 4개 aspect 실패 0 |
| A4 | PASS | 4×/6× 앞뒤 겹침 4쌍 framebuffer 일치율 모두 1.0 |
| A5 | FAIL | median 0.1058 <0.15, min 0.0011 <0.10; 측정 불가 0; 반투명 canopy의 실제 blended pixels 포함 |
| A6 | PASS | 낮/밤 평균 0.293529/0.147316, 화덕 반경 평균 0.266445 |
| A7 | PASS | 이동 표본 37, 초별 distinct frames 4/4/8, 변화율 1.000; 3.967초, 간격 0.072–0.134초 |
| A8 | PASS | 미선택 장면 89개, clutter 실패 0 |
| A9 | PASS | UI 장면 90개, 실패 0 |
| A10 | PASS | live 300프레임 median 6.111ms, p95 15.230ms; 기준 ≤16.7/25ms |
| A11 | FAIL | 24개; 4× 16/24=66.67% <80%, 2× 13/24=54.17% <60%; 누락 활동 없음 |
| A12 | PASS | 주민 25명 외형 tuple 25개, 인접 동일 0/23 |
| A13 | PASS(기하) | 3체형×8활동 24그룹 모두 ≥15%; 최소 0.156584; 누락 0, 금지 활동 berry≥6 위반 0 |
| A14 | PASS | 전체 숲 1,190타일 중 나무 326=27.395%; 중복 0, clearing 중심 16, 검사 자연물 456, 침범 0 |

추가 RAMP 검사는 4×/6× 각 12개 외형 모두 framebuffer 일치율 1.0이다. **A13은 S idle과 다른 E/N idle의 방향 차이만으로도 통과할 수 있어 의미 있는 활동 자세나 식별 가능성의 증명이 아니다.** A11 실패를 A13으로 상쇄하지 않는다.

블라인드 라벨 SHA256: `adb25937a7bd7e432d8920b5e6aa056fd5594c782d5e217f48cb366d2b5b7ee0`. [라벨](blind/labels.json), [해시](blind/labels.sha256), [결과](blind_result.json), [4×](blind/blind_4x.png), [2×](blind/blind_2x.png). 라벨을 저장/해시한 뒤 별도 정답키로 채점했다.

**4x 혼동 행렬** — 행은 정답, 열은 관찰자 응답.

| 정답/응답 | walk | carry | gather | build | eat | drink | sit | talk | child |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| walk | 2 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| carry | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| gather | 0 | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| build | 2 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 |
| eat | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| drink | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 |
| sit | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 |
| talk | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 1 | 0 |
| child | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 1 | 1 |

**2x 혼동 행렬** — 행은 정답, 열은 관찰자 응답.

| 정답/응답 | walk | carry | gather | build | eat | drink | sit | talk | child |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| walk | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| carry | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| gather | 0 | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| build | 2 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 |
| eat | 0 | 0 | 0 | 0 | 1 | 2 | 0 | 0 | 0 |
| drink | 1 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| sit | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 |
| talk | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 1 | 0 |
| child | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | 2 |

GOAL §15 확인표: 자동/테스트 증거를 전 항목 시각 승인으로 바꾸지 않는다.

| 항목 | 현재 판단/증거 |
|---|---|
| 1. 방향별 모습·걸음 | A7 PASS. 전 체형/방향의 자연스러움까지 승인한 것은 아님 |
| 2. 식량 이동 때만 바구니 | 제한/A13 berry 검사 PASS. carry26프레임은 실제 자세 대신 idle+아이콘 |
| 3. 활동 구분·2× 가독성 | 미달. A11 4× 66.67%, 2× 54.17%; 건설은 양쪽 줌 모두 정답 0/3 |
| 4. 군집 숲·빈터·반투명 | A14 27.395%, 침범 0; 실제 scene-order 진입/이탈 canopy 회귀 테스트 PASS. A5 대비는 여전히 FAIL |
| 5. 나무/소품 종류 | N1 회수15/억제1, D3 재등장 차단. 모든 종류의 동시 화면 출현은 미확인 |
| 6. 아이 크기·두 성인 체형 | native 높이 21/28 및 체형 분리 구현. 블라인드 child 정답 4× 1/3, 2× 2/3이므로 식별은 불완전 |
| 7. 원인별 사망 소식 | 네 원인/중복/회색표식 테스트 PASS. 네 원인 자연발생 화면은 미확인 |
| 8. Pass 4 노인·노환·계절 | 최신green bridge/22검사/10캡처 통과, 봄2일째 확인. 노인·노환 자연발생 및 계절전환 전체는 미관측 |
| 9. 콘솔·프레임 시간 | 세 windowed 로그 SCRIPT ERROR/ERROR0. 현재6.111ms/p95 15.230ms, P1 약5.9ms 규모. 최종 smoke도 오류0 |

[before_after.png](before_after.png)는 Production 1의 동일 noon/night/overview framing과 비교한다. [village_noon.gif](village_noon.gif)는 40/40프레임·3,042,629바이트, [village_evening.gif](village_evening.gif)는 40/40프레임·3,621,033바이트로 각각 8MB 미만이다. 검토 시트는 `review/`, `review/round2/`, `fills/review/P2_fill_XX/` 및 각 `round2/`에 보존했다. 원본 framebuffer와 JSON은 `.harness/evidence/art-production2/`, 공개 sidecar는 manifest가 가리키는 lossless gzip이다.

### 시뮬레이션 작업과의 호환

Pass 3 `a8611e07aabbb4043f5fac2e8dd14b375a0ada5d`의 병합 결과 `a9763bf73df355006c375091b8442053e6de97e2`는 충돌 없이 기준선 gate를 통과했다. art 기준은 `f3f2cafc950811ba7e798eb9cbf6e007e5d35dc7`이다.

[compatibility/result.json](compatibility/result.json): 최신 green `7a8ae053059f3d3eed376a427eadec235259fecd`와 art `4908036fb1e4039616b5c7262d9dcc9f6724a105`의 임시 병합 `276d857242159855cea23a9608d70cf586e4166b`는 **충돌 0, bridge PASS, headless 22/22 PASS, windowed 10장**이다. 리더가 [낮](compatibility/shots/d2_1200_F2.jpg)과 [밤](compatibility/shots/d2_2200_F2N.jpg)의 “봄 2일째”를 확인했다. 임시 병합은 푸시하지 않았고 브랜치를 삭제한 뒤 WIP로 복귀했다.

`life_stages`는 존재하지만 0/1만 관측됐다. `deaths_old_age`는 존재하나 0이며 reason3 발생도 없었다. season0, 7일 중 1/2/6일을 확인했다. **실제 노인·노환 발생과 계절 전환 전체는 관측하지 못했다.** 네 계절/경계는 Godot 테스트 증거와 구분한다. 1-based 날짜 수정 `6a30aa33caf469bc98174391eb966e10e66e2b25`는 WIP 푸시 후 `ls-remote` 일치까지 확인했다.

### 변경된 파일 목록

[commit_files.json.gz](commit_files.json.gz)에 커밋별 정확한 파일 원장을 둔다. [scope_audit.json](scope_audit.json)은 금지 경로 변경 0, 기존 Production1 문서 변경 0, Rust 정적 assertion 변경 0을 확인했다. 따라서 이번 pass에 새 mutation 대상은 없다. 임시 병합의 시뮬레이션 변경을 art 작성 변경으로 집계하지 않는다.

원장은 Part 0–7의 모든 커밋과 파일을 포함한다. Part 8 게시 커밋은 `REPORT.md`, `TASKS.md`, `DECISIONS.md`, `gates/final/`, `commit_files.json.gz`, `scope_audit.json`, `capture_source_validity.json`, `report_artifacts.json`, `review/final_report_review.json`만 추가/갱신한다. 자기 자신 SHA를 문서에 넣는 순환을 피하기 위해 게시 SHA·두 원격 HEAD·PR 갱신·종료 UTC는 `.harness/evidence/art-production2/publication.json` 및 [PR #511](https://github.com/hyunlord/new-world/pull/511)에 기록한다. 게시 커밋의 전체 재검증 로그는 `.harness/evidence/art-production2/final-head-gate/`에 둔다.

### 자율 결정 목록

[DECISIONS.md](DECISIONS.md): 28/21 축척 유지, native16 U1/outline 구분, 두 프레임 N-carry 경고 보존, child 보충 슬롯 별도 집계, blue-crown 후보 설치 hold, 동일 attempt 그룹 교체, 검토 두 번 제한, 6회 뒤 15/96 생성 중단을 기록했다.

비승인 fallback과 일곱 carry leg_difference 진단을 보존했다. gain0.65·보간 뒤 canopy는 실제 seed/회귀 증거로 결정했다. CPU6·공유 lock을 준수하고 공개 sidecar를 lossless gzip으로 줄였다. A1/A5/A11 기준을 낮추지 않았으며 실제 bridge에 맞춰 계절 날짜를 1-based로 수정했다.

### 확인된 제한사항

- A1·A5·A11 미달: 대비와 건설/걷기/먹기/마시기 구분이 부족하다.
- procedural75/fallback31은 승인 원본이 아니다. carry idle_icon26과 일곱 leg_difference 그룹은 carry 애니메이션 승인이 아니다.
- A13은 E/N idle과 S idle의 방향 차이로도 통과하므로 의미 식별을 보장하지 않는다. 장식27개는 억제 상태다.
- canopy 평균635.067µs는 200주민/400나무의 headless CPU bucket/alpha 검사이며 GPU 제외다.
- assets 약1.81MB, 보고서·호환성·최종 로그를 포함한 docs 약39.93MB로 25MB/40MB 한도 이내다. 정확한 바이트는 [report_artifacts.json](report_artifacts.json)을 참조한다. JSON 압축은 무손실이고 원본 검토 그림/해시는 변경하지 않았다.
- 최신green 호환성은 통과했지만 실제 노인/노환/계절전환 관측은 부족하다. green은 기술 gate 통과를 뜻하며 시각 기준 통과를 뜻하지 않는다.

### Harness 결과

| 단계/검사 | 확인된 결과 | 근거/경계 |
|---|---|---|
| Part 0 전체 gate | 19/19 PASS; Rust 1,850 passed/0 failed, suite 117 | `gates/p0/summary.json`; 최종 구현 gate 아님 |
| Part 0 clippy/strict/bridge/smoke | 각각 exit 0 | `gates/p0/` 로그; smoke_errors exit 0 |
| Part 0 Python/Godot | Python 57, Godot scene 12 PASS | 기준선 기록 |
| Part 1 Python | 77/77 PASS | `gates/p1/python.log` |
| Part 2 Python | 97/97 PASS, derived metadata 30개 불일치 0 | `gates/p2/verification.json` |
| Part 3 Python/native/manifest | Python 126 PASS, native-size exit 0, PNG 612/612 등록 | `gates/p3/verification.json`, `manifest_audit.json` |
| Part 4 activity/elder/strict | exit 0; elder ID 1,000개 | `gates/p4/verification.json` |
| Part 5 Godot/strict | 장면 7개 PASS, strict exit 0, reload 동일 | `gates/p5/verification.json`; canopy 실패 재현 뒤 수정 PASS |
| Part 6 HUD/locale | 노환/계절 테스트 및 strict·locale PASS | `gates/p6/`; 1-based 수정 전 실패/후 통과는 `compatibility/result.json` |
| Part 7 QA | A1/A5/A11 FAIL, 나머지 A 검사 PASS | `ingame_qa.json`; 시각 gate 미통과 |
| 최종 cargo/clippy/strict/bridge/smoke/Python/Godot | **25/25 PASS**: Rust1850/0·117 suites, Python129, Godot18; Clippy all-targets/strict26파일·15FFI/bridge/smoke900프레임 오류0 | `gates/final/summary.json`, `results.tsv`; 실제 소스HEAD `6a30aa33caf469bc98174391eb966e10e66e2b25`, 게시 커밋은 전체 gate 재실행 |
| 최신 green 호환성 gate | bridge PASS, headless 22/22 PASS, 캡처10장 | `compatibility/result.json` |

### 정직한 판단

원본 회수와 숲·빈터·깊이·외형 변형은 개선했지만 **처음 보는 사람이 활동을 명확히 읽는 완성도 높은 픽셀 아트 게임이라고 판정하기에는 부족하다.** 대비와 블라인드 식별 실패가 남아 있다. drawable266이나 A13 PASS를 완성도의 대리 지표로 쓰지 않는다. 검토 원본과 실패 수치를 보존한 부분 개선이며 기술 gate 통과 여부와 시각 수용은 별개다.
