use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entry {
    pub player_id: String,
    pub score: u64,
    pub timestamp: u64,
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher score comes first; if equal, earlier timestamp first
        other
            .score
            .cmp(&self.score)
            .then_with(|| self.timestamp.cmp(&other.timestamp))
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Leaderboard {
    max_entries: usize,
    entries: Vec<Entry>,
}

impl Leaderboard {
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, entry: Entry) {
        // If board not full, just insert and sort
        if self.entries.len() < self.max_entries {
            self.entries.push(entry);
            self.entries.sort();
            return;
        }
        // Board full: compare with lowest-ranked (last after descending sort).
        // Our Ord puts higher scores first, so `entry < *worst` means entry
        // outranks worst (has a higher score).
        if let Some(worst) = self.entries.last()
            && entry < *worst
        {
            self.entries.pop();
            self.entries.push(entry);
            self.entries.sort();
        }
    }

    pub fn top(&self) -> &[Entry] {
        &self.entries
    }

    // Helpers for serialization
    pub fn to_json(&self) -> Result<String, AppError> {
        Ok(serde_json::to_string(&self)?)
    }

    pub fn from_json(s: &str) -> Result<Self, AppError> {
        Ok(serde_json::from_str(s)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn insertion_and_ordering() {
        let mut lb = Leaderboard::new(3);
        lb.insert(Entry {
            player_id: "a".into(),
            score: 10,
            timestamp: 1,
        });
        lb.insert(Entry {
            player_id: "b".into(),
            score: 30,
            timestamp: 2,
        });
        lb.insert(Entry {
            player_id: "c".into(),
            score: 20,
            timestamp: 3,
        });
        let scores: Vec<u64> = lb.top().iter().map(|e| e.score).collect();
        assert_eq!(scores, vec![30, 20, 10]);
    }

    #[test]
    fn replacement_when_full() {
        let mut lb = Leaderboard::new(2);
        lb.insert(Entry {
            player_id: "a".into(),
            score: 10,
            timestamp: 1,
        });
        lb.insert(Entry {
            player_id: "b".into(),
            score: 20,
            timestamp: 2,
        });
        // Insert lower score – should be ignored
        lb.insert(Entry {
            player_id: "c".into(),
            score: 5,
            timestamp: 3,
        });
        assert_eq!(lb.top().len(), 2);
        // Insert higher score – should replace lowest (10)
        lb.insert(Entry {
            player_id: "d".into(),
            score: 30,
            timestamp: 4,
        });
        let ids: Vec<&str> = lb.top().iter().map(|e| e.player_id.as_str()).collect();
        assert!(ids.contains(&"b") && ids.contains(&"d"));
        assert!(!ids.contains(&"a"));
    }

    #[test]
    fn serialization_roundtrip() {
        let mut lb = Leaderboard::new(2);
        lb.insert(Entry {
            player_id: "x".into(),
            score: 100,
            timestamp: 10,
        });
        let json = lb.to_json().expect("serialize");
        let restored = Leaderboard::from_json(&json).expect("deserialize");
        assert_eq!(lb.top(), restored.top());
    }
}
