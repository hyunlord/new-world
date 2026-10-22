# Snapshot schema — the bridge's read surface

Schema version: 3 (M3: built space, rooms, the place preview and the overlay's `item_covered`; M2 added the economy reads and the overlay's node, mark and quality columns).

Every `#[func] fn get_*` that `WorldSimNode` (`rust/crates/sim-bridge/src/ffi/node.rs`) exposes to GDScript, with the
keys it returns, their types, id spaces, units and row order, and the pure collector that backs it. A `#[func]` body
only calls a collector (`sim_bridge::ffi::collect_*`, callable headless from sim-test) and a `*_to_dict` marshaller;
two views built on the same collectors must agree (ADR 0016, clause C2). The second view is the text renderer
(`rust/crates/sim-test/src/probe/ascii.rs`, `worldsim_probe --scene colony --ascii`, section "Text renderer glyphs").

Checked by `python3 tools/v8/check_docs_helpers.py schema` (`tools/v8/check_docs.sh` check 10): every `get_*`
method found by `tools/v8/bridge_api.py` has a `### get_x` section here, and the first table column of that section
names exactly the keys the getter sets (its own `dict.set("k", …)` calls plus those of the `*_to_dict` /
`*_to_variant_array` builders it calls). An absent or extra getter or key fails.

## Conventions

- **Types** are the Godot types GDScript receives: `int` (64-bit; Rust `i64`/`i32`/`u8` scalars), `float` (64-bit),
  `bool`, `String`, `Vector2i`, `Vector4i`, and packed columns `PackedInt32Array`, `PackedInt64Array`,
  `PackedByteArray` (0–255), `PackedFloat32Array`. Feature-detect every key (`snap.has(k)`) and never write to a
  returned array (`duplicate()` first; packed arrays are shared with the `SimReads` cache).
- **Columns**: a getter returning parallel packed columns has one row per index; every column has the same length.
- **Id spaces** (never mix them): **AgentId** (`Agent.id`, a monotonically minted `u64` sent as `int`; stable for
  the life of a world), **entity bits** (`hecs::Entity::to_bits()` as a signed `int`; stable within one engine
  session, not across `reset_world`), **row index** (a position in one snapshot; never a join key), **BuildingId**
  (`building_registry` key), **SettlementId**. People are joined by AgentId, porters and activities by entity bits.
- **Units**: 1 tick = 1 game minute; 1,440 ticks = 1 day (`ticks_per_day`); tiles are integer map cells, `(0, 0)`
  top-left, x east, y south; permille = 0–1,000.
- **Ticks**: `tick` / `current_tick` is the number of completed ticks. `advance_ticks(n)` returns the final absolute
  tick, not a delta. A read reflects the state after the last completed tick.
- **Change counters**: `get_colony_versions` returns `trees_version` and `overlay_version`; `get_trees` and
  `get_colony_overlay` carry the same counter as `version`. Fetch them again only when a counter changed. Terrain and
  world info change only on `reset_world` / `reset_world_stress` (signal `world_reset`, emitted deferred).
- **Caching**: argument-less getters are read through `SimReads.read(world, "get_x")` (`scripts/core/sim_reads.gd`
  `GETTERS`), frozen once per frame; getters with arguments are called directly and never per node per frame.
- **Colony getters in a baseline world** return their empty form (version 0, empty columns).

## Clock and world

### `get_time_snapshot`
`collect_time_snapshot(&SimEngine) -> TimeSnapshot` → `time_snapshot_to_dict` (`ffi/snapshot.rs`).

| key | type | meaning |
|---|---|---|
| `tick` | int | completed ticks |
| `ticks_per_day` | int | ticks in a day (1,440); 0 disables the clock |
| `day_index` | int | zero-based day of `tick + clock_offset_ticks` |
| `time_of_day` | float | hour 0–24 (the colony starts at 08:00) |
| `season` | int | 0 spring, 1 summer, 2 autumn, 3 winter; 0 without seasons |
| `day_of_season` | int | one-based day in the season; 0 without seasons |
| `days_per_season` | int | days per season (`WorldRules::season_days`); 0 without seasons |
| `winter_severity` | int | 0 mild, 1 normal, 2 harsh; -1 outside a winter or without the rule |
| `year_index` | int | M1.5: zero-based calendar year of `day_index`; 0 without seasons |
| `season_index` | int | M1.5: zero-based season (same value as `season`) |
| `day_of_year` | int | M1.5: one-based day in the year; 0 without seasons |
| `days_per_year` | int | M1.5: days per calendar year; 0 without seasons |

### `get_world_info`
`collect_world_info(&SimResources) -> WorldInfoSnapshot` → `world_info_to_dict` (`ffi/snapshot.rs`). Fixed per world.

| key | type | meaning |
|---|---|---|
| `seed` | int | world seed (`reset_world(seed)` sets it; the game starts on `DEFAULT_GAME_SEED` 20261010) |
| `width` | int | map width in tiles (colony: 192) |
| `height` | int | map height in tiles (colony: 192) |
| `ticks_per_day` | int | ticks per day |
| `move_interval_ticks` | int | ticks between movement steps (the view tweens a step over this) |
| `clock_offset_ticks` | int | display clock offset (colony: 480 = 08:00) |
| `ticks_per_year` | int | M1.5: `CalendarRule::ticks_per_year`; ages in years are whole `age_ticks / ticks_per_year` |

### `get_tallies`
`collect_tallies(&SimResources) -> Tallies` → `tallies_to_dict` (`ffi/snapshot.rs`). Cumulative counters since tick 0,
never the rolling event buffers.

| key | type | meaning |
|---|---|---|
| `births` | int | births |
| `deaths_starvation` | int | deaths by starvation |
| `deaths_dehydration` | int | deaths by dehydration |
| `deaths_combat` | int | deaths in combat |
| `deaths_old_age` | int | deaths of old age |
| `deaths_exposure` | int | cold-night deaths (exposure rule) |
| `fissions` | int | village fissions completed |
| `fissions_cancelled` | int | fissions cancelled |
| `remnant_merges` | int | remnant villages merged |
| `area_floor_placed` | int | settled-area floor wood placed |
| `meals_from_store` | int | meals eaten from a store |
| `ground_meals` | int | meals eaten on the ground |
| `wood_trips` | int | wood gathering trips |
| `wood_gathered` | int | wood units gathered |
| `gather_preempted` | int | gather trips preempted |
| `cold_member_hours` | int | member-hours spent cold |

### `get_terrain_snapshot`
`collect_terrain_snapshot(&SimResources) -> TerrainSnapshot` → `terrain_snapshot_to_dict` (`ffi/snapshot.rs`).
Row-major (`index = y * width + x`), fixed per world.

| key | type | meaning |
|---|---|---|
| `width` | int | map width |
| `height` | int | map height |
| `terrain` | PackedByteArray | `TerrainType` in declaration order: 0 Plain, 1 Forest, 2 Mountain, 3 Hill, 4 River, 5 Coast, 6 Desert, 7 Tundra, 8 Swamp, 9 Cave |
| `elevation` | PackedByteArray | elevation byte per tile |

### `get_tile_detail`
`get_tile_detail(x, y)`: built in the `#[func]` body from `resources.influence_grid` (`InfluenceGrid::sample`; key names
from `channel_key` in `ffi/causal.rs`). Argument tile in map coordinates.

| key | type | meaning |
|---|---|---|
| `tile_x` | int | the argument x |
| `tile_y` | int | the argument y |
| `in_bounds` | bool | whether the tile is on the map |
| `warmth` | int | influence channel value 0–255 (0 when out of bounds) |
| `light` | int | influence channel value |
| `noise` | int | influence channel value |
| `food_aroma` | int | influence channel value |
| `danger` | int | influence channel value |
| `social` | int | influence channel value |
| `spiritual` | int | influence channel value |
| `beauty` | int | influence channel value |

### `get_influence_overlay`
`get_influence_overlay(channel) -> PackedByteArray`: the current buffer of influence channel `channel` (0 warmth,
1 light, 2 noise, 3 food_aroma, 4 danger, 5 social, 6 spiritual, 7 beauty), row-major `width × height` bytes; empty for
an unknown channel. Keys: none (not a dictionary).

## People

### `get_agent_snapshot`
`collect_agent_snapshot(&World) -> Vec<AgentSnapshotRow>` → `agent_rows_to_dict` (`ffi/snapshot.rs`), plus columns
the `#[func]` adds in the same row order: `collect_agent_ages_days`, `collect_agent_ages_years`,
`collect_agent_infancy_and_pregnancy`, `collect_agent_colds` (`ffi/snapshot.rs`) and `collect_agent_jobs`
(`ffi/colony.rs`). One row per entity with `Agent` and `Position`, in hecs query order (archetype iteration: stable
between two reads of an unchanged world, **not** sorted by id and not stable across ticks — join by `agent_ids` or
`ids`).

| key | type | meaning |
|---|---|---|
| `ids` | PackedInt64Array | entity bits (the argument of `get_agent_detail`, `get_agent_family`, …) |
| `agent_ids` | PackedInt64Array | AgentId |
| `xs` | PackedInt32Array | tile x |
| `ys` | PackedInt32Array | tile y |
| `states` | PackedByteArray | 0 idle, 1 seeking, 2 consuming with an agent (social), 3 consuming (other), 4 seeking/consuming an activity |
| `seek_kinds` | PackedByteArray | resource trip: 0 none, 1 food, 2 water, 3 sleep |
| `target_xs` | PackedInt32Array | `SeekTarget` tile x, -1 without one |
| `target_ys` | PackedInt32Array | `SeekTarget` tile y, -1 without one |
| `target_kinds` | PackedByteArray | 0 none, 1 food, 2 water, 3 sleep, 4 construction site, 5 agent, 6 gather food, 7 gather wood, 8 activity, 9 job |
| `hungers` | PackedFloat32Array | hunger 0–100 (100 = sated); 0 without the component |
| `thirsts` | PackedFloat32Array | thirst 0–100 |
| `sleeps` | PackedFloat32Array | fatigue 0–100 (`Sleep.fatigue`) |
| `carried_foods` | PackedInt32Array | `Inventory` food units |
| `carried_woods` | PackedInt32Array | `Inventory` wood units |
| `life_stages` | PackedByteArray | 0 adult, 1 child, 2 elder |
| `ages_days` | PackedInt32Array | whole days of age; -1 without `Life` |
| `ages_years` | PackedInt32Array | M1.5: whole years (`age_ticks / ticks_per_year`); -1 without `Life` |
| `infants` | PackedByteArray | M1.5: 1 for an infant (`Child::is_infant` under `infant_ticks`), else 0 |
| `pregnant_days` | PackedInt32Array | M1.5: whole days left of a pregnancy (`Pregnant::days_left`); -1 when not pregnant |
| `colds` | PackedByteArray | 1 with the `Cold` marker |
| `jobs` | PackedByteArray | `JobKind::code`: 0 none, 1 chop, 2 haul, 3 deliver, 4 build, 5 quarry, 6 supply, 7 craft, 8 gather (M2) |
| `carry_kinds` | PackedInt32Array | carried item kind, -1 none, else `ItemKind::code`: 0 log, 1 firewood, 2 stone, 3 flint, 4 flint blade, 5 stone axe, 6 hide (M2) |
| `carry_counts` | PackedInt32Array | M1: carried item count |

A person sleeps when `states` is 3 and `target_kinds` is 3 (the agent layer's sleep frame; the text renderer's `z`).

### `get_agent_detail`
`get_agent_detail(entity_bits)`: `collect_agent_detail(&World, entity_bits) -> AgentDetailRow` →
`agent_detail_to_dict` (`ffi/reads_agent.rs`). Exactly the nine keys of `AGENT_DETAIL_DICT_KEYS`; zeros and
`found: false` when the bits are not a live agent with the full need bundle.

| key | type | meaning |
|---|---|---|
| `found` | bool | the entity exists with `(Agent, Position, AgentState, Hunger, Thirst, Sleep)` |
| `agent_id` | int | AgentId |
| `x` | int | tile x |
| `y` | int | tile y |
| `state_tag` | int | as `states` in `get_agent_snapshot` |
| `hunger` | float | hunger 0–100 |
| `thirst` | float | thirst 0–100 |
| `sleep` | float | fatigue 0–100 |
| `target_kind` | int | as `target_kinds` in `get_agent_snapshot` |

### `get_agent_temperament`
`get_agent_temperament(entity_bits)`: `collect_agent_temperament` → `agent_temperament_to_dict` (`ffi/reads_agent.rs`).

| key | type | meaning |
|---|---|---|
| `found` | bool | the entity has a `Temperament` |
| `ns` | int | novelty seeking, permille |
| `ha` | int | harm avoidance, permille |
| `rd` | int | reward dependence, permille |
| `p` | int | persistence, permille |

### `get_agent_family`
`get_agent_family(entity_bits)`: `collect_agent_family(&World, entity_bits)` → `agent_family_to_dict` (`ffi/family.rs`).
Input in entity bits; links in AgentId.

| key | type | meaning |
|---|---|---|
| `found` | bool | the entity is an agent with family components |
| `female` | bool | sex |
| `mother` | int | mother's AgentId, -1 unknown |
| `father` | int | father's AgentId, -1 unknown |
| `partner` | int | partner's AgentId, -1 none |
| `children` | PackedInt64Array | living children's AgentIds, sorted |

### `get_agent_id`
`get_agent_id(entity_bits) -> int`: `collect_agent_id` (`ffi/chronicle.rs`); the AgentId of the entity, or -1. Keys:
none (a scalar).

### `get_activity_snapshot`
`collect_activity_snapshot(&World) -> Vec<ActivitySnapshotRow>` → `activity_rows_to_dict` (`ffi/activity.rs`). Rows of
agents with an `Activity`, ordered by `(AgentId, entity bits)`.

| key | type | meaning |
|---|---|---|
| `ids` | PackedInt64Array | entity bits (join with `get_agent_snapshot().ids`) |
| `kinds` | PackedByteArray | 0 TendFire, 1 Craft, 2 Repair, 3 HearthCircle, 4 Explore, 5 Play, 6 SitByFire |
| `place_x` | PackedInt32Array | the activity's place, tile x |
| `place_y` | PackedInt32Array | the activity's place, tile y |
| `until_tick` | PackedInt64Array | completion tick after arrival; -1 while walking there |

### `get_relationship_snapshot`
`collect_relationship_snapshot(&SimResources)` → `relationship_rows_to_variant_array` (`ffi/reads_agent.rs`).
Returns an `Array` of dictionaries, one per pair with familiarity or hostility above 0, in `resources.relationships`
iteration order (a hash map: sort before relying on order).

| key | type | meaning |
|---|---|---|
| `id_a` | int | the smaller AgentId of the pair |
| `id_b` | int | the larger AgentId |
| `familiarity` | float | 0–1 |
| `hostility` | float | 0–1 |

### `get_porter_snapshot`
`collect_porter_snapshot(&World)` → `porter_rows_to_dict` (`ffi/relations.rs`). Every porter on a trip, by AgentId.
`SimReads` caches it once per tick.

| key | type | meaning |
|---|---|---|
| `ids` | PackedInt64Array | entity bits |
| `from` | PackedInt64Array | SettlementId the trip starts from |
| `to` | PackedInt64Array | SettlementId it goes to |
| `kind` | PackedByteArray | 0 food, 1 wood |
| `amount` | PackedInt32Array | units carried |
| `stage` | PackedByteArray | 0 outbound, 1 returning |

## Chronicle

### `get_person`
`get_person(agent_id)`: `collect_person(&World, &Chronicle, agent_id) -> PersonView` → `person_to_dict`
(`ffi/chronicle.rs`). Works for the dead. Unknown values are -1.

| key | type | meaning |
|---|---|---|
| `found` | bool | the person table has this AgentId |
| `alive` | bool | a living entity has this AgentId |
| `female` | int | -1 unknown, 0 male, 1 female |
| `born_tick` | int | birth tick (founders before 0) |
| `died_tick` | int | death tick; -1 while alive |
| `death_reason` | int | `DeathReason::as_u8`; -1 while alive |
| `mother` | int | mother's AgentId |
| `father` | int | father's AgentId |
| `ns` | int | temperament NS, permille |
| `ha` | int | temperament HA, permille |
| `rd` | int | temperament RD, permille |
| `p` | int | temperament P, permille |

### `get_person_chronicle`
`get_person_chronicle(agent_id)`: `collect_person_chronicle(&Chronicle, agent_id) -> ChronicleRows` →
`rows_to_dict(rows, false)` (`ffi/chronicle.rs`). Every entry naming the person, in chronicle order (append order).

| key | type | meaning |
|---|---|---|
| `found` | bool | the person is known |
| `tick` | PackedInt64Array | entry tick |
| `kind` | PackedInt32Array | `ChronicleKind::as_u8` (22 = HutBuilt since M1) |
| `subject` | PackedInt64Array | subject AgentId, -1 none |
| `other` | PackedInt64Array | other AgentId, -1 none |
| `third` | PackedInt64Array | third AgentId, -1 none |
| `settlement` | PackedInt64Array | SettlementId, -1 none |
| `settlement2` | PackedInt64Array | second SettlementId, -1 none |
| `cause` | PackedInt32Array | kind-specific cause code |
| `value` | PackedInt64Array | kind-specific value |

### `get_settlement_chronicle`
`get_settlement_chronicle(settlement_id)`: `collect_settlement_chronicle` → `rows_to_dict(rows, false)`
(`ffi/chronicle.rs`). Every entry naming the settlement; the same columns as `get_person_chronicle`.

| key | type | meaning |
|---|---|---|
| `found` | bool | the settlement is known |
| `tick` | PackedInt64Array | entry tick |
| `kind` | PackedInt32Array | `ChronicleKind::as_u8` |
| `subject` | PackedInt64Array | subject AgentId, -1 none |
| `other` | PackedInt64Array | other AgentId, -1 none |
| `third` | PackedInt64Array | third AgentId, -1 none |
| `settlement` | PackedInt64Array | SettlementId, -1 none |
| `settlement2` | PackedInt64Array | second SettlementId, -1 none |
| `cause` | PackedInt32Array | kind-specific cause code |
| `value` | PackedInt64Array | kind-specific value |

### `get_chronicle_since`
`get_chronicle_since(start, max)`: `collect_chronicle_since(&Chronicle, start, max)` → `rows_to_dict(rows, true)`
(`ffi/chronicle.rs`). Entries from index `start`, at most `max` (clamped to 1–512), the same columns plus paging.

| key | type | meaning |
|---|---|---|
| `found` | bool | the page request is valid |
| `tick` | PackedInt64Array | entry tick |
| `kind` | PackedInt32Array | `ChronicleKind::as_u8` |
| `subject` | PackedInt64Array | subject AgentId, -1 none |
| `other` | PackedInt64Array | other AgentId, -1 none |
| `third` | PackedInt64Array | third AgentId, -1 none |
| `settlement` | PackedInt64Array | SettlementId, -1 none |
| `settlement2` | PackedInt64Array | second SettlementId, -1 none |
| `cause` | PackedInt32Array | kind-specific cause code |
| `value` | PackedInt64Array | kind-specific value |
| `next` | int | the index after the last returned entry (the next `start`) |
| `total` | int | entries in the chronicle |

## Causal "why?"

The three causal getters return an `Array` of dictionaries built by `event_views_to_variant_array` →
`event_view_to_dict` → `event_view_to_owned_dict` (`ffi/causal.rs`), one per `CausalEventView`. `kind`, `id`, `parent`
and `tick` are always present; the other keys only for the event kinds that carry them.

### `get_tile_causal_history`
`get_tile_causal_history(x, y)`: `try_collect_tile_causal_history` (`ffi/causal.rs`); the tile's causal ring, oldest
first (at most `TILE_CAUSAL_RING_SIZE` = 8); empty out of bounds.

| key | type | meaning |
|---|---|---|
| `kind` | String | `building_placed`, `stamp_dirty`, `influence_changed`, `agent_decision`, `memory_recalled`, `combat_started`, `combat_completed`, … |
| `id` | int | event id (monotonic) |
| `parent` | int | parent event id, -1 for a root or an evicted parent |
| `tick` | int | tick recorded |
| `channel` | int | influence channel index (stamp/influence events) |
| `position` | Vector2i | origin / sample / decision tile |
| `radius` | int | influence radius (building placed) |
| `region` | Vector4i | dirty region `(min_x, min_y, max_x, max_y)` (stamp dirty) |
| `old` | float | intensity before propagation (influence changed) |
| `new_value` | float | intensity after propagation; for `combat_completed` the defender's HP after damage |
| `agent_id` | int | deciding / acting AgentId |
| `reason` | String | decision reason (`hunger_threshold_breach`, `thirst_threshold_breach`, …) |
| `triggered_by` | String | memory recall trigger (`cascade_bias`, `similarity_search`, `periodic`, `combat_context`) |
| `recalled_event` | int | event id of the recalled memory |
| `defender_id` | int | defender AgentId (combat events) |

### `get_event_chain`
`get_event_chain(x, y, event_id)`: `try_collect_event_chain` (`ffi/causal.rs`); `[event, parent, grand-parent, …]`
on that tile, ending at a root or an evicted parent; empty out of bounds or for a negative or unknown id. The same
dictionary keys as `get_tile_causal_history`.

| key | type | meaning |
|---|---|---|
| `kind` | String | event kind |
| `id` | int | event id |
| `parent` | int | parent event id, -1 none |
| `tick` | int | tick recorded |
| `channel` | int | influence channel index |
| `position` | Vector2i | event tile |
| `radius` | int | influence radius |
| `region` | Vector4i | dirty region |
| `old` | float | intensity before |
| `new_value` | float | intensity after / HP after |
| `agent_id` | int | AgentId |
| `reason` | String | decision reason |
| `triggered_by` | String | memory recall trigger |
| `recalled_event` | int | recalled event id |
| `defender_id` | int | defender AgentId |

### `get_tile_causal_events`
`get_tile_causal_events(xs, ys, kinds)`: `collect_tile_causal_events` (`ffi/causal.rs`); the events of the given
tiles whose `kind` is in `kinds`, once per id, in input-tile then ring order. GDScript reaches it through
`SimReads.causal_events`. The same dictionary keys as `get_tile_causal_history`.

| key | type | meaning |
|---|---|---|
| `kind` | String | event kind |
| `id` | int | event id |
| `parent` | int | parent event id, -1 none |
| `tick` | int | tick recorded |
| `channel` | int | influence channel index |
| `position` | Vector2i | event tile |
| `radius` | int | influence radius |
| `region` | Vector4i | dirty region |
| `old` | float | intensity before |
| `new_value` | float | intensity after / HP after |
| `agent_id` | int | AgentId |
| `reason` | String | decision reason |
| `triggered_by` | String | memory recall trigger |
| `recalled_event` | int | recalled event id |
| `defender_id` | int | defender AgentId |

## Places and settlements

### `get_building_snapshot`
`collect_building_snapshot(&SimResources) -> Vec<BuildingSnapshotRow>` → `building_rows_to_dict` (`ffi/snapshot.rs`).
Every registered building, sorted by BuildingId. Buildings register on tick 1 (the hearth is absent at tick 0).

| key | type | meaning |
|---|---|---|
| `ids` | PackedInt64Array | BuildingId |
| `xs` | PackedInt32Array | tile x (a hut's **centre** tile; its footprint is 3 × 3 around it) |
| `ys` | PackedInt32Array | tile y |
| `kinds` | PackedByteArray | `BuildingKind::code`: 0 hearth, 1 shelter, 2 stockpile marker (V7), 3 hut |

### `get_construction_snapshot`
`collect_construction_snapshot(&World) -> Vec<ConstructionSnapshotRow>` → `construction_rows_to_dict`
(`ffi/snapshot.rs`). Every `ConstructionSite` entity, in hecs query order.

| key | type | meaning |
|---|---|---|
| `ids` | PackedInt64Array | entity bits of the site |
| `xs` | PackedInt32Array | tile x of the site's `Position` |
| `ys` | PackedInt32Array | tile y |
| `progresses` | PackedInt32Array | work done, in work ticks |
| `required_progresses` | PackedInt32Array | work needed (may be 0: divide by `max(req, 1)`) |

### `get_resource_snapshot`
`collect_resource_snapshot(&SimResources) -> Vec<ResourceSnapshotRow>` → `resource_rows_to_dict`
(`ffi/snapshot.rs`). Source tiles sorted by `(kind, x, y)`.

| key | type | meaning |
|---|---|---|
| `xs` | PackedInt32Array | tile x |
| `ys` | PackedInt32Array | tile y |
| `kinds` | PackedInt32Array | 0 food (bush), 1 water, 2 sleep, 3 deadwood |
| `amounts` | PackedInt32Array | current amount, clamped to `maxes` (a depleted bush stays at 0) |
| `maxes` | PackedInt32Array | source capacity, at least 1 |

### `get_recent_deaths`
`collect_recent_deaths(&SimResources) -> Vec<RecentDeathRow>` → `recent_death_rows_to_dict` (`ffi/snapshot.rs`). The
rolling recent-deaths buffer in buffer order, plus the clock for fading.

| key | type | meaning |
|---|---|---|
| `xs` | PackedInt32Array | death tile x |
| `ys` | PackedInt32Array | death tile y |
| `reasons` | PackedInt32Array | `DeathReason::as_u8` (0 starvation, 1 dehydration, 2 combat, …) |
| `ticks` | PackedInt64Array | tick of death |
| `current_tick` | int | completed ticks (`resources.current_tick`) |

### `get_settlement_snapshot`
`collect_settlement_snapshot(&World, &settlements) -> Vec<SettlementSnapshotRow>` and
`collect_settlement_wood_targets` → `settlement_rows_to_dict` (`ffi/reads_settlement.rs`). One row per settlement
with a resolvable member, sorted by SettlementId.

| key | type | meaning |
|---|---|---|
| `ids` | PackedInt64Array | a stable key per row (carries the SettlementId; settlements are not entities) |
| `settlement_ids` | PackedInt32Array | SettlementId |
| `centroid_xs` | PackedInt32Array | floor of the members' mean tile x |
| `centroid_ys` | PackedInt32Array | floor of the members' mean tile y |
| `member_counts` | PackedInt32Array | resolvable members |
| `formation_xs` | PackedInt32Array | fixed formation tile x |
| `formation_ys` | PackedInt32Array | fixed formation tile y |
| `food_stocks` | PackedInt32Array | stockpile food units |
| `wood_stocks` | PackedInt32Array | stockpile wood units |
| `fire_lit` | PackedByteArray | 1 while the settlement's fire burns |
| `wood_targets` | PackedInt32Array | floor of the firewood target (0 without the firewood rule) |

### `get_society_snapshot`
`collect_society_snapshot(&World, &SimResources)` → `society_snapshot_to_dict` (`ffi/reads_settlement.rs`). Four row
groups of parallel columns, each with its own length: settlements (in `get_settlement_snapshot` order), homes (by
AgentId), founders (by AgentId), fissions (by fission id); plus retired hearths and the clock.

| key | type | meaning |
|---|---|---|
| `settlement_ids` | PackedInt32Array | settlements: SettlementId |
| `parent_ids` | PackedInt32Array | settlements: parent SettlementId, -1 none |
| `founded_ticks` | PackedInt64Array | settlements: founding tick |
| `member_counts` | PackedInt32Array | settlements: members |
| `fission_pressures` | PackedInt64Array | settlements: fission pressure |
| `fission_thresholds` | PackedInt64Array | settlements: fission threshold |
| `home_agent_ids` | PackedInt64Array | homes: AgentId |
| `home_settlement_ids` | PackedInt32Array | homes: SettlementId |
| `founder_agent_ids` | PackedInt64Array | founders: AgentId |
| `founder_fission_ids` | PackedInt32Array | founders: fission id |
| `founder_site_xs` | PackedInt32Array | founders: site tile x |
| `founder_site_ys` | PackedInt32Array | founders: site tile y |
| `founder_leader` | PackedByteArray | founders: 1 for the leader |
| `founder_since_ticks` | PackedInt64Array | founders: tick since founding |
| `fission_ids` | PackedInt32Array | fissions: id |
| `fission_parents` | PackedInt32Array | fissions: parent SettlementId |
| `fission_site_xs` | PackedInt32Array | fissions: site tile x |
| `fission_site_ys` | PackedInt32Array | fissions: site tile y |
| `fission_sizes` | PackedInt32Array | fissions: members |
| `fission_depart_ticks` | PackedInt64Array | fissions: departure tick |
| `fission_founded_ticks` | PackedInt64Array | fissions: founding tick, -1 not yet |
| `fission_cancelled_ticks` | PackedInt64Array | fissions: cancel tick, -1 not cancelled |
| `fission_daughters` | PackedInt32Array | fissions: daughter SettlementId, -1 none |
| `fission_food_taken` | PackedInt64Array | fissions: food units taken |
| `retired_hearth_xs` | PackedInt32Array | retired hearths: tile x |
| `retired_hearth_ys` | PackedInt32Array | retired hearths: tile y |
| `current_tick` | int | completed ticks |

### `get_settlement_relations`
`collect_settlement_relations(&SettlementRelations)` → `relation_rows_to_dict` (`ffi/relations.rs`). Every pair of
living villages, lower id first, in key order. Cached once per tick by `SimReads`.

| key | type | meaning |
|---|---|---|
| `a` | PackedInt64Array | lower SettlementId |
| `b` | PackedInt64Array | higher SettlementId |
| `value` | PackedInt32Array | relation value |
| `estranged` | PackedByteArray | 1 when estranged |
| `base` | PackedInt32Array | the kin base the value drifts toward |

## Colony (M1)

### `get_colony_versions`
`collect_colony_versions(&SimResources) -> ColonyVersions` → `versions_to_dict` (`ffi/colony.rs`). Cheap; poll every
frame.

| key | type | meaning |
|---|---|---|
| `trees_version` | int | `colony.trees_version`, bumped when a tree is felled or added |
| `overlay_version` | int | `colony.overlay_version`, bumped when marks, zones, stacks or sites change |
| `rooms_version` | int | M3: `colony.rooms_version`, bumped when the rooms or their roles change |

### `get_trees`
`collect_trees(&SimResources)` → `trees_to_dict` (`ffi/colony.rs`). Every standing tree in `(x, y)` order (sorted by
x, then y). Stumps are not in the simulation: `tree_layer.gd` (and the text renderer's `StumpTracker`) derive them
from successive reads.

| key | type | meaning |
|---|---|---|
| `version` | int | `trees_version` of this read |
| `xs` | PackedInt32Array | tile x |
| `ys` | PackedInt32Array | tile y |

### `get_colony_overlay`
`collect_colony_overlay(&World, &SimResources) -> ColonyOverlay` → `overlay_to_dict` (`ffi/colony.rs`). Four row
groups, each in its map's key order: designations by tile, zone cells by tile, ground stacks by tile, sites by centre
tile; M2 adds resource nodes by tile.

| key | type | meaning |
|---|---|---|
| `version` | int | `overlay_version` of this read |
| `designation_xs` | PackedInt32Array | marked tile x (a tree to chop or a node to quarry) |
| `designation_ys` | PackedInt32Array | marked tile y |
| `designation_kinds` | PackedInt32Array | M2: the mark, 0 chop, 1 quarry; M3: 2 dig, 3 deconstruct |
| `node_xs` | PackedInt32Array | M2: resource node tile x |
| `node_ys` | PackedInt32Array | M2: resource node tile y |
| `node_kinds` | PackedInt32Array | M2: `NodeKind::code`, 0 boulder, 1 flint outcrop |
| `node_left` | PackedInt32Array | M2: items still in the node |
| `item_qualities` | PackedInt32Array | M2: quality 0–100 of the ground stack (same rows as `item_xs`) |
| `zone_ids` | PackedInt32Array | zone id of the cell |
| `zone_xs` | PackedInt32Array | zone cell tile x |
| `zone_ys` | PackedInt32Array | zone cell tile y |
| `item_xs` | PackedInt32Array | ground stack tile x |
| `item_ys` | PackedInt32Array | ground stack tile y |
| `item_kinds` | PackedInt32Array | `ItemKind::code` (0 log … 6 hide, as `carry_kinds`) |
| `item_counts` | PackedInt32Array | items in the stack |
| `site_xs` | PackedInt32Array | blueprint site footprint top-left x |
| `site_ys` | PackedInt32Array | footprint top-left y |
| `site_ws` | PackedInt32Array | footprint width |
| `site_hs` | PackedInt32Array | footprint height |
| `site_kinds` | PackedInt32Array | `BlueprintKind::code`: 0 hut, 1 knapping spot, 2 wood bench, 3 drying rack; M3: 4 log wall, 5 stone wall, 6 door, 7 packed floor, 8 bedroll |
| `site_needed` | PackedInt32Array | materials needed (all lines summed; M2: a wood bench needs logs and stone) |
| `site_delivered` | PackedInt32Array | materials delivered (all lines summed) |
| `site_progress` | PackedInt32Array | work done |
| `site_required` | PackedInt32Array | work needed |
| `item_covered` | PackedInt32Array | M3: 1 = the ground stack (same rows as `item_xs`) lies under a storehouse roof |

## Built space and rooms (M3)

### `get_space`
`collect_space(&SimResources) -> SpaceView` → `space_to_dict` (`ffi/space.rs`). Every wall, door, floor and dug
tile in `(y, x)` order (a scan of the tile grid), bedrolls in `(y, x)` order; empty in the baseline. Rebuild the
layer when `version` moves.

| key | type | meaning |
|---|---|---|
| `version` | int | `overlay_version` of this read (every built change bumps it) |
| `wall_xs` | PackedInt32Array | wall or door tile x |
| `wall_ys` | PackedInt32Array | wall or door tile y |
| `wall_kinds` | PackedInt32Array | 0 log wall, 1 stone wall, 2 door |
| `floor_xs` | PackedInt32Array | floor tile x |
| `floor_ys` | PackedInt32Array | floor tile y |
| `floor_kinds` | PackedInt32Array | 0 packed earth, 1 dug rock |
| `bed_xs` | PackedInt32Array | bedroll tile x |
| `bed_ys` | PackedInt32Array | bedroll tile y |
| `dug_xs` | PackedInt32Array | dug hill tile x |
| `dug_ys` | PackedInt32Array | dug hill tile y |
| `ford_xs` | PackedInt32Array | ford tile x (M4: a shallow river tile people wade, cost 30) |
| `ford_ys` | PackedInt32Array | ford tile y |

### `get_rooms`
`collect_rooms(&SimResources) -> RoomsView` → `rooms_to_dict` (`ffi/space.rs`). Rooms in id order; every room tile in
`(x, y)` order with its room id. Ids are `RoomId` (stable across re-detection, ADR 0023).

| key | type | meaning |
|---|---|---|
| `version` | int | `rooms_version` of this read |
| `ids` | PackedInt32Array | room id |
| `roles` | PackedInt32Array | `RoomRole::code`: 0 unassigned, 1 bedroom, 2 storehouse, 3 workshop, 4 hall |
| `role_names` | PackedStringArray | `RoomRole::name` |
| `overridden` | PackedByteArray | 1 = the player set the role |
| `tiles` | PackedInt32Array | tiles in the room |
| `beds` | PackedInt32Array | bedrolls in the room |
| `roofed` | PackedByteArray | 1 = roofed |
| `dug` | PackedByteArray | 1 = dug out of a hill |
| `doors` | PackedInt32Array | doors on its border |
| `centre_xs` | PackedInt32Array | label tile x (the room tile nearest its mean) |
| `centre_ys` | PackedInt32Array | label tile y |
| `tile_xs` | PackedInt32Array | room tile x |
| `tile_ys` | PackedInt32Array | room tile y |
| `tile_rooms` | PackedInt32Array | the tile's room id |

### `get_room_detail`
`collect_room_detail(&World, &SimResources, i64) -> RoomDetail` → `room_detail_to_dict` (`ffi/space.rs`). Argument:
the room id. Not in `SimReads.GETTERS` (it takes an argument).

| key | type | meaning |
|---|---|---|
| `found` | bool | false when there is no such room |
| `id` | int | room id, -1 when not found |
| `role` | int | `RoomRole::code`, -1 when not found |
| `role_name` | String | `RoomRole::name` |
| `overridden` | bool | the player set the role |
| `tiles` | int | tiles |
| `beds` | int | bedrolls |
| `roofed` | bool | roofed |
| `dug` | bool | dug out of a hill |
| `doors` | int | doors on its border |
| `items` | int | items lying on its tiles |
| `covered` | int | of them, under a storehouse roof |
| `sleepers` | int | people asleep in it now |

### `get_place_preview`
`collect_place_preview(&World, &SimResources, &str, (i32, i32, i32, i32), i64) -> PlacePreview` → `preview_to_dict`
(`ffi/space.rs`). Arguments: blueprint name, drag `x0, y0, x1, y1`, shape (0 one tile or a building's top-left,
1 line, 2 hollow rectangle, 3 filled rectangle). Rows in the shape's tile order. Read-only.

| key | type | meaning |
|---|---|---|
| `xs` | PackedInt32Array | tile x |
| `ys` | PackedInt32Array | tile y |
| `ok` | PackedByteArray | 1 = the blueprint fits there |
| `reasons` | PackedStringArray | `RejectReason::name` (`person`, `hill`, `built`, `blocked`, `water`, …), `""` when it fits |

## Economy (M2)

### `get_workshops`
`collect_workshops(&SimResources) -> Vec<WorkshopRow>` → `workshops_to_dict` (`ffi/economy.rs`). Workshops in id
order; their bills (work order) and shelf stacks flattened, each with the row index of its workshop.

| key | type | meaning |
|---|---|---|
| `ids` | PackedInt32Array | workshop id |
| `kinds` | PackedInt32Array | `BlueprintKind::code` (1 knapping spot, 2 wood bench, 3 drying rack) |
| `xs` | PackedInt32Array | footprint top-left x |
| `ys` | PackedInt32Array | footprint top-left y |
| `ws` | PackedInt32Array | footprint width |
| `hs` | PackedInt32Array | footprint height |
| `work_xs` | PackedInt32Array | work tile x (south of the footprint's centre column) |
| `work_ys` | PackedInt32Array | work tile y |
| `bill_shops` | PackedInt32Array | row index (in `ids`) of the bill's workshop |
| `bill_ids` | PackedInt32Array | bill id |
| `bill_recipes` | PackedInt32Array | `RecipeKind::code`: 0 knap blade, 1 haft axe, 2 split firewood |
| `bill_modes` | PackedInt32Array | 0 make n, 1 maintain n |
| `bill_counts` | PackedInt32Array | n |
| `bill_done` | PackedInt32Array | crafts done under the bill |
| `bill_suspended` | PackedInt32Array | 1 = suspended |
| `bill_auto` | PackedInt32Array | 1 = the hearth's auto-bill |
| `bill_active` | PackedInt32Array | 1 = the bill the workshop works now |
| `shelf_shops` | PackedInt32Array | row index of the shelf stack's workshop |
| `shelf_kinds` | PackedInt32Array | `ItemKind::code` |
| `shelf_counts` | PackedInt32Array | items on the shelf |

### `get_work_priorities`
`collect_work_priorities(&World) -> Vec<PriorityRow>` → `priorities_to_dict` (`ffi/economy.rs`). Adults (no
`Child`) in `AgentId` order; levels are the person's `WorkPriorities` or the defaults (3; elders' quarrying 0).

| key | type | meaning |
|---|---|---|
| `agent_ids` | PackedInt64Array | `AgentId` |
| `elders` | PackedInt32Array | 1 = an elder |
| `levels` | PackedByteArray | 7 per row in `work_types` order: 0 never, 1 highest … 4 lowest |
| `work_types` | PackedStringArray | `craft, build, quarry, mine, chop, split, haul` (`WorkType::name`, tie-break order; M3 adds `mine`) |

### `get_recipes`
`collect_recipes()` → `recipes_to_dict` (`ffi/economy.rs`). The recipe table (`data/colony/recipes.ron`) in code order.

| key | type | meaning |
|---|---|---|
| `codes` | PackedInt32Array | `RecipeKind::code` |
| `names` | PackedStringArray | `knap_blade`, `haft_axe`, `split_firewood` |
| `workshop_kinds` | PackedInt32Array | `BlueprintKind::code` of the workshop that makes it |
| `output_kinds` | PackedInt32Array | `ItemKind::code` of the output |
| `output_counts` | PackedInt32Array | items per craft |
| `work_ticks` | PackedInt32Array | work ticks at skill level 5 |
| `skills` | PackedInt32Array | `SkillKind::code` (0 knap, 1 woodwork, 2 chop, 3 quarry, 4 build, 5 mining) |

### `get_agent_economy`
`collect_agent_economy(&World, &SimResources, AgentId) -> Option<AgentEconomyRow>` → `agent_economy_to_dict`
(`ffi/economy.rs`). Argument: the person's `AgentId`. Skill arrays are indexed by `SkillKind::code`.

| key | type | meaning |
|---|---|---|
| `found` | bool | false when no person has that id |
| `skill_levels` | PackedInt32Array | level 0–20 per skill |
| `skill_xp` | PackedInt32Array | XP (work ticks) per skill |
| `skill_tiers` | PackedInt32Array | 0 novice (0–4), 1 apprentice (5–9), 2 skilled (10–14), 3 master (15–20) |
| `skill_names` | PackedStringArray | `knap, woodwork, chop, quarry, build, mining` |
| `tool_kind` | int | held tool's `ItemKind::code`, -1 none |
| `tool_quality` | int | tool quality 0–100 |
| `tool_material` | String | tool material name (`flint`, …; `""` unknown) |
| `carry_kind` | int | carried item kind, -1 none |
| `carry_count` | int | items carried |
| `carry_quality` | int | carried items' quality |
| `carry_material` | String | carried items' material name |
| `bed_x` | int | M3: assigned bed tile x, -1 none |
| `bed_y` | int | M3: assigned bed tile y, -1 none |
| `bed_room` | int | M3: the bed's room id, -1 when the bed is not in a room |
| `bed_room_role` | int | M3: that room's `RoomRole::code`, -1 none |
| `bed_roofed` | bool | M3: that room is roofed |

## Commands and lifecycle (not snapshot reads)

The mutators GDScript may call (`SimReads.MUTATORS`): the M1 orders `cmd_designate_chop`, `cmd_designate_chop_tiles`,
`cmd_cancel_area`, `cmd_add_stockpile`, `cmd_remove_zone_area`, `cmd_place_blueprint` (kinds `hut`, `knapping_spot`,
`wood_bench`, `drying_rack`) and the M2 orders `cmd_designate_quarry`, `cmd_add_bill`, `cmd_remove_bill`,
`cmd_set_bill_mode`, `cmd_move_bill`, `cmd_set_bill_suspended`, `cmd_set_work_priority` (`ffi/economy.rs`; names
parsed at the boundary, an unknown name is `unknown` without a log entry), the M3 orders `cmd_place_line(kind, x0,
y0, x1, y1)`, `cmd_place_rect(kind, x0, y0, x1, y1, hollow)` (space kinds `log_wall`, `stone_wall`, `door`,
`packed_floor`, `bedroll`; `cmd_place_blueprint` takes them too, one tile), `cmd_designate_dig`,
`cmd_designate_deconstruct` (rectangles) and `cmd_set_room_role(room, role)` (`bedroom`, `storehouse`,
`workshop`, `hall`, `unassigned` or `auto`; `ffi/space.rs`), each returning
`{accepted: bool, count: int, reason: String}` (`result_to_dict` in `ffi/colony.rs`; `reason` is `""` or a
`RejectReason` name) and logged with the next tick (ADR 0012); `can_place_blueprint(kind, x, y) -> bool`;
`advance_ticks(n) -> int` (the final absolute tick, at most 200,000 per call); `reset_world(seed)`,
`reset_world_stress(seed, agents)`; `on_building_placed(x, y, radius) -> bool`; `set_sim_speed(speed)`; the signal
`world_reset`.

## Text renderer glyphs

`rust/crates/sim-test/src/probe/ascii.rs` draws the colony as one character per tile, rows top to bottom, from the
collectors above only (`collect_terrain_snapshot`, `collect_colony_overlay`, `collect_building_snapshot`,
`collect_trees`, `collect_agent_snapshot`, `collect_agent_jobs`, `collect_time_snapshot`). Run it with
`worldsim_probe --scene colony --seeds S..S+1 --ascii --tick T [--rect x,y,w,h] [--commands FILE] [--ascii-legend]
[--out FILE] [--people-json FILE]`; `--tick T` renders after T completed ticks, applying the script's entries for
ticks below T right before that tick's systems, as the colony probe does. `--rect` crops the full grid and is clipped
to the map.

Layer order, later wins: **terrain < marks/zones < items < buildings/sites < trees < people**.

| layer | glyph | meaning | source |
|---|---|---|---|
| terrain | `.` | Plain (grass) | `terrain` 0 |
| terrain | `,` | Forest floor (the trees are drawn by the tree layer) | `terrain` 1 |
| terrain | `#` | Mountain | `terrain` 2 |
| terrain | `^` | Hill | `terrain` 3 |
| terrain | `~` | River (fresh water) | `terrain` 4 |
| terrain | `_` | Coast | `terrain` 5 |
| terrain | `:` | Desert (bare dirt, sand) | `terrain` 6 |
| terrain | `-` | Tundra | `terrain` 7 |
| terrain | `"` | Swamp | `terrain` 8 |
| terrain | `o` | Cave | `terrain` 9 |
| terrain | `?` | an unknown terrain code | — |
| marks/zones | `=` | stockpile zone cell | `zone_xs/ys` |
| marks/zones | `x` | chop mark (a designation; on a standing tree the tree layer draws it, see below) | `designation_xs/ys` |
| items | `l` | ground log stack of fewer than 9 logs | `item_*`, `item_counts` < 9 |
| items | `L` | ground log stack of 9 or more | `item_counts` ≥ 9 |
| buildings/sites | `b` | blueprint site footprint, not finished | `site_xs/ys/ws/hs` |
| buildings/sites | `*` | hearth | `kinds` 0 |
| buildings/sites | `h` | shelter | `kinds` 1 |
| buildings/sites | `H` | finished hut, its 3 × 3 footprint around the centre tile | `kinds` 3 |
| trees | `t` | stump: a tree felled less than a day (1,440 ticks) ago | successive `get_trees` reads |
| trees | `T` | standing tree | `get_trees` |
| trees | `x` | standing tree with a chop mark | `get_trees` ∩ designations |
| people | `@` | adult (or elder) awake | `life_stages` ≠ 1 |
| people | `a` | child awake | `life_stages` 1 |
| people | `z` | asleep | `states` 3 and `target_kinds` 3 |
| people | `2`–`9` | a tile shared by that many people (`9` = nine or more) | positions |
| people | `+` | the load of an awake carrier (`carry_counts` > 0), on the cell right of it | `carry_counts` |

Rules:

- **No collector, not drawn**: the V7 stockpile marker (building kind 2) is not drawn, as in `building_layer.gd`. Stumps
  have no collector; the renderer keeps them as `tree_layer.gd` does (a tree missing from the next `get_trees` read
  leaves a stump at that tick for 1,440 ticks; the first read records none), so `render(engine, rect)` alone shows
  none and the probe observes the trees after every tick. Berry bushes, deadwood and water sources
  (`get_resource_snapshot`) have no glyph in this version.
- **Shared tiles**: the simulation puts every sleeper of a shelter or hut on its tile (the Godot view fans them out
  for drawing only), so a tile with k ≥ 2 people shows the digit k instead of a person glyph; the people a grid
  shows (1 per `@`/`a`/`z`, k per digit) equal the agents alive.
- **Carrying**: the `+` goes on the cell at `x + 1` of an awake carrier (sleepers drop the carry frame, as the agent
  layer does). It is not written when that cell holds a person or lies past the map's right edge; otherwise it covers
  whatever lies there. `--rect` crops the full-map grid, so a carrier on the rect's last column has its `+` outside
  the rect and one just left of the rect shows its `+` in the first column.
- **Agreement with the Godot view** (gate step `renderer_agreement`, `tools/v8/renderer_agreement.sh`): the
  multiset of person tiles of `people_tiles` equals the tiles of the people `scripts/world/colony/agent_layer.gd`
  draws, converted from its person nodes' drawn positions minus the sleeper spread.
