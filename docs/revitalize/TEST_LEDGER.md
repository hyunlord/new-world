# Revitalize test ledger

Only source-inspection assertions for intentionally replaced renderer details
may change. Legacy behavioral assertions and the Part 1 population pin remain
unchanged. Every re-pinned or retired assertion is listed individually below.

Part 1: no existing assertion changed or retired. New tests cover cumulative
tallies, frozen legacy trajectory, probe schema/fingerprints/summary/arguments,
and tooling time/tallies/clamped advancement collectors.

| File | Test fn | Old assertion | New assertion or retired | Reason |
|---|---|---|---|---|
| rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs | harness_p13_epsilon_a3_three_on_building_placed_calls_total | exactly three renderer boot spawn calls | zero renderer spawn calls | Part 2 moves world construction into seeded Rust worldgen. |
| rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs | harness_p13_epsilon_a4_on_building_placed_arg_signatures | three fixed building spawn argument tuples | get_building_snapshot and four parallel data arrays | Part 2 moves world construction into seeded Rust worldgen. |
| rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs | harness_p13_epsilon_a5_extra_sprite_block_reuses_building_tex | two fake flank sprites reuse campfire texture | real building row uses cached texture indexed by Rust kind | Part 2 moves world construction into seeded Rust worldgen. |
| rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs | harness_p13_epsilon_a6_extra_sprite_z_index_is_z_building | fake flank sprites use building z layer | real building rows use building z layer | Part 2 moves world construction into seeded Rust worldgen. |
| rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs | harness_p13_epsilon_a7_extra_sprite_y_position_uses_bootstrap_y | fake flank sprite Y uses bootstrap constant | real building coordinates use snapshot xs/ys and sprite origin | Part 2 moves world construction into seeded Rust worldgen. |
| rust/crates/sim-test/tests/harness_p12_beta_terrain.rs | harness_p12_beta_a11_bootstrap_sprite_positioned_at_bootstrap_tile | bootstrap sprite position references fixed bootstrap tile | real building sprite position uses snapshot tile plus origin and tile centre | Part 2 moves world construction into seeded Rust worldgen. |
| rust/crates/sim-test/tests/harness_t7_9_b_render_mechanism.rs | harness_t7_9_b_world_renderer_render_path_present | renderer boots fixed building via on_building_placed | renderer reads get_building_snapshot and never calls on_building_placed | Part 2 moves world construction into seeded Rust worldgen. |
| rust/crates/sim-test/tests/harness_resource_scarcity.rs | harness_scarcity_a16_production_init_seeds_all_three_kinds_finite | Comments and failure messages describe init_production_engine as the live dylib path | Wording identifies frozen legacy V7 fixture; all predicates and test data unchanged | Part 2 live entry is worldgen::build_world; wording-only correction, not an assertion retirement. |
| rust/crates/sim-test/tests/harness_show_resource_depletion.rs | harness_resource_depletion_a9_production_scene_registers_finite_capacities | Module/test comments and failure message call legacy constructor the live production path | Wording identifies legacy V7 fixture; finite-capacity predicate unchanged | Correct provenance after Part 2; wording-only correction. |
| rust/crates/sim-test/tests/harness_show_resource_depletion.rs | harness_resource_depletion_a10_live_depletion_surfaces | Comments/failure message imply current live-game depletion trajectory | Wording identifies legacy V7 fixture; depletion predicates and tick counts unchanged | Correct provenance after Part 2; wording-only correction. |
| rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs | harness_p14_gamma_a23_world_renderer_no_extra_ffi_added | World renderer call allowlist includes boot mutation and excludes snapshot/reset subscription | Replace on_building_placed with typed get_building_snapshot; permit only world_reset refresh connect; retain all other FFI and inspector guards | Part 2 moves boot ownership to Rust and refreshes after deterministic reset; dynamic call remains forbidden. |
