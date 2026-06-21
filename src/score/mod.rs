
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Score {
    current_score: u32,
    high_score: u32,
}

impl Score {
    pub fn new() -> Self {
        Score {
            current_score: 0,
            high_score: 0,
        }
    }

    pub fn add_score(&mut self, difficulty: u32, time_taken: u32) {
        let score_to_add = difficulty * 100 / (time_taken + 1);
        self.current_score += score_to_add;
        if self.current_score > self.high_score {
            self.high_score = self.current_score;
        }
    }

    pub fn get_current_score(&self) -> u32 {
        self.current_score
    }

    pub fn get_high_score(&self) -> u32 {
        self.high_score
    }

    /// Awards a time bonus based on elapsed milliseconds: faster completions earn more points.
    /// Bonus = difficulty * 1000 / (elapsed_ms / 100 + 1), capped to prevent overflow.
    pub fn apply_time_bonus(&mut self, elapsed_ms: u64, difficulty: u32) {
        let time_units = (elapsed_ms / 100) as u32;
        let bonus = difficulty.saturating_mul(1000) / (time_units + 1);
        self.current_score = self.current_score.saturating_add(bonus);
        if self.current_score > self.high_score {
            self.high_score = self.current_score;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_score() {
        let mut score = Score::new();
        score.add_score(1, 10);
        assert_eq!(score.get_current_score(), 9);
        assert_eq!(score.get_high_score(), 9);
    }

    #[test]
    fn test_high_score() {
        let mut score = Score::new();
        score.add_score(1, 10);
        score.add_score(2, 5);
        assert_eq!(score.get_current_score(), 42);
        assert_eq!(score.get_high_score(), 42);
    }

    #[test]
    fn test_apply_time_bonus_fast_completion() {
        let mut score = Score::new();
        score.apply_time_bonus(500, 2); // 500ms elapsed, difficulty 2
        assert!(score.get_current_score() > 0);
        assert_eq!(score.get_current_score(), score.get_high_score());
    }

    #[test]
    fn test_apply_time_bonus_slow_completion_less_reward() {
        let mut score = Score::new();
        let mut fast_score = Score::new();
        score.apply_time_bonus(10_000, 2); // 10 seconds
        fast_score.apply_time_bonus(500, 2); // 0.5 seconds
        assert!(fast_score.get_current_score() > score.get_current_score());
    }

    #[test]
    fn test_serialization() {
        let score = Score {
            current_score: 100,
            high_score: 200,
        };
        let serialized = serde_json::to_string(&score).unwrap();
        let deserialized: Score = serde_json::from_str(&serialized).unwrap();
        assert_eq!(score, deserialized);
    }
}