use tokio::fs;
use std::io;
use std::path::Path;

pub async fn ensure_storage_dir(dir_path: &str) -> Result<(), io::Error> {
    let path = Path::new(dir_path);
    if !path.exists() {
        fs::create_dir_all(path).await?;
    }
    Ok(())
}

pub async fn save_game_state(file_path: &str, contents: &[u8]) -> Result<(), io::Error> {
    if let Some(parent) = Path::new(file_path).parent {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent).await?;
        }
    }
    fs::write(file_path, contents).await?;
    Ok(())
}

pub async fn load_game_state(file_path: &str) -> Result<Vec<u8>, io::Error> {
    let bytes = fs::read(file_path).await?;
    Ok(bytes)
}

pub async fn remove_game_state(file_path: &str) -> Result<(), io::Error> {
    fs::remove_file(file_path).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_async_save_and_load_state() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("save_state.dat");
        let path_str = file_path.to_str().unwrap();

        let payload = b"{\"player\": \"hero\", \"score\": 5000}";

        // Test asynchronous write
        let save_result = save_game_state(path_str, payload).await;
        assert!(save_result.is_ok(), "Failed to save game state asynchronously");

        // Test asynchronous read
        let loaded_data = load_game_state(path_str).await;
        assert!(loaded_data.is_ok(), "Failed to load game state asynchronously");
        assert_eq!(loaded_data.unwrap(), payload);

        // Cleanup
        let remove_result = remove_game_state(path_str).await;
        assert!(remove_result.is_ok());
    }

    #[tokio::test]
    async fn test_async_load_non_existent_file() {
        let result = load_game_state("non_existent_save_file_9999.dat").await;
        assert!(result.is_err(), "Expected an error when reading a non-existent file");
    }
}