
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