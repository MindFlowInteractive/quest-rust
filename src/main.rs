fn main() {
    use smart_contract_game::player::Player;

    let mut player = Player::new("player-1");
    player.add_score(100);
    player.advance_puzzle();
    player.add_item("compass");

    match player.to_json() {
        Ok(json) => println!("Player state: {json}"),
        Err(e) => eprintln!("Failed to serialize player: {e}"),
    }
}
