# Art Pass 7 변경 목록

감사 범위: `e1e894ef217350be8f1c456f2665836199a7358f`부터 `28f58e56a2289bde129946b7f630192cd94496b2`까지 첫 부모 이력의 14개 커밋. Part 8·9 후속 커밋은 이 고정 범위에 포함하지 않는다. 파일 목록은 각 커밋의 첫 부모 대비 `git diff --name-status`에서 그대로 추출했다. 상태 `A`는 추가, `M`은 수정이다.

## 범위·호환성 감사

- 승인된 시뮬레이션 병합 이후 `e1e894ef217350be8f1c456f2665836199a7358f` → `28f58e56a2289bde129946b7f630192cd94496b2`의 순변경은 **217개 경로(추가 187, 수정 30)**이며 GOAL §2 허용 경로 위반은 **0개**다.
- Rust 변경은 `rust/crates/sim-bridge/src/ffi/world_node.rs`의 **210줄 추가·삭제 0줄**뿐이다. `get_society_snapshot(&self)`와 읽기 전용 수집기, 스냅샷 구조체·FFI 변환기, `#[cfg(test)]` 조인·정렬·필터 검증을 추가했다. 기존 시뮬레이션 로직은 변경하지 않았다.
- `scenes/main.tscn`은 병합 직후와 바이트가 같고 `load_steps=21`을 유지한다. SHA-256: `ac65b73a98bcd8f6dd65443bbb0ab0960a9f675a68d74a2f9b6d4fffdf86b52b`.
- `0f634fec` → `28f58e56`에서 에이전트 렌더러·운반 레이어·캡처 메타데이터·보정 장면은 동일하다. 렌더링 경로의 변경은 `scripts/ui/settlement_labels.gd`뿐이다. 이전 A11 보정 자료의 해당 소스 동등성을 확인했으며 전체 게임 화면의 동등성을 뜻하지 않는다.
- 이름표 12px는 GOAL의 9–12px 범위 안이다. 건물 충돌 회피 계산이 추가되었으므로 A30은 최신 헤드에서 측정한 결과로 판단한다. HUD 경계 때문에 위치를 유지하는 회피 예외는 모든 카메라 배치의 무겹침을 보장하지 않으며, 현재 캡처의 위반을 확인한 것은 아니다.
- 이 감사는 해당 저장소의 커밋을 대상으로 한다. 다른 체크아웃의 쓰기 이력이나 미커밋 변경을 무변경으로 인증하지 않는다.

## 커밋 색인

| Part | 전체 커밋 해시 | 제목 |
|---|---|---|
| 0 | `e1e894ef217350be8f1c456f2665836199a7358f` | art-p7(p0): bring in simulation passes 9 to 11 |
| 0 | `7efbf61b045393c67d54eb463e0f760c073b9fd0` | art-p7(p0): preserve the execution contract and merge provenance |
| 0 | `07e148dfe8d1960d16076209b40e91ed3bf0d926` | art-p7(p0): make scenario captures preserve seed and settlement identity |
| 0 | `164e93f7a99c4424826bbc67dcafe2e56757926c` | art-p7(p0): bind v17 measurements and untouched-screen baselines |
| 1 | `6b013bfd26fcbeb9c97ba78b973020f33bce1a74` | art-p7(p1): prevent abandoned hearths from borrowing a neighbouring fire |
| 2 | `cff5691b820d8fbf7e8537fd750e9ba2cdfd6aca` | art-p7(p2): count and explain every death without losing simultaneous victims |
| 3 | `deef001bc7420efd8e45e03de32350e4d9138f0f` | art-p7(p3): expose village lineage without changing simulation state |
| 4 | `b2b28db8c9581d59dfa42d93113eec7b0994f338` | art-p7(p4): make village identity and intervention targets visible |
| 4 | `c84981b22dc8ba5860a04f0ba9674135e3c468bc` | art-p7(p4): hide the single-village counter without hiding the clock |
| 5 | `c4a253b2d9f8b404f10c175a596c09e6ce1cb5a9` | art-p7(p5): show why a group leaves and when its village is founded |
| 6 | `509262294e56957b25251a5dcd6efbd27b0a316d` | art-p7(p6): give doorway breath and falling wood one clearer visual attempt |
| 7 | `0f634fec92381315538d865c53ff44b88998c157` | art-p7(p7): bind captures to measured society and death evidence |
| 7 | `eb690d377ec2fe3fc69c06ede4843c0c9fa93005` | art-p7(p7): measure exposure closeup cycles without missing evidence |
| 7 | `28f58e56a2289bde129946b7f630192cd94496b2` | art-p7(p7): make village lineage readable after independent review |

## 커밋별 정확한 변경 경로

### Part 0 — `e1e894ef217350be8f1c456f2665836199a7358f`

art-p7(p0): bring in simulation passes 9 to 11

승인된 시뮬레이션 Pass 9–11 병합은 아래 원본과 범위로 별도 기록한다. 대량의 시뮬레이션 파일을 아트 수정 목록으로 혼합하지 않는다.

- 아트 부모: `cd998d3807358521ead77b0a7314e93fd8283878`.
- 병합한 시뮬레이션 부모: `31559232e0a24af12d29f490d440d1c5ee5f3661`.
- 첫 부모 대비 병합 변경: 513개 경로. 상위 경로별 수: `docs/` 424, `rust/` 62, `tools/` 27.
- 이 병합 이후의 아트 작업만 위 허용 경로 감사의 대상으로 삼았다.

### Part 0 — `7efbf61b045393c67d54eb463e0f760c073b9fd0`

art-p7(p0): preserve the execution contract and merge provenance

```text
A	docs/art/production7/DECISIONS.md
A	docs/art/production7/GOAL.md
A	docs/art/production7/TASKS.md
```

### Part 0 — `07e148dfe8d1960d16076209b40e91ed3bf0d926`

art-p7(p0): make scenario captures preserve seed and settlement identity

```text
M	docs/art/production7/DECISIONS.md
A	docs/art/production7/DESIGN.md
A	scripts/core/settlement_selection.gd
M	scripts/test/art/capture_plans.gd
M	scripts/test/capture_framings.gd
M	tools/art_pipeline/capture_art.sh
```

### Part 0 — `164e93f7a99c4424826bbc67dcafe2e56757926c`

art-p7(p0): bind v17 measurements and untouched-screen baselines

```text
A	docs/art/production7/BASELINE.md
M	docs/art/production7/DECISIONS.md
A	docs/art/production7/MEASURED_DAYS.md
M	docs/art/production7/TASKS.md
A	docs/art/production7/baseline/live/a_calib_z4.entities.json.gz
A	docs/art/production7/baseline/live/a_calib_z4.jpg
A	docs/art/production7/baseline/live/a_calib_z6.entities.json.gz
A	docs/art/production7/baseline/live/a_calib_z6.jpg
A	docs/art/production7/baseline/live/a_d2_1200_F2.entities.json.gz
A	docs/art/production7/baseline/live/a_d2_1200_F2.png
A	docs/art/production7/baseline/live/a_lineup_z2.entities.json.gz
A	docs/art/production7/baseline/live/a_lineup_z2.jpg
A	docs/art/production7/baseline/live/a_lineup_z4.entities.json.gz
A	docs/art/production7/baseline/live/a_lineup_z4.jpg
A	docs/art/production7/baseline/live/forest_evidence.json.gz
A	docs/art/production7/baseline/live/manifest.json
A	docs/art/production7/baseline/live/manifest_art.json
A	docs/art/production7/baseline/live_bound/a_calib_z4.entities.json.gz
A	docs/art/production7/baseline/live_bound/a_calib_z4.jpg
A	docs/art/production7/baseline/live_bound/a_calib_z6.entities.json.gz
A	docs/art/production7/baseline/live_bound/a_calib_z6.jpg
A	docs/art/production7/baseline/live_bound/a_d2_1200_F2.entities.json.gz
A	docs/art/production7/baseline/live_bound/a_d2_1200_F2.png
A	docs/art/production7/baseline/live_bound/a_lineup_z2.entities.json.gz
A	docs/art/production7/baseline/live_bound/a_lineup_z2.jpg
A	docs/art/production7/baseline/live_bound/a_lineup_z4.entities.json.gz
A	docs/art/production7/baseline/live_bound/a_lineup_z4.jpg
A	docs/art/production7/baseline/live_bound/forest_evidence.json.gz
A	docs/art/production7/baseline/live_bound/manifest.json
A	docs/art/production7/baseline/live_bound/manifest_art.json
A	docs/art/production7/baseline/society/manifest_art_society.json
A	docs/art/production7/baseline/society/soc_d29_0909_FV_all.entities.json.gz
A	docs/art/production7/baseline/society/soc_d29_0909_FV_all.jpg
A	docs/art/production7/gates/p0/bridge.log
A	docs/art/production7/gates/p0/cargo_test.log
A	docs/art/production7/gates/p0/clippy.log
A	docs/art/production7/gates/p0/python.log
A	docs/art/production7/gates/p0/results.tsv
A	docs/art/production7/gates/p0/smoke.log
A	docs/art/production7/gates/p0/smoke_errors.log
A	docs/art/production7/gates/p0/strict.log
A	docs/art/production7/gates/p0/test_agent_activity.log
A	docs/art/production7/gates/p0/test_agent_click_drawn.log
A	docs/art/production7/gates/p0/test_agent_drawn_positions.log
A	docs/art/production7/gates/p0/test_agent_native_size.log
A	docs/art/production7/gates/p0/test_art_hash.log
A	docs/art/production7/gates/p0/test_camp_seats.log
A	docs/art/production7/gates/p0/test_canopy_fade.log
A	docs/art/production7/gates/p0/test_capture_cold_trace_p6.log
A	docs/art/production7/gates/p0/test_capture_metadata.log
A	docs/art/production7/gates/p0/test_capture_plans_p6.log
A	docs/art/production7/gates/p0/test_carry_load_layer.log
A	docs/art/production7/gates/p0/test_cold_hearth.log
A	docs/art/production7/gates/p0/test_cold_presentation.log
A	docs/art/production7/gates/p0/test_declump.log
A	docs/art/production7/gates/p0/test_effect_lifecycle.log
A	docs/art/production7/gates/p0/test_elder_variant.log
A	docs/art/production7/gates/p0/test_god_hand.log
A	docs/art/production7/gates/p0/test_hud_old_age.log
A	docs/art/production7/gates/p0/test_hud_season_label.log
A	docs/art/production7/gates/p0/test_hud_wood.log
A	docs/art/production7/gates/p0/test_idle_camp_pose.log
A	docs/art/production7/gates/p0/test_idle_facing.log
A	docs/art/production7/gates/p0/test_inspector_age.log
A	docs/art/production7/gates/p0/test_inspector_cold.log
A	docs/art/production7/gates/p0/test_inspector_wood.log
A	docs/art/production7/gates/p0/test_interventions_real_p6.log
A	docs/art/production7/gates/p0/test_meal_pose.log
A	docs/art/production7/gates/p0/test_morning_chill.log
A	docs/art/production7/gates/p0/test_nature_clearance.log
A	docs/art/production7/gates/p0/test_nature_density.log
A	docs/art/production7/gates/p0/test_need_warning.log
A	docs/art/production7/gates/p0/test_season_tint.log
A	docs/art/production7/gates/p0/test_shelter_cold.log
A	docs/art/production7/gates/p0/test_structure_states.log
A	docs/art/production7/gates/p0/test_terrain_layers.log
A	docs/art/production7/gates/p0/test_variant_codes.log
A	docs/art/production7/gates/p0/test_ysort_props.log
A	docs/art/production7/gates/p0/test_zoom_ladder.log
A	docs/art/production7/measurements/headless_20261010.json
A	docs/art/production7/measurements/probe_20261010.json
A	docs/art/production7/measurements/probe_20261013.json
A	docs/art/production7/measurements/probe_calendar_20261010.json
A	docs/art/production7/measurements/probe_calendar_20261013.json
A	scripts/test/art/measure_days_p7.gd
```

### Part 1 — `6b013bfd26fcbeb9c97ba78b973020f33bce1a74`

art-p7(p1): prevent abandoned hearths from borrowing a neighbouring fire

```text
M	docs/art/production7/MEASURED_DAYS.md
A	docs/art/production7/MULTI_SETTLEMENT_UI_AUDIT.md
M	docs/art/production7/TASKS.md
A	docs/art/production7/gates/p1/cold_hearth.log
A	docs/art/production7/gates/p1/every_hearth.log
A	docs/art/production7/gates/p1/strict.log
A	docs/art/production7/gates/p1/structure_states.log
A	docs/art/production7/measurements/headless_20261013.json
M	scripts/test/art/test_cold_hearth.gd
A	scripts/test/art/test_every_hearth.gd
M	scripts/test/art/test_structure_states.gd
M	scripts/ui/building_renderer.gd
M	scripts/ui/wood_visuals.gd
M	scripts/ui/world_effects.gd
```

### Part 2 — `cff5691b820d8fbf7e8537fd750e9ba2cdfd6aca`

art-p7(p2): count and explain every death without losing simultaneous victims

```text
M	docs/art/production7/DECISIONS.md
M	docs/art/production7/MULTI_SETTLEMENT_UI_AUDIT.md
M	docs/art/production7/TASKS.md
A	docs/art/production7/gates/p2/check_only.log
A	docs/art/production7/gates/p2/death_exposure_green.log
A	docs/art/production7/gates/p2/death_exposure_red.log
A	docs/art/production7/gates/p2/hud_old_age_green.log
A	docs/art/production7/gates/p2/localization.log
M	localization/compiled/en.json
M	localization/compiled/ko.json
M	localization/fluent/en/messages.ftl
M	localization/fluent/ko/messages.ftl
M	localization/key_registry.json
A	scripts/test/art/test_death_exposure.gd
M	scripts/test/art/test_hud_old_age.gd
M	scripts/ui/death_viz_renderer.gd
M	scripts/ui/panels/hud_status_panel.gd
M	scripts/ui/panels/hud_topbar.gd
```

### Part 3 — `deef001bc7420efd8e45e03de32350e4d9138f0f`

art-p7(p3): expose village lineage without changing simulation state

```text
M	docs/art/production7/DECISIONS.md
M	docs/art/production7/TASKS.md
A	docs/art/production7/gates/p3/NOTES.md
A	docs/art/production7/gates/p3/bridge.exit
A	docs/art/production7/gates/p3/bridge.log
A	docs/art/production7/gates/p3/cargo-test.exit
A	docs/art/production7/gates/p3/cargo-test.log
A	docs/art/production7/gates/p3/clippy-all.exit
A	docs/art/production7/gates/p3/clippy-all.log
A	docs/art/production7/gates/p3/clippy.exit
A	docs/art/production7/gates/p3/clippy.log
A	docs/art/production7/gates/p3/society-natural.exit
A	docs/art/production7/gates/p3/society-natural.log
A	docs/art/production7/gates/p3/society-red.exit
A	docs/art/production7/gates/p3/society-red.log
A	docs/art/production7/gates/p3/society-short.exit
A	docs/art/production7/gates/p3/society-short.log
M	rust/crates/sim-bridge/src/ffi/world_node.rs
A	scripts/test/art/test_society_snapshot_p7.gd
```

### Part 4 — `b2b28db8c9581d59dfa42d93113eec7b0994f338`

art-p7(p4): make village identity and intervention targets visible

```text
M	docs/art/production7/DECISIONS.md
M	docs/art/production7/TASKS.md
A	docs/art/production7/gates/p4/abandoned.log
A	docs/art/production7/gates/p4/every_hearth.log
A	docs/art/production7/gates/p4/god_hand.log
A	docs/art/production7/gates/p4/hud_wood.log
A	docs/art/production7/gates/p4/import.log
A	docs/art/production7/gates/p4/labels.log
A	docs/art/production7/gates/p4/localization_compile.json
A	docs/art/production7/gates/p4/localization_compile.log
A	docs/art/production7/gates/p4/localization_duplicates_before.json
A	docs/art/production7/gates/p4/localization_verify.json
A	docs/art/production7/gates/p4/localization_verify.log
A	docs/art/production7/gates/p4/select.log
A	docs/art/production7/gates/p4/smoke.log
A	docs/art/production7/gates/p4/static-contracts.exit
A	docs/art/production7/gates/p4/static-contracts.log
A	docs/art/production7/gates/p4/supplementary-rust.json
A	docs/art/production7/review/p4/manifest_art_society.json
A	docs/art/production7/review/p4/soc_d29_0909_FV_all.entities.json.gz
A	docs/art/production7/review/p4/soc_d29_0909_FV_all.jpg
A	docs/art/production7/review/p4/soc_d29_1200_FV_all_selected_daughter.entities.json.gz
A	docs/art/production7/review/p4/soc_d29_1200_FV_all_selected_daughter.jpg
A	docs/art/production7/review/p4/soc_d29_1200_FV_all_selected_orig.entities.json.gz
A	docs/art/production7/review/p4/soc_d29_1200_FV_all_selected_orig.jpg
A	docs/art/production7/review/p4/verdict.json
M	localization/compiled/en.json
M	localization/compiled/ko.json
M	localization/fluent/en/messages.ftl
M	localization/fluent/ko/messages.ftl
M	localization/key_registry.json
M	scripts/core/settlement_selection.gd
M	scripts/rendering/art/god_hand_effects.gd
A	scripts/test/art/test_abandoned_village.gd
A	scripts/test/art/test_settlement_labels.gd
A	scripts/test/art/test_settlement_select.gd
M	scripts/ui/agent_renderer.gd
M	scripts/ui/panels/agent_inspector_panel.gd
M	scripts/ui/panels/god_hand_panel.gd
M	scripts/ui/panels/hud_topbar.gd
A	scripts/ui/settlement_labels.gd
M	scripts/ui/settlement_overview_renderer.gd
A	shaders/abandoned_building.gdshader
A	shaders/settlement_outline.gdshader
```

### Part 4 — `c84981b22dc8ba5860a04f0ba9674135e3c468bc`

art-p7(p4): hide the single-village counter without hiding the clock

```text
A	docs/art/production7/gates/p4/labels_pair_hidden.log
A	docs/art/production7/gates/p4/labels_pair_isolated.log
A	docs/art/production7/review/p4_pair/manifest_art_society.json
A	docs/art/production7/review/p4_pair/soc_d2_1200_F2_orig.entities.json.gz
A	docs/art/production7/review/p4_pair/soc_d2_1200_F2_orig.jpg
A	docs/art/production7/review/p4_pair/verdict.json
A	docs/art/production7/review/p4_pair_fixed/manifest_art_society.json
A	docs/art/production7/review/p4_pair_fixed/soc_d2_1200_F2_orig.entities.json.gz
A	docs/art/production7/review/p4_pair_fixed/soc_d2_1200_F2_orig.jpg
A	docs/art/production7/review/p4_pair_fixed/verdict.json
M	scripts/test/art/test_settlement_labels.gd
M	scripts/ui/panels/hud_topbar.gd
```

### Part 5 — `c4a253b2d9f8b404f10c175a596c09e6ce1cb5a9`

art-p7(p5): show why a group leaves and when its village is founded

```text
M	docs/art/production7/DECISIONS.md
M	docs/art/production7/TASKS.md
A	docs/art/production7/gates/p5/check_only.log
A	docs/art/production7/gates/p5/founding_group_attempt1.log
A	docs/art/production7/gates/p5/founding_group_green.log
A	docs/art/production7/gates/p5/static-contracts.exit
A	docs/art/production7/gates/p5/static-contracts.log
A	docs/art/production7/gates/p5/test_carry_load_layer.log
A	docs/art/production7/gates/p5/test_death_exposure.log
A	docs/art/production7/gates/p5/test_hud_old_age.log
A	docs/art/production7/gates/p5/test_settlement_labels.log
A	docs/art/production7/gates/p5/test_settlement_select.log
A	docs/art/production7/review/p5/manifest_art_society.json
A	docs/art/production7/review/p5/soc_d29_0630_FD_leader.entities.json.gz
A	docs/art/production7/review/p5/soc_d29_0630_FD_leader.jpg
A	docs/art/production7/review/p5/soc_d29_0800_FD_leader.entities.json.gz
A	docs/art/production7/review/p5/soc_d29_0800_FD_leader.jpg
A	docs/art/production7/review/p5/soc_d29_0819_FS_daughter.entities.json.gz
A	docs/art/production7/review/p5/soc_d29_0819_FS_daughter.jpg
A	docs/art/production7/review/p5/verdict.json
A	scripts/test/art/test_founding_group.gd
M	scripts/ui/agent_renderer.gd
M	scripts/ui/carry_load_layer.gd
A	scripts/ui/founding_group_renderer.gd
M	scripts/ui/panels/hud_status_panel.gd
```

### Part 6 — `509262294e56957b25251a5dcd6efbd27b0a316d`

art-p7(p6): give doorway breath and falling wood one clearer visual attempt

```text
M	docs/art/production7/DECISIONS.md
M	docs/art/production7/TASKS.md
A	docs/art/production7/gates/p6/god_hand.log
A	docs/art/production7/gates/p6/shelter_cold.log
M	scripts/rendering/art/god_hand_effects.gd
M	scripts/test/art/test_god_hand.gd
M	scripts/test/art/test_shelter_cold.gd
M	scripts/ui/shelter_cold.gd
```

### Part 7 — `0f634fec92381315538d865c53ff44b88998c157`

art-p7(p7): bind captures to measured society and death evidence

```text
M	docs/art/TEST_LEDGER.md
M	docs/art/production7/MULTI_SETTLEMENT_UI_AUDIT.md
M	docs/art/production7/TASKS.md
A	docs/art/production7/gates/p7/death-calibration-parse-attempt1.log
A	docs/art/production7/gates/p7/death-calibration-parse.log
A	docs/art/production7/gates/p7/death-calibration.md
A	scripts/test/art/capture_deaths_p7.gd
M	scripts/test/art/capture_metadata.gd
M	scripts/test/art/capture_plans.gd
A	scripts/test/art/capture_society_trace.gd
M	scripts/test/art/test_capture_metadata.gd
A	scripts/test/art/test_capture_society_trace.gd
M	scripts/test/capture_framings.gd
M	tools/art_pipeline/capture_art.sh
```

### Part 7 — `eb690d377ec2fe3fc69c06ede4843c0c9fa93005`

art-p7(p7): measure exposure closeup cycles without missing evidence

```text
M	docs/art/production7/DECISIONS.md
A	docs/art/production7/gates/p7/death-calibration-fixture.log
M	docs/art/production7/gates/p7/death-calibration.md
M	scripts/test/art/capture_deaths_p7.gd
M	scripts/test/capture_framings.gd
```

### Part 7 — `28f58e56a2289bde129946b7f630192cd94496b2`

art-p7(p7): make village lineage readable after independent review

```text
M	docs/art/TEST_LEDGER.md
M	docs/art/production7/DECISIONS.md
A	docs/art/production7/gates/p7/label-review-fix.log
M	scripts/test/art/test_settlement_labels.gd
M	scripts/ui/settlement_labels.gd
```

## Part 7·8 증거 공개 후속 커밋

### `369f2200406233e6c32454a4389b5296f4ea8276`

art-p7(p7): preserve final screen evidence and rejected capture cases

```text
M	docs/art/production7/DECISIONS.md
M	docs/art/production7/TASKS.md
A	docs/art/production7/calibration/a_calib_z4.entities.json.gz
A	docs/art/production7/calibration/a_calib_z4.jpg
A	docs/art/production7/calibration/a_calib_z6.entities.json.gz
A	docs/art/production7/calibration/a_calib_z6.jpg
A	docs/art/production7/calibration/a_lineup_z2.entities.json.gz
A	docs/art/production7/calibration/a_lineup_z2.jpg
A	docs/art/production7/calibration/a_lineup_z4.entities.json.gz
A	docs/art/production7/calibration/a_lineup_z4.jpg
A	docs/art/production7/calibration/calibration_deaths_p7.json.gz
A	docs/art/production7/calibration/calibration_deaths_p7.png
A	docs/art/production7/calibration/manifest.json
A	docs/art/production7/calibration/manifest_calibration.json
A	docs/art/production7/compact_previews.json
A	docs/art/production7/final_capture_integrity.json
A	docs/art/production7/gates/p7/death-calibration-window-attempt1.log
A	docs/art/production7/gates/p7/death-calibration-window-attempt2.log
A	docs/art/production7/gates/p7/final-art.log
A	docs/art/production7/gates/p7/final-art_god.log
A	docs/art/production7/gates/p7/final-art_long.log
A	docs/art/production7/gates/p7/final-art_society.log
A	docs/art/production7/gates/p7/final-calibration.log
A	docs/art/production7/gates/p7/final_capture_status.json
A	docs/art/production7/gates/p7/rejected-plus4h-FD.log
A	docs/art/production7/shots/a_calib_z4.entities.json.gz
A	docs/art/production7/shots/a_calib_z4.jpg
A	docs/art/production7/shots/a_calib_z6.entities.json.gz
A	docs/art/production7/shots/a_calib_z6.jpg
A	docs/art/production7/shots/a_d1_0830_F2.entities.json.gz
A	docs/art/production7/shots/a_d1_0830_F2.jpg
A	docs/art/production7/shots/a_d2_1200_F1.entities.json.gz
A	docs/art/production7/shots/a_d2_1200_F1.png
A	docs/art/production7/shots/a_d2_1200_F2.entities.json.gz
A	docs/art/production7/shots/a_d2_1200_F2.png
A	docs/art/production7/shots/a_d2_1200_F2A.entities.json.gz
A	docs/art/production7/shots/a_d2_1200_F2A.jpg
A	docs/art/production7/shots/a_d2_1200_F2Z.entities.json.gz
A	docs/art/production7/shots/a_d2_1200_F2Z.jpg
A	docs/art/production7/shots/a_d2_1200_F3.entities.json.gz
A	docs/art/production7/shots/a_d2_1200_F3.jpg
A	docs/art/production7/shots/a_d2_1200_F4.entities.json.gz
A	docs/art/production7/shots/a_d2_1200_F4.jpg
A	docs/art/production7/shots/a_d2_1930_F2.entities.json.gz
A	docs/art/production7/shots/a_d2_1930_F2.jpg
A	docs/art/production7/shots/a_d2_2300_F2N.entities.json.gz
A	docs/art/production7/shots/a_d2_2300_F2N.png
A	docs/art/production7/shots/a_d6_1200_F2.entities.json.gz
A	docs/art/production7/shots/a_d6_1200_F2.jpg
A	docs/art/production7/shots/a_lineup_z2.entities.json.gz
A	docs/art/production7/shots/a_lineup_z2.jpg
A	docs/art/production7/shots/a_lineup_z4.entities.json.gz
A	docs/art/production7/shots/a_lineup_z4.jpg
A	docs/art/production7/shots/evening.json.gz
A	docs/art/production7/shots/forest_evidence.json.gz
A	docs/art/production7/shots/frame_times.json.gz
A	docs/art/production7/shots/god_d10_1200_F2_spoil.entities.json.gz
A	docs/art/production7/shots/god_d10_1200_F2_spoil.jpg
A	docs/art/production7/shots/god_d17_0200_F2N_cold_snap.entities.json.gz
A	docs/art/production7/shots/god_d17_0200_F2N_cold_snap.jpg
A	docs/art/production7/shots/god_d23_1200_F2_gift.entities.json.gz
A	docs/art/production7/shots/god_d23_1200_F2_gift.jpg
A	docs/art/production7/shots/god_d23_1205_F2_windfall_0200ms.entities.json.gz
A	docs/art/production7/shots/god_d23_1205_F2_windfall_0200ms.jpg
A	docs/art/production7/shots/god_d23_1205_F2_windfall_1000ms.entities.json.gz
A	docs/art/production7/shots/god_d23_1205_F2_windfall_1000ms.jpg
A	docs/art/production7/shots/god_d24_1830_F2_rain.entities.json.gz
A	docs/art/production7/shots/god_d24_1830_F2_rain.jpg
A	docs/art/production7/shots/god_d25_0200_F2N_rain_night.entities.json.gz
A	docs/art/production7/shots/god_d25_0200_F2N_rain_night.jpg
A	docs/art/production7/shots/god_d25_0200_F2_panel_log.entities.json.gz
A	docs/art/production7/shots/god_d25_0200_F2_panel_log.jpg
A	docs/art/production7/shots/god_d50_1700_F2_soc_rain_daughter.entities.json.gz
A	docs/art/production7/shots/god_d50_1700_F2_soc_rain_daughter.jpg
A	docs/art/production7/shots/god_d51_0200_F2N_soc_original_night_orig.entities.json.gz
A	docs/art/production7/shots/god_d51_0200_F2N_soc_original_night_orig.jpg
A	docs/art/production7/shots/god_d51_0200_F2N_soc_rain_night_daughter.entities.json.gz
A	docs/art/production7/shots/god_d51_0200_F2N_soc_rain_night_daughter.jpg
A	docs/art/production7/shots/god_d51_0200_FV_soc_both_night_all.entities.json.gz
A	docs/art/production7/shots/god_d51_0200_FV_soc_both_night_all.jpg
A	docs/art/production7/shots/l_d10_1200_F2.entities.json.gz
A	docs/art/production7/shots/l_d10_1200_F2.jpg
A	docs/art/production7/shots/l_d10_1500_F2.entities.json.gz
A	docs/art/production7/shots/l_d10_1500_F2.jpg
A	docs/art/production7/shots/l_d10_1500_F5.entities.json.gz
A	docs/art/production7/shots/l_d10_1500_F5.jpg
A	docs/art/production7/shots/l_d15_1500_F2.entities.json.gz
A	docs/art/production7/shots/l_d15_1500_F2.jpg
A	docs/art/production7/shots/l_d15_1530_FC.entities.json.gz
A	docs/art/production7/shots/l_d15_1530_FC.jpg
A	docs/art/production7/shots/l_d16_1500_FC.entities.json.gz
A	docs/art/production7/shots/l_d16_1500_FC.jpg
A	docs/art/production7/shots/l_d18_1200_F2.entities.json.gz
A	docs/art/production7/shots/l_d18_1200_F2.jpg
A	docs/art/production7/shots/l_d23_0200_F2N.cold_cycle.gif
A	docs/art/production7/shots/l_d23_0200_F2N.cold_cycle.json.gz
A	docs/art/production7/shots/l_d23_0200_F2N.entities.json.gz
A	docs/art/production7/shots/l_d23_0200_F2N.jpg
A	docs/art/production7/shots/l_d24_1200_F1.entities.json.gz
A	docs/art/production7/shots/l_d24_1200_F1.jpg
A	docs/art/production7/shots/l_d24_1200_F2.entities.json.gz
A	docs/art/production7/shots/l_d24_1200_F2.jpg
A	docs/art/production7/shots/l_d24_2300_F2N.cold_cycle.gif
A	docs/art/production7/shots/l_d24_2300_F2N.cold_cycle.json.gz
A	docs/art/production7/shots/l_d24_2300_F2N.entities.json.gz
A	docs/art/production7/shots/l_d24_2300_F2N.jpg
A	docs/art/production7/shots/l_d25_0200_F2N.cold_cycle.gif
A	docs/art/production7/shots/l_d25_0200_F2N.cold_cycle.json.gz
A	docs/art/production7/shots/l_d25_0200_F2N.entities.json.gz
A	docs/art/production7/shots/l_d25_0200_F2N.jpg
A	docs/art/production7/shots/l_d27_0200_F2N.cold_cycle.gif
A	docs/art/production7/shots/l_d27_0200_F2N.cold_cycle.json.gz
A	docs/art/production7/shots/l_d27_0200_F2N.entities.json.gz
A	docs/art/production7/shots/l_d27_0200_F2N.jpg
A	docs/art/production7/shots/l_d27_0715_F2.entities.json.gz
A	docs/art/production7/shots/l_d27_0715_F2.jpg
A	docs/art/production7/shots/l_d28_0200_F2N.cold_cycle.gif
A	docs/art/production7/shots/l_d28_0200_F2N.cold_cycle.json.gz
A	docs/art/production7/shots/l_d28_0200_F2N.entities.json.gz
A	docs/art/production7/shots/l_d28_0200_F2N.jpg
A	docs/art/production7/shots/l_d28_0200_F2Z.cold_cycle.gif
A	docs/art/production7/shots/l_d28_0200_F2Z.cold_cycle.json.gz
A	docs/art/production7/shots/l_d28_0200_F2Z.entities.json.gz
A	docs/art/production7/shots/l_d28_0200_F2Z.jpg
A	docs/art/production7/shots/l_d29_0200_F2N.cold_cycle.gif
A	docs/art/production7/shots/l_d29_0200_F2N.cold_cycle.json.gz
A	docs/art/production7/shots/l_d29_0200_F2N.entities.json.gz
A	docs/art/production7/shots/l_d29_0200_F2N.jpg
A	docs/art/production7/shots/l_d29_0630_F2.entities.json.gz
A	docs/art/production7/shots/l_d29_0630_F2.jpg
A	docs/art/production7/shots/l_d29_0715_F2.entities.json.gz
A	docs/art/production7/shots/l_d29_0715_F2.jpg
A	docs/art/production7/shots/l_d32_1200_F2.entities.json.gz
A	docs/art/production7/shots/l_d32_1200_F2.jpg
A	docs/art/production7/shots/l_d40_1200_F2.entities.json.gz
A	docs/art/production7/shots/l_d40_1200_F2.jpg
A	docs/art/production7/shots/l_d56_0200_F2N.cold_cycle.gif
A	docs/art/production7/shots/l_d56_0200_F2N.cold_cycle.json.gz
A	docs/art/production7/shots/l_d56_0200_F2N.entities.json.gz
A	docs/art/production7/shots/l_d56_0200_F2N.jpg
A	docs/art/production7/shots/l_d57_0200_F2N.cold_cycle.gif
A	docs/art/production7/shots/l_d57_0200_F2N.cold_cycle.json.gz
A	docs/art/production7/shots/l_d57_0200_F2N.entities.json.gz
A	docs/art/production7/shots/l_d57_0200_F2N.jpg
A	docs/art/production7/shots/l_d57_0301_DE.cold_cycle.gif
A	docs/art/production7/shots/l_d57_0301_DE.cold_cycle.json.gz
A	docs/art/production7/shots/l_d57_0301_DE.entities.json.gz
A	docs/art/production7/shots/l_d57_0301_DE.jpg
A	docs/art/production7/shots/l_d84_0200_F2N.cold_cycle.gif
A	docs/art/production7/shots/l_d84_0200_F2N.cold_cycle.json.gz
A	docs/art/production7/shots/l_d84_0200_F2N.entities.json.gz
A	docs/art/production7/shots/l_d84_0200_F2N.jpg
A	docs/art/production7/shots/l_d85_0200_F2N.cold_cycle.gif
A	docs/art/production7/shots/l_d85_0200_F2N.cold_cycle.json.gz
A	docs/art/production7/shots/l_d85_0200_F2N.entities.json.gz
A	docs/art/production7/shots/l_d85_0200_F2N.jpg
A	docs/art/production7/shots/l_d85_0401_DE.cold_cycle.gif
A	docs/art/production7/shots/l_d85_0401_DE.cold_cycle.json.gz
A	docs/art/production7/shots/l_d85_0401_DE.entities.json.gz
A	docs/art/production7/shots/l_d85_0401_DE.jpg
A	docs/art/production7/shots/manifest.json
A	docs/art/production7/shots/manifest_art.json
A	docs/art/production7/shots/manifest_art_god.json
A	docs/art/production7/shots/manifest_art_long.json
A	docs/art/production7/shots/manifest_art_society.json
A	docs/art/production7/shots/soc_d28_2200_F2_orig.entities.json.gz
A	docs/art/production7/shots/soc_d28_2200_F2_orig.jpg
A	docs/art/production7/shots/soc_d29_0630_FD_leader.entities.json.gz
A	docs/art/production7/shots/soc_d29_0630_FD_leader.jpg
A	docs/art/production7/shots/soc_d29_0630_FT_site.entities.json.gz
A	docs/art/production7/shots/soc_d29_0630_FT_site.jpg
A	docs/art/production7/shots/soc_d29_0700_FD_leader.entities.json.gz
A	docs/art/production7/shots/soc_d29_0700_FD_leader.jpg
A	docs/art/production7/shots/soc_d29_0800_FD_leader.entities.json.gz
A	docs/art/production7/shots/soc_d29_0800_FD_leader.jpg
A	docs/art/production7/shots/soc_d29_0819_FS_daughter.entities.json.gz
A	docs/art/production7/shots/soc_d29_0819_FS_daughter.jpg
A	docs/art/production7/shots/soc_d29_0909_FV_all.entities.json.gz
A	docs/art/production7/shots/soc_d29_0909_FV_all.jpg
A	docs/art/production7/shots/soc_d29_1200_FV_all.entities.json.gz
A	docs/art/production7/shots/soc_d29_1200_FV_all.jpg
A	docs/art/production7/shots/soc_d29_1200_FV_all_selected_daughter.entities.json.gz
A	docs/art/production7/shots/soc_d29_1200_FV_all_selected_daughter.jpg
A	docs/art/production7/shots/soc_d29_1200_FV_all_selected_orig.entities.json.gz
A	docs/art/production7/shots/soc_d29_1200_FV_all_selected_orig.jpg
A	docs/art/production7/shots/soc_d51_0200_F2N_daughter.entities.json.gz
A	docs/art/production7/shots/soc_d51_0200_F2N_daughter.jpg
A	docs/art/production7/shots/soc_d51_1200_FV_all.entities.json.gz
A	docs/art/production7/shots/soc_d51_1200_FV_all.jpg
A	docs/art/production7/shots/soc_d84_0200_F2N_daughter.entities.json.gz
A	docs/art/production7/shots/soc_d84_0200_F2N_daughter.jpg
A	docs/art/production7/shots/soc_d84_1200_FV_all.entities.json.gz
A	docs/art/production7/shots/soc_d84_1200_FV_all.jpg
A	docs/art/production7/shots/soc_d85_0101_DE_daughter.entities.json.gz
A	docs/art/production7/shots/soc_d85_0101_DE_daughter.jpg
A	docs/art/production7/shots/soc_d85_0630_FD_leader.entities.json.gz
A	docs/art/production7/shots/soc_d85_0630_FD_leader.jpg
A	docs/art/production7/shots/soc_d85_0842_FV_all.entities.json.gz
A	docs/art/production7/shots/soc_d85_0842_FV_all.jpg
A	docs/art/production7/shots/strip.json.gz
```

### `847f96b928352cd48d7e5e49bf3c748ee8cfffdc`

art-p7(p8): distinguish visible acceptance from measured failures

```text
A	docs/art/production7/CHANGE_INVENTORY.md
M	docs/art/production7/DECISIONS.md
M	docs/art/production7/MEASURED_DAYS.md
M	docs/art/production7/MULTI_SETTLEMENT_UI_AUDIT.md
M	docs/art/production7/TASKS.md
A	docs/art/production7/artboard_sources.json
A	docs/art/production7/audit_closure.json
A	docs/art/production7/before-after-live.png
A	docs/art/production7/before-after-society.png
A	docs/art/production7/blind/activity/blind_2x.png
A	docs/art/production7/blind/activity/blind_4x.png
A	docs/art/production7/blind/activity/blind_instructions.txt
A	docs/art/production7/blind/activity/observer_a.json
A	docs/art/production7/blind/activity/observer_a.sha256
A	docs/art/production7/blind/activity/observer_b.json
A	docs/art/production7/blind/activity/observer_b.sha256
A	docs/art/production7/blind/activity/result_a.json
A	docs/art/production7/blind/activity/result_b.json
A	docs/art/production7/blind_identity_final.json
A	docs/art/production7/blind_results.json
A	docs/art/production7/blind_society/A27_1.png
A	docs/art/production7/blind_society/A27_2.png
A	docs/art/production7/blind_society/A27_3.png
A	docs/art/production7/blind_society/A27_4.png
A	docs/art/production7/blind_society/A28_1.png
A	docs/art/production7/blind_society/A28_2.png
A	docs/art/production7/blind_society/frozen_answers.json
A	docs/art/production7/blind_society/frozen_answers.sha256
A	docs/art/production7/blind_society/instructions.txt
A	docs/art/production7/blind_society/observer_d.json
A	docs/art/production7/blind_society/observer_d.sha256
A	docs/art/production7/blind_society/observer_e.json
A	docs/art/production7/blind_society/observer_e.sha256
A	docs/art/production7/blind_society/results.json
A	docs/art/production7/blind_society_round1_archive.json
A	docs/art/production7/deaths.png
A	docs/art/production7/founding.png
A	docs/art/production7/harvest/installed_groups_final.json
A	docs/art/production7/harvest/integrity_final.json
A	docs/art/production7/ingame_qa.json
A	docs/art/production7/ingame_qa.md
A	docs/art/production7/measurement_summary.json
A	docs/art/production7/measurements.json
A	docs/art/production7/measurements.md
D	docs/art/production7/measurements/headless_20261010.json
A	docs/art/production7/measurements/headless_20261010.json.gz
D	docs/art/production7/measurements/headless_20261013.json
A	docs/art/production7/measurements/headless_20261013.json.gz
A	docs/art/production7/measurements/headless_compression.json
A	docs/art/production7/performance/frame_times_soc_1.json.gz
A	docs/art/production7/performance/frame_times_soc_3.json.gz
A	docs/art/production7/performance/manifest_art_society.json
A	docs/art/production7/performance_final/frame_times_soc_1.json.gz
A	docs/art/production7/performance_final/frame_times_soc_3.json.gz
A	docs/art/production7/performance_final/manifest_art_society.json
A	docs/art/production7/provisional_analysis_archive.json
A	docs/art/production7/review/board_sources.json
A	docs/art/production7/review/boards/hud_batch_001.png
A	docs/art/production7/review/boards/hud_batch_002.png
A	docs/art/production7/review/boards/hud_batch_003.png
A	docs/art/production7/review/boards/hud_batch_004.png
A	docs/art/production7/review/boards/hud_batch_005.png
A	docs/art/production7/review/boards/hud_batch_006.png
A	docs/art/production7/review/boards/hud_batch_007.png
A	docs/art/production7/review/boards/hud_batch_008.png
A	docs/art/production7/review/boards/hud_batch_009.png
A	docs/art/production7/review/boards/hud_batch_010.png
A	docs/art/production7/review/boards/hud_batch_011.png
A	docs/art/production7/review/boards/hud_batch_012.png
A	docs/art/production7/review/boards/hud_batch_013.png
A	docs/art/production7/review/boards/hud_batch_014.png
A	docs/art/production7/review/boards/hud_batch_015.png
A	docs/art/production7/review/boards/hud_batch_016.png
A	docs/art/production7/review/boards/hud_batch_017.png
A	docs/art/production7/review/boards/hud_batch_018.png
A	docs/art/production7/review/boards/hud_batch_019.png
A	docs/art/production7/review/boards/hud_batch_020.png
A	docs/art/production7/review/boards/hud_batch_021.png
A	docs/art/production7/review/boards/hud_batch_022.png
A	docs/art/production7/review/boards/hud_batch_023.png
A	docs/art/production7/review/boards/hud_batch_024.png
A	docs/art/production7/review/boards/hud_batch_025.png
A	docs/art/production7/review/boards/hud_batch_026.png
A	docs/art/production7/review/boards/hud_batch_027.png
A	docs/art/production7/review/boards/hud_batch_028.png
A	docs/art/production7/review/boards/hud_batch_029.png
A	docs/art/production7/review/boards/hud_batch_030.png
A	docs/art/production7/review/boards/hud_batch_031.png
A	docs/art/production7/review/boards/hud_batch_032.png
A	docs/art/production7/review/boards/hud_batch_033.png
A	docs/art/production7/review/boards/hud_batch_034.png
A	docs/art/production7/review/boards/hud_batch_035.png
A	docs/art/production7/review/boards/hud_batch_036.png
A	docs/art/production7/review/boards/hud_batch_037.png
A	docs/art/production7/review/cold_observer.json
A	docs/art/production7/review/cold_observer_final.json
A	docs/art/production7/review/cycle_contacts.json
A	docs/art/production7/review/hud_batches.json
A	docs/art/production7/review/hud_observer.json
A	docs/art/production7/review/hud_observer_final.json
A	docs/art/production7/review/independent_b.json
A	docs/art/production7/review/independent_b_round2.json
A	docs/art/production7/review/independent_c.json
A	docs/art/production7/review/independent_c_round2.json
A	docs/art/production7/review/intervention_observer.json
A	docs/art/production7/review/intervention_observer_clarification.json
A	docs/art/production7/review/intervention_observer_final.json
A	docs/art/production7/review/measurement_code_review.json
A	docs/art/production7/review_round1_archive.json
A	docs/art/production7/villages.png
A	docs/art/production7/visual_observations.json
A	docs/art/production7/visual_observations_archive.json
A	tools/art_pipeline/artboards_p7.py
A	tools/art_pipeline/blind_p7.py
A	tools/art_pipeline/capture_contract_p7.py
A	tools/art_pipeline/compact_previews_p7.py
A	tools/art_pipeline/harvest_integrity_p7.py
A	tools/art_pipeline/ingame_qa_age_p7.py
A	tools/art_pipeline/ingame_qa_cold_p7.py
A	tools/art_pipeline/ingame_qa_deaths_p7.py
A	tools/art_pipeline/ingame_qa_founding_p7.py
A	tools/art_pipeline/ingame_qa_intervention_p7.py
A	tools/art_pipeline/ingame_qa_observed_p7.py
A	tools/art_pipeline/ingame_qa_p7.py
A	tools/art_pipeline/ingame_qa_performance_p7.py
A	tools/art_pipeline/ingame_qa_pixels_p7.py
A	tools/art_pipeline/ingame_qa_society_p7.py
A	tools/art_pipeline/ingame_qa_villages_p7.py
A	tools/art_pipeline/ingame_qa_wood_p7.py
A	tools/art_pipeline/measure_p7.py
A	tools/art_pipeline/measurement_summary_p7.py
A	tools/art_pipeline/observation_rules_p7.py
A	tools/art_pipeline/review_boards_p7.py
A	tools/art_pipeline/test_measurement_p7.py
A	tools/art_pipeline/verify_blind_identity_p7.py
```

## Part 9 캡처 계약 회귀 수정

### `90d8e6dc85153b842b48e315603f36f77948f2a5`

```text
M	docs/art/TEST_LEDGER.md
M	docs/art/production7/DECISIONS.md
M	scripts/test/art/test_capture_plans_p6.gd
```
