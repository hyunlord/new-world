## 구현 완료 보고 — M-UX2.1

### D22 최종 수용·게시 기록 — 00:50KST 오너 지시

**M-UX2.1 수용·게시: STRICT_PASS.** 전체 gate 소스와 먼저 게시한 제품 소스는 [ff6af820a](https://github.com/hyunlord/new-world/commit/ff6af820aa1fb4e2d894fa7e4e814bdf897a7868)다. 이 뒤의 문서 전용 후속은 **NOT_FULL_GATED**이며 새 전체 gate 실행을 주장하지 않는다.

근거: [gate](/Users/rexxa/work/mux21-data/resume-20261029/continuation-0050/full-gate/gate.txt) · [terminal](/Users/rexxa/work/mux21-data/resume-20261029/continuation-0050/full-gate-terminal.json) · [수용 판정](/Users/rexxa/work/mux21-data/resume-20261029/continuation-0050/candidate-waiver-audit.json) · [런타임 동일성](/Users/rexxa/work/mux21-data/resume-20261029/continuation-0050/docs-runtime-equality.json) · [원격 확인](/Users/rexxa/work/mux21-data/resume-20261029/continuation-0050/source-publication.json). raw gate exit=0; raw 결과를 수용 판정으로 덮어쓰지 않았다.

원래40단계 네 실패 중 폭 초과는 EN 제목의 최소 폭 때문에342local×1.25=427.5screen px가 되어 재현됐다. 제목 줄바꿈과 실제 헤더 높이 반영으로420px·반 화면 높이 상한을 유지했다. 나머지 세 열기 실패(top why tool/scale1.0/scale1.25)는 수정 전 EN 재현에서도 통과했으므로 원인 해결을 주장하지 않는다. 실패 당시 EN·열린 연대기·빈 기록 토스트는 보존하며 외부 입력/카메라 상태 가설은 미확정이다. 원래42검사는 byte-identical, 추가 EN/KO53검사씩과 원래 독립42검사 통과 근거를 별도로 기록한다. 성능 개선이 입증되지 않은 Topbar 변경을 되돌렸고 #657 배치 수정은 유지했다.

집중 검증: [입력/제목/닫기/폭/높이](/Users/rexxa/work/mux21-data/resume-20261029/continuation-0050/why-product-verification.json) · [#657 정보창 비겹침](/Users/rexxa/work/mux21-data/resume-20261029/continuation-0050/why-code-visual-review.json). 이전 FPS7/7을 보존하며 이 문서 추가가 새로운 FPS 측정이라는 뜻은 아니다.

00:50KST 오너 예산은 목표3시간/하드4시간, 03:50/04:50KST(18:50/19:50UTC)다.

00:50KST 오너 조건: 이번 패스에 한해 zoom별 완전·유효한 평가 창에서 50ms 초과 프레임은 최대 2개, 최대 프레임 시간은 60ms 이하이고 캡처 오류는 0이어야 하며, 다른 45단계는 모두 통과해야 한다. 기간·순서·틱 진행·원시 해시·캡처 및 호스트 유효성 검사는 유지한다. 원래 strict FAIL과 raw exit는 보존한다.

이번 실행 waiver 사용 여부: **미사용 — STRICT_PASS**. raw 결과와 별도 수용 판정이다.

| 최종 zoom | 평가 시간 | 최대 ms | >50ms 프레임 목록 (elapsed / interval / tick / ticks_advanced) | strict 결과 |
|---|---:|---:|---|---|
| 1× | 300.000148s | 49.313 | 없음 (0개) | PASS |
| ½× | 300.007146s | 46.243 | 없음 (0개) | PASS |

각 zoom의 >33ms/분은 `[1,0,0,0,0]`. 캡처 오류0, raw complete=true, wrapper exit0. 시작 가능 구간과 전체 캡처 관측 모두 PASS; 최대 load1=4.572754, 다른 Godot/빌드 없음. 관측 간격 최대1.104s이며 정확한 arm 시작 시각이나 subsecond 활동 부재를 주장하지 않는다.

| 소스 / topbar 상태 | 1× >50ms 수 / 최대 | ½× >50ms 수 / 최대 |
|---|---|---|
| e47ee10d8 / 원래 topbar | 1 / 50.018ms | 0 / 47.588ms |
| 99a35a8c2 / theme 변경 | 1 / 50.768ms | 0 / 47.121ms |
| ff6af820 / theme 변경 되돌림 | 0 / 49.313ms | 0 / 46.243ms |

이전 e47 초과 프레임: elapsed34.534225s,50.018ms,tick361,ticks_advanced=1. 이전99 초과 프레임: elapsed34.537230s,50.768ms,tick361,ticks_advanced=1(실제 sim tick 1개 진행). 99에서는40단계 실패 때문에 waiver를 적용하지 못했다. theme 변경은 초과 프레임 수를 줄이지 못해 도움이 입증되지 않았고 #657에 필요하지 않아 되돌렸다. 이번0개 결과는 단일 관측이며 되돌림의 인과적 개선이나 stall 해결·재현성을 입증하지 않는다.

위 초과 프레임 수와 최대 시간은 각 실행의 관측값이다. 중복 무효화 제거가 원래 stall의 원인이나 시간 개선을 증명했다고 주장하지 않는다.

필수 FPS7/7과 기존24개 요청 화면 근거는 유지하며 D22 새 화면은 위 집중 검증에 연결했다. 원래 strict stall 결과와 모든 실패 영수증은 보존했다. ISS030/#515와 자연 재굽기10.493ms/4ms 목표 미달 ISS347/#658은 OPEN이다. 이번 stall 예외를 자연 재굽기나 M-CORE side64에 확대하지 않는다.

[ISS-343/#638 폐쇄](https://github.com/hyunlord/new-world/issues/638#issuecomment-6476850413) · [ISS-344/#655 폐쇄](https://github.com/hyunlord/new-world/issues/655#issuecomment-6476851601) · [ISS-345/#656 폐쇄](https://github.com/hyunlord/new-world/issues/656#issuecomment-6476852746) · [ISS-346/#657 폐쇄](https://github.com/hyunlord/new-world/issues/657#issuecomment-6476854052). 폐쇄 후 재조회로 확인했다. 원격 wip/main에 먼저 게시한 SHA는 모두 `ff6af820aa1fb4e2d894fa7e4e814bdf897a7868`다. 이후 문서 전용 게시 SHA는 부모 실행의 최종 원격 재조회/최종 답변에 따로 기록하며, 이 문서는 자기 커밋 SHA나 자기 전체 gate 통과를 주장하지 않는다.

아래 D18/D20/D21의 미게시·실패·대기 문구와 이전 결과는 당시 기록이며 위 판정과 구분해 보존한다.

### 최종 전체 gate — 46단계 각각의 원시 결과와 수용 판정

| 단계 | 검사 | raw exit | 결과 / 수용 |
|---:|---|---:|---|
| 1 | fmt | 0 | PASS / strict |
| 2 | bridge_build | 0 | PASS / strict |
| 3 | godot_import | 0 | PASS / strict |
| 4 | test_debug | 0 | PASS / strict |
| 5 | test_release | 0 | PASS / strict |
| 6 | clippy | 0 | PASS / strict |
| 7 | forbidden_math | 0 | PASS / strict |
| 8 | gdscript_strict | 0 | PASS / strict |
| 9 | smoke | 0 | PASS / strict |
| 10 | identity | 0 | PASS / strict |
| 11 | determinism | 0 | PASS / strict |
| 12 | perf | 0 | PASS / strict |
| 13 | bridge_api | 0 | PASS / strict |
| 14 | docs | 0 | PASS / strict |
| 15 | inventory_closure | 0 | PASS / strict |
| 16 | python_unittest | 0 | PASS / strict |
| 17 | m1_tests | 0 | PASS / strict |
| 18 | colony_determinism | 0 | PASS / strict |
| 19 | replay | 0 | PASS / strict |
| 20 | uid_check | 0 | PASS / strict |
| 21 | play_check | 0 | PASS / strict |
| 22 | schema_docs | 0 | PASS / strict |
| 23 | renderer_agreement | 0 | PASS / strict |
| 24 | save_roundtrip | 0 | PASS / strict |
| 25 | bench | 0 | PASS / strict |
| 26 | soak | 0 | PASS / strict |
| 27 | nav_bench | 0 | PASS / strict |
| 28 | knowledge | 0 | PASS / strict |
| 29 | story | 0 | PASS / strict |
| 30 | neighbours | 0 | PASS / strict |
| 31 | art_manifest | 0 | PASS / strict |
| 32 | resolver_parity | 0 | PASS / strict |
| 33 | extensibility_grep | 0 | PASS / strict |
| 34 | mart2_render | 0 | PASS / strict |
| 35 | steward | 0 | PASS / strict |
| 36 | data_only_ext | 0 | PASS / strict |
| 37 | mux2_defence | 0 | PASS / strict |
| 38 | mux2_oracles | 0 | PASS / strict |
| 39 | mux2_visibility | 0 | PASS / strict |
| 40 | mux2_why | 0 | PASS / strict |
| 41 | mux2_stall_receipt | 0 | PASS / strict |
| 42 | baseline_isolation | 0 | PASS / strict |
| 43 | mind_data_only | 0 | PASS / strict |
| 44 | core_shadow | 0 | PASS / strict |
| 45 | core_evaluation | 0 | PASS / strict |
| 46 | mux21_regressions | 0 | PASS / strict |

이하 이전 gate 기록은 당시 이력이며 위 최종 실행과 구분한다.

### D22 00:50KST 후속 — 제품 수정 후 전체 gate 대기

왜? 제목이 영어1.25배에서 최소 폭427.5px를 강제하던 문제를 줄바꿈으로 수정했다. 실제 제목·테마 높이를 반영해 본문 스크롤 영역을 제한하며420px 폭·화면 절반 높이 기준을 유지한다. 이 폭 문제는 e47 topbar 복원 상태에서도 재현됐다. 세 열기 실패는 수정 전 재현에서 이미 통과했으므로 입력 원인을 확정하거나 카메라를 임의 수정하지 않았다. 기존42검사는 그대로이며 단독42/42, 추가 EN/KO 각53/53이 통과했다.

효과가 입증되지 않은 topbar 테마 최적화와 전용 검사를 오너 지시로 되돌렸다. #657 비겹침·마커·Rust는 유지한다. 최종 커밋 전체 gate와 새 stall 영수증으로 판정하며, 아래 D21 실패는 당시 이력으로 보존한다. 작성 시점 게시·이슈 폐쇄는 PENDING이다.

### D21 최종 중단 — 21:55KST 오너 조건 미충족

**BLOCKED / RED / NOT_PUBLISHED — 최종 전체 gate 44/46 PASS, 40·41단계 FAIL.** 전체 gate 소스는 `99a35a8c2a28b20858f40833a04355399d4f83a6`이며 2026-10-28 21:55:49–23:44:48 KST 실행의 최종 exit는1이다. 이후 로컬 문서 전용 후속은 **NOT_FULL_GATED / NOT_PUBLISHED**다.

21:55KST 오너 지시의 예외는 (a) 다른45단계 전부 PASS, (b) 각 zoom의 평가 창에서50ms 초과 최대2프레임 및 최대60ms 이하, (c) 완전·유효한 영수증과 캡처 오류0을 모두 만족해야 한다. 이번40단계 실패로 (a)가 성립하지 않아 **NO WAIVER**다. 수정·재실행·게시·이슈 폐쇄 없이 중단했다. 목표4시간/하드5시간, 보수적 마감2026-10-29 02:52:54KST(17:52:54UTC)를 사용했으며 deadline 강제 종료가 아니다. [오너 지시 초안](/Users/rexxa/work/mux21-data/resume-20261028/continuation-2155/D21-owner-note-draft.md) · [실행 예산](/Users/rexxa/work/mux21-data/resume-20261028/continuation-2155/plan.json).

40단계 `mux2_why`:42검사 중4실패(raw exit1). 현재 캡처의 EN·열린 연대기 상태와1.25배 폭427.5px를 관측했으나 외부 입력 인과는 미확정 가설이며 실패를 무효화하지 않는다. [실패 진단](/Users/rexxa/work/mux21-data/resume-20261028/continuation-2155/why-failure-diagnosis.json). 41단계는1×50.768ms로 원래 strict 기준 FAIL, ½×47.121ms PASS; 원본 raw FAIL과 wrapper exit1을 보존한다.

| zoom | 이전 e47 >50ms 개수 / 최대ms | 이번99 >50ms 개수 / 최대ms | 이번 초과 프레임 elapsed초 / intervalms / tick / ticks_advanced |
|---|---|---|---|
| 1× | 1 / 50.018 | 1 / 50.768 | 34.53723 / 50.768 / 361 / 1 |
| ½× | 0 / 47.588 | 0 / 47.121 | 없음 |

HudTopbar 중복 테마 무효화 방지 후1× 초과 개수는1→1, 최대는50.018→50.768ms다. ½×는0→0, 47.588→47.121ms다. 이 관측으로 stall 개선이나 원인 해결을 입증하지 못했다. [전후 비교](/Users/rexxa/work/mux21-data/resume-20261028/continuation-2155/e47-vs99-stall-comparison.json).

**입장 부하 감사 PASS(보수적 시작 구간 관측).** 두 순차 arm의 가능한 시작 구간 전체를315회 관측으로 둘러싸 확인했으며1분 최대 부하4.4741<8, 다른 Godot·cargo/rustc 없음이다. 정확한 arm별 UTC를 추정하지 않았다. 후반14:41:57UTC의1분 부하11.6421도 보존한다. 완전한 캡처·오류0과 시작 조건은 확인되지만 실행 내내 저부하였다는 뜻은 아니다. 실행 전체가 조용했다고 주장하지 않는다. 후반 부하 상승 원본을 보존하며 시작 시점 조건과 실행 중 안정성을 구분한다. [시작 구간 감사](/Users/rexxa/work/mux21-data/resume-20261028/continuation-2155/step41-start-validity.json) · [실행 중 관측](/Users/rexxa/work/mux21-data/resume-20261028/continuation-2155/step41-validity.json).

[46단계 원본](/Users/rexxa/work/mux21-data/resume-20261028/continuation-2155/full-gate/gate.txt) · [종료 영수증](/Users/rexxa/work/mux21-data/resume-20261028/continuation-2155/full-gate-terminal.json) · [예외 불가 판정](/Users/rexxa/work/mux21-data/resume-20261028/continuation-2155/99-waiver-audit.json). 필수 FPS7/7, 최신24개 요청 화면과 이전 실패 근거는 유지한다. 자연 재굽기10.493ms/4ms 목표 미달도 유지한다.

닫은 이슈 없음. ISS030/#515, ISS343/#638, ISS344/#655, ISS345/#656, ISS346/#657, ISS347/#658 모두 OPEN이다. 원장347행/fixed203/open111/deferred31/won't fix2 유지; 게시·폐쇄 링크를 만들지 않는다.

### 최종 전체 gate — 46단계 원시 결과

| 단계 | 검사 | raw exit | 결과 | 소요 초 | 시작 KST |
|---:|---|---:|---|---:|---|
| 1 | fmt | 0 | PASS | 1 | 21:55:49 |
| 2 | bridge_build | 0 | PASS | 2 | 21:55:50 |
| 3 | godot_import | 0 | PASS | 7 | 21:55:52 |
| 4 | test_debug | 0 | PASS | 2178 | 21:55:59 |
| 5 | test_release | 0 | PASS | 121 | 22:32:17 |
| 6 | clippy | 0 | PASS | 1 | 22:34:18 |
| 7 | forbidden_math | 0 | PASS | 0 | 22:34:19 |
| 8 | gdscript_strict | 0 | PASS | 17 | 22:34:19 |
| 9 | smoke | 0 | PASS | 27 | 22:34:36 |
| 10 | identity | 0 | PASS | 545 | 22:35:03 |
| 11 | determinism | 0 | PASS | 129 | 22:44:08 |
| 12 | perf | 0 | PASS | 576 | 22:46:17 |
| 13 | bridge_api | 0 | PASS | 0 | 22:55:53 |
| 14 | docs | 0 | PASS | 2 | 22:55:53 |
| 15 | inventory_closure | 0 | PASS | 0 | 22:55:55 |
| 16 | python_unittest | 0 | PASS | 33 | 22:55:55 |
| 17 | m1_tests | 0 | PASS | 164 | 22:56:28 |
| 18 | colony_determinism | 0 | PASS | 4 | 22:59:12 |
| 19 | replay | 0 | PASS | 2 | 22:59:16 |
| 20 | uid_check | 0 | PASS | 0 | 22:59:18 |
| 21 | play_check | 0 | PASS | 185 | 22:59:18 |
| 22 | schema_docs | 0 | PASS | 0 | 23:02:24 |
| 23 | renderer_agreement | 0 | PASS | 8 | 23:02:24 |
| 24 | save_roundtrip | 0 | PASS | 64 | 23:02:32 |
| 25 | bench | 0 | PASS | 1245 | 23:03:36 |
| 26 | soak | 0 | PASS | 9 | 23:24:21 |
| 27 | nav_bench | 0 | PASS | 11 | 23:24:30 |
| 28 | knowledge | 0 | PASS | 55 | 23:24:41 |
| 29 | story | 0 | PASS | 17 | 23:25:36 |
| 30 | neighbours | 0 | PASS | 25 | 23:25:53 |
| 31 | art_manifest | 0 | PASS | 0 | 23:26:18 |
| 32 | resolver_parity | 0 | PASS | 0 | 23:26:18 |
| 33 | extensibility_grep | 0 | PASS | 10 | 23:26:18 |
| 34 | mart2_render | 0 | PASS | 20 | 23:26:28 |
| 35 | steward | 0 | PASS | 10 | 23:26:48 |
| 36 | data_only_ext | 0 | PASS | 7 | 23:26:58 |
| 37 | mux2_defence | 0 | PASS | 216 | 23:27:06 |
| 38 | mux2_oracles | 0 | PASS | 8 | 23:30:42 |
| 39 | mux2_visibility | 0 | PASS | 146 | 23:30:50 |
| 40 | mux2_why | 1 | FAIL | 13 | 23:33:16 |
| 41 | mux2_stall_receipt | 1 | FAIL | 638 | 23:33:29 |
| 42 | baseline_isolation | 0 | PASS | 0 | 23:27:05 |
| 43 | mind_data_only | 0 | PASS | 1 | 23:27:05 |
| 44 | core_shadow | 0 | PASS | 30 | 23:44:07 |
| 45 | core_evaluation | 0 | PASS | 6 | 23:44:37 |
| 46 | mux21_regressions | 0 | PASS | 5 | 23:44:43 |

**이하 이전 통과·실패·대기·미게시 문구는 당시 이력이며 위 최종 결과와 구분해 보존한다.**

### 20시 후속: 45/46 결과 보존, UI 테마 수정 후 전체 재검증

`e47ee10d8` 전체46단계 중45통과. 41단계1×의 tick361 프레임50.018ms가 엄격한50ms 기준을 넘었다(½×47.588ms PASS). 부하·프로세스 조건은 유효하며 실패를 무효 실행으로 버리지 않는다. GPU8장면/76인물, Python409시험, 새BASE 성능33쌍, shadow/evaluation/회귀는 통과했다. [전체 게이트](/Users/rexxa/work/mux21-data/resume-20261028/continuation-1615/full-gate/gate.txt), [지연 유효성 및 실패](/Users/rexxa/work/mux21-data/resume-20261028/continuation-1615/step41-validity.json).

후속 후보는 HudTopbar의 동일 색상 테마 재지정만 막는다. 실제 theme_changed 시험이 변경 전5실패/20개에서 변경 후20/20통과했다. native StewardSystem33.06ms가 실패 프레임의 주된 비용이므로 이 UI 수정이 지연 기준을 해결했다고 아직 주장하지 않는다. 마커·Rust 불변, 기존 FPS7/7 유지. 요청 정보창·왜?·연대기·방어를 `continuation-1615/topbar-retry/`에 재캡처했다. 이 커밋에서 전체 게이트를 다시 실행하며 작성 시점 결과·게시·이슈폐쇄는 PENDING이다. 이전 실패와 최초 캡처는 보존한다.

### 2026-10-28 16:15 KST 지시 후속 — 전체 게이트 재검증 대기

이하 RED GATE 본문은 이전 시도 기록으로 보존한다. 새 지시는 일반 red gate를 수정·전체 재실행하도록 정정했다(D19). `9934ef0f2`에서 독립 추출되는 performance_gate를 원래 순수 함수로 복원하고 UI 검증을 실제 호출부로 옮겼다. 봉인 환경에서 새 successor27시험과 기존 실패 시험1개가 통과했다. step12/27 측정 입장 설정은 자식에만 적용하며 P7 수용 예외를 켜지 않는다.

ISS-346은 실제 정보창 경계를 경고/소집 영역이 침범하지 않도록 UI 배치만 수정했다. 마커 경로·Rust는 바뀌지 않았으며 오너 지시에 따라 기존 FPS7/7을 유지한다. 일/배고픔/도주 정보창6장·왜?6장, 연대기2장, 방어4장을 재캡처했고 오류0이다. ¼× 방어·선택 이동6장도 오류0이며 토스트와 소집 링을 유지한다. 독립 시각 검토24/24 PASS로 두 배율의 경고/소집과 정보창 제목·초상 비겹침, 이유 가독성과 링을 확인했다. [시각 검토](/Users/rexxa/work/mux21-data/resume-20261028/continuation-1615/visual-review.json). [재캡처 실행](/Users/rexxa/work/mux21-data/resume-20261028/continuation-1615/capture-runs.json), [¼× 근거](/Users/rexxa/work/mux21-data/resume-20261028/continuation-1615/capture-quarter-defence-a1/evidence.json).

| 재캡처 | KO1.0 | KO1.25 |
|---|---|---|
| 일 | [정보창](/Users/rexxa/work/mux21-data/resume-20261028/continuation-1615/capture-work-a1/work-1.0-person.png) | [정보창](/Users/rexxa/work/mux21-data/resume-20261028/continuation-1615/capture-work-a1/work-1.25-person.png) |
| 배고픔 | [정보창](/Users/rexxa/work/mux21-data/resume-20261028/continuation-1615/capture-hunger-a1/hunger-1.0-person.png) | [정보창](/Users/rexxa/work/mux21-data/resume-20261028/continuation-1615/capture-hunger-a1/hunger-1.25-person.png) |
| 도주 | [정보창](/Users/rexxa/work/mux21-data/resume-20261028/continuation-1615/capture-flee-a1/flee-1.0-person.png) | [정보창](/Users/rexxa/work/mux21-data/resume-20261028/continuation-1615/capture-flee-a1/flee-1.25-person.png) |
| 연대기 | [화면](/Users/rexxa/work/mux21-data/resume-20261028/continuation-1615/capture-chronicle-a1/chronicle-day3-1.0.png) | [화면](/Users/rexxa/work/mux21-data/resume-20261028/continuation-1615/capture-chronicle-a1/chronicle-day3-1.25.png) |
| 방어·선택·정보창 | [화면](/Users/rexxa/work/mux21-data/resume-20261028/continuation-1615/capture-quarter-defence-a1/defence-quarter-selected-1.0.png) | [화면](/Users/rexxa/work/mux21-data/resume-20261028/continuation-1615/capture-quarter-defence-a1/defence-quarter-selected-1.25.png) |

이 문서를 포함한 최종 커밋을 새로 봉인해 step1부터 전체 게이트를 실행한다. 작성 시점 전체 게이트·GPU 수용·게시는 PENDING이며 성공을 미리 선언하지 않는다. 최신 실행 결과는 외부 `continuation-1615/full-gate/`와 최종 Korean addendum에 귀속한다. ISS343–346은 게시 전 OPEN, 자연 재굽기10.493ms의 ISS347/#658과 ISS030/#515는 계속 OPEN이다.

### 이전 시도: 6ed4e1996의 실패 기록

**BLOCKED · RED GATE · NOT_PUBLISHED.** 오너 지정 제목이며 전체 완료 선언이 아니다. 필수 FPS7/7과 요청 화면 검증은 완료했지만, 전체 gate의16번 Python 시험이 실패해 stop-on-red 규칙으로 중단했다. 수정·재시험·재게이트·게시를 하지 않았다. 측정 소스는 `b0401ca2e40de0ee972d451b3c4c1ecf0c1fa816`, 실패 gate 소스는 `6ed4e19966ec368812441c4f12a74fca22ae251b`다. 외부 최종 측정 감사는 두 HEAD의 런타임 동일성을 확인한다. 이 실패 종료 문서 후속 HEAD는 전체 gate를 거치지 않았다.

2026-10-28 재개 기준은 게시 M-CORE `3814828c06d1a7a211b13b54368e4e0a488cc0e3`, 브랜치는 `v8/mux2-1b`다. 원래 브랜치와 [2026-10-26 차단 보고](MUX21_BLOCKED_20261026.md)는 보존했다. [최종 측정 감사](/Users/rexxa/work/mux21-data/resume-20261028/final-measurement-audit.json).

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

최종 HEAD의 GPU 낮/밤140px 대비·선택/징집 링 수용 재검사는 **NOT_RUN**이다. red gate 중단으로 도달하지 못했다. 표식 회귀 도구의 실제 창 검사와 개별 방어 캡처는 전체 GPU 수용 검사를 대체하지 않는다. 자연 ¼× 재굽기는 seed0/day3 저장에서30초 준비+60초 관측, 시작/종료1분 부하2.81→2.70의 유효 실행이다. 측정 구간의17재굽기/16프레임을 `(reset_epoch, frame)`별로 합산한 최대는 **10.493ms/프레임**, 새 굽기는0건이다. **4ms 목표 미달**을 그대로 남기며 FPS 통과로 덮지 않는다. 준비 구간은 합계에서 제외했다. [프레임별 원시 집계](/Users/rexxa/work/mux21-data/resume-20261028/acceptance-summary.json).

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

닫은 이슈 없음. ISS-343 [#638](https://github.com/hyunlord/new-world/issues/638) OPEN 유지. 새 ISS-344 방어 토스트 [#655](https://github.com/hyunlord/new-world/issues/655), ISS-345 후속 UI gate 어댑터 [#656](https://github.com/hyunlord/new-world/issues/656)는 수정 커밋이 로컬 미게시이며 red gate 실패·미게시로 **OPEN**이다. 어댑터 시험 실패는 기존 ISS-345/#656 범위에 기록하며 같은 결함의 새 이슈를 만들지 않는다.

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
| p13 `6ed4e1996` | 자연 재굽기 미달 ISS-347 등록·보고·결정 기록; 실패 전체 gate 소스 |
| p14 (이 보고서를 마감하는 로컬 문서 커밋) | 본 보고·결정 기록·HISTORY/ROADMAP/CHANGELOG/README·이슈 기록; 코드 수정·재시험·재게이트 없음 |

### 확인된 제한사항

전체 `rust/` tree는 게시 BASE381 및 기존 측정826과 동일하다(`c942d93fcd52bea5d7bfed2fb7e9c82de87b7a99`). 이번 gate의 identity 단계는228,748값과64 chronicle/19,683항목 동일성을 통과했지만, 소스 동일성과 이 단계의 통과를 미실행 후속 colony fixture 전체의 통과로 확대하지 않는다. debug/release bridge 재빌드 완료 근거와 실제 캡처의 bridge 해시는 외부 증거에 보존한다.

도주1.25의 늑대 경고가 인물 이름을 덮지만 이유와 왜?는 가리지 않는다(ISS-346/#657 OPEN·미수정). 일부1.0 캡처의 작은 지도는 빈 화면이다. UI 설정1.25는 기존 화면 맞춤 상한으로 실제 약1.2233이 될 수 있다. 물리 트랙패드/오너 플레이 수용은 이 캡처가 대신하지 않는다.

필수 FPS7막대는 통과했지만 전체 gate는16번에서 실패했다. 최종 GPU 재검사와18–46단계는 NOT_RUN,17번은 중단됐다. 자연 재굽기10.493ms는4ms 목표 미달이며 원격 게시·이슈 폐쇄를 수행하지 않았다. 이전 혼잡 실행과 잘못된 캡처·보간 후보 실패는 보존한다. 역사 M-CORE side64 FAIL은 어댑터가 지우지 않으며 새 비교는 예외 없이 통과해야 한다. 원래 워크트리와 외부 도구 `.gitignore`/`.ignore` 변경은 보존하고 임의 제거하지 않는다.

### Gate 결과

**전체 gate: FAIL / STOPPED_ON_RED. 게시: NOT_PUBLISHED.** clean 소스 `6ed4e19966ec368812441c4f12a74fca22ae251b`에서2026-10-28 12:57:29KST 시작했다. [gate 원문](/Users/rexxa/work/mux21-data/resume-20261028/full-gate-1/gate.txt), [terminal](/Users/rexxa/work/mux21-data/resume-20261028/full-gate-terminal.json), [중단 근거](/Users/rexxa/work/mux21-data/resume-20261028/red-gate-stop.json).

| 단계 | 결과 |
|---|---|
| 1–15 | 명령 exit0. debug2,595통과/release2,594통과·각8 ignored, 실패0. identity228,748값·64 chronicle/19,683항목 동일. docs·inventory도 이 gate 소스에서 통과 |
| 12 legacy perf | 명령 exit0이나 반복별 입장 부하/독점 실행 근거가 없어 이번 성능 수용 결과로 사용하지 않음 |
| 16 python_unittest | **exit1 / ERROR1**,405시험. `test_core_shadow_freeze.FreezeTests.test_gate_reports_only_core_ratio_misses_and_rejects_other_failures`가 추출한 `performance_gate` line105에서 `NameError: name 'root' is not defined` |
| 17 | runner가 시작했으나 red 확인 후 중단. 완료·통과 주장 없음 |
| 18–46 | **NOT_RUN**. 새 step25 BASE 성능 비교와 마지막 GPU 재확인도 실행되지 않음 |

원인은 이번 패스가 추가한 어댑터 통합 경로다. 기존 동결 시험은 순수 `performance_gate` 함수만 AST로 추출해 빈 namespace에서 실행하고 `CORE_P7_RUN`만 지운다. 상속된 `CORE_UI_CONFIG`가 새 분기를 켜면서 실제 PYEVAL 문맥에만 있는 `root`/`output`을 참조했다. 기존 시험 자체의 선행 결함으로 돌리지 않는다. [ISS-345/#656 실패 댓글](https://github.com/hyunlord/new-world/issues/656#issuecomment-6462768982)에 기록했다. 이후 별도 승인된 수정에서는 UI dispatch를 실제 호출부로 옮기고 동결 함수/시험을 유지하는 방안을 검토할 수 있으나 이번 종료에서 수정하지 않았다.

오너의 red gate 중단 규칙에 따라 소유 process group27094만 SIGTERM했다. terminal actual/wrapper exit는 모두-15, hard_deadline_reached=false이며 소유 프로세스 잔존 없음이 확인됐다. deadline 초과나 성공 종료로 바꾸지 않는다. 실패 뒤 코드 수정·시험 재실행·게이트 재시작·push는 하지 않았다.

집중 검사(p7 알림22, p8 어댑터22, p12 어댑터23, 실제 MultiMesh244, a5·추가 mind UI)는 각 이전 범위의 근거로 보존한다. 집중 검사 통과가 전체 시험의 이번 오류를 해소하지 않는다. 봉인 최초 실패·이전 무효 측정도 유지한다. 이 종료 문서 커밋은 **report-only / NOT_FULL_GATED**이며 게이트 소스6ed와 구분한다.

### 오너가 할 일 (실행 한 줄: /Users/rexxa/work/new-world-play2-dev/tools/play.sh main)

현재 명령은 게시된 버전을 실행한다. 이 M-UX2.1 후보는 아직 게시되지 않았다.

```bash
/Users/rexxa/work/new-world-play2-dev/tools/play.sh main
```

`git ls-remote` 중단 후 확인: 두 원격 모두 기존 게시 BASE에 머문다. 후보 push는 없었다.

```text
3814828c06d1a7a211b13b54368e4e0a488cc0e3 refs/heads/v8/main
3814828c06d1a7a211b13b54368e4e0a488cc0e3 refs/heads/v8/wip
```

미게시 후보·원본 증거·외부 도구 변경을 보존하기 위해 작업트리를 남겼다. 게시 후 제거 조건에 도달하지 않았으며 force 제거하지 않았다.


D21 공개 상태 코멘트: [#515](https://github.com/hyunlord/new-world/issues/515#issuecomment-6472172832) · [#658](https://github.com/hyunlord/new-world/issues/658#issuecomment-6472176210) · [#638](https://github.com/hyunlord/new-world/issues/638#issuecomment-6472176796). [이슈 재조회](/Users/rexxa/work/mux21-data/resume-20261028/continuation-2155/issue-readback.json) · [git ls-remote](/Users/rexxa/work/mux21-data/resume-20261028/continuation-2155/remote-final.txt). 두 원격은 모두 `3814828c06d1a7a211b13b54368e4e0a488cc0e3`이며 새 게시 SHA는 없다.


D22 focused verification: [원시 검증](/Users/rexxa/work/mux21-data/resume-20261029/continuation-0050/why-product-verification.json), [독립 검토](/Users/rexxa/work/mux21-data/resume-20261029/continuation-0050/why-code-visual-review.json). [새 방어/정보창1.25](/Users/rexxa/work/mux21-data/resume-20261029/continuation-0050/capture-defence-a1/defence-inspector-1.25.png). 최신 컨택트시트는 D22 새 왜?/방어4장과 D20 보존 정보창/연대기4장을 캡션으로 구분한다.
