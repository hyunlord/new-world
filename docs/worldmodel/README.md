# WorldSim recorded worlds (`worldsim_wm`, recorder schema 1)

WorldSim is a deterministic village simulation. The recorder `worldsim_wm` runs a scene for a number of days, optionally under a schedule of interventions (the god's hand), and writes one directory of NPY arrays per episode: what the renderer sees, plus the hidden state the next step depends on. The same arguments give byte-identical files whatever the worker count (`tools/worldmodel/test_export_determinism.py`).

This page documents the format. The datasets live outside git (`WM_DATA_ROOT`, default `/Users/rexxa/work/worldsim-wm-data`); `sample/` holds two `game_v13` episodes × 10 days with agents. See `DATASET_v0.md` for the v0 splits and `BENCH_v0.md` for the first benchmark.

## Export

```bash
WM_DATA_ROOT=/Users/rexxa/work/worldsim-wm-data \
tools/worldmodel/export.sh --scene game_v13 --seeds 0..4 --days 120 --every 60 \
    [--agents] [--schedule-dir DIR [--include-factual]] [--resume] --workers 4 --out "$WM_DATA_ROOT/mine"
```

- `export.sh` builds the release binary (`rust/target/release/worldsim_wm`) and only writes under `WM_DATA_ROOT` or `.harness/evidence/worldmodel/`. `WM_BIN` selects another binary (no build).
- Scenes: `game`, `game_v9` … `game_v13`. Seeds are a half-open range. Days 1–400.
- Without `--schedule-dir` every seed gives one episode, id `none`. With it, every file `s<seed>_<id>.json` in the directory (sorted by name) is one episode of that seed, plus the no-intervention episode `none` under `--include-factual`; a seed with neither is an error.
- `--resume` keeps an episode whose `meta.json` hashes verify. An episode is written to `<dir>.tmp` and renamed when complete.
- Wall time goes to stderr and `timing.json`, which no hash or check reads.

A schedule is a JSON list of `{"tick": u64, "kind": K, "settlement": "home" | <id>, "amount": u32}` in non-decreasing tick order. Kinds: `gift_food`, `spoil_food`, `gift_wood`, `rot_wood` (units), `rain` (hours: the settlement's fire stays out and burns nothing), `cold_snap` (nights: every fire burns one more unit an hour and an unlit hearth leaves its members cold in any season; `settlement` ignored), `windfall` (units of deadwood placed on the registered spots, one a spot per pass, up to their ceilings, the winter snow cap included; `settlement` ignored). `"home"` is the lowest settlement id at that tick (none: the row applies 0). Rows apply when the completed tick equals `tick`, in file order, after that tick's row is taken and before the next tick runs. Interventions draw no random numbers; an intervention with amount 0 changes nothing.

## An episode directory: `<scene>_s<seed>_<id>/`

Rows are taken at ticks 0, `every`, 2·`every` … `days` × `ticks_per_day` inclusive (120 days at `every` 60 → 2,881 rows). **Row t is the state after t completed ticks, taken before the interventions of tick t.** `interventions.json` pairs with it: an intervention logged at tick t first shows in the row after t.

Float tables are `float32`, NaN where an entity does not exist (a settlement before it forms, an agent not yet born or dead, an absent resource tile). Ids in `*_ids.npy` / `*_keys.npy` are in first-seen order.

| File | dtype, shape | Content |
|---|---|---|
| `meta.json` | JSON | the keys below |
| `global.npy` | float32 [T, 33] | world-level row (fields below) |
| `settlements.npy` | float32 [T, S, 9] | one row per settlement in `settlement_ids.npy` order |
| `settlement_ids.npy` | int64 [S] | settlement ids, first-seen order |
| `terrain.npy` | uint8 [H, W] | terrain code at tick 0 (Plain 0 … Cave 9); `terrain_static` says whether the last row equals it |
| `elevation.npy` | uint8 [H, W] | elevation |
| `building_keys.npy` | int64 [B, 4] | `id`, `kind` (BuildingKind code, 0 hearth), `x`, `y`; first-seen order |
| `building_present.npy` | uint8 [T, B] | 1 while the building is registered |
| `agent_ids.npy` | int64 [N] | agent ids, first-seen order (`--agents`) |
| `agent_alive.npy` | uint8 [T, N] | 1 while alive (`--agents`) |
| `agents.npy` | float32 [T, N, 17] | per-agent row (`--agents`) |
| `resource_keys.npy` | int32 [R, 3] | `kind` (0 food, 1 water, 2 sleep, 3 deadwood), `x`, `y`; tiles at the infinite sentinel (water, camp and shelter sleep tiles) are left out (`--agents`) |
| `resources.npy` | float32 [T, R, 3] | `amount`, `max`, `present` (`--agents`) |
| `latent.npy` | float32 [T, 7] | hidden state the next row depends on |
| `row_digest.npy` | uint64 [T] | `state_digest` of each row (see below) |
| `interventions.json` | JSON | every intervention applied: `tick`, `kind`, `settlement` (null for cold snaps and windfalls), `amount`, `applied` (0 when nothing changed) |
| `timing.json` | JSON | wall seconds (not reproducible; excluded from every hash) |

### `meta.json` keys

`schema_version` (1), `scene`, `seed`, `episode_id` (`none` or the schedule id), `days`, `every`, `rows` (T), `ticks_per_day` (1440), `clock_offset_ticks` (480: tick 0 is 08:00 of display day 0), `git_sha` (the commit the recorder was built from), `agents_recorded`, `schedule` (the schedule file verbatim, or null), `arrays` (per file: `dtype`, `shape`, `sha256`), `fields` (per table: a list of `name`, `dtype`, `unit`, `meaning`, `hidden`), `terrain_static`, `final_fingerprint` (the probe fingerprint after the last tick), `row_rule`, `missing_value`, `not_markov`.

### `global.npy` fields

| Field | Unit | Meaning |
|---|---|---|
| `tick` | ticks | completed simulation ticks of the row |
| `day_index` | days | zero-based display day |
| `minute_of_day` | minutes | display minute, 0–1439 |
| `season` | index | 0 spring, 1 summer, 2 autumn, 3 winter; −1 without seasons |
| `day_of_season` | days | one-based day of the season |
| `year` | years | display day / 28 |
| `winter_severity` | index | 0 mild, 1 normal, 2 harsh, −1 outside a winter or without weights (v12); from 00:00 to 07:00 it reads the evening's winter |
| `alive`, `children`, `elders`, `adults` | agents | living agents; `adults` = alive − children − elders |
| `births` | agents | cumulative births |
| `deaths_starvation`, `deaths_dehydration`, `deaths_combat`, `deaths_old_age` | agents | cumulative deaths by cause |
| `cold_members` | agents | agents carrying the Cold marker |
| `deadwood_total` | units | wood lying on deadwood spots |
| `food_on_bushes_total` | units | food on finite food tiles |
| `adults_working` | agents | settlement adults gathering, building or hauling |
| `idle_adults` | agents | non-child agents that are Idle |
| `meals_from_store_total`, `ground_meals_total` | meals | cumulative meals from the store / at a food tile |
| `forage_trips_total`, `wood_trips_total`, `urgent_wood_trips_total` | trips | cumulative trips |
| `wood_gathered_total` | units | cumulative wood picked up |
| `wood_retargets_total` | events | cumulative re-targets of a wood gatherer whose spot was empty |
| `cold_member_hours_total` | agent-hours | cumulative Cold markers over hourly fire updates |
| `interventions_applied_total` | events | cumulative interventions that changed something |
| `settlements_under_rain` | settlements | settlements whose rain lasts past this row |
| `cold_snap_nights_left` | nights | cold-snap counter |
| `cold_snap_tonight` | flag | the current night is a cold-snap night |

### `settlements.npy` fields

`members` (roster size), `food_stock`, `wood_stock`, `wood_target` (the firewood target; NaN without the rule), `fire_lit` (1 when the last hourly update burned a full hour), `wood_in_hand` (stockpile + carried by members), `formation_x`, `formation_y` (the hearth tile), `rain_ticks_left` (max(0, rain end − tick)).

### `agents.npy` fields

`x`, `y`, `state_tag` (0 idle, 1 seeking, 2 social, 3 consuming), `seek_kind` (0 none, 1 food, 2 water, 3 sleep), `target_kind` (0 none, 1 food, 2 water, 3 sleep, 4 site, 5 agent, 6 gather food, 7 gather wood), `target_x`, `target_y` (−1 without a target), `hunger`, `thirst`, `fatigue` (0–100), `carried_food`, `carried_wood`, `life_stage` (0 adult, 1 child, 2 elder), `cold` (flag), `settlement` (index into `settlement_ids.npy`, −1 none), `age_days`, `lifespan_days`. **`lifespan_days` is marked `hidden`: the game never shows it** (it decides when an agent dies of old age); a model that must work from what a player sees should not read it.

### `resources.npy` fields

`amount` (live amount, clamped to its ceiling as the renderer sees it), `max` (regrowth ceiling), `present` (1 when the resource snapshot lists the tile; amount and max are NaN when it does not).

### `latent.npy` fields

`food_regen_acc`, `wood_regen_acc` (regrowth accumulators in thousandths of a unit), `wood_winter_acc` (the winter spread-regrowth accumulator), `wood_winter_cursor_x`, `wood_winter_cursor_y` (the last deadwood spot that received a winter unit, −1 none), `cold_snap_nights`, `cold_snap_tonight`. Rain lives in the settlement table (`rain_ticks_left`).

### `row_digest.npy`

FNV-1a over the probe fingerprint string followed by every settlement's food and wood stock, the food and deadwood tiles, fires, rain ends, the cold-snap state and the four regrowth accumulators and cursor. Two runs agree on a row exactly when this state agrees; it is how prefix identity of interventions is checked (Z6).

## Loading

```python
import sys; sys.path.insert(0, "tools/worldmodel")
from wm_data import load_episode
ep = load_episode("docs/worldmodel/sample/game_v13_s0_none")   # verifies every sha256
ep.meta["scene"], ep.arrays["global.npy"].shape                 # 'game_v13', (241, 33)
wood = ep.column("settlements.npy", "wood_stock")                # [T, S]
lit = ep.column("settlements.npy", "fire_lit")
```

`load_episode(path, verify=True, arrays=None)` reads `meta.json`, `interventions.json` and the arrays, checks every file's sha256, shape and dtype against `meta.json`, and looks columns up by name (`ep.column(array, field)`). numpy only.

## What is not exported

**The tables plus `latent.npy` are not a complete Markov state.** Agents' decision timers and retry cooldowns, memories, social bookkeeping (relationships, interaction progress, chase timers), bed assignments, construction progress, the influence grid and every agent's random stream are not exported. A model that predicts the next row from this row alone is predicting from a partial observation; the simulation itself is exactly reproducible from (scene, seed, schedule).
