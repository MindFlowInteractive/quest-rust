pub mod cli;
pub mod engine;
pub mod l10n;
pub mod logic;
pub mod session;
pub mod time;

#[cfg(not(feature = "wasm"))]
pub mod api_client;

#[cfg(not(feature = "wasm"))]
pub mod tui;

pub mod config;
pub mod difficulty;
pub mod errors;
pub mod events;
pub mod generator;
pub mod hints;
pub mod input;
pub mod inventory;
pub mod leaderboard;
pub mod loader;
pub mod nft;
pub mod player;
pub mod plugin;
pub mod puzzle;
pub mod score;
pub mod timer;
