use crate::errors::AppError;
use crate::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Achievement {
    pub player_id: String,
    pub milestone_type: String,
    pub timestamp: u64,
}

pub struct NftModule {
    minted_achievements: HashMap<(String, String), Achievement>,
}

impl Default for NftModule {
    fn default() -> Self {
        Self::new()
    }
}

impl NftModule {
    pub fn new() -> Self {
        Self {
            minted_achievements: HashMap::new(),
        }
    }

    pub fn mint(
        &mut self,
        player_id: String,
        milestone_type: String,
    ) -> Result<Achievement, AppError> {
        let key = (player_id.clone(), milestone_type.clone());
        if let std::collections::hash_map::Entry::Vacant(e) = self.minted_achievements.entry(key) {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();

            let achievement = Achievement {
                player_id,
                milestone_type,
                timestamp,
            };
            e.insert(achievement.clone());
            Ok(achievement)
        } else {
            Err(AppError::NftAlreadyMinted {
                player_id,
                milestone_type,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mint_achievement() {
        let mut nft_module = NftModule::new();
        let player_id = "player1".to_string();
        let milestone = "level10".to_string();

        let result = nft_module.mint(player_id.clone(), milestone.clone());

        assert!(result.is_ok());
        let achievement = result.unwrap();
        assert_eq!(achievement.player_id, player_id);
        assert_eq!(achievement.milestone_type, milestone);
        assert!(achievement.timestamp > 0);
    }

    #[test]
    fn test_mint_idempotency() {
        let mut nft_module = NftModule::new();
        let player_id = "player1".to_string();
        let milestone = "level10".to_string();

        let result1 = nft_module.mint(player_id.clone(), milestone.clone());
        assert!(result1.is_ok());

        let result2 = nft_module.mint(player_id.clone(), milestone.clone());
        assert!(result2.is_err());
        assert!(matches!(
            result2.unwrap_err(),
            AppError::NftAlreadyMinted { .. }
        ));
    }
}
