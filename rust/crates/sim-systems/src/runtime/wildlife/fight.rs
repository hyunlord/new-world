//! M5 fights (ADR 0028): one side's attack in `ColonyState::fights`, resolved by `CombatSystem` with the V7 pair
//! model — a pair, a progress count, a resolution that applies damage and books deaths.
//!
//! Each fight `(attacker, defender)` gains one progress a tick; at its `ticks`
//! one hit resolves: the defender loses `damage` (a person's `BodyHealth`, an
//! animal's hp; a person's blow uses the weapon held at that hit, a quarrel's the fight's fixed damage), a spear in the attacker's hand wears one use, a wolf's bite is
//! counted and gives the person's "why?" its pressure (`PressureCause`, wound).
//! A fight ends when either side is gone or more than one tile apart, after its
//! `hits_left` (a quarrel's fight: one hit), or when the defender falls — a
//! person dies through `despawn_agent` (combat), a wolf leaves a wolf hide
//! (`wolf_hides`) and the chronicle `WolfKilled`. Fights resolve in key order.
use hecs::{Entity, World};
use sim_core::causal::event::DeathReason;
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::story::PressureKind;
use sim_core::colony::wildlife::{AnimalKind, Fighter};
use sim_core::colony::ItemKind;
use sim_core::components::{Agent, AgentId, BodyHealth, Position};
use sim_engine::SimResources;

/// Remove every fight `who` is part of.
pub fn drop_fights_of(res: &mut SimResources, who: Fighter) {
    res.colony.fights.retain(|(a, d), _| *a != who && *d != who);
}

/// Remove every fight `who` attacks in.
pub fn drop_fights_of_attacker(res: &mut SimResources, who: Fighter) {
    res.colony.fights.retain(|(a, _), _| *a != who);
}

fn person(world: &World, id: AgentId) -> Option<(Entity, (u32, u32))> {
    world
        .query::<(&Agent, &Position)>()
        .iter()
        .find(|(_, (a, _))| a.id == id)
        .map(|(e, (_, p))| (e, (p.x, p.y)))
}

fn where_is(world: &World, res: &SimResources, f: Fighter) -> Option<(u32, u32)> {
    match f {
        Fighter::Person(id) => person(world, id).map(|(_, p)| p),
        Fighter::Animal(id) => res.colony.wildlife.animals.get(&id).map(|a| a.pos),
    }
}

/// Advance and resolve every fight (called by `CombatSystem` after its V7 pairs).
pub fn resolve_fights(world: &mut World, res: &mut SimResources) {
    if res.colony.fights.is_empty() {
        return;
    }
    let Some(rule) = super::rule(res) else {
        res.colony.fights.clear();
        return;
    };
    let keys: Vec<(Fighter, Fighter)> = res.colony.fights.keys().copied().collect();
    for key in keys {
        let Some(mut fight) = res.colony.fights.get(&key).copied() else {
            continue;
        };
        let (attacker, defender) = key;
        let (Some(a), Some(d)) = (
            where_is(world, res, attacker),
            where_is(world, res, defender),
        ) else {
            res.colony.fights.remove(&key);
            continue;
        };
        if sim_core::nav::chebyshev(a, d) > 1 {
            res.colony.fights.remove(&key);
            continue;
        }
        fight.progress += 1;
        if fight.progress < fight.ticks {
            res.colony.fights.insert(key, fight);
            continue;
        }
        fight.progress = 0;
        let ended = fight.hits_left == 1;
        fight.hits_left = fight.hits_left.saturating_sub(1);
        if ended {
            res.colony.fights.remove(&key);
        } else {
            res.colony.fights.insert(key, fight);
        }
        // A person's blow is the weapon held now (a spear that broke mid-fight no longer hits
        // for 12; review m2); a quarrel's fight keeps its fixed damage. The spear wears one use a hit.
        let mut damage = fight.damage;
        if let Fighter::Person(id) = attacker {
            if let Some((e, _)) = person(world, id) {
                if fight.quarrel.is_none() {
                    damage = crate::runtime::jobs::hunt::weapon_damage(world, &rule, e);
                }
                crate::runtime::jobs::hunt::wear_tool(
                    world,
                    res,
                    id,
                    e,
                    ItemKind::Spear,
                    rule.spear_uses,
                );
            }
        }
        match defender {
            Fighter::Person(id) => hit_person(world, res, attacker, id, damage, d),
            Fighter::Animal(id) => hit_animal(world, res, &rule, attacker, id, damage),
        }
    }
}

fn hit_person(
    world: &mut World,
    res: &mut SimResources,
    attacker: Fighter,
    id: AgentId,
    damage: u16,
    pos: (u32, u32),
) {
    let Some((e, _)) = person(world, id) else {
        return;
    };
    if let Fighter::Animal(_) = attacker {
        let t = &mut res.colony.wildlife.tallies;
        t.bites += 1;
        t.bite_damage += u64::from(damage);
        if !res
            .colony
            .story
            .touched
            .contains(&(PressureKind::Beasts.code(), id, 1))
        {
            res.colony.wildlife.tallies.bitten += 1;
        }
        crate::runtime::story::note_cause(res, id, pos, PressureKind::Beasts, 1);
    }
    let dead = match world.get::<&mut BodyHealth>(e) {
        Ok(mut h) => {
            h.apply_damage(f64::from(damage));
            h.is_dead()
        }
        Err(_) => false,
    };
    if dead {
        drop_fights_of(res, Fighter::Person(id));
        crate::runtime::survival::despawn_agent(
            world,
            res,
            e,
            id,
            pos,
            DeathReason::Combat,
            res.current_tick,
        );
    }
}

fn hit_animal(
    world: &mut World,
    res: &mut SimResources,
    rule: &sim_core::rules::WildlifeRule,
    attacker: Fighter,
    id: u32,
    damage: u16,
) {
    let Some(a) = res.colony.wildlife.animals.get_mut(&id) else {
        return;
    };
    a.hp = a.hp.saturating_sub(damage);
    res.colony.wildlife.version += 1;
    if a.hp > 0 {
        return;
    }
    let (kind, pos) = (a.kind, a.pos);
    res.colony.wildlife.animals.remove(&id);
    res.colony.wildlife.marks.remove(&id);
    drop_fights_of(res, Fighter::Animal(id));
    let killer = match attacker {
        Fighter::Person(p) => Some(p),
        Fighter::Animal(_) => None,
    };
    match kind {
        AnimalKind::Wolf => {
            res.colony.wildlife.tallies.wolves_killed += 1;
            if rule.wolf_hides {
                res.colony.item_ledger.create(ItemKind::WolfHide, 1);
                crate::runtime::jobs::items::put_items(
                    res,
                    pos,
                    sim_core::colony::ItemSpec::raw(ItemKind::WolfHide, None),
                    1,
                );
                res.colony.wildlife.tallies.hides_dropped += 1;
            }
            let left = res.colony.wildlife.wolves().count() as i64;
            let mut entry =
                ChronicleEntry::new(res.current_tick, ChronicleKind::WolfKilled).value(left);
            if let Some(p) = killer {
                entry = entry.subject(p);
            }
            let cause = crate::runtime::story::cause_ref(res, PressureKind::Beasts);
            res.chronicle.push_caused(entry, cause);
        }
        AnimalKind::Deer => {
            // A deer killed in a fight (drafted): its meat and hide as a hunt's, without a knife.
            crate::runtime::jobs::hunt::drop_carcass(
                res,
                pos,
                rule.meat_without_knife,
                ItemKind::Hide,
            );
            res.colony.wildlife.tallies.deer_killed += 1;
        }
    }
    let _ = world;
}
