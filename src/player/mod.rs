//! Player module.
//!
//! Tracks all player-related data across sessions: identity, score, progress
//! through the puzzle sequence, and collected inventory items. The [`Player`]
//! state is fully serializable via `serde` so it can be persisted between
//! sessions and restored later.

use crate::errors::AppError;
use serde::{Deserialize, Serialize};

/// Persistent state for a single player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    /// Stable, unique identifier for the player.
    pub id: String,
    /// Accumulated score across all solved puzzles.
    pub score: u64,
    /// Index of the puzzle the player is currently on (zero-based).
    pub current_puzzle_index: usize,
    /// Items the player has collected during play.
    pub inventory: Vec<String>,
}

impl Player {
    /// Creates a new player with the given `id`, a zeroed score, progress at
    /// the first puzzle, and an empty inventory.
    pub fn new(id: impl Into<String>) -> Self {
        Player {
            id: id.into(),
            score: 0,
            current_puzzle_index: 0,
            inventory: Vec::new(),
        }
    }

    /// Adds `points` to the player's score, saturating at [`u64::MAX`] rather
    /// than overflowing.
    pub fn add_score(&mut self, points: u64) {
        self.score = self.score.saturating_add(points);
    }

    /// Advances the player to the next puzzle, returning the new index.
    pub fn advance_puzzle(&mut self) -> usize {
        self.current_puzzle_index = self.current_puzzle_index.saturating_add(1);
        self.current_puzzle_index
    }

    /// Sets the current puzzle index explicitly (e.g. when loading a save or
    /// jumping to a specific puzzle).
    pub fn set_puzzle_index(&mut self, index: usize) {
        self.current_puzzle_index = index;
    }

    /// Adds an item to the player's inventory.
    pub fn add_item(&mut self, item: impl Into<String>) {
        self.inventory.push(item.into());
    }

    /// Removes the first matching item from the inventory, returning `true` if
    /// an item was removed.
    pub fn remove_item(&mut self, item: &str) -> bool {
        if let Some(pos) = self.inventory.iter().position(|i| i == item) {
            self.inventory.remove(pos);
            true
        } else {
            false
        }
    }

    /// Returns `true` if the inventory contains `item`.
    pub fn has_item(&self, item: &str) -> bool {
        self.inventory.iter().any(|i| i == item)
    }

    /// Serializes the player state to a JSON string.
    pub fn to_json(&self) -> Result<String, AppError> {
        Ok(serde_json::to_string(self)?)
    }

    /// Deserializes a player from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, AppError> {
        Ok(serde_json::from_str(json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_player_has_default_state() {
        let player = Player::new("p1");
        assert_eq!(player.id, "p1");
        assert_eq!(player.score, 0);
        assert_eq!(player.current_puzzle_index, 0);
        assert!(player.inventory.is_empty());
    }

    #[test]
    fn add_score_accumulates() {
        let mut player = Player::new("p1");
        player.add_score(10);
        player.add_score(5);
        assert_eq!(player.score, 15);
    }

    #[test]
    fn add_score_saturates_on_overflow() {
        let mut player = Player::new("p1");
        player.add_score(u64::MAX);
        player.add_score(10);
        assert_eq!(player.score, u64::MAX);
    }

    #[test]
    fn advance_puzzle_increments_and_returns_index() {
        let mut player = Player::new("p1");
        assert_eq!(player.advance_puzzle(), 1);
        assert_eq!(player.advance_puzzle(), 2);
        assert_eq!(player.current_puzzle_index, 2);
    }

    #[test]
    fn set_puzzle_index_overrides_progress() {
        let mut player = Player::new("p1");
        player.set_puzzle_index(7);
        assert_eq!(player.current_puzzle_index, 7);
    }

    #[test]
    fn inventory_add_remove_and_query() {
        let mut player = Player::new("p1");
        player.add_item("key");
        player.add_item("torch");
        assert!(player.has_item("key"));
        assert_eq!(player.inventory.len(), 2);

        assert!(player.remove_item("key"));
        assert!(!player.has_item("key"));
        assert_eq!(player.inventory.len(), 1);

        assert!(!player.remove_item("missing"));
    }

    #[test]
    fn round_trips_through_json() {
        let mut player = Player::new("hero");
        player.add_score(42);
        player.advance_puzzle();
        player.add_item("map");

        let json = player.to_json().expect("serialize");
        let restored = Player::from_json(&json).expect("deserialize");

        assert_eq!(player, restored);
    }
}
