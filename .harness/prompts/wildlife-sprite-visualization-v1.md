# wildlife-sprite-visualization-v1: Wildlife Rendering Infrastructure

## Feature Description

Adds visual representation of wildlife entities (Wolf/Bear/Boar) to the game renderer.
Prior to A4, wildlife wandered/attacked/fled entirely behind the scenes — invisible.
This feature exposes them via a new `WildlifeSnapshot` 24-byte binary protocol and
Sprite2D-per-entity rendering in `entity_renderer.gd`.

Split into two phases:
- **This commit**: rendering infrastructure + placeholder PNGs (32×32 RGBA geometric shapes)
- **Future session**: real ComfyUI pixel art replaces placeholders (drop-in PNG swap)

## Architecture

### WildlifeSnapshot (sim-engine/frame_snapshot.rs)

```rust
#[repr(C, packed)]
pub struct WildlifeSnapshot {
    pub entity_id: u32,      // 4
    pub x: f32,              // 4
    pub y: f32,              // 4
    pub vel_x: f32,          // 4
    pub vel_y: f32,          // 4
    pub kind: u8,            // 1  (0=Wolf, 1=Bear, 2=Boar)
    pub hp_normalized: u8,   // 1  (current_hp / max_hp × 255)
    pub alive: u8,           // 1  (1=alive, 0=dead)
    pub _reserved: u8,       // 1
}
// compile-time: size_of == 24
```

`build_wildlife_snapshots(world: &World) -> Vec<WildlifeSnapshot>` collects all
`(&Wildlife, &Position)` entities each tick.

### SimBridge (sim-bridge/lib.rs)

New `#[func] fn get_wildlife_snapshots(&self) -> PackedByteArray` builds snapshots
on-demand from `state.engine.world()` and serializes to bytes.

### entity_renderer.gd

- `WILDLIFE_TEXTURE_PATHS`: 3-element array pointing to `assets/sprites/wildlife/*.png`
- `_load_wildlife_textures()`: called from `_ready()`, loads PNG textures
- `_decode_wildlife_snapshot(bytes, offset)`: decodes one 24-byte record
- `_update_wildlife_sprites()`: called from `_process` when `alpha_or_tick_changed`,
  fetches `SimBridge.get_wildlife_snapshots()`, updates Sprite2D pool per entity
- HP modulate: `Color(1.0, hp_ratio*0.5+0.5, hp_ratio*0.5+0.5, 1.0)` (low HP = red tint)
- vel_x flip: `flip_h = true` when moving left

### Placeholder PNGs

`tools/scripts/generate_wildlife_placeholder.py` (PIL) generates 32×32 RGBA:
- wolf.png: gray trapezoid with ears
- bear.png: brown circles with ears
- boar.png: pink trapezoid with tusks

## Files Changed

- `rust/crates/sim-engine/src/frame_snapshot.rs`: `WildlifeSnapshot` struct + `build_wildlife_snapshots()`
- `rust/crates/sim-engine/src/lib.rs`: re-export `WildlifeSnapshot`, `build_wildlife_snapshots`
- `rust/crates/sim-bridge/src/lib.rs`: `get_wildlife_snapshots` `#[func]`
- `scripts/ui/renderers/entity_renderer.gd`: wildlife vars + 3 new functions + `_process` hook
- `tools/scripts/generate_wildlife_placeholder.py`: PIL placeholder generator
- `assets/sprites/wildlife/wolf.png`, `bear.png`, `boar.png`: generated placeholders
- `rust/crates/sim-test/src/main.rs`: 6 harness tests in `mod harness_a4_wildlife_sprite_viz`
- `docs/sprite_session_handoff_wildlife_a4.md`: next-session handoff

## Harness Tests (6)

1. `harness_a4_wildlife_snapshot_size` — `size_of::<WildlifeSnapshot>() == 24` (Type A)
2. `harness_a4_build_wildlife_snapshots_count` — snapshot count matches entity count after 5 ticks (Type A)
3. `harness_a4_wildlife_kind_encoding` — Wolf=0, Bear=1, Boar=2 present after spawn (Type A)
4. `harness_a4_wildlife_hp_normalization` — full HP=255, half HP≈127 (Type A)
5. `harness_a4_wildlife_position_bounds` — all positions within [0,256) (Type A)
6. `harness_a4_placeholder_pngs_exist` — 3 PNG files exist on disk (Type A)

## Crates: sim-engine, sim-bridge, sim-test
