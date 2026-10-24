//! MUX bulk observer: sorted raw facts for alerts and lessons, never simulation commands.
//!
//! Content selection and alert thresholds belong to UI data. In particular meat
//! lots are a global FIFO, not ages attached to individual ground stacks.

mod economy;
mod marshal;
mod people;
mod places;

use super::{knowledge, neighbours, space, story};
pub use economy::SettlementFact;
pub(super) use marshal::ui_facts_to_dict;
pub use people::PersonFact;
pub use places::{StackFact, StockpileTileFact, UnreachableFact};
use sim_engine::SimResources;

/// Complete immutable input for one UI evaluation, with deterministic row orders.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiFacts {
    /// Whether colony rules are active.
    pub enabled: bool,
    /// Current simulation tick.
    pub tick: u64,
    /// Current zero-based display day (clock offset included).
    pub day: u64,
    /// Configured ticks per simulation day.
    pub ticks_per_day: u64,
    /// Display clock offset in ticks.
    pub clock_offset_ticks: u64,
    /// Current season and one-based season day, or none without seasons.
    pub season: Option<(usize, u32)>,
    /// Configured length of one season.
    pub days_per_season: u32,
    /// Spoilage lifetime in display days, or none when wildlife is off.
    pub meat_spoil_days: Option<u32>,
    /// Global FIFO lots (creation display day, count), retaining simulator order.
    pub meat_lots: Vec<(u32, u32)>,
    /// Completed teaching orders (durable simulation counter).
    pub taught_count: u64,
    /// Wolves spawned so far (durable simulation counter, not current proximity).
    pub wolves_spawned: u64,
    /// Answered or timed-out waves plus currently waiting waves.
    pub migrant_waves_arrived: u64,
    /// First calendar year has ended with no death during that year; later deaths do not revoke it.
    pub first_winter_all_alive: bool,
    /// People sorted by AgentId.
    pub people: Vec<PersonFact>,
    /// Settlements sorted by settlement id.
    pub settlements: Vec<SettlementFact>,
    /// Storage cells sorted by zone id then `(x, y)`.
    pub stockpile_tiles: Vec<StockpileTileFact>,
    /// Ground stacks in native `(x, y)` spoilage allocation order.
    pub stacks: Vec<StackFact>,
    /// Live avoidance records sorted by worker id then target tile.
    pub unreachable: Vec<UnreachableFact>,
    /// Existing technology facts, unchanged in shape.
    pub tech: knowledge::TechView,
    /// Existing knowledge waits, sorted by kind and identity.
    pub waits: Vec<knowledge::WaitRow>,
    /// Existing neighbour facts, unchanged in shape.
    pub neighbours: neighbours::NeighboursView,
    /// Existing animal facts with raw kind and state codes.
    pub animals: story::AnimalRows,
    /// Existing rooms with raw role codes.
    pub rooms: space::RoomsView,
    /// Existing pressure and hearth accounting facts.
    pub story: story::StoryView,
    /// Existing workshops for resolving waiting bill locations.
    pub workshops: Vec<super::economy::WorkshopRow>,
}

/// Read all facts without advancing systems, refreshing caches, drawing RNG or writing state.
/// Non-colony worlds return the same empty tables with current clock metadata.
pub fn collect_ui_facts(world: &hecs::World, res: &SimResources) -> UiFacts {
    let mut out = UiFacts {
        enabled: res.rules.colony.is_some(),
        tick: res.current_tick,
        day: sim_systems::runtime::resource_regen::day_index(res),
        ticks_per_day: res.ticks_per_day,
        clock_offset_ticks: res.rules.clock_offset_ticks,
        season: sim_systems::runtime::resource_regen::season_at(res),
        days_per_season: res.rules.season_days(),
        ..UiFacts::default()
    };
    if !out.enabled {
        return out;
    }
    let year_days = res.rules.calendar.days_per_year();
    out.first_winter_all_alive = out.season.is_some()
        && year_days > 0
        && out.day >= year_days
        && !res.chronicle.entries().iter().any(|entry| {
            entry.kind == sim_core::chronicle::ChronicleKind::Died
                && sim_systems::runtime::resource_regen::day_index_at(res, entry.tick) < year_days
        });
    out.people = people::collect(world, res);
    out.settlements = economy::collect(world, res);
    out.stockpile_tiles = places::storage(res);
    out.stacks = places::stacks(res);
    out.unreachable = places::unreachable(world, res);
    out.meat_spoil_days = sim_systems::runtime::wildlife::rule(res).map(|r| r.meat_spoil_days);
    out.meat_lots = res.colony.wildlife.meat_lots.clone();
    out.taught_count = res.colony.tallies.teach_done;
    out.wolves_spawned = res.colony.wildlife.tallies.wolves_spawned;
    let nb = &res.colony.neighbours;
    out.migrant_waves_arrived = nb.tallies.accepted
        + nb.tallies.declined
        + nb.tallies.unanswered
        + u64::try_from(
            nb.waves
                .iter()
                .filter(|w| {
                    matches!(
                        w.state,
                        sim_core::colony::neighbours::VisitState::Waiting { .. }
                    )
                })
                .count(),
        )
        .unwrap_or(0);
    out.tech = knowledge::collect_tech(world, res);
    out.waits = knowledge::collect_knowledge_waits(world, res);
    out.waits.sort_by_key(|r| (r.kind, r.a, r.b, r.node));
    out.neighbours = neighbours::collect_neighbours(world, res);
    out.animals = story::collect_animals(res);
    out.rooms = space::collect_rooms(res);
    out.story = story::collect_story(world, res);
    out.workshops = super::economy::collect_workshops(res);
    out
}
