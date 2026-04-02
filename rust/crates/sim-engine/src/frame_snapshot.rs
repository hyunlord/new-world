use hecs::World;
use sim_core::components::{Age, Behavior, Body, Emotion, Identity, Needs, Position, Stress};
use sim_core::config;
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
    /// Packed atlas variant: upper nibble job variant, lower nibble anim frame.
    pub atlas_var: u8,
    /// Danger bitflags for renderer overlays.
    pub danger_icon: u8,
    /// Band color index or `0xFF` when the agent has no band.
    pub band_color_idx: u8,
    /// Low byte of the encoded band id or `0xFF` when the agent has no band.
    pub band_id_lo: u8,
    /// High byte of the encoded band id or `0xFF` when the agent has no band.
    pub band_id_hi: u8,
    /// Reserved bytes kept to preserve the 36-byte stride contract.
    pub _reserved: [u8; 2],
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
        let atlas_var = self.atlas_var;
        let danger_icon = self.danger_icon;
        let band_color_idx = self.band_color_idx;
        let band_id_lo = self.band_id_lo;
        let band_id_hi = self.band_id_hi;
        let reserved = self._reserved;

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
        out.push(atlas_var);
        out.push(danger_icon);
        out.push(band_color_idx);
        out.push(band_id_lo);
        out.push(band_id_hi);
        out.extend_from_slice(&reserved);
    }
}

/// Builds render snapshots for all alive entities in stable raw-id order.
pub fn build_agent_snapshots(world: &World) -> Vec<AgentSnapshot> {
    let mut snapshots: Vec<AgentSnapshot> = Vec::new();

    for (
        entity,
        (position, identity, age, emotion_opt, needs_opt, stress_opt, behavior_opt, body_opt),
    ) in world
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

        // `0xFF`/`0xFFFF` reserve the "no band" sentinel in the byte protocol.
        let band_raw: u64 = identity.band_id.map(|bid| bid.0).unwrap_or(u64::MAX);
        let (band_color_idx, band_id_lo, band_id_hi) = if band_raw == u64::MAX {
            (0xFF_u8, 0xFF_u8, 0xFF_u8)
        } else {
            let color_idx = (band_raw % 8) as u8;
            let id_lo = (band_raw & 0xFF) as u8;
            let id_hi = ((band_raw >> 8) & 0xFF) as u8;
            (color_idx, id_lo, id_hi)
        };

        let job_variant = job_icon_code(behavior.job.as_str()) & 0x0F;
        let anim_frame = 0_u8;
        let atlas_var = (job_variant << 4) | anim_frame;

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
            atlas_var,
            danger_icon: compute_danger_flags(body, needs, stress),
            band_color_idx,
            band_id_lo,
            band_id_hi,
            _reserved: [0_u8; 2],
        });
    }

    snapshots.sort_unstable_by_key(|snapshot| snapshot.entity_id);
    snapshots
}

/// Builds a flat `f32` buffer for Godot `MultiMeshInstance2D` with `use_colors=true,
/// use_custom_data=true`. Each agent occupies exactly [`config::MULTIMESH_FLOATS_PER_INSTANCE`]
/// (14) floats laid out as:
///
/// ```text
/// Godot TRANSFORM_2D column-major layout (8 floats) + Color (4) + CustomData (4) = 16 total.
/// [0]  col-a.x = scale   (Transform2D column 0, x)
/// [1]  col-a.y = 0.0     (Transform2D column 0, y)
/// [2]  col-b.x = 0.0     (Transform2D column 1, x)
/// [3]  col-b.y = scale   (Transform2D column 1, y)
/// [4]  origin.x (pixels)
/// [5]  origin.y (pixels)
/// [6]  0.0 (padding — Godot TRANSFORM_2D stride = 8 floats)
/// [7]  0.0 (padding)
/// [8]  color.r  (job × gender blend)
/// [9]  color.g
/// [10] color.b
/// [11] color.a  = 1.0
/// [12] custom.r = movement_dir sprite frame  (dir * 4 / 255)
/// [13] custom.g = mood                       (mood / 4)
/// [14] custom.b = band_color_idx             (idx / 8, or -1.0 if no band)
/// [15] custom.a = growth stage               (stage / 5)
/// ```
///
/// Returns `(buffer, count)` where `buffer.len() == count * 16`.
pub fn build_agent_multimesh_buffer(world: &World) -> (Vec<f32>, usize) {
    const MALE_TINT: (f32, f32, f32) = (0.2, 0.4, 0.85);
    const FEMALE_TINT: (f32, f32, f32) = (0.9, 0.3, 0.45);
    const GENDER_TINT_WEIGHT: f32 = 0.2;
    const TILE_F: f32 = config::TILE_SIZE as f32;

    let mut buffer: Vec<f32> = Vec::new();
    let mut count: usize = 0;

    for (
        _entity,
        (position, identity, age, emotion_opt, needs_opt, behavior_opt),
    ) in world
        .query::<(
            &Position,
            &Identity,
            &Age,
            Option<&Emotion>,
            Option<&Needs>,
            Option<&Behavior>,
        )>()
        .iter()
    {
        if !age.alive {
            continue;
        }

        let emotion_default = Emotion::default();
        let needs_default = Needs::default();
        let behavior_default = Behavior::default();

        let emotion = emotion_opt.unwrap_or(&emotion_default);
        let needs = needs_opt.unwrap_or(&needs_default);
        let behavior = behavior_opt.unwrap_or(&behavior_default);

        // Scale from growth stage
        let stage_code = growth_stage_code(identity.growth_stage) as usize;
        let scale = config::AGE_SIZE_MULTIPLIERS
            .get(stage_code)
            .copied()
            .unwrap_or(1.0_f32);

        // Pixel-space origin (center of tile)
        let ox = position.x as f32 * TILE_F + TILE_F * 0.5;
        let oy = position.y as f32 * TILE_F + TILE_F * 0.5;

        // Job color
        let job_code = job_icon_code(behavior.job.as_str()) as usize;
        let (jr, jg, jb) = config::JOB_COLORS
            .get(job_code)
            .copied()
            .unwrap_or(config::JOB_COLORS[0]);

        // Gender tint blend
        let (tr, tg, tb) = match identity.sex {
            Sex::Male => MALE_TINT,
            Sex::Female => FEMALE_TINT,
        };
        let w = GENDER_TINT_WEIGHT;
        let cr = (jr * (1.0 - w) + tr * w).clamp(0.0, 1.0);
        let cg = (jg * (1.0 - w) + tg * w).clamp(0.0, 1.0);
        let cb = (jb * (1.0 - w) + tb * w).clamp(0.0, 1.0);

        // Custom data
        let mood = compute_mood_color(emotion, needs);
        let movement_dir = position.movement_dir;
        let band_color = identity
            .band_id
            .map(|bid| ((bid.0 % 8) as f32) / 8.0)
            .unwrap_or(-1.0_f32);
        let growth_norm = stage_code as f32 / 5.0;

        // Append 16 floats — Godot TRANSFORM_2D column-major + 2 padding + Color + CustomData
        buffer.push(scale);           // [0]  col-a.x
        buffer.push(0.0_f32);         // [1]  col-a.y
        buffer.push(0.0_f32);         // [2]  col-b.x
        buffer.push(scale);           // [3]  col-b.y
        buffer.push(ox);              // [4]  origin.x
        buffer.push(oy);              // [5]  origin.y
        buffer.push(0.0_f32);         // [6]  padding (stride = 8)
        buffer.push(0.0_f32);         // [7]  padding
        buffer.push(cr);              // [8]  color.r
        buffer.push(cg);              // [9]  color.g
        buffer.push(cb);              // [10] color.b
        buffer.push(1.0_f32);         // [11] color.a
        buffer.push((movement_dir as f32 * 4.0) / 255.0); // [12] custom.r
        buffer.push(mood as f32 / 4.0);                   // [13] custom.g
        buffer.push(band_color);      // [14] custom.b
        buffer.push(growth_norm);     // [15] custom.a

        count += 1;
    }

    (buffer, count)
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
    fn stage1_agent_snapshot_is_36_bytes() {
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
            atlas_var: 12,
            danger_icon: 3,
            band_color_idx: 0,
            band_id_lo: 0,
            band_id_hi: 0,
            _reserved: [0_u8; 2],
        };
        let mut out = Vec::new();
        snapshot.write_bytes(&mut out);
        assert_eq!(out.len(), 36);
        assert_eq!(
            u32::from_le_bytes(out[0..4].try_into().unwrap_or([0_u8; 4])),
            7
        );
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

    #[test]
    fn stage1_agent_snapshot_field_offsets_match_decoder_contract() {
        let snapshot = AgentSnapshot {
            entity_id: 42,
            x: 123.456,
            y: 789.012,
            vel_x: 1.5,
            vel_y: -2.25,
            mood_color: 3,
            stress_phase: 2,
            active_break: 1,
            atlas_var: 0b1101_0011,
            ..AgentSnapshot::default()
        };
        let mut out = Vec::new();
        snapshot.write_bytes(&mut out);
        let expected_x = snapshot.x;
        let expected_vel_x = snapshot.vel_x;
        assert_eq!(out.len(), 36);
        assert_eq!(
            u32::from_le_bytes(out[0..4].try_into().expect("entity id bytes")),
            42
        );
        assert_eq!(
            f32::from_le_bytes(out[4..8].try_into().expect("x bytes")),
            expected_x
        );
        assert_eq!(
            f32::from_le_bytes(out[12..16].try_into().expect("vel_x bytes")),
            expected_vel_x
        );
        assert_eq!(out[20], 3);
        assert_eq!(out[25], 2);
        assert_eq!(out[26], 1);
        assert_eq!(out[29], 0b1101_0011);
    }

    #[test]
    fn stage1_snapshot_serialization_roundtrip() {
        let snapshot = AgentSnapshot {
            entity_id: 42,
            x: 123.456,
            y: 789.012,
            vel_x: 1.5,
            vel_y: -2.3,
            mood_color: 3,
            stress_phase: 2,
            active_break: 0,
            action_state: 5,
            movement_dir: 7,
            atlas_var: 0b1101_0011,
            ..AgentSnapshot::default()
        };

        let mut bytes = Vec::new();
        snapshot.write_bytes(&mut bytes);
        assert_eq!(bytes.len(), 36);
        assert_eq!(
            u32::from_le_bytes(bytes[0..4].try_into().expect("entity bytes")),
            42
        );
        assert!(
            (f32::from_le_bytes(bytes[4..8].try_into().expect("x bytes")) - 123.456).abs() < 0.001
        );
        assert_eq!(bytes[20], 3);
        assert_eq!(bytes[29], 0b1101_0011);
    }

    #[test]
    fn stage1_compute_mood_color_stays_in_range() {
        let mut emotion = Emotion::default();
        emotion.primary[EmotionType::Joy as usize] = 1.0;
        emotion.primary[EmotionType::Sadness as usize] = 0.0;
        let high = compute_mood_color(&emotion, &Needs::default());
        emotion.primary[EmotionType::Joy as usize] = 0.0;
        emotion.primary[EmotionType::Sadness as usize] = 1.0;
        let low = compute_mood_color(&emotion, &Needs::default());
        assert!(high <= 4);
        assert!(low <= 4);
        assert!(high >= low);
    }

    #[test]
    fn stage1_atlas_var_bit_encoding_roundtrips() {
        let atlas_var: u8 = 0b1101_0011;
        let job_variant = (atlas_var >> 4) & 0x0F;
        let anim_frame = atlas_var & 0x0F;
        assert_eq!(job_variant, 13);
        assert_eq!(anim_frame, 3);
    }
}
