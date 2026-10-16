# Art Pass 9 FFI inventory

Baseline source: `49bc0b7c` (Part 0 merge). Current source inspected on working tree at HEAD `011a5b46305ea9cc30283f8c0f334b302a220b30`, including uncommitted `scripts/ui/nature_renderer.gd` Git blob `ce8eb3dbab9910339e821cc724d22365abb43a03`. Current line references below bind to that source snapshot. This is a static inventory with conditional call formulas, **not a runtime measurement or F4 PASS**. Runtime before/after getter-count samples remain pending. Do not turn source-site totals into measured calls/frame.

## Source sites versus actual FFI calls

The table counts executable references in production `scripts/**/*.gd`, excluding `scripts/test/**`, comments, method definitions and has_method guards. It includes typed calls, literal `.call(...)`, and literal HUD `_snapshot(...)` helper inputs. Current rows count `SimReads.read(...)` and migrated `_snapshot(...)` helper inputs. A single source site inside a loop can execute many times; some sites run only on ready/reset, selection/click, visible panels, tick change or reconciliation. The generic SimReads implementation is excluded from these consumer totals.

| Getter | Baseline consumer sites | Current routed consumer sites | Current FFI behavior in one real-main frame without mutations |
|---|---:|---:|---|
| `get_agent_snapshot` | 15 | 15 | 0 if unused; at most 1 counted miss |
| `get_settlement_snapshot` | 17 | 16 | 0 if unused; at most 1 counted miss |
| `get_construction_snapshot` | 6 | 6 | 0 if unused; at most 1 counted miss |
| `get_building_snapshot` | 8 | 8 | 0 if unused; at most 1 counted miss |
| `get_resource_snapshot` | 7 | 7 | 0 if unused; at most 1 counted miss |
| `get_time_snapshot` | 15 | 14 | 0 if unused; at most 1 counted miss |
| `get_tallies` | 2 | 2 | 0 if unused; at most 1 counted miss |
| `get_recent_deaths` | 3 | 3 | 0 if unused; at most 1 counted miss |
| `get_society_snapshot` | 5 | 5 | 0 if unused; at most 1 counted miss |
| `get_intervention_state` | 6 | 6 | 0 if unused; at most 1 counted miss |
| `get_world_info` | 4 | 4 | 0 if unused; at most 1 counted miss |
| `get_terrain_snapshot` | 4 | 4 | 0 after first world read; 1 on cold/reset world read |

The baseline's 15 agent-snapshot source sites are not a contradiction of the brief's nine routine consumers plus conditional inspector sites: this inventory also includes input/selection and initialization paths. Likewise the 17 settlement sites include conditional/refresh sites. These totals deliberately describe all source paths rather than claiming each runs every frame.

`sim_reads.gd:99–126` returns an existing per-frame entry before calling WorldSim; counters increment only at `:117–118` around actual `.call`. `:110–111` returns cached terrain across frames. `_sync_frame` at `:83–89` clears frame cache/counters. `invalidate` at `:91–92` clears cache only, so same-frame invalidation can legitimately cause a second counted read: the one-call bound is specifically for the frozen framing with no mutations/reset, not a universal bound for capture loops.

## Baseline multiplicities that matter

Let `T` be the number of distinct occupied agent tiles in snapshot row order, `H` the rendered hearth count, and `R` be 1 when `BuildingRenderer._process` invokes `refresh_snapshot` in that frame, otherwise 0. These are symbolic source-derived quantities; no W500 value is invented here.

| Source path at baseline 49bc0b7c | Conditional source formula | Current path |
|---|---|---|
| `agent_renderer.gd:323–351` and `:375–403` | Two separate visited-tile sets and one `get_tile_causal_history` at `:334` and `:386` per distinct tile: **2T history FFI calls per process**, provided valid arrays/world and history method. Both scans run on frozen frames too. | `agent_renderer.gd:342–361`: one deduplicated tile list and one batch through `SimReads.causal_events` at `:359`, only on changed tick or seen-set rearm. |
| `agent_renderer.gd:452–477` (`_activity_context`) | Whole-map terrain snapshot at `:477` on every valid process; building + construction/resource snapshots also read to derive context. | `agent_renderer.gd:462–489` (terrain at `:487`): same consumer behavior routed through SimReads; terrain FFI is once per world, other snapshots at most once per frame across all readers. |
| `building_renderer.gd:62,72,136,285,319–320` | Time reads in ordinary process pipeline: **1 + 4H + R(1 + H)**. `:62` supplies 1; `_animate_hearths` uses `_hour()` twice and winter getter twice per hearth; refresh adds tick read and one `_hour()` per hearth while choosing initial art. | Current `:65,75,131,290,320–322`: time data hoisted before hearth loops. Ordinary process invokes time route **2 + 2R** times, all merged into one real-main FFI miss. Repeated explicit fake refreshes remain pass-through. |
| `building_renderer.gd:136,231,314,351,357` | Settlement reads: **1 + R(2 + 2H)** in ordinary process pipeline. Animate reads once, refresh reads once per hearth in `_update_buildings`, once in stockpiles, once for prop totals, and once per hearth in props. | Current `:130,236,319,359`: hoisted snapshots yield **1 + 3R** route invocations, merged into one real-main FFI miss. |

These building formulas apply to the baseline's normal process pipeline and supported hearth kind, not arbitrary manual calls or reset callbacks. `refresh_snapshot` triggers on a tick change, every 30 frames, or empty buildings (baseline `building_renderer.gd:63–65`; current `:65–68`). Do not multiply an all-source-site count by H or add these formulas to that count; that would count the same sites twice.

## Remaining direct calls and F4 accounting

- `scripts/ui/panels/agent_inspector_panel.gd:220–227` calls `get_agent_detail` once per process only when visible, selected entity bits are valid, and the world exists. Closed inspector in W500 frozen Z4 means **0**. Four memoized agent-snapshot consumers inside inspector are now routed; opening it does not multiply their FFI misses.
- `scripts/ui/world_renderer.gd:79–88` calls `get_influence_overlay` only when `current_channel != CHANNEL_OFF`. Overlay off in the specified F4 framing means **0**. Explicitly retain framing state evidence; do not assume 0 for another framing.
- `world_renderer.gd:129,162` has two alternate input-driven agent-detail paths. They are action/click work, not routine no-input F4 frame reads. `:174` retains click-driven tile causal history; it is not the former AgentRenderer per-frame scan.
- `sim_reads.gd:141–151` counts every real-world batched causal getter under **`get_tile_causal_events`**. Therefore current F4 total is `sum(frame_call_counts.values()) + remaining uncounted direct calls`. **Do not add another +1 for causal events:** that key is already inside the sum. The static fallback at `sim_reads.gd:42–49` is not counted, but the main-scene reader route is the required F4 setup.
- The renderer's guard `agent_renderer.gd:343` permits one batch on first observed/reset tick, a changed tick, or next-frame overflow rearm. Each kind's seen-set clearing sets `_causal_rearm` (`:374–376`, `:411–413`). Steady frozen frames ordinarily issue 0 batches after initial/rearm work; a batch frame issues 1, independent of T/H. Do not replace observed batch counts with this expectation.
- The 11 frame-memoized getter kinds plus at most one causal batch produce a **source-derived ceiling of 12** in a normal frozen frame with warm terrain, or 13 on a cold terrain read, before remaining direct calls. This is a conditional bound, not a measured F4 result. Invalidations, extra explicit process calls or a missing/mismatched reader invalidate that bound and must be investigated, not hidden.

## Diagnostic reads and report contract

`sim_reads.gd:128–139` implements `peek_uncounted`: a fresh getter call without counter increments or cache insertion. Measurement support uses that route so its own world/time/population sampling does not warm the cache or inflate player-reader counts. The calls still exist physically and must be described as excluded harness diagnostics, not zero total FFI at the process level. `perf_frames_p9.gd:140–154` records direct-read framing, copies frame call counts and wall microseconds from the reader, and verifies counter/process-frame alignment after each collected frame.

Baseline has no SimReads instrumentation; baseline JSON frame timing does **not** by itself provide measured getter counts. Report baseline symbolic formulas/source inventory as such. Current per-getter counts, totals, distributions/maxima and sample/frame alignment must come from actual after JSON. Verify a matching reader exists, the selected framing has inspector/overlay closed, and the causal count is included exactly once before assigning F4 PASS/FAIL against the unchanged **20** threshold.

| Evidence item | Status at source-reference refresh |
|---|---|
| Baseline executable consumer-source inventory | Inspected at 49bc0b7c |
| Current routed-source inventory and counter implementation | Inspected at HEAD 011a5b46 plus the NatureRenderer blob above; all 12 routed-site totals rechecked and unchanged |
| Baseline measured per-frame FFI counts | Not collected by SimReads; no fabricated numbers |
| Current W500 frozen Z4 per-getter frame counters | Pending after measurement |
| F4 verdict | Pending runtime evidence; no verdict from source ceiling alone |

## Exact getter reference ledger

Each baseline reference is pinned to git 49bc0b7c. Current references are the HEAD 011a5b46 working-tree snapshot identified above and should be refreshed if source lines move before publication. All current routed consumer sites were enumerated again; their counts remain unchanged. Baseline references and formulas were retained verbatim.

### `get_agent_snapshot`

Baseline: `scripts/ui/activity_trail_renderer.gd:62`, `scripts/ui/agent_renderer.gd:78`, `scripts/ui/building_renderer.gd:511`, `scripts/ui/camera_controller.gd:146`, `scripts/ui/founding_group_renderer.gd:32`, `scripts/ui/need_bar_renderer.gd:48`, `scripts/ui/panels/agent_inspector_panel.gd:205`, `scripts/ui/panels/agent_inspector_panel.gd:253`, `scripts/ui/panels/agent_inspector_panel.gd:269`, `scripts/ui/panels/agent_inspector_panel.gd:296`, `scripts/ui/panels/hud_status_panel.gd:184`, `scripts/ui/panels/hud_topbar.gd:114`, `scripts/ui/panels/hud_topbar.gd:182`, `scripts/ui/seek_viz_renderer.gd:77`, `scripts/ui/world_renderer.gd:134`.

Current: `scripts/ui/activity_trail_renderer.gd:62`, `scripts/ui/agent_renderer.gd:84`, `scripts/ui/building_renderer.gd:523`, `scripts/ui/camera_controller.gd:146`, `scripts/ui/founding_group_renderer.gd:32`, `scripts/ui/need_bar_renderer.gd:48`, `scripts/ui/panels/agent_inspector_panel.gd:205`, `scripts/ui/panels/agent_inspector_panel.gd:253`, `scripts/ui/panels/agent_inspector_panel.gd:269`, `scripts/ui/panels/agent_inspector_panel.gd:296`, `scripts/ui/panels/hud_status_panel.gd:184`, `scripts/ui/panels/hud_topbar.gd:114`, `scripts/ui/panels/hud_topbar.gd:182`, `scripts/ui/seek_viz_renderer.gd:77`, `scripts/ui/world_renderer.gd:134`.

### `get_settlement_snapshot`

Baseline: `scripts/rendering/art/god_hand_effects.gd:80`, `scripts/rendering/art/god_hand_effects.gd:280`, `scripts/ui/building_renderer.gd:136`, `scripts/ui/building_renderer.gd:231`, `scripts/ui/building_renderer.gd:314`, `scripts/ui/building_renderer.gd:351`, `scripts/ui/building_renderer.gd:357`, `scripts/ui/camera_controller.gd:183`, `scripts/ui/daylight_controller.gd:54`, `scripts/ui/nature_renderer.gd:94`, `scripts/ui/panels/god_hand_panel.gd:135`, `scripts/ui/panels/hud_status_panel.gd:113`, `scripts/ui/panels/hud_topbar.gd:122`, `scripts/ui/panels/hud_topbar.gd:169`, `scripts/ui/settlement_labels.gd:54`, `scripts/ui/settlement_overview_renderer.gd:72`, `scripts/ui/terrain_renderer.gd:137`.

Current: `scripts/rendering/art/god_hand_effects.gd:80`, `scripts/rendering/art/god_hand_effects.gd:280`, `scripts/ui/building_renderer.gd:130`, `scripts/ui/building_renderer.gd:236`, `scripts/ui/building_renderer.gd:319`, `scripts/ui/building_renderer.gd:359`, `scripts/ui/camera_controller.gd:183`, `scripts/ui/daylight_controller.gd:54`, `scripts/ui/nature_renderer.gd:98`, `scripts/ui/panels/god_hand_panel.gd:135`, `scripts/ui/panels/hud_status_panel.gd:113`, `scripts/ui/panels/hud_topbar.gd:122`, `scripts/ui/panels/hud_topbar.gd:169`, `scripts/ui/settlement_labels.gd:54`, `scripts/ui/settlement_overview_renderer.gd:72`, `scripts/ui/terrain_renderer.gd:137`.

### `get_construction_snapshot`

Baseline: `scripts/ui/agent_renderer.gd:466`, `scripts/ui/building_renderer.gd:145`, `scripts/ui/nature_renderer.gd:92`, `scripts/ui/panels/hud_status_panel.gd:167`, `scripts/ui/panels/hud_topbar.gd:130`, `scripts/ui/terrain_renderer.gd:136`.

Current: `scripts/ui/agent_renderer.gd:476`, `scripts/ui/building_renderer.gd:150`, `scripts/ui/nature_renderer.gd:96`, `scripts/ui/panels/hud_status_panel.gd:167`, `scripts/ui/panels/hud_topbar.gd:130`, `scripts/ui/terrain_renderer.gd:136`.

### `get_building_snapshot`

Baseline: `scripts/ui/agent_renderer.gd:454`, `scripts/ui/building_renderer.gd:126`, `scripts/ui/building_renderer.gd:512`, `scripts/ui/daylight_controller.gd:63`, `scripts/ui/nature_renderer.gd:91`, `scripts/ui/panels/hud_topbar.gd:138`, `scripts/ui/terrain_renderer.gd:109`, `scripts/ui/terrain_renderer.gd:134`.

Current: `scripts/ui/agent_renderer.gd:464`, `scripts/ui/building_renderer.gd:129`, `scripts/ui/building_renderer.gd:524`, `scripts/ui/daylight_controller.gd:63`, `scripts/ui/nature_renderer.gd:95`, `scripts/ui/panels/hud_topbar.gd:138`, `scripts/ui/terrain_renderer.gd:109`, `scripts/ui/terrain_renderer.gd:134`.

### `get_resource_snapshot`

Baseline: `scripts/rendering/art/god_hand_effects.gd:81`, `scripts/ui/agent_renderer.gd:466`, `scripts/ui/building_renderer.gd:161`, `scripts/ui/building_renderer.gd:181`, `scripts/ui/nature_renderer.gd:93`, `scripts/ui/panels/hud_topbar.gd:195`, `scripts/ui/terrain_renderer.gd:135`.

Current: `scripts/rendering/art/god_hand_effects.gd:81`, `scripts/ui/agent_renderer.gd:476`, `scripts/ui/building_renderer.gd:166`, `scripts/ui/building_renderer.gd:186`, `scripts/ui/nature_renderer.gd:97`, `scripts/ui/panels/hud_topbar.gd:195`, `scripts/ui/terrain_renderer.gd:135`.

### `get_time_snapshot`

Baseline: `scripts/ui/agent_renderer.gd:110`, `scripts/ui/building_renderer.gd:62`, `scripts/ui/building_renderer.gd:72`, `scripts/ui/building_renderer.gd:285`, `scripts/ui/building_renderer.gd:319`, `scripts/ui/building_renderer.gd:320`, `scripts/ui/daylight_controller.gd:44`, `scripts/ui/daylight_controller.gd:100`, `scripts/ui/nature_renderer.gd:166`, `scripts/ui/panels/hud_status_panel.gd:209`, `scripts/ui/panels/hud_status_panel.gd:426`, `scripts/ui/panels/hud_topbar.gd:159`, `scripts/ui/settlement_labels.gd:70`, `scripts/ui/terrain_renderer.gd:184`, `scripts/ui/world_effects.gd:82`.

Current: `scripts/ui/agent_renderer.gd:118`, `scripts/ui/building_renderer.gd:65`, `scripts/ui/building_renderer.gd:75`, `scripts/ui/building_renderer.gd:290`, `scripts/ui/building_renderer.gd:320`, `scripts/ui/daylight_controller.gd:44`, `scripts/ui/daylight_controller.gd:100`, `scripts/ui/nature_renderer.gd:174`, `scripts/ui/panels/hud_status_panel.gd:209`, `scripts/ui/panels/hud_status_panel.gd:426`, `scripts/ui/panels/hud_topbar.gd:159`, `scripts/ui/settlement_labels.gd:70`, `scripts/ui/terrain_renderer.gd:184`, `scripts/ui/world_effects.gd:82`.

### `get_tallies`

Baseline: `scripts/ui/panels/hud_status_panel.gd:192`, `scripts/ui/panels/hud_topbar.gd:202`.

Current: `scripts/ui/panels/hud_status_panel.gd:192`, `scripts/ui/panels/hud_topbar.gd:202`.

### `get_recent_deaths`

Baseline: `scripts/ui/building_renderer.gd:413`, `scripts/ui/death_viz_renderer.gd:66`, `scripts/ui/panels/hud_status_panel.gd:199`.

Current: `scripts/ui/building_renderer.gd:422`, `scripts/ui/death_viz_renderer.gd:66`, `scripts/ui/panels/hud_status_panel.gd:199`.

### `get_society_snapshot`

Baseline: `scripts/ui/agent_renderer.gd:79`, `scripts/ui/founding_group_renderer.gd:32`, `scripts/ui/panels/god_hand_panel.gd:137`, `scripts/ui/panels/hud_status_panel.gd:114`, `scripts/ui/settlement_labels.gd:53`.

Current: `scripts/ui/agent_renderer.gd:85`, `scripts/ui/founding_group_renderer.gd:32`, `scripts/ui/panels/god_hand_panel.gd:137`, `scripts/ui/panels/hud_status_panel.gd:114`, `scripts/ui/settlement_labels.gd:53`.

### `get_intervention_state`

Baseline: `scripts/rendering/art/god_hand_effects.gd:79`, `scripts/rendering/art/god_hand_effects.gd:277`, `scripts/ui/building_renderer.gd:519`, `scripts/ui/panels/god_hand_panel.gd:113`, `scripts/ui/panels/god_hand_panel.gd:154`, `scripts/ui/panels/hud_status_panel.gd:119`.

Current: `scripts/rendering/art/god_hand_effects.gd:79`, `scripts/rendering/art/god_hand_effects.gd:277`, `scripts/ui/building_renderer.gd:541`, `scripts/ui/panels/god_hand_panel.gd:113`, `scripts/ui/panels/god_hand_panel.gd:154`, `scripts/ui/panels/hud_status_panel.gd:119`.

### `get_world_info`

Baseline: `scripts/ui/agent_renderer.gd:72`, `scripts/ui/building_renderer.gd:432`, `scripts/ui/camera_controller.gd:202`, `scripts/ui/world_effects.gd:46`.

Current: `scripts/ui/agent_renderer.gd:78`, `scripts/ui/building_renderer.gd:441`, `scripts/ui/camera_controller.gd:202`, `scripts/ui/world_effects.gd:46`.

### `get_terrain_snapshot`

Baseline: `scripts/ui/agent_renderer.gd:477`, `scripts/ui/nature_renderer.gd:39`, `scripts/ui/terrain_renderer.gd:63`, `scripts/ui/world_effects.gd:47`.

Current: `scripts/ui/agent_renderer.gd:487`, `scripts/ui/nature_renderer.gd:43`, `scripts/ui/terrain_renderer.gd:63`, `scripts/ui/world_effects.gd:47`.

Graft was attempted with `graft skeleton scripts/ui/sim_reads.gd`; the graph reported no definitions indexed for this file and no token-savings estimate. Source was inspected directly.
