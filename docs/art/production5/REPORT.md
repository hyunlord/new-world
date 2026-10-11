## 구현 완료 보고

작성 기준: 실제 캡처 head `42f15c7e5231e788f540227e949c770d928211b2`와 2차 측정. **A11·A18 FAIL, 독립 시각 리뷰 NEEDS WORK**다. A19(b)는 자연 화면에서 가시 cold 0명으로 N/A다. 전체 게이트37/37와 로컬 호환 검사34/34를 통과했고 PR #511 설명을 갱신했다. 발행 HEAD·양쪽 원격 일치·exact-HEAD 재검증은 외부 publication 영수증에 기록한다. 아래 PASS는 적힌 범위의 증거만 뜻한다. 경로는 별도 표시가 없으면 저장소 루트 기준이다.

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
| 7 | 추운 밤 일정과 자리·겹침·마스크 출처·추위 메타데이터, 제어된 추위 장면 준비 | `scripts/test/art/capture_plans.gd`, `capture_metadata.gd`, `capture_cold_p5.gd`; ART 94장·ART_LONG 13장·calibration 완료 |
| 8 | 기존 임계값을 보존하는 A1–A20 집계·비교 이미지 생성 코드 준비 | `tools/art_pipeline/measure_p5.py`, `presentation_p5.py`; A11/A18 FAIL, 나머지 A검사 PASS(A19b N/A), 8종 산출 완료 |
| 9 | 전체 게이트37/37, 로컬 호환34/34·10창, 예산 검증, draft PR 설명 갱신 | `gates/final/`, `compatibility/result.json`, 외부 publication 영수증 |

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

캡처 head는 `42f15c7e5231e788f540227e949c770d928211b2`다. ART 94/94장, ART_LONG 13/13장과 calibration을 수집했다. ART_LONG은 기존 9장에 d23/d25/d27/d29 02:00 네 장을 추가한다. 원시 무손실 증거는 `/Users/rexxa/work/art-production5-evidence/`, 배포 화면은 `docs/art/production5/shots/`와 `calibration/`에 있다. 수치 원장은 `measurements.json`, 읽기용 결과는 `measurements.md`다.

| 검사 | Art Pass 4 기록 | Art Pass 5 실제 결과 |
|---|---|---|
| A1 | PASS gameplay 99/99 | PASS gameplay 103/103; 보정 포함 옛 범위 107장에서는 lineup z2 명도 표준편차 .0389로 FAIL 유지 |
| A2 | PASS 혼합 픽셀 0 | PASS 4×/6× 혼합 픽셀 0 |
| A3 | PASS draw rect 1,272 오류 0 | PASS draw rect 1,327 오류 0; adult S-idle 4개 비율 오류 0 |
| A4 | PASS 방향/배율 및 ramp 일치 | PASS north/south×4×/6× 일치율 모두 1.0; RAMP framebuffer 24조합 1.0 |
| A5 | FAIL 겨울 측정 가능 25/42=59.52% | PASS d2/겨울 대비 중앙 .368935/.310707; ≥.10 비율 100%; 측정 가능 14/15=93.33%, 27/37=72.97% |
| A6 | PASS 낮/밤 .312281/.153805 | PASS .309831/.153412, lit hearth 1개 평균 .253576, unlit glow 위반 0 |
| A7 | 당시 PASS, 이동 샘플 8 | PASS 현재 옛 추적 범위 40샘플, 새 전체 이동 343샘플≥30; 초별 서로 다른 프레임 6/4/7, 변화율 1.000 |
| A8 | PASS 98장 오류 0 | PASS 102장 오류 0 |
| A9 | PASS 99장 오류 0 | PASS 103장 오류 0 |
| A10 | PASS 중앙 7.547ms/p95 15.329ms | PASS live simulation 300프레임 중앙 6.907ms/p95 15.399ms |
| A11 | FAIL 4× 19/24, 2× 14/24 | **FAIL** 4× 21/24=.875≥.8, 2× 14/24=.583333<.6 |
| A12 | FAIL 17명/11조합, 같은 이웃 0/7 | PASS 15명/13조합, 같은 이웃 0/4; P4 정의 반사실 12조합 PASS |
| A13 | PASS 체형×행동 24/24 | PASS 24/24 기하 차이 존재, missing 0, 금지 berry ramp 0 |
| A14 | PASS 숲 25.378%, 빈터 위반 0 | PASS 302/1,190=25.378%, 자연물 421개/중심 18개 위반 0 |
| A15 | PASS 배치 638/중심 18 위반 0 | PASS 동일 집계 638/18, rock/decal 중복 0, native 장식 13종 PASS |
| A16 | PASS 계절·겨울색·노인 | PASS d10/18/24/32 계절 일치, 겨울 B/R 1.167361배; 주민 35/37 측정 가능·각 일치율 1.0; d40 노인 9명 |
| A17 | PASS이나 이동 장작 실프레임 0/3 | PASS hearth 103/deadwood 536/HUD 103 검사, 오류 0; 현재 이동 장작 1명=**실프레임 0/대체 1** |
| A18 | FAIL d18 3/39, d24 6/42 | **FAIL** d2 정오 0/15·저녁 0/19·d18 0/32, d24 3/37>허용 1 |
| A19 | 새 항목 | (a) 3장면 테스트 PASS; (b) **N/A 가시 cold 0, 검사 주기 0**; (c) target 라벨 101개 오류 0; 집계 PASS는 자연 추위 수락이 아님 |
| A20 | 새 항목 | PASS 설치 기록 2, bound 74, 변경 주민 61, 애니메이션 20그룹, 오류 0 |

A17의 이동은 현재 snapshot state=walking이면서 마지막 동기 advance ≤8 ticks에서 변위가 있는 경우다. 변위가 있었던 장작 운반자는 3명이지만 현재 정지 2명을 제외하면 1명이다. 설치된 N carry_wood가 있어도 이번 가시 이동자는 fallback이므로 **실제 장작 그림의 자연 이동 시연은 0건**이다. 승인 W1 단계 검사 536건과 이 한계를 별도로 보고한다.

범위 수정과 수정 전 결과를 보존했다.

| 항목 | 이전 정의·비교 결과 | 이번 정의·결과 |
|---|---|---|
| A7 | P4 당시 8 이동 표본; 현재 같은 추적 방식 40표본 PASS | 정오 GIF 모든 가시 이동 343표본, 최소 30 유지, PASS |
| A12 | 같은 ID를 옛 hair 0–2/tone 0–2로 에뮬레이션하면 12조합·이웃 0/4 PASS. 더 오래된 P3 diagnostic은 15조합 | 현재 5 일반 hair×4 tone에서 13조합·이웃 0/4 PASS. 반사실 계산은 과거 픽셀의 실측이 아님 |
| A16 | 같은 현재 hair 코드에 옛 노인 3/4·일반 0–2 규칙 적용하면 FAIL; 더 오래된 d40=봄 진단도 FAIL | 새 노인 5/6·일반 0–4 PASS, d32 봄 PASS. 계절과 hair 분류를 각각 기록 |

1차 측정 원장은 `measurements_round1/`에 남겼다. LONG의 13/9 실패는 P4 기대 수 9를 그대로 사용한 검사 오류였고 명시된 추가 밤 4장으로 13/13 검사로 고쳤다. A16은 저장소 Label의 앞쪽 가림을 제외하지 않아 생긴 잘못된 불일치를 수정했다. 임계값을 낮추지 않았으며 1차 실패 원장을 덮지 않았다. A5-old diagnostic은 현재도 d2 중앙 .1077/겨울 .0664로 FAIL이다.

블라인드 결과는 `blind/answer_key.json`, `observer_2x.json`, `observer_4x.json`, `observer_labels.sha256`, `result.json`에 묶였다. 정답 선동결 후 독립 관찰자의 24개 라벨을 비교했다. 아래는 정답 행→응답 열의 **희소 혼동 행렬**이며 생략한 칸은 0이다.

| 정답 | 4× 응답별 수 | 2× 응답별 수 |
|---|---|---|
| child | child 3 | child 1, drink 1, walk 1 |
| talk | talk 2 | talk 2 |
| gather | gather 3 | gather 3 |
| build | build 3 | build 3 |
| eat | eat 2, drink 1 | carry 2, drink 1 |
| sit | sit 2 | sit 2 |
| carry | carry 3 | eat 3 |
| drink | drink 1, walk 1 | drink 1, walk 1 |
| walk | walk 2, drink 1 | walk 2, carry 1 |

특히 2×의 carry 3개가 모두 eat로 읽혀 작은 화면에서 운반 개선을 충분히 달성하지 못했다. 블라인드는 calibration lineup 식별이며 자연 화면 전체의 행동 추적 시험은 아니다.

GOAL 14항의 직접 플레이 확인표:

| 항목 | 관측·테스트 증거 | 판정과 남은 범위 |
|---|---|---|
| 1 아이 실제 걸음/발만 교체되지 않음 | 실제 walk 12프레임 설치, A7 이동 343샘플 PASS | 이동 프레임 변화 확인. 독립 리뷰는 특정 아이의 완전한 사이클을 입증하지 못했으므로 발 미끄러짐까지 전면 수락하지 않음 |
| 2 실제 바구니 운반/아이콘 제거/두 포즈 연속성 | carry 13실제+7복사, C1/C2 재생 테스트, 4× carry 3/3 식별 | **부분 충족**: 2× carry 0/3 식별, 독립 GIF 표본은 loop seam·전 체형 아이콘 부재까지 증명하지 않음 |
| 3 가지 단계·소진 흔적·장작 그림/대체 | A17 deadwood 536건 오류 0, W1 13개, 이동 장작 1명 fallback | 단계 sprite 계약 PASS. 실제 장작 그림 자연 이동 0건; 한 장소를 끝까지 줍는 연속 사용자 관측은 별도 미검증 |
| 4 장작 머리색 번짐·앵커 흔들림 | 승인 mask·foot anchor·애니메이션 리뷰, sampled 보드에서 짐 색 보존 | **자연 가시 이동 미검증**: real carry_wood 0건이라 실제 화면의 좌우 튐 제거를 선언하지 않음 |
| 5 흰머리 전체·팔/도구/외곽선 보존 | 15마스크 적용, 자연색 보드 독립 표본 GOOD, A16 측정 35명 RGB 일치 | **부분 충족**: 긴 머리 미승인 및 cycle_pop 4그룹 잔존, 모든 노인의 마스크 완전성 아님 |
| 6 일반 머리 5색·피부 4톤·노인 전용 2색 | A12 13조합 PASS, A16 노인 9명 새 코드 PASS, shader 픽셀/자연색 보드 | 정의·분포·측정 픽셀 PASS. sampled 시각 GOOD이며 모든 조합의 인간 구별 가능성을 보증하지 않음 |
| 7 착석·정지 겹침/물·보행 보존/클릭 | 자리/declump/그려진 위치 클릭 headless PASS; 3장면 과반 가림 0 | **FAIL** d24 ID 25/8/7 과반 가림 3/37. 실제 crowded 화면 클릭은 별도 미검증 |
| 8 꺼진 겨울 화덕의 팔짱·떨림·입김 | 3장면 테스트 및 labelled cold fixture, 독립 시각 대비 GOOD | **자연 발생 N/A**: 모든 gameplay frame visible cold 0, 주기 0. 제어 장면으로 겨울 자연 발생/회복을 주장하지 않음 |
| 9 장작 stock/target·장작만 tint·추위 HUD/정보창 | target 라벨 101개 오류 0, `37/39` 화면 독립 확인, HUD/정보창 테스트 | 목표 표시 PASS. 자연 cold HUD/정보창 발생 미관측. 작은 저장소 글씨와 밤 대비는 독립 리뷰 NEEDS WORK |
| 10 SCRIPT ERROR 없음·비슷한 프레임 시간 | pre-capture 전체 게이트 PASS, A10 300프레임 6.907/15.399ms | 측정 성능 PASS. 최종 코드 gate37/37 PASS이며 캡처·calibration·controlled scene 콘솔4개에서 SCRIPT ERROR/ERROR 0 (`visual/capture_console.json`) |

독립 리뷰 `visual/integrity_review.md`, `visual/precision_review.md`와 합산 `visual/verdict.json`은 모두 **NEEDS WORK**다. 픽셀 윤곽·팔레트·주요 한국어 HUD·낮밤 대비는 GOOD이지만, 늦은 정착지 군집, 작은 저장소 라벨, 2× 행동, 약한 계절 구분이 남았다. 40프레임 GIF 중 표본 0/10/20/30/39를 본 리뷰이므로 연속 재생의 부드러움·발 미끄러짐·클릭 정확도를 검증한 것으로 확대하지 않는다.

8종 결과 파일은 모두 `docs/art/production5/`에 존재하며 출처·SHA256·크기는 `presentation_artifacts.json`에 있다.

| 산출물 | 경로 | 확인된 범위 |
|---|---|---|
| 전후 비교 | `docs/art/production5/before_after.png` | 같은 named framing의 P4/P5 비교 |
| 계절 | `docs/art/production5/season_strip.png` | d10/18/24/32; 미묘한 계절 시각 차이는 리뷰 제한 |
| 회수 | `docs/art/production5/harvest_strip.png` | 최종 설치 그룹, REAL/COPY 등 출처 표시 |
| 마스크 | `docs/art/production5/hair_masks_before_after.png` | 감사 29개, 실제 현재 region 픽셀과 비교 |
| 추운 밤 | `docs/art/production5/cold_night.png` | **controlled scene test**, 자연 gameplay 아님 |
| 정오 GIF | `docs/art/production5/village_noon.gif` | 40프레임, 2,188,055 bytes<8MB |
| 저녁 GIF | `docs/art/production5/village_evening.gif` | 40프레임, 1,020,967 bytes<8MB |
| 자연색 주민 | `docs/art/production5/villagers_natural.png` | tone 0–3/hair 0–6, fallback 표시 |

리뷰 보드는 `docs/art/production5/harvest/animation/`, `masks/round1/`, `masks/round2/`, `masks/raw_context/`, `masks/animation/`, `masks/harvest/animation/`, `keys/processed/V_a6_attempt_1/`, `keys/processed/V_c4_attempt_1/`에 있다. 실제 게임 화면과 후보 보드를 구분한다.

캡처 provenance: pre-capture gate는 `f3418ef7c19bf98da19823db52616158fe005c24`, 촬영은 `42f15c7e5231e788f540227e949c770d928211b2`이며 사이 변경은 보고서·작업 기록·게이트 로그뿐이다. ART/ART_LONG/calibration dirty 개수는 0/1/2로 기록되었다. 생성 중 증거 파일이 있었지만 당시 dirty 경로 자체는 동결하지 않았으므로 정확한 경로 목록은 미보존이다. Git diff로 게임 renderer·simulation 변경 없음과 이후 별도 controlled scene viewport 수정만 확인했다. GIF 80프레임 메타데이터는 외부 원본 절대 경로+SHA로 연결하고, 게시 이미지 크기와 GIF frame index를 명시했다. `visual/capture_provenance.json`, `visual/manifest_links.json` 참조.

### 시뮬레이션 작업과의 호환

Pass 6 정확한 SHA `fe60de21249ad46ad2e2d9ca420d896066277c82`를 merge commit `34b9d4b4f0e7323197125f208d3002244b9f40fe`로 깨끗하게 병합했다. 충돌 해결 파일은 없다. baseline bridge의 live constructor/reset은 `game_v12`다. `gates/p0/live_snapshot.log`에서 시작 agent 24명에 `colds` 24개, `wood_targets` 키 존재를 확인했다. 초기 settlement 0개는 겨울 목표 라벨의 실제 화면 증거가 아니다.

초기 관측 `a3669b665f8d636058392a7a73cc78f47ecc0ca1` 뒤 최종 호환 fetch에서 simulation green은 `0c46f69c31344f3bde23ce625a84214c5ed72fc0`였다. 캡처 baseline은 계속 `fe60de21`이며 새 시뮬레이션을 공개 art 브랜치에 병합하지 않는다. 같은 checkout의 detached HEAD에서 `631434e331265800b6d7524d40e513ea04947f3d`로 임시 병합했다. `harness_p14_delta_status_panel.rs`의 빈 줄 충돌1개만 정리하고 양쪽 assertion을 유지했으며 해당17테스트가 통과했다. Bridge·strict·smoke·Godot30개·오류 검사34/34 PASS, 실제 창10장과 원본SHA10개 일치, SCRIPT ERROR0, 낮·밤2장을 직접 확인했다. 결과는 `compatibility/result.json`과 두 JPEG에 있다. 이 병합은 미푸시이며 WIP `84cb3da6939d5776e1e692f605db77b255dd9ea3`로 복귀해 Pass6 bridge를 다시 빌드했다. game_v13의 d1/d2/d6에 한정한 호환 증거이며 장기 A1–A20 재측정이 아니다. `colds`/`wood_targets` 없는 스냅샷의 기존 동작 보존은 headless 회귀로 확인했다. 자연 화면 target 라벨 101개를 검증했지만 visible cold는 모두 0이라 자연 추위 표시 수락은 N/A다. d32·d40 누적 `cold_member_hours=378`은 simulation에서 cold가 있었음을 뜻할 뿐, 그 시점을 화면에서 관측했다는 증거가 아니다.

### 변경된 파일 목록

현재 first-parent 커밋 목록을 읽어 확인했다. 아래는 커밋별 주요 경로 묶음이며 대량 후보 PNG·리뷰 JSON을 모두 나열하지 않는다. P8까지 전체 경로는 `commit_files.json.gz`에 있으며 이후 기록은 각 커밋 diff가 원본이다. 보고서 봉인 커밋 자체와 원격 양쪽 head는 최종 응답 및 외부 publication 영수증에 기록한다.

| Part / 커밋 | 주요 변경 경로 |
|---|---|
| P0 `546854844c4cdc998d6e314b5ffddbc5a0d3d6b1` | `docs/art/production5/GOAL.md`, 실행 기록 |
| P0 `34b9d4b4f0e7323197125f208d3002244b9f40fe` | 정확한 simulation Pass 6 merge; art 자체 simulation 수정 없음 |
| P0 `0d47cccf3f42d8882644e7dd521d626e0e782a1a` | `docs/art/production5/BASELINE.md`, `gates/p0/` |
| P1 `0a4c2f3b588816b385cc5cb399439bf9f180d8ec` | `tools/art_pipeline/{harvest_p5,merge_p5_selection,harvest_integrity_p5,v4_contracts,despeckle,regions,install_v4}.py`와 관련 가드·테스트, `scripts/ui/agent_activity.gd`, `assets/art/{manifest.json,atlas/,production/}`, `docs/art/production5/harvest/` |
| P2 `0ce263372e52461f4fd7c4ad3281beaa5ce04fcb` | `scripts/ui/{art_hash,agent_activity}.gd`, `shaders/villager_ramp.gdshader`, `assets/art/villager_ramps.png`, calibration·variant 테스트 및 `ingame_qa*`, `production_palette.json`, `variety/` |
| P3 `fc18aea879ee2d07bd29a042d610095452c45e89` | `tools/art_pipeline/{hair_masks_v5,remask_candidates_p5,remask_round2_p5,mask_installation_p5,finalize_masks_p5}.py`, `hair_masks_p5/`, `reviewed_hair_maps_p5.json`, atlas/장작 PNG, `docs/art/production5/masks/`, `P3SUMMARY.md` |
| P4 `63e9f764801c702d16d62ec12758f05583d74726` | `tools/art_pipeline/key_*_p5.py`, `hair_key_palette.json`, `harvest_bindings_p5.py`, `harvest_baseline_p5.py`, A20/installer/회귀 강화, `docs/art/production5/{keys,references,prompts}/`, `image_calls.json` |
| P5 `7552a4eca4e8cb713f84f8d88978e434165f7060` | `scripts/ui/{camp_seats,agent_renderer}.gd`, `scripts/test/art/{test_camp_seats,test_declump,test_agent_drawn_positions}.gd`, `docs/art/production5/spacing/` |
| P6 `abda06fe85a08668cca9326d45497f020388a479` | `scripts/ui/{cold_effects,agent_renderer,agent_activity,building_renderer}.gd`, `cold_audit.json`, HUD/inspector, `localization/{fluent,compiled}/`, `key_registry.json`, cold/HUD/inspector/wood QA 테스트, `docs/art/production5/cold/` |
| P7 코드 `f3418ef7c19bf98da19823db52616158fe005c24` | `scripts/test/art/{capture_metadata,capture_plans,test_capture_metadata}.gd`, `scripts/test/capture_framings.gd`, `tools/art_pipeline/{capture_art.sh,ingame_qa.py,ingame_qa_cold_p5.py,measure_p5.py,presentation_p5.py,test_ingame_qa_cold_p5.py}` |
| P7 사전 게이트 `42f15c7e5231e788f540227e949c770d928211b2` | `gates/pre_capture/`, 보고서·실행 기록; 게임 코드 변화 없음 |
| P7 실제 캡처 `63f22e5e3396b6df5ad5ecf2ee9edb0f80516a1f` | `docs/art/production5/{shots,calibration,cold}/` 실제 증거; 촬영 코드 head `42f15c7e5231e788f540227e949c770d928211b2`와 구분 |
| P8 `49f9a00cd9439741eb901e084152766c52012676` | QA 범위·라벨 가림 보정, 실패 보존,8개 산출물,독립 리뷰,불변 GIF 최적화 |
| P9 `84cb3da6939d5776e1e692f605db77b255dd9ea3` | 전체 게이트와 캡처 source/metadata 링크,보고서·전수 검증 |
| P9 최종 봉인 | `compatibility/`, 보고서·예산·발행 기록; 전체 SHA는 최종 응답 |

PR [#511](https://github.com/hyunlord/new-world/pull/511)의 기존 설명을 보존하고 Art Pass5 절을 갱신했으며 draft=true를 재조회했다. WIP/green은 전체 exact-HEAD 게이트 통과 후 같은 커밋으로 fast-forward하며 강제 push는 하지 않는다. 최종 HEAD·직렬 `git ls-remote` 확인·clean status·재검증·종료 시각은 `/Users/rexxa/work/art-production5-evidence/publication.json`에 기록한다. exact-HEAD 게이트 원본은 `/Users/rexxa/work/art-production5-evidence/final-head-gate/`다. 자기 자신을 포함한 SHA를 보고서에 내장하지 않는다.

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
- 실제 측정 A11과 A18은 FAIL이다. d24 ID 25/8/7의 과반 가림이 남았으며 최대 8px 이동·착석 제외라는 현재 계약 안에서 완전히 없애지 못했다. 기준을 낮추거나 이를 PASS로 바꾸지 않았다.
- 자연 visible cold는 0, 실제 그림의 가시 이동 장작도 0이다. fixture·후보 보드·simulation 누적치를 자연 화면 수락으로 바꾸지 않는다. 독립 리뷰의 작은 라벨·계절 구분·행동 식별 NEEDS WORK도 남는다.
- seat 메타데이터 생성은 회귀로 확인했다. unseat 시 제거는 독립 리뷰에서 코드로 확인했고 해당 전이의 추가 테스트는 하지 않았다.
- 전체 코드 gate와 한정된 로컬 호환 gate는 PASS다. 최종 산출 용량은 `report_artifacts.json`/`budget.json`, 발행 상태는 외부 publication 영수증이 원본이다. Godot가 재생성하는 무시된 .import 캐시는 산출물이 아니며 종료 시 production5 캐시를 제거한다. 두40프레임 GIF는8MB 이하이며 저녁 GIF 최적화 전후 decoded RGB·duration이 전부 같다.

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
| P7 pre-capture 전체 gate | **PASS 37검사**, Rust 1,941/Python 274/Godot 30, clippy·strict·bridge·smoke | `gates/pre_capture/results.tsv`; 최종 head gate와 구분 |
| 측정 수정 후 Python | 275 PASS (leader 실행 확인) | 최종 gate 수치와 분리; 최종 로그에서 다시 확인 |
| Part 9 final gate | PASS 37/37, Rust 1,941 / Python 275 / Godot 30 | `gates/final/`; clippy·strict·bridge·smoke PASS |

3,000개 ID의 톤 분포 `[723,772,755,750]`, 일반 머리 분포 `[595,620,575,601,609]`, 인코딩 28조합 통과는 분포/코드 검증이다. 실제 framebuffer RAMP 24조합과 겨울 측정 주민 35명은 별도 픽셀 검증으로 통과했다. 모든 hair/tone 조합의 인간 식별성을 테스트한 것은 아니다.

### 정직한 판단

회수 가능한 승인 그림을 되살렸고, A5 대비·측정 가능 비율과 A12 다양성이 통과했으며 d18까지 과반 가림이 0으로 줄었다. 픽셀 윤곽·팔레트·주요 한국어 HUD와 낮밤 분위기도 독립 리뷰에서 좋게 평가되었다. 그러나 **완성도 있는 픽셀 아트 게임으로 무조건 수락할 단계는 아니다**. 2× 운반이 식사로 읽히는 A11 실패, d24 군집의 A18 실패, 작은 저장소 라벨과 약한 계절 구분이 직접 보인다. 신규 키 그림 설치는 0이고 자연 cold·실제 장작 그림 이동은 이번 화면에서 관측하지 못했다. 따라서 구현·출처·측정 개선은 인정하되 전체 시각 판정은 NEEDS WORK로 남긴다. 최종 기술 게이트가 통과하더라도 이 시각 실패가 사라지는 것은 아니다.
