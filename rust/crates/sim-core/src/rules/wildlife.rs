//! M5 (ADR 0028, 0029): animals, hunting, fights, drafting, hides and apparel. `None` outside the M5 colony.
use serde::{Deserialize, Serialize};

/// Tunables of the animals and of what people do about them (`ColonyRule::wildlife`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WildlifeRule {
    /// Fewest and most deer at worldgen (goal 3.2: 6–12; also the cap).
    pub deer: (u32, u32),
    /// A deer is born every this many days while below the cap (goal 3.2: 4).
    pub deer_respawn_days: u32,
    /// Deer live only this far or farther from the hearth (goal 3.2: 20).
    pub deer_min_hearth_distance: u32,
    /// A deer flees from people within this distance (goal 3.2: 6; droppable 7.6).
    pub deer_flee_tiles: u32,
    /// Tiles a fleeing deer moves per animal step (goal 3.2: 2 per 3 ticks).
    pub deer_flee_speed: u32,
    /// Steps a deer flees before it tires and grazes again (DECISIONS D8).
    pub deer_flee_steps: u32,
    /// Animal steps a tired deer grazes before it can flee again (DECISIONS D8).
    pub deer_rest_steps: u32,
    /// Whether deer flee at all (droppable 7.6).
    pub deer_flee: bool,
    /// A deer wanders within this distance of its home ground (DECISIONS D8).
    pub deer_home_tiles: u32,
    /// The herd's centre lies within this many tiles of the hearth when the forest allows (a hunt stays within the job
    /// scan radius; DECISIONS D8: 48).
    pub deer_herd_max_tiles: u32,
    /// A deer's hit points (goal 3.2: 30).
    pub deer_hp: u16,
    /// A wolf's hit points (goal 3.2: 40).
    pub wolf_hp: u16,
    /// A wolf looks for a person outside a roofed room within this distance (goal 3.2: 24).
    pub wolf_sight: u32,
    /// Damage of a wolf's bite (goal 3.2: 6).
    pub wolf_damage: u16,
    /// Damage of bare hands, a stone axe, a ground axe and a spear (goal 3.2: 4, 8, 10, 12).
    pub hand_damage: u16,
    /// See [`Self::hand_damage`].
    pub axe_damage: u16,
    /// See [`Self::hand_damage`].
    pub ground_axe_damage: u16,
    /// See [`Self::hand_damage`].
    pub spear_damage: u16,
    /// Ticks between two resolved hits of one fighter (DECISIONS D8).
    pub hit_ticks: u32,
    /// Attacks a spear lasts (goal 3.2: 30).
    pub spear_uses: u32,
    /// A drafted person without a weapon fetches one within this distance (goal 3.2: 12).
    pub arm_reach: u32,
    /// A hunter attacks from within this distance (goal 3.2: 2).
    pub hunt_reach: u32,
    /// Ticks of one hunting attack at skill level 5 (goal 3.2: 60).
    pub hunt_attack_ticks: u32,
    /// Hit chance at hunting level 0 and at 20, ‰ (goal 3.2: 400, 800).
    pub hunt_hit_permille: (u32, u32),
    /// Most hit chance with a spear (it doubles the chance), ‰ (DECISIONS D8).
    pub spear_hit_cap_permille: u32,
    /// Meat a deer gives with a knife, and without (goal 3.3: 3, 2).
    pub meat_with_knife: u16,
    /// See [`Self::meat_with_knife`].
    pub meat_without_knife: u16,
    /// Settlement food units a piece of meat becomes (goal 3.2: 2).
    pub meat_food_units: u32,
    /// Days after the kill meat spoils (goal 3.2: 6).
    pub meat_spoil_days: u32,
    /// Uses a knife (a held or workshop flint blade) lasts (goal 3.3: 8).
    pub knife_uses: u32,
    /// Whether wolves leave a wolf hide (droppable 8.5).
    pub wolf_hides: bool,
    /// M5.5 (ADR 0028 amendment): undrafted people near a hunting wolf flee (`FleeSystem`). `false` = M5.
    #[serde(default)]
    pub flee: bool,
    /// A person flees from a hunting or attacking wolf within this distance (goal M5.5 §3.1.2: 8).
    #[serde(default = "flee_tiles")]
    pub flee_tiles: u32,
    /// Ticks one flight lasts, renewed while a wolf is near (goal M5.5 §3.1.2: 30).
    #[serde(default = "flee_ticks")]
    pub flee_ticks: u64,
    /// A fleeing person's blows at a wolf, ‰ of the weapon's damage (goal M5.5 §3.1.2: 500).
    #[serde(default = "flee_defence")]
    pub flee_defence_permille: u32,
    /// Tiles a wolf runs a step while chasing a person within `flee_tiles` (M5.5 calibration r1, DECISIONS D15: 2 —
    /// a wolf outruns a person; M5: 1, the person's pace).
    #[serde(default = "one")]
    pub wolf_chase_tiles: u32,
    /// M5.5 (ADR 0028 amendment): a hunter carries the kill's meat to a stockpile right after the kill. `false` = M5.
    #[serde(default)]
    pub hunter_hauls: bool,
    /// Most meat the hunter carries home (goal M5.5 §3.1.3: 3).
    #[serde(default = "hunter_meat")]
    pub hunter_meat: u16,
}

const fn one() -> u32 {
    1
}

const fn flee_tiles() -> u32 {
    8
}

const fn flee_ticks() -> u64 {
    30
}

const fn flee_defence() -> u32 {
    500
}

const fn hunter_meat() -> u16 {
    3
}

impl WildlifeRule {
    /// The M5 values (goal 3.2, 3.3; DECISIONS D8).
    pub const M5: Self = Self {
        deer: (6, 12),
        deer_respawn_days: 4,
        deer_min_hearth_distance: 20,
        deer_flee_tiles: 6,
        deer_flee_speed: 2,
        deer_flee_steps: 4,
        deer_rest_steps: 40,
        deer_flee: true,
        deer_home_tiles: 16,
        deer_herd_max_tiles: 48,
        deer_hp: 30,
        wolf_hp: 40,
        wolf_sight: 24,
        wolf_damage: 6,
        hand_damage: 4,
        axe_damage: 8,
        ground_axe_damage: 10,
        spear_damage: 12,
        hit_ticks: 20,
        spear_uses: 30,
        arm_reach: 12,
        hunt_reach: 2,
        hunt_attack_ticks: 60,
        hunt_hit_permille: (400, 800),
        spear_hit_cap_permille: 950,
        meat_with_knife: 3,
        meat_without_knife: 2,
        meat_food_units: 2,
        meat_spoil_days: 6,
        knife_uses: 8,
        wolf_hides: true,
        flee: false,
        flee_tiles: 8,
        flee_ticks: 30,
        flee_defence_permille: 500,
        wolf_chase_tiles: 1,
        hunter_hauls: false,
        hunter_meat: 3,
    };

    /// The M5.5 values (goal M5.5 §3.1.2–5): the undrafted flee, the hunter carries the meat home, a knife lasts 4 uses.
    pub const M5_5: Self = Self {
        knife_uses: 4,
        flee: true,
        wolf_chase_tiles: 2,
        hunter_hauls: true,
        ..Self::M5
    };
}
