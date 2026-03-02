use hecs::World;
use sim_core::components::{Behavior, Identity, Needs, Skills, Social, Values};
use sim_core::config;
use sim_core::{GrowthStage, NeedType, ValueType};
use sim_engine::{SimResources, SimSystem};
use crate::body;

/// First production Rust runtime system migrated from GDScript.
///
/// This system mirrors the scheduler slot of `stats_recorder.gd` and
/// executes inside the Rust engine tick loop.
#[derive(Debug, Clone)]
pub struct StatsRecorderSystem {
    priority: u32,
    tick_interval: u64,
}

impl StatsRecorderSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for StatsRecorderSystem {
    fn name(&self) -> &'static str {
        "stats_recorder"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        // Initial Rust-port baseline: keep this system side-effect free while
        // moving scheduler ownership into Rust. Follow-up phases will port the
        // full history/snapshot behavior from GDScript.
        let _population_count: u32 = world.len();
        let _settlement_count: usize = resources.settlements.len();
        let _queued_events: usize = resources.event_bus.pending_count();
    }
}

/// Rust runtime baseline system for stat-cache synchronization.
///
/// Full parity requires Rust-owned entity stat storage (Phase D). Until then this
/// keeps scheduler ownership migration and registry tracking in Rust.
#[derive(Debug, Clone)]
pub struct StatSyncSystem {
    priority: u32,
    tick_interval: u64,
}

impl StatSyncSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for StatSyncSystem {
    fn name(&self) -> &'static str {
        "stat_sync_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        // Baseline: this system intentionally has no gameplay side effects
        // until entity stat caches are migrated into Rust-owned data.
        let _population_count: u32 = world.len();
        let _map_tile_count: usize = resources.map.tile_count();
    }
}

/// Rust runtime system for tile resource regeneration.
///
/// This is the Rust execution counterpart of `resource_regen_system.gd`.
#[derive(Debug, Clone)]
pub struct ResourceRegenSystem {
    priority: u32,
    tick_interval: u64,
}

impl ResourceRegenSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for ResourceRegenSystem {
    fn name(&self) -> &'static str {
        "resource_regen_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, _world: &mut World, resources: &mut SimResources, _tick: u64) {
        let width: u32 = resources.map.width;
        let height: u32 = resources.map.height;
        for y in 0..height {
            for x in 0..width {
                let tile = resources.map.get_mut(x, y);
                for deposit in &mut tile.resources {
                    if deposit.regen_rate <= 0.0 || deposit.amount >= deposit.max_amount {
                        continue;
                    }
                    let next_amount = deposit.amount + deposit.regen_rate;
                    deposit.amount = next_amount.min(deposit.max_amount);
                }
            }
        }
    }
}

/// Rust runtime system for upper-needs decay/fulfillment.
///
/// The step formula mirrors the `upper_needs_system.gd` Rust-bridge path.
#[derive(Debug, Clone)]
pub struct UpperNeedsRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl UpperNeedsRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[inline]
fn upper_needs_job_code(job: &str) -> i32 {
    match job {
        "builder" | "miner" => 1,
        "gatherer" | "lumberjack" => 2,
        _ => 0,
    }
}

#[inline]
fn upper_need_value(values: &Values, key: ValueType) -> f32 {
    values.get(key) as f32
}

impl SimSystem for UpperNeedsRuntimeSystem {
    fn name(&self) -> &'static str {
        "upper_needs_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, _tick: u64) {
        let mut query = world.query::<(
            &mut Needs,
            Option<&Skills>,
            Option<&Values>,
            Option<&Behavior>,
            Option<&Identity>,
            Option<&Social>,
        )>();
        for (_, (needs, skills_opt, values_opt, behavior_opt, identity_opt, social_opt)) in &mut query {
            if let Some(identity) = identity_opt {
                if matches!(identity.growth_stage, GrowthStage::Infant | GrowthStage::Toddler) {
                    continue;
                }
            }

            let has_job = behavior_opt
                .map(|behavior| behavior.job.as_str() != "none")
                .unwrap_or(false);
            let has_settlement = identity_opt
                .and_then(|identity| identity.settlement_id)
                .is_some();
            let has_partner = social_opt.and_then(|social| social.spouse).is_some();

            let foraging = skills_opt
                .map(|skills| skills.get_level("SKILL_FORAGING") as i32)
                .unwrap_or(0);
            let woodcutting = skills_opt
                .map(|skills| skills.get_level("SKILL_WOODCUTTING") as i32)
                .unwrap_or(0);
            let mining = skills_opt
                .map(|skills| skills.get_level("SKILL_MINING") as i32)
                .unwrap_or(0);
            let construction = skills_opt
                .map(|skills| skills.get_level("SKILL_CONSTRUCTION") as i32)
                .unwrap_or(0);
            let hunting = skills_opt
                .map(|skills| skills.get_level("SKILL_HUNTING") as i32)
                .unwrap_or(0);
            let skill_levels = [foraging, woodcutting, mining, construction, hunting];
            let best_skill_norm = body::upper_needs_best_skill_normalized(&skill_levels, 100);

            let job_code = behavior_opt
                .map(|behavior| upper_needs_job_code(behavior.job.as_str()))
                .unwrap_or(0);
            let (craftsmanship, skill, hard_work, nature, independence, sacrifice) =
                if let Some(values) = values_opt {
                    (
                        upper_need_value(values, ValueType::Craftsmanship),
                        upper_need_value(values, ValueType::Skill),
                        upper_need_value(values, ValueType::HardWork),
                        upper_need_value(values, ValueType::Nature),
                        upper_need_value(values, ValueType::Independence),
                        upper_need_value(values, ValueType::Sacrifice),
                    )
                } else {
                    (0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
                };
            let alignment = body::upper_needs_job_alignment(
                job_code,
                craftsmanship,
                skill,
                hard_work,
                nature,
                independence,
            );

            let current_values = [
                needs.get(NeedType::Competence) as f32,
                needs.get(NeedType::Autonomy) as f32,
                needs.get(NeedType::SelfActualization) as f32,
                needs.get(NeedType::Meaning) as f32,
                needs.get(NeedType::Transcendence) as f32,
                needs.get(NeedType::Recognition) as f32,
                needs.get(NeedType::Belonging) as f32,
                needs.get(NeedType::Intimacy) as f32,
            ];
            let decay_values = [
                config::UPPER_NEEDS_COMPETENCE_DECAY as f32,
                config::UPPER_NEEDS_AUTONOMY_DECAY as f32,
                config::UPPER_NEEDS_SELF_ACTUATION_DECAY as f32,
                config::UPPER_NEEDS_MEANING_DECAY as f32,
                config::UPPER_NEEDS_TRANSCENDENCE_DECAY as f32,
                config::UPPER_NEEDS_RECOGNITION_DECAY as f32,
                config::UPPER_NEEDS_BELONGING_DECAY as f32,
                config::UPPER_NEEDS_INTIMACY_DECAY as f32,
            ];
            let out = body::upper_needs_step(
                &current_values,
                &decay_values,
                config::UPPER_NEEDS_COMPETENCE_JOB_GAIN as f32,
                config::UPPER_NEEDS_AUTONOMY_JOB_GAIN as f32,
                config::UPPER_NEEDS_BELONGING_SETTLEMENT_GAIN as f32,
                config::UPPER_NEEDS_INTIMACY_PARTNER_GAIN as f32,
                config::UPPER_NEEDS_RECOGNITION_SKILL_COEFF as f32,
                config::UPPER_NEEDS_SELF_ACTUATION_SKILL_COEFF as f32,
                config::UPPER_NEEDS_MEANING_BASE_GAIN as f32,
                config::UPPER_NEEDS_MEANING_ALIGNED_GAIN as f32,
                config::UPPER_NEEDS_TRANSCENDENCE_SETTLEMENT_GAIN as f32,
                config::UPPER_NEEDS_TRANSCENDENCE_SACRIFICE_COEFF as f32,
                best_skill_norm,
                alignment,
                sacrifice,
                has_job,
                has_settlement,
                has_partner,
            );

            needs.set(NeedType::Competence, out[0] as f64);
            needs.set(NeedType::Autonomy, out[1] as f64);
            needs.set(NeedType::SelfActualization, out[2] as f64);
            needs.set(NeedType::Meaning, out[3] as f64);
            needs.set(NeedType::Transcendence, out[4] as f64);
            needs.set(NeedType::Recognition, out[5] as f64);
            needs.set(NeedType::Belonging, out[6] as f64);
            needs.set(NeedType::Intimacy, out[7] as f64);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ResourceRegenSystem, UpperNeedsRuntimeSystem};
    use crate::body;
    use hecs::World;
    use sim_core::components::{Behavior, Identity, Needs, SkillEntry, Skills, Social, Values};
    use sim_core::{GameCalendar, GrowthStage, NeedType, ResourceType, SettlementId, ValueType, WorldMap, config::GameConfig};
    use sim_core::world::TileResource;
    use sim_core::ids::EntityId;
    use sim_engine::{SimResources, SimSystem};

    fn make_resources() -> SimResources {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let mut map = WorldMap::new(2, 1, 123);
        map.get_mut(0, 0).resources.push(TileResource {
            resource_type: ResourceType::Food,
            amount: 3.0,
            max_amount: 5.0,
            regen_rate: 0.75,
        });
        map.get_mut(1, 0).resources.push(TileResource {
            resource_type: ResourceType::Wood,
            amount: 4.8,
            max_amount: 5.0,
            regen_rate: 1.0,
        });
        SimResources::new(calendar, map, 999)
    }

    #[test]
    fn resource_regen_system_applies_regen_and_caps_at_max() {
        let mut world = World::new();
        let mut resources = make_resources();
        let mut system = ResourceRegenSystem::new(5, 10);
        system.run(&mut world, &mut resources, 10);

        let first = &resources.map.get(0, 0).resources[0];
        assert!((first.amount - 3.75).abs() < 1e-6);

        let second = &resources.map.get(1, 0).resources[0];
        assert!((second.amount - 5.0).abs() < 1e-6);
    }

    #[test]
    fn upper_needs_runtime_system_updates_upper_need_buckets() {
        let mut world = World::new();
        let mut resources = make_resources();
        let mut needs = Needs::default();
        needs.set(NeedType::Competence, 0.45);
        needs.set(NeedType::Autonomy, 0.52);
        needs.set(NeedType::SelfActualization, 0.33);
        needs.set(NeedType::Meaning, 0.41);
        needs.set(NeedType::Transcendence, 0.29);
        needs.set(NeedType::Recognition, 0.35);
        needs.set(NeedType::Belonging, 0.38);
        needs.set(NeedType::Intimacy, 0.32);

        let mut skills = Skills::default();
        skills
            .entries
            .insert("SKILL_FORAGING".to_string(), SkillEntry { level: 78, xp: 0.0 });
        skills
            .entries
            .insert("SKILL_MINING".to_string(), SkillEntry { level: 40, xp: 0.0 });

        let mut values = Values::default();
        values.set(ValueType::Nature, 0.7);
        values.set(ValueType::Independence, 0.4);
        values.set(ValueType::HardWork, 0.5);
        values.set(ValueType::Sacrifice, 0.2);

        let behavior = Behavior {
            job: "gatherer".to_string(),
            ..Behavior::default()
        };
        let identity = Identity {
            settlement_id: Some(SettlementId(1)),
            growth_stage: GrowthStage::Adult,
            ..Identity::default()
        };
        let social = Social {
            spouse: Some(EntityId(2)),
            ..Social::default()
        };

        let entity = world.spawn((needs, skills, values, behavior, identity, social));
        let mut system = UpperNeedsRuntimeSystem::new(12, 5);
        system.run(&mut world, &mut resources, 5);

        let best_skill_norm = body::upper_needs_best_skill_normalized(&[78, 0, 40, 0, 0], 100);
        let alignment = body::upper_needs_job_alignment(2, 0.0, 0.0, 0.5, 0.7, 0.4);
        let expected = body::upper_needs_step(
            &[0.45, 0.52, 0.33, 0.41, 0.29, 0.35, 0.38, 0.32],
            &[
                sim_core::config::UPPER_NEEDS_COMPETENCE_DECAY as f32,
                sim_core::config::UPPER_NEEDS_AUTONOMY_DECAY as f32,
                sim_core::config::UPPER_NEEDS_SELF_ACTUATION_DECAY as f32,
                sim_core::config::UPPER_NEEDS_MEANING_DECAY as f32,
                sim_core::config::UPPER_NEEDS_TRANSCENDENCE_DECAY as f32,
                sim_core::config::UPPER_NEEDS_RECOGNITION_DECAY as f32,
                sim_core::config::UPPER_NEEDS_BELONGING_DECAY as f32,
                sim_core::config::UPPER_NEEDS_INTIMACY_DECAY as f32,
            ],
            sim_core::config::UPPER_NEEDS_COMPETENCE_JOB_GAIN as f32,
            sim_core::config::UPPER_NEEDS_AUTONOMY_JOB_GAIN as f32,
            sim_core::config::UPPER_NEEDS_BELONGING_SETTLEMENT_GAIN as f32,
            sim_core::config::UPPER_NEEDS_INTIMACY_PARTNER_GAIN as f32,
            sim_core::config::UPPER_NEEDS_RECOGNITION_SKILL_COEFF as f32,
            sim_core::config::UPPER_NEEDS_SELF_ACTUATION_SKILL_COEFF as f32,
            sim_core::config::UPPER_NEEDS_MEANING_BASE_GAIN as f32,
            sim_core::config::UPPER_NEEDS_MEANING_ALIGNED_GAIN as f32,
            sim_core::config::UPPER_NEEDS_TRANSCENDENCE_SETTLEMENT_GAIN as f32,
            sim_core::config::UPPER_NEEDS_TRANSCENDENCE_SACRIFICE_COEFF as f32,
            best_skill_norm,
            alignment,
            0.2,
            true,
            true,
            true,
        );

        let updated = world
            .get::<&Needs>(entity)
            .expect("updated needs component should be queryable");
        assert!((updated.get(NeedType::Competence) as f32 - expected[0]).abs() < 1e-6);
        assert!((updated.get(NeedType::Recognition) as f32 - expected[5]).abs() < 1e-6);
        assert!((updated.get(NeedType::Belonging) as f32 - expected[6]).abs() < 1e-6);
        assert!((updated.get(NeedType::Intimacy) as f32 - expected[7]).abs() < 1e-6);
    }
}
