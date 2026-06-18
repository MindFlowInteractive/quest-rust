//! Inventory module.
//!
//! Manages items (tools, tokens) a player collects during gameplay.
//! The [`Inventory`] is fully serializable to support game save persistence.

use serde::{Deserialize, Serialize};

/// An item collected by the player during gameplay.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    /// Unique identifier for the item type.
    pub id: String,
    /// Human-readable name of the item.
    pub name: String,
    /// Number of units of this item in possession.
    pub quantity: u32,
}

/// A player's inventory containing collected items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Inventory {
    /// List of items currently in the inventory.
    pub items: Vec<Item>,
}

impl Inventory {
    /// Creates a new empty inventory.
    pub fn new() -> Self {
        Inventory { items: Vec::new() }
    }

    /// Adds an item with the given `id`, `name`, and `quantity`.
    /// If an item with the same `id` already exists, its quantity is increased
    /// using saturating addition to prevent overflow. The name is also updated.
    pub fn add_item(&mut self, id: String, name: String, quantity: u32) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.name = name;
            item.quantity = item.quantity.saturating_add(quantity);
        } else {
            self.items.push(Item { id, name, quantity });
        }
    }

    /// Removes a specified `quantity` of an item with the given `id`.
    /// If the item's quantity falls to 0 or below, it is removed from the inventory.
    /// Returns `true` if the item was found and modified/removed, `false` otherwise.
    pub fn remove_item(&mut self, id: &str, quantity: u32) -> bool {
        if let Some(pos) = self.items.iter().position(|i| i.id == id) {
            let item = &mut self.items[pos];
            if item.quantity <= quantity {
                self.items.remove(pos);
            } else {
                item.quantity -= quantity;
            }
            true
        } else {
            false
        }
    }

    /// Queries the quantity of an item with the given `id`.
    /// Returns 0 if the item is not present.
    pub fn get_quantity(&self, id: &str) -> u32 {
        self.items.iter().find(|i| i.id == id).map(|i| i.quantity).unwrap_or(0)
    }

    /// Queries an item by its `id`.
    /// Returns `None` if the item is not present.
    pub fn get_item(&self, id: &str) -> Option<&Item> {
        self.items.iter().find(|i| i.id == id)
    }

    /// Returns `true` if the inventory contains no items.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Serializes the inventory state to a JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserializes an inventory state from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_items() {
        let mut inv = Inventory::new();
        assert!(inv.is_empty());

        // Add a new item
        inv.add_item("potion".to_string(), "Health Potion".to_string(), 5);
        assert_eq!(inv.get_quantity("potion"), 5);
        assert_eq!(inv.items.len(), 1);

        // Add quantity to existing item
        inv.add_item("potion".to_string(), "Super Health Potion".to_string(), 3);
        assert_eq!(inv.get_quantity("potion"), 8);
        assert_eq!(inv.get_item("potion").unwrap().name, "Super Health Potion");
        assert_eq!(inv.items.len(), 1);

        // Add a different item
        inv.add_item("key".to_string(), "Iron Key".to_string(), 1);
        assert_eq!(inv.get_quantity("key"), 1);
        assert_eq!(inv.items.len(), 2);
    }

    #[test]
    fn test_remove_items() {
        let mut inv = Inventory::new();
        inv.add_item("gold".to_string(), "Gold Coins".to_string(), 100);

        // Remove partial quantity
        assert!(inv.remove_item("gold", 40));
        assert_eq!(inv.get_quantity("gold"), 60);

        // Remove exact remaining quantity
        assert!(inv.remove_item("gold", 60));
        assert_eq!(inv.get_quantity("gold"), 0);
        assert!(inv.is_empty());

        // Remove non-existent item
        assert!(!inv.remove_item("gold", 10));
    }

    #[test]
    fn test_overflow_protection() {
        let mut inv = Inventory::new();
        inv.add_item("token".to_string(), "Game Token".to_string(), u32::MAX - 5);
        assert_eq!(inv.get_quantity("token"), u32::MAX - 5);

        // Exceed u32::MAX
        inv.add_item("token".to_string(), "Game Token".to_string(), 10);
        assert_eq!(inv.get_quantity("token"), u32::MAX);
    }

    #[test]
    fn test_empty_and_serialization() {
        let inv = Inventory::new();
        assert!(inv.is_empty());
        assert_eq!(inv.get_quantity("any"), 0);
        assert!(!inv.items.iter().any(|_| true));

        // Serialize empty
        let json_empty = inv.to_json().unwrap();
        let restored_empty = Inventory::from_json(&json_empty).unwrap();
        assert!(restored_empty.is_empty());

        // Serialize non-empty
        let mut populated = Inventory::new();
        populated.add_item("sword".to_string(), "Iron Sword".to_string(), 1);
        let json_populated = populated.to_json().unwrap();
        let restored_populated = Inventory::from_json(&json_populated).unwrap();
        assert_eq!(restored_populated.get_quantity("sword"), 1);
        assert_eq!(restored_populated.get_item("sword").unwrap().name, "Iron Sword");
    }
}
