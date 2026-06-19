//! Hints module.
//!
//! Provides contextual hints for puzzles. Each puzzle may have 1–3 ordered
//! hints that are revealed one at a time. Revealing a hint applies a score
//! penalty to the player.

use serde::{Deserialize, Serialize};

/// The score penalty deducted each time a hint is revealed.
pub const HINT_PENALTY: u64 = 10;

/// A single hint for a puzzle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hint {
    /// The hint text shown to the player.
    pub text: String,
}

/// Manages the ordered hints for a single puzzle and tracks how many have
/// been revealed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HintSystem {
    /// Ordered list of hints (at most 3).
    hints: Vec<Hint>,
    /// Number of hints revealed so far.
    revealed: usize,
}

impl HintSystem {
    /// Creates a new [`HintSystem`] with the given hints.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if more than 3 hints are supplied.
    pub fn new(hints: Vec<Hint>) -> Self {
        debug_assert!(hints.len() <= 3, "a puzzle may have at most 3 hints");
        Self { hints, revealed: 0 }
    }

    /// Reveals the next hint and returns it together with the penalty that
    /// should be deducted from the player's score.
    ///
    /// Returns `None` when all hints have already been revealed.
    pub fn reveal_next(&mut self) -> Option<(&Hint, u64)> {
        if self.revealed < self.hints.len() {
            let hint = &self.hints[self.revealed];
            self.revealed += 1;
            Some((hint, HINT_PENALTY))
        } else {
            None
        }
    }

    /// Returns the number of hints that have been revealed so far.
    pub fn revealed_count(&self) -> usize {
        self.revealed
    }

    /// Returns the total number of hints available for this puzzle.
    pub fn total_hints(&self) -> usize {
        self.hints.len()
    }

    /// Returns `true` if all hints have been revealed.
    pub fn all_revealed(&self) -> bool {
        self.revealed >= self.hints.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_hints(texts: &[&str]) -> Vec<Hint> {
        texts.iter().map(|t| Hint { text: t.to_string() }).collect()
    }

    #[test]
    fn reveals_hints_in_order() {
        let mut hs = HintSystem::new(make_hints(&["First", "Second", "Third"]));

        let (h1, _) = hs.reveal_next().expect("first hint");
        assert_eq!(h1.text, "First");
        assert_eq!(hs.revealed_count(), 1);

        let (h2, _) = hs.reveal_next().expect("second hint");
        assert_eq!(h2.text, "Second");
        assert_eq!(hs.revealed_count(), 2);

        let (h3, _) = hs.reveal_next().expect("third hint");
        assert_eq!(h3.text, "Third");
        assert_eq!(hs.revealed_count(), 3);
    }

    #[test]
    fn returns_none_when_exhausted() {
        let mut hs = HintSystem::new(make_hints(&["Only"]));
        hs.reveal_next();
        assert!(hs.reveal_next().is_none());
        assert!(hs.all_revealed());
    }

    #[test]
    fn score_penalty_applied_per_reveal() {
        let mut hs = HintSystem::new(make_hints(&["A", "B"]));
        let mut score: u64 = 100;

        let (_, penalty) = hs.reveal_next().unwrap();
        score = score.saturating_sub(penalty);
        assert_eq!(score, 90);

        let (_, penalty) = hs.reveal_next().unwrap();
        score = score.saturating_sub(penalty);
        assert_eq!(score, 80);
    }

    #[test]
    fn penalty_value_is_correct() {
        let mut hs = HintSystem::new(make_hints(&["hint"]));
        let (_, penalty) = hs.reveal_next().unwrap();
        assert_eq!(penalty, HINT_PENALTY);
    }

    #[test]
    fn total_and_revealed_counts() {
        let mut hs = HintSystem::new(make_hints(&["a", "b"]));
        assert_eq!(hs.total_hints(), 2);
        assert_eq!(hs.revealed_count(), 0);
        hs.reveal_next();
        assert_eq!(hs.revealed_count(), 1);
        assert!(!hs.all_revealed());
        hs.reveal_next();
        assert!(hs.all_revealed());
    }

    #[test]
    fn single_hint_puzzle() {
        let mut hs = HintSystem::new(make_hints(&["Only hint"]));
        assert_eq!(hs.total_hints(), 1);
        let (h, p) = hs.reveal_next().unwrap();
        assert_eq!(h.text, "Only hint");
        assert_eq!(p, HINT_PENALTY);
        assert!(hs.all_revealed());
    }
}