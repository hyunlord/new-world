use serde::{Deserialize, Serialize};

use crate::ids::{BuildingId, EntityId, SettlementId};

/// Stable item instance identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ItemId(pub u64);

/// Item ownership discriminator.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ItemOwner {
    /// Carried by an agent.
    Agent(EntityId),
    /// In a settlement's shared stockpile.
    Settlement(SettlementId),
    /// Stored inside a building.
    Building(BuildingId),
    /// On the ground at tile coordinates.
    Ground(i32, i32),
}

/// Equipment slot for tools or weapons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquipSlot {
    MainHand,
    OffHand,
}

/// Auto-derived performance stats from material properties.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ItemDerivedStats {
    /// Damage output: hardness × 1.2
    pub damage: f64,
    /// Action speed modifier: 5.0 / density
    pub speed: f64,
    /// Maximum durability: hardness × density × 10.0
    pub max_durability: f64,
}

impl Default for ItemDerivedStats {
    fn default() -> Self {
        Self {
            damage: 1.0,
            speed: 1.0,
            max_durability: 100.0,
        }
    }
}

impl ItemDerivedStats {
    /// Creates per-instance stats from registry-derived material stats.
    pub fn from_material_stats(damage: f64, speed: f64, durability: f64) -> Self {
        Self {
            damage,
            speed,
            max_durability: durability,
        }
    }
}

/// One item instance in the world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemInstance {
    /// Unique identifier.
    pub id: ItemId,
    /// RON template reference (e.g. `knife`, `raw_wood`).
    pub template_id: String,
    /// Material used in creation (e.g. `flint`, `oak`).
    pub material_id: String,
    /// Auto-derived performance stats.
    pub derived_stats: ItemDerivedStats,
    /// Current durability (decreases on use, destroyed at 0).
    pub current_durability: f64,
    /// Crafting quality (0.0..=1.0). Phase 1 default is 0.5.
    pub quality: f64,
    /// Current owner.
    pub owner: ItemOwner,
    /// Stack count (raw materials stack, crafted tools do not).
    pub stack_count: u32,
    /// Tick when created.
    pub created_tick: u64,
    /// Creator entity, if any.
    pub creator_id: Option<EntityId>,
    /// Equipped slot, if equipped.
    pub equipped_slot: Option<EquipSlot>,
}

impl ItemInstance {
    /// Returns true if this is a stackable raw-material style item.
    pub fn is_stackable(&self) -> bool {
        self.equipped_slot.is_none() && self.max_durability_is_default()
    }

    /// Returns true if this item can stack with another item.
    pub fn can_stack_with(&self, other: &ItemInstance) -> bool {
        self.template_id == other.template_id
            && self.material_id == other.material_id
            && self.is_stackable()
            && other.is_stackable()
    }

    /// Returns true once durability reaches zero or below.
    pub fn is_broken(&self) -> bool {
        self.current_durability <= 0.0
    }

    fn max_durability_is_default(&self) -> bool {
        (self.derived_stats.max_durability - 100.0).abs() < f64::EPSILON
            && (self.current_durability - 100.0).abs() < f64::EPSILON
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_item(id: u64) -> ItemInstance {
        ItemInstance {
            id: ItemId(id),
            template_id: "raw_wood".to_string(),
            material_id: "oak".to_string(),
            derived_stats: ItemDerivedStats::default(),
            current_durability: 100.0,
            quality: 0.5,
            owner: ItemOwner::Ground(0, 0),
            stack_count: 1,
            created_tick: 0,
            creator_id: None,
            equipped_slot: None,
        }
    }

    #[test]
    fn item_derived_stats_from_material_stats_preserves_values() {
        let stats = ItemDerivedStats::from_material_stats(4.2, 2.5, 18.0);
        assert_eq!(stats.damage, 4.2);
        assert_eq!(stats.speed, 2.5);
        assert_eq!(stats.max_durability, 18.0);
    }

    #[test]
    fn item_instance_stacking_rules() {
        let base = sample_item(1);
        let same = sample_item(2);
        let mut different_material = sample_item(3);
        different_material.material_id = "pine".to_string();

        assert!(base.is_stackable());
        assert!(base.can_stack_with(&same));
        assert!(!base.can_stack_with(&different_material));
    }

    #[test]
    fn item_instance_equipped_items_are_not_stackable() {
        let mut item = sample_item(1);
        item.equipped_slot = Some(EquipSlot::MainHand);
        assert!(!item.is_stackable());
    }

    #[test]
    fn item_is_broken_at_zero_durability() {
        let mut item = sample_item(1);
        item.current_durability = 0.0;
        assert!(item.is_broken());
    }
}
