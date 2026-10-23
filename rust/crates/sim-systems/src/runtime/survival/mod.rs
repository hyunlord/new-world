//! StarvationSystem: damages agents whose hunger or thirst is saturated, heals fed ones, kills at 0 hp; the file also hosts `despawn_agent`.
//!
//! Priority: 139 · Interval: 1 tick · Gate: none
//! Reads: `Agent`, `Position`, `Hunger`, `Thirst`, `Cold`, `ExposureClock`; `SimResources.rules.society.exposure`
//! Writes: `BodyHealth`; deaths via `despawn_agent` (despawn; `SimResources.settlements` rosters, `relationships`, social and combat maps, `bed_of`, `haulers`, `recent_deaths`, `chronicle`; M1: a worker's load and reservations in `colony`)
//! Events: `CausalEvent::AgentDied`; chronicle `Died`, `Widowed`; tallies `deaths_starvation`, `deaths_dehydration`
//!
//! V7 feature `add-starvation-death` — survival / death subsystem.
//!
//! Hosts:
//!   - [`despawn_agent`]: the SINGLE shared agent-death helper. Both
//!     [`StarvationSystem`] (needs death) and
//!     [`CombatSystem`](crate::runtime::combat::CombatSystem) (combat death)
//!     route through it, unifying the cleanup path (resource-map retain +
//!     settlement-roster removal + `AgentDied` chronicle emit). This ALSO
//!     fixes a pre-feature leak where combat death never cleaned
//!     `settlement.member_agents`.
//!   - [`StarvationSystem`] (priority **139**, interval 1): after needs-decay
//!     (130-132), combat (137), and settlement (138). When `Hunger.value >=
//!     Hunger::SATURATION` it damages hp; when `Thirst.value >=
//!     Thirst::SATURATION` it damages hp faster (thirst kills sooner). When
//!     BOTH needs are below `SAFE_NEED_CEILING` and hp < max, it heals slowly
//!     (recovery after eating → population stability). hp ≤ 0 → death.
//!
//! All damage/heal constants are `f64` (project-wide determinism rule). The
//! system is pure arithmetic over a query plus a deferred despawn list (no
//! RNG, no mid-query despawn) → deterministic.

use hecs::{Entity, World};
use sim_core::causal::event::{CausalEvent, DeathReason};
use sim_core::components::{
    Agent, AgentId, BodyHealth, Cold, ExposureClock, Hunger, Position, Thirst,
};
use sim_engine::{RecentDeath, RuntimeSystem, SimResources};

/// HP removed per tick while `Hunger.value >= Hunger::SATURATION`.
/// `100 / 0.08 = 1250` ticks of continuous hunger saturation to die (~42s @
/// 30 TPS) — survival pressure without population collapse.
pub const STARVATION_DMG_PER_TICK: f64 = 0.08;

/// HP removed per tick while `Thirst.value >= Thirst::SATURATION`.
/// Strictly greater than [`STARVATION_DMG_PER_TICK`] so thirst kills sooner
/// (`100 / 0.12 ≈ 833` ticks). When both needs are saturated the two rates
/// STACK ADDITIVELY (`0.20/tick ⇒ 500` ticks).
pub const DEHYDRATION_DMG_PER_TICK: f64 = 0.12;

/// A need at or below this value is "met" — no starvation damage accrues, and
/// (with hp < max) the agent heals. The interval `(SAFE_NEED_CEILING,
/// SATURATION)` is a NEUTRAL gray zone: neither damage nor heal.
pub const SAFE_NEED_CEILING: f64 = 50.0;

/// HP restored per tick when BOTH needs are below [`SAFE_NEED_CEILING`] and
/// hp < max_hp. `100 × 0.05 = 5.0 hp / 100 ticks` — a survived starvation
/// spell recovers, preventing a death spiral.
pub const STARVATION_HEAL_PER_TICK: f64 = 0.05;

/// Shared agent-death helper. Despawns `entity`, purges the four agent-keyed
/// resource maps, removes the agent from every settlement's `member_agents`
/// (keeping `population_stats.current` in sync and incrementing
/// `total_deaths`), and pushes a [`CausalEvent::AgentDied`] at the death tile.
///
/// The settlement-cleanup step is a no-op for a bandless (unaffiliated) agent
/// — it never panics. Iterating the `settlements` HashMap only mutates
/// per-agent membership, so the operation is order-independent (deterministic).
pub fn despawn_agent(
    world: &mut World,
    resources: &mut SimResources,
    entity: Entity,
    agent_id: AgentId,
    position: (u32, u32),
    reason: DeathReason,
    tick: u64,
) {
    // Pass 19 chronicle: read the person, their age and recorded partner
    // before the entity is gone (reads only).
    let person = crate::runtime::chronicle_log::person_record(world, entity);
    let partner = world
        .get::<&sim_core::components::Kin>(entity)
        .ok()
        .and_then(|kin| kin.partner);
    let age = world
        .get::<&sim_core::components::Life>(entity)
        .ok()
        .map_or(-1, |life| life.age_ticks(tick));
    // M5 (ADR 0030): the dead's kin grieve; read before the despawn.
    let kin = world
        .get::<&sim_core::components::Kin>(entity)
        .ok()
        .map(|k| *k);
    // M1: a dying worker's load stays where it fell; its reservations free up.
    crate::runtime::jobs::on_death(world, resources, entity);
    let _ = world.despawn(entity);
    resources.social_chase_since.remove(&agent_id);
    resources.social_retry_after.remove(&agent_id);
    resources.bed_of.remove(&agent_id);
    resources.haulers.remove(&agent_id);

    // Purge the four agent-keyed resource maps (drop any key referencing the
    // dead agent) — mirrors the pre-feature combat cleanup block.
    resources
        .relationships
        .retain(|k, _| k.0 != agent_id && k.1 != agent_id);
    resources
        .interaction_progress
        .retain(|k, _| k.0 != agent_id && k.1 != agent_id);
    resources
        .combat_pairs
        .retain(|(a, d)| *a != agent_id && *d != agent_id);
    resources
        .combat_progress
        .retain(|(a, d), _| *a != agent_id && *d != agent_id);

    // Settlement roster + population stats (no-op when the agent is bandless).
    // Pass 19: the roster the agent left (lowest id if ever several) is
    // remembered for the chronicle only.
    let mut last_roster: Option<sim_core::components::SettlementId> = None;
    for (sid, settlement) in resources.settlements.iter_mut() {
        if settlement.remove_member_agent(agent_id) {
            settlement.population_stats.current = settlement.member_agents.len() as u32;
            settlement.population_stats.total_deaths =
                settlement.population_stats.total_deaths.saturating_add(1);
            last_roster = Some(last_roster.map_or(*sid, |known| known.min(*sid)));
        }
    }

    match reason {
        DeathReason::Starvation => resources.tallies.deaths_starvation += 1,
        DeathReason::Dehydration => resources.tallies.deaths_dehydration += 1,
        DeathReason::Combat => resources.tallies.deaths_combat += 1,
        DeathReason::OldAge => resources.tallies.deaths_old_age += 1,
        DeathReason::Exposure => resources.tallies.deaths_exposure += 1,
    }

    // Chronicle the death at its tile.
    let width = resources.tile_grid.width;
    let tile_idx = position.1 * width + position.0;
    let event_id = resources.issue_event_id();
    resources.causal_log.push(
        tile_idx,
        CausalEvent::AgentDied {
            id: event_id,
            parent: None,
            agent: agent_id,
            position,
            reason,
            tick,
        },
    );

    // V7 viz-D — ALSO push to the display-only recent-deaths buffer (additive;
    // the causal_log push above is untouched). The dedicated buffer is reliable
    // for "show every recent death" where the 8-slot per-tile causal ring would
    // evict quickly. Pruned by `SimEngine::tick`.
    resources.recent_deaths.push(RecentDeath {
        x: position.0 as i32,
        y: position.1 as i32,
        reason,
        tick: tick as u32,
    });

    // Pass 19 chronicle (write-only): close the person's record, record the
    // death and, if the recorded partner is alive, their widowhood.
    resources
        .chronicle
        .close_person(agent_id, tick, reason.as_u8(), || person);
    let died_entry = resources.chronicle.len() as u32;
    crate::runtime::chronicle_log::record(
        world,
        resources,
        sim_core::chronicle::ChronicleEntry::new(tick, sim_core::chronicle::ChronicleKind::Died)
            .subject(agent_id)
            .other(partner)
            .settlement(last_roster)
            .cause(reason.as_u8())
            .value(age),
    );
    crate::runtime::mood::kin_died(world, resources, agent_id, kin, died_entry);
    if let Some(partner) = partner {
        if crate::runtime::chronicle_log::alive_entity(world, partner).is_some() {
            crate::runtime::chronicle_log::record(
                world,
                resources,
                sim_core::chronicle::ChronicleEntry::new(
                    tick,
                    sim_core::chronicle::ChronicleKind::Widowed,
                )
                .subject(partner)
                .other(Some(agent_id)),
            );
        }
    }
}

/// Needs → frailty → death system (priority 139, interval 1).
#[derive(Debug, Default)]
pub struct StarvationSystem;

impl StarvationSystem {
    /// Construct a fresh instance.
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeSystem for StarvationSystem {
    fn name(&self) -> &str {
        "StarvationSystem"
    }

    fn priority(&self) -> u32 {
        139
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let tick = resources.current_tick;
        // Pass 10: under the exposure rule a resident does not heal while
        // `Cold` or within `heal_block_hours` after its last cold hour.
        let heal_block = resources
            .rules
            .society
            .exposure
            .map(|rule| rule.heal_block_hours);

        // Deferred death list — never despawn mid-query.
        let mut deaths: Vec<(Entity, AgentId, (u32, u32), DeathReason)> = Vec::new();

        for (entity, (agent, position, hunger, thirst, health, cold, clock)) in world
            .query::<(
                &Agent,
                &Position,
                &Hunger,
                &Thirst,
                &mut BodyHealth,
                Option<&Cold>,
                Option<&ExposureClock>,
            )>()
            .iter()
        {
            let hunger_saturated = hunger.value >= Hunger::SATURATION;
            let thirst_saturated = thirst.value >= Thirst::SATURATION;

            if hunger_saturated || thirst_saturated {
                let mut damage = 0.0;
                if hunger_saturated {
                    damage += STARVATION_DMG_PER_TICK;
                }
                if thirst_saturated {
                    damage += DEHYDRATION_DMG_PER_TICK;
                }
                health.apply_damage(damage);

                if health.is_dead() {
                    // Thirst takes precedence as the faster killer.
                    let reason = if thirst_saturated {
                        DeathReason::Dehydration
                    } else {
                        DeathReason::Starvation
                    };
                    deaths.push((entity, agent.id, (position.x, position.y), reason));
                }
            } else if (hunger.value as f64) < SAFE_NEED_CEILING
                && thirst.value < SAFE_NEED_CEILING
                && health.hp < health.max_hp
                && !heal_block.is_some_and(|hours| {
                    cold.is_some() || clock.is_some_and(|clock| clock.blocks_healing(tick, hours))
                })
            {
                // Both needs met and not at full hp → recover slowly.
                health.heal(STARVATION_HEAL_PER_TICK);
            }
            // Gray zone (need in (SAFE_NEED_CEILING, SATURATION)) → no-op.
        }

        // Pass 9: under keyed draws deaths are booked in AgentId order (event
        // ids, the per-tile causal ring and despawn order no longer follow hecs order).
        if resources.rules.agent.keyed_draws {
            deaths.sort_by_key(|row| row.1);
        }
        for (entity, agent_id, position, reason) in deaths {
            despawn_agent(world, resources, entity, agent_id, position, reason, tick);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn death_removes_the_assigned_bed() {
        let mut world = World::new();
        let entity = world.spawn((Agent { id: 7 },));
        let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        resources.bed_of.insert(7, (2, 2));
        resources.bed_of.insert(8, (2, 2));
        despawn_agent(
            &mut world,
            &mut resources,
            entity,
            7,
            (2, 2),
            DeathReason::Starvation,
            0,
        );
        assert!(!resources.bed_of.contains_key(&7));
        assert_eq!(resources.bed_of.get(&8), Some(&(2, 2)));
    }

    #[test]
    fn metadata() {
        let s = StarvationSystem::new();
        assert_eq!(s.name(), "StarvationSystem");
        assert_eq!(s.priority(), 139);
        assert_eq!(s.tick_interval(), 1);
    }

    #[test]
    #[allow(clippy::assertions_on_constants)] // deliberate invariant lock on the tuning consts
    fn damage_constants_are_locked() {
        assert_eq!(STARVATION_DMG_PER_TICK, 0.08);
        assert_eq!(DEHYDRATION_DMG_PER_TICK, 0.12);
        assert_eq!(SAFE_NEED_CEILING, 50.0);
        assert_eq!(STARVATION_HEAL_PER_TICK, 0.05);
        // Thirst must kill strictly faster than hunger.
        assert!(DEHYDRATION_DMG_PER_TICK > STARVATION_DMG_PER_TICK);
    }
}
