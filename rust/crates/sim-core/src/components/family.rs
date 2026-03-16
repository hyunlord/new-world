use crate::ids::EntityId;
use serde::{Deserialize, Serialize};

/// Descent rule used when evaluating familial belonging.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum KinshipType {
    /// Both maternal and paternal kin are recognized equally.
    #[default]
    Bilateral = 0,
    /// Only paternal lineage is socially emphasized.
    Patrilineal = 1,
    /// Only maternal lineage is socially emphasized.
    Matrilineal = 2,
}

/// Cold genealogy data for one agent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FamilyComponent {
    /// Biological father, if known.
    pub father: Option<EntityId>,
    /// Biological mother, if known.
    pub mother: Option<EntityId>,
    /// Current spouse, if any.
    pub spouse: Option<EntityId>,
    /// Optional lineage / clan identifier for later clan systems.
    pub clan_id: Option<u32>,
    /// Generation counter relative to initial spawned population.
    pub generation: u16,
    /// Tick when the agent was born or spawned.
    pub birth_tick: u32,
    /// Descent rule inherited from settlement culture.
    pub kinship_type: KinshipType,
}

impl FamilyComponent {
    /// Returns true when both family records share at least one known parent.
    pub fn is_sibling_of(&self, other: &FamilyComponent) -> bool {
        let shared_father = self.father.is_some() && self.father == other.father;
        let shared_mother = self.mother.is_some() && self.mother == other.mother;
        shared_father || shared_mother
    }

    /// Returns true when `my_id` is recorded as one of the child's parents.
    pub fn is_parent_of(&self, child: &FamilyComponent, my_id: EntityId) -> bool {
        let _ = self;
        child.father == Some(my_id) || child.mother == Some(my_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{FamilyComponent, KinshipType};
    use crate::EntityId;

    #[test]
    fn default_is_orphan() {
        let family = FamilyComponent::default();
        assert!(family.father.is_none());
        assert!(family.mother.is_none());
        assert!(family.spouse.is_none());
        assert!(family.clan_id.is_none());
        assert_eq!(family.generation, 0);
        assert_eq!(family.birth_tick, 0);
        assert_eq!(family.kinship_type, KinshipType::Bilateral);
    }

    #[test]
    fn siblings_share_father() {
        let father = EntityId(10);
        let left = FamilyComponent {
            father: Some(father),
            ..FamilyComponent::default()
        };
        let right = FamilyComponent {
            father: Some(father),
            ..FamilyComponent::default()
        };

        assert!(left.is_sibling_of(&right));
    }

    #[test]
    fn siblings_share_mother() {
        let mother = EntityId(11);
        let left = FamilyComponent {
            mother: Some(mother),
            ..FamilyComponent::default()
        };
        let right = FamilyComponent {
            mother: Some(mother),
            ..FamilyComponent::default()
        };

        assert!(left.is_sibling_of(&right));
    }

    #[test]
    fn none_parents_not_siblings() {
        let left = FamilyComponent::default();
        let right = FamilyComponent::default();

        assert!(!left.is_sibling_of(&right));
    }

    #[test]
    fn parent_of_check() {
        let parent_id = EntityId(42);
        let parent = FamilyComponent::default();
        let child = FamilyComponent {
            father: Some(parent_id),
            ..FamilyComponent::default()
        };

        assert!(parent.is_parent_of(&child, parent_id));
        assert!(!parent.is_parent_of(&child, EntityId(99)));
    }
}
