use hecs::{Entity, World};
use rand::Rng;
use sim_core::components::{
    Age, Behavior, Emotion, Identity, InfluenceReceiver, Needs, Personality, Position,
    SteeringParams, Stress, Temperament,
};
use sim_core::ids::BandId;
use sim_core::{config, ActionType, CauseRef, CausalEvent, ChannelId, EntityId};
use sim_engine::{
    ChronicleEvent, ChronicleEventCause, ChronicleEventMagnitude, ChronicleEventType, SimResources,
    SimSystem,
};

use super::steering_derive::derive_steering_params;

/// Unified influence-driven steering system that converts sampled world signals
/// and local behavior pressure into per-agent velocity.
#[derive(Debug, Clone)]
pub struct InfluenceSteeringSystem {
    priority: u32,
    tick_interval: u64,
}

impl InfluenceSteeringSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

/// Backward-compatible alias for the unified influence steering runtime system.
pub type SteeringRuntimeSystem = InfluenceSteeringSystem;

#[derive(Debug, Clone, Copy)]
struct SteeringSnapshot {
    entity: Entity,
    x: f64,
    y: f64,
    alive: bool,
    band_id: Option<BandId>,
}

#[derive(Debug, Clone, Copy)]
struct NeighborSnapshot {
    x: f64,
    y: f64,
    band_id: Option<BandId>,
}

impl SimSystem for InfluenceSteeringSystem {
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
        let snapshots: Vec<SteeringSnapshot> = world
            .query::<(&Position, Option<&Age>, Option<&Identity>)>()
            .iter()
            .map(|(entity, (position, age_opt, identity_opt))| {
                SteeringSnapshot {
                    entity,
                    x: position.x,
                    y: position.y,
                    alive: age_opt.map(|age| age.alive).unwrap_or(true),
                    band_id: identity_opt.and_then(|identity| identity.band_id),
                }
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
                Option<&Identity>,
                Option<&InfluenceReceiver>,
                Option<&Needs>,
                Option<&Emotion>,
                Option<&Stress>,
                Option<&Temperament>,
                Option<&Age>,
            )>();
            for (
                entity,
                (
                    position,
                    behavior_opt,
                    steering_opt,
                    personality_opt,
                    identity_opt,
                    receiver_opt,
                    needs_opt,
                    emotion_opt,
                    stress_opt,
                    temperament_opt,
                    age_opt,
                ),
            ) in &mut query
            {
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

                let neighbors = find_neighbors(
                    &snapshots,
                    entity,
                    position,
                    config::STEERING_NEIGHBOR_RADIUS,
                );
                let self_band_id = identity_opt.and_then(|identity| identity.band_id);
                let desired_force = desired_force_for_action(
                    position,
                    behavior_opt,
                    &params,
                    &mut resources.rng,
                    tick,
                    entity,
                );
                let influence_decision = influence_decision_for_entity(
                    resources,
                    position,
                    receiver_opt,
                    needs_opt,
                    emotion_opt,
                    stress_opt,
                    temperament_opt,
                );
                let influence_force = influence_decision.force;
                let influence_cause = influence_decision.cause;
                let desired_force = if influence_decision.suppress_action_target_blend {
                    (0.0, 0.0)
                } else {
                    desired_force
                };
                let separation = separation_force(
                    position,
                    &neighbors,
                    params.personal_space_radius,
                    self_band_id,
                    separation_multiplier_for_behavior(behavior_opt),
                );
                let cohesion = cohesion_force(position, &neighbors);
                let band_cohesion = band_cohesion_force(position, behavior_opt);

                let mut force_x = desired_force.0
                    + influence_force.0 * config::STEERING_INFLUENCE_FORCE_WEIGHT
                    + separation.0 * params.separation_weight
                    + cohesion.0 * params.cohesion_weight
                    + band_cohesion.0 * config::BAND_COHESION_WEIGHT;
                let mut force_y = desired_force.1
                    + influence_force.1 * config::STEERING_INFLUENCE_FORCE_WEIGHT
                    + separation.1 * params.separation_weight
                    + cohesion.1 * params.cohesion_weight
                    + band_cohesion.1 * config::BAND_COHESION_WEIGHT;

                if !influence_decision.suppress_action_target_blend {
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
                }

                let (force_x, force_y) =
                    clamp_magnitude(force_x, force_y, config::STEERING_MAX_FORCE);
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
                if let Some(cause) = influence_cause {
                    resources.causal_log.push(
                        EntityId(entity.id() as u64),
                        CausalEvent {
                            tick,
                            cause: CauseRef {
                                system: "steering_system".to_string(),
                                kind: cause.kind.to_string(),
                                entity: Some(EntityId(entity.id() as u64)),
                                building: None,
                                settlement: None,
                            },
                            effect_key: "steering_velocity".to_string(),
                            summary_key: cause.summary_key.to_string(),
                            magnitude: cause.magnitude,
                        },
                    );
                    if let Some(event) = chronicle_event_for_decision(
                        tick,
                        EntityId(entity.id() as u64),
                        position,
                        cause,
                        (dir_x * speed_tiles, dir_y * speed_tiles),
                    ) {
                        resources.chronicle_log.append_event(event);
                    }
                }
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

#[derive(Debug, Clone, Copy)]
struct InfluenceCause {
    channel: ChannelId,
    event_type: ChronicleEventType,
    kind: &'static str,
    summary_key: &'static str,
    magnitude: f64,
}

#[derive(Debug, Clone, Copy, Default)]
struct SteeringSignalSample {
    signal: f64,
    weight: f64,
    force: (f64, f64),
}

impl SteeringSignalSample {
    fn magnitude(self) -> f64 {
        (self.force.0 * self.force.0 + self.force.1 * self.force.1).sqrt()
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct SteeringWeights {
    hunger: f64,
    fear: f64,
    cold: f64,
    loneliness: f64,
    novelty_scale: f64,
    fear_scale: f64,
    warmth_scale: f64,
}

#[derive(Debug, Clone, Copy, Default)]
struct AgentSteeringContext {
    food: SteeringSignalSample,
    warmth: SteeringSignalSample,
    shelter: SteeringSignalSample,
    social: SteeringSignalSample,
    danger: SteeringSignalSample,
}

#[derive(Debug, Clone, Copy, Default)]
struct InfluenceSteeringDecision {
    force: (f64, f64),
    cause: Option<InfluenceCause>,
    suppress_action_target_blend: bool,
}

#[cfg(test)]
fn influence_force_for_entity(
    resources: &SimResources,
    position: &Position,
    receiver_opt: Option<&InfluenceReceiver>,
    needs_opt: Option<&Needs>,
    emotion_opt: Option<&Emotion>,
    stress_opt: Option<&Stress>,
    temperament_opt: Option<&Temperament>,
) -> ((f64, f64), Option<InfluenceCause>) {
    let decision = influence_decision_for_entity(
        resources,
        position,
        receiver_opt,
        needs_opt,
        emotion_opt,
        stress_opt,
        temperament_opt,
    );
    (decision.force, decision.cause)
}

fn influence_decision_for_entity(
    resources: &SimResources,
    position: &Position,
    receiver_opt: Option<&InfluenceReceiver>,
    needs_opt: Option<&Needs>,
    emotion_opt: Option<&Emotion>,
    stress_opt: Option<&Stress>,
    temperament_opt: Option<&Temperament>,
) -> InfluenceSteeringDecision {
    let tile_x = position.tile_x();
    let tile_y = position.tile_y();
    if !resources.map.in_bounds(tile_x, tile_y) {
        return InfluenceSteeringDecision::default();
    }
    let x = tile_x as u32;
    let y = tile_y as u32;
    let weights = steering_weights_for_entity(needs_opt, emotion_opt, stress_opt, temperament_opt);
    let context = build_steering_context(resources, x, y, receiver_opt, weights);
    resolve_influence_steering(context)
}

fn steering_weights_for_entity(
    needs_opt: Option<&Needs>,
    emotion_opt: Option<&Emotion>,
    stress_opt: Option<&Stress>,
    temperament_opt: Option<&Temperament>,
) -> SteeringWeights {
    let hunger = needs_opt
        .map(|needs| 1.0 - needs.get(sim_core::NeedType::Hunger))
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    let cold = needs_opt
        .map(|needs| 1.0 - needs.get(sim_core::NeedType::Warmth))
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    let loneliness = needs_opt
        .map(|needs| 1.0 - needs.get(sim_core::NeedType::Belonging))
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    let safety = needs_opt
        .map(|needs| 1.0 - needs.get(sim_core::NeedType::Safety))
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    let fear = emotion_opt
        .map(|emotion| emotion.get(sim_core::EmotionType::Fear))
        .unwrap_or(0.0)
        .max(stress_opt.map(|stress| stress.level).unwrap_or(0.0))
        .max(safety)
        .clamp(0.0, 1.0);
    let novelty_scale = temperament_opt
        .map(|temperament| 0.75 + temperament.expressed.ns * 0.5)
        .unwrap_or(1.0);
    let fear_scale = temperament_opt
        .map(|temperament| 0.75 + temperament.expressed.ha * 0.75)
        .unwrap_or(1.0);
    let warmth_scale = temperament_opt
        .map(|temperament| 0.75 + temperament.expressed.rd * 0.5)
        .unwrap_or(1.0);

    SteeringWeights {
        hunger,
        fear,
        cold,
        loneliness,
        novelty_scale,
        fear_scale,
        warmth_scale,
    }
}

fn build_steering_context(
    resources: &SimResources,
    x: u32,
    y: u32,
    receiver_opt: Option<&InfluenceReceiver>,
    weights: SteeringWeights,
) -> AgentSteeringContext {
    let social_weight = if weights.loneliness >= config::STEERING_SOCIAL_MIN_LONELINESS {
        weights.loneliness * config::STEERING_SOCIAL_INFLUENCE_WEIGHT
    } else {
        0.0
    };

    AgentSteeringContext {
        food: weighted_channel_sample_with_sign(
            resources,
            x,
            y,
            receiver_opt,
            ChannelId::Food,
            weights.hunger * config::STEERING_HUNGER_INFLUENCE_WEIGHT * weights.novelty_scale,
            1.0,
        ),
        warmth: weighted_channel_sample_with_sign(
            resources,
            x,
            y,
            receiver_opt,
            ChannelId::Warmth,
            weights.cold * config::STEERING_WARMTH_INFLUENCE_WEIGHT * weights.warmth_scale,
            1.0,
        ),
        shelter: room_shelter_sample(
            resources,
            x,
            y,
            receiver_opt,
            weights.cold,
            weights.warmth_scale,
        ),
        social: weighted_channel_sample_with_sign(
            resources,
            x,
            y,
            receiver_opt,
            ChannelId::Social,
            social_weight,
            1.0,
        ),
        danger: weighted_channel_sample_with_sign(
            resources,
            x,
            y,
            receiver_opt,
            ChannelId::Danger,
            weights.fear * config::STEERING_DANGER_INFLUENCE_WEIGHT * weights.fear_scale,
            -1.0,
        ),
    }
}

fn resolve_influence_steering(context: AgentSteeringContext) -> InfluenceSteeringDecision {
    let danger_overrides = context.danger.signal >= config::STEERING_DANGER_PRIORITY_SIGNAL_THRESHOLD
        && context.danger.weight > 0.0
        && context.danger.magnitude() >= config::STEERING_INFLUENCE_MIN_GRADIENT;

    let force = if danger_overrides {
        context.danger.force
    } else {
        (
            context.food.force.0
                + context.warmth.force.0
                + context.shelter.force.0
                + context.social.force.0
                + context.danger.force.0,
            context.food.force.1
                + context.warmth.force.1
                + context.shelter.force.1
                + context.social.force.1
                + context.danger.force.1,
        )
    };

    InfluenceSteeringDecision {
        force,
        cause: dominant_influence_cause(context, danger_overrides),
        suppress_action_target_blend: danger_overrides,
    }
}

fn weighted_channel_sample_with_sign(
    resources: &SimResources,
    x: u32,
    y: u32,
    receiver_opt: Option<&InfluenceReceiver>,
    channel: ChannelId,
    weight: f64,
    gradient_sign: f64,
) -> SteeringSignalSample {
    if weight <= 0.0 || !receiver_listens_to(receiver_opt, channel) {
        return SteeringSignalSample::default();
    }
    let signal = resources.influence_grid.sample(x, y, channel).max(0.0);
    let gradient = resources.influence_grid.sample_gradient(channel, x, y);
    if gradient.0.abs() + gradient.1.abs() < config::STEERING_INFLUENCE_MIN_GRADIENT {
        return SteeringSignalSample {
            signal,
            weight,
            force: (0.0, 0.0),
        };
    }
    SteeringSignalSample {
        signal,
        weight,
        force: (
            gradient.0 * weight * gradient_sign,
            gradient.1 * weight * gradient_sign,
        ),
    }
}

#[cfg(test)]
fn room_shelter_force(
    resources: &SimResources,
    x: u32,
    y: u32,
    receiver_opt: Option<&InfluenceReceiver>,
    warmth_drive: f64,
    warmth_scale: f64,
) -> (f64, f64) {
    room_shelter_sample(resources, x, y, receiver_opt, warmth_drive, warmth_scale).force
}

fn room_shelter_sample(
    resources: &SimResources,
    x: u32,
    y: u32,
    receiver_opt: Option<&InfluenceReceiver>,
    warmth_drive: f64,
    warmth_scale: f64,
) -> SteeringSignalSample {
    if warmth_drive < config::STEERING_SHELTER_MIN_COLD_PRESSURE
        || !receiver_listens_to(receiver_opt, ChannelId::Warmth)
    {
        return SteeringSignalSample::default();
    }

    let current_score = shelter_tile_score(resources, x, y);
    let mut best_candidate: Option<(u32, u32, f64)> = None;

    for (next_x, next_y) in orthogonal_tile_candidates(resources, x, y) {
        let score = shelter_tile_score(resources, next_x, next_y);
        if score <= current_score + config::STEERING_SHELTER_ROOM_MIN_SCORE_DELTA {
            continue;
        }
        match best_candidate {
            Some((_, _, best_score)) if score <= best_score => {}
            _ => best_candidate = Some((next_x, next_y, score)),
        }
    }

    let Some((best_x, best_y, best_score)) = best_candidate else {
        return SteeringSignalSample::default();
    };

    let gradient = (
        f64::from(best_x) - f64::from(x),
        f64::from(best_y) - f64::from(y),
    );
    let (dir_x, dir_y) = normalize(gradient.0, gradient.1);
    let magnitude = ((best_score - current_score)
        * warmth_drive
        * warmth_scale
        * config::STEERING_SHELTER_ROOM_BIAS_WEIGHT)
        .clamp(0.0, config::STEERING_SHELTER_ROOM_BIAS_WEIGHT);
    SteeringSignalSample {
        signal: (best_score - current_score).max(0.0),
        weight: magnitude,
        force: (dir_x * magnitude, dir_y * magnitude),
    }
}

fn shelter_tile_score(resources: &SimResources, x: u32, y: u32) -> f64 {
    if !resources.map.in_bounds(x as i32, y as i32) {
        return 0.0;
    }
    let warmth = resources.influence_grid.sample(x, y, ChannelId::Warmth).max(0.0);
    if warmth <= 0.0 {
        return 0.0;
    }
    let room_multiplier = if resources.tile_grid.get(x, y).room_id.is_some() {
        config::STEERING_SHELTER_ROOM_WARMTH_MULTIPLIER
    } else {
        1.0
    };
    warmth * room_multiplier
}

fn orthogonal_tile_candidates(resources: &SimResources, x: u32, y: u32) -> [(u32, u32); 4] {
    let center_x = x as i32;
    let center_y = y as i32;
    let mut candidates = [(x, y); 4];
    let offsets = [(0, -1), (1, 0), (0, 1), (-1, 0)];
    for (index, (dx, dy)) in offsets.into_iter().enumerate() {
        let next_x = center_x + dx;
        let next_y = center_y + dy;
        candidates[index] = if resources.map.in_bounds(next_x, next_y)
            && resources.map.get(next_x as u32, next_y as u32).passable
        {
            (next_x as u32, next_y as u32)
        } else {
            (x, y)
        };
    }
    candidates
}

fn receiver_listens_to(receiver_opt: Option<&InfluenceReceiver>, channel: ChannelId) -> bool {
    receiver_opt
        .map(|receiver| receiver.listens_to(channel))
        .unwrap_or(true)
}

fn dominant_influence_cause(
    context: AgentSteeringContext,
    danger_overrides: bool,
) -> Option<InfluenceCause> {
    if danger_overrides {
        return influence_cause_from_sample(
            ChannelId::Danger,
            ChronicleEventType::InfluenceAvoidance,
            "danger_gradient",
            "CAUSE_INFLUENCE_DANGER_GRADIENT",
            context.danger,
        );
    }

    let candidates = [
        (
            ChannelId::Food,
            ChronicleEventType::InfluenceAttraction,
            "food_gradient",
            "CAUSE_INFLUENCE_FOOD_GRADIENT",
            context.food.magnitude(),
        ),
        (
            ChannelId::Warmth,
            ChronicleEventType::ShelterSeeking,
            "warmth_gradient",
            "CAUSE_INFLUENCE_WARMTH_GRADIENT",
            context.warmth.magnitude(),
        ),
        (
            ChannelId::Warmth,
            ChronicleEventType::ShelterSeeking,
            "shelter_gradient",
            "CAUSE_INFLUENCE_SHELTER_GRADIENT",
            context.shelter.magnitude(),
        ),
        (
            ChannelId::Social,
            ChronicleEventType::GatheringFormation,
            "social_gradient",
            "CAUSE_INFLUENCE_SOCIAL_GRADIENT",
            context.social.magnitude(),
        ),
        (
            ChannelId::Danger,
            ChronicleEventType::InfluenceAvoidance,
            "danger_gradient",
            "CAUSE_INFLUENCE_DANGER_GRADIENT",
            context.danger.magnitude(),
        ),
    ];
    let best = candidates
        .into_iter()
        .max_by(|left, right| left.4.partial_cmp(&right.4).unwrap_or(std::cmp::Ordering::Equal))?;
    if best.4 < config::STEERING_INFLUENCE_MIN_GRADIENT {
        return None;
    }
    Some(InfluenceCause {
        channel: best.0,
        event_type: best.1,
        kind: best.2,
        summary_key: best.3,
        magnitude: best.4,
    })
}

fn influence_cause_from_sample(
    channel: ChannelId,
    event_type: ChronicleEventType,
    kind: &'static str,
    summary_key: &'static str,
    sample: SteeringSignalSample,
) -> Option<InfluenceCause> {
    let magnitude = sample.magnitude();
    if magnitude < config::STEERING_INFLUENCE_MIN_GRADIENT {
        return None;
    }
    Some(InfluenceCause {
        channel,
        event_type,
        kind,
        summary_key,
        magnitude,
    })
}

fn chronicle_event_for_decision(
    tick: u64,
    entity_id: EntityId,
    position: &Position,
    cause: InfluenceCause,
    final_velocity: (f64, f64),
) -> Option<ChronicleEvent> {
    let steering_magnitude = vector_magnitude(final_velocity.0, final_velocity.1);
    let significance = cause.magnitude.max(steering_magnitude);
    if significance < config::CHRONICLE_SIGNIFICANCE_THRESHOLD {
        return None;
    }

    Some(ChronicleEvent {
        tick,
        entity_id,
        event_type: cause.event_type,
        cause: ChronicleEventCause::from(cause.channel),
        magnitude: ChronicleEventMagnitude {
            influence: cause.magnitude,
            steering: steering_magnitude,
            significance,
        },
        tile_x: position.tile_x(),
        tile_y: position.tile_y(),
        summary_key: cause.summary_key.to_string(),
        effect_key: "steering_velocity".to_string(),
    })
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
                (Some(tx), Some(ty)) => arrive_force(
                    position,
                    tx,
                    ty,
                    params.wander_radius / f64::from(config::TILE_SIZE),
                ),
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
    let ahead_x =
        position.x + heading.cos() * (params.wander_distance / f64::from(config::TILE_SIZE));
    let ahead_y =
        position.y + heading.sin() * (params.wander_distance / f64::from(config::TILE_SIZE));
    let target_x = ahead_x + angle.cos() * (params.wander_radius / f64::from(config::TILE_SIZE));
    let target_y = ahead_y + angle.sin() * (params.wander_radius / f64::from(config::TILE_SIZE));
    seek_force(position, target_x, target_y)
}

fn seek_force(position: &Position, target_x: f64, target_y: f64) -> (f64, f64) {
    let dx = target_x - position.x;
    let dy = target_y - position.y;
    normalize(dx, dy)
}

fn arrive_force(
    position: &Position,
    target_x: f64,
    target_y: f64,
    slowing_radius: f64,
) -> (f64, f64) {
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

fn separation_force(
    position: &Position,
    neighbors: &[NeighborSnapshot],
    personal_space_radius_px: f64,
    self_band_id: Option<BandId>,
    outsider_separation_mult: f64,
) -> (f64, f64) {
    let personal_space = personal_space_radius_px / f64::from(config::TILE_SIZE);
    let mut fx = 0.0;
    let mut fy = 0.0;
    for neighbor in neighbors {
        let dx = position.x - neighbor.x;
        let dy = position.y - neighbor.y;
        let dist = (dx * dx + dy * dy).sqrt().max(0.001);
        if dist < personal_space {
            let is_outsider = self_band_id.is_some() && neighbor.band_id != self_band_id;
            let band_mult = if is_outsider {
                outsider_separation_mult.max(1.0)
            } else {
                1.0
            };
            let strength = ((personal_space - dist) / personal_space.max(0.001)) * band_mult;
            fx += dx / dist * strength;
            fy += dy / dist * strength;
        }
    }
    (fx, fy)
}

fn cohesion_force(position: &Position, neighbors: &[NeighborSnapshot]) -> (f64, f64) {
    if neighbors.is_empty() {
        return (0.0, 0.0);
    }
    let center_x = neighbors.iter().map(|entry| entry.x).sum::<f64>() / neighbors.len() as f64;
    let center_y = neighbors.iter().map(|entry| entry.y).sum::<f64>() / neighbors.len() as f64;
    seek_force(position, center_x, center_y)
}

fn band_cohesion_force(position: &Position, behavior_opt: Option<&Behavior>) -> (f64, f64) {
    let Some(behavior) = behavior_opt else {
        return (0.0, 0.0);
    };
    let (Some(center_x), Some(center_y)) = (behavior.band_center_x, behavior.band_center_y) else {
        return (0.0, 0.0);
    };
    seek_force(position, center_x, center_y)
}

fn separation_multiplier_for_behavior(behavior_opt: Option<&Behavior>) -> f64 {
    behavior_opt
        .map(|behavior| behavior.outsider_separation_mult.max(1.0))
        .unwrap_or(1.0)
}

fn find_neighbors(
    snapshots: &[SteeringSnapshot],
    self_entity: Entity,
    self_position: &Position,
    radius_px: f64,
) -> Vec<NeighborSnapshot> {
    let radius_tiles = radius_px / f64::from(config::TILE_SIZE);
    let radius_sq = radius_tiles * radius_tiles;
    snapshots
        .iter()
        .filter_map(|snapshot| {
            if snapshot.entity == self_entity || !snapshot.alive {
                return None;
            }
            let dx = snapshot.x - self_position.x;
            let dy = snapshot.y - self_position.y;
            if dx * dx + dy * dy <= radius_sq {
                Some(NeighborSnapshot {
                    x: snapshot.x,
                    y: snapshot.y,
                    band_id: snapshot.band_id,
                })
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

fn vector_magnitude(x: f64, y: f64) -> f64 {
    (x * x + y * y).sqrt()
}

#[cfg(test)]
mod tests {
    use crate::runtime::MovementRuntimeSystem;
    use hecs::World;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use sim_core::components::{
        Age, Behavior, Emotion, InfluenceReceiver, Needs, Personality, Position, SteeringParams,
        Stress, Temperament,
    };
    use sim_core::config::GameConfig;
    use sim_core::{
        config, ActionType, BandId, ChannelId, EmitterRecord, FalloffType, GameCalendar,
        GrowthStage, NeedType, RoomId, WorldMap,
    };
    use sim_engine::{ChronicleEventType, SimResources, SimSystem};

    use super::{
        arrive_force, band_cohesion_force, chronicle_event_for_decision, cohesion_force,
        influence_force_for_entity, room_shelter_force, seek_force, separation_force,
        wander_force, InfluenceCause, NeighborSnapshot, SteeringRuntimeSystem,
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
        let force = separation_force(
            &position,
            &[NeighborSnapshot {
                x: 5.2,
                y: 5.0,
                band_id: None,
            }],
            25.0,
            None,
            1.0,
        );
        assert!(force.0 < 0.0);
    }

    #[test]
    fn cohesion_force_pulls_toward_neighbor_center() {
        let position = Position::from_f64(1.0, 1.0);
        let force = cohesion_force(
            &position,
            &[
                NeighborSnapshot {
                    x: 3.0,
                    y: 1.0,
                    band_id: None,
                },
                NeighborSnapshot {
                    x: 5.0,
                    y: 1.0,
                    band_id: None,
                },
            ],
        );
        assert!(force.0 > 0.0);
    }

    #[test]
    fn band_cohesion_force_steers_toward_band_center() {
        let position = Position::from_f64(1.0, 1.0);
        let behavior = Behavior {
            band_center_x: Some(4.0),
            band_center_y: Some(1.0),
            ..Behavior::default()
        };
        let force = band_cohesion_force(&position, Some(&behavior));
        assert!(force.0 > 0.0);
        assert!(force.1.abs() < 1e-6);
    }

    #[test]
    fn outsider_separation_multiplier_does_not_apply_to_same_band_members() {
        let position = Position::from_f64(5.0, 5.0);
        let same_band_force = separation_force(
            &position,
            &[NeighborSnapshot {
                x: 5.2,
                y: 5.0,
                band_id: Some(BandId(1)),
            }],
            25.0,
            Some(BandId(1)),
            1.5,
        );
        let outsider_force = separation_force(
            &position,
            &[NeighborSnapshot {
                x: 5.2,
                y: 5.0,
                band_id: Some(BandId(2)),
            }],
            25.0,
            Some(BandId(1)),
            1.5,
        );

        assert!(outsider_force.0 < same_band_force.0);
        assert!(outsider_force.0.abs() > same_band_force.0.abs());
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
        let entity = world.spawn((
            Position::new(5, 5),
            behavior,
            personality,
            SteeringParams::default(),
            age,
        ));
        let mut system = SteeringRuntimeSystem::new(config::STEERING_SYSTEM_PRIORITY, 1);
        system.run(&mut world, &mut resources, 1);
        let position = world.get::<&Position>(entity).expect("position exists");
        assert!(position.vel_x.abs() > 0.0 || position.vel_y.abs() > 0.0);
    }

    #[test]
    fn chronicle_event_for_decision_filters_low_significance_force() {
        let cause = InfluenceCause {
            channel: ChannelId::Food,
            event_type: ChronicleEventType::InfluenceAttraction,
            kind: "food_gradient",
            summary_key: "CAUSE_INFLUENCE_FOOD_GRADIENT",
            magnitude: config::CHRONICLE_SIGNIFICANCE_THRESHOLD * 0.5,
        };

        let event = chronicle_event_for_decision(
            12,
            sim_core::EntityId(9),
            &Position::new(4, 4),
            cause,
            (0.0, 0.0),
        );

        assert!(event.is_none());
    }

    #[test]
    fn steering_runtime_system_appends_food_chronicle_event() {
        let mut world = World::new();
        let mut resources = resources();
        resources.influence_grid.register_emitter(EmitterRecord {
            x: 9,
            y: 5,
            channel: ChannelId::Food,
            radius: 5.0,
            base_intensity: 0.9,
            falloff: FalloffType::Linear,
            decay_rate: None,
            tags: Vec::new(),
            dirty: true,
        });
        resources.influence_grid.tick_update();

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.05);
        let entity = world.spawn((
            Position::new(5, 5),
            Behavior::default(),
            Personality::default(),
            SteeringParams::default(),
            InfluenceReceiver::default(),
            needs,
            Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            },
        ));

        let mut system = SteeringRuntimeSystem::new(config::STEERING_SYSTEM_PRIORITY, 1);
        system.run(&mut world, &mut resources, 20);

        let event = resources
            .chronicle_log
            .latest_for_entity(sim_core::EntityId(entity.id() as u64))
            .expect("food chronicle event");
        assert_eq!(event.event_type, ChronicleEventType::InfluenceAttraction);
        assert_eq!(event.cause.id(), "food");
        assert!(event.magnitude.significance >= config::CHRONICLE_SIGNIFICANCE_THRESHOLD);
    }

    #[test]
    fn steering_runtime_system_records_danger_as_dominant_chronicle_cause() {
        let mut world = World::new();
        let mut resources = resources();
        resources.influence_grid.replace_emitters(vec![
            EmitterRecord {
                x: 9,
                y: 5,
                channel: ChannelId::Food,
                radius: 6.0,
                base_intensity: 0.9,
                falloff: FalloffType::Linear,
                decay_rate: None,
                tags: Vec::new(),
                dirty: true,
            },
            EmitterRecord {
                x: 6,
                y: 5,
                channel: ChannelId::Danger,
                radius: 6.0,
                base_intensity: 1.0,
                falloff: FalloffType::Exponential,
                decay_rate: None,
                tags: Vec::new(),
                dirty: true,
            },
        ]);
        resources.influence_grid.tick_update();

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.05);
        needs.set(NeedType::Safety, 0.15);
        let mut emotion = Emotion::default();
        *emotion.get_mut(sim_core::EmotionType::Fear) = 0.95;
        let entity = world.spawn((
            Position::new(5, 5),
            Behavior::default(),
            Personality::default(),
            SteeringParams::default(),
            InfluenceReceiver::default(),
            needs,
            emotion,
            Stress {
                level: 0.75,
                ..Stress::default()
            },
            Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            },
        ));

        let mut system = SteeringRuntimeSystem::new(config::STEERING_SYSTEM_PRIORITY, 1);
        system.run(&mut world, &mut resources, 30);

        let event = resources
            .chronicle_log
            .latest_for_entity(sim_core::EntityId(entity.id() as u64))
            .expect("danger chronicle event");
        assert_eq!(event.event_type, ChronicleEventType::InfluenceAvoidance);
        assert_eq!(event.cause.id(), "danger");
        assert!(event.magnitude.steering > 0.0);
    }

    #[test]
    fn steering_runtime_system_clamps_velocity_under_combined_influences() {
        let mut world = World::new();
        let mut resources = resources();
        resources.influence_grid.replace_emitters(vec![
            EmitterRecord {
                x: 12,
                y: 8,
                channel: ChannelId::Food,
                radius: 8.0,
                base_intensity: 1.0,
                falloff: FalloffType::Gaussian,
                decay_rate: None,
                tags: Vec::new(),
                dirty: true,
            },
            EmitterRecord {
                x: 9,
                y: 8,
                channel: ChannelId::Danger,
                radius: 6.0,
                base_intensity: 1.0,
                falloff: FalloffType::Exponential,
                decay_rate: None,
                tags: Vec::new(),
                dirty: true,
            },
            EmitterRecord {
                x: 8,
                y: 12,
                channel: ChannelId::Warmth,
                radius: 8.0,
                base_intensity: 1.0,
                falloff: FalloffType::Linear,
                decay_rate: None,
                tags: Vec::new(),
                dirty: true,
            },
            EmitterRecord {
                x: 8,
                y: 4,
                channel: ChannelId::Social,
                radius: 8.0,
                base_intensity: 1.0,
                falloff: FalloffType::Linear,
                decay_rate: None,
                tags: Vec::new(),
                dirty: true,
            },
        ]);
        resources.influence_grid.tick_update();

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.10);
        needs.set(NeedType::Warmth, 0.10);
        needs.set(NeedType::Belonging, 0.10);
        needs.set(NeedType::Safety, 0.10);

        let mut emotion = Emotion::default();
        *emotion.get_mut(sim_core::EmotionType::Fear) = 0.90;

        let entity = world.spawn((
            Position::new(8, 8),
            Behavior::default(),
            InfluenceReceiver::default(),
            needs,
            emotion,
            Stress::default(),
            Temperament::default(),
            SteeringParams {
                base_speed: config::STEERING_MAX_SPEED * 10.0,
                speed_variance: 0.0,
                ..SteeringParams::default()
            },
            Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            },
        ));

        let mut system = SteeringRuntimeSystem::new(config::STEERING_SYSTEM_PRIORITY, 1);
        system.run(&mut world, &mut resources, 1);

        let position = world.get::<&Position>(entity).expect("position exists");
        let speed = (position.vel_x * position.vel_x + position.vel_y * position.vel_y).sqrt();
        let max_speed_tiles = config::STEERING_MAX_SPEED / f64::from(config::TILE_SIZE);
        assert!(speed <= max_speed_tiles + 1e-6);
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
        let force = separation_force(
            &position,
            &[NeighborSnapshot {
                x: 50.2,
                y: 50.0,
                band_id: None,
            }],
            25.0,
            None,
            1.0,
        );
        assert!(force.0 < 0.0);
    }

    #[test]
    fn influence_force_moves_hungry_agents_toward_food_gradient() {
        let mut resources = resources();
        resources.influence_grid.register_emitter(EmitterRecord {
            x: 10,
            y: 8,
            channel: ChannelId::Food,
            radius: 4.0,
            base_intensity: 0.8,
            falloff: FalloffType::Gaussian,
            decay_rate: None,
            tags: Vec::new(),
            dirty: true,
        });
        resources.influence_grid.tick_update();

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.1);
        let (force, cause) = influence_force_for_entity(
            &resources,
            &Position::new(6, 8),
            Some(&InfluenceReceiver::default()),
            Some(&needs),
            None,
            None,
            Some(&Temperament::default()),
        );

        assert!(force.0 > 0.0);
        assert_eq!(cause.expect("food cause").kind, "food_gradient");
    }

    #[test]
    fn influence_force_avoids_danger_when_safety_is_low() {
        let mut resources = resources();
        resources.influence_grid.register_emitter(EmitterRecord {
            x: 10,
            y: 8,
            channel: ChannelId::Danger,
            radius: 4.0,
            base_intensity: 0.8,
            falloff: FalloffType::Exponential,
            decay_rate: None,
            tags: Vec::new(),
            dirty: true,
        });
        resources.influence_grid.tick_update();

        let mut needs = Needs::default();
        needs.set(NeedType::Safety, 0.1);
        let mut stress = Stress::default();
        stress.level = 0.8;
        let (force, cause) = influence_force_for_entity(
            &resources,
            &Position::new(8, 8),
            Some(&InfluenceReceiver::default()),
            Some(&needs),
            None,
            Some(&stress),
            Some(&Temperament::default()),
        );

        assert!(force.0 < 0.0);
        assert_eq!(cause.expect("danger cause").kind, "danger_gradient");
    }

    #[test]
    fn influence_force_scales_with_hunger_pressure() {
        let mut resources = resources();
        resources.influence_grid.register_emitter(EmitterRecord {
            x: 10,
            y: 8,
            channel: ChannelId::Food,
            radius: 4.0,
            base_intensity: 0.8,
            falloff: FalloffType::Gaussian,
            decay_rate: None,
            tags: Vec::new(),
            dirty: true,
        });
        resources.influence_grid.tick_update();

        let mut high_hunger = Needs::default();
        high_hunger.set(NeedType::Hunger, 0.1);
        let mut low_hunger = Needs::default();
        low_hunger.set(NeedType::Hunger, 0.85);

        let (high_force, _) = influence_force_for_entity(
            &resources,
            &Position::new(6, 8),
            Some(&InfluenceReceiver::default()),
            Some(&high_hunger),
            None,
            None,
            Some(&Temperament::default()),
        );
        let (low_force, _) = influence_force_for_entity(
            &resources,
            &Position::new(6, 8),
            Some(&InfluenceReceiver::default()),
            Some(&low_hunger),
            None,
            None,
            Some(&Temperament::default()),
        );

        let high_magnitude = (high_force.0 * high_force.0 + high_force.1 * high_force.1).sqrt();
        let low_magnitude = (low_force.0 * low_force.0 + low_force.1 * low_force.1).sqrt();
        assert!(high_magnitude > low_magnitude);
    }

    #[test]
    fn influence_force_scales_with_fear_pressure() {
        let mut resources = resources();
        resources.influence_grid.register_emitter(EmitterRecord {
            x: 10,
            y: 8,
            channel: ChannelId::Danger,
            radius: 4.0,
            base_intensity: 0.8,
            falloff: FalloffType::Exponential,
            decay_rate: None,
            tags: Vec::new(),
            dirty: true,
        });
        resources.influence_grid.tick_update();

        let mut low_fear = Emotion::default();
        *low_fear.get_mut(sim_core::EmotionType::Fear) = 0.05;
        let mut high_fear = Emotion::default();
        *high_fear.get_mut(sim_core::EmotionType::Fear) = 0.95;

        let (low_force, _) = influence_force_for_entity(
            &resources,
            &Position::new(8, 8),
            Some(&InfluenceReceiver::default()),
            None,
            Some(&low_fear),
            None,
            Some(&Temperament::default()),
        );
        let (high_force, _) = influence_force_for_entity(
            &resources,
            &Position::new(8, 8),
            Some(&InfluenceReceiver::default()),
            None,
            Some(&high_fear),
            None,
            Some(&Temperament::default()),
        );

        let low_magnitude = (low_force.0 * low_force.0 + low_force.1 * low_force.1).sqrt();
        let high_magnitude = (high_force.0 * high_force.0 + high_force.1 * high_force.1).sqrt();
        assert!(high_magnitude > low_magnitude);
    }

    #[test]
    fn influence_force_has_no_false_food_attraction_without_signal() {
        let resources = resources();
        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.05);

        let (force, cause) = influence_force_for_entity(
            &resources,
            &Position::new(6, 8),
            Some(&InfluenceReceiver::default()),
            Some(&needs),
            None,
            None,
            Some(&Temperament::default()),
        );

        assert_eq!(force, (0.0, 0.0));
        assert!(cause.is_none());
    }

    #[test]
    fn influence_force_has_no_false_danger_avoidance_without_signal() {
        let resources = resources();
        let mut fear = Emotion::default();
        *fear.get_mut(sim_core::EmotionType::Fear) = 0.95;

        let (force, cause) = influence_force_for_entity(
            &resources,
            &Position::new(8, 8),
            Some(&InfluenceReceiver::default()),
            None,
            Some(&fear),
            None,
            Some(&Temperament::default()),
        );

        assert_eq!(force, (0.0, 0.0));
        assert!(cause.is_none());
    }

    #[test]
    fn influence_force_moves_lonely_agents_toward_social_gradient() {
        let mut resources = resources();
        resources.influence_grid.register_emitter(EmitterRecord {
            x: 10,
            y: 8,
            channel: ChannelId::Social,
            radius: 4.0,
            base_intensity: 0.7,
            falloff: FalloffType::Linear,
            decay_rate: None,
            tags: Vec::new(),
            dirty: true,
        });
        resources.influence_grid.tick_update();

        let mut needs = Needs::default();
        needs.set(NeedType::Belonging, 0.10);
        let (force, cause) = influence_force_for_entity(
            &resources,
            &Position::new(6, 8),
            Some(&InfluenceReceiver::default()),
            Some(&needs),
            None,
            None,
            Some(&Temperament::default()),
        );

        assert!(force.0 > 0.0);
        assert_eq!(cause.expect("social cause").kind, "social_gradient");
    }

    #[test]
    fn influence_force_ignores_social_gradient_when_belonging_is_satisfied() {
        let mut resources = resources();
        resources.influence_grid.register_emitter(EmitterRecord {
            x: 10,
            y: 8,
            channel: ChannelId::Social,
            radius: 4.0,
            base_intensity: 0.7,
            falloff: FalloffType::Linear,
            decay_rate: None,
            tags: Vec::new(),
            dirty: true,
        });
        resources.influence_grid.tick_update();

        let mut needs = Needs::default();
        needs.set(NeedType::Belonging, 0.95);
        let (force, cause) = influence_force_for_entity(
            &resources,
            &Position::new(6, 8),
            Some(&InfluenceReceiver::default()),
            Some(&needs),
            None,
            None,
            Some(&Temperament::default()),
        );

        assert_eq!(force, (0.0, 0.0));
        assert!(cause.is_none());
    }

    #[test]
    fn lonely_agents_cluster_toward_shared_social_anchor() {
        let mut resources = resources();
        resources.influence_grid.register_emitter(EmitterRecord {
            x: 10,
            y: 8,
            channel: ChannelId::Social,
            radius: 5.0,
            base_intensity: 0.9,
            falloff: FalloffType::Linear,
            decay_rate: None,
            tags: vec!["campfire".to_string()],
            dirty: true,
        });
        resources.influence_grid.tick_update();

        let mut left_needs = Needs::default();
        left_needs.set(NeedType::Belonging, 0.10);
        let mut right_needs = Needs::default();
        right_needs.set(NeedType::Belonging, 0.10);

        let (left_force, left_cause) = influence_force_for_entity(
            &resources,
            &Position::new(6, 8),
            Some(&InfluenceReceiver::default()),
            Some(&left_needs),
            None,
            None,
            Some(&Temperament::default()),
        );
        let (right_force, right_cause) = influence_force_for_entity(
            &resources,
            &Position::new(14, 8),
            Some(&InfluenceReceiver::default()),
            Some(&right_needs),
            None,
            None,
            Some(&Temperament::default()),
        );

        assert!(left_force.0 > 0.0);
        assert!(right_force.0 < 0.0);
        assert_eq!(left_cause.expect("left social cause").kind, "social_gradient");
        assert_eq!(right_cause.expect("right social cause").kind, "social_gradient");
    }

    #[test]
    fn danger_outweighs_social_gathering_when_fear_is_high() {
        let mut resources = resources();
        resources.influence_grid.replace_emitters(vec![
            EmitterRecord {
                x: 11,
                y: 8,
                channel: ChannelId::Social,
                radius: 5.0,
                base_intensity: 0.9,
                falloff: FalloffType::Linear,
                decay_rate: None,
                tags: vec!["campfire".to_string()],
                dirty: true,
            },
            EmitterRecord {
                x: 9,
                y: 8,
                channel: ChannelId::Danger,
                radius: 4.0,
                base_intensity: 0.95,
                falloff: FalloffType::Exponential,
                decay_rate: None,
                tags: vec!["fire".to_string()],
                dirty: true,
            },
        ]);
        resources.influence_grid.tick_update();

        let mut needs = Needs::default();
        needs.set(NeedType::Belonging, 0.10);
        let mut fear = Emotion::default();
        *fear.get_mut(sim_core::EmotionType::Fear) = 0.95;

        let (force, cause) = influence_force_for_entity(
            &resources,
            &Position::new(8, 8),
            Some(&InfluenceReceiver::default()),
            Some(&needs),
            Some(&fear),
            None,
            Some(&Temperament::default()),
        );

        assert!(force.0 < 0.0);
        assert_eq!(cause.expect("danger cause").kind, "danger_gradient");
    }

    #[test]
    fn hungry_agent_moves_closer_to_food_than_sated_agent() {
        fn run_entity_step(hunger: f64) -> (f64, ActionType) {
            let mut world = World::new();
            let mut resources = resources();
            resources.influence_grid.register_emitter(EmitterRecord {
                x: 12,
                y: 8,
                channel: ChannelId::Food,
                radius: 5.0,
                base_intensity: 0.9,
                falloff: FalloffType::Gaussian,
                decay_rate: None,
                tags: Vec::new(),
                dirty: true,
            });
            resources.influence_grid.tick_update();

            let mut needs = Needs::default();
            needs.set(NeedType::Hunger, hunger);
            let entity = world.spawn((
                Position::new(6, 8),
                Behavior::default(),
                InfluenceReceiver::default(),
                needs,
                SteeringParams::default(),
                Age {
                    stage: GrowthStage::Adult,
                    ..Age::default()
                },
                Temperament::default(),
            ));

            let mut behavior_system = crate::runtime::BehaviorRuntimeSystem::new(20, 1);
            behavior_system.run(&mut world, &mut resources, 1);
            let mut steering = SteeringRuntimeSystem::new(config::STEERING_SYSTEM_PRIORITY, 1);
            steering.run(&mut world, &mut resources, 1);
            let mut movement = MovementRuntimeSystem::new(
                config::MOVEMENT_SYSTEM_PRIORITY,
                config::MOVEMENT_TICK_INTERVAL,
            );
            movement.run(&mut world, &mut resources, 1);

            let position = world.get::<&Position>(entity).expect("position should exist");
            let behavior = world.get::<&Behavior>(entity).expect("behavior should exist");
            (position.x, behavior.current_action)
        }

        let (hungry_x, hungry_action) = run_entity_step(0.10);
        let (sated_x, sated_action) = run_entity_step(0.85);
        assert_eq!(hungry_action, ActionType::Forage);
        assert_ne!(sated_action, ActionType::Forage);
        assert!(hungry_x > sated_x);
    }

    #[test]
    fn danger_overrides_food_when_fear_is_high() {
        let mut resources = resources();
        resources.influence_grid.replace_emitters(vec![
            EmitterRecord {
                x: 11,
                y: 8,
                channel: ChannelId::Food,
                radius: 5.0,
                base_intensity: 0.8,
                falloff: FalloffType::Gaussian,
                decay_rate: None,
                tags: Vec::new(),
                dirty: true,
            },
            EmitterRecord {
                x: 9,
                y: 8,
                channel: ChannelId::Danger,
                radius: 4.0,
                base_intensity: 0.95,
                falloff: FalloffType::Exponential,
                decay_rate: None,
                tags: Vec::new(),
                dirty: true,
            },
        ]);
        resources.influence_grid.tick_update();

        let mut hungry = Needs::default();
        hungry.set(NeedType::Hunger, 0.10);
        hungry.set(NeedType::Safety, 0.90);

        let mut low_fear = Emotion::default();
        *low_fear.get_mut(sim_core::EmotionType::Fear) = 0.05;
        let mut high_fear = Emotion::default();
        *high_fear.get_mut(sim_core::EmotionType::Fear) = 0.95;

        let (low_force, _) = influence_force_for_entity(
            &resources,
            &Position::new(8, 8),
            Some(&InfluenceReceiver::default()),
            Some(&hungry),
            Some(&low_fear),
            None,
            Some(&Temperament::default()),
        );
        let (high_force, cause) = influence_force_for_entity(
            &resources,
            &Position::new(8, 8),
            Some(&InfluenceReceiver::default()),
            Some(&hungry),
            Some(&high_fear),
            None,
            Some(&Temperament::default()),
        );

        assert!(low_force.0 > high_force.0);
        assert!(high_force.0 < 0.0);
        assert_eq!(cause.expect("danger should dominate").kind, "danger_gradient");
    }

    #[test]
    fn danger_override_blocks_direct_target_blend_in_runtime() {
        let mut world = World::new();
        let mut resources = resources();
        resources.influence_grid.replace_emitters(vec![
            EmitterRecord {
                x: 12,
                y: 8,
                channel: ChannelId::Food,
                radius: 5.0,
                base_intensity: 0.8,
                falloff: FalloffType::Gaussian,
                decay_rate: None,
                tags: Vec::new(),
                dirty: true,
            },
            EmitterRecord {
                x: 9,
                y: 8,
                channel: ChannelId::Danger,
                radius: 4.0,
                base_intensity: 0.95,
                falloff: FalloffType::Exponential,
                decay_rate: None,
                tags: Vec::new(),
                dirty: true,
            },
        ]);
        resources.influence_grid.tick_update();

        let mut hungry = Needs::default();
        hungry.set(NeedType::Hunger, 0.10);
        hungry.set(NeedType::Safety, 0.90);

        let mut high_fear = Emotion::default();
        *high_fear.get_mut(sim_core::EmotionType::Fear) = 0.95;

        let entity = world.spawn((
            Position::new(8, 8),
            Behavior {
                current_action: ActionType::Forage,
                action_target_x: Some(12),
                action_target_y: Some(8),
                ..Behavior::default()
            },
            InfluenceReceiver::default(),
            hungry,
            high_fear,
            Stress::default(),
            Temperament::default(),
            SteeringParams {
                speed_variance: 0.0,
                ..SteeringParams::default()
            },
            Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            },
        ));

        let mut system = SteeringRuntimeSystem::new(config::STEERING_SYSTEM_PRIORITY, 1);
        system.run(&mut world, &mut resources, 1);

        let position = world.get::<&Position>(entity).expect("position should exist");
        assert!(position.vel_x < 0.0);
    }

    #[test]
    fn room_shelter_force_prefers_neighboring_room_tile_when_cold() {
        let mut resources = resources();
        resources.influence_grid.register_emitter(EmitterRecord {
            x: 5,
            y: 5,
            channel: ChannelId::Warmth,
            radius: 2.0,
            base_intensity: 0.8,
            falloff: FalloffType::Constant,
            decay_rate: None,
            tags: Vec::new(),
            dirty: true,
        });
        resources.influence_grid.tick_update();
        resources.tile_grid.assign_room(5, 5, RoomId(1));

        let force = room_shelter_force(
            &resources,
            5,
            6,
            Some(&InfluenceReceiver::default()),
            0.9,
            1.0,
        );

        assert!(force.1 < 0.0);
        assert!(force.0.abs() < 0.01);
    }

    #[test]
    fn room_shelter_force_is_zero_when_warmth_need_is_satisfied() {
        let mut resources = resources();
        resources.influence_grid.register_emitter(EmitterRecord {
            x: 5,
            y: 5,
            channel: ChannelId::Warmth,
            radius: 2.0,
            base_intensity: 0.8,
            falloff: FalloffType::Constant,
            decay_rate: None,
            tags: Vec::new(),
            dirty: true,
        });
        resources.influence_grid.tick_update();
        resources.tile_grid.assign_room(5, 5, RoomId(1));

        let force = room_shelter_force(
            &resources,
            5,
            6,
            Some(&InfluenceReceiver::default()),
            0.05,
            1.0,
        );

        assert_eq!(force, (0.0, 0.0));
    }

    #[test]
    fn influence_force_reports_shelter_cause_when_room_bias_applies() {
        let mut resources = resources();
        resources.influence_grid.register_emitter(EmitterRecord {
            x: 5,
            y: 5,
            channel: ChannelId::Warmth,
            radius: 2.0,
            base_intensity: 0.8,
            falloff: FalloffType::Constant,
            decay_rate: None,
            tags: Vec::new(),
            dirty: true,
        });
        resources.influence_grid.tick_update();
        resources.tile_grid.assign_room(5, 5, RoomId(1));

        let mut needs = Needs::default();
        needs.set(NeedType::Warmth, 0.1);
        let (force, cause) = influence_force_for_entity(
            &resources,
            &Position::new(5, 6),
            Some(&InfluenceReceiver::default()),
            Some(&needs),
            None,
            None,
            Some(&Temperament::default()),
        );

        assert!(force.1 < 0.0);
        assert_eq!(cause.expect("shelter cause").kind, "shelter_gradient");
    }

    #[test]
    fn cold_agent_prefers_room_warmth_more_than_comfortable_agent() {
        let mut resources = resources();
        resources.influence_grid.register_emitter(EmitterRecord {
            x: 5,
            y: 5,
            channel: ChannelId::Warmth,
            radius: 2.0,
            base_intensity: 0.8,
            falloff: FalloffType::Constant,
            decay_rate: None,
            tags: Vec::new(),
            dirty: true,
        });
        resources.influence_grid.tick_update();
        resources.tile_grid.assign_room(5, 5, RoomId(1));

        let mut cold_needs = Needs::default();
        cold_needs.set(NeedType::Warmth, 0.10);
        let mut comfortable_needs = Needs::default();
        comfortable_needs.set(NeedType::Warmth, 0.90);

        let (cold_force, cold_cause) = influence_force_for_entity(
            &resources,
            &Position::new(5, 6),
            Some(&InfluenceReceiver::default()),
            Some(&cold_needs),
            None,
            None,
            Some(&Temperament::default()),
        );
        let (comfortable_force, comfortable_cause) = influence_force_for_entity(
            &resources,
            &Position::new(5, 6),
            Some(&InfluenceReceiver::default()),
            Some(&comfortable_needs),
            None,
            None,
            Some(&Temperament::default()),
        );

        assert!(cold_force.1 < 0.0);
        assert_eq!(
            cold_cause.expect("cold shelter cause").kind,
            "shelter_gradient"
        );
        assert_eq!(comfortable_force, (0.0, 0.0));
        assert!(comfortable_cause.is_none());
    }
}
