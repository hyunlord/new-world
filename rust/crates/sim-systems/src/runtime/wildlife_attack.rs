use hecs::World;
use sim_core::components::{BodyHealth, Identity, Position, Wildlife};
use sim_core::{config, EffectEntry, EffectPrimitive, EffectSource, EntityId};
use sim_engine::{SimResources, SimSystem};

/// Bidirectional combat attack system (Phase A3).
///
/// On each tick, alive wildlife within `WILDLIFE_ATTACK_RANGE` of a human agent
/// push a `DamagePart` effect for that agent, subject to a per-entity cooldown
/// of `WILDLIFE_ATTACK_COOLDOWN` ticks.
///
/// Two-pass pattern: read-only snapshot of human positions, then mutating pass
/// to push effects and update `last_attack_tick`.
pub struct WildlifeAttackSystem {
    priority: u32,
    tick_interval: u64,
}

impl WildlifeAttackSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self { priority, tick_interval }
    }
}

struct AttackIntent {
    wildlife_entity: hecs::Entity,
    target_entity_id: EntityId,
    damage: u8,
    kind_name: &'static str,
}

impl SimSystem for WildlifeAttackSystem {
    fn name(&self) -> &'static str {
        "wildlife_attack_system"
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        let tick_u32 = tick as u32;

        // Pass 1 (read-only): snapshot human agent positions + entity IDs.
        let agents: Vec<(EntityId, f64, f64)> = world
            .query::<(&Identity, &Position, &BodyHealth)>()
            .iter()
            .filter(|(_, (id, _, _))| id.species_id == "human")
            .map(|(e, (_, pos, _))| (EntityId(e.id() as u64), pos.x, pos.y))
            .collect();

        if agents.is_empty() {
            return;
        }

        // Gather attack intents (read-only wildlife pass).
        let intents: Vec<AttackIntent> = world
            .query::<(&Wildlife, &Position)>()
            .iter()
            .filter_map(|(we, (wildlife, pos))| {
                if !wildlife.is_alive() {
                    return None;
                }
                if tick_u32.saturating_sub(wildlife.last_attack_tick)
                    < config::WILDLIFE_ATTACK_COOLDOWN
                {
                    return None;
                }
                let wx = pos.x;
                let wy = pos.y;
                // Nearest human within attack range.
                let mut best: Option<(EntityId, f64)> = None;
                for &(eid, ax, ay) in &agents {
                    let dist = ((wx - ax).powi(2) + (wy - ay).powi(2)).sqrt();
                    if dist <= config::WILDLIFE_ATTACK_RANGE
                        && best.map(|(_, d)| dist < d).unwrap_or(true)
                    {
                        best = Some((eid, dist));
                    }
                }
                best.map(|(target_id, _)| AttackIntent {
                    wildlife_entity: we,
                    target_entity_id: target_id,
                    damage: wildlife.kind.attack_damage(),
                    kind_name: wildlife.kind.kind_name(),
                })
            })
            .collect();

        // Pass 2 (mutating): push effects + update cooldowns.
        for intent in intents {
            resources.effect_queue.push(EffectEntry {
                entity: intent.target_entity_id,
                effect: EffectPrimitive::DamagePart {
                    part_idx: 255,
                    severity: intent.damage,
                    flags_bits: 0x01,
                    bleed_rate: 2,
                },
                source: EffectSource {
                    system: "wildlife_attack".to_string(),
                    kind: format!("{}_attack", intent.kind_name),
                },
            });
            if let Ok(mut w) = world.get::<&mut Wildlife>(intent.wildlife_entity) {
                w.last_attack_tick = tick_u32;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildlife_attack_system_has_correct_priority() {
        let system = WildlifeAttackSystem::new(23, 1);
        assert_eq!(system.priority(), 23);
        assert_eq!(system.tick_interval(), 1);
        assert_eq!(system.name(), "wildlife_attack_system");
    }
}
