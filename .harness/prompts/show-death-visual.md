# Show Death Visual (Visualization D — final)

## Section 1: Implementation Intent

**Problem:** When an agent dies (starvation / dehydration / combat) it is
`despawn`ed instantly via `despawn_agent` (sim-systems survival/mod.rs). The
agent renderer is a single `MultiMeshInstance2D` (no per-agent node), so a death
just makes a dot vanish — the player never sees WHO died, WHERE, or WHY. This is
the missing 4th visualization. Together with A (resource depletion markers) and
B (per-agent need bars) it completes the on-screen causal chain
**resource depletion → starvation → death**.

**Approach:** Death is an instantaneous event, not persistent state, so it needs
a short-lived "recent deaths" buffer that the renderer can read and fade out.
`AgentDied` already exists in the causal log, but the causal log is an 8-slot
per-tile ring that evicts quickly — unreliable for "show every recent death".
So add a dedicated `recent_deaths: Vec<RecentDeath>` on `SimResources`, pushed in
`despawn_agent` (the single shared death helper) and pruned each tick once an
entry ages past a retain window. A new `get_recent_deaths()` FFI returns the
buffer + the current tick; a new `death_viz_renderer.gd` (mirroring
`need_bar_renderer.gd`'s standalone-overlay pattern) draws a fading,
reason-coloured marker at each death tile.

**Why a dedicated buffer (not the causal log):** ring eviction would silently
drop deaths; the buffer is display-only, bounded by `retain` (pruned every tick),
and never read by simulation logic — so it cannot affect determinism. `SimResources`
is NOT serde-serialized (no derive), so the new field needs no serde attribute and
does not enter any save/load or lockstep byte comparison.

**Tradeoff:** an extra `Vec` push per death (deaths are rare — negligible) and a
per-tick `retain` over a small buffer (bounded by deaths within the retain
window). No change to death logic itself.

## Section 2: What to Build

### Rust — sim-core (`rust/crates/sim-core/src/causal/event.rs`)
- Add `DeathReason::as_u8(&self) -> u8` mirroring the existing `as_str`:
  `Starvation => 0`, `Dehydration => 1`, `Combat => 2`. Doc comment required.

### Rust — sim-engine (`rust/crates/sim-engine/src/lib.rs`)
- New pub struct `RecentDeath { pub x: u32, pub y: u32, pub reason: DeathReason, pub tick: u64 }`
  with `#[derive(Debug, Clone, Copy)]` and a doc comment. Import `DeathReason`
  from sim-core.
- New `pub const RECENT_DEATH_RETAIN_TICKS: u64 = 120;` (doc: must be ≥ the
  renderer's FADE_TICKS=90 so the fade window always has data; 120 gives margin).
- Add field `pub recent_deaths: Vec<RecentDeath>` to `SimResources`, initialised
  to `Vec::new()` wherever `SimResources` is constructed (default-empty).
- In `SimEngine::tick()`, after `current_tick` is refreshed to `tick`, prune:
  `self.resources.recent_deaths.retain(|d| tick.saturating_sub(d.tick) < RECENT_DEATH_RETAIN_TICKS);`
  (order-independent → deterministic).

### Rust — sim-systems (`rust/crates/sim-systems/src/runtime/survival/mod.rs`)
- In `despawn_agent`, AFTER the existing `causal_log.push(... AgentDied ...)`
  (keep that untouched), also push to the buffer:
  `resources.recent_deaths.push(RecentDeath { x: position.0, y: position.1, reason, tick });`
  Import `RecentDeath` from sim-engine (already imports `SimResources` from sim_engine).

### Rust — sim-bridge (`rust/crates/sim-bridge/src/ffi/world_node.rs`)
Follow the existing Bridge Identity Contract pattern (see `get_resource_snapshot`
+ `collect_resource_snapshot` + `resource_rows_to_dict` + `resource_rows_split`):
- View struct `RecentDeathRow { x: i32, y: i32, reason_u8: i32, tick: i64 }`.
- `pub fn collect_recent_deaths(resources: &SimResources) -> Vec<RecentDeathRow>`
  — maps each `RecentDeath` (preserving buffer order) to a row
  (`reason_u8 = d.reason.as_u8() as i32`).
- `recent_death_rows_to_dict(rows: &[RecentDeathRow], current_tick: i64) -> VarDictionary`
  with keys: `xs` (PackedInt32Array), `ys` (PackedInt32Array),
  `reasons` (PackedInt32Array), `ticks` (PackedInt64Array), `current_tick` (i64).
  All four arrays equal length.
- `recent_death_rows_split(rows: &[RecentDeathRow]) -> (Vec<i32>, Vec<i32>, Vec<i32>, Vec<i64>)`
  for sim-test direct exercise.
- `#[func] fn get_recent_deaths(&self) -> VarDictionary` whose body forwards ONLY to
  `recent_death_rows_to_dict(&collect_recent_deaths(&self.engine.resources), self.engine.resources.current_tick as i64)`.

### GDScript — new `scripts/ui/death_viz_renderer.gd`
- `extends Node2D`. Mirror `need_bar_renderer.gd`'s structure and its tile→pixel
  coordinate convention (`TILE_SIZE`, `SPRITE_ORIGIN_X`, `SPRITE_ORIGIN_Y`).
- `const FADE_TICKS := 90.0` (≈3 s at 30 TPS). `const Z_DEATH := 8` (above
  NeedBarRenderer's 7).
- Reason colours (locked): `Starvation` (0) brown `Color(0.45,0.30,0.15)`,
  `Dehydration` (1) blue `Color(0.20,0.45,0.95)`, `Combat` (2) red
  `Color(0.95,0.15,0.10)`.
- `_process`: read `get_recent_deaths()`; for each death compute
  `age = current_tick - tick`; skip if `age >= FADE_TICKS` or negative; else draw
  a marker (an "X" of two strokes, or an expanding ring) at the death tile with
  `alpha = 1.0 - age/FADE_TICKS` (fade-out) and the reason colour. Marker large
  enough to read (≈ TILE_SIZE). `queue_redraw()` each frame; draw in `_draw()`.
- Read-only; never mutates sim state. Type-guard all FFI reads
  (`is Dictionary`, `is PackedInt32Array`, `is PackedInt64Array`) like need_bar.

### Scene — `scenes/main.tscn`
- `load_steps` 12 → 13. Add `ext_resource` (id e.g. `13_deathviz`) for
  `res://scripts/ui/death_viz_renderer.gd`. Add node
  `[node name="DeathVizRenderer" type="Node2D" parent="."]` with the script.

### Authorized locked-harness retargets (load_steps 12 → 13)
These three EXISTING harnesses pin `load_steps == 12` and MUST be retargeted to
13 (intent preserved — exact-match invariant, only the constant bumped). This is
explicitly authorized scope:
- `rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs` — `a17` `== 12` → `== 13`.
- `rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs` — `a12` `== 12` → `== 13`.
- `rust/crates/sim-test/tests/harness_show_agent_needs_bars.rs` — `A6` bump its
  `LOAD_STEPS_BEFORE` and `EXT_COUNT_BEFORE` baseline consts 11 → 12 so the
  `baseline+1` assertion resolves to 13 (the +1-additive invariant is preserved,
  now describing the 12→13 transition). Update its comment accordingly.

### New harness `rust/crates/sim-test/tests/harness_show_death_visual.rs` (see §3).

**Scope boundary — DO NOT:** change death/despawn LOGIC (StarvationSystem,
CombatSystem, BodyHealth, the existing `causal_log` AgentDied push); add corpse
persistence; touch other renderers/panels; alter `CausalEventView`; add locale
keys (no new user-visible text).

## Section 3: How to Implement

1. sim-core: add `DeathReason::as_u8`.
2. sim-engine: add `RecentDeath` struct + `RECENT_DEATH_RETAIN_TICKS` + the
   `recent_deaths` field (init empty at every `SimResources` construction site) +
   the `retain` prune in `SimEngine::tick()` right after `current_tick` is set.
3. sim-systems: one `recent_deaths.push(...)` line in `despawn_agent` after the
   causal_log push. Both pushes happen — buffer is additive, causal log unchanged.
4. sim-bridge: add the view struct, collector, to_dict, split, and `#[func]`.
5. GDScript renderer + main.tscn wiring.
6. Retarget the three authorized locked load_steps harnesses to 13.
7. Build dylib (`cargo build -p sim-bridge`) so Godot binds the new `#[func]`.

**Harness assertions** (`harness_show_death_visual.rs`):
- A1 (Type A/C): build a `SimEngine`, call `despawn_agent` (or run StarvationSystem
  to a death) and assert `resources.recent_deaths` gains an entry with the exact
  `x`, `y`, `reason`, `tick`.
- A2 (Type A): `collect_recent_deaths` / `recent_death_rows_split` returns rows whose
  `x`/`y`/`reason_u8`/`tick` match the buffer (live mirror, bit-exact).
- A3 (Type C — prune): push a death at tick T, advance the engine past
  `RECENT_DEATH_RETAIN_TICKS`, assert the aged entry is removed (buffer bounded,
  no unbounded growth). Also assert a fresh death within the window survives.
- A4 (Type A): `DeathReason::as_u8` mapping — Starvation→0, Dehydration→1, Combat→2;
  and rows carry the right `reason_u8` per reason.
- A5 (Type A): `death_viz_renderer.gd` exists, `extends Node2D`, calls
  `get_recent_deaths`, declares `FADE_TICKS`, all three reason colours, and
  `Z_DEATH` above need-bar's 7.
- A6 (Type D): `main.tscn` additive — `load_steps` 12→13, ext_resource count +1,
  `DeathVizRenderer` node present (mirror viz-B A6 style).
- A7 (Type A): `get_recent_deaths` `#[func]` present and Bridge-Identity (body
  forwards only to collector + to_dict).
- A8 (Type A — regression): `despawn_agent` STILL pushes `CausalEvent::AgentDied`
  to `causal_log` (the new buffer is additive, the chronicle path is intact).

Use `make_stage1_engine`-style setup; for controlled deaths pin a need at
SATURATION (see `harness_starvation_death.rs`) or call `despawn_agent` directly.

## Section 4: Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|--------|--------------|------|------------|
| T1 | sim-core `DeathReason::as_u8` | 🟢 DISPATCH | — |
| T2 | sim-engine buffer + retain + struct | 🔴 DIRECT | T1 (shared resource struct) |
| T3 | sim-systems despawn_agent push | 🔴 DIRECT | T2 |
| T4 | sim-bridge FFI collector/dict/split/#[func] | 🟢 DISPATCH | T2 |
| T5 | death_viz_renderer.gd + main.tscn | 🟢 DISPATCH | T4 |
| T6 | locked load_steps harness retargets (3) | 🟢 DISPATCH | T5 |
| T7 | harness_show_death_visual.rs | 🟢 DISPATCH | T1–T5 |

DIRECT for T2/T3: shared `SimResources` struct change + one-line wiring into the
shared death helper (interface change, <50 lines).

## Section 5: Localization Checklist

No new localization keys. (Markers are colour/shape only; no user-visible text.)

## Section 6: Verification & Notion

```bash
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail
cd rust && cargo test -p sim-test --test harness_show_death_visual -- --nocapture
cd rust && cargo build -p sim-bridge 2>&1 | tail -3   # dylib for Godot FFI
```

Expected: workspace green; `harness_show_death_visual` A1–A8 pass; the three
retargeted load_steps harnesses pass at 13; clippy clean; dylib rebuilt.

Windowed confirmation (manual, after merge): run a resource-scarce sim — an agent
starving (red need bar, viz-B) should, on death, leave a fading reason-coloured
marker at its tile (brown=starvation, blue=dehydration, red=combat), visible a few
seconds then gone. Completes the A+B+D causal chain.

Notion: V7 visualization tracking — D complete; all four visualizations shipped.
