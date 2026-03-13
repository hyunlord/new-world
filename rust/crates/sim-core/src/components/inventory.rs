use serde::{Deserialize, Serialize};

use crate::item::ItemId;

/// Per-agent inventory component. Holds item IDs only.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Inventory {
    /// Item IDs carried by the agent.
    pub items: Vec<ItemId>,
    /// Maximum tool slots available to the agent.
    pub max_tool_slots: u32,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            max_tool_slots: 2,
        }
    }

    /// Number of items currently held.
    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// Returns true if the inventory contains the given item.
    pub fn contains(&self, id: ItemId) -> bool {
        self.items.contains(&id)
    }

    /// Adds one item ID to the inventory.
    pub fn add(&mut self, id: ItemId) {
        self.items.push(id);
    }

    /// Removes one item ID from the inventory.
    pub fn remove(&mut self, id: ItemId) -> bool {
        if let Some(pos) = self.items.iter().position(|item_id| *item_id == id) {
            self.items.swap_remove(pos);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_add_remove() {
        let mut inventory = Inventory::new();
        let item = ItemId(42);
        assert_eq!(inventory.count(), 0);
        assert!(!inventory.contains(item));

        inventory.add(item);
        assert_eq!(inventory.count(), 1);
        assert!(inventory.contains(item));

        assert!(inventory.remove(item));
        assert_eq!(inventory.count(), 0);
        assert!(!inventory.contains(item));
        assert!(!inventory.remove(item));
    }
}
