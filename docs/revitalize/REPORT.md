## 구현 완료 보고

**부분 납품 / 전체 게이트 FAIL.** 새 3일 생존 테스트의 실패를 보존했다. P0–P2는 원격 반영, P3–P5는 DGX 로컬 커밋과 별도 인계 bundle로 보존한다. 최종 debug/release 각각1781 PASS·1 FAIL이며 아래 수치는 최종 재실행 로그에서 확인했다.

### 구현 의도

고정 배치와 탈수·출생 반복, 실제 상태와 다른 장식 화면을 시드 기반 Rust 세계와 관측 가능한 생존 규칙, 스냅샷을 읽는 화면으로 바꾸려 했다.

### 구현 내용

- P0: 전용 DGX 클론, Rust 1.93.1·Godot 4.6 arm64, 확장 라이브러리 설정과 기준선 측정.
- P1: 누적 계수, 공통 장면, 정확한 네이티브 비트 지문, JSONL probe·집계·결정론 비교, 캡처 도구, 변경 금지 레거시 고정 테스트.
- P2: 기본값이 레거시인 WorldRules, 결정론적 지형·강·열매·24명 캠프 생성, 지형·건물·시간 등 FFI와 reset 알림.
- P3: 규칙으로 분리한 이동 간격·대기·거주 반경, 식량·주거 조건부 출생, 거처 계획·건설·채집. 허용된 조정 세 차례 후 round2 유지. 생존 목표는 미달이다.
- P4: 결정론적 지형 아틀라스·덤불·그림자·빛, 실제 스냅샷 기반 지형/건물/자원, 선택 주민 경로, 시뮬레이션 시계·집계 HUD, 낮밤, 카메라, 주민 윤곽·보간. 영어·한국어 각각 새 키 22개와 컴파일 결과.
- P5: 전체 debug/release 검사, 두 Clippy, strict·smoke·UI, 레거시4회와 game64시드 fresh probe, 원장·한국어 보고서를 완료했다. 생존 gate 실패로 push하지 않는다.

### 구현 방법

시드 해시와 다항 보간으로 지형을 만들고, 정렬된 자료와 독립 난수 스트림으로 결정론을 유지한다. 드문 열매 후보 부족 시드는 먼 숲과 가까운 평지의 제한적 교환으로 보정하며 지형 종류별 총수는 유지한다. 기본 WorldRules에서는 새 동작과 추가 난수 소비를 하지 않는다. 생성된 캠프는 실제 Hearth를 정착 후보로 사용한다.

출생은 비축 식량·주거 여유·쿨다운을 확인하고 비용을 차감한다. 거처는 계획된 실제 건설지를 주민이 찾아 완성한다. 최종 설정은 이동8틱, 대기확률0.4, 반경8, 거처당4명, 출생360틱/식량3, 덤불8/용량4, 건설자4/진척240이다. 일반 사회적 Seeking의 긴급 갈증 선점은 범위 밖이므로 변경하지 않았다.

GDScript는 Rust 스냅샷을 읽고 실제 id에 맞춰 표시를 갱신한다. reset 알림은 Rust 가변 대여 중 재진입을 피하도록 지연 전달한다. 기존 MultiMesh, 팔레트, STATE_TINTS를 유지한다. 원래 Hunger f32는 레거시 보존을 위해 그대로 두고 지문은 native 비트를 사용한다.

### 기능 설명

플레이어는 시드에 따라 다른 강가 캠프에서 시작하고, 실제 열매·비축 식량·거처·건설 진행·출생 및 사망을 읽을 수 있도록 구현됐다. 시계는 렌더 프레임 대신 시뮬레이션 시간을 표시한다. 선택한 주민에게만 경로가 나타나며 정보창 문구는 번역 키를 사용한다. 이 설명은 구현 내용이며 화면 가독성이나 장기 생존의 합격 판정은 아니다.

### 변경된 파일 목록

브랜치: `codex/revitalize-2026-10`. 기준: `d409e9b941ccc868747e4eb63953eafa6831bff7`.

원격 확인: `git ls-remote origin refs/heads/codex/revitalize-2026-10` 결과는 `c43802600db209b0b32b20fbaecb5f7f8ddf8cf5`다. 마지막 green 원격 경계는 P2다. P3 `a5b906bd0f91d9a9c71c05a0bfb3eb21f219ed38` 및 P4 `47ace74548cd5ad5d7ca36b178bb60d9e231c6c6`는 로컬에만 있다. 생존 테스트 실패 때문에 이후 push는 하지 않는다.

[원격 비교](https://github.com/hyunlord/new-world/compare/lead/main...codex/revitalize-2026-10)는 P2까지만 포함한다. DGX에서 `gh` 실행 파일이 없어 draft PR은 생성하지 않았다. 전체 gate 실패로 P3 이후를 담은 PR도 제공하지 않는다. P3/P4까지 원격에 있는 것으로 소개하면 안 된다.

| Part | 구간 (KST, 2026-10-10) | 소요 |
|---|---|---|
| P0 | 12:07:00–12:18:00 | 약 11분 |
| P1 | 12:18:00–12:29:11 | 11분 11초 |
| P2 | 12:29:11–12:43:42 | 14분 31초 |
| P3 | 12:43:42–12:52:18 | 8분 36초 |
| P4 | 12:52:18–13:07:42 | 15분 24초 |
| P5 | 13:07:42–13:15:24 (검증·보고서 확정) | 약 7.7분; 이후 인계 압축·복사 별도 |

아래는 P0–P4 각 커밋의 `git diff-tree --name-status` 결과다. P5는 이 보고서를 포함하므로 자기 SHA를 본문에 넣지 않는다. 인계 `MANIFEST.txt`와 최종 응답에 확정 SHA를 기록한다. P5 파일은 `REPORT.md`, `DECISIONS.md`, `METRICS.md`, `probe/p5_legacy/summary.{json,md}`, `probe/p5_game/summary.{json,md}`다.

**049e8043a4e2fd49e3cc99b699e4ae2a24b8ab9b — revitalize(p0): arm64 gdextension + baseline record**

```text
A	docs/revitalize/BASELINE.md
A	docs/revitalize/DECISIONS.md
M	sim_bridge.gdextension
```

**856828ec85cd089c042c657b71b6fbe34d4fc768 — revitalize(p1): probe runner, capture tooling, legacy pin**

```text
M	docs/revitalize/BASELINE.md
M	docs/revitalize/DECISIONS.md
A	docs/revitalize/METRICS.md
A	docs/revitalize/TEST_LEDGER.md
A	docs/revitalize/probe/p1_legacy/summary.json
A	docs/revitalize/probe/p1_legacy/summary.md
M	rust/crates/sim-bridge/src/ffi/world_node.rs
M	rust/crates/sim-engine/src/lib.rs
M	rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs
M	rust/crates/sim-systems/src/runtime/survival/mod.rs
M	rust/crates/sim-test/Cargo.toml
A	rust/crates/sim-test/src/bin/worldsim_probe.rs
M	rust/crates/sim-test/src/lib.rs
A	rust/crates/sim-test/src/probe/args.rs
A	rust/crates/sim-test/src/probe/fingerprint.rs
A	rust/crates/sim-test/src/probe/metrics.rs
A	rust/crates/sim-test/src/probe/mod.rs
A	rust/crates/sim-test/src/probe/runner.rs
A	rust/crates/sim-test/src/probe/summary.rs
A	rust/crates/sim-test/src/scenes.rs
A	rust/crates/sim-test/tests/harness_revitalize_legacy_pin.rs
A	rust/crates/sim-test/tests/harness_revitalize_tallies.rs
A	scripts/test/capture_framings.gd
M	scripts/ui/camera_controller.gd
A	tools/revitalize/capture.sh
A	tools/revitalize/contact_sheet.py
A	tools/revitalize/downscale.py
A	tools/revitalize/probe.sh
```

**c43802600db209b0b32b20fbaecb5f7f8ddf8cf5 — revitalize(p2): seeded worldgen in Rust, world rules, terrain/building FFI**

```text
M	docs/revitalize/DECISIONS.md
M	docs/revitalize/METRICS.md
M	docs/revitalize/TEST_LEDGER.md
A	docs/revitalize/probe/p2_game_untuned/summary.json
A	docs/revitalize/probe/p2_game_untuned/summary.md
M	rust/crates/sim-bridge/src/ffi/world_node.rs
A	rust/crates/sim-core/src/components/building_kind.rs
M	rust/crates/sim-core/src/components/mod.rs
M	rust/crates/sim-core/src/lib.rs
A	rust/crates/sim-core/src/rules.rs
M	rust/crates/sim-engine/src/lib.rs
M	rust/crates/sim-systems/src/lib.rs
M	rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs
A	rust/crates/sim-systems/src/worldgen/camp.rs
A	rust/crates/sim-systems/src/worldgen/mod.rs
A	rust/crates/sim-systems/src/worldgen/noise.rs
A	rust/crates/sim-systems/src/worldgen/terrain.rs
M	rust/crates/sim-test/src/probe/args.rs
M	rust/crates/sim-test/src/probe/metrics.rs
M	rust/crates/sim-test/src/probe/runner.rs
M	rust/crates/sim-test/src/scenes.rs
M	rust/crates/sim-test/tests/harness_p12_beta_terrain.rs
M	rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs
M	rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs
M	rust/crates/sim-test/tests/harness_resource_scarcity.rs
A	rust/crates/sim-test/tests/harness_revitalize_worldgen.rs
M	rust/crates/sim-test/tests/harness_show_resource_depletion.rs
M	rust/crates/sim-test/tests/harness_t7_9_b_render_mechanism.rs
M	scripts/ui/world_renderer.gd
```

**a5b906bd0f91d9a9c71c05a0bfb3eb21f219ed38 — revitalize(p3): retain rules-gated loop and bounded tuning evidence**

```text
M	docs/revitalize/DECISIONS.md
M	docs/revitalize/METRICS.md
A	docs/revitalize/probe/p3_game_final/summary.json
A	docs/revitalize/probe/p3_game_final/summary.md
A	docs/revitalize/probe/p3_round0/summary.json
A	docs/revitalize/probe/p3_round0/summary.md
A	docs/revitalize/probe/p3_round1/summary.json
A	docs/revitalize/probe/p3_round1/summary.md
A	docs/revitalize/probe/p3_round2/summary.json
A	docs/revitalize/probe/p3_round2/summary.md
A	docs/revitalize/probe/p3_round3/summary.json
A	docs/revitalize/probe/p3_round3/summary.md
M	rust/crates/sim-core/src/rules.rs
M	rust/crates/sim-systems/src/lib.rs
M	rust/crates/sim-systems/src/runtime/agent/movement.rs
M	rust/crates/sim-systems/src/runtime/construction/construction_system.rs
M	rust/crates/sim-systems/src/runtime/decision/agent_decision.rs
M	rust/crates/sim-systems/src/runtime/settlement/mod.rs
M	rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs
A	rust/crates/sim-systems/src/runtime/settlement/shelter_planning.rs
M	rust/crates/sim-systems/src/worldgen/mod.rs
M	rust/crates/sim-test/src/probe/runner.rs
A	rust/crates/sim-test/tests/harness_revitalize_loop.rs
```

**47ace74548cd5ad5d7ca36b178bb60d9e231c6c6 — revitalize(p4): truthful terrain, objects, HUD, daylight, readable agents**

```text
A	assets/sprites/fx/glow_radial.png
A	assets/sprites/fx/shadow_ellipse.png
A	assets/sprites/resources/berry_bush_0.png
A	assets/sprites/resources/berry_bush_1.png
A	assets/sprites/resources/berry_bush_2.png
A	assets/sprites/resources/berry_bush_3.png
A	assets/sprites/terrain/terrain_v8.png
M	docs/revitalize/DECISIONS.md
A	docs/revitalize/TEST_ASSERTIONS.md
M	docs/revitalize/TEST_LEDGER.md
M	localization/compiled/en.json
M	localization/compiled/ko.json
M	localization/fluent/en/messages.ftl
M	localization/fluent/ko/messages.ftl
M	localization/key_registry.json
M	rust/crates/sim-bridge/src/ffi/world_node.rs
M	rust/crates/sim-test/tests/harness_b1_zoom_default_raise.rs
M	rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs
M	rust/crates/sim-test/tests/harness_h_phase_a_input_camera_color.rs
M	rust/crates/sim-test/tests/harness_p11_alpha_agent_renderer.rs
M	rust/crates/sim-test/tests/harness_p12_alpha_camera_zoom.rs
M	rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs
M	rust/crates/sim-test/tests/harness_p12_beta_terrain.rs
M	rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs
M	rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs
M	rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs
M	rust/crates/sim-test/tests/harness_p13_delta_basic_hud.rs
M	rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs
M	rust/crates/sim-test/tests/harness_p13_gamma_interaction_cues.rs
M	rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs
M	rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs
M	rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs
M	rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs
M	rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs
M	rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs
A	rust/crates/sim-test/tests/harness_revitalize_screen_contract.rs
M	rust/crates/sim-test/tests/harness_s16_alpha0_resource_substrate.rs
M	rust/crates/sim-test/tests/harness_s16_gamma_visual.rs
M	rust/crates/sim-test/tests/harness_show_agent_needs_bars.rs
M	rust/crates/sim-test/tests/harness_show_death_visual.rs
M	rust/crates/sim-test/tests/harness_t7_10_b1_space_toggle.rs
M	rust/crates/sim-test/tests/harness_t7_9_b_render_mechanism.rs
M	scenes/main.tscn
M	scripts/ui/activity_trail_renderer.gd
M	scripts/ui/agent_renderer.gd
A	scripts/ui/building_renderer.gd
M	scripts/ui/camera_controller.gd
A	scripts/ui/daylight_controller.gd
M	scripts/ui/panels/agent_inspector_panel.gd
M	scripts/ui/panels/hud_status_panel.gd
M	scripts/ui/panels/hud_topbar.gd
M	scripts/ui/seek_viz_renderer.gd
A	scripts/ui/terrain_renderer.gd
M	scripts/ui/world_renderer.gd
M	shaders/palette_swap.gdshader
A	tools/art/gen_terrain_atlas.py
```

### 측정 결과

**최종 코드 SHA `47ace74548cd5ad5d7ca36b178bb60d9e231c6c6`에서 재실행한 수치다.** fresh 자료는 `probe/p5_legacy`(4회), `probe/p5_game`(64시드)다. `probe/p3_game_final`은 round2 복사본으로 별도 표시를 유지한다. legacy는 시드를 무시하므로 행동 분포값이 같다. 샘플 간격120틱, 10일, 하루1440틱, T6는 tick0 포함 누적 agent-sample 비율이다.

| 목표 | 항목 | legacy 중앙값 | game 중앙값 | p10 | p90 | 최소 | 최대 | 최종 판정 |
|---|---|---:|---:|---:|---:|---:|---:|---|
| T1 | 탈수 사망 | 233 | 75 | 63.3 | 83.7 | 47 | 90 | FAIL |
| T2 | 기아 사망 | 0 | 0 | 0 | 0 | 0 | 0 | PASS |
| T3 | 생존 인구 최솟값 ≥16 | 44 | 33 | 28.3 | 40 | 24 | 44 | PASS |
| T4 | 인구 중앙값 30–50 | 44 | 33 | 28.3 | 40 | 24 | 44 | PASS |
| T4 | 출생 중앙값 ≥8 | 213 | 83 | 69.3 | 95 | 52 | 109 | PASS |
| T5 | 거처 완공 | 0 | 6 | 4 | 10 | 2 | 14 | PASS |
| T6 | 욕구 초과 대기 비율 | 0.704031 | 0.107077 | 0.0481602 | 0.173049 | 0.0163201 | 0.332048 | FAIL |
| T7 | 집까지 평균 거리 | 17.3864 | 3.70588 | 2.51882 | 5.40966 | 1.96774 | 6.64286 | PASS |
| T8 | 결정론·레거시 고정 | 4회 동일 | 64/64 twins PASS | — | — | — | — | PASS |
| T9 | ms/agent-tick | 0.011784043 | 0.000944597 | 0.000854435 | 0.001027271 | 0.000673780 | 0.001137710 | PASS (0.0802×) |

T1 목표는 중앙값0·최대2, T6 목표는 중앙값0.05 이하다. 둘 다 최종 재실행에서도 실패했다. 생존 인구만 유지된다고 생존 루프가 해결된 것은 아니다.

레거시4회와 game64시드 모두 독립 twin 결정론 PASS. git_sha만 제외한 전체 필드 비교에서 legacy484샘플은 P1과, game7744샘플은 round2와 완전히 일치했다. 고정 테스트 지문 `527233d10db4ed16`, legacy day10 지문 `b9ce9cb344a98de5`는 그대로다. 같은 DGX에서 측정한 T9 중앙값 비율은 0.080159×로 기준2× 이하이나 일반 CPU/렌더 성능 보장은 아니다. legacy 성능 p10/p90/min/max=0.007181054/0.011978082/0.005238295/0.012031291. 전체 day1/3/5/10 분포는 두 fresh summary에 있다.

### 화면 비교

기준선 캡처: 없음. 최종 F1/F2 캡처: 없음. 대조 시트: 생성하지 않음 (`docs/revitalize/shots/final/contact_sheet.png` 없음). Xvfb가 없고 비밀번호 없는 sudo가 거부되어 goal Part0 예외에 따라 모두 생략했다. headless 실행으로 시각적 합격을 대신하지 않는다.

아틀라스 생성기의 실제 채도 출력:

```text
Plain: mean saturation 0.438004
Forest: mean saturation 0.424960
Hill: mean saturation 0.358192
Mountain: mean saturation 0.068777
River: mean saturation 0.529755
```

육지 네 종류는 모두 0.45 이하이며 강은 별도 물 팔레트다. 픽셀 수치의 통과는 실제 화면 가독성 검증이 아니다.

### 테스트 원장 요약

기준선 DGX 실측은 1,729개다. 제공된 x86 1,730개와의 한 개 차이는 원본 실행 로그가 없어 원인 미확정이다. 새 테스트 함수는 53개, 제거된 테스트 함수는 0개이므로 예상 전체는 1,782개다. 이는 정적 목록 계산이며 실제 실행 합계는 Harness 결과에 별도로 적는다.

정본 원장 [TEST_LEDGER.md](TEST_LEDGER.md)와 [TEST_ASSERTIONS.md](TEST_ASSERTIONS.md)는 **230개 단언 재고정 = 매크로 변경224 + 입력 범위 변경6**, 현재 함수 계약113개, **대체 계약 없이 폐기한 단언0개**를 기록한다. 여러 옛 단언을 하나의 현재 함수 계약으로 연결한 경우 이를 명시하며 일대일 동등성을 주장하지 않는다. 역사적 226행은 혼합 단위 기록이므로 정본 단언 수가 아니다. 마지막 소스·줄·SHA256·문서 앵커 대조는 오류0이다.

새 테스트 전체 목록 (최종 debug/release 로그와 개별 대조 완료):

| 파일 | 새 테스트 함수 | 최종 실행 결과 |
|---|---|---|
| `rust/crates/sim-bridge/src/ffi/world_node.rs` | `building_snapshot_sorts_ids_and_defaults_legacy_to_hearth` | debug PASS / release PASS |
| `rust/crates/sim-bridge/src/ffi/world_node.rs` | `game_snapshot_keeps_depleted_bush_and_sorts_without_duplicates` | debug PASS / release PASS |
| `rust/crates/sim-bridge/src/ffi/world_node.rs` | `legacy_snapshot_omits_source_without_live_food` | debug PASS / release PASS |
| `rust/crates/sim-bridge/src/ffi/world_node.rs` | `tallies_collector_preserves_all_four_counters` | debug PASS / release PASS |
| `rust/crates/sim-bridge/src/ffi/world_node.rs` | `terrain_snapshot_preserves_all_codes_and_row_major_elevation` | debug PASS / release PASS |
| `rust/crates/sim-bridge/src/ffi/world_node.rs` | `time_snapshot_handles_disabled_clock` | debug PASS / release PASS |
| `rust/crates/sim-bridge/src/ffi/world_node.rs` | `time_snapshot_uses_completed_ticks_and_resource_clock` | debug PASS / release PASS |
| `rust/crates/sim-bridge/src/ffi/world_node.rs` | `tooling_advance_clamps_negative_and_excessive_requests` | debug PASS / release PASS |
| `rust/crates/sim-bridge/src/ffi/world_node.rs` | `world_info_reads_world_seed_dimensions_and_rules` | debug PASS / release PASS |
| `rust/crates/sim-core/src/components/building_kind.rs` | `stable_codes_and_shelter_work` | debug PASS / release PASS |
| `rust/crates/sim-core/src/rules.rs` | `legacy_has_no_opt_in_behavior` | debug PASS / release PASS |
| `rust/crates/sim-engine/src/lib.rs` | `generated_world_resources_start_legacy_and_empty` | debug PASS / release PASS |
| `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs` | `hearth_first_formation_is_opt_in` | debug PASS / release PASS |
| `rust/crates/sim-systems/src/runtime/settlement/shelter_planning.rs` | `legacy_planning_is_inert` | debug PASS / release PASS |
| `rust/crates/sim-systems/src/worldgen/camp.rs` | `chebyshev_distance_matches_rings` | debug PASS / release PASS |
| `rust/crates/sim-systems/src/worldgen/mod.rs` | `default_configuration_is_explicit` | debug PASS / release PASS |
| `rust/crates/sim-systems/src/worldgen/noise.rs` | `lattice_values_and_unit_interval` | debug PASS / release PASS |
| `rust/crates/sim-systems/src/worldgen/terrain.rs` | `river_and_exact_coverage` | debug PASS / release PASS |
| `rust/crates/sim-test/src/bin/worldsim_probe.rs` | `cli_rejects_unknown_flags` | debug PASS / release PASS |
| `rust/crates/sim-test/src/lib.rs` | `shared_scene_starts_at_tick_zero` | debug PASS / release PASS |
| `rust/crates/sim-test/src/probe/args.rs` | `validates_half_open_range_and_hard_worker_limit` | debug PASS / release PASS |
| `rust/crates/sim-test/src/probe/fingerprint.rs` | `fnv_matches_known_vector` | debug PASS / release PASS |
| `rust/crates/sim-test/src/probe/fingerprint.rs` | `independent_legacy_engines_match` | debug PASS / release PASS |
| `rust/crates/sim-test/src/probe/fingerprint.rs` | `tally_changes_are_observable` | debug PASS / release PASS |
| `rust/crates/sim-test/src/probe/metrics.rs` | `game_boot_shelters_are_initial_not_new_completions` | debug PASS / release PASS |
| `rust/crates/sim-test/src/probe/metrics.rs` | `quantiles_interpolate_and_empty_is_finite` | debug PASS / release PASS |
| `rust/crates/sim-test/src/probe/metrics.rs` | `states_partition_population_and_snapshot_does_not_mutate` | debug PASS / release PASS |
| `rust/crates/sim-test/src/probe/mod.rs` | `schema_remains_version_one` | debug PASS / release PASS |
| `rust/crates/sim-test/src/probe/runner.rs` | `reports_first_observed_divergence_with_seed_and_tick` | debug PASS / release PASS |
| `rust/crates/sim-test/src/probe/summary.rs` | `distribution_covers_extrema_and_interpolated_quantiles` | debug PASS / release PASS |
| `rust/crates/sim-test/src/scenes.rs` | `game_scene_preserves_seed_and_starts_with_twenty_four_agents` | debug PASS / release PASS |
| `rust/crates/sim-test/src/scenes.rs` | `legacy_boot_buildings_are_queued_without_advancing_time` | debug PASS / release PASS |
| `rust/crates/sim-test/tests/harness_h_phase_a_input_camera_color.rs` | `harness_revitalize_camera_frames_settlement_before_optional_tracking` | debug PASS / release PASS |
| `rust/crates/sim-test/tests/harness_h_phase_a_input_camera_color.rs` | `harness_revitalize_camera_reset_preserves_explicit_framing_only` | debug PASS / release PASS |
| `rust/crates/sim-test/tests/harness_p11_alpha_agent_renderer.rs` | `harness_revitalize_agent_reset_discards_old_world_interpolation` | debug PASS / release PASS |
| `rust/crates/sim-test/tests/harness_p11_alpha_agent_renderer.rs` | `harness_revitalize_agent_shadows_follow_interpolated_feet` | debug PASS / release PASS |
| `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs` | `harness_revitalize_inspector_gather_food_is_not_social_target` | debug PASS / release PASS |
| `rust/crates/sim-test/tests/harness_revitalize_legacy_pin.rs` | `harness_population_legacy_scene_pinned` | debug PASS / release PASS |
| `rust/crates/sim-test/tests/harness_revitalize_loop.rs` | `harness_building_assigned_builder_walks_and_clears_missing_site` | debug PASS / release PASS |
| `rust/crates/sim-test/tests/harness_revitalize_loop.rs` | `harness_building_shelters_planned_and_completed` | debug PASS / release PASS |
| `rust/crates/sim-test/tests/harness_revitalize_loop.rs` | `harness_population_births_gated_by_food_and_housing` | debug PASS / release PASS |
| `rust/crates/sim-test/tests/harness_revitalize_loop.rs` | `harness_population_game_world_survives_3_days` | debug FAIL (seed1 탈수11) / release FAIL (seed1 탈수11) |
| `rust/crates/sim-test/tests/harness_revitalize_loop.rs` | `harness_population_movement_only_steps_on_configured_tick_multiples` | debug PASS / release PASS |
| `rust/crates/sim-test/tests/harness_revitalize_loop.rs` | `harness_population_rules_default_is_legacy` | debug PASS / release PASS |
| `rust/crates/sim-test/tests/harness_revitalize_loop.rs` | `harness_social_idle_agents_stay_near_home` | debug PASS / release PASS |
| `rust/crates/sim-test/tests/harness_revitalize_screen_contract.rs` | `harness_building_renderer_reads_snapshot_only` | debug PASS / release PASS |
| `rust/crates/sim-test/tests/harness_revitalize_screen_contract.rs` | `harness_economy_hud_reads_sim_time_and_tallies` | debug PASS / release PASS |
| `rust/crates/sim-test/tests/harness_revitalize_screen_contract.rs` | `harness_territory_terrain_renderer_uses_ffi` | debug PASS / release PASS |
| `rust/crates/sim-test/tests/harness_revitalize_tallies.rs` | `harness_economy_combat_death_uses_cumulative_tally` | debug PASS / release PASS |
| `rust/crates/sim-test/tests/harness_revitalize_tallies.rs` | `harness_economy_tallies_count_births_and_deaths` | debug PASS / release PASS |
| `rust/crates/sim-test/tests/harness_revitalize_worldgen.rs` | `harness_building_snapshot_matches_registry` | debug PASS / release PASS |
| `rust/crates/sim-test/tests/harness_revitalize_worldgen.rs` | `harness_territory_worldgen_deterministic` | debug PASS / release PASS |
| `rust/crates/sim-test/tests/harness_revitalize_worldgen.rs` | `harness_territory_worldgen_invariants` | debug PASS / release PASS |

### 자율 결정 목록

[DECISIONS.md](DECISIONS.md)에 근거를 기록했다. 전용 클론과 요청 도구만 사용했고 기존 서비스·다른 체크아웃·OMO/OMX 상태를 건드리지 않았다. Xvfb 예외를 적용해 캡처를 생략했다. 소수 시드의 열매 후보 부족은 지형 수 보존 교환으로 보정했다. 기존 f32 Hunger와 기본 규칙 경로는 보존했다. 사회적 Seeking의 긴급 욕구 전환은 범위 밖이므로 수정하지 않고 조정 상한에서 멈췄다. 목표 우선순위상 비멸종 구성 중 탈수가 더 적은 round2를 유지했다. 매 push green 규칙 때문에 P3 이후 원격 반영을 보류했다. T1–T8 미충족으로 야간 수면 확장은 생략했다.

### 확인된 제한사항

3일 생존 테스트의 탈수0 조건이 실패한다. 초기 추적에서 높은 갈증 상태의 일반 Seeking Agent가 물 위에서도 관계 대상을 계속 추적했다. 이는 관측 근거이며 조정만으로 해결되지 않았다. T1·T6 미달이며 T9는 같은 호스트의 probe 측정 범위에서 통과했다. 시각적 가독성, 실제 클릭/키보드 경험, 10분 4배속 프레임 체감, 영어/한국어 실제 화면은 미검증이다. headless en/ko 문자열·HUD·reset 등은 실제 main scene에서 검사했지만 화면 외관을 증명하지 않는다. 여러 정착지가 한 비축소를 공유할 경우 라벨 겹침 가능성도 시각 확인이 필요하다. 새 소셜 긴급 욕구 선점은 별도 범위 승인된 후속 작업 대상이다.

### Harness 결과

최종 debug/release 모두 `--no-fail-fast -- --test-threads=8`로 실행했다. 각각 **1781 통과, 1 실패, 0 ignored, 113 result groups**다. 총1782 = 기준선1729 + 신규53 − 제거0으로 일치한다. 실패한 함수 내부 seed1 단언에서 중단되어 해당 함수의 seed2/3 이후 단언은 미실행됐지만 다른 테스트 바이너리는 모두 계속 실행됐다. 로그: `.harness/evidence/revitalize/logs/p5/tests.log`, `tests-release.log`, `test-counts.json`. 전체 gate는 FAIL이다.

| 검사 | 현재 확인 가능한 근거 | 최종 결과 |
|---|---|---|
| 기준선 debug/release | 각각1729 PASS, 실패0 | 기준선 확정 |
| P1 debug | 1748 PASS | 해당 경계 확정 |
| P2 debug | 1764 PASS | 해당 경계 확정 |
| 3일 생존 | debug/release seed1 탈수11, 기대0; 단언 보존 | FAIL |
| 신규53개 및 레거시 pin | 신규52 PASS / 생존1 FAIL; 고정 pin PASS | 위 개별 결과 참조 |
| clippy 두 명령 | 최종 `logs/p5/status.tsv`와 각 로그 | PASS, 두 명령 exit0 |
| GDScript strict | 17파일·14 FFI call 일치 | PASS, exit0 |
| Godot 900프레임 smoke / UI reset | 비오디오 오류0; P4 SHA에서 UI322 PASS / GPU96 NOT_OBSERVABLE | PASS (시각 합격 아님) |
| 금지 수학·production unwrap | sim-core/systems/engine 스캔0건; 변경 production 27파일의 test 모듈 이전 주석 제외 unwrap0건 | PASS |
| 캡처 | Xvfb 예외 | SKIPPED, 시각 합격 아님 |

### 정직한 판단

F2 2일차 캡처를 열어보지 못했으므로 10초 안에 마을·행동·부족 자원이 읽힌다고 판정할 수 없다. 스냅샷을 읽는 구현과 headless 계약 검증은 화면 가독성 증거가 아니다. 시뮬레이션은 아직 탈수 문제를 해결하지 못했으며 전체 완료 또는 green으로 보고할 수 없다.

**인게임 확인사항 — 모두 실제 화면 확인 대기. 아래는 합격 기록이 아닌 원문 체크리스트다.**

1. 게임을 켜면 첫 화면이 강가의 작은 마을을 화면 중앙에 보여준다. 모닥불 1개, 거처 3채, 비축소 1개가 보이고 주민 24명이 그 주변에 모여 있다.
2. 지형이 초원, 숲, 언덕, 바위, 강으로 구분되어 보이고, 타일 단위로 무작위 체크무늬처럼 보이지 않는다. 강가 경계선이 보인다.
3. 화면에 보이는 모든 물체가 실제 시뮬레이션 대상이다. 장식용 자원 스프라이트와 마을 장식물이 없다. 열매 덤불은 채집될수록 열매 수가 줄고, 시간이 지나면 다시 열린다.
4. 주민이 매 틱 떨리듯 움직이지 않고, 한 칸씩 부드럽게 걷는다. 할 일이 없는 주민은 마을 반경 안에서 쉬거나 천천히 거닌다.
5. 목마른 주민이 강가로 가서 물을 마시고 돌아온다. 화면 전체를 가로지르는 긴 대각선이 보이지 않는다. 경로선은 선택한 주민 한 명에게만 표시된다.
6. 인구가 거처 수용량에 닿으면 마을 근처에 새 거처 건설지가 생기고, 주민들이 그곳으로 걸어가 건설하며, 진행 막대가 차오르고 완성되면 거처가 나타난다.
7. 비축 식량이 충분하고 거처에 여유가 있을 때만 출생이 일어나며, 출생 때 비축 식량이 줄어든다.
8. 상단 정보줄의 인구, 정착지, 거처, 건설 중 수치가 실제 상황과 일치하고, 시계가 "N일차 HH:MM" 형식으로 시뮬레이션 시간과 맞게 흐른다. 21시가 되면 화면이 어두워지고 모닥불 주변이 밝게 빛나며, 아침 7시에 다시 밝아진다.
9. 비축소 위의 식량 표시는 확대·축소와 관계없이 작고 일정한 크기로 보이며, 멀리 축소하면 숨겨진다.
10. 주민에게 윤곽선과 발밑 그림자가 있어서 확대 4배에서 지형과 확실히 구분된다.
11. 스페이스바 영향력 오버레이 전환, 주민 클릭 정보창이 이전처럼 동작한다.
12. 콘솔에 SCRIPT ERROR가 없고, 10분 동안 4배속으로 돌려도 프레임이 눈에 띄게 떨어지지 않는다.
13. 영어로 언어를 바꿨을 때 새로 추가된 모든 문구가 영어로, 한국어일 때 한국어로 나온다.
