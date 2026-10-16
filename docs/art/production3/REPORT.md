## 구현 완료 보고

### 구현 의도

캠프를 가리는 바위와 장식을 줄이고 주민의 나이·행동·계절을 실제 시뮬레이션 데이터로 읽을 수 있게 한다. **Part 0–7의 제한된 작업 절차를 수행했으며, 시각 품질은 부분 완료다. A5·A7·A11·A16은 실패로 남긴다.**

### 구현 내용

| Part | 구현·산출물 |
|---|---|
| 0 | 원문 GOAL 보존, Pass4 `5cbcbc4f` 병합, 변경 전 전체 게이트 |
| 1 | 모든 자연물의 캠프 빈터 제외, 지형별 바위 군집, 작은 장식 13종과 희박한 군집 |
| 2 | 생애단계로만 결정하는 머리색, 1초 대기 후 정면, 캠프의 앉기·어린이 hop |
| 3 | 색 기반 region v2·단일 잡티 제거, 366후보/33보드 재검토, 원본 7칸 회복, 생성 4회 후 중단, 실제 운반 뷰 재사용 |
| 4 | 실제 snapshot의 나이·생애단계 정보창, en/ko 4키, 노환 소식·사망 합계·계절 확인 |
| 5 | art94 + art_long7 + 별도 calibration4 창 캡처 |
| 6 | A1–A16, 독립 블라인드 식별, 두 GIF·비교·계절 스트립. 캡처가 숨겨진 주민을 되살리는 오류 수정 후 전체 재촬영 |
| 7 | 최종 코드 게이트, 최신 simulation green 임시 호환 병합, 보고·브랜치·draft PR 발행 |

### 구현 방법

캠프 Chebyshev4 안의 바위·갈대 등 자연물을 모두 제외하고, 장식은 Chebyshev3 안에서 제외한다. 독립 salt를 쓰는 value-noise 순위로 전체 언덕의 6%, 산의 45%에 바위를 배치하고 장식은 초원 최대4%·숲 최대3%로 제한한다. 장식 native 크기는8px, 낙엽6px 이하다.

`life_stages`의 0=어른, 1=아이, 2=노인 계약을 그대로 소비한다. 노인은 hair3/4, 나머지는0–2이므로 첫 노인 등장 여부가 다른 주민의 머리색에 영향을 주지 않는다. render timer가 1초 후 idle 방향을 S로 바꾸고 이동하면 초기화한다. 06–23시 화덕·천막 Chebyshev3 안에서 노인은 앉고 어른 일부는 앉으며 어린이 일부는 hop한다. 의미상 activity나 Rust 상태는 바꾸지 않았다.

색 기반 영역 지도는 동결한 S-idle 참조의 exact RGB 다수 영역, 이어서 ΔE76≤8 최근접 영역, 남는 부분의 제한된 기존 휴리스틱 순서다. 잡티 제거는 원본 native grid에서 동시 판정하며 3% 초과 변경은 탈락한다. alpha·기하·색 충실도·프레임/사이클 guard는 유지했다. 모든 v2 지도 설치는 실제 화면의 기존 색을 훼손하여 채택하지 않았다. 정확한 body+RGBA fingerprint로 승인 지도를 재사용하고 v2 결과를 진단에 남긴다. **설치된 169개 지도 모두 reviewed_map을 사용한다. v2 전면 적용을 달성한 것이 아니다.** 변경 없는 색 프레임154개의 지도 차이는0이다.

운반은 같은 체형의 승인된 실제 carry 방향을 먼저 빌리거나 반전하고, 그것도 없으면 기존 idle+icon을 유지한다. 캡처 도구는 AgentRenderer의 생존/수면 표시 규칙에 한 번만 위임하도록 고쳤다. 이전에는 일반 show 호출이 은퇴한 sprite pool과 천막 수면자를 강제로 표시했다. 실패 회귀 테스트 후 수정했고 실제 d40 재촬영에서 생존 목록에 없는 보이는 ID가0이다.

### 되살린 결과

- 원본 미승인109칸의 물리 조상298개 중 Part1에서 다룬 N3 12개를 제외한286개와 이전 fill80개를 합쳐 **366개**를 새로 처리·검토했다. 162개 frame guard 통과, 101개 잡티 변경3% 초과, 4개 icon canvas 변환 불가다. guard 통과는 승인이 아니다.
- 새 승인 **원본7칸, 이전 fill0칸**: `V_a3_08@3:p3`, `V_a1_13@1:p3`, `V_a1_14@1:p3`, `V_a1_15@1:p3`, `V_a1_16@1:p3`, `V_a3_14@3:p3`, `V_a3_15@3:p3`.
- 새 이미지 생성 **4/12회, 64후보, 22 frame guard 통과, 시각 승인0, 설치0**. 4회 checkpoint의50% 기준에 못 미쳐 중단했다. 모든 원본·거절 사유·검토 보드를 보존했다. [image_calls.json](image_calls.json), [fills](fills/).
- 최종269 drawables = 승인167 + 파생3 + procedural68 + fallback31. suppressed24, unresolved0. fallback31 = idle_icon18 + borrowed_carry_view8 + retained_installed5.
- adult_b carry는 실제 승인2프레임 + 빌린 방향8프레임이다. adult_a10·child8은 idle+icon이 남았다. 새 이름이나 중복 파일로 3프레임 조건을 가장하지 않았다.

### 실제 게임 화면 검증

수치는 [ingame_qa.json](ingame_qa.json), 판정 설명은 [ingame_qa.md](ingame_qa.md). 최종 게시 캡처는 `p6_fixed` 원본의 재촬영이다. 캡처 당시 HEAD는 `bd1f22df`이고 수정 작업물이 있었으므로 clean 최종 HEAD 촬영이라고 주장하지 않는다. 그 캡처 수정과 QA 코드는 Part6 `f2f93835`에 들어 있다. 이후 runtime 코드는 바꾸지 않았다. 최초 실패 결과와 원본도 보존했다.

| 검사 | 판정 | 관측 수치·범위 |
|---|---|---|
| A1 | PASS | gameplay97/97. σ≥.04, 색≥64 유지 |
| A2 | PASS | 4×·6× 혼합 픽셀0 |
| A3 | PASS | draw rect1,702개 오류0; adult S-idle4개 비율 오류0 |
| A4 | PASS | 4×/6× 북·남 겹침4그룹 일치율1.0 |
| A5 | **FAIL** | day2 중앙.0948/최소.0029/측정불가1; winter 중앙.1093/최소.0031/측정불가2. 기준.15/.10/0 |
| A6 | PASS | 낮/밤 world 밝기.313667/.154578, 화덕.256976 |
| A7 | **FAIL** | 이동37샘플, 각 완전한 초에 서로 다른 프레임2개(요구3), 변화율1.0 |
| A8 | PASS | 미선택96장 검사, 오류0 |
| A9 | PASS | gameplay97장 label/panel 검사, 오류0 |
| A10 | PASS | 실제300프레임 중앙6.048ms/p95 15.162ms; Art2 6.111/15.230ms |
| A11 | **FAIL** | 4×18/24=.75<.8, 2×14/24=.58333<.6 |
| A12 | PASS | 23주민/22변형조합, 동일한 이웃0/58 |
| A13 | PASS | 체형3×행동8의 기하 차이24/24, 금지 berry ramp0. 의미 식별 통과는 아님 |
| A14 | PASS | 숲 나무325/1,190=27.311%, 자연물453개 빈터 위반0 |
| A15 | PASS | 배치670개/중심18개 빈터 위반0; 바위 언덕24/397=6.045%, 산72/159=45.283%; 장식 초원72/2,222=3.240%, 숲29/1,190=2.437%; native13종 모두 크기 이내 |
| A16 | **FAIL** | 실제 HUD·계절·사망 합계4/4일 일치, 겨울 B/R비 증가16.767%, d40 노인9명 머리색 통과·missing ID0. 요청서의 d40봄만 불일치: 실제는 여름5일째 |

별도 RAMP는 보정24조합 모두 실제 framebuffer 일치율1.0이다. 겨울 주민 region2/3/4의 측정 가능한 픽셀은 모두1.0이며, 전경 가림으로5명 `[0,4,28,44,42]`은 측정하지 못했다. 이를 모든 주민의 색을 직접 측정한 것으로 확대하지 않는다.

A1 범위 수정 전후: Art2 방식으로101장 전체를 검사하면3장 실패한다. `a_calib_z4` σ.0645/61색, `a_calib_z6` σ.0967/61색, `a_lineup_z2` σ.0397/82색이다. 이번 요청서대로 보정·라인업4장을 제외하면 gameplay97장 실패0이다. 임계값은 그대로이며 제외한 장면은 A2/A3/RAMP/A11에서 계속 검증했다.

블라인드는 정답 파일을 먼저 동결하고 서로 독립된 관찰자에게 각 배율의 이미지와 허용 label만 주었다. 라벨 SHA256 `94fca0335bc21d31f3d7e159bcc69da205b94ae8e1ac5f069ac70ed76f246036`. 재촬영 뒤 라인업/보정4장의 PNG hash가 이전과 같음을 확인했으며 정답을 보고 다시 답하게 하지 않았다. 아래는 행=정답, 열 대신 관찰 label별 건수를 적은 희소 혼동 행렬이다.

| 정답 | 4× 관찰 | 2× 관찰 |
|---|---|---|
| child | child2, drink1 | child1, walk2 |
| talk | talk2 | talk1, build1 |
| gather | gather3 | gather3 |
| build | build1, walk1, carry1 | build1, walk1, sit1 |
| eat | eat3 | eat2, drink1 |
| sit | sit2 | sit1, build1 |
| carry | carry2, drink1 | carry3 |
| drink | drink1, build1 | drink1, walk1 |
| walk | walk2, carry1 | walk1, talk2 |

[blind/results.json](blind/results.json)에 전체 결과가 있다. 라인업 adult_b carry는 실제 `V_b3_02`, adult_a 두 carry는 `V_a2_04` idle icon이다.

요청서 §12의 아홉 인게임 항목:

1. **확인:** 화덕·천막 빈터와 바위 군집을 낮 overview/camp 창에서 확인했다. A14/A15 위반0.
2. **확인:** 장식은 작은 군집이며13종 모두8px 이하, 낙엽6px 이하. 초원3.240%·숲2.437%.
3. **확인:** d40 보이는27명 중 노인9명만 회색/흰색. 첫 노인 전후 다른 주민 불변은 회귀 테스트로 검증했다.
4. **확인:** 대기1초 정면과 timer reset은 Godot 테스트, 낮 캠프의 정면·앉기 모습은 실제 창에서 확인했다. 정지 activity는 계속 idle이다.
5. **미달:** adult_b만 실제 바구니 운반을 방향 재사용한다. adult_a/child의 떠 있는 아이콘18개와 build/drink 혼동이 남았다.
6. **부분 확인:** 실제 여름·가을·겨울·다음 봄을 d10/18/24/32에서 확인했다. 요구된 d40은 7일 계절제에서 여름이다. 요청한 d40 스트립을 유지하고 d32 보충본을 별도로 제공한다. 겨울의 측정 가능한 주민 ramp는 불변, 가려진5명은 미측정.
7. **확인:** 자연 진행22일차01:04 첫 노환, HUD 사망1과 “주민이 노환으로 세상을 떠났다” 소식 및 회색 reason3를 실제 창에서 확인했다. d40 사망14는 모두 노환. 다른 사망 원인은 fixture 검증뿐이다.
8. **부분 확인:** 실제 게임 정보창의 정상 표시 경로로 나이·단계를 열어 d40 노인56세와 레이아웃을 확인했다. OS 마우스 클릭 자체는 자동화하지 않았다. 클릭/inspector 관련 Rust38테스트와 Godot 나이 계약 테스트는 통과했다.
9. **확인:** 최종 창 로그 SCRIPT ERROR/ERROR0, 실제300프레임의 성능은 Art2와 비슷하다. 아래 전체 게이트와 분리해 기록한다.

산출물: [before_after.png](before_after.png), [season_strip.png](season_strip.png)(요청 d10/18/24/40), [four_seasons_actual.png](four_seasons_actual.png)(실제 봄 d32), [village_noon.gif](village_noon.gif), [village_evening.gif](village_evening.gif). 두 GIF는 원본40장·인코딩40프레임, 각각2,759,219/2,952,123bytes다. 검토 시트는 [reprocess/boards](reprocess/boards/), [reprocess/review_index.json](reprocess/review_index.json), [fills](fills/), [decals](decals/)에 보존한다.

### 시뮬레이션 작업과의 호환

Pass4 `5cbcbc4f9bd02bf3f00695bae810368b93fe6300`를 `c18077ea1f5a5e4a66081e78be09583aad221ba6`로 충돌 없이 병합했다. 병합 이후 art 변경은 Rust simulation/bridge를 수정하지 않았다. Rust static-source 테스트 수정도0이므로 새 ledger·mutation 대상도0이다. 노인·노환·계절 관측은 실제 `game_v10`을 자연 진행한 결과다.

작업 중 simulation green이 `2f04737989301b9b753f14053b19e39207411eb2`로 이동했다. 같은 checkout의 detached HEAD에서 임시 병합 `29988ca07f1571bd3be938abace5a63abdc02854`를 만들었고 충돌0, bridge·strict·smoke와 Godot21개를 포함한25검사 모두 통과했다. 실제 창10장을 촬영하고 낮/밤의 봄2일째·캠프·주민 표시를 직접 열어 확인했다. 창 SCRIPT ERROR/ERROR0. [compatibility/result.json](compatibility/result.json), [낮](compatibility/d2_1200_F2.jpg), [밤](compatibility/d2_2200_F2N.jpg).

최신 bridge는 `game_v11`을 사용하므로 이 결과는 day1/2/6의 제한된 호환 확인이다. 최신 simulation으로 장기 A1–A16 전체를 재측정한 결과는 아니다. 임시 병합은 푸시하지 않았고 WIP `c393015a`로 복귀했다. 임시 merge SHA는 provenance를 위해 기록하지만 발행 커밋에는 포함하지 않는다.

### 변경된 파일 목록

커밋별 전체 파일·상태는 [changed_files_by_commit.json.gz](changed_files_by_commit.json.gz)에 보존한다. 주요 소유 파일은 아래와 같다.

| Part | 주요 파일·디렉터리 |
|---|---|
| 0 | `docs/art/production3/GOAL.md`, `TASKS.md`, `DECISIONS.md`, `gates/p0/`; 승인된 Pass4 merge |
| 1 | `scripts/ui/nature_renderer.gd`, `terrain_renderer.gd`, 관련 art 테스트·N3 자산·선택 증거 |
| 2 | AgentActivity·AgentRenderer의 머리색/대기/캠프 표시, 관련 Godot 테스트 |
| 3 | `tools/art_pipeline/regions_v2.py`, `despeckle.py`, `reprocess_p3.py`, `merge_p3_selection.py`, `process_p3_fill.py`, `regions.py`, `install_v4.py`, 테스트·자산·manifest·region registry·리뷰 |
| 4 | `scripts/ui/panels/agent_inspector_panel.gd`, en/ko Fluent/compiled/registry, `test_inspector_age.gd`, `capture_age_consumers.gd` |
| 5 | `scripts/test/capture_framings.gd`, `capture_plans.gd`, `capture_metadata.gd`, `tools/art_pipeline/capture_art.sh`, 창 캡처 증거 |
| 6 | `ingame_qa*.py`, `report_artifacts.py`, `reprocess_p3.py` 동결 보호, 관련 테스트, 캡처 표시 위임 수정, 재촬영·QA·리뷰·GIF |
| 7 | 최종 gate·compatibility·이 보고서·결정/작업/파일 기록 |

| Part | SHA | 기록 |
|---|---|---|
| 0 | `a063d7d87456ea3c258cf3c6e38947dc59aac606` | art-p3(p0): preserve the camp and villager completion contract |
| 0 | `c18077ea1f5a5e4a66081e78be09583aad221ba6` | art-p3(p0): bring in simulation pass 4 |
| 0 | `0c6ff0ce28419c154e1795dacd2e617dfabdbe8d` | art-p3(p0): establish a verified Pass 4 baseline |
| 1 | `fa21393fdcd3d8810e5354e5d0b2136d5edaef67` | art-p3(p1): restore readable space around the camp |
| 2 | `9efc3b20e558da83401e2b5fd33b71d7fe93097d` | art-p3(p2): make age and idle camp poses readable |
| 3 | `8017a4e97a8e3d8b82bdf72fde76746cadaa38ce` | art-p3(p3): recover reviewed poses without degrading approved body colors |
| 4 | `3dc37c8bb9427b5ff35ec59ee64fcfaf9c665507` | art-p3(p4): show real life stage and age beside observed old-age events |
| 5 | `bd1f22dfaf68b6822deb05d0fe967e4a7d5ecf5d` | art-p3(p5): capture real seasons and elders through day forty |
| 6 | `f2f93835207bf7587df27dda27d70dbdd7cf04b3` | art-p3(p6): measure visual limits with faithful capture visibility |
| 7 | `c393015ac31ba0994f86f86240b5e928ceffba6f` | art-p3(p7): preserve full gate before isolated compatibility verification |
| 7 보고서·호환 증거 | 이 파일을 추가한 commit (`git log -1 --format=%H -- docs/art/production3/REPORT.md`) | 전체 SHA·원격 일치·PR 갱신 결과는 최종 응답과 외부 publication.json |

### 자율 결정 목록

[DECISIONS.md](DECISIONS.md)에 이유와 실패 이력을 보존했다. 핵심은 전체 지형을 밀도 분모로 사용, 고정 승인 참조 유지, 원본/기존 fill 모두 재검토, 생성4회에서 중단, 모든 v2 지도 설치 대신 정확한 fingerprint의 승인 지도 보존, 없는 실제 carry를 승인한 것으로 꾸미지 않기, d40 날짜를 바꾸지 않고 d32 보충, 캡처 표시 오류를 실제 runtime 규칙으로 고친 뒤 재촬영이다. 새 의존성을 설치하지 않았고 CPU/test threads6 및 모든 창 실행의 공유 lock을 지켰다. 다른 checkout이나 타인 프로세스를 조작하지 않았다.

### 확인된 제한사항

시각 실패4개(A5/A7/A11/A16)를 그대로 유지한다. A16은 구현 시계 불일치가 아니라 요청서의 d40봄 가정 불일치다. A13의 기하 차이는 망치·마시기의 의미 식별을 보장하지 않는다. 승인 지도169개를 보존하므로 영역 v2의 일반적인 비정면 pose 해결은 미완성이다. 겨울 ramp5명 미측정, 실제 mouse click 미구동, 다른 사망 원인 미관측을 구분한다. 전용 ruff/basedpyright는 설치되어 있지 않아 실행하지 않았으며 기존 strict/Python compile·unit 게이트로 검증했다.

발행 보고서 작성 시 디스크 전체 합계: assets/art **1,894,882bytes /25,000,000**, docs/art/production3 **39,930,528bytes /40,000,000**. GIF 둘 모두8,000,000bytes 이하다. 최종 재측정값은 외부 publication.json에 남긴다.

### Harness 결과

소스 `f2f93835207bf7587df27dda27d70dbdd7cf04b3`의 전체 게이트 **28/28 exit0**. [gates/final/results.tsv](gates/final/results.tsv)에 개별 종료 코드와 시간이 있다.

| 항목 | 결과 |
|---|---|
| cargo test workspace | **1,881 passed, 0 failed**, 120 result suites, jobs/threads6 |
| Clippy | workspace all-targets, `-D warnings`, exit0 |
| strict | GDScript26파일·FFI15호출, 모두 통과 |
| bridge | sim-bridge build exit0 |
| smoke | headless900프레임, SCRIPT ERROR/ERROR0 |
| Python | **177 tests, OK** |
| Godot art | **21장면/스크립트 모두 exit0** |

초기 P0 게이트도25/25, Rust1,881·Python129·Godot18로 통과했다. 최종 발행 커밋의 전체 게이트는 tracked 파일을 더럽히지 않도록 `.harness/evidence/art-production3/final-head-gate/`로 반복한다. 실제 발행 결과·양쪽 `ls-remote`·PR draft 상태는 `.harness/evidence/art-production3/publication.json`과 최종 응답에 기록한다. 이 문서가 자기 자신의 commit hash를 포함하는 순환을 피한다.

### 정직한 판단

캠프와 땅은 훨씬 차분하고 노인·나이·노환·계절은 실제 데이터로 읽힌다. 그러나 처음 보는 사람이 **완성도 있는 픽셀 아트 게임**이라고 판단하기에는 주민 대비, 건설/마시기 식별, 떠 있는 운반 아이콘과 2프레임 운반이 부족하다. 새 생성의 실패를 숨기지 않았으며, 코드 green은 시각 품질 승인이나 이 draft PR의 merge 승인이 아니다.
