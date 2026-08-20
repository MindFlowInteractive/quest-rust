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

use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::Mutex;

#[derive(Clone, Debug, PartialEq)]
pub enum GameEvent {
    PlayerMoved { x: i32, y: i32 },
    ItemMinted { item_id: String },
}

#[async_trait]
pub trait EventListener: Send + Sync {
    async fn handle(&self, event: GameEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub struct EventBus {
    listeners: Vec<Arc<dyn EventListener>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self { listeners: Vec::new() }
    }

    pub fn register(&mut self, listener: Arc<dyn EventListener>) {
        self.listeners.push(listener);
    }

    pub async fn dispatch(&self, event: GameEvent) {
        for listener in &self.listeners {
            let listener_clone = Arc::clone(listener);
            let event_clone = event.clone();

            // Spawn each listener onto the Tokio runtime concurrently
            tokio::spawn(async move {
                if let Err(err) = listener_clone.handle(event_clone).await {
                    eprintln!("Error handling event asynchronously: {}", err);
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::time::{sleep, Duration};

    struct CountingListener {
        counter: Arc<AtomicUsize>,
        delay_ms: u64,
    }

    #[async_trait]
    impl EventListener for CountingListener {
        async fn handle(&self, _event: GameEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            if self.delay_ms > 0 {
                sleep(Duration::from_millis(self.delay_ms)).await;
            }
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_concurrent_event_dispatch() {
        let counter = Arc::new(AtomicUsize::new(0));
        
        let listener_one = Arc::new(CountingListener {
            counter: Arc::clone(&counter),
            delay_ms: 10,
        });
        
        let listener_two = Arc::new(CountingListener {
            counter: Arc::clone(&counter),
            delay_ms: 5,
        });

        let mut bus = EventBus::new();
        bus.register(listener_one);
        bus.register(listener_two);

        // Dispatch event asynchronously to all registered listeners
        bus.dispatch(GameEvent::ItemMinted { item_id: "nft_123".into() }).await;

        // Allow background spawned tasks time to complete execution
        sleep(Duration::from_millis(30)).await;

        // Verify that both listeners successfully handled the event concurrently
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }
}