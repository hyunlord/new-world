## 구현 완료 보고

### 구현 의도

주민의 원래 피부·옷 색을 보존하고, 머리카락과 연령을 분리하며, 캠프의 겹침과 땔감·불 표시를 개선해 실제 시뮬레이션 결과를 화면에서 읽게 한다.

**작성 상태: Parts 0–6 구현·두 번째 창 촬영·측정을 완료했다. 최종 시각 FAIL은 A5·A11·A12·A18이며 A17은 오류 0으로 PASS다. Part 7 최종 HEAD gate·호환 확인·발행만 대기다. 시각 완전 승인을 선언하지 않는다.**

### 구현 내용

| Part | 결과와 근거 |
|---|---|
| 0 | 계약 동결, Pass 5 충돌 없는 병합, baseline gate 28/28. `GOAL.md`, `gates/p0/` |
| 1 | 자연색 보존 셰이더, 머리카락 전용 지도·검토·설치, 톤 3종. `PART1.md`, `natural_audit/` |
| 2 | 기존 206개 후보 시도 재검토, 6개 프레임 복구. `PART2.md`, `recovery/selection_run/adjudication.json` |
| 3 | 시뮬레이션 위치와 그리는 위치 분리, 화덕 자리·같은 칸 퍼짐·실제 그림 클릭. `PART3.md` |
| 4 | Pass 5 장작·화덕·가지 더미·끼니·HUD 연결, 새 소품 4개 설치. `PART4.md`, `wood/` |
| 5 | 첫 촬영 ART 94/94·ART_LONG 9/9·보정 4장 완료, 원본 SHA·sidecar 확인. `PART5.md`, `capture_verification.json` |
| 6 | round2 A1–A18, 독립 블라인드, 비교·계절·GIF·주민 시트 완료. A17 오류 88→0, A5·A11·A12·A18 실패 유지 |
| 7 | 대기: 최종 HEAD gate, 호환 확인, green FF, PR 설명과 remote 증명 |

### 구현 방법

원본 피부·옷 RGB는 유지하고 어두운 외곽선 외에 0.90/1.00/1.08 톤만 적용한다. 머리카락만 검토된 마스크로 재색칠하며 기존 ramp PNG는 보존했다. 3,000개 ID의 톤 분포는 1,037/1,015/948이다. 정확한 이미지·마스크 SHA, 알파와 영역 UV를 검증하고 기존 설치 경로의 기하·주기 검사를 통과한 후보만 설치했다.

캠프 자리는 14px 간격과 48px 배정 반경을 사용한다. 시뮬레이션 보간 위치 `ipos`는 유지하고 별도 `drawn`을 0.08초 지수 보간한다. 주민 그림·그림자·표시와 클릭 판정은 그려진 위치를 사용한다. Rust 수정은 허용된 정적 소스 검사에 한정했다. Part 3 변이 6/6개와 Part 4 화덕·AgentId·목표 라벨 변이 60/60개를 검출했다.

Pass 5의 타입이 맞는 행별 데이터가 있을 때 장작 활동·재고·불을 반영한다. 장작 줍기 1초, 화덕 근처 정지 상태에서 배고픔이 20 이상 감소한 뒤 끼니 3초를 표시한다. 없는 키의 기존 동작은 유지한다. 장작 주민 그림이 승인되지 않아 명시된 대체를 사용하며 실제 운반 애니메이션으로 보고하지 않는다.

### 되살린 결과

Part 2는 **206개 후보 시도 / 142개 물리 칸 / 93개 대상 칸**이다. 원본 native와 outlined 픽셀을 보존했다. 독립 시각 승인 130, 거절 76(머리 지도 45, artefact 27, 방향 4)이며 기술 제외 93건과 시각 제외 76건 뒤 37개 후보를 조합 검사했다. 실제 설치 6개는 승인 수와 구분한다.

| 체형 | 물리 칸 | 시도 | 시각 승인 | 거절 | 설치 |
|---|---:|---:|---:|---:|---:|
| adult_a | 18 | 20 | 15 | 5 | 2 |
| adult_b | 50 | 73 | 18 | 55 | 1 |
| child | 74 | 113 | 97 | 16 | 3 |
| 합계 | 142 | 206 | 130 | 76 | 6 |

다음은 `recovery/candidates.json`의 후보 ID와 `recovery/reviews/final_reviews.json` 및 설치 adjudication을 결합한 동작별 **시도 기준** 집계다. 여러 시도가 같은 대상 칸에 속하므로 고유 대상 승인 수로 읽으면 안 된다.

| 체형 | 동작 | 후보 시도 | 시각 승인 | 설치 |
|---|---|---:|---:|---:|
| adult_a | asleep | 2 | 2 | 0 |
| adult_a | build | 4 | 3 | 2 |
| adult_a | carry | 9 | 6 | 0 |
| adult_a | drink | 1 | 1 | 0 |
| adult_a | gather | 2 | 1 | 0 |
| adult_a | talk | 1 | 1 | 0 |
| adult_a | walk | 1 | 1 | 0 |
| adult_b | asleep | 6 | 5 | 1 |
| adult_b | build | 10 | 1 | 0 |
| adult_b | carry | 28 | 10 | 0 |
| adult_b | crossed | 3 | 0 | 0 |
| adult_b | drink | 4 | 0 | 0 |
| adult_b | eat | 2 | 0 | 0 |
| adult_b | gather | 8 | 0 | 0 |
| adult_b | walk | 12 | 2 | 0 |
| child | asleep | 6 | 6 | 0 |
| child | build | 8 | 7 | 0 |
| child | carry | 14 | 14 | 0 |
| child | drink | 6 | 5 | 0 |
| child | eat | 5 | 4 | 1 |
| child | gather | 5 | 2 | 0 |
| child | hop | 8 | 7 | 1 |
| child | hug | 3 | 2 | 1 |
| child | run | 12 | 10 | 0 |
| child | talk | 3 | 2 | 0 |
| child | walk | 43 | 38 | 0 |

설치: `V_a2_15`, `V_a2_16`(짓기), `V_b3_13`(잠), `V_c2_03`(뛰기), `V_c2_14`(먹기), `V_c2_16`(껴안기). 운반 복구는 0개다. 실제 carry 주기, 아동 걷기와 식별성의 완료를 주장하지 않는다.

| production 상태/대체 종류 | Art Pass 3 `ece4e85e` | Part 2 이후 | Part 4 현재 |
|---|---:|---:|---:|
| accepted | 167 | 148 | 152 |
| derived | 3 | 3 | 3 |
| procedural | 68 | 51 | 51 |
| fallback | 31 | 67 | 67 |
| retained_installed(위 fallback 포함) | 5 | 41 | 41 |
| idle_icon(위 fallback 포함) | 18 | 18 | 18 |
| borrowed_carry_view(위 fallback 포함) | 8 | 8 | 8 |
| production 전체 | 269 | 269 | 273 |

이 비교는 각 시점 `assets/art/atlas/world_atlas.json`에서 `direction_*` 16개를 제외한 값이다. 승인 감소는 Part 1의 새 자연색 검토에서 거절된 기존 픽셀을 미승인 대체로 재분류한 결과다. 추가 4개는 Part 4 소품이며 주민 복구가 아니다. 현재 atlas 총 289개에는 방향 참조 16개(accepted 7, provisional 9)가 포함된다.

Part 1 자연색·동작·머리 지도 검토: 165개 중 최종 **82 승인 / 83 거절**(wrong_pose 48, hair_mask_wrong 29, wrong_view 6). 1라운드 44 승인 후 한 번의 전역 분류기 수정으로 38개가 추가 승인됐다. registry는 `ok` 82 / `retained_rejected_fallback` 44 / `unreviewed_fallback` 39로 구분한다. 성인 B 44개 대상의 승인 대체가 없으며 마스크 존재를 시각 승인으로 계산하지 않는다.

새 이미지 생성은 **4/8회**, 64칸 중 독립 시각 승인 **24/64**(성인 A 0, 성인 B 0, 아동 11, W1 소품 13)다. 기술 통과 상한 20/64가 절반 미만이므로 추가 생성을 중단했다. 실제 설치는 소품 `W1_06/09/11/13` **4개**, 장작 주민 **0개**다. `image_calls.json`, `wood/reviews/`, `wood/selection/`에 출처와 판단을 보존했다.

### 실제 게임 화면 검증

최종 촬영은 runtime `9f8b49af7e67f5947c247c027d654604b7027188`에서 별도 window lock으로 ART 94장·ART_LONG 9장·보정 4장을 얻었다. `capture_verification.json`은 원본 hash **107개**, sidecar schema4 **99개**·schema3 **8개**, SCRIPT ERROR **0개**를 검증한다. schema3 8개는 보정/라인업이다. 최대 8tick 관측 변위 sidecar는 **일시정지 gameplay 정지화면 19개에만** 있고, 실시간 strip 80장은 시간순 이미지/메타데이터로 움직임을 판정한다. 모든 gameplay에 변위 sidecar가 있다고 주장하지 않는다. runtime은 커밋됐으며 촬영 당시 증거·검토 파일 변경 수(ART 20/LONG 49/보정 68)는 기록에 남아 있다.

아래는 **round2 최종 측정**이다. 근거는 `ingame_qa.json`, `measurement_summary.json`이며 첫 runtime `48a34af77bb10bfc5d315c5204212f183d830e3f`의 원본과 진단은 별도로 보존했다.

| 항목 | round2 최종 수치 / 판정 |
|---|---|
| A1 | PASS · gameplay 99/99, 보정 포함 옛 범위 103/103 |
| A2 | PASS · 4×/6× 혼합 픽셀 0 |
| A3 | PASS · draw rect 1,272개 오류 0, adult S-idle 4개 비율 오류 0 |
| A4 | PASS · 4개 방향/배율 조합 일치율 1.0; RAMP 24조합 1.0 |
| A5 | FAIL · d2/겨울 경계 중앙 .345408/.354405, ≥.10 비율 모두 100%; 측정 가능 13/17=76.471%·25/42=59.5238%<60% |
| A6 | PASS · 낮/밤 world .312281/.153805, lit 화덕 1개 평균 .265365, unlit glow 위반 0 |
| A7 | PASS · 이동 샘플 8, 완전한 이동 초의 서로 다른 frame 5, 변화율 .938 |
| A8 | PASS · 98장 오류 0 |
| A9 | PASS · 99장 오류 0 |
| A10 | PASS · 300프레임 중앙 7.547ms/p95 15.329ms; Art Pass 3 6.048/15.162ms |
| A11 | FAIL · 4× 19/24=.791667<.8, 2× 14/24=.583333<.6 |
| A12 | FAIL · 17명/11조합<12, 동일 이웃 0/7 |
| A13 | PASS · 체형 3×행동 8 기하 차이 24/24, 금지 berry ramp 0 |
| A14 | PASS · 숲 302/1,190=25.378%, 자연물 421개 빈터 위반 0 |
| A15 | PASS · 배치 638개/중심 18개 위반 0, native 장식 13종 통과 |
| A16 | PASS · d10/18/24/32 계절 일치, 겨울 B/R 1.167483배, 주민색 41/42 측정·일치 1.0, 노인 9명 |
| A17 | PASS · 오류 0; hearth 70/deadwood 586/HUD 99; 현재 이동 장작 3명=실프레임 0/대체 3 |
| A18 | FAIL · d2 정오 1/17≤1, 저녁 0/20≤1, d18 3/39>1, d24 6/42>2 |

| 범위 수정 | Art Pass 3 값 | 이번 round2의 옛 기준 진단 | 이번 round2의 새 기준 |
|---|---|---|---|
| A5 평균 명도 → 경계 대비 | d2 중앙 .0948/최소 .0029/불가 1; 겨울 .1093/.0031/불가 2 | A5-old FAIL: d2 .1091/.0004/0; 겨울 .1232/.0088/7 | FAIL: 중앙 .345408/.354405, ≥.10 비율 100%/100%, 측정 가능 13/17·25/42(59.5238%<60%) |
| A12 옛 look tuple → body/hair/tone | 23명/22조합, 같은 이웃 0/58 | 동결 P3 ArtHash 에뮬레이션 17명/16조합, 같은 이웃 0/7, PASS; 옛 런타임을 실제 관측한 값은 아님 | 17명/11조합<12, 같은 이웃 0/7, FAIL |
| A16 봄 d40 → d32 | d40 실제 여름 5일째; 겨울 B/R +16.767% | d40봄 요구 FAIL | d32봄 PASS; 겨울 B/R +16.7483%, d40 노인 9명; 일별 사망 합계 d10/18/24/32/40=0/0/4/10/14 |

A17 최초 90개 오류에서 88개로 정정한 것은 화면 직전 최대 8tick의 과거 변위와 촬영 순간 이동의 구분이다. ID 15·6은 과거 변위가 있지만 snapshot state 0으로 이미 정지했다. 현재 이동은 state 1과 관측 변위를 함께 요구하므로 장작 변위 5명 중 현재 이동 3명, 정지 2명이다. 렌더러의 idle 판정을 carry로 바꾸거나 대체 검사 기준을 완화하지 않았다. `ingame_qa_initial.json`의 90개와 `ingame_qa_initial_temporal_corrected.json`의 88개를 모두 보존했다.

A17의 실제 표시 문제는 서쪽 한 자리만 시도하던 장작더미 배치와 자원/저장소 라벨의 30프레임 지연이었다. `9f8b49af`에서 인접 합법 위치를 시도하고 시뮬레이션 tick 변경에 맞춰 갱신했다. 원본과 첫 진단은 `initial_capture/` 및 외부 첫 evidence root에 보존하고 round2는 별도 root로 촬영했다. 최종 round2는 장작더미 누락 69·저장소 라벨 지연 15·가지 단계 지연 4건을 모두 해소하여 **88→0, A17 PASS**다. 현재 이동 장작 3명 모두 선언된 대체를 사용하며 실프레임은 0이다.

A5는 첫 촬영에서 겨울 25/41=60.976%로 통과했으나 round2의 보이는 주민은 42명이며 25/42=59.5238%로 60% 미만이다. 경계 명도 자체는 충분해도 측정 가능한 주민 비율 조건을 만족하지 않아 **최종 FAIL**로 남긴다. 분모와 임계값을 바꾸지 않았다.

A18은 Art Pass 3의 6/23, 저녁 미측정, 9/41, 8/41과 비교한다. 첫 촬영 d18/d24 실패에는 반경 4 밖 주민, 이동 중 주민, 48px 이내 합법 좌석이 남지 않은 주민이 있다. 정확한 지정 격자·거리·물 타일 배제를 유지했다. `crowd_diagnosis.md`는 물 후보 두 곳의 실제 terrain code 4를 확인하며 명시된 범위를 넓혀 수치를 맞추지 않는다.

독립 블라인드 첫 결과는 4× 19/24, 2× 14/24다. 관찰 label SHA256은 `d699cb57129a9c4c48843947fd087a306a51b4ba7cbd96238c9b52aa04b0e372`; 정답 선동결과 독립 관찰 기록은 `blind/freeze.json`, `blind/answers_frozen.json`, `blind/observer_2x.json`, `blind/observer_4x.json`에 있다. 행=정답, 셀=관찰 라벨별 건수인 희소 혼동 행렬:

| 정답 | 4× 관찰 | 2× 관찰 |
|---|---|---|
| child | drink 1, child 1, walk 1 | eat 1, child 1, talk 1 |
| talk | talk 2 | walk 1, talk 1 |
| gather | gather 3 | gather 3 |
| build | build 3 | sit 2, build 1 |
| eat | eat 3 | eat 2, drink 1 |
| sit | sit 2 | child 1, sit 1 |
| carry | drink 1, carry 2 | drink 1, carry 2 |
| drink | drink 1, walk 1 | drink 1, talk 1 |
| walk | walk 2, drink 1 | walk 2, eat 1 |

| §12 번호 | 요구 | 증거와 현재 수용 범위 |
|---|---|---|
| 1 | 자연 피부·원래 가죽 옷·얼룩 없음 | 독립 정지화면 검토에서 따뜻한 피부/옷 분리 개선 확인, 보정 RAMP 일치. 모든 pose 승인 아님; 자연색·동작 감사 거절 83개 유지(머리 지도 사유 29개) |
| 2 | 노인만 회백색 머리, 동작 간 안정 | A16 노인 9명 규칙 통과, 겨울 41/42 색 측정 일치. 모든 동작의 깜빡임은 미검증 |
| 3 | 실제 운반·아동 걷기·망치/물 식별 | carry 새 설치 0, idle_icon 18/borrowed 8 유지. 블라인드 4×/2× 모두 FAIL. 완전 충족 아님 |
| 4 | 화덕 자리 겹침 해소와 클릭 일치 | 자리·클릭 헤드리스 검사 통과, 최종 A18 d18/d24 FAIL. 실제 창에서 mouse click은 미검증 |
| 5 | 아침·저녁 화덕 식사 | 저녁 40프레임 중 `a_evening_03/04`에서 agent 11의 eat pose 2회 관측. 서로 다른 두 명/두 끼가 아님. 정오 strip 40장 0회, 정지 08:30/12:00/19:30은 0회; 자연 발생 아침 식사는 미검증 |
| 6 | 낮 장작 줍기·어깨 운반·가지 감소/회복 | d10 현재 이동 장작 3명 모두 대체, 실프레임 0. 수집·고갈·재생의 전 과정 시각 수용은 미검증 |
| 7 | 시간·재고별 불/빛, 계절 장작 변화 | 정지화면의 낮 빈 화덕/밤 불·빛 및 A6 확인. A17 화덕·장작 단계 불일치 0. 독립 검토에서도 더미 노출 확인; 정확한 소진 시점 연속 관측 미검증 |
| 8 | HUD·저장소·정보창 장작 | 정지화면 HUD·저장소 양쪽 표시 확인. 저장소 갱신 지연 수정 후 A17 라벨·HUD 합계 오류 0. 정보창은 헤드리스 검사만 완료 |
| 9 | d32 봄, 주민색 계절 불변 | A16 계절 4장 일치, 겨울 주민 41/42 일치 1.0; 나머지 1명 미측정. 스트립은 d32 봄 |
| 10 | SCRIPT ERROR 없음, 비슷한 프레임 시간 | round2 SCRIPT ERROR 0, 300프레임 7.547/15.329ms(이전 대비 +1.499/+0.167ms). 통제된 동등 성능 실험은 아님; 최종 HEAD gate만 대기 |

최종 round2 산출물은 다음과 같다. 두 GIF 모두 원본 40프레임/인코딩 40프레임이며 각 8MB 이하이다. 경로는 `docs/art/production4/` 기준이고 `report_artifacts.json`·`presentation_artifacts.json`에 원본과 SHA를 기록했다.

| 산출물 | bytes | SHA256 |
|---|---:|---|
| [before_after.png](before_after.png) | 330,897 | `4af8763b787c3d1c26bb012617855c2924f84ac6f95aabb0fb81bed6d9c7ac2b` |
| [colour_before_after.png](colour_before_after.png) | 39,424 | `119e9efbac8803fb630f194554a349eacbfee0d8e136c993447526767ac8ed06` |
| [season_strip.png](season_strip.png) | 143,141 | `3fae1a38314e97db3fbf0f6cae3b80031dd7a241edea433ba97d928a38485498` |
| [village_noon.gif](village_noon.gif) | 2,103,090 | `5382c914678c2ce51d9f254e02b31b354a787f45f38b65fdfcfbceeef13466d8` |
| [village_evening.gif](village_evening.gif) | 2,897,462 | `c897270aea92e2d9e16d8092a5c2fa3f25f1c4fe13feb502ea040ea25e2d2efc` |
| [villagers_natural.png](villagers_natural.png) | 223,127 | `4622ae537a2be83a0f59bbd5573c57aae59d79d3e6f93bd6dfc8bdddd9e2eaf2` |

전후 비교 camera/zoom 3쌍 일치를 확인했다. 주민 자체 위치가 달라 겹침의 통제 실험은 아니다. 색 비교는 같은 P4 framebuffer에 P3 ramp를 에뮬레이션했으며 실제 P3 캡처가 아니다. 16명 exact mapping, ID24 한 명 미매핑은 원래 픽셀을 유지했다. 검토 보드: `natural_audit/`, `recovery/boards/`, `wood/boards/`. 최종 독립 정지화면 검토 `manual_visual_review.md`는 자연색·실제 장작더미 노출 개선과 d18/d24 겹침·분리된 바구니 한계를 함께 기록한다. 정지화면 리뷰 자체는 모션·클릭·성능의 증거가 아니다.

### 시뮬레이션 작업과의 호환

Pass 5 `682b442617d3f4bb9b151c29d324b17dfe82da93`를 `ddfb856156a3d827a1f8098010c3c762e4b77f78`로 충돌 없이 병합했다. 그 baseline은 전체 gate 28/28을 통과했다. Pass 5 장작·불·재고·배고픔은 UI에서 타입과 행을 확인하여 읽고 시뮬레이션 규칙은 변경하지 않는다. 장기 화면의 Pass 5 화덕·장작·가지 단계와 UI 재고를 비교했고 round2 A17 오류 0을 확인했다. 저녁 agent11 eat pose는 2프레임 관측했으나 자연 아침 식사는 미검증이다. 이번 실행 중 **20:30 UTC fetch**에서 green은 같은 `682b442617d3f4bb9b151c29d324b17dfe82da93`였다. simulation WIP `0a40b57c`는 의도적으로 병합하지 않았다. 최종 fetch·필요시 로컬 미발행 호환 병합은 **대기**다.

### 변경된 파일 목록

아래는 커밋별 핵심 파일이다. 전체 목록은 해당 커밋의 Git diff가 원본이며 이미지·지도·검토 증거는 각 Part 디렉터리에 함께 들어 있다.

| Part | 커밋 | 핵심 변경 파일 |
|---|---|---|
| 0 | `6bbca38c87b168904e97c16e9fa69a402a3e5c0b` | `docs/art/production4/` 계약·검증 기록 |
| 0 | `ddfb856156a3d827a1f8098010c3c762e4b77f78` | 시뮬레이션 Pass 5 병합(병합 커밋) |
| 0 | `8d69d05fe21cc0709843504f08b75592abc15f43` | `docs/art/production4/` 계약·검증 기록 |
| 1 | `abff72f339166e7444b1eaa7deaa33d65067c221` | `scripts/test/art/calibration_scene.gd`, `scripts/test/art/test_agent_activity.gd`, `scripts/test/art/test_art_hash.gd`, `scripts/test/art/test_elder_variant.gd`, `scripts/test/art/test_variant_codes.gd`, `scripts/ui/agent_renderer.gd`, `scripts/ui/art_hash.gd`, `scripts/ui/art_sprite_library.gd`, `shaders/villager_ramp.gdshader`, `tools/art_pipeline/apply_natural_audit_p4.py`, `tools/art_pipeline/hair_inventory_p4.py`, `tools/art_pipeline/hair_maps.py` |
| 2 | `a327260b4979f717ed0bcb29e5a18e3e3549ffd7` | `tools/art_pipeline/hair_freeze_p4.py`, `tools/art_pipeline/merge_p4_selection.py`, `tools/art_pipeline/natural_boards_p4.py`, `tools/art_pipeline/recovery_candidates_p4.py`, `tools/art_pipeline/reviewed_hair_maps_p4.json`, `tools/art_pipeline/test_hair_freeze_p4.py`, `tools/art_pipeline/test_merge_p4_selection.py`, `tools/art_pipeline/test_recovery_candidates_p4.py` |
| 3 | `97bf6497175c1e61810925faf1756430bb86353a` | `rust/crates/sim-test/tests/harness_p11_alpha_agent_renderer.rs`, `rust/crates/sim-test/tests/harness_p8_delta_memory_ui.rs`, `scripts/test/art/test_agent_click_drawn.gd`, `scripts/test/art/test_agent_drawn_positions.gd`, `scripts/test/art/test_camp_seats.gd`, `scripts/test/art/test_canopy_fade.gd`, `scripts/test/art/test_ysort_props.gd`, `scripts/ui/activity_trail_renderer.gd`, `scripts/ui/agent_renderer.gd`, `scripts/ui/camp_seats.gd`, `scripts/ui/seek_viz_renderer.gd`, `scripts/ui/world_renderer.gd` |
| 4 | `0e3481c73e09a9d7f9228a8bd45721c3eeb87cf9` | `scripts/ui/agent_pose_moments.gd`, `wood_visuals.gd`, agent/nature/building/daylight/HUD/inspector/seek 렌더링, 한·영 localization, wood 후보·선택 도구와 atlas |
| 5 | `48a34af77bb10bfc5d315c5204212f183d830e3f` | `scripts/test/capture_framings.gd` 등 Pass 5 메타데이터·관측 이동 캡처 |
| 5 | `cdbd840123655abd577778290d8a19f38f90115d` | `PART5.md`, 첫 촬영/sidecar/hash 증거 |
| 6 | `9f8b49af7e67f5947c247c027d654604b7027188` | woodpile 인접 위치, tick 동기 자원/라벨 갱신, QA 시간 범위·불일치 회귀 검사 |
| 6 | `4739738ebbf8160e4456add220d31cdb06562005` | round2 QA·비교 이미지·GIF·블라인드 재사용 증명 |
| 7 | 대기 | 최종 gate·보고·발행 증거 |

WIP `codex/art-production-2026-10-wip` 최종 HEAD 및 `git ls-remote` 동등성: **대기**. green `codex/art-production-2026-10` 최종 HEAD 및 원격 동등성: **대기**. PR [#511](https://github.com/hyunlord/new-world/pull/511) Art Pass 4 설명 업데이트: **대기**; draft 유지 확인: **대기**.

### 자율 결정 목록

`DECISIONS.md`의 결정에 따라 승인과 설치를 분리하고, 실패를 감추지 않는 부분 결과를 보존했다. 명시된 작업 경로·브랜치와 검증된 설치 경로를 유지했다. Part 2의 실패에 정당한 전역 분류기 변경 근거가 없어 두 번째 검토를 억지로 만들지 않았다. 방향 참조 16개를 production 수에서 분리해 잘못된 276개 설명을 269개로 수정했다. strict가 일시적으로 warning 정책을 바꾸므로 Godot 테스트와 순차 실행했다. 가지 더미 빈터 확장 후 기존 숲 밀도 기준을 완화하지 않고 기존 grove 순위의 결정적 보충으로 302/1,190=25.378%를 확보했다. 새 이미지 4회 후 기술 통과 상한이 절반 미만이라 생성을 중단했다.

### 확인된 제한사항

자연색·동작 감사 최종 거절 83개(머리 지도 사유 29개), 승인 대체가 없는 성인 B 44개, 실제 carry 복구 0개, 새 장작 주민 0개를 남겼다. 승인 130개를 설치 130개로 보고하지 않는다. 최종 A5·A11·A12·A18 실패를 남긴다. A17은 수정 후 PASS지만 새 장작 주민 그림 승인을 의미하지 않는다. 실제 창 클릭, 자연 발생 아침 식사, 장작 수집부터 재생까지의 연속 시각 수용은 미검증이다. round2 산출물 예산 기록은 assets 1,851,001bytes/docs 29,942,328bytes로 한도 내지만 최종 25MB/40MB 계측은 **대기**다.

### Harness 결과

| 범위 | 확인된 수치 | 한계 |
|---|---|---|
| Part 0 전체 | 28/28; Rust 1,912개/123 suites; Python 177; Godot 21; clippy/strict/bridge/smoke 통과 | baseline 증거이며 최종 HEAD 아님 |
| Part 1 | Python 207, Godot 21/21, strict·FFI 통과 | 창 시각 미검증 |
| Part 2 | Python 221; PNG manifest 612/612; import·ACTIVITY 통과 | 후보 시각 승인과 런타임 검증 구분 |
| Part 3 | Godot 24/24; 관련 Rust 85/85; strict 27파일·15FFI; 변이 6/6 검출 | 군중 A18은 미측정 |
| Part 4/5 촬영 전 전체 | Rust 1,912; Python 239; Godot 27; clippy/strict/bridge/smoke 통과; PNG manifest 620/620 | runtime 캡처 전 gate, 최종 HEAD 아님 |
| 최종 HEAD | **대기** | `gates/final/` 생성·판독 후 Rust/clippy/strict/bridge/smoke/Python/Godot 수치 갱신 |

### 정직한 판단

자연색·그리는 위치·장작 UI의 구현과 일부 프레임 복구는 근거가 있다. 그러나 장작 운반 주민 그림은 한 장도 설치되지 않았고 기존 운반 대체와 머리 지도 거절이 남는다. 독립 정지화면 검토는 피부·옷과 낮/밤 화덕의 개선을 확인했지만 겨울 캠프 겹침과 몸에서 분리된 바구니를 지적했다. 블라인드 식별도 두 배율 모두 기준 미달이다. 완성도 있는 픽셀 아트 게임으로 수용됐다고 판단하지 않는다. round2에서 A17 표시 오류는 해소했으나 겨울 경계 대비 측정률, 동작 식별, 자연색 조합 수, 군중 겹침의 네 실패는 남는다. 최종 코드 gate와 시각 수용을 구분한다.
