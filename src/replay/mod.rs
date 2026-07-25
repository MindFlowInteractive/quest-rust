use crate::config::DEFAULT_SAVE_PATH;
use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayEvent {
    SessionStart,
    SessionEnd,
    HintReveal,
    AnswerAttempt { answer: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplaySession {
    pub puzzle_content_hash: String,
    pub events: Vec<(u64, ReplayEvent)>,
    pub total_time_secs: u64,
    pub hints_used: u32,
    pub attempt_count: u32,
    pub final_score: u64,
}

impl ReplaySession {
    pub fn new(puzzle_content_hash: impl Into<String>) -> Self {
        ReplaySession {
            puzzle_content_hash: puzzle_content_hash.into(),
            events: Vec::new(),
            total_time_secs: 0,
            hints_used: 0,
            attempt_count: 0,
            final_score: 0,
        }
    }

    pub fn record(&mut self, event: ReplayEvent) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;
        self.events.push((now, event));
    }

    pub fn save(&self) -> Result<String, AppError> {
        let dir = Path::new(DEFAULT_SAVE_PATH);
        if !dir.exists() {
            fs::create_dir_all(dir).map_err(AppError::Io)?;
        }
        let filename = format!(
            "{}.qreplay",
            &self.puzzle_content_hash[..16.min(self.puzzle_content_hash.len())]
        );
        let full_path = dir.join(&filename);
        let bytes = bincode::serialize(self).map_err(|e| AppError::Io(std::io::Error::other(e)))?;
        fs::write(&full_path, bytes).map_err(AppError::Io)?;
        Ok(full_path.to_string_lossy().to_string())
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let bytes = fs::read(path.as_ref()).map_err(AppError::Io)?;
        bincode::deserialize(&bytes)
            .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
    }

    pub fn verify_integrity(&self, puzzle_content_hash: &str) -> bool {
        self.puzzle_content_hash == puzzle_content_hash
    }

    pub fn playback(&self, speed: u32) {
        if self.events.is_empty() {
            println!("Replay contains no events.");
            return;
        }

        let speed = speed.max(1);
        let mut last_ts = self.events[0].0;

        println!(
            "Replaying {} events at {}x speed...",
            self.events.len(),
            speed
        );
        println!("Puzzle content hash: {}", self.puzzle_content_hash);
        println!();

        for (ts, event) in &self.events {
            let delta = ts.saturating_sub(last_ts) / speed as u64;
            if delta > 0 {
                sleep(Duration::from_micros(delta));
            }
            last_ts = *ts;

            match event {
                ReplayEvent::SessionStart => println!("[Session Start]"),
                ReplayEvent::SessionEnd => println!("[Session End]"),
                ReplayEvent::HintReveal => println!("[Hint Reveal]"),
                ReplayEvent::AnswerAttempt { answer } => {
                    println!("[Answer Attempt] \"{answer}\"")
                }
            }
        }

        println!();
        println!("=== Replay Summary ===");
        println!("Total time:     {}s", self.total_time_secs);
        println!("Hints used:     {}", self.hints_used);
        println!("Attempts:       {}", self.attempt_count);
        println!("Final score:    {}", self.final_score);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn records_events_in_order() {
        let mut replay = ReplaySession::new("abc123");
        replay.record(ReplayEvent::SessionStart);
        replay.record(ReplayEvent::HintReveal);
        replay.record(ReplayEvent::AnswerAttempt {
            answer: "test".into(),
        });
        replay.record(ReplayEvent::SessionEnd);

        assert_eq!(replay.events.len(), 4);
        assert!(matches!(replay.events[0].1, ReplayEvent::SessionStart));
        assert!(matches!(replay.events[1].1, ReplayEvent::HintReveal));
        assert!(matches!(
            replay.events[2].1,
            ReplayEvent::AnswerAttempt { .. }
        ));
        assert!(matches!(replay.events[3].1, ReplayEvent::SessionEnd));
    }

    #[test]
    fn timestamps_are_monotonic() {
        let mut replay = ReplaySession::new("abc123");
        replay.record(ReplayEvent::SessionStart);
        std::thread::sleep(Duration::from_micros(10));
        replay.record(ReplayEvent::HintReveal);

        assert!(replay.events[1].0 > replay.events[0].0);
    }

    #[test]
    fn serialization_roundtrip() {
        let mut replay = ReplaySession::new("hash123");
        replay.record(ReplayEvent::SessionStart);
        replay.record(ReplayEvent::HintReveal);
        replay.total_time_secs = 42;
        replay.hints_used = 1;
        replay.attempt_count = 3;
        replay.final_score = 100;

        let bytes = bincode::serialize(&replay).unwrap();
        let restored: ReplaySession = bincode::deserialize(&bytes).unwrap();

        assert_eq!(replay, restored);
    }

    #[test]
    fn save_and_load_qreplay() {
        let mut replay = ReplaySession::new("test-hash-123");
        replay.record(ReplayEvent::SessionStart);
        replay.record(ReplayEvent::AnswerAttempt {
            answer: "foo".into(),
        });
        replay.record(ReplayEvent::SessionEnd);
        replay.total_time_secs = 10;
        replay.hints_used = 2;
        replay.attempt_count = 1;
        replay.final_score = 50;

        let path = replay.save().expect("save");
        assert!(path.ends_with(".qreplay"));

        let loaded = ReplaySession::load(&path).expect("load");
        assert_eq!(replay, loaded);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn integrity_check_passes_for_matching_hash() {
        let replay = ReplaySession::new("correct-hash");
        assert!(replay.verify_integrity("correct-hash"));
    }

    #[test]
    fn integrity_check_fails_for_mismatched_hash() {
        let replay = ReplaySession::new("original-hash");
        assert!(!replay.verify_integrity("tampered-hash"));
    }

    #[test]
    fn playback_empty_replay_does_not_panic() {
        let replay = ReplaySession::new("hash");
        replay.playback(1);
    }

    #[test]
    fn new_replay_has_no_events() {
        let replay = ReplaySession::new("hash");
        assert!(replay.events.is_empty());
        assert_eq!(replay.hints_used, 0);
        assert_eq!(replay.attempt_count, 0);
        assert_eq!(replay.final_score, 0);
    }

    #[test]
    fn load_nonexistent_file_returns_error() {
        let result = ReplaySession::load("/nonexistent/path.qreplay");
        assert!(result.is_err());
    }

    #[test]
    fn load_corrupted_file_returns_error() {
        let mut f = NamedTempFile::new().expect("temp file");
        f.write_all(b"not valid bincode data").expect("write");
        let result = ReplaySession::load(f.path());
        assert!(result.is_err());
    }
}
