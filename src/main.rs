pub mod engine;

use std::time::Duration;

fn main() {
    use smart_contract_game::player::Player;

    // Initialize and run the core engine for a short duration to ensure clean startup/shutdown.
    let engine = engine::Engine::new(Duration::from_millis(16));
    engine.init();
    engine.run_for(Duration::from_millis(100));
    engine.shutdown();

    let mut player = Player::new("player-1");
    player.add_score(100);
    player.advance_puzzle();
    player.add_item("compass");

    match player.to_json() {
        Ok(json) => println!("Player state: {json}"),
        Err(e) => eprintln!("Failed to serialize player: {e}"),
    }
}
