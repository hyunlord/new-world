//! M5.5 (ADR 0031): a migrant wave joins the colony — the player's `AcceptMigrants` turns each of its people into a
//! colonist; `DeclineMigrants` sends it away.
//!
//! A joining migrant is spawned as a founder is (`worldgen::camp`), on the
//! free tiles nearest the wave's tile: needs a little raised (they walked),
//! `Lineage` with the wave's keyed key, hardiness, temperament, sex and an
//! empty `Kin` (households form through the normal pairing), a `Life` whose
//! age is the founders' keyed age at the day they join, `Knowledge` from the
//! spec (source `Migrant`, floored as apprenticed), a hide wrap worn when the
//! spec says so (ledger created, counted as held), default work priorities
//! (no component: the defaults), and membership of the colony's settlement
//! (beds come from `BedAssignmentSystem` at 20:00). Chronicle
//! `MigrantsJoined`.
use hecs::{Entity, World};
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::knowledge::{day_of, Knowledge, KnowledgeEntry, Source};
use sim_core::colony::neighbours::{VisitState, WaveAnswer};
use sim_core::colony::people::{Apparel, Top};
use sim_core::colony::ItemKind;
use sim_core::components::{
    Agent, AgentId, AgentState, BodyHealth, Hunger, Inventory, Kin, Lineage, Memory, Position,
    Sleep, Social, Thirst,
};
use sim_engine::SimResources;

use crate::runtime::agent::MovementRng;
use crate::runtime::jobs::items::chebyshev;

/// Hunger and thirst a migrant arrives with (they walked for days; M5.5 DECISIONS).
const ARRIVAL_NEED: f64 = 20.0;

/// The `n` free tiles nearest `at` where people may stand (passable, no building), `(distance, y, x)` order, in
/// `at`'s nav region.
fn free_tiles(res: &SimResources, at: (u32, u32), n: usize) -> Vec<(u32, u32)> {
    let region = res.nav.region_of(at);
    let mut out = Vec::new();
    for d in 0..16u32 {
        let mut ring: Vec<(u32, u32)> = Vec::new();
        for y in at.1.saturating_sub(d)..=at.1 + d {
            for x in at.0.saturating_sub(d)..=at.0 + d {
                let t = (x, y);
                if chebyshev(t, at) == d
                    && x < res.tile_grid.width
                    && y < res.tile_grid.height
                    && res.nav.passable(t)
                    && res.nav.reaches(region, t)
                    && !crate::runtime::jobs::items::under_building(res, t)
                {
                    ring.push(t);
                }
            }
        }
        ring.sort_by_key(|t| (t.1, t.0));
        out.extend(ring);
        if out.len() >= n {
            break;
        }
    }
    out.truncate(n);
    out
}

/// Spawn one migrant of `spec` on `tile` as a colonist; returns its id and entity.
fn spawn(
    world: &mut World,
    res: &mut SimResources,
    spec: &sim_core::colony::neighbours::MigrantSpec,
    tile: (u32, u32),
) -> (AgentId, Entity) {
    let seed = res.world_seed;
    let now = res.current_tick;
    // The id reserved when the wave set out (a spec without one takes the next).
    let id = if spec.agent != 0 {
        spec.agent
    } else {
        res.issue_agent_id()
    };
    let entity = world.spawn((Position::new(tile.0, tile.1), Agent { id }));
    let rng = MovementRng::new(spec.key ^ 0x4D49_4752_414E_5453);
    let _ = world.insert(
        entity,
        (
            rng,
            AgentState::Idle,
            Hunger::new(ARRIVAL_NEED as f32, 0.05),
            Thirst::new(ARRIVAL_NEED, 0.08),
            Sleep::new(10.0, 0.03),
            Social::new(0.0, 0.04),
            Memory::new(),
            BodyHealth::new(),
            Inventory::default(),
        ),
    );
    let day = day_of(now);
    let entries: Vec<KnowledgeEntry> = spec
        .knowledge
        .iter()
        .map(|&(node, p)| KnowledgeEntry::new(node, Source::Migrant, p, day))
        .collect();
    let _ = world.insert_one(entity, Knowledge::from_entries(entries));
    let lineage = Lineage {
        key: spec.key,
        births: 0,
    };
    let _ = world.insert_one(entity, lineage);
    if res
        .rules
        .society
        .exposure
        .is_some_and(|rule| rule.use_hardiness)
    {
        let _ = world.insert_one(
            entity,
            sim_core::components::Hardiness::draw(seed, spec.key),
        );
    }
    if let Some(rule) = res.rules.agent.temperament {
        let temperament = sim_core::components::Temperament::founder(seed, spec.key);
        let _ = world.insert_one(entity, temperament);
        if rule.effects {
            if let Ok(mut social) = world.get::<&mut Social>(entity) {
                social.growth_rate = rule.social_rate(social.growth_rate, &temperament);
            }
        }
    }
    if res.rules.society.family.is_some() {
        let _ = world.insert_one(
            entity,
            sim_core::components::Sex::from_lineage(seed, spec.key),
        );
        let _ = world.insert_one(entity, Kin::default());
    }
    if let Some(rule) = res.rules.society.aging {
        // The founders' keyed age, counted back from today (a founder is born before tick 0).
        let life = crate::runtime::agent::aging::keyed_founder_life(seed, spec.key, 0, &rule);
        let _ = world.insert_one(
            entity,
            sim_core::components::Life {
                born_tick: life.born_tick.saturating_add(now as i64),
                lifespan_ticks: life.lifespan_ticks,
            },
        );
    }
    if let Some((kind, top)) = migrant_top().filter(|_| spec.wrap) {
        res.colony.item_ledger.create(kind, 1);
        let since_day =
            crate::runtime::resource_regen::day_index(res).min(u64::from(u16::MAX)) as u16;
        let _ = world.insert_one(entity, Apparel { top, since_day });
    }
    crate::runtime::chronicle_log::ensure_entity(world, res, id, entity);
    (id, entity)
}

/// The top a migrant may arrive wearing: `migrant_top` of `data/colony/neighbours.ron` (an apparel item), if any.
pub fn migrant_top() -> Option<(ItemKind, Top)> {
    sim_core::rules::neighbour_data()
        .and_then(|d| d.migrant_top.as_deref())
        .and_then(ItemKind::from_name)
        .and_then(|kind| Top::of_item(kind).map(|top| (kind, top)))
}

/// Accept wave `wave` (its number): every person joins; returns how many, or `None` when no wave of that number
/// waits at the hearth or is on its way. A wave still on its way keeps walking with the answer noted and joins when it
/// reaches the hearth (`VisitorSystem`; review F3: never at the map edge).
pub fn accept(world: &mut World, res: &mut SimResources, wave: u32) -> Option<u32> {
    let index = res.colony.neighbours.waves.iter().position(|w| {
        w.id == wave && matches!(w.state, VisitState::Waiting { .. } | VisitState::Arriving)
    })?;
    let w = res.colony.neighbours.waves[index].clone();
    if w.state == VisitState::Arriving {
        let nb = &mut res.colony.neighbours;
        nb.waves[index].answer = Some(WaveAnswer::Accepted);
        nb.version += 1;
        return Some(w.people.len() as u32);
    }
    let tiles = free_tiles(res, w.pos, w.people.len());
    let mut ids = Vec::new();
    for (i, spec) in w.people.iter().enumerate() {
        let tile = tiles.get(i).copied().unwrap_or(w.pos);
        let (id, _) = spawn(world, res, spec, tile);
        ids.push(id);
    }
    // Members of the colony's settlement (the lowest id: the colony has one).
    let mut sids: Vec<_> = res.settlements.keys().copied().collect();
    sids.sort_unstable();
    if let Some(s) = sids.first().and_then(|sid| res.settlements.get_mut(sid)) {
        for id in &ids {
            s.add_member_agent(*id);
        }
        s.population_stats.current = s.member_agents.len() as u32;
    }
    let count = ids.len() as u32;
    let day = super::today(res);
    let nb = &mut res.colony.neighbours;
    let wv = &mut nb.waves[index];
    wv.state = VisitState::Gone;
    wv.answer = Some(WaveAnswer::Accepted);
    wv.joined = ids.clone();
    nb.tallies.accepted += 1;
    nb.tallies.migrants_joined += u64::from(count);
    if let Some(n) = nb.list.iter_mut().find(|n| n.id == w.neighbour) {
        n.last_contact_day = Some(day);
    }
    nb.version += 1;
    res.colony.wildlife.version += 1;
    let mut entry = ChronicleEntry::new(res.current_tick, ChronicleKind::MigrantsJoined)
        .cause(u32::from(w.neighbour))
        .value((i64::from(w.id) << 8) | i64::from(count));
    if let Some(first) = ids.first() {
        entry = entry.subject(*first);
    }
    res.chronicle.push(entry);
    Some(count)
}

/// Decline wave `wave`: it walks away and its neighbour's attitude drops; `false` when no such wave waits or arrives.
pub fn decline(res: &mut SimResources, wave: u32) -> bool {
    let Some(index) = res.colony.neighbours.waves.iter().position(|w| {
        w.id == wave && matches!(w.state, VisitState::Waiting { .. } | VisitState::Arriving)
    }) else {
        return false;
    };
    super::visitors::send_away(res, index, 0);
    true
}
