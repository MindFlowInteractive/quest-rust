//! Config module.
//!
//! Loads runtime configuration from a `config.toml` file at startup.
//! If the file is absent or a field is missing, hardcoded defaults are used.
//! The resulting [`Config`] is a plain typed struct accessible anywhere in the
//! application.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// ── Defaults ────────────────────────────────────────────────────────────────

pub const DEFAULT_SAVE_PATH: &str = "save_data";
pub const DEFAULT_LOG_PATH: &str = "logs/game.log";
pub const DEFAULT_DIFFICULTY: &str = "medium";
pub const DEFAULT_MAX_HINTS: u8 = 3;
pub const DEFAULT_SCORE_MULTIPLIER: f64 = 1.0;

/// Runtime configuration for the game.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Directory where player save files are stored.
    pub save_path: String,
    /// Path to the log file.
    pub log_path: String,
    /// Game difficulty: `"easy"`, `"medium"`, or `"hard"`.
    pub difficulty: String,
    /// Maximum number of hints a player may reveal per puzzle.
    pub max_hints: u8,
    /// Multiplier applied to the base score for each solved puzzle.
    pub score_multiplier: f64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            save_path: DEFAULT_SAVE_PATH.to_string(),
            log_path: DEFAULT_LOG_PATH.to_string(),
            difficulty: DEFAULT_DIFFICULTY.to_string(),
            max_hints: DEFAULT_MAX_HINTS,
            score_multiplier: DEFAULT_SCORE_MULTIPLIER,
        }
    }
}

impl Config {
    /// Loads configuration from `path`.
    ///
    /// - Returns the parsed config on success.
    /// - Returns [`Config::default`] if the file does not exist.
    /// - Returns an error for IO errors or malformed TOML.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Config::default());
        }
        let raw = fs::read_to_string(path).map_err(ConfigError::Io)?;
        let partial: PartialConfig =
            toml::from_str(&raw).map_err(|e| ConfigError::Parse(e.to_string()))?;
        Ok(partial.into_config())
    }
}

// ── Internal deserialization helper ─────────────────────────────────────────

/// Every field is `Option<T>` so missing keys silently fall back to defaults.
#[derive(Debug, Deserialize)]
struct PartialConfig {
    save_path: Option<String>,
    log_path: Option<String>,
    difficulty: Option<String>,
    max_hints: Option<u8>,
    score_multiplier: Option<f64>,
}

impl PartialConfig {
    fn into_config(self) -> Config {
        let d = Config::default();
        Config {
            save_path: self.save_path.unwrap_or(d.save_path),
            log_path: self.log_path.unwrap_or(d.log_path),
            difficulty: self.difficulty.unwrap_or(d.difficulty),
            max_hints: self.max_hints.unwrap_or(d.max_hints),
            score_multiplier: self.score_multiplier.unwrap_or(d.score_multiplier),
        }
    }
}

// ── Error type ───────────────────────────────────────────────────────────────

/// Errors that can occur while loading configuration.
#[derive(Debug)]
pub enum ConfigError {
    /// An underlying IO error (e.g. permission denied).
    Io(std::io::Error),
    /// The TOML content could not be parsed.
    Parse(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "config IO error: {e}"),
            ConfigError::Parse(msg) => write!(f, "config parse error: {msg}"),
        }
    }
}

impl std::error::Error for ConfigError {}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_toml(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().expect("temp file");
        f.write_all(content.as_bytes()).expect("write");
        f
    }

    #[test]
    fn defaults_when_file_missing() {
        let cfg = Config::load("/nonexistent/path/config.toml").expect("load");
        assert_eq!(cfg, Config::default());
    }

    #[test]
    fn loads_full_config_from_toml() {
        let file = write_toml(
            r#"
save_path       = "custom_saves"
log_path        = "custom/log.log"
difficulty      = "hard"
max_hints       = 1
score_multiplier = 2.5
"#,
        );

        let cfg = Config::load(file.path()).expect("load");
        assert_eq!(cfg.save_path, "custom_saves");
        assert_eq!(cfg.log_path, "custom/log.log");
        assert_eq!(cfg.difficulty, "hard");
        assert_eq!(cfg.max_hints, 1);
        assert!((cfg.score_multiplier - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn partial_config_falls_back_to_defaults() {
        let file = write_toml(r#"difficulty = "easy""#);

        let cfg = Config::load(file.path()).expect("load");
        let d = Config::default();

        assert_eq!(cfg.difficulty, "easy");
        assert_eq!(cfg.save_path, d.save_path);
        assert_eq!(cfg.log_path, d.log_path);
        assert_eq!(cfg.max_hints, d.max_hints);
        assert!((cfg.score_multiplier - d.score_multiplier).abs() < f64::EPSILON);
    }

    #[test]
    fn empty_toml_returns_all_defaults() {
        let file = write_toml("");
        let cfg = Config::load(file.path()).expect("load");
        assert_eq!(cfg, Config::default());
    }

    #[test]
    fn invalid_toml_returns_parse_error() {
        let file = write_toml("not = [ valid toml");
        let err = Config::load(file.path()).expect_err("should fail");
        assert!(matches!(err, ConfigError::Parse(_)));
    }

    #[test]
    fn default_values_are_correct() {
        let cfg = Config::default();
        assert_eq!(cfg.save_path, DEFAULT_SAVE_PATH);
        assert_eq!(cfg.log_path, DEFAULT_LOG_PATH);
        assert_eq!(cfg.difficulty, DEFAULT_DIFFICULTY);
        assert_eq!(cfg.max_hints, DEFAULT_MAX_HINTS);
        assert!((cfg.score_multiplier - DEFAULT_SCORE_MULTIPLIER).abs() < f64::EPSILON);
    }
}
