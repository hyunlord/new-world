## 구현 완료 보고

> **최종 결과: PARTIAL — 기술 gate PASS, 미달·미측정 항목 유지.** 보고 시각 2026-10-19T17:26:40.760393+00:00. 측정/runtime 동결 HEAD는 `40aa7a463d740b776248828211251187319c6af6`, 증거 게시 HEAD는 `3159d83c77646d2efaba9a0228bb0abcd206b39f`다. Part3 merged identity와 최종 gate120/120·ERROR0을 통과했다. Part4는 대기 예산으로 TERMINAL_PARTIAL_WAIT_BUDGET 종료했고 네 세계만 유효하며 W-multi·sections·ablation은 NOT_VERIFIED다. 게시 fast-forward, PR511 갱신 및 두 작업용 worktree 정리를 확인했다. 이 보고서 자체의 Part5 커밋 해시는 생성 후 최종 전달 영수증에 기록한다.

### 구현 의도

Art Pass 13에서 남은 화면 검증을 실제 창에서 수행하고, 생애 글자를 12px 설계 그리드의 2배인 24px로 복원하며, 쓰기 생략과 걷는 사람 위치 계산의 비용을 각각 검증한다. 속도·픽셀 동일성·읽기·게시를 별도 판정으로 유지한다.

### 구현 내용

오너 피드백 파일은 시작 시 없었다. Life48→24px, 새 glyph bar10,442쌍 PASS, 실제입력17/17이다. 쓰기 생략은 측정 COMPLETE·W200 실패로 OFF, walker는 수치/실호출/브리지·동일성·성능 통과로 ON이다. Seats는 구현 전 보류했다.

B단계는 적격12/14로 NOT_VERIFIED(범례10/12, 무범례9/12), 마을요약2/6 NOT_RUN, 대비12/21이다. H10은 관계·화해 성공에도 도로 짐꾼 가림으로 FAIL이다.161장 판독의 검증 가능한 비이름 오답0, 이름 오독56회는 별도다. 그림만으로 활동 인지는 NOT_ESTABLISHED다. 이전 실패와 historical24/36/48 글리프 결과는 보존했다. Part4는 부분 종료이며 증거 게시·PR511 갱신·작업용 두 worktree 정리는 완료했다. 최종 기술 gate는120/120·ERROR0이다.

### 구현 방법

GDScript 기준 함수를 oracle/off-switch로 유지하고 art-owned Rust 커널·브리지·fixture만 변경했다. f32 Vector2/Rect2와 f64 scalar의 연산 순서 및 ID 순서를 검증했다. 판독 답변은 키/reference 공개 전에 동결했고, 미검증·화면 밖·추론을 정답에 넣지 않았다. 성능의 release·교대 쌍·행렬 및 before/after 비교 한계는 측정 결과에 구분했다.

### 기능 설명

생애 일반/굵은 글자는24px다. 툴팁은 전달된 입력 좌표에서 이름·활동을 표시하며 mouse-out/무활동에는 숨긴다. Walker는 동일성·속도 기준 통과로 ON, 쓰기 생략은 W200 기준 미달로 OFF다. 최종 기술 gate는 통과했고 성능은 부분 종료 판정이다. GOAL §8.2 기술 게시 조건으로 fast-forward 게시했다; 전체 아트·성능 수락은 PARTIAL이다.

### 변경된 파일 목록

아래는 최초 증거 게시까지의19개 작업 커밋과 커밋별 변경 경로다. 경로는 Git에서 직접 읽었으며 증거 파일은 디렉터리와 개수로 묶었다. 이 보고서를 담는 Part5 커밋의 자체 해시는 생성 후 최종 전달 영수증에 추가한다.

- `f962f5cd6e671c6b000ca6e0734b904449360650` — Part 0
  - [GOAL.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/GOAL.md) (1개)

- `9c023aea430c9d0122c9883f7509712f88c2b188` — Part 0
  - `tools/art_pipeline/run_agent_sections_p13b.py`; `tools/art_pipeline/run_alternating_p13b.py`; `tools/art_pipeline/run_perf_matrix_p13b.sh`; `tools/art_pipeline/with_quiet_lock_p13b.sh`; [DECISIONS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/DECISIONS.md) (1개); [TASKS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/TASKS.md) (1개); [TOOLING.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/TOOLING.md) (1개)

- `22d92922f513569df8f4a6e6a9b4e8d866dbb3be` — Part 0
  - [TASKS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/TASKS.md) (1개); [perf/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/perf/) (1개)

- `8900e1c69f8ba3aa8775d6f1af154e622bcb87df` — Part 0
  - [BLIND-PROTOCOL.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/BLIND-PROTOCOL.md) (1개); [H10-SOURCES.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/H10-SOURCES.md) (1개); [TASKS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/TASKS.md) (1개)

- `32b33df8a53f8d6f4c2958d21106308e358e54f0` — Part 0
  - [TASKS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/TASKS.md) (1개); [WALKER-SCOPE.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/WALKER-SCOPE.md) (1개)

- `c774ff518ace7abd0e01e77af3d4752149d7901e` — Part 0
  - [TASKS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/TASKS.md) (1개); [perf/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/perf/) (5개)

- `4a954fa9266ab24f7d3f9ba5c1fc3414ca33ba85` — Part 0
  - [TASKS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/TASKS.md) (1개); [gates/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/gates/) (150개)

- `f55cf1e0e60a381c9c0d0199c26d07e1c57fc155` — Part 1
  - `scripts/test/art/glyph_distinct_p13b.gd`; `scripts/test/art/real_input_p13b.gd`; `scripts/test/art/test_glyph_distinct_p13.gd`; `scripts/test/art/test_real_input_p13.gd`; `scripts/ui/panels/agent_inspector_panel.gd`; `scripts/ui/person_activity_tooltip_p13.gd`; [DECISIONS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/DECISIONS.md) (1개); [GLYPH-CHANGE-PLAN.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/GLYPH-CHANGE-PLAN.md) (1개); [TASKS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/TASKS.md) (1개); [TOOLTIP-DEBUG.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/TOOLTIP-DEBUG.md) (1개); [gates/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/gates/) (24개); [glyph/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/glyph/) (14개)

- `8ca6b93018b0ec0b795f8312c549197a97f851da` — Part 2
  - `scripts/test/art/capture_h10_p13b.gd`; `scripts/test/art/perf_alternating_frames_p13b.gd`; `tools/art_pipeline/badge_evidence_p13b.py`; `tools/art_pipeline/run_alternating_p13b.py`; [BADGE-METHOD.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/BADGE-METHOD.md) (1개); [H10-CAPTURE-PROTOCOL.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/H10-CAPTURE-PROTOCOL.md) (1개); [PERF-COLLECTOR-DEBUG.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/PERF-COLLECTOR-DEBUG.md) (1개); [TASKS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/TASKS.md) (1개); [perf/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/perf/) (6개)

- `3bee1316acdbcb891bda0be1385ba959bbb1e7f1` — Part 2
  - `scripts/test/art/capture_h10_p13b.gd`; `scripts/test/art/capture_life_p13b.gd`; `tools/art_pipeline/prepare_blind_crops_p13b.py`; `tools/art_pipeline/run_life_captures_p13b.py`; [DECISIONS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/DECISIONS.md) (1개); [H10-CAPTURE-PROTOCOL.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/H10-CAPTURE-PROTOCOL.md) (1개); [TASKS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/TASKS.md) (1개)

- `75c2ea6a2cdfb085a4a4a90ceaf9fe257cda16a3` — Part 2
  - `scripts/test/art/capture_h10_p13b.gd`; `scripts/test/art/capture_life_p13b.gd`; [DECISIONS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/DECISIONS.md) (1개); [H10-CAPTURE-PROTOCOL.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/H10-CAPTURE-PROTOCOL.md) (1개); [contrast/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/contrast/) (40개)

- `af30bb29b01c88b6c3f6475e7cf179dc8232bb45` — Part 1
  - `rust/crates/sim-bridge/src/ffi/art_kernels.rs`; `rust/crates/sim-bridge/src/ffi/world_node.rs`; `rust/crates/sim-bridge/tests/fixtures/art_p13b/walker_numeric_v1.bin`; `rust/crates/sim-bridge/tests/fixtures/art_p13b/walker_numeric_v1.bin.json`; `rust/crates/sim-bridge/tests/fixtures/art_p13b/walker_v1.bin`; `rust/crates/sim-bridge/tests/fixtures/art_p13b/walker_v1.bin.json`; `scripts/test/art/capture_framings_p13b.gd`; `scripts/test/art/capture_identity_p13b.gd`; `scripts/test/art/capture_running_identity_p13b.gd`; `scripts/test/art/capture_walker_fixture_p13b.gd`; `scripts/test/art/perf_alternating_frames_p13b.gd`; `scripts/test/art/walker_bridge_p13b.gd`; `scripts/test/art/walker_fixture_recorder_p13b.gd`; `scripts/test/art/walker_numeric_probe_p13b.gd`; `scripts/ui/agent_renderer.gd`; `scripts/ui/art_walker_p13b.gd`; `tools/art_pipeline/run_alternating_p13b.py`; `tools/art_pipeline/run_identity_p13b.sh`; `tools/art_pipeline/run_running_identity_p13b.sh`; [DECISIONS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/DECISIONS.md) (1개); [TASKS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/TASKS.md) (1개); [walker/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/walker/) (3개)

- `d56a926e9f665f4e0c04b70c9502533b9c96aedc` — Part 1
  - `scripts/test/art/capture_framings_p13b.gd`; `tools/art_pipeline/validate_walker_identity_p13b.py`; [DECISIONS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/DECISIONS.md) (1개); [walker/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/walker/) (126개)

- `c1827b3b68bfac11155c8cd00e45ecfe69e3fdc6` — Part 2
  - [TASKS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/TASKS.md) (1개); [activity/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/activity/) (24개); [execution-timeline.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/execution-timeline.json) (1개); [execution-timeline.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/execution-timeline.md) (1개); [fact-reading/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/fact-reading/) (19개); [h10/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/h10/) (19개); [life-capture-report/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/life-capture-report/) (14개); [perf/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/perf/) (29개); [retries.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/retries.json) (1개); [retries.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/retries.md) (1개)

- `a5771f1f489084d6e29032cb06365ffb87ff5731` — Part 1
  - `scripts/ui/agent_renderer.gd`; `tools/art_pipeline/validate_walker_identity_p13b.py`; `tools/art_pipeline/validate_walker_pairs_p13b.py`; [DECISIONS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/DECISIONS.md) (1개); [TASKS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/TASKS.md) (1개); [perf/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/perf/) (2개); [retries.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/retries.json) (1개); [retries.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/retries.md) (1개); [review/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/review/) (7개); [walker/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/walker/) (299개)

- `1f6ee3a0a51f41bd5b688f44738ee3a98b8ab9d0` — Part 3
  - `scripts/test/art/capture_framings_p13b.gd`; `scripts/test/art/perf_alternating_frames_p13b.gd`; `scripts/test/art/test_meal_ring.gd`; [DECISIONS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/DECISIONS.md) (1개); [TASKS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/TASKS.md) (1개); [gates/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/gates/) (157개)

- `f5c8951e0a0a5861defe632cbd2a088dac71f692` — Part 3
  - `scripts/test/art/test_agent_drawn_positions.gd`; `scripts/test/art/test_cold_presentation.gd`; [DECISIONS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/DECISIONS.md) (1개); [TASKS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/TASKS.md) (1개); [gates/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/gates/) (155개)

- `40aa7a463d740b776248828211251187319c6af6` — Part 3
  - [TASKS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/TASKS.md) (1개); [execution-timeline.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/execution-timeline.json) (1개); [execution-timeline.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/execution-timeline.md) (1개); [gates/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/gates/) (157개); [perf/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/perf/) (1개); [retries.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/retries.json) (1개); [retries.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/retries.md) (1개); [walker/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/walker/) (270개)


- `3159d83c77646d2efaba9a0228bb0abcd206b39f` — Part 4, 증거만288개 경로
  - `docs/art/production13b/DECISIONS.md`; `TASKS.md`; `execution-timeline.json`; `execution-timeline.md`; `retries.json`; `retries.md` (동일 production13b 디렉터리); [perf/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/perf/)의 `after.json`, `after.md`, `attempts.json`, `freeze.json`, `holds.md` (5개); [after-evidence/](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/perf/after-evidence/) (277개, [전체 manifest](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/perf/after-evidence/SHA256-MANIFEST.json)). 제품·도구 소스 변경 없음.

전체 경로 기계 판독 목록: [execution-timeline.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/execution-timeline.json). 2026-10-19T17:24:47.008395+00:00 snapshot에19개 커밋 전체가 포함됐음을 확인했다. 위 커밋별 목록은 제품·도구 경로를 그대로 쓰고 증거 파일만 디렉터리·개수로 묶었다.

### 측정 결과

**H1~H11: before는 Art Pass 13 유효 release 기준선이다. W0/W1/W200/W500의 유효 after만 아래에 반영했다. 나머지 after와 비율은 NOT_VERIFIED이다.** 화면 검증과 최종 기술 빌드 판정은 별도 범위다.

| 항목 | before / 기준 | 최종 after | after/before | 현재 증거 |
|---|---|---|---|---|
| H1 | W0 frozen 중앙값 2.8235ms / ≤6 | 3.1880ms | 1.129095 | PASS |
| H2 | W1 2× p99 13.9210ms / ≤16.67 | 13.4150ms | 0.963652 | PASS |
| H3 | W200 2× 중앙값 26.1450ms / ≤16.67 | 17.0395ms | 0.651731 | FAIL |
| H4 | W200 frozen 중앙값 16.2575ms / ≤10 | 11.2805ms | 0.693864 | FAIL |
| H5 | W500 frozen 중앙값 28.4035ms / ≤16.67 | 23.3245ms | 0.821184 | FAIL |
| H6 | W-multi 2× 중앙값16.3505ms; W500 population loss 제외 | W500 running41.2405ms; W-multi NOT_VERIFIED | W500 N/A | 보고용 |
| H7 | declump 동일성 exact / 2,280 frames | PASS | 해당 없음 | accepted walker merged compound PASS; write-skip OFF |
| H8 | A 7/7; B/화면 대비 미검증 | NOT_VERIFIED | 해당 없음 | B12/14 부족; 대비12/21 |
| H9 | 48px; 이전 glyph bar PASS; 사실/입력 미검증 | PASS(정의된 범위) | 해당 없음 | 24px glyph·검증가능 사실·입력17/17; 이름 별도 |
| H10 | 최종 화면 미검증 | FAIL | 해당 없음 | 도로3/3 실패; 관계/화해 PASS |
| H11 | W200 walker9.8755ms/seats4.4075ms; W-multi seats5.6970ms | NOT_VERIFIED | NOT_VERIFIED | 아래 후보 쌍은 최종 행렬과 별개 |

**세계·조건별 before 행렬** — 각각 3블록×300 counted+30 discarded. before 표와 별도로 유효 네 세계 after를 다음에 제시한다. 다른 after는 NOT_VERIFIED. GPU0은 unavailable이며 실측 0비용이 아니다.

| World / condition | Median ms | p90 | p99 | Max | Eligible frames | Ticks/frame median | Counted ticks | Population start→end blocks |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| W0 frozen | 2.8235 | 3.2300 | 3.8020 | 4.5000 | 900 | 0.0 | 0 | 29→29, 29→29, 29→29 |
| W0 running | 5.3035 | 5.7640 | 6.0620 | 6.2550 | 900 | 1.0 | 900 | 29→29, 29→29, 29→29 |
| W500 frozen | 28.4035 | 29.7040 | 30.9490 | 31.5260 | 900 | 0.0 | 0 | 500→500, 500→500, 500→500 |
| W1 frozen | 6.2970 | 7.1710 | 10.9150 | 13.0250 | 900 | 0.0 | 0 | 71→71, 71→71, 71→71 |
| W1 running | 12.5550 | 13.2380 | 13.9210 | 14.4940 | 900 | 1.0 | 900 | 71→71, 71→71, 71→71 |
| W200 frozen | 16.2575 | 17.0700 | 17.7780 | 18.4360 | 900 | 0.0 | 0 | 200→200, 200→200, 200→200 |
| W200 running | 26.1450 | 27.8510 | 30.0760 | 62.1090 | 900 | 2.0 | 1507 | 200→200, 200→200, 200→200 |
| W-multi frozen | 7.3015 | 8.0800 | 14.4980 | 15.5280 | 900 | 0.0 | 0 | 88→88, 88→88, 88→88 |
| W-multi running | 16.3505 | 17.4210 | 18.2170 | 19.0050 | 900 | 1.0 | 963 | 88→87, 88→87, 88→87 |
| W500 running | N/A | N/A | N/A | N/A | 0 | N/A | excluded | 500→392,392,404 |

정확한 블록별 틱 범위·인구·제외행과 p90/p99/최대는 [perf/before-reference.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/perf/before-reference.json) 및 [perf/before-reference.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/perf/before-reference.md)에 보존했다. W500 running 세 블록의 500→392/392/404 인구 감소는 속도 PASS/FAIL 대상이 아니다.

**유효 after — 네 세계만**

W1은 시나리오 이름이며1명이 아니다. 실제 live71명/W200200명이다. GPU raw0은 unavailable/NOT_MEASURED다.

| 세계 | 조건 | median ms | p90 | p99 | max | frames | counted ticks | tick-carrying frames | ticks/frame median |
|---|---|---|---|---|---|---|---|---|---|
| W0 | frozen_Z4_0.0x | 3.188000 | 3.556000 | 3.969000 | 4.633000 | 900 | 0.0 | 0 | 0.000000 |
| W0 | running_Z4_2.0x | 5.262000 | 5.599000 | 5.862000 | 6.100000 | 900 | 899.0 | 899 | 1.000000 |
| W1 | frozen_Z4_0.0x | 6.106000 | 6.615000 | 10.959000 | 11.640000 | 900 | 0.0 | 0 | 0.000000 |
| W1 | running_Z4_2.0x | 12.050000 | 12.767000 | 13.415000 | 14.723000 | 900 | 900.0 | 900 | 1.000000 |
| W200 | frozen_Z4_0.0x | 11.280500 | 12.325000 | 12.971000 | 13.342000 | 900 | 0.0 | 0 | 0.000000 |
| W200 | running_Z4_2.0x | 17.039500 | 18.210000 | 19.785000 | 60.354000 | 900 | 1019.0 | 900 | 1.000000 |
| W500 | frozen_Z4_0.0x | 23.324500 | 24.236000 | 25.085000 | 25.760000 | 900 | 0.0 | 0 | 0.000000 |
| W500 | running_Z4_2.0x | 41.240500 | 42.977000 | 44.771000 | 46.189000 | 900 | 2369.0 | 900 | 3.000000 |

| 세계/블록 | frames | counted ticks | tick frames | 인구 시작→끝 | tick 시작→끝 | 제외 |
|---|---|---|---|---|---|---|
| W0 frozen_Z4_0 | 300 | 0.000000 | 0 | 29 → 29 | 40560 → 40560 | False |
| W0 frozen_Z4_1 | 300 | 0.000000 | 0 | 29 → 29 | 40560 → 40560 | False |
| W0 frozen_Z4_2 | 300 | 0.000000 | 0 | 29 → 29 | 40560 → 40560 | False |
| W0 running_2x_0 | 300 | 299.000000 | 299 | 29 → 29 | 40560 → 40896 | False |
| W0 running_2x_1 | 300 | 300.000000 | 300 | 29 → 29 | 40560 → 40896 | False |
| W0 running_2x_2 | 300 | 300.000000 | 300 | 29 → 29 | 40560 → 40896 | False |
| W1 frozen_Z4_0 | 300 | 0.000000 | 0 | 71 → 71 | 126994 → 126994 | False |
| W1 frozen_Z4_1 | 300 | 0.000000 | 0 | 71 → 71 | 126994 → 126994 | False |
| W1 frozen_Z4_2 | 300 | 0.000000 | 0 | 71 → 71 | 126994 → 126994 | False |
| W1 running_2x_0 | 300 | 300.000000 | 300 | 71 → 71 | 126994 → 127330 | False |
| W1 running_2x_1 | 300 | 300.000000 | 300 | 71 → 71 | 126994 → 127330 | False |
| W1 running_2x_2 | 300 | 300.000000 | 300 | 71 → 71 | 126994 → 127330 | False |
| W200 frozen_Z4_0 | 300 | 0.000000 | 0 | 200 → 200 | 1440 → 1440 | False |
| W200 frozen_Z4_1 | 300 | 0.000000 | 0 | 200 → 200 | 1440 → 1440 | False |
| W200 frozen_Z4_2 | 300 | 0.000000 | 0 | 200 → 200 | 1440 → 1440 | False |
| W200 running_2x_0 | 300 | 346.000000 | 300 | 200 → 200 | 1440 → 1832 | False |
| W200 running_2x_1 | 300 | 338.000000 | 300 | 200 → 200 | 1440 → 1822 | False |
| W200 running_2x_2 | 300 | 335.000000 | 300 | 200 → 200 | 1440 → 1818 | False |
| W500 frozen_Z4_0 | 300 | 0.000000 | 0 | 500 → 500 | 1440 → 1440 | False |
| W500 frozen_Z4_1 | 300 | 0.000000 | 0 | 500 → 500 | 1440 → 1440 | False |
| W500 frozen_Z4_2 | 300 | 0.000000 | 0 | 500 → 500 | 1440 → 1440 | False |
| W500 running_2x_0 | 300 | 798.000000 | 300 | 500 → 500 | 1440 → 2324 | False |
| W500 running_2x_1 | 300 | 791.000000 | 300 | 500 → 500 | 1440 → 2318 | False |
| W500 running_2x_2 | 300 | 780.000000 | 300 | 500 → 500 | 1440 → 2304 | False |

**Primary CPU after/before 배수(정규화 없음)**

| 세계 | 조건 | median | p90 | p99 | max |
|---|---|---|---|---|---|
| W0 | frozen_Z4_0.0x | 1.129095 | 1.100929 | 1.043924 | 1.029556 |
| W0 | running_Z4_2.0x | 0.992175 | 0.971374 | 0.967008 | 0.975220 |
| W1 | frozen_Z4_0.0x | 0.969668 | 0.922465 | 1.004031 | 0.893666 |
| W1 | running_Z4_2.0x | 0.959777 | 0.964421 | 0.963652 | 1.015800 |
| W200 | frozen_Z4_0.0x | 0.693864 | 0.722027 | 0.729610 | 0.723693 |
| W200 | running_Z4_2.0x | 0.651731 | 0.653836 | 0.657833 | 0.971743 |
| W500 | frozen_Z4_0.0x | 0.821184 | 0.815917 | 0.810527 | 0.817103 |
| W500 | running_Z4_2.0x | N/A | N/A | N/A | N/A |

W500 running의 역사적 before는 인구 감소로 제외되어 비교 배수N/A다. Process/render/setup/pacing 및 tick-carrying 분포 전체는 [승인 비교표](/Users/rexxa/work/art-production13b-evidence/after-report-prep/comparison.md)에 보존했다. W-multi/timer/ablation은 NOT_VERIFIED이다.

**실행 진단 before → after**

| 세계 | AgentRenderer median | walker median | seats median | counted ticks/frames | after |
|---|---|---|---|---|---|
| W200 | 22.3270ms | 9.8755ms | 4.4075ms | 512/300 | NOT_VERIFIED |
| W-multi | 11.5090ms | 1.3535ms | 5.6970ms | 328/300 | NOT_VERIFIED |
| W500 frozen | 24.5845ms | 1.8035ms | 0.6200ms | 0/300 | NOT_VERIFIED |

모든 exclusive span과 nested getter의 before 분포는 before-reference.json에 있다. 중첩 getter 시간을 합산하지 않는다. 최종 두 running section timer는 NOT_VERIFIED이다.

**후보별 성능·수락**

| 후보/조건 | pooled candidate/control | 쌍별 비율 | 유효 쌍 | 판정 |
|---|---|---|---|---|
| declump 이전 W200 | 0.9642023799 | 0.9582264068, 0.9657586634, 0.9729393708 | 3 | 기수락 ON |
| declump 이전 W500 | 0.7962943016 | 0.7820915220, 0.7962567983, 0.7986974329 | 3 | 기수락 ON |
| write-skip W200 | 0.9986985165 | 1.0228750841, 0.9823399557, 0.9946989399 | 3, 2/3 faster | 속도 FAIL |
| write-skip W500 | 0.9472853932 | 0.9468757316, 0.9414535143, 0.9509198587 | 3, 3/3 faster | 속도 PASS / 전체 OFF |
| walker W200 | 0.6492237648 | 0.6472445182, 0.6466932538, 0.6541020268 | 3, 3/3 faster | 속도·동일성 PASS / 기본 ON |
| walker W500 | 0.9446215390 | 0.9499485291, 0.9436013447, 0.9386296695 | 3, 3/3 faster | 속도·동일성 PASS / 기본 ON |

쓰기 생략 필수 측정 COMPLETE. W500 attempt1 collector FAIL, attempt2 입장 거부(창 미실행), attempt3 VALID를 모두 보존했다. W200 실패를 더 좋은 값이 나올 때까지 재측정하지 않는다. Walker W500은 pooled≤1.00와 ≥2/3 faster를 모두 요구한다. 첫 comparator의 잘못된 calibration 선택 및 두 번째 supplemental 판정식 누락을 보존했고, 세 번째 comparator는 이 조건을 복원했다. 실제 재측정은 아니다.

[perf/write-skip/REPORT.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/perf/write-skip/REPORT.md) · [walker/performance/README.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/walker/performance/README.md)

**Walker 수락 근거 갱신:** identity attempt2는 28개 창 실행 모두 exit0, 여섯 P1–P6의 A/A′ 및 A/B 각각 RGBA 차이0, 다섯 off/on running JSONL byte-exact다(600/600/600/240/240프레임). `compound_walker_candidate_identity`가 실제 polarity·provenance와 inherited reference-only 비교를 결합한다. 18개 still capture stamp, 4,560개 running-frame stamp와10개 final stamp를 검증했다. Still은 매 프레임 stamp가 아니다. 첫 P3 control924픽셀 실패는 그대로 보존했다. 동결 캡처 HEAD는c1827b3b, 기본 ON 적용은a5771f1f이며 둘을 같은 캡처 HEAD로 쓰지 않는다. 속도는 W2000.6492237648/W5000.9446215390, 각각3/3 faster로 corrected 기준을 만족한다.

**후보 실험의 pooled 분포(최종 after 아님)**

| 후보 | 세계 | 모드 | counted frames | median ms | p90 | p99 | max |
|---|---|---|---|---|---|---|---|
| write-skip | W200 | control | 450 | 24.9714999958873 | 27.0559999989271 | 29.0710000016689 | 62.8640000003576 |
| write-skip | W200 | candidate | 450 | 24.93900000065565 | 27.1550000016689 | 29.0009999983311 | 62.8499999963045 |
| write-skip | W500 | control | 450 | 32.01200000703335 | 32.9640000128746 | 33.838000002861 | 34.4330000007153 |
| write-skip | W500 | candidate | 450 | 30.324500014424302 | 31.2760000121593 | 32.4260000047684 | 35.6660000140667 |
| walker | W200 | control | 450 | 35.36299999588725 | 37.4859999985695 | 40.178999989748 | 90.1529999939203 |
| walker | W200 | candidate | 450 | 22.958499993145452 | 23.9390000019073 | 27.2309999995232 | 73.6660000047684 |
| walker | W500 | control | 450 | 31.99800000452995 | 33.1860000109673 | 40.843000005722 | 42.8069999933243 |
| walker | W500 | candidate | 450 | 30.226000009655948 | 31.0639999866486 | 32.6429999864101 | 36.4630000011921 |

각 세계 6블록×150 counted+30 discarded다. 쓰기 생략 W200은 population200 유지, 각 블록 tick1440→1732/1735/1736/1734/1733/1732; W500은 population500/tick1440 고정이다. Walker의 블록별 실제 tick/population과 ticks/frame은 원본 frames 및 보존 summary를 참조하며, 최종 running timer로 대신하지 않는다.

**보정 및 모든 probe 값**

Art Pass13 R=2.0089961669873446s; 13b R=2.762010041042231s. **왼쪽은 Art Pass13에서 이미 수집한 역사적 15회**, 오른쪽만 **이번 Art Pass13b의 새 15회**다. 이번 작업이 30회 새 교정을 했다는 뜻이 아니다. 새 교정 간격은20초다. 완전한 process snapshot/epoch/checksum은 각 원본 JSON에 남아 있다.

| 각 실행 내 회차 | Art Pass13 역사적 calibration seconds | Art Pass13b 새 calibration seconds |
|---|---|---|
| 1 | 2.056977750035003 | 2.8725179590983316 |
| 2 | 2.0658965839538723 | 2.838410999975167 |
| 3 | 2.141888541984372 | 3.034052166971378 |
| 4 | 2.037486167042516 | 2.754094292060472 |
| 5 | 2.054103208007291 | 2.9070539579261094 |
| 6 | 2.0768123329617083 | 3.359423916088417 |
| 7 | 2.3467857079813257 | 2.787542709033005 |
| 8 | 2.031399750034325 | 2.8077280829893425 |
| 9 | 2.2023411249974743 | 2.7696236670017242 |
| 10 | 2.0089961669873446 | 2.762029125005938 |
| 11 | 2.0538550410419703 | 2.762010041042231 |
| 12 | 1.977922167046927 | 2.7651633749483153 |
| 13 | 2.0016578329959884 | 2.790780666982755 |
| 14 | 2.0640369160100818 | 2.7614519170019776 |
| 15 | 2.025686457986012 | 3.019692290923558 |

| 세션(원본 JSON 경로) | 시작 probe s | 종료 probe s | 시작 load1m | 종료 load1m | 입장/완료 |
|---|---|---|---|---|---|
| before/matrix/0/admission | 1.9612876659957692 | 2.0049986250232905 | 2.50439453125 | 3.0498046875 | VALID |
| before/matrix/1/admission | 1.963445374974981 | 1.9780847090296447 | 3.04541015625 | 3.447265625 | VALID |
| before/matrix/2/admission | 2.0479197499807924 | 2.0829888749867678 | 3.3310546875 | 3.60302734375 | VALID |
| before/matrix/3/admission | 1.9620209169806913 | 2.021250291960314 | 3.71484375 | 3.47265625 | VALID |
| before/matrix/4/admission | 2.0066879999358207 | 1.988965625059791 | 3.5146484375 | 3.78955078125 | VALID |
| before/sections/0/admission | 1.9415477910079062 | 2.052496583084576 | 3.54248046875 | 3.87744140625 | VALID |
| before/sections/1/admission | 1.9961711670039222 | 1.9911151661071926 | 4.20751953125 | 3.40185546875 | VALID |
| before/sections/2/admission | 1.956553625059314 | 2.022985417046584 | 3.96630859375 | 3.54248046875 | VALID |
| before/node_ablation_W200/admission | 2.008760750060901 | 2.0180110409855843 | 3.60986328125 | 6.0234375 | VALID |
| before/accepted_declump/workloads/0/admission | 2.0033213749993593 | 2.021951709059067 | 3.98876953125 | 3.8291015625 | VALID |
| before/accepted_declump/workloads/1/admission | 1.9380350839346647 | 1.9753036251058802 | 3.8291015625 | 6.2890625 | VALID |
| W200-attempt1 | 2.034634416922927 | 2.0638448749668896 | 4.4619140625 | 5.32861328125 | VALID |
| W500-attempt1 | 2.0151671250350773 | 2.036255292012356 | 5.8466796875 | 6.02197265625 | NOT_VERIFIED (contended or unsuccessful) |
| W500-attempt2 | 2.7825928339734674 | 2.8250994589179754 | 23.0029296875 | 21.72119140625 | NOT_VERIFIED (contended or unsuccessful) |
| W500-attempt3 | 2.8551294580101967 | 2.802626542048529 | 6.6953125 | 6.53125 | VALID |
| walker-W200 | 2.88879304099828 | 2.8703962919535115 | 5.9990234375 | 6.80322265625 | VALID |
| walker-W500 | 2.8955912499222904 | 2.8603887079516426 | 6.1767578125 | 5.720703125 | VALID |

이 표는 현재 보존된 before/후보 세션 probe 전체다. Part4 endpoint는 다음 표에 추가했다. 모든 종료 슬롯을 반영했다. Part1 대기3600초를 포함한 최종 union 대기는 **9487.316187858582초(158.121936분)**; 교정 간격 및 UI 병행 시간을 별도 작업시간으로 중복 합산하지 않는다.

**Part4 최종 endpoint 장부 —29슬롯, 거부/무효 포함**

실제 probe sidecar28개/endpoint56개와 probe 없는 policy refusal1개를 구분한다. 승인 비교표는192개 official/raw 통계를 대조했고, 원시7collector inventory의 무효3개는 성능 비교에 넣지 않았다. [terminal-verification.json](/Users/rexxa/work/art-production13b-evidence/after-report-prep/terminal-verification.json), [terminal-disposition.md](/Users/rexxa/work/art-production13b-evidence/after-report-prep/terminal-disposition.md).

| 슬롯 | 시작probe s | 종료probe s | 시작load | 종료load | valid | exit |
|---|---|---|---|---|---|---|
| ablation-W200-attempt1 | 2.0424122089752927 | 2.043478582985699 | 17.2421875 | 17.2421875 | False | 75 |
| ablation-W200-attempt2 | 2.5450988749507815 | 2.1508032500278205 | 38.0810546875 | 35.912109375 | False | 75 |
| ablation-W200-attempt3 | 1.993954541045241 | 1.9969077910063788 | 8.12255859375 | 8.0322265625 | False | 75 |
| ablation-W200-attempt4 | 2.070060541969724 | 2.1146300000837073 | 10.63671875 | 10.63671875 | False | 75 |
| after-W-multi-attempt1 | 1.996134917018935 | 2.0218232090119272 | 4.9443359375 | 17.36376953125 | False | 0 |
| after-W-multi-attempt2 | 2.370870958082378 | 2.536675291019492 | 36.236328125 | 36.236328125 | False | 75 |
| after-W-multi-attempt3 | 2.077279124991037 | 2.0972419580211863 | 5.88916015625 | 8.11865234375 | False | 0 |
| after-W-multi-attempt4 | N/A | N/A | N/A | N/A | False | N/A |
| after-W-multi-attempt5 | 2.0881396669428796 | 2.128351124934852 | 17.5810546875 | 17.5810546875 | False | 75 |
| after-W0-attempt1 | 2.0783221250167117 | 1.9986704170005396 | 8.83251953125 | 8.765625 | False | 75 |
| after-W0-attempt2 | 1.9984747080598027 | 3.663989166961983 | 7.8330078125 | 30.97119140625 | False | 0 |
| after-W0-attempt3 | 2.0207529170438647 | 2.036457416950725 | 4.92529296875 | 4.4921875 | True | 0 |
| after-W1-attempt1 | 2.0040063749765977 | 2.047847916954197 | 7.958984375 | 4.751953125 | True | 0 |
| after-W200-attempt1 | 2.116670334013179 | 2.0974191669374704 | 4.751953125 | 4.6787109375 | True | 0 |
| after-W500-attempt1 | 1.9752058750018477 | 2.0059507919941097 | 8.765625 | 8.3037109375 | False | 75 |
| after-W500-attempt2 | 2.613776292069815 | 3.265273208031431 | 32.01416015625 | 34.2548828125 | False | 75 |
| after-W500-attempt3 | 2.0768032500054687 | 2.1021817500004545 | 4.21240234375 | 6.31494140625 | True | 0 |
| sections-W-multi-attempt1 | 2.0292082079686224 | 2.00958766695112 | 16.21826171875 | 17.72216796875 | False | 75 |
| sections-W-multi-attempt2 | 2.142576500074938 | 2.179716166923754 | 38.15966796875 | 37.986328125 | False | 75 |
| sections-W-multi-attempt3 | 2.0056214159121737 | 2.0299411669839174 | 8.4140625 | 8.4140625 | False | 75 |
| sections-W-multi-attempt4 | 2.0853448329726234 | 2.14754425000865 | 11.7587890625 | 11.37744140625 | False | 75 |
| sections-W200-attempt1 | 2.1967883749166504 | 2.179264000034891 | 16.93408203125 | 16.93408203125 | False | 75 |
| sections-W200-attempt2 | 2.2404428330482915 | 2.137166333035566 | 37.73876953125 | 38.15966796875 | False | 75 |
| sections-W200-attempt3 | 1.9865784160792828 | 2.0046513330889866 | 8.18896484375 | 8.18896484375 | False | 75 |
| sections-W200-attempt4 | 2.110640041064471 | 2.1339136250317097 | 12.4345703125 | 11.7587890625 | False | 75 |
| sections-W500-attempt1 | 2.0215846670325845 | 2.0347752498928457 | 17.72216796875 | 17.263671875 | False | 75 |
| sections-W500-attempt2 | 2.191086082952097 | 2.203790667001158 | 37.8271484375 | 37.8271484375 | False | 75 |
| sections-W500-attempt3 | 2.002092625014484 | 1.988937458023429 | 8.220703125 | 8.220703125 | False | 75 |
| sections-W500-attempt4 | 2.1483573329169303 | 2.147741875029169 | 10.86669921875 | 10.86669921875 | False | 75 |

**역사적 before와 새 after의 비교 한계 — 정규화 없음.** 두 R은 각각 가장 빠른5회 probe의 중앙값이다. 역사적 R **2.0089961669873446s**의1.05R은 **2.109445975336712s**, 새 R **2.762010041042231s**의1.05R은 **2.9001105430943426s**다. 새/이전 R은 **1.3748209610494642**(약37.4821% 증가)이며 probe 시간의 관측 차이일 뿐 프레임 시간 보정계수나 원인 분석이 아니다. 열·전원·배경부하 탓이라고 확정하지 않는다.

before/after는 원시 ms와 after/before를 **그대로** 보고했다. 환경 차이를 포함한 세션 간 기술적 비교이며 walker 단독 인과 효과로 읽지 않는다. 같은 세션의 교대 쌍은 더 강한 후보 효과 증거이지만 시간 경과/상태 변화가 남으며 통계적 유의성·신뢰구간은 추정하지 않는다. H1~H5 절대 기준과 후보 기준은 변경하지 않으며 R로 실패를 구제하지 않는다. 이전 세션은 이전 reference/hash, 새 세션은 새 reference/hash로 검증하고 P≤1.05R·시작/끝 load<8·exit·frame/provenance 조건을 각각 적용한다.

Collector는 live game_v26/stress game_v19, matrix3×300+30 warmup, section300+30을 유지한다. Seed만으로 실제 인구·틱·프레이밍 일치를 주장하지 않는다. 유효 after의 all-frame/tick-carrying 분포·인구·틱은 raw에서 집계했다. 중첩 getter 호출/시간은 sections가 입장 거부되어 NOT_VERIFIED다. GPU/FoundingGroups 별도 callback은 unavailable이다. Frozen snapshot과 walkerON/declumpON/writeSkipOFF 환경을 보존하지만 matrix/section에는 **실제 per-frame switch stamp가 없다**. [비교 한계 감사](/Users/rexxa/work/art-production13b-evidence/before-after-comparison-caveats.md)는 비교 방법의 한정 기록이며 최종 after 인증을 대신하지 않는다.

**Identity→measurement provenance 범위:** 전체 manifest는 **655개 untracked 경로가 다르다**.183개 `.gd.uid`의 checkout-local ID,471개 `.import`의 UID만 다르며 해당 compiled imported-resource471쌍은 byte-identical이다. 나머지 import 설정과 Git-tracked runtime source는 같다. 남은1개 ignored `rust/Cargo.lock` 차이는 measurement 브리지 빌드에 사용되지 않았다. 측정 release/debug dylib는 root f5c8951e의30ed9754 빌드를 byte-copy했고 merged identity와 동일하다. 양쪽 lockfile/해시를 보존했다. **전체 manifest 동일** 대신 tracked runtime source·compiled resource bytes·actual bridge 동등성과 measurement-local snapshot 안정성만 주장한다. [동등성 한정 기록](/Users/rexxa/work/art-production13b-evidence/after/identity-source-equivalence.md).

**활동 판독**

A단계는 이전 7/7을 유지(새 반복 아님). 범례 제공 reader와 무범례 fresh reader의 동결 답을 별도로 채점했다. 14장 중 12장만 적격이므로 부분 진단을 12/14 달성으로 환산하지 않는다.

| 활동 | 적격/필요 | 범례 정답/제시 | 무범례 정답/제시 |
|---|---|---|---|
| TendFire | 1/2 | 1/1 | 1/1 |
| Craft | 2/2 | 2/2 | 0/2 |
| Repair | 1/2 | 0/1 | 0/1 |
| HearthCircle | 2/2 | 2/2 | 2/2 |
| Explore | 2/2 | 2/2 | 2/2 |
| Play | 2/2 | 2/2 | 2/2 |
| SitByFire | 2/2 | 1/2 | 2/2 |

합계 범례 **10/12**, 무범례 **9/12**. 전체 B는 NOT_VERIFIED, Repair는 0/1이다. Repair→Play(범례), Repair→Craft 및 Craft→Repair 2회(무범례), SitByFire→HearthCircle(범례) 혼동을 보존했다. 마을 요약 **NOT_RUN: 적격2/필요6쌍**. 키·제외목록·동결 SHA는 [activity/scores.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/activity/scores.json) 및 해당 디렉터리에 있다.

범례 선행의 원본 timestamped acknowledgment는 **NOT_AVAILABLE**이다. 당시 TASKS 서술은 순서를 뒷받침하지만 동결 답변 SHA는 답변 무결성만 증명하며 선행 확인까지 증명하지 않는다. 활동 획득은 최초 일반6회(Z4 3회+Z1 3회)와 부족종0/2/4/5/6별 보충3회씩15회, 총21회였다. 최초 Z4 3회를 각 종의 시도에 포함하면 보충종별6회 노출이다. 목표 정의가 모호했던 이 집계를 숨기지 않으며 활동 검사 전체의 three-strike 준수를 인증하지 않는다. [재시도 장부](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/retries.md).

**H10 실제 화면**

모두 game_v26 seed19603의 실제 EN/KO 화면이며 debug 캡처다. 도로 짐꾼은 tick122430/122450/122479의3회 모두 FAIL이다. 첫 화면의 지정 짐꾼155는 민트색 SitByFire 배지와 주변 몸체에 가려졌다. 이후 엄격한 draw-order 검사에서도 몸체·아이콘 가림이 남아 종료했다. Z1 관계선은 주황/초록 실선과 빨강 점선3색(d9893a/7fb069/c0392b)으로 PASS다. 실제 kind21, day86의 마을1·2 화해 항목(tick122400)을 tick122430에서 EN/KO 연대기 page000에 표시했다. Fake bridge나 이력 대체는 없었다. 관계·화해 성공은 도로 실패를 상쇄하지 않는다. [H10 판정·원본 링크](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/h10/REPORT.md).

**실제 framebuffer 대비 행렬**

| Activity | Grass | Dirt | Shelter |
|---|---:|---:|---:|
| TendFire | 3.3384:1 | NOT_MEASURED | NOT_MEASURED |
| Craft | 5.7405:1 | 3.4895:1 | 1.5297:1 |
| Repair | 2.2294:1 | NOT_MEASURED | NOT_MEASURED |
| HearthCircle | 3.0239:1 | 1.3714:1 | NOT_MEASURED |
| Explore | 4.9532:1 | NOT_MEASURED | NOT_MEASURED |
| Play | 2.7327:1 | NOT_MEASURED | NOT_MEASURED |
| SitByFire | 5.2206:1 | 3.1734:1 | 1.3912:1 |

**12/21, PARTIAL**. 대비 통과 임계값을 새로 만들지 않았다. 실제 main-fill 대 인접 지형이며 어두운 배지판 대 fill 비율이 아니다. 배지 크기는 Z4 80×80(Craft88×88) draw extent이고 불투명 픽셀 분할 크기와 다르다. 원본 가림 지형을 복원했다고 주장하지 않는다. [contrast/README.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/contrast/README.md)

**글리프 최근접 40쌍** — 12px 설계/24px 실제 마스크를 각각 렌더링했다. 아래 화면 차이는 계산으로 4배 한 추정치가 아니다. 총10,442쌍 실패0. 일반 EN 최소3/KO1, 굵게 EN6/KO2 설계 픽셀이다.

**en regular**

| 왼쪽 | 오른쪽 | 설계 픽셀 차이 | 24px 화면 픽셀 차이 |
|---|---|---|---|
| na | ha | 3 | 12 |
| ni | hi | 3 | 12 |
| no | ho | 3 | 12 |
| nu | hu | 3 | 12 |
| Do | De | 6 | 24 |
| Go | Ge | 6 | 24 |
| do | de | 6 | 24 |
| go | ge | 6 | 24 |
| i | I | 6 | 24 |
| o | e | 6 | 24 |

**ko regular**

| 왼쪽 | 오른쪽 | 설계 픽셀 차이 | 24px 화면 픽셀 차이 |
|---|---|---|---|
| 나 | 니 | 1 | 4 |
| 라 | 리 | 1 | 4 |
| 마 | 미 | 1 | 4 |
| 바 | 비 | 1 | 4 |
| 사 | 시 | 1 | 4 |
| 아 | 이 | 1 | 4 |
| 카 | 키 | 1 | 4 |
| 타 | 티 | 1 | 4 |
| 하 | 히 | 1 | 4 |
| 이 | 야 | 2 | 8 |

**en bold**

| 왼쪽 | 오른쪽 | 설계 픽셀 차이 | 24px 화면 픽셀 차이 |
|---|---|---|---|
| na | ha | 6 | 24 |
| ni | hi | 6 | 24 |
| no | ho | 6 | 24 |
| nu | hu | 6 | 24 |
| Ba | Da | 7 | 28 |
| Ba | Ga | 7 | 28 |
| Bo | Do | 7 | 28 |
| Bo | Go | 7 | 28 |
| ha | ba | 7 | 28 |
| hi | bi | 7 | 28 |

**ko bold**

| 왼쪽 | 오른쪽 | 설계 픽셀 차이 | 24px 화면 픽셀 차이 |
|---|---|---|---|
| 나 | 니 | 2 | 8 |
| 라 | 리 | 2 | 8 |
| 마 | 미 | 2 | 8 |
| 바 | 비 | 2 | 8 |
| 사 | 시 | 2 | 8 |
| 아 | 이 | 2 | 8 |
| 카 | 키 | 2 | 8 |
| 타 | 티 | 2 | 8 |
| 하 | 히 | 2 | 8 |
| 모 | 도 | 4 | 16 |

**사실/이름 판독**

161장(119+6+36), 동결된 3배치. 판독자의 reference 접근은 false로 기록됐으며 161 PNG 및 3답변 SHA를 검증했다. 이벤트 묶음/원자 scalar/관찰 기록의 단위가 달라 통합 정답률을 만들지 않는다.

| 묶음 | 이미지 | 독립검증 정답 | 검증 오답 | 화면 일치지만 독립 미검증 | 별도 미채점 |
|---|---|---|---|---|---|
| 1a | 65 | 665 | 0 | 127 | 판독 유보8/추론2 |
| 1b | 54 | 595 | 0 | 4 | 질적 주석54/viewport16 |
| 2 | 6 | 117 | 0 | 14 | 그림 추론6 |
| 3 | 36 | 624 | 0 | 68 | 그림 추론7 |

검증 가능한 비이름 사실 bar만 PASS. 미검증/화면 밖/추론은 제외한다. 1a의 daytime 처리와 2/3의 screen-only 처리는 원래 판정 차이를 유지했다. 그림 기반 활동 인지는 NOT_ESTABLISHED. 이름은 **1,383회=정확1,323+부분·불확실 정답2+오독56+명시적 대안2**다. 1b의 부분 가시 이름2개는 정확586 안에 이미 포함돼 있다.

| 기대 이름 | 판독 이름 | 오독 횟수 |
|---|---|---|
| Awado | Avado | 1 |
| Baga | Baya | 2 |
| Bubono | Budoho | 1 |
| Buyobi | Buyohi | 1 |
| Dakoha | Dakona | 1 |
| Dedo | Oedo | 4 |
| Dehona | Dehoha | 2 |
| Dehona | Oehona | 2 |
| Demu | Oemu | 8 |
| Denoba | Oenoba | 4 |
| Dotibi | Ootibi | 3 |
| Etuba | Etuha | 1 |
| Iano | Iaho | 1 |
| Inayo | Irayo | 5 |
| Nieba | Neba | 2 |
| Nitiso | Ntiso | 3 |
| Robonu | Roboru | 2 |
| Sogaru | Sogeru | 2 |
| Tanuka | Taruka | 3 |
| Tugami | Tugemi | 1 |
| 부보노 | 부도호 | 1 |
| 부요비 | 부요히 | 1 |
| 소가루 | 소거루 | 2 |
| 이아노 | 이아호 | 2 |
| 투가미 | 투거미 | 1 |

| 판독 묶음 | 이름 발생 | 정확 | 부분·불확실 정답 | 오독 | 명시적 대안 |
|---|---:|---:|---:|---:|---:|
| 1a |424|411|2|11|0|
| 1b |617|586|0|29|2|
| 2 |6|6|0|0|0|
| 3 |336|320|0|16|0|

부분 판독 Simowa/Nuyoe와 대안 Varihi/Warihi를 별도로 보존했다. 고유 인물 수나 독립 표본 수가 아닌 표기 발생 횟수다. [fact-reading/REPORT.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/fact-reading/REPORT.md)

**목표별 최종 재시도/실패 기록**

| 목표 | 실행/입장 횟수 | 결과/제한 |
|---|---|---|
| real-input p13 | baseline1 실패 + targeted repair1 성공 + Part3 gate3회 + final gate1회 | 총6; repair17/17; 마지막 gate120/120·ERROR0 |
| glyph | 1 | PASS |
| H10 도로 | 3 | FAIL 종료; 추가 시도 없음 |
| H10 관계/화해 | 각1 | EN/KO PASS |
| Life estranged | 3 | 앞2 실패; 최종51870 성공 |
| 활동 부족종 보충 | 종별3 | TendFire/Repair 각1장뿐; 종료 |
| write-skip W200 | 입장1/실행1 | VALID FAIL |
| write-skip W500 | 입장3/실행2 | collectorFAIL, prelaunch거부, VALID PASS |
| walker cargo | 2 | 첫 컴파일 실패,57test PASS |
| walker bridge | 2 | 첫 nonempty coverageFAIL, 다음24/24 PASS |
| walker 수치/실호출 fixture | 각1 | 560수치,2000호출/4000records |
| walker identity | 후보2회+병합1회 | 첫 P3 controlFAIL 보존; 후보2·merged compound PASS |
| walker performance | 세계별1 | 각3유효쌍 |
| walker comparator | 3 | 입장/측정 재실행 아님; 3번째 조건수정 |
| Part3 full gate | 1차118/120;2차120/120+ERROR;3차120/120·ERROR0 | 3차 수락; 앞선 실패·ERROR 보존 |
| Part4/final | Part4 terminal29slots; final gate1 PASS | 7collectors=4VALID+3INVALID;21probe prelaunch거부+1wrong-item policy거부; 대기예산 종료 |

최종 장부는 병합·게이트·merged-head identity·after 실행을 포함해 대조했다. 28개 실제 probe sidecar/56개 endpoint와 probe 없는 정책거부1건을 구분한다. 동일성/시각 실패를 runtime exit0으로 지우지 않는다.

### 환경

Mac Apple M4 Max, macOS27.0.1 arm64, Godot4.6 stable(89cea143), Metal4 Forward+, 화면1920×1080/Z4. Python3.14.4. DGX 미사용. 화면 캡처는 window lock, 성능은 quiet/admission lock을 사용했다. Glyph atlas는 headless TextServer 검사이며 창 기반 사실 판독과 구분한다.

- Before release SHA: `b4d29cf526676adc3fa77f21454d2bee79c7865a08b32f398c975e7d8bf6a9f6`.
- Write-skip release SHA: `7c4144a3ab6633568c9a17e565c8634c7b074afa7d43b80034fd0e45e5e2c73b`.
- Walker release SHA: `30ed97544d8fab39146c612c0cf87cd4b59245f427bfba5896fe188e9c6ad042`.
- H10 debug SHA: `ab9be4eda437835c79d0ff7d96f3a7addfd73a52c677bb4a241120e0069267c2` (성능 증거 아님).
- Part4 frozen HEAD: `40aa7a463d740b776248828211251187319c6af6`; release SHA는 위 walker30ed9754와 동일. 2026-10-19T15:29:20.921086Z freeze, main release/measure release/measure copied-debug3개 해시가 일치한다. Import exit0/errors[]이며 walker=1/declump=1/writeSkip=0을 명시했다. W0/W1/W200/W500만 VALID이며 나머지는 NOT_VERIFIED로 종료했다.

### 독립 리뷰

1차 코드·증거 리뷰는 완료했다. W500의 ≥2/3 faster 조건 누락은 comparator3에서 수정됐고 결과는 변하지 않았다. Duplicate-ID 정렬 검증은 고유 ID fixture 범위로 제한하며 임의 중복 ID 전체를 검증했다고 확대하지 않는다. 증거 리뷰는161 PNG/동결 답/요약과 H10 화면을 확인했지만 모든 주장을 다시 독립 판독한 것은 아니다. 2차 코드 리뷰는 P0/P1/P2 없음. 같은 ROUND2 증거 리뷰도 terminal 결과로 종료했고, 검사한 증거에 남은 P1/P2 보고 결함이 없다. 280개 manifest record의 외부280+저장소269=549개 SHA/크기, 네 유효 세계 raw median/p99,29개 validity flag와9487.316187858582초 대기를 독립 확인했다. GOAL §8.2 기술 게시 자격은 인정하지만 H3/H4/H5 FAIL, H8 NOT_VERIFIED, H10 FAIL과 미측정 timers를 유지하는 부분 결과다. 게시 수행은 아래 원격 영수증으로 별도 확인했다. [최종 ROUND2 증거 리뷰](/Users/rexxa/work/art-production13b-evidence/review-round2-evidence.md). [코드 리뷰](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/review/round1/code.md), [증거 리뷰](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/review/round1/evidence.md).

### 자율 결정 목록

명시 GOAL 우선, 기존 ignore/과거 증거 보존, 필요한 도구만 `_p13b`로 복사했다. Tooltip은 cursor warp 없이 입력좌표를 고쳤다. 도로 짐꾼은 세 번 제한에서 종료했고, 활동 표본은 종별 보충3회에서 종료했으나 최초 일반 획득을 포함한 제한 준수는 위 한정을 유지한다. P3 pause-icon924픽셀 제어 차이를 숨기지 않고 입력을 격리했다(발생 입력은 미기록). Seats는 당시 walker 미수락과170–305분 예상 작업량/필수 시간 부족으로 구현 전 보류했다. 후보 쌍을 최종 행렬로 대체하지 않는다. 상세 [DECISIONS.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/DECISIONS.md).

### 실행 기록

12:08Z는 실제 시작이 아니라 보수적 deadline anchor다. GOAL 생성12:10:39Z, 첫 커밋12:10:57Z가 가장 이른 영속 증거다. Part1 마감17:08Z, Part4 시작/freeze 마감19:08Z, 10시간 cutoff22:08Z.

| Part | 확인된 경계 | 시간 보고 |
|---|---|---|
| 0 | 12:40:03Z 커밋까지 완료 기록 | 정확한 실제 종료/소요시간 미확인 |
| 1 | 12:42:45Z 커밋부터 진행 확인;14:11:11.838176Z 완료 checkpoint | 정확한 실제 시작/종료 소요는 미확인 |
| 2 | 12:48:22Z 첫 커밋;13:48:38.995382Z 완료제한 checkpoint | Part1과 겹침; 정확 소요 미확인 |
| 3 | gate3 root audit15:13:04.508215Z PASS; merged capture15:13:29–15:26:34Z, 완료관측15:26:57.646379Z 이후40aa7a46 push | COMPLETE; 정확한 phase 종료 소요는 미산정 |
| 4 | freeze15:29:20.921086Z; terminal audit17:21:02.485487Z | 부분 종료; 관측 구간1시간51분41.564401초(실제 CPU 작업시간 아님) |
| 5 | 2차 코드·증거 리뷰/최종 기술 gate,3159d83c 게시·PR511 갱신·worktree 정리 완료 | 보고 관측 2026-10-19T17:26:40.760393+00:00; 정확 phase 시작/소요 미확인; 자체 보고 커밋은 전달 영수증 참조 |

Part별 실제 소요는 정확한 시작/종료 관측이 없어 임의 계산하지 않았다. Part0 artifact→완료기록29분24초, Part1 진행기록→완료checkpoint1시간28분26.838176초, Part2 첫label→완료checkpoint1시간00분16.995382초는 **문서화된 관측 구간**일 뿐 실제 작업시간이 아니다. Part1/2는 겹치므로 합산하지 않는다.

최종 union 입장 대기는 **9487.316187858582초=158.121936분**(Part1 3600초 포함)이다. 잔여1312.6838121414185초는 필수 다음 cooldown1800초보다 작아 추가 시도를 하지 않았다. 병행 UI/문서 작업과 겹치므로 Part별 벽시계 시간에 더하지 않는다. 과거 장부 snapshot은 이 최종 장부를 대체하지 않는다.

최초 증거 게시 후 실제 `git ls-remote` 결과:

```text
3159d83c77646d2efaba9a0228bb0abcd206b39f refs/heads/codex/art-production-2026-10
3159d83c77646d2efaba9a0228bb0abcd206b39f refs/heads/codex/art-production-2026-10-wip
```

게시 브랜치를57ae7ac40abde7d24369606c9250d3e06e0f18d6에서3159d83c77646d2efaba9a0228bb0abcd206b39f로 fast-forward했고 두 원격 ref가 일치한다. [원격 영수증](/Users/rexxa/work/art-production13b-evidence/remote-heads-first-publish.txt). 동결40aa 이후3159는 문서·증거 변경뿐이며 기술 gate120/120·ERROR0과 runtime 소스 동일성을 유지한다. UI의 이전 검증 head는a5771f1f489084d6e29032cb06365ffb87ff5731, simulation의 이전 조회 head는354e4a13b21a20b28c65d84fab103c40b5143fa0이며 simulation 병합은 하지 않았다.

[PR511](https://github.com/hyunlord/new-world/pull/511)에 Art Pass13b section을 추가하고 readback 본문 일치를 확인했다. OPEN/DRAFT를 유지하며 base `codex/revitalize-2026-10`, head `codex/art-production-2026-10`을 바꾸지 않았다. [PR readback](/Users/rexxa/work/art-production13b-evidence/pr511-after-update.json). 첫 게시 완료 관측은2026-10-19T17:26:08.715167Z다. [게시 감사](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/publication/first-publication-audit.json)는 최종120행·ERROR0,40aa→게시 head runtime diff 없음, 작업용 경로 부재를 재확인했다. 작업용 base/measure worktree 두 개를 제거했고 root와 무관한 audit worktree만 남았다. [정리 영수증](/Users/rexxa/work/art-production13b-evidence/worktrees-after-cleanup.txt). 이 보고서 자체의 Part5 커밋과 후속 fast-forward 최종 head는 생성 후 전달 영수증으로 추가하며 미래 해시를 만들지 않는다.

### 확인된 제한사항

- Stage B 표본12/14 및 Repair0/1, 대비9칸 미측정, 마을요약2/6, H10 도로3회 실패는 미해결 제한이다. 기술 gate가 green이 되어도 이 결과는 바뀌지 않는다.
- 이름56회 오독, 부분/불확실 판독, 원시 reference에 없는 건설·berries·일부 daytime 값은 별도로 남는다. 24px 글리프 구별 가능성과 이름을 모두 정확히 읽음은 다르다.
- Walker 실제 fixture는 고유 ID 작업부하를 검증하며 임의 duplicate-ID 정렬 전체를 검증했다고 확대하지 않는다. 후보 수락·merged-head 동일성·최종 기술 gate 완료; Part4 부분 종료·미측정 NOT_VERIFIED; 3159d83c 증거 게시 완료.
- before W500 running 인구감소는 성능 수치에서 제외한다. after는 부분 종료, 기술 gate는PASS, 최초 증거 게시3159d83c는 완료다. 보고서 자체의 최종 게시 해시는 전달 영수증에 기록한다.

보존된 원본 comparator Markdown 두 곳에는 trailing-empty-line `diff --check` 경고가 있다. SHA provenance를 위해 원문 그대로 유지했으며 **전체 cached diff-check가 clean하다고 주장하지 않는다**. 이 형식 경고와 실행 검증 결과는 별개로 보고한다.

게시 전 제한적 Git 감사(16:21:10.909264Z,HEAD40aa7a46)는 brief 원문 동일, backup8c1b18be, committed1572경로 내 과거증거/ignore 변경없음, Rust변경은 art-owned2소스+4fixture, CampSeats 기준 소스 불변을 확인했다. 미커밋 문서와 최종 성능/게시 완료는 이 감사 범위가 아니다. [prepublication-invariants.json](/Users/rexxa/work/art-production13b-evidence/prepublication-invariants.json).

### 게이트 결과

Part0 **119/120**, `test_real_input_p13` 실패를 보존했다. Part1 repair 실제입력 **17/17**, Life layout p12/p13 및 tooltip windowed 결과 exit0. Rust57test/clippy 및 walker fixture는 해당 범위 통과다. 이것은 merged/final full gate 대신이 아니다. **Part3 gate3 PASS(120/120·ERROR0), merged identity PASS, Part4 네 세계 VALID·나머지 NOT_VERIFIED 종료, final technical gate120/120·ERROR0 PASS, 3159d83c 증거 게시 완료**. 기술 gate 통과와 게시 수행을 별도 증거로 확인했다; 전체 수락 PARTIAL은 유지한다.

Part3 실행 기록: 첫 gate(a5771f1f, session45941)는 **118/120**으로 실패했다. SimReads 정적 경계 검사에서 새 collector의 dynamic dispatch가 걸렸고 meal_ring fixture에는 실제 art bridge가 없었다. 네 동적 호출을 literal renderer 메서드로 바꾸고 meal_ring에 실제 브리지를 공급했으며, 기존 assertion을 유지했다. 대상 검사4/4 및 meal_ring 실패0 확인 후1f6ee3a0으로 WIP에 push했다.

두 번째 gate(1f6ee3a0, session73134)는 **120/120 종료코드, exit0**이지만 `test_agent_drawn_positions`와 `test_cold_presentation` 로그의 missing-bridge ERROR 때문에 **clean gate로 수락하지 않았다**. 두 격리 fixture에 실제 WorldSimNode를 제공한 f5c8951e 이후 개별 재검증은 실패0/ERROR 없음이다. **세 번째 전체 gate는 session30784 exit0, root audit120/120·nonzero[]·log_errors[]로 PASS**다(검사 HEAD f5c8951e, 확인15:13:04.508215Z). 생산 커널이나 assertion을 완화해 통과시킨 것이 아니다. 앞선 두 gate와 오류 로그는 그대로 보존했다.

Merged identity: 캡처 HEAD f5c8951e/session53070/28launch exit0, compound PASS. 총12 RGBA 비교0·다섯 JSONL 쌍 exact, 리더542경로 대조. 캡처 HEAD와 보존·동결 HEAD40aa7a46을 구별한다. 같은 release30ed9754로 Part4를 시작했다.

원본 로그/비교기 Markdown의 EOF 빈 줄 경고는 SHA 보존을 위해 유지했다. 소스·작성 문서 검사 범위는 clean이지만 전체 cached diff-check가 무경고라고 확대하지 않는다. Attempt3 terminal은 W0/W500 새 VALID, W-multi는 exit0이나 종료load8.11865234375>8로 무효, sections3개/ablation은 모두 prelaunch 거부다. 무효 raw를 after에 넣지 않았다. 최종29슬롯 중 collector 실행7회=VALID4+종료부하 무효3, probe 기반 prelaunch 거부21, 잘못 상속된 item 정책거부1이다. Attempt4 재배열 과정에서 `P13_ITEM=after-ablation-W200`가 matrix에 남아 W-multi 파일slot4가 probe/Godot 전에 잘못된 ablation item으로 거부됐다. 이를 실행 조정 오류로 기록했다. 새 shell에서 `P13_ITEM/P13_VALIDITY_OUT`을 명시적으로 unset한 파일slot5(실제 W-multi 네 번째 정상 입장 기회)는 시작load17.5810546875로 거부됐다. 입장 장부를 다시 쓰지 않았고 raw/오류를 보존했다. Attempt4와 recovery supervisor exit0은 측정 통과를 의미하지 않는다. 두 terminal snapshot은 measurement-local 전체 frozen snapshot과 정확히 같았으며655개 cross-checkout 차이 한정은 그대로다.

최종 기술 gate root audit: HEAD40aa7a46, 확인2026-10-19T16:11:30.179086Z,120checks/nonzero[]/log_errors[]/pass=true. 프로세스6436 handle은 더 이상 없지만 process 부재와 supervisor terminal 전체 결과로 종료를 확인했다. [root-audit.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/gates/final/root-audit.json). 기술 gate 통과가 H3/H4·H8/H10 미충족을 바꾸지 않는다.

### 정직한 판단

현재 유효 측정에서71명 W1은 p99 13.415ms로 기준을 통과했고200명은 중앙값17.0395ms로16.67ms 기준을 넘었다. W0 frozen3.188ms는PASS, W500 frozen23.3245ms는FAIL이며 W-multi는 끝내 NOT_VERIFIED로 남았다. 이전 before는71명 W1 p99 13.921ms, 200명은26.145ms로 예산 초과였다. Walker 후보는 W200 교대 실험에서0.6492배(약35.1% 감소)였지만 이는 후보 교대 실험의 효과다. 최종 W200 행렬은0.651731배지만 환경 차이를 포함한 기술적 비율이며 전체 행렬은 부분 결과로 종료했다. 범례가 있어도 부분 진단10/12이며 Repair를 못 읽었으므로 B 통과를 주장할 수 없다. 24px는 기계 글리프 bar와 검증 가능한 사실 판독 범위에서 통과했지만 이름 오독56회를 숨기지 않는다. 게시 브랜치는 최초 게시 영수증상3159d83c이며 PR 갱신과 작업용 worktree 정리도 완료했다. 다음 아트 과제는 가려지지 않는 짐꾼 표현, Repair/Craft/SitByFire 구별, 부족 대비 표본과 이름 판독 개선이며, 이번 종료 제한을 넘는 재시도는 하지 않는다.

ROUND2 코드·증거 리뷰, 최초 게시·원격 확인·PR511 갱신·정리는 종료했다. 이 보고서 자체의 Part5 커밋 해시와 그 게시 영수증만 생성 후 최종 전달문에 추가한다. 이는 실패·미측정 항목을 PASS로 바꾸지 않는다.
