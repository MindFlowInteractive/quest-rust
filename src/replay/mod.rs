//! Replay recording and playback engine.
//!
//! Records every player action during a session as a timestamped event log
//! and supports deterministic playback. Replay files are serialized as
//! compact `.qreplay` binary files using `bincode`.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::time::Duration;

/// All recordable actions that can occur during a session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReplayEvent {
    /// Session started with the given puzzle ID.
    SessionStart { puzzle_id: String },
    /// Player revealed a hint (hint index, 0-based).
    HintReveal { hint_index: u32 },
    /// Player submitted an answer attempt.
    AnswerAttempt { answer: String, correct: bool },
    /// Session ended with final score.
    SessionEnd { final_score: u64 },
}

impl fmt::Display for ReplayEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReplayEvent::SessionStart { puzzle_id } => {
                write!(f, "SESSION_START puzzle={puzzle_id}")
            }
            ReplayEvent::HintReveal { hint_index } => {
                write!(f, "HINT_REVEAL index={hint_index}")
            }
            ReplayEvent::AnswerAttempt { answer, correct } => {
                let status = if *correct { "CORRECT" } else { "WRONG" };
                write!(f, "ANSWER_ATTEMPT answer=\"{answer}\" result={status}")
            }
            ReplayEvent::SessionEnd { final_score } => {
                write!(f, "SESSION_END score={final_score}")
            }
        }
    }
}

/// A single timestamped event in the replay log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimestampedEvent {
    /// Microseconds elapsed since session start.
    pub timestamp_us: u64,
    /// The event that occurred.
    pub event: ReplayEvent,
}

/// Header data stored alongside the event log in a `.qreplay` file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayHeader {
    /// Format version for forward compatibility.
    pub version: u32,
    /// Content hash of the puzzle for integrity cross-checking.
    pub puzzle_content_hash: Option<String>,
}

/// Complete replay data serialized to a `.qreplay` file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayData {
    pub header: ReplayHeader,
    pub events: Vec<TimestampedEvent>,
}

/// Summary statistics computed from a completed replay.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplaySummary {
    pub total_time_ms: u64,
    pub hints_used: u32,
    pub attempt_count: u32,
    pub final_score: u64,
}

/// Records events during a live session.
pub struct ReplayRecorder {
    start: std::time::Instant,
    events: Vec<TimestampedEvent>,
    puzzle_content_hash: Option<String>,
}

impl ReplayRecorder {
    /// Creates a new recorder. Call this when the session begins.
    pub fn new(puzzle_content_hash: Option<String>) -> Self {
        Self {
            start: std::time::Instant::now(),
            events: Vec::new(),
            puzzle_content_hash,
        }
    }

    /// Records an event with the current timestamp relative to session start.
    pub fn record(&mut self, event: ReplayEvent) {
        let elapsed = self.start.elapsed();
        let timestamp_us = elapsed.as_micros() as u64;
        self.events.push(TimestampedEvent {
            timestamp_us,
            event,
        });
    }

    /// Records an event with an explicit microsecond timestamp (for testing).
    pub fn record_at(&mut self, timestamp_us: u64, event: ReplayEvent) {
        self.events.push(TimestampedEvent {
            timestamp_us,
            event,
        });
    }

    /// Returns the number of recorded events.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Builds the final replay data.
    pub fn finish(self) -> ReplayData {
        ReplayData {
            header: ReplayHeader {
                version: 1,
                puzzle_content_hash: self.puzzle_content_hash,
            },
            events: self.events,
        }
    }

    /// Saves the replay to a `.qreplay` binary file.
    pub fn save_to_file(self, path: impl AsRef<Path>) -> io::Result<()> {
        let data = self.finish();
        save_replay(&data, path)
    }
}

/// Handles playback of a recorded replay.
pub struct ReplayPlayer {
    data: ReplayData,
}

impl ReplayPlayer {
    /// Loads a replay from a `.qreplay` binary file.
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let data = load_replay(path)?;
        Ok(Self { data })
    }

    /// Creates a player from existing replay data (for testing).
    pub fn from_data(data: ReplayData) -> Self {
        Self { data }
    }

    /// Returns the puzzle content hash from the replay header.
    pub fn puzzle_content_hash(&self) -> Option<&str> {
        self.data.header.puzzle_content_hash.as_deref()
    }

    /// Checks whether the replay's puzzle hash matches an expected hash.
    /// Returns `true` if they match or if no hash is stored. Returns `false`
    /// on mismatch.
    pub fn verify_integrity(&self, expected_hash: &str) -> bool {
        match &self.data.header.puzzle_content_hash {
            Some(stored) => stored == expected_hash,
            None => true,
        }
    }

    /// Replays all events in order, returning them with adjusted timestamps
    /// based on the playback speed multiplier.
    pub fn playback(&self, speed: u32) -> Vec<(u64, String)> {
        let speed = if speed == 0 { 1 } else { speed };
        self.data
            .events
            .iter()
            .map(|e| {
                let adjusted_us = e.timestamp_us / speed as u64;
                (adjusted_us, e.event.to_string())
            })
            .collect()
    }

    /// Returns the raw events for inspection.
    pub fn events(&self) -> &[TimestampedEvent] {
        &self.data.events
    }

    /// Computes a summary of the replay.
    pub fn summary(&self) -> ReplaySummary {
        let total_time_us = self.data.events.last().map(|e| e.timestamp_us).unwrap_or(0);

        let mut hints_used: u32 = 0;
        let mut attempt_count: u32 = 0;
        let mut final_score: u64 = 0;

        for te in &self.data.events {
            match &te.event {
                ReplayEvent::HintReveal { .. } => hints_used += 1,
                ReplayEvent::AnswerAttempt { .. } => attempt_count += 1,
                ReplayEvent::SessionEnd { final_score: score } => final_score = *score,
                _ => {}
            }
        }

        ReplaySummary {
            total_time_ms: total_time_us / 1000,
            hints_used,
            attempt_count,
            final_score,
        }
    }
}

/// Serializes replay data to a binary file.
pub fn save_replay(data: &ReplayData, path: impl AsRef<Path>) -> io::Result<()> {
    let bytes =
        bincode::serialize(data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, bytes)
}

/// Deserializes replay data from a binary file.
pub fn load_replay(path: impl AsRef<Path>) -> io::Result<ReplayData> {
    let bytes = fs::read(path)?;
    bincode::deserialize(&bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Simulates playback by sleeping between events (real-time or accelerated).
/// Intended for interactive CLI playback.
pub fn playback_with_delays(data: &ReplayData, speed: u32) {
    let speed = if speed == 0 { 1 } else { speed };
    let mut last_us: u64 = 0;

    for te in &data.events {
        let delta_us = te.timestamp_us.saturating_sub(last_us);
        let sleep_us = delta_us / speed as u64;

        if sleep_us > 0 {
            std::thread::sleep(Duration::from_micros(sleep_us));
        }

        let ms = te.timestamp_us / 1000;
        println!("[{ms:>8}ms] {}", te.event);
        last_us = te.timestamp_us;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_event_recording() {
        let mut recorder = ReplayRecorder::new(Some("abc123".to_string()));
        recorder.record_at(
            0,
            ReplayEvent::SessionStart {
                puzzle_id: "puzzle-1".to_string(),
            },
        );
        recorder.record_at(1_000_000, ReplayEvent::HintReveal { hint_index: 0 });
        recorder.record_at(
            2_000_000,
            ReplayEvent::AnswerAttempt {
                answer: "lever".to_string(),
                correct: false,
            },
        );
        recorder.record_at(
            3_500_000,
            ReplayEvent::AnswerAttempt {
                answer: "pull_lever".to_string(),
                correct: true,
            },
        );
        recorder.record_at(4_000_000, ReplayEvent::SessionEnd { final_score: 85 });

        assert_eq!(recorder.event_count(), 5);

        let data = recorder.finish();
        assert_eq!(data.header.version, 1);
        assert_eq!(data.header.puzzle_content_hash, Some("abc123".to_string()));
        assert_eq!(data.events.len(), 5);
        assert_eq!(data.events[0].timestamp_us, 0);
        assert_eq!(data.events[4].timestamp_us, 4_000_000);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut recorder = ReplayRecorder::new(Some("hash-xyz".to_string()));
        recorder.record_at(
            0,
            ReplayEvent::SessionStart {
                puzzle_id: "test-puzzle".to_string(),
            },
        );
        recorder.record_at(500_000, ReplayEvent::HintReveal { hint_index: 0 });
        recorder.record_at(
            1_000_000,
            ReplayEvent::AnswerAttempt {
                answer: "test".to_string(),
                correct: true,
            },
        );
        recorder.record_at(1_500_000, ReplayEvent::SessionEnd { final_score: 100 });

        let original = recorder.finish();

        // Serialize
        let bytes = bincode::serialize(&original).expect("serialize");
        // Deserialize
        let restored: ReplayData = bincode::deserialize(&bytes).expect("deserialize");

        assert_eq!(original, restored);
    }

    #[test]
    fn test_file_save_and_load() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let path = dir.path().join("test.qreplay");

        let mut recorder = ReplayRecorder::new(Some("file-test-hash".to_string()));
        recorder.record_at(
            0,
            ReplayEvent::SessionStart {
                puzzle_id: "file-test".to_string(),
            },
        );
        recorder.record_at(
            1_000_000,
            ReplayEvent::AnswerAttempt {
                answer: "answer".to_string(),
                correct: true,
            },
        );
        recorder.record_at(2_000_000, ReplayEvent::SessionEnd { final_score: 50 });

        let original = recorder.finish();
        save_replay(&original, &path).expect("save");

        let loaded = load_replay(&path).expect("load");
        assert_eq!(original, loaded);
    }

    #[test]
    fn test_deterministic_playback() {
        let mut recorder = ReplayRecorder::new(None);
        recorder.record_at(
            0,
            ReplayEvent::SessionStart {
                puzzle_id: "det-test".to_string(),
            },
        );
        recorder.record_at(1_000_000, ReplayEvent::HintReveal { hint_index: 0 });
        recorder.record_at(2_000_000, ReplayEvent::HintReveal { hint_index: 1 });
        recorder.record_at(
            3_000_000,
            ReplayEvent::AnswerAttempt {
                answer: "wrong".to_string(),
                correct: false,
            },
        );
        recorder.record_at(
            4_000_000,
            ReplayEvent::AnswerAttempt {
                answer: "right".to_string(),
                correct: true,
            },
        );
        recorder.record_at(5_000_000, ReplayEvent::SessionEnd { final_score: 80 });

        let data = recorder.finish();
        let player = ReplayPlayer::from_data(data.clone());

        // Playback at 1x
        let playback_1x = player.playback(1);
        assert_eq!(playback_1x.len(), 6);
        assert_eq!(playback_1x[0].0, 0); // 0us
        assert_eq!(playback_1x[1].0, 1_000_000); // 1s in us

        // Playback at 2x — timestamps halved
        let playback_2x = player.playback(2);
        assert_eq!(playback_2x[1].0, 500_000); // 0.5s in us

        // Playback at 4x — timestamps quartered
        let playback_4x = player.playback(4);
        assert_eq!(playback_4x[1].0, 250_000); // 0.25s in us

        // Event content is identical regardless of speed
        assert_eq!(playback_1x[0].1, playback_2x[0].1);
        assert_eq!(playback_1x[0].1, playback_4x[0].1);

        // Run it again — deterministic, same output
        let player2 = ReplayPlayer::from_data(data);
        let playback_1x_again = player2.playback(1);
        assert_eq!(playback_1x, playback_1x_again);
    }

    #[test]
    fn test_replay_summary() {
        let mut recorder = ReplayRecorder::new(None);
        recorder.record_at(
            0,
            ReplayEvent::SessionStart {
                puzzle_id: "summary-test".to_string(),
            },
        );
        recorder.record_at(1_000_000, ReplayEvent::HintReveal { hint_index: 0 });
        recorder.record_at(2_000_000, ReplayEvent::HintReveal { hint_index: 1 });
        recorder.record_at(
            3_000_000,
            ReplayEvent::AnswerAttempt {
                answer: "wrong".to_string(),
                correct: false,
            },
        );
        recorder.record_at(
            4_000_000,
            ReplayEvent::AnswerAttempt {
                answer: "right".to_string(),
                correct: true,
            },
        );
        recorder.record_at(5_000_000, ReplayEvent::SessionEnd { final_score: 75 });

        let data = recorder.finish();
        let player = ReplayPlayer::from_data(data);
        let summary = player.summary();

        assert_eq!(summary.total_time_ms, 5000); // 5 seconds
        assert_eq!(summary.hints_used, 2);
        assert_eq!(summary.attempt_count, 2);
        assert_eq!(summary.final_score, 75);
    }

    #[test]
    fn test_integrity_check_match() {
        let data = ReplayData {
            header: ReplayHeader {
                version: 1,
                puzzle_content_hash: Some("expected-hash-123".to_string()),
            },
            events: vec![],
        };
        let player = ReplayPlayer::from_data(data);

        assert!(player.verify_integrity("expected-hash-123"));
        assert!(!player.verify_integrity("wrong-hash"));
    }

    #[test]
    fn test_integrity_check_no_hash() {
        let data = ReplayData {
            header: ReplayHeader {
                version: 1,
                puzzle_content_hash: None,
            },
            events: vec![],
        };
        let player = ReplayPlayer::from_data(data);

        // No hash stored — passes any check
        assert!(player.verify_integrity("anything"));
    }

    #[test]
    fn test_integrity_mismatch_detection() {
        let data = ReplayData {
            header: ReplayHeader {
                version: 1,
                puzzle_content_hash: Some("correct-hash".to_string()),
            },
            events: vec![TimestampedEvent {
                timestamp_us: 0,
                event: ReplayEvent::SessionStart {
                    puzzle_id: "integrity-test".to_string(),
                },
            }],
        };
        let player = ReplayPlayer::from_data(data);

        assert!(player.verify_integrity("correct-hash"));
        assert!(!player.verify_integrity("tampered-hash"));
        assert!(!player.verify_integrity(""));
    }

    #[test]
    fn test_empty_replay_summary() {
        let data = ReplayData {
            header: ReplayHeader {
                version: 1,
                puzzle_content_hash: None,
            },
            events: vec![],
        };
        let player = ReplayPlayer::from_data(data);
        let summary = player.summary();

        assert_eq!(summary.total_time_ms, 0);
        assert_eq!(summary.hints_used, 0);
        assert_eq!(summary.attempt_count, 0);
        assert_eq!(summary.final_score, 0);
    }

    #[test]
    fn test_event_display() {
        let start = ReplayEvent::SessionStart {
            puzzle_id: "p1".to_string(),
        };
        assert_eq!(start.to_string(), "SESSION_START puzzle=p1");

        let hint = ReplayEvent::HintReveal { hint_index: 2 };
        assert_eq!(hint.to_string(), "HINT_REVEAL index=2");

        let attempt = ReplayEvent::AnswerAttempt {
            answer: "foo".to_string(),
            correct: true,
        };
        assert_eq!(
            attempt.to_string(),
            "ANSWER_ATTEMPT answer=\"foo\" result=CORRECT"
        );

        let end = ReplayEvent::SessionEnd { final_score: 42 };
        assert_eq!(end.to_string(), "SESSION_END score=42");
    }

    #[test]
    fn test_save_creates_parent_dirs() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let path = dir.path().join("nested").join("deep").join("test.qreplay");

        let data = ReplayData {
            header: ReplayHeader {
                version: 1,
                puzzle_content_hash: None,
            },
            events: vec![],
        };

        save_replay(&data, &path).expect("save with nested dirs");
        assert!(path.exists());

        let loaded = load_replay(&path).expect("load");
        assert_eq!(data, loaded);
    }

    #[test]
    fn test_load_invalid_file_fails() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let path = dir.path().join("bad.qreplay");

        // Write invalid data
        let mut f = fs::File::create(&path).expect("create");
        f.write_all(b"this is not valid bincode").expect("write");

        let result = load_replay(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_nonexistent_file_fails() {
        let result = load_replay("/nonexistent/path/replay.qreplay");
        assert!(result.is_err());
    }

    #[test]
    fn test_recorder_with_live_timestamps() {
        let mut recorder = ReplayRecorder::new(None);
        recorder.record(ReplayEvent::SessionStart {
            puzzle_id: "live".to_string(),
        });
        std::thread::sleep(Duration::from_millis(5));
        recorder.record(ReplayEvent::SessionEnd { final_score: 10 });

        let data = recorder.finish();
        assert_eq!(data.events.len(), 2);
        // Second event should have a larger timestamp than the first
        assert!(data.events[1].timestamp_us > data.events[0].timestamp_us);
    }

    #[test]
    fn test_file_roundtrip_with_all_event_types() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let path = dir.path().join("full.qreplay");

        let mut recorder = ReplayRecorder::new(Some("full-hash".to_string()));
        recorder.record_at(
            0,
            ReplayEvent::SessionStart {
                puzzle_id: "full-test".to_string(),
            },
        );
        recorder.record_at(100_000, ReplayEvent::HintReveal { hint_index: 0 });
        recorder.record_at(200_000, ReplayEvent::HintReveal { hint_index: 1 });
        recorder.record_at(300_000, ReplayEvent::HintReveal { hint_index: 2 });
        recorder.record_at(
            400_000,
            ReplayEvent::AnswerAttempt {
                answer: "wrong1".to_string(),
                correct: false,
            },
        );
        recorder.record_at(
            500_000,
            ReplayEvent::AnswerAttempt {
                answer: "wrong2".to_string(),
                correct: false,
            },
        );
        recorder.record_at(
            600_000,
            ReplayEvent::AnswerAttempt {
                answer: "correct".to_string(),
                correct: true,
            },
        );
        recorder.record_at(700_000, ReplayEvent::SessionEnd { final_score: 42 });

        let original = recorder.finish();
        save_replay(&original, &path).expect("save");
        let loaded = load_replay(&path).expect("load");
        assert_eq!(original, loaded);

        let player = ReplayPlayer::from_data(loaded);
        let summary = player.summary();
        assert_eq!(summary.hints_used, 3);
        assert_eq!(summary.attempt_count, 3);
        assert_eq!(summary.final_score, 42);
        assert_eq!(summary.total_time_ms, 700);
    }
}
