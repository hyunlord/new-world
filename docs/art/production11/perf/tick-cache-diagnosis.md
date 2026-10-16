# Part 3: rejected tick-cache diagnosis and appearance readers

Historical diagnosis prepared on 2026-10-16 and incorporated into Part 3 documentation at source HEAD `f0d04f8e5bb698385b4751f0e0ea9f401ebef5ac`. Sources: `docs/art/production11/GOAL.md` section 7; committed Pass 10 ZIP; renderer source; official upstream Godot 4.6 source. The analysis used offline image comparison and source inspection, not a new Godot or performance run. No tick-cache candidate is accepted by this report. Historical exact CPU/render-clock causality remains **NOT_VERIFIED**.

## Historical pixel failures independently checked

Offline Pillow/NumPy analysis read the original baseline `/Users/rexxa/work/art-production10-evidence/identity/baseline/A/P5.png`, each candidate directly from `docs/art/production10/perf/tick-rejected/raw-evidence.zip`, and `assets/art/terrain/water.png`. It projected each differing screen pixel to the tile atlas with manifest camera (1016,500), zoom 4, origin (440,20), the actual deterministic variant hash, and the two observed spring color mappings. No masks or tolerances were changed.

| Attempt | Changed pixels | Inclusive bounds | Baseline matching frame | Candidate matching frame | Candidate PNG SHA256 |
|---|---:|---|---|---|---|
| 1 | 1680 | 648,196–803,1075 | 2:1680/1680 | 0:1680/1680 | 8004d917518bc7a2e6a83a75db08ac3e8d8eef45ef295be2a3f01a6936fe176c |
| 2 | 816 | 620,196–807,1031 | 2:816/816 | 3:816/816 | 4340c8b5056655c35c1d2b9268469c52a0bb5baf091d964078ac535fd6cf1719 |
| 3 | 1680 | 648,196–803,1075 | 2:1680/1680 | 0:1680/1680 | 8004d917518bc7a2e6a83a75db08ac3e8d8eef45ef295be2a3f01a6936fe176c |

Attempt 2 swaps RGB (46,92,126)→(42,91,108) at 432 pixels and reverse at 384; attempts 1 and 3 have 864 and 816 respectively. Attempt 3 PNG is byte-identical to attempt 1. This independently extends the archived first-attempt analysis to the other two failures. All changed pixels are attributable to water wave phase. This does not prove unseen pixels or running-agent cache equivalence. Historical REJECTED verdict remains unchanged.

## Actual time source, and what is not proven

The archived log identifies runtime `4.6.stable.official.89cea1439`, Metal Forward+. `terrain_renderer.gd:34–37` creates four-frame, 3 FPS TileSet animation. It does not choose wave frames using SimReads, simulation tick, `_frame`, or a GDScript elapsed time.

The exact upstream source path is:

1. [TileMapLayer::draw_tile](https://github.com/godotengine/godot/blob/4.6/scene/2d/tile_map_layer.cpp#L2705) creates animation slices from durations divided by animation speed. Default animation mode supplies zero offset (random offsets apply only to nondefault mode, lines 376–384).
2. [RendererCanvasRenderRD animation slice](https://github.com/godotengine/godot/blob/4.6/servers/rendering/renderer_rd/renderer_canvas_render_rd.cpp#L2879) reads rasterizer total time and tests modulo animation duration against the slice.
3. [RendererCompositorRD::begin_frame](https://github.com/godotengine/godot/blob/4.6/servers/rendering/renderer_rd/renderer_compositor_rd.cpp#L108) accumulates the render `frame_step` into compositor time.
4. [Main rendering dispatch](https://github.com/godotengine/godot/blob/4.6/main/main.cpp#L4980) calls draw with scaled process step only when a window can draw/additional outputs or pending rendering resources require work. Process iteration and render iteration are distinct clocks.

`capture_identity_p10.gd:47,88–104` validates process frame 10→130, process delta, simulation tick and framebuffer. It never records compositor time, render callback count, render frame steps, or window-draw eligibility. Its final `RenderingServer.force_draw(false)` supplies no positive delta; it renders the already accumulated compositor phase.

The rejected sources alter agent CPU work and SimReads cache generations, not terrain atlas animation or the renderer clock. Therefore the statement “the cache changes water's wall-clock time” is **not supported**: the verified time source is accumulated rendering steps. CPU timing can affect presentation eligibility/startup interleaving, but the archived metadata cannot determine whether/when that occurred. Exact historical causal attribution is **NOT_VERIFIED**; it must not be promoted from this plausible mechanism into a finding. Stable process delta alone is insufficient identity provenance.

Before a new cache attempt, obtain A/A′ reproducibility with matched render steps before comparing B. The accompanying `scripts/test/art/identity_draw_clock_p11.gd` harness requires `--disable-render-loop`, records `RenderingServer.frame_pre_draw` / `frame_post_draw` and `Engine.get_frames_drawn()`, and uses explicit `RenderingServer.force_draw(false, 1.0 / 60.0)` with sequential-frame checks. Its evidence distinguishes manual drawing from process frames and requires draw callbacks to match requested steps. This addresses the missing clock control without freezing, recoloring, replacing or masking water. Source presence and static checks do not establish runtime reproducibility: actual A/A′ and B results must be recorded separately. No successful identity run is claimed here.

## Appearance-only candidate reader inventory

The allowed list in section 7.2a is already deliberately narrow. Search scope was `scripts/**` and `tools/art_pipeline/**`, including metadata names and child node names; source references below were inspected before any Part 3 optimization, with the integration HEAD recorded above. This inventory supports only the expressly listed appearance work; it is not permission to skip additional fields or the whole agent loop. Production consumers and important test consumers:

| Skipped operation | Writes / readers that constrain the change |
|---|---|
| `SpriteLibrary.apply_variant` | `art_sprite_library.gd:106–124`: material/self_modulate only. Inspector `panels/agent_inspector_panel.gd:258–269` reads sprite_id, flip and variant metadata and calls apply_variant on its own portrait. Keep variant metadata and selected agents fully current. |
| `_update_activity_icon` | renderer-owned `_activity_icons`, child sprite texture/visibility/offset; `agent_activity.gd:185–205`. Sidecar/capture and `ingame_qa_wood_p{4,6,7}.py` inspect `art_kind=activity_icon`. No production layout dependence found. |
| `Cold.apply` | `cold_effects.gd:51–72`: position.x shiver, cold/shiver_offset/puff_alpha metadata, ColdBreath child. **Not purely material work:** current rendered position is also exposed by `agent_renderer.gd:711–714` and read by world effects. Keep all pose calculation and movement history; skipping is explicitly permitted only outside the margin. Re-entry must restore shiver in that first frame. |
| `_morning_chill.apply` | `morning_chill.gd:45–102`: reads activity/visible, writes chill metadata plus ChillBreath/ChillOutline; test_morning_chill and v2 read these. Its ingest/alpha calculations must stay global because Cold.pose uses alpha before appearance. |
| `_apply_settlement_outline` | SettlementOutline child and settlement_outline metadata. Called **twice**, at `agent_renderer.gd:275` and `:683`; guard both if measured optimization is chosen. `test_settlement_select.gd` reads metadata. |
| Shadow position/visibility | renderer-owned `_shadows`, art_kind agent_shadow, capture sidecar consumption. Keep lifecycle/retirement and pool code. |
| `_update_cue` | renderer-owned recall/combat indicator nodes; keep lifecycle/retirement and cue data sources. |
| `activity_oracle_inputs` | Capture-only metadata set behind existing oracle flag; sidecar consumers are test/evidence code. Keep all ordinary activity/pose metadata and active-holder resolution. |

Mandatory retained consumers: `agent_renderer.gd:656–670` footprint uses frame_size, anchor, flips and CarryLoad child rect; `camp_seats.gd` consumes all rows for seats/spread/declump; `world_effects.gd:86–103` reads positions/activity/sheltered_sleep/tile metadata; `shelter_cold.gd:71` consumes shelter presentation; `morning_chill.gd:46` reads activity; inspector portrait reads sprite fields. A full-loop offscreen `continue` is invalid.

Feasible narrow implementation after section timers: compute one per-agent appearance eligibility from previous drawn rect transformed into current canvas coordinates and viewport.grow(64), with selected/new sprites always eligible. Preserve all normal row creation, sprite frame, pose, facing, CarryLoad, seats, declump and metadata work; branch only the eight expressly listed operations. Test current re-entry in the same frame after pan/motion rather than retaining an old camera-space boolean. Do not enable until timers demonstrate a worthwhile bucket; then six framings, W0/W200 running dumps and W500 frozen/2× 240-frame pan identity plus three alternating pairs are mandatory.

Potential cheaper candidate discovered: the duplicated settlement-outline call can be measured as a narrow removal candidate, retaining the final post-Cold call. This is not yet safe to accept: its first call may affect child state inspected between positioning and final appearance, and must pass the same identity/measurement gates. No performance estimate is asserted.

## Status

Completed for this diagnosis: code/source tracing, offline attribution of all three rejected P5 pixel differences, and narrow reader inventory. Historical REJECTED verdicts remain unchanged. Historical render-clock divergence is not proven; harness reproducibility, current section measurements, candidate speedup and all identity acceptance gates require separate runtime artifacts. This document does not claim any of them passed and does not accept a cache optimization.

## P5 acceptance guard (2026-10-16T13:06Z)
The P11 pixel comparator now explicitly rejects any nonzero P5 A/A′ control or A/B difference. Other framings retain the unchanged Art Pass 10 two-dimensional control bound. Alpha-only image regressions reproduced two false-accept cases in the inherited generic comparator before the fix. The P11 and P9 comparator tests pass (14 tests, 5 subtests); this is comparator verification, not an actual identity verdict.
