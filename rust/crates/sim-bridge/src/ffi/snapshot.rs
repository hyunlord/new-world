//! Per-frame snapshot reads behind `WorldSimNode`'s getters: agent rows,
//! construction sites, resource tiles, recent deaths, the clock, tallies,
//! world info, terrain and buildings. Each pure collector has a `*_to_dict`
//! marshaller that the `#[func]` in `node.rs` forwards to.

mod resources;

pub(super) use resources::resource_rows_to_dict;
pub use resources::{
    collect_resource_snapshot, resource_rows_amounts, resource_rows_split, ResourceSnapshotRow,
};

use godot::prelude::*;
use sim_core::components::{
    Agent, AgentState, Child, Cold, ConstructionSite, Elder, Hunger, Inventory, Life, Position,
    ResourceKind, SeekTarget, Sleep, TargetKind, Thirst,
};
use sim_engine::{SimEngine, SimResources};

#[cfg(test)]
use super::reads_agent::collect_agent_detail;
#[cfg(test)]
use super::reads_settlement::{collect_settlement_snapshot, collect_settlement_wood_targets};
#[cfg(test)]
use sim_core::components::Settlement;
#[cfg(test)]
use sim_core::material::MaterialRegistry;
#[cfg(test)]
use sim_core::rules::WorldRules;

// ────────────────────────────────────────────────────────────────────────
// P4-γ: Agent snapshot FFI surface — pure-Rust collector + helpers
// ────────────────────────────────────────────────────────────────────────

/// Single row of the agent snapshot returned by [`collect_agent_snapshot`].
///
/// V7 Phase 7-δ extends the row with `state_tag: u8` so the AgentRenderer
/// can tint socializing agents distinctly from Idle / Seeking / non-social
/// Consuming agents.
///
/// Tag table (locked, §2-A-1):
///   - `0` = `AgentState::Idle`
///   - `1` = `AgentState::Seeking { .. }` (non-Activity target)
///   - `2` = `AgentState::Consuming { target: TargetKind::Agent(_) }`
///   - `3` = `AgentState::Consuming { .. }` (non-Agent, non-Activity target)
///   - `4` = Seeking or Consuming with `TargetKind::Activity` (Pass 17)
// `Eq` intentionally NOT derived: viz-B added `f32` need fields (hunger/
// thirst/sleep) which are not `Eq`. `PartialEq` (used by the harness
// `assert_eq!` on `Vec<AgentSnapshotRow>`) is sufficient; no `HashSet`/
// `BTreeSet<AgentSnapshotRow>` exists anywhere.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AgentSnapshotRow {
    /// `hecs::Entity::to_bits().get()` — stable id within a single
    /// `SimEngine` session (not stable across resets or save/load).
    pub entity_bits: u64,
    /// Tile-x coordinate (post-tick if called after `engine.tick()`).
    pub x: u32,
    /// Tile-y coordinate (post-tick if called after `engine.tick()`).
    pub y: u32,
    /// Phase 7-δ: state tag for renderer tint keying. See type doc for the
    /// locked mapping.
    pub state_tag: u8,
    /// V7 Phase 8-δ (code-attempt 3) — the `Agent.id` value carried by the
    /// row's entity. This is the *AgentId* domain (monotonically minted
    /// `u64`), NOT the `hecs::Entity::to_bits` domain. Surfaced because
    /// `CausalEvent::MemoryRecalled.agent` is an `AgentId` and the
    /// renderer needs to map FFI causal events to the corresponding
    /// rendered row. Preserves `entity_bits` for the Phase 4-γ A5
    /// contract so existing callers (palette swap, click handling) are
    /// untouched.
    pub agent_id: u64,
    /// V7 Section 16-ε: Seeking need kind — 0=none, 1=Food, 2=Water,
    /// 3=Sleep. `Seeking{Agent}` / `Seeking{ConstructionSite}` → 0 (they
    /// are not resource trips); `Idle` / `Consuming{..}` → 0. Carries the
    /// head-dot / goal-line colour key for `seek_viz_renderer.gd`.
    pub seek_kind: u8,
    /// V7 Section 16-ε: `SeekTarget.tile.0` as `i32`, or `-1` when the
    /// agent has no `SeekTarget` component. The renderer's goal-line guard
    /// is `target_x >= 0`, so the absent sentinel must be negative.
    pub target_x: i32,
    /// V7 Section 16-ε: `SeekTarget.tile.1` as `i32`, or `-1` when the
    /// agent has no `SeekTarget` component.
    pub target_y: i32,
    /// V7 viz-B: `Hunger.value` (already `f32`, `[0, 100]` where 100 =
    /// SATURATION). The renderer's need-bar danger ratio is `value / 100`.
    /// `0.0` for an agent with no `Hunger` component (defensive — every real
    /// agent has one).
    pub hunger: f32,
    /// V7 viz-B: `Thirst.value` cast `f64 → f32` (`[0, 100]`). Thirst grows
    /// fastest, so it is usually the dominant (killer) need.
    pub thirst: f32,
    /// V7 viz-B: `Sleep.fatigue` cast `f64 → f32` (`[0, 100]`). Note the
    /// source field is `fatigue`, not `value`.
    pub sleep: f32,
    /// V7 Direction-2 slice 2-5b: the agent's `Inventory` Food count
    /// (`ResourceKind::Food`), `0` when the agent has no `Inventory`. Carries
    /// the carry-indicator presence flag for `seek_viz_renderer.gd` — an agent
    /// hauling gathered Food (`carried_food > 0`) gets a visible glyph, making
    /// the gather → CARRY → deposit → stockpile chain visible. Read-only of
    /// `Inventory`; does not affect determinism.
    pub carried_food: i32,
    /// Revitalize Pass 5 — carried `ResourceKind::Wood` count, 0 without an
    /// `Inventory`. Read-only; drives a future stick-carry glyph.
    pub carried_wood: i32,
    /// Detail-compatible target: 0 none, 1 Food, 2 Water, 3 Sleep,
    /// 4 ConstructionSite, 5 Agent, 6 GatherFood, 7 GatherWood, 8 Activity.
    pub target_kind: u8,
    /// Life stage: 0 adult, 1 child with a live Child component, 2 elder
    /// (Pass 4, live Elder component).
    pub life_stage: u8,
}

/// Pass 4 — whole-day age of every snapshot row, in [`collect_agent_snapshot`]
/// order (the same `Agent` + `Position` archetype iteration); `-1` for a
/// resident without `Life` (legacy, v8 and v9 worlds never age).
pub fn collect_agent_ages_days(world: &hecs::World, tick: u64, ticks_per_day: u64) -> Vec<i32> {
    world
        .query::<(&Agent, &Position, Option<&Life>)>()
        .iter()
        .map(|(_, (_, _, life))| {
            life.map_or(-1, |life| {
                (life.age_ticks(tick).max(0) as u64)
                    .checked_div(ticks_per_day)
                    .map_or(0, |days| days.min(i32::MAX as u64) as i32)
            })
        })
        .collect()
}

/// M1.5 — whole-year age of every snapshot row (`age_ticks / ticks_per_year`),
/// in [`collect_agent_snapshot`] order; `-1` without `Life`.
pub fn collect_agent_ages_years(world: &hecs::World, tick: u64, ticks_per_year: u64) -> Vec<i32> {
    collect_agent_ages_days(world, tick, ticks_per_year)
}

/// M1.5 — per snapshot row (in [`collect_agent_snapshot`] order): whether the
/// agent is an infant (`Child::is_infant` under the life-stage rule's
/// `infant_ticks`; 1 or 0), and the whole days left of a pregnancy (`-1` when
/// not pregnant; `Pregnant::days_left`).
pub fn collect_agent_infancy_and_pregnancy(
    world: &hecs::World,
    resources: &SimResources,
    tick: u64,
) -> (Vec<u8>, Vec<i32>) {
    let infant_ticks = resources
        .rules
        .society
        .life_stage
        .and_then(|rule| rule.infant_ticks);
    let tpd = resources.rules.calendar.ticks_per_day;
    world
        .query::<(
            &Agent,
            &Position,
            Option<&Child>,
            Option<&sim_core::components::Pregnant>,
        )>()
        .iter()
        .map(|(_, (_, _, child, pregnant))| {
            (
                u8::from(child.is_some_and(|c| c.is_infant(tick, infant_ticks))),
                pregnant.map_or(-1, |p| p.days_left(tick, tpd).min(i32::MAX as u64) as i32),
            )
        })
        .unzip()
}

/// Pass 6 — `1` for every snapshot row whose agent carries the `Cold` marker,
/// else `0`, in [`collect_agent_snapshot`] order (the same `Agent` +
/// `Position` archetype iteration). All zero without the firewood rule.
pub fn collect_agent_colds(world: &hecs::World) -> Vec<u8> {
    world
        .query::<(&Agent, &Position, Option<&Cold>)>()
        .iter()
        .map(|(_, (_, _, cold))| u8::from(cold.is_some()))
        .collect()
}

/// P4-γ pure-Rust collector (Phase 7-δ extension): iterate the world for
/// `(Agent, Position, AgentState)` and return one row per matching entity
/// in hecs query order.
///
/// Order across two consecutive calls on an unchanged world is stable
/// because hecs archetype iteration order is deterministic. Entities
/// possessing `Position` but *not* `Agent` are excluded by the query
/// filter (the `(&Agent, &Position, &AgentState)` tuple requires all
/// three). The `state_tag` value is computed from the same `AgentState`
/// reference returned by the query — no caching layer is introduced.
///
/// Mirrors `WorldSimNode::get_agent_snapshot` minus the Godot
/// `PackedArray` marshalling — sim-test exercises this directly without
/// a Godot runtime.
pub fn collect_agent_snapshot(world: &hecs::World) -> Vec<AgentSnapshotRow> {
    let mut rows = Vec::new();
    for (
        entity,
        (
            agent,
            pos,
            maybe_state,
            maybe_seek,
            maybe_hunger,
            maybe_thirst,
            maybe_sleep,
            maybe_inv,
            maybe_child,
            maybe_elder,
        ),
    ) in world
        .query::<(
            &Agent,
            &Position,
            Option<&AgentState>,
            Option<&SeekTarget>,
            Option<&Hunger>,
            Option<&Thirst>,
            Option<&Sleep>,
            Option<&Inventory>,
            Option<&Child>,
            Option<&Elder>,
        )>()
        .iter()
    {
        let state_tag: u8 = match maybe_state {
            None | Some(AgentState::Idle) => 0,
            Some(
                AgentState::Seeking {
                    target: TargetKind::Activity,
                }
                | AgentState::Consuming {
                    target: TargetKind::Activity,
                },
            ) => 4,
            Some(AgentState::Seeking { .. }) => 1,
            Some(AgentState::Consuming {
                target: TargetKind::Agent(_),
            }) => 2,
            Some(AgentState::Consuming { .. }) => 3,
        };
        // V7 Section 16-ε — Seeking need kind (resource trips only). State
        // is the source of truth: Food/Water/Sleep seeks colour-key the
        // head-dot + goal-line; ConstructionSite/Agent seeks and every
        // non-Seeking state are 0.
        let seek_kind: u8 = match maybe_state {
            Some(AgentState::Seeking {
                target: TargetKind::Food,
            }) => 1,
            Some(AgentState::Seeking {
                target: TargetKind::Water,
            }) => 2,
            Some(AgentState::Seeking {
                target: TargetKind::Sleep,
            }) => 3,
            _ => 0,
        };
        // V7 Section 16-ε — goal tile, or the -1 no-SeekTarget sentinel.
        let (target_x, target_y) = match maybe_seek {
            Some(st) => (st.tile.0 as i32, st.tile.1 as i32),
            None => (-1, -1),
        };
        // V7 viz-B — need values for the head need-bar. Source fields differ:
        // Hunger.value (f32), Thirst.value (f64), Sleep.fatigue (f64). Cast to
        // f32; default 0.0 when a component is absent (no bar drawn).
        let hunger = maybe_hunger.map(|h| h.value).unwrap_or(0.0);
        let thirst = maybe_thirst.map(|t| t.value as f32).unwrap_or(0.0);
        let sleep = maybe_sleep.map(|s| s.fatigue as f32).unwrap_or(0.0);
        // V7 slice 2-5b — carried Food count for the carry indicator. 0 when
        // the agent has no `Inventory`. Read-only; order-independent.
        let carried_food = maybe_inv
            .map(|inv| inv.get(ResourceKind::Food) as i32)
            .unwrap_or(0);
        let carried_wood = maybe_inv
            .map(|inv| inv.get(ResourceKind::Wood) as i32)
            .unwrap_or(0);
        let target_kind: u8 = match maybe_state.and_then(AgentState::target) {
            None => 0,
            Some(TargetKind::Food) => 1,
            Some(TargetKind::Water) => 2,
            Some(TargetKind::Sleep) => 3,
            Some(TargetKind::ConstructionSite) => 4,
            Some(TargetKind::Agent(_)) => 5,
            Some(TargetKind::GatherFood) => 6,
            Some(TargetKind::GatherWood) => 7,
            Some(TargetKind::Activity) => 8,
            Some(TargetKind::Job) => 9,
        };
        let life_stage = if maybe_child.is_some() {
            1
        } else if maybe_elder.is_some() {
            2
        } else {
            0
        };
        rows.push(AgentSnapshotRow {
            entity_bits: entity.to_bits().get(),
            x: pos.x,
            y: pos.y,
            state_tag,
            // V7 Phase 8-δ (code-attempt 3) — surface the `Agent.id` so the
            // renderer can map `CausalEvent::MemoryRecalled.agent` (AgentId)
            // to the correct rendered row without conflating it with
            // `entity_bits`.
            agent_id: agent.id,
            seek_kind,
            target_x,
            target_y,
            hunger,
            thirst,
            sleep,
            carried_food,
            carried_wood,
            target_kind,
            life_stage,
        });
    }
    rows
}

/// Pure-Rust split of `[AgentSnapshotRow]` into four parallel `Vec`s
/// matching the Godot-side `PackedArray` types (`i64`, `i32`, `i32`, `u8`).
///
/// Lengths are equal by construction. Exposed so sim-test can validate
/// the FFI marshalling invariant without a Godot runtime — `agent_rows_to_dict`
/// is a thin `Vec → PackedArray` adapter over this function.
pub fn agent_rows_split(rows: &[AgentSnapshotRow]) -> (Vec<i64>, Vec<i32>, Vec<i32>, Vec<u8>) {
    let n = rows.len();
    let mut ids = Vec::with_capacity(n);
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    let mut states = Vec::with_capacity(n);
    for row in rows {
        ids.push(row.entity_bits as i64);
        xs.push(row.x as i32);
        ys.push(row.y as i32);
        states.push(row.state_tag);
    }
    (ids, xs, ys, states)
}

/// Marshal a [`AgentSnapshotRow`] slice into the FFI dictionary shape
/// documented on [`WorldSimNode::get_agent_snapshot`]. Four keys, four
/// `PackedArray`s, lengths always equal to `rows.len()`.
pub(super) fn agent_rows_to_dict(rows: &[AgentSnapshotRow]) -> VarDictionary {
    let (ids_vec, xs_vec, ys_vec, states_vec) = agent_rows_split(rows);
    let mut ids = PackedInt64Array::new();
    let mut xs = PackedInt32Array::new();
    let mut ys = PackedInt32Array::new();
    let mut states = PackedByteArray::new();
    // V7 Phase 8-δ (code-attempt 3) — parallel `agent_ids` array carrying
    // `Agent.id` per row, used by the renderer to match
    // `CausalEvent::MemoryRecalled.agent` (AgentId) against the rendered
    // row. `ids` remains keyed by `entity_bits` for the Phase 4-γ A5
    // contract (palette swap, click handling).
    let mut agent_ids = PackedInt64Array::new();
    ids.resize(ids_vec.len());
    xs.resize(xs_vec.len());
    ys.resize(ys_vec.len());
    states.resize(states_vec.len());
    agent_ids.resize(rows.len());
    for (i, v) in ids_vec.iter().enumerate() {
        ids[i] = *v;
    }
    for (i, v) in xs_vec.iter().enumerate() {
        xs[i] = *v;
    }
    for (i, v) in ys_vec.iter().enumerate() {
        ys[i] = *v;
    }
    for (i, v) in states_vec.iter().enumerate() {
        states[i] = *v;
    }
    for (i, row) in rows.iter().enumerate() {
        agent_ids[i] = row.agent_id as i64;
    }
    // V7 Section 16-ε — three additive parallel arrays carrying the head-dot
    // / goal-line fields. Built directly from `rows` (NOT routed through
    // `agent_rows_split`, whose 4-tuple signature is locked by
    // `harness_p4_gamma_rendering`). Lengths equal `rows.len()` by
    // construction, matching the existing `ids`/`xs`/`ys`/`states` contract.
    let mut seek_kinds = PackedByteArray::new();
    let mut target_xs = PackedInt32Array::new();
    let mut target_ys = PackedInt32Array::new();
    seek_kinds.resize(rows.len());
    target_xs.resize(rows.len());
    target_ys.resize(rows.len());
    for (i, row) in rows.iter().enumerate() {
        seek_kinds[i] = row.seek_kind;
        target_xs[i] = row.target_x;
        target_ys[i] = row.target_y;
    }
    // V7 viz-B — three additive parallel `f32` arrays carrying the per-agent
    // need values (Hunger / Thirst / Sleep, `[0, 100]`) for the head need-bar.
    // Built directly from `rows` (NOT via `agent_rows_split`, whose 4-tuple is
    // locked by `harness_p4_gamma_rendering`), mirroring the Section 16-ε
    // seek_kinds/target_xs/target_ys additive pattern. Lengths == `rows.len()`.
    let mut hungers = PackedFloat32Array::new();
    let mut thirsts = PackedFloat32Array::new();
    let mut sleeps = PackedFloat32Array::new();
    hungers.resize(rows.len());
    thirsts.resize(rows.len());
    sleeps.resize(rows.len());
    for (i, row) in rows.iter().enumerate() {
        hungers[i] = row.hunger;
        thirsts[i] = row.thirst;
        sleeps[i] = row.sleep;
    }
    // V7 slice 2-5b — one additive parallel `i32` array carrying the per-agent
    // carried Food count for the carry indicator. Built directly from `rows`
    // (NOT via `agent_rows_split`, whose 4-tuple is locked by
    // `harness_p4_gamma_rendering`), mirroring the Section 16-ε / viz-B additive
    // pattern. Length == `rows.len()` by construction.
    let mut carried_foods = PackedInt32Array::new();
    carried_foods.resize(rows.len());
    for (i, row) in rows.iter().enumerate() {
        carried_foods[i] = row.carried_food;
    }
    // Revitalize Pass 5 — additive parallel array of carried wood.
    let mut carried_woods = PackedInt32Array::new();
    carried_woods.resize(rows.len());
    for (i, row) in rows.iter().enumerate() {
        carried_woods[i] = row.carried_wood;
    }
    let mut target_kinds = PackedByteArray::new();
    let mut life_stages = PackedByteArray::new();
    target_kinds.resize(rows.len());
    life_stages.resize(rows.len());
    for (i, row) in rows.iter().enumerate() {
        target_kinds[i] = row.target_kind;
        life_stages[i] = row.life_stage;
    }
    let mut dict = VarDictionary::new();
    dict.set("ids", ids);
    dict.set("xs", xs);
    dict.set("ys", ys);
    dict.set("states", states);
    dict.set("agent_ids", agent_ids);
    dict.set("seek_kinds", seek_kinds);
    dict.set("target_xs", target_xs);
    dict.set("target_ys", target_ys);
    dict.set("hungers", hungers);
    dict.set("thirsts", thirsts);
    dict.set("sleeps", sleeps);
    dict.set("carried_foods", carried_foods);
    dict.set("carried_woods", carried_woods);
    dict.set("target_kinds", target_kinds);
    dict.set("life_stages", life_stages);
    dict
}

// ────────────────────────────────────────────────────────────────────────
// V7 Phase 12-β.2 (A3): Construction snapshot FFI surface
// ────────────────────────────────────────────────────────────────────────

/// Single row of the construction snapshot returned by
/// [`collect_construction_snapshot`].
///
/// V7 Phase 12-β.2 (A3) — surfaces `ConstructionSite` entities to the
/// GDScript renderer so the user can see agent construction activity.
/// Only the `progress` ratio is exposed; the `BlueprintId` and
/// `footprint` fields of the underlying `BuildingBlueprint` are
/// intentionally NOT in the row because the substrate has no
/// `BuildingType` taxonomy — multi-type rendering is a separate
/// (deferred) phase that first adds that substrate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstructionSnapshotRow {
    /// `hecs::Entity::to_bits().get()` of the construction-site entity.
    pub entity_bits: u64,
    /// Tile-x coordinate of the site's footprint top-left.
    pub x: u32,
    /// Tile-y coordinate of the site's footprint top-left.
    pub y: u32,
    /// Current construction progress in `ConstructionSystem` ticks.
    pub progress: u32,
    /// Total ticks required for completion (from `BuildingBlueprint`).
    pub required_progress: u32,
}

/// Pure-Rust collector for [`ConstructionSnapshotRow`] — mirrors
/// [`collect_agent_snapshot`] but queries `(&ConstructionSite, &Position)`
/// instead of `(&Agent, &Position, Option<&AgentState>)`.
///
/// Position is taken from the entity's `Position` component (the
/// canonical sim-core source) rather than `ConstructionSite::position`
/// so the rendered tile matches whatever the simulation considers
/// authoritative for that entity. The two should agree in practice.
pub fn collect_construction_snapshot(world: &hecs::World) -> Vec<ConstructionSnapshotRow> {
    let mut rows = Vec::new();
    for (entity, (site, pos)) in world.query::<(&ConstructionSite, &Position)>().iter() {
        rows.push(ConstructionSnapshotRow {
            entity_bits: entity.to_bits().get(),
            x: pos.x,
            y: pos.y,
            progress: site.progress,
            required_progress: site.blueprint.required_progress,
        });
    }
    rows
}

/// Marshal a [`ConstructionSnapshotRow`] slice into the FFI dictionary
/// shape consumed by `WorldRenderer._process()`. Five parallel
/// `PackedArray`s, lengths always equal to `rows.len()`.
///
/// Keys:
/// - `ids`:  `PackedInt64Array` — `entity_bits` per row (signed cast
///   matches the agent snapshot precedent).
/// - `xs`:  `PackedInt32Array` — tile-x per row, as `i32`.
/// - `ys`:  `PackedInt32Array` — tile-y per row, as `i32`.
/// - `progresses`: `PackedInt32Array` — current progress per row, as `i32`.
/// - `required_progresses`: `PackedInt32Array` — required progress per
///   row, as `i32`. The renderer must still defend against div-by-zero
///   via `max(req, 1)` because the substrate permits `required_progress == 0`
///   blueprints.
pub(super) fn construction_rows_to_dict(rows: &[ConstructionSnapshotRow]) -> VarDictionary {
    let n = rows.len();
    let mut ids = PackedInt64Array::new();
    let mut xs = PackedInt32Array::new();
    let mut ys = PackedInt32Array::new();
    let mut progresses = PackedInt32Array::new();
    let mut required_progresses = PackedInt32Array::new();
    ids.resize(n);
    xs.resize(n);
    ys.resize(n);
    progresses.resize(n);
    required_progresses.resize(n);
    for (i, row) in rows.iter().enumerate() {
        ids[i] = row.entity_bits as i64;
        xs[i] = row.x as i32;
        ys[i] = row.y as i32;
        progresses[i] = row.progress as i32;
        required_progresses[i] = row.required_progress as i32;
    }
    let mut dict = VarDictionary::new();
    dict.set("ids", ids);
    dict.set("xs", xs);
    dict.set("ys", ys);
    dict.set("progresses", progresses);
    dict.set("required_progresses", required_progresses);
    dict
}

// ────────────────────────────────────────────────────────────────────────
// V7 viz-D: Recent-deaths snapshot FFI surface
// ────────────────────────────────────────────────────────────────────────

/// Single row of the recent-deaths snapshot returned by
/// [`collect_recent_deaths`]. Documented casts from the buffer entry:
/// `x`/`y` `i32 → i32` (identity), `reason` `DeathReason → u8 → i32`,
/// `tick` `u32 → i64` (widening).
/// `reason_u8` encoding mirrors [`DeathReason::as_u8`]: `0 = Starvation`
/// (brown), `1 = Dehydration` (blue), `2 = Combat` (red).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecentDeathRow {
    /// Death tile-x.
    pub x: i32,
    /// Death tile-y.
    pub y: i32,
    /// Reason discriminant (`0/1/2`) — see [`DeathReason::as_u8`].
    pub reason_u8: i32,
    /// Simulation tick at which the death occurred.
    pub tick: i64,
}

/// Pure-Rust collector over [`SimResources::recent_deaths`], preserving buffer
/// order (the renderer fades each marker by `current_tick - tick`, so order
/// only matters for stable iteration). Sim-test exercises this directly.
pub fn collect_recent_deaths(resources: &SimResources) -> Vec<RecentDeathRow> {
    resources
        .recent_deaths
        .iter()
        .map(|d| RecentDeathRow {
            // `d.x`/`d.y` are already `i32` (buffer schema) → identity cast,
            // written without `as i32` to satisfy `clippy::unnecessary_cast`.
            x: d.x,
            y: d.y,
            reason_u8: d.reason.as_u8() as i32,
            tick: d.tick as i64,
        })
        .collect()
}

/// Pure-Rust marshalling split — the SINGLE source of the
/// `(xs, ys, reasons, ticks)` integer arrays, in buffer order. Extracted so the
/// harness can verify the exact integers the FFI emits WITHOUT a Godot runtime
/// (`VarDictionary` / `PackedInt32Array` require Godot).
/// [`recent_death_rows_to_dict`] MUST build its arrays from this — no duplicate
/// marshalling logic.
pub fn recent_death_rows_split(
    rows: &[RecentDeathRow],
) -> (Vec<i32>, Vec<i32>, Vec<i32>, Vec<i64>) {
    let mut xs = Vec::with_capacity(rows.len());
    let mut ys = Vec::with_capacity(rows.len());
    let mut reasons = Vec::with_capacity(rows.len());
    let mut ticks = Vec::with_capacity(rows.len());
    for r in rows {
        xs.push(r.x);
        ys.push(r.y);
        reasons.push(r.reason_u8);
        ticks.push(r.tick);
    }
    (xs, ys, reasons, ticks)
}

/// Marshal a [`RecentDeathRow`] slice + the engine's `current_tick` into the FFI
/// dictionary shape consumed by `death_viz_renderer.gd`. Four parallel arrays
/// (`xs`, `ys` as `PackedInt32Array`; `reasons` as `PackedInt32Array`; `ticks`
/// as `PackedInt64Array`) of equal length `rows.len()`, plus the scalar
/// `current_tick` (`i64`) the renderer uses to compute each marker's fade age.
/// Built solely from [`recent_death_rows_split`] (the single marshalling path
/// the harness tests).
pub(super) fn recent_death_rows_to_dict(
    rows: &[RecentDeathRow],
    current_tick: i64,
) -> VarDictionary {
    let (xv, yv, rv, tv) = recent_death_rows_split(rows);
    let n = rows.len();
    let mut xs = PackedInt32Array::new();
    let mut ys = PackedInt32Array::new();
    let mut reasons = PackedInt32Array::new();
    let mut ticks = PackedInt64Array::new();
    xs.resize(n);
    ys.resize(n);
    reasons.resize(n);
    ticks.resize(n);
    for i in 0..n {
        xs[i] = xv[i];
        ys[i] = yv[i];
        reasons[i] = rv[i];
        ticks[i] = tv[i];
    }
    let mut dict = VarDictionary::new();
    dict.set("xs", xs);
    dict.set("ys", ys);
    dict.set("reasons", reasons);
    dict.set("ticks", ticks);
    dict.set("current_tick", current_tick);
    dict
}

/// Native simulation clock snapshot, testable without a Godot runtime.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeSnapshot {
    /// Number of completed simulation ticks.
    pub tick: u64,
    /// Simulation ticks in a day; zero disables the simulation clock.
    pub ticks_per_day: u64,
    /// Zero-based day, or zero when the simulation clock is disabled.
    pub day_index: u64,
    /// Authoritative simulation hour in the range 0..24.
    pub time_of_day: f64,
    /// Pass 4 season of `day_index`: 0 spring, 1 summer, 2 autumn, 3 winter (0 without seasons).
    pub season: u32,
    /// One-based day within the season (0 without seasons).
    pub day_of_season: u32,
    /// Days per season (0 without seasons).
    pub days_per_season: u32,
    /// Pass 7 severity of the winter at the completed tick: 0 mild, 1 normal,
    /// 2 harsh, -1 outside a winter or without the rule or weights; from 00:00
    /// to the burn end it reads the evening's winter (night booking).
    pub winter_severity: i64,
    /// M1.5: zero-based calendar year of `day_index` (0 without seasons).
    pub year_index: u64,
    /// M1.5: zero-based season (the same value as `season`).
    pub season_index: u32,
    /// M1.5: one-based day within the year (0 without seasons).
    pub day_of_year: u32,
    /// M1.5: days per calendar year (0 without seasons).
    pub days_per_year: u32,
}

/// Read the completed tick count and clock resources without mutation.
pub fn collect_time_snapshot(engine: &SimEngine) -> TimeSnapshot {
    let ticks_per_day = engine.resources.ticks_per_day;
    let day_index = engine
        .current_tick()
        .saturating_add(engine.resources.rules.clock_offset_ticks)
        .checked_div(ticks_per_day)
        .unwrap_or(0);
    // M1.5: the calendar's days per season (`WorldRules::season_days`).
    let rules = &engine.resources.rules;
    let days_per_season = rules.season_days();
    let (year_index, season, day_of_season, day_of_year, days_per_year) = if days_per_season == 0 {
        (0, 0, 0, 0, 0)
    } else {
        let (year, season, day, day_of_year) = rules.calendar.date_of_day(day_index);
        (
            year,
            season,
            day + 1,
            day_of_year + 1,
            rules.calendar.days_per_year() as u32,
        )
    };
    TimeSnapshot {
        tick: engine.current_tick(),
        ticks_per_day,
        day_index,
        time_of_day: engine.resources.time_of_day,
        season,
        day_of_season,
        days_per_season,
        winter_severity: sim_systems::runtime::settlement::firewood::winter_severity_at(
            &engine.resources,
            engine.current_tick(),
        ),
        year_index,
        season_index: season,
        day_of_year,
        days_per_year,
    }
}

/// Read the cumulative counters without depending on event retention.
pub fn collect_tallies(resources: &SimResources) -> sim_engine::Tallies {
    resources.tallies
}

pub(super) fn time_snapshot_to_dict(snapshot: TimeSnapshot) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("tick", snapshot.tick as i64);
    dict.set("ticks_per_day", snapshot.ticks_per_day as i64);
    dict.set("day_index", snapshot.day_index as i64);
    dict.set("time_of_day", snapshot.time_of_day);
    dict.set("season", snapshot.season as i64);
    dict.set("day_of_season", snapshot.day_of_season as i64);
    dict.set("days_per_season", snapshot.days_per_season as i64);
    dict.set("winter_severity", snapshot.winter_severity);
    dict.set("year_index", snapshot.year_index as i64);
    dict.set("season_index", snapshot.season_index as i64);
    dict.set("day_of_year", snapshot.day_of_year as i64);
    dict.set("days_per_year", snapshot.days_per_year as i64);
    dict
}

pub(super) fn tallies_to_dict(tallies: sim_engine::Tallies) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("births", tallies.births as i64);
    dict.set("deaths_starvation", tallies.deaths_starvation as i64);
    dict.set("deaths_dehydration", tallies.deaths_dehydration as i64);
    dict.set("deaths_combat", tallies.deaths_combat as i64);
    dict.set("deaths_old_age", tallies.deaths_old_age as i64);
    // Revitalize Pass 10: cold-night deaths under the exposure rule.
    dict.set("deaths_exposure", tallies.deaths_exposure as i64);
    // Revitalize Pass 11 tallies (display-only).
    dict.set("fissions", tallies.fissions as i64);
    dict.set("fissions_cancelled", tallies.fissions_cancelled as i64);
    dict.set("remnant_merges", tallies.remnant_merges as i64);
    // Revitalize Pass 12: wood a raining settlement burned wet (display-only).
    // Revitalize Pass 13: settled-area floor wood placed (display-only).
    dict.set("area_floor_placed", tallies.area_floor_placed as i64);
    // Revitalize Pass 5 tallies (display-only).
    dict.set("meals_from_store", tallies.meals_from_store as i64);
    dict.set("ground_meals", tallies.ground_meals as i64);
    dict.set("wood_trips", tallies.wood_trips as i64);
    dict.set("wood_gathered", tallies.wood_gathered as i64);
    dict.set("gather_preempted", tallies.gather_preempted as i64);
    dict.set("cold_member_hours", tallies.cold_member_hours as i64);
    dict
}

/// Seed and pacing metadata consumed by renderers and deterministic tooling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldInfoSnapshot {
    /// Seed used by world generation.
    pub seed: u64,
    /// Map width in tiles.
    pub width: u32,
    /// Map height in tiles.
    pub height: u32,
    /// Simulation ticks per day.
    pub ticks_per_day: u64,
    /// Ticks between agent movement opportunities.
    pub move_interval_ticks: u64,
    /// Display clock offset from elapsed ticks.
    pub clock_offset_ticks: u64,
    /// M1.5: ticks per calendar year (`CalendarRule::ticks_per_year`); ages
    /// in years are whole `age_ticks / ticks_per_year`.
    pub ticks_per_year: u64,
}

/// Read immutable world metadata directly from the authoritative resources.
pub fn collect_world_info(resources: &SimResources) -> WorldInfoSnapshot {
    WorldInfoSnapshot {
        seed: resources.world_seed,
        width: resources.tile_grid.width,
        height: resources.tile_grid.height,
        ticks_per_day: resources.ticks_per_day,
        move_interval_ticks: resources.rules.agent.move_interval_ticks,
        clock_offset_ticks: resources.rules.clock_offset_ticks,
        ticks_per_year: resources.rules.calendar.ticks_per_year(),
    }
}

pub(super) fn world_info_to_dict(snapshot: WorldInfoSnapshot) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("seed", snapshot.seed as i64);
    dict.set("width", snapshot.width as i64);
    dict.set("height", snapshot.height as i64);
    dict.set("ticks_per_day", snapshot.ticks_per_day as i64);
    dict.set("move_interval_ticks", snapshot.move_interval_ticks as i64);
    dict.set("clock_offset_ticks", snapshot.clock_offset_ticks as i64);
    dict.set("ticks_per_year", snapshot.ticks_per_year as i64);
    dict
}

/// Row-major terrain arrays independent of Godot packed-array allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainSnapshot {
    /// Map width in tiles.
    pub width: u32,
    /// Map height in tiles.
    pub height: u32,
    /// Declaration-order terrain codes (Plain 0 through Cave 9).
    pub terrain: Vec<u8>,
    /// Elevation bytes, aligned with terrain.
    pub elevation: Vec<u8>,
}

/// Copy simulation grid data in its canonical row-major order.
pub fn collect_terrain_snapshot(resources: &SimResources) -> TerrainSnapshot {
    TerrainSnapshot {
        width: resources.tile_grid.width,
        height: resources.tile_grid.height,
        terrain: resources
            .tile_grid
            .terrain_type
            .iter()
            .map(|terrain| *terrain as u8)
            .collect(),
        elevation: resources.tile_grid.elevation.clone(),
    }
}

pub(super) fn terrain_snapshot_to_dict(snapshot: &TerrainSnapshot) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("width", snapshot.width as i64);
    dict.set("height", snapshot.height as i64);
    dict.set(
        "terrain",
        PackedByteArray::from(snapshot.terrain.as_slice()),
    );
    dict.set(
        "elevation",
        PackedByteArray::from(snapshot.elevation.as_slice()),
    );
    dict
}

/// One real registered building and its simulation-owned kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildingSnapshotRow {
    /// Stable BuildingId from the building registry.
    pub id: u64,
    /// Building tile x coordinate.
    pub x: u32,
    /// Building tile y coordinate.
    pub y: u32,
    /// BuildingKind::code, defaulting to Hearth (0) for legacy buildings.
    pub kind: u8,
}

/// Join registered buildings with their tile-keyed kinds and sort by id.
pub fn collect_building_snapshot(resources: &SimResources) -> Vec<BuildingSnapshotRow> {
    let mut rows: Vec<BuildingSnapshotRow> = resources
        .building_registry
        .iter()
        .map(|(&id, &(x, y))| BuildingSnapshotRow {
            id,
            x,
            y,
            kind: resources
                .building_kinds
                .get(&(x, y))
                .map_or(0, |kind| kind.code()),
        })
        .collect();
    rows.sort_by_key(|row| row.id);
    rows
}

pub(super) fn building_rows_to_dict(rows: &[BuildingSnapshotRow]) -> VarDictionary {
    let mut ids = PackedInt64Array::new();
    let mut xs = PackedInt32Array::new();
    let mut ys = PackedInt32Array::new();
    let mut kinds = PackedByteArray::new();
    ids.resize(rows.len());
    xs.resize(rows.len());
    ys.resize(rows.len());
    kinds.resize(rows.len());
    for (index, row) in rows.iter().enumerate() {
        ids[index] = row.id as i64;
        xs[index] = row.x as i32;
        ys[index] = row.y as i32;
        kinds[index] = row.kind;
    }
    let mut dict = VarDictionary::new();
    dict.set("ids", ids);
    dict.set("xs", xs);
    dict.set("ys", ys);
    dict.set("kinds", kinds);
    dict
}

#[cfg(test)]
mod revitalize_world_snapshot_tests {
    use super::{
        collect_building_snapshot, collect_terrain_snapshot, collect_world_info,
        BuildingSnapshotRow,
    };
    use sim_core::components::BuildingKind;
    use sim_core::material::{MaterialRegistry, TerrainType};
    use sim_engine::SimResources;

    #[test]
    fn world_info_reads_world_seed_dimensions_and_rules() {
        let mut resources = SimResources::new(3, 7, MaterialRegistry::new());
        resources.world_seed = 41;
        resources.ticks_per_day = 120;
        resources.rules.agent.move_interval_ticks = 6;
        let snapshot = collect_world_info(&resources);
        assert_eq!(snapshot.seed, 41);
        assert_eq!((snapshot.width, snapshot.height), (3, 7));
        assert_eq!(snapshot.ticks_per_day, 120);
        assert_eq!(snapshot.move_interval_ticks, 6);
    }

    #[test]
    fn terrain_snapshot_preserves_all_codes_and_row_major_elevation() {
        let mut resources = SimResources::new(5, 2, MaterialRegistry::new());
        resources.tile_grid.terrain_type = vec![
            TerrainType::Plain,
            TerrainType::Forest,
            TerrainType::Mountain,
            TerrainType::Hill,
            TerrainType::River,
            TerrainType::Coast,
            TerrainType::Desert,
            TerrainType::Tundra,
            TerrainType::Swamp,
            TerrainType::Cave,
        ];
        resources.tile_grid.elevation = vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0];
        let snapshot = collect_terrain_snapshot(&resources);
        assert_eq!((snapshot.width, snapshot.height), (5, 2));
        assert_eq!(snapshot.terrain, (0..10).collect::<Vec<u8>>());
        assert_eq!(snapshot.elevation, vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
    }

    #[test]
    fn building_snapshot_sorts_ids_and_defaults_legacy_to_hearth() {
        let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
        resources.building_registry.insert(7, (1, 2));
        resources.building_registry.insert(2, (3, 4));
        resources.building_registry.insert(5, (5, 6));
        resources
            .building_kinds
            .insert((3, 4), BuildingKind::Shelter);
        resources
            .building_kinds
            .insert((5, 6), BuildingKind::Stockpile);
        // An unregistered kind must not invent a real building.
        resources
            .building_kinds
            .insert((0, 0), BuildingKind::Hearth);
        assert_eq!(
            collect_building_snapshot(&resources),
            vec![
                BuildingSnapshotRow {
                    id: 2,
                    x: 3,
                    y: 4,
                    kind: 1
                },
                BuildingSnapshotRow {
                    id: 5,
                    x: 5,
                    y: 6,
                    kind: 2
                },
                BuildingSnapshotRow {
                    id: 7,
                    x: 1,
                    y: 2,
                    kind: 0
                },
            ]
        );
    }
}

#[cfg(test)]
mod agent_snapshot_tests {
    use super::*;

    #[test]
    fn empty_world_has_empty_parallel_snapshot_rows() {
        let world = hecs::World::new();
        let rows = collect_agent_snapshot(&world);
        let (ids, xs, ys, states) = agent_rows_split(&rows);
        assert!(rows.is_empty());
        assert_eq!([ids.len(), xs.len(), ys.len(), states.len()], [0; 4]);
        let target_kinds: Vec<u8> = rows.iter().map(|row| row.target_kind).collect();
        let life_stages: Vec<u8> = rows.iter().map(|row| row.life_stage).collect();
        assert!(target_kinds.is_empty());
        assert!(life_stages.is_empty());
    }

    #[test]
    fn missing_state_and_child_default_to_none_and_adult() {
        let mut world = hecs::World::new();
        world.spawn((Agent { id: 7 }, Position::new(2, 3)));
        let rows = collect_agent_snapshot(&world);
        assert_eq!(rows.len(), 1);
        assert_eq!((rows[0].target_kind, rows[0].life_stage), (0, 0));
    }

    #[test]
    fn mixed_snapshot_targets_match_detail_and_live_child_markers() {
        let mut world = hecs::World::new();
        let targets = [
            None,
            Some(TargetKind::Food),
            Some(TargetKind::Water),
            Some(TargetKind::Sleep),
            Some(TargetKind::ConstructionSite),
            Some(TargetKind::Agent(99)),
            Some(TargetKind::GatherFood),
            Some(TargetKind::GatherWood),
            Some(TargetKind::Activity),
        ];
        let mut expected = Vec::new();
        for (code, target) in targets.into_iter().enumerate() {
            for consuming in [false, true] {
                let state = match target {
                    None => AgentState::Idle,
                    Some(target) if consuming => AgentState::Consuming { target },
                    Some(target) => AgentState::Seeking { target },
                };
                let entity = world.spawn((
                    Agent {
                        id: expected.len() as u64,
                    },
                    Position::new(2, 3),
                    state,
                    Hunger::new(10.0, 0.05),
                    Thirst::new(20.0, 0.08),
                    Sleep::new(30.0, 0.03),
                ));
                if consuming {
                    world
                        .insert_one(entity, Child { born_tick: 5 })
                        .expect("child fixture");
                }
                let state_code = match (code, consuming) {
                    (0, _) => 0,
                    (8, _) => 4,
                    (5, true) => 2,
                    (_, false) => 1,
                    (_, true) => 3,
                };
                expected.push((entity, code as u8, u8::from(consuming), state_code));
            }
        }
        let rows = collect_agent_snapshot(&world);
        let (ids, xs, ys, states) = agent_rows_split(&rows);
        let target_kinds: Vec<u8> = rows.iter().map(|row| row.target_kind).collect();
        let life_stages: Vec<u8> = rows.iter().map(|row| row.life_stage).collect();
        assert_eq!(rows.len(), expected.len());
        assert_eq!(
            [
                ids.len(),
                xs.len(),
                ys.len(),
                states.len(),
                target_kinds.len(),
                life_stages.len()
            ],
            [expected.len(); 6]
        );
        for (entity, target, stage, state_code) in expected {
            let index = rows
                .iter()
                .position(|row| row.entity_bits == entity.to_bits().get())
                .expect("fixture row present across archetypes");
            let detail = collect_agent_detail(&world, entity.to_bits().get());
            assert!(detail.found);
            assert_eq!((target_kinds[index], life_stages[index]), (target, stage));
            assert_eq!(i32::from(target_kinds[index]), detail.target_kind);
            assert_eq!(states[index], state_code);
            assert_eq!(detail.state_tag, state_code);
        }
    }

    #[test]
    fn elders_report_stage_two_and_ages_follow_snapshot_order() {
        use sim_core::components::{Elder, Life};
        let mut world = hecs::World::new();
        world.spawn((Agent { id: 1 }, Position::new(2, 3)));
        world.spawn((
            Agent { id: 2 },
            Position::new(2, 3),
            Life {
                born_tick: -2_880,
                lifespan_ticks: 90_000,
            },
            Elder,
        ));
        world.spawn((
            Agent { id: 3 },
            Position::new(2, 3),
            Child { born_tick: 100 },
            Life {
                born_tick: 100,
                lifespan_ticks: 90_000,
            },
        ));
        let rows = collect_agent_snapshot(&world);
        let ages = collect_agent_ages_days(&world, 1_540, 1_440);
        assert_eq!(rows.len(), ages.len());
        let by_id: std::collections::BTreeMap<_, _> = rows
            .iter()
            .zip(&ages)
            .map(|(row, age)| (row.agent_id, (row.life_stage, *age)))
            .collect();
        assert_eq!(by_id[&1], (0, -1));
        assert_eq!(by_id[&2], (2, 3));
        assert_eq!(by_id[&3], (1, 1));
    }

    #[test]
    fn colds_follow_snapshot_order() {
        let mut world = hecs::World::new();
        world.spawn((Agent { id: 1 }, Position::new(2, 3)));
        world.spawn((Agent { id: 2 }, Position::new(2, 3), Cold));
        world.spawn((
            Agent { id: 3 },
            Position::new(2, 3),
            Child { born_tick: 0 },
            Cold,
        ));
        world.spawn((Agent { id: 4 }, Position::new(2, 3), Child { born_tick: 0 }));
        let rows = collect_agent_snapshot(&world);
        let colds = collect_agent_colds(&world);
        assert_eq!(rows.len(), colds.len());
        let by_id: std::collections::BTreeMap<_, _> = rows
            .iter()
            .zip(&colds)
            .map(|(row, cold)| (row.agent_id, *cold))
            .collect();
        assert_eq!(
            by_id.into_iter().collect::<Vec<_>>(),
            vec![(1, 0), (2, 1), (3, 1), (4, 0)]
        );
    }

    #[test]
    fn wood_targets_floor_the_target_per_settlement_row() {
        let mut resources = SimResources::new(16, 16, MaterialRegistry::new());
        let mut world = hecs::World::new();
        for (sid, x) in [(1u32, 2u32), (2, 9)] {
            let entity = world.spawn((Agent { id: u64::from(sid) }, Position::new(x, 3)));
            let _ = entity;
            let mut settlement = Settlement::new_with_id(sid, 0);
            settlement.formation_tile = (x, 3);
            settlement.add_member_agent(u64::from(sid));
            resources.settlements.insert(sid, settlement);
        }
        let rows = collect_settlement_snapshot(&world, &resources.settlements);
        assert_eq!(rows.len(), 2);
        // No firewood rule: zero for every row.
        assert_eq!(
            collect_settlement_wood_targets(&rows, &resources),
            vec![0, 0]
        );
        // The v11 firewood rule (a test-local copy, triage recipe R2) without
        // seasons: 1.5 x 26 = 39; summer of a seasonal world 1.5 x 13 + 7 x 39 = 292.5 -> 292.
        let v11_firewood = sim_core::rules::FirewoodRule {
            gather_start_minute: 600,
            gather_end_minute: 1020,
            wood_radius: 18,
            haul_threshold: 2,
            max_gatherers_per_spot: 1,
            regen_permille: 500,
            base_days: 1.5,
            burn_start_minute: 1080,
            burn_end_minute: 420,
            burn_per_hour: [2, 1, 2, 3],
            cold_start_minute: 1320,
            cold_end_minute: 360,
            cold_hunger_permille: 1500,
            winter_prep_days: None,
            winter_prep_in_summer: true,
            winter_base_days: None,
            winter_regen_units_per_day: None,
            winter_deadwood_cap: None,
            night_season_from_evening: false,
            urgent_start_minute: None,
            winter_severity_permille: None,
            winter_severity_burn_per_hour: [3, 3, 3],
            winter_severity_regen_units_per_day: [6, 6, 6],
            claims_bounded_by_amount: false,
            winter_night_burn_permille: None,
            winter_night_burn_step: 0,
            winter_fall_spread: 0,
            windfall_over_snow: false,
            rain_soak_permille: 0,
            regional_wood: false,
            area_fall_floor_permille: 0,
            floor_separate: false,
        };
        resources.rules = WorldRules::baseline(0);
        resources.rules.society.firewood = Some(v11_firewood);
        resources.rules.seasons = None;
        assert_eq!(
            collect_settlement_wood_targets(&rows, &resources),
            vec![39, 39]
        );
        resources.rules = WorldRules::baseline(0);
        resources.rules.society.firewood = Some(v11_firewood);
        resources.current_tick = 7 * 1_440;
        assert_eq!(
            collect_settlement_wood_targets(&rows, &resources),
            vec![292, 292]
        );
        assert!(collect_settlement_wood_targets(&[], &resources).is_empty());
    }

    #[test]
    fn maturation_removes_child_flag_from_next_snapshot() {
        let mut world = hecs::World::new();
        let entity = world.spawn((Agent { id: 1 }, Position::new(2, 3), Child { born_tick: 0 }));
        assert_eq!(collect_agent_snapshot(&world)[0].life_stage, 1);
        world
            .remove_one::<Child>(entity)
            .expect("child fixture exists");
        assert_eq!(collect_agent_snapshot(&world)[0].life_stage, 0);
    }
}
