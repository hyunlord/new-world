---
feature: test-v4-fix3
code_attempt: 1
---

## Files Changed
- `rust/crates/sim-test/src/main.rs`: Updated 8 harness tests — tightened thresholds (floor ≤80, wall ≤120, adjacency ≥5, furniture [1,60]), replaced bridge_uses_snapshot with coordinate_bounds check

## Observed Values (seed 42, 20 agents)
- render_floor_alpha: 0.55
- render_floor_border_width: 0.5
- render_furniture_icon_scale: 0.7
- render_wall_autotile: true
- render_wall_bridge_px: 2.0
- floor_count: 10
- wall_count: 15
- empty_material_count: 0
- wall_adj_right: 6, wall_adj_down: 8, total_adj: 14
- storage_pit_count: 1
- total_furniture: 3
- grid_dimensions: 256×256
- en/BUILDING_TYPE_STOCKPILE: "Stockpile"
- ko/BUILDING_TYPE_STOCKPILE: "보관소"

## Threshold Compliance
- Assertion 1 (render_config): plan=exact match 5 constants, observed=all match, PASS
- Assertion 2 (regression_guard): plan=alpha≠0.35 & icon_scale≠0.6, observed=0.55/0.7, PASS
- Assertion 3 (floor_bounded): plan=[6, 80], observed=10, PASS
- Assertion 4 (wall_valid): plan=[8, 120] + zero empty materials + triple parity, observed=15/0, PASS
- Assertion 5 (adjacency_consistency): plan=cross-validated + total ≥5, observed=14 (6+8), PASS
- Assertion 6 (furniture_storage_pit): plan=storage_pit≥1 + total∈[1,60] + triple parity, observed=1/3, PASS
- Assertion 7 (localization_stockpile): plan=key in en+ko JSON, observed=present both, PASS
- Assertion 8 (coordinate_bounds): plan=all coords in [0, grid_dim), observed=all within 256×256, PASS

## Gate Result
- cargo test: PASS (920 passed, 0 failed, 3 ignored)
- clippy: PASS (zero warnings)
- harness: PASS (8/8 passed)

## Notes
- No threshold modifications were made. All plan thresholds are preserved exactly as specified.
- The feature implementation (GDScript building_renderer.gd, Rust config, tile_grid_snapshot) was already in place from previous attempts. This attempt only updates test thresholds to match the tightened plan.
- Pre-existing `stage1_simulation_tick_under_budget` test is `#[ignore]`-d (timing-sensitive benchmark, not a harness test).
- The old assertion 8 (`bridge_uses_snapshot` source grep) was replaced with the new plan's assertion 8 (`coordinate_bounds`) which validates all 6 coordinate arrays against the tile grid dimensions.
- All observed values are well within the tightened bounds (floor 10/80, wall 15/120, adjacency 14/5, furniture 3/60).
