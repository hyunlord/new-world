//! Rules-gated aging: deterministic lifespans, the elder stage and old-age death.
//!
//! Every resident's lifespan comes from its own salted splitmix64 stream keyed
//! by world seed and agent id. The stream is a fresh [`MovementRng`] value used
//! only here; the agent's movement `MovementRng` component is never advanced.
use hecs::World;
use sim_core::causal::event::DeathReason;
use sim_core::components::{Agent, Elder, Life, Position};
use sim_core::rules::AgingRule;
use sim_engine::{RuntimeSystem, SimResources};

use super::MovementRng;
use crate::runtime::survival::despawn_agent;
use sim_core::rng::{below, keyed_u64, STREAM_LIFESPAN};

/// Salt of the per-agent aging stream; distinct from every other stream salt.
pub const AGING_RNG_SALT: u64 = 0x4C1F_E5A1_0000_0001;

/// Engine priority of [`AgingSystem`]: after maturation (144) and beds (145).
pub const AGING_PRIORITY: u32 = 146;

/// Ticks between aging passes.
pub const AGING_INTERVAL: u64 = 60;

/// Range of the founder top-up drawn when a founder would die too soon (two days).
const FOUNDER_TOP_UP_TICKS: u64 = 2880;

/// The aging stream of `agent_id` in a world generated from `world_seed`.
pub fn aging_stream(world_seed: u64, agent_id: u64) -> MovementRng {
    MovementRng::new(
        world_seed
            ^ agent_id
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(AGING_RNG_SALT),
    )
}

/// Mean of three uniform draws in `[mean - spread, mean + spread]`, integer only.
pub fn draw_lifespan(rng: &mut MovementRng, rule: &AgingRule) -> u64 {
    let span = 2 * rule.lifespan_spread_ticks + 1;
    let sum: u64 = (0..3).map(|_| rng.next_u64() % span).sum();
    rule.lifespan_mean_ticks - rule.lifespan_spread_ticks + sum / 3
}

/// A founder's age (born before tick zero) and lifespan, with the guaranteed remainder.
pub fn founder_life(world_seed: u64, agent_id: u64, rule: &AgingRule) -> Life {
    let mut rng = aging_stream(world_seed, agent_id);
    let range = rule.founder_age_max_ticks - rule.founder_age_min_ticks + 1;
    let age = rule.founder_age_min_ticks + rng.next_u64() % range;
    let mut lifespan = draw_lifespan(&mut rng, rule);
    if lifespan < age + rule.founder_min_remaining_ticks {
        lifespan = age + rule.founder_min_remaining_ticks + rng.next_u64() % FOUNDER_TOP_UP_TICKS;
    }
    Life {
        born_tick: -(age as i64),
        lifespan_ticks: lifespan,
    }
}

/// A newborn's life, born at simulation tick `tick`.
pub fn newborn_life(world_seed: u64, agent_id: u64, tick: u64, rule: &AgingRule) -> Life {
    let mut rng = aging_stream(world_seed, agent_id);
    Life {
        born_tick: tick as i64,
        lifespan_ticks: draw_lifespan(&mut rng, rule),
    }
}

/// Draw `slot` of a resident's keyed lifespan stream (Pass 9 section 7.1):
/// `keyed_u64(seed, STREAM_LIFESPAN, lineage key, 0, slot, replicate)`. No tick
/// in the key, so a birth shifted by a tick keeps its lifespan.
fn life_draw(world_seed: u64, key: u64, slot: u32, replicate: u32) -> u64 {
    keyed_u64(world_seed, STREAM_LIFESPAN, key, 0, slot, replicate)
}

/// Mean of three keyed uniforms in `[mean - spread, mean + spread]` (slots 1–3, multiply-shift).
fn keyed_lifespan(world_seed: u64, key: u64, replicate: u32, rule: &AgingRule) -> u64 {
    let span = 2 * rule.lifespan_spread_ticks + 1;
    let sum: u64 = (1..=3)
        .map(|slot| below(life_draw(world_seed, key, slot, replicate), span))
        .sum();
    rule.lifespan_mean_ticks - rule.lifespan_spread_ticks + sum / 3
}

/// A founder's age and lifespan under keyed draws: slot 0 the age, slots 1–3
/// the lifespan, slot 4 the top-up (always drawn, used only when the founder
/// would die too soon, so the draw count never depends on the values).
pub fn keyed_founder_life(world_seed: u64, key: u64, replicate: u32, rule: &AgingRule) -> Life {
    let range = rule.founder_age_max_ticks - rule.founder_age_min_ticks + 1;
    let age = rule.founder_age_min_ticks + below(life_draw(world_seed, key, 0, replicate), range);
    let top_up = below(
        life_draw(world_seed, key, 4, replicate),
        FOUNDER_TOP_UP_TICKS,
    );
    let mut lifespan = keyed_lifespan(world_seed, key, replicate, rule);
    if lifespan < age + rule.founder_min_remaining_ticks {
        lifespan = age + rule.founder_min_remaining_ticks + top_up;
    }
    Life {
        born_tick: -(age as i64),
        lifespan_ticks: lifespan,
    }
}

/// A newborn's life under keyed draws, born at simulation tick `tick`
/// (slots 1–3; `replicate` is the noise replicate in force at the birth).
pub fn keyed_newborn_life(
    world_seed: u64,
    key: u64,
    tick: u64,
    replicate: u32,
    rule: &AgingRule,
) -> Life {
    Life {
        born_tick: tick as i64,
        lifespan_ticks: keyed_lifespan(world_seed, key, replicate, rule),
    }
}

/// Promotes elders and removes residents whose age reached their lifespan.
#[derive(Debug, Default)]
pub struct AgingSystem;

impl RuntimeSystem for AgingSystem {
    fn name(&self) -> &str {
        "AgingSystem"
    }
    fn priority(&self) -> u32 {
        AGING_PRIORITY
    }
    fn tick_interval(&self) -> u64 {
        AGING_INTERVAL
    }
    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let Some(rule) = resources.rules.society.aging else {
            return;
        };
        let tick = resources.current_tick;
        let mut residents: Vec<_> = world
            .query::<(&Agent, &Life, &Position, Option<&Elder>)>()
            .iter()
            .map(|(entity, (agent, life, pos, elder))| {
                (
                    agent.id,
                    entity,
                    life.age_ticks(tick),
                    life.lifespan_ticks,
                    (pos.x, pos.y),
                    elder.is_some(),
                )
            })
            .collect();
        residents.sort_by_key(|row| row.0);
        for (id, entity, age, lifespan, position, elder) in residents {
            if age >= lifespan as i64 {
                despawn_agent(
                    world,
                    resources,
                    entity,
                    id,
                    position,
                    DeathReason::OldAge,
                    tick,
                );
            } else if age >= rule.elder_age_ticks as i64 && !elder {
                let _ = world.insert_one(entity, Elder);
                // Pass 19 chronicle (write-only).
                crate::runtime::chronicle_log::ensure_entity(world, resources, id, entity);
                crate::runtime::chronicle_log::record(
                    world,
                    resources,
                    sim_core::chronicle::ChronicleEntry::new(
                        tick,
                        sim_core::chronicle::ChronicleKind::BecameElder,
                    )
                    .subject(id),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::material::MaterialRegistry;

    const RULE: AgingRule = AgingRule {
        elder_age_ticks: 66_240,
        lifespan_mean_ticks: 83_520,
        lifespan_spread_ticks: 11_520,
        founder_age_min_ticks: 8_640,
        founder_age_max_ticks: 57_600,
        founder_min_remaining_ticks: 4_320,
        elder_tether_radius: 4,
        elder_evening_tether: false,
    };

    #[test]
    fn lifespan_stays_in_range_and_never_advances_movement_streams() {
        for id in 0..200u64 {
            let life = newborn_life(9, id, 100, &RULE);
            assert!((72_000..=95_040).contains(&life.lifespan_ticks));
            assert_eq!(life, newborn_life(9, id, 100, &RULE));
        }
    }

    #[test]
    fn keyed_lifespans_ignore_the_birth_tick_and_stay_in_range() {
        // Pass 9: no tick in the key, so a birth shifted by a tick keeps its lifespan.
        for key in 0..300u64 {
            let a = keyed_newborn_life(9, key, 100, 0, &RULE);
            let b = keyed_newborn_life(9, key, 101, 0, &RULE);
            assert_eq!(a.lifespan_ticks, b.lifespan_ticks);
            assert_eq!((a.born_tick, b.born_tick), (100, 101));
            assert!((72_000..=95_040).contains(&a.lifespan_ticks));
        }
        // The replicate changes the draw; another key does too.
        let differs = (0..50u64).any(|key| {
            keyed_newborn_life(9, key, 0, 0, &RULE).lifespan_ticks
                != keyed_newborn_life(9, key, 0, 1, &RULE).lifespan_ticks
        });
        assert!(differs);
    }

    #[test]
    fn keyed_founders_keep_the_minimum_remaining_life() {
        // With these values the top-up is never needed (57,600 + 4,320 < 72,000);
        // older founders exercise slot 4.
        let old = AgingRule {
            founder_age_max_ticks: 90_000,
            ..RULE
        };
        let mut topped = 0;
        for key in 0..2_000u64 {
            for rule in [RULE, old] {
                let life = keyed_founder_life(3, key, 0, &rule);
                let age = (-life.born_tick) as u64;
                assert!((rule.founder_age_min_ticks..=rule.founder_age_max_ticks).contains(&age));
                assert!(life.lifespan_ticks >= age + 4_320);
                if keyed_lifespan(3, key, 0, &rule) < age + 4_320 {
                    topped += 1;
                    assert!(life.lifespan_ticks < age + 4_320 + FOUNDER_TOP_UP_TICKS);
                }
            }
        }
        assert!(topped > 0, "the top-up slot is exercised");
    }

    #[test]
    fn founders_keep_the_minimum_remaining_life() {
        for id in 0..500u64 {
            let life = founder_life(3, id, &RULE);
            let age = (-life.born_tick) as u64;
            assert!((8_640..=57_600).contains(&age));
            assert!(life.lifespan_ticks >= age + 4_320);
        }
    }

    #[test]
    fn system_promotes_then_retires_in_agent_order() {
        let mut world = World::new();
        let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
        resources.rules.society.aging = Some(RULE);
        resources.current_tick = 70_000;
        let elder = world.spawn((
            Agent { id: 1 },
            Position::new(1, 1),
            Life {
                born_tick: 0,
                lifespan_ticks: 80_000,
            },
        ));
        let dying = world.spawn((
            Agent { id: 2 },
            Position::new(2, 2),
            Life {
                born_tick: -20_000,
                lifespan_ticks: 90_000,
            },
        ));
        let young = world.spawn((
            Agent { id: 3 },
            Position::new(3, 3),
            Life {
                born_tick: 60_000,
                lifespan_ticks: 80_000,
            },
        ));
        AgingSystem.tick(&mut world, &mut resources);
        assert!(world.get::<&Elder>(elder).is_ok());
        assert!(!world.contains(dying));
        assert!(world.get::<&Elder>(young).is_err());
        assert_eq!(resources.tallies.deaths_old_age, 1);
        assert_eq!(resources.recent_deaths[0].reason, DeathReason::OldAge);
    }

    #[test]
    fn absent_rule_is_a_no_op() {
        let mut world = World::new();
        let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
        resources.current_tick = 1_000_000;
        let entity = world.spawn((
            Agent { id: 1 },
            Position::new(1, 1),
            Life {
                born_tick: 0,
                lifespan_ticks: 1,
            },
        ));
        AgingSystem.tick(&mut world, &mut resources);
        assert!(world.contains(entity));
        assert_eq!(resources.tallies.deaths_old_age, 0);
    }
}
