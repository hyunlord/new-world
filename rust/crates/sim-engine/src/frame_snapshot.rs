use hecs::World;
use sim_core::components::{Age, Behavior, Body, Emotion, Identity, Needs, Position, Stress};
use sim_core::enums::{ActionType, EmotionType, GrowthStage, MentalBreakType, Sex, StressState};

/// Per-agent render snapshot encoded as a fixed 36-byte record.
///
/// The Godot bridge serializes this as a `PackedByteArray` so the renderer can
/// decode positions and visual-state flags without per-field dictionary boxing.
#[repr(C, packed)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct AgentSnapshot {
    /// Stable 32-bit raw entity id (`hecs::Entity::id()`).
    pub entity_id: u32,
    /// Continuous X position in tile-space.
    pub x: f32,
    /// Continuous Y position in tile-space.
    pub y: f32,
    /// X velocity in tile-space per tick.
    pub vel_x: f32,
    /// Y velocity in tile-space per tick.
    pub vel_y: f32,
    /// Mood bucket `0..=4`.
    pub mood_color: u8,
    /// Growth stage code `0..=5`.
    pub growth_stage: u8,
    /// Sex code `0..=1`.
    pub sex: u8,
    /// Job icon code.
    pub job_icon: u8,
    /// Health tier `0..=2`.
    pub health_tier: u8,
    /// Stress phase bucket `0..=4`.
    pub stress_phase: u8,
    /// Active mental-break code `0..=10`.
    pub active_break: u8,
    /// Action-state code.
    pub action_state: u8,
    /// Movement direction `0..=7`.
    pub movement_dir: u8,
    /// Packed sprite-variation placeholder bits.
    pub sprite_var: u8,
    /// Danger bitflags for renderer overlays.
    pub danger_icon: u8,
    /// Faction color placeholder.
    pub faction_color: u8,
    /// Explicit alignment pad to preserve 36-byte stride.
    pub _pad: [u8; 4],
}

const _: () = assert!(std::mem::size_of::<AgentSnapshot>() == 36);

impl AgentSnapshot {
    /// Appends this snapshot to a byte buffer using little-endian field layout.
    pub fn write_bytes(&self, out: &mut Vec<u8>) {
        let entity_id = self.entity_id;
        let x = self.x;
        let y = self.y;
        let vel_x = self.vel_x;
        let vel_y = self.vel_y;
        let mood_color = self.mood_color;
        let growth_stage = self.growth_stage;
        let sex = self.sex;
        let job_icon = self.job_icon;
        let health_tier = self.health_tier;
        let stress_phase = self.stress_phase;
        let active_break = self.active_break;
        let action_state = self.action_state;
        let movement_dir = self.movement_dir;
        let sprite_var = self.sprite_var;
        let danger_icon = self.danger_icon;
        let faction_color = self.faction_color;
        let pad = self._pad;

        out.extend_from_slice(&entity_id.to_le_bytes());
        out.extend_from_slice(&x.to_le_bytes());
        out.extend_from_slice(&y.to_le_bytes());
        out.extend_from_slice(&vel_x.to_le_bytes());
        out.extend_from_slice(&vel_y.to_le_bytes());
        out.push(mood_color);
        out.push(growth_stage);
        out.push(sex);
        out.push(job_icon);
        out.push(health_tier);
        out.push(stress_phase);
        out.push(active_break);
        out.push(action_state);
        out.push(movement_dir);
        out.push(sprite_var);
        out.push(danger_icon);
        out.push(faction_color);
        out.extend_from_slice(&pad);
    }
}

/// Builds render snapshots for all alive entities in stable raw-id order.
pub fn build_agent_snapshots(world: &World) -> Vec<AgentSnapshot> {
    let mut snapshots: Vec<AgentSnapshot> = Vec::new();

    for (entity, (position, identity, age, emotion_opt, needs_opt, stress_opt, behavior_opt, body_opt)) in
        world
            .query::<(
                &Position,
                &Identity,
                &Age,
                Option<&Emotion>,
                Option<&Needs>,
                Option<&Stress>,
                Option<&Behavior>,
                Option<&Body>,
            )>()
            .iter()
    {
        if !age.alive {
            continue;
        }

        let emotion_default = Emotion::default();
        let needs_default = Needs::default();
        let stress_default = Stress::default();
        let behavior_default = Behavior::default();
        let body_default = Body::default();

        let emotion = emotion_opt.unwrap_or(&emotion_default);
        let needs = needs_opt.unwrap_or(&needs_default);
        let stress = stress_opt.unwrap_or(&stress_default);
        let behavior = behavior_opt.unwrap_or(&behavior_default);
        let body = body_opt.unwrap_or(&body_default);

        snapshots.push(AgentSnapshot {
            entity_id: entity.id(),
            x: position.x as f32,
            y: position.y as f32,
            vel_x: position.vel_x as f32,
            vel_y: position.vel_y as f32,
            mood_color: compute_mood_color(emotion, needs),
            growth_stage: growth_stage_code(age.stage),
            sex: sex_code(identity.sex),
            job_icon: job_icon_code(behavior.job.as_str()),
            health_tier: compute_health_tier(body),
            stress_phase: stress_phase_code(stress.state),
            active_break: mental_break_code(stress.active_mental_break),
            action_state: action_state_code(behavior.current_action),
            movement_dir: position.movement_dir,
            sprite_var: compute_sprite_var(body, identity, age),
            danger_icon: compute_danger_flags(body, needs, stress),
            faction_color: 0,
            _pad: [0_u8; 4],
        });
    }

    snapshots.sort_unstable_by_key(|snapshot| snapshot.entity_id);
    snapshots
}

fn compute_mood_color(emotion: &Emotion, _needs: &Needs) -> u8 {
    let positive = emotion.get(EmotionType::Joy)
        + emotion.get(EmotionType::Trust)
        + emotion.get(EmotionType::Anticipation);
    let negative = emotion.get(EmotionType::Sadness)
        + emotion.get(EmotionType::Fear)
        + emotion.get(EmotionType::Anger);
    let balance = ((positive / 3.0) - (negative / 3.0)).clamp(-1.0, 1.0);
    (((balance + 1.0) * 2.0).round()).clamp(0.0, 4.0) as u8
}

fn compute_health_tier(body: &Body) -> u8 {
    if body.health < 0.3 {
        0
    } else if body.health < 0.7 {
        1
    } else {
        2
    }
}

fn compute_sprite_var(body: &Body, identity: &Identity, age: &Age) -> u8 {
    let hair = (identity.name.len() % 8) as u8;
    let body_type = ((body.height.clamp(0.0, 0.999) * 4.0).floor() as u8) & 0b11;
    let skin = growth_stage_code(age.stage) & 0b111;
    (hair << 5) | (body_type << 3) | skin
}

fn compute_danger_flags(body: &Body, needs: &Needs, stress: &Stress) -> u8 {
    let mut flags: u8 = 0;
    if body.health < 0.2 {
        flags |= 0b0001;
    }
    if needs.values.first().copied().unwrap_or(1.0) < 0.1 {
        flags |= 0b0010;
    }
    if matches!(
        stress.state,
        StressState::Resistance | StressState::Exhaustion | StressState::Collapse
    ) {
        flags |= 0b0100;
    }
    flags
}

fn growth_stage_code(stage: GrowthStage) -> u8 {
    match stage {
        GrowthStage::Infant => 0,
        GrowthStage::Toddler => 1,
        GrowthStage::Child => 2,
        GrowthStage::Teen => 3,
        GrowthStage::Adult => 4,
        GrowthStage::Elder => 5,
    }
}

fn sex_code(sex: Sex) -> u8 {
    match sex {
        Sex::Male => 0,
        Sex::Female => 1,
    }
}

fn job_icon_code(job: &str) -> u8 {
    match job {
        "gatherer" => 1,
        "lumberjack" => 2,
        "builder" => 3,
        "miner" => 4,
        _ => 0,
    }
}

fn stress_phase_code(state: StressState) -> u8 {
    match state {
        StressState::Calm => 0,
        StressState::Alert => 1,
        StressState::Resistance => 2,
        StressState::Exhaustion => 3,
        StressState::Collapse => 4,
    }
}

fn mental_break_code(active_break: Option<MentalBreakType>) -> u8 {
    match active_break {
        None => 0,
        Some(MentalBreakType::Panic) => 1,
        Some(MentalBreakType::Rage) => 2,
        Some(MentalBreakType::OutrageViolence) => 3,
        Some(MentalBreakType::Shutdown) => 4,
        Some(MentalBreakType::Purge) => 5,
        Some(MentalBreakType::GriefWithdrawal) => 6,
        Some(MentalBreakType::Fugue) => 7,
        Some(MentalBreakType::Paranoia) => 8,
        Some(MentalBreakType::CompulsiveRitual) => 9,
        Some(MentalBreakType::HystericalBonding) => 10,
    }
}

fn action_state_code(action: ActionType) -> u8 {
    match action {
        ActionType::Idle => 0,
        ActionType::Forage => 1,
        ActionType::Hunt => 2,
        ActionType::Fish => 3,
        ActionType::Build => 4,
        ActionType::Craft => 5,
        ActionType::Socialize => 6,
        ActionType::Rest => 7,
        ActionType::Sleep => 8,
        ActionType::Eat => 9,
        ActionType::Drink => 10,
        ActionType::Explore => 11,
        ActionType::Flee => 12,
        ActionType::Fight => 13,
        ActionType::Migrate => 14,
        ActionType::Teach => 15,
        ActionType::Learn => 16,
        ActionType::MentalBreak => 17,
        ActionType::Pray => 18,
        ActionType::Wander => 19,
        ActionType::GatherWood => 20,
        ActionType::GatherStone => 21,
        ActionType::GatherHerbs => 22,
        ActionType::DeliverToStockpile => 23,
        ActionType::TakeFromStockpile => 24,
        ActionType::SeekShelter => 25,
        ActionType::SitByFire => 26,
        ActionType::VisitPartner => 27,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::components::{Age, Behavior, Body, Emotion, Identity, Needs, Position, Stress};

    #[test]
    fn agent_snapshot_is_36_bytes() {
        assert_eq!(std::mem::size_of::<AgentSnapshot>(), 36);
    }

    #[test]
    fn write_bytes_produces_fixed_stride() {
        let snapshot = AgentSnapshot {
            entity_id: 7,
            x: 1.5,
            y: 2.5,
            vel_x: 0.25,
            vel_y: -0.5,
            mood_color: 4,
            growth_stage: 5,
            sex: 1,
            job_icon: 3,
            health_tier: 2,
            stress_phase: 1,
            active_break: 0,
            action_state: 7,
            movement_dir: 6,
            sprite_var: 12,
            danger_icon: 3,
            faction_color: 0,
            _pad: [0_u8; 4],
        };
        let mut out = Vec::new();
        snapshot.write_bytes(&mut out);
        assert_eq!(out.len(), 36);
        assert_eq!(u32::from_le_bytes(out[0..4].try_into().unwrap_or([0_u8; 4])), 7);
    }

    #[test]
    fn build_agent_snapshots_filters_dead_and_sorts() {
        let mut world = World::new();
        world.spawn((
            Position::new(3, 4),
            Identity::default(),
            Age::default(),
            Emotion::default(),
            Needs::default(),
            Stress::default(),
            Behavior::default(),
            Body::default(),
        ));
        let mut dead_age = Age::default();
        dead_age.alive = false;
        world.spawn((
            Position::new(1, 2),
            Identity::default(),
            dead_age,
            Emotion::default(),
            Needs::default(),
            Stress::default(),
            Behavior::default(),
            Body::default(),
        ));

        let snapshots = build_agent_snapshots(&world);
        assert_eq!(snapshots.len(), 1);
        let x = snapshots[0].x;
        assert_eq!(x, 3.0);
    }
}
