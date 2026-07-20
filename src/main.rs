pub mod engine;

use std::env;
use std::time::Duration;

fn main() {
    use smart_contract_game::persistence::{GameState, export_as_json};
    use smart_contract_game::player::Player;

    let args: Vec<String> = env::args().collect();

    if let Some(pos) = args.iter().position(|a| a == "--export-save") {
        let path = args.get(pos + 1).expect("usage: --export-save <path>");
        match export_as_json(path) {
            Ok(json) => println!("{json}"),
            Err(e) => eprintln!("export failed: {e}"),
        }
        return;
    }

    // Initialize and run the core engine for a short duration to ensure clean startup/shutdown.
    let engine = engine::Engine::new(Duration::from_millis(16));
    engine.init();
    engine.run_for(Duration::from_millis(100));
    engine.shutdown();

    let mut player = Player::new("player-1");
    player.add_score(100);
    player.advance_puzzle();
    player.add_item("compass");

    // Demonstrate binary save/load
    match GameState::new(player.clone()).save("save_data/player.bin") {
        Ok(_) => eprintln!("Save written."),
        Err(e) => eprintln!("Save failed: {e}"),
    }
}
