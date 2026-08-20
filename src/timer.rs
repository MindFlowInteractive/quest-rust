use tokio::time::{interval, sleep, Duration};
use std::error::Error;

/// Runs a non-blocking game loop ticker at a specified frame rate interval.
pub async fn run_game_loop(tick_count: usize) -> Result<(), Box<dyn Error>> {
    let mut ticker = interval(Duration::from_millis(16)); // ~60 FPS tick rate

    for _ in 0..tick_count {
        ticker.tick().await;
        // Game tick logic execution point
    }

    Ok(())
}

/// Asynchronously pauses execution for a given duration without blocking threads.
pub async fn async_delay(duration: Duration) {
    sleep(duration).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    #[tokio::test]
    async fn test_async_timer_interval_ticks() {
        let mut ticker = interval(Duration::from_millis(10));
        
        // First tick completes immediately
        let first_tick = ticker.tick().await;
        let start = Instant::now();

        // Await second tick
        let _second_tick = ticker.tick().await;
        let elapsed = start.elapsed();

        // Verify that the interval respected the duration (with tolerance for test overhead)
        assert!(
            elapsed >= Duration::from_millis(8),
            "Expected interval to take at least 8ms, took {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_async_sleep_delay() {
        let delay_duration = Duration::from_millis(20);
        let start = Instant::now();

        async_delay(delay_duration).await;

        let elapsed = start.elapsed();
        assert!(
            elapsed >= delay_duration,
            "Expected delay of at least 20ms, elapsed {:?}",
            elapsed
        );
    }
}