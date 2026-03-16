use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartFlags(pub u8);

impl PartFlags {
    pub const BLEEDING: Self = Self(0x01);
    pub const FRACTURED: Self = Self(0x02);
    pub const BURNED: Self = Self(0x04);
    pub const FROSTBITE: Self = Self(0x08);
    pub const INFECTED: Self = Self(0x10);
    pub const DISABLED: Self = Self(0x20);

    pub fn has(self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }

    pub fn set(&mut self, flag: Self) {
        self.0 |= flag.0;
    }

    pub fn clear(&mut self, flag: Self) {
        self.0 &= !flag.0;
    }

    pub fn any(self) -> bool {
        self.0 != 0
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartState {
    pub hp: u8,
    pub flags: PartFlags,
    pub bleed_rate: u8,
    pub infection_sev: u8,
}

impl PartState {
    pub const fn healthy() -> Self {
        Self {
            hp: 100,
            flags: PartFlags(0),
            bleed_rate: 0,
            infection_sev: 0,
        }
    }

    pub fn is_damaged(self) -> bool {
        self.hp < 100 || self.flags.any()
    }
}

impl Default for PartState {
    fn default() -> Self {
        Self::healthy()
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthLod {
    Full = 0,
    Standard = 1,
    Simplified = 2,
    #[default]
    Aggregate = 3,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LimbGroup {
    Head = 0,
    Neck = 1,
    UpperTorso = 2,
    LowerTorso = 3,
    ArmL = 4,
    ArmR = 5,
    LegL = 6,
    LegR = 7,
    HandL = 8,
    HandR = 9,
}

impl LimbGroup {
    const fn idx(self) -> usize {
        self as usize
    }
}

pub const PART_TO_GROUP: [u8; 85] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3,
    3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6, 6, 6, 7, 7,
    7, 7, 7, 7, 7, 7, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
];

pub const PART_NAMES: [&str; 85] = [
    "Skull",
    "Brain",
    "Left Eye",
    "Right Eye",
    "Left Ear",
    "Right Ear",
    "Nose",
    "Mouth",
    "Tongue",
    "Teeth",
    "Jaw",
    "Hair",
    "Neck",
    "Throat",
    "Upper Spine",
    "Ribcage",
    "Heart",
    "Left Lung",
    "Right Lung",
    "Liver",
    "Stomach",
    "Pancreas",
    "Spleen",
    "Left Shoulder",
    "Right Shoulder",
    "Upper Back",
    "Lower Spine",
    "Guts",
    "Left Kidney",
    "Right Kidney",
    "Groin",
    "Lower Back",
    "L Upper Arm",
    "L Forearm",
    "L Hand",
    "L Wrist",
    "L Thumb",
    "L Index",
    "L Middle",
    "L Ring",
    "L Little",
    "R Upper Arm",
    "R Forearm",
    "R Hand",
    "R Wrist",
    "R Thumb",
    "R Index",
    "R Middle",
    "R Ring",
    "R Little",
    "L Upper Leg",
    "L Lower Leg",
    "L Foot",
    "L Big Toe",
    "L Toe 2",
    "L Toe 3",
    "L Toe 4",
    "L Little Toe",
    "R Upper Leg",
    "R Lower Leg",
    "R Foot",
    "R Big Toe",
    "R Toe 2",
    "R Toe 3",
    "R Toe 4",
    "R Little Toe",
    "Trachea",
    "Esophagus",
    "Aorta",
    "Vena Cava",
    "Diaphragm",
    "Bladder",
    "Uterus",
    "Appendix",
    "Gallbladder",
    "Adrenal L",
    "Adrenal R",
    "Thyroid",
    "Thymus",
    "Pituitary",
    "Pineal",
    "Bone Marrow",
    "Lymph Node",
    "Spinal Cord",
    "Pelvis",
];

pub const PART_RELSIZE: [u16; 85] = [
    200, 180, 140, 140, 110, 110, 150, 160, 120, 140, 170, 50, 180, 200, 190, 200, 170, 170,
    170, 160, 160, 130, 110, 160, 160, 200, 180, 200, 150, 150, 160, 180, 180, 170, 150, 120,
    100, 90, 90, 90, 80, 180, 170, 150, 120, 100, 90, 90, 90, 80, 200, 200, 180, 120, 100, 100,
    100, 90, 200, 200, 180, 120, 100, 100, 100, 90, 30, 25, 30, 30, 28, 25, 20, 15, 15, 12, 12,
    14, 14, 10, 10, 30, 15, 30, 30,
];

pub const PART_VITAL: [bool; 85] = [
    false, true, false, false, false, false, false, false, false, false, false, false, false,
    true, false, false, true, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, true, true, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false,
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyHealth {
    pub aggregate_hp: f64,
    pub damaged_groups: u16,
    pub group_hp: [u8; 10],
    #[serde(serialize_with = "serialize_parts", deserialize_with = "deserialize_parts")]
    pub parts: [PartState; 85],
    pub lod_tier: HealthLod,
    pub active_conditions: u8,
}

impl Default for BodyHealth {
    fn default() -> Self {
        Self {
            aggregate_hp: 1.0,
            damaged_groups: 0,
            group_hp: [100; 10],
            parts: [PartState::healthy(); 85],
            lod_tier: HealthLod::Aggregate,
            active_conditions: 0,
        }
    }
}

impl BodyHealth {
    pub fn recalculate_aggregates(&mut self) {
        let mut group_hp = [100_u8; 10];
        let mut damaged_groups = 0_u16;
        let mut weighted_hp = 0_u32;
        let mut total_weight = 0_u32;
        let mut active_conditions = 0_u8;

        for (index, part) in self.parts.iter().copied().enumerate() {
            let group_index = PART_TO_GROUP[index] as usize;
            group_hp[group_index] = group_hp[group_index].min(part.hp);
            if part.is_damaged() {
                damaged_groups |= 1_u16 << group_index;
            }
            if part.flags.any() {
                active_conditions = active_conditions.saturating_add(1);
            }
            let weight = u32::from(PART_RELSIZE[index]);
            weighted_hp += weight * u32::from(part.hp);
            total_weight += weight;
        }

        self.group_hp = group_hp;
        self.damaged_groups = damaged_groups;
        self.active_conditions = active_conditions;
        self.aggregate_hp = if total_weight == 0 {
            1.0
        } else {
            weighted_hp as f64 / (total_weight as f64 * 100.0)
        };
    }

    pub fn move_mult(&self) -> f64 {
        let left = self.group_hp[LimbGroup::LegL.idx()] as f64;
        let right = self.group_hp[LimbGroup::LegR.idx()] as f64;
        let leg_min = left.min(right) / 100.0;
        let leg_avg = (left + right) / 200.0;
        leg_min * 0.5 + leg_avg * 0.5
    }

    pub fn work_mult(&self) -> f64 {
        let arm_avg =
            (self.group_hp[LimbGroup::ArmL.idx()] as f64 + self.group_hp[LimbGroup::ArmR.idx()] as f64)
                / 200.0;
        let head = self.group_hp[LimbGroup::Head.idx()] as f64 / 100.0;
        arm_avg * head
    }

    pub fn combat_mult(&self) -> f64 {
        let best_arm = self.group_hp[LimbGroup::ArmL.idx()]
            .max(self.group_hp[LimbGroup::ArmR.idx()]) as f64
            / 100.0;
        let head = self.group_hp[LimbGroup::Head.idx()] as f64 / 100.0;
        let upper_torso = self.group_hp[LimbGroup::UpperTorso.idx()] as f64 / 100.0;
        best_arm * head * upper_torso
    }

    pub fn pain(&self) -> f64 {
        let mut weighted_pain = 0_u32;
        let mut total_weight = 0_u32;
        for (index, part) in self.parts.iter().enumerate() {
            let weight = u32::from(PART_RELSIZE[index]);
            weighted_pain += weight * u32::from(100_u8.saturating_sub(part.hp));
            total_weight += weight;
        }
        if total_weight == 0 {
            0.0
        } else {
            weighted_pain as f64 / (total_weight as f64 * 100.0)
        }
    }

    pub fn is_dead(&self) -> bool {
        self.parts
            .iter()
            .enumerate()
            .any(|(index, part)| PART_VITAL[index] && part.hp == 0)
    }
}

fn serialize_parts<S>(parts: &[PartState; 85], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    parts.as_slice().serialize(serializer)
}

fn deserialize_parts<'de, D>(deserializer: D) -> Result<[PartState; 85], D::Error>
where
    D: Deserializer<'de>,
{
    let values = Vec::<PartState>::deserialize(deserializer)?;
    values.try_into().map_err(|values: Vec<PartState>| {
        D::Error::custom(format!("expected 85 parts, got {}", values.len()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_healthy() {
        let health = BodyHealth::default();
        assert_eq!(health.aggregate_hp, 1.0);
        assert_eq!(health.damaged_groups, 0);
        assert_eq!(health.group_hp, [100; 10]);
        assert_eq!(health.active_conditions, 0);
        assert!(!health.is_dead());
    }

    #[test]
    fn damage_sets_group_flag() {
        let mut health = BodyHealth::default();
        health.parts[32].hp = 45;
        health.recalculate_aggregates();
        assert_ne!(health.damaged_groups & (1 << LimbGroup::ArmL.idx()), 0);
        assert_eq!(health.group_hp[LimbGroup::ArmL.idx()], 45);
        assert!(health.aggregate_hp < 1.0);
    }

    #[test]
    fn vital_part_zero_is_dead() {
        let mut health = BodyHealth::default();
        health.parts[16].hp = 0;
        assert!(health.is_dead());
    }

    #[test]
    fn non_vital_zero_not_dead() {
        let mut health = BodyHealth::default();
        health.parts[34].hp = 0;
        assert!(!health.is_dead());
    }

    #[test]
    fn vital_parts_are_correct() {
        let expected_vital = [1, 13, 16, 68, 69];
        let actual_vital: Vec<usize> = PART_VITAL
            .iter()
            .enumerate()
            .filter(|(_, vital)| **vital)
            .map(|(index, _)| index)
            .collect();
        assert_eq!(
            actual_vital,
            expected_vital,
            "Vital parts mismatch. Expected {:?}, got {:?}. Names: {:?}",
            expected_vital,
            actual_vital,
            actual_vital
                .iter()
                .map(|&index| PART_NAMES[index])
                .collect::<Vec<_>>()
        );
    }
}
