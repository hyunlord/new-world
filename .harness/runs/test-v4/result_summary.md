---
feature: test-v4
code_attempt: 1
---

## Files Changed
- `rust/crates/sim-test/src/main.rs`: Restructured 6 building-visuals tests → 9 tests with clearer separation (config discriminators A1-A3 as ticks=0 static checks, replaced brittle A4 with cross-source consistency A7)

## Observed Values (seed 42, 20 agents)
- floor_alpha config: 0.55
- floor_border_width config: 0.5
- wall_autotile_enabled config: true
- wall_autotile_bridge_px config: 2.0
- furniture_icon_scale config: 0.7
- floor_count (4380 ticks): 10
- wall_count (4380 ticks): 15
- empty_material_count: 0
- adjacent_pairs: 14
- storage_pit_count: 1
- complete_stockpile_count: 1
- locale en BUILDING_TYPE_STOCKPILE: "Stockpile"
- locale ko BUILDING_TYPE_STOCKPILE: "보관소"

## Threshold Compliance
- A1 (config_floor): plan=floor_alpha==0.55 AND border==0.5, observed=0.55/0.5, PASS
- A2 (config_wall_autotile): plan=enabled==true AND bridge_px==2.0, observed=true/2.0, PASS
- A3 (config_furniture_icon): plan=icon_scale==0.7, observed=0.7, PASS
- A4 (floor_tiles_stamped): plan=>=6 AND <=500, observed=10, PASS
- A5 (wall_material_valid): plan=>=8 walls AND 0 empty, observed=15/0, PASS
- A6 (adjacent_wall_pairs): plan=>=4, observed=14, PASS
- A7 (storage_pit_consistency): plan=pit_count==stockpile_count AND >=1, observed=1==1, PASS
- A8 (localization_stockpile): plan=key present+non-empty in en+ko, observed="Stockpile"/"보관소", PASS
- A9 (wall_count_bounded): plan=<=500, observed=15, PASS

## Gate Result
- cargo test: PASS (921 passed, 0 failed)
- clippy: PASS
- harness: PASS (9/9 passed)

## Notes
- This is a RE-CODE from the previous attempt. The implementation (building_renderer.gd, config constants, localization keys) was already complete.
- Key improvement: replaced brittle A4 (storage_pit >= 1 with observed=1, zero slack) with A7 cross-source consistency invariant (storage_pit_count == complete_stockpile_count). The equality assertion is not brittle regardless of count because stamp_stockpile_structure() deterministically places exactly one storage_pit per completed stockpile.
- Config discriminators (A1-A3) are now standalone ticks=0 static checks, cleanly separated from simulation-based checks.
- The clippy failures mentioned in previous review (knowledge.rs, social.rs, band.rs) are no longer present — they were fixed in prior commits.
- No threshold changes made from plan. All thresholds are locked and all assertions pass within plan bounds.
