# Mutation Audit — Revitalize Pass 2

597개 기본 mutant 감사 완료: **593 killed / 4 survived / 0 invalid / 0 baseline-invalid**. 원시 kill rate는 **593/597 = 99.329983%**이며 100%가 아니다. 네 survivor의 실제 계약상 이유를 아래에 모두 기록했다. 별도 의미 검증 보충 3개는 3/3 killed이며 기본 분모에 섞지 않았다.

## 코드와 실행 경계

초기 코드 HEAD: 005ca150e3b5daa75cfe6a21047a6897744f780b. 초기 감사가 오래된 HUD snapshot 기준을 발견했다. 승인된 legend 변경의 159행과 SHA256을 strict equality로 다시 고정한 커밋 68bb8fff32c713a85018cf92751eb43580067b48에서 597개 전체를 다시 실행했다. 동작 코드나 허용 조건을 약화하지 않았다.

실행 worktree: /home/hyunlord/work/mutation-audit. 모든 Cargo 명령은 그 안의 rust/에서 실행했고 CARGO_TARGET_DIR도 그 작업 트리의 rust/target이다. jobs=10, RUST_TEST_THREADS=8 환경에서 개별 감사 함수는 --exact --test-threads=1 --nocapture로 격리했다. 기준 함수가 통과한 경우에만 mutant를 실행했다. 컴파일 오류·0개 선택·timeout·signal 종료는 kill이 아니다.

최초 실행 전 가용 RAM 79 GiB / 디스크 802 GiB, 전체 재실행 전 84 GiB / 795 GiB. 출력을 임의로 pass 처리하지 않았고 매 변형 후 원문을 복구했다. active-mutant journal, 파일 fingerprint, 잠금 및 중단 복구를 사용했다. 최종 소스 복구 여부는 별도 restoration.json에 기록했다.

## 범위

Pass 1 canonical 113개 함수 계약 + 역사적 장부 추가 함수 1개 + screen contract 3개 = **117개 함수**. 소스 토큰 계약은 114개, 나머지 3개 행동 검증 이력은 N/A로 명시했다. 465개 literal 치환과 132개 금지 토큰 삽입을 각각 수행했다. 초기 1,213개 lexical 항목은 파일 연결·helper·진단 문자열·대안 표현을 분류한 reconciliation.json으로 추적한다. 미해결 매핑은 0개이다.

원본 token 치환은 같은 파일의 모든 동일 문자열을 __MUTATED__로 치환한다. 부재 검사에는 별도 삽입 mutant를 사용하며 주석 제거 및 함수 범위를 고려했다. 조건부 허용 목록이나 동등한 공백 표현을 불필요하게 강화하지 않았다. 117개 함수 목록은 아래에 전부 포함되어 있다.

## 전후 결과

| 단계 | 기본 대상 | killed | survived | invalid | baseline-invalid | 원시 kill rate |
|---|---:|---:|---:|---:|---:|---|
| 최초 005ca150 | 597 | 592 | 4 | 0 | 1 | 592/596 scored = 99.328859%; 1개 baseline 부적격 별도 |
| snapshot repin 후 68bb8fff | 597 | 593 | 4 | 0 | 0 | 593/597 = 99.329983% |
| 별도 의미 보충 | 3 | 3 | 0 | 0 | 0 | 3/3; 기본 분모와 분리 |

네 계약상 동등/적용 제외 mutant를 제외하면 기본 비동등 사례는 593/593 killed이다. 이것은 원시 597개 전체 kill rate를 100%라고 부르는 근거가 아니며, 프로그램 전체 의미나 런타임 reset 작동을 증명하지 않는다.

## 발견한 snapshot 기준 불일치

harness_p14_delta_status_panel.rs의 A17은 HUD topbar를 105행 및 이전 SHA256으로 고정하고 있었다. 최종 localized activity legend는 159행으로 변경되어 첫 baseline이 실패했다. 최초 결과의 baseline-invalid를 보존했고, TEST_LEDGER.md에 A17.2/A17.3 두 input repin을 기록했다. 새 SHA256은 d24ba0db899d068a58d8804cc04b80a5db0f7ffdc9704f832033b3e4309a02b1이다. equality 자체는 그대로다. 앞선 제한된 정적 검사 묶음은 이 guard를 포함하지 않아 놓쳤다. 전체 재감사에서 baseline PASS 및 extends 치환 kill을 확인했다.

## 모든 survivor와 보충 결과

### e85593cda3abf27bcd71

함수: rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs :: harness_g_a13_existing_p12_alpha_a4_updated_to_eight

변형 대상: rust/crates/sim-test/tests/harness_p12_alpha_camera_zoom.rs / Vector2(8.0,8.0)

a13 inspects no_ws(maximum-zoom test). Removing the compact literal leaves spaced accepted alternatives with the same normalized value. Requiring one arbitrary formatting spelling would weaken the intended whitespace tolerance. Separate normalized replacement removes all equivalent spellings and is killed.

원본과 보정 후 모두 survived. 원본 로그 ID는 pass2-mutation-run/results.jsonl에 보존되며, 보정 후 결과 로그: pass2-mutation-final/e85593cda3abf27bcd71-d8a77ac773a8-1791622176774508456-mutant.log

### 4f909ec40c28761bd9f2

함수: rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs :: harness_g_a14_existing_p13_alpha_a3_updated_to_eight

변형 대상: rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs / Vector2(8.0,8.0)

a14 has the same normalized maximum-zoom contract: remaining spaced alternatives satisfy the semantic value. Separate normalized replacement is killed. Original exact-literal survivor is retained.

원본과 보정 후 모두 survived. 원본 로그 ID는 pass2-mutation-run/results.jsonl에 보존되며, 보정 후 결과 로그: pass2-mutation-final/4f909ec40c28761bd9f2-34646965bb20-1791622177424778817-mutant.log

### b8b8b9e8a91931b8392a

함수: rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs :: harness_p14_gamma_a23_world_renderer_no_extra_ffi_added

변형 대상: scripts/ui/world_renderer.gd / world_sim.connect(

a23 permits only allowed FFI names and constrains connect only when present. Removing world_sim.connect( removes the optional call from collection, so the conditional restriction does not apply. A forbidden-connect insertion is killed. Required reset-subscription presence is NOT established by this test.

원본과 보정 후 모두 survived. 원본 로그 ID는 pass2-mutation-run/results.jsonl에 보존되며, 보정 후 결과 로그: pass2-mutation-final/b8b8b9e8a91931b8392a-ce01ba11a582-1791622449466945166-mutant.log

### 6bdd0422fab1ef7cd6f1

함수: rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs :: harness_p14_gamma_a23_world_renderer_no_extra_ffi_added

변형 대상: scripts/ui/world_renderer.gd / world_sim.connect("world_reset", refresh_snapshot)

Removing the only exact reset connect statement also removes connect from the observed call set; the optional allowlist remains satisfied. This test does not require reset presence. The separately killed forbidden-connect insertion validates the conditional restriction without adding an arbitrary presence requirement.

원본과 보정 후 모두 survived. 원본 로그 ID는 pass2-mutation-run/results.jsonl에 보존되며, 보정 후 결과 로그: pass2-mutation-final/6bdd0422fab1ef7cd6f1-c91e1318c482-1791622454496110117-mutant.log

정규화 보충은 a13/a14의 공백 동등 zoom-eight 문자열을 모두 치환해 각각 killed. 세 번째 보충은 world_sim.connect("unapproved_signal", refresh_snapshot)를 삽입해 a23에서 killed. reset 연결의 필수 존재는 a23이 증명하는 계약이 아니며, 이번 범위에서 별도 필수 존재 assertion을 확인하지 못했다.

## 파일별 기본 kill rate 전후

| 테스트 파일 | 이전 killed/scored | 이전 baseline-invalid | 이후 killed/전체 | 이후 survivor |
|---|---:|---:|---:|---:|
| rust/crates/sim-test/tests/harness_b1_zoom_default_raise.rs | 3/3 | 0 | 3/3 | 0 |
| rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs | 15/17 | 0 | 15/17 | 2 |
| rust/crates/sim-test/tests/harness_h_phase_a_input_camera_color.rs | 7/7 | 0 | 7/7 | 0 |
| rust/crates/sim-test/tests/harness_p11_alpha_agent_renderer.rs | 16/16 | 0 | 16/16 | 0 |
| rust/crates/sim-test/tests/harness_p12_alpha_camera_zoom.rs | 3/3 | 0 | 3/3 | 0 |
| rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs | 27/27 | 0 | 27/27 | 0 |
| rust/crates/sim-test/tests/harness_p12_beta_terrain.rs | 32/32 | 0 | 32/32 | 0 |
| rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs | 32/32 | 0 | 32/32 | 0 |
| rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs | 31/31 | 0 | 31/31 | 0 |
| rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs | 50/50 | 0 | 50/50 | 0 |
| rust/crates/sim-test/tests/harness_p13_delta_basic_hud.rs | 17/17 | 0 | 17/17 | 0 |
| rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs | 42/42 | 0 | 42/42 | 0 |
| rust/crates/sim-test/tests/harness_p13_gamma_interaction_cues.rs | 11/11 | 0 | 11/11 | 0 |
| rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs | 16/16 | 0 | 16/16 | 0 |
| rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs | 91/91 | 0 | 91/91 | 0 |
| rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs | 32/32 | 1 | 33/33 | 0 |
| rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs | 16/16 | 0 | 16/16 | 0 |
| rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs | 25/27 | 0 | 25/27 | 2 |
| rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs | 19/19 | 0 | 19/19 | 0 |
| rust/crates/sim-test/tests/harness_revitalize_screen_contract.rs | 29/29 | 0 | 29/29 | 0 |
| rust/crates/sim-test/tests/harness_s16_alpha0_resource_substrate.rs | 6/6 | 0 | 6/6 | 0 |
| rust/crates/sim-test/tests/harness_s16_gamma_visual.rs | 22/22 | 0 | 22/22 | 0 |
| rust/crates/sim-test/tests/harness_show_agent_needs_bars.rs | 18/18 | 0 | 18/18 | 0 |
| rust/crates/sim-test/tests/harness_show_death_visual.rs | 19/19 | 0 | 19/19 | 0 |
| rust/crates/sim-test/tests/harness_t7_10_b1_space_toggle.rs | 4/4 | 0 | 4/4 | 0 |
| rust/crates/sim-test/tests/harness_t7_9_b_render_mechanism.rs | 9/9 | 0 | 9/9 | 0 |

## 검사 대상 소스 파일별 기본 kill rate 전후

| 검사 대상 소스 | 이전 killed/scored | 이전 baseline-invalid | 이후 killed/전체 | 이후 survivor |
|---|---:|---:|---:|---:|
| rust/crates/sim-test/tests/harness_p12_alpha_camera_zoom.rs | 3/4 | 0 | 3/4 | 1 |
| rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs | 3/4 | 0 | 3/4 | 1 |
| rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs | 4/4 | 0 | 4/4 | 0 |
| scenes/main.tscn | 51/51 | 0 | 51/51 | 0 |
| scripts/ui/activity_trail_renderer.gd | 3/3 | 0 | 3/3 | 0 |
| scripts/ui/agent_renderer.gd | 11/11 | 0 | 11/11 | 0 |
| scripts/ui/building_renderer.gd | 262/262 | 0 | 262/262 | 0 |
| scripts/ui/camera_controller.gd | 53/53 | 0 | 53/53 | 0 |
| scripts/ui/panels/hud_status_panel.gd | 33/33 | 0 | 33/33 | 0 |
| scripts/ui/panels/hud_topbar.gd | 10/10 | 1 | 11/11 | 0 |
| scripts/ui/terrain_renderer.gd | 63/63 | 0 | 63/63 | 0 |
| scripts/ui/world_renderer.gd | 91/93 | 0 | 91/93 | 2 |
| shaders/palette_swap.gdshader | 5/5 | 0 | 5/5 | 0 |

## 117개 함수 전체 명세

| 출처 | 파일 | 함수 | killed | survived | 적용 |
|---|---|---|---:|---:|---|
| F001 | rust/crates/sim-test/tests/harness_b1_zoom_default_raise.rs | harness_b1_a1_zoom_default_equals_five | 3 | 0 | source-token |
| F002 | rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs | harness_g_a10_zoom_min_and_default_preserved | 5 | 0 | source-token |
| F106 | rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs | harness_g_a13_existing_p12_alpha_a4_updated_to_eight | 3 | 1 | source-token |
| F107 | rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs | harness_g_a14_existing_p13_alpha_a3_updated_to_eight | 3 | 1 | source-token |
| F003 | rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs | harness_g_a15_existing_p14_zeta_regression_literal_updated | 4 | 0 | source-token |
| F004 | rust/crates/sim-test/tests/harness_h_phase_a_input_camera_color.rs | harness_h_a9_zoom_invariants_unchanged | 7 | 0 | source-token |
| historical ledger extra | rust/crates/sim-test/tests/harness_p11_alpha_agent_renderer.rs | harness_p11_alpha_agent_renderer_interpolates_positions | 11 | 0 | source-token |
| F005 | rust/crates/sim-test/tests/harness_p11_alpha_agent_renderer.rs | harness_p11_alpha_shader_modulates_palette_output | 5 | 0 | source-token |
| F006 | rust/crates/sim-test/tests/harness_p12_alpha_camera_zoom.rs | harness_p12_alpha_a5_zoom_default_constant | 3 | 0 | source-token |
| F007 | rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs | harness_p12_beta2_a10_renderer_calls_ffi_snapshot_method | 3 | 0 | source-token |
| F008 | rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs | harness_p12_beta2_a11_reaper_loop_frees_despawned_sprites | 3 | 0 | source-token |
| F009 | rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs | harness_p12_beta2_a14_phase12_alpha_and_beta1_invariants_preserved | 15 | 0 | source-token |
| F010 | rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs | harness_p12_beta2_a8_renderer_declares_construction_sprite_path_constant | 3 | 0 | source-token |
| F011 | rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs | harness_p12_beta2_a9_alpha_remap_constants_have_exact_values | 3 | 0 | source-token |
| F012 | rust/crates/sim-test/tests/harness_p12_beta_terrain.rs | harness_p12_beta_a10_bootstrap_building_sprite_loaded | 3 | 0 | source-token |
| F013 | rust/crates/sim-test/tests/harness_p12_beta_terrain.rs | harness_p12_beta_a11_bootstrap_sprite_positioned_at_bootstrap_tile | 3 | 0 | source-token |
| F014 | rust/crates/sim-test/tests/harness_p12_beta_terrain.rs | harness_p12_beta_a12_z_order_constants_match_spec | 4 | 0 | source-token |
| F015 | rust/crates/sim-test/tests/harness_p12_beta_terrain.rs | harness_p12_beta_a13_z_order_constants_are_assigned_to_nodes | 4 | 0 | source-token |
| F016 | rust/crates/sim-test/tests/harness_p12_beta_terrain.rs | harness_p12_beta_a6_world_renderer_loads_terrain_tileset | 5 | 0 | source-token |
| F017 | rust/crates/sim-test/tests/harness_p12_beta_terrain.rs | harness_p12_beta_a7_world_renderer_instantiates_tilemaplayer_with_tileset | 5 | 0 | source-token |
| F018 | rust/crates/sim-test/tests/harness_p12_beta_terrain.rs | harness_p12_beta_a8_world_renderer_populates_all_grid_cells | 3 | 0 | source-token |
| F019 | rust/crates/sim-test/tests/harness_p12_beta_terrain.rs | harness_p12_beta_a9_terrain_placement_is_deterministic | 5 | 0 | source-token |
| F020 | rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs | harness_p12_gamma_a10_world_renderer_polls_ffi_and_reaps_stale_sprites | 5 | 0 | source-token |
| F021 | rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs | harness_p12_gamma_a12_phase12_alpha_beta1_beta2_invariants_preserved | 19 | 0 | source-token |
| F022 | rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs | harness_p12_gamma_a15_world_renderer_reads_formation_arrays | 5 | 0 | source-token |
| F023 | rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs | harness_p12_gamma_a9_world_renderer_declares_furniture_sprite_path_and_z_constants | 3 | 0 | source-token |
| F024 | rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs | harness_p13_alpha_a11_phase12_beta1_terrain_tileset_preserved | 5 | 0 | source-token |
| F025 | rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs | harness_p13_alpha_a12_phase12_beta2_construction_z_preserved | 4 | 0 | source-token |
| F026 | rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs | harness_p13_alpha_a13_phase12_gamma_furniture_z_preserved | 5 | 0 | source-token |
| F027 | rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs | harness_p13_alpha_a14_three_distinct_building_sprite_paths | 3 | 0 | source-token |
| F028 | rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs | harness_p13_alpha_a1_camera_controller_zoom_default_3x | 3 | 0 | source-token |
| F029 | rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs | harness_p13_alpha_a4_bootstrap_building_path_campfire | 2 | 0 | source-token |
| F030 | rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs | harness_p13_alpha_a5_bootstrap_no_longer_points_to_cairn | 3 | 0 | source-token |
| F031 | rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs | harness_p13_alpha_a6_construction_sprite_path_unchanged | 3 | 0 | source-token |
| F032 | rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs | harness_p13_alpha_a7_furniture_sprite_path_unchanged | 3 | 0 | source-token |
| F033 | rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs | harness_p13_beta_a11_phase13_alpha_zoom_default_3x_preserved | 3 | 0 | source-token |
| F034 | rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs | harness_p13_beta_a12_phase13_alpha_bootstrap_campfire_preserved | 2 | 0 | source-token |
| F035 | rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs | harness_p13_beta_a13_phase12_beta1_terrain_tileset_preserved | 5 | 0 | source-token |
| F036 | rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs | harness_p13_beta_a14_phase12_beta2_construction_z_5_preserved | 3 | 0 | source-token |
| F037 | rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs | harness_p13_beta_a15_phase12_gamma_furniture_z_4_preserved | 5 | 0 | source-token |
| F038 | rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs | harness_p13_beta_a1_resource_sprite_path_constant_declared | 6 | 0 | source-token |
| F039 | rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs | harness_p13_beta_a2_z_resource_constant_equals_3 | 4 | 0 | source-token |
| F040 | rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs | harness_p13_beta_a3_resource_count_equals_20 | 6 | 0 | source-token |
| F041 | rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs | harness_p13_beta_a4_resource_seed_constant_declared | 6 | 0 | source-token |
| F042 | rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs | harness_p13_beta_a5_resource_placement_uses_seeded_rng | 6 | 0 | source-token |
| F043 | rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs | harness_p13_beta_a7_z_order_chain_invariant | 4 | 0 | source-token |
| F044 | rust/crates/sim-test/tests/harness_p13_delta_basic_hud.rs | harness_p13_delta_a14_phase13_alpha_default_zoom_3x_unchanged | 3 | 0 | source-token |
| F045 | rust/crates/sim-test/tests/harness_p13_delta_basic_hud.rs | harness_p13_delta_a15_phase13_alpha_bootstrap_campfire_unchanged | 2 | 0 | source-token |
| F046 | rust/crates/sim-test/tests/harness_p13_delta_basic_hud.rs | harness_p13_delta_a16_phase13_beta_resource_constants_unchanged | 6 | 0 | source-token |
| F047 | rust/crates/sim-test/tests/harness_p13_delta_basic_hud.rs | harness_p13_delta_a3_hud_topbar_declares_four_label_members | 6 | 0 | source-token |
| F048 | rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs | harness_p13_epsilon_a11_phase13_alpha_default_zoom_3x_preserved | 3 | 0 | source-token |
| F049 | rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs | harness_p13_epsilon_a12_phase13_alpha_centre_bootstrap_preserved | 2 | 0 | source-token |
| F050 | rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs | harness_p13_epsilon_a13_phase13_beta_resource_constants_preserved | 6 | 0 | source-token |
| F051 | rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs | harness_p13_epsilon_a1_bootstrap_x_left_constant_declared | 6 | 0 | source-token |
| F052 | rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs | harness_p13_epsilon_a2_bootstrap_x_right_constant_declared | 6 | 0 | source-token |
| F053 | rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs | harness_p13_epsilon_a3_three_on_building_placed_calls_total | 6 | 0 | source-token |
| F054 | rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs | harness_p13_epsilon_a4_on_building_placed_arg_signatures | 3 | 0 | source-token |
| F055 | rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs | harness_p13_epsilon_a5_extra_sprite_block_reuses_building_tex | 3 | 0 | source-token |
| F056 | rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs | harness_p13_epsilon_a6_extra_sprite_z_index_is_z_building | 4 | 0 | source-token |
| F057 | rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs | harness_p13_epsilon_a7_extra_sprite_y_position_uses_bootstrap_y | 3 | 0 | source-token |
| F058 | rust/crates/sim-test/tests/harness_p13_gamma_interaction_cues.rs | harness_p13_gamma_a13_phase13_alpha_zoom_default_3x_preserved | 3 | 0 | source-token |
| F059 | rust/crates/sim-test/tests/harness_p13_gamma_interaction_cues.rs | harness_p13_gamma_a14_phase13_alpha_bootstrap_campfire_preserved | 2 | 0 | source-token |
| F060 | rust/crates/sim-test/tests/harness_p13_gamma_interaction_cues.rs | harness_p13_gamma_a15_phase13_beta_resource_constants_preserved | 6 | 0 | source-token |
| F061 | rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs | harness_p14_alpha_a15_camera_zoom_invariants_preserved | 7 | 0 | source-token |
| F062 | rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs | harness_p14_alpha_a16_phase13_bootstrap_three_campfires_preserved | 3 | 0 | source-token |
| F063 | rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs | harness_p14_alpha_a17_phase13_beta_resource_constants_preserved | 6 | 0 | source-token |
| F064 | rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs | harness_p14_beta_a10_water_modulate_branch_present | 5 | 0 | source-token |
| F065 | rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs | harness_p14_beta_a11_village_fixture_paths_has_exactly_four_literals | 6 | 0 | source-token |
| F066 | rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs | harness_p14_beta_a12_village_fixture_positions_match_diamond_layout | 3 | 0 | source-token |
| F067 | rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs | harness_p14_beta_a13_z_village_fixture_equals_five | 4 | 0 | source-token |
| F068 | rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs | harness_p14_beta_a14_village_fixture_placement_loop_present | 3 | 0 | source-token |
| F069 | rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs | harness_p14_beta_a15_phase_13_beta_resource_constants_preserved | 6 | 0 | source-token |
| F070 | rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs | harness_p14_beta_a16_phase_13_alpha_bootstrap_constants_preserved | 3 | 0 | source-token |
| F071 | rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs | harness_p14_beta_a17_phase_12_construction_constants_preserved | 3 | 0 | source-token |
| F072 | rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs | harness_p14_beta_a18_phase_12_gamma_furniture_constants_preserved | 5 | 0 | source-token |
| F073 | rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs | harness_p14_beta_a1_resource_type_paths_array_declared | 6 | 0 | source-token |
| F074 | rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs | harness_p14_beta_a2_resource_type_paths_has_exactly_five_entries | 6 | 0 | source-token |
| F075 | rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs | harness_p14_beta_a3_resource_type_paths_index_0_references_legacy_constant | 6 | 0 | source-token |
| F076 | rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs | harness_p14_beta_a4_resource_type_paths_index_1_wood_literal | 6 | 0 | source-token |
| F077 | rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs | harness_p14_beta_a5_resource_type_paths_index_2_stone_literal | 6 | 0 | source-token |
| F078 | rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs | harness_p14_beta_a6_resource_type_paths_index_3_water_literal | 6 | 0 | source-token |
| F079 | rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs | harness_p14_beta_a7_resource_type_paths_index_4_food_literal | 6 | 0 | source-token |
| F080 | rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs | harness_p14_beta_a8_resource_water_tint_constant_value | 5 | 0 | source-token |
| F081 | rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs | harness_p14_beta_a9_resource_scatter_loop_uses_modulo_cycle_and_indexed_iterator | 6 | 0 | source-token |
| F082 | rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs | harness_p14_delta_a14_phase_14_beta_resource_registry_preserved | 6 | 0 | source-token |
| F083 | rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs | harness_p14_delta_a16_village_fixture_constants_preserved_in_world_renderer | 6 | 0 | source-token |
| F084 | rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs | harness_p14_delta_a17_hud_topbar_file_byte_or_line_count_unchanged_against_pre_phase_14_delta_head | 1 | 0 | source-token |
| F085 | rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs | harness_p14_delta_a2_ticks_per_day_constant_present | 3 | 0 | source-token |
| F086 | rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs | harness_p14_delta_a3_days_per_year_constant_present | 5 | 0 | source-token |
| F087 | rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs | harness_p14_delta_a4_resource_count_consistency_with_phase_14_beta | 6 | 0 | source-token |
| F088 | rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs | harness_p14_delta_a5_resource_labels_array_exact_order | 6 | 0 | source-token |
| F108 | rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs | harness_p14_epsilon_a12_main_tscn_load_steps_equals_12 | 4 | 0 | source-token |
| F089 | rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs | harness_p14_epsilon_a14_regression_guard_world_renderer_phase_14_beta_intact | 6 | 0 | source-token |
| F110 | rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs | harness_p14_epsilon_a16_regression_guard_hud_status_panel_phase_14_delta_intact | 3 | 0 | source-token |
| F090 | rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs | harness_p14_epsilon_a4_trail_alpha_constant_equals_0_4 | 3 | 0 | source-token |
| F091 | rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs | harness_p14_gamma_a23_world_renderer_no_extra_ffi_added | 3 | 2 | source-token |
| F092 | rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs | harness_p14_gamma_a27_phase14_beta_invariants_preserved | 6 | 0 | source-token |
| F093 | rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs | harness_p14_gamma_a28_phase12_to_13_visual_invariants_preserved | 16 | 0 | source-token |
| F109 | rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs | harness_zoom_a17_main_tscn_load_steps_equals_12 | 4 | 0 | source-token |
| F094 | rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs | harness_zoom_a19_phase12a_13a_camera_controller_zoom_invariants_intact | 6 | 0 | source-token |
| F095 | rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs | harness_zoom_a21_phase14b_world_renderer_invariants_intact | 6 | 0 | source-token |
| F111 | rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs | harness_zoom_a23_phase14d_hud_status_panel_intact | 3 | 0 | source-token |
| F096 | rust/crates/sim-test/tests/harness_resource_scarcity.rs | harness_scarcity_a16_production_init_seeds_all_three_kinds_finite | 0 | 0 | N/A: behavioral provenance row; no source-text assertion |
| screen contract | rust/crates/sim-test/tests/harness_revitalize_screen_contract.rs | harness_building_renderer_reads_snapshot_only | 15 | 0 | source-token |
| screen contract | rust/crates/sim-test/tests/harness_revitalize_screen_contract.rs | harness_economy_hud_reads_sim_time_and_tallies | 11 | 0 | source-token |
| screen contract | rust/crates/sim-test/tests/harness_revitalize_screen_contract.rs | harness_territory_terrain_renderer_uses_ffi | 3 | 0 | source-token |
| F097 | rust/crates/sim-test/tests/harness_s16_alpha0_resource_substrate.rs | harness_resource_decorative_layer_preserved_static | 6 | 0 | source-token |
| F098 | rust/crates/sim-test/tests/harness_s16_gamma_visual.rs | harness_s16_gamma_a4_world_renderer_marker_is_polygon2d | 6 | 0 | source-token |
| F099 | rust/crates/sim-test/tests/harness_s16_gamma_visual.rs | harness_s16_gamma_a5_z_order_above_decorative | 4 | 0 | source-token |
| F100 | rust/crates/sim-test/tests/harness_s16_gamma_visual.rs | harness_s16_gamma_a6_decorative_invariant_preserved | 6 | 0 | source-token |
| F101 | rust/crates/sim-test/tests/harness_s16_gamma_visual.rs | harness_s16_gamma_a7_backend_truth_positions_preserved | 6 | 0 | source-token |
| F112 | rust/crates/sim-test/tests/harness_show_agent_needs_bars.rs | harness_needs_a6_scene_wiring_additive | 18 | 0 | source-token |
| F113 | rust/crates/sim-test/tests/harness_show_death_visual.rs | harness_death_a6_scene_wiring_additive | 19 | 0 | source-token |
| F102 | rust/crates/sim-test/tests/harness_show_resource_depletion.rs | harness_resource_depletion_a10_live_depletion_surfaces | 0 | 0 | N/A: behavioral provenance row; no source-text assertion |
| F103 | rust/crates/sim-test/tests/harness_show_resource_depletion.rs | harness_resource_depletion_a9_production_scene_registers_finite_capacities | 0 | 0 | N/A: behavioral provenance row; no source-text assertion |
| F104 | rust/crates/sim-test/tests/harness_t7_10_b1_space_toggle.rs | harness_t7_10_b1_substrate_calls_survive_off_gate | 4 | 0 | source-token |
| F105 | rust/crates/sim-test/tests/harness_t7_9_b_render_mechanism.rs | harness_t7_9_b_world_renderer_render_path_present | 9 | 0 | source-token |

## 증거와 재현

증거 루트: .harness/evidence/revitalize/staged/ (생성 실행 자료, Git ignored).

- pass2-mutation-run/: 초기 597개 결과, 개별 baseline/mutant 로그, inventory와 reconciliation.
- pass2-mutation-final/: 보정 HEAD 597개 결과, summary.json, function-coverage.json, survivor-dispositions.json, restoration.json.
- pass2-mutation-supplemental/: 별도 3개 보충 결과와 개별 로그.
- pass2-mutation-review1-test.py: 중단 복구 선행/잠금, timeout·signal invalid, infrastructure invalid 재시도 회귀 검사.
- pass2-mutation-test.py: lexer, 정확한 함수 선택, 소스 연결, 삽입 범위 검사.

실행기는 tools/revitalize/mutation_audit.py이다. 다른 쓰기 작업이 없는 별도 sibling mutation-audit 작업 트리를 위 HEAD로 만들고, main clone에서 다음을 실행한다:

    source /home/hyunlord/.cargo/env
    CARGO_BUILD_JOBS=10 RUST_TEST_THREADS=8 python3 tools/revitalize/mutation_audit.py run --root ../mutation-audit --output <new-evidence-directory>

HUD snapshot 변경 시에는 새 bytes를 검토하고 strict snapshot input을 의도적으로 repin해야 한다. 이번 감사는 열거된 소스 토큰 계약의 mutation 감도 검사이며 생존·게임플레이·렌더링 전체의 correctness 판정이 아니다.

## 실행기 최종 검증

Python py_compile PASS 및 두 오프라인 회귀 검사 PASS. 증거는 pass2-mutation-final/offline-checks.log와 restoration.json이다. 최종 worktree git status --porcelain 출력은 빈 문자열이며 active-mutant journal은 0개, 검사 대상 소스 13개 모두 inventory의 SHA256과 일치했다. 실행기 SHA256: a35dfbdfdd88e2e5b7ec1ba9924cf30ddfee69c04e2a8e00172b630a77a57726. 실행 완료 후 Cargo 슬롯을 리더에게 반환했다.
