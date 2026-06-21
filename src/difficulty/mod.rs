//! Difficulty module.
//!
//! Defines and manages difficulty tiers that affect puzzle complexity,
//! score multipliers, and reward amounts. Puzzles are tagged with a
//! [`Difficulty`] level at load time.

use serde::{Deserialize, Serialize};

/// The difficulty tier of a puzzle.
///
/// Each tier carries a score multiplier and a base reward amount that scale
/// with difficulty to incentivise harder puzzles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    /// Beginner-friendly puzzles with a 1× score multiplier.
    Easy,
    /// Intermediate puzzles with a 1.5× score multiplier.
    Medium,
    /// Challenging puzzles with a 2× score multiplier.
    Hard,
}

impl Difficulty {
    /// Returns the score multiplier for this difficulty tier as a rational
    /// pair `(numerator, denominator)` to avoid floating-point arithmetic.
    ///
    /// | Tier   | Multiplier |
    /// |--------|-----------|
    /// | Easy   | 1 / 1     |
    /// | Medium | 3 / 2     |
    /// | Hard   | 2 / 1     |
    pub fn score_multiplier(&self) -> (u64, u64) {
        match self {
            Difficulty::Easy => (1, 1),
            Difficulty::Medium => (3, 2),
            Difficulty::Hard => (2, 1),
        }
    }

    /// Returns the base reward amount (e.g. tokens) granted for completing a
    /// puzzle at this difficulty level.
    ///
    /// | Tier   | Reward |
    /// |--------|--------|
    /// | Easy   | 10     |
    /// | Medium | 25     |
    /// | Hard   | 50     |
    pub fn reward_amount(&self) -> u32 {
        match self {
            Difficulty::Easy => 10,
            Difficulty::Medium => 25,
            Difficulty::Hard => 50,
        }
    }

    /// Applies the difficulty multiplier to `base_score`, rounding down.
    ///
    /// Uses integer arithmetic (no floating point) to keep results
    /// deterministic across all platforms.
    pub fn apply_multiplier(&self, base_score: u64) -> u64 {
        let (num, den) = self.score_multiplier();
        base_score.saturating_mul(num) / den
    }
}

/// A puzzle tagged with its difficulty level at load time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Puzzle {
    /// Unique identifier for the puzzle.
    pub id: String,
    /// The difficulty level assigned to this puzzle at load time.
    pub difficulty: Difficulty,
}

impl Puzzle {
    /// Creates a new [`Puzzle`] with the given `id` and `difficulty`.
    pub fn new(id: impl Into<String>, difficulty: Difficulty) -> Self {
        Puzzle {
            id: id.into(),
            difficulty,
        }
    }

    /// Returns the score earned for solving this puzzle given a `base_score`,
    /// scaled by the puzzle's difficulty multiplier.
    pub fn score_for_solving(&self, base_score: u64) -> u64 {
        self.difficulty.apply_multiplier(base_score)
    }

    /// Returns the reward amount granted for solving this puzzle.
    pub fn reward_amount(&self) -> u32 {
        self.difficulty.reward_amount()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Multiplier tests ---

    #[test]
    fn easy_multiplier_is_one_to_one() {
        assert_eq!(Difficulty::Easy.score_multiplier(), (1, 1));
    }

    #[test]
    fn medium_multiplier_is_three_halves() {
        assert_eq!(Difficulty::Medium.score_multiplier(), (3, 2));
    }

    #[test]
    fn hard_multiplier_is_two_to_one() {
        assert_eq!(Difficulty::Hard.score_multiplier(), (2, 1));
    }

    #[test]
    fn apply_multiplier_easy_keeps_score_unchanged() {
        assert_eq!(Difficulty::Easy.apply_multiplier(100), 100);
    }

    #[test]
    fn apply_multiplier_medium_gives_one_point_five_times() {
        assert_eq!(Difficulty::Medium.apply_multiplier(100), 150);
    }

    #[test]
    fn apply_multiplier_hard_doubles_score() {
        assert_eq!(Difficulty::Hard.apply_multiplier(100), 200);
    }

    #[test]
    fn apply_multiplier_truncates_on_odd_medium_score() {
        // 101 * 3 / 2 = 151 (integer division)
        assert_eq!(Difficulty::Medium.apply_multiplier(101), 151);
    }

    #[test]
    fn apply_multiplier_zero_base_score() {
        assert_eq!(Difficulty::Easy.apply_multiplier(0), 0);
        assert_eq!(Difficulty::Medium.apply_multiplier(0), 0);
        assert_eq!(Difficulty::Hard.apply_multiplier(0), 0);
    }

    // --- Reward tests ---

    #[test]
    fn easy_reward_amount() {
        assert_eq!(Difficulty::Easy.reward_amount(), 10);
    }

    #[test]
    fn medium_reward_amount() {
        assert_eq!(Difficulty::Medium.reward_amount(), 25);
    }

    #[test]
    fn hard_reward_amount() {
        assert_eq!(Difficulty::Hard.reward_amount(), 50);
    }

    #[test]
    fn reward_amounts_scale_with_difficulty() {
        assert!(Difficulty::Easy.reward_amount() < Difficulty::Medium.reward_amount());
        assert!(Difficulty::Medium.reward_amount() < Difficulty::Hard.reward_amount());
    }

    // --- Puzzle struct tests ---

    #[test]
    fn puzzle_new_assigns_id_and_difficulty() {
        let p = Puzzle::new("puzzle-1", Difficulty::Hard);
        assert_eq!(p.id, "puzzle-1");
        assert_eq!(p.difficulty, Difficulty::Hard);
    }

    #[test]
    fn puzzle_score_for_solving_uses_difficulty_multiplier() {
        let easy = Puzzle::new("e1", Difficulty::Easy);
        let medium = Puzzle::new("m1", Difficulty::Medium);
        let hard = Puzzle::new("h1", Difficulty::Hard);

        assert_eq!(easy.score_for_solving(200), 200);
        assert_eq!(medium.score_for_solving(200), 300);
        assert_eq!(hard.score_for_solving(200), 400);
    }

    #[test]
    fn puzzle_reward_amount_matches_difficulty() {
        assert_eq!(
            Puzzle::new("p1", Difficulty::Easy).reward_amount(),
            Difficulty::Easy.reward_amount()
        );
        assert_eq!(
            Puzzle::new("p2", Difficulty::Medium).reward_amount(),
            Difficulty::Medium.reward_amount()
        );
        assert_eq!(
            Puzzle::new("p3", Difficulty::Hard).reward_amount(),
            Difficulty::Hard.reward_amount()
        );
    }

    #[test]
    fn puzzle_serializes_and_deserializes() {
        let puzzle = Puzzle::new("soroban-1", Difficulty::Medium);
        let json = serde_json::to_string(&puzzle).expect("serialize");
        let restored: Puzzle = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(puzzle, restored);
    }

    #[test]
    fn difficulty_enum_serializes_to_string_variant() {
        let json = serde_json::to_string(&Difficulty::Hard).expect("serialize");
        assert_eq!(json, "\"Hard\"");
        let restored: Difficulty = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored, Difficulty::Hard);
    }
}
