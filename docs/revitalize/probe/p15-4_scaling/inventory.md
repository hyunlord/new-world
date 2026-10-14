## Per-frame caller inventory (6.5)

Read-only inventory of every `WorldSimNode` getter call site in `scripts/` on this branch (`codex/revitalize-2026-10-wip` at Part 1) and on `origin/codex/art-production-2026-10` (`56059e0f`), made by a fresh `general-purpose` subagent and spot-checked (grep of both branches). The *calls per frame at n = 500* column uses T (distinct agent tiles) and H (hearths) from the n = 500 fed stress runs (snapshot table above).

Classes: **PF** = per frame (unconditional `_process`), **PT** = per tick change (counted as PF), **CPF** = conditional per frame, **EV** = event (`_ready`, `world_reset`, click, key, button).
Variables: A = agents, T = distinct tiles occupied by agents (T <= A, every agent tile is queried once per pass), B = buildings, H = hearth buildings (`kind == 0`, H <= B), S = settlements.
All scripts below are instanced in `scenes/main.tscn` (or, on the art branch, created at runtime by an instanced script). Godot runs `_process` even when a node is hidden, so a renderer is only gated when its code checks `visible` itself.
Not called from non-test GDScript on either branch: `get_tile_detail`, `get_event_chain`, `get_relationship_snapshot`.

### A. Working tree `scripts/` (codex/revitalize-2026-10-wip)

| getter | file:line | entry point / chain | class | loop | calls per frame |
|---|---|---|---|---|---|
| get_world_info | ui/agent_renderer.gd:238 | `_process` (also `_on_world_reset` -> `_process(0)`) | PF | — | 1 |
| get_world_info | ui/camera_controller.gd:203 | `_ready` / `world_reset` -> `_on_world_reset` | EV | — | 0 |
| get_terrain_snapshot | ui/terrain_renderer.gd:40 | `_ready` / `world_reset` -> `refresh_snapshot` | EV | — | 0 |
| get_building_snapshot | ui/building_renderer.gd:71 | `_process` -> `refresh_snapshot` -> `_update_buildings`; also `_ready`/reset | CPF: `_frame % 30 == 0` **or `_buildings` empty** | — | 1/30 (1 while no buildings) |
| get_building_snapshot | ui/daylight_controller.gd:52 | `_process` -> `_reconcile_hearths`; also `_ready`/reset | CPF: `_frame % 30 == 0` **or `_glows` empty** (no hearths) | — | 1/30 (1 while no hearth) |
| get_building_snapshot | ui/panels/hud_topbar.gd:138 | `_process` -> `_refresh` (also `_ready`) | PF | — | 1 |
| get_time_snapshot | ui/daylight_controller.gd:37 | `_process` -> `_update_daylight` | PF | — | 1 |
| get_time_snapshot | ui/panels/hud_status_panel.gd:206 | `_process` -> `_refresh` -> `_snapshot()` wrapper (:201) | PF | — | 1 |
| get_tallies | ui/panels/hud_status_panel.gd:169 | `_process` -> `_ingest_snapshots` -> `_snapshot()` | PF | — | 1 |
| get_tallies | ui/panels/hud_status_panel.gd:232 | `_process` -> `_refresh` -> `_snapshot()` | PF | — | 1 |
| get_influence_overlay | ui/world_renderer.gd:85 | `_process` (also reset -> `refresh_snapshot` -> `_process(0)`) | CPF: overlay channel != OFF (SPACE key cycles) | — | 1 when overlay on |
| get_tile_causal_history | ui/agent_renderer.gd:373 | `_process` -> `_ingest_memory_recalls` | PF | **per distinct agent tile** (visited-tile dedupe) | T |
| get_tile_causal_history | ui/agent_renderer.gd:447 | `_process` -> `_ingest_combat_events` | PF | **per distinct agent tile** (separate dedupe dict) | T |
| get_tile_causal_history | ui/world_renderer.gd:164 | left click (no agent hit) -> `_fetch_causal_history` | EV | — | 0 |
| get_agent_snapshot | ui/agent_renderer.gd:247 | `_process` | PF | — | 1 |
| get_agent_snapshot | ui/activity_trail_renderer.gd:60 | `_process` (no `visible` check; still runs at FAR zoom) | PF | — | 1 |
| get_agent_snapshot | ui/need_bar_renderer.gd:57 | `_process` | PF | — | 1 |
| get_agent_snapshot | ui/seek_viz_renderer.gd:72 | `_process` | PF | — | 1 |
| get_agent_snapshot | ui/panels/hud_topbar.gd:114 | `_process` -> `_refresh` | PF | — | 1 |
| get_agent_snapshot | ui/panels/hud_status_panel.gd:161 | `_process` -> `_ingest_snapshots` | PF | — | 1 |
| get_agent_snapshot | ui/camera_controller.gd:147 | `_process` | CPF: auto-track on (KEY_T) | — | 1 when tracking |
| get_agent_snapshot | ui/panels/agent_inspector_panel.gd:184 | `_process` -> `display_agent` -> `_resolve_entity_bits`; also click via world_renderer | CPF: panel visible and an agent selected | — | 1 when panel open |
| get_agent_snapshot | ui/world_renderer.gd:124 | left click -> `_try_agent_click` | EV | — | 0 |
| get_agent_detail | ui/panels/agent_inspector_panel.gd:202 | `_process` | CPF: panel visible and an agent selected | — | 1 when panel open |
| get_agent_detail | ui/world_renderer.gd:152 | left click -> `_try_agent_click` | EV | — | 0 |
| get_construction_snapshot | ui/building_renderer.gd:89 | `_process` -> `refresh_snapshot` | CPF: every 30 frames or no buildings | — | 1/30 |
| get_construction_snapshot | ui/panels/hud_topbar.gd:130 | `_process` -> `_refresh` | PF | — | 1 |
| get_construction_snapshot | ui/panels/hud_status_panel.gd:144 | `_process` -> `_ingest_snapshots` | PF | — | 1 |
| get_resource_snapshot | ui/building_renderer.gd:126 | `_process` -> `refresh_snapshot` -> `_update_bushes` | CPF: every 30 frames or no buildings | — | 1/30 |
| get_resource_snapshot | ui/panels/hud_status_panel.gd:223 | `_process` -> `_refresh` -> `_snapshot()` | PF | — | 1 |
| get_recent_deaths | ui/death_viz_renderer.gd:59 | `_process` | PF | — | 1 |
| get_settlement_snapshot | ui/building_renderer.gd:166 | `_process` -> `refresh_snapshot` -> `_update_stockpiles` | CPF: every 30 frames or no buildings | — | 1/30 |
| get_settlement_snapshot | ui/settlement_overview_renderer.gd:64 | `_process` | CPF: `visible` (zoom < 2.0, MEDIUM/FAR tier; default zoom 4.0 = hidden) | — | 1 when zoomed out |
| get_settlement_snapshot | ui/camera_controller.gd:184 | `_process` -> `_center_on_first_settlement`; also `_ready`/reset | CPF: `_initial_center_pending` (until the first settlement exists) | — | 1 until centred, then 0 |
| get_settlement_snapshot | ui/panels/hud_topbar.gd:122 | `_process` -> `_refresh` | PF | — | 1 |
| get_settlement_snapshot | ui/panels/hud_status_panel.gd:96 | `_process` -> `_ingest_snapshots` | PF | — | 1 |
| get_settlement_snapshot | ui/panels/hud_status_panel.gd:216 | `_process` -> `_refresh` -> `_snapshot()` | PF | — | 1 |

#### A. Summary (working tree)

| getter | PF/PT call sites | calls per frame (unconditional) | plus conditional |
|---|---|---|---|
| get_world_info | 1 | 1 | — |
| get_terrain_snapshot | 0 | 0 | — |
| get_building_snapshot | 1 | 1 | + 2/30 (+2 while no buildings / no hearths) |
| get_time_snapshot | 2 | 2 | — |
| get_tallies | 2 | 2 | — |
| get_influence_overlay | 0 | 0 | +1 overlay on |
| get_tile_causal_history | 2 | **2T** | — |
| get_agent_snapshot | 6 | **6** | +1 auto-track, +1 inspector open |
| get_agent_detail | 0 | 0 | +1 inspector open |
| get_construction_snapshot | 2 | 2 | + 1/30 |
| get_resource_snapshot | 1 | 1 | + 1/30 |
| get_recent_deaths | 1 | 1 | — |
| get_settlement_snapshot | 3 | 3 | + 1/30, +1 zoom < 2, +1 until first settlement |
| **Total** | 21 | **19 + 2T** | |

No getter is inside a per-building/per-settlement loop on this branch; the only loop-driven calls are the 2T tile-history queries.

### B. `origin/codex/art-production-2026-10` `scripts/`

| getter | file:line | entry point / chain | class | loop | calls per frame |
|---|---|---|---|---|---|
| get_world_info | ui/agent_renderer.gd:72 | `_process` | PF | — | 1 |
| get_world_info | ui/building_renderer.gd:432 | `_process` -> `_update_cairns` | PF | — | 1 |
| get_world_info | ui/camera_controller.gd:202 | `_ready` / reset -> `_on_world_reset` | EV | — | 0 |
| get_world_info | ui/world_effects.gd:46 | `_ready` / reset -> `_reset` | EV | — | 0 |
| get_terrain_snapshot | ui/agent_renderer.gd:477 | `_process` -> `_activity_context` (:109) | PF | — | **1 (full map per frame)** |
| get_terrain_snapshot | ui/terrain_renderer.gd:63 | `_ready` / reset -> `refresh_snapshot` | EV | — | 0 |
| get_terrain_snapshot | ui/nature_renderer.gd:39 | `_ready` / reset -> `refresh_snapshot` | EV | — | 0 |
| get_terrain_snapshot | ui/world_effects.gd:47 | `_ready` / reset -> `_reset` | EV | — | 0 |
| get_building_snapshot | ui/agent_renderer.gd:454 | `_process` -> `_activity_context` | PF | — | 1 |
| get_building_snapshot | ui/building_renderer.gd:126 | `_process` -> `refresh_snapshot` -> `_update_buildings` | PT: tick != `_snapshot_tick`, or `_frame % 30 == 0`, or no buildings | — | 1 |
| get_building_snapshot | ui/building_renderer.gd:512 | `_process` -> `_update_shelter_cold` | PF | — | 1 |
| get_building_snapshot | ui/daylight_controller.gd:63 | `_process` -> `_reconcile_hearths` | CPF: every 30 frames or no hearth glows | — | 1/30 |
| get_building_snapshot | ui/nature_renderer.gd:91 | `_process` -> `_reconcile` (fingerprint guard runs *after* the calls) | CPF: every 30 frames | — | 1/30 |
| get_building_snapshot | ui/terrain_renderer.gd:109 | `_process` -> `_refresh_camp` (hash guard after call) | CPF: every 30 frames | — | 1/30 |
| get_building_snapshot | ui/terrain_renderer.gd:134 | `_process` -> `_refresh_decals` (hash guard after call) | CPF: every 30 frames | — | 1/30 |
| get_building_snapshot | ui/panels/hud_topbar.gd:138 | `_process` -> `_refresh` | PF | — | 1 |
| get_time_snapshot | ui/agent_renderer.gd:110 | `_process` | PF | — | 1 |
| get_time_snapshot | ui/building_renderer.gd:62 | `_process` (tick guard read) | PF | — | 1 |
| get_time_snapshot | ui/building_renderer.gd:72 | `_process` -> `refresh_snapshot` | PT | — | 1 |
| get_time_snapshot | ui/building_renderer.gd:285 | `_hour()` helper, called from :136 (refresh), :319 and :320 (`_animate_hearths`) | PT (via :136) + PF (via :319/:320) | **per hearth** | H (PT) + 2H (PF) = 3H |
| get_time_snapshot | ui/building_renderer.gd:319 | `_process` -> `_animate_hearths` (inline `winter_severity`) | PF | **per hearth** | H |
| get_time_snapshot | ui/building_renderer.gd:320 | `_process` -> `_animate_hearths` (inline `winter_severity`) | PF | **per hearth** | H |
| get_time_snapshot | ui/daylight_controller.gd:44 | `_process` -> `_update_daylight` | PF | — | 1 |
| get_time_snapshot | ui/daylight_controller.gd:100 | `_process` -> `_update_doorways` | PF | — | 1 |
| get_time_snapshot | ui/nature_renderer.gd:166 | `_process` -> `_update_canopies` | PF | — | 1 |
| get_time_snapshot | ui/terrain_renderer.gd:184 | `_process` -> `_update_season` | PF | — | 1 |
| get_time_snapshot | ui/world_effects.gd:82 | `_process` | PF | — | 1 |
| get_time_snapshot | ui/settlement_labels.gd:70 | `_process` (tick guard read) | PF | — | 1 |
| get_time_snapshot | ui/panels/hud_status_panel.gd:209 | `_read_clock()` (`_clock_provider` is never set outside tests) from :106 `_ingest_snapshots`, :230 `_prune_old`, :215 `_push` | PF (x2: :106, :230 via `_process`) + 2 per new feed line (`_push` -> :215 and -> `_prune_old`) | — | 2 (+2 per pushed line) |
| get_time_snapshot | ui/panels/hud_status_panel.gd:426 | `_process` -> `_ingest_snapshots` -> `_ingest_society` (default argument of `Dictionary.get`, always evaluated) | PF | — | 1 |
| get_time_snapshot | ui/panels/hud_topbar.gd:159 | `_process` -> `_refresh` -> `_snapshot()` (:222) | PF | — | 1 |
| get_tallies | ui/panels/hud_status_panel.gd:192 | `_process` -> `_ingest_snapshots` -> `_snapshot()` (:309) | PF | — | 1 |
| get_tallies | ui/panels/hud_topbar.gd:202 | `_process` -> `_refresh` -> `_snapshot()` | PF | — | 1 |
| get_influence_overlay | ui/world_renderer.gd:85 | `_process` | CPF: overlay channel != OFF | — | 1 when overlay on |
| get_tile_causal_history | ui/agent_renderer.gd:334 | `_process` -> `_ingest_memory_recalls` | PF | **per distinct agent tile** | T |
| get_tile_causal_history | ui/agent_renderer.gd:386 | `_process` -> `_ingest_combat_events` | PF | **per distinct agent tile** | T |
| get_tile_causal_history | ui/world_renderer.gd:174 | click -> `_fetch_causal_history` | EV | — | 0 |
| get_agent_snapshot | ui/agent_renderer.gd:78 | `_process` | PF | — | 1 |
| get_agent_snapshot | ui/founding_group_renderer.gd:32 | `_process` (child node created by AgentRenderer `_ready`) | PF | — | 1 |
| get_agent_snapshot | ui/building_renderer.gd:511 | `_process` -> `_update_shelter_cold` | PF | — | 1 |
| get_agent_snapshot | ui/activity_trail_renderer.gd:62 | `_process` (no `visible` check) | PF | — | 1 |
| get_agent_snapshot | ui/need_bar_renderer.gd:48 | `_process` | PF | — | 1 |
| get_agent_snapshot | ui/seek_viz_renderer.gd:77 | `_process` | PF | — | 1 |
| get_agent_snapshot | ui/panels/hud_status_panel.gd:184 | `_process` -> `_ingest_snapshots` | PF | — | 1 |
| get_agent_snapshot | ui/panels/hud_topbar.gd:114 | `_process` -> `_refresh` | PF | — | 1 |
| get_agent_snapshot | ui/panels/hud_topbar.gd:182 | `_process` -> `_refresh` -> `_snapshot()` (cold count) | PF | — | 1 |
| get_agent_snapshot | ui/camera_controller.gd:146 | `_process` | CPF: auto-track on | — | 1 when tracking |
| get_agent_snapshot | ui/panels/agent_inspector_panel.gd:205 | `_process` -> `display_agent` -> `_resolve_entity_bits` | CPF: panel open | — | 1 when open |
| get_agent_snapshot | ui/panels/agent_inspector_panel.gd:253 | `_process` -> `display_agent` -> `_carried_amount` (woods; then foods via `_carried_food` if wood == 0) | CPF: panel open | — | 1-2 when open |
| get_agent_snapshot | ui/panels/agent_inspector_panel.gd:269 | `_process` -> `display_agent` -> `_age_line` | CPF: panel open | — | 1 when open |
| get_agent_snapshot | ui/panels/agent_inspector_panel.gd:296 | `_process` -> `display_agent` -> `_is_cold` | CPF: panel open | — | 1 when open |
| get_agent_snapshot | ui/world_renderer.gd:134 | click -> `_try_agent_click` | EV | — | 0 |
| get_agent_detail | ui/panels/agent_inspector_panel.gd:223 | `_process` | CPF: panel open | — | 1 when open |
| get_agent_detail | ui/world_renderer.gd:129 | click on a drawn sprite -> `_try_agent_click` | EV | — | 0 |
| get_agent_detail | ui/world_renderer.gd:162 | click (radius fallback) -> `_try_agent_click` | EV | — | 0 |
| get_construction_snapshot | ui/agent_renderer.gd:466 | `_process` -> `_activity_context` (loop over the 2 strings "sites"/"food"; ternary picks this once) | PF | — | 1 |
| get_construction_snapshot | ui/building_renderer.gd:145 | `_process` -> `refresh_snapshot` | PT | — | 1 |
| get_construction_snapshot | ui/nature_renderer.gd:92 | `_process` -> `_reconcile` | CPF: every 30 frames | — | 1/30 |
| get_construction_snapshot | ui/terrain_renderer.gd:136 | `_process` -> `_refresh_decals` | CPF: every 30 frames | — | 1/30 |
| get_construction_snapshot | ui/panels/hud_status_panel.gd:167 | `_process` -> `_ingest_snapshots` | PF | — | 1 |
| get_construction_snapshot | ui/panels/hud_topbar.gd:130 | `_process` -> `_refresh` | PF | — | 1 |
| get_resource_snapshot | ui/agent_renderer.gd:466 | `_process` -> `_activity_context` (same line, "food" iteration) | PF | — | 1 |
| get_resource_snapshot | ui/building_renderer.gd:161 | `_process` -> `refresh_snapshot` -> `_update_bushes` | PT | — | 1 |
| get_resource_snapshot | ui/building_renderer.gd:181 | `_process` -> `refresh_snapshot` -> `_update_deadwood` | PT | — | 1 |
| get_resource_snapshot | ui/nature_renderer.gd:93 | `_process` -> `_reconcile` | CPF: every 30 frames | — | 1/30 |
| get_resource_snapshot | ui/terrain_renderer.gd:135 | `_process` -> `_refresh_decals` | CPF: every 30 frames | — | 1/30 |
| get_resource_snapshot | rendering/art/god_hand_effects.gd:81 | `_process` | PF | — | 1 |
| get_resource_snapshot | ui/panels/hud_topbar.gd:195 | `_process` -> `_refresh` -> `_snapshot()` | PF | — | 1 |
| get_recent_deaths | ui/building_renderer.gd:413 | `_process` -> `_update_cairns` | PF | — | 1 |
| get_recent_deaths | ui/death_viz_renderer.gd:66 | `_process` | PF | — | 1 |
| get_recent_deaths | ui/panels/hud_status_panel.gd:199 | `_process` -> `_ingest_snapshots` -> `_snapshot()` | PF | — | 1 |
| get_settlement_snapshot | ui/building_renderer.gd:136 | `_process` -> `refresh_snapshot` -> `_update_buildings` (only the `kind == 0` ternary branch) | PT | **per hearth** | H |
| get_settlement_snapshot | ui/building_renderer.gd:231 | `_process` -> `refresh_snapshot` -> `_update_stockpiles` | PT | — | 1 |
| get_settlement_snapshot | ui/building_renderer.gd:314 | `_process` -> `_animate_hearths` | PF | — | 1 |
| get_settlement_snapshot | ui/building_renderer.gd:351 | `_process` -> `refresh_snapshot` -> `_update_camp_props` | PT | — | 1 |
| get_settlement_snapshot | ui/building_renderer.gd:357 | `_process` -> `refresh_snapshot` -> `_update_camp_props` | PT | **per hearth** (loop over `_buildings`, kind 0) | H |
| get_settlement_snapshot | ui/daylight_controller.gd:54 | `_process` -> `_update_daylight` | PF | — | 1 |
| get_settlement_snapshot | ui/nature_renderer.gd:94 | `_process` -> `_reconcile` | CPF: every 30 frames | — | 1/30 |
| get_settlement_snapshot | ui/terrain_renderer.gd:137 | `_process` -> `_refresh_decals` | CPF: every 30 frames | — | 1/30 |
| get_settlement_snapshot | rendering/art/god_hand_effects.gd:80 | `_process` | PF | — | 1 |
| get_settlement_snapshot | rendering/art/god_hand_effects.gd:280 | god-hand button -> `cast_kind` -> `notify_cast` | EV | — | 0 |
| get_settlement_snapshot | ui/settlement_labels.gd:54 | `_process` -> `refresh_snapshot` when time tick != cached society tick (node created by SettlementOverviewRenderer) | PT | — | 1 |
| get_settlement_snapshot | ui/settlement_overview_renderer.gd:72 | `_process` | CPF: `visible` (zoom < 2.0) and `layers_enabled` | — | 1 when zoomed out |
| get_settlement_snapshot | ui/camera_controller.gd:183 | `_process` -> `_center_on_first_settlement` | CPF: until first settlement exists | — | 1 until centred |
| get_settlement_snapshot | ui/panels/god_hand_panel.gd:135 | `_process` -> `refresh_state` | PF | — | 1 |
| get_settlement_snapshot | ui/panels/hud_status_panel.gd:113 | `_process` -> `_ingest_snapshots` | PF | — | 1 |
| get_settlement_snapshot | ui/panels/hud_topbar.gd:122 | `_process` -> `_refresh` | PF | — | 1 |
| get_settlement_snapshot | ui/panels/hud_topbar.gd:169 | `_process` -> `_refresh` -> `_snapshot()` | PF | — | 1 |
| get_society_snapshot | ui/agent_renderer.gd:79 | `_process` | PF | — | 1 |
| get_society_snapshot | ui/founding_group_renderer.gd:32 | `_process` | PF | — | 1 |
| get_society_snapshot | ui/settlement_labels.gd:53 | `_process` -> `refresh_snapshot` (tick guard) | PT | — | 1 |
| get_society_snapshot | ui/panels/god_hand_panel.gd:137 | `_process` -> `refresh_state` | PF | — | 1 |
| get_society_snapshot | ui/panels/hud_status_panel.gd:114 | `_process` -> `_ingest_snapshots` -> `_snapshot()` | PF | — | 1 |
| get_intervention_state | ui/building_renderer.gd:519 | `_process` -> `_update_shelter_cold` | PF | — | 1 |
| get_intervention_state | rendering/art/god_hand_effects.gd:79 | `_process` | PF | — | 1 |
| get_intervention_state | rendering/art/god_hand_effects.gd:277 | god-hand button -> `notify_cast` | EV | — | 0 |
| get_intervention_state | ui/panels/god_hand_panel.gd:113 | god-hand button -> `cast_kind` | EV | — | 0 |
| get_intervention_state | ui/panels/god_hand_panel.gd:154 | `_process` -> `refresh_state` | PF | — | 1 |
| get_intervention_state | ui/panels/hud_status_panel.gd:119 | `_process` -> `_ingest_snapshots` -> `_snapshot()` | PF | — | 1 |

#### B. Summary (art branch; PT counted as PF)

| getter | PF/PT call sites | calls per frame (unconditional) | plus conditional |
|---|---|---|---|
| get_world_info | 2 | 2 | — |
| get_terrain_snapshot | 1 | 1 | — |
| get_building_snapshot | 4 | 4 | + 4/30 |
| get_time_snapshot | 15 (lines 62, 72, 285, 319, 320 in building_renderer + 10) | **13 + 5H** | +2 per new feed line |
| get_tallies | 2 | 2 | — |
| get_influence_overlay | 0 | 0 | +1 overlay on |
| get_tile_causal_history | 2 | **2T** | — |
| get_agent_snapshot | 9 | **9** | +1 auto-track, +4..5 inspector open |
| get_agent_detail | 0 | 0 | +1 inspector open |
| get_construction_snapshot | 4 | 4 | + 2/30 |
| get_resource_snapshot | 5 | 5 | + 2/30 |
| get_recent_deaths | 3 | 3 | — |
| get_settlement_snapshot | 12 | **10 + 2H** | + 2/30, +1 zoom < 2, +1 until first settlement |
| get_society_snapshot | 5 | 5 | — |
| get_intervention_state | 4 | 4 | — |
| **Total** | 68 | **62 + 7H + 2T** | |

Building_renderer alone contributes 14 + 7H per frame (time 2+5H, settlement 3+2H, building 2, resource 2, construction 1, agent 1, deaths 1, world_info 1, intervention 1).

### Notes and uncertainties

1. **Per tick change vs per frame.** building_renderer.gd (art) refreshes when `tick != _snapshot_tick`, every 30 frames, or while `_buildings` is empty. settlement_labels.gd refreshes when the time-snapshot tick differs from the society snapshot's `current_tick`. How often the tick changes depends on TPS vs FPS and sim speed (keys 1-4). Paused means about 0 refreshes. These are counted as per frame above, as requested.
2. **Guard caches that don't save the call.** nature_renderer `_reconcile`, terrain_renderer `_refresh_camp` and `_refresh_decals` hash the snapshots *after* fetching them, so the fingerprint skips only the rebuild, not the getter calls. building_renderer's tick cache is the only guard that actually skips getter calls. The tick reads at :62 and settlement_labels :70 run every frame either way.
3. **"Every 30 frames OR empty" fallbacks.** WT building_renderer:71/89/126/166 and art building_renderer refresh run every frame while no buildings exist. WT and art daylight `_reconcile_hearths` run every frame while there is no hearth (kind 0), which is the normal case before a campfire exists.
4. **Short-circuit evaluation.** art building_renderer:136 is in a loop over all B buildings, but the GDScript ternary evaluates `get_settlement_snapshot` and `_hour()` only for `kind == 0`, so H rather than B. :357 likewise runs only for kind 0. On art :466 sits in a 2-iteration loop: each of construction and resource is called once.
5. **Eager default argument.** art hud_status_panel:426 calls `get_time_snapshot` every frame even when `society` has `current_tick`, because GDScript evaluates arguments before `Dictionary.get`.
6. **`_read_clock` (art hud_status_panel:209).** `_clock_provider` is assigned only in tests, so in game every `_read_clock()` hits the FFI: twice per frame, plus twice per pushed feed line (`_push` -> :215, then `_push` -> `_prune_old` -> :230).
7. **has_method guards.** Calls to `get_society_snapshot` and `get_intervention_state` are wrapped in `has_method`. Both `#[func]` exist in `rust/crates/sim-bridge/src/ffi/world_node.rs` on the art branch, so they count as live. Neither getter exists on the working tree.
8. **Hidden nodes still process.** activity_trail_renderer (both branches) is hidden at FAR zoom by zoom_lod_controller but has no `visible` check, so its getter runs every frame. settlement_overview_renderer does check `visible` (shown only at zoom < 2.0; default zoom is 4.0).
9. **Event paths that reuse the same sites.**
   - `world_reset` re-enters the `_process`/refresh code in agent_renderer, world_renderer, hud_status_panel, and others.
   - On art, a god-hand button press (`cast_kind`) calls `hud_status_panel._ingest_snapshots` twice, then `notify_cast`, then `refresh_state`. That is about 2x9 + 6 extra calls on that frame (more if feed lines are pushed).
   - A click on an agent calls `display_agent`, which hits the inspector agent-snapshot sites once.
10. **Inspector count (art).** :253 runs once for `carried_woods`, plus a second time for `carried_foods` when the carried wood is 0. So 4-5 agent snapshots plus 1 `get_agent_detail` per frame while the panel is open (WT: 1 + 1).
11. **Unused getters.** `get_tile_detail`, `get_event_chain` and `get_relationship_snapshot` appear only in comments (causal_panel.gd:5-6) or tests.
12. **Excluded test-only scripts that call getters.**
   - WT (5): p4_gamma_rendering/harness_agent_rendering.gd, harness_visual_verify.gd, capture_framings.gd, fix_settlement_marker/harness_marker_fixed_a12.gd, stockpile_feed_2_5c/harness_2_5c.gd.
   - Art (38): the same 5, plus 33 under scripts/test/art/ (test_*.gd, capture_*.gd, measure_days_p7/p8.gd, probe_p6.gd, frame_measure_session.gd, qa_offscreen_marker_windowed.gd).
