use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, PartialEq, Clone)]
pub struct Achievement {
    pub player_id: String,
    pub milestone_type: String,
    pub timestamp: u64,
}

pub struct NftModule {
    minted_achievements: HashMap<(String, String), Achievement>,
}

impl NftModule {
    pub fn new() -> Self {
        Self {
            minted_achievements: HashMap::new(),
        }
    }

    pub fn mint(&mut self, player_id: String, milestone_type: String) -> Option<Achievement> {
        let key = (player_id.clone(), milestone_type.clone());
        if self.minted_achievements.contains_key(&key) {
            None
        } else {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();

            let achievement = Achievement {
                player_id,
                milestone_type,
                timestamp,
            };
            self.minted_achievements.insert(key, achievement.clone());
            Some(achievement)
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

        let achievement = nft_module.mint(player_id.clone(), milestone.clone());

        assert!(achievement.is_some());
        let achievement = achievement.unwrap();
        assert_eq!(achievement.player_id, player_id);
        assert_eq!(achievement.milestone_type, milestone);
        assert!(achievement.timestamp > 0);
    }

    #[test]
    fn test_mint_idempotency() {
        let mut nft_module = NftModule::new();
        let player_id = "player1".to_string();
        let milestone = "level10".to_string();

        let achievement1 = nft_module.mint(player_id.clone(), milestone.clone());
        assert!(achievement1.is_some());

        let achievement2 = nft_module.mint(player_id.clone(), milestone.clone());
        assert!(achievement2.is_none());
    }
}