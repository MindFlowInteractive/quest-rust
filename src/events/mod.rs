//! Events module.
//!
//! Allows modules to emit and subscribe to game events (puzzle solved, reward granted, etc.)
//! without direct coupling.

use std::collections::VecDeque;
use std::sync::Arc;

/// Core events that occur within the game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEvent {
    /// Fired when a puzzle is successfully solved.
    PuzzleSolved {
        /// Unique identifier for the puzzle.
        puzzle_id: String,
        /// Score awarded for solving the puzzle.
        score: u32,
    },
    /// Fired when a reward is granted to the player.
    RewardGranted {
        /// ID of the reward item.
        item_id: String,
        /// Quantity of the item granted.
        quantity: u32,
    },
    /// Fired when an achievement is unlocked.
    AchievementUnlocked {
        /// Name of the achievement.
        name: String,
    },
}

/// A subscriber callback function.
pub type SubscriberFn = Arc<dyn Fn(&GameEvent) + Send + Sync + 'static>;

/// Event dispatching system that processes events in FIFO order.
#[derive(Default)]
pub struct EventSystem {
    subscribers: Vec<SubscriberFn>,
    queue: VecDeque<GameEvent>,
}

impl EventSystem {
    /// Creates a new event system.
    pub fn new() -> Self {
        EventSystem {
            subscribers: Vec::new(),
            queue: VecDeque::new(),
        }
    }

    /// Subscribes a callback to all emitted events.
    pub fn subscribe<F>(&mut self, callback: F)
    where
        F: Fn(&GameEvent) + Send + Sync + 'static,
    {
        self.subscribers.push(Arc::new(callback));
    }

    /// Emits a new event, placing it at the back of the FIFO queue.
    pub fn emit(&mut self, event: GameEvent) {
        self.queue.push_back(event);
    }

    /// Processes all queued events in FIFO order (front of queue first),
    /// dispatching each event to all subscribers.
    pub fn process_events(&mut self) {
        while let Some(event) = self.queue.pop_front() {
            for subscriber in &self.subscribers {
                subscriber(&event);
            }
        }
    }

    /// Returns the number of events currently in the queue.
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn test_emit_and_subscribe() {
        let mut system = EventSystem::new();
        
        let received = Arc::new(Mutex::new(Vec::new()));
        let received_clone = Arc::clone(&received);

        // Subscribe to events
        system.subscribe(move |event| {
            received_clone.lock().unwrap().push(event.clone());
        });

        // Emit an event
        let event = GameEvent::PuzzleSolved {
            puzzle_id: "soroban-1".to_string(),
            score: 100,
        };
        system.emit(event.clone());
        assert_eq!(system.queue_len(), 1);

        // Process events
        system.process_events();
        assert_eq!(system.queue_len(), 0);

        // Verify callback received the event
        let received_events = received.lock().unwrap();
        assert_eq!(received_events.len(), 1);
        assert_eq!(received_events[0], event);
    }

    #[test]
    fn test_queue_ordering_fifo() {
        let mut system = EventSystem::new();

        let received_ids = Arc::new(Mutex::new(Vec::new()));
        let received_ids_clone = Arc::clone(&received_ids);

        system.subscribe(move |event| {
            match event {
                GameEvent::PuzzleSolved { puzzle_id, .. } => {
                    received_ids_clone.lock().unwrap().push(puzzle_id.clone());
                }
                _ => {}
            }
        });

        // Emit events in a specific order: A -> B -> C
        system.emit(GameEvent::PuzzleSolved {
            puzzle_id: "A".to_string(),
            score: 10,
        });
        system.emit(GameEvent::PuzzleSolved {
            puzzle_id: "B".to_string(),
            score: 20,
        });
        system.emit(GameEvent::PuzzleSolved {
            puzzle_id: "C".to_string(),
            score: 30,
        });

        assert_eq!(system.queue_len(), 3);

        // Process them
        system.process_events();

        // Verify FIFO order: A, then B, then C
        let ids = received_ids.lock().unwrap();
        assert_eq!(*ids, vec!["A".to_string(), "B".to_string(), "C".to_string()]);
    }

    #[test]
    fn test_multiple_subscribers() {
        let mut system = EventSystem::new();

        let count1 = Arc::new(Mutex::new(0));
        let count1_clone = Arc::clone(&count1);
        system.subscribe(move |_| {
            *count1_clone.lock().unwrap() += 1;
        });

        let count2 = Arc::new(Mutex::new(0));
        let count2_clone = Arc::clone(&count2);
        system.subscribe(move |_| {
            *count2_clone.lock().unwrap() += 1;
        });

        system.emit(GameEvent::AchievementUnlocked {
            name: "First Steps".to_string(),
        });
        system.emit(GameEvent::RewardGranted {
            item_id: "gold".to_string(),
            quantity: 50,
        });

        system.process_events();

        assert_eq!(*count1.lock().unwrap(), 2);
        assert_eq!(*count2.lock().unwrap(), 2);
    }
}
