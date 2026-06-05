# Feature: show-resource-depletion (viz-A — resource markers show amount)

Make resource depletion/regen VISIBLE. The scarcity feature shipped
(`ad3f2ec8`) but in a windowed run you cannot tell whether sources deplete:
the on-screen source markers never change. Make each marker's size + opacity
track its backend `amount / max` ratio, updated every frame.

First of the 4-part visualisation pass: **A (amount) → B (need bars) → C
(inspector overlap) → D (death visual)**. This is A only.

---

## Section 1: Implementation Intent

### Root cause (two reasons the markers don't show depletion)
1. The FFI resource snapshot (`ResourceSnapshotRow`) carried only `(x, y,
   kind)` — no current amount, no capacity. The renderer had nothing to size
   a marker by.
2. `world_renderer.gd::_render_resource_sources()` drew each marker ONCE,
   guarded by `_resource_sources_drawn`, so even backend changes were never
   reflected — markers were static from `_ready`.

### Approach
- Extend the FFI snapshot with `amount` (live `*_tiles` counter) + `max`
  (`*_source_max` ceiling, already populated by the scarcity seeder).
- Reconcile markers EVERY frame (borrow the `_furniture_sprites` create/
  update/reap pattern): a marker's node `scale` and fill `alpha` both lerp
  across `amount / max`, so a depleting source shrinks + fades, a regenerating
  one recovers, and a source consumed to 0 (removed from the backend map →
  absent from the snapshot) is reaped.

Backend resource LOGIC is untouched — this is a pure read-only visualisation.

---

## Section 2: What to Build

**Authorised scope (entire diff):**

FFI:
- `rust/crates/sim-bridge/src/ffi/world_node.rs` — `ResourceSnapshotRow` gains
  `pub amount: u8` + `pub max: u8`; `collect_resource_snapshot` reads the tile
  value (amount) and `*_source_max` (max, defaulting to amount + floored at 1);
  NEW `resource_rows_amounts(rows) -> (Vec<i32>, Vec<i32>)`;
  `resource_rows_to_dict` adds `amounts` + `maxes` `PackedInt32Array` keys.
  `resource_rows_split` (xs/ys/kinds) is UNCHANGED.
- `rust/crates/sim-bridge/src/ffi/mod.rs` — export `resource_rows_amounts`.

GDScript:
- `scripts/ui/world_renderer.gd` — replace once-only `_render_resource_sources`
  with per-frame `_update_resource_markers` (`_resource_markers` dict, keyed by
  packed (x,y,kind); scale = lerp(`SOURCE_RATIO_MIN_SCALE` 0.3, 1.0, ratio),
  alpha = lerp(`SOURCE_RATIO_MIN_ALPHA` 0.2, 1.0, ratio); reap absent markers);
  remove `_resource_sources_drawn`; update the per-frame call site.

Harness:
- `rust/crates/sim-test/tests/harness_show_resource_depletion.rs` (NEW, 5
  assertions): amounts/maxes present + aligned; amount == tile value, max ==
  source_max; ratio reflects depletion; max floored for unregistered source;
  (xs,ys,kinds) contract preserved.
- `rust/crates/sim-test/tests/harness_s16_gamma_visual.rs` — A4 + A7 retarget
  the GDScript-source lookup from `_render_resource_sources` →
  `_update_resource_markers` (the function was renamed; the assertions — marker
  is Polygon2D from SOURCE_KIND_COLORS with z_index, reads get_resource_snapshot
  — are unchanged in intent).

- `rust/crates/sim-test/tests/harness_t7_10_b1_space_toggle.rs` — Assertion 7
  retargets the `_process`-must-call substrate check from
  `_render_resource_sources()` → `_update_resource_markers()` (same per-frame,
  outside-the-overlay-OFF-gate intent; the resource render is still called every
  frame for every channel).

**Locked-test authorisation:** modifying `harness_s16_gamma_visual.rs` (A4/A7)
AND `harness_t7_10_b1_space_toggle.rs` (A7) is EXPLICITLY authorised — both only
retarget GDScript-source lookups of the renamed function
`_render_resource_sources` → `_update_resource_markers`; every assertion's intent
(marker is Polygon2D from SOURCE_KIND_COLORS, reads get_resource_snapshot, called
per-frame outside the overlay gate) is preserved. NO new locale keys, NO new
GDScript files, NO backend resource-logic changes.

---

## Section 3: How to Implement

1. `ResourceSnapshotRow { x, y, kind, amount, max }`; in
   `collect_resource_snapshot` iterate `*_tiles.iter()` for `(key, amount)` and
   `max = *_source_max.get(key).unwrap_or(amount).max(1)`. Sort key
   `(kind, x, y)` unchanged (amount/max not in the sort).
2. `resource_rows_amounts` mirrors `resource_rows_split` for amounts/maxes in
   the same row order; `resource_rows_to_dict` sets `amounts`/`maxes`.
3. GDScript `_update_resource_markers`: per-frame, key=(x<<16|y<<8|kind),
   create Polygon2D once per key, update position + scale + color.a, reap keys
   not in `seen`. `ratio = clampf(amount / maxi(max,1), 0, 1)`.

---

## Section 4: Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|--------|--------------|------|------------|
| T1 | FFI amount/max + companion split + dict | 🔴 DIRECT | — |
| T2 | GDScript per-frame markers (scale/alpha) | 🔴 DIRECT | T1 |
| T3 | New harness + s16_gamma A4/A7 retarget | 🔴 DIRECT | T1,T2 |

Already implemented on the working tree — Generator verifies/no-ops; Evaluator
reviews the diff. DIRECT throughout (FFI + matching renderer, small).

---

## Section 5: Localization Checklist

No new localization keys (markers are non-text visuals).

---

## Section 6: Verification & Notion

- Gate: `cargo test --workspace` (release acceptable for slow tests) +
  `cargo clippy --workspace --all-targets -- -D warnings` clean.
- New harness `harness_show_resource_depletion` 5/5 PASS; `harness_s16_alpha0`
  16/16 (split/snapshot contract); `harness_s16_gamma_visual` 11/11 (A4/A7
  retargeted); `harness_p14_gamma_click_inspector` green (unaffected).
- Visual Verify: markers present; depletion/regen requires a windowed run to
  confirm scale/alpha animate (VLM single-frame cannot animate-confirm).
- No Notion page update required.
