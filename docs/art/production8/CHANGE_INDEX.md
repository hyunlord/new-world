# Art Pass 8 변경 인덱스 — Part 9 준비

- 기준: `a54486e8075c502964fffb48120fd2d61a825d50`
- 캡처: `cefc77b088d6fa3baf21b766cdc40e3fdf045d36`
- 조사 HEAD: `41723f37bd53b6d0dd7c17f90d2b6047117b3119`. 최종 HEAD 확정이 아니다.
- 아트 소유 커밋 경로 위반: **0**. 허용된 시뮬레이션 병합은 별도 분류한다.
- 런타임 지문: 1074개 경로, 캡처/조사 HEAD 동일: **True**.
- SHA-256: `f041a88c7a7daadaac570b87500c3abe2fff9a72fb470ad6b29f5ef97b41f516`.
- 런타임 tracked 작업트리 변경 0개, untracked 0개.

## 실질 변경과 핵심 파일

- Part 0: Pass 12/13 병합, getter fixture 호환성, 실제 날짜·분열 조건 측정과 기준 캡처.
- Part 1: `scripts/ui/panels/god_hand_panel.gd`, `scripts/ui/panels/feed_food_ledger.gd` 중심의 우선순위 피드, 보호 이벤트 누적 +N, 재고 변동 요약 및 getter 이벤트 식별자.
- Part 2: `scripts/ui/offscreen_marker.gd`, `scripts/ui/founding_group_renderer.gd`, `scripts/ui/settlement_labels.gd`: 9px 경계 방향표, 거리, 클릭 이동, 말뚝과 배타 표시.
- Part 3: `scripts/core/settlement_selection.gd`, `scripts/ui/restless_meter.gd`, `world_node.rs`: P/θ 추가 컬럼과 정확한 정수 3단계 계기, 선택 마을 상태.
- Part 4: 좌석 충돌 완화, 아침 냉기 표현과 세계 효과, `frame_measure_session.gd`/`frame_cpu_probe.gd` 동일 프로세스 A30 측정, 단일 마을 레이어 토글.
- Part 5: `capture_plans.gd`/`capture_framings.gd`의 측정 날짜 재매핑, P/G 별도 리셋과 원시 getter 메타데이터.

## 커밋별 변경 파일

### Part 0 — `88f5a907d2a3345ef444074f7549cd1e4b2ecc5b`

art-p8(p0): preserve the execution and evidence contract

분류: `art_owned`

- `docs/art/production8/GOAL.md`

### Part 0 — `50a12830161407eebc130853cc955995dccc4346`

art-p8(p0): bring in simulation passes 12 and 13

분류: `authorized_simulation_merge`

- `docs/revitalize/DAUGHTER_DIAGNOSIS.md`
- `docs/revitalize/DECISIONS.md`
- `docs/revitalize/METRICS.md`
- `docs/revitalize/PASS12_GOAL.md`
- `docs/revitalize/PASS13_GOAL.md`
- `docs/revitalize/REPORT_PASS12.md`
- `docs/revitalize/TEST_LEDGER.md`
- `docs/revitalize/probe/p12-0_daughters/DAUGHTER_DIAGNOSIS_TABLES.md`
- `docs/revitalize/probe/p12-0_daughters/daughter_diag.json`
- `docs/revitalize/probe/p12-0_legacy/summary.json`
- `docs/revitalize/probe/p12-0_legacy/summary.md`
- `docs/revitalize/probe/p12-0_ubs/p10_pilot/ubs.json`
- `docs/revitalize/probe/p12-0_ubs/p10_pilot/ubs.md`
- `docs/revitalize/probe/p12-0_ubs/p11_pilot/ubs.json`
- `docs/revitalize/probe/p12-0_ubs/p11_pilot/ubs.md`
- `docs/revitalize/probe/p12-0_v15_10day/summary.json`
- `docs/revitalize/probe/p12-0_v15_10day/summary.md`
- `docs/revitalize/probe/p12-0_v15_120day/summary.json`
- `docs/revitalize/probe/p12-0_v15_120day/summary.md`
- `docs/revitalize/probe/p12-0_v16_10day/summary.json`
- `docs/revitalize/probe/p12-0_v16_10day/summary.md`
- `docs/revitalize/probe/p12-0_v16_120day/summary.json`
- `docs/revitalize/probe/p12-0_v16_120day/summary.md`
- `docs/revitalize/probe/p12-0_v17_10day/summary.json`
- `docs/revitalize/probe/p12-0_v17_10day/summary.md`
- `docs/revitalize/probe/p12-0_v17_120day/summary.json`
- `docs/revitalize/probe/p12-0_v17_120day/summary.md`
- `docs/revitalize/probe/p12-0_v17n_10day/summary.json`
- `docs/revitalize/probe/p12-0_v17n_10day/summary.md`
- `docs/revitalize/probe/p12-0_v17n_120day/summary.json`
- `docs/revitalize/probe/p12-0_v17n_120day/summary.md`
- `docs/revitalize/probe/p12-1_abl_local_food2_120day/summary.json`
- `docs/revitalize/probe/p12-1_abl_local_food2_120day/summary.md`
- `docs/revitalize/probe/p12-1_abl_local_food3_120day/summary.json`
- `docs/revitalize/probe/p12-1_abl_local_food3_120day/summary.md`
- `docs/revitalize/probe/p12-1_abl_local_food_120day/summary.json`
- `docs/revitalize/probe/p12-1_abl_local_food_120day/summary.md`
- `docs/revitalize/probe/p12-1_abl_no_hardiness_120day/summary.json`
- `docs/revitalize/probe/p12-1_abl_no_hardiness_120day/summary.md`
- `docs/revitalize/probe/p12-1_abl_no_hunger_weight_120day/summary.json`
- `docs/revitalize/probe/p12-1_abl_no_hunger_weight_120day/summary.md`
- `docs/revitalize/probe/p12-1_abl_no_rain_soak_120day/summary.json`
- `docs/revitalize/probe/p12-1_abl_no_rain_soak_120day/summary.md`
- `docs/revitalize/probe/p12-1_abl_no_regional_wood_120day/summary.json`
- `docs/revitalize/probe/p12-1_abl_no_regional_wood_120day/summary.md`
- `docs/revitalize/probe/p12-1_abl_no_regions_120day/summary.json`
- `docs/revitalize/probe/p12-1_abl_no_regions_120day/summary.md`
- `docs/revitalize/probe/p12-1_abl_v16_damage_120day/summary.json`
- `docs/revitalize/probe/p12-1_abl_v16_damage_120day/summary.md`
- `docs/revitalize/probe/p12-1_abl_v16_exposure_120day/summary.json`
- `docs/revitalize/probe/p12-1_abl_v16_exposure_120day/summary.md`
- `docs/revitalize/probe/p12-1_bisect/BISECTION.md`
- `docs/revitalize/probe/p12-1_bisect/bisect.json`
- `docs/revitalize/probe/p12-1_bisect/bisect.md`
- `docs/revitalize/probe/p12-1_fwd_exposure_v2_120day/summary.json`
- `docs/revitalize/probe/p12-1_fwd_exposure_v2_120day/summary.md`
- `docs/revitalize/probe/p12-1_fwd_hardiness_120day/summary.json`
- `docs/revitalize/probe/p12-1_fwd_hardiness_120day/summary.md`
- `docs/revitalize/probe/p12-1_fwd_hunger_weight_120day/summary.json`
- `docs/revitalize/probe/p12-1_fwd_hunger_weight_120day/summary.md`
- `docs/revitalize/probe/p12-1_fwd_regions_120day/summary.json`
- `docs/revitalize/probe/p12-1_fwd_regions_120day/summary.md`
- `docs/revitalize/probe/p12-1_fwd_regions_bushes_only_120day/summary.json`
- `docs/revitalize/probe/p12-1_fwd_regions_bushes_only_120day/summary.md`
- `docs/revitalize/probe/p12-1_fwd_regions_regional_wood_120day/summary.json`
- `docs/revitalize/probe/p12-1_fwd_regions_regional_wood_120day/summary.md`
- `docs/revitalize/probe/p12-1_fwd_regions_wood_only_120day/summary.json`
- `docs/revitalize/probe/p12-1_fwd_regions_wood_only_120day/summary.md`
- `docs/revitalize/probe/p12-1_fwd_v17_damage_120day/summary.json`
- `docs/revitalize/probe/p12-1_fwd_v17_damage_120day/summary.md`
- `docs/revitalize/probe/p12-1_game_v16_120day/summary.json`
- `docs/revitalize/probe/p12-1_game_v16_120day/summary.md`
- `docs/revitalize/probe/p12-1_game_v17_nofission_120day/summary.json`
- `docs/revitalize/probe/p12-1_game_v17_nofission_120day/summary.md`
- `docs/revitalize/probe/p12-2_guard_v16_10day/summary.json`
- `docs/revitalize/probe/p12-2_guard_v16_10day/summary.md`
- `docs/revitalize/probe/p12-2_guard_v16_120day/summary.json`
- `docs/revitalize/probe/p12-2_guard_v16_120day/summary.md`
- `docs/revitalize/probe/p12-2_guard_v17_10day/summary.json`
- `docs/revitalize/probe/p12-2_guard_v17_10day/summary.md`
- `docs/revitalize/probe/p12-2_guard_v17_120day/summary.json`
- `docs/revitalize/probe/p12-2_guard_v17_120day/summary.md`
- `docs/revitalize/probe/p12-2_guard_v17n_10day/summary.json`
- `docs/revitalize/probe/p12-2_guard_v17n_10day/summary.md`
- `docs/revitalize/probe/p12-2_guard_v17n_120day/summary.json`
- `docs/revitalize/probe/p12-2_guard_v17n_120day/summary.md`
- `docs/revitalize/probe/p12-3_guard_v16_10day/summary.json`
- `docs/revitalize/probe/p12-3_guard_v16_10day/summary.md`
- `docs/revitalize/probe/p12-3_guard_v16_120day/summary.json`
- `docs/revitalize/probe/p12-3_guard_v16_120day/summary.md`
- `docs/revitalize/probe/p12-3_guard_v17_10day/summary.json`
- `docs/revitalize/probe/p12-3_guard_v17_10day/summary.md`
- `docs/revitalize/probe/p12-3_guard_v17_120day/summary.json`
- `docs/revitalize/probe/p12-3_guard_v17_120day/summary.md`
- `docs/revitalize/probe/p12-3_guard_v17n_10day/summary.json`
- `docs/revitalize/probe/p12-3_guard_v17n_10day/summary.md`
- `docs/revitalize/probe/p12-3_guard_v17n_120day/summary.json`
- `docs/revitalize/probe/p12-3_guard_v17n_120day/summary.md`
- `docs/revitalize/probe/p12-3_round1_v18_10day/summary.json`
- `docs/revitalize/probe/p12-3_round1_v18_10day/summary.md`
- `docs/revitalize/probe/p12-3_round1_v18_120day/summary.json`
- `docs/revitalize/probe/p12-3_round1_v18_120day/summary.md`
- `docs/revitalize/probe/p12-3_round1_v18_targets/j_targets.json`
- `docs/revitalize/probe/p12-3_round1_v18_targets/j_targets.md`
- `docs/revitalize/probe/p12-3_round1_v18n_10day/summary.json`
- `docs/revitalize/probe/p12-3_round1_v18n_10day/summary.md`
- `docs/revitalize/probe/p12-3_round1_v18n_120day/summary.json`
- `docs/revitalize/probe/p12-3_round1_v18n_120day/summary.md`
- `docs/revitalize/probe/p12-3_round1_v18n_targets/j_targets.json`
- `docs/revitalize/probe/p12-3_round1_v18n_targets/j_targets.md`
- `docs/revitalize/probe/p12-3_round2_v18_10day/summary.json`
- `docs/revitalize/probe/p12-3_round2_v18_10day/summary.md`
- `docs/revitalize/probe/p12-3_round2_v18_120day/summary.json`
- `docs/revitalize/probe/p12-3_round2_v18_120day/summary.md`
- `docs/revitalize/probe/p12-3_round2_v18_targets/j_targets.json`
- `docs/revitalize/probe/p12-3_round2_v18_targets/j_targets.md`
- `docs/revitalize/probe/p12-3_round2_v18n_10day/summary.json`
- `docs/revitalize/probe/p12-3_round2_v18n_10day/summary.md`
- `docs/revitalize/probe/p12-3_round2_v18n_120day/summary.json`
- `docs/revitalize/probe/p12-3_round2_v18n_120day/summary.md`
- `docs/revitalize/probe/p12-3_round2_v18n_targets/j_targets.json`
- `docs/revitalize/probe/p12-3_round2_v18n_targets/j_targets.md`
- `docs/revitalize/probe/p12-3_round3_v18_10day/summary.json`
- `docs/revitalize/probe/p12-3_round3_v18_10day/summary.md`
- `docs/revitalize/probe/p12-3_round3_v18_120day/summary.json`
- `docs/revitalize/probe/p12-3_round3_v18_120day/summary.md`
- `docs/revitalize/probe/p12-3_round3_v18_targets/j_targets.json`
- `docs/revitalize/probe/p12-3_round3_v18_targets/j_targets.md`
- `docs/revitalize/probe/p12-3_round3_v18n_10day/summary.json`
- `docs/revitalize/probe/p12-3_round3_v18n_10day/summary.md`
- `docs/revitalize/probe/p12-3_round3_v18n_120day/summary.json`
- `docs/revitalize/probe/p12-3_round3_v18n_120day/summary.md`
- `docs/revitalize/probe/p12-3_round3_v18n_targets/j_targets.json`
- `docs/revitalize/probe/p12-3_round3_v18n_targets/j_targets.md`
- `docs/revitalize/probe/p12-4_heldout_v18_10day/summary.json`
- `docs/revitalize/probe/p12-4_heldout_v18_10day/summary.md`
- `docs/revitalize/probe/p12-4_heldout_v18_120day/summary.json`
- `docs/revitalize/probe/p12-4_heldout_v18_120day/summary.md`
- `docs/revitalize/probe/p12-4_heldout_v18_targets/j_targets.json`
- `docs/revitalize/probe/p12-4_heldout_v18_targets/j_targets.md`
- `docs/revitalize/probe/p12-4_heldout_v18n_10day/summary.json`
- `docs/revitalize/probe/p12-4_heldout_v18n_10day/summary.md`
- `docs/revitalize/probe/p12-4_heldout_v18n_120day/summary.json`
- `docs/revitalize/probe/p12-4_heldout_v18n_120day/summary.md`
- `docs/revitalize/probe/p12-4_heldout_v18n_targets/j_targets.json`
- `docs/revitalize/probe/p12-4_heldout_v18n_targets/j_targets.md`
- `docs/revitalize/probe/p12-5_pilot/check_v1/check_v1.json`
- `docs/revitalize/probe/p12-5_pilot/commands.md`
- `docs/revitalize/probe/p12-5_pilot/diversity.json`
- `docs/revitalize/probe/p12-5_pilot/diversity.md`
- `docs/revitalize/probe/p12-5_pilot/p11_metrics/diversity.json`
- `docs/revitalize/probe/p12-5_pilot/p11_metrics/diversity.md`
- `docs/revitalize/probe/p12-5_pilot/p5_pilot.sh`
- `docs/revitalize/probe/p12-5_pilot/ubs/ubs.json`
- `docs/revitalize/probe/p12-5_pilot/ubs/ubs.md`
- `docs/revitalize/probe/p12-6_forks/commands.md`
- `docs/revitalize/probe/p12-6_forks/fork_extras.json`
- `docs/revitalize/probe/p12-6_forks/fork_extras.md`
- `docs/revitalize/probe/p12-6_forks/fork_metrics.json`
- `docs/revitalize/probe/p12-6_forks/fork_metrics.md`
- `docs/revitalize/probe/p12-6_forks/p12_forks.sh`
- `docs/revitalize/probe/p12-6_forks/p12_macro.sh`
- `docs/revitalize/probe/p12-6_macro/macro.json`
- `docs/revitalize/probe/p12-6_macro/macro.md`
- `docs/revitalize/probe/p12-7_final_forks/fork_extras.json`
- `docs/revitalize/probe/p12-7_final_forks/fork_extras.md`
- `docs/revitalize/probe/p12-7_final_forks/fork_metrics.json`
- `docs/revitalize/probe/p12-7_final_forks/fork_metrics.md`
- `docs/revitalize/probe/p12-7_final_heldout_v17_10day/summary.json`
- `docs/revitalize/probe/p12-7_final_heldout_v17_10day/summary.md`
- `docs/revitalize/probe/p12-7_final_heldout_v17_120day/summary.json`
- `docs/revitalize/probe/p12-7_final_heldout_v17_120day/summary.md`
- `docs/revitalize/probe/p12-7_final_heldout_v17n_10day/summary.json`
- `docs/revitalize/probe/p12-7_final_heldout_v17n_10day/summary.md`
- `docs/revitalize/probe/p12-7_final_heldout_v17n_120day/summary.json`
- `docs/revitalize/probe/p12-7_final_heldout_v17n_120day/summary.md`
- `docs/revitalize/probe/p12-7_final_heldout_v18_10day/summary.json`
- `docs/revitalize/probe/p12-7_final_heldout_v18_10day/summary.md`
- `docs/revitalize/probe/p12-7_final_heldout_v18_120day/summary.json`
- `docs/revitalize/probe/p12-7_final_heldout_v18_120day/summary.md`
- `docs/revitalize/probe/p12-7_final_heldout_v18_targets/j_targets.json`
- `docs/revitalize/probe/p12-7_final_heldout_v18_targets/j_targets.md`
- `docs/revitalize/probe/p12-7_final_heldout_v18n_10day/summary.json`
- `docs/revitalize/probe/p12-7_final_heldout_v18n_10day/summary.md`
- `docs/revitalize/probe/p12-7_final_heldout_v18n_120day/summary.json`
- `docs/revitalize/probe/p12-7_final_heldout_v18n_120day/summary.md`
- `docs/revitalize/probe/p12-7_final_heldout_v18n_targets/j_targets.json`
- `docs/revitalize/probe/p12-7_final_heldout_v18n_targets/j_targets.md`
- `docs/revitalize/probe/p12-7_final_legacy/summary.json`
- `docs/revitalize/probe/p12-7_final_legacy/summary.md`
- `docs/revitalize/probe/p12-7_final_macro/macro.json`
- `docs/revitalize/probe/p12-7_final_macro/macro.md`
- `docs/revitalize/probe/p12-7_final_pilot/diversity.json`
- `docs/revitalize/probe/p12-7_final_pilot/diversity.md`
- `docs/revitalize/probe/p12-7_final_pilot/p11_metrics/diversity.json`
- `docs/revitalize/probe/p12-7_final_pilot/p11_metrics/diversity.md`
- `docs/revitalize/probe/p12-7_final_pilot/ubs/ubs.json`
- `docs/revitalize/probe/p12-7_final_pilot/ubs/ubs.md`
- `docs/revitalize/probe/p12-7_final_tuning_v18_targets/j_targets.json`
- `docs/revitalize/probe/p12-7_final_tuning_v18_targets/j_targets.md`
- `docs/revitalize/probe/p12-7_final_tuning_v18n_targets/j_targets.json`
- `docs/revitalize/probe/p12-7_final_tuning_v18n_targets/j_targets.md`
- `docs/revitalize/probe/p12-7_final_v10_10day/summary.json`
- `docs/revitalize/probe/p12-7_final_v10_10day/summary.md`
- `docs/revitalize/probe/p12-7_final_v10_120day/summary.json`
- `docs/revitalize/probe/p12-7_final_v10_120day/summary.md`
- `docs/revitalize/probe/p12-7_final_v11_10day/summary.json`
- `docs/revitalize/probe/p12-7_final_v11_10day/summary.md`
- `docs/revitalize/probe/p12-7_final_v11_120day/summary.json`
- `docs/revitalize/probe/p12-7_final_v11_120day/summary.md`
- `docs/revitalize/probe/p12-7_final_v12_10day/summary.json`
- `docs/revitalize/probe/p12-7_final_v12_10day/summary.md`
- `docs/revitalize/probe/p12-7_final_v12_120day/summary.json`
- `docs/revitalize/probe/p12-7_final_v12_120day/summary.md`
- `docs/revitalize/probe/p12-7_final_v13_10day/summary.json`
- `docs/revitalize/probe/p12-7_final_v13_10day/summary.md`
- `docs/revitalize/probe/p12-7_final_v13_120day/summary.json`
- `docs/revitalize/probe/p12-7_final_v13_120day/summary.md`
- `docs/revitalize/probe/p12-7_final_v14_10day/summary.json`
- `docs/revitalize/probe/p12-7_final_v14_10day/summary.md`
- `docs/revitalize/probe/p12-7_final_v14_120day/summary.json`
- `docs/revitalize/probe/p12-7_final_v14_120day/summary.md`
- `docs/revitalize/probe/p12-7_final_v15_10day/summary.json`
- `docs/revitalize/probe/p12-7_final_v15_10day/summary.md`
- `docs/revitalize/probe/p12-7_final_v15_120day/summary.json`
- `docs/revitalize/probe/p12-7_final_v15_120day/summary.md`
- `docs/revitalize/probe/p12-7_final_v16_10day/summary.json`
- `docs/revitalize/probe/p12-7_final_v16_10day/summary.md`
- `docs/revitalize/probe/p12-7_final_v16_120day/summary.json`
- `docs/revitalize/probe/p12-7_final_v16_120day/summary.md`
- `docs/revitalize/probe/p12-7_final_v16n_10day/summary.json`
- `docs/revitalize/probe/p12-7_final_v16n_10day/summary.md`
- `docs/revitalize/probe/p12-7_final_v16n_120day/summary.json`
- `docs/revitalize/probe/p12-7_final_v16n_120day/summary.md`
- `docs/revitalize/probe/p12-7_final_v17_10day/summary.json`
- `docs/revitalize/probe/p12-7_final_v17_10day/summary.md`
- `docs/revitalize/probe/p12-7_final_v17_120day/summary.json`
- `docs/revitalize/probe/p12-7_final_v17_120day/summary.md`
- `docs/revitalize/probe/p12-7_final_v17n_10day/summary.json`
- `docs/revitalize/probe/p12-7_final_v17n_10day/summary.md`
- `docs/revitalize/probe/p12-7_final_v17n_120day/summary.json`
- `docs/revitalize/probe/p12-7_final_v17n_120day/summary.md`
- `docs/revitalize/probe/p12-7_final_v8/summary.json`
- `docs/revitalize/probe/p12-7_final_v8/summary.md`
- `docs/revitalize/probe/p12-7_final_v9_10day/summary.json`
- `docs/revitalize/probe/p12-7_final_v9_10day/summary.md`
- `docs/revitalize/probe/p12-7_final_v9_120day/summary.json`
- `docs/revitalize/probe/p12-7_final_v9_120day/summary.md`
- `docs/revitalize/probe/p12-7_timing_v16_10day/summary.json`
- `docs/revitalize/probe/p12-7_timing_v16_10day/summary.md`
- `docs/revitalize/probe/p12-7_timing_v16_120day/summary.json`
- `docs/revitalize/probe/p12-7_timing_v16_120day/summary.md`
- `docs/revitalize/probe/p12-7_timing_v16b_10day/summary.json`
- `docs/revitalize/probe/p12-7_timing_v16b_10day/summary.md`
- `docs/revitalize/probe/p12-7_timing_v16b_120day/summary.json`
- `docs/revitalize/probe/p12-7_timing_v16b_120day/summary.md`
- `docs/revitalize/probe/p12-7_timing_v18_10day/summary.json`
- `docs/revitalize/probe/p12-7_timing_v18_10day/summary.md`
- `docs/revitalize/probe/p12-7_timing_v18_120day/summary.json`
- `docs/revitalize/probe/p12-7_timing_v18_120day/summary.md`
- `docs/revitalize/probe/p12-7_timing_v18n_10day/summary.json`
- `docs/revitalize/probe/p12-7_timing_v18n_10day/summary.md`
- `docs/revitalize/probe/p12-7_timing_v18n_120day/summary.json`
- `docs/revitalize/probe/p12-7_timing_v18n_120day/summary.md`
- `docs/revitalize/probe/p13-0_j5/j5_diag.json`
- `docs/revitalize/probe/p13-0_j5/j5_diag.md`
- `docs/revitalize/probe/p13-0_legacy/summary.json`
- `docs/revitalize/probe/p13-0_legacy/summary.md`
- `docs/revitalize/probe/p13-0_paths/commands.md`
- `docs/revitalize/probe/p13-0_paths/paths.json`
- `docs/revitalize/probe/p13-0_paths/paths.md`
- `docs/revitalize/probe/p13-0_paths/ubs_r0/ubs.json`
- `docs/revitalize/probe/p13-0_paths/ubs_r0/ubs.md`
- `docs/revitalize/probe/p13-0_v15_10day/summary.json`
- `docs/revitalize/probe/p13-0_v15_10day/summary.md`
- `docs/revitalize/probe/p13-0_v15_120day/summary.json`
- `docs/revitalize/probe/p13-0_v15_120day/summary.md`
- `docs/revitalize/probe/p13-0_v16_10day/summary.json`
- `docs/revitalize/probe/p13-0_v16_10day/summary.md`
- `docs/revitalize/probe/p13-0_v16_120day/summary.json`
- `docs/revitalize/probe/p13-0_v16_120day/summary.md`
- `docs/revitalize/probe/p13-0_v17_10day/summary.json`
- `docs/revitalize/probe/p13-0_v17_10day/summary.md`
- `docs/revitalize/probe/p13-0_v17_120day/summary.json`
- `docs/revitalize/probe/p13-0_v17_120day/summary.md`
- `docs/revitalize/probe/p13-0_v18_10day/summary.json`
- `docs/revitalize/probe/p13-0_v18_10day/summary.md`
- `docs/revitalize/probe/p13-0_v18_120day/summary.json`
- `docs/revitalize/probe/p13-0_v18_120day/summary.md`
- `docs/revitalize/probe/p13-0_v18n_10day/summary.json`
- `docs/revitalize/probe/p13-0_v18n_10day/summary.md`
- `docs/revitalize/probe/p13-0_v18n_120day/summary.json`
- `docs/revitalize/probe/p13-0_v18n_120day/summary.md`
- `docs/revitalize/probe/p13-1_abl18_no_area_floor_120day/summary.json`
- `docs/revitalize/probe/p13-1_abl18_no_area_floor_120day/summary.md`
- `docs/revitalize/probe/p13-1_abl18_no_hardiness_120day/summary.json`
- `docs/revitalize/probe/p13-1_abl18_no_hardiness_120day/summary.md`
- `docs/revitalize/probe/p13-1_abl18_no_hunger_weight_120day/summary.json`
- `docs/revitalize/probe/p13-1_abl18_no_hunger_weight_120day/summary.md`
- `docs/revitalize/probe/p13-1_abl18_no_region_bushes_120day/summary.json`
- `docs/revitalize/probe/p13-1_abl18_no_region_bushes_120day/summary.md`
- `docs/revitalize/probe/p13-1_abl18_no_region_deadwood_120day/summary.json`
- `docs/revitalize/probe/p13-1_abl18_no_region_deadwood_120day/summary.md`
- `docs/revitalize/probe/p13-1_abl18_no_regional_wood_120day/summary.json`
- `docs/revitalize/probe/p13-1_abl18_no_regional_wood_120day/summary.md`
- `docs/revitalize/probe/p13-1_abl18_no_regions_120day/summary.json`
- `docs/revitalize/probe/p13-1_abl18_no_regions_120day/summary.md`
- `docs/revitalize/probe/p13-1_abl18_no_wet_burn_120day/summary.json`
- `docs/revitalize/probe/p13-1_abl18_no_wet_burn_120day/summary.md`
- `docs/revitalize/probe/p13-1_abl18_v16_damage_120day/summary.json`
- `docs/revitalize/probe/p13-1_abl18_v16_damage_120day/summary.md`
- `docs/revitalize/probe/p13-1_abl18_v16_exposure_120day/summary.json`
- `docs/revitalize/probe/p13-1_abl18_v16_exposure_120day/summary.md`
- `docs/revitalize/probe/p13-1_j6/bisect.json`
- `docs/revitalize/probe/p13-1_j6/bisect.md`
- `docs/revitalize/probe/p13-2_guard_v16_10day/summary.json`
- `docs/revitalize/probe/p13-2_guard_v16_10day/summary.md`
- `docs/revitalize/probe/p13-2_guard_v16_120day/summary.json`
- `docs/revitalize/probe/p13-2_guard_v16_120day/summary.md`
- `docs/revitalize/probe/p13-2_guard_v17_10day/summary.json`
- `docs/revitalize/probe/p13-2_guard_v17_10day/summary.md`
- `docs/revitalize/probe/p13-2_guard_v17_120day/summary.json`
- `docs/revitalize/probe/p13-2_guard_v17_120day/summary.md`
- `docs/revitalize/probe/p13-2_guard_v18_10day/summary.json`
- `docs/revitalize/probe/p13-2_guard_v18_10day/summary.md`
- `docs/revitalize/probe/p13-2_guard_v18_120day/summary.json`
- `docs/revitalize/probe/p13-2_guard_v18_120day/summary.md`
- `docs/revitalize/probe/p13-2_guard_v18n_10day/summary.json`
- `docs/revitalize/probe/p13-2_guard_v18n_10day/summary.md`
- `docs/revitalize/probe/p13-2_guard_v18n_120day/summary.json`
- `docs/revitalize/probe/p13-2_guard_v18n_120day/summary.md`
- `docs/revitalize/probe/p13-3_round1_macro/macro.json`
- `docs/revitalize/probe/p13-3_round1_macro/macro.md`
- `docs/revitalize/probe/p13-3_round1_v19_10day/summary.json`
- `docs/revitalize/probe/p13-3_round1_v19_10day/summary.md`
- `docs/revitalize/probe/p13-3_round1_v19_120day/summary.json`
- `docs/revitalize/probe/p13-3_round1_v19_120day/summary.md`
- `docs/revitalize/probe/p13-3_round1_v19_targets/j_targets.json`
- `docs/revitalize/probe/p13-3_round1_v19_targets/j_targets.md`
- `docs/revitalize/probe/p13-3_round1_v19n_10day/summary.json`
- `docs/revitalize/probe/p13-3_round1_v19n_10day/summary.md`
- `docs/revitalize/probe/p13-3_round1_v19n_120day/summary.json`
- `docs/revitalize/probe/p13-3_round1_v19n_120day/summary.md`
- `docs/revitalize/probe/p13-3_round1_v19n_targets/j_targets.json`
- `docs/revitalize/probe/p13-3_round1_v19n_targets/j_targets.md`
- `docs/revitalize/probe/p13-3_round2_macro/macro.json`
- `docs/revitalize/probe/p13-3_round2_macro/macro.md`
- `docs/revitalize/probe/p13-3_round2_v19_10day/summary.json`
- `docs/revitalize/probe/p13-3_round2_v19_10day/summary.md`
- `docs/revitalize/probe/p13-3_round2_v19_120day/summary.json`
- `docs/revitalize/probe/p13-3_round2_v19_120day/summary.md`
- `docs/revitalize/probe/p13-3_round2_v19_targets/j_targets.json`
- `docs/revitalize/probe/p13-3_round2_v19_targets/j_targets.md`
- `docs/revitalize/probe/p13-3_round2_v19n_10day/summary.json`
- `docs/revitalize/probe/p13-3_round2_v19n_10day/summary.md`
- `docs/revitalize/probe/p13-3_round2_v19n_120day/summary.json`
- `docs/revitalize/probe/p13-3_round2_v19n_120day/summary.md`
- `docs/revitalize/probe/p13-3_round2_v19n_targets/j_targets.json`
- `docs/revitalize/probe/p13-3_round2_v19n_targets/j_targets.md`
- `docs/revitalize/probe/p13-3_round3_macro/macro.json`
- `docs/revitalize/probe/p13-3_round3_macro/macro.md`
- `docs/revitalize/probe/p13-3_round3_v19_10day/summary.json`
- `docs/revitalize/probe/p13-3_round3_v19_10day/summary.md`
- `docs/revitalize/probe/p13-3_round3_v19_120day/summary.json`
- `docs/revitalize/probe/p13-3_round3_v19_120day/summary.md`
- `docs/revitalize/probe/p13-3_round3_v19_targets/j_targets.json`
- `docs/revitalize/probe/p13-3_round3_v19_targets/j_targets.md`
- `docs/revitalize/probe/p13-3_round3_v19n_10day/summary.json`
- `docs/revitalize/probe/p13-3_round3_v19n_10day/summary.md`
- `docs/revitalize/probe/p13-3_round3_v19n_120day/summary.json`
- `docs/revitalize/probe/p13-3_round3_v19n_120day/summary.md`
- `docs/revitalize/probe/p13-3_round3_v19n_targets/j_targets.json`
- `docs/revitalize/probe/p13-3_round3_v19n_targets/j_targets.md`
- `docs/revitalize/probe/p13-4_heldout_v19_10day/summary.json`
- `docs/revitalize/probe/p13-4_heldout_v19_10day/summary.md`
- `docs/revitalize/probe/p13-4_heldout_v19_120day/summary.json`
- `docs/revitalize/probe/p13-4_heldout_v19_120day/summary.md`
- `docs/revitalize/probe/p13-4_heldout_v19_targets/j_targets.json`
- `docs/revitalize/probe/p13-4_heldout_v19_targets/j_targets.md`
- `docs/revitalize/probe/p13-4_heldout_v19n_10day/summary.json`
- `docs/revitalize/probe/p13-4_heldout_v19n_10day/summary.md`
- `docs/revitalize/probe/p13-4_heldout_v19n_120day/summary.json`
- `docs/revitalize/probe/p13-4_heldout_v19n_120day/summary.md`
- `docs/revitalize/probe/p13-4_heldout_v19n_targets/j_targets.json`
- `docs/revitalize/probe/p13-4_heldout_v19n_targets/j_targets.md`
- `docs/revitalize/probe/p13-5_pilot/check_v1/check_v1.json`
- `docs/revitalize/verification/p12-2/gate.txt`
- `docs/revitalize/verification/p12-7/gate.txt`
- `docs/revitalize/verification/p12-7/smoke_errors.log`
- `docs/revitalize/verification/p13-2/gate.txt`
- `rust/crates/sim-bridge/src/ffi/world_node.rs`
- `rust/crates/sim-core/src/components/settlement.rs`
- `rust/crates/sim-core/src/fission_threshold.rs`
- `rust/crates/sim-core/src/lib.rs`
- `rust/crates/sim-core/src/rng.rs`
- `rust/crates/sim-core/src/rules.rs`
- `rust/crates/sim-core/src/variety.rs`
- `rust/crates/sim-engine/src/lib.rs`
- `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs`
- `rust/crates/sim-systems/src/runtime/resource_regen/mod.rs`
- `rust/crates/sim-systems/src/runtime/settlement/fire.rs`
- `rust/crates/sim-systems/src/runtime/settlement/firewood.rs`
- `rust/crates/sim-systems/src/runtime/settlement/fission.rs`
- `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs`
- `rust/crates/sim-systems/src/worldgen/camp.rs`
- `rust/crates/sim-systems/src/worldgen/mod.rs`
- `rust/crates/sim-test/src/probe/ablation.rs`
- `rust/crates/sim-test/src/probe/args.rs`
- `rust/crates/sim-test/src/probe/metrics.rs`
- `rust/crates/sim-test/src/probe/mod.rs`
- `rust/crates/sim-test/src/probe/pass12.rs`
- `rust/crates/sim-test/src/probe/pass13.rs`
- `rust/crates/sim-test/src/probe/runner.rs`
- `rust/crates/sim-test/src/probe/summary.rs`
- `rust/crates/sim-test/src/scenes.rs`
- `rust/crates/sim-test/src/worldmodel/export.rs`
- `rust/crates/sim-test/src/worldmodel/schema.rs`
- `rust/crates/sim-test/tests/harness_revitalize_pass10_forks.rs`
- `rust/crates/sim-test/tests/harness_revitalize_pass10_frozen.rs`
- `rust/crates/sim-test/tests/harness_revitalize_pass11_frozen.rs`
- `rust/crates/sim-test/tests/harness_revitalize_pass12_fission.rs`
- `rust/crates/sim-test/tests/harness_revitalize_pass12_frozen.rs`
- `rust/crates/sim-test/tests/harness_revitalize_pass12_world.rs`
- `rust/crates/sim-test/tests/harness_revitalize_pass13_fission.rs`
- `rust/crates/sim-test/tests/harness_revitalize_pass13_frozen.rs`
- `rust/crates/sim-test/tests/harness_revitalize_pass13_world.rs`
- `rust/crates/sim-test/tests/harness_revitalize_pass6_frozen.rs`
- `rust/crates/sim-test/tests/harness_revitalize_pass7_frozen.rs`
- `rust/crates/sim-test/tests/harness_revitalize_pass8_frozen.rs`
- `rust/crates/sim-test/tests/harness_revitalize_pass9_frozen.rs`
- `rust/crates/sim-test/tests/harness_revitalize_tallies.rs`
- `tools/revitalize/bisect_p12.py`
- `tools/revitalize/bisect_p13.py`
- `tools/revitalize/daughter_diag.py`
- `tools/revitalize/diversity_p12.py`
- `tools/revitalize/evaluate_pass12.py`
- `tools/revitalize/evaluate_pass13.py`
- `tools/revitalize/exp_table_p13.py`
- `tools/revitalize/fork_extras_p12.py`
- `tools/revitalize/gate_pass12.sh`
- `tools/revitalize/gate_pass13.sh`
- `tools/revitalize/j5_diag_p13.py`
- `tools/revitalize/macro_sensitivity.py`
- `tools/revitalize/paths_p13.py`
- `tools/revitalize/pilot_p12.py`
- `tools/revitalize/pilot_p13.py`
- `tools/revitalize/probe.sh`
- `tools/revitalize/test_diversity_p12.py`
- `tools/revitalize/test_evaluate_pass12.py`
- `tools/revitalize/test_evaluate_pass13.py`
- `tools/revitalize/test_macro_sensitivity.py`
- `tools/revitalize/test_paths_p13.py`
- `tools/revitalize/test_pilot_p13.py`
- `tools/worldmodel/test_export_determinism.py`
- `tools/worldmodel/test_hidden_columns.py`
- `tools/worldmodel/test_wm_data.py`
- `tools/worldmodel/wm_data.py`

### Part 0 — `2c52dd436842247bcbc3bf754f89b367b793c7fe`

art-p8(p0): keep art fixtures compatible with pressure-based fission

분류: `art_owned`

- `docs/art/production8/DECISIONS.md`
- `docs/art/production8/DESIGN.md`
- `docs/art/production8/TASKS.md`
- `rust/crates/sim-bridge/src/ffi/world_node.rs`

### Part 0 — `6a808fb9af5004521327f9e19070b78d9cb0014b`

art-p8(p0): verify the merged world before selecting capture dates

분류: `art_owned`

- `docs/art/production8/CROWD_DIAGNOSIS.json`
- `docs/art/production8/CROWD_DIAGNOSIS.md`
- `docs/art/production8/DECISIONS.md`
- `docs/art/production8/FEED_PLAN.md`
- `docs/art/production8/MERGE_CONTRACT.md`
- `docs/art/production8/PRESENTATION_PLAN.md`
- `docs/art/production8/gates/p0-attempt1/bridge.log`
- `docs/art/production8/gates/p0-attempt1/cargo_test.log`
- `docs/art/production8/gates/p0-attempt1/clippy.log`
- `docs/art/production8/gates/p0-attempt1/load.txt`
- `docs/art/production8/gates/p0-attempt1/python.log`
- `docs/art/production8/gates/p0-attempt1/results.tsv`
- `docs/art/production8/gates/p0-attempt1/smoke.log`
- `docs/art/production8/gates/p0-attempt1/smoke_errors.log`
- `docs/art/production8/gates/p0-attempt1/strict.log`
- `docs/art/production8/gates/p0-attempt1/test_abandoned_village.log`
- `docs/art/production8/gates/p0-attempt1/test_agent_activity.log`
- `docs/art/production8/gates/p0-attempt1/test_agent_click_drawn.log`
- `docs/art/production8/gates/p0-attempt1/test_agent_drawn_positions.log`
- `docs/art/production8/gates/p0-attempt1/test_agent_native_size.log`
- `docs/art/production8/gates/p0-attempt1/test_art_hash.log`
- `docs/art/production8/gates/p0-attempt1/test_camp_seats.log`
- `docs/art/production8/gates/p0-attempt1/test_canopy_fade.log`
- `docs/art/production8/gates/p0-attempt1/test_capture_cold_trace_p6.log`
- `docs/art/production8/gates/p0-attempt1/test_capture_metadata.log`
- `docs/art/production8/gates/p0-attempt1/test_capture_plans_p6.log`
- `docs/art/production8/gates/p0-attempt1/test_capture_society_trace.log`
- `docs/art/production8/gates/p0-attempt1/test_carry_load_layer.log`
- `docs/art/production8/gates/p0-attempt1/test_cold_hearth.log`
- `docs/art/production8/gates/p0-attempt1/test_cold_presentation.log`
- `docs/art/production8/gates/p0-attempt1/test_death_exposure.log`
- `docs/art/production8/gates/p0-attempt1/test_declump.log`
- `docs/art/production8/gates/p0-attempt1/test_effect_lifecycle.log`
- `docs/art/production8/gates/p0-attempt1/test_elder_variant.log`
- `docs/art/production8/gates/p0-attempt1/test_every_hearth.log`
- `docs/art/production8/gates/p0-attempt1/test_founding_group.log`
- `docs/art/production8/gates/p0-attempt1/test_god_hand.log`
- `docs/art/production8/gates/p0-attempt1/test_hud_old_age.log`
- `docs/art/production8/gates/p0-attempt1/test_hud_season_label.log`
- `docs/art/production8/gates/p0-attempt1/test_hud_wood.log`
- `docs/art/production8/gates/p0-attempt1/test_idle_camp_pose.log`
- `docs/art/production8/gates/p0-attempt1/test_idle_facing.log`
- `docs/art/production8/gates/p0-attempt1/test_inspector_age.log`
- `docs/art/production8/gates/p0-attempt1/test_inspector_cold.log`
- `docs/art/production8/gates/p0-attempt1/test_inspector_wood.log`
- `docs/art/production8/gates/p0-attempt1/test_interventions_real_p6.log`
- `docs/art/production8/gates/p0-attempt1/test_meal_pose.log`
- `docs/art/production8/gates/p0-attempt1/test_morning_chill.log`
- `docs/art/production8/gates/p0-attempt1/test_nature_clearance.log`
- `docs/art/production8/gates/p0-attempt1/test_nature_density.log`
- `docs/art/production8/gates/p0-attempt1/test_need_warning.log`
- `docs/art/production8/gates/p0-attempt1/test_season_tint.log`
- `docs/art/production8/gates/p0-attempt1/test_settlement_labels.log`
- `docs/art/production8/gates/p0-attempt1/test_settlement_select.log`
- `docs/art/production8/gates/p0-attempt1/test_shelter_cold.log`
- `docs/art/production8/gates/p0-attempt1/test_society_snapshot_p7.log`
- `docs/art/production8/gates/p0-attempt1/test_structure_states.log`
- `docs/art/production8/gates/p0-attempt1/test_terrain_layers.log`
- `docs/art/production8/gates/p0-attempt1/test_variant_codes.log`
- `docs/art/production8/gates/p0-attempt1/test_ysort_props.log`
- `docs/art/production8/gates/p0-attempt1/test_zoom_ladder.log`
- `docs/art/production8/gates/p0/bridge.log`
- `docs/art/production8/gates/p0/cargo_test.log`
- `docs/art/production8/gates/p0/clippy.log`
- `docs/art/production8/gates/p0/load.txt`
- `docs/art/production8/gates/p0/python.log`
- `docs/art/production8/gates/p0/results.tsv`
- `docs/art/production8/gates/p0/smoke.log`
- `docs/art/production8/gates/p0/smoke_errors.log`
- `docs/art/production8/gates/p0/strict.log`
- `docs/art/production8/gates/p0/test_abandoned_village.log`
- `docs/art/production8/gates/p0/test_agent_activity.log`
- `docs/art/production8/gates/p0/test_agent_click_drawn.log`
- `docs/art/production8/gates/p0/test_agent_drawn_positions.log`
- `docs/art/production8/gates/p0/test_agent_native_size.log`
- `docs/art/production8/gates/p0/test_art_hash.log`
- `docs/art/production8/gates/p0/test_camp_seats.log`
- `docs/art/production8/gates/p0/test_canopy_fade.log`
- `docs/art/production8/gates/p0/test_capture_cold_trace_p6.log`
- `docs/art/production8/gates/p0/test_capture_metadata.log`
- `docs/art/production8/gates/p0/test_capture_plans_p6.log`
- `docs/art/production8/gates/p0/test_capture_society_trace.log`
- `docs/art/production8/gates/p0/test_carry_load_layer.log`
- `docs/art/production8/gates/p0/test_cold_hearth.log`
- `docs/art/production8/gates/p0/test_cold_presentation.log`
- `docs/art/production8/gates/p0/test_death_exposure.log`
- `docs/art/production8/gates/p0/test_declump.log`
- `docs/art/production8/gates/p0/test_effect_lifecycle.log`
- `docs/art/production8/gates/p0/test_elder_variant.log`
- `docs/art/production8/gates/p0/test_every_hearth.log`
- `docs/art/production8/gates/p0/test_founding_group.log`
- `docs/art/production8/gates/p0/test_god_hand.log`
- `docs/art/production8/gates/p0/test_hud_old_age.log`
- `docs/art/production8/gates/p0/test_hud_season_label.log`
- `docs/art/production8/gates/p0/test_hud_wood.log`
- `docs/art/production8/gates/p0/test_idle_camp_pose.log`
- `docs/art/production8/gates/p0/test_idle_facing.log`
- `docs/art/production8/gates/p0/test_inspector_age.log`
- `docs/art/production8/gates/p0/test_inspector_cold.log`
- `docs/art/production8/gates/p0/test_inspector_wood.log`
- `docs/art/production8/gates/p0/test_interventions_real_p6.log`
- `docs/art/production8/gates/p0/test_meal_pose.log`
- `docs/art/production8/gates/p0/test_morning_chill.log`
- `docs/art/production8/gates/p0/test_nature_clearance.log`
- `docs/art/production8/gates/p0/test_nature_density.log`
- `docs/art/production8/gates/p0/test_need_warning.log`
- `docs/art/production8/gates/p0/test_season_tint.log`
- `docs/art/production8/gates/p0/test_settlement_labels.log`
- `docs/art/production8/gates/p0/test_settlement_select.log`
- `docs/art/production8/gates/p0/test_shelter_cold.log`
- `docs/art/production8/gates/p0/test_society_snapshot_p7.log`
- `docs/art/production8/gates/p0/test_structure_states.log`
- `docs/art/production8/gates/p0/test_terrain_layers.log`
- `docs/art/production8/gates/p0/test_variant_codes.log`
- `docs/art/production8/gates/p0/test_ysort_props.log`
- `docs/art/production8/gates/p0/test_zoom_ladder.log`
- `docs/art/production8/plans/baseline-live.json`
- `scripts/test/art/measure_days_p8.gd`
- `tools/art_pipeline/measure_days_p8.py`
- `tools/art_pipeline/probe_models_p8.py`

### Part 0 — `792696a00100c29de9951f1de2453a191b4a0080`

art-p8(p0): anchor visual comparisons to measured v19 worlds

분류: `art_owned`

- `docs/art/production8/BASELINE.md`
- `docs/art/production8/MEASURED_DAYS.md`
- `docs/art/production8/TASKS.md`
- `docs/art/production8/baseline-live/d2_1200_F2.entities.json.gz`
- `docs/art/production8/baseline-live/d2_1200_F2.jpg`
- `docs/art/production8/baseline-live/manifest.json`
- `docs/art/production8/baseline-live/manifest_final.json`
- `docs/art/production8/baseline-society/manifest_art_society.json`
- `docs/art/production8/baseline-society/soc_d33_0949_FV_all.entities.json.gz`
- `docs/art/production8/baseline-society/soc_d33_0949_FV_all.jpg`
- `docs/art/production8/measurements/probe_calendar_p8.json`
- `docs/art/production8/plans/baseline-society.json`
- `tools/art_pipeline/crosscheck_days_p8.py`

### Part 0 — `b526860e351548008849842b433c30a1fedef635`

art-p8(p0): confirm capture dates through the live bridge

분류: `art_owned`

- `docs/art/production8/MEASURED_DAYS.md`
- `docs/art/production8/TASKS.md`
- `docs/art/production8/measurements/headless_20261010.json.gz`
- `docs/art/production8/measurements/headless_20261013.json.gz`
- `docs/art/production8/measurements/headless_crosscheck_p8.json`
- `tools/art_pipeline/crosscheck_days_p8.py`

### Part 1 — `5e9ac2955281f625bbd872d33d1edf682d420a30`

art-p8(p1): keep village events readable above stock movement noise

분류: `art_owned`

- `docs/art/TEST_LEDGER.md`
- `docs/art/production8/DECISIONS.md`
- `docs/art/production8/DESIGN.md`
- `docs/art/production8/MEASURED_DAYS.md`
- `docs/art/production8/TASKS.md`
- `docs/art/production8/gates/p1/bridge_tests.log`
- `docs/art/production8/gates/p1/clippy.log`
- `docs/art/production8/gates/p1/feed_casts.log`
- `docs/art/production8/gates/p1/feed_casts_attempt1.log`
- `docs/art/production8/gates/p1/locale_final.log`
- `docs/art/production8/gates/p1/static.log`
- `docs/art/production8/gates/p1/strict.log`
- `docs/art/production8/gates/p1/test_death_exposure.log`
- `docs/art/production8/gates/p1/test_feed_casts.log`
- `docs/art/production8/gates/p1/test_feed_priority.log`
- `docs/art/production8/gates/p1/test_feed_wording.log`
- `docs/art/production8/gates/p1/test_founding_group.log`
- `docs/art/production8/gates/p1/test_god_hand.log`
- `docs/art/production8/gates/p1/test_hud_old_age.log`
- `docs/art/production8/gates/p1/test_settlement_select.log`
- `docs/art/production8/measurements/p1-news-audit.json`
- `docs/art/production8/p1-live/d2_1200_F2.entities.json.gz`
- `docs/art/production8/p1-live/d2_1200_F2.jpg`
- `docs/art/production8/p1-live/d2_2000_F2.entities.json.gz`
- `docs/art/production8/p1-live/d2_2000_F2.jpg`
- `docs/art/production8/p1-live/manifest.json`
- `docs/art/production8/p1-live/manifest_final.json`
- `docs/art/production8/p1-society-final/manifest_art_society.json`
- `docs/art/production8/p1-society-final/soc_d33_0900_F2_orig.entities.json.gz`
- `docs/art/production8/p1-society-final/soc_d33_0900_F2_orig.jpg`
- `docs/art/production8/p1-society/manifest_art_society.json`
- `docs/art/production8/p1-society/soc_d33_0900_F2_orig.entities.json.gz`
- `docs/art/production8/p1-society/soc_d33_0900_F2_orig.jpg`
- `docs/art/production8/plans/p1-live.json`
- `docs/art/production8/plans/p1-society.json`
- `docs/art/production8/verification/p1-live.json`
- `docs/art/production8/verification/p1-society.json`
- `localization/compiled/en.json`
- `localization/compiled/ko.json`
- `localization/fluent/en/messages.ftl`
- `localization/fluent/ko/messages.ftl`
- `localization/key_registry.json`
- `rust/crates/sim-bridge/src/ffi/world_node.rs`
- `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs`
- `scripts/test/art/capture_event_trace_p8.gd`
- `scripts/test/art/test_capture_event_trace_p8.gd`
- `scripts/test/art/test_death_exposure.gd`
- `scripts/test/art/test_feed_casts.gd`
- `scripts/test/art/test_feed_priority.gd`
- `scripts/test/art/test_feed_wording.gd`
- `scripts/test/art/test_hud_old_age.gd`
- `scripts/test/capture_framings.gd`
- `scripts/ui/panels/feed_food_ledger.gd`
- `scripts/ui/panels/god_hand_panel.gd`
- `scripts/ui/panels/hud_status_panel.gd`

### Part 2 — `cad635946407a37df7d2679a9419f173195e90e5`

art-p8(p2): keep distant founding sites reachable from the viewport

분류: `art_owned`

- `docs/art/production8/DECISIONS.md`
- `docs/art/production8/DESIGN.md`
- `docs/art/production8/PART2_VERIFICATION.md`
- `docs/art/production8/TASKS.md`
- `docs/art/production8/verification/p2-marker/after.png`
- `docs/art/production8/verification/p2-marker/before.png`
- `docs/art/production8/verification/p2-marker/provenance.json`
- `docs/art/production8/verification/p2-marker/result.json`
- `docs/art/production8/verification/p2-marker/selected-after.png`
- `docs/art/production8/verification/p2-marker/selected-before.png`
- `localization/compiled/en.json`
- `localization/compiled/ko.json`
- `localization/fluent/en/messages.ftl`
- `localization/fluent/ko/messages.ftl`
- `localization/key_registry.json`
- `scripts/test/art/qa_offscreen_marker_windowed.gd`
- `scripts/test/art/test_offscreen_marker.gd`
- `scripts/test/capture_framings.gd`
- `scripts/ui/founding_group_renderer.gd`
- `scripts/ui/offscreen_marker.gd`
- `scripts/ui/settlement_labels.gd`

### Part 3 — `2ee8f1fbc360c9db2279f6986e9181b0fdfc23c3`

art-p8(p3): expose village pressure before a split becomes visible

분류: `art_owned`

- `docs/art/production8/DECISIONS.md`
- `docs/art/production8/DESIGN.md`
- `docs/art/production8/PART3_PRESENTATION_VERIFICATION.md`
- `docs/art/production8/TASKS.md`
- `docs/art/production8/gates/p3/bridge_build.log`
- `docs/art/production8/gates/p3/bridge_clippy.log`
- `docs/art/production8/gates/p3/bridge_tests.log`
- `docs/art/production8/plans/p3-meter.json`
- `docs/art/production8/verification/p3-meter/manifest_art_society.json`
- `docs/art/production8/verification/p3-meter/soc_d32_2000_F2_orig_selected_orig.entities.json.gz`
- `docs/art/production8/verification/p3-meter/soc_d32_2000_F2_orig_selected_orig.jpg`
- `docs/art/production8/verification/p3-meter/soc_d33_0630_FD_leader.entities.json.gz`
- `docs/art/production8/verification/p3-meter/soc_d33_0630_FD_leader.jpg`
- `docs/art/production8/verification/p3-meter/soc_d33_0859_FS_daughter_selected_daughter.entities.json.gz`
- `docs/art/production8/verification/p3-meter/soc_d33_0859_FS_daughter_selected_daughter.jpg`
- `docs/art/production8/verification/p3-values.json`
- `localization/compiled/en.json`
- `localization/compiled/ko.json`
- `localization/fluent/en/messages.ftl`
- `localization/fluent/ko/messages.ftl`
- `localization/key_registry.json`
- `rust/crates/sim-bridge/src/ffi/world_node.rs`
- `scripts/core/settlement_selection.gd`
- `scripts/test/art/test_restless_meter.gd`
- `scripts/ui/panels/god_hand_panel.gd`
- `scripts/ui/restless_meter.gd`
- `scripts/ui/settlement_labels.gd`

### Part 4 — `95d257a75e952145797d8642634dae3641c4cffe`

art-p8(p4): make persistent visual failures measurable and visible

분류: `art_owned`

- `docs/art/production8/DECISIONS.md`
- `docs/art/production8/FRAME_MEASURE_README.md`
- `docs/art/production8/PART4_CROWD_VERIFICATION.md`
- `docs/art/production8/PART4_VERIFICATION.md`
- `docs/art/production8/TASKS.md`
- `docs/art/production8/gates/p4-crowd/strict-final.log`
- `docs/art/production8/gates/p4-crowd/strict.log`
- `docs/art/production8/gates/p4-crowd/test_agent_drawn_positions-final.log`
- `docs/art/production8/gates/p4-crowd/test_camp_seats-final.log`
- `docs/art/production8/gates/p4-crowd/test_declump-final.log`
- `docs/art/production8/gates/p4-crowd/test_meal_ring-attempt2.log`
- `docs/art/production8/gates/p4-crowd/test_meal_ring-final.log`
- `docs/art/production8/gates/p4-crowd/test_meal_ring.log`
- `docs/art/production8/gates/p4-crowd/test_morning_chill-final.log`
- `docs/art/production8/gates/p4-crowd/test_morning_chill_v2-final.log`
- `docs/art/production8/gates/p4-crowd/test_morning_chill_v2.log`
- `docs/art/production8/gates/p4/frame_measure.log`
- `docs/art/production8/gates/p4/strict.log`
- `docs/art/production8/gates/p4/test_intervention_world_effects.log`
- `docs/art/production8/gates/p4/test_meal_ring.log`
- `docs/art/production8/gates/p4/test_morning_chill_v2.log`
- `docs/art/production8/gates/p4/test_village_layers.log`
- `docs/art/production8/gates/p4effects/NOTES.md`
- `docs/art/production8/gates/p4effects/god_hand.log`
- `docs/art/production8/gates/p4effects/settlement_select.log`
- `docs/art/production8/gates/p4effects/strict.log`
- `docs/art/production8/gates/p4effects/world_effects.log`
- `docs/art/production8/plans/p4-crowd-chill.json`
- `docs/art/production8/plans/p4-effects.json`
- `docs/art/production8/verification/p4-crowd/l_d26_0715_F2.entities.json.gz`
- `docs/art/production8/verification/p4-crowd/l_d26_0715_F2.jpg`
- `docs/art/production8/verification/p4-crowd/l_d28_0200_F2N.cold_cycle.gif`
- `docs/art/production8/verification/p4-crowd/l_d28_0200_F2N.cold_cycle.json.gz`
- `docs/art/production8/verification/p4-crowd/l_d28_0200_F2N.entities.json.gz`
- `docs/art/production8/verification/p4-crowd/l_d28_0200_F2N.jpg`
- `docs/art/production8/verification/p4-crowd/l_d28_0715_F2.entities.json.gz`
- `docs/art/production8/verification/p4-crowd/l_d28_0715_F2.jpg`
- `docs/art/production8/verification/p4-crowd/l_d29_0200_F2N.cold_cycle.gif`
- `docs/art/production8/verification/p4-crowd/l_d29_0200_F2N.cold_cycle.json.gz`
- `docs/art/production8/verification/p4-crowd/l_d29_0200_F2N.entities.json.gz`
- `docs/art/production8/verification/p4-crowd/l_d29_0200_F2N.jpg`
- `docs/art/production8/verification/p4-crowd/l_d29_0630_F2.entities.json.gz`
- `docs/art/production8/verification/p4-crowd/l_d29_0630_F2.jpg`
- `docs/art/production8/verification/p4-crowd/l_d29_0715_F2.entities.json.gz`
- `docs/art/production8/verification/p4-crowd/l_d29_0715_F2.jpg`
- `docs/art/production8/verification/p4-crowd/manifest.json`
- `docs/art/production8/verification/p4-crowd/manifest_art_long.json`
- `docs/art/production8/verification/p4-crowd/numeric.json`
- `docs/art/production8/verification/p4-crowd/preliminary-all-sprites.json`
- `docs/art/production8/verification/p4-effects/god_d23_1200_F2_spoil.entities.json.gz`
- `docs/art/production8/verification/p4-effects/god_d23_1200_F2_spoil.jpg`
- `docs/art/production8/verification/p4-effects/god_d23_1201_F2_gift_food.entities.json.gz`
- `docs/art/production8/verification/p4-effects/god_d23_1201_F2_gift_food.jpg`
- `docs/art/production8/verification/p4-effects/god_d23_1202_F2_rot_wood.entities.json.gz`
- `docs/art/production8/verification/p4-effects/god_d23_1202_F2_rot_wood.jpg`
- `docs/art/production8/verification/p4-effects/god_d23_1203_F2_gift.entities.json.gz`
- `docs/art/production8/verification/p4-effects/god_d23_1203_F2_gift.jpg`
- `docs/art/production8/verification/p4-effects/god_d23_1205_F2_windfall_0200ms.entities.json.gz`
- `docs/art/production8/verification/p4-effects/god_d23_1205_F2_windfall_0200ms.jpg`
- `docs/art/production8/verification/p4-effects/god_d23_1205_F2_windfall_1000ms.entities.json.gz`
- `docs/art/production8/verification/p4-effects/god_d23_1205_F2_windfall_1000ms.jpg`
- `docs/art/production8/verification/p4-effects/manifest_art_god.json`
- `docs/art/production8/verification/p4-effects/runtime-summary.json`
- `docs/art/production8/verification/p4-perf/frame_times_soc_layers.json.gz`
- `docs/art/production8/verification/p4-perf/load-before.log`
- `docs/art/production8/verification/p4-perf/manifest_art_society.json`
- `scripts/core/settlement_selection.gd`
- `scripts/rendering/art/god_hand_effects.gd`
- `scripts/test/art/capture_metadata.gd`
- `scripts/test/art/frame_cpu_probe.gd`
- `scripts/test/art/frame_measure_session.gd`
- `scripts/test/art/test_frame_measure.gd`
- `scripts/test/art/test_god_hand.gd`
- `scripts/test/art/test_intervention_world_effects.gd`
- `scripts/test/art/test_meal_ring.gd`
- `scripts/test/art/test_morning_chill_v2.gd`
- `scripts/test/art/test_village_layers.gd`
- `scripts/test/capture_framings.gd`
- `scripts/ui/agent_renderer.gd`
- `scripts/ui/camp_seats.gd`
- `scripts/ui/morning_chill.gd`
- `scripts/ui/panels/god_hand_panel.gd`
- `scripts/ui/settlement_overview_renderer.gd`
- `tools/art_pipeline/capture_art.sh`

### Part 5 — `cefc77b088d6fa3baf21b766cdc40e3fdf045d36`

art-p8(p5): bind captures to measured v19 worlds and paired interventions

분류: `art_owned`

- `docs/art/TEST_LEDGER.md`
- `docs/art/production8/CAPTURE_PLAN.md`
- `docs/art/production8/DECISIONS.md`
- `docs/art/production8/FRAME_MEASURE_README.md`
- `docs/art/production8/TASKS.md`
- `docs/art/production8/gates/p5capture/NOTES.md`
- `docs/art/production8/gates/p5capture/metadata-v19.log`
- `docs/art/production8/gates/p5capture/metadata.log`
- `docs/art/production8/gates/p5capture/pair-parent.log`
- `docs/art/production8/gates/p5capture/paired_fixture.log`
- `docs/art/production8/gates/p5capture/parse.log`
- `docs/art/production8/gates/p5capture/plans-v19.log`
- `docs/art/production8/gates/p5capture/plans.log`
- `docs/art/production8/gates/p5capture/strict.log`
- `docs/art/production8/plans/p5-god-g.json`
- `docs/art/production8/plans/p5-god-live.json`
- `docs/art/production8/plans/p5-god-p.json`
- `docs/art/production8/plans/p5-god-society.json`
- `scripts/test/art/capture_plans.gd`
- `scripts/test/art/test_capture_metadata.gd`
- `scripts/test/art/test_capture_pair_p8.gd`
- `scripts/test/art/test_capture_plans_p6.gd`
- `scripts/test/capture_framings.gd`

### Part 5 — `72c8cee35b0205824794db33d166bb3fc82f8e34`

art-p8(p5): preserve complete measured worlds in reversible evidence archives

분류: `art_owned`

- `docs/art/production8/CAPTURE_RESULTS.json`
- `docs/art/production8/CAPTURE_VERIFICATION.md`
- `docs/art/production8/DECISIONS.md`
- `docs/art/production8/EVIDENCE_ARCHIVE.md`
- `docs/art/production8/TASKS.md`
- `docs/art/production8/baseline-live/d2_1200_F2.entities.json.gz`
- `docs/art/production8/baseline-live/d2_1200_F2.jpg`
- `docs/art/production8/baseline-live/d2_1200_F2.png`
- `docs/art/production8/baseline-live/manifest.json`
- `docs/art/production8/baseline-live/manifest_final.json`
- `docs/art/production8/baseline-society/manifest_art_society.json`
- `docs/art/production8/baseline-society/soc_d33_0949_FV_all.entities.json.gz`
- `docs/art/production8/baseline-society/soc_d33_0949_FV_all.jpg`
- `docs/art/production8/baseline-society/soc_d33_0949_FV_all.png`
- `docs/art/production8/diagnostic-gifs-and-logs.archive.json`
- `docs/art/production8/diagnostic-gifs-and-logs.tar.xz`
- `docs/art/production8/entities.archive.json`
- `docs/art/production8/entities.tar.xz`
- `docs/art/production8/gates/p0-attempt1/bridge.log`
- `docs/art/production8/gates/p0-attempt1/cargo_test.log`
- `docs/art/production8/gates/p0-attempt1/clippy.log`
- `docs/art/production8/gates/p0-attempt1/python.log`
- `docs/art/production8/gates/p0-attempt1/smoke.log`
- `docs/art/production8/gates/p0-attempt1/smoke_errors.log`
- `docs/art/production8/gates/p0-attempt1/strict.log`
- `docs/art/production8/gates/p0-attempt1/test_abandoned_village.log`
- `docs/art/production8/gates/p0-attempt1/test_agent_activity.log`
- `docs/art/production8/gates/p0-attempt1/test_agent_click_drawn.log`
- `docs/art/production8/gates/p0-attempt1/test_agent_drawn_positions.log`
- `docs/art/production8/gates/p0-attempt1/test_agent_native_size.log`
- `docs/art/production8/gates/p0-attempt1/test_art_hash.log`
- `docs/art/production8/gates/p0-attempt1/test_camp_seats.log`
- `docs/art/production8/gates/p0-attempt1/test_canopy_fade.log`
- `docs/art/production8/gates/p0-attempt1/test_capture_cold_trace_p6.log`
- `docs/art/production8/gates/p0-attempt1/test_capture_metadata.log`
- `docs/art/production8/gates/p0-attempt1/test_capture_plans_p6.log`
- `docs/art/production8/gates/p0-attempt1/test_capture_society_trace.log`
- `docs/art/production8/gates/p0-attempt1/test_carry_load_layer.log`
- `docs/art/production8/gates/p0-attempt1/test_cold_hearth.log`
- `docs/art/production8/gates/p0-attempt1/test_cold_presentation.log`
- `docs/art/production8/gates/p0-attempt1/test_death_exposure.log`
- `docs/art/production8/gates/p0-attempt1/test_declump.log`
- `docs/art/production8/gates/p0-attempt1/test_effect_lifecycle.log`
- `docs/art/production8/gates/p0-attempt1/test_elder_variant.log`
- `docs/art/production8/gates/p0-attempt1/test_every_hearth.log`
- `docs/art/production8/gates/p0-attempt1/test_founding_group.log`
- `docs/art/production8/gates/p0-attempt1/test_god_hand.log`
- `docs/art/production8/gates/p0-attempt1/test_hud_old_age.log`
- `docs/art/production8/gates/p0-attempt1/test_hud_season_label.log`
- `docs/art/production8/gates/p0-attempt1/test_hud_wood.log`
- `docs/art/production8/gates/p0-attempt1/test_idle_camp_pose.log`
- `docs/art/production8/gates/p0-attempt1/test_idle_facing.log`
- `docs/art/production8/gates/p0-attempt1/test_inspector_age.log`
- `docs/art/production8/gates/p0-attempt1/test_inspector_cold.log`
- `docs/art/production8/gates/p0-attempt1/test_inspector_wood.log`
- `docs/art/production8/gates/p0-attempt1/test_interventions_real_p6.log`
- `docs/art/production8/gates/p0-attempt1/test_meal_pose.log`
- `docs/art/production8/gates/p0-attempt1/test_morning_chill.log`
- `docs/art/production8/gates/p0-attempt1/test_nature_clearance.log`
- `docs/art/production8/gates/p0-attempt1/test_nature_density.log`
- `docs/art/production8/gates/p0-attempt1/test_need_warning.log`
- `docs/art/production8/gates/p0-attempt1/test_season_tint.log`
- `docs/art/production8/gates/p0-attempt1/test_settlement_labels.log`
- `docs/art/production8/gates/p0-attempt1/test_settlement_select.log`
- `docs/art/production8/gates/p0-attempt1/test_shelter_cold.log`
- `docs/art/production8/gates/p0-attempt1/test_society_snapshot_p7.log`
- `docs/art/production8/gates/p0-attempt1/test_structure_states.log`
- `docs/art/production8/gates/p0-attempt1/test_terrain_layers.log`
- `docs/art/production8/gates/p0-attempt1/test_variant_codes.log`
- `docs/art/production8/gates/p0-attempt1/test_ysort_props.log`
- `docs/art/production8/gates/p0-attempt1/test_zoom_ladder.log`
- `docs/art/production8/gates/p0/bridge.log`
- `docs/art/production8/gates/p0/cargo_test.log`
- `docs/art/production8/gates/p0/clippy.log`
- `docs/art/production8/gates/p0/python.log`
- `docs/art/production8/gates/p0/smoke.log`
- `docs/art/production8/gates/p0/smoke_errors.log`
- `docs/art/production8/gates/p0/strict.log`
- `docs/art/production8/gates/p0/test_abandoned_village.log`
- `docs/art/production8/gates/p0/test_agent_activity.log`
- `docs/art/production8/gates/p0/test_agent_click_drawn.log`
- `docs/art/production8/gates/p0/test_agent_drawn_positions.log`
- `docs/art/production8/gates/p0/test_agent_native_size.log`
- `docs/art/production8/gates/p0/test_art_hash.log`
- `docs/art/production8/gates/p0/test_camp_seats.log`
- `docs/art/production8/gates/p0/test_canopy_fade.log`
- `docs/art/production8/gates/p0/test_capture_cold_trace_p6.log`
- `docs/art/production8/gates/p0/test_capture_metadata.log`
- `docs/art/production8/gates/p0/test_capture_plans_p6.log`
- `docs/art/production8/gates/p0/test_capture_society_trace.log`
- `docs/art/production8/gates/p0/test_carry_load_layer.log`
- `docs/art/production8/gates/p0/test_cold_hearth.log`
- `docs/art/production8/gates/p0/test_cold_presentation.log`
- `docs/art/production8/gates/p0/test_death_exposure.log`
- `docs/art/production8/gates/p0/test_declump.log`
- `docs/art/production8/gates/p0/test_effect_lifecycle.log`
- `docs/art/production8/gates/p0/test_elder_variant.log`
- `docs/art/production8/gates/p0/test_every_hearth.log`
- `docs/art/production8/gates/p0/test_founding_group.log`
- `docs/art/production8/gates/p0/test_god_hand.log`
- `docs/art/production8/gates/p0/test_hud_old_age.log`
- `docs/art/production8/gates/p0/test_hud_season_label.log`
- `docs/art/production8/gates/p0/test_hud_wood.log`
- `docs/art/production8/gates/p0/test_idle_camp_pose.log`
- `docs/art/production8/gates/p0/test_idle_facing.log`
- `docs/art/production8/gates/p0/test_inspector_age.log`
- `docs/art/production8/gates/p0/test_inspector_cold.log`
- `docs/art/production8/gates/p0/test_inspector_wood.log`
- `docs/art/production8/gates/p0/test_interventions_real_p6.log`
- `docs/art/production8/gates/p0/test_meal_pose.log`
- `docs/art/production8/gates/p0/test_morning_chill.log`
- `docs/art/production8/gates/p0/test_nature_clearance.log`
- `docs/art/production8/gates/p0/test_nature_density.log`
- `docs/art/production8/gates/p0/test_need_warning.log`
- `docs/art/production8/gates/p0/test_season_tint.log`
- `docs/art/production8/gates/p0/test_settlement_labels.log`
- `docs/art/production8/gates/p0/test_settlement_select.log`
- `docs/art/production8/gates/p0/test_shelter_cold.log`
- `docs/art/production8/gates/p0/test_society_snapshot_p7.log`
- `docs/art/production8/gates/p0/test_structure_states.log`
- `docs/art/production8/gates/p0/test_terrain_layers.log`
- `docs/art/production8/gates/p0/test_variant_codes.log`
- `docs/art/production8/gates/p0/test_ysort_props.log`
- `docs/art/production8/gates/p0/test_zoom_ladder.log`
- `docs/art/production8/gates/p1/bridge_tests.log`
- `docs/art/production8/gates/p1/clippy.log`
- `docs/art/production8/gates/p1/feed_casts.log`
- `docs/art/production8/gates/p1/feed_casts_attempt1.log`
- `docs/art/production8/gates/p1/locale_final.log`
- `docs/art/production8/gates/p1/static.log`
- `docs/art/production8/gates/p1/strict.log`
- `docs/art/production8/gates/p1/test_death_exposure.log`
- `docs/art/production8/gates/p1/test_feed_casts.log`
- `docs/art/production8/gates/p1/test_feed_priority.log`
- `docs/art/production8/gates/p1/test_feed_wording.log`
- `docs/art/production8/gates/p1/test_founding_group.log`
- `docs/art/production8/gates/p1/test_god_hand.log`
- `docs/art/production8/gates/p1/test_hud_old_age.log`
- `docs/art/production8/gates/p1/test_settlement_select.log`
- `docs/art/production8/gates/p3/bridge_build.log`
- `docs/art/production8/gates/p3/bridge_clippy.log`
- `docs/art/production8/gates/p3/bridge_tests.log`
- `docs/art/production8/gates/p4-crowd/strict-final.log`
- `docs/art/production8/gates/p4-crowd/strict.log`
- `docs/art/production8/gates/p4-crowd/test_agent_drawn_positions-final.log`
- `docs/art/production8/gates/p4-crowd/test_camp_seats-final.log`
- `docs/art/production8/gates/p4-crowd/test_declump-final.log`
- `docs/art/production8/gates/p4-crowd/test_meal_ring-attempt2.log`
- `docs/art/production8/gates/p4-crowd/test_meal_ring-final.log`
- `docs/art/production8/gates/p4-crowd/test_meal_ring.log`
- `docs/art/production8/gates/p4-crowd/test_morning_chill-final.log`
- `docs/art/production8/gates/p4-crowd/test_morning_chill_v2-final.log`
- `docs/art/production8/gates/p4-crowd/test_morning_chill_v2.log`
- `docs/art/production8/gates/p4/frame_measure.log`
- `docs/art/production8/gates/p4/strict.log`
- `docs/art/production8/gates/p4/test_intervention_world_effects.log`
- `docs/art/production8/gates/p4/test_meal_ring.log`
- `docs/art/production8/gates/p4/test_morning_chill_v2.log`
- `docs/art/production8/gates/p4/test_village_layers.log`
- `docs/art/production8/gates/p4effects/god_hand.log`
- `docs/art/production8/gates/p4effects/settlement_select.log`
- `docs/art/production8/gates/p4effects/strict.log`
- `docs/art/production8/gates/p4effects/world_effects.log`
- `docs/art/production8/gates/p5capture/metadata-v19.log`
- `docs/art/production8/gates/p5capture/metadata.log`
- `docs/art/production8/gates/p5capture/pair-parent.log`
- `docs/art/production8/gates/p5capture/paired_fixture.log`
- `docs/art/production8/gates/p5capture/parse.log`
- `docs/art/production8/gates/p5capture/plans-v19.log`
- `docs/art/production8/gates/p5capture/plans.log`
- `docs/art/production8/gates/p5capture/strict.log`
- `docs/art/production8/god-segments-relocation.json`
- `docs/art/production8/god/god_d10_1200_F2_spoil.png`
- `docs/art/production8/god/god_d10_1205_F2_rot_wood.png`
- `docs/art/production8/god/god_d10_1210_F2_gift_food.png`
- `docs/art/production8/god/god_d17_0200_F2N_cold_snap.jpg`
- `docs/art/production8/god/god_d23_1200_F2_gift.png`
- `docs/art/production8/god/god_d23_1205_F2_windfall_0200ms.png`
- `docs/art/production8/god/god_d23_1205_F2_windfall_1000ms.png`
- `docs/art/production8/god/god_d24_1830_F2_rain.png`
- `docs/art/production8/god/god_d25_0200_F2N_rain_night.jpg`
- `docs/art/production8/god/god_d25_0200_F2_panel_log.jpg`
- `docs/art/production8/god/god_d50_1700_F2_soc_rain_daughter.png`
- `docs/art/production8/god/god_d51_0200_F2N_soc_original_night_orig.jpg`
- `docs/art/production8/god/god_d51_0200_F2N_soc_rain_night_daughter.png`
- `docs/art/production8/god/god_d51_0200_FV_soc_both_night_all.jpg`
- `docs/art/production8/god/god_p_gift_d120_1200_FV_final_all.jpg`
- `docs/art/production8/god/god_p_gift_d30_0600_F2_cast_orig.jpg`
- `docs/art/production8/god/god_p_gift_d31_0600_F2_cast_orig.jpg`
- `docs/art/production8/god/god_p_gift_d31_0600_F2_morning_orig.jpg`
- `docs/art/production8/god/god_p_gift_d32_0600_F2_cast_orig.jpg`
- `docs/art/production8/god/god_p_gift_d32_0600_F2_morning_orig.jpg`
- `docs/art/production8/god/god_p_gift_d33_0600_F2_morning_orig.jpg`
- `docs/art/production8/god/god_p_spoil_d120_1200_FV_final_all.jpg`
- `docs/art/production8/god/god_p_spoil_d30_0600_F2_cast_orig.jpg`
- `docs/art/production8/god/god_p_spoil_d31_0600_F2_cast_orig.jpg`
- `docs/art/production8/god/god_p_spoil_d31_0600_F2_morning_orig.jpg`
- `docs/art/production8/god/god_p_spoil_d32_0600_F2_cast_orig.jpg`
- `docs/art/production8/god/god_p_spoil_d32_0600_F2_morning_orig.jpg`
- `docs/art/production8/god/god_p_spoil_d33_0600_F2_morning_orig.jpg`
- `docs/art/production8/god/manifest_art_god.json`
- `docs/art/production8/god/source_runs/g/manifest_art_god.json`
- `docs/art/production8/god/source_runs/live/manifest_art_god.json`
- `docs/art/production8/god/source_runs/p/manifest_art_god.json`
- `docs/art/production8/god/source_runs/society/manifest_art_god.json`
- `docs/art/production8/p1-live/d2_1200_F2.entities.json.gz`
- `docs/art/production8/p1-live/d2_1200_F2.jpg`
- `docs/art/production8/p1-live/d2_1200_F2.png`
- `docs/art/production8/p1-live/d2_2000_F2.entities.json.gz`
- `docs/art/production8/p1-live/manifest.json`
- `docs/art/production8/p1-live/manifest_final.json`
- `docs/art/production8/p1-society-final/manifest_art_society.json`
- `docs/art/production8/p1-society-final/soc_d33_0900_F2_orig.entities.json.gz`
- `docs/art/production8/p1-society-final/soc_d33_0900_F2_orig.jpg`
- `docs/art/production8/p1-society-final/soc_d33_0900_F2_orig.png`
- `docs/art/production8/p1-society/manifest_art_society.json`
- `docs/art/production8/p1-society/soc_d33_0900_F2_orig.entities.json.gz`
- `docs/art/production8/p1-society/soc_d33_0900_F2_orig.jpg`
- `docs/art/production8/p1-society/soc_d33_0900_F2_orig.png`
- `docs/art/production8/perf/frame_times_soc_layers.json.gz`
- `docs/art/production8/perf/manifest_art_society.json`
- `docs/art/production8/png-upgrades-diagnostics.json`
- `docs/art/production8/png-upgrades-god.json`
- `docs/art/production8/png-upgrades-shots-society.json`
- `docs/art/production8/report_artifacts.json`
- `docs/art/production8/shots/a_calib_z4.png`
- `docs/art/production8/shots/a_calib_z6.png`
- `docs/art/production8/shots/a_d1_0830_F2.png`
- `docs/art/production8/shots/a_d2_1200_F1.png`
- `docs/art/production8/shots/a_d2_1200_F2.png`
- `docs/art/production8/shots/a_d2_1200_F2A.png`
- `docs/art/production8/shots/a_d2_1200_F2Z.png`
- `docs/art/production8/shots/a_d2_1200_F3.png`
- `docs/art/production8/shots/a_d2_1200_F4.png`
- `docs/art/production8/shots/a_d2_1930_F2.jpg`
- `docs/art/production8/shots/a_d2_2300_F2N.png`
- `docs/art/production8/shots/a_d6_1200_F2.png`
- `docs/art/production8/shots/a_lineup_z2.jpg`
- `docs/art/production8/shots/a_lineup_z4.png`
- `docs/art/production8/shots/evening.json.gz`
- `docs/art/production8/shots/forest_evidence.json.gz`
- `docs/art/production8/shots/frame_times.json.gz`
- `docs/art/production8/shots/l_d10_1200_F2.png`
- `docs/art/production8/shots/l_d10_1500_F2.png`
- `docs/art/production8/shots/l_d10_1500_F5.png`
- `docs/art/production8/shots/l_d15_1500_F2.png`
- `docs/art/production8/shots/l_d15_1500_FC.png`
- `docs/art/production8/shots/l_d16_1500_FC.png`
- `docs/art/production8/shots/l_d18_1200_F2.png`
- `docs/art/production8/shots/l_d23_0200_F2N.cold_cycle.json.gz`
- `docs/art/production8/shots/l_d23_0200_F2N.jpg`
- `docs/art/production8/shots/l_d24_1200_F1.jpg`
- `docs/art/production8/shots/l_d24_1200_F2.png`
- `docs/art/production8/shots/l_d24_2300_F2N.cold_cycle.json.gz`
- `docs/art/production8/shots/l_d24_2300_F2N.jpg`
- `docs/art/production8/shots/l_d26_0715_F2.png`
- `docs/art/production8/shots/l_d28_0200_F2N.cold_cycle.json.gz`
- `docs/art/production8/shots/l_d28_0200_F2N.jpg`
- `docs/art/production8/shots/l_d28_0200_F2Z.cold_cycle.json.gz`
- `docs/art/production8/shots/l_d28_0200_F2Z.png`
- `docs/art/production8/shots/l_d28_0715_F2.png`
- `docs/art/production8/shots/l_d29_0200_F2N.cold_cycle.json.gz`
- `docs/art/production8/shots/l_d29_0200_F2N.jpg`
- `docs/art/production8/shots/l_d29_0630_F2.png`
- `docs/art/production8/shots/l_d29_0715_F2.png`
- `docs/art/production8/shots/l_d32_1200_F2.png`
- `docs/art/production8/shots/l_d40_1200_F2.png`
- `docs/art/production8/shots/l_d43_1500_FC.png`
- `docs/art/production8/shots/l_d55_0200_F2N.cold_cycle.json.gz`
- `docs/art/production8/shots/l_d55_0200_F2N.jpg`
- `docs/art/production8/shots/l_d56_0200_F2N.cold_cycle.json.gz`
- `docs/art/production8/shots/l_d56_0200_F2N.jpg`
- `docs/art/production8/shots/l_d57_0200_F2N.cold_cycle.json.gz`
- `docs/art/production8/shots/l_d57_0200_F2N.jpg`
- `docs/art/production8/shots/l_d57_0401_DE.cold_cycle.json.gz`
- `docs/art/production8/shots/l_d57_0401_DE.jpg`
- `docs/art/production8/shots/l_d57_0715_F2.png`
- `docs/art/production8/shots/l_d71_1500_FC.png`
- `docs/art/production8/shots/l_d84_0200_F2N.cold_cycle.json.gz`
- `docs/art/production8/shots/l_d84_0200_F2N.jpg`
- `docs/art/production8/shots/l_d85_0200_F2N.cold_cycle.json.gz`
- `docs/art/production8/shots/l_d85_0200_F2N.jpg`
- `docs/art/production8/shots/l_d85_0715_F2.png`
- `docs/art/production8/shots/manifest.json`
- `docs/art/production8/shots/manifest_art.json`
- `docs/art/production8/shots/manifest_art_long.json`
- `docs/art/production8/shots/strip.json.gz`
- `docs/art/production8/society/manifest_art_society.json`
- `docs/art/production8/society/soc_d120_1200_FV_all.jpg`
- `docs/art/production8/society/soc_d32_2200_F2W_orig.png`
- `docs/art/production8/society/soc_d33_0630_FD_leader.png`
- `docs/art/production8/society/soc_d33_0630_FT_site.png`
- `docs/art/production8/society/soc_d33_0800_FD_leader.png`
- `docs/art/production8/society/soc_d33_0859_FS_daughter.png`
- `docs/art/production8/society/soc_d33_0900_F2W_orig.png`
- `docs/art/production8/society/soc_d33_0949_FV_all.png`
- `docs/art/production8/society/soc_d33_1200_FV_all.jpg`
- `docs/art/production8/society/soc_d33_1200_FV_all_selected_daughter.jpg`
- `docs/art/production8/society/soc_d33_1200_FV_all_selected_orig.jpg`
- `docs/art/production8/society/soc_d51_0200_F2N_daughter.png`
- `docs/art/production8/society/soc_d51_1200_FV_all.jpg`
- `docs/art/production8/society/soc_d84_0200_F2N_daughter.png`
- `docs/art/production8/society/soc_d84_1200_FV_all.jpg`
- `docs/art/production8/society/soc_d89_0630_FD_leader.png`
- `docs/art/production8/society/soc_d89_1234_FV_all.jpg`
- `docs/art/production8/verification/p3-meter/manifest_art_society.json`
- `docs/art/production8/verification/p3-meter/soc_d32_2000_F2_orig_selected_orig.entities.json.gz`
- `docs/art/production8/verification/p3-meter/soc_d32_2000_F2_orig_selected_orig.jpg`
- `docs/art/production8/verification/p3-meter/soc_d32_2000_F2_orig_selected_orig.png`
- `docs/art/production8/verification/p3-meter/soc_d33_0630_FD_leader.entities.json.gz`
- `docs/art/production8/verification/p3-meter/soc_d33_0630_FD_leader.jpg`
- `docs/art/production8/verification/p3-meter/soc_d33_0630_FD_leader.png`
- `docs/art/production8/verification/p3-meter/soc_d33_0859_FS_daughter_selected_daughter.entities.json.gz`
- `docs/art/production8/verification/p3-meter/soc_d33_0859_FS_daughter_selected_daughter.jpg`
- `docs/art/production8/verification/p3-meter/soc_d33_0859_FS_daughter_selected_daughter.png`
- `docs/art/production8/verification/p4-crowd/l_d26_0715_F2.entities.json.gz`
- `docs/art/production8/verification/p4-crowd/l_d26_0715_F2.jpg`
- `docs/art/production8/verification/p4-crowd/l_d26_0715_F2.png`
- `docs/art/production8/verification/p4-crowd/l_d28_0200_F2N.cold_cycle.gif`
- `docs/art/production8/verification/p4-crowd/l_d28_0200_F2N.entities.json.gz`
- `docs/art/production8/verification/p4-crowd/l_d28_0715_F2.entities.json.gz`
- `docs/art/production8/verification/p4-crowd/l_d28_0715_F2.jpg`
- `docs/art/production8/verification/p4-crowd/l_d28_0715_F2.png`
- `docs/art/production8/verification/p4-crowd/l_d29_0200_F2N.cold_cycle.gif`
- `docs/art/production8/verification/p4-crowd/l_d29_0200_F2N.entities.json.gz`
- `docs/art/production8/verification/p4-crowd/l_d29_0630_F2.entities.json.gz`
- `docs/art/production8/verification/p4-crowd/l_d29_0630_F2.jpg`
- `docs/art/production8/verification/p4-crowd/l_d29_0630_F2.png`
- `docs/art/production8/verification/p4-crowd/l_d29_0715_F2.entities.json.gz`
- `docs/art/production8/verification/p4-crowd/l_d29_0715_F2.jpg`
- `docs/art/production8/verification/p4-crowd/l_d29_0715_F2.png`
- `docs/art/production8/verification/p4-crowd/manifest.json`
- `docs/art/production8/verification/p4-crowd/manifest_art_long.json`
- `docs/art/production8/verification/p4-effects/god_d23_1200_F2_spoil.entities.json.gz`
- `docs/art/production8/verification/p4-effects/god_d23_1200_F2_spoil.jpg`
- `docs/art/production8/verification/p4-effects/god_d23_1200_F2_spoil.png`
- `docs/art/production8/verification/p4-effects/god_d23_1201_F2_gift_food.entities.json.gz`
- `docs/art/production8/verification/p4-effects/god_d23_1201_F2_gift_food.jpg`
- `docs/art/production8/verification/p4-effects/god_d23_1201_F2_gift_food.png`
- `docs/art/production8/verification/p4-effects/god_d23_1202_F2_rot_wood.entities.json.gz`
- `docs/art/production8/verification/p4-effects/god_d23_1202_F2_rot_wood.jpg`
- `docs/art/production8/verification/p4-effects/god_d23_1202_F2_rot_wood.png`
- `docs/art/production8/verification/p4-effects/god_d23_1203_F2_gift.entities.json.gz`
- `docs/art/production8/verification/p4-effects/god_d23_1203_F2_gift.jpg`
- `docs/art/production8/verification/p4-effects/god_d23_1203_F2_gift.png`
- `docs/art/production8/verification/p4-effects/god_d23_1205_F2_windfall_0200ms.entities.json.gz`
- `docs/art/production8/verification/p4-effects/god_d23_1205_F2_windfall_0200ms.jpg`
- `docs/art/production8/verification/p4-effects/god_d23_1205_F2_windfall_0200ms.png`
- `docs/art/production8/verification/p4-effects/god_d23_1205_F2_windfall_1000ms.entities.json.gz`
- `docs/art/production8/verification/p4-effects/god_d23_1205_F2_windfall_1000ms.jpg`
- `docs/art/production8/verification/p4-effects/god_d23_1205_F2_windfall_1000ms.png`
- `docs/art/production8/verification/p4-effects/manifest_art_god.json`
- `docs/art/production8/village_evening.gif`
- `docs/art/production8/village_noon.gif`
- `tools/art_pipeline/merge_capture_segments_p8.py`
- `tools/art_pipeline/pack_evidence_p8.py`
- `tools/art_pipeline/test_pack_evidence_p8.py`
- `tools/art_pipeline/upgrade_published_png_p8.py`

### Part 6 — `485a533fef91be5ef63b73c00f2882f19f074165`

art-p8(p6): retain independent visual failures beside measured improvements

분류: `art_owned`

- `docs/art/production8/BLIND_PROTOCOL.md`
- `docs/art/production8/EVIDENCE_FORMAT.md`
- `docs/art/production8/QA_PLAN.md`
- `docs/art/production8/TASKS.md`
- `docs/art/production8/artboard_sources.json`
- `docs/art/production8/before-after-live.png`
- `docs/art/production8/before-after-society.png`
- `docs/art/production8/blind/activity-freeze.json`
- `docs/art/production8/blind/activity.json`
- `docs/art/production8/blind/activity.sha256`
- `docs/art/production8/blind/observer-a-activity.json`
- `docs/art/production8/blind/observer-a-activity.sha256`
- `docs/art/production8/blind/observer-a-graded.json`
- `docs/art/production8/blind/observer-b-activity.json`
- `docs/art/production8/blind/observer-b-activity.sha256`
- `docs/art/production8/blind/observer-b-graded.json`
- `docs/art/production8/blind_identity_final.json`
- `docs/art/production8/blind_results.json`
- `docs/art/production8/blind_society/NORMALIZATION.md`
- `docs/art/production8/blind_society/frozen_answers.json`
- `docs/art/production8/blind_society/frozen_answers.sha256`
- `docs/art/production8/blind_society/observer-a-numeric-types.json`
- `docs/art/production8/blind_society/observer-a-numeric-types.sha256`
- `docs/art/production8/blind_society/observer-a-original.json`
- `docs/art/production8/blind_society/observer-a-original.sha256`
- `docs/art/production8/blind_society/observer-b-numeric-types.json`
- `docs/art/production8/blind_society/observer-b-numeric-types.sha256`
- `docs/art/production8/blind_society/observer-b-original.json`
- `docs/art/production8/blind_society/observer-b-original.sha256`
- `docs/art/production8/blind_society/results.json`
- `docs/art/production8/boards_config.json`
- `docs/art/production8/crowds.png`
- `docs/art/production8/effects.png`
- `docs/art/production8/gates/p6-tools/README.md`
- `docs/art/production8/gates/p6-tools/type-diagnostics.json.gz`
- `docs/art/production8/gates/p6-tools/type-summary.json`
- `docs/art/production8/gates/p6-tools/types.log.gz`
- `docs/art/production8/gates/p6normalize/NOTES.md`
- `docs/art/production8/gates/preqa/results.tsv`
- `docs/art/production8/harvest/installed_groups_final.json`
- `docs/art/production8/harvest/integrity_final.json`
- `docs/art/production8/ingame_qa.json.gz`
- `docs/art/production8/ingame_qa.md.gz`
- `docs/art/production8/measurements.json.gz`
- `docs/art/production8/measurements.md`
- `docs/art/production8/news.png`
- `docs/art/production8/qa_config.json`
- `docs/art/production8/qa_evidence.json.gz`
- `docs/art/production8/restless.png`
- `docs/art/production8/review/a26-society-exposure-diagnostic.json`
- `docs/art/production8/review/a31-preliminary.json`
- `docs/art/production8/review/a31-preliminary.md`
- `docs/art/production8/review/code_review.json`
- `docs/art/production8/review/code_review.md`
- `docs/art/production8/review/god-live-observation-normalization.json`
- `docs/art/production8/review/god-society-observation-normalization.json`
- `docs/art/production8/review/long-observation-normalization.json`
- `docs/art/production8/review/measurement-attempt1.md`
- `docs/art/production8/review/originals/god-live-objects-key.json`
- `docs/art/production8/review/originals/god-society-objects-key.json`
- `docs/art/production8/review/originals/long-objects-key.json`
- `docs/art/production8/review/originals/observer-a-crowds.json`
- `docs/art/production8/review/originals/observer-a-crowds.sha256`
- `docs/art/production8/review/originals/observer-a-god-live-objects.json`
- `docs/art/production8/review/originals/observer-a-god-society-objects.json`
- `docs/art/production8/review/originals/observer-a-hud-art.json`
- `docs/art/production8/review/originals/observer-a-long-objects.json`
- `docs/art/production8/review/originals/observer-b-crowds.json`
- `docs/art/production8/review/originals/observer-b-crowds.sha256`
- `docs/art/production8/review/originals/observer-b-god-live-objects.json`
- `docs/art/production8/review/originals/observer-b-hud-art.json`
- `docs/art/production8/review/originals/observer-b-long-objects.json`
- `docs/art/production8/review/qa-scope-basedpyright.log`
- `docs/art/production8/review/qa-scope-ruff.log`
- `docs/art/production8/visual_observations.json`
- `docs/art/production8/way.png`
- `tools/art_pipeline/artboards_p8.py`
- `tools/art_pipeline/blind_identity_p8.py`
- `tools/art_pipeline/event_oracle_p8.py`
- `tools/art_pipeline/feed_adapter_p8.py`
- `tools/art_pipeline/harvest_integrity_p8.py`
- `tools/art_pipeline/ingame_qa_founding_p8.py`
- `tools/art_pipeline/ingame_qa_intervention_p8.py`
- `tools/art_pipeline/ingame_qa_new_p8.py`
- `tools/art_pipeline/ingame_qa_news_p8.py`
- `tools/art_pipeline/ingame_qa_observed_p8.py`
- `tools/art_pipeline/ingame_qa_society_p8.py`
- `tools/art_pipeline/measure_days_p8.py`
- `tools/art_pipeline/measure_p8.py`
- `tools/art_pipeline/normalize_evidence_p8.py`
- `tools/art_pipeline/observation_rules_p8.py`
- `tools/art_pipeline/prepare_blind_p8.py`
- `tools/art_pipeline/test_measurement_p8.py`
- `tools/art_pipeline/test_normalize_evidence_p8.py`
- `tools/art_pipeline/test_prepare_blind_p8.py`
- `tools/art_pipeline/way_adapter_p8.py`

### Part 7 — `326f1665c8d32c7cd7a9723e39ce9cd6ceddf951`

art-p8(p7): verify both languages preserve required village wording

분류: `art_owned`

- `docs/art/production8/LOCALIZATION_AUDIT.md`
- `docs/art/production8/TASKS.md`
- `docs/art/production8/localization_audit.json`

### Part 8 — `dd6e246140be10508b743d2dd687cd967f6df0c8`

art-p8(p8): preserve the independent screen verdict without reinterpretation

분류: `art_owned`

- `docs/art/production8/TASKS.md`
- `docs/art/production8/review/final/round1-independent-review.json`
- `docs/art/production8/review/final/round1-independent-review.ko.md`
- `docs/art/production8/review/final/sources.json`
- `docs/art/production8/review/final/sources.sha256`
- `docs/art/production8/review/final/verification.json`

### Part 9 — `41723f37bd53b6d0dd7c17f90d2b6047117b3119`

art-p8(p9): make the evidence tools pass the full static audit

분류: `art_owned`

- `docs/art/production8/DECISIONS.md`
- `docs/art/production8/gates/final-tools/SUMMARY.md`
- `docs/art/production8/gates/final-tools/dict-literal-fix.log`
- `docs/art/production8/gates/final-tools/files.json`
- `docs/art/production8/gates/final-tools/pytest.log`
- `docs/art/production8/gates/final-tools/report-audit.json`
- `docs/art/production8/gates/final-tools/ruff-final.log`
- `docs/art/production8/gates/final-tools/ruff-fix.log`
- `docs/art/production8/gates/final-tools/ruff-summary.json`
- `docs/art/production8/gates/final-tools/ruff.json.gz`
- `docs/art/production8/gates/final-tools/types-after-fix-summary.json`
- `docs/art/production8/gates/final-tools/types-after-fix.json.gz`
- `docs/art/production8/gates/final-tools/types-summary.json`
- `docs/art/production8/gates/final-tools/types.json.gz`
- `docs/art/production8/gates/final-tools/unittest.log`
- `tools/art_pipeline/artboards_p8.py`
- `tools/art_pipeline/event_oracle_p8.py`
- `tools/art_pipeline/feed_adapter_p8.py`
- `tools/art_pipeline/harvest_integrity_p8.py`
- `tools/art_pipeline/ingame_qa_intervention_p8.py`
- `tools/art_pipeline/ingame_qa_news_p8.py`
- `tools/art_pipeline/ingame_qa_observed_p8.py`
- `tools/art_pipeline/ingame_qa_society_p8.py`
- `tools/art_pipeline/measure_days_p8.py`
- `tools/art_pipeline/merge_capture_segments_p8.py`
- `tools/art_pipeline/observation_rules_p8.py`
- `tools/art_pipeline/test_normalize_evidence_p8.py`
- `tools/art_pipeline/test_prepare_blind_p8.py`

## 범위 증거와 제한

첫 부모의 각 일반 커밋 diff를 GOAL §2.1 허용 경로와 대조했다. 병합 커밋의 Rust/시뮬레이션 문서 변화는 §2.2의 명시적 가져오기이며, 아트 편집으로 합산하지 않는다. `world_node.rs` 일반 커밋 hunk는 두 FissionRecord fixture 보완, getter 데이터/직렬화 및 관련 단위 테스트에 한정된다.

보존한 무관한 사용자 변경: tracked `['.gitignore']`, untracked `.ignore` (DECISIONS.md:6). Part 0부터 존재한 변경이며 아트 변경이나 범위 예외가 아니다. 게시 대상에서 제외한다. 위의 0건은 커밋된 아트 변경 판정이다.

JSON에는 각 런타임 경로와 내용 SHA-256, 파일 모드, 정렬된 전체 지문을 보존한다. 런타임 범위는 scripts/scenes/shaders/assets/localization/project.godot 및 world_node.rs이며 scripts/test도 포함한다. Gate 성공이나 시각 PASS를 대신하지 않는다.

## 재현 및 최종 갱신

아래 스크립트를 저장소 루트에서 Python 3으로 실행하면 당시 HEAD를 기준으로 두 산출물을 갱신한다. 생성기의 `CAP`은 실제 캡처 커밋으로 고정하고, 최종 커밋 이후 재실행한다.

```python
import gzip
import hashlib,json,re,subprocess
from pathlib import Path
R=Path.cwd(); OUT=R/'docs/art/production8'; CAP='cefc77b088d6fa3baf21b766cdc40e3fdf045d36'; BASE='a54486e8'; WORLD='rust/crates/sim-bridge/src/ffi/world_node.rs'
def git(*args): return subprocess.check_output(['git',*args],cwd=R).decode().strip()
def allowed(p): return p==WORLD or p=='project.godot' or p=='docs/art/TEST_LEDGER.md' or p.startswith(('scripts/','scenes/','shaders/','assets/','localization/','docs/art/production8/','tools/art_pipeline/','rust/crates/sim-test/tests/'))
def runtime(p): return p==WORLD or p=='project.godot' or p.startswith(('scripts/','scenes/','shaders/','assets/','localization/'))
def changed(a,b): return git('diff','--name-only',a,b).splitlines()
def fingerprint(ref):
    entries=[]
    for line in git('ls-tree','-r',ref).splitlines():
        info,path=line.split('\t',1); mode,kind,oid=info.split()
        if runtime(path): entries.append((path,oid,mode))
    proc=subprocess.Popen(['git','cat-file','--batch'],stdin=subprocess.PIPE,stdout=subprocess.PIPE)
    raw,_=proc.communicate(('\n'.join(e[1] for e in entries)+'\n').encode()); pos=0; rows=[]
    for path,oid,mode in entries:
        end=raw.index(b'\n',pos); size=int(raw[pos:end].split()[-1]); start=end+1; data=raw[start:start+size];pos=start+size+1
        rows.append({'path':path,'mode':mode,'content_sha256':hashlib.sha256(data).hexdigest()})
    return {'file_count':len(rows),'tree_sha256':hashlib.sha256(json.dumps(rows,sort_keys=True,separators=(',',':')).encode()).hexdigest(),'files':rows}
head=git('rev-parse','HEAD'); base=git('rev-parse',BASE); commits=[]; violations=[]; merge=None
for sha in git('rev-list','--first-parent','--reverse',f'{base}..{head}').splitlines():
    subject=git('show','-s','--format=%s',sha); parents=git('show','-s','--format=%P',sha).split(); files=changed(parents[0],sha)
    entry={'sha':sha,'part':int(re.search(r'art-p8\(p(\d+)\)',subject)[1]),'subject':subject,'changed_files':files,'scope':'authorized_simulation_merge' if len(parents)>1 else 'art_owned'}
    if len(parents)>1:
        merge=entry|{'parents':parents,'imported_commits':git('rev-list',f'{parents[0]}..{parents[1]}').splitlines()}
    else:
        bad=[p for p in files if not allowed(p)]; entry['outside_allowed_paths']=bad;violations.extend({'sha':sha,'path':p} for p in bad)
    commits.append(entry)
cap=fingerprint(CAP);current=fingerprint(head)
working=[p for p in git('diff','--name-only','HEAD').splitlines() if runtime(p)]
untracked=[p for p in git('ls-files','--others','--exclude-standard').splitlines() if runtime(p)]
workoutside=[p for p in git('diff','--name-only','HEAD').splitlines() if not allowed(p)]
payload={'status':'part9_preparation_not_final_head','baseline':base,'capture_commit':CAP,'inventory_head':head,'commits':commits,'simulation_merge':merge,'art_owned_scope_violations':violations,'preserved_unrelated_user_changes':{'tracked':workoutside,'untracked':['.ignore'],'source':'docs/art/production8/DECISIONS.md:6','publication':'excluded'},'runtime_fingerprints':{'capture':cap,'inventory_head':current},'runtime_changed_paths':changed(CAP,head),'runtime_fingerprints_equal':cap==current,'runtime_worktree_tracked_changes':working,'runtime_worktree_untracked_files':untracked,'limitations':['Commit inventory uses first-parent deltas; authorized simulation merge is listed separately.','Rust world_node.rs exception reviewed by hunk: getter collectors/serialization and their tests only.','Static-source tests require individual TEST_LEDGER rows; this index does not replace the gate.','Pending worktree files and future Parts 6-9 commits require refresh before final publication.']}
payload['runtime_changed_paths']=[p for p in payload['runtime_changed_paths'] if runtime(p)]
(OUT/'change_index.json.gz').write_bytes(gzip.compress((json.dumps(payload,ensure_ascii=False,indent=2)+'\n').encode(),mtime=0))
lines=['# Art Pass 8 변경 인덱스 — Part 9 준비','',f'- 기준: `{base}`',f'- 캡처: `{CAP}`',f'- 조사 HEAD: `{head}`. 최종 HEAD 확정이 아니다.',f'- 아트 소유 커밋 경로 위반: **{len(violations)}**. 허용된 시뮬레이션 병합은 별도 분류한다.',f'- 런타임 지문: {cap["file_count"]}개 경로, 캡처/조사 HEAD 동일: **{cap==current}**.',f'- SHA-256: `{cap["tree_sha256"]}`.',f'- 런타임 tracked 작업트리 변경 {len(working)}개, untracked {len(untracked)}개.','', '## 실질 변경과 핵심 파일','', '- Part 0: Pass 12/13 병합, getter fixture 호환성, 실제 날짜·분열 조건 측정과 기준 캡처.', '- Part 1: `scripts/ui/panels/god_hand_panel.gd`, `scripts/ui/panels/feed_food_ledger.gd` 중심의 우선순위 피드, 보호 이벤트 누적 +N, 재고 변동 요약 및 getter 이벤트 식별자.', '- Part 2: `scripts/ui/offscreen_marker.gd`, `scripts/ui/founding_group_renderer.gd`, `scripts/ui/settlement_labels.gd`: 9px 경계 방향표, 거리, 클릭 이동, 말뚝과 배타 표시.', '- Part 3: `scripts/core/settlement_selection.gd`, `scripts/ui/restless_meter.gd`, `world_node.rs`: P/θ 추가 컬럼과 정확한 정수 3단계 계기, 선택 마을 상태.', '- Part 4: 좌석 충돌 완화, 아침 냉기 표현과 세계 효과, `frame_measure_session.gd`/`frame_cpu_probe.gd` 동일 프로세스 A30 측정, 단일 마을 레이어 토글.', '- Part 5: `capture_plans.gd`/`capture_framings.gd`의 측정 날짜 재매핑, P/G 별도 리셋과 원시 getter 메타데이터.','', '## 커밋별 변경 파일','']
for c in commits:
    lines.extend([f'### Part {c["part"]} — `{c["sha"]}`', '',c['subject'], '',f'분류: `{c["scope"]}`',''])
    lines.extend('- `'+p+'`' for p in c['changed_files']);lines.append('')
lines.extend(['## 범위 증거와 제한','', '첫 부모의 각 일반 커밋 diff를 GOAL §2.1 허용 경로와 대조했다. 병합 커밋의 Rust/시뮬레이션 문서 변화는 §2.2의 명시적 가져오기이며, 아트 편집으로 합산하지 않는다. `world_node.rs` 일반 커밋 hunk는 두 FissionRecord fixture 보완, getter 데이터/직렬화 및 관련 단위 테스트에 한정된다.', '',f'보존한 무관한 사용자 변경: tracked `{workoutside}`, untracked `.ignore` (DECISIONS.md:6). Part 0부터 존재한 변경이며 아트 변경이나 범위 예외가 아니다. 게시 대상에서 제외한다. 위의 0건은 커밋된 아트 변경 판정이다.', '', 'JSON에는 각 런타임 경로와 내용 SHA-256, 파일 모드, 정렬된 전체 지문을 보존한다. 런타임 범위는 scripts/scenes/shaders/assets/localization/project.godot 및 world_node.rs이며 scripts/test도 포함한다. Gate 성공이나 시각 PASS를 대신하지 않는다.', '', '## 재현 및 최종 갱신', '', '아래 스크립트를 저장소 루트에서 Python 3으로 실행하면 당시 HEAD를 기준으로 두 산출물을 갱신한다. 생성기의 `CAP`은 실제 캡처 커밋으로 고정하고, 최종 커밋 이후 재실행한다.', '', '```python',Path('/tmp/p8_change_index.py').read_text(),'```'])
(OUT/'CHANGE_INDEX.md').write_text('\n'.join(lines)+'\n')
print(json.dumps({'commits':len(commits),'scope_violations':violations,'runtime_files':cap['file_count'],'runtime_equal':cap==current,'pending_scope':workoutside}))

```
