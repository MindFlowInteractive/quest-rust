use crate::errors::AppError;
use crate::player::Player;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const SAVE_VERSION: u32 = 1;
const BINARY_EXT: &str = "bin";
const ARCHIVE_EXT: &str = "json.old";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaveHeader {
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaveEnvelope {
    pub header: SaveHeader,
    pub state: GameState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameState {
    pub player: Player,
}

impl GameState {
    pub fn new(player: Player) -> Self {
        GameState { player }
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), AppError> {
        save(path, self)
    }
}

pub fn save(path: impl AsRef<Path>, state: &GameState) -> Result<(), AppError> {
    let envelope = SaveEnvelope {
        header: SaveHeader {
            version: SAVE_VERSION,
        },
        state: state.clone(),
    };
    let encoded =
        bincode::serialize(&envelope).map_err(|e| AppError::Persistence(e.to_string()))?;
    std::fs::write(path, encoded)?;
    Ok(())
}

pub fn load(path: impl AsRef<Path>) -> Result<SaveEnvelope, AppError> {
    let path = path.as_ref();
    let data = std::fs::read(path)?;

    if data.first() == Some(&b'{') {
        return migrate_from_json(path, &data);
    }

    bincode::deserialize(&data)
        .map_err(|e| AppError::Persistence(format!("corrupted save file: {e}")))
}

pub fn export_as_json(path: impl AsRef<Path>) -> Result<String, AppError> {
    let envelope = load(path)?;
    serde_json::to_string_pretty(&envelope).map_err(|e| AppError::Persistence(e.to_string()))
}

fn migrate_from_json(path: &Path, data: &[u8]) -> Result<SaveEnvelope, AppError> {
    let json_str = std::str::from_utf8(data)
        .map_err(|e| AppError::Persistence(format!("invalid UTF-8 in JSON save: {e}")))?;
    let player: Player = serde_json::from_str(json_str)
        .map_err(|e| AppError::Persistence(format!("invalid JSON save: {e}")))?;
    let envelope = SaveEnvelope {
        header: SaveHeader {
            version: SAVE_VERSION,
        },
        state: GameState { player },
    };
    let binary_path = path.with_extension(BINARY_EXT);
    save(&binary_path, &envelope.state)?;
    let archived_path = path.with_extension(ARCHIVE_EXT);
    std::fs::rename(path, archived_path)?;
    Ok(envelope)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_player() -> Player {
        let mut p = Player::new("test-player");
        p.add_score(100);
        p.advance_puzzle();
        p.add_item("sword");
        p.add_item("shield");
        p
    }

    fn binary_path(dir: &TempDir) -> std::path::PathBuf {
        dir.path().join("save.bin")
    }

    #[test]
    fn roundtrip_binary_save_load() {
        let dir = TempDir::new().unwrap();
        let path = binary_path(&dir);
        let state = GameState::new(test_player());

        save(&path, &state).unwrap();
        let loaded = load(&path).unwrap();

        assert_eq!(loaded.header.version, SAVE_VERSION);
        assert_eq!(loaded.state, state);
    }

    #[test]
    fn json_migration_converts_to_binary_and_archives_old() {
        let dir = TempDir::new().unwrap();
        let json_path = dir.path().join("save.json");
        let player = test_player();
        let json = serde_json::to_string(&player).unwrap();
        std::fs::write(&json_path, &json).unwrap();

        let loaded = load(&json_path).unwrap();

        assert_eq!(loaded.header.version, SAVE_VERSION);
        assert_eq!(loaded.state.player, player);
        assert!(!json_path.exists());
        assert!(json_path.with_extension(ARCHIVE_EXT).exists());
        assert!(json_path.with_extension(BINARY_EXT).exists());
    }

    #[test]
    fn version_mismatch_detected() {
        let dir = TempDir::new().unwrap();
        let path = binary_path(&dir);
        let state = GameState::new(test_player());
        let envelope = SaveEnvelope {
            header: SaveHeader { version: 999 },
            state,
        };
        let encoded = bincode::serialize(&envelope).unwrap();
        std::fs::write(&path, encoded).unwrap();

        let loaded = load(&path).unwrap();
        assert_eq!(loaded.header.version, 999);
    }

    #[test]
    fn corrupted_binary_returns_descriptive_error() {
        let dir = TempDir::new().unwrap();
        let path = binary_path(&dir);
        std::fs::write(&path, [0u8; 16]).unwrap();

        let err = load(&path).unwrap_err();
        match &err {
            AppError::Persistence(msg) => {
                assert!(msg.contains("corrupted save file"), "got: {msg}");
            }
            other => panic!("expected Persistence error, got: {other:?}"),
        }
    }

    #[test]
    fn export_as_json_produces_valid_json() {
        let dir = TempDir::new().unwrap();
        let path = binary_path(&dir);
        let state = GameState::new(test_player());
        save(&path, &state).unwrap();

        let json = export_as_json(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed["header"]["version"].as_u64(),
            Some(SAVE_VERSION as u64)
        );
        assert_eq!(
            parsed["state"]["player"]["id"].as_str(),
            Some("test-player")
        );
    }

    #[test]
    fn corrupted_json_returns_descriptive_error() {
        let dir = TempDir::new().unwrap();
        let json_path = dir.path().join("save.json");
        std::fs::write(&json_path, b"{corrupted").unwrap();

        let err = load(&json_path).unwrap_err();
        match &err {
            AppError::Persistence(msg) => {
                assert!(msg.contains("invalid JSON"), "got: {msg}");
            }
            other => panic!("expected Persistence error, got: {other:?}"),
        }
    }

    #[test]
    fn non_existent_file_returns_io_error() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("does_not_exist.bin");
        let err = load(&path).unwrap_err();
        assert!(matches!(err, AppError::Io(_)));
    }
}
