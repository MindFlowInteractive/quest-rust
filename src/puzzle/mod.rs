//! Puzzle module.
//!
//! Models individual puzzles as cause-and-effect chains.
//! A [`Puzzle`] holds an id, description, a set of required [`Condition`]s
//! that must be satisfied, and the [`Effect`]s that are triggered once the
//! puzzle is solved.
//!
//! Calling [`Puzzle::evaluate`] with a player action advances the puzzle
//! through the [`PuzzleState`] lifecycle:
//!
//! ```text
//! Unsolved ──(first matching action)──► InProgress
//!           ──(all conditions met)────► Solved
//! ```

use serde::{Deserialize, Serialize};

// ── Condition ────────────────────────────────────────────────────────────────

/// A condition that must be satisfied to make progress on a puzzle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Condition {
    /// Unique identifier for this condition.
    pub id: String,
    /// Human-readable description of what must be done.
    pub description: String,
    /// Whether this condition has been satisfied.
    pub satisfied: bool,
}

impl Condition {
    /// Creates a new unsatisfied condition.
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        Condition {
            id: id.into(),
            description: description.into(),
            satisfied: false,
        }
    }
}

// ── Effect ────────────────────────────────────────────────────────────────────

/// An effect that is triggered when a puzzle is solved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    /// Award the player a number of score points.
    AwardScore(u64),
    /// Grant the player an inventory item (item-id, quantity).
    GrantItem { item_id: String, quantity: u32 },
    /// Unlock a named achievement.
    UnlockAchievement(String),
}

// ── PuzzleState ───────────────────────────────────────────────────────────────

/// The lifecycle state of a puzzle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PuzzleState {
    /// The player has not yet interacted with this puzzle.
    #[default]
    Unsolved,
    /// The player has started working on the puzzle but has not met all
    /// conditions yet.
    InProgress,
    /// All conditions have been met and the puzzle is complete.
    Solved,
}

// ── Puzzle ────────────────────────────────────────────────────────────────────

/// A puzzle modelled as a cause-and-effect chain.
///
/// Progress is driven by [`Puzzle::evaluate`], which accepts a player action
/// string and attempts to satisfy matching conditions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Puzzle {
    /// Unique identifier for this puzzle.
    pub id: String,
    /// Human-readable description shown to the player.
    pub description: String,
    /// Ordered list of conditions that must all be satisfied to solve the
    /// puzzle.
    pub conditions: Vec<Condition>,
    /// Effects triggered when all conditions are satisfied.
    pub effects: Vec<Effect>,
    /// Current lifecycle state of the puzzle.
    pub state: PuzzleState,
}

impl Puzzle {
    /// Creates a new puzzle in the [`PuzzleState::Unsolved`] state.
    ///
    /// # Arguments
    ///
    /// * `id`          – stable unique identifier.
    /// * `description` – player-facing description.
    /// * `conditions`  – the conditions that must be satisfied.
    /// * `effects`     – the effects triggered on completion.
    pub fn new(
        id: impl Into<String>,
        description: impl Into<String>,
        conditions: Vec<Condition>,
        effects: Vec<Effect>,
    ) -> Self {
        Puzzle {
            id: id.into(),
            description: description.into(),
            conditions,
            effects,
            state: PuzzleState::Unsolved,
        }
    }

    /// Returns `true` if the puzzle is in the [`PuzzleState::Solved`] state.
    pub fn is_solved(&self) -> bool {
        self.state == PuzzleState::Solved
    }

    /// Returns `true` if every condition has been satisfied.
    fn all_conditions_met(&self) -> bool {
        self.conditions.iter().all(|c| c.satisfied)
    }

    /// Processes a player `action` string and updates the puzzle state.
    ///
    /// An action satisfies a condition whose `id` matches the action string.
    /// The first time any condition is satisfied the state transitions from
    /// [`PuzzleState::Unsolved`] to [`PuzzleState::InProgress`].
    /// When all conditions are satisfied the state transitions to
    /// [`PuzzleState::Solved`] and the puzzle's effects are returned.
    ///
    /// Returns a slice of [`Effect`]s that should be applied (non-empty only
    /// when the puzzle transitions to [`PuzzleState::Solved`] during this
    /// call); returns an empty slice otherwise.
    ///
    /// Calling `evaluate` on an already-[`PuzzleState::Solved`] puzzle is a
    /// no-op and returns an empty slice.
    pub fn evaluate(&mut self, action: &str) -> &[Effect] {
        if self.state == PuzzleState::Solved {
            return &[];
        }

        // A puzzle with no conditions is immediately solvable on the first
        // evaluate call regardless of the action string.
        if self.conditions.is_empty() {
            self.state = PuzzleState::Solved;
            return &self.effects;
        }

        // Satisfy matching unsatisfied conditions.
        let mut any_satisfied = false;
        for condition in self.conditions.iter_mut() {
            if !condition.satisfied && condition.id == action {
                condition.satisfied = true;
                any_satisfied = true;
            }
        }

        // Advance state based on what changed.
        if any_satisfied || self.state == PuzzleState::InProgress {
            if self.all_conditions_met() {
                self.state = PuzzleState::Solved;
                return &self.effects;
            } else if any_satisfied && self.state == PuzzleState::Unsolved {
                self.state = PuzzleState::InProgress;
            }
        }

        &[]
    }

    /// Serializes the puzzle state to a JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserializes a puzzle from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn single_condition_puzzle() -> Puzzle {
        Puzzle::new(
            "p1",
            "Pull the lever",
            vec![Condition::new("pull_lever", "Pull the lever")],
            vec![Effect::AwardScore(50)],
        )
    }

    fn multi_condition_puzzle() -> Puzzle {
        Puzzle::new(
            "p2",
            "Light all torches",
            vec![
                Condition::new("light_torch_a", "Light torch A"),
                Condition::new("light_torch_b", "Light torch B"),
                Condition::new("light_torch_c", "Light torch C"),
            ],
            vec![
                Effect::AwardScore(100),
                Effect::GrantItem {
                    item_id: "fire_badge".to_string(),
                    quantity: 1,
                },
                Effect::UnlockAchievement("Flame Keeper".to_string()),
            ],
        )
    }

    // ── Initial state ──────────────────────────────────────────────────────

    #[test]
    fn new_puzzle_is_unsolved() {
        let puzzle = single_condition_puzzle();
        assert_eq!(puzzle.state, PuzzleState::Unsolved);
        assert!(!puzzle.is_solved());
    }

    // ── Single-condition puzzles ───────────────────────────────────────────

    #[test]
    fn irrelevant_action_leaves_state_unchanged() {
        let mut puzzle = single_condition_puzzle();
        let effects = puzzle.evaluate("do_nothing");
        assert!(effects.is_empty());
        assert_eq!(puzzle.state, PuzzleState::Unsolved);
    }

    #[test]
    fn correct_action_solves_single_condition_puzzle() {
        let mut puzzle = single_condition_puzzle();
        let effects = puzzle.evaluate("pull_lever");
        assert_eq!(puzzle.state, PuzzleState::Solved);
        assert!(puzzle.is_solved());
        assert_eq!(effects, &[Effect::AwardScore(50)]);
    }

    // ── Multi-condition puzzles ────────────────────────────────────────────

    #[test]
    fn first_action_transitions_to_in_progress() {
        let mut puzzle = multi_condition_puzzle();
        let effects = puzzle.evaluate("light_torch_a");
        assert_eq!(puzzle.state, PuzzleState::InProgress);
        assert!(effects.is_empty());
    }

    #[test]
    fn partial_conditions_keep_puzzle_in_progress() {
        let mut puzzle = multi_condition_puzzle();
        puzzle.evaluate("light_torch_a");
        puzzle.evaluate("light_torch_b");
        assert_eq!(puzzle.state, PuzzleState::InProgress);
        assert!(!puzzle.is_solved());
    }

    #[test]
    fn all_conditions_met_solves_puzzle_and_returns_effects() {
        let mut puzzle = multi_condition_puzzle();
        puzzle.evaluate("light_torch_a");
        puzzle.evaluate("light_torch_b");
        let effects = puzzle.evaluate("light_torch_c");

        assert_eq!(puzzle.state, PuzzleState::Solved);
        assert!(puzzle.is_solved());
        assert_eq!(effects.len(), 3);
        assert!(effects.contains(&Effect::AwardScore(100)));
        assert!(effects.contains(&Effect::GrantItem {
            item_id: "fire_badge".to_string(),
            quantity: 1,
        }));
        assert!(effects.contains(&Effect::UnlockAchievement("Flame Keeper".to_string())));
    }

    #[test]
    fn duplicate_action_does_not_re_satisfy_condition() {
        let mut puzzle = multi_condition_puzzle();
        puzzle.evaluate("light_torch_a");
        puzzle.evaluate("light_torch_a"); // duplicate — should be ignored
        assert_eq!(puzzle.state, PuzzleState::InProgress);
        let satisfied = puzzle.conditions.iter().filter(|c| c.satisfied).count();
        assert_eq!(satisfied, 1);
    }

    // ── Solved puzzle is a no-op ───────────────────────────────────────────

    #[test]
    fn evaluate_on_solved_puzzle_is_noop() {
        let mut puzzle = single_condition_puzzle();
        puzzle.evaluate("pull_lever"); // solve it
        let effects = puzzle.evaluate("pull_lever"); // call again
        assert!(effects.is_empty());
        assert_eq!(puzzle.state, PuzzleState::Solved);
    }

    // ── Out-of-order actions ───────────────────────────────────────────────

    #[test]
    fn conditions_can_be_satisfied_in_any_order() {
        let mut puzzle = multi_condition_puzzle();
        puzzle.evaluate("light_torch_c");
        puzzle.evaluate("light_torch_a");
        let effects = puzzle.evaluate("light_torch_b");
        assert_eq!(puzzle.state, PuzzleState::Solved);
        assert!(!effects.is_empty());
    }

    // ── No conditions edge case ────────────────────────────────────────────

    #[test]
    fn puzzle_with_no_conditions_solves_immediately_on_any_action() {
        let mut puzzle = Puzzle::new(
            "p-trivial",
            "Just walk in",
            vec![],
            vec![Effect::AwardScore(10)],
        );
        // all_conditions_met() is vacuously true, but evaluate() only checks
        // after a condition fires or state is InProgress.
        // A puzzle with no conditions is already fully met — first evaluate
        // call should recognise this and jump to Solved.
        let effects = puzzle.evaluate("anything");
        assert_eq!(puzzle.state, PuzzleState::Solved);
        assert_eq!(effects, &[Effect::AwardScore(10)]);
    }

    // ── Serialization round-trip ───────────────────────────────────────────

    #[test]
    fn serialization_round_trip() {
        let mut puzzle = multi_condition_puzzle();
        puzzle.evaluate("light_torch_a");

        let json = puzzle.to_json().expect("serialize");
        let restored = Puzzle::from_json(&json).expect("deserialize");
        assert_eq!(puzzle, restored);
    }
}
