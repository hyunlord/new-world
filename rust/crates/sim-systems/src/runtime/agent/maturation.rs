//! MaturationSystem: ends childhood for children older than the life-stage child age.
//!
//! Priority: 144 · Interval: 60 ticks · Gate: `WorldRules.society.life_stage`
//! Reads: `Agent`, `Child`; `SimResources.current_tick`
//! Writes: `Child` (removed), `Hunger`, `Thirst`, `Sleep` (adult growth rates); `SimResources.chronicle`
//! Events: chronicle `CameOfAge`
//!
//! Rules-gated childhood completion without perturbing resident RNG streams.
use hecs::World;
use sim_core::components::{Agent, Child, Hunger, Sleep, Thirst};
use sim_engine::{RuntimeSystem, SimResources};

/// Promotes due children in stable agent order every sixty ticks.
#[derive(Debug, Default)]
pub struct MaturationSystem;

impl RuntimeSystem for MaturationSystem {
    fn name(&self) -> &str {
        "MaturationSystem"
    }
    fn priority(&self) -> u32 {
        144
    }
    fn tick_interval(&self) -> u64 {
        60
    }
    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let Some(rule) = resources.rules.society.life_stage else {
            return;
        };
        let mut due: Vec<_> = world
            .query::<(&Agent, &Child)>()
            .iter()
            .filter(|(_, (_, child))| {
                resources.current_tick.saturating_sub(child.born_tick) >= rule.child_ticks
            })
            .map(|(entity, (agent, _))| (agent.id, entity))
            .collect();
        due.sort_by_key(|&(id, _)| id);
        for (id, entity) in due {
            if world.remove_one::<Child>(entity).is_err() {
                continue;
            }
            // Pass 19 chronicle (write-only).
            crate::runtime::chronicle_log::ensure_entity(world, resources, id, entity);
            crate::runtime::chronicle_log::record(
                world,
                resources,
                sim_core::chronicle::ChronicleEntry::new(
                    resources.current_tick,
                    sim_core::chronicle::ChronicleKind::CameOfAge,
                )
                .subject(id),
            );
            if let Ok(mut hunger) = world.get::<&mut Hunger>(entity) {
                hunger.growth_rate = 0.05;
            }
            if let Ok(mut thirst) = world.get::<&mut Thirst>(entity) {
                thirst.growth_rate = 0.08;
            }
            if let Ok(mut sleep) = world.get::<&mut Sleep>(entity) {
                sleep.growth_rate = 0.03;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::{rules::LifeStageRule, MaterialRegistry};
    #[test]
    fn due_children_restore_rates_without_resetting_needs() {
        let mut world = World::new();
        let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
        resources.rules.society.life_stage = Some(LifeStageRule {
            child_ticks: 120,
            hunger_rate: 0.03,
            thirst_rate: 0.048,
            sleep_rate: 0.03,
            tether_radius: 3,
            infant_ticks: None,
        });
        let entity = world.spawn((
            Agent { id: 2 },
            Child { born_tick: 60 },
            Hunger::new(17.0, 0.03),
            Thirst::new(21.0, 0.048),
            Sleep::new(25.0, 0.03),
        ));
        let mut system = MaturationSystem;
        assert_eq!((system.priority(), system.tick_interval()), (144, 60));
        resources.current_tick = 179;
        system.tick(&mut world, &mut resources);
        assert!(world.get::<&Child>(entity).is_ok());
        resources.current_tick = 180;
        system.tick(&mut world, &mut resources);
        assert!(world.get::<&Child>(entity).is_err());
        assert_eq!(
            *world.get::<&Hunger>(entity).expect("hunger"),
            Hunger::new(17.0, 0.05)
        );
        assert_eq!(
            *world.get::<&Thirst>(entity).expect("thirst"),
            Thirst::new(21.0, 0.08)
        );
        assert_eq!(
            *world.get::<&Sleep>(entity).expect("sleep"),
            Sleep::new(25.0, 0.03)
        );
    }
    #[test]
    fn disabled_rule_preserves_even_due_child_components() {
        let mut world = World::new();
        let entity = world.spawn((
            Agent { id: 0 },
            Child { born_tick: 0 },
            Hunger::new(17.0, 0.03),
        ));
        let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
        resources.current_tick = 10000;
        MaturationSystem.tick(&mut world, &mut resources);
        assert!(world.get::<&Child>(entity).is_ok());
        assert_eq!(
            world.get::<&Hunger>(entity).expect("hunger").growth_rate,
            0.03
        );
    }
}
