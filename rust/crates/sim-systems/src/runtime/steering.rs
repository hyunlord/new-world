use hecs::{Entity, World};
use rand::Rng;
use sim_core::components::{Age, Behavior, Personality, Position, SteeringParams};
use sim_core::{config, ActionType};
use sim_engine::{SimResources, SimSystem};

use super::steering_derive::derive_steering_params;

/// Runtime steering system that converts behavior and personality into velocity.
#[derive(Debug, Clone)]
pub struct SteeringRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl SteeringRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for SteeringRuntimeSystem {
    fn name(&self) -> &'static str {
        "steering_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        let snapshots: Vec<(Entity, f64, f64, bool)> = world
            .query::<(&Position, Option<&Age>)>()
            .iter()
            .map(|(entity, (position, age_opt))| {
                (
                    entity,
                    position.x,
                    position.y,
                    age_opt.map(|age| age.alive).unwrap_or(true),
                )
            })
            .collect();

        let mut velocities: Vec<(Entity, f64, f64)> = Vec::new();
        let mut inserts: Vec<(Entity, SteeringParams)> = Vec::new();

        {
            let mut query = world.query::<(
                &Position,
                Option<&Behavior>,
                Option<&SteeringParams>,
                Option<&Personality>,
                Option<&Age>,
            )>();
            for (entity, (position, behavior_opt, steering_opt, personality_opt, age_opt)) in &mut query {
                if age_opt.map(|age| !age.alive).unwrap_or(false) {
                    velocities.push((entity, 0.0, 0.0));
                    continue;
                }
                let params = steering_opt
                    .copied()
                    .or_else(|| personality_opt.map(derive_steering_params))
                    .unwrap_or_default();
                if steering_opt.is_none() {
                    inserts.push((entity, params));
                }

                let neighbors = find_neighbors(&snapshots, entity, position, config::STEERING_NEIGHBOR_RADIUS);
                let desired_force = desired_force_for_action(
                    position,
                    behavior_opt,
                    &params,
                    &mut resources.rng,
                    tick,
                    entity,
                );
                let separation = separation_force(position, &neighbors, params.personal_space_radius);
                let cohesion = cohesion_force(position, &neighbors);

                let mut force_x = desired_force.0
                    + separation.0 * params.separation_weight
                    + cohesion.0 * params.cohesion_weight;
                let mut force_y = desired_force.1
                    + separation.1 * params.separation_weight
                    + cohesion.1 * params.cohesion_weight;

                if let Some(behavior) = behavior_opt {
                    if behavior.action_target_x.is_some() && behavior.action_target_y.is_some() {
                        let direct = seek_force(
                            position,
                            f64::from(behavior.action_target_x.unwrap_or_default()),
                            f64::from(behavior.action_target_y.unwrap_or_default()),
                        );
                        let blend = params.path_directness.clamp(0.0, 1.0);
                        force_x = force_x * (1.0 - blend) + direct.0 * blend;
                        force_y = force_y * (1.0 - blend) + direct.1 * blend;
                    }
                }

                let (force_x, force_y) = clamp_magnitude(force_x, force_y, config::STEERING_MAX_FORCE);
                let speed_px = params.base_speed
                    * params.mood_speed_multiplier
                    * params.stress_speed_multiplier
                    * (1.0
                        + resources
                            .rng
                            .gen_range(-params.speed_variance..=params.speed_variance));
                let speed_tiles = (speed_px / f64::from(config::TILE_SIZE)).clamp(
                    0.0,
                    config::STEERING_MAX_SPEED / f64::from(config::TILE_SIZE),
                );

                let (dir_x, dir_y) = normalize(force_x, force_y);
                velocities.push((entity, dir_x * speed_tiles, dir_y * speed_tiles));
            }
        }

        for (entity, params) in inserts {
            let _ = world.insert_one(entity, params);
        }
        for (entity, vel_x, vel_y) in velocities {
            if let Ok(mut position) = world.get::<&mut Position>(entity) {
                position.vel_x = vel_x;
                position.vel_y = vel_y;
            }
        }
    }
}

fn desired_force_for_action(
    position: &Position,
    behavior_opt: Option<&Behavior>,
    params: &SteeringParams,
    rng: &mut impl Rng,
    tick: u64,
    entity: Entity,
) -> (f64, f64) {
    let Some(behavior) = behavior_opt else {
        return (0.0, 0.0);
    };
    match behavior.current_action {
        ActionType::Idle | ActionType::Rest | ActionType::Sleep => (0.0, 0.0),
        ActionType::Wander => wander_force(position, params, rng, tick, entity),
        ActionType::Flee => {
            let (fx, fy) = wander_force(position, params, rng, tick, entity);
            (-fx * params.flee_multiplier, -fy * params.flee_multiplier)
        }
        _ => {
            let target_x = behavior.action_target_x.map(f64::from);
            let target_y = behavior.action_target_y.map(f64::from);
            match (target_x, target_y) {
                (Some(tx), Some(ty)) => arrive_force(position, tx, ty, params.wander_radius / f64::from(config::TILE_SIZE)),
                _ => wander_force(position, params, rng, tick, entity),
            }
        }
    }
}

fn wander_force(
    position: &Position,
    params: &SteeringParams,
    rng: &mut impl Rng,
    tick: u64,
    entity: Entity,
) -> (f64, f64) {
    let heading = if position.vel_x.abs() + position.vel_y.abs() > 0.001 {
        position.vel_y.atan2(position.vel_x)
    } else {
        ((entity.id() as u64 ^ tick) % 360) as f64 * std::f64::consts::PI / 180.0
    };
    let jitter = rng.gen_range(-1.0..=1.0) * params.wander_jitter.to_radians();
    let angle = heading + jitter;
    let ahead_x = position.x + heading.cos() * (params.wander_distance / f64::from(config::TILE_SIZE));
    let ahead_y = position.y + heading.sin() * (params.wander_distance / f64::from(config::TILE_SIZE));
    let target_x = ahead_x + angle.cos() * (params.wander_radius / f64::from(config::TILE_SIZE));
    let target_y = ahead_y + angle.sin() * (params.wander_radius / f64::from(config::TILE_SIZE));
    seek_force(position, target_x, target_y)
}

fn seek_force(position: &Position, target_x: f64, target_y: f64) -> (f64, f64) {
    let dx = target_x - position.x;
    let dy = target_y - position.y;
    normalize(dx, dy)
}

fn arrive_force(position: &Position, target_x: f64, target_y: f64, slowing_radius: f64) -> (f64, f64) {
    let dx = target_x - position.x;
    let dy = target_y - position.y;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 0.001 {
        return (0.0, 0.0);
    }
    let speed_factor = if dist < slowing_radius.max(0.25) {
        dist / slowing_radius.max(0.25)
    } else {
        1.0
    };
    (dx / dist * speed_factor, dy / dist * speed_factor)
}

fn separation_force(position: &Position, neighbors: &[(f64, f64)], personal_space_radius_px: f64) -> (f64, f64) {
    let personal_space = personal_space_radius_px / f64::from(config::TILE_SIZE);
    let mut fx = 0.0;
    let mut fy = 0.0;
    for (nx, ny) in neighbors {
        let dx = position.x - *nx;
        let dy = position.y - *ny;
        let dist = (dx * dx + dy * dy).sqrt().max(0.001);
        if dist < personal_space {
            let strength = (personal_space - dist) / personal_space.max(0.001);
            fx += dx / dist * strength;
            fy += dy / dist * strength;
        }
    }
    (fx, fy)
}

fn cohesion_force(position: &Position, neighbors: &[(f64, f64)]) -> (f64, f64) {
    if neighbors.is_empty() {
        return (0.0, 0.0);
    }
    let center_x = neighbors.iter().map(|entry| entry.0).sum::<f64>() / neighbors.len() as f64;
    let center_y = neighbors.iter().map(|entry| entry.1).sum::<f64>() / neighbors.len() as f64;
    seek_force(position, center_x, center_y)
}

fn find_neighbors(
    snapshots: &[(Entity, f64, f64, bool)],
    self_entity: Entity,
    self_position: &Position,
    radius_px: f64,
) -> Vec<(f64, f64)> {
    let radius_tiles = radius_px / f64::from(config::TILE_SIZE);
    let radius_sq = radius_tiles * radius_tiles;
    snapshots
        .iter()
        .filter_map(|(entity, x, y, alive)| {
            if *entity == self_entity || !*alive {
                return None;
            }
            let dx = *x - self_position.x;
            let dy = *y - self_position.y;
            if dx * dx + dy * dy <= radius_sq {
                Some((*x, *y))
            } else {
                None
            }
        })
        .collect()
}

fn normalize(x: f64, y: f64) -> (f64, f64) {
    let len = (x * x + y * y).sqrt();
    if len <= 0.0001 {
        (0.0, 0.0)
    } else {
        (x / len, y / len)
    }
}

fn clamp_magnitude(x: f64, y: f64, max_magnitude: f64) -> (f64, f64) {
    let len = (x * x + y * y).sqrt();
    if len <= max_magnitude || len <= 0.0001 {
        (x, y)
    } else {
        let scale = max_magnitude / len;
        (x * scale, y * scale)
    }
}

#[cfg(test)]
mod tests {
    use hecs::World;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use sim_core::components::{Age, Behavior, Personality, Position, SteeringParams};
    use sim_core::{config, ActionType, GameCalendar, GrowthStage, WorldMap};
    use sim_core::config::GameConfig;
    use sim_engine::{SimResources, SimSystem};

    use super::{
        arrive_force, cohesion_force, seek_force, separation_force, wander_force,
        SteeringRuntimeSystem,
    };

    fn resources() -> SimResources {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(32, 32, 7);
        SimResources::new(calendar, map, 11)
    }

    #[test]
    fn separation_force_pushes_away_from_close_neighbors() {
        let position = Position::from_f64(5.0, 5.0);
        let force = separation_force(&position, &[(5.2, 5.0)], 25.0);
        assert!(force.0 < 0.0);
    }

    #[test]
    fn cohesion_force_pulls_toward_neighbor_center() {
        let position = Position::from_f64(1.0, 1.0);
        let force = cohesion_force(&position, &[(3.0, 1.0), (5.0, 1.0)]);
        assert!(force.0 > 0.0);
    }

    #[test]
    fn steering_runtime_system_writes_velocity() {
        let mut world = World::new();
        let mut resources = resources();
        let personality = Personality::default();
        let behavior = Behavior {
            current_action: ActionType::Wander,
            ..Behavior::default()
        };
        let age = Age {
            stage: GrowthStage::Adult,
            ..Age::default()
        };
        let entity = world.spawn((Position::new(5, 5), behavior, personality, SteeringParams::default(), age));
        let mut system = SteeringRuntimeSystem::new(config::STEERING_SYSTEM_PRIORITY, 1);
        system.run(&mut world, &mut resources, 1);
        let position = world.get::<&Position>(entity).expect("position exists");
        assert!(position.vel_x.abs() > 0.0 || position.vel_y.abs() > 0.0);
    }

    #[test]
    fn stage1_wander_force_produces_nonzero_vector() {
        let position = Position {
            x: 100.0,
            y: 100.0,
            vel_x: 1.0,
            vel_y: 0.0,
            movement_dir: 0,
        };
        let params = SteeringParams::default();
        let mut rng = SmallRng::seed_from_u64(7);
        let force = wander_force(&position, &params, &mut rng, 1, hecs::Entity::DANGLING);
        assert!(force.0.abs() + force.1.abs() > 0.0);
    }

    #[test]
    fn stage1_seek_force_points_toward_target() {
        let position = Position::from_f64(0.0, 0.0);
        let force = seek_force(&position, 100.0, 0.0);
        assert!(force.0 > 0.0);
        assert!(force.1.abs() < 0.01);
    }

    #[test]
    fn stage1_arrive_force_slows_near_target() {
        let far_position = Position::from_f64(95.0, 0.0);
        let near_position = Position::from_f64(98.0, 0.0);
        let far_force = arrive_force(&far_position, 200.0, 0.0, 30.0);
        let near_force = arrive_force(&near_position, 100.0, 0.0, 30.0);
        assert!(near_force.0 < far_force.0);
    }

    #[test]
    fn stage1_separation_force_repels_neighbors() {
        let position = Position::from_f64(50.0, 50.0);
        let force = separation_force(&position, &[(50.2, 50.0)], 25.0);
        assert!(force.0 < 0.0);
    }
}
