## 구현 완료 보고

작성 기준: Part 0–6 및 Part 7 캡처 코드 `f3418ef7`. **중간 보고 초안**이며 실제 캡처·측정·최종 게이트·브랜치 승격·PR 갱신은 PENDING이다. 아래 PASS는 적힌 범위의 증거만 뜻한다. 경로는 별도 표시가 없으면 저장소 루트 기준이다.

### 구현 의도

승인된 그림이 기술 가드와 혼합 세트 탐색 때문에 버려지고 머리 마스크·겹침·겨울 표시가 화면의 읽기 쉬움을 해치던 문제를, 승인 이력을 보존한 회수와 위치 기반 마스크, 외형 다양화, 렌더링 전용 자리 조정·추위 표시로 개선한다.

### 구현 내용

| Part | 구현과 현재 결과 | 핵심 증거 |
|---|---|---|
| 0 | GOAL 원문 보존, 정확한 Pass 6 병합, 기준 전체 게이트 PASS | `docs/art/production5/BASELINE.md`, `gates/p0/results.tsv` |
| 1 | 물리 세트별 회수·동일 위상 복사·독립 애니메이션 리뷰·표준 설치. 주민 10그룹 38프레임, W1 13개 선택 | `docs/art/production5/harvest/SUMMARY.md`, `harvest/adjudication.json` |
| 2 | 성인 일반 머리 5색/노인 2색, 피부 4톤, 28가지 셰이더 인코딩 및 과거 정의 병행 측정 | `scripts/ui/art_hash.gd`, `shaders/villager_ramp.gdshader`, `assets/art/villager_ramps.png`, `docs/art/production5/variety/` |
| 3 | 위치 기반 마스크 105후보를 두 회차 안에서 처리. 장작 8프레임 추가, 현재 주민 마스크 15개 적용 | `docs/art/production5/P3SUMMARY.md`, `masks/mask_installation.json` |
| 4 | 생성 2회, 첫 키 시트 체크포인트 STOP, 새 그림 설치 0. 승인-픽셀-애니메이션 바인딩 강화 | `docs/art/production5/keys/SUMMARY.md`, `image_calls.json`, `harvest/HARDENING.md` |
| 5 | 화덕 자리 격자 확대, 지속적 자리 배정, 정지 주민의 수평 겹침 완화, 그려진 위치 선택 | `scripts/ui/camp_seats.gd`, `scripts/ui/agent_renderer.gd`, `docs/art/production5/spacing/README.md` |
| 6 | 선택적 `colds`/`wood_targets`, 팔짱·떨림·입김, 분리한 장작 라벨, HUD·정보창 한국어 | `scripts/ui/cold_effects.gd`, `scripts/ui/building_renderer.gd`, `docs/art/production5/cold/IMPLEMENTATION.md` |
| 7 | 추운 밤 일정과 자리·겹침·마스크 출처·추위 메타데이터, 제어된 추위 장면 준비 | `scripts/test/art/capture_plans.gd`, `capture_metadata.gd`, `capture_cold_p5.gd`; 실제 캡처 PENDING |
| 8 | 기존 임계값을 보존하는 A1–A20 집계·비교 이미지 생성 코드 준비 | `tools/art_pipeline/measure_p5.py`, `presentation_p5.py`; 측정 PENDING |
| 9 | 최종 게이트·호환 병합·예산 확인·green 승격·PR #511 갱신 | PENDING |

표 안에서 `gates/`, `harvest/`, `masks/`처럼 줄인 증거 경로는 `docs/art/production5/` 아래다.

### 구현 방법

**가드와 회수.** 기존 후보 원장은 수정하지 않고 fingerprint가 기존 리뷰 인덱스와 일치하는 승인만 새 설치 원장으로 옮겼다. despeckle 3% 초과는 경고, 10% 초과는 오류로 구분한다. 장작 너비는 성인 30/아이 26px, W1 deadwood 24×18/woodpile 26×20을 적용한다. 굽힌 `gather_wood` frame 0의 높이 예외, 머리 마스크 첫 행 기준의 높이, 발 아래 3행 중앙 기준 앵커를 전달한다. 팔레트·이진 알파·색차·발 기준선·머리 높이·다리 차이 등 나머지 기준은 보존한다.

`harvest_p5.py`는 (시트, 시도)별 C0 완전 세트, C1 같은 위상 1개 복사, 운반 전용 C2 두 포즈 반복을 만들고 복사 수·색차·시도 순으로 탐색한다. 계산 가능한 가드 실패는 리뷰 strike를 소모하지 않는다. `merge_p5_selection.py`에서 독립 애니메이션 승인을 확인한 후 표준 installer/atlas/regions 경로로 설치한다. 복사는 승인된 같은 세트의 같은 parity 픽셀에만 허용한다. 성인 운반의 같은 위상 복사도 실제 운반 재생에 포함하며, 실제 운반 프레임이 없는 체형의 fallback을 무조건 실제 그림으로 분류하지 않는다.

**마스크와 키 아트.** 승인 참조를 RGBA·마스크·리뷰 해시로 동결하고 상단 머리 영역의 위치 정렬/IoU, 제한된 참조 색과 연결 성분으로 마스크를 옮겼다. 2차에는 아래쪽 탐색 영역만 확장했다. 새 생성의 예약 키 색은 별도 검출·오류 규칙으로 처리했으며, 첫 시트 key-clean 기준 미달에서 추가 키 생성을 중단했다. 정규화한 후보, 순서가 있는 애니메이션 후보, 리뷰 보드/GIF, 최종 native/sprite/region 픽셀을 서로 결속한다. Part 1/3의 사후 바인딩 복구 기록은 사후 증거라고 명시하고 과거 시점의 동결인 것처럼 제시하지 않는다.

**외형·자리·추위.** 일반 성인 hair 0–4, 노인 5–6, tone 0–3을 셰이더와 QA가 같은 규칙으로 사용한다. 화덕 자리는 16px 열/18px 행, 104×72 타원, 최소 간격 16px, 6타일 자격/64px 도달 범위다. 자리·기존 spread 뒤 정지 주민만 최대 좌우 8px로 조정하고 물·장애물·이동/착석 제외를 검사한다. 시뮬레이션 위치는 변경하지 않으며 0.08초 smoothing과 실제 그려진 위치의 클릭 판정을 유지한다.

추위 pose는 현재 방향 crossed → S crossed → 현재 방향 sit → idle 순이며 원본 RGBA가 그대로인 `wrong_pose`/`wrong_view` 거절은 제외한다. 식사·수면·대화는 보존한다. 떨림은 smoothing 뒤 적용해 누적 드리프트를 피한다. `cold_effects.gd`가 런타임 3×2 흰 입김과 2.4초 주기를 담당한다. 저장소 food/wood Label을 분리해 장작만 목표 부족 tint를 받고 settlement target은 합산하지 않는다. 새 번역 2키는 strict duplicate 검사와 컴파일/registry 갱신을 거쳤다.

### 회수 결과

**Part 1.** 10개 주민 그룹, 38프레임 = 실제 31 + 복사 7. W1 13개는 별도 물체이며 주민 프레임에 합산하지 않는다.

| 체형 | 동작 | 방향 | 실제 | 복사 | 완료 형태 / 출처 |
|---|---|---|---:|---:|---|
| adult_a | carry | S | 3 | 1 | C1 / P3_fill_01 attempt 1 |
| adult_a | carry | E | 2 | 2 | C2 / P3_fill_01 attempt 1 |
| adult_b | carry | S | 3 | 1 | C1 / P2_fill_02 attempt 1 |
| adult_b | carry | E | 3 | 1 | C1 / P2_fill_02 attempt 1 |
| child | carry | E | 2 | 2 | C2 / P2_fill_06 attempt 1 |
| child | walk | S | 4 | 0 | C0 / P2_fill_03 attempt 1 |
| child | walk | E | 4 | 0 | C0 / P2_fill_03 attempt 1 |
| child | walk | N | 4 | 0 | C0 / P2_fill_03 attempt 1 |
| child | build | S | 2 | 0 | C0 / P3_fill_03 attempt 1 |
| child | run | E | 4 | 0 | C0 / V_c2 attempt 3 |

계획자가 예상한 아이 걷기 12, 운반 20(실제 13), 아이 S build 2/E run 4를 모두 회수했다. W1 승인 13개 중 과거 억제된 9개가 drawable이 되었고 기존 4개는 픽셀 변화 없이 다시 바인딩했다. 추가로 검토한 아이 N drink는 필수 두 번째 프레임 거절 때문에 리뷰 전에 제외했다. child run은 자산 회수이며 실제 게임에서 run이 보인다는 주장은 아니다.

| drawable 상태/대체 유형 | Art Pass 4 | Part 1 후 | Part 3 후 현재 |
|---|---:|---:|---:|
| accepted | 152 | 192 | 211 |
| derived | 3 | 10 | 13 |
| procedural | 51 | 33 | 33 |
| fallback | 67 | 47 | 33 |
| drawable 합계 | 273 | 282 | 290 |
| fallback: retained_installed | 41 | 41 | 27 |
| fallback: idle_icon | 18 | 6 | 6 |
| fallback: borrowed_carry_view | 8 | 0 | 0 |

전후 원장은 `harvest/adjudication.json`이며 Part 3 최종 상태는 `P3SUMMARY.md`와 `masks/mask_installation.json`에 기록되어 있다. 설치 이력의 프레임 수와 현재 drawable 수는 서로 다른 집계다.

**Part 3.** 105레코드/103고유 RGBA, 승인 참조 73개를 사용했다. 첫 처리에서 새 마스크는 a44/b27/c29이며 IoU 부족 5개는 기존 마스크를 유지했다.

| 범위 | 재처리 | 최종 승인 | 거절 |
|---|---:|---:|---:|
| (a) 회수 후보 | 45 | 23 | 22 |
| (b) 장작 후보 | 31 | 13 | 18 |
| (c) 설치 adult_b 감사 대상 | 29 | 24 | 5 |
| 합계 | 105 | 60 | 45 |

1차 승인 54/거절 51. 허용된 2차 변경은 T+를 불투명 높이 20%(최대 6px)만큼 아래로 늘리는 것이다. 전체 적용 시 69개 마스크가 달라졌고, 이전 거절 51개만 다시 리뷰하여 6개 추가 승인했다. 첫 승인 54개는 첫 마스크를 그대로 보존했다. 후보·판정은 `masks/final_candidates.json`, `masks/final_review_index.json`, `masks/final_reviews.json`이다.

설치는 승인 수보다 작다. a/b에서 adult_a N carry_wood 실제 2+복사 2, adult_b N carry_wood 실제 3+복사 1, 합계 2그룹 8프레임을 추가했다. c 승인 24개가 속한 13그룹을 다시 리뷰해 9그룹 승인/4그룹 `cycle_pop` 거절을 받았다. 현재 RGBA·그룹 승인을 통과한 마스크 **15개 적용**, 감사 당시 동일 RGBA인 **14개 상태 복구**다. 미설치 9개는 애니메이션 거절 5개와 Part 1에서 이미 RGBA가 바뀐 E carry 4개다. `V_b3_12`는 새 마스크를 적용하되 감사 이후 RGBA이므로 상태 복구에 포함하지 않는다. 원본 비교는 `masks/audit_binding.json`을 따른다.

**Part 4.** 최대 8회 중 **2회 사용**, 설치 **0그룹/0프레임**. 두 원본은 1254×1254이며 git 밖 `/Users/rexxa/work/art-production5-raw/`에 있다.

| 시트/시도 | 시각 승인 | key-clean | 최종 결과 |
|---|---:|---:|---|
| V_a6 / 1 | 12/16 | 8/16 | 12/16 기준 미달, 추가 키 생성 STOP |
| V_c4 / 1 | 10/16 | 해당 없음 | 허용된 아이 시트; 완전한 승인 세트 없음 |

어느 시트도 절반 미만 승인 재시도 조건에 해당하지 않아 재생성하지 않았다. V_a6의 낡은 초기 `conversion_failure`는 원장을 덮지 않고 `candidates_reguarded.json`에 별도로 바로잡았다. 112개 이미지 파일 해시는 불변이며 STOP은 바뀌지 않는다. authoritative search는 `keys/harvest_validated/search.json`: 10개 물리 그룹 시험, 설치 가능 0. frame guard 통과는 V_a6 2개/V_c4 6개지만 승인된 완전한 애니메이션을 이루지 못했다.

누적 A20은 `harvest/integrity_final.json`에서 **PASS, 설치 기록 2개, bound 74프레임, 변경 주민 61프레임, 애니메이션 20그룹, 오류 0**이다. 이 수에는 마스크만 바뀐 주민과 반복 바인딩한 W1 기록이 포함되므로 74를 신규 그림 수로 읽으면 안 된다.

### 실제 게임 화면 검증

실제 windowed 캡처를 아직 집계하지 않았다. 아래 P4 수치는 비교 기준일 뿐 현재 PASS로 승계하지 않는다. PENDING은 0회 관측 또는 실패를 의미하는 숫자가 아니라 미측정 상태다.

| 검사 | Art Pass 4 기록 | Art Pass 5 현재 증거/판정 |
|---|---|---|
| A1 | PASS 99/99 | PENDING: 새 화면 색 검사 |
| A2 | PASS 혼합 픽셀 0 | PENDING: 4×/6× 픽셀 |
| A3 | PASS draw rect 1,272개 오류 0 | PENDING: 현재 기하·크기 |
| A4 | PASS 방향/배율 4조합 및 RAMP 24조합 1.0 | PENDING: 확장 28조합 실제 셰이더 픽셀; 인코딩 테스트만 PASS |
| A5 | FAIL 겨울 측정 가능 25/42=59.5238% | PENDING: 경계 대비·측정 가능 비율, 기준 유지 |
| A6 | PASS 낮/밤 .312281/.153805 | PENDING: 밤·불빛 화면 |
| A7 | 당시 PASS, 이동 샘플 8 | PENDING: 새 최소 30 이동 샘플, 필요시 정오 GIF 추가 |
| A8 | PASS 98장 오류 0 | PENDING |
| A9 | PASS 99장 오류 0 | PENDING |
| A10 | PASS 300프레임 중앙 7.547ms/p95 15.329ms | PENDING: 새 300프레임/콘솔 검사 |
| A11 | FAIL 4× 19/24, 2× 14/24 | PENDING: 정답 선동결 후 독립 블라인드/혼동 행렬 |
| A12 | FAIL 성인 17명/11조합, 같은 이웃 0/7 | PENDING: 새 5 hair×4 tone와 이전 3×3 정의 모두 산출 |
| A13 | PASS 체형×행동 24/24 | PENDING: 현재 자연색 시트 |
| A14 | PASS 숲 302/1,190=25.378%, 빈터 위반 0 | PENDING |
| A15 | PASS 배치 638/중심 18 위반 0 | PENDING |
| A16 | PASS 계절 d10/18/24/32, 겨울 B/R 1.167483, 주민 41/42 | PENDING: 일반 0–4/노인 5–6 및 옛 분류 병행 |
| A17 | PASS이나 이동 장작 실프레임 0/3 | PENDING: W1 단계별 art 사용 및 실제 carry_wood 가시 인원 별도 보고 |
| A18 | FAIL d18 3/39, d24 6/42 과도 겹침 | PENDING: 41자리·declump headless PASS는 화면 대체 증거가 아님 |
| A19 | 새 항목 | (a) 추위/목표/정보창 테스트 PASS; (b) 자연 추위·2.4초 입김 관측 PENDING; (c) 가시 장작/목표 라벨 PENDING |
| A20 | 새 항목 | PASS: 74 bound/61 변경 주민/20그룹/오류 0; 최종 설치 상태 변경 시 재실행 필요 |

범위 수정 전후: A7의 P4 표본은 8개이며 이번 최소는 30개다. A12는 tuple 형태와 임계값을 보존하고 adult hair 0–2/tone 0–2에서 hair 0–4/tone 0–3으로 설계를 늘렸다. A16의 옛 노인 hair 3/4·일반 0–2와 새 노인 5/6·일반 0–4 판정은 각각 별도 결과를 남길 예정이다. 새 정의의 PASS로 옛 정의 결과를 지우지 않는다. 블라인드 혼동 행렬은 `docs/art/production5/blind/result.json` 생성·검증 전이므로 PENDING이며 승인 리뷰 점수를 A11로 대신하지 않는다.

GOAL 14항의 직접 플레이 확인표:

| 항목 | 확인 가능한 구현·테스트 | 실제 플레이 수락 |
|---|---|---|
| 1 아이 실제 걸음/발만 교체되지 않음 | walk S/E/N 실제 12프레임, 3그룹 애니메이션 승인 | PENDING: A7/A11 이동 관측 |
| 2 실제 바구니 운반/아이콘 제거/두 포즈 연속성 | carry 실제 13+복사 7, C1/C2 재생 테스트 | PENDING: 세 체형 가시 운반 및 아이콘 |
| 3 가지 단계·소진 흔적·장작 그림/대체 | W1 승인 13개, 성인 N carry_wood 8프레임 | PENDING: A17 단계·실제 운반 인원 |
| 4 장작 머리색 번짐·앵커 흔들림 | 승인 마스크·foot anchor·애니메이션 리뷰 | PENDING: 가시 장작 운반 연속 화면 |
| 5 흰머리 전체·팔/도구/외곽선 보존 | c 마스크 15개 설치, 4그룹 cycle_pop 미해결 | PENDING: 모든 노인이 해결됐다고 주장하지 않음 |
| 6 일반 머리 5색·피부 4톤·노인 전용 2색 | 3,000 ID 분포, 28 인코딩 조합 테스트 PASS | PENDING: shader 픽셀/A12/A16 |
| 7 착석·정지 겹침/물·보행 보존/클릭 | 41자리·declump·drawn click headless PASS | PENDING: A18 및 실제 화면 선택 |
| 8 꺼진 겨울 화덕의 팔짱·떨림·입김 | 추위 pose/제외 조건/표식 없음 회귀 PASS | PENDING: 자연 cold 관측; 제어 장면은 별도 표시 |
| 9 장작 stock/target·장작만 tint·추위 HUD/정보창 | 분리 Label·선택적 target·한국어 2키 테스트 PASS | PENDING: 실제 화면 라벨/정보창 |
| 10 SCRIPT ERROR 없음·비슷한 프레임 시간 | P0 smoke 및 bounded headless tests PASS | PENDING: 현재 전체 캡처 콘솔/A10 |

최종 산출 예정 경로는 `docs/art/production5/` 아래 `before_after.png`, `season_strip.png`, `harvest_strip.png`, `hair_masks_before_after.png`, `cold_night.png`, `village_noon.gif`, `village_evening.gif`, `villagers_natural.png`이다. **이 목록은 생성 완료 목록이 아니다.** shots/calibration·동일 framing 비교·GIF 40프레임/8MB·최종 파일 존재/용량 확인은 PENDING이다. 자연 cold가 없으면 `cold_night.png`의 제어 장면 출처를 명시하고 A19(b)는 프레임별 cold 인원과 함께 N/A로 판단해야 한다.

이미 존재하는 리뷰 보드는 `docs/art/production5/harvest/animation/`, `masks/round1/`, `masks/round2/`, `masks/raw_context/`, `masks/animation/`, `masks/harvest/animation/`, `keys/processed/V_a6_attempt_1/`, `keys/processed/V_c4_attempt_1/`에 있다. 실제 게임 화면과 후보 보드를 구분한다.

### 시뮬레이션 작업과의 호환

Pass 6 정확한 SHA `fe60de21249ad46ad2e2d9ca420d896066277c82`를 merge commit `34b9d4b4f0e7323197125f208d3002244b9f40fe`로 깨끗하게 병합했다. 충돌 해결 파일은 없다. baseline bridge의 live constructor/reset은 `game_v12`다. `gates/p0/live_snapshot.log`에서 시작 agent 24명에 `colds` 24개, `wood_targets` 키 존재를 확인했다. 초기 settlement 0개는 겨울 목표 라벨의 실제 화면 증거가 아니다.

05:35 UTC에 원격 simulation green이 `a3669b665f8d636058392a7a73cc78f47ecc0ca1`로 이동했다는 leader 확인을 받았다. 캡처 baseline은 계속 `fe60de21`이며 새 시뮬레이션을 공개 art 브랜치에 병합하지 않는다. Part 9의 로컬·미푸시 호환 병합 및 그 게이트 결과는 PENDING이다. `colds`/`wood_targets` 없는 스냅샷의 기존 동작 보존은 headless 회귀로 확인했지만, Pass 6 데이터가 자연 게임 화면에서 보인다는 결론은 아직 내리지 않는다.

### 변경된 파일 목록

현재 first-parent 커밋 목록을 읽어 확인했다. 아래는 커밋별 주요 경로 묶음이며 대량 후보 PNG·리뷰 JSON을 모두 나열하지 않는다. 최종 추가 커밋/원격 양쪽 head 확인은 PENDING이다.

| Part / 커밋 | 주요 변경 경로 |
|---|---|
| P0 `54685484` | `docs/art/production5/GOAL.md`, 실행 기록 |
| P0 `34b9d4b4` | 정확한 simulation Pass 6 merge; art 자체 simulation 수정 없음 |
| P0 `0d47cccf` | `docs/art/production5/BASELINE.md`, `gates/p0/` |
| P1 `0a4c2f3b` | `tools/art_pipeline/{harvest_p5,merge_p5_selection,harvest_integrity_p5,v4_contracts,despeckle,regions,install_v4}.py`와 관련 가드·테스트, `scripts/ui/agent_activity.gd`, `assets/art/{manifest.json,atlas/,production/}`, `docs/art/production5/harvest/` |
| P2 `0ce26337` | `scripts/ui/{art_hash,agent_activity}.gd`, `shaders/villager_ramp.gdshader`, `assets/art/villager_ramps.png`, calibration·variant 테스트 및 `ingame_qa*`, `production_palette.json`, `variety/` |
| P3 `fc18aea8` | `tools/art_pipeline/{hair_masks_v5,remask_candidates_p5,remask_round2_p5,mask_installation_p5,finalize_masks_p5}.py`, `hair_masks_p5/`, `reviewed_hair_maps_p5.json`, atlas/장작 PNG, `docs/art/production5/masks/`, `P3SUMMARY.md` |
| P4 `63e9f764` | `tools/art_pipeline/key_*_p5.py`, `hair_key_palette.json`, `harvest_bindings_p5.py`, `harvest_baseline_p5.py`, A20/installer/회귀 강화, `docs/art/production5/{keys,references,prompts}/`, `image_calls.json` |
| P5 `7552a4ec` | `scripts/ui/{camp_seats,agent_renderer}.gd`, `scripts/test/art/{test_camp_seats,test_declump,test_agent_drawn_positions}.gd`, `docs/art/production5/spacing/` |
| P6 `abda06fe` | `scripts/ui/{cold_effects,agent_renderer,agent_activity,building_renderer}.gd`, `cold_audit.json`, HUD/inspector, `localization/{fluent,compiled}/`, `key_registry.json`, cold/HUD/inspector/wood QA 테스트, `docs/art/production5/cold/` |
| P7 코드 `f3418ef7` | `scripts/test/art/{capture_metadata,capture_plans,test_capture_metadata}.gd`, `scripts/test/capture_framings.gd`, `tools/art_pipeline/{capture_art.sh,ingame_qa.py,ingame_qa_cold_p5.py,measure_p5.py,presentation_p5.py,test_ingame_qa_cold_p5.py}` |

최종 WIP·green SHA 및 `git ls-remote` 동등성: PENDING. PR #511 Art Pass 5 설명 갱신: PENDING. PR은 draft 유지 대상이다.

### 자율 결정 목록

`docs/art/production5/DECISIONS.md`가 원장이다.

- GOAL이 기존 lead/main reset/harness 지시를 대체하므로 WIP art 브랜치에서 진행했고, 정확히 일치한 시작 원격 상태에서 불필요한 reset/backup은 만들지 않았다.
- 그림 자체 승인과 설치 자격을 분리하고, 가드 완화는 명시된 범위에만 적용했다. 후보 실패와 과거 판정은 보존했다.
- Part 3은 딱 한 번의 알고리즘 수정/재리뷰만 사용했다. 마스크 작업으로 기존 cycle_pop을 임의 수정하지 않았다.
- 참조·설치 RGBA가 달라진 경우 옛 감사 상태를 복구하지 않았다. 감사 원본은 손실 없는 보드로 다시 결속했다.
- 좌표 메타데이터 누락이라는 독립 리뷰 finding은 실패 회귀를 추가한 후 고쳤고 재리뷰 PASS를 받았다.
- 키 시트 STOP을 남은 생성 예산으로 무시하지 않았다. 원장 정규화와 가드 수정은 별도 파일로 남겨 원본 리뷰와 체크포인트를 보존했다.
- A20은 이름/상태뿐 아니라 실제 픽셀과 마스크, 리뷰 및 보드 해시를 검사하도록 강화했다. 최종 capture는 Part 1 단독 원장 대신 누적 설치 그룹을 사용한다.

### 확인된 제한사항

- 마스크 최종 미승인 45/105개가 남았다. c 애니메이션 `E drink`, `E eat`, `S drink`, `S idle` 네 그룹은 `cycle_pop` 거절이며 해결됐다고 세지 않는다.
- Part 4 신규 그림 0설치다. 키 색이 정확한 신규 성인 장작 그림을 확보했다는 주장은 할 수 없다. 장작 회수는 기존 승인 후보에 기반한 성인 N 두 그룹뿐이다.
- 복사/두 포즈 반복은 의도적으로 허용된 절충이다. 실제 신규 그림과 복사를 혼동하지 않는다. 아이 wood는 simulation에서 쓰지 않아 설치 범위 밖이다.
- Part 1 incomplete child N drink는 미회수이고, run 회수는 gameplay 사용을 보증하지 않는다.
- mask round 1의 일부 회수 보드에는 raw가 빠졌다. 원본/미정렬 보드 76개를 나중에 제공했고 최초 판정을 소급 수정하지 않았다.
- headless 테스트, 독립 후보 리뷰, A20은 실제 화면 A1–A19·블라인드 식별·프레임 시간을 대신하지 않는다. 자연 cold가 보이지 않으면 제어 장면으로 자연 발생을 주장하지 않는다.
- seat 메타데이터 생성은 회귀로 확인했다. unseat 시 제거는 독립 리뷰에서 코드로 확인했고 해당 전이의 추가 테스트는 하지 않았다.
- 최종 gate·호환 병합·아트 25MB/문서 40MB·GIF 8MB 예산·PR·remote head 확인은 아직 PENDING이다.

### Harness 결과

GOAL이 금지한 `harness_pipeline.sh`는 이 작업의 검증 경로가 아니다. 지정 `tools/art_pipeline/gate.sh`와 bounded 테스트의 결과를 적는다.

| 검증 | 확인된 결과 | 증거와 한계 |
|---|---|---|
| P0 전체 gate | Rust 1,941 / Python 243 / Godot 31 테스트 PASS, clippy·strict·bridge·smoke PASS | `docs/art/production5/gates/p0/results.tsv`, `BASELINE.md`; 기준선 결과 |
| Part 1 Python | 264 PASS | `harvest/final_tests.log` |
| Part 2 | Python 52 및 Godot 3 PASS | `variety/`; 픽셀 캡처 별도 |
| Part 3 | hair 29, region 23 PASS, fingerprint 315개 확인 | `P3SUMMARY.md`, `masks/tests_final.log` |
| Part 4/A20 강화 | Python 274 PASS, mutation 검출·최종 A20 PASS | `harvest/HARDENING.md`, `hardening_tests.log`, `integrity_final.json` |
| Part 5 | 자리·declump·그려진 좌표·클릭 4장면 PASS | `spacing/*.log` |
| Part 6 | Godot 7, Python wood 12 PASS, strict PASS | `cold/*.log`, localization 5,184키·중복/누락 0 |
| 독립 runtime review | 10장면 PASS; seat 수정 후 drawn/metadata 재실행 PASS | `runtime_review.md`; 렌더 픽셀·자연 캡처 미검증 |
| P7 pre-capture 전체 gate | 진행 중, PENDING | `gates/pre_capture/`; 부분 로그로 전체 PASS를 주장하지 않음 |
| Part 9 final gate | PENDING | 최종 head에 실행 후 갱신 필요 |

3,000개 ID의 톤 분포 `[723,772,755,750]`, 일반 머리 분포 `[595,620,575,601,609]`, 인코딩 28조합 통과는 분포/코드 검증이다. 화면에서 5색·4톤이 구별된다는 시각 판정은 아니다.

### 정직한 판단

회수 가능한 승인 그림을 되살리고 마스크와 표시 코드의 출처를 검증한 것은 분명한 개선이다. 하지만 현재 문서 시점에는 새 실제 게임 캡처와 A1–A19 측정이 없고, 신규 키 그림 설치는 0이며, 긴 머리 및 네 애니메이션의 cycle_pop 문제가 남아 있다. 따라서 처음 보는 사람이 완성도 있는 픽셀 아트 게임으로 볼 것이라고 아직 결론 내릴 수 없다. 최종 화면과 블라인드 식별 결과를 보고 이 판단을 갱신해야 한다.
