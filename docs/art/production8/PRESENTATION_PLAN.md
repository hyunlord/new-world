# Parts 2, 3 and A30 implementation contract

Prepared from the complete GOAL.md and DESIGN.md, before Part 0 baseline finishes. This is a plan only: no product code, cargo, Godot or commits run by this preparation lane.

## Existing integration points

| Surface | Current file and anchor | Required extension |
|---|---|---|
| Founding trail / stake | `scripts/ui/founding_group_renderer.gd:33` refresh; `:61` draw | Route each site through common screen-edge geometry; stake and arrow exclusive. Keep trail and existing group metadata. |
| Village identity / raw state | `scripts/core/settlement_selection.gd:27` observe; `:147` capture | Store P/theta by settlement id, expose pure meter state; own one debug layer-enabled flag. |
| Village labels / click | `scripts/ui/settlement_labels.gd:72` update; `:151` draw; `:172` input | Meter beside label at zoom >= 2; selected offscreen hearth arrow and pan hit box. |
| Selected member outlines | `scripts/ui/agent_renderer.gd:623` `_apply_settlement_outline` | Gate selected outline with the same debug flag; continue drawing base agents. |
| Inspector context | `scripts/ui/panels/god_hand_panel.gd:111` refresh | Existing panel already follows selected village. Add a selected-village restlessness inspector row here; do not misrepresent the agent-only inspector as a village inspector. |
| Agent inspector | `scripts/ui/panels/agent_inspector_panel.gd:145` display | Optional home-village state only; selected-village inspector requirement is met independently of selecting an agent. |
| Camera | `scripts/ui/camera_controller.gd:135` set_auto_track | Click disables auto-track, sets `position` to tile centre, calls `force_update_scroll`; preserve integer zoom. |
| A30 | `scripts/test/capture_framings.gd:541` timing; `:850` society pair | Replace mismatched snapshots with one frozen >=3-settlement frame and paired on/off blocks. Preserve wall-clock samples labelled pacing. |
| Rust getter | `rust/crates/sim-bridge/src/ffi/world_node.rs:2382`, `:2419`, `:2487` | Two Vec<i64> fields, per-alive-row collection, PackedInt64Array dictionary loop; additive getter tests only. |

Line references describe the inspected pre-implementation source and will move.

## Part 2: one reusable screen-space marker

Add `scripts/ui/offscreen_marker.gd` as a small pure geometry/drawing helper shared by founding renderer and labels. Compute target screen position with the actual canvas transform. If viewport rect contains the target, return on-screen state and no edge marker. Otherwise intersect the ray from viewport centre to the target with an inset viewport rect, keeping the full 9x9 arrow and distance text inside. Clamp against the actual visible world area/HUD bounds without changing the pointing angle. Use exact target direction for arrow orientation rather than snapping its orientation to 8 directions; 8 directions are the test matrix and blind-answer classes.

Draw 9x9 screen-pixel procedural arrow with a 1px dark outline and Selection.color_for(parent) for founding sites, Selection.color_for(selected_id) for selected hearth. Founding distance is rounded Euclidean leader tile to destination tile distance; store the unrounded distance plus displayed integer in capture metadata. No distance text on selected-village arrow. Locale key FOUNDING_SITE_DISTANCE_FMT must ship with Part 2 in both Fluent files, compiled JSONs and key registry.

Use screen coordinates for hit rectangles and inverse canvas transform for Node2D drawing so zoom never scales the 9px glyph. Marker click handles input before empty-ground deselection, disables camera auto-track and centres on the destination. Founding stake draws only for on-screen destination; metadata explicitly carries stake_visible/edge_visible, edge screen position, direction, destination screen position, distance and colour. Refresh camera-dependent marker geometry every frame even if simulation tick is frozen.

`scripts/test/art/test_offscreen_marker.gd`: root SceneTree test with Camera2D stub and synthetic viewport; centre plus cardinal/diagonal targets, edge containment and direction <=10 degrees; exactly one stake/arrow; native size at zoom1/2/4/8; leader-distance rounding; click camera destination and no selection clearing; parent palette (parent id differs from daughter); selected hearth arrow has no distance; reset clears markers. Reuse existing test_founding_group.gd fixture and preserve its raw group semantics.

## Part 3: getter and meter

In SocietySnapshot add fission_pressures / fission_thresholds Vec<i64>; collect values from the exact Settlement row used for settlement_ids, marshal through the existing PackedInt64Array loop. No simulation mutation. Unit test nonzero distinct values for two alive rows, row ordering, old-rule zeros, and unaffected founder/departure arrays.

Selection.observe reads optional arrays defensively; absence or theta<=0 produces hidden state. Pure `restlessness_for(id)` returns visible, pressure, threshold, clamped ratio, stage index and localization key. Stage uses thirds: P/theta<1/3 calm, <2/3 uneasy, otherwise ready; retain unclamped raw P/theta in evidence. Three segments fill by thirds, stable village colour, empty segment muted. Labels hide meters below zoom2, and absent fields never synthesize pressure.

The current UI has no standalone selected-village inspector: god_hand_panel is the existing selected-village context. Add a compact titled inspection row with same three-segment graphic and localized state, shown only for explicit selected_id>=0 and valid theta. This is an autonomous UI placement decision to record in DECISIONS.md. It avoids modifying the agent inspector visibility contract or requiring a selected agent. The shared debug flag also hides this row for A30.

`scripts/test/art/test_restless_meter.gd`: optional arrays, zero theta, P=0/333/334/666/667/1000 at theta1000 (exact rational thirds), ratio overflow, two villages with distinct values/colours, reset, zoom1 hidden/zoom2 shown; actual selected inspector Label en/ko strings and actual segment visibility. Keys VILLAGE_RESTLESS_{CALM,UNEASY,READY,TITLE} ship with Part3.

## A30: one debug flag, one frozen world, paired CPU evidence

Keep one authoritative boolean `Selection.layers_enabled`, default true. Setter coordinates node group `art_society_layers` visibility AND disables its processing; merely hiding nodes leaves presentation CPU work in the off condition. Labels include meters, founding renderer includes trail/stake/arrow, selection arrows share these parents. Gate outlines explicitly in agent renderer because those are children of agents outside art_society_layers: hide existing outlines on transition and early-return before per-agent outline geometry/material updates. The selected inspector row observes the same flag and skips its calculation while off. Do not disable base agents, terrain, stockpiles or world effects. Audit overview renderer: its disc predates Pass7, but Pass7 adds palette; record whether whole overview is excluded and apply symmetrically. Prefer include the village overview consistently in the layer set to avoid leaving village presentation behind. Restore original visibility/process states after measurement, not unconditional show().

Create `scripts/test/art/frame_cpu_probe.gd`: two ordinary Node process callbacks, priority -1000 and +1000; first stores Time.get_ticks_usec(), last records elapsed and process frame id. Do not use SceneTree.process_frame as the end bracket. Probe stamps must bracket ordinary renderer _process methods, with same-frame identity checked. Enable viewport render timing with viewport_set_measure_render_time(rid,true); acquire viewport_get_measured_render_time_cpu and get_frame_setup_time_cpu after rendering; record both separately and summed with process CPU, as mandated. GPU==0 becomes unavailable, not zero-cost. Keep frame_post_draw wall time as pacing-only.

Freeze WorldSim by speed0 plus PROCESS_MODE_DISABLED before warmup. Disable camera tracking and camera processing; set FV camera once at Part0-measured first >=3-alive frame; record tick/camera/population before and after. Save vsync mode then disable; restore vsync, measure-render flag, layer state and probe nodes on success and failure paths. Warm up enabled/disabled layers before measurement and record excluded warmup frames. Collect exactly12 alternating blocks x50 counted frames =300 on+300 off at identical tick/camera. Include frame/block index, flag, all CPU components, GPU/unavailable, pacing, seed, tick, camera/zoom, population, alive count, run_id and load snapshot. A30 passes only median(total_cpu_on)<=1.15*median(total_cpu_off) OR <=median(total_cpu_off)+1ms. No CPU number from old pacing may enter numerator/denominator.

`scripts/test/art/test_frame_measure.gd`: injected clock/render backend stub proves priority endpoints, same-frame pairing,12x50 sequence,300/300 samples, metric arithmetic, GPU-zero unavailable, threshold OR boundary, fixed tick/camera guard, excluded warmup, and finally-style restoration on write/error. Stub cannot establish actual performance: runtime measurement and window lock remain mandatory. `_performance_load_status` must include Godot in pgrep pattern as GOAL requires; parent owns foreign-process wait policy.

### API evidence and delayed timestamp handling

Installed app Info.plist reports Godot4.6 (no app launched for this check). [Official RenderingServer documentation](https://docs.godotengine.org/en/stable/classes/class_renderingserver.html#class-renderingserver-method-viewport-get-measured-render-time-cpu) specifies viewport render CPU milliseconds, excludes script processing, and requires measurement enabled. [Frame setup API](https://docs.godotengine.org/en/stable/classes/class_renderingserver.html#class-renderingserver-method-get-frame-setup-time-cpu) also returns milliseconds. Add process microseconds /1000 directly to these values, with no further render-time scaling.

[Godot4.6 viewport implementation](https://github.com/godotengine/godot/blob/4.6-stable/servers/rendering/renderer_viewport.cpp#L1544) converts the CPU timestamp difference by1000 and GPU nanoseconds by1000000. [Compatibility GLES3 implementation](https://github.com/godotengine/godot/blob/4.6-stable/drivers/gles3/storage/utilities.cpp#L324) does capture CPU timestamps; GPU query capture is conditional on GL_API_ENABLED and is_gles_over_gl. Therefore Compatibility CPU is not inherently unsupported, but runtime zero/near-zero samples need explicit availability reporting and cannot establish A30 by assuming zero rendering cost.

GLES3 uses [three timestamp slots](https://github.com/godotengine/godot/blob/4.6-stable/drivers/gles3/storage/utilities.h#L183); the result-retrieval implementation says it queries frame minus2. Consequently render readout at frame_post_draw is not guaranteed to describe that same process callback frame. The collector must either align delayed samples with their process stamp or, conservatively, discard at least3 transition frames after EVERY layer toggle before counting each50-frame block, logging excluded frames. Report remaining per-frame timestamp lag honestly; no on/off boundary sample may leak into the opposite condition. Keep600 counted samples plus recorded warmups/transitions, not600 total callbacks with mixed conditions.

## Coordination and risks

- Capture framing writer owns the measurement integration; do not race it with capture-plan/date remapping work. Agree ownership before implementation.
- Selection and god_hand_panel are shared with Part1 god-cast feed wiring: merge additive edits carefully and re-read before editing.
- Current founding capture metadata always reports marker_visible=true even when offscreen; A32 requires actual observable state, not this inherited boolean.
- Current A30 hardcodes seed20261013 and days28/85; replace from Part0 results, never reuse these numbers as v19 evidence.
- Meter and arrow tests are behavioral stubs; live views, pixel metadata and independent observers are separate acceptance evidence.
