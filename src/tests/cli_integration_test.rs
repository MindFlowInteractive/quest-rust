use std::io::Cursor;

// Bring stubs and module types into local scope context
use smart_contract_game::cli::{CliModule, GameEngine, PuzzleState};

#[test]
fn test_full_successful_puzzle_session_via_cli() {
    // 1. Setup mock game engine data registers
    let puzzle = PuzzleState {
        description: "What blockchain engine uses Soroban smart contracts?".to_string(),
        hint: "Starts with an 'S' and secures digital assets globally.".to_string(),
        current_score: 0,
        is_solved: false,
    };
    
    let mut engine = GameEngine {
        puzzle,
        unlocked_achievements: Vec::new(),
    };

    // 2. Simulate user terminal inputs (incorrect attempt first, then correct solution)
    let simulated_input = "bitcoin\nstellar\n";
    let input_cursor = Cursor::new(simulated_input.as_bytes());
    let mut output_buffer = Vec::new();

    // 3. Instantiate the CLI wrapper and execute the engine loop sequence
    {
        let mut cli = CliModule::new(input_cursor, &mut output_buffer);
        let result = cli.start_game_loop(&mut engine);
        assert!(result.is_ok());
    }

    // 4. Verify system changes inside internal storage engine
    assert!(engine.puzzle.is_solved);
    assert_eq!(engine.puzzle.current_score, 100);
    assert_eq!(engine.unlocked_achievements.len(), 1);
    assert_eq!(engine.unlocked_achievements[0], "Soroban Pioneer");

    // 5. Parse output buffer back to a string to verify correct terminal formatting
    let terminal_output = String::from_utf8(output_buffer).unwrap();
    
    // Assert layout renders accurately
    assert!(terminal_output.contains("PUZZLE SESSION | Score: 0 pts"));
    assert!(terminal_output.contains("Description: What blockchain engine uses Soroban smart contracts?"));
    assert!(terminal_output.contains("Enter your solution > "));
    
    // Assert event notification banner pops accurately
    assert!(terminal_output.contains("ACHIEVEMENT UNLOCKED: [Soroban Pioneer]"));
    assert!(terminal_output.contains("Puzzle Completed Successfully!"));
}