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
    };
}
