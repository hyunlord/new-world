# Snapshot schema — the bridge's read surface

Schema version: 13 — M6a adds plant, meal and physical-food storage reads plus harvest designation. Version 12 — M-CORE adds the read-only `get_agent_mind` choice and reason view. Version 11 — M-UX2 adds independent defence state/command and the optional native timing diagnostic. Version 10 — M-UX Stage B adds pressure/animal catalogue metadata, actual pressure banner identity and species presentation columns. Schema 9 introduced the immutable core catalogue, factory preflight, full-width content columns and mechanical door flags. Catalogue DTO version remains 1; it is independent of this document version.

History: 8 (M-UX adds `get_ui_facts`, `get_steward`, and the logged `cmd_set_steward` mode command. 7 — M-ART2: the snapshot's appearance columns `sexes`, `look_ids`, `variants` from `collect_appearance` (`ffi/appearance.rs`, rule file `rust/crates/sim-bridge/data/appearance.ron`) and `get_space`'s `wall_materials`, `wall_styles`, `wall_categories`, `floor_materials`, `floor_styles`, `floor_categories`; no getter added or removed. 6 — M5.5: the neighbour reads `get_neighbours`, `get_wave`, `get_caravan`, the snapshot's `fleeing`, `get_agent_m5`'s `fleeing`, `get_year_summary`'s `migrants_joined`, `trades`, `eurekas` and chronicle kinds 38–44; M5: the story, animal and mood reads `get_story`, `get_animals`, `get_agent_m5`, `get_drafted`, `get_chronicle_why`, `get_year_summary`, `get_teach_suggestion`, the snapshot's `tops`, `mood_bands`, `drafted`, `sick` and the chronicle rows' `index` and links; M4 the knowledge reads `get_tech`, `get_tech_node`, `get_agent_knowledge`, `get_teach_orders`, `get_knowledge_waits`, `get_space`'s fords and the new item, recipe, blueprint, job and chronicle codes; M3 added built space, rooms, the place preview and the overlay's `item_covered`; M2 the economy reads and the overlay's node, mark and quality columns).

Every `#[func] fn get_*` that `WorldSimNode` (`rust/crates/sim-bridge/src/ffi/node.rs`) exposes to GDScript, with the
keys it returns, their types, id spaces, units and row order, and the pure collector that backs it. A `#[func]` body
only calls a collector (`sim_bridge::ffi::collect_*`, callable headless from sim-test) and a `*_to_dict` marshaller;
two views built on the same collectors must agree (ADR 0016, clause C2). The second view is the text renderer
(`rust/crates/sim-test/src/probe/ascii.rs`, `worldsim_probe --scene colony --ascii`, section "Text renderer glyphs").

Checked by `python3 tools/v8/check_docs_helpers.py schema` (`tools/v8/check_docs.sh` check 10): every `get_*`
method found by `tools/v8/bridge_api.py` has a `### get_x` section here, and the first table column of that section
names exactly the keys the getter sets (its own `dict.set("k", …)` calls plus those of the `*_to_dict` /
`*_to_variant_array` builders it calls). An absent or extra getter or key fails. The extractor unions literal keys from followed builders, including nested dictionaries; it does not prove nesting, runtime presence, value types, or keys loaded from JSON. Tables below distinguish their actual scopes; nested coverage requires separate validation.

## Conventions

- **Types** are the Godot types GDScript receives: `int` (64-bit; Rust `i64`/`i32`/`u8` scalars), `float` (64-bit),
  `bool`, `String`, `Vector2i`, `Vector4i`, and packed columns `PackedInt32Array`, `PackedInt64Array`,
  `PackedByteArray` (0–255), `PackedFloat32Array`. Feature-detect every key (`snap.has(k)`) and never write to a
  returned array (`duplicate()` first; packed arrays are shared with the `SimReads` cache).
- **Content codes (schema 10)**: item, recipe, blueprint, pressure and animal codes span u32 and use Godot int / PackedInt64Array. Existing numeric examples below describe bundled historical content, not closed enumerations. Mixed Chronicle cause/link codes also use PackedInt64Array. Material bit-pattern columns retain PackedInt32Array. Nested `get_ui_facts` tables inherit their referenced getter column types.
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

## Immutable content and construction preflight (M-UX Stage B)

`WorldSimNode.bootstrap_catalog() -> Dictionary` is a static, engine-free preflight returning `ok: bool`,
`error: String`, and on success `registry_revision: String`. `WorldSimNode.create() -> WorldSimNode | null`
is the static construction route: preflight failure returns null without constructing a fallback world.
The host freezes `WORLDSIM_CONTENT_PACK` once, validates selected presentation and core data, and installs
that selection before engine construction. Empty/invalid explicit selection fails; absent selection uses bundled data.

### `get_content_catalog`
`WorldSimNode.get_content_catalog() -> Dictionary`, backed by `content::catalog_to_dict(&Catalog)`.
Each call returns a fresh projection of installed definitions; an unavailable catalogue yields an empty dictionary.
Identity codes are unsigned u32 represented losslessly as Godot int, not array positions. Items, blueprints and
recipes use resolved registry definitions; knowledge metadata comes from the selected knowledge definitions.
Material metadata comes from the bundled `assets/ui/content_presentation.json` materials array, retaining its
separate historical raw-id namespace. Pressure and animal definitions are projected from the same selected catalogue.

Top-level fields:

| key | type | meaning |
|---|---|---|
| `schema_version` | int | catalogue DTO version 1 |
| `registry_revision` | String | selected catalogue revision fingerprint, independent of staging path |
| `items` | Array[Dictionary] | item definitions |
| `item_roles` | Dictionary | exported semantic item role mapping |
| `blueprints` | Array[Dictionary] | blueprint definitions |
| `recipes` | Array[Dictionary] | recipe definitions |
| `pressures` | Array[Dictionary] | pressure definitions |
| `animals` | Array[Dictionary] | animal definitions |
| `plants` | Array[Dictionary] | plant identity and harvest-output metadata |
| `knowledge` | Array[Dictionary] | knowledge node metadata |
| `materials` | Array[Dictionary] | bundled material presentation metadata; JSON rows contain id, name_key, category, raw_id |

Nested fields (the literal-key checker flattens these into the same union; they are **not** top-level fields):

| key | type | scope and meaning |
|---|---|---|
| `code` | int | item/blueprint/recipe identity code; knowledge numeric node id |
| `id` | String | canonical identity name in item/blueprint/recipe/knowledge rows |
| `name_key` | String | localization key in those rows |
| `category` | String | registered presentation category |
| `mechanic` | String | pressure cold/drought/pack/illness or animal grazer/predator behavior |
| `yield_item` | int | plant harvest output item code |
| `huntable` | bool | animal definition permits hunting |
| `hp` | int | animal definition default HP; get_animals.max_hps resolves per-world overrides |
| `material_format_key` | String | item material-aware localization format key |
| `tags` | PackedStringArray | item behavior tags |
| `tree_output` | int | item_roles entry, when resolved; item code |
| `width` | int | blueprint footprint width |
| `height` | int | blueprint footprint height |
| `placeable` | bool | blueprint placement availability |
| `menu_order` | int | blueprint menu order; -1 absent |
| `shortcut_slot` | int | blueprint shortcut slot; -1 absent |
| `construction_style` | String | blueprint construction style id |
| `placement_category` | String | blueprint: structure, wall, floor, furnishing |
| `completion_category` | String | blueprint: dwelling, workshop, wall, door, floor, bed |
| `workshop_code` | int | recipe's primary workshop blueprint code |
| `compatible_workshops` | PackedInt64Array | recipe's accepted workshop blueprint codes |
| `output_code` | int | recipe output item code |
| `output_count` | int | recipe output units |
| `work_ticks` | int | recipe authored work ticks |
| `skill_code` | int | recipe mechanical skill code |
| `lane` | String | knowledge node lane name |

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
`collect_agent_infancy_and_pregnancy`, `collect_agent_colds` (`ffi/snapshot.rs`), `collect_agent_jobs`
(`ffi/colony.rs`), `collect_agent_m5_columns` (`ffi/story.rs`), `collect_fleeing_column` (`ffi/neighbours.rs`) and
`collect_appearance` (`ffi/appearance.rs`, M-ART2) and `collect_kin_columns` (`ffi/family.rs`, M-ART2.1). One row per entity with `Agent` and `Position`, in hecs query order (archetype iteration: stable
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
| `carried_foods` | PackedInt32Array | baseline: `Inventory` food units; colony: fresh raw-edible physical carried stack units (pieces × food units) |
| `carried_woods` | PackedInt32Array | `Inventory` wood units |
| `life_stages` | PackedByteArray | 0 adult, 1 child, 2 elder |
| `ages_days` | PackedInt32Array | whole days of age; -1 without `Life` |
| `ages_years` | PackedInt32Array | M1.5: whole years (`age_ticks / ticks_per_year`); -1 without `Life` |
| `infants` | PackedByteArray | M1.5: 1 for an infant (`Child::is_infant` under `infant_ticks`), else 0 |
| `pregnant_days` | PackedInt32Array | M1.5: whole days left of a pregnancy (`Pregnant::days_left`); -1 when not pregnant |
| `colds` | PackedByteArray | 1 with the `Cold` marker |
| `jobs` | PackedByteArray | `JobKind::code`: 0 none, 1 chop, 2 haul, 3 deliver, 4 build, 5 quarry, 6 supply, 7 craft, 8 gather (M2) |
| `carry_kinds` | PackedInt64Array | carried item kind, -1 none, else `ItemKind::code`: 0 log, 1 firewood, 2 stone, 3 flint, 4 flint blade, 5 stone axe, 6 hide (M2); 7 clay, 8 raw pot, 9 clay pot, 10 ground axe (M4); 11 meat, 12 cured hide, 13 hide wrap, 14 fur cloak, 15 spear, 16 wolf hide (M5) |
| `carry_counts` | PackedInt32Array | M1: carried item count |
| `tops` | PackedByteArray | M5: the worn top, `Top::code`: 0 loincloth (no `Apparel`), 1 hide wrap, 2 fur cloak |
| `mood_bands` | PackedByteArray | M5: 0 no mood (a child under 10), 1 우울 (below 250), 2 보통, 3 흡족 (750 or more), 4 in a breakdown |
| `drafted` | PackedByteArray | M5: 1 when drafted |
| `sick` | PackedByteArray | M5: 1 when sick |
| `fleeing` | PackedByteArray | M5.5: 1 when running from a wolf (`Fleeing`) |
| `sexes` | PackedByteArray | M-ART2: 0 male (also a person without a `Sex` component), 1 female (`Sex::female`) |
| `look_ids` | PackedInt32Array | M-ART2: look index `((age × 2 + sex) × 3 + top) × 3 + hair` into the renderer's 36 logical looks (nested order age child/adult, sex M/F, top loincloth/hide wrap/fur cloak, hair short/long tied/bun); age from `life_stages` (child → 0, adult and elder → 1), top from `tops`, hair `(((agent_id × 2246822519) mod 2^32) >> 16) mod 3` |
| `variants` | PackedByteArray | M-ART2: face variant 0–5, `h mod 6` with `h = (agent_id × 2654435761) mod 2^32`; an elder (`life_stages` 2) takes `[3, 4][h mod 2]` |
| `mothers` | PackedInt64Array | M-ART2.1: recorded mother's AgentId (`Kin.mother`, kept after her death), -1 unknown or without `Kin` |
| `fathers` | PackedInt64Array | M-ART2.1: recorded father's AgentId (`Kin.father`), -1 unknown or without `Kin` |
| `partners` | PackedInt64Array | M-ART2.1: recorded partner's AgentId (`Kin.partner`), -1 none; the renderer's LOD household rings read these three (`collect_kin_columns`, `ffi/family.rs`) |

A person sleeps when `states` is 3 and `target_kinds` is 3 (the agent layer's sleep frame; the text renderer's `z`).

Appearance (M-ART2, MART2_GOAL 3.2): hair and face variant are pure functions of the AgentId, not simulation state
(nothing saved; the same person keeps them across save/load). Every list and constant above is data in
`rust/crates/sim-bridge/data/appearance.ron` (schema 1, read strictly; a malformed file sends look 0 and variant 0 for
everyone, `sexes` stays right). The text renderer's `--people-json` prints the same values (`appearance`, below).

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

### `get_agent_mind`
`get_agent_mind(entity_bits)`: `collect_agent_mind(&SimEngine, entity_bits)` →
`agent_mind_to_dict` (`ffi/mind.rs`). Reads the latest retained choice without evaluating
policy or mutating simulation state. Missing people and worlds without mind history return
`found=false`; a corrupt dictionary binding also sets `invalid=true`. Other keys exist only
when found. Unsigned IDs use their exact 64-bit signed wire representation; absent causal
references use -1. The history retains at most 16 records per person.

| Key | Type | Meaning |
|---|---|---|
| `found` | bool | a retained choice is available |
| `invalid` | bool | dictionary decoding failed |
| `agent_id` | int | stable AgentId, not entity bits |
| `event_id` | int | causal event identity |
| `parent` | int | actual cause event identity, or -1 |
| `tick` | int | choice simulation tick |
| `node` | String | authored node identity |
| `name_key` | String | node localization key |
| `behaviour` | String | registered lifecycle identity |
| `band` | String | authored priority band |
| `score` | int | exact signed integer score |
| `history_count` | int | retained records for this person, 1–16 |
| `top3` | Array[Dictionary] | descending candidates, data order breaks ties; each has node/name_key/behaviour/score |
| `facts` | Array[Dictionary] | all selected-node inputs ranked by positive score contribution descending, then positive has/>/>= conditions, then remaining conditions; ties use authored order. Rejected-only witnesses remain in the retained causal record |
| `name` | String | registered fact identity in a facts row |
| `value` | int | exact factual value in provider units |
| `cause` | int | fact's actual causal event, or -1 |
| `reason_kind` | String | facts row: score/present/absent/high/low/equal; derived from selected-node metadata, never provider-name heuristics |
| `score_contribution` | int | facts row: checked sum of its signed term contributions |
| `predicates` | Array[Dictionary] | facts row: all authored conditions, preserving order and duplicates |
| `op` | String | predicate row: <, <=, >, >=, ==, has, not |
| `threshold` | int or null | predicate row: exact provider-unit comparison value; null for has/not |
| `score_terms` | Array[Dictionary] | facts row: all authored score terms for this input, retaining duplicates |
| `weight_permille` | int | score-term row: exact signed authored weight |
| `contribution` | int | score-term row: i128 value*weight/1000 truncated toward zero, checked into i64 |

The immutable per-world dictionary retains predicate operators/thresholds, score terms and
constant, and validates their exact equality against the compiled policy on load. The getter
never recompiles policy. Positive score contributions rank above positive conditions; a low
hunger guard therefore does not outrank a positive job condition. Negative conditions retain
their direction. All inputs remain visible; ranking changes presentation only. Overflow or
missing selected-input metadata returns `invalid=true` instead of inventing an explanation.


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
`food_chronicle_rows_to_dict(rows, false, res)` (`ffi/food/chronicle.rs`). Every entry naming the person, in chronicle order (append order).

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
| `cause` | PackedInt64Array | kind-specific cause code |
| `value` | PackedInt64Array | kind-specific value |
| `index` | PackedInt64Array | M5: the entry's chronicle index (the key of `get_chronicle_why`) |
| `link` | PackedInt32Array | M5: the entry's "→" link: -1 none, 0 another entry, 1 a mood modifier, 2 a pressure |
| `link_entry` | PackedInt64Array | M5: the chronicle index the link points at, -1 none |
| `link_code` | PackedInt64Array | M5: the link's `Modifier::code` (100 = the relation) or `PressureKind::code`, -1 none |

M6a historical food metadata (same number of rows as the corresponding index/link column):

| key | type | meaning |
|---|---|---|
| `food_daily_milli` | PackedInt64Array | historical winter-plan daily demand in thousandths; -1 absent, 0 zero demand; aligned with `index` |
| `food_evidence` | Array[Dictionary] | immutable food operation receipt aligned with `index`; empty dictionary when absent or on non-entry Why links; see M6a immutable food operation evidence below |


### `get_settlement_chronicle`
`get_settlement_chronicle(settlement_id)`: `collect_settlement_chronicle` → `food_chronicle_rows_to_dict(rows, false, res)`
(`ffi/food/chronicle.rs`). Every entry naming the settlement; the same columns as `get_person_chronicle`.

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
| `cause` | PackedInt64Array | kind-specific cause code |
| `value` | PackedInt64Array | kind-specific value |
| `index` | PackedInt64Array | M5: the entry's chronicle index (the key of `get_chronicle_why`) |
| `link` | PackedInt32Array | M5: the entry's "→" link: -1 none, 0 another entry, 1 a mood modifier, 2 a pressure |
| `link_entry` | PackedInt64Array | M5: the chronicle index the link points at, -1 none |
| `link_code` | PackedInt64Array | M5: the link's `Modifier::code` (100 = the relation) or `PressureKind::code`, -1 none |

M6a historical food metadata (same number of rows as the corresponding index/link column):

| key | type | meaning |
|---|---|---|
| `food_daily_milli` | PackedInt64Array | historical winter-plan daily demand in thousandths; -1 absent, 0 zero demand; aligned with `index` |
| `food_evidence` | Array[Dictionary] | immutable food operation receipt aligned with `index`; empty dictionary when absent or on non-entry Why links; see M6a immutable food operation evidence below |


### `get_chronicle_since`
`get_chronicle_since(start, max)`: `collect_chronicle_since(&Chronicle, start, max)` → `food_chronicle_rows_to_dict(rows, true, res)`
(`ffi/food/chronicle.rs`). Entries from index `start`, at most `max` (clamped to 1–512), the same columns plus paging.

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
| `cause` | PackedInt64Array | kind-specific cause code |
| `value` | PackedInt64Array | kind-specific value |
| `index` | PackedInt64Array | M5: the entry's chronicle index (the key of `get_chronicle_why`) |
| `link` | PackedInt32Array | M5: the entry's "→" link: -1 none, 0 another entry, 1 a mood modifier, 2 a pressure |
| `link_entry` | PackedInt64Array | M5: the chronicle index the link points at, -1 none |
| `link_code` | PackedInt64Array | M5: the link's `Modifier::code` (100 = the relation) or `PressureKind::code`, -1 none |
| `next` | int | the index after the last returned entry (the next `start`) |
| `total` | int | entries in the chronicle |
| `food_daily_milli` | PackedInt64Array | historical winter-plan daily demand in thousandths; -1 absent, 0 zero demand; aligned with `index` |
| `food_evidence` | Array[Dictionary] | immutable food operation receipt aligned with `index`; empty dictionary when absent or on non-entry Why links; see M6a immutable food operation evidence below |

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
| `food_stocks` | PackedInt32Array | baseline: stockpile food units; colony: fresh physically stored edible units assigned to the nearest reachable settlement, excluding carried and loose ground food |
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
`collect_tree_snapshot(&SimResources)` → `trees_to_dict` (`ffi/colony.rs`). Every standing tree in `(x, y)` order (sorted by
x, then y). Stumps are not in the simulation: `tree_layer.gd` (and the text renderer's `StumpTracker`) derive them
from successive reads. The visual stump retains the removed tree’s last authoritative species. Coordinate-only ASCII clients retain `collect_trees`; Godot species comes from persisted ContentState and saved material names, never a coordinate hash.

| key | type | meaning |
|---|---|---|
| `version` | int | `trees_version` of this read |
| `xs` | PackedInt32Array | tile x |
| `ys` | PackedInt32Array | tile y |
| `species` | PackedStringArray | canonical saved wood-material name, aligned with xs/ys; empty exposes invalid identity |

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
| `item_kinds` | PackedInt64Array | `ItemKind::code` (0 log … 16 wolf hide, as `carry_kinds`) |
| `item_counts` | PackedInt32Array | items in the stack |
| `site_xs` | PackedInt32Array | blueprint site footprint top-left x |
| `site_ys` | PackedInt32Array | footprint top-left y |
| `site_ws` | PackedInt32Array | footprint width |
| `site_hs` | PackedInt32Array | footprint height |
| `site_kinds` | PackedInt64Array | `BlueprintKind::code`: 0 hut, 1 knapping spot, 2 wood bench, 3 drying rack; M3: 4 log wall, 5 stone wall, 6 door, 7 packed floor, 8 bedroll |
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
| `wall_kinds` | PackedInt32Array | legacy projection: 0 log wall, 1 stone wall, 2 door; not the authoritative content identity |
| `wall_is_door` | PackedByteArray | mechanical opening flag, 1 iff the completed blueprint has Door completion category; aligned with wall rows, independent of style name |
| `wall_materials` | PackedInt32Array | M-ART2: the tile's `MaterialId` (`TileGrid::wall_material`, a djb2 hash, not an index) as its u32 bit pattern in an i32 (read `v & 0xFFFFFFFF`); `-1` none (no known material hashes to `0xFFFFFFFF`) |
| `wall_styles` | PackedStringArray | M-UX Stage B: authoritative saved construction style id, falling back to the completed blueprint definition; bundled examples `log`, `stone`, `slab_door`. M-ART2 historically used PascalCase blueprint names |
| `wall_categories` | PackedStringArray | M-ART2: the material's registry category: `Stone`, `Wood`, `Animal`, `Mineral`, `Plant`, `Mod` (any `Mod(u8)`); `""` when unknown (live `material_registry`, else the sim-core material files embedded in the bridge, `ffi/material_category.rs`) |
| `floor_xs` | PackedInt32Array | floor tile x |
| `floor_ys` | PackedInt32Array | floor tile y |
| `floor_kinds` | PackedInt32Array | 0 packed earth, 1 dug rock |
| `floor_materials` | PackedInt32Array | M-ART2: `TileGrid::floor_material` as in `wall_materials` (`packed_earth` for a packed floor, the rock for a dug tile); `-1` none |
| `floor_styles` | PackedStringArray | M-UX Stage B: authoritative saved floor construction style id, falling back to the completed blueprint definition; dug tiles retain `DugFloor` (no blueprint). M-ART2 historically used `PackedFloor` |
| `floor_categories` | PackedStringArray | M-ART2: as `wall_categories` (`""` for `packed_earth`, which has no material file) |
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
| `kinds` | PackedInt64Array | `BlueprintKind::code` (1 knapping spot, 2 wood bench, 3 drying rack) |
| `xs` | PackedInt32Array | footprint top-left x |
| `ys` | PackedInt32Array | footprint top-left y |
| `ws` | PackedInt32Array | footprint width |
| `hs` | PackedInt32Array | footprint height |
| `work_xs` | PackedInt32Array | work tile x (south of the footprint's centre column) |
| `work_ys` | PackedInt32Array | work tile y |
| `bill_shops` | PackedInt32Array | row index (in `ids`) of the bill's workshop |
| `bill_ids` | PackedInt32Array | bill id |
| `bill_recipes` | PackedInt64Array | `RecipeKind::code`: 0 knap blade, 1 haft axe, 2 split firewood |
| `bill_modes` | PackedInt32Array | 0 make n, 1 maintain n |
| `bill_counts` | PackedInt32Array | n |
| `bill_done` | PackedInt32Array | crafts done under the bill |
| `bill_suspended` | PackedInt32Array | 1 = suspended |
| `bill_auto` | PackedInt32Array | 1 = the hearth's auto-bill |
| `bill_active` | PackedInt32Array | 1 = the bill the workshop works now |
| `shelf_shops` | PackedInt32Array | row index of the shelf stack's workshop |
| `shelf_kinds` | PackedInt64Array | `ItemKind::code` |
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
| `codes` | PackedInt64Array | `RecipeKind::code` |
| `names` | PackedStringArray | `knap_blade`, `haft_axe`, `split_firewood` |
| `workshop_kinds` | PackedInt64Array | `BlueprintKind::code` of the workshop that makes it |
| `output_kinds` | PackedInt64Array | `ItemKind::code` of the output |
| `output_counts` | PackedInt32Array | items per craft |
| `work_ticks` | PackedInt64Array | work ticks at skill level 5 |
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

## Knowledge (M4)

Every knowledge read returns its empty form (version 0, empty columns, `found` false) in the baseline world and under
colony rules without `ColonyRule::knowledge`. Node ids are `KnowledgeId` (the index in
`rust/crates/sim-core/data/colony/knowledge.ron`); people are `AgentId`.

### `get_tech`
`collect_tech(&World, &SimResources) -> TechView` → `tech_to_dict` (`ffi/knowledge.rs`). Nodes in id order.

| key | type | meaning |
|---|---|---|
| `version` | int | `TechAggregate::version` (bumped when a state or the progress changes) |
| `progress` | int | era progress ‰ (Σ weight · coverage / Σ weight, absent markers included; ceiling 643) |
| `era` | int | era label: 0 구석기 (< 250), 1 신석기 전환기 (250–749), 2 신석기 시대 (≥ 750) |
| `adults` | int | adults counted at the last aggregate |
| `at_risk` | int | nodes with exactly one holder (the HUD badge) |
| `ids` | PackedInt64Array | node id |
| `names` | PackedStringArray | stable node name (`knowledge.ron` id) |
| `name_keys` | PackedStringArray | locale key of the node's name |
| `lanes` | PackedInt64Array | `Lane::code`: 0 survival, 1 craft, 2 build, 3 mine, 4 pottery, 5 social |
| `eras` | PackedInt64Array | `Era::code`: 0 paleolithic, 1 neolithic |
| `thresholds` | PackedInt64Array | practitioner threshold ‰ |
| `statuses` | PackedInt64Array | `TechStatus::code`: 0 unknown, 1 at risk, 2 known, 3 stable, 4 lost |
| `practitioners` | PackedInt64Array | adults at threshold who used it (or gained it) in the last 6 days |
| `holders` | PackedInt64Array | people at threshold |
| `marker_weights` | PackedInt64Array | era-marker weight (0 = not a marker) |
| `first_names` | PackedInt64Array | `AgentId` of the first named practitioner (or holder; the one holder of an at-risk node), -1 when none |
| `conditions_met` | PackedInt64Array | 1 = every discovery condition holds now |
| `prereq_from` | PackedInt64Array | prerequisite edge: the prerequisite node |
| `prereq_to` | PackedInt64Array | prerequisite edge: the node that needs it |

### `get_tech_node`
`collect_tech_node(&World, &SimResources, i64) -> TechNodeView` → `tech_node_to_dict` (`ffi/knowledge.rs`). Argument:
the node id. Not in `SimReads.GETTERS` (it takes an argument). People in `AgentId` order.

| key | type | meaning |
|---|---|---|
| `found` | bool | false for an unknown node or without knowledge |
| `id` | int | node id |
| `name` | String | stable node name |
| `name_key` | String | locale key of its name |
| `status` | int | `TechStatus::code` |
| `threshold` | int | threshold ‰ |
| `practitioners` | int | practitioners (as `get_tech`) |
| `holders` | int | holders |
| `lane` | int | `Lane::code` |
| `era` | int | `Era::code` |
| `marker_weight` | int | era-marker weight |
| `prereqs` | PackedInt64Array | prerequisite node ids |
| `holder_ids` | PackedInt64Array | `AgentId` of each holder |
| `holder_proficiency` | PackedInt64Array | the holder's proficiency ‰ |
| `holder_instruction` | PackedInt64Array | 1 = the holder knows instruction (teaches at full fidelity) |
| `learner_ids` | PackedInt64Array | `AgentId` of each person who could learn it (adults and children of 10+ years below the threshold) |
| `learner_proficiency` | PackedInt64Array | their proficiency ‰ (0 = none) |
| `cond_kinds` | PackedInt64Array | discovery condition: 0 stockpile, 1 skill, 2 knows, 3 clay bank near |
| `cond_names` | PackedStringArray | item, skill or node name (empty for a clay bank) |
| `cond_amounts` | PackedInt64Array | items, skill level, ‰, or tiles |
| `cond_met` | PackedInt64Array | 1 = the condition holds now |
| `gate_kinds` | PackedInt64Array | what it gates: 0 recipe, 1 blueprint, 2 workshop, 3 designation, 4 channel, 5 lore |
| `gate_names` | PackedStringArray | recipe, blueprint, designation, channel or lore name |

### `get_agent_knowledge`
`collect_agent_knowledge(&World, &SimResources, i64) -> AgentKnowledgeView` → `agent_knowledge_to_dict`
(`ffi/knowledge.rs`). Argument: the person's `AgentId`. Not in `SimReads.GETTERS`. Entries in node id order.

| key | type | meaning |
|---|---|---|
| `found` | bool | false when there is no such person |
| `ids` | PackedInt64Array | node id |
| `proficiency` | PackedInt64Array | proficiency ‰ |
| `thresholds` | PackedInt64Array | the node's threshold ‰ |
| `sources` | PackedInt64Array | `Source::code`: 0 seeded, 1 discovered, 2 oral, 3 observed, 4 apprenticed |
| `last_used` | PackedInt64Array | tick of the start of the day of the last use |
| `days_unused` | PackedInt64Array | whole days since the last use |
| `states` | PackedInt64Array | 0 숙련 (at threshold), 1 배우는 중 (below, used lately), 2 잊어 감 (unused past 6 days) |
| `lesson_role` | int | the person's lesson: 0 teaching, 1 learning, -1 none |
| `lesson_order` | int | its order id, -1 none |
| `lesson_partner` | int | the partner's `AgentId`, -1 none |
| `lesson_node` | int | the node taught, -1 none |

### `get_teach_orders`
`collect_teach_orders(&World, &SimResources) -> Vec<TeachRow>` → `teach_orders_to_dict` (`ffi/knowledge.rs`). Open
apprenticeship orders in id order.

| key | type | meaning |
|---|---|---|
| `ids` | PackedInt64Array | order id |
| `teachers` | PackedInt64Array | teacher `AgentId` |
| `learners` | PackedInt64Array | learner `AgentId` |
| `knowledge` | PackedInt64Array | node id |
| `sessions` | PackedInt64Array | sessions held |
| `proficiency` | PackedInt64Array | the learner's proficiency ‰ now |
| `thresholds` | PackedInt64Array | the node's threshold ‰ (the order ends there) |
| `in_session` | PackedInt64Array | 1 = a lesson is under way |

### `get_knowledge_waits`
`collect_knowledge_waits(&World, &SimResources) -> Vec<WaitRow>` → `waits_to_dict` (`ffi/knowledge.rs`). Every bill
with work left, building site and work mark whose gate no living adult practises: bills by workshop id and list order,
then sites by centre `(x, y)`, then marks by tile.

| key | type | meaning |
|---|---|---|
| `kinds` | PackedInt64Array | 0 a bill, 1 a site, 2 a mark |
| `a` | PackedInt64Array | bill: workshop id; site or mark: tile x |
| `b` | PackedInt64Array | bill: bill id; site or mark: tile y |
| `nodes` | PackedInt64Array | the node no living adult practises ("기다리는 중: … 아는 사람이 없음") |

## Story, animals and mood (M5)

The M5 reads (ADR 0027–0030; `ffi/story.rs`, the teach suggestion in `ffi/knowledge.rs`). Ids: people are
`AgentId`, animals `AnimalId`, chronicle entries their index. Outside the M5 colony each returns its empty form
(`get_story().on` false, no animals, every agent's M5 column 0). Not in `SimReads.GETTERS` except `get_story` and
`get_animals`, which the HUD and the animal layer poll.

### `get_story`
`collect_story(&World, &SimResources) -> StoryView` → `story_to_dict`. Pressures in force in begin order (parallel
columns) and the banner's numbers.

| key | type | meaning |
|---|---|---|
| `on` | bool | the world has a storyteller |
| `kinds` | PackedInt64Array | `PressureKind::code`: 0 cold snap, 1 drought, 2 beasts, 3 illness |
| `names` | PackedStringArray | registered pressure names; bundled examples cold_snap/drought/beasts/illness |
| `name_keys` | PackedStringArray | registered localization keys |
| `categories` | PackedStringArray | registered presentation categories |
| `mechanics` | PackedStringArray | cold/drought/pack/illness behavior categories |
| `start_days` | PackedInt64Array | display day it began |
| `end_days` | PackedInt64Array | display day it ends (exclusive; beasts end earlier when the pack is gone) |
| `days_of` | PackedInt64Array | which day of it today is (1 = its first day: "한파 2일째") |
| `intensities` | PackedInt64Array | wolves in the pack; extra days of a snap or drought |
| `subjects` | PackedInt64Array | the person it struck (illness), -1 none |
| `deaths` | PackedInt64Array | deaths while it has been in force |
| `ended` | int | pressures ended so far |
| `firewood` | int | firewood the hearth can burn now: settlement wood + feedable firewood ("장작 12") |
| `firewood_day_need` | int | one night's burn at a cold snap's rate: burn hours × the night's rate × hearths × `snap_burn_permille` ("하루 18 필요") |
| `wolves` | int | wolves on the map |
| `pack_size` | int | the present pack's size, 0 when no pack came |
| `pack_kind` | int | actual pack pressure identity, default role for independent packs, -1 unavailable |
| `wolves_x` | int | the wolves' mean tile x, -1 without wolves (the banner's edge pulse) |
| `wolves_y` | int | the wolves' mean tile y, -1 without wolves |
| `sick` | int | people sick now |
| `dress_order` | bool | the standing order "모두 가죽 옷" is on |
| `version` | int | pressures begun + ended (changes when the banner should) |

### `get_animals`
`collect_animals(&SimResources) -> AnimalRows` → `animals_to_dict`. Animals by `AnimalId` (at most 12 deer and 4
wolves) and the hunt zones.

| key | type | meaning |
|---|---|---|
| `ids` | PackedInt64Array | `AnimalId`, ascending |
| `kinds` | PackedInt64Array | registered animal code; bundled 0 wolf, 1 deer |
| `names` | PackedStringArray | registered canonical names |
| `name_keys` | PackedStringArray | registered localization keys |
| `categories` | PackedStringArray | registered presentation categories |
| `mechanics` | PackedStringArray | grazer/predator behavior categories |
| `max_hps` | PackedInt32Array | resolved per-world maximum hit points |
| `xs` | PackedInt32Array | tile x |
| `ys` | PackedInt32Array | tile y |
| `hps` | PackedInt32Array | hit points (deer 30, wolf 40 at full) |
| `states` | PackedInt32Array | `AnimalState::code`: 0 wander, 1 flee, 2 rest, 3 hunt, 4 attack, 5 retreat |
| `huntable` | PackedByteArray | 1 when a hunt-capable animal is marked or inside a hunt zone |
| `marked` | PackedByteArray | 1 when an animal carries a hunt mark of its own |
| `zones` | PackedInt32Array | hunt zones as `x0, y0, x1, y1` quadruples, in the order they were added |
| `version` | int | `Wildlife::version` |

A carcass is not an animal: a kill drops meat and a hide as ground items (`get_colony_overlay`, item kinds 11 meat,
6 hide).

### `get_agent_m5`
`get_agent_m5(agent_id)`: `collect_agent_m5(&World, &SimResources, i64) -> AgentM5View` → `agent_m5_to_dict`.

| key | type | meaning |
|---|---|---|
| `found` | bool | the person is alive |
| `mood` | int | mood 0–1,000, -1 without a mood |
| `band` | int | as `mood_bands` in `get_agent_snapshot` |
| `target` | int | the value the mood moves toward now, -1 without a mood |
| `modifier_codes` | PackedInt32Array | the modifiers in force, `Modifier::code`, largest magnitude first, then code |
| `modifier_names` | PackedStringArray | their names (`slept_roofed`, `slept_outside`, `fed`, `hungry`, `thirsty`, `cold`, `quarrelled`, `learned`, `discovered`, `grief`, `sick`, `crafted`, `drafted`, `pressure`) |
| `modifier_values` | PackedInt32Array | their weights after temperament |
| `top` | int | as `tops` in `get_agent_snapshot` |
| `top_cold_permille` | int | the chance ‰ a cold hour bites the wearer (1,000 loincloth, 700 hide wrap, 500 fur cloak): "추위 −30 %" |
| `top_since` | int | display day the top was put on, -1 none |
| `drafted` | bool | drafted |
| `draft_order` | int | 0 hold, 1 move, 2 attack an animal, 3 attack a person, -1 not drafted |
| `draft_x` | int | the move's tile x, or the attack's target id; -1 none |
| `draft_y` | int | the move's tile y; -1 otherwise |
| `sick` | bool | sick |
| `fleeing` | bool | M5.5: running from a wolf (`Fleeing`; the inspector's "도망 중: 늑대") |
| `sick_ticks_left` | int | ticks until the illness passes, 0 when not sick |
| `hp` | int | hit points 0–100 |

### `get_drafted`
`collect_drafted(&World, &SimResources) -> DraftedRows` → `drafted_to_dict`. Drafted people by `AgentId` (the HUD's
list).

| key | type | meaning |
|---|---|---|
| `ids` | PackedInt64Array | `AgentId` |
| `xs` | PackedInt32Array | tile x |
| `ys` | PackedInt32Array | tile y |
| `hps` | PackedInt32Array | hit points 0–100 |
| `orders` | PackedInt32Array | 0 hold, 1 move, 2 attack an animal, 3 attack a person |
| `weapons` | PackedInt64Array | the held tool's `ItemKind::code` (4 flint blade, 5 stone axe, 10 ground axe, 15 spear), -1 bare hands |
| `ticks` | PackedInt64Array | ticks since drafted |

### `get_chronicle_why`
`get_chronicle_why(index)`: `collect_chronicle_why(&Chronicle, i64) -> WhyRows` → `why_to_dict`. The "→ 왜?" chain
of the entry: one row per link, from the entry itself to its first cause (`mood::why_chain`, at most 8 links).

| key | type | meaning |
|---|---|---|
| `found` | bool | the entry exists |
| `links` | PackedInt32Array | 0 a chronicle entry, 1 a mood modifier, 2 a pressure |
| `codes` | PackedInt64Array | the entry index (0), `Modifier::code` (1; 100 = the relation) or `PressureKind::code` (2) |
| `ticks` | PackedInt64Array | an entry's tick, -1 otherwise |
| `kinds` | PackedInt32Array | an entry's `ChronicleKind::as_u8`, -1 otherwise |
| `subjects` | PackedInt64Array | an entry's subject `AgentId`, -1 otherwise |
| `others` | PackedInt64Array | an entry's other `AgentId`, -1 otherwise |
| `causes` | PackedInt64Array | an entry's cause code, -1 otherwise |
| `values` | PackedInt64Array | an entry's value, 0 otherwise |

M6a historical food metadata (same number of rows as the corresponding index/link column):

| key | type | meaning |
|---|---|---|
| `food_daily_milli` | PackedInt64Array | historical winter-plan daily demand in thousandths; -1 absent, 0 zero demand; aligned with `links`; only link0 uses its actual `codes` Chronicle index |
| `food_evidence` | Array[Dictionary] | immutable food operation receipt aligned with `links`; empty dictionary when absent or on non-entry Why links; see M6a immutable food operation evidence below |


### `get_year_summary`
`get_year_summary(year)`: `collect_year_summary(&SimResources, i64) -> YearSummary` → `year_summary_to_dict`. Year
`year` (0-based, `CalendarRule::ticks_per_year` long) counted from the chronicle by entry tick.

| key | type | meaning |
|---|---|---|
| `year` | int | the year asked |
| `found` | bool | the year has begun |
| `complete` | bool | the year is over |
| `births` | int | births |
| `deaths` | PackedInt32Array | deaths by `DeathReason::as_u8`: starvation, dehydration, combat, old age, exposure |
| `pressures` | PackedInt32Array | pressures begun in fixed mechanical buckets cold/drought/pack/illness; includes novel identities |
| `discoveries` | int | knowledge discovered |
| `lost` | int | knowledge lost |
| `quarrels` | int | quarrels |
| `fights` | int | fights after a quarrel |
| `breakdowns` | int | breakdowns |
| `hunted` | int | deer hunted |
| `wolves_killed` | int | wolves killed |
| `era_progress` | int | era progress ‰ now |
| `migrants_joined` | int | M5.5: people who joined from migrant waves in the year |
| `trades` | int | M5.5: trades and gifts made with caravans in the year |
| `eurekas` | int | M5.5: discoveries helped by an artifact in the year |

### `get_teach_suggestion`
`get_teach_suggestion(teacher, knowledge) -> int`: `collect_teach_suggestion(&World, i64, i64)` (`ffi/knowledge.rs`).
The teach dialog's pre-selected learner (M4 owner decision; M5 DECISIONS D5): among living adults other than the
teacher who do not practise the node, the highest novelty seeking, then the youngest, then the lowest `AgentId`; -1
when there is none.

## Neighbours (M5.5)

The two neighbours, their migrant waves and caravans (`ffi/neighbours.rs`, ADR 0031, 0032). Visitors are not agents:
a wave or a caravan is one group on one tile. Every read returns its empty form (empty columns, `found` false) in the
baseline world and without the neighbour rule.

### `get_neighbours`
`collect_neighbours(&World, &SimResources) -> NeighboursView` → `neighbours_to_dict`. Neighbours by id, waves and
caravans by number (the gone ones stay), clues by node id. In `SimReads.GETTERS`.

| key | type | meaning |
|---|---|---|
| `version` | int | `Neighbours::version` (bumped when anything shown changes) |
| `now` | int | the current tick |
| `ids` | PackedInt32Array | neighbour id (0, 1) |
| `name_keys` | PackedStringArray | locale key of the neighbour's name (`sim-core/data/colony/neighbours.ron` `names`) |
| `edges` | PackedInt32Array | 0 north, 1 east, 2 south, 3 west |
| `distance_days` | PackedInt32Array | days of travel (2–5) |
| `populations` | PackedInt32Array | its people |
| `attitudes` | PackedInt32Array | attitude toward the colony 0–100 |
| `last_contact` | PackedInt32Array | display day of the last wave or caravan at the hearth, -1 none |
| `rumoured` | PackedInt64Array | bitmask of the Neolithic nodes it knows (bit = `KnowledgeId`) |
| `wave_ids` | PackedInt32Array | wave number (1, 2, …) |
| `wave_neighbours` | PackedInt32Array | its neighbour id |
| `wave_states` | PackedInt32Array | 0 walking in, 1 waiting at the hearth, 2 leaving, 3 gone |
| `wave_x` | PackedInt32Array | the group's tile x |
| `wave_y` | PackedInt32Array | the group's tile y |
| `wave_sizes` | PackedInt32Array | people in it |
| `wave_deadline` | PackedInt64Array | the tick an unanswered waiting wave leaves, -1 unless waiting |
| `wave_answers` | PackedInt32Array | -1 none, 0 accepted, 1 declined, 2 unanswered |
| `caravan_ids` | PackedInt32Array | caravan number (1, 2, …) |
| `caravan_neighbours` | PackedInt32Array | its neighbour id |
| `caravan_states` | PackedInt32Array | as `wave_states` (1 = trading at the hearth) |
| `caravan_x` | PackedInt32Array | the group's tile x |
| `caravan_y` | PackedInt32Array | the group's tile y |
| `caravan_leaves` | PackedInt64Array | the tick a trading caravan leaves, -1 unless trading |
| `caravan_trades` | PackedInt32Array | trades made with it |
| `clue_nodes` | PackedInt32Array | nodes hinted by an artifact the colony holds that no colonist practises (eureka) |
| `clue_items` | PackedInt64Array | the hinting artifact's `ItemKind::code`, one per `clue_nodes` row |
| `migrants_joined` | int | people who joined so far |
| `trades` | int | trades so far |
| `gifts` | int | gifts so far |
| `eurekas` | int | discoveries helped by an artifact so far |

### `get_wave`
`get_wave(id)`: `collect_wave(&SimResources, i64) -> WaveView` → `wave_to_dict`. Wave `id`'s people in their order;
three knowledge slots per person (their best by ‰, then node id), padded with node -1.

| key | type | meaning |
|---|---|---|
| `found` | bool | the wave exists |
| `id` | int | its number |
| `neighbour` | int | its neighbour id |
| `state` | int | as `wave_states` |
| `deadline` | int | as `wave_deadline` |
| `agents` | PackedInt64Array | the `AgentId` each will have when the wave joins (reserved when it set out; names) |
| `female` | PackedInt32Array | 1 a woman |
| `age_years` | PackedInt32Array | age in years |
| `wrap` | PackedInt32Array | 1 arrives wearing a top |
| `tops` | PackedInt32Array | the top they arrive wearing, `Top::code` (0 none; the data's `migrant_top`, `sim-core/data/colony/neighbours.ron`) |
| `know_nodes` | PackedInt32Array | three per person: `KnowledgeId`, -1 none |
| `know_permille` | PackedInt32Array | three per person: proficiency ‰ |
| `know_practice` | PackedInt32Array | three per person: 1 at or above the node's threshold (숙련) |

### `get_caravan`
`get_caravan(id)`: `collect_caravan(&World, &SimResources, i64) -> CaravanView` → `caravan_to_dict`. Caravan `id`'s
goods in its order (the `take` index of `cmd_trade`) and the colony's stockpiled stacks in `(y, x)` order.

| key | type | meaning |
|---|---|---|
| `found` | bool | the caravan exists |
| `id` | int | its number |
| `neighbour` | int | its neighbour id |
| `state` | int | as `caravan_states` |
| `leaves` | int | as `caravan_leaves` |
| `buy_permille` | int | the traders buy at this ‰ of value (800) |
| `sell_permille` | int | and sell at this ‰ (1,200) |
| `goods_kind` | PackedInt64Array | `ItemKind::code` |
| `goods_count` | PackedInt32Array | items left |
| `goods_quality` | PackedInt32Array | quality 0–100 |
| `goods_value` | PackedInt32Array | one item's value at 100 % (`values.ron`) |
| `goods_unknown` | PackedInt32Array | 1 when no colonist practises a node that makes it (처음 보는 물건 — 단서) |
| `stock_x` | PackedInt32Array | a stockpiled stack's tile x |
| `stock_y` | PackedInt32Array | its tile y |
| `stock_kind` | PackedInt64Array | `ItemKind::code` |
| `stock_count` | PackedInt32Array | items |
| `stock_quality` | PackedInt32Array | quality 0–100 |
| `stock_value` | PackedInt32Array | one item's value at 100 % |

## Commands and lifecycle (not snapshot reads)

The mutators GDScript may call (`SimReads.MUTATORS`): the M1 orders `cmd_designate_chop`, `cmd_designate_chop_tiles`,
`cmd_cancel_area`, `cmd_add_stockpile`, `cmd_remove_zone_area`, `cmd_place_blueprint` (kinds `hut`, `knapping_spot`,
`wood_bench`, `drying_rack`; M4 `grinding_stone`, `pottery_bench`) and the M2 orders `cmd_designate_quarry`, `cmd_add_bill`, `cmd_remove_bill`,
`cmd_set_bill_mode`, `cmd_move_bill`, `cmd_set_bill_suspended`, `cmd_set_work_priority` (`ffi/economy.rs`; names
parsed at the boundary, an unknown name is `unknown` without a log entry), the M3 orders `cmd_place_line(kind, x0,
y0, x1, y1)`, `cmd_place_rect(kind, x0, y0, x1, y1, hollow)` (space kinds `log_wall`, `stone_wall`, `door`,
`packed_floor`, `bedroll`; `cmd_place_blueprint` takes them too, one tile), `cmd_designate_dig`,
`cmd_designate_deconstruct` (rectangles) and `cmd_set_room_role(room, role)` (`bedroom`, `storehouse`,
`workshop`, `hall`, `unassigned` or `auto`; `ffi/space.rs`), the M4 orders `cmd_designate_clay(x0, y0, x1, y1)`,
`cmd_order_teach(teacher, learner, knowledge)` (`AgentId`s and a node id; `count` = the new order id) and
`cmd_cancel_teach(order)` (`ffi/knowledge.rs`; reasons add `unreachable`, `no_practitioner`, `too_young`), the M5 orders
`cmd_draft(agent, on)`, `cmd_undraft_all()` (`count` = people released), `cmd_draft_move(agent, x, y)`,
`cmd_draft_attack(agent, animal)`, `cmd_draft_attack_person(agent, target)`, `cmd_designate_hunt(animal)`,
`cmd_add_hunt_zone(x0, y0, x1, y1)`, `cmd_cancel_hunt(x0, y0, x1, y1)` and `cmd_set_dress_order(on)` (`ffi/story.rs`;
reasons add `not_drafted`), the M5.5 orders `cmd_accept_migrants(wave)`, `cmd_decline_migrants(wave)` and
`cmd_trade(caravan, give, take)` (`give` flat `(x, y, count)` triples of stockpiled stacks, `take` flat
`(goods index, count)` pairs; an empty `take` is a gift; `ffi/neighbours.rs`; reasons add `no_visitor`,
`unbalanced`), each returning
`{accepted: bool, count: int, reason: String}` (`result_to_dict` in `ffi/colony.rs`; `reason` is `""` or a
`RejectReason` name) and logged with the next tick (ADR 0012); `can_place_blueprint(kind, x, y) -> bool`;
`advance_ticks(n) -> int` (the final absolute tick, at most 200,000 per call); `reset_world(seed)`,
`reset_world_stress(seed, agents)`; `on_building_placed(x, y, radius) -> bool`; `set_sim_speed(speed)`; the signal
`world_reset`.


## M-UX bulk UI facts and steward

### `get_ui_facts`
`collect_ui_facts(&World, &SimResources) -> UiFacts` (`ffi/ui_facts/mod.rs`), marshalled by
`ffi/ui_facts/marshal.rs`. This is a read-only bulk observer: it does not advance systems, refresh mutable
simulation caches, consume RNG, or issue orders. `schema_version` is this getter's local format version,
independent of this document's version. Non-colony worlds retain clock metadata with `enabled=false` and
empty fact tables. UI thresholds and content classification are not embedded in this read.

| key | type | meaning |
|---|---|---|
| `schema_version` | int | 1 |
| `enabled` | bool | colony rules active |
| `tick` | int | completed simulation tick; saturated at i64 maximum |
| `day` | int | zero-based display day including clock offset |
| `ticks_per_day` | int | configured ticks per simulation day |
| `clock_offset_ticks` | int | display clock offset |
| `season` | int | season index, -1 when absent |
| `season_day` | int | one-based season day, 0 when absent |
| `days_per_season` | int | configured season duration |
| `meat_spoil_days` | int | display-day lifetime, -1 when wildlife rules absent |
| `taught_count` | int | durable completed-teaching count |
| `wolves_spawned` | int | durable spawned count, not current nearby animals |
| `migrant_waves_arrived` | int | accepted + declined + unanswered + currently waiting waves |
| `first_winter_all_alive` | bool | a seasonal first calendar year has ended with no recorded death during that year; later deaths do not revoke it |
| `people` | Dictionary | living people; columns below, ascending AgentId |
| `settlements` | Dictionary | stock and authoritative needs; ascending SettlementId |
| `stockpile_tiles` | Dictionary | storage cells; zone id then (x,y) |
| `stacks` | Dictionary | ground stacks in native (x,y) spoilage-allocation order |
| `unreachable` | Dictionary | unexpired avoidance records for extant orders, worker id then target tile |
| `meat_lots` | Dictionary | global FIFO creation-day/count lots; not ages attached to individual stacks |
| `tech` | Dictionary | exactly the `get_tech` dictionary |
| `waits` | Dictionary | exactly `get_knowledge_waits`, ordered by kind, a, b, node |
| `neighbours` | Dictionary | exactly `get_neighbours` |
| `animals` | Dictionary | exactly `get_animals`; raw kind/state codes |
| `rooms` | Dictionary | exactly `get_rooms`; raw role codes |
| `story` | Dictionary | exactly `get_story` |
| `workshops` | Dictionary | exactly `get_workshops` |

**`people` columns.** `PackedByteArray` flags are 0/1. Missing optional identities and coordinates are -1;
AgentId and entity bits are distinct, transported using their signed 64-bit bit pattern.

| key | type | meaning |
|---|---|---|
| `ids` | PackedInt64Array | AgentId |
| `entity_ids` | PackedInt64Array | signed hecs entity bits, used by inspector selection |
| `household_ids` | PackedInt64Array | minimum living AgentId in the authoritative settlement household; own AgentId outside one |
| `xs` | PackedInt64Array | current tile x |
| `ys` | PackedInt64Array | current tile y |
| `hps` | PackedFloat64Array | current BodyHealth hp, -1 when absent |
| `max_hps` | PackedFloat64Array | maximum hp, -1 when absent |
| `children` | PackedByteArray | Child component present |
| `infants` | PackedByteArray | configured infant age window applies |
| `born_ticks` | PackedInt64Array | Child birth tick, -1 for absent component |
| `idle` | PackedByteArray | FSM Idle, separate from job/incapacity flags |
| `sleeping` | PackedByteArray | FSM consuming Sleep |
| `roofed` | PackedByteArray | current tile lies in a roofed room |
| `bed_xs` | PackedInt64Array | assigned bed tile x, -1 without a bed |
| `bed_ys` | PackedInt64Array | assigned bed tile y, -1 without a bed |
| `has_job` | PackedByteArray | Job component present |
| `drafted` | PackedByteArray | Drafted component present |
| `sick` | PackedByteArray | Sick component present |
| `breakdown` | PackedByteArray | recorded breakdown includes current display day |
| `nights` | PackedInt64Array | raw Mood night flags, 0 without Mood |
| `mothers` | PackedInt64Array | recorded mother AgentId, -1 absent |
| `fathers` | PackedInt64Array | recorded father AgentId, -1 absent |
| `partners` | PackedInt64Array | recorded partner AgentId, -1 absent |

**`settlements` columns.** Food/wood include stock plus living members' carried amounts.

| key | type | meaning |
|---|---|---|
| `ids` | PackedInt64Array | SettlementId |
| `xs` | PackedInt64Array | formation anchor x |
| `ys` | PackedInt64Array | formation anchor y |
| `food` | PackedInt64Array | food in hand |
| `food_need` | PackedFloat64Array | authoritative daily food consumption |
| `wood` | PackedInt64Array | wood in hand |
| `wood_target` | PackedFloat64Array | seasonal preparation target, 0 when absent |
| `winter_wood_need` | PackedFloat64Array | daily winter burn before future unknown severity |

**`stockpile_tiles` columns.** Room ids and role codes are -1 outside a room.

| key | type | meaning |
|---|---|---|
| `zones` | PackedInt64Array | owning zone id |
| `xs` | PackedInt64Array | tile x |
| `ys` | PackedInt64Array | tile y |
| `can_hold` | PackedByteArray | terrain/occupancy permit storage, 0/1 |
| `rooms` | PackedInt64Array | room id or -1 |
| `room_roles` | PackedInt64Array | raw room role or -1 |
| `roofed` | PackedByteArray | room has roof, 0/1 |

**`stacks` columns.** Reserved stacks are skipped by spoilage; classification uses raw item codes.

| key | type | meaning |
|---|---|---|
| `xs` | PackedInt64Array | tile x |
| `ys` | PackedInt64Array | tile y |
| `kinds` | PackedInt64Array | item kind code |
| `counts` | PackedInt64Array | stack quantity |
| `capacities` | PackedInt64Array | authoritative kind capacity under active rules |
| `reserved` | PackedByteArray | stack reserved by a job, 0/1 |
| `zones` | PackedInt64Array | stockpile zone id, -1 outside zones |

**`unreachable` columns.** Records are retained only while `until_ticks > tick` and a designation,
construction site, or unfinished unsuspended workshop bill still targets that tile.

| key | type | meaning |
|---|---|---|
| `agent_ids` | PackedInt64Array | worker AgentId |
| `xs` | PackedInt64Array | target x |
| `ys` | PackedInt64Array | target y |
| `until_ticks` | PackedInt64Array | exclusive avoidance expiry |

**`meat_lots` columns.** Rows retain simulator FIFO order; use alongside native-order ground stacks,
not as a row-wise join to stacks.

| key | type | meaning |
|---|---|---|
| `days` | PackedInt64Array | creation display day |
| `counts` | PackedInt64Array | lot quantity |

**Reused nested field index.** Each path below has exactly the type, units, row ordering and nested shape
of the corresponding getter linked in the top-level table; these are nested fields, not extra root keys.
The static checker compares the union of literal keys, including nested builders.

| key | nested path(s); full contract above |
|---|---|
| `a` | `waits.a` |
| `adults` | `tech.adults` |
| `at_risk` | `tech.at_risk` |
| `attitudes` | `neighbours.attitudes` |
| `b` | `waits.b` |
| `beds` | `rooms.beds` |
| `bill_active` | `workshops.bill_active` |
| `bill_auto` | `workshops.bill_auto` |
| `bill_counts` | `workshops.bill_counts` |
| `bill_done` | `workshops.bill_done` |
| `bill_ids` | `workshops.bill_ids` |
| `bill_modes` | `workshops.bill_modes` |
| `bill_recipes` | `workshops.bill_recipes` |
| `bill_shops` | `workshops.bill_shops` |
| `bill_suspended` | `workshops.bill_suspended` |
| `caravan_ids` | `neighbours.caravan_ids` |
| `caravan_leaves` | `neighbours.caravan_leaves` |
| `caravan_neighbours` | `neighbours.caravan_neighbours` |
| `caravan_states` | `neighbours.caravan_states` |
| `caravan_trades` | `neighbours.caravan_trades` |
| `caravan_x` | `neighbours.caravan_x` |
| `caravan_y` | `neighbours.caravan_y` |
| `centre_xs` | `rooms.centre_xs` |
| `centre_ys` | `rooms.centre_ys` |
| `clue_items` | `neighbours.clue_items` |
| `clue_nodes` | `neighbours.clue_nodes` |
| `conditions_met` | `tech.conditions_met` |
| `days_of` | `story.days_of` |
| `deaths` | `story.deaths` |
| `distance_days` | `neighbours.distance_days` |
| `doors` | `rooms.doors` |
| `dress_order` | `story.dress_order` |
| `dug` | `rooms.dug` |
| `edges` | `neighbours.edges` |
| `end_days` | `story.end_days` |
| `ended` | `story.ended` |
| `era` | `tech.era` |
| `eras` | `tech.eras` |
| `eurekas` | `neighbours.eurekas` |
| `firewood` | `story.firewood` |
| `firewood_day_need` | `story.firewood_day_need` |
| `first_names` | `tech.first_names` |
| `gifts` | `neighbours.gifts` |
| `holders` | `tech.holders` |
| `hps` | `animals.hps` |
| `hs` | `workshops.hs` |
| `huntable` | `animals.huntable` |
| `ids` | `tech.ids`, `neighbours.ids`, `animals.ids`, `rooms.ids`, `workshops.ids` |
| `intensities` | `story.intensities` |
| `kinds` | `waits.kinds`, `animals.kinds`, `story.kinds`, `workshops.kinds` |
| `lanes` | `tech.lanes` |
| `last_contact` | `neighbours.last_contact` |
| `marked` | `animals.marked` |
| `marker_weights` | `tech.marker_weights` |
| `migrants_joined` | `neighbours.migrants_joined` |
| `name_keys` | `tech.name_keys`, `neighbours.name_keys`, `story.name_keys`, `animals.name_keys` |
| `names` | `tech.names`, `story.names`, `animals.names` |
| `nodes` | `waits.nodes` |
| `now` | `neighbours.now` |
| `on` | `story.on` |
| `overridden` | `rooms.overridden` |
| `pack_size` | `story.pack_size` |
| `pack_kind` | `story.pack_kind` |
| `categories` | `story.categories`, `animals.categories` |
| `mechanics` | `story.mechanics`, `animals.mechanics` |
| `populations` | `neighbours.populations` |
| `practitioners` | `tech.practitioners` |
| `prereq_from` | `tech.prereq_from` |
| `prereq_to` | `tech.prereq_to` |
| `progress` | `tech.progress` |
| `role_names` | `rooms.role_names` |
| `roles` | `rooms.roles` |
| `roofed` | `rooms.roofed` |
| `rumoured` | `neighbours.rumoured` |
| `shelf_counts` | `workshops.shelf_counts` |
| `shelf_kinds` | `workshops.shelf_kinds` |
| `shelf_shops` | `workshops.shelf_shops` |
| `sick` | `story.sick` |
| `start_days` | `story.start_days` |
| `states` | `animals.states` |
| `statuses` | `tech.statuses` |
| `subjects` | `story.subjects` |
| `thresholds` | `tech.thresholds` |
| `tile_rooms` | `rooms.tile_rooms` |
| `tile_xs` | `rooms.tile_xs` |
| `tile_ys` | `rooms.tile_ys` |
| `tiles` | `rooms.tiles` |
| `trades` | `neighbours.trades` |
| `version` | `tech.version`, `neighbours.version`, `animals.version`, `rooms.version`, `story.version` |
| `wave_answers` | `neighbours.wave_answers` |
| `wave_deadline` | `neighbours.wave_deadline` |
| `wave_ids` | `neighbours.wave_ids` |
| `wave_neighbours` | `neighbours.wave_neighbours` |
| `wave_sizes` | `neighbours.wave_sizes` |
| `wave_states` | `neighbours.wave_states` |
| `wave_x` | `neighbours.wave_x` |
| `wave_y` | `neighbours.wave_y` |
| `wolves` | `story.wolves` |
| `wolves_x` | `story.wolves_x` |
| `wolves_y` | `story.wolves_y` |
| `work_xs` | `workshops.work_xs` |
| `work_ys` | `workshops.work_ys` |
| `ws` | `workshops.ws` |
| `xs` | `animals.xs`, `workshops.xs` |
| `ys` | `animals.ys`, `workshops.ys` |
| `zones` | `animals.zones` |

**M6a `food_chain` dictionary.** Read from the same physical-food summary used by the steward;
no simulation transitions, RNG, freshening or policy decisions occur in this getter. The UI's
winter-shortage and expiry-fraction predicates remain in `assets/ui/alerts.json`.

| key | type | meaning |
|---|---|---|
| `food_chain` | Dictionary | colony-wide food observation; `found=false` when absent |
| `found` | bool | colony food policy and settlement exist |
| `stored_units` | int | reachable fresh edible satiety units, excluding uncooked grain |
| `daily_milli` | int | all-member daily demand in thousandths of units |
| `winter_need` | int | authoritative warm-adult reserve bar, ceil(72 × adults / 5) |
| `winter_in` | int | display days until winter; zero in winter |
| `preparing` | bool | authored preparation window is active |
| `expiring_units` | int | stored edible units with at most two days remaining |
| `x` | int | real settlement formation tile x for alert navigation |
| `y` | int | real settlement formation tile y for alert navigation |


### `get_steward`
`collect_steward(&SimResources) -> StewardView` (`ffi/steward.rs`). Persisted mode and accepted-order
provenance, never inferred task completion. Rules are loaded from data; missing rule data yields empty labels.

| key | type | meaning |
|---|---|---|
| `enabled` | bool | automatic planning enabled |
| `defence_enabled` | bool | independent defence policy toggle; defaults true, requires overall enabled |
| `defence_count` | int | actual accepted count in the most recent recruitment batch |
| `defence_active_count` | int | currently steward-owned defender ids, saturated at i64 maximum |
| `scope` | int | 0 all, 1 housing, 2 food |
| `sequence` | int | accepted automatic-action sequence, saturated at i64 maximum |
| `last_rule` | int | most recent successful rule id, -1 before any action |
| `last_tick` | int | tick of last accepted action, saturated at i64 maximum |
| `last_label` | String | localization key for last rule, empty if unavailable |
| `accepted_rules` | PackedInt32Array | distinct rule ids with retained accepted-order provenance, including defence after a nonzero recruitment batch, ascending |
| `rules` | Dictionary[int,String] | configured rule id → localization key, inserted in ascending id order; dynamic numeric keys |

### `cmd_set_steward(on: bool, scope: int)`
Mode command, not a read. `submit_set_steward` parses scope 0/1/2 and sends `Command::SetSteward` through
the ordinary logged command path. An out-of-range or unknown scope rejects with `reason="unknown"`
without submitting a command. The result is the shared `result_to_dict` contract (`ffi/colony.rs`).

| key | type | meaning |
|---|---|---|
| `accepted` | bool | command accepted |
| `count` | int | command result count |
| `reason` | String | empty on success, rejection reason otherwise |

### `cmd_set_steward_defence(on: bool)`
Mode command, not a read. `submit_set_steward_defence` forwards appended
`Command::SetStewardDefence { on }` through the ordinary command log. It changes
only the independent defence toggle, leaving the scope codes 0/1/2 unchanged.
Returns the shared `{accepted: bool, count: int, reason: String}` command result.

### `mux2_native_timings()` — optional diagnostic
Read-only profiling method in `ffi/node.rs`, enabled only when the node is created
with `WORLDSIM_MUX2_PROFILE=1`. This is not a simulation policy API or a cached
`get_*` snapshot. Disabled returns only `status="DISABLED"`. Enabled returns:

| key | type | meaning |
|---|---|---|
| `status` | String | `"MEASURED"` when instrumentation is enabled |
| `epoch` | int | counter generation; increments on engine replacement or system-count reset |
| `frame` | int | Godot process frame last sampled; -1 before first process |
| `frame_ticks` | int | ticks executed in that process frame |
| `executed_ticks` | int | cumulative profiled ticks in this epoch |
| `totals_ns` | Dictionary[String,int] | cumulative nanoseconds: `process`, `tick`, `prune`, `process_residual` (process minus tick), `tick_residual` (tick minus systems and prune); subtraction saturates at zero |
| `systems_ns` | Dictionary[String,int] | runtime system name to cumulative measured nanoseconds |

Difference counters only within one epoch. Instrumentation does not become save
state or a simulation input; residual time is unattributed, not a claim about GC
or a particular script. Timings cover the native process loop, not all rendering.

M-UX2 appends Chronicle kind **46 `StewardDefence`**: `value` is the actual
new recruitment batch count, `cause` is the data-defined rule id. Existing kind
45 `StewardAction` continues to carry its monotonically increasing sequence.

## Text renderer glyphs

`rust/crates/sim-test/src/probe/ascii.rs` draws the colony as one character per tile, rows top to bottom, from the
collectors above only (`collect_terrain_snapshot`, `collect_colony_overlay`, `collect_building_snapshot`,
`collect_trees`, `collect_agent_snapshot`, `collect_agent_jobs`, `collect_appearance`, `collect_time_snapshot`). Run
it with `worldsim_probe --scene colony --seeds S..S+1 --ascii --tick T [--rect x,y,w,h] [--commands FILE]
[--ascii-legend] [--out FILE] [--people-json FILE]`; `--tick T` renders after T completed ticks, applying the
script's entries for ticks below T right before that tick's systems, as the colony probe does. `--rect` crops the full
grid and is clipped to the map.

`--people-json FILE` writes `{seed, tick, count, people, appearance}`: `people` is one `[x, y]` per person, sorted
(compared with the Godot agent layer by `tools/v8/renderer_agreement.sh`); M-ART2 adds `appearance`, one
`{agent_id, x, y, sex, look, variant}` per person in AgentId order — the snapshot's `agent_ids`, `xs`, `ys`, `sexes`,
`look_ids`, `variants` from the same collectors (C2 agreement extended to appearance; `engine_appearance_ffi.rs`).

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


## M6a physical food and seasonal plants

These reads are projections of `SimResources.food` and the immutable content catalog. They do
not advance plants, consume food, repair missing age records or mutate caches. Baseline worlds
return empty columns or `found=false`. All content codes remain open u32 identities transported
in Godot int/PackedInt64Array. Parameterized reads are direct calls; they are not added to the
zero-argument `SimReads.GETTERS` cache.

### `get_plants`
`get_plants(x0: int, y0: int, x1: int, y1: int) -> Dictionary` returns actual plants in an
inclusive viewport tile rectangle, ordered by tile. Negative lower bounds are clipped to zero;
reversed rectangles or negative upper bounds return no rows. No ECS entities are synthesized.

| key | type | meaning |
|---|---|---|
| `xs` | PackedInt64Array | tile x |
| `ys` | PackedInt64Array | tile y |
| `kinds` | PackedInt64Array | registered plant code |
| `phases` | PackedInt64Array | 0 dormant, 1 growing, 2 ripe, 3 spent; depleted ripe plants are spent |
| `units` | PackedInt64Array | remaining harvest pieces |

### `get_agent_food`
`get_agent_food(entity_bits: int) -> Dictionary` accepts signed hecs entity bits, like
`get_agent_detail`, not AgentId. Missing/dead entities return only `found=false`. Existing
`get_agent_detail` fields remain unchanged; this is a separate selected-person read.

| key | type | meaning |
|---|---|---|
| `found` | bool | living colony Agent with Hunger exists |
| `agent_id` | int | stable AgentId |
| `hunger` | float | authoritative current Hunger points |
| `meal_kinds` | PackedInt64Array | completed meal item codes, newest first, at most eight |
| `meal_ticks` | PackedInt64Array | corresponding completion ticks |
| `groups` | PackedStringArray | actual ingredient groups eaten within the trailing three calendar days |

### `get_tile_food`
`get_tile_food(x: int, y: int) -> Dictionary` returns real ground food, pot contents and workshop
shelf food at the tile. Coordinates below zero are clipped to zero. Stack age and storage are
resolved by runtime storage APIs; missing/invalid age data remains unknown, never refreshed.

| key | type | meaning |
|---|---|---|
| `kinds` | PackedInt64Array | food item codes |
| `counts` | PackedInt64Array | physical pieces |
| `remaining_days` | PackedInt64Array | authoritative whole days left; 0 expired, -1 unknown |
| `storage` | PackedStringArray | ground, stockpile, roofed_store or pot |

### `cmd_designate_harvest`
`cmd_designate_harvest(x0, y0, x1, y1) -> Dictionary` submits `Command::DesignateHarvest`
through the ordinary replay-recorded command path. The result uses the existing accepted/count/reason
command contract. The F7 → 5 tool dispatches through `SimReads.mutate`, invalidating cached reads.
It designates real plants; the simulation determines readiness, reservations, travel and yield.


### M6a immutable food operation evidence (kind53)

`get_person_chronicle`, `get_settlement_chronicle`, `get_chronicle_since` append
`food_evidence: Array[Dictionary]` aligned with `index`; `get_chronicle_why`
appends the same field aligned with `links`. Non-entry links and absent legacy
records are `{}`. Existing fields remain unchanged. Evidence is keyed by actual
Chronicle entry identity, never reconstructed from current world state.

Present rows have schema_version1, entry_index, recorded_tick, subject and type.
`acquisition` contains receipt(tick, reason, optional target/job_started and
cancel_reason) plus colony-scoped flow(from_tick,to_tick,harvested_units,
craft_input_units,craft_output_units,eaten_units,spoiled_units,lost_units,
imported_units,exported_units). These flows are food units for the whole colony,
not causal attribution to one person. `intake` contains owner,item,born_day,
lifetime_days; `stored` contains owner,item,pieces,born_day. Owner carries its
kind and recorded tile/agent/workshop/pot identity. No source ancestry, recipe,
command provenance, or opening/closing stock is inferred. Kind53 is hidden in
the owner feed by data but reachable through actual causal parents in Why.

---

## M-READ diagnosis, why-not, requests and current task

### `get_settlement_diagnosis` (no arguments)

Cached hourly settlement diagnosis. Returns a dictionary with parallel arrays:

| key | type | description |
|---|---|---|
| `tick` | `int` | Tick of last evaluation |
| `check_ids` | `PackedStringArray` | Check id from diagnosis.ron |
| `text_keys` | `PackedStringArray` | Locale text key |
| `severities` | `PackedInt32Array` | 0=ok, 1=notice, 2=warning, 3=danger |
| `tags` | `Array[Array[String]]` | Tags per check |
| `params` | `Array[Dictionary]` | Resolved parameters per check |
| `chains` | `Array[Array[Dictionary]]` | Chain entries per check |
| `subject_kind` | `int` | Chain link: subject kind code |
| `check_id` | `String` | Chain link: check id |
| `text_key` | `String` | Chain link: locale text key |
| `severity` | `int` | Chain link: severity code |

### `get_person_diagnosis` (entity_bits: int)

On-demand person diagnosis. Same parallel-array format as `get_settlement_diagnosis`:

| key | type | description |
|---|---|---|
| `check_ids` | `PackedStringArray` | Check id from diagnosis.ron |
| `text_keys` | `PackedStringArray` | Locale text key |
| `severities` | `PackedInt32Array` | 0=ok, 1=notice, 2=warning, 3=danger |
| `tags` | `Array[Array[String]]` | Tags per check |
| `params` | `Array[Dictionary]` | Resolved parameters per check |
| `chains` | `Array[Array[Dictionary]]` | Chain entries per check |
| `subject_kind` | `int` | Chain link: subject kind code |
| `check_id` | `String` | Chain link: check id |
| `text_key` | `String` | Chain link: locale text key |
| `severity` | `int` | Chain link: severity code |

### `get_why_not` (entity_bits: int)

Per-person blocked candidates from the latest mind choice. Returns:

| key | type | description |
|---|---|---|
| `nodes` | `PackedStringArray` | Blocked node authored names |
| `name_keys` | `PackedStringArray` | Locale keys for the blocked nodes (e.g. MIND_BEHAVIOUR_EAT) |
| `bands` | `PackedStringArray` | Band names |
| `fact_names` | `PackedStringArray` | Failing fact name |
| `ops` | `PackedStringArray` | Operator (has/not/</>/>=/<=/>=/==) |
| `thresholds` | `PackedInt64Array` | Authored threshold |
| `actuals` | `PackedInt64Array` | Actual observed value |
| `ticks` | `PackedInt64Array` | Recording tick |
| `byte_size` | `int` | Approximate bytes used per person |

### `get_requests` (no arguments)

Active villager requests per settlement, hourly. Returns:

| key | type | description |
|---|---|---|
| `tick` | `int` | Tick of last aggregation |
| `rule_ids` | `PackedStringArray` | Request rule id from requests.ron |
| `text_keys` | `PackedStringArray` | Locale text key |
| `action_keys` | `PackedStringArray` | Suggested action locale key |
| `priorities` | `PackedInt32Array` | Priority (lower = more urgent) |
| `counts` | `PackedInt32Array` | Number of blocked adults |
| `voices` | `PackedInt64Array` | AgentId of most affected adult |
| `voice_names` | `PackedStringArray` | Always empty; GDScript fills from voice id via PersonNames |
| `steward_active` | `Array[bool]` | Whether the steward addresses this |

### `get_current_task` (no arguments)

The top active request or next unfinished goal. Returns:

| key | type | description |
|---|---|---|
| `source` | `String` | "request" or "none" |
| `text_key` | `String` | Locale text key (when source != "none") |
| `action_key` | `String` | Action locale key |
| `voice` | `int` | AgentId of the voice |
| `voice_name` | `String` | Always empty; GDScript fills from voice id via PersonNames |
| `steward_active` | `bool` | Whether the steward handles it |
| `has_marker` | `bool` | Whether a marker tile is provided |
| `marker_x` | `int` | Marker tile X (when has_marker) |
| `marker_y` | `int` | Marker tile Y (when has_marker) |

### `get_subject_diagnosis` (kind: String, id: int)

On-demand diagnosis for a stockpile, hearth or room. `kind` is `"stockpile"` (id = zone_id), `"hearth"` (id = settlement_id) or `"room"` (id = room_id). Returns the same format as `get_person_diagnosis`:

| key | type | description |
|---|---|---|
| `check_ids` | `PackedStringArray` | Check id from diagnosis.ron |
| `text_keys` | `PackedStringArray` | Locale text key |
| `severities` | `PackedInt32Array` | 0=ok, 1=notice, 2=warning, 3=danger |
| `tags` | `Array[Array[String]]` | Tags per check |
| `params` | `Array[Dictionary]` | Resolved parameters per check |
| `chains` | `Array[Array[Dictionary]]` | Chain entries per check |
| `subject_kind` | `int` | Chain link: subject kind code |
| `check_id` | `String` | Chain link: check id |
| `text_key` | `String` | Chain link: locale text key |
| `severity` | `int` | Chain link: severity code |
