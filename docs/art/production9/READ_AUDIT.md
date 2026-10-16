# Part 1 pre-migration read audit

Read-only source audit for GOAL section 5. No game/Rust edits, builds, probes, or Godot launches were performed by this auditor. Source baseline HEAD: `49bc0b7c87159340b8024d6b9327bc394dc6daa0`. Other agents may be adding Part 0 tools concurrently; line references below describe the inspected pre-migration files, not the eventual final implementation.

## Terrain lifetime proof

Exhaustive `rg -n 'terrain_type|elevation' rust/crates` includes every crate, not just sim-systems. Production field writes are initialization in `sim-core/src/tile/grid.rs:46–47`, world generation in `sim-systems/src/worldgen/terrain.rs:19,42–43,57,60,65`, and **a terrain swap at terrain.rs:127** inside `repair_berry_edges`. The latter is easy to miss with an assignment-only search. It is called by worldgen `camp.rs:201` (`berries`), reached during `camp::populate`; `worldgen/mod.rs:224–226` generates terrain then populates camp before returning the constructed engine. No runtime terrain/elevation writer was found.

Other actual writes are unit fixture assignment in `sim-bridge/src/ffi/world_node.rs:3414,3419`, inside `terrain_snapshot_preserves_all_codes_and_row_major_elevation`. Bridge collector `world_node.rs:3326–3327` reads/clones the fields. Runtime settlement references (`runtime/settlement/fission.rs:198`, `shelter_planning.rs:84`) only read terrain classifications.

The whole `TileGrid` API (`sim-core/src/tile/grid.rs:35–125`) has constructor, index/size, neighbor, wall and outdoor readers; it exposes **no mutator method**, mutable slice accessor, or `DerefMut` implementation. Arrays are public fields, so the proof requires the field-use search in addition to API review. An all-crate search for whole-grid assignment, `TileGrid {`, `&mut ...tile_grid`, and `&mut TileGrid` found only the type/impl, worldgen mutable borrow, and read-only influence aliases; no runtime whole-grid replacement. Terrain per-world caching is justified for this revision, with immediate reset invalidation. Future field setters or runtime world replacement invalidate this conclusion.

## Reset and process ordering

`scenes/main.tscn:28` has WorldSim as the first child; no process priority property is set in that scene. Rust `world_node.rs:208` implements WorldSim process. Production explicit priority changes are positive: `god_hand_effects.gd:42` = 20, `founding_group_renderer.gd:16` = 31, `settlement_labels.gd:24` = 30, `nature_renderer.gd:24` = AgentRenderer priority + 1. No production negative priority was found under scripts/scenes. Negative priorities are test instrumentation: frame bracket `frame_cpu_probe.gd:21` = -1000 and `test_canopy_fade.gd:108` agent priority - 1. Keep the main reader at default priority, directly after WorldSim; this preserves the sibling ready/connect ordering and WorldSim-before-reader process ordering.

**Reset signal is deferred:** `world_node.rs:241,254` uses `call_deferred("emit_signal", ["world_reset"])` for normal/stress resets. A reset wrapper must evict terrain immediately as well as per-frame entries; waiting for the signal allows a same-frame terrain read to return the previous world's map. Signal invalidation remains necessary for external resets and must connect before renderer reset callbacks. Frame number alone also cannot invalidate an `advance_ticks` call followed by reads inside the capture loop (`capture_framings.gd:390–395`).

## Shared-object mutation audit

| Reader / location | Ownership and required treatment |
|---|---|
| `scripts/ui/panels/hud_status_panel.gd:115–117` | `settlements` aliases the returned Dictionary and adds `settlement_ids`. Duplicate before adding compatibility key. |
| `scripts/rendering/art/god_hand_effects.gd:79` and `:65` | `_state` aliases intervention snapshot; `_reset()` calls `_state.clear()`. Replace `_state = {}` on reset or duplicate at acquisition. This is an additional read-only violation beyond the brief's known HUD case. |
| `hud_status_panel.gd:106,109,366` | `_clock` is cleared on reset, but ingestion already takes `_read_clock().duplicate()`: safe existing ownership. Preserve duplicate. |
| `scripts/test/art/test_nature_clearance.gd:110,139–144` | Real construction snapshot is `duplicate(true)` before becoming `mock.sites`; packed `xs/ys.append` mutates the copy. Preserve the deep copy, and verify packed copy independence in runtime probe. |
| `scripts/ui/camp_seats.gd:17–18,28–29,108–113` | Sorts duplicated hearth array, fresh `rows.keys()`, and newly-created grouping arrays; these are not FFI packed aliases. |
| `scripts/test/art/capture_cold_trace.gd:20–21` | Sorts fresh `_rows.keys()`, not the `agent_ids` packed array used by ingest. |
| `scripts/ui/terrain_renderer.gd:66` | Stores returned terrain packed array in `_terrain`, reads it thereafter; clears tile layers and own `_camp`, not packed terrain. |
| `scripts/ui/seek_viz_renderer.gd`, `death_viz_renderer.gd`, `settlement_overview_renderer.gd` | Keep returned packed arrays as read-only aliases; drawable output arrays are separately allocated. No in-place mutation of those packed aliases found. |
| `scripts/ui/agent_renderer.gd:78–86` and its context helpers | Snapshot packed arrays are indexed/read; sprite/cue/position dictionaries and draw-agent rows are separately allocated. No packed alias append/sort/index-write found. |
| `scripts/ui/nature_layout.gd`, `nature_canopy.gd`, `wood_visuals.gd`, `shelter_cold.gd` | Derived clearing/point/wood-state/count collections are fresh. `nature_canopy` appends to its own new PackedVector2Array, not an FFI array. |
| capture event/death/cold trace helpers | Ingest reads scalar fields/array elements into owned rows. Returns deep duplicates of owned logs where needed. |
| mock-only tests | Deliberately mutate world dictionaries and packed fields repeatedly inside one engine frame (e.g. `test_feed_priority.gd:39–91`, `test_hud_wood.gd:53–73`). Never freeze or memoize a fake world. |

The audit searched all scripts for dictionary/index assignments and array mutators (append, append_array, push/pop, clear, resize, sort, sort_custom, reverse, shuffle, remove/erase, insert, fill, merge), then traced snapshot aliases at the relevant call sites. **No additional production packed-array mutation was found.** Runtime sharing is now separately verified by the leader: the exactly five-line `scripts/test/art/probe_packed_p9.gd` extracts `xs` from a Dictionary, assigns `xs[0] = 9`, and observes the Dictionary's packed array changed. `docs/art/production9/perf/packed_sharing.log` identifies Godot `v4.6.stable.official.89cea1439` and records `P9_PACKED_SHARED=true`; the leader reported exit 0. The auditor read both the script and captured log. This proves this packed-array mutation shares storage in the installed runtime; it does not prove all array operations or duplicate semantics. The 600-frame W0/W500 error-log runs remain pending and are not marked passed.

## Migration and static-test hazards

- HUD string dispatch: `hud_status_panel.gd:306–309`, `hud_topbar.gd:219–222`; callers pass literal getter names to `_snapshot`. Migrate helper dispatch, not just `.get_*` calls.
- `test_nature_clearance.gd:49–50` loops over getter strings and uses `world.call(method)`. A static scanner limited to literal `.call("...")` misses this real-main call. Cover it explicitly or remove this indirect dispatch during migration.
- `test_nature_clearance.gd` and `test_every_hearth.gd` are **mixed real scene and fake-world tests**. Detecting a `func get_*` definition must not exempt an entire file from real-main coverage. Ignore definitions only.
- Standalone-real test/probe scripts instantiate WorldSimNode without main. They need pass-through migration too, although the stated static coverage floor only requires main-loading tests.
- Source-replacement fake seams (`test_every_hearth.gd:29`, `test_nature_clearance.gd:103–105,183`, `test_structure_states.gd:110`) depend on exactly one `var _world_sim: WorldSimNode` declaration. Preserve seams and ensure a reader resolves against the injected world, not an unrelated `/root/Main/SimReads` cache.
- Per-hearth reads are at `building_renderer.gd:136,319–320,357`; hoist shared snapshots before loops, retaining `_elapsed`/hour/winter semantics.
- `frame_measure_session.gd:35–39` performs diagnostic reads outside the frame bracket. It must use the uncounted reader path on migrated builds and direct fallback on base builds.
- Allowed remaining direct reads include agent detail and influence overlay; tile click causal history remains a separate action-driven path (`world_renderer.gd:174`). Agent-renderer per-tile scans are the batched-causal migration target.
- Static source tests pin many getter spellings in addition to the approximately 13 detailed hearth assertions. Source tests under `harness_p12_*`, `harness_p13_*`, `harness_p14_*`, `harness_s16_*`, `harness_show_*`, and `harness_revitalize_screen_contract.rs` require intent-preserving updates and ledger rows. Do not satisfy them with comments containing dead getter strings.

## Per-file getter/mutator inventory

This is an exhaustive **name-reference inventory**, not a runtime count. It includes has_method guards and diagnostic strings intentionally so migration cannot silently lose an indirect dispatch. Fake definitions and full-line comments are excluded from the rows. All game files are in static coverage. `main` means literal main-scene load is present; `mixed` means real scene plus fake definitions; `standalone-real` means direct WorldSimNode construction; `helper` must be reviewed through its callers. Newly added measurement scripts can change concurrently after this snapshot.

| File | Class | Relevant source lines |
|---|---|---|
| `scripts/rendering/art/god_hand_effects.gd` | game | 76, 79, 80, 81, 277, 280 |
| `scripts/test/art/capture_age_consumers.gd` | main | 40, 47, 51, 56, 71, 79, 99, 139 |
| `scripts/test/art/capture_deaths_p7.gd` | mock | 126 |
| `scripts/test/art/capture_identity_p9.gd` | main | 46, 48, 49, 52, 67, 88 |
| `scripts/test/art/capture_plans.gd` | helper | 45 |
| `scripts/test/art/frame_measure_session.gd` | helper | 35, 39 |
| `scripts/test/art/measure_days_p7.gd` | standalone-real | 17, 20, 31, 33, 38, 39, 52, 54, 55, 56, 68 |
| `scripts/test/art/measure_days_p8.gd` | standalone-real | 29, 32, 48, 50, 56, 64, 68, 69, 87, 91, 92, 93, 105 |
| `scripts/test/art/perf_frames_p9.gd` | main | 17, 21, 82, 84, 90 |
| `scripts/test/art/probe_p6.gd` | standalone-real | 22, 25, 27, 32, 35, 36, 37, 78, 94, 96, 101, 102, 104, 109 |
| `scripts/test/art/qa_offscreen_marker_windowed.gd` | main | 19, 24, 25, 31, 32, 46, 47, 48, 94 |
| `scripts/test/art/test_agent_activity.gd` | main | 60 |
| `scripts/test/art/test_agent_click_drawn.gd` | main | 24 |
| `scripts/test/art/test_capture_pair_p8.gd` | mock | 8 |
| `scripts/test/art/test_feed_casts.gd` | standalone-real | 15, 17, 18, 33 |
| `scripts/test/art/test_god_hand.gd` | mock | 15 |
| `scripts/test/art/test_idle_facing.gd` | main | 28, 29, 50, 51 |
| `scripts/test/art/test_interventions_real_p6.gd` | standalone-real | 11, 15, 16, 22, 28 |
| `scripts/test/art/test_nature_clearance.gd` | mixed | 43, 47, 49, 55, 77, 108, 109, 110, 111, 112, 186 |
| `scripts/test/art/test_nature_density.gd` | main | 21, 27, 30, 32, 98, 114, 117 |
| `scripts/test/art/test_restless_meter.gd` | mock | 31, 36, 39, 44, 48 |
| `scripts/test/art/test_settlement_select.gd` | mock | 16 |
| `scripts/test/art/test_society_snapshot_p7.gd` | standalone-real | 19, 20, 24, 27, 29, 33, 35, 37, 48, 50, 58, 61, 71, 72, 73 |
| `scripts/test/art/test_terrain_layers.gd` | main | 20, 33 |
| `scripts/test/art/test_ysort_props.gd` | main | 21 |
| `scripts/test/capture_framings.gd` | main | 106, 110, 112, 168, 209, 254, 317, 330, 348, 354, 379, 387, 390, 391, 395, 408, 425, 426, 433, 434, 436, 437, 438, 452, 486, 487, 564, 565, 576, 610, 702, 750, 753, 754, 763, 779, 788, 809, 811, 814, 825, 827, 834, 881, 927, 950, 979, 990, 991, 997 |
| `scripts/test/capture_framings_p9.gd` | main | 108, 112, 114, 170, 211, 261, 326, 339, 357, 363, 388, 396, 399, 400, 404, 417, 434, 435, 442, 443, 445, 446, 447, 461, 495, 496, 573, 574, 585, 620, 712, 760, 763, 764, 773, 789, 798, 819, 821, 824, 835, 837, 844, 891, 937, 960, 989, 1000, 1001, 1007 |
| `scripts/test/fix_settlement_marker/harness_marker_fixed_a12.gd` | main | 238, 240 |
| `scripts/test/p4_gamma_rendering/harness_agent_rendering.gd` | main | 72 |
| `scripts/ui/activity_trail_renderer.gd` | game | 62 |
| `scripts/ui/agent_renderer.gd` | game | 72, 78, 79, 110, 454, 466, 477 |
| `scripts/ui/building_renderer.gd` | game | 62, 72, 126, 136, 145, 161, 181, 231, 285, 314, 319, 320, 351, 357, 413, 432, 511, 512, 518, 519 |
| `scripts/ui/camera_controller.gd` | game | 146, 183, 202 |
| `scripts/ui/daylight_controller.gd` | game | 44, 54, 63, 100 |
| `scripts/ui/death_viz_renderer.gd` | game | 66 |
| `scripts/ui/founding_group_renderer.gd` | game | 30, 32 |
| `scripts/ui/nature_renderer.gd` | game | 39, 91, 92, 93, 94, 166 |
| `scripts/ui/need_bar_renderer.gd` | game | 48 |
| `scripts/ui/panels/agent_inspector_panel.gd` | game | 205, 253, 269, 296 |
| `scripts/ui/panels/god_hand_panel.gd` | game | 110, 113, 135, 136, 137, 154 |
| `scripts/ui/panels/hud_status_panel.gd` | game | 113, 114, 119, 167, 184, 192, 198, 199, 209, 426 |
| `scripts/ui/panels/hud_topbar.gd` | game | 114, 122, 130, 138, 159, 169, 182, 195, 202 |
| `scripts/ui/seek_viz_renderer.gd` | game | 77 |
| `scripts/ui/settlement_labels.gd` | game | 51, 53, 54, 70 |
| `scripts/ui/settlement_overview_renderer.gd` | game | 72 |
| `scripts/ui/terrain_renderer.gd` | game | 63, 109, 134, 135, 136, 137, 184 |
| `scripts/ui/world_effects.gd` | game | 46, 47, 82 |
| `scripts/ui/world_renderer.gd` | game | 134 |

## Fake-world definitions (do not memoize/freeze)

- `scripts/test/art/capture_deaths_p7.gd`: getter definitions at 17, 19, 20, 21, 22, 23, 24, 25, 26.
- `scripts/test/art/test_capture_pair_p8.gd`: getter definitions at 10, 11, 12, 13, 15.
- `scripts/test/art/test_death_exposure.gd`: getter definitions at 11, 12, 13, 14, 15, 16, 17, 18.
- `scripts/test/art/test_every_hearth.gd`: getter definitions at 14, 16.
- `scripts/test/art/test_feed_priority.gd`: getter definitions at 13, 14, 15, 16, 17, 18, 19.
- `scripts/test/art/test_founding_group.gd`: getter definitions at 12, 13, 14, 15, 16, 17.
- `scripts/test/art/test_god_hand.gd`: getter definitions at 12, 13, 14.
- `scripts/test/art/test_hud_old_age.gd`: getter definitions at 11, 12, 13, 14, 15, 16, 17, 18.
- `scripts/test/art/test_hud_wood.gd`: getter definitions at 12, 13, 14, 15, 16, 17, 18.
- `scripts/test/art/test_inspector_age.gd`: getter definitions at 8.
- `scripts/test/art/test_inspector_cold.gd`: getter definitions at 6.
- `scripts/test/art/test_inspector_wood.gd`: getter definitions at 8.
- `scripts/test/art/test_intervention_world_effects.gd`: getter definitions at 8, 9, 10.
- `scripts/test/art/test_nature_clearance.gd`: getter definitions at 11, 14, 17, 20, 23.
- `scripts/test/art/test_restless_meter.gd`: getter definitions at 10, 12, 14, 15.
- `scripts/test/art/test_settlement_labels.gd`: getter definitions at 8, 9, 10, 11, 12, 13, 14.
- `scripts/test/art/test_settlement_select.gd`: getter definitions at 10, 12, 14.
- `scripts/test/art/test_structure_states.gd`: getter definitions at 9, 11, 13, 15, 17, 19, 21.
- `scripts/test/stockpile_feed_2_5c/harness_2_5c.gd`: getter definitions at 49, 51, 53.

## Reproduction commands

```sh
graft ask "terrain_type elevation TileGrid mutation set get_mut" --source
rg -n "terrain_type|elevation" rust/crates
rg -n "tile_grid\s*=|TileGrid\s*\{|&mut.*tile_grid|&mut TileGrid" rust/crates
rg -n "process_priority" scripts scenes
rg -n "call\(method|main\.tscn|func get_" scripts
```

One graft call reported approximately 62,036 tokens saved. No dollar estimate was supplied.
