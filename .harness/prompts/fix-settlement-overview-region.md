# Fix settlement overview circle — fixed center + actual-region radius (A+B)

HEAD: 475fbfe1. GDScript renderer + one harness re-point (`--quick` lane). No Rust/sim change.

## Section 1: Implementation Intent

**Why this exists.** The translucent blue settlement-region circle (drawn by
`settlement_overview_renderer.gd` at MEDIUM/FAR overview zoom) has two user
complaints: (1) it shakes — its center is the live member **centroid** (mean of
member positions), so it drifts every frame as members wander; (2) it overlaps
excessively — its radius is **member-count-scaled** (`BASE_RADIUS_PX 24 +
PER_MEMBER_RADIUS_PX 6 × members`, clamped to 160px), so a 20-46 member
settlement draws a 144-160px disc, ~2× the actual region (PROXIMITY_RADIUS 5
tiles = 80px). The renderer's own comment admits "no true polygon boundary in
substrate → member-count-scaled disc" — it was a rough affordance.

**Approach (A+B, keep the circle).** (A) Center the circle on the FIXED
`formation_tile` (already in the snapshot as `formation_xs/ys` from the marker
fix, commit 475fbfe1) instead of the moving centroid → the circle stops shaking
and sits exactly under the settlement marker. (B) Set the radius to the ACTUAL
region size: `SETTLEMENT_PROXIMITY_RADIUS` (5 tiles) × `TILE_SIZE` (16) = 80px,
fixed for every settlement → removes the member-count exaggeration and the
excessive overlap. The membership-judgment region is actually a Chebyshev-5
square (11×11 tiles); the circle remains an approximation of that, but now sized
to the real region rather than inflated by population.

**Tradeoffs.** The circle still approximates a square region — the user
explicitly chose to keep the circle (no square conversion in scope). The centroid
FFI keys (`centroid_xs/ys`) stay in the snapshot (untouched — they have no other
consumer now, but removing them is a separate FFI change out of this GDScript
ticket and the marker fix's p12_gamma A6/A8 still unit-test the centroid mean).

## Section 2: What to Build  ⚠️ LOCKED SCOPE — DO NOT EXPAND ⚠️

**Exactly two files. Re-anchor the overview circle to the fixed formation tile
and size it to the actual region. No FFI/sim change, no other renderer, no
zoom-LOD change.**

| # | Authorized file | Change |
|---|-----------------|--------|
| 1 | `scripts/ui/settlement_overview_renderer.gd` | (a) `_process`: read `snap_dict.get("formation_xs"/"formation_ys")` instead of `centroid_xs/centroid_ys` (fixed center, same basis as the marker). Stop reading `member_counts` (no longer needed for radius). (b) Replace the member-scaled radius: remove `const BASE_RADIUS_PX` / `PER_MEMBER_RADIUS_PX` / `MAX_RADIUS_PX`; add `const SETTLEMENT_REGION_RADIUS_TILES := 5` (doc: "mirrors sim-core SETTLEMENT_PROXIMITY_RADIUS") and compute `var r := float(SETTLEMENT_REGION_RADIUS_TILES * TILE_SIZE)` (= 80px) for every settlement. (c) Update the module + inline comments: center = fixed `formation_tile`; radius = actual PROXIMITY region, not member-scaled. Keep: `extends Node2D`, `Z_OVERVIEW=1`, `OVERVIEW_ALPHA=0.28`, `TILE_SIZE`/`SPRITE_ORIGIN_X`/`SPRITE_ORIGIN_Y`, `visible=false` start + `not visible` early-return, `get_settlement_snapshot` as the only FFI call, `draw_circle` in `_draw`. |
| 2 | `rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs` | **a15 re-point (authorized, assertion not weakened).** `harness_zoom_a15_draw_circle_radius_scales_by_member_count` asserts the file contains `member_counts` AND `BASE_RADIUS_PX` — the exact member-scaling being removed. Rename to `…_radius_is_fixed_region` and re-point: keep the `draw_circle` in `_draw` check; replace the `member_counts`/`BASE_RADIUS_PX` substring checks with `SETTLEMENT_REGION_RADIUS_TILES` present AND `PER_MEMBER_RADIUS_PX` ABSENT (negative invariant — radius no longer member-scaled). Update the doc comment. Do NOT touch a10-a14, a16 (a16 `TILE_SIZE`/`SPRITE_ORIGIN_*` basis stays green; a13 reads-only `get_settlement_snapshot` stays green; a11/a12 Z/alpha unchanged). |

**Scope boundary — NOT in this ticket:** the snapshot FFI / `formation_xs/ys` /
`centroid_xs/ys` (all already exist — read-only consumer change); circle→square
conversion (user kept the circle); zoom-LOD visibility / whether the disc shows
at 1.0× (separate D follow-up — note only); formation/marker/sim logic; the
centroid computation in world_node.rs (kept).

## Section 3: How to Implement

**a15 substring-lock note.** The `--quick` gate runs `cargo test --workspace`
incl. `harness_p14_zeta`. After the re-point, a15 must assert the NEW behavior;
a16 (`TILE_SIZE`+`SPRITE_ORIGIN_X`+`SPRITE_ORIGIN_Y` all present) stays satisfied
because those consts are kept; a13 (`get_settlement_snapshot` is the only FFI
literal; the 8 other FFI names absent) stays satisfied — `formation_xs`/`centroid_xs`
are dict `.get` keys, not FFI method names, so switching the read key is unlocked.

**Renderer shape.** `_process` keeps the same structure: guard `not visible`,
fetch snapshot, validate `formation_xs`/`formation_ys` are `PackedInt32Array` of
equal size, build `_centroids` (now formation-anchored) + `_radii` (now all 80px),
`queue_redraw()`. `_draw` is unchanged (`draw_circle(_centroids[i], _radii[i],
OVERVIEW_COLOR)`). The per-settlement radius is a constant, so `_radii` is N
copies of 80.0.

## Section 4: Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|--------|--------------|------|-----------|
| T1 | `settlement_overview_renderer.gd` center+radius | 🔴 DIRECT | — |
| T2 | `harness_p14_zeta` a15 re-point | 🟢 DISPATCH | T1 |

T1 DIRECT (single renderer edit). 1/2 dispatchable.

## Section 5: Localization Checklist

No new localization keys. (Renderer geometry only; no user-visible text.)

## Section 6: Verification & Notion

**Gate (no ENV-BYPASS):**
```bash
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail
cargo test -p sim-test harness_p14_zeta -- --nocapture 2>&1 | tail   # a15 re-pointed, a10-a16 green
```
GDScript parse: the `--quick` Step 2.4 strict check parses the renderer
(treat-warnings-as-errors) — the edit must introduce no parse error / new warning
(watch for unused-variable warnings if `members`/`member_counts` reads are
removed but a leftover local remains).

**Reproduction:** before — circle center moves with members + radius inflates to
~160px for big settlements; after — circle is centered on the fixed formation tile
(co-located with the marker) and is a constant 80px (5-tile region) regardless of
member count → no shake, less overlap.

**Visual:** `--quick` Visual Verify launches Godot; at overview zoom the blue
discs should sit still and be uniformly sized. (NOTE for the operator: overview
discs are only `visible` at MEDIUM/FAR zoom per zoom_lod_controller — windowed
confirm needs zoom-out; whether they should show at 1.0× is a separate follow-up.)

**★ No dylib rebuild** (GDScript + test only; no Rust production change).

**Notion:** update the V7 progress log with the overview-region fix.
