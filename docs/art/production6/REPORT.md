## 구현 완료 보고

### 구현 의도

잠든 주민 때문에 보이지 않던 추위를 오두막·화덕에 표시하고, 운반과 식사를 구분하며, 신의 개입을 실제 화면에 연결했다. **코드와 시각 수락은 별도이며 A1–A25는 21 PASS / 4 FAIL이다.** A19의 몸 단위 자연 추위 관측은 N/A다.

### 구현 내용

| Part | 구현·증거 |
|---|---|
| 0 | 정확한 simulation green 병합, v14 probe, 변경 전 ART94/LONG21 baseline |
| 1 | `shelter_cold.gd`, building/world effects: 지붕 서리·문간 입김·꺼진 겨울 화덕의 재·화면 색조 |
| 2 | `morning_chill.gd`, agent renderer: 별도 chill 기억과 fade, reset, 최대8tick 캡처 feed |
| 3 | HUD·en/ko: 겨울 마지막 밤·혹독함·유효한 추위0 포함 count |
| 4 | carry source46프레임 측정, 승인 food6그룹20프레임의 붙은9×7 짐, 기존 wood art 선택 복구 |
| 5 | read-only FFI getter, 일곱 버튼·toast·최근8기록·비/한파/바람/저장소 효과 |
| 6 | seated/idle 군중 앞뒤 배치 보강, walking/running/carrying 경로 보존 |
| 7 | 동일44011f92 소스 ART94/LONG21/GOD8/calibration4; 원본·metadata·주기 증거 |
| 8 | 전수 측정·두 관찰자 블라인드2회·독립 화면 판정·비교판·GIF |
| 9 | 전체 게이트, 정적 회귀 보강, 호환·제한 보고 및 draft PR 갱신 |

### 구현 방법

차가운 Sleep/Consuming 주민과 오두막 타일을 읽어서만 서리를 결정한다. 불투명 상단40% 경계 픽셀과 한 줄 아래 교대 픽셀 외에는 바꾸지 않으며0.4초 fade, 문간3×2 puff와2.4초 주기, 재3px, 자연 tint0.12를 사용한다. 화덕 연기는 실제 fire_lit가 켜져 있을 때만 그린다.

한기는 마지막 cold 이후의 first-clear 시점을 UI에서 기억하고90분 동안 유지하며 마지막30분에 사라진다. `chill_*` 키는 실제 `cold` 키와 섞지 않는다. capture는8tick마다 FFI를 읽고, 독립 cold trace도 별도로 남긴다. HUD는 달력 대신 booked winter_severity로 마지막 밤을 표시한다.

food carry만 독립 리뷰의 승인20프레임을 byte-bind했다. body patch와 붙은 prop은 동일 flip·z-order·가림 footprint를 쓴다. 거절된 wood 합성은 설치하지 않았고 기존 승인 N방향 그림을 다른 방향에서도 fallback보다 우선한다. 원래 region map에는 torso/clothing 라벨이 없어20개 기하 proxy 통과를 정확한 실루엣 인증으로 부르지 않는다.

신의 손은 `apply_intervention`만 호출하며 읽기는 `get_intervention_state`와 snapshot을 쓴다. 비13시간, 한파1밤, 바람20, 식량±60, 장작±40 preset을 검증했다. 실제 캡처는 패널 Button.pressed를 통해 명령을 실행했고 직접 snapshot 값을 조작하지 않았다. 비는 화면1×3px, 한파 tint0.18, 낙하는0.6초, 저장소 색상은1.2초다. 아이콘은 추가 생성 없이 번역된 텍스트로 대체했다.

군중 보정은 이동하지 않는 대상만1–8px 이동시키고 물/막힌 타일을 피한다. 실제 움직이는 주민을 옮겨 수치를 맞추지 않았으며, 그 결과 전수 A18은 여전히 실패한다.

### 실제 게임 화면 검증

자연 ART/LONG의111장과 별도 `INTERVENTION SCENARIO` GOD8장을 구분했다. ART의 나머지4장은 calibration/lineup이며 자연 수치에서 제외한다. 별도 calibration4장을 포함해 최종 계획 캡처127장이다. 모든 최종 capture manifest는 `44011f92347dc4176cc115195b1cc55a16ff93ae`에 묶였다. 카메라 before/after 일치, 원본 SHA, body/prop 바인딩은 각 원장에 기록했다.

| 항목 | 판정 | 수치·근거 |
|---|---|---|
| A1 | PASS | 자연 111프레임 모두 통과; calibration/lineup 4장은 별도 |
| A2 | PASS | 4×·6× checker 혼합 픽셀 0 |
| A3 | PASS | drawn rect 1,325개; 크기 오류 0 |
| A4 | PASS | 4×·6× N/S 불투명 마스크 일치율 1.0 |
| A5 | PASS | d2/겨울 대비 중앙 .39246/.35441; 측정 가능 86.67%/76.32%; 대비≥.10 비율 100% |
| A6 | PASS | 낮/밤 휘도 .31346/.15558; 불 꺼진 화덕 glow 위반 0 |
| A7 | PASS | 이동 329샘플; 초별 프레임 7/4/6; 변화율 1.000 |
| A8 | PASS | 110장 검사, 오류 0 |
| A9 | PASS | 111장 검사, 오류 0 |
| A10 | PASS | live simulation 300프레임; 중앙 8.212ms / p95 14.918ms |
| A11 | PASS | 관찰자1 4×/2× 87.5%/87.5%; 관찰자2 87.5%/91.67%; 운반 각각 3/3 |
| A12 | PASS | 15명 중 서로 다른 조합 13; 동일 이웃 0/4 |
| A13 | PASS | 3체형×8활동=24개 기준 충족; 누락 0 |
| A14 | PASS | 숲 나무 302/1,190=25.378%; 421개 검사, 위반 0 |
| A15 | PASS | 638배치 검사, 위반 0; 설치 decal 13 |
| A16 | PASS | 지형 191,168픽셀; 겨울/여름 B:R 1.16753≥1.03; 노인 9명 |
| A17 | PASS | 실제 carry_wood 14관측, 고유 10명·2체형; fallback 0; 자연 111장 전수 |
| A18 | FAIL | 111장 중 12장 한계 초과; d24 F2 6명, 최대 34명; 허용≤1 |
| A19 | PASS | 회귀 3개 통과·target label 108개; 보이는 실제 cold 주민 0, 몸 표현 N/A |
| A20 | PASS | 설치 원장 2개, bound 74프레임·변경 주민 61·20그룹, 오류 0 |
| A21 | FAIL | 겨울 밤 8장 검사; 추운 3장 모두 문간 입김 반복 식별 실패 |
| A22 | FAIL | 3장 실패; d29 chill metadata 42/31명이나 픽셀 식별 0; d27 대조군도 실제 최근 cold |
| A23 | PASS | 자연 111장 HUD 전수 일치; d29 겨울 마지막 밤·혹독함·추위 수 일치 |
| A24 | FAIL | GOD 8장; 바람 +0.2s/+1s 2장 효과/종료 식별 실패; 명령 응답은 모두 ok |
| A25 | PASS | 자연 추운 밤 3/3장, 각각 서리·재·색조·HUD 4개 단서 관찰 |

수치 원장: [measurements.md](measurements.md), [measurements.json](measurements.json), [독립 관찰](visual_observations.json). A19 PASS는 테스트·잘못된 cold cue 없음·라벨 검사에 한정되며, 몸 단위 자연 추위 가시성은 명시적으로 **N/A**다. A10은 강제 draw 없는 실제300프레임: Pass5 중앙6.907/p95 15.399ms 대비 중앙 약18.9% 증가, p95 약3.1% 감소다. 성능 수치 하나로 장기 안정성을 주장하지 않는다.

블라인드는 답안 키를 먼저 동결하고 독립 Codex 서브에이전트 세션2개에 이미지와 허용 라벨만 제공했다. 첫 회 실패를 `blind/round1`에 유지했다. 두 번째 회는 실제 runtime load helper가 적용된 framebuffer이며, 두 사람 모두 운반3/3을 식별했다. 다음 혼동 행렬은 합산하지 않은 각각의 결과다.

관찰자 1, 4x: 행=정답, 열=응답.

| 정답 | sit | gather | build | walk | eat | carry | child | talk | drink |
|---|---|---|---|---|---|---|---|---|---|
| sit | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| gather | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| build | 0 | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| walk | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 | 0 |
| eat | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 1 |
| carry | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| child | 0 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | 0 |
| talk | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| drink | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | 1 |

관찰자 1, 2x: 행=정답, 열=응답.

| 정답 | sit | gather | build | walk | eat | carry | child | talk | drink |
|---|---|---|---|---|---|---|---|---|---|
| sit | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| gather | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| build | 0 | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| walk | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 | 0 |
| eat | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 1 |
| carry | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| child | 0 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | 0 |
| talk | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| drink | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | 1 |

관찰자 2, 4x: 행=정답, 열=응답.

| 정답 | sit | gather | build | walk | eat | carry | child | talk | drink |
|---|---|---|---|---|---|---|---|---|---|
| sit | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| gather | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| build | 0 | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| walk | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 | 0 |
| eat | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 1 |
| carry | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| child | 0 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | 0 |
| talk | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| drink | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | 1 |

관찰자 2, 2x: 행=정답, 열=응답.

| 정답 | sit | gather | build | walk | eat | carry | child | talk | drink |
|---|---|---|---|---|---|---|---|---|---|
| sit | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| gather | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| build | 0 | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| walk | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 0 | 0 |
| eat | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 1 |
| carry | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| child | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 |
| talk | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 |
| drink | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | 1 |

독립 최종 판정 원문은 [scene_observations.json](review/scene_observations.json), 색상 재확인은 [scene_effect_observations.json](review/scene_effect_observations.json)에 보존했다. 자연 추운 밤 **NEEDS WORK**(서리·재·tint는 보이나 반복 입김 불명확), 아침 **ACCEPT**(낮으로의 전환에 한정, 한기 식별 승인이 아님), 운반2× **ACCEPT**(비교판 범위), 신의 손 **NEEDS WORK**(비·tint·패널은 보이나 바람 식별 부족). 기존 판정을 수정하지 않았다.

GOAL §14의8항목: (1) 서리·재·색조 확인, 입김 반복 미수락; (2) 마지막 밤/혹독함/count 전수 통과; (3) 아침 한기 실패; (4) 블라인드 운반/식사 통과, 정확한 region-map 실루엣 미인증; (5) 자연 장작 운반10명2체형 관측; (6) 일곱 버튼·실노드 응답 통과, 비 관측·바람 미수락; (7) 겨울 낮 겹침 실패; (8) 최종 캡처 SCRIPT ERROR0, 성능은 A10 수치 범위에서 통과.

산출물:
- [before_after.png](before_after.png): Pass5와 같은 카메라의 두 장면 및 실제2× 운반 crop.
- [cold_night.png](cold_night.png): 자연 d28 02:00 F2Z.
- [god_hand.png](god_hand.png): 개입 패널·네 효과, INTERVENTION SCENARIO 표시. 표시된 영역이 모두 시각 수락되었다는 뜻은 아니다.
- [carry_2x.png](carry_2x.png): 모든 carry그룹/36프레임의 staged native 비교; [carry_lineup_2x.png](carry_lineup_2x.png)는 실제 framebuffer 비교.
- [season_strip.png](season_strip.png), [낮 GIF](village_noon.gif), [저녁 GIF](village_evening.gif).
- [운반 리뷰 보드](carry/boards/), [HUD 리뷰 보드](review/boards/), [blind2×](blind/blind_2x.png), [blind4×](blind/blind_4x.png), [5초 주기 원본 인덱스](review/cycle_contacts.json).

### 시뮬레이션 작업과의 호환

병합한 SHA는 `4c35e29573bc28878de0cfed9ac67e0df0aa4c48`이며 더 최신 simulation head는 병합하지 않았다. 충돌1개 `harness_p14_delta_status_panel.rs`는 art의 content assertion을 유지했고 미사용 sha256 import는 남기지 않았다. 실행 world는 game_v14다.

시뮬레이션 source 변경은 `world_node.rs`의 additive102줄뿐이다: 읽기 전용 getter `get_intervention_state`, 순수 collector/columns, unit test. 다음 시뮬레이션 작업에서 이 FFI를 이어받아야 한다. 그 외 Rust 변경은 허용된 GDScript 정적 소스 검사이며 [TEST_LEDGER](../TEST_LEDGER.md)에 화덕 호출13개와 장면 리소스 검사4개의 변경을 남겼다.

v14 실제02:00 cold는28·29·54–57·84–85일,42/42명이다.23–27일02:00에는 불이 켜져 있고 cold0이다.15/16일15:00 GatherWood14/8명, 이동 운반9/6명으로 계획자의9/7과 한 명 차이가 있다. 날짜는 유지했다. 다만27일 아침의 독립 trace는06:00까지 cold가 있었음을 보여 warm-control 전제가 성립하지 않는다. 실제 flag를 숨기거나 다른 날로 대체하지 않고 A22 FAIL을 유지했다.

### 변경된 파일 목록

커밋별 전체 경로와 status는 [CHANGED_FILES.md](CHANGED_FILES.md)에 기록했다. 핵심은 UI/rendering·번역·capture와 `_p6` 측정 도구이며 simulation source는 위 예외뿐이다. 아래 목록 이후의 최종 보고/게이트 증거 커밋은 최종 응답의 SHA와 git diff로 확인한다.

- `54ad34583c0f9bc5fc253509cf89d9d3a17bf4d7` art-p6(p0): bring in simulation passes 7 and 8
- `b73a5e0513ad3a724a949345738c65326406e3e4` art-p6(p0): preserve comparable v14 baseline observations
- `7bcca1971c6115f20da6973e819a03e3b01f52e1` art-p6(p0): establish natural v14 cold and carrier baselines
- `d28cd95777d14b58d9cf39a3aa5a2cbb371ae659` art-p6(p1): make sleeping villagers cold visible at their shelters
- `487147f35a1624c3e9fd69cf24efb37e9fb1d541` art-p6(p2): retain visible chill after cold flags clear
- `af3583bdf8944f477b801d46ba4ca91382c809aa` art-p6(p3): label the booked winter night truthfully
- `9afd793f3e08029cdffbcf1c4cddfffad6c1c091` art-p6(p4): distinguish approved food loads from meals
- `23c7c972fdc1034a1439a93bc44519f18e81acde` art-p6(p5): make the seven divine interventions usable on screen
- `cbef2d790ffbbdfd6889fbdd1e7bea56386c7a11` art-p6(p6): separate seated crowds while preserving walkers
- `f2e860ee8058fc26f5a206311d5e45d4f39dbf74` art-p6(p7): capture real panel actions and visible cold cycles
- `56fd7eb9eb8be9a40be0b1f6a533ac557ed98061` art-p6(p7): load carry textures through imported resources
- `11fbba3a89bc59f13fb10f6b9daaf39a55c9736a` art-p6(p7): start live timing only after the Mac window can draw
- `44011f92347dc4176cc115195b1cc55a16ff93ae` art-p6(p7): keep blind lineup and zero counters faithful to gameplay
- `129d0d0674ca39f611a1ef31e5b963947c354b82` art-p6(p7): preserve reproducible natural and intervention captures
- `d6a40d36af69272d0370dd6d182a8571138a967c` art-p6(p8): bind visual judgments to final captured pixels
- `56b92482c071e224305d841e76c78f475e6459e2` art-p6(p9): keep source regressions aligned with booked winter nights
- `7afa014dbca504f965c63d5581cd6220200615c6` art-p6(p9): verify the intervention scene and exact effect lifetime

### 자율 결정 목록

[DECISIONS.md](DECISIONS.md)에 원인과 근거를 기록했다. GOAL의 art branch 계약을 우선하고 원본/user edit을 보존했다. 생성 아이콘 대신 텍스트를 사용하여 이미지 호출 **0/6**이다. 거절 wood 합성은 제외했다. Mac foreground/draw 문제를 capture에서만 해결하고 성능 측정은 자연 frame_post_draw로 유지했다. 잘못된 calibration surface를 한 번 수정한 후 두 번째 blind를 최종으로 삼았다. 측정기 오류는 원본 실패를 보존한 채 수정했으며 임계값을 낮추지 않았다.30MB 예산을 위해 warm-control GIF4개만 SHA 동일하게 외부로 옮겼다.

### 확인된 제한사항

A18: 자연111장 중12장 초과, d24 F2는6명, 아침 최대34명. 이동 중인 주민은 계약에 따라 보정하지 않는다. A21: 추운3장 모두 doorway 주기 식별 실패. A22: 표시 metadata와 실제 읽히는 한기가 다르고 지정 control도 실제 cold 이력이 있다. A24: 바람0.2초/1초 장면을 독립 관찰자가 효과·종료로 식별하지 못했다.

추가로 torso region-map 부재 때문에 §8.1 정확한 인증은0이며 food20개는 proxy만 통과했다. 기존 wood는 방향이 다른 N-view 재사용이고 food torso patch는 거칠다. A19 visible cold body0이므로 N/A다. 코드 리뷰에서 같은 정지 tick에 동일 개입을32회 이상 반복하면 truncated log가 같아져 이후 flash/burst 재시작을 놓칠 수 있는 medium edge case를 발견했다. 명령과 toast는 동작하지만 이 경계는 미수정이며 [code_audit.md](review/code_audit.md)에 남겼다. 장작 선물/부패도 food stock label에 색을 적용하고 wood child 글자에는 self_modulate가 상속되지 않는 표현 한계가 있다.

### Harness 결과

최종 전체 게이트 **45/45 PASS, 종료 코드0**. 입력 소스 HEAD는 `7afa014dbca504f965c63d5581cd6220200615c6`이며 이후 커밋은 문서·증거만 포함한다. `cargo test --workspace`: **2,006 passed / 0 failed**, 133개 결과 블록. clippy `--all-targets -D warnings`, GDScript strict, bridge build, 900-frame smoke와 SCRIPT ERROR 검사 모두 PASS. Python unittest **275개**, Godot 아트 장면 **38개** 모두 PASS. 별도 측정 pytest **12개**도 PASS.

[최종 결과](gates/final/results.tsv), [집계](gates/final/summary.json), [게이트 입력](gates/final/input.json)에 기록했다. 세 번째이자 마지막 전체 게이트에서 통과했다. 앞선 두 실패는 정적 소스 pin의 호환 문제였으며 삭제하거나 완화하지 않고 새 호출·리소스와 보존할 의미를 함께 검사하도록 수정했다. 보충 sim-test 전수 검사도1,517개/120바이너리 PASS다.

예산은 assets/art **1,484,074 bytes / 25 MB**, production6 문서·증거 **27 MB 미만 / 30 MB**다. .import sidecar를 제외한 실제 파일 byte 합으로 검사했다.

원래 P0 실패(ART_LONG13→21 pin), 최종 attempt1 실패(옛 hearth호출 pin), attempt2 실패(옛 scene 리소스 수 pin), 초기 캡처 두 실패, 첫 blind실패는 모두 보존했다. 성공 로그로 덮어쓰지 않았다. 최종 framebuffer 이후 바뀐 실행 렌더 코드는 없고, 정적 테스트와 측정/보고만 추가했다. 원본 evidence는 `/Users/rexxa/work/art-production6-evidence`에 있다.

### 정직한 판단

처음 보는 사람도 자연 추운 밤의 지붕 서리·꺼진 재·푸른 화면과 추위42를 함께 볼 수 있다. A25는3/3장 통과했다. 바구니와 식사도2× blind에서 구분됐다. 그러나 문간 입김, 아침 떨림, 바람의 낙하, 겨울 군중은 충분히 읽히지 않는다. **Part0–9 실행·증거 인계와 시각 목표 완전 달성은 다르며, 이 결과를 전체 시각 수락으로 선언하지 않는다.**
