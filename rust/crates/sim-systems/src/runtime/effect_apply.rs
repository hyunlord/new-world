use hecs::World;
use log::{debug, warn};
use sim_core::components::{BodyHealth, EffectFlags, Emotion, InjurySpec, Needs, PartFlags, Position, Wildlife};
use sim_core::{
    config, CausalEvent, CauseRef, EffectEntry, EffectFlag, EffectPrimitive, EffectSource,
    EffectStat, EmotionType, EntityId, NeedType, ScheduledEffect,
};
use sim_engine::{GameEvent, SimResources, SimSystem};

/// Final-pass system that drains the shared effect queue and applies queued effects.
pub struct EffectApplySystem {
    priority: u32,
    tick_interval: u64,
}

impl EffectApplySystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval,
        }
    }

    fn resolve_entity(world: &World, entity_id: EntityId) -> Option<hecs::Entity> {
        world.iter().find_map(|entity_ref| {
            let entity = entity_ref.entity();
            (entity.id() as u64 == entity_id.0).then_some(entity)
        })
    }

    fn stat_to_need(stat: EffectStat) -> Option<NeedType> {
        match stat {
            EffectStat::Hunger => Some(NeedType::Hunger),
            EffectStat::Warmth => Some(NeedType::Warmth),
            EffectStat::Safety => Some(NeedType::Safety),
            EffectStat::Comfort => Some(NeedType::Comfort),
            EffectStat::Energy => None,
            EffectStat::Meaning => Some(NeedType::Meaning),
        }
    }

    fn damped(delta: f64) -> f64 {
        delta * (1.0 - config::EFFECT_DAMPING_FACTOR)
    }

    fn clamp_stat(value: f64) -> f64 {
        value.clamp(config::EFFECT_STAT_CLAMP_MIN, config::EFFECT_STAT_CLAMP_MAX)
    }

    fn push_causal_event(
        resources: &mut SimResources,
        entity: EntityId,
        tick: u64,
        source: &EffectSource,
        effect_key: String,
        magnitude: f64,
    ) {
        resources.causal_log.push(
            entity,
            CausalEvent {
                tick,
                cause: CauseRef {
                    system: source.system.clone(),
                    kind: source.kind.clone(),
                    entity: Some(entity),
                    building: None,
                    settlement: None,
                },
                effect_key,
                summary_key: format!("EFFECT_{}", source.kind.to_uppercase()),
                magnitude,
            },
        );
    }

    fn apply_add_stat(
        resources: &mut SimResources,
        world: &mut World,
        entry: &EffectEntry,
        stat: EffectStat,
        amount: f64,
        tick: u64,
    ) {
        let Some(entity) = Self::resolve_entity(world, entry.entity) else {
            return;
        };
        let damped = Self::damped(amount);

        match stat {
            EffectStat::Energy => {
                if let Ok(mut needs) = world.get::<&mut Needs>(entity) {
                    needs.energy = Self::clamp_stat(needs.energy + damped);
                    Self::push_causal_event(
                        resources,
                        entry.entity,
                        tick,
                        &entry.source,
                        "effect_add_energy".to_string(),
                        damped,
                    );
                }
            }
            _ => {
                if let (Some(need), Ok(mut needs)) =
                    (Self::stat_to_need(stat), world.get::<&mut Needs>(entity))
                {
                    let current = needs.get(need);
                    needs.set(need, Self::clamp_stat(current + damped));
                    Self::push_causal_event(
                        resources,
                        entry.entity,
                        tick,
                        &entry.source,
                        format!("effect_add_{need:?}").to_lowercase(),
                        damped,
                    );
                }
            }
        }
    }

    fn apply_mul_stat(
        resources: &mut SimResources,
        world: &mut World,
        entry: &EffectEntry,
        stat: EffectStat,
        factor: f64,
        tick: u64,
    ) {
        let Some(entity) = Self::resolve_entity(world, entry.entity) else {
            return;
        };

        match stat {
            EffectStat::Energy => {
                if let Ok(mut needs) = world.get::<&mut Needs>(entity) {
                    needs.energy = Self::clamp_stat(needs.energy * factor);
                    Self::push_causal_event(
                        resources,
                        entry.entity,
                        tick,
                        &entry.source,
                        "effect_mul_energy".to_string(),
                        factor,
                    );
                }
            }
            _ => {
                if let (Some(need), Ok(mut needs)) =
                    (Self::stat_to_need(stat), world.get::<&mut Needs>(entity))
                {
                    let current = needs.get(need);
                    needs.set(need, Self::clamp_stat(current * factor));
                    Self::push_causal_event(
                        resources,
                        entry.entity,
                        tick,
                        &entry.source,
                        format!("effect_mul_{need:?}").to_lowercase(),
                        factor,
                    );
                }
            }
        }
    }

    fn apply_set_flag(
        resources: &mut SimResources,
        world: &mut World,
        entry: &EffectEntry,
        flag: EffectFlag,
        active: bool,
        tick: u64,
    ) {
        let Some(entity) = Self::resolve_entity(world, entry.entity) else {
            return;
        };

        match world.get::<&mut EffectFlags>(entity) {
            Ok(mut flags) => {
                match flag {
                    EffectFlag::Sheltered => flags.sheltered = active,
                    EffectFlag::Unsafe => flags.is_unsafe = active,
                    EffectFlag::Resting => flags.resting = active,
                }
                Self::push_causal_event(
                    resources,
                    entry.entity,
                    tick,
                    &entry.source,
                    format!("effect_flag_{flag:?}").to_lowercase(),
                    if active { 1.0 } else { 0.0 },
                );
            }
            Err(_) => {
                debug!(
                    "[EffectApply] skipping flag effect for entity {} without EffectFlags",
                    entry.entity.0
                );
            }
        }
    }

    fn apply_emit_influence(
        resources: &mut SimResources,
        world: &World,
        entry: &EffectEntry,
        emitter: &sim_core::InfluenceEmitter,
    ) {
        let Some(entity) = Self::resolve_entity(world, entry.entity) else {
            return;
        };
        let Ok(position) = world.get::<&Position>(entity) else {
            return;
        };
        let Some(x) = u32::try_from(position.tile_x()).ok() else {
            return;
        };
        let Some(y) = u32::try_from(position.tile_y()).ok() else {
            return;
        };
        let record = emitter.to_record(x, y);
        resources.influence_grid.stamp(&record);
    }

    fn apply_spawn_event(
        resources: &mut SimResources,
        entry: &EffectEntry,
        event_key: &str,
        tick: u64,
    ) {
        resources.event_bus.emit(GameEvent::EffectEventTriggered {
            entity_id: entry.entity,
            event_key: event_key.to_string(),
        });
        Self::push_causal_event(
            resources,
            entry.entity,
            tick,
            &entry.source,
            "effect_spawn_event".to_string(),
            1.0,
        );
    }

    fn apply_adjust_emotion(
        resources: &mut SimResources,
        world: &mut World,
        entry: &EffectEntry,
        emotion: EmotionType,
        amount: f64,
        tick: u64,
    ) {
        let Some(entity) = Self::resolve_entity(world, entry.entity) else {
            return;
        };
        let Ok(mut emo) = world.get::<&mut Emotion>(entity) else {
            return;
        };
        let damped = Self::damped(amount);
        emo.add(emotion, damped);
        Self::push_causal_event(
            resources,
            entry.entity,
            tick,
            &entry.source,
            format!("effect_adjust_{emotion:?}").to_lowercase(),
            damped,
        );
    }

    fn apply_schedule(
        resources: &mut SimResources,
        entry: &EffectEntry,
        delay_ticks: u64,
        effect: &EffectPrimitive,
        tick: u64,
    ) {
        if resources.effect_queue.scheduled_len() >= config::EFFECT_QUEUE_MAX_SCHEDULED {
            warn!("[EffectApply] scheduled queue full, dropping scheduled effect");
            return;
        }
        resources.effect_queue.push_scheduled(ScheduledEffect {
            fire_tick: tick.saturating_add(delay_ticks),
            entry: EffectEntry {
                entity: entry.entity,
                effect: effect.clone(),
                source: entry.source.clone(),
            },
        });
    }

    fn apply_damage_part(
        resources: &mut SimResources,
        world: &mut World,
        entry: &EffectEntry,
        part_idx: u8,
        severity: u8,
        flags_bits: u8,
        bleed_rate: u8,
        tick: u64,
    ) {
        let Some(entity) = Self::resolve_entity(world, entry.entity) else {
            return;
        };
        if let Ok(mut health) = world.get::<&mut BodyHealth>(entity) {
            let report = health.apply_injury(InjurySpec {
                part_idx,
                severity,
                flags: PartFlags(flags_bits),
                bleed_rate,
            });
            Self::push_causal_event(
                resources,
                entry.entity,
                tick,
                &entry.source,
                format!("part_{}_hp_{}", report.part_idx, report.hp_after),
                f64::from(severity),
            );
        }
    }
}

impl SimSystem for EffectApplySystem {
    fn name(&self) -> &'static str {
        "effect_apply_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        resources.effect_queue.flush(tick);
        let entries = resources.effect_queue.drain_active();

        for entry in entries {
            match &entry.effect {
                EffectPrimitive::AddStat { stat, amount } => {
                    Self::apply_add_stat(resources, world, &entry, *stat, *amount, tick);
                }
                EffectPrimitive::MulStat { stat, factor } => {
                    Self::apply_mul_stat(resources, world, &entry, *stat, *factor, tick);
                }
                EffectPrimitive::SetFlag { flag, active } => {
                    Self::apply_set_flag(resources, world, &entry, *flag, *active, tick);
                }
                EffectPrimitive::EmitInfluence { emitter } => {
                    Self::apply_emit_influence(resources, world, &entry, emitter);
                }
                EffectPrimitive::SpawnEvent { event_key } => {
                    Self::apply_spawn_event(resources, &entry, event_key, tick);
                }
                EffectPrimitive::Schedule {
                    delay_ticks,
                    effect,
                } => {
                    Self::apply_schedule(resources, &entry, *delay_ticks, effect, tick);
                }
                EffectPrimitive::AdjustEmotion { emotion, amount } => {
                    Self::apply_adjust_emotion(resources, world, &entry, *emotion, *amount, tick);
                }
                EffectPrimitive::DamagePart { part_idx, severity, flags_bits, bleed_rate } => {
                    Self::apply_damage_part(
                        resources, world, &entry, *part_idx, *severity, *flags_bits, *bleed_rate, tick,
                    );
                }
                EffectPrimitive::DamageWildlife { entity_id, damage } => {
                    if let Some(e) = Self::resolve_entity(world, *entity_id) {
                        if let Ok(mut w) = world.get::<&mut Wildlife>(e) {
                            w.current_hp = (w.current_hp - *damage as f64).max(0.0);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EffectApplySystem;
    use hecs::World;
    use sim_core::components::{EffectFlags, Needs, Position};
    use sim_core::{
        config::GameConfig, EffectEntry, EffectFlag, EffectPrimitive, EffectSource, EffectStat,
        EntityId, GameCalendar, NeedType, WorldMap,
    };
    use sim_engine::{SimResources, SimSystem};

    fn make_resources() -> SimResources {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(4, 4, 7);
        SimResources::new(calendar, map, 17)
    }

    fn spawn_target(world: &mut World, needs: Needs) -> (hecs::Entity, EntityId) {
        let entity = world.spawn((needs, EffectFlags::default(), Position::new(1, 1)));
        (entity, EntityId(entity.id() as u64))
    }

    fn source(kind: &str) -> EffectSource {
        EffectSource {
            system: "effect_test".to_string(),
            kind: kind.to_string(),
        }
    }

    #[test]
    fn effect_apply_add_stat_modifies_need() {
        let mut world = World::new();
        let mut resources = make_resources();
        let (entity, entity_id) = spawn_target(&mut world, Needs::default());
        {
            let mut needs = world.get::<&mut Needs>(entity).expect("needs");
            needs.set(NeedType::Hunger, 0.5);
        }

        resources.effect_queue.push(EffectEntry {
            entity: entity_id,
            effect: EffectPrimitive::AddStat {
                stat: EffectStat::Hunger,
                amount: -0.1,
            },
            source: source("add_hunger"),
        });

        let mut system = EffectApplySystem::new(9999, 1);
        system.run(&mut world, &mut resources, 1);

        let needs = world.get::<&Needs>(entity).expect("needs after apply");
        assert!((needs.get(NeedType::Hunger) - 0.4).abs() < 1e-6);
    }

    #[test]
    fn effect_apply_mul_stat_multiplies_need() {
        let mut world = World::new();
        let mut resources = make_resources();
        let (entity, entity_id) = spawn_target(&mut world, Needs::default());
        {
            let mut needs = world.get::<&mut Needs>(entity).expect("needs");
            needs.set(NeedType::Safety, 0.8);
        }

        resources.effect_queue.push(EffectEntry {
            entity: entity_id,
            effect: EffectPrimitive::MulStat {
                stat: EffectStat::Safety,
                factor: 0.5,
            },
            source: source("mul_safety"),
        });

        let mut system = EffectApplySystem::new(9999, 1);
        system.run(&mut world, &mut resources, 1);

        let needs = world.get::<&Needs>(entity).expect("needs after apply");
        assert!((needs.get(NeedType::Safety) - 0.4).abs() < 1e-6);
    }

    #[test]
    fn effect_apply_clamps_to_bounds() {
        let mut world = World::new();
        let mut resources = make_resources();
        let (entity, entity_id) = spawn_target(&mut world, Needs::default());
        {
            let mut needs = world.get::<&mut Needs>(entity).expect("needs");
            needs.set(NeedType::Warmth, 0.95);
        }

        resources.effect_queue.push(EffectEntry {
            entity: entity_id,
            effect: EffectPrimitive::AddStat {
                stat: EffectStat::Warmth,
                amount: 0.2,
            },
            source: source("clamp_warmth"),
        });

        let mut system = EffectApplySystem::new(9999, 1);
        system.run(&mut world, &mut resources, 1);

        let needs = world.get::<&Needs>(entity).expect("needs after apply");
        assert!((needs.get(NeedType::Warmth) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn effect_apply_writes_causal_log() {
        let mut world = World::new();
        let mut resources = make_resources();
        let (_entity, entity_id) = spawn_target(&mut world, Needs::default());

        resources.effect_queue.push(EffectEntry {
            entity: entity_id,
            effect: EffectPrimitive::SetFlag {
                flag: EffectFlag::Resting,
                active: true,
            },
            source: source("set_resting"),
        });

        let mut system = EffectApplySystem::new(9999, 1);
        system.run(&mut world, &mut resources, 3);

        let recent = resources.causal_log.recent(entity_id, 4);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].cause.system, "effect_test");
        assert_eq!(recent[0].tick, 3);
    }
}
