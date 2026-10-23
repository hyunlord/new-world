//! AnimalSystem: deer graze, flee and are born; a Beasts pressure's wolves come to the camp and bite; meat feeds the larder and spoils.
//!
//! Priority: 171 (`ANIMAL_PRIORITY`). Interval: 3 ticks. Gate: `ColonyRule::wildlife`.
//! Reads: `Agent`, `Position`, `Drafted`, `Tool`; `SimResources.colony` (wildlife, story, rooms, zones, ground items), `nav`, `tile_grid`, `settlements`, `building_kinds`, `rules`, `world_seed`
//! Writes: `SimResources.colony` (wildlife: animals, meat lots, tallies; fights; ground items, item ledger; story via `story::end_pressure`), `settlements` (food), `spatial.agents`
//! Events: chronicle `PressureEnded` (the pack is gone); `WolfKilled` and person deaths come from `CombatSystem` (`resolve_fights`)
//!
//! M5 (ADR 0028). Animals live in `ColonyState::wildlife` (not ECS entities,
//! DECISIONS D6), at most 12 deer and 4 wolves.
//! - **Deer**: on the first step 6–12 are placed (keyed, `STREAM_WILDLIFE`) as a
//!   herd — on forest tiles at least `deer_min_hearth_distance` from the hearth
//!   that the camp can reach, within `deer_home_tiles` of a keyed centre (itself
//!   within `deer_herd_max_tiles` of the hearth when the forest allows); one
//!   more is born every `deer_respawn_days` up to that number, near the home of a
//!   living deer (the herd keeps its forest; DECISIONS D8). A deer wanders (a keyed step a third of the time, onto forest or
//!   plain at least that far from the hearth); with `deer_flee` a person
//!   within `deer_flee_tiles` makes it flee `deer_flee_speed` tiles a step away
//!   from the nearest person for `deer_flee_steps` steps, then it rests in
//!   place for `deer_rest_steps` steps (it does not flee while resting, and
//!   wanders within `deer_home_tiles` of where it was born; DECISIONS D8).
//! - **Wolves**: when a Beasts pressure is active and its pack has not come, the
//!   pack (its intensity in wolves) appears at the reachable map-edge tile
//!   nearest the centroid of the camp region's forest. Each step a wolf picks
//!   the nearest person outside a roofed room within `wolf_sight` that no other
//!   wolf of the step has picked (else the nearest; DECISIONS D8); adjacent, it
//!   bites (a fight `Animal → Person`, resolved by `CombatSystem`) and the
//!   person strikes back with what they hold (unless drafted: the draft decides);
//!   otherwise it walks a nav path toward the person, or toward the hearth. When
//!   the pack has lost half its number it retreats to its edge and leaves; after
//!   `beasts_max_days` it leaves; when no wolf of the pack is left the pressure ends.
//! - **Meat** (every 60 ticks): meat on a stockpile cell or by a hearth goes into
//!   the first settlement's food at `meat_food_units` each (eaten as foraged food
//!   is); once a day, the meat lots older than `meat_spoil_days` spoil (ground
//!   stacks in tile order; ledger consumed).
use hecs::World;
use sim_core::colony::story::PressureKind;
use sim_core::colony::wildlife::{Animal, AnimalKind, AnimalState, Fight, Fighter};
use sim_core::colony::ItemKind;
use sim_core::components::{Agent, AgentId, Position, ResourceKind};
use sim_core::rng::{keyed_below, STREAM_WILDLIFE};
use sim_core::rules::WildlifeRule;
use sim_core::TerrainType;
use sim_engine::{RuntimeSystem, SimResources};

use crate::runtime::jobs::items::chebyshev;

pub mod draft;
pub mod fight;
pub mod flee;

pub use draft::{DraftSystem, DRAFT_PRIORITY};
pub use flee::{FleeSystem, FLEE_PRIORITY};

/// Priority of [`AnimalSystem`].
pub const ANIMAL_PRIORITY: u32 = 171;

/// The wildlife rule in force (`None` outside the M5 colony).
pub fn rule(res: &SimResources) -> Option<WildlifeRule> {
    res.rules.colony.and_then(|c| c.wildlife)
}

fn terrain(res: &SimResources, t: (u32, u32)) -> Option<TerrainType> {
    crate::runtime::jobs::items::terrain(res, t)
}

/// Whether a deer may stand on `t`: forest or plain, passable, far enough from the hearth.
fn deer_ground(res: &SimResources, rule: &WildlifeRule, hearth: (u32, u32), t: (u32, u32)) -> bool {
    matches!(
        terrain(res, t),
        Some(TerrainType::Forest | TerrainType::Plain)
    ) && res.nav.passable(t)
        && chebyshev(t, hearth) >= rule.deer_min_hearth_distance
}

/// The deer's places: forest tiles far enough from the hearth that the camp reaches, in `(y, x)` order.
fn deer_places(res: &SimResources, rule: &WildlifeRule, hearth: (u32, u32)) -> Vec<(u32, u32)> {
    let region = res.nav.region_of(hearth);
    let (w, h) = (res.tile_grid.width, res.tile_grid.height);
    let mut out = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let t = (x, y);
            if terrain(res, t) == Some(TerrainType::Forest)
                && deer_ground(res, rule, hearth, t)
                && res.nav.reaches(region, t)
                && !res.colony.trees.contains_key(&t)
            {
                out.push(t);
            }
        }
    }
    out
}

fn new_animal(
    res: &mut SimResources,
    kind: AnimalKind,
    hp: u16,
    pos: (u32, u32),
    pack: Option<u32>,
) -> u32 {
    let w = &mut res.colony.wildlife;
    w.next_id = w.next_id.max(1);
    let id = w.next_id;
    w.next_id += 1;
    w.animals.insert(
        id,
        Animal {
            id,
            kind,
            hp,
            state: if kind == AnimalKind::Wolf {
                AnimalState::Hunt
            } else {
                AnimalState::Wander
            },
            pos,
            pack,
            born_tick: res.current_tick,
            path: Vec::new(),
            goal: None,
            home: (kind == AnimalKind::Deer).then_some(pos),
        },
    );
    w.version += 1;
    id
}

/// Bring a pack of `n` wolves to `at` as the pack of day `day` (tests, the probe's
/// fixtures; the AnimalSystem's own packs come with a Beasts pressure).
pub fn spawn_pack(res: &mut SimResources, at: (u32, u32), n: u32, day: u32) -> Vec<u32> {
    let Some(rule) = rule(res) else {
        return Vec::new();
    };
    res.colony.wildlife.pack_day = Some(day);
    res.colony.wildlife.pack_size = n.max(1);
    (0..n)
        .map(|_| {
            res.colony.wildlife.tallies.wolves_spawned += 1;
            new_animal(res, AnimalKind::Wolf, rule.wolf_hp, at, Some(day))
        })
        .collect()
}

/// Put one deer on `at` (tests).
pub fn spawn_deer(res: &mut SimResources, at: (u32, u32)) -> Option<u32> {
    let rule = rule(res)?;
    Some(new_animal(res, AnimalKind::Deer, rule.deer_hp, at, None))
}

/// The deer's places near `centre`: those within `deer_home_tiles` of it, else all (a herd keeps its forest).
fn near_herd(places: &[(u32, u32)], centre: Option<(u32, u32)>, reach: u32) -> Vec<(u32, u32)> {
    match centre {
        Some(c) => {
            let near: Vec<(u32, u32)> = places
                .iter()
                .copied()
                .filter(|p| chebyshev(*p, c) <= reach)
                .collect();
            if near.is_empty() {
                places.to_vec()
            } else {
                near
            }
        }
        None => places.to_vec(),
    }
}

/// Place the first deer (once) as a herd around a keyed centre, and a new one every
/// `deer_respawn_days` below the cap, near the home of a living deer (DECISIONS D8).
fn deer_births(res: &mut SimResources, rule: &WildlifeRule, hearth: (u32, u32)) {
    let day = u32::try_from(crate::runtime::resource_regen::day_index(res)).unwrap_or(u32::MAX);
    let seed = res.world_seed;
    let reach = rule.deer_home_tiles;
    if !res.colony.wildlife.seeded {
        res.colony.wildlife.seeded = true;
        res.colony.wildlife.last_deer_day = day;
        let (lo, hi) = rule.deer;
        let n = lo + keyed_below(seed, STREAM_WILDLIFE, 0, 0, 0, 0, u64::from(hi - lo + 1)) as u32;
        res.colony.wildlife.deer_cap = n;
        let all = deer_places(res, rule, hearth);
        if all.is_empty() {
            return;
        }
        // The centre: a place within `deer_herd_max_tiles` of the hearth when there is one.
        let close: Vec<(u32, u32)> = all
            .iter()
            .copied()
            .filter(|p| chebyshev(*p, hearth) <= rule.deer_herd_max_tiles)
            .collect();
        let pool = if close.is_empty() { &all } else { &close };
        let centre =
            pool[keyed_below(seed, STREAM_WILDLIFE, 0, 0, 0x200, 0, pool.len() as u64) as usize];
        let places = near_herd(&all, Some(centre), reach);
        for i in 0..n {
            let at = places
                [keyed_below(seed, STREAM_WILDLIFE, 0, 0, 1 + i, 0, places.len() as u64) as usize];
            new_animal(res, AnimalKind::Deer, rule.deer_hp, at, None);
        }
        return;
    }
    let cap = res.colony.wildlife.deer_cap;
    let deer = res.colony.wildlife.deer().count() as u32;
    if deer < cap && day >= res.colony.wildlife.last_deer_day + rule.deer_respawn_days {
        res.colony.wildlife.last_deer_day = day;
        let all = deer_places(res, rule, hearth);
        if !all.is_empty() {
            let homes: Vec<(u32, u32)> =
                res.colony.wildlife.deer().filter_map(|a| a.home).collect();
            let centre = (!homes.is_empty()).then(|| {
                homes[keyed_below(
                    seed,
                    STREAM_WILDLIFE,
                    0,
                    u64::from(day),
                    0x101,
                    0,
                    homes.len() as u64,
                ) as usize]
            });
            let places = near_herd(&all, centre, reach);
            let at = places[keyed_below(
                seed,
                STREAM_WILDLIFE,
                0,
                u64::from(day),
                0x100,
                0,
                places.len() as u64,
            ) as usize];
            new_animal(res, AnimalKind::Deer, rule.deer_hp, at, None);
            res.colony.wildlife.tallies.deer_born += 1;
        }
    }
}

/// The tile one step from `from` toward (`toward` = true) or away from `to`, among
/// the passable 8-neighbours `ok` accepts (ties `(y, x)`); `None` if none is better.
pub(crate) fn step_rel(
    res: &SimResources,
    from: (u32, u32),
    to: (u32, u32),
    toward: bool,
    ok: impl Fn((u32, u32)) -> bool,
) -> Option<(u32, u32)> {
    let (w, h) = (res.tile_grid.width, res.tile_grid.height);
    let here = chebyshev(from, to);
    let mut best: Option<(i64, u32, u32)> = None;
    for dy in -1i64..=1 {
        for dx in -1i64..=1 {
            let (x, y) = (i64::from(from.0) + dx, i64::from(from.1) + dy);
            if (dx, dy) == (0, 0) || x < 0 || y < 0 || x >= i64::from(w) || y >= i64::from(h) {
                continue;
            }
            let t = (x as u32, y as u32);
            if !res.nav.can_step(from, t) || !ok(t) {
                continue;
            }
            let d = i64::from(chebyshev(t, to));
            let score = if toward { d } else { -d };
            if (toward && d >= i64::from(here)) || (!toward && -score <= i64::from(here)) {
                continue;
            }
            if best.is_none_or(|b| (score, t.1, t.0) < b) {
                best = Some((score, t.1, t.0));
            }
        }
    }
    best.map(|(_, y, x)| (x, y))
}

/// People's tiles, ids and whether each stands in a roofed room.
fn people(world: &World, res: &SimResources) -> Vec<(AgentId, (u32, u32), bool)> {
    let mut out: Vec<(AgentId, (u32, u32), bool)> = world
        .query::<(&Agent, &Position)>()
        .iter()
        .map(|(_, (a, p))| {
            (
                a.id,
                (p.x, p.y),
                crate::runtime::rooms::roofed(res, (p.x, p.y)),
            )
        })
        .collect();
    out.sort_by_key(|row| row.0);
    out
}

/// One step of every deer (wander, flee, rest).
fn deer_steps(
    res: &mut SimResources,
    rule: &WildlifeRule,
    hearth: (u32, u32),
    folk: &[(AgentId, (u32, u32), bool)],
) {
    let seed = res.world_seed;
    let tick = res.current_tick;
    let ids: Vec<u32> = res.colony.wildlife.deer().map(|a| a.id).collect();
    for id in ids {
        let Some(a) = res.colony.wildlife.animals.get(&id).cloned() else {
            continue;
        };
        let near = folk
            .iter()
            .filter(|(_, p, _)| chebyshev(*p, a.pos) <= rule.deer_flee_tiles)
            .min_by_key(|(pid, p, _)| (chebyshev(*p, a.pos), *pid))
            .map(|(_, p, _)| *p);
        let ok = |t: (u32, u32)| deer_ground(res, rule, hearth, t);
        let (mut pos, mut state) = (a.pos, a.state);
        match (a.state, near) {
            (AnimalState::Wander, Some(_)) if rule.deer_flee => {
                state = AnimalState::Flee {
                    steps: rule.deer_flee_steps,
                };
            }
            (AnimalState::Rest { steps }, _) => {
                state = if steps <= 1 {
                    AnimalState::Wander
                } else {
                    AnimalState::Rest { steps: steps - 1 }
                };
            }
            _ => {}
        }
        match state {
            AnimalState::Flee { steps } => {
                if let Some(from) = near {
                    for _ in 0..rule.deer_flee_speed {
                        match step_rel(res, pos, from, false, ok) {
                            Some(next) => pos = next,
                            None => break,
                        }
                    }
                }
                state = if steps <= 1 || near.is_none() {
                    AnimalState::Rest {
                        steps: rule.deer_rest_steps,
                    }
                } else {
                    AnimalState::Flee { steps: steps - 1 }
                };
            }
            // A resting deer grazes in place; a wandering one keeps near its home ground.
            AnimalState::Wander => {
                let home = a.home.unwrap_or(a.pos);
                let ok = |t: (u32, u32)| {
                    ok(t) && chebyshev(t, home) <= rule.deer_home_tiles.max(chebyshev(pos, home))
                };
                if keyed_below(seed, STREAM_WILDLIFE, u64::from(id), tick, 1, 0, 3) == 0 {
                    let dx =
                        keyed_below(seed, STREAM_WILDLIFE, u64::from(id), tick, 2, 0, 3) as i64 - 1;
                    let dy =
                        keyed_below(seed, STREAM_WILDLIFE, u64::from(id), tick, 3, 0, 3) as i64 - 1;
                    let (x, y) = (i64::from(pos.0) + dx, i64::from(pos.1) + dy);
                    if x >= 0
                        && y >= 0
                        && x < i64::from(res.tile_grid.width)
                        && y < i64::from(res.tile_grid.height)
                    {
                        let t = (x as u32, y as u32);
                        if t != pos && res.nav.can_step(pos, t) && ok(t) {
                            pos = t;
                        }
                    }
                }
            }
            _ => {}
        }
        if let Some(a) = res.colony.wildlife.animals.get_mut(&id) {
            if a.pos != pos || a.state != state {
                res.colony.wildlife.version += 1;
            }
            a.pos = pos;
            a.state = state;
        }
    }
}

/// The reachable map-edge tile nearest the centroid of the camp region's forest (the wolves' door).
pub fn wolf_edge(res: &SimResources, hearth: (u32, u32)) -> Option<(u32, u32)> {
    let region = res.nav.region_of(hearth);
    let (w, h) = (res.tile_grid.width, res.tile_grid.height);
    let (mut sx, mut sy, mut n) = (0u64, 0u64, 0u64);
    for y in 0..h {
        for x in 0..w {
            if terrain(res, (x, y)) == Some(TerrainType::Forest) && res.nav.reaches(region, (x, y))
            {
                sx += u64::from(x);
                sy += u64::from(y);
                n += 1;
            }
        }
    }
    let centre = if n == 0 {
        hearth
    } else {
        ((sx / n) as u32, (sy / n) as u32)
    };
    let edge = (0..w)
        .flat_map(|x| [(x, 0), (x, h.saturating_sub(1))])
        .chain((0..h).flat_map(|y| [(0, y), (w.saturating_sub(1), y)]))
        .filter(|t| res.nav.passable(*t) && res.nav.reaches(region, *t));
    edge.min_by_key(|t| (chebyshev(*t, centre), t.1, t.0))
}

/// Move wolf `id` one step along a nav path toward `goal` (replanned when the goal moved).
fn wolf_walk(res: &mut SimResources, id: u32, goal: (u32, u32)) {
    let Some(a) = res.colony.wildlife.animals.get(&id).cloned() else {
        return;
    };
    let stale = a.goal.is_none_or(|g| chebyshev(g, goal) > 2) || a.path.is_empty();
    let mut path = a.path.clone();
    if stale {
        path = res
            .nav
            .path(a.pos, goal)
            .map_or_else(Vec::new, |p| p.to_vec());
    }
    let next = if path.is_empty() {
        None
    } else {
        Some(path.remove(0))
    };
    if let Some(a) = res.colony.wildlife.animals.get_mut(&id) {
        if stale {
            a.goal = Some(goal);
        }
        if let Some(next) = next.filter(|n| res.nav.can_step(a.pos, *n)) {
            a.pos = next;
        } else {
            path.clear();
        }
        a.path = path;
    }
    res.colony.wildlife.version += 1;
}

/// One step of the pack: arrive, retreat, leave, hunt, bite.
fn wolf_steps(
    world: &World,
    res: &mut SimResources,
    rule: &WildlifeRule,
    hearth: (u32, u32),
    folk: &[(AgentId, (u32, u32), bool)],
) {
    let beasts = res.colony.story.active_of(PressureKind::Beasts).copied();
    // The pack arrives with its pressure.
    if let Some(p) = beasts {
        if res.colony.wildlife.pack_day != Some(p.start_day) {
            res.colony.wildlife.pack_day = Some(p.start_day);
            res.colony.wildlife.pack_size = p.intensity.max(1);
            res.colony.wildlife.pack_edge = wolf_edge(res, hearth);
            if let Some(edge) = res.colony.wildlife.pack_edge {
                for i in 0..p.intensity.max(1) {
                    let at = step_rel(res, edge, hearth, true, |_| true)
                        .filter(|_| i % 2 == 1)
                        .unwrap_or(edge);
                    new_animal(res, AnimalKind::Wolf, rule.wolf_hp, at, Some(p.start_day));
                    res.colony.wildlife.tallies.wolves_spawned += 1;
                }
            }
        }
    }
    let wolves: Vec<u32> = res.colony.wildlife.wolves().map(|a| a.id).collect();
    if wolves.is_empty() {
        if beasts.is_some() && res.colony.wildlife.pack_day == beasts.map(|p| p.start_day) {
            crate::runtime::story::end_pressure(res, PressureKind::Beasts);
        }
        return;
    }
    let tick = res.current_tick;
    let day_ticks = res.rules.calendar.days_to_ticks(1);
    let size = res.colony.wildlife.pack_size.max(1);
    let alive = wolves.len() as u32;
    let routed = alive * 2 <= size;
    // The pack's edge, found when it came; searched again (the whole map) only when it can no longer be reached.
    let region = res.nav.region_of(hearth);
    let edge = match res.colony.wildlife.pack_edge {
        Some(e) if res.nav.passable(e) && res.nav.reaches(region, e) => Some(e),
        _ => {
            let e = wolf_edge(res, hearth);
            res.colony.wildlife.pack_edge = e;
            e
        }
    };
    let story_days = crate::runtime::story::rule(res).map_or(2, |s| s.beasts_max_days);
    // The pack spreads over its prey: a wolf takes the nearest person no other wolf has this step.
    let mut taken: std::collections::BTreeSet<AgentId> = std::collections::BTreeSet::new();
    for id in wolves {
        let Some(a) = res.colony.wildlife.animals.get(&id).cloned() else {
            continue;
        };
        let overstayed = tick.saturating_sub(a.born_tick) >= u64::from(story_days) * day_ticks;
        if overstayed
            || (a.state == AnimalState::Retreat && edge.is_some_and(|e| chebyshev(e, a.pos) <= 1))
        {
            leave(res, id);
            continue;
        }
        if routed || a.state == AnimalState::Retreat {
            fight::drop_fights_of(res, Fighter::Animal(id));
            if let Some(w) = res.colony.wildlife.animals.get_mut(&id) {
                w.state = AnimalState::Retreat;
            }
            if let Some(e) = edge {
                wolf_walk(res, id, e);
            }
            continue;
        }
        let seen = |(_, p, roofed): &&(AgentId, (u32, u32), bool)| {
            !roofed && chebyshev(*p, a.pos) <= rule.wolf_sight
        };
        let target = folk
            .iter()
            .filter(seen)
            .filter(|(pid, ..)| !taken.contains(pid))
            .min_by_key(|(pid, p, _)| (chebyshev(*p, a.pos), *pid))
            .or_else(|| {
                folk.iter()
                    .filter(seen)
                    .min_by_key(|(pid, p, _)| (chebyshev(*p, a.pos), *pid))
            })
            .copied();
        if let Some((pid, ..)) = target {
            taken.insert(pid);
        }
        match target {
            Some((pid, p, _)) if chebyshev(p, a.pos) <= 1 => {
                if let Some(w) = res.colony.wildlife.animals.get_mut(&id) {
                    w.state = AnimalState::Attack { target: pid };
                }
                let key = (Fighter::Animal(id), Fighter::Person(pid));
                if !res.colony.fights.contains_key(&key) {
                    fight::drop_fights_of_attacker(res, Fighter::Animal(id));
                    res.colony.fights.insert(
                        key,
                        Fight {
                            progress: 0,
                            ticks: rule.hit_ticks,
                            damage: rule.wolf_damage,
                            hits_left: 0,
                            quarrel: None,
                        },
                    );
                }
                // The bitten strike back with what they hold (a drafted person follows orders).
                if let Some(e) = entity_of(world, pid) {
                    let back = (Fighter::Person(pid), Fighter::Animal(id));
                    let busy = res
                        .colony
                        .fights
                        .keys()
                        .any(|(att, _)| *att == Fighter::Person(pid));
                    if !busy && world.get::<&sim_core::colony::people::Drafted>(e).is_err() {
                        let damage = crate::runtime::jobs::hunt::defence_damage(world, rule, e);
                        res.colony.fights.insert(
                            back,
                            Fight {
                                progress: 0,
                                ticks: rule.hit_ticks,
                                damage,
                                hits_left: 0,
                                quarrel: None,
                            },
                        );
                    }
                }
            }
            Some((_, p, _)) => {
                fight::drop_fights_of_attacker(res, Fighter::Animal(id));
                if let Some(w) = res.colony.wildlife.animals.get_mut(&id) {
                    w.state = AnimalState::Hunt;
                }
                wolf_walk(res, id, p);
                // M5.5 (calibration r1): a wolf outruns a person it chases within the flight distance.
                let chasing = chebyshev(p, a.pos) <= rule.flee_tiles;
                for _ in 1..rule.wolf_chase_tiles.max(1) {
                    let at = res.colony.wildlife.animals.get(&id).map(|w| w.pos);
                    if !chasing || at.is_none_or(|at| chebyshev(at, p) <= 1) {
                        break;
                    }
                    wolf_walk(res, id, p);
                }
            }
            None => {
                fight::drop_fights_of_attacker(res, Fighter::Animal(id));
                wolf_walk(res, id, hearth);
            }
        }
    }
}

fn entity_of(world: &World, id: AgentId) -> Option<hecs::Entity> {
    world
        .query::<&Agent>()
        .iter()
        .find(|(_, a)| a.id == id)
        .map(|(e, _)| e)
}

/// A wolf leaves the map.
fn leave(res: &mut SimResources, id: u32) {
    fight::drop_fights_of(res, Fighter::Animal(id));
    if res.colony.wildlife.animals.remove(&id).is_some() {
        res.colony.wildlife.tallies.wolves_left += 1;
        res.colony.wildlife.version += 1;
    }
}

/// Meat on a stockpile cell or by a hearth goes into the first settlement's food.
fn feed_larder(res: &mut SimResources, rule: &WildlifeRule) {
    let hearths = crate::runtime::jobs::hearth::hearth_tiles(res);
    let reach = crate::runtime::jobs::hearth::HEARTH_PILE_REACH;
    let tiles: Vec<((u32, u32), u16)> = res
        .colony
        .ground_items
        .iter()
        .filter(|(t, s)| {
            s.kind == ItemKind::Meat
                && !res
                    .colony
                    .reservations
                    .contains_key(&sim_core::colony::ReserveKey::Stack(**t))
                && (res.colony.zone_of_tile.contains_key(*t)
                    || hearths.iter().any(|h| chebyshev(**t, *h) <= reach))
        })
        .map(|(t, s)| (*t, s.count))
        .collect();
    let mut ids: Vec<_> = res.settlements.keys().copied().collect();
    ids.sort_unstable();
    let Some(sid) = ids.first().copied() else {
        return;
    };
    for (tile, n) in tiles {
        let (taken, _) = crate::runtime::jobs::items::take_items(res, tile, ItemKind::Meat, n);
        if taken == 0 {
            continue;
        }
        res.colony
            .item_ledger
            .consume(ItemKind::Meat, u64::from(taken));
        take_from_lots(res, u32::from(taken));
        res.colony.wildlife.tallies.meat_eaten += u64::from(taken);
        if let Some(s) = res.settlements.get_mut(&sid) {
            s.store(ResourceKind::Food, u32::from(taken) * rule.meat_food_units);
        }
        res.colony.touch(false);
    }
}

/// Remove `n` pieces from the meat lots, oldest first.
fn take_from_lots(res: &mut SimResources, mut n: u32) {
    let lots = &mut res.colony.wildlife.meat_lots;
    while n > 0 && !lots.is_empty() {
        let take = lots[0].1.min(n);
        lots[0].1 -= take;
        n -= take;
        if lots[0].1 == 0 {
            lots.remove(0);
        }
    }
}

/// Spoil the meat lots older than `meat_spoil_days`: ground stacks of meat in tile order.
fn spoil(res: &mut SimResources, rule: &WildlifeRule) {
    let day = u32::try_from(crate::runtime::resource_regen::day_index(res)).unwrap_or(u32::MAX);
    let mut due: u32 = res
        .colony
        .wildlife
        .meat_lots
        .iter()
        .filter(|(d, _)| d + rule.meat_spoil_days <= day)
        .map(|(_, n)| *n)
        .sum();
    if due == 0 {
        return;
    }
    let stacks: Vec<((u32, u32), u16)> = res
        .colony
        .ground_items
        .iter()
        .filter(|(t, s)| {
            s.kind == ItemKind::Meat
                && !res
                    .colony
                    .reservations
                    .contains_key(&sim_core::colony::ReserveKey::Stack(**t))
        })
        .map(|(t, s)| (*t, s.count))
        .collect();
    let mut spoiled = 0u32;
    for (tile, n) in stacks {
        if due == 0 {
            break;
        }
        let want = (u32::from(n)).min(due) as u16;
        let (taken, _) = crate::runtime::jobs::items::take_items(res, tile, ItemKind::Meat, want);
        due -= u32::from(taken);
        spoiled += u32::from(taken);
        if !res.colony.zone_of_tile.contains_key(&tile) {
            res.colony.wildlife.tallies.meat_spoiled_afield += u64::from(taken);
        }
    }
    if spoiled > 0 {
        res.colony
            .item_ledger
            .consume(ItemKind::Meat, u64::from(spoiled));
        take_from_lots(res, spoiled);
        res.colony.wildlife.tallies.meat_spoiled += u64::from(spoiled);
        res.colony.touch(false);
    }
}

/// See the module header.
#[derive(Debug, Default)]
pub struct AnimalSystem;

impl RuntimeSystem for AnimalSystem {
    fn name(&self) -> &str {
        "AnimalSystem"
    }

    fn priority(&self) -> u32 {
        ANIMAL_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        3
    }

    fn tick(&mut self, world: &mut World, res: &mut SimResources) {
        let Some(rule) = rule(res) else {
            return;
        };
        let Some(hearth) = crate::worldgen::colony::hearth(res) else {
            return;
        };
        crate::runtime::nav::ensure(res);
        let tick = res.current_tick;
        if tick.is_multiple_of(res.ticks_per_day.max(1)) || !res.colony.wildlife.seeded {
            deer_births(res, &rule, hearth);
        }
        let folk = people(world, res);
        deer_steps(res, &rule, hearth, &folk);
        wolf_steps(world, res, &rule, hearth, &folk);
        if tick.is_multiple_of(60) {
            feed_larder(res, &rule);
            // M5.5: a carcass tile leaves the list once nothing of the kill (items tagged `carcass` in values.ron) lies
            // there off the stockpiles.
            if !res.colony.wildlife.carcasses.is_empty() {
                let colony = &res.colony;
                let keep: std::collections::BTreeSet<(u32, u32)> = colony
                    .wildlife
                    .carcasses
                    .iter()
                    .copied()
                    .filter(|t| {
                        !colony.zone_of_tile.contains_key(t)
                            && colony.ground_items.get(t).is_some_and(|s| {
                                sim_core::colony::values::has_tag(s.kind, "carcass")
                            })
                    })
                    .collect();
                res.colony.wildlife.carcasses = keep;
            }
        }
        if tick.is_multiple_of(res.ticks_per_day.max(1)) {
            spoil(res, &rule);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata() {
        let s = AnimalSystem;
        assert_eq!(
            (s.name(), s.priority(), s.tick_interval()),
            ("AnimalSystem", 171, 3)
        );
    }
}
