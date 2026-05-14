---
feature: p3-gamma-2-beta-tile-click-chain
code_attempt: 2
---

## Files Changed
- rust/crates/sim-test/tests/harness_p3_gamma_2_beta_tile_click_chain.rs: A1 + A2 now use `matches(...).count()` with `assert_eq!(count, 1, ...)` instead of `contains(...)`, enforcing exact occurrence count per plan thresholds.

## Observed Values (seed 42, 20 agents)
- A1 occurrence count of `SPRITE_ORIGIN_X := 448` in scripts/ui/world_renderer.gd: 1
- A2 occurrence count of `SPRITE_ORIGIN_Y := 28` in scripts/ui/world_renderer.gd: 1

## Threshold Compliance
- Assertion 1 (SPRITE_ORIGIN_X declared): plan=`count == 1`, observed=1, PASS
- Assertion 2 (SPRITE_ORIGIN_Y declared): plan=`count == 1`, observed=1, PASS

## Gate Result
- cargo test: PASS (443 passed, 0 failed across workspace)
- clippy: PASS (cargo clippy --workspace --all-targets -- -D warnings exit 0)
- harness: PASS (23/23 passed in harness_p3_gamma_2_beta_tile_click_chain)

## Notes
- Only the two specified test functions (A1, A2) were modified, scope-limited to the previous-issues list.
- The matched literal `SPRITE_ORIGIN_X := 448` / `SPRITE_ORIGIN_Y := 28` includes the exact value 448/28; if a maintainer changes the constant value, A1/A2 will fail loudly (intended).
- No production code (Rust or GDScript) was touched; this is a pure test-harness tightening per the planner's exact-count threshold.
