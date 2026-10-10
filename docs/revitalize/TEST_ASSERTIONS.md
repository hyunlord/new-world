# Exact baseline and replacement assertion contracts

Source inventory only, not test execution evidence. Macro text is exact source without its trailing semicolon.

Baseline `d409e9b941ccc868747e4eb63953eafa6831bff7`; 230 re-pinned occurrences (224 macro-text changes, 6 input-scope changes); 113 current function contracts.

## F001

Current `rust/crates/sim-test/tests/harness_b1_zoom_default_raise.rs:198` / `harness_b1_a1_zoom_default_equals_five`. File SHA256 `3dbf0e03b9fb2ac01b5439a80da98cb4d5c125f5d9b78fe557dab52f08b9e459`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 205:

```rust
assert_eq!(
        no_ws(&def_rhs),
        "Vector2(4.0,4.0)",
        "a1.2: ZOOM_DEFAULT must equal Vector2(4.0,4.0) (set to 4.0 by Revitalize Part 4); got `{def_rhs}`"
    )
```

## F002

Current `rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs:388` / `harness_g_a10_zoom_min_and_default_preserved`. File SHA256 `a3b093d1058ca954e817d07c47d122bde7c511a0d6d498b67bf017532162ba0c`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 394:

```rust
assert_eq!(
        no_ws(&min_rhs),
        "Vector2(0.5,0.5)",
        "a10.3: ZOOM_MIN must equal Vector2(0.5,0.5) (Phase 12-α invariant); got `{min_rhs}`"
    )
```

Current line 399:

```rust
assert_eq!(
        no_ws(&def_rhs),
        "Vector2(4.0,4.0)",
        "a10.4: ZOOM_DEFAULT must equal Vector2(4.0,4.0) (set to 4.0 by Revitalize Part 4); got `{def_rhs}`"
    )
```

## F003

Current `rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs:510` / `harness_g_a15_existing_p14_zeta_regression_literal_updated`. File SHA256 `a3b093d1058ca954e817d07c47d122bde7c511a0d6d498b67bf017532162ba0c`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 516:

```rust
assert!(
        compact.contains("Vector2(8.0,8.0)"),
        "a15.1: harness_p14_zeta_zoom_adaptive.rs must contain Vector2(8.0,8.0)"
    )
```

Current line 520:

```rust
assert!(
        compact.contains("Vector2(0.5,0.5)"),
        "a15.2: harness_p14_zeta_zoom_adaptive.rs must retain Vector2(0.5,0.5)"
    )
```

Current line 524:

```rust
assert!(
        compact.contains("Vector2(4.0,4.0)"),
        "a15.3: harness_p14_zeta_zoom_adaptive.rs must carry Vector2(4.0,4.0) \
         (ZOOM_DEFAULT set to 4.0 by Revitalize Part 4)"
    )
```

Current line 529:

```rust
assert!(
        !compact.contains("Vector2(5.0,5.0)"),
        "a15.4: harness_p14_zeta_zoom_adaptive.rs must NOT still require Vector2(5.0,5.0)"
    )
```

## F004

Current `rust/crates/sim-test/tests/harness_h_phase_a_input_camera_color.rs:430` / `harness_h_a9_zoom_invariants_unchanged`. File SHA256 `b7147ef54304b141794084d620b7ff16f2229ee5a71bf6e06a3f60eee3b5ebbe`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 438:

```rust
assert_eq!(
        no_ws(&min_rhs),
        "Vector2(0.5,0.5)",
        "a9.4: ZOOM_MIN must equal Vector2(0.5,0.5); got `{min_rhs}`"
    )
```

Current line 443:

```rust
assert_eq!(
        no_ws(&max_rhs),
        "Vector2(8.0,8.0)",
        "a9.5: ZOOM_MAX must equal Vector2(8.0,8.0); got `{max_rhs}`"
    )
```

Current line 448:

```rust
assert_eq!(
        no_ws(&def_rhs),
        "Vector2(4.0,4.0)",
        "a9.6: ZOOM_DEFAULT must equal Vector2(4.0,4.0) (set to 4.0 by Revitalize Part 4); got `{def_rhs}`"
    )
```

## F005

Current `rust/crates/sim-test/tests/harness_p11_alpha_agent_renderer.rs:612` / `harness_p11_alpha_shader_modulates_palette_output`. File SHA256 `cc2d97de44ad91a254e4bd2a3935f8cc77c1549c263449dc899da826c0897e59`. Goal 9.8 (agent interpolation/shadows/outline); 9.7 for camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 625:

```rust
assert!(
        multiplies,
        "A17.1: palette_swap.gdshader must multiply `palette_color.rgb * v_tint.rgb` \
         so the per-instance state tint composites over the palette output"
    )
```

Current line 641:

```rust
assert!(
        multiply_pos > alpha_guard_pos,
        "A17.4: the palette_color.rgb * v_tint.rgb multiply must appear \
         after the `tex.a == 0.0` alpha guard so transparent fragments \
         are not tinted (multiply_pos={multiply_pos}, alpha_guard_pos={alpha_guard_pos})"
    )
```

Current line 647:

```rust
assert!(stripped.contains("neighbor_alpha > 0.5"))
```

Current line 648:

```rust
assert!(stripped.contains("vec4(vec3(20.0 / 255.0), 0.9)"))
```

Current line 649:

```rust
assert_eq!(stripped.matches("TEXTURE_PIXEL_SIZE").count(), 4)
```

## F006

Current `rust/crates/sim-test/tests/harness_p12_alpha_camera_zoom.rs:207` / `harness_p12_alpha_a5_zoom_default_constant`. File SHA256 `c70d79bcf6a8138e4b94b3cc2ce509132d99da3a5cc08ac7ab21b8edd48beed7`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 230:

```rust
assert!(
        matched,
        "A5: ZOOM_DEFAULT must be declared with `Vector2(4.0, 4.0)` \
         (whitespace tolerant; set to 4.0× by Revitalize Part 4). Line: `{line}`"
    )
```

## F007

Current `rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs:333` / `harness_p12_beta2_a10_renderer_calls_ffi_snapshot_method`. File SHA256 `91e284573e0916860bfabf433ee238f684999405f2343ddd295659442289b10e`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 335:

```rust
assert!(src.contains("get_construction_snapshot()"))
```

Current line 336:

```rust
assert!(src.contains("_reap(_sites, seen)"))
```

Current line 337:

```rust
assert!(src.contains("stale.queue_free()"))
```

## F008

Current `rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs:342` / `harness_p12_beta2_a11_reaper_loop_frees_despawned_sprites`. File SHA256 `91e284573e0916860bfabf433ee238f684999405f2343ddd295659442289b10e`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 344:

```rust
assert!(src.contains("get_construction_snapshot()"))
```

Current line 345:

```rust
assert!(src.contains("_reap(_sites, seen)"))
```

Current line 346:

```rust
assert!(src.contains("stale.queue_free()"))
```

## F009

Current `rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs:396` / `harness_p12_beta2_a14_phase12_alpha_and_beta1_invariants_preserved`. File SHA256 `91e284573e0916860bfabf433ee238f684999405f2343ddd295659442289b10e`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 404:

```rust
assert!(
        zoom_ok,
        "A14.1: main.tscn must preserve Camera2D `zoom = Vector2(2, 2)` (Phase 12-α)"
    )
```

Current line 410:

```rust
assert!(
        camera_script_ok,
        "A14.2: main.tscn must preserve camera_controller.gd ExtResource attachment (Phase 12-α)"
    )
```

Current line 418:

```rust
assert!(src.contains("get_terrain_snapshot()"))
```

Current line 419:

```rust
assert!(src.contains("TileSetAtlasSource.new()"))
```

Current line 420:

```rust
assert!(src.contains("_layer.set_cell"))
```

Current line 421:

```rust
assert!(!src.contains("RandomNumberGenerator"))
```

Current line 422:

```rust
assert!(!src.contains("world_terrain.tres"))
```

Current line 424:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 425:

```rust
assert!(src.contains("_building_textures[kind]"))
```

Current line 426:

```rust
assert!(src.contains("sprite.position = _tile_center(xs[i], ys[i])"))
```

Current line 441:

```rust
assert!(
        (val - 0.65).abs() < 1e-9,
        "A14.7: OVERLAY_ALPHA must equal exactly 0.65; got {val}"
    )
```

## F010

Current `rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs:315` / `harness_p12_beta2_a8_renderer_declares_construction_sprite_path_constant`. File SHA256 `91e284573e0916860bfabf433ee238f684999405f2343ddd295659442289b10e`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 317:

```rust
assert!(src.contains("res://assets/sprites/furniture/lean_to/1.png"))
```

Current line 318:

```rust
assert!(src.contains("get_construction_snapshot()"))
```

Current line 319:

```rust
assert!(src.contains("sprite.modulate.a = 0.35"))
```

## F011

Current `rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs:324` / `harness_p12_beta2_a9_alpha_remap_constants_have_exact_values`. File SHA256 `91e284573e0916860bfabf433ee238f684999405f2343ddd295659442289b10e`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 326:

```rust
assert!(src.contains("float(progresses[i]) / float(maxi(required[i], 1))"))
```

Current line 327:

```rust
assert!(src.contains("Vector2(14.0 * ratio, 2)"))
```

Current line 328:

```rust
assert!(src.contains("Color(\"e0c060\")"))
```

## F012

Current `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:189` / `harness_p12_beta_a10_bootstrap_building_sprite_loaded`. File SHA256 `3725554e570e43696f4131b7ab283d7823f91d12b20112da5b2733c835ee4459`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 191:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 192:

```rust
assert!(src.contains("_building_textures[kind]"))
```

Current line 193:

```rust
assert!(src.contains("sprite.position = _tile_center(xs[i], ys[i])"))
```

## F013

Current `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:198` / `harness_p12_beta_a11_bootstrap_sprite_positioned_at_bootstrap_tile`. File SHA256 `3725554e570e43696f4131b7ab283d7823f91d12b20112da5b2733c835ee4459`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 200:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 201:

```rust
assert!(src.contains("_building_textures[kind]"))
```

Current line 202:

```rust
assert!(src.contains("sprite.position = _tile_center(xs[i], ys[i])"))
```

## F014

Current `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:207` / `harness_p12_beta_a12_z_order_constants_match_spec`. File SHA256 `3725554e570e43696f4131b7ab283d7823f91d12b20112da5b2733c835ee4459`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 209:

```rust
assert!(src.contains("z_index = 0"))
```

Current line 211:

```rust
assert!(src.contains("z_index = 5"))
```

Current line 213:

```rust
assert!(src.contains("sprite.z_index = Z_OVERLAY"))
```

Current line 214:

```rust
assert!(src.contains("const Z_OVERLAY: int = 10"))
```

## F015

Current `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:219` / `harness_p12_beta_a13_z_order_constants_are_assigned_to_nodes`. File SHA256 `3725554e570e43696f4131b7ab283d7823f91d12b20112da5b2733c835ee4459`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 221:

```rust
assert!(src.contains("z_index = 0"))
```

Current line 223:

```rust
assert!(src.contains("z_index = 5"))
```

Current line 225:

```rust
assert!(src.contains("sprite.z_index = Z_OVERLAY"))
```

Current line 226:

```rust
assert!(src.contains("const Z_OVERLAY: int = 10"))
```

## F016

Current `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:147` / `harness_p12_beta_a6_world_renderer_loads_terrain_tileset`. File SHA256 `3725554e570e43696f4131b7ab283d7823f91d12b20112da5b2733c835ee4459`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 149:

```rust
assert!(src.contains("terrain_v8.png"))
```

Current line 150:

```rust
assert!(src.contains("range(5)"))
```

Current line 151:

```rust
assert!(src.contains("range(4)"))
```

Current line 152:

```rust
assert!(src.contains("atlas.create_tile"))
```

Current line 153:

```rust
assert!(src.contains("tiles.add_source(atlas, 0)"))
```

## F017

Current `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:158` / `harness_p12_beta_a7_world_renderer_instantiates_tilemaplayer_with_tileset`. File SHA256 `3725554e570e43696f4131b7ab283d7823f91d12b20112da5b2733c835ee4459`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 160:

```rust
assert!(src.contains("get_terrain_snapshot()"))
```

Current line 161:

```rust
assert!(src.contains("TileSetAtlasSource.new()"))
```

Current line 162:

```rust
assert!(src.contains("_layer.set_cell"))
```

Current line 163:

```rust
assert!(!src.contains("RandomNumberGenerator"))
```

Current line 164:

```rust
assert!(!src.contains("world_terrain.tres"))
```

## F018

Current `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:169` / `harness_p12_beta_a8_world_renderer_populates_all_grid_cells`. File SHA256 `3725554e570e43696f4131b7ab283d7823f91d12b20112da5b2733c835ee4459`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 171:

```rust
assert!(src.contains("_terrain[y * _width + x]"))
```

Current line 172:

```rust
assert!(src.contains("TERRAIN_ROWS.get(code, 0)"))
```

Current line 173:

```rust
assert!(src.contains("2: 3, 3: 2"))
```

## F019

Current `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:178` / `harness_p12_beta_a9_terrain_placement_is_deterministic`. File SHA256 `3725554e570e43696f4131b7ab283d7823f91d12b20112da5b2733c835ee4459`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 180:

```rust
assert!(src.contains("73856093"))
```

Current line 181:

```rust
assert!(src.contains("19349663"))
```

Current line 182:

```rust
assert!(src.contains("0x7fffffff"))
```

Current line 183:

```rust
assert!(src.contains("% 4"))
```

Current line 184:

```rust
assert!(!src.contains("RandomNumberGenerator"))
```

## F020

Current `rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs:396` / `harness_p12_gamma_a10_world_renderer_polls_ffi_and_reaps_stale_sprites`. File SHA256 `57387f9d0b1a20048d831e656db04bded1f446c8669939736da1f242dc164b19`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 398:

```rust
assert!(src.contains("get_settlement_snapshot()"))
```

Current line 399:

```rust
assert!(src.contains("\"formation_xs\""))
```

Current line 400:

```rust
assert!(src.contains("\"formation_ys\""))
```

Current line 401:

```rust
assert!(src.contains("_tile_center(nearest.x, nearest.y)"))
```

Current line 402:

```rust
assert!(src.contains("_reap(_labels, seen)"))
```

## F021

Current `rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs:443` / `harness_p12_gamma_a12_phase12_alpha_beta1_beta2_invariants_preserved`. File SHA256 `57387f9d0b1a20048d831e656db04bded1f446c8669939736da1f242dc164b19`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 448:

```rust
assert!(
        zoom_ok,
        "A12.1: main.tscn must preserve Camera2D `zoom = Vector2(2, 2)` (Phase 12-α)"
    )
```

Current line 454:

```rust
assert!(
        camera_script_ok,
        "A12.2: main.tscn must preserve camera_controller.gd ExtResource attachment (Phase 12-α)"
    )
```

Current line 462:

```rust
assert!(src.contains("get_terrain_snapshot()"))
```

Current line 463:

```rust
assert!(src.contains("TileSetAtlasSource.new()"))
```

Current line 464:

```rust
assert!(src.contains("_layer.set_cell"))
```

Current line 465:

```rust
assert!(!src.contains("RandomNumberGenerator"))
```

Current line 466:

```rust
assert!(!src.contains("world_terrain.tres"))
```

Current line 468:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 469:

```rust
assert!(src.contains("_building_textures[kind]"))
```

Current line 470:

```rust
assert!(src.contains("sprite.position = _tile_center(xs[i], ys[i])"))
```

Current line 484:

```rust
assert!(
        (val - 0.65).abs() < 1e-9,
        "A12.7: OVERLAY_ALPHA must equal exactly 0.65; got {val}"
    )
```

Current line 491:

```rust
assert!(src.contains("res://assets/sprites/furniture/lean_to/1.png"))
```

Current line 492:

```rust
assert!(src.contains("get_construction_snapshot()"))
```

Current line 493:

```rust
assert!(src.contains("sprite.modulate.a = 0.35"))
```

Current line 495:

```rust
assert!(src.contains("z_index = 5"))
```

## F022

Current `rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs:572` / `harness_p12_gamma_a15_world_renderer_reads_formation_arrays`. File SHA256 `57387f9d0b1a20048d831e656db04bded1f446c8669939736da1f242dc164b19`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 574:

```rust
assert!(src.contains("get_settlement_snapshot()"))
```

Current line 575:

```rust
assert!(src.contains("\"formation_xs\""))
```

Current line 576:

```rust
assert!(src.contains("\"formation_ys\""))
```

Current line 577:

```rust
assert!(src.contains("_tile_center(nearest.x, nearest.y)"))
```

Current line 578:

```rust
assert!(src.contains("_reap(_labels, seen)"))
```

## F023

Current `rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs:387` / `harness_p12_gamma_a9_world_renderer_declares_furniture_sprite_path_and_z_constants`. File SHA256 `57387f9d0b1a20048d831e656db04bded1f446c8669939736da1f242dc164b19`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 389:

```rust
assert!(src.contains("res://assets/sprites/buildings/stockpile/1.png"))
```

Current line 390:

```rust
assert!(src.contains("kind == 2"))
```

Current line 391:

```rust
assert!(src.contains("_stockpile_positions.append"))
```

## F024

Current `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs:434` / `harness_p13_alpha_a11_phase12_beta1_terrain_tileset_preserved`. File SHA256 `2ccbda1fb2cd34730eb0933b0008b1ed44029a0737b7e7862641973b6aa08c64`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 436:

```rust
assert!(src.contains("get_terrain_snapshot()"))
```

Current line 437:

```rust
assert!(src.contains("TileSetAtlasSource.new()"))
```

Current line 438:

```rust
assert!(src.contains("_layer.set_cell"))
```

Current line 439:

```rust
assert!(!src.contains("RandomNumberGenerator"))
```

Current line 440:

```rust
assert!(!src.contains("world_terrain.tres"))
```

## F025

Current `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs:445` / `harness_p13_alpha_a12_phase12_beta2_construction_z_preserved`. File SHA256 `2ccbda1fb2cd34730eb0933b0008b1ed44029a0737b7e7862641973b6aa08c64`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 447:

```rust
assert!(src.contains("z_index = 0"))
```

Current line 449:

```rust
assert!(src.contains("z_index = 5"))
```

Current line 451:

```rust
assert!(src.contains("sprite.z_index = Z_OVERLAY"))
```

Current line 452:

```rust
assert!(src.contains("const Z_OVERLAY: int = 10"))
```

## F026

Current `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs:457` / `harness_p13_alpha_a13_phase12_gamma_furniture_z_preserved`. File SHA256 `2ccbda1fb2cd34730eb0933b0008b1ed44029a0737b7e7862641973b6aa08c64`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 459:

```rust
assert!(src.contains("get_settlement_snapshot()"))
```

Current line 460:

```rust
assert!(src.contains("\"formation_xs\""))
```

Current line 461:

```rust
assert!(src.contains("\"formation_ys\""))
```

Current line 462:

```rust
assert!(src.contains("_tile_center(nearest.x, nearest.y)"))
```

Current line 463:

```rust
assert!(src.contains("_reap(_labels, seen)"))
```

## F027

Current `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs:468` / `harness_p13_alpha_a14_three_distinct_building_sprite_paths`. File SHA256 `2ccbda1fb2cd34730eb0933b0008b1ed44029a0737b7e7862641973b6aa08c64`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 470:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 471:

```rust
assert!(src.contains("_building_textures[kind]"))
```

Current line 472:

```rust
assert!(src.contains("sprite.position = _tile_center(xs[i], ys[i])"))
```

## F028

Current `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs:194` / `harness_p13_alpha_a1_camera_controller_zoom_default_3x`. File SHA256 `2ccbda1fb2cd34730eb0933b0008b1ed44029a0737b7e7862641973b6aa08c64`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 206:

```rust
assert!(
        accepted.contains(&compact.as_str()),
        "A1: ZOOM_DEFAULT RHS must equal exactly one of Vector2(4.0,4.0) | \
         Vector2(4,4) | Vector2(4.0,4) | Vector2(4,4.0) (whitespace-stripped; \
         set to 4.0 by Revitalize Part 4); got RHS=`{rhs}` compact=`{compact}`"
    )
```

## F029

Current `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs:246` / `harness_p13_alpha_a4_bootstrap_building_path_campfire`. File SHA256 `2ccbda1fb2cd34730eb0933b0008b1ed44029a0737b7e7862641973b6aa08c64`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 248:

```rust
assert!(src.contains("res://assets/sprites/buildings/campfire/1.png"))
```

Current line 249:

```rust
assert!(src.contains("get_building_snapshot()"))
```

## F030

Current `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs:254` / `harness_p13_alpha_a5_bootstrap_no_longer_points_to_cairn`. File SHA256 `2ccbda1fb2cd34730eb0933b0008b1ed44029a0737b7e7862641973b6aa08c64`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 256:

```rust
assert!(src.contains("res://assets/sprites/furniture/lean_to/1.png"))
```

Current line 257:

```rust
assert!(src.contains("get_construction_snapshot()"))
```

Current line 258:

```rust
assert!(src.contains("sprite.modulate.a = 0.35"))
```

## F031

Current `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs:263` / `harness_p13_alpha_a6_construction_sprite_path_unchanged`. File SHA256 `2ccbda1fb2cd34730eb0933b0008b1ed44029a0737b7e7862641973b6aa08c64`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 265:

```rust
assert!(src.contains("res://assets/sprites/furniture/lean_to/1.png"))
```

Current line 266:

```rust
assert!(src.contains("get_construction_snapshot()"))
```

Current line 267:

```rust
assert!(src.contains("sprite.modulate.a = 0.35"))
```

## F032

Current `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs:272` / `harness_p13_alpha_a7_furniture_sprite_path_unchanged`. File SHA256 `2ccbda1fb2cd34730eb0933b0008b1ed44029a0737b7e7862641973b6aa08c64`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 274:

```rust
assert!(src.contains("res://assets/sprites/buildings/stockpile/1.png"))
```

Current line 275:

```rust
assert!(src.contains("kind == 2"))
```

Current line 276:

```rust
assert!(src.contains("_stockpile_positions.append"))
```

## F033

Current `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:346` / `harness_p13_beta_a11_phase13_alpha_zoom_default_3x_preserved`. File SHA256 `b46a7b40355f672dd539711a720672f0070236372ca3ee9a83784086e6d75898`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 357:

```rust
assert!(
        accepted.contains(&compact.as_str()),
        "A11: ZOOM_DEFAULT must equal Vector2(4.0, 4.0) (set to 4.0 by Revitalize Part 4); got `{rhs}`"
    )
```

## F034

Current `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:366` / `harness_p13_beta_a12_phase13_alpha_bootstrap_campfire_preserved`. File SHA256 `b46a7b40355f672dd539711a720672f0070236372ca3ee9a83784086e6d75898`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 368:

```rust
assert!(src.contains("res://assets/sprites/buildings/campfire/1.png"))
```

Current line 369:

```rust
assert!(src.contains("get_building_snapshot()"))
```

## F035

Current `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:374` / `harness_p13_beta_a13_phase12_beta1_terrain_tileset_preserved`. File SHA256 `b46a7b40355f672dd539711a720672f0070236372ca3ee9a83784086e6d75898`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 376:

```rust
assert!(src.contains("get_terrain_snapshot()"))
```

Current line 377:

```rust
assert!(src.contains("TileSetAtlasSource.new()"))
```

Current line 378:

```rust
assert!(src.contains("_layer.set_cell"))
```

Current line 379:

```rust
assert!(!src.contains("RandomNumberGenerator"))
```

Current line 380:

```rust
assert!(!src.contains("world_terrain.tres"))
```

## F036

Current `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:385` / `harness_p13_beta_a14_phase12_beta2_construction_z_5_preserved`. File SHA256 `b46a7b40355f672dd539711a720672f0070236372ca3ee9a83784086e6d75898`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 387:

```rust
assert!(src.contains("res://assets/sprites/furniture/lean_to/1.png"))
```

Current line 388:

```rust
assert!(src.contains("get_construction_snapshot()"))
```

Current line 389:

```rust
assert!(src.contains("sprite.modulate.a = 0.35"))
```

## F037

Current `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:394` / `harness_p13_beta_a15_phase12_gamma_furniture_z_4_preserved`. File SHA256 `b46a7b40355f672dd539711a720672f0070236372ca3ee9a83784086e6d75898`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 396:

```rust
assert!(src.contains("get_settlement_snapshot()"))
```

Current line 397:

```rust
assert!(src.contains("\"formation_xs\""))
```

Current line 398:

```rust
assert!(src.contains("\"formation_ys\""))
```

Current line 399:

```rust
assert!(src.contains("_tile_center(nearest.x, nearest.y)"))
```

Current line 400:

```rust
assert!(src.contains("_reap(_labels, seen)"))
```

## F038

Current `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:137` / `harness_p13_beta_a1_resource_sprite_path_constant_declared`. File SHA256 `b46a7b40355f672dd539711a720672f0070236372ca3ee9a83784086e6d75898`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 139:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

Current line 140:

```rust
assert!(src.contains("if kinds[i] != 0:"))
```

Current line 141:

```rust
assert!(src.contains("berry_bush_%d.png"))
```

Current line 142:

```rust
assert!(src.contains("_berry_textures[stage]"))
```

Current line 143:

```rust
assert!(!src.contains("SOURCE_KIND_COLORS"))
```

Current line 144:

```rust
assert!(!src.contains("Polygon2D"))
```

## F039

Current `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:149` / `harness_p13_beta_a2_z_resource_constant_equals_3`. File SHA256 `b46a7b40355f672dd539711a720672f0070236372ca3ee9a83784086e6d75898`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 151:

```rust
assert!(src.contains("z_index = 0"))
```

Current line 153:

```rust
assert!(src.contains("z_index = 5"))
```

Current line 155:

```rust
assert!(src.contains("sprite.z_index = Z_OVERLAY"))
```

Current line 156:

```rust
assert!(src.contains("const Z_OVERLAY: int = 10"))
```

## F040

Current `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:161` / `harness_p13_beta_a3_resource_count_equals_20`. File SHA256 `b46a7b40355f672dd539711a720672f0070236372ca3ee9a83784086e6d75898`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 163:

```rust
assert!(!src.contains("on_building_placed("))
```

Current line 164:

```rust
assert!(!src.contains("RESOURCE_SEED"))
```

Current line 165:

```rust
assert!(!src.contains("VILLAGE_FIXTURE"))
```

Current line 166:

```rust
assert!(!src.contains("BOOTSTRAP_"))
```

Current line 168:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 169:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

## F041

Current `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:174` / `harness_p13_beta_a4_resource_seed_constant_declared`. File SHA256 `b46a7b40355f672dd539711a720672f0070236372ca3ee9a83784086e6d75898`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 176:

```rust
assert!(!src.contains("on_building_placed("))
```

Current line 177:

```rust
assert!(!src.contains("RESOURCE_SEED"))
```

Current line 178:

```rust
assert!(!src.contains("VILLAGE_FIXTURE"))
```

Current line 179:

```rust
assert!(!src.contains("BOOTSTRAP_"))
```

Current line 181:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 182:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

## F042

Current `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:187` / `harness_p13_beta_a5_resource_placement_uses_seeded_rng`. File SHA256 `b46a7b40355f672dd539711a720672f0070236372ca3ee9a83784086e6d75898`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 189:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

Current line 190:

```rust
assert!(src.contains("if kinds[i] != 0:"))
```

Current line 191:

```rust
assert!(src.contains("berry_bush_%d.png"))
```

Current line 192:

```rust
assert!(src.contains("_berry_textures[stage]"))
```

Current line 193:

```rust
assert!(!src.contains("SOURCE_KIND_COLORS"))
```

Current line 194:

```rust
assert!(!src.contains("Polygon2D"))
```

## F043

Current `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:233` / `harness_p13_beta_a7_z_order_chain_invariant`. File SHA256 `b46a7b40355f672dd539711a720672f0070236372ca3ee9a83784086e6d75898`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 235:

```rust
assert!(src.contains("z_index = 0"))
```

Current line 237:

```rust
assert!(src.contains("z_index = 5"))
```

Current line 239:

```rust
assert!(src.contains("sprite.z_index = Z_OVERLAY"))
```

Current line 240:

```rust
assert!(src.contains("const Z_OVERLAY: int = 10"))
```

## F044

Current `rust/crates/sim-test/tests/harness_p13_delta_basic_hud.rs:610` / `harness_p13_delta_a14_phase13_alpha_default_zoom_3x_unchanged`. File SHA256 `b5298517e3f402356c329353e676ae6bcc184c4a8a2865b99e432c44a383c2eb`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 621:

```rust
assert!(
        accepted.contains(&compact.as_str()),
        "A14: ZOOM_DEFAULT must equal Vector2(4.0, 4.0) (set to 4.0 by Revitalize Part 4); got `{rhs}`"
    )
```

## F045

Current `rust/crates/sim-test/tests/harness_p13_delta_basic_hud.rs:630` / `harness_p13_delta_a15_phase13_alpha_bootstrap_campfire_unchanged`. File SHA256 `b5298517e3f402356c329353e676ae6bcc184c4a8a2865b99e432c44a383c2eb`. Goal 9.5 (simulation-driven HUD and localization).

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 632:

```rust
assert!(src.contains("res://assets/sprites/buildings/campfire/1.png"))
```

Current line 633:

```rust
assert!(src.contains("get_building_snapshot()"))
```

## F046

Current `rust/crates/sim-test/tests/harness_p13_delta_basic_hud.rs:638` / `harness_p13_delta_a16_phase13_beta_resource_constants_unchanged`. File SHA256 `b5298517e3f402356c329353e676ae6bcc184c4a8a2865b99e432c44a383c2eb`. Goal 9.5 (simulation-driven HUD and localization).

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 640:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

Current line 641:

```rust
assert!(src.contains("if kinds[i] != 0:"))
```

Current line 642:

```rust
assert!(src.contains("berry_bush_%d.png"))
```

Current line 643:

```rust
assert!(src.contains("_berry_textures[stage]"))
```

Current line 644:

```rust
assert!(!src.contains("SOURCE_KIND_COLORS"))
```

Current line 645:

```rust
assert!(!src.contains("Polygon2D"))
```

## F047

Current `rust/crates/sim-test/tests/harness_p13_delta_basic_hud.rs:198` / `harness_p13_delta_a3_hud_topbar_declares_four_label_members`. File SHA256 `b5298517e3f402356c329353e676ae6bcc184c4a8a2865b99e432c44a383c2eb`. Goal 9.5 (simulation-driven HUD and localization).

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 257:

```rust
assert!(
        missing.is_empty(),
        "A3: hud_topbar.gd must declare four Label members containing each of shelter/agent/settlement/site \
         (case-insensitive); missing={missing:?}; found Label idents={label_idents:?}"
    )
```

## F048

Current `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:299` / `harness_p13_epsilon_a11_phase13_alpha_default_zoom_3x_preserved`. File SHA256 `6e266fc5b255b4b884e026fc8da31df0cdb0fecd729fdb92d6b72e7bf5d094e3`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 311:

```rust
assert!(
        accepted.contains(&compact.as_str()),
        "A11: ZOOM_DEFAULT must equal Vector2(4.0, 4.0) (set to 4.0 by Revitalize Part 4); got `{rhs}`"
    )
```

## F049

Current `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:320` / `harness_p13_epsilon_a12_phase13_alpha_centre_bootstrap_preserved`. File SHA256 `6e266fc5b255b4b884e026fc8da31df0cdb0fecd729fdb92d6b72e7bf5d094e3`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 322:

```rust
assert!(src.contains("res://assets/sprites/buildings/campfire/1.png"))
```

Current line 323:

```rust
assert!(src.contains("get_building_snapshot()"))
```

## F050

Current `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:328` / `harness_p13_epsilon_a13_phase13_beta_resource_constants_preserved`. File SHA256 `6e266fc5b255b4b884e026fc8da31df0cdb0fecd729fdb92d6b72e7bf5d094e3`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 330:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

Current line 331:

```rust
assert!(src.contains("if kinds[i] != 0:"))
```

Current line 332:

```rust
assert!(src.contains("berry_bush_%d.png"))
```

Current line 333:

```rust
assert!(src.contains("_berry_textures[stage]"))
```

Current line 334:

```rust
assert!(!src.contains("SOURCE_KIND_COLORS"))
```

Current line 335:

```rust
assert!(!src.contains("Polygon2D"))
```

## F051

Current `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:143` / `harness_p13_epsilon_a1_bootstrap_x_left_constant_declared`. File SHA256 `6e266fc5b255b4b884e026fc8da31df0cdb0fecd729fdb92d6b72e7bf5d094e3`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 145:

```rust
assert!(!src.contains("on_building_placed("))
```

Current line 146:

```rust
assert!(!src.contains("RESOURCE_SEED"))
```

Current line 147:

```rust
assert!(!src.contains("VILLAGE_FIXTURE"))
```

Current line 148:

```rust
assert!(!src.contains("BOOTSTRAP_"))
```

Current line 150:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 151:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

## F052

Current `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:156` / `harness_p13_epsilon_a2_bootstrap_x_right_constant_declared`. File SHA256 `6e266fc5b255b4b884e026fc8da31df0cdb0fecd729fdb92d6b72e7bf5d094e3`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 158:

```rust
assert!(!src.contains("on_building_placed("))
```

Current line 159:

```rust
assert!(!src.contains("RESOURCE_SEED"))
```

Current line 160:

```rust
assert!(!src.contains("VILLAGE_FIXTURE"))
```

Current line 161:

```rust
assert!(!src.contains("BOOTSTRAP_"))
```

Current line 163:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 164:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

## F053

Current `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:169` / `harness_p13_epsilon_a3_three_on_building_placed_calls_total`. File SHA256 `6e266fc5b255b4b884e026fc8da31df0cdb0fecd729fdb92d6b72e7bf5d094e3`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 171:

```rust
assert!(!src.contains("on_building_placed("))
```

Current line 172:

```rust
assert!(!src.contains("RESOURCE_SEED"))
```

Current line 173:

```rust
assert!(!src.contains("VILLAGE_FIXTURE"))
```

Current line 174:

```rust
assert!(!src.contains("BOOTSTRAP_"))
```

Current line 176:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 177:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

## F054

Current `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:182` / `harness_p13_epsilon_a4_on_building_placed_arg_signatures`. File SHA256 `6e266fc5b255b4b884e026fc8da31df0cdb0fecd729fdb92d6b72e7bf5d094e3`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 184:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 185:

```rust
assert!(src.contains("_building_textures[kind]"))
```

Current line 186:

```rust
assert!(src.contains("sprite.position = _tile_center(xs[i], ys[i])"))
```

## F055

Current `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:191` / `harness_p13_epsilon_a5_extra_sprite_block_reuses_building_tex`. File SHA256 `6e266fc5b255b4b884e026fc8da31df0cdb0fecd729fdb92d6b72e7bf5d094e3`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 193:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 194:

```rust
assert!(src.contains("_building_textures[kind]"))
```

Current line 195:

```rust
assert!(src.contains("sprite.position = _tile_center(xs[i], ys[i])"))
```

## F056

Current `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:200` / `harness_p13_epsilon_a6_extra_sprite_z_index_is_z_building`. File SHA256 `6e266fc5b255b4b884e026fc8da31df0cdb0fecd729fdb92d6b72e7bf5d094e3`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 202:

```rust
assert!(src.contains("z_index = 0"))
```

Current line 204:

```rust
assert!(src.contains("z_index = 5"))
```

Current line 206:

```rust
assert!(src.contains("sprite.z_index = Z_OVERLAY"))
```

Current line 207:

```rust
assert!(src.contains("const Z_OVERLAY: int = 10"))
```

## F057

Current `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:212` / `harness_p13_epsilon_a7_extra_sprite_y_position_uses_bootstrap_y`. File SHA256 `6e266fc5b255b4b884e026fc8da31df0cdb0fecd729fdb92d6b72e7bf5d094e3`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 214:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 215:

```rust
assert!(src.contains("_building_textures[kind]"))
```

Current line 216:

```rust
assert!(src.contains("sprite.position = _tile_center(xs[i], ys[i])"))
```

## F058

Current `rust/crates/sim-test/tests/harness_p13_gamma_interaction_cues.rs:533` / `harness_p13_gamma_a13_phase13_alpha_zoom_default_3x_preserved`. File SHA256 `c21d48eb81cf3ec9cf08a87991e3e951587cd36b0aec2bd7bf8a109f5b2f6a64`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 544:

```rust
assert!(
        accepted.contains(&compact.as_str()),
        "A13: ZOOM_DEFAULT must equal Vector2(4.0, 4.0) (set to 4.0 by Revitalize Part 4); got `{rhs}`"
    )
```

## F059

Current `rust/crates/sim-test/tests/harness_p13_gamma_interaction_cues.rs:553` / `harness_p13_gamma_a14_phase13_alpha_bootstrap_campfire_preserved`. File SHA256 `c21d48eb81cf3ec9cf08a87991e3e951587cd36b0aec2bd7bf8a109f5b2f6a64`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 555:

```rust
assert!(src.contains("res://assets/sprites/buildings/campfire/1.png"))
```

Current line 556:

```rust
assert!(src.contains("get_building_snapshot()"))
```

## F060

Current `rust/crates/sim-test/tests/harness_p13_gamma_interaction_cues.rs:561` / `harness_p13_gamma_a15_phase13_beta_resource_constants_preserved`. File SHA256 `c21d48eb81cf3ec9cf08a87991e3e951587cd36b0aec2bd7bf8a109f5b2f6a64`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 563:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

Current line 564:

```rust
assert!(src.contains("if kinds[i] != 0:"))
```

Current line 565:

```rust
assert!(src.contains("berry_bush_%d.png"))
```

Current line 566:

```rust
assert!(src.contains("_berry_textures[stage]"))
```

Current line 567:

```rust
assert!(!src.contains("SOURCE_KIND_COLORS"))
```

Current line 568:

```rust
assert!(!src.contains("Polygon2D"))
```

## F061

Current `rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs:489` / `harness_p14_alpha_a15_camera_zoom_invariants_preserved`. File SHA256 `fc42014373168dd843773a51f22071a0ef902f82812cc4c73bd8d397431df2e8`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 495:

```rust
assert_eq!(
        no_ws(&zoom_min),
        "Vector2(0.5,0.5)",
        "A15.4: ZOOM_MIN must equal Vector2(0.5, 0.5); got `{zoom_min}`"
    )
```

Current line 500:

```rust
assert_eq!(
        no_ws(&zoom_max),
        "Vector2(8.0,8.0)",
        "A15.5: ZOOM_MAX must equal Vector2(8.0, 8.0) (raised from 4.0 in G Phase A); got `{zoom_max}`"
    )
```

Current line 512:

```rust
assert!(
        accepted.contains(&default_compact.as_str()),
        "A15.6: ZOOM_DEFAULT must equal Vector2(4.0, 4.0) (set to 4.0 by Revitalize Part 4); got `{zoom_default}`"
    )
```

## F062

Current `rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs:521` / `harness_p14_alpha_a16_phase13_bootstrap_three_campfires_preserved`. File SHA256 `fc42014373168dd843773a51f22071a0ef902f82812cc4c73bd8d397431df2e8`. Goal 9.8 (agent interpolation/shadows/outline); 9.7 for camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 523:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 524:

```rust
assert!(src.contains("_building_textures[kind]"))
```

Current line 525:

```rust
assert!(src.contains("sprite.position = _tile_center(xs[i], ys[i])"))
```

## F063

Current `rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs:530` / `harness_p14_alpha_a17_phase13_beta_resource_constants_preserved`. File SHA256 `fc42014373168dd843773a51f22071a0ef902f82812cc4c73bd8d397431df2e8`. Goal 9.8 (agent interpolation/shadows/outline); 9.7 for camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 532:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

Current line 533:

```rust
assert!(src.contains("if kinds[i] != 0:"))
```

Current line 534:

```rust
assert!(src.contains("berry_bush_%d.png"))
```

Current line 535:

```rust
assert!(src.contains("_berry_textures[stage]"))
```

Current line 536:

```rust
assert!(!src.contains("SOURCE_KIND_COLORS"))
```

Current line 537:

```rust
assert!(!src.contains("Polygon2D"))
```

## F064

Current `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:260` / `harness_p14_beta_a10_water_modulate_branch_present`. File SHA256 `a250aea532cdcebbca2f471a5e1e0a6e15594a106668e040d7ecc0e8e6aae665`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 262:

```rust
assert!(src.contains("get_terrain_snapshot()"))
```

Current line 263:

```rust
assert!(src.contains("TileSetAtlasSource.new()"))
```

Current line 264:

```rust
assert!(src.contains("_layer.set_cell"))
```

Current line 265:

```rust
assert!(!src.contains("RandomNumberGenerator"))
```

Current line 266:

```rust
assert!(!src.contains("world_terrain.tres"))
```

## F065

Current `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:271` / `harness_p14_beta_a11_village_fixture_paths_has_exactly_four_literals`. File SHA256 `a250aea532cdcebbca2f471a5e1e0a6e15594a106668e040d7ecc0e8e6aae665`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 273:

```rust
assert!(!src.contains("on_building_placed("))
```

Current line 274:

```rust
assert!(!src.contains("RESOURCE_SEED"))
```

Current line 275:

```rust
assert!(!src.contains("VILLAGE_FIXTURE"))
```

Current line 276:

```rust
assert!(!src.contains("BOOTSTRAP_"))
```

Current line 278:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 279:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

## F066

Current `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:284` / `harness_p14_beta_a12_village_fixture_positions_match_diamond_layout`. File SHA256 `a250aea532cdcebbca2f471a5e1e0a6e15594a106668e040d7ecc0e8e6aae665`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 286:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 287:

```rust
assert!(src.contains("_building_textures[kind]"))
```

Current line 288:

```rust
assert!(src.contains("sprite.position = _tile_center(xs[i], ys[i])"))
```

## F067

Current `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:293` / `harness_p14_beta_a13_z_village_fixture_equals_five`. File SHA256 `a250aea532cdcebbca2f471a5e1e0a6e15594a106668e040d7ecc0e8e6aae665`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 295:

```rust
assert!(src.contains("z_index = 0"))
```

Current line 297:

```rust
assert!(src.contains("z_index = 5"))
```

Current line 299:

```rust
assert!(src.contains("sprite.z_index = Z_OVERLAY"))
```

Current line 300:

```rust
assert!(src.contains("const Z_OVERLAY: int = 10"))
```

## F068

Current `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:305` / `harness_p14_beta_a14_village_fixture_placement_loop_present`. File SHA256 `a250aea532cdcebbca2f471a5e1e0a6e15594a106668e040d7ecc0e8e6aae665`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 307:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 308:

```rust
assert!(src.contains("_building_textures[kind]"))
```

Current line 309:

```rust
assert!(src.contains("sprite.position = _tile_center(xs[i], ys[i])"))
```

## F069

Current `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:314` / `harness_p14_beta_a15_phase_13_beta_resource_constants_preserved`. File SHA256 `a250aea532cdcebbca2f471a5e1e0a6e15594a106668e040d7ecc0e8e6aae665`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 316:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

Current line 317:

```rust
assert!(src.contains("if kinds[i] != 0:"))
```

Current line 318:

```rust
assert!(src.contains("berry_bush_%d.png"))
```

Current line 319:

```rust
assert!(src.contains("_berry_textures[stage]"))
```

Current line 320:

```rust
assert!(!src.contains("SOURCE_KIND_COLORS"))
```

Current line 321:

```rust
assert!(!src.contains("Polygon2D"))
```

## F070

Current `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:326` / `harness_p14_beta_a16_phase_13_alpha_bootstrap_constants_preserved`. File SHA256 `a250aea532cdcebbca2f471a5e1e0a6e15594a106668e040d7ecc0e8e6aae665`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 328:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 329:

```rust
assert!(src.contains("_building_textures[kind]"))
```

Current line 330:

```rust
assert!(src.contains("sprite.position = _tile_center(xs[i], ys[i])"))
```

## F071

Current `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:335` / `harness_p14_beta_a17_phase_12_construction_constants_preserved`. File SHA256 `a250aea532cdcebbca2f471a5e1e0a6e15594a106668e040d7ecc0e8e6aae665`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 337:

```rust
assert!(src.contains("res://assets/sprites/furniture/lean_to/1.png"))
```

Current line 338:

```rust
assert!(src.contains("get_construction_snapshot()"))
```

Current line 339:

```rust
assert!(src.contains("sprite.modulate.a = 0.35"))
```

## F072

Current `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:344` / `harness_p14_beta_a18_phase_12_gamma_furniture_constants_preserved`. File SHA256 `a250aea532cdcebbca2f471a5e1e0a6e15594a106668e040d7ecc0e8e6aae665`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 346:

```rust
assert!(src.contains("get_settlement_snapshot()"))
```

Current line 347:

```rust
assert!(src.contains("\"formation_xs\""))
```

Current line 348:

```rust
assert!(src.contains("\"formation_ys\""))
```

Current line 349:

```rust
assert!(src.contains("_tile_center(nearest.x, nearest.y)"))
```

Current line 350:

```rust
assert!(src.contains("_reap(_labels, seen)"))
```

## F073

Current `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:150` / `harness_p14_beta_a1_resource_type_paths_array_declared`. File SHA256 `a250aea532cdcebbca2f471a5e1e0a6e15594a106668e040d7ecc0e8e6aae665`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 152:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

Current line 153:

```rust
assert!(src.contains("if kinds[i] != 0:"))
```

Current line 154:

```rust
assert!(src.contains("berry_bush_%d.png"))
```

Current line 155:

```rust
assert!(src.contains("_berry_textures[stage]"))
```

Current line 156:

```rust
assert!(!src.contains("SOURCE_KIND_COLORS"))
```

Current line 157:

```rust
assert!(!src.contains("Polygon2D"))
```

## F074

Current `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:162` / `harness_p14_beta_a2_resource_type_paths_has_exactly_five_entries`. File SHA256 `a250aea532cdcebbca2f471a5e1e0a6e15594a106668e040d7ecc0e8e6aae665`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 164:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

Current line 165:

```rust
assert!(src.contains("if kinds[i] != 0:"))
```

Current line 166:

```rust
assert!(src.contains("berry_bush_%d.png"))
```

Current line 167:

```rust
assert!(src.contains("_berry_textures[stage]"))
```

Current line 168:

```rust
assert!(!src.contains("SOURCE_KIND_COLORS"))
```

Current line 169:

```rust
assert!(!src.contains("Polygon2D"))
```

## F075

Current `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:174` / `harness_p14_beta_a3_resource_type_paths_index_0_references_legacy_constant`. File SHA256 `a250aea532cdcebbca2f471a5e1e0a6e15594a106668e040d7ecc0e8e6aae665`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 176:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

Current line 177:

```rust
assert!(src.contains("if kinds[i] != 0:"))
```

Current line 178:

```rust
assert!(src.contains("berry_bush_%d.png"))
```

Current line 179:

```rust
assert!(src.contains("_berry_textures[stage]"))
```

Current line 180:

```rust
assert!(!src.contains("SOURCE_KIND_COLORS"))
```

Current line 181:

```rust
assert!(!src.contains("Polygon2D"))
```

## F076

Current `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:186` / `harness_p14_beta_a4_resource_type_paths_index_1_wood_literal`. File SHA256 `a250aea532cdcebbca2f471a5e1e0a6e15594a106668e040d7ecc0e8e6aae665`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 188:

```rust
assert!(!src.contains("on_building_placed("))
```

Current line 189:

```rust
assert!(!src.contains("RESOURCE_SEED"))
```

Current line 190:

```rust
assert!(!src.contains("VILLAGE_FIXTURE"))
```

Current line 191:

```rust
assert!(!src.contains("BOOTSTRAP_"))
```

Current line 193:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 194:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

## F077

Current `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:199` / `harness_p14_beta_a5_resource_type_paths_index_2_stone_literal`. File SHA256 `a250aea532cdcebbca2f471a5e1e0a6e15594a106668e040d7ecc0e8e6aae665`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 201:

```rust
assert!(!src.contains("on_building_placed("))
```

Current line 202:

```rust
assert!(!src.contains("RESOURCE_SEED"))
```

Current line 203:

```rust
assert!(!src.contains("VILLAGE_FIXTURE"))
```

Current line 204:

```rust
assert!(!src.contains("BOOTSTRAP_"))
```

Current line 206:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 207:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

## F078

Current `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:212` / `harness_p14_beta_a6_resource_type_paths_index_3_water_literal`. File SHA256 `a250aea532cdcebbca2f471a5e1e0a6e15594a106668e040d7ecc0e8e6aae665`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 214:

```rust
assert!(!src.contains("on_building_placed("))
```

Current line 215:

```rust
assert!(!src.contains("RESOURCE_SEED"))
```

Current line 216:

```rust
assert!(!src.contains("VILLAGE_FIXTURE"))
```

Current line 217:

```rust
assert!(!src.contains("BOOTSTRAP_"))
```

Current line 219:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 220:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

## F079

Current `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:225` / `harness_p14_beta_a7_resource_type_paths_index_4_food_literal`. File SHA256 `a250aea532cdcebbca2f471a5e1e0a6e15594a106668e040d7ecc0e8e6aae665`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 227:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

Current line 228:

```rust
assert!(src.contains("if kinds[i] != 0:"))
```

Current line 229:

```rust
assert!(src.contains("berry_bush_%d.png"))
```

Current line 230:

```rust
assert!(src.contains("_berry_textures[stage]"))
```

Current line 231:

```rust
assert!(!src.contains("SOURCE_KIND_COLORS"))
```

Current line 232:

```rust
assert!(!src.contains("Polygon2D"))
```

## F080

Current `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:237` / `harness_p14_beta_a8_resource_water_tint_constant_value`. File SHA256 `a250aea532cdcebbca2f471a5e1e0a6e15594a106668e040d7ecc0e8e6aae665`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 239:

```rust
assert!(src.contains("get_terrain_snapshot()"))
```

Current line 240:

```rust
assert!(src.contains("TileSetAtlasSource.new()"))
```

Current line 241:

```rust
assert!(src.contains("_layer.set_cell"))
```

Current line 242:

```rust
assert!(!src.contains("RandomNumberGenerator"))
```

Current line 243:

```rust
assert!(!src.contains("world_terrain.tres"))
```

## F081

Current `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:248` / `harness_p14_beta_a9_resource_scatter_loop_uses_modulo_cycle_and_indexed_iterator`. File SHA256 `a250aea532cdcebbca2f471a5e1e0a6e15594a106668e040d7ecc0e8e6aae665`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 250:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

Current line 251:

```rust
assert!(src.contains("if kinds[i] != 0:"))
```

Current line 252:

```rust
assert!(src.contains("berry_bush_%d.png"))
```

Current line 253:

```rust
assert!(src.contains("_berry_textures[stage]"))
```

Current line 254:

```rust
assert!(!src.contains("SOURCE_KIND_COLORS"))
```

Current line 255:

```rust
assert!(!src.contains("Polygon2D"))
```

## F082

Current `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs:554` / `harness_p14_delta_a14_phase_14_beta_resource_registry_preserved`. File SHA256 `6f2e66803cdf0fadf6ed9583e7f485b42d18bd7642ed85d2a990fe06c9210fa5`. Goal 9.5 (simulation-driven HUD and localization).

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 556:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

Current line 557:

```rust
assert!(src.contains("if kinds[i] != 0:"))
```

Current line 558:

```rust
assert!(src.contains("berry_bush_%d.png"))
```

Current line 559:

```rust
assert!(src.contains("_berry_textures[stage]"))
```

Current line 560:

```rust
assert!(!src.contains("SOURCE_KIND_COLORS"))
```

Current line 561:

```rust
assert!(!src.contains("Polygon2D"))
```

## F083

Current `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs:582` / `harness_p14_delta_a16_village_fixture_constants_preserved_in_world_renderer`. File SHA256 `6f2e66803cdf0fadf6ed9583e7f485b42d18bd7642ed85d2a990fe06c9210fa5`. Goal 9.5 (simulation-driven HUD and localization).

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 584:

```rust
assert!(!src.contains("on_building_placed("))
```

Current line 585:

```rust
assert!(!src.contains("RESOURCE_SEED"))
```

Current line 586:

```rust
assert!(!src.contains("VILLAGE_FIXTURE"))
```

Current line 587:

```rust
assert!(!src.contains("BOOTSTRAP_"))
```

Current line 589:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 590:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

## F084

Current `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs:595` / `harness_p14_delta_a17_hud_topbar_file_byte_or_line_count_unchanged_against_pre_phase_14_delta_head`. File SHA256 `6f2e66803cdf0fadf6ed9583e7f485b42d18bd7642ed85d2a990fe06c9210fa5`. Goal 9.5 (simulation-driven HUD and localization).

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 616:

```rust
assert_eq!(
        got_lines, EXPECTED_LINES,
        "A17.2: hud_topbar.gd line count must equal Revitalize Part 4 ({EXPECTED_LINES}); got {got_lines}"
    )
```

Current line 620:

```rust
assert_eq!(
        got_hash, EXPECTED_HASH,
        "A17.3: hud_topbar.gd SHA-256 must equal Revitalize Part 4 `{EXPECTED_HASH}`; got `{got_hash}`. \
         Revitalize HUD truthfulness snapshot changed; review and re-pin deliberately."
    )
```

## F085

Current `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs:301` / `harness_p14_delta_a2_ticks_per_day_constant_present`. File SHA256 `6f2e66803cdf0fadf6ed9583e7f485b42d18bd7642ed85d2a990fe06c9210fa5`. Goal 9.5 (simulation-driven HUD and localization).

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 303:

```rust
assert!(stripped.contains("get_time_snapshot"))
```

Current line 304:

```rust
assert!(!stripped.contains("TICKS_PER_DAY"))
```

Current line 305:

```rust
assert!(stripped.contains("time_of_day"))
```

## F086

Current `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs:310` / `harness_p14_delta_a3_days_per_year_constant_present`. File SHA256 `6f2e66803cdf0fadf6ed9583e7f485b42d18bd7642ed85d2a990fe06c9210fa5`. Goal 9.5 (simulation-driven HUD and localization).

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 312:

```rust
assert!(stripped.contains("day_index"))
```

Current line 313:

```rust
assert!(stripped.contains("HUD_DAY_FMT"))
```

Current line 314:

```rust
assert!(stripped.contains("HUD_PHASE_DAY") && stripped.contains("HUD_PHASE_NIGHT"))
```

Current line 315:

```rust
assert!(!stripped.contains("DAYS_PER_YEAR"))
```

## F087

Current `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs:320` / `harness_p14_delta_a4_resource_count_consistency_with_phase_14_beta`. File SHA256 `6f2e66803cdf0fadf6ed9583e7f485b42d18bd7642ed85d2a990fe06c9210fa5`. Goal 9.5 (simulation-driven HUD and localization).

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 323:

```rust
assert!(stripped.contains(source), "HUD data source missing: {source}")
```

Current line 325:

```rust
assert!(!stripped.contains("RESOURCE_PER_TYPE"))
```

## F088

Current `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs:330` / `harness_p14_delta_a5_resource_labels_array_exact_order`. File SHA256 `6f2e66803cdf0fadf6ed9583e7f485b42d18bd7642ed85d2a990fe06c9210fa5`. Goal 9.5 (simulation-driven HUD and localization).

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 333:

```rust
assert!(stripped.contains(key), "Localized metric missing: {key}")
```

Current line 335:

```rust
assert!(!stripped.contains("RESOURCE_LABELS"))
```

Current line 336:

```rust
assert!(no_ws(&stripped).contains("_loc(keys[i])"))
```

## F089

Current `rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs:535` / `harness_p14_epsilon_a14_regression_guard_world_renderer_phase_14_beta_intact`. File SHA256 `30a29e84c3ac3ec17d339e434d7ce9d6dd252775148736aa007483da6cbc66cf`. Goal 9.4 (selection, localized inspector and lines); 9.7 for camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 537:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

Current line 538:

```rust
assert!(src.contains("if kinds[i] != 0:"))
```

Current line 539:

```rust
assert!(src.contains("berry_bush_%d.png"))
```

Current line 540:

```rust
assert!(src.contains("_berry_textures[stage]"))
```

Current line 541:

```rust
assert!(!src.contains("SOURCE_KIND_COLORS"))
```

Current line 542:

```rust
assert!(!src.contains("Polygon2D"))
```

## F090

Current `rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs:277` / `harness_p14_epsilon_a4_trail_alpha_constant_equals_0_4`. File SHA256 `30a29e84c3ac3ec17d339e434d7ce9d6dd252775148736aa007483da6cbc66cf`. Goal 9.4 (selection, localized inspector and lines); 9.7 for camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 283:

```rust
assert!(
        (f - 0.8).abs() < 1e-9,
        "A4: TRAIL_ALPHA must equal 0.8; got {f}"
    )
```

## F091

Current `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs:951` / `harness_p14_gamma_a23_world_renderer_no_extra_ffi_added`. File SHA256 `d9a80471e927855cb4b22327dad48ed27a06bf2e65638bb7d50e02e9ee24887c`. Goal 9.4 (selection, localized inspector and lines); 9.7 for camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 999:

```rust
assert_eq!(
            stripped.matches("world_sim.connect(").count(),
            stripped.matches("world_sim.connect(\"world_reset\", refresh_snapshot)").count(),
            "A23: only the world_reset snapshot-refresh subscription is allowed"
        )
```

Current line 1006:

```rust
assert!(
        extras.is_empty(),
        "A23: world_renderer.gd introduces FFI calls outside the allowed renderer snapshot set: \
         {extras:?}; all calls = {found:?}"
    )
```

Current line 1011:

```rust
assert!(
        found.contains("get_agent_detail"),
        "A23: world_renderer.gd MUST call `world_sim.get_agent_detail(...)`. \
         Calls observed: {found:?}"
    )
```

## F092

Current `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs:1066` / `harness_p14_gamma_a27_phase14_beta_invariants_preserved`. File SHA256 `d9a80471e927855cb4b22327dad48ed27a06bf2e65638bb7d50e02e9ee24887c`. Goal 9.4 (selection, localized inspector and lines); 9.7 for camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 1068:

```rust
assert!(!src.contains("on_building_placed("))
```

Current line 1069:

```rust
assert!(!src.contains("RESOURCE_SEED"))
```

Current line 1070:

```rust
assert!(!src.contains("VILLAGE_FIXTURE"))
```

Current line 1071:

```rust
assert!(!src.contains("BOOTSTRAP_"))
```

Current line 1073:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 1074:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

## F093

Current `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs:1078` / `harness_p14_gamma_a28_phase12_to_13_visual_invariants_preserved`. File SHA256 `d9a80471e927855cb4b22327dad48ed27a06bf2e65638bb7d50e02e9ee24887c`. Goal 9.4 (selection, localized inspector and lines); 9.7 for camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 1081:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 1082:

```rust
assert!(src.contains("_building_textures[kind]"))
```

Current line 1083:

```rust
assert!(src.contains("sprite.position = _tile_center(xs[i], ys[i])"))
```

Current line 1085:

```rust
assert!(src.contains("res://assets/sprites/furniture/lean_to/1.png"))
```

Current line 1086:

```rust
assert!(src.contains("get_construction_snapshot()"))
```

Current line 1087:

```rust
assert!(src.contains("sprite.modulate.a = 0.35"))
```

Current line 1089:

```rust
assert!(src.contains("res://assets/sprites/buildings/stockpile/1.png"))
```

Current line 1090:

```rust
assert!(src.contains("kind == 2"))
```

Current line 1091:

```rust
assert!(src.contains("_stockpile_positions.append"))
```

Current line 1096:

```rust
assert_eq!(
        no_ws(&zd),
        "Vector2(4.0,4.0)",
        "A28: ZOOM_DEFAULT must be Vector2(4.0, 4.0) (set to 4.0 by Revitalize Part 4); got `{zd}`"
    )
```

Current line 1102:

```rust
assert_eq!(
        no_ws(&zmin),
        "Vector2(0.5,0.5)",
        "A28: ZOOM_MIN must be Vector2(0.5, 0.5); got `{zmin}`"
    )
```

Current line 1108:

```rust
assert_eq!(
        no_ws(&zmax),
        "Vector2(8.0,8.0)",
        "A28: ZOOM_MAX must be Vector2(8.0, 8.0) (raised from 4.0 in G Phase A); got `{zmax}`"
    )
```

## F094

Current `rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs:654` / `harness_zoom_a19_phase12a_13a_camera_controller_zoom_invariants_intact`. File SHA256 `3ad62bc56cfb25d62c70de880026f29ed2bffad87f0e534466a8ac085717d716`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 664:

```rust
assert!(
        missing.is_empty(),
        "a19.1: camera_controller.gd must keep ZOOM_MIN/MAX/DEFAULT; missing={missing:?}"
    )
```

Current line 676:

```rust
assert!(
        missing_lits.is_empty(),
        "a19.2: camera_controller.gd must keep 0.5/8.0/5.0 zoom literals \
         (ZOOM_DEFAULT set to 4.0 by Revitalize Part 4); missing={missing_lits:?}"
    )
```

## F095

Current `rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs:711` / `harness_zoom_a21_phase14b_world_renderer_invariants_intact`. File SHA256 `3ad62bc56cfb25d62c70de880026f29ed2bffad87f0e534466a8ac085717d716`. Goal 9.7 (camera), with 9.2–9.3 when the same function also pins world objects.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 713:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

Current line 714:

```rust
assert!(src.contains("if kinds[i] != 0:"))
```

Current line 715:

```rust
assert!(src.contains("berry_bush_%d.png"))
```

Current line 716:

```rust
assert!(src.contains("_berry_textures[stage]"))
```

Current line 717:

```rust
assert!(!src.contains("SOURCE_KIND_COLORS"))
```

Current line 718:

```rust
assert!(!src.contains("Polygon2D"))
```

## F096

Current `rust/crates/sim-test/tests/harness_resource_scarcity.rs:1263` / `harness_scarcity_a16_production_init_seeds_all_three_kinds_finite`. File SHA256 `899f8b75fe5afe6b61db86675bac227bea90a9faf36cf18c72824f34bb0e5f18`. Goal 2 stale-documentation warning and 7 (live world becomes seeded; legacy fixture predicates retained).

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 1274:

```rust
assert!(finite_food, "A16: legacy V7 fixture must seed a FINITE FOOD source (∈ [1,254])")
```

Current line 1275:

```rust
assert!(finite_water, "A16: legacy V7 fixture must seed a FINITE WATER source (∈ [1,254])")
```

Current line 1276:

```rust
assert!(finite_sleep, "A16: legacy V7 fixture must seed a FINITE SLEEP source (∈ [1,254])")
```

Current line 1277:

```rust
assert!(!e.resources.food_source_max.is_empty(), "A16: food_source_max must be non-empty")
```

Current line 1278:

```rust
assert!(!e.resources.water_source_max.is_empty(), "A16: water_source_max must be non-empty")
```

Current line 1279:

```rust
assert!(!e.resources.sleep_source_max.is_empty(), "A16: sleep_source_max must be non-empty")
```

## F097

Current `rust/crates/sim-test/tests/harness_s16_alpha0_resource_substrate.rs:421` / `harness_resource_decorative_layer_preserved_static`. File SHA256 `308259207541dc7672fa9161d3a2fae5a87cf0cf934090ca69713a6e1431ddfa`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 423:

```rust
assert!(!src.contains("on_building_placed("))
```

Current line 424:

```rust
assert!(!src.contains("RESOURCE_SEED"))
```

Current line 425:

```rust
assert!(!src.contains("VILLAGE_FIXTURE"))
```

Current line 426:

```rust
assert!(!src.contains("BOOTSTRAP_"))
```

Current line 428:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 429:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

## F098

Current `rust/crates/sim-test/tests/harness_s16_gamma_visual.rs:197` / `harness_s16_gamma_a4_world_renderer_marker_is_polygon2d`. File SHA256 `afd36dee71c1d40bacce73ccd3e034e104d2e5298059ffbbf7e5e523afaa391d`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 199:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

Current line 200:

```rust
assert!(src.contains("if kinds[i] != 0:"))
```

Current line 201:

```rust
assert!(src.contains("berry_bush_%d.png"))
```

Current line 202:

```rust
assert!(src.contains("_berry_textures[stage]"))
```

Current line 203:

```rust
assert!(!src.contains("SOURCE_KIND_COLORS"))
```

Current line 204:

```rust
assert!(!src.contains("Polygon2D"))
```

## F099

Current `rust/crates/sim-test/tests/harness_s16_gamma_visual.rs:211` / `harness_s16_gamma_a5_z_order_above_decorative`. File SHA256 `afd36dee71c1d40bacce73ccd3e034e104d2e5298059ffbbf7e5e523afaa391d`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 213:

```rust
assert!(src.contains("z_index = 0"))
```

Current line 215:

```rust
assert!(src.contains("z_index = 5"))
```

Current line 217:

```rust
assert!(src.contains("sprite.z_index = Z_OVERLAY"))
```

Current line 218:

```rust
assert!(src.contains("const Z_OVERLAY: int = 10"))
```

## F100

Current `rust/crates/sim-test/tests/harness_s16_gamma_visual.rs:225` / `harness_s16_gamma_a6_decorative_invariant_preserved`. File SHA256 `afd36dee71c1d40bacce73ccd3e034e104d2e5298059ffbbf7e5e523afaa391d`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 227:

```rust
assert!(!src.contains("on_building_placed("))
```

Current line 228:

```rust
assert!(!src.contains("RESOURCE_SEED"))
```

Current line 229:

```rust
assert!(!src.contains("VILLAGE_FIXTURE"))
```

Current line 230:

```rust
assert!(!src.contains("BOOTSTRAP_"))
```

Current line 232:

```rust
assert!(src.contains("get_building_snapshot()"))
```

Current line 233:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

## F101

Current `rust/crates/sim-test/tests/harness_s16_gamma_visual.rs:240` / `harness_s16_gamma_a7_backend_truth_positions_preserved`. File SHA256 `afd36dee71c1d40bacce73ccd3e034e104d2e5298059ffbbf7e5e523afaa391d`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 242:

```rust
assert!(src.contains("get_resource_snapshot()"))
```

Current line 243:

```rust
assert!(src.contains("if kinds[i] != 0:"))
```

Current line 244:

```rust
assert!(src.contains("berry_bush_%d.png"))
```

Current line 245:

```rust
assert!(src.contains("_berry_textures[stage]"))
```

Current line 246:

```rust
assert!(!src.contains("SOURCE_KIND_COLORS"))
```

Current line 247:

```rust
assert!(!src.contains("Polygon2D"))
```

## F102

Current `rust/crates/sim-test/tests/harness_show_resource_depletion.rs:406` / `harness_resource_depletion_a10_live_depletion_surfaces`. File SHA256 `16fd36350a1732ae706b748547c6e79b691c8230a468d0795af5b6233e32869f`. Goal 2 stale-documentation warning and 7 (live world becomes seeded; legacy fixture predicates retained).

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 422:

```rust
assert!(
        partial || reaped,
        "A10: after 4380 legacy fixture ticks depletion must surface — either a partially \
         depleted row (amount < max) OR fewer present source rows than at tick \
         2000 (count_2000={count_2000}, count_4380={}, partial={partial}, reaped={reaped})",
        rows.len()
    )
```

## F103

Current `rust/crates/sim-test/tests/harness_show_resource_depletion.rs:374` / `harness_resource_depletion_a9_production_scene_registers_finite_capacities`. File SHA256 `16fd36350a1732ae706b748547c6e79b691c8230a468d0795af5b6233e32869f`. Goal 2 stale-documentation warning and 7 (live world becomes seeded; legacy fixture predicates retained).

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 388:

```rust
assert!(
        finite >= 1,
        "A9: legacy V7 snapshot must carry >= 1 row with a registered finite \
         capacity (max != {} sentinel and max >= 1); rows={}, finite={}",
        RESOURCE_SOURCE_INFINITE,
        rows.len(),
        finite
    )
```

## F104

Current `rust/crates/sim-test/tests/harness_t7_10_b1_space_toggle.rs:372` / `harness_t7_10_b1_substrate_calls_survive_off_gate`. File SHA256 `4dd1b938df33fbb5592d26246d16c81753c0e6d6672d3b1f5ad994a5e57fe254`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 374:

```rust
assert!(src.contains("_frame % RECONCILE_FRAMES == 0 or _buildings.is_empty()"))
```

Current line 375:

```rust
assert!(src.contains("_update_construction_sites()"))
```

Current line 376:

```rust
assert!(src.contains("_update_bushes()"))
```

Current line 377:

```rust
assert!(!src.contains("CHANNEL_OFF"))
```

## F105

Current `rust/crates/sim-test/tests/harness_t7_9_b_render_mechanism.rs:103` / `harness_t7_9_b_world_renderer_render_path_present`. File SHA256 `3810f984a1763a20cfac8b569a5f3f6e09cd456519d1a4055ed6ac6c1cae3665`. Goal 9.2–9.3 (simulation terrain and real objects), 9.4 (labels), with 9.7 where the function includes camera guards.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 106:

```rust
assert!(
        src.contains("CHANNEL_WARMTH := 0"),
        "world_renderer.gd must declare `CHANNEL_WARMTH := 0` \
         (Warmth = InfluenceChannel index 0, the first overlay render target)"
    )
```

Current line 111:

```rust
assert!(include_str!("../../../../scripts/ui/building_renderer.gd").contains("_world_sim.get_building_snapshot()"), "building renderer must read real buildings")
```

Current line 112:

```rust
assert!(!src.contains("on_building_placed("), "renderer must not bootstrap simulation state")
```

Current line 113:

```rust
assert!(
        src.contains("Image.FORMAT_L8"),
        "world_renderer.gd must use `Image.FORMAT_L8` (single-channel 8-bit \
         matches InfluenceGrid u8 cell layout)"
    )
```

Current line 118:

```rust
assert!(
        src.contains("Sprite2D.new()"),
        "world_renderer.gd must instantiate `Sprite2D.new()` as the texture host"
    )
```

Current line 128:

```rust
assert!(
        src.contains("get_influence_overlay(current_channel)"),
        "world_renderer.gd _process must call \
         `world_sim.get_influence_overlay(current_channel)` (channel cycle)"
    )
```

Current line 133:

```rust
assert!(
        src.contains("current_channel: int = CHANNEL_OFF"),
        "world_renderer.gd must initialise `current_channel: int = CHANNEL_OFF` \
         so the first visible frame is a clean screen (overlay opt-in via SPACE; \
         fix-overlay-darkness re-point of the old CHANNEL_WARMTH default)"
    )
```

Current line 139:

```rust
assert!(
        src.contains("CHANNEL_LIGHT := 1"),
        "world_renderer.gd must declare `CHANNEL_LIGHT := 1` for the toggle target \
         (Light = InfluenceChannel index 1)"
    )
```

Current line 144:

```rust
assert!(
        src.contains("texture.update(image)"),
        "world_renderer.gd must call `texture.update(image)` per frame \
         (re-uploads pixels without re-creating the GPU resource)"
    )
```

## F106

Current `rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs:467` / `harness_g_a13_existing_p12_alpha_a4_updated_to_eight`. File SHA256 `a3b093d1058ca954e817d07c47d122bde7c511a0d6d498b67bf017532162ba0c`. Goal 9.7: preserve maximum zoom 8x while allowing the new default 4x; input narrows from entire test file to its exact maximum-zoom test function.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 475:

```rust
assert!(
        compact.contains("Vector2(8.0,8.0)"),
        "a13.1: harness_p12_alpha_camera_zoom.rs must accept Vector2(8.0,8.0) (raised in G Phase A)"
    )
```

Current line 479:

```rust
assert!(
        !compact.contains("Vector2(4.0,4.0)"),
        "a13.2: harness_p12_alpha_camera_zoom.rs must NOT still assert the old Vector2(4.0,4.0) \
         ZOOM_MAX literal"
    )
```

Baseline input scope:

```rust
fn harness_g_a13_existing_p12_alpha_a4_updated_to_eight() {
    // Type D — p12-α A4 accepted-literal list must now track 8.0, not 4.0.
    let src = read_test_file("harness_p12_alpha_camera_zoom.rs");
    let compact = no_ws(&src);
    assert!(
        compact.contains("Vector2(8.0,8.0)"),
        "a13.1: harness_p12_alpha_camera_zoom.rs must accept Vector2(8.0,8.0) (raised in G Phase A)"
    );
    assert!(
        !compact.contains("Vector2(4.0,4.0)"),
        "a13.2: harness_p12_alpha_camera_zoom.rs must NOT still assert the old Vector2(4.0,4.0) \
         ZOOM_MAX literal"
    );
    println!("[G a13] p12-α A4 tracks ZOOM_MAX = 8.0 ✓");
}
```

Current input scope:

```rust
fn harness_g_a13_existing_p12_alpha_a4_updated_to_eight() {
    // Type D — p12-α A4 accepted-literal list must now track 8.0, not 4.0.
    let src = read_test_file("harness_p12_alpha_camera_zoom.rs");
    // Default zoom is four; this guard belongs only to the maximum-zoom contract.
    let max_test = src.split_once("fn harness_p12_alpha_a4_zoom_max_constant()")
        .expect("maximum zoom test exists").1.split_once("\n}")
        .expect("maximum zoom test closes").0;
    let compact = no_ws(max_test);
    assert!(
        compact.contains("Vector2(8.0,8.0)"),
        "a13.1: harness_p12_alpha_camera_zoom.rs must accept Vector2(8.0,8.0) (raised in G Phase A)"
    );
    assert!(
        !compact.contains("Vector2(4.0,4.0)"),
        "a13.2: harness_p12_alpha_camera_zoom.rs must NOT still assert the old Vector2(4.0,4.0) \
         ZOOM_MAX literal"
    );
    println!("[G a13] p12-α A4 tracks ZOOM_MAX = 8.0 ✓");
}
```

## F107

Current `rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs:489` / `harness_g_a14_existing_p13_alpha_a3_updated_to_eight`. File SHA256 `a3b093d1058ca954e817d07c47d122bde7c511a0d6d498b67bf017532162ba0c`. Goal 9.7: preserve maximum zoom 8x while allowing the new default 4x; input narrows from entire test file to its exact maximum-zoom test function.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 497:

```rust
assert!(
        compact.contains("Vector2(8.0,8.0)"),
        "a14.1: harness_p13_alpha_camera_and_buildings.rs must expect Vector2(8.0,8.0)"
    )
```

Current line 501:

```rust
assert!(
        !compact.contains("Vector2(4.0,4.0)"),
        "a14.2: harness_p13_alpha_camera_and_buildings.rs must NOT still expect Vector2(4.0,4.0)"
    )
```

Baseline input scope:

```rust
fn harness_g_a14_existing_p13_alpha_a3_updated_to_eight() {
    // Type D — p13-α A3 expected ZOOM_MAX literal must now equal 8.0.
    let src = read_test_file("harness_p13_alpha_camera_and_buildings.rs");
    let compact = no_ws(&src);
    assert!(
        compact.contains("Vector2(8.0,8.0)"),
        "a14.1: harness_p13_alpha_camera_and_buildings.rs must expect Vector2(8.0,8.0)"
    );
    assert!(
        !compact.contains("Vector2(4.0,4.0)"),
        "a14.2: harness_p13_alpha_camera_and_buildings.rs must NOT still expect Vector2(4.0,4.0)"
    );
    println!("[G a14] p13-α A3 expects ZOOM_MAX = 8.0 ✓");
}
```

Current input scope:

```rust
fn harness_g_a14_existing_p13_alpha_a3_updated_to_eight() {
    // Type D — p13-α A3 expected ZOOM_MAX literal must now equal 8.0.
    let src = read_test_file("harness_p13_alpha_camera_and_buildings.rs");
    // Default zoom is four; this guard belongs only to the maximum-zoom contract.
    let max_test = src.split_once("fn harness_p13_alpha_a3_camera_controller_zoom_max_unchanged()")
        .expect("maximum zoom test exists").1.split_once("\n}")
        .expect("maximum zoom test closes").0;
    let compact = no_ws(max_test);
    assert!(
        compact.contains("Vector2(8.0,8.0)"),
        "a14.1: harness_p13_alpha_camera_and_buildings.rs must expect Vector2(8.0,8.0)"
    );
    assert!(
        !compact.contains("Vector2(4.0,4.0)"),
        "a14.2: harness_p13_alpha_camera_and_buildings.rs must NOT still expect Vector2(4.0,4.0)"
    );
    println!("[G a14] p13-α A3 expects ZOOM_MAX = 8.0 ✓");
}
```

## F108

Current `rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs:470` / `harness_p14_epsilon_a12_main_tscn_load_steps_equals_12`. File SHA256 `30a29e84c3ac3ec17d339e434d7ce9d6dd252775148736aa007483da6cbc66cf`. Goal 9.2, 9.3, 9.6: three renderer script resources plus correct root load-step accounting (17).

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 504:

```rust
assert_eq!(n, 17, "A12.2: load_steps must equal 17 (Phase 14-ζ 9 → 11, viz-B 11 → 12, viz-D 12 → 13); got {n}")
```

Baseline input scope:

```rust
fn harness_p14_epsilon_a12_main_tscn_load_steps_equals_12() {
    // Type A — `[gd_scene ... load_steps=13 ...]`, whitespace around `=` tolerated.
    // Phase 14-ε registered ActivityTrailRenderer at load_steps=9; Phase 14-ζ
    // added SettlementOverviewRenderer + ZoomLodController (9 → 11); viz-B added
    // NeedBarRenderer (11 → 12); viz-D adds DeathVizRenderer (12 → 13). The scene
    // is shared, so this prior-phase invariant tracks the current ext_resource
    // count. ε's ActivityTrailRenderer registration itself is unchanged (A11).
    let src = read_main_tscn_src();
    let mut found_n: Option<i64> = None;
    for line in src.lines() {
        let t = line.trim();
        if !t.starts_with("[gd_scene") {
            continue;
        }
        if let Some(pos) = t.find("load_steps") {
            let after = &t[pos + "load_steps".len()..];
            let after = after.trim_start();
            let after = after.strip_prefix('=').unwrap_or(after);
            let after = after.trim_start();
            let mut digits = String::new();
            for c in after.chars() {
                if c.is_ascii_digit() {
                    digits.push(c);
                } else {
                    break;
                }
            }
            if !digits.is_empty() {
                found_n = Some(digits.parse::<i64>().expect("digit parse"));
            }
        }
        break;
    }
    let n = found_n.expect("A12.1: main.tscn must have `[gd_scene ... load_steps=N ...]`");
    assert_eq!(n, 13, "A12.2: load_steps must equal 13 (Phase 14-ζ 9 → 11, viz-B 11 → 12, viz-D 12 → 13); got {n}");
    println!("[P14-ε A12] load_steps = 13 (post viz-D DeathVizRenderer) ✓");
}
```

Current input scope:

```rust
fn harness_p14_epsilon_a12_main_tscn_load_steps_equals_12() {
    // Type A — `[gd_scene ... load_steps=13 ...]`, whitespace around `=` tolerated.
    // Phase 14-ε registered ActivityTrailRenderer at load_steps=9; Phase 14-ζ
    // added SettlementOverviewRenderer + ZoomLodController (9 → 11); viz-B added
    // NeedBarRenderer (11 → 12); viz-D adds DeathVizRenderer (12 → 13). The scene
    // is shared, so this prior-phase invariant tracks the current ext_resource
    // count. ε's ActivityTrailRenderer registration itself is unchanged (A11).
    let src = read_main_tscn_src();
    let mut found_n: Option<i64> = None;
    for line in src.lines() {
        let t = line.trim();
        if !t.starts_with("[gd_scene") {
            continue;
        }
        if let Some(pos) = t.find("load_steps") {
            let after = &t[pos + "load_steps".len()..];
            let after = after.trim_start();
            let after = after.strip_prefix('=').unwrap_or(after);
            let after = after.trim_start();
            let mut digits = String::new();
            for c in after.chars() {
                if c.is_ascii_digit() {
                    digits.push(c);
                } else {
                    break;
                }
            }
            if !digits.is_empty() {
                found_n = Some(digits.parse::<i64>().expect("digit parse"));
            }
        }
        break;
    }
    let n = found_n.expect("A12.1: main.tscn must have `[gd_scene ... load_steps=N ...]`");
    assert_eq!(n, 17, "A12.2: load_steps must equal 17 (Phase 14-ζ 9 → 11, viz-B 11 → 12, viz-D 12 → 13); got {n}");
    println!("[P14-ε A12] load_steps = 17 (post viz-D DeathVizRenderer) ✓");
}
```

## F109

Current `rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs:597` / `harness_zoom_a17_main_tscn_load_steps_equals_12`. File SHA256 `3ad62bc56cfb25d62c70de880026f29ed2bffad87f0e534466a8ac085717d716`. Goal 9.2, 9.3, 9.6: three renderer script resources plus correct root load-step accounting (17).

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 606:

```rust
assert_eq!(n, 17, "a17.2: load_steps must equal 17; got {n}")
```

Baseline input scope:

```rust
fn harness_zoom_a17_main_tscn_load_steps_equals_12() {
    // Type A — `[gd_scene ... load_steps=13 ...]`, ws around `=` tolerated.
    // viz-B added the 12th ext_resource (NeedBarRenderer); viz-D adds the 13th
    // (death_viz_renderer.gd → DeathVizRenderer node). Godot requires load_steps
    // to match the resource count, so the locked value advances 12 → 13 with the
    // scene.
    let src = read_main_tscn_src();
    let n = tscn_load_steps(&src)
        .expect("a17.1: main.tscn must have `[gd_scene ... load_steps=N ...]`");
    assert_eq!(n, 13, "a17.2: load_steps must equal 13; got {n}");
    println!("[P14-ζ a17] load_steps = 13 ✓");
}
```

Current input scope:

```rust
fn harness_zoom_a17_main_tscn_load_steps_equals_12() {
    // Type A — `[gd_scene ... load_steps=13 ...]`, ws around `=` tolerated.
    // viz-B added the 12th ext_resource (NeedBarRenderer); viz-D adds the 13th
    // (death_viz_renderer.gd → DeathVizRenderer node). Godot requires load_steps
    // to match the resource count, so the locked value advances 12 → 13 with the
    // scene.
    let src = read_main_tscn_src();
    let n = tscn_load_steps(&src)
        .expect("a17.1: main.tscn must have `[gd_scene ... load_steps=N ...]`");
    assert_eq!(n, 17, "a17.2: load_steps must equal 17; got {n}");
    println!("[P14-ζ a17] load_steps = 17 ✓");
}
```

## F110

Current `rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs:574` / `harness_p14_epsilon_a16_regression_guard_hud_status_panel_phase_14_delta_intact`. File SHA256 `30a29e84c3ac3ec17d339e434d7ce9d6dd252775148736aa007483da6cbc66cf`. Goal 9.5: replace frame-clock/decorative resource needle inputs with simulation time/resource/tally FFI names.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 584:

```rust
assert!(
        missing.is_empty(),
        "A16: hud_status_panel.gd must keep all 3 invariant literals (Phase 14-δ A2/A4/A5); \
         missing={missing:?}"
    )
```

Baseline input scope:

```rust
fn harness_p14_epsilon_a16_regression_guard_hud_status_panel_phase_14_delta_intact() {
    // Type D — three canonical literals all present.
    let stripped = strip_gd_comments(&read_hud_status_panel_src());
    let needles = ["TICKS_PER_DAY", "RESOURCE_TYPES_COUNT", "RESOURCE_LABELS"];
    let mut missing: Vec<&str> = Vec::new();
    for n in needles.iter() {
        if !stripped.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "A16: hud_status_panel.gd must keep all 3 invariant literals (Phase 14-δ A2/A4/A5); \
         missing={missing:?}"
    );
    println!("[P14-ε A16] hud_status_panel.gd Phase 14-δ invariants intact ✓");
}
```

Current input scope:

```rust
fn harness_p14_epsilon_a16_regression_guard_hud_status_panel_phase_14_delta_intact() {
    // Type D — three canonical literals all present.
    let stripped = strip_gd_comments(&read_hud_status_panel_src());
    let needles = ["get_time_snapshot", "get_resource_snapshot", "get_tallies"];
    let mut missing: Vec<&str> = Vec::new();
    for n in needles.iter() {
        if !stripped.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "A16: hud_status_panel.gd must keep all 3 invariant literals (Phase 14-δ A2/A4/A5); \
         missing={missing:?}"
    );
    println!("[P14-ε A16] hud_status_panel.gd Phase 14-δ invariants intact ✓");
}
```

## F111

Current `rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs:750` / `harness_zoom_a23_phase14d_hud_status_panel_intact`. File SHA256 `3ad62bc56cfb25d62c70de880026f29ed2bffad87f0e534466a8ac085717d716`. Goal 9.5: replace frame-clock/decorative resource needle inputs with simulation time/resource/tally FFI names.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 760:

```rust
assert!(
        missing.is_empty(),
        "a23: hud_status_panel.gd must keep all 3 invariant literals; missing={missing:?}"
    )
```

Baseline input scope:

```rust
fn harness_zoom_a23_phase14d_hud_status_panel_intact() {
    // Type D — three canonical literals all present.
    let stripped = strip_gd_comments(&read_hud_status_panel_src());
    let needles = ["TICKS_PER_DAY", "RESOURCE_TYPES_COUNT", "RESOURCE_LABELS"];
    let mut missing: Vec<&str> = Vec::new();
    for n in needles.iter() {
        if !stripped.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "a23: hud_status_panel.gd must keep all 3 invariant literals; missing={missing:?}"
    );
    println!("[P14-ζ a23] hud_status_panel.gd Phase 14-δ invariants intact ✓");
}
```

Current input scope:

```rust
fn harness_zoom_a23_phase14d_hud_status_panel_intact() {
    // Type D — three canonical literals all present.
    let stripped = strip_gd_comments(&read_hud_status_panel_src());
    let needles = ["get_time_snapshot", "get_resource_snapshot", "get_tallies"];
    let mut missing: Vec<&str> = Vec::new();
    for n in needles.iter() {
        if !stripped.contains(n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "a23: hud_status_panel.gd must keep all 3 invariant literals; missing={missing:?}"
    );
    println!("[P14-ζ a23] hud_status_panel.gd Phase 14-δ invariants intact ✓");
}
```

## F112

Current `rust/crates/sim-test/tests/harness_show_agent_needs_bars.rs:470` / `harness_needs_a6_scene_wiring_additive`. File SHA256 `d6d93dfd638024b3ce0956b877b44c8968f37872722778972e837123a81c9a93`. Goal 9.2, 9.3, 9.6: three new renderer resources bring ext_resource count to 16 and scene load_steps to 17; existing node and script-identity guards preserved.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 481:

```rust
assert_eq!(ext_count_after, 16, "all sixteen renderer/UI script resources remain")
```

Current line 482:

```rust
assert_eq!(load_steps_after, ext_count_after as i64 + 1, "resource steps include the scene itself")
```

Current line 485:

```rust
assert!(
        scene.contains("need_bar_renderer.gd"),
        "A6(c): new ext_resource must reference need_bar_renderer.gd"
    )
```

Current line 493:

```rust
assert!(has_node, "A6(c): NeedBarRenderer Node2D node must exist")
```

Current line 506:

```rust
assert!(scene.contains(token), "A6(c): pre-existing wiring `{token}` must be preserved (removal==0)")
```

## F113

Current `rust/crates/sim-test/tests/harness_show_death_visual.rs:423` / `harness_death_a6_scene_wiring_additive`. File SHA256 `3d4d034811272291e6636de255e1ec0fad0af85b7da800c7fa099e4e11c3b2d7`. Goal 9.2, 9.3, 9.6: three new renderer resources bring ext_resource count to 16 and scene load_steps to 17; existing node and script-identity guards preserved.

Full current assertion contract for mapped old pins; no claim of one-to-one equivalence:

Current line 433:

```rust
assert_eq!(ext_count_after, 16, "all sixteen renderer/UI script resources remain")
```

Current line 434:

```rust
assert_eq!(load_steps_after, ext_count_after as i64 + 1, "resource steps include the scene itself")
```

Current line 466:

```rust
assert!(
        node_binds_script,
        "A6: DeathVizRenderer Node2D must exist AND bind the death_viz_renderer.gd ext_resource \
         (id={ext_id}) — a present node with a wrong/missing script path fails"
    )
```

Current line 481:

```rust
assert!(scene.contains(token), "A6: pre-existing wiring `{token}` must be preserved")
```

## A0001

Baseline `rust/crates/sim-test/tests/harness_b1_zoom_default_raise.rs:205` / `harness_b1_a1_zoom_default_equals_five`. Replacement [F001](#f001). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        no_ws(&def_rhs),
        "Vector2(5.0,5.0)",
        "a1.2: ZOOM_DEFAULT must equal Vector2(5.0,5.0) (raised to 5.0 in B-1); got `{def_rhs}`"
    )
```

## A0002

Baseline `rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs:399` / `harness_g_a10_zoom_min_and_default_preserved`. Replacement [F002](#f002). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        no_ws(&def_rhs),
        "Vector2(5.0,5.0)",
        "a10.4: ZOOM_DEFAULT must equal Vector2(5.0,5.0) (raised from 3.0 in B-1); got `{def_rhs}`"
    )
```

## A0003

Baseline `rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs:516` / `harness_g_a15_existing_p14_zeta_regression_literal_updated`. Replacement [F003](#f003). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        compact.contains("Vector2(5.0,5.0)"),
        "a15.3: harness_p14_zeta_zoom_adaptive.rs must carry Vector2(5.0,5.0) \
         (ZOOM_DEFAULT raised from 3.0 in B-1)"
    )
```

## A0004

Baseline `rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs:521` / `harness_g_a15_existing_p14_zeta_regression_literal_updated`. Replacement [F003](#f003). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        !compact.contains("Vector2(4.0,4.0)"),
        "a15.4: harness_p14_zeta_zoom_adaptive.rs must NOT still require Vector2(4.0,4.0)"
    )
```

## A0005

Baseline `rust/crates/sim-test/tests/harness_h_phase_a_input_camera_color.rs:448` / `harness_h_a9_zoom_invariants_unchanged`. Replacement [F004](#f004). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        no_ws(&def_rhs),
        "Vector2(5.0,5.0)",
        "a9.6: ZOOM_DEFAULT must equal Vector2(5.0,5.0) (raised from 3.0 in B-1); got `{def_rhs}`"
    )
```

## A0006

Baseline `rust/crates/sim-test/tests/harness_p11_alpha_agent_renderer.rs:637` / `harness_p11_alpha_shader_modulates_palette_output`. Replacement [F005](#f005). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        multiply_pos > alpha_guard_pos,
        "A17.4: the palette_color.rgb * v_tint.rgb multiply must appear \
         after the `tex.a <= 0.01` alpha guard so transparent fragments \
         are not tinted (multiply_pos={multiply_pos}, alpha_guard_pos={alpha_guard_pos})"
    )
```

## A0007

Baseline `rust/crates/sim-test/tests/harness_p12_alpha_camera_zoom.rs:230` / `harness_p12_alpha_a5_zoom_default_constant`. Replacement [F006](#f006). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        matched,
        "A5: ZOOM_DEFAULT must be declared with `Vector2(5.0, 5.0)` \
         (whitespace tolerant; raised from 3.0× by B-1). Line: `{line}`"
    )
```

## A0008

Baseline `rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs:395` / `harness_p12_beta2_a10_renderer_calls_ffi_snapshot_method`. Replacement [F007](#f007). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains("world_sim.get_construction_snapshot()"),
        "A10: world_renderer.gd must call `world_sim.get_construction_snapshot()`"
    )
```

## A0009

Baseline `rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs:411` / `harness_p12_beta2_a11_reaper_loop_frees_despawned_sprites`. Replacement [F008](#f008). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains("_construction_sprites.keys()"),
        "A11.1: must iterate over `_construction_sprites.keys()`"
    )
```

## A0010

Baseline `rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs:415` / `harness_p12_beta2_a11_reaper_loop_frees_despawned_sprites`. Replacement [F008](#f008). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains("queue_free()"),
        "A11.2: must contain `queue_free()` call"
    )
```

## A0011

Baseline `rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs:419` / `harness_p12_beta2_a11_reaper_loop_frees_despawned_sprites`. Replacement [F008](#f008). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains("_construction_sprites.erase"),
        "A11.3: must contain `_construction_sprites.erase` call"
    )
```

## A0012

Baseline `rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs:494` / `harness_p12_beta2_a14_phase12_alpha_and_beta1_invariants_preserved`. Replacement [F009](#f009). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains("TERRAIN_TILESET_PATH"),
        "A14.3: world_renderer.gd must still declare TERRAIN_TILESET_PATH (Phase 12-β.1)"
    )
```

## A0013

Baseline `rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs:498` / `harness_p12_beta2_a14_phase12_alpha_and_beta1_invariants_preserved`. Replacement [F009](#f009). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains("BUILDING_SPRITE_PATH"),
        "A14.4: world_renderer.gd must still declare BUILDING_SPRITE_PATH (Phase 12-β.1)"
    )
```

## A0014

Baseline `rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs:329` / `harness_p12_beta2_a8_renderer_declares_construction_sprite_path_constant`. Replacement [F010](#f010). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        line.contains("assets/sprites/buildings/"),
        "A8.2: CONSTRUCTION_SPRITE_PATH must reference `assets/sprites/buildings/`; line=`{line}`"
    )
```

## A0015

Baseline `rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs:333` / `harness_p12_beta2_a8_renderer_declares_construction_sprite_path_constant`. Replacement [F010](#f010). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        line.contains(".png"),
        "A8.3: CONSTRUCTION_SPRITE_PATH must end with `.png`; line=`{line}`"
    )
```

## A0016

Baseline `rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs:364` / `harness_p12_beta2_a9_alpha_remap_constants_have_exact_values`. Replacement [F011](#f011). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        (min_parsed - 0.3).abs() < 1e-9,
        "A9.2: CONSTRUCTION_ALPHA_MIN must equal exactly 0.3; got {min_parsed}"
    )
```

## A0017

Baseline `rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs:373` / `harness_p12_beta2_a9_alpha_remap_constants_have_exact_values`. Replacement [F011](#f011). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        (max_parsed - 1.0).abs() < 1e-9,
        "A9.4: CONSTRUCTION_ALPHA_MAX must equal exactly 1.0; got {max_parsed}"
    )
```

## A0018

Baseline `rust/crates/sim-test/tests/harness_p12_beta2_construction_sites.rs:382` / `harness_p12_beta2_a9_alpha_remap_constants_have_exact_values`. Replacement [F011](#f011). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        z_parsed, 5,
        "A9.6: Z_CONSTRUCTION must equal exactly 5; got {z_parsed}"
    )
```

## A0019

Baseline `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:346` / `harness_p12_beta_a10_bootstrap_building_sprite_loaded`. Replacement [F012](#f012). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains(&load_call),
        "A10.2: must call `{load_call}` in non-comment code"
    )
```

## A0020

Baseline `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:374` / `harness_p12_beta_a10_bootstrap_building_sprite_loaded`. Replacement [F012](#f012). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        !tex_var.is_empty(),
        "A10.4: could not parse texture variable name from `{load_line}`"
    )
```

## A0021

Baseline `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:396` / `harness_p12_beta_a10_bootstrap_building_sprite_loaded`. Replacement [F012](#f012). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        !sprite_var.is_empty(),
        "A10.7: could not parse sprite variable name from `{assign_line}`"
    )
```

## A0022

Baseline `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:402` / `harness_p12_beta_a10_bootstrap_building_sprite_loaded`. Replacement [F012](#f012). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains("Sprite2D.new()"),
        "A10.8: must contain a `Sprite2D.new()` for the bootstrap building"
    )
```

## A0023

Baseline `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:408` / `harness_p12_beta_a10_bootstrap_building_sprite_loaded`. Replacement [F012](#f012). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains(&add_child_call),
        "A10.9: must call `add_child({sprite_var})`; missing"
    )
```

## A0024

Baseline `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:472` / `harness_p12_beta_a11_bootstrap_sprite_positioned_at_bootstrap_tile`. Replacement [F013](#f013). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        found_all,
        "A11.2: a `.position = ` assignment must reference ALL of \
         BOOTSTRAP_X, BOOTSTRAP_Y, SPRITE_ORIGIN_X, SPRITE_ORIGIN_Y, TILE_SIZE \
         within a single expression"
    )
```

## A0025

Baseline `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:511` / `harness_p12_beta_a12_z_order_constants_match_spec`. Replacement [F014](#f014). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        line_has_int_value(z_terrain_line, 0),
        "A12.1: Z_TERRAIN must equal exactly 0. Line: `{z_terrain_line}`"
    )
```

## A0026

Baseline `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:517` / `harness_p12_beta_a12_z_order_constants_match_spec`. Replacement [F014](#f014). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        line_has_int_value(z_building_line, 5),
        "A12.2: Z_BUILDING must equal exactly 5. Line: `{z_building_line}`"
    )
```

## A0027

Baseline `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:523` / `harness_p12_beta_a12_z_order_constants_match_spec`. Replacement [F014](#f014). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        line_has_int_value(z_overlay_line, 10),
        "A12.3: Z_OVERLAY must equal exactly 10. Line: `{z_overlay_line}`"
    )
```

## A0028

Baseline `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:539` / `harness_p12_beta_a13_z_order_constants_are_assigned_to_nodes`. Replacement [F015](#f015). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
            stripped.contains(needle),
            "A13: must contain `{needle}` assignment; missing"
        )
```

## A0029

Baseline `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:172` / `harness_p12_beta_a6_world_renderer_loads_terrain_tileset`. Replacement [F016](#f016). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        !const_name.is_empty(),
        "A6.2: could not parse constant name from line `{const_line}`"
    )
```

## A0030

Baseline `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:179` / `harness_p12_beta_a6_world_renderer_loads_terrain_tileset`. Replacement [F016](#f016). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains(&load_pattern),
        "A6.3: world_renderer.gd must contain `{load_pattern}` in non-comment \
         code; constant `{const_name}` was declared but never loaded"
    )
```

## A0031

Baseline `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:210` / `harness_p12_beta_a7_world_renderer_instantiates_tilemaplayer_with_tileset`. Replacement [F017](#f017). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        !var_name.is_empty(),
        "A7.2: could not parse TileMapLayer variable name from `{new_line}`"
    )
```

## A0032

Baseline `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:216` / `harness_p12_beta_a7_world_renderer_instantiates_tilemaplayer_with_tileset`. Replacement [F017](#f017). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains(&tile_set_assign),
        "A7.3: must assign `{var_name}.tile_set = ...`; missing"
    )
```

## A0033

Baseline `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:222` / `harness_p12_beta_a7_world_renderer_instantiates_tilemaplayer_with_tileset`. Replacement [F017](#f017). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains(&add_child_call),
        "A7.4: must call `add_child({var_name})`; missing"
    )
```

## A0034

Baseline `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:259` / `harness_p12_beta_a8_world_renderer_populates_all_grid_cells`. Replacement [F018](#f018). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        found,
        "A8: world_renderer.gd must contain a nested loop over GRID_W and \
         GRID_H whose body calls `set_cell(`"
    )
```

## A0035

Baseline `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:291` / `harness_p12_beta_a9_terrain_placement_is_deterministic`. Replacement [F019](#f019). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        !seed_name.is_empty(),
        "A9.2: could not parse seed constant name from `{seed_const_line}`"
    )
```

## A0036

Baseline `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:298` / `harness_p12_beta_a9_terrain_placement_is_deterministic`. Replacement [F019](#f019). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains(&seed_assign_needle),
        "A9.3: must assign `<rng>.seed = {seed_name}`; missing"
    )
```

## A0037

Baseline `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs:310` / `harness_p12_beta_a9_terrain_placement_is_deterministic`. Replacement [F019](#f019). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        assign_idx < set_cell_idx,
        "A9.6: seed assignment must come before set_cell loop \
         (assign_idx={assign_idx}, set_cell_idx={set_cell_idx})"
    )
```

## A0038

Baseline `rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs:441` / `harness_p12_gamma_a10_world_renderer_polls_ffi_and_reaps_stale_sprites`. Replacement [F020](#f020). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains("world_sim.get_settlement_snapshot()"),
        "A10.1: world_renderer.gd must call `world_sim.get_settlement_snapshot()`"
    )
```

## A0039

Baseline `rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs:445` / `harness_p12_gamma_a10_world_renderer_polls_ffi_and_reaps_stale_sprites`. Replacement [F020](#f020). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains("_furniture_sprites.keys()"),
        "A10.2: world_renderer.gd must iterate `_furniture_sprites.keys()` for reaping"
    )
```

## A0040

Baseline `rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs:449` / `harness_p12_gamma_a10_world_renderer_polls_ffi_and_reaps_stale_sprites`. Replacement [F020](#f020). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains("queue_free()"),
        "A10.3: world_renderer.gd must contain `queue_free()` (reaper loop)"
    )
```

## A0041

Baseline `rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs:512` / `harness_p12_gamma_a12_phase12_alpha_beta1_beta2_invariants_preserved`. Replacement [F021](#f021). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains("TERRAIN_TILESET_PATH"),
        "A12.3: world_renderer.gd must still declare TERRAIN_TILESET_PATH (Phase 12-β.1)"
    )
```

## A0042

Baseline `rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs:516` / `harness_p12_gamma_a12_phase12_alpha_beta1_beta2_invariants_preserved`. Replacement [F021](#f021). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains("BUILDING_SPRITE_PATH"),
        "A12.4: world_renderer.gd must still declare BUILDING_SPRITE_PATH (Phase 12-β.1)"
    )
```

## A0043

Baseline `rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs:539` / `harness_p12_gamma_a12_phase12_alpha_beta1_beta2_invariants_preserved`. Replacement [F021](#f021). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains("CONSTRUCTION_SPRITE_PATH"),
        "A12.8: world_renderer.gd must still declare CONSTRUCTION_SPRITE_PATH (Phase 12-β.2 A3)"
    )
```

## A0044

Baseline `rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs:555` / `harness_p12_gamma_a12_phase12_alpha_beta1_beta2_invariants_preserved`. Replacement [F021](#f021). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        z_parsed, 5,
        "A12.11: Z_CONSTRUCTION must remain exactly 5 (Phase 12-β.2 A3); got {z_parsed}"
    )
```

## A0045

Baseline `rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs:658` / `harness_p12_gamma_a15_world_renderer_reads_formation_arrays`. Replacement [F022](#f022). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
            body.contains(key),
            "A15.2: _update_settlement_furniture must reference `{key}`; body=\n{body}"
        )
```

## A0046

Baseline `rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs:405` / `harness_p12_gamma_a9_world_renderer_declares_furniture_sprite_path_and_z_constants`. Replacement [F023](#f023). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        sprite_line.contains("assets/sprites/buildings/gathering_marker/"),
        "A9.2: FURNITURE_SPRITE_PATH must reference `assets/sprites/buildings/gathering_marker/`; line=`{sprite_line}`"
    )
```

## A0047

Baseline `rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs:409` / `harness_p12_gamma_a9_world_renderer_declares_furniture_sprite_path_and_z_constants`. Replacement [F023](#f023). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        !sprite_line.contains("assets/sprites/furniture/hearth/"),
        "A9.2: FURNITURE_SPRITE_PATH must NOT reference the old `assets/sprites/furniture/hearth/`; line=`{sprite_line}`"
    )
```

## A0048

Baseline `rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs:426` / `harness_p12_gamma_a9_world_renderer_declares_furniture_sprite_path_and_z_constants`. Replacement [F023](#f023). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        (1..=4).contains(&z_parsed),
        "A9.5: Z_FURNITURE must be in (0, 5) exclusive — i.e. ∈ {{1,2,3,4}} — to sit between terrain (z=0) and ConstructionSite (z=5); got {z_parsed}"
    )
```

## A0049

Baseline `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs:484` / `harness_p13_alpha_a11_phase12_beta1_terrain_tileset_preserved`. Replacement [F024](#f024). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        terrain_rhss.len(),
        1,
        "A11.1: TERRAIN_TILESET_PATH must be declared exactly once; got {n}: {terrain_rhss:?}",
        n = terrain_rhss.len()
    )
```

## A0050

Baseline `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs:496` / `harness_p13_alpha_a11_phase12_beta1_terrain_tileset_preserved`. Replacement [F024](#f024). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        (val - 0.65).abs() < 1e-9,
        "A11.4: OVERLAY_ALPHA must equal exactly 0.65; got {val}"
    )
```

## A0051

Baseline `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs:510` / `harness_p13_alpha_a12_phase12_beta2_construction_z_preserved`. Replacement [F025](#f025). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        rhs, "5",
        "A12: Z_CONSTRUCTION RHS must be EXACTLY `5`; got `{rhs}`"
    )
```

## A0052

Baseline `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs:524` / `harness_p13_alpha_a13_phase12_gamma_furniture_z_preserved`. Replacement [F026](#f026). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        rhs, "4",
        "A13: Z_FURNITURE RHS must be EXACTLY `4`; got `{rhs}`"
    )
```

## A0053

Baseline `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs:549` / `harness_p13_alpha_a14_three_distinct_building_sprite_paths`. Replacement [F027](#f027). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        set.len(),
        3,
        "A14.4: BUILDING_SPRITE_PATH, CONSTRUCTION_SPRITE_PATH, FURNITURE_SPRITE_PATH \
         must be pairwise distinct RHSs; got: building={b:?}, construction={c:?}, furniture={f:?}"
    )
```

## A0054

Baseline `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs:208` / `harness_p13_alpha_a1_camera_controller_zoom_default_3x`. Replacement [F028](#f028). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        accepted.contains(&compact.as_str()),
        "A1: ZOOM_DEFAULT RHS must equal exactly one of Vector2(5.0,5.0) | \
         Vector2(5,5) | Vector2(5.0,5) | Vector2(5,5.0) (whitespace-stripped; \
         raised from 3.0 in B-1); got RHS=`{rhs}` compact=`{compact}`"
    )
```

## A0055

Baseline `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs:254` / `harness_p13_alpha_a4_bootstrap_building_path_campfire`. Replacement [F029](#f029). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        rhs, "\"res://assets/sprites/buildings/campfire/1.png\"",
        "A4: BUILDING_SPRITE_PATH RHS must be EXACTLY the campfire string \
         literal with no concatenation or extra tokens; got `{rhs}`"
    )
```

## A0056

Baseline `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs:279` / `harness_p13_alpha_a5_bootstrap_no_longer_points_to_cairn`. Replacement [F030](#f030). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        offenders.is_empty(),
        "A5: no line may bind BUILDING_SPRITE_PATH to the cairn path \
         (replacement must occur, not addition); offenders: {offenders:?}"
    )
```

## A0057

Baseline `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs:295` / `harness_p13_alpha_a6_construction_sprite_path_unchanged`. Replacement [F031](#f031). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        rhs, "\"res://assets/sprites/buildings/cairn/1.png\"",
        "A6: CONSTRUCTION_SPRITE_PATH RHS must be EXACTLY the cairn literal; got `{rhs}`"
    )
```

## A0058

Baseline `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs:312` / `harness_p13_alpha_a7_furniture_sprite_path_unchanged`. Replacement [F032](#f032). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        rhs, "\"res://assets/sprites/buildings/gathering_marker/1.png\"",
        "A7: FURNITURE_SPRITE_PATH RHS must be EXACTLY the gathering_marker literal; got `{rhs}`"
    )
```

## A0059

Baseline `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:414` / `harness_p13_beta_a11_phase13_alpha_zoom_default_3x_preserved`. Replacement [F033](#f033). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        accepted.contains(&compact.as_str()),
        "A11: ZOOM_DEFAULT must equal Vector2(5.0, 5.0) (raised from 3.0 in B-1); got `{rhs}`"
    )
```

## A0060

Baseline `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:427` / `harness_p13_beta_a12_phase13_alpha_bootstrap_campfire_preserved`. Replacement [F034](#f034). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        rhs, "\"res://assets/sprites/buildings/campfire/1.png\"",
        "A12: BUILDING_SPRITE_PATH must remain the campfire literal (Phase 13-α invariant); got `{rhs}`"
    )
```

## A0061

Baseline `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:441` / `harness_p13_beta_a13_phase12_beta1_terrain_tileset_preserved`. Replacement [F035](#f035). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        terrain_rhss.len(),
        1,
        "A13.1: TERRAIN_TILESET_PATH must be declared exactly once; got {n}: {terrain_rhss:?}",
        n = terrain_rhss.len()
    )
```

## A0062

Baseline `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:452` / `harness_p13_beta_a13_phase12_beta1_terrain_tileset_preserved`. Replacement [F035](#f035). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        (val - 0.65).abs() < 1e-9,
        "A13.4: OVERLAY_ALPHA must equal exactly 0.65; got {val}"
    )
```

## A0063

Baseline `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:465` / `harness_p13_beta_a14_phase12_beta2_construction_z_5_preserved`. Replacement [F036](#f036). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        rhs, "5",
        "A14: Z_CONSTRUCTION must remain `5` (Phase 12-β.2 A3 invariant); got `{rhs}`"
    )
```

## A0064

Baseline `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:478` / `harness_p13_beta_a15_phase12_gamma_furniture_z_4_preserved`. Replacement [F037](#f037). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        rhs, "4",
        "A15: Z_FURNITURE must remain `4` (Phase 12-γ invariant); got `{rhs}`"
    )
```

## A0065

Baseline `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:143` / `harness_p13_beta_a1_resource_sprite_path_constant_declared`. Replacement [F038](#f038). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        rhs, "\"res://assets/sprites/furniture/storage_pit/1.png\"",
        "A1: RESOURCE_SPRITE_PATH RHS must be EXACTLY the storage_pit literal; got `{rhs}`"
    )
```

## A0066

Baseline `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:156` / `harness_p13_beta_a2_z_resource_constant_equals_3`. Replacement [F039](#f039). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        rhs, "3",
        "A2: Z_RESOURCE RHS must be EXACTLY `3`; got `{rhs}`"
    )
```

## A0067

Baseline `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:169` / `harness_p13_beta_a3_resource_count_equals_20`. Replacement [F040](#f040). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        rhs, "20",
        "A3: RESOURCE_COUNT RHS must be EXACTLY `20`; got `{rhs}`"
    )
```

## A0068

Baseline `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:182` / `harness_p13_beta_a4_resource_seed_constant_declared`. Replacement [F041](#f041). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        rhs, "88675123",
        "A4: RESOURCE_SEED RHS must be EXACTLY `88675123`; got `{rhs}`"
    )
```

## A0069

Baseline `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:196` / `harness_p13_beta_a5_resource_placement_uses_seeded_rng`. Replacement [F042](#f042). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains("RandomNumberGenerator.new()"),
        "A5.1: resource block must call `RandomNumberGenerator.new()`; not found in stripped source"
    )
```

## A0070

Baseline `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:203` / `harness_p13_beta_a5_resource_placement_uses_seeded_rng`. Replacement [F042](#f042). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        compact.contains(".seed=RESOURCE_SEED"),
        "A5.2: resource block must contain an assignment `<rng>.seed = RESOURCE_SEED`; \
         not found in whitespace-stripped source"
    )
```

## A0071

Baseline `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:210` / `harness_p13_beta_a5_resource_placement_uses_seeded_rng`. Replacement [F042](#f042). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        compact.contains("randi_range(0,GRID_W-1)"),
        "A5.3: resource block must contain a `randi_range(0, GRID_W - 1)` call \
         (mirroring grid-bound pattern); not found in whitespace-stripped source"
    )
```

## A0072

Baseline `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:277` / `harness_p13_beta_a7_z_order_chain_invariant`. Replacement [F043](#f043). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(z_terrain, 0, "A7.6: Z_TERRAIN must be 0; got {z_terrain}")
```

## A0073

Baseline `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:278` / `harness_p13_beta_a7_z_order_chain_invariant`. Replacement [F043](#f043). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(z_resource, 3, "A7.7: Z_RESOURCE must be 3; got {z_resource}")
```

## A0074

Baseline `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:279` / `harness_p13_beta_a7_z_order_chain_invariant`. Replacement [F043](#f043). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(z_furniture, 4, "A7.8: Z_FURNITURE must be 4; got {z_furniture}")
```

## A0075

Baseline `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:280` / `harness_p13_beta_a7_z_order_chain_invariant`. Replacement [F043](#f043). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        z_construction, 5,
        "A7.9: Z_CONSTRUCTION must be 5; got {z_construction}"
    )
```

## A0076

Baseline `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:284` / `harness_p13_beta_a7_z_order_chain_invariant`. Replacement [F043](#f043). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(z_overlay, 10, "A7.10: Z_OVERLAY must be 10; got {z_overlay}")
```

## A0077

Baseline `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs:286` / `harness_p13_beta_a7_z_order_chain_invariant`. Replacement [F043](#f043). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        z_terrain < z_resource
            && z_resource < z_furniture
            && z_furniture < z_construction
            && z_construction < z_overlay,
        "A7.11: z-order chain violated: terrain({z_terrain}) < resource({z_resource}) < \
         furniture({z_furniture}) < construction({z_construction}) < overlay({z_overlay})"
    )
```

## A0078

Baseline `rust/crates/sim-test/tests/harness_p13_delta_basic_hud.rs:623` / `harness_p13_delta_a14_phase13_alpha_default_zoom_3x_unchanged`. Replacement [F044](#f044). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        accepted.contains(&compact.as_str()),
        "A14: ZOOM_DEFAULT must equal Vector2(5.0, 5.0) (raised from 3.0 in B-1); got `{rhs}`"
    )
```

## A0079

Baseline `rust/crates/sim-test/tests/harness_p13_delta_basic_hud.rs:636` / `harness_p13_delta_a15_phase13_alpha_bootstrap_campfire_unchanged`. Replacement [F045](#f045). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        rhs, "\"res://assets/sprites/buildings/campfire/1.png\"",
        "A15: BUILDING_SPRITE_PATH must remain the campfire literal (Phase 13-α invariant); got `{rhs}`"
    )
```

## A0080

Baseline `rust/crates/sim-test/tests/harness_p13_delta_basic_hud.rs:650` / `harness_p13_delta_a16_phase13_beta_resource_constants_unchanged`. Replacement [F046](#f046). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        resource_path.contains("res://"),
        "A16.2: RESOURCE_SPRITE_PATH must be a res:// path; got `{resource_path}`"
    )
```

## A0081

Baseline `rust/crates/sim-test/tests/harness_p13_delta_basic_hud.rs:656` / `harness_p13_delta_a16_phase13_beta_resource_constants_unchanged`. Replacement [F046](#f046). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        z_resource, "3",
        "A16.4: Z_RESOURCE must equal 3; got `{z_resource}`"
    )
```

## A0082

Baseline `rust/crates/sim-test/tests/harness_p13_delta_basic_hud.rs:662` / `harness_p13_delta_a16_phase13_beta_resource_constants_unchanged`. Replacement [F046](#f046). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        resource_count, "20",
        "A16.6: RESOURCE_COUNT must equal 20; got `{resource_count}`"
    )
```

## A0083

Baseline `rust/crates/sim-test/tests/harness_p13_delta_basic_hud.rs:259` / `harness_p13_delta_a3_hud_topbar_declares_four_label_members`. Replacement [F047](#f047). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        missing.is_empty(),
        "A3: hud_topbar.gd must declare four Label members containing each of tick/agent/settlement/site \
         (case-insensitive); missing={missing:?}; found Label idents={label_idents:?}"
    )
```

## A0084

Baseline `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:485` / `harness_p13_epsilon_a11_phase13_alpha_default_zoom_3x_preserved`. Replacement [F048](#f048). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        accepted.contains(&compact.as_str()),
        "A11: ZOOM_DEFAULT must equal Vector2(5.0, 5.0) (raised from 3.0 in B-1); got `{rhs}`"
    )
```

## A0085

Baseline `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:501` / `harness_p13_epsilon_a12_phase13_alpha_centre_bootstrap_preserved`. Replacement [F049](#f049). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(x, "32", "A12.4: BOOTSTRAP_X must be `32`; got `{x}`")
```

## A0086

Baseline `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:502` / `harness_p13_epsilon_a12_phase13_alpha_centre_bootstrap_preserved`. Replacement [F049](#f049). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(y, "32", "A12.5: BOOTSTRAP_Y must be `32`; got `{y}`")
```

## A0087

Baseline `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:503` / `harness_p13_epsilon_a12_phase13_alpha_centre_bootstrap_preserved`. Replacement [F049](#f049). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(r, "8", "A12.6: BOOTSTRAP_RADIUS must be `8`; got `{r}`")
```

## A0088

Baseline `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:513` / `harness_p13_epsilon_a13_phase13_beta_resource_constants_preserved`. Replacement [F050](#f050). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        path, "\"res://assets/sprites/furniture/storage_pit/1.png\"",
        "A13.2: RESOURCE_SPRITE_PATH must remain storage_pit literal; got `{path}`"
    )
```

## A0089

Baseline `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:518` / `harness_p13_epsilon_a13_phase13_beta_resource_constants_preserved`. Replacement [F050](#f050). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(z, "3", "A13.4: Z_RESOURCE must equal 3; got `{z}`")
```

## A0090

Baseline `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:520` / `harness_p13_epsilon_a13_phase13_beta_resource_constants_preserved`. Replacement [F050](#f050). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(count, "20", "A13.6: RESOURCE_COUNT must equal 20; got `{count}`")
```

## A0091

Baseline `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:522` / `harness_p13_epsilon_a13_phase13_beta_resource_constants_preserved`. Replacement [F050](#f050). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        seed, "88675123",
        "A13.8: RESOURCE_SEED must equal 88675123; got `{seed}`"
    )
```

## A0092

Baseline `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:254` / `harness_p13_epsilon_a1_bootstrap_x_left_constant_declared`. Replacement [F051](#f051). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        rhs, "24",
        "A1: BOOTSTRAP_X_LEFT RHS must be EXACTLY `24`; got `{rhs}`"
    )
```

## A0093

Baseline `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:267` / `harness_p13_epsilon_a2_bootstrap_x_right_constant_declared`. Replacement [F052](#f052). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        rhs, "40",
        "A2: BOOTSTRAP_X_RIGHT RHS must be EXACTLY `40`; got `{rhs}`"
    )
```

## A0094

Baseline `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:280` / `harness_p13_epsilon_a3_three_on_building_placed_calls_total`. Replacement [F053](#f053). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        calls.len(),
        3,
        "A3: stripped world_renderer.gd must contain EXACTLY 3 `on_building_placed(` \
         call sites; got {n}: args={calls:?}",
        n = calls.len()
    )
```

## A0095

Baseline `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:297` / `harness_p13_epsilon_a4_on_building_placed_arg_signatures`. Replacement [F054](#f054). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        calls.len(),
        3,
        "A4.0: precondition — must have 3 calls (covered by A3)"
    )
```

## A0096

Baseline `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:330` / `harness_p13_epsilon_a4_on_building_placed_arg_signatures`. Replacement [F054](#f054). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
            *ok,
            "A4: expected on_building_placed tuple {tuple:?} not found among observed call args: {observed:?}",
            tuple = want[i]
        )
```

## A0097

Baseline `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:350` / `harness_p13_epsilon_a5_extra_sprite_block_reuses_building_tex`. Replacement [F055](#f055). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        compact.contains(".texture=building_tex"),
        "A5.2: extra_x loop body must contain `.texture = building_tex` (reuse, no new \
         load); body=\n{body}"
    )
```

## A0098

Baseline `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:356` / `harness_p13_epsilon_a5_extra_sprite_block_reuses_building_tex`. Replacement [F055](#f055). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        !compact.contains("load(\"res://assets/sprites/buildings/campfire/"),
        "A5.3: extra_x loop body must NOT reload the campfire asset; reuse the \
         existing `building_tex` variable. body=\n{body}"
    )
```

## A0099

Baseline `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:371` / `harness_p13_epsilon_a6_extra_sprite_z_index_is_z_building`. Replacement [F056](#f056). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        compact.contains(".z_index=Z_BUILDING"),
        "A6.2: extra_x loop body must contain `.z_index = Z_BUILDING` (Phase 12-β.1 \
         canonical building layer constant); body=\n{body}"
    )
```

## A0100

Baseline `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs:385` / `harness_p13_epsilon_a7_extra_sprite_y_position_uses_bootstrap_y`. Replacement [F057](#f057). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        body.contains("BOOTSTRAP_Y"),
        "A7.2: extra_x loop body must reference `BOOTSTRAP_Y` in the Vector2 y \
         component so all three buildings share the same horizontal row; body=\n{body}"
    )
```

## A0101

Baseline `rust/crates/sim-test/tests/harness_p13_gamma_interaction_cues.rs:546` / `harness_p13_gamma_a13_phase13_alpha_zoom_default_3x_preserved`. Replacement [F058](#f058). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        accepted.contains(&compact.as_str()),
        "A13: ZOOM_DEFAULT must equal Vector2(5.0, 5.0) (raised from 3.0 in B-1); got `{rhs}`"
    )
```

## A0102

Baseline `rust/crates/sim-test/tests/harness_p13_gamma_interaction_cues.rs:559` / `harness_p13_gamma_a14_phase13_alpha_bootstrap_campfire_preserved`. Replacement [F059](#f059). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        rhs, "\"res://assets/sprites/buildings/campfire/1.png\"",
        "A14: BUILDING_SPRITE_PATH must remain the campfire literal (Phase 13-α invariant); got `{rhs}`"
    )
```

## A0103

Baseline `rust/crates/sim-test/tests/harness_p13_gamma_interaction_cues.rs:573` / `harness_p13_gamma_a15_phase13_beta_resource_constants_preserved`. Replacement [F060](#f060). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        resource_path, "\"res://assets/sprites/furniture/storage_pit/1.png\"",
        "A15.2: RESOURCE_SPRITE_PATH must remain the storage_pit literal; got `{resource_path}`"
    )
```

## A0104

Baseline `rust/crates/sim-test/tests/harness_p13_gamma_interaction_cues.rs:579` / `harness_p13_gamma_a15_phase13_beta_resource_constants_preserved`. Replacement [F060](#f060). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(z_resource, "3", "A15.4: Z_RESOURCE must equal 3; got `{z_resource}`")
```

## A0105

Baseline `rust/crates/sim-test/tests/harness_p13_gamma_interaction_cues.rs:582` / `harness_p13_gamma_a15_phase13_beta_resource_constants_preserved`. Replacement [F060](#f060). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        resource_count, "20",
        "A15.6: RESOURCE_COUNT must equal 20; got `{resource_count}`"
    )
```

## A0106

Baseline `rust/crates/sim-test/tests/harness_p13_gamma_interaction_cues.rs:588` / `harness_p13_gamma_a15_phase13_beta_resource_constants_preserved`. Replacement [F060](#f060). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        resource_seed, "88675123",
        "A15.8: RESOURCE_SEED must equal 88675123; got `{resource_seed}`"
    )
```

## A0107

Baseline `rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs:514` / `harness_p14_alpha_a15_camera_zoom_invariants_preserved`. Replacement [F061](#f061). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        accepted.contains(&default_compact.as_str()),
        "A15.6: ZOOM_DEFAULT must equal Vector2(5.0, 5.0) (raised from 3.0 in B-1); got `{zoom_default}`"
    )
```

## A0108

Baseline `rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs:531` / `harness_p14_alpha_a16_phase13_bootstrap_three_campfires_preserved`. Replacement [F062](#f062). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(x, "32", "A16.6: BOOTSTRAP_X must be `32`; got `{x}`")
```

## A0109

Baseline `rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs:532` / `harness_p14_alpha_a16_phase13_bootstrap_three_campfires_preserved`. Replacement [F062](#f062). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(y, "32", "A16.7: BOOTSTRAP_Y must be `32`; got `{y}`")
```

## A0110

Baseline `rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs:533` / `harness_p14_alpha_a16_phase13_bootstrap_three_campfires_preserved`. Replacement [F062](#f062). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(xl, "24", "A16.8: BOOTSTRAP_X_LEFT must be `24`; got `{xl}`")
```

## A0111

Baseline `rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs:534` / `harness_p14_alpha_a16_phase13_bootstrap_three_campfires_preserved`. Replacement [F062](#f062). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(xr, "40", "A16.9: BOOTSTRAP_X_RIGHT must be `40`; got `{xr}`")
```

## A0112

Baseline `rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs:535` / `harness_p14_alpha_a16_phase13_bootstrap_three_campfires_preserved`. Replacement [F062](#f062). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        path.contains("campfire/1.png"),
        "A16.10: BUILDING_SPRITE_PATH must reference campfire/1.png; got `{path}`"
    )
```

## A0113

Baseline `rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs:548` / `harness_p14_alpha_a17_phase13_beta_resource_constants_preserved`. Replacement [F063](#f063). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        path.contains("storage_pit/1.png"),
        "A17.2: RESOURCE_SPRITE_PATH must reference storage_pit/1.png; got `{path}`"
    )
```

## A0114

Baseline `rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs:553` / `harness_p14_alpha_a17_phase13_beta_resource_constants_preserved`. Replacement [F063](#f063). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(z, "3", "A17.4: Z_RESOURCE must equal 3; got `{z}`")
```

## A0115

Baseline `rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs:555` / `harness_p14_alpha_a17_phase13_beta_resource_constants_preserved`. Replacement [F063](#f063). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(count, "20", "A17.6: RESOURCE_COUNT must equal 20; got `{count}`")
```

## A0116

Baseline `rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs:557` / `harness_p14_alpha_a17_phase13_beta_resource_constants_preserved`. Replacement [F063](#f063). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        seed, "88675123",
        "A17.8: RESOURCE_SEED must equal 88675123; got `{seed}`"
    )
```

## A0117

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:625` / `harness_p14_beta_a10_water_modulate_branch_present`. Replacement [F064](#f064). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        after.contains("RESOURCE_WATER_TINT"),
        "A10.3: `RESOURCE_WATER_TINT` must appear AFTER the `{branch_form}` branch; \
         compact-after=`{after}`"
    )
```

## A0118

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:641` / `harness_p14_beta_a11_village_fixture_paths_has_exactly_four_literals`. Replacement [F065](#f065). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        entries.len(),
        4,
        "A11.2: VILLAGE_FIXTURE_PATHS must contain EXACTLY 4 entries; got {n}: {entries:?}",
        n = entries.len()
    )
```

## A0119

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:654` / `harness_p14_beta_a11_village_fixture_paths_has_exactly_four_literals`. Replacement [F065](#f065). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
            &entries[i], want_v,
            "A11.3: VILLAGE_FIXTURE_PATHS[{i}] must equal `{want_v}`; got `{e}`",
            e = entries[i]
        )
```

## A0120

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:671` / `harness_p14_beta_a12_village_fixture_positions_match_diamond_layout`. Replacement [F066](#f066). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        entries.len(),
        4,
        "A12.2: VILLAGE_FIXTURE_POSITIONS must contain EXACTLY 4 entries; got {n}: {entries:?}",
        n = entries.len()
    )
```

## A0121

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:685` / `harness_p14_beta_a12_village_fixture_positions_match_diamond_layout`. Replacement [F066](#f066). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
            got, *want_v,
            "A12.3: VILLAGE_FIXTURE_POSITIONS[{i}] must equal `{want_v}`; got `{e}`",
            e = entries[i]
        )
```

## A0122

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:700` / `harness_p14_beta_a13_z_village_fixture_equals_five`. Replacement [F067](#f067). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        rhs, "5",
        "A13.2: Z_VILLAGE_FIXTURE must equal `5`; got `{rhs}`"
    )
```

## A0123

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:725` / `harness_p14_beta_a14_village_fixture_placement_loop_present`. Replacement [F068](#f068). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        total_hits, 1,
        "A14.1: exactly ONE village fixture placement loop header must exist \
         (`for fi in VILLAGE_FIXTURE_PATHS.size()` or `for fi in 4`); found {total_hits} \
         total matches"
    )
```

## A0124

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:758` / `harness_p14_beta_a14_village_fixture_placement_loop_present`. Replacement [F068](#f068). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        body_only.contains("VILLAGE_FIXTURE_PATHS[fi]")
            || body_only.contains("VILLAGE_FIXTURE_PATHS [fi]"),
        "A14.4: loop body must reference `VILLAGE_FIXTURE_PATHS[fi]`; header=`{header}`; \
         body=\n{body_only}"
    )
```

## A0125

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:764` / `harness_p14_beta_a14_village_fixture_placement_loop_present`. Replacement [F068](#f068). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        body_only.contains("VILLAGE_FIXTURE_POSITIONS[fi]")
            || body_only.contains("VILLAGE_FIXTURE_POSITIONS [fi]"),
        "A14.5: loop body must reference `VILLAGE_FIXTURE_POSITIONS[fi]`; header=`{header}`; \
         body=\n{body_only}"
    )
```

## A0126

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:770` / `harness_p14_beta_a14_village_fixture_placement_loop_present`. Replacement [F068](#f068). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        body_only.contains("add_child(fixture_sprite)"),
        "A14.6: loop body must call `add_child(fixture_sprite)`; header=`{header}`; \
         body=\n{body_only}"
    )
```

## A0127

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:784` / `harness_p14_beta_a15_phase_13_beta_resource_constants_preserved`. Replacement [F069](#f069). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        path, "\"res://assets/sprites/furniture/storage_pit/1.png\"",
        "A15.2: RESOURCE_SPRITE_PATH must remain exactly `storage_pit/1.png`; got `{path}`"
    )
```

## A0128

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:789` / `harness_p14_beta_a15_phase_13_beta_resource_constants_preserved`. Replacement [F069](#f069). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(count, "20", "A15.4: RESOURCE_COUNT must be 20; got `{count}`")
```

## A0129

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:791` / `harness_p14_beta_a15_phase_13_beta_resource_constants_preserved`. Replacement [F069](#f069). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        seed, "88675123",
        "A15.6: RESOURCE_SEED must be 88675123; got `{seed}`"
    )
```

## A0130

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:796` / `harness_p14_beta_a15_phase_13_beta_resource_constants_preserved`. Replacement [F069](#f069). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(z, "3", "A15.8: Z_RESOURCE must be 3; got `{z}`")
```

## A0131

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:806` / `harness_p14_beta_a16_phase_13_alpha_bootstrap_constants_preserved`. Replacement [F070](#f070). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        path, "\"res://assets/sprites/buildings/campfire/1.png\"",
        "A16.2: BUILDING_SPRITE_PATH must remain exact campfire literal; got `{path}`"
    )
```

## A0132

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:814` / `harness_p14_beta_a16_phase_13_alpha_bootstrap_constants_preserved`. Replacement [F070](#f070). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(bx, "32", "A16.7: BOOTSTRAP_X must be 32; got `{bx}`")
```

## A0133

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:815` / `harness_p14_beta_a16_phase_13_alpha_bootstrap_constants_preserved`. Replacement [F070](#f070). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(by, "32", "A16.8: BOOTSTRAP_Y must be 32; got `{by}`")
```

## A0134

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:816` / `harness_p14_beta_a16_phase_13_alpha_bootstrap_constants_preserved`. Replacement [F070](#f070). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(bxl, "24", "A16.9: BOOTSTRAP_X_LEFT must be 24; got `{bxl}`")
```

## A0135

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:817` / `harness_p14_beta_a16_phase_13_alpha_bootstrap_constants_preserved`. Replacement [F070](#f070). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        bxr, "40",
        "A16.10: BOOTSTRAP_X_RIGHT must be 40; got `{bxr}`"
    )
```

## A0136

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:831` / `harness_p14_beta_a17_phase_12_construction_constants_preserved`. Replacement [F071](#f071). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        path, "\"res://assets/sprites/buildings/cairn/1.png\"",
        "A17.2: CONSTRUCTION_SPRITE_PATH must equal EXACT `buildings/cairn/1.png` literal; \
         got `{path}`"
    )
```

## A0137

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:837` / `harness_p14_beta_a17_phase_12_construction_constants_preserved`. Replacement [F071](#f071). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(z, "5", "A17.4: Z_CONSTRUCTION must be 5; got `{z}`")
```

## A0138

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:851` / `harness_p14_beta_a18_phase_12_gamma_furniture_constants_preserved`. Replacement [F072](#f072). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        path, "\"res://assets/sprites/buildings/gathering_marker/1.png\"",
        "A18.2: FURNITURE_SPRITE_PATH must equal exact `buildings/gathering_marker/1.png`; got `{path}`"
    )
```

## A0139

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:856` / `harness_p14_beta_a18_phase_12_gamma_furniture_constants_preserved`. Replacement [F072](#f072). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(z, "4", "A18.4: Z_FURNITURE must be 4; got `{z}`")
```

## A0140

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:363` / `harness_p14_beta_a1_resource_type_paths_array_declared`. Replacement [F073](#f073). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        total_token_hits, 1,
        "A1.1: exactly ONE typed `const RESOURCE_TYPE_PATHS: Array = [` declaration token \
         must exist in the stripped source; the inferred-type `:=` form is REJECTED. \
         Found {total_token_hits} matches across the canonical typed declaration forms \
         {decl_tokens:?}"
    )
```

## A0141

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:383` / `harness_p14_beta_a1_resource_type_paths_array_declared`. Replacement [F073](#f073). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        walrus_hits, 0,
        "A1.1b: the inferred-type `const RESOURCE_TYPE_PATHS := [` form is FORBIDDEN; \
         the plan requires the explicit `: Array =` type annotation. Found {walrus_hits} \
         walrus matches across forms {walrus_forms:?}"
    )
```

## A0142

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:392` / `harness_p14_beta_a1_resource_type_paths_array_declared`. Replacement [F073](#f073). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        rhss.len(),
        1,
        "A1.2: cross-check — find_decl_rhss must also yield exactly 1 \
         RESOURCE_TYPE_PATHS declaration; got {n}: {rhss:?}",
        n = rhss.len()
    )
```

## A0143

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:403` / `harness_p14_beta_a1_resource_type_paths_array_declared`. Replacement [F073](#f073). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        !entries.is_empty(),
        "A1.4: RESOURCE_TYPE_PATHS array literal must contain at least one entry; got 0"
    )
```

## A0144

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:421` / `harness_p14_beta_a2_resource_type_paths_has_exactly_five_entries`. Replacement [F074](#f074). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        entries.len(),
        5,
        "A2.2: RESOURCE_TYPE_PATHS must contain EXACTLY 5 entries; got {n}: {entries:?}",
        n = entries.len()
    )
```

## A0145

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:439` / `harness_p14_beta_a3_resource_type_paths_index_0_references_legacy_constant`. Replacement [F075](#f075). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        !entries.is_empty(),
        "A3.2: RESOURCE_TYPE_PATHS must have at least 1 entry"
    )
```

## A0146

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:444` / `harness_p14_beta_a3_resource_type_paths_index_0_references_legacy_constant`. Replacement [F075](#f075). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        entry0, "RESOURCE_SPRITE_PATH",
        "A3.3: RESOURCE_TYPE_PATHS[0] must be the bare identifier `RESOURCE_SPRITE_PATH` \
         (NOT the inlined string literal); got `{entry0}`"
    )
```

## A0147

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:460` / `harness_p14_beta_a4_resource_type_paths_index_1_wood_literal`. Replacement [F076](#f076). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(entries.len() >= 2, "A4.2: need entry at index 1")
```

## A0148

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:462` / `harness_p14_beta_a4_resource_type_paths_index_1_wood_literal`. Replacement [F076](#f076). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        entry1, "\"res://assets/sprites/furniture/workbench/1.png\"",
        "A4.3: RESOURCE_TYPE_PATHS[1] must equal exact Wood literal; got `{entry1}`"
    )
```

## A0149

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:477` / `harness_p14_beta_a5_resource_type_paths_index_2_stone_literal`. Replacement [F077](#f077). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(entries.len() >= 3, "A5.2: need entry at index 2")
```

## A0150

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:479` / `harness_p14_beta_a5_resource_type_paths_index_2_stone_literal`. Replacement [F077](#f077). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        entry2, "\"res://assets/sprites/walls/limestone/1.png\"",
        "A5.3: RESOURCE_TYPE_PATHS[2] must equal exact Stone literal; got `{entry2}`"
    )
```

## A0151

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:494` / `harness_p14_beta_a6_resource_type_paths_index_3_water_literal`. Replacement [F078](#f078). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(entries.len() >= 4, "A6.2: need entry at index 3")
```

## A0152

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:496` / `harness_p14_beta_a6_resource_type_paths_index_3_water_literal`. Replacement [F078](#f078). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        entry3, "\"res://assets/sprites/floors/stone_slab/1.png\"",
        "A6.3: RESOURCE_TYPE_PATHS[3] must equal exact Water literal; got `{entry3}`"
    )
```

## A0153

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:511` / `harness_p14_beta_a7_resource_type_paths_index_4_food_literal`. Replacement [F079](#f079). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(entries.len() >= 5, "A7.2: need entry at index 4")
```

## A0154

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:513` / `harness_p14_beta_a7_resource_type_paths_index_4_food_literal`. Replacement [F079](#f079). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        entry4, "\"res://assets/sprites/furniture/hearth/2.png\"",
        "A7.3: RESOURCE_TYPE_PATHS[4] must equal exact Food literal; got `{entry4}`"
    )
```

## A0155

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:535` / `harness_p14_beta_a8_resource_water_tint_constant_value`. Replacement [F080](#f080). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        accepted.contains(&compact.as_str()),
        "A8.2: RESOURCE_WATER_TINT must equal Color(0.45, 0.65, 1.0, 1.0); got `{rhs}`"
    )
```

## A0156

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:553` / `harness_p14_beta_a9_resource_scatter_loop_uses_modulo_cycle_and_indexed_iterator`. Replacement [F081](#f081). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        block.starts_with("for i in RESOURCE_COUNT"),
        "A9.2: loop header must be `for i in RESOURCE_COUNT` (NOT `for _i in ...`); \
         block prefix=`{}`",
        &block[..block.len().min(40)]
    )
```

## A0157

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:562` / `harness_p14_beta_a9_resource_scatter_loop_uses_modulo_cycle_and_indexed_iterator`. Replacement [F081](#f081). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        has_size_form || has_literal_form,
        "A9.3: loop body must contain `i % RESOURCE_TYPE_PATHS.size()` or `i % 5`; block=\n{block}"
    )
```

## A0158

Baseline `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs:589` / `harness_p14_beta_a9_resource_scatter_loop_uses_modulo_cycle_and_indexed_iterator`. Replacement [F081](#f081). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        continue_abs < add_child_abs,
        "A9.7: the `continue` (offset {continue_abs}) must come BEFORE \
         `add_child(res_sprite)` (offset {add_child_abs}) so null textures are \
         skipped; block=\n{block}"
    )
```

## A0159

Baseline `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs:585` / `harness_p14_delta_a14_phase_14_beta_resource_registry_preserved`. Replacement [F082](#f082). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        count.trim(),
        "20",
        "A14.1: RESOURCE_COUNT must equal 20; got `{count}`"
    )
```

## A0160

Baseline `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs:592` / `harness_p14_delta_a14_phase_14_beta_resource_registry_preserved`. Replacement [F082](#f082). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        seed.trim(),
        "88675123",
        "A14.2: RESOURCE_SEED must equal 88675123; got `{seed}`"
    )
```

## A0161

Baseline `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs:617` / `harness_p14_delta_a14_phase_14_beta_resource_registry_preserved`. Replacement [F082](#f082). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        depth, 0,
        "A14.5: RESOURCE_TYPE_PATHS array must terminate with `]`"
    )
```

## A0162

Baseline `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs:629` / `harness_p14_delta_a14_phase_14_beta_resource_registry_preserved`. Replacement [F082](#f082). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        entries.len(),
        5,
        "A14.6: RESOURCE_TYPE_PATHS must contain exactly 5 entries; got {} ({entries:?})",
        entries.len()
    )
```

## A0163

Baseline `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs:662` / `harness_p14_delta_a16_village_fixture_constants_preserved_in_world_renderer`. Replacement [F083](#f083). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        n >= 1,
        "A16: world_renderer.gd must keep at least one `VILLAGE_FIXTURE_` identifier; got {n}"
    )
```

## A0164

Baseline `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs:692` / `harness_p14_delta_a17_hud_topbar_file_byte_or_line_count_unchanged_against_pre_phase_14_delta_head`. Replacement [F084](#f084). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        got_lines, EXPECTED_LINES,
        "A17.2: hud_topbar.gd line count must equal pre-δ HEAD ({EXPECTED_LINES}); got {got_lines}"
    )
```

## A0165

Baseline `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs:696` / `harness_p14_delta_a17_hud_topbar_file_byte_or_line_count_unchanged_against_pre_phase_14_delta_head`. Replacement [F084](#f084). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        got_hash, EXPECTED_HASH,
        "A17.3: hud_topbar.gd SHA-256 must equal pre-δ HEAD `{EXPECTED_HASH}`; got `{got_hash}`. \
         Phase 13-δ requires hud_topbar.gd to be absolutely untouched."
    )
```

## A0166

Baseline `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs:309` / `harness_p14_delta_a2_ticks_per_day_constant_present`. Replacement [F085](#f085). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(n, 100, "A2: TICKS_PER_DAY must equal 100; got {n}")
```

## A0167

Baseline `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs:321` / `harness_p14_delta_a3_days_per_year_constant_present`. Replacement [F086](#f086). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(n, 30, "A3: DAYS_PER_YEAR must equal 30; got {n}")
```

## A0168

Baseline `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs:338` / `harness_p14_delta_a4_resource_count_consistency_with_phase_14_beta`. Replacement [F087](#f087). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(types, 5, "A4: RESOURCE_TYPES_COUNT must equal 5; got {types}")
```

## A0169

Baseline `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs:339` / `harness_p14_delta_a4_resource_count_consistency_with_phase_14_beta`. Replacement [F087](#f087). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(per, 4, "A4: RESOURCE_PER_TYPE must equal 4; got {per}")
```

## A0170

Baseline `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs:340` / `harness_p14_delta_a4_resource_count_consistency_with_phase_14_beta`. Replacement [F087](#f087). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        types * per,
        20,
        "A4: RESOURCE_TYPES_COUNT * RESOURCE_PER_TYPE must equal 20; got {}",
        types * per
    )
```

## A0171

Baseline `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs:357` / `harness_p14_delta_a5_resource_labels_array_exact_order`. Replacement [F088](#f088). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        compact, want,
        "A5: RESOURCE_LABELS must equal {want} (whitespace-collapsed); got `{compact}`"
    )
```

## A0172

Baseline `rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs:541` / `harness_p14_epsilon_a14_regression_guard_world_renderer_phase_14_beta_intact`. Replacement [F089](#f089). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains("RESOURCE_TYPE_PATHS"),
        "A14.1: world_renderer.gd must keep RESOURCE_TYPE_PATHS"
    )
```

## A0173

Baseline `rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs:545` / `harness_p14_epsilon_a14_regression_guard_world_renderer_phase_14_beta_intact`. Replacement [F089](#f089). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains("VILLAGE_FIXTURE_"),
        "A14.2: world_renderer.gd must keep at least one VILLAGE_FIXTURE_ identifier"
    )
```

## A0174

Baseline `rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs:553` / `harness_p14_epsilon_a14_regression_guard_world_renderer_phase_14_beta_intact`. Replacement [F089](#f089). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(count, 20, "A14.4: RESOURCE_COUNT must equal 20; got {count}")
```

## A0175

Baseline `rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs:558` / `harness_p14_epsilon_a14_regression_guard_world_renderer_phase_14_beta_intact`. Replacement [F089](#f089). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        seed, 88675123,
        "A14.6: RESOURCE_SEED must equal 88675123; got {seed}"
    )
```

## A0176

Baseline `rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs:285` / `harness_p14_epsilon_a4_trail_alpha_constant_equals_0_4`. Replacement [F090](#f090). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        (f - 0.4).abs() < 1e-9,
        "A4: TRAIL_ALPHA must equal 0.4; got {f}"
    )
```

## A0177

Baseline `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs:998` / `harness_p14_gamma_a23_world_renderer_no_extra_ffi_added`. Replacement [F091](#f091). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        extras.is_empty(),
        "A23: world_renderer.gd introduces FFI calls outside the allowed Phase 14-γ set: \
         {extras:?}; all calls = {found:?}"
    )
```

## A0178

Baseline `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs:1062` / `harness_p14_gamma_a27_phase14_beta_invariants_preserved`. Replacement [F092](#f092). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(count.trim(), "20", "A27: RESOURCE_COUNT must be 20; got `{count}`")
```

## A0179

Baseline `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs:1064` / `harness_p14_gamma_a27_phase14_beta_invariants_preserved`. Replacement [F092](#f092). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        seed.trim(),
        "88675123",
        "A27: RESOURCE_SEED must be 88675123; got `{seed}`"
    )
```

## A0180

Baseline `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs:1070` / `harness_p14_gamma_a27_phase14_beta_invariants_preserved`. Replacement [F092](#f092). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(z.trim(), "3", "A27: Z_RESOURCE must be 3; got `{z}`")
```

## A0181

Baseline `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs:1072` / `harness_p14_gamma_a27_phase14_beta_invariants_preserved`. Replacement [F092](#f092). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(zvf.trim(), "5", "A27: Z_VILLAGE_FIXTURE must be 5; got `{zvf}`")
```

## A0182

Baseline `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs:1075` / `harness_p14_gamma_a27_phase14_beta_invariants_preserved`. Replacement [F092](#f092). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        has_5_types,
        "A27: world_renderer.gd should reference ≥9 sprite paths (5 resource types + 4 fixtures)"
    )
```

## A0183

Baseline `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs:1089` / `harness_p14_gamma_a28_phase12_to_13_visual_invariants_preserved`. Replacement [F093](#f093). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        con.trim(),
        "\"res://assets/sprites/buildings/cairn/1.png\"",
        "A28: CONSTRUCTION_SPRITE_PATH must be buildings/cairn/1.png; got `{con}`"
    )
```

## A0184

Baseline `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs:1095` / `harness_p14_gamma_a28_phase12_to_13_visual_invariants_preserved`. Replacement [F093](#f093). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        furn.trim(),
        "\"res://assets/sprites/buildings/gathering_marker/1.png\"",
        "A28: FURNITURE_SPRITE_PATH must be buildings/gathering_marker/1.png \
         (re-pointed by fix-settlement-marker-fixed-position); got `{furn}`"
    )
```

## A0185

Baseline `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs:1102` / `harness_p14_gamma_a28_phase12_to_13_visual_invariants_preserved`. Replacement [F093](#f093). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        bld.trim(),
        "\"res://assets/sprites/buildings/campfire/1.png\"",
        "A28: BUILDING_SPRITE_PATH must be buildings/campfire/1.png; got `{bld}`"
    )
```

## A0186

Baseline `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs:1111` / `harness_p14_gamma_a28_phase12_to_13_visual_invariants_preserved`. Replacement [F093](#f093). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(bx.trim(), "32")
```

## A0187

Baseline `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs:1112` / `harness_p14_gamma_a28_phase12_to_13_visual_invariants_preserved`. Replacement [F093](#f093). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(by.trim(), "32")
```

## A0188

Baseline `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs:1113` / `harness_p14_gamma_a28_phase12_to_13_visual_invariants_preserved`. Replacement [F093](#f093). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(bxl.trim(), "24")
```

## A0189

Baseline `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs:1114` / `harness_p14_gamma_a28_phase12_to_13_visual_invariants_preserved`. Replacement [F093](#f093). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(bxr.trim(), "40")
```

## A0190

Baseline `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs:1118` / `harness_p14_gamma_a28_phase12_to_13_visual_invariants_preserved`. Replacement [F093](#f093). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        no_ws(&zd),
        "Vector2(5.0,5.0)",
        "A28: ZOOM_DEFAULT must be Vector2(5.0, 5.0) (raised from 3.0 in B-1); got `{zd}`"
    )
```

## A0191

Baseline `rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs:678` / `harness_zoom_a19_phase12a_13a_camera_controller_zoom_invariants_intact`. Replacement [F094](#f094). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        missing_lits.is_empty(),
        "a19.2: camera_controller.gd must keep 0.5/8.0/5.0 zoom literals \
         (ZOOM_DEFAULT raised from 3.0 in B-1); missing={missing_lits:?}"
    )
```

## A0192

Baseline `rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs:717` / `harness_zoom_a21_phase14b_world_renderer_invariants_intact`. Replacement [F095](#f095). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains("RESOURCE_TYPE_PATHS"),
        "a21.1: world_renderer.gd must keep RESOURCE_TYPE_PATHS"
    )
```

## A0193

Baseline `rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs:721` / `harness_zoom_a21_phase14b_world_renderer_invariants_intact`. Replacement [F095](#f095). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains("VILLAGE_FIXTURE_"),
        "a21.2: world_renderer.gd must keep at least one VILLAGE_FIXTURE_ identifier"
    )
```

## A0194

Baseline `rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs:728` / `harness_zoom_a21_phase14b_world_renderer_invariants_intact`. Replacement [F095](#f095). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(count, 20, "a21.4: RESOURCE_COUNT must equal 20; got {count}")
```

## A0195

Baseline `rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs:732` / `harness_zoom_a21_phase14b_world_renderer_invariants_intact`. Replacement [F095](#f095). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(seed, 88675123, "a21.6: RESOURCE_SEED must equal 88675123; got {seed}")
```

## A0196

Baseline `rust/crates/sim-test/tests/harness_resource_scarcity.rs:1273` / `harness_scarcity_a16_production_init_seeds_all_three_kinds_finite`. Replacement [F096](#f096). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(finite_food, "A16: production init must seed a FINITE FOOD source (∈ [1,254])")
```

## A0197

Baseline `rust/crates/sim-test/tests/harness_resource_scarcity.rs:1274` / `harness_scarcity_a16_production_init_seeds_all_three_kinds_finite`. Replacement [F096](#f096). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(finite_water, "A16: production init must seed a FINITE WATER source (∈ [1,254])")
```

## A0198

Baseline `rust/crates/sim-test/tests/harness_resource_scarcity.rs:1275` / `harness_scarcity_a16_production_init_seeds_all_three_kinds_finite`. Replacement [F096](#f096). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(finite_sleep, "A16: production init must seed a FINITE SLEEP source (∈ [1,254])")
```

## A0199

Baseline `rust/crates/sim-test/tests/harness_s16_alpha0_resource_substrate.rs:424` / `harness_resource_decorative_layer_preserved_static`. Replacement [F097](#f097). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        src.contains("RESOURCE_SEED"),
        "A12.1: world_renderer.gd must still contain RESOURCE_SEED"
    )
```

## A0200

Baseline `rust/crates/sim-test/tests/harness_s16_alpha0_resource_substrate.rs:428` / `harness_resource_decorative_layer_preserved_static`. Replacement [F097](#f097). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        src.contains("RESOURCE_COUNT"),
        "A12.2: world_renderer.gd must still contain RESOURCE_COUNT"
    )
```

## A0201

Baseline `rust/crates/sim-test/tests/harness_s16_gamma_visual.rs:258` / `harness_s16_gamma_a4_world_renderer_marker_is_polygon2d`. Replacement [F098](#f098). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        body.contains("Polygon2D"),
        "a4.1: marker must be a Polygon2D. Body:\n{body}"
    )
```

## A0202

Baseline `rust/crates/sim-test/tests/harness_s16_gamma_visual.rs:263` / `harness_s16_gamma_a4_world_renderer_marker_is_polygon2d`. Replacement [F098](#f098). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        body.contains("SOURCE_KIND_COLORS"),
        "a4.2: marker color must come from SOURCE_KIND_COLORS. Body:\n{body}"
    )
```

## A0203

Baseline `rust/crates/sim-test/tests/harness_s16_gamma_visual.rs:268` / `harness_s16_gamma_a4_world_renderer_marker_is_polygon2d`. Replacement [F098](#f098). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        body.contains("z_index"),
        "a4.3: marker must set z_index. Body:\n{body}"
    )
```

## A0204

Baseline `rust/crates/sim-test/tests/harness_s16_gamma_visual.rs:273` / `harness_s16_gamma_a4_world_renderer_marker_is_polygon2d`. Replacement [F098](#f098). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        !body.contains("Sprite2D"),
        "a4.4: _update_resource_markers must NOT construct a Sprite2D marker \
         (the α0 tinted-Sprite2D path is replaced, not augmented). Body:\n{body}"
    )
```

## A0205

Baseline `rust/crates/sim-test/tests/harness_s16_gamma_visual.rs:291` / `harness_s16_gamma_a5_z_order_above_decorative`. Replacement [F099](#f099). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        src_rhss.len(),
        1,
        "a5.1: expected exactly 1 Z_RESOURCE_SOURCE declaration; got {src_rhss:?}"
    )
```

## A0206

Baseline `rust/crates/sim-test/tests/harness_s16_gamma_visual.rs:297` / `harness_s16_gamma_a5_z_order_above_decorative`. Replacement [F099](#f099). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        dec_rhss.len(),
        1,
        "a5.2: expected exactly 1 Z_RESOURCE declaration; got {dec_rhss:?}"
    )
```

## A0207

Baseline `rust/crates/sim-test/tests/harness_s16_gamma_visual.rs:306` / `harness_s16_gamma_a5_z_order_above_decorative`. Replacement [F099](#f099). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        z_source > z_decor,
        "a5.5: Z_RESOURCE_SOURCE ({z_source}) must be strictly greater than Z_RESOURCE ({z_decor})"
    )
```

## A0208

Baseline `rust/crates/sim-test/tests/harness_s16_gamma_visual.rs:320` / `harness_s16_gamma_a6_decorative_invariant_preserved`. Replacement [F100](#f100). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        stripped.contains("RESOURCE_SEED"),
        "a6.1: world_renderer.gd must keep RESOURCE_SEED (p13-β decorative layer)"
    )
```

## A0209

Baseline `rust/crates/sim-test/tests/harness_s16_gamma_visual.rs:325` / `harness_s16_gamma_a6_decorative_invariant_preserved`. Replacement [F100](#f100). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(
        rhss.len(),
        1,
        "a6.2: expected exactly 1 RESOURCE_COUNT declaration; got {rhss:?}"
    )
```

## A0210

Baseline `rust/crates/sim-test/tests/harness_s16_gamma_visual.rs:332` / `harness_s16_gamma_a6_decorative_invariant_preserved`. Replacement [F100](#f100). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert_eq!(count, 20, "a6.4: RESOURCE_COUNT must remain 20; got {count}")
```

## A0211

Baseline `rust/crates/sim-test/tests/harness_s16_gamma_visual.rs:347` / `harness_s16_gamma_a7_backend_truth_positions_preserved`. Replacement [F101](#f101). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        body.contains("get_resource_snapshot"),
        "a7.1: _update_resource_markers must keep calling get_resource_snapshot \
         (backend-truth positions). Body:\n{body}"
    )
```

## A0212

Baseline `rust/crates/sim-test/tests/harness_show_resource_depletion.rs:426` / `harness_resource_depletion_a10_live_depletion_surfaces`. Replacement [F102](#f102). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        partial || reaped,
        "A10: after 4380 ticks depletion must surface — either a partially \
         depleted row (amount < max) OR fewer present source rows than at tick \
         2000 (count_2000={count_2000}, count_4380={}, partial={partial}, reaped={reaped})",
        rows.len()
    )
```

## A0213

Baseline `rust/crates/sim-test/tests/harness_show_resource_depletion.rs:392` / `harness_resource_depletion_a9_production_scene_registers_finite_capacities`. Replacement [F103](#f103). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        finite >= 1,
        "A9: production snapshot must carry >= 1 row with a registered finite \
         capacity (max != {} sentinel and max >= 1); rows={}, finite={}",
        RESOURCE_SOURCE_INFINITE,
        rows.len(),
        finite
    )
```

## A0214

Baseline `rust/crates/sim-test/tests/harness_t7_10_b1_space_toggle.rs:383` / `harness_t7_10_b1_substrate_calls_survive_off_gate`. Replacement [F104](#f104). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        process.contains("_update_settlement_furniture()"),
        "world_renderer.gd `_process` must call `_update_settlement_furniture()` \
         every frame (substrate render, not the overlay)"
    )
```

## A0215

Baseline `rust/crates/sim-test/tests/harness_t7_10_b1_space_toggle.rs:398` / `harness_t7_10_b1_substrate_calls_survive_off_gate`. Replacement [F104](#f104). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        construction_idx > off_idx,
        "the substrate render calls must come AFTER the overlay channel gate \
         (so they are not nested inside the OFF skip)"
    )
```

## A0216

Baseline `rust/crates/sim-test/tests/harness_t7_10_b1_space_toggle.rs:405` / `harness_t7_10_b1_substrate_calls_survive_off_gate`. Replacement [F104](#f104). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        resource_idx > off_idx,
        "`_update_resource_markers()` must appear AFTER the `current_channel == \
         CHANNEL_OFF` discriminator (so resource markers are not nested inside \
         the OFF skip and still render on the default OFF launch screen)"
    )
```

## A0217

Baseline `rust/crates/sim-test/tests/harness_t7_10_b1_space_toggle.rs:415` / `harness_t7_10_b1_substrate_calls_survive_off_gate`. Replacement [F104](#f104). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        !gate_region.contains("return"),
        "the OFF branch must NOT `return` before the substrate render calls \
         (otherwise construction/settlement/resource markers vanish on launch — \
         a worse regression than the original dark-screen bug)"
    )
```

## A0218

Baseline `rust/crates/sim-test/tests/harness_t7_9_b_render_mechanism.rs:111` / `harness_t7_9_b_world_renderer_render_path_present`. Replacement [F105](#f105). Disposition: re-pinned to the explicit current function contract; not a one-to-one assertion mapping.

```rust
assert!(
        src.contains("on_building_placed(BOOTSTRAP_X"),
        "world_renderer.gd _ready must call \
         `world_sim.on_building_placed(BOOTSTRAP_X, BOOTSTRAP_Y, BOOTSTRAP_RADIUS)` \
         to give BuildingStampSystem something to emit"
    )
```

## A0219

Baseline `rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs:471` / `harness_g_a13_existing_p12_alpha_a4_updated_to_eight`. Replacement [F106](#f106). Disposition: semantic input-scope re-pin; macro text unchanged; exact old/current function inputs included in replacement group.

```rust
assert!(
        compact.contains("Vector2(8.0,8.0)"),
        "a13.1: harness_p12_alpha_camera_zoom.rs must accept Vector2(8.0,8.0) (raised in G Phase A)"
    )
```

## A0220

Baseline `rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs:475` / `harness_g_a13_existing_p12_alpha_a4_updated_to_eight`. Replacement [F106](#f106). Disposition: semantic input-scope re-pin; macro text unchanged; exact old/current function inputs included in replacement group.

```rust
assert!(
        !compact.contains("Vector2(4.0,4.0)"),
        "a13.2: harness_p12_alpha_camera_zoom.rs must NOT still assert the old Vector2(4.0,4.0) \
         ZOOM_MAX literal"
    )
```

## A0221

Baseline `rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs:489` / `harness_g_a14_existing_p13_alpha_a3_updated_to_eight`. Replacement [F107](#f107). Disposition: semantic input-scope re-pin; macro text unchanged; exact old/current function inputs included in replacement group.

```rust
assert!(
        compact.contains("Vector2(8.0,8.0)"),
        "a14.1: harness_p13_alpha_camera_and_buildings.rs must expect Vector2(8.0,8.0)"
    )
```

## A0222

Baseline `rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs:493` / `harness_g_a14_existing_p13_alpha_a3_updated_to_eight`. Replacement [F107](#f107). Disposition: semantic input-scope re-pin; macro text unchanged; exact old/current function inputs included in replacement group.

```rust
assert!(
        !compact.contains("Vector2(4.0,4.0)"),
        "a14.2: harness_p13_alpha_camera_and_buildings.rs must NOT still expect Vector2(4.0,4.0)"
    )
```

## A0223

Baseline `rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs:506` / `harness_p14_epsilon_a12_main_tscn_load_steps_equals_12`. Replacement [F108](#f108). Disposition: textual re-pin.

```rust
assert_eq!(n, 13, "A12.2: load_steps must equal 13 (Phase 14-ζ 9 → 11, viz-B 11 → 12, viz-D 12 → 13); got {n}")
```

## A0224

Baseline `rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs:608` / `harness_zoom_a17_main_tscn_load_steps_equals_12`. Replacement [F109](#f109). Disposition: textual re-pin.

```rust
assert_eq!(n, 13, "a17.2: load_steps must equal 13; got {n}")
```

## A0225

Baseline `rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs:604` / `harness_p14_epsilon_a16_regression_guard_hud_status_panel_phase_14_delta_intact`. Replacement [F110](#f110). Disposition: semantic input-scope re-pin; macro unchanged.

```rust
assert!(
        missing.is_empty(),
        "A16: hud_status_panel.gd must keep all 3 invariant literals (Phase 14-δ A2/A4/A5); \
         missing={missing:?}"
    )
```

## A0226

Baseline `rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs:775` / `harness_zoom_a23_phase14d_hud_status_panel_intact`. Replacement [F111](#f111). Disposition: semantic input-scope re-pin; macro unchanged.

```rust
assert!(
        missing.is_empty(),
        "a23: hud_status_panel.gd must keep all 3 invariant literals; missing={missing:?}"
    )
```

## A0227

Baseline `rust/crates/sim-test/tests/harness_show_agent_needs_bars.rs:488` / `harness_needs_a6_scene_wiring_additive`. Replacement [F112](#f112). Disposition: re-pinned scene-resource accounting; node/script-identity assertions retained.

```rust
assert_eq!(
        load_steps_after,
        LOAD_STEPS_BEFORE + 1,
        "A6(b): load_steps must be baseline+1 ({}); got {load_steps_after}",
        LOAD_STEPS_BEFORE + 1
    )
```

## A0228

Baseline `rust/crates/sim-test/tests/harness_show_agent_needs_bars.rs:494` / `harness_needs_a6_scene_wiring_additive`. Replacement [F112](#f112). Disposition: re-pinned scene-resource accounting; node/script-identity assertions retained.

```rust
assert_eq!(
        ext_count_after,
        EXT_COUNT_BEFORE + 1,
        "A6(b): ext_resource count must be baseline+1 ({}); got {ext_count_after}",
        EXT_COUNT_BEFORE + 1
    )
```

## A0229

Baseline `rust/crates/sim-test/tests/harness_show_death_visual.rs:439` / `harness_death_a6_scene_wiring_additive`. Replacement [F113](#f113). Disposition: re-pinned scene-resource accounting; node/script-identity assertions retained.

```rust
assert_eq!(
        load_steps_after,
        LOAD_STEPS_BEFORE + 1,
        "A6: load_steps must be baseline+1 ({}); got {load_steps_after}",
        LOAD_STEPS_BEFORE + 1
    )
```

## A0230

Baseline `rust/crates/sim-test/tests/harness_show_death_visual.rs:445` / `harness_death_a6_scene_wiring_additive`. Replacement [F113](#f113). Disposition: re-pinned scene-resource accounting; node/script-identity assertions retained.

```rust
assert_eq!(
        ext_count_after,
        EXT_COUNT_BEFORE + 1,
        "A6: ext_resource count must be baseline+1 ({}); got {ext_count_after}",
        EXT_COUNT_BEFORE + 1
    )
```

## Functions with no removed old macro after normalization

- `rust/crates/sim-test/tests/harness_p11_alpha_agent_renderer.rs:436` / `harness_p11_alpha_agent_renderer_interpolates_positions`: No old macro removed after whitespace/comment normalization; additional assertions or formatting-only change.
