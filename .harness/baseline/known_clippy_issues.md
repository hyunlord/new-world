# Known Pre-existing Clippy Issues (Baseline)

**Established**: 2026-05-03 during Path 1.6 toolchain-drift handling for
shelter-ring-center-fix-v1.
**Toolchain**: Rust 1.93.0 (stable)
**Trigger**: Rust 1.93 introduced new lints not present in 1.92.0; pre-existing
code that was clean under 1.92 now emits warnings/errors under
`-D warnings`.

## CLAUDE.md Rule 7.1 Coverage (v3.2.1)

ENV-BYPASS authorization tolerates clippy issues whose location appears in the
companion file `clippy_baseline_raw.txt`. Any clippy issue whose
`file:line:col` is **not** in that registry counts as a NEW regression and
blocks authorization.

This file is the human-readable explanation; `clippy_baseline_raw.txt` is the
authoritative machine-readable list consumed by `authorize_env_bypass.sh`.

## Update Protocol

- When a baseline issue is FIXED, regenerate the registry in the same commit.
  **Important**: do NOT use `cargo clippy --workspace --all-targets -- -D warnings`
  alone — it short-circuits when the first crate's lib-test errors abort
  dependent compilation, leaving downstream crates unchecked. Use per-crate
  iteration instead:
  ```bash
  cd rust
  for crate in sim-core sim-data sim-engine sim-systems sim-bridge sim-test; do
      cargo clippy -p $crate --all-targets -- -D warnings \
          > /tmp/clippy_${crate//-/_}.log 2>&1 || true
  done
  cat /tmp/clippy_sim_*.log | grep -E "^\s*--> " | sed 's/^[[:space:]]*//' \
      | sort -u > .harness/baseline/clippy_baseline_raw.txt
  ```
- When a NEW issue appears (toolchain bump, dependency update, etc.) judged
  unrelated to the change under review, document it here AND extend
  `clippy_baseline_raw.txt` in a separate commit with rationale.
- Never silently expand the registry to make a noisy commit pass — the whole
  point of the baseline is to detect regressions, not paper over them.

## Affected Lint Categories (2026-05-03 snapshot, 187 locations)

> **Note**: The initial 2026-05-03 morning generation produced 63 entries because
> `cargo clippy --workspace --all-targets -- -D warnings` short-circuits when the
> first crate's lib-test errors abort dependent compilation, leaving downstream
> crates (sim-bridge, sim-test) unchecked. Afternoon regeneration switched to
> per-crate iteration (`cargo clippy -p <crate>` for each of the 6 workspace
> members) and captured 187 unique locations across all crates.

| Lint | Approx count | Notes |
|------|--------------|-------|
| `clippy::useless_vec` | 3 | `vec![...]` literals in tests where `[...]` works |
| `clippy::comparison_to_empty` | 9 | `vec.len() == 0` → `vec.is_empty()` |
| `clippy::field_reassign_with_default` | ~50 | `let x = T::default(); x.f = …` patterns |
| `clippy::needless_range_loop` | ~5 | `for i in 0..arr.len()` instead of `iter().enumerate()` |
| `clippy::unnecessary_cast` (u64→u64) | ~6 | `(x as u64)` where x is already u64 |
| `clippy::int_plus_one` | ~2 | `>= y + 1` should be `> y` |
| `clippy::unused_variable` | ~1 | `b` introduced but unused |
| `clippy::redundant_pattern_matching` | 4 | `get("…").is_some()` → `contains_key("…")` |
| `clippy::manual_range_contains` | ~30 | `x >= a && x <= b` → `(a..=b).contains(&x)` |
| `clippy::manual_is_multiple_of` | ~5 | `n % k == 0` → `n.is_multiple_of(k)` |
| `clippy::len_zero` | ~2 | `vec.len() > 0` → `!vec.is_empty()` |
| `clippy::needless_borrow` | ~10 | unnecessary `&` in function call args |
| `clippy::unused_imports` | ~1 | `use super::*;` in test mod with no usage |

## Affected Files (origin/lead/main HEAD as of 2026-05-03 afternoon, complete)

```
rust/crates/sim-bridge/src/lib.rs
rust/crates/sim-core/src/components/knowledge.rs
rust/crates/sim-core/src/components/social.rs
rust/crates/sim-core/src/config.rs
rust/crates/sim-data/src/lib.rs
rust/crates/sim-data/tests/ron_registry_test.rs
rust/crates/sim-engine/src/frame_snapshot.rs
rust/crates/sim-systems/src/runtime/band.rs
rust/crates/sim-systems/src/runtime/cognition.rs
rust/crates/sim-systems/src/runtime/health.rs
rust/crates/sim-systems/src/runtime/knowledge_learning.rs
rust/crates/sim-systems/src/runtime/mod.rs
rust/crates/sim-systems/src/runtime/pairwise.rs
rust/crates/sim-systems/src/runtime/steering.rs
rust/crates/sim-systems/src/runtime/steering_derive.rs
rust/crates/sim-test/src/main.rs
```

Per-crate counts:
- sim-bridge: 11 entries
- sim-core: 6 entries
- sim-data: 14 entries
- sim-engine: 2 entries
- sim-systems: 42 entries
- sim-test: 112 entries
- **Total: 187 unique `file:line:col` locations**

All issues are in code that was clippy-clean under Rust 1.92 and remain
correct at runtime — they are stylistic refinements introduced by Rust 1.93.

## Cleanup Plan (Backlog)

Tracked in `.harness/audit/rust_193_cleanup_backlog.md`. Mechanical fixes
only; planned as a single dedicated cleanup commit at developer convenience.
After cleanup, regenerate the raw registry — empty file means baseline is
clear.

## Tolerance Boundary

| Scenario | Action |
|----------|--------|
| Clippy issue at location ∈ registry | Tolerated by ENV-BYPASS |
| New clippy issue at location ∉ registry | Blocks ENV-BYPASS |
| Toolchain bump introduces new lint | Document here + extend registry in separate commit, then re-authorize |
| Same lint at NEW location after refactor | Blocks (you must fix or explicitly extend registry) |
