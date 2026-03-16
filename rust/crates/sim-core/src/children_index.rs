use crate::ids::EntityId;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::HashMap;

/// Global reverse index from parent id to child ids.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChildrenIndex {
    /// Parent entity id → owned child list.
    pub map: HashMap<EntityId, SmallVec<[EntityId; 8]>>,
}

impl ChildrenIndex {
    /// Adds one child to the parent's reverse index.
    pub fn add_child(&mut self, parent: EntityId, child: EntityId) {
        self.map.entry(parent).or_default().push(child);
    }

    /// Returns a slice of children for the requested parent.
    pub fn children_of(&self, parent: EntityId) -> &[EntityId] {
        self.map.get(&parent).map(|children| children.as_slice()).unwrap_or(&[])
    }

    /// Removes one child from the parent's reverse index.
    pub fn remove_child(&mut self, parent: EntityId, child: EntityId) {
        if let Some(children) = self.map.get_mut(&parent) {
            children.retain(|existing| *existing != child);
        }
    }

    /// Returns how many indexed children the parent currently has.
    pub fn child_count(&self, parent: EntityId) -> usize {
        self.map.get(&parent).map(|children| children.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::ChildrenIndex;
    use crate::EntityId;

    #[test]
    fn add_and_retrieve() {
        let mut index = ChildrenIndex::default();
        index.add_child(EntityId(1), EntityId(2));
        index.add_child(EntityId(1), EntityId(3));

        assert_eq!(index.children_of(EntityId(1)), &[EntityId(2), EntityId(3)]);
    }

    #[test]
    fn remove_child() {
        let mut index = ChildrenIndex::default();
        index.add_child(EntityId(1), EntityId(2));
        index.add_child(EntityId(1), EntityId(3));

        index.remove_child(EntityId(1), EntityId(2));

        assert_eq!(index.children_of(EntityId(1)), &[EntityId(3)]);
    }

    #[test]
    fn empty_returns_empty_slice() {
        let index = ChildrenIndex::default();
        assert!(index.children_of(EntityId(77)).is_empty());
    }

    #[test]
    fn child_count() {
        let mut index = ChildrenIndex::default();
        index.add_child(EntityId(5), EntityId(9));
        index.add_child(EntityId(5), EntityId(10));

        assert_eq!(index.child_count(EntityId(5)), 2);
        assert_eq!(index.child_count(EntityId(6)), 0);
    }
}
