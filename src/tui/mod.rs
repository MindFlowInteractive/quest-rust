//! Terminal UI layer built on `ratatui` and `crossterm`.
//!
//! Provides a rich terminal interface with bordered panels, colour-coded
//! difficulty indicators, progress bars, and keyboard shortcut display.
//! The existing CLI module is preserved as a `--no-tui` fallback.

use crate::difficulty::Difficulty;
use crate::puzzle::PuzzleState;
use ratatui::style::Color;

/// Game status for colour coding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameStatus {
    InProgress,
    Solved,
    Failed,
}

impl GameStatus {
    /// Returns the colour associated with this status.
    pub fn color(&self) -> Color {
        match self {
            GameStatus::InProgress => Color::Yellow,
            GameStatus::Solved => Color::Green,
            GameStatus::Failed => Color::Red,
        }
    }

    /// Derive game status from a puzzle state.
    pub fn from_puzzle_state(state: &PuzzleState) -> Self {
        match state {
            PuzzleState::Unsolved | PuzzleState::InProgress => GameStatus::InProgress,
            PuzzleState::Solved => GameStatus::Solved,
        }
    }
}

/// Data extracted from a puzzle for the TUI view.
#[derive(Debug, Clone)]
pub struct PuzzleViewData {
    pub title: String,
    pub description: String,
    pub difficulty: String,
    pub remaining_hints: u32,
    pub total_hints: u32,
    pub status: GameStatus,
}

impl PuzzleViewData {
    /// Creates view data from puzzle components.
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        difficulty: &Difficulty,
        remaining_hints: u32,
        total_hints: u32,
        status: GameStatus,
    ) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            difficulty: difficulty_label(difficulty),
            remaining_hints,
            total_hints,
            status,
        }
    }
}

/// State for the countdown timer progress bar.
#[derive(Debug, Clone)]
pub struct TimerState {
    pub total_seconds: u64,
    pub elapsed_seconds: u64,
}

impl TimerState {
    pub fn new(total_seconds: u64) -> Self {
        Self {
            total_seconds,
            elapsed_seconds: 0,
        }
    }

    /// Returns remaining seconds.
    pub fn remaining(&self) -> u64 {
        self.total_seconds.saturating_sub(self.elapsed_seconds)
    }

    /// Returns progress as a ratio (0.0 to 1.0) of time *used*.
    pub fn progress_ratio(&self) -> f64 {
        if self.total_seconds == 0 {
            return 1.0;
        }
        (self.elapsed_seconds as f64 / self.total_seconds as f64).min(1.0)
    }

    /// Returns true if the timer has expired.
    pub fn is_expired(&self) -> bool {
        self.elapsed_seconds >= self.total_seconds
    }

    /// Advance the timer by one second.
    pub fn tick(&mut self) {
        if self.elapsed_seconds < self.total_seconds {
            self.elapsed_seconds += 1;
        }
    }
}

/// Score display state.
#[derive(Debug, Clone)]
pub struct ScoreState {
    pub current_score: u64,
    pub status: GameStatus,
}

impl ScoreState {
    pub fn new(current_score: u64, status: GameStatus) -> Self {
        Self {
            current_score,
            status,
        }
    }

    /// Returns the colour for the score based on game status.
    pub fn color(&self) -> Color {
        self.status.color()
    }
}

/// Keyboard shortcut entry for the footer bar.
#[derive(Debug, Clone)]
pub struct Shortcut {
    pub key: String,
    pub action: String,
}

/// Returns the default keyboard shortcuts displayed in the footer.
pub fn default_shortcuts() -> Vec<Shortcut> {
    vec![
        Shortcut {
            key: "h".to_string(),
            action: "Hint".to_string(),
        },
        Shortcut {
            key: "Enter".to_string(),
            action: "Submit".to_string(),
        },
        Shortcut {
            key: "q".to_string(),
            action: "Quit".to_string(),
        },
        Shortcut {
            key: "?".to_string(),
            action: "Help".to_string(),
        },
    ]
}

/// Formats keyboard shortcuts as a single display string.
pub fn format_shortcuts(shortcuts: &[Shortcut]) -> String {
    shortcuts
        .iter()
        .map(|s| format!("[{}] {}", s.key, s.action))
        .collect::<Vec<_>>()
        .join("  |  ")
}

/// Returns a human-readable difficulty label.
pub fn difficulty_label(difficulty: &Difficulty) -> String {
    match difficulty {
        Difficulty::Easy => "Easy".to_string(),
        Difficulty::Medium => "Medium".to_string(),
        Difficulty::Hard => "Hard".to_string(),
    }
}

/// Returns the colour for a difficulty badge.
pub fn difficulty_color(difficulty: &Difficulty) -> Color {
    match difficulty {
        Difficulty::Easy => Color::Green,
        Difficulty::Medium => Color::Yellow,
        Difficulty::Hard => Color::Red,
    }
}

/// Full application state for the TUI.
#[derive(Debug, Clone)]
pub struct TuiApp {
    pub puzzle: Option<PuzzleViewData>,
    pub timer: TimerState,
    pub score: ScoreState,
    pub status: GameStatus,
}

impl TuiApp {
    /// Creates a new TUI application state.
    pub fn new(total_time_seconds: u64) -> Self {
        Self {
            puzzle: None,
            timer: TimerState::new(total_time_seconds),
            score: ScoreState::new(0, GameStatus::InProgress),
            status: GameStatus::InProgress,
        }
    }

    /// Sets the active puzzle for display.
    pub fn set_puzzle(&mut self, data: PuzzleViewData) {
        self.status = data.status;
        self.score.status = data.status;
        self.puzzle = Some(data);
    }

    /// Updates the score.
    pub fn update_score(&mut self, score: u64) {
        self.score.current_score = score;
    }

    /// Marks the game as solved.
    pub fn mark_solved(&mut self) {
        self.status = GameStatus::Solved;
        self.score.status = GameStatus::Solved;
        if let Some(ref mut p) = self.puzzle {
            p.status = GameStatus::Solved;
        }
    }

    /// Marks the game as failed.
    pub fn mark_failed(&mut self) {
        self.status = GameStatus::Failed;
        self.score.status = GameStatus::Failed;
        if let Some(ref mut p) = self.puzzle {
            p.status = GameStatus::Failed;
        }
    }

    /// Advances the timer by one second.
    pub fn tick(&mut self) {
        self.timer.tick();
        if self.timer.is_expired() && self.status == GameStatus::InProgress {
            self.mark_failed();
        }
    }
}

/// Entry point for the TUI rendering mode.
/// This initializes the terminal, runs the main render loop, and restores
/// the terminal on exit.
pub fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    use crossterm::{
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    };
    use ratatui::Terminal;
    use ratatui::backend::CrosstermBackend;

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Draw a single demo frame to verify the TUI works
    terminal.draw(|frame| {
        use ratatui::layout::{Constraint, Direction, Layout};
        use ratatui::style::{Modifier, Style};
        use ratatui::widgets::{Block, Borders, Gauge, Paragraph};

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // Puzzle info
                Constraint::Length(3), // Timer
                Constraint::Length(3), // Score
                Constraint::Min(1),    // Content area
                Constraint::Length(1), // Footer
            ])
            .split(frame.area());

        // Puzzle panel
        let puzzle_block = Block::default()
            .title(" Puzzle ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let puzzle_text =
            Paragraph::new("Welcome to Quest! Use --no-tui for headless mode.").block(puzzle_block);
        frame.render_widget(puzzle_text, chunks[0]);

        // Timer bar
        let timer_block = Block::default()
            .title(" Timer ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        let timer_gauge = Gauge::default()
            .block(timer_block)
            .gauge_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .ratio(0.0)
            .label("Ready");
        frame.render_widget(timer_gauge, chunks[1]);

        // Score display
        let score_block = Block::default()
            .title(" Score ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        let score_text = Paragraph::new("Score: 0").block(score_block);
        frame.render_widget(score_text, chunks[2]);

        // Footer with shortcuts
        let shortcuts = format_shortcuts(&default_shortcuts());
        let footer = Paragraph::new(shortcuts).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(footer, chunks[4]);
    })?;

    // Wait briefly then restore terminal
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_status_colors() {
        assert_eq!(GameStatus::InProgress.color(), Color::Yellow);
        assert_eq!(GameStatus::Solved.color(), Color::Green);
        assert_eq!(GameStatus::Failed.color(), Color::Red);
    }

    #[test]
    fn test_game_status_from_puzzle_state() {
        assert_eq!(
            GameStatus::from_puzzle_state(&PuzzleState::Unsolved),
            GameStatus::InProgress
        );
        assert_eq!(
            GameStatus::from_puzzle_state(&PuzzleState::InProgress),
            GameStatus::InProgress
        );
        assert_eq!(
            GameStatus::from_puzzle_state(&PuzzleState::Solved),
            GameStatus::Solved
        );
    }

    #[test]
    fn test_timer_state_new() {
        let timer = TimerState::new(60);
        assert_eq!(timer.total_seconds, 60);
        assert_eq!(timer.elapsed_seconds, 0);
        assert_eq!(timer.remaining(), 60);
        assert!(!timer.is_expired());
    }

    #[test]
    fn test_timer_progress_ratio() {
        let mut timer = TimerState::new(100);
        assert!((timer.progress_ratio() - 0.0).abs() < f64::EPSILON);

        timer.elapsed_seconds = 50;
        assert!((timer.progress_ratio() - 0.5).abs() < f64::EPSILON);

        timer.elapsed_seconds = 100;
        assert!((timer.progress_ratio() - 1.0).abs() < f64::EPSILON);

        // Over-elapsed should cap at 1.0
        timer.elapsed_seconds = 150;
        assert!((timer.progress_ratio() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_timer_zero_total() {
        let timer = TimerState::new(0);
        assert!((timer.progress_ratio() - 1.0).abs() < f64::EPSILON);
        assert!(timer.is_expired());
        assert_eq!(timer.remaining(), 0);
    }

    #[test]
    fn test_timer_tick() {
        let mut timer = TimerState::new(3);
        assert_eq!(timer.remaining(), 3);

        timer.tick();
        assert_eq!(timer.elapsed_seconds, 1);
        assert_eq!(timer.remaining(), 2);
        assert!(!timer.is_expired());

        timer.tick();
        timer.tick();
        assert_eq!(timer.elapsed_seconds, 3);
        assert_eq!(timer.remaining(), 0);
        assert!(timer.is_expired());

        // Tick past expiry should not go beyond total
        timer.tick();
        assert_eq!(timer.elapsed_seconds, 3);
    }

    #[test]
    fn test_score_state() {
        let score = ScoreState::new(100, GameStatus::InProgress);
        assert_eq!(score.current_score, 100);
        assert_eq!(score.color(), Color::Yellow);

        let score = ScoreState::new(200, GameStatus::Solved);
        assert_eq!(score.color(), Color::Green);

        let score = ScoreState::new(50, GameStatus::Failed);
        assert_eq!(score.color(), Color::Red);
    }

    #[test]
    fn test_difficulty_label() {
        assert_eq!(difficulty_label(&Difficulty::Easy), "Easy");
        assert_eq!(difficulty_label(&Difficulty::Medium), "Medium");
        assert_eq!(difficulty_label(&Difficulty::Hard), "Hard");
    }

    #[test]
    fn test_difficulty_color() {
        assert_eq!(difficulty_color(&Difficulty::Easy), Color::Green);
        assert_eq!(difficulty_color(&Difficulty::Medium), Color::Yellow);
        assert_eq!(difficulty_color(&Difficulty::Hard), Color::Red);
    }

    #[test]
    fn test_default_shortcuts() {
        let shortcuts = default_shortcuts();
        assert_eq!(shortcuts.len(), 4);
        assert_eq!(shortcuts[0].key, "h");
        assert_eq!(shortcuts[0].action, "Hint");
        assert_eq!(shortcuts[2].key, "q");
        assert_eq!(shortcuts[2].action, "Quit");
    }

    #[test]
    fn test_format_shortcuts() {
        let shortcuts = vec![
            Shortcut {
                key: "a".to_string(),
                action: "Action1".to_string(),
            },
            Shortcut {
                key: "b".to_string(),
                action: "Action2".to_string(),
            },
        ];
        assert_eq!(format_shortcuts(&shortcuts), "[a] Action1  |  [b] Action2");
    }

    #[test]
    fn test_format_shortcuts_empty() {
        let shortcuts: Vec<Shortcut> = vec![];
        assert_eq!(format_shortcuts(&shortcuts), "");
    }

    #[test]
    fn test_puzzle_view_data() {
        let data = PuzzleViewData::new(
            "Test Puzzle",
            "Solve the riddle",
            &Difficulty::Hard,
            2,
            3,
            GameStatus::InProgress,
        );
        assert_eq!(data.title, "Test Puzzle");
        assert_eq!(data.description, "Solve the riddle");
        assert_eq!(data.difficulty, "Hard");
        assert_eq!(data.remaining_hints, 2);
        assert_eq!(data.total_hints, 3);
        assert_eq!(data.status, GameStatus::InProgress);
    }

    #[test]
    fn test_tui_app_new() {
        let app = TuiApp::new(120);
        assert!(app.puzzle.is_none());
        assert_eq!(app.timer.total_seconds, 120);
        assert_eq!(app.score.current_score, 0);
        assert_eq!(app.status, GameStatus::InProgress);
    }

    #[test]
    fn test_tui_app_set_puzzle() {
        let mut app = TuiApp::new(60);
        let data = PuzzleViewData::new(
            "Puzzle 1",
            "Description",
            &Difficulty::Easy,
            3,
            3,
            GameStatus::InProgress,
        );
        app.set_puzzle(data);

        assert!(app.puzzle.is_some());
        assert_eq!(app.puzzle.as_ref().unwrap().title, "Puzzle 1");
    }

    #[test]
    fn test_tui_app_update_score() {
        let mut app = TuiApp::new(60);
        app.update_score(500);
        assert_eq!(app.score.current_score, 500);
    }

    #[test]
    fn test_tui_app_mark_solved() {
        let mut app = TuiApp::new(60);
        let data = PuzzleViewData::new("P", "D", &Difficulty::Easy, 1, 3, GameStatus::InProgress);
        app.set_puzzle(data);
        app.mark_solved();

        assert_eq!(app.status, GameStatus::Solved);
        assert_eq!(app.score.status, GameStatus::Solved);
        assert_eq!(app.puzzle.as_ref().unwrap().status, GameStatus::Solved);
    }

    #[test]
    fn test_tui_app_mark_failed() {
        let mut app = TuiApp::new(60);
        let data = PuzzleViewData::new("P", "D", &Difficulty::Medium, 0, 3, GameStatus::InProgress);
        app.set_puzzle(data);
        app.mark_failed();

        assert_eq!(app.status, GameStatus::Failed);
        assert_eq!(app.score.status, GameStatus::Failed);
        assert_eq!(app.puzzle.as_ref().unwrap().status, GameStatus::Failed);
    }

    #[test]
    fn test_tui_app_tick_causes_failure_on_expiry() {
        let mut app = TuiApp::new(2);
        assert_eq!(app.status, GameStatus::InProgress);

        app.tick();
        assert_eq!(app.status, GameStatus::InProgress);
        assert_eq!(app.timer.remaining(), 1);

        app.tick();
        assert_eq!(app.status, GameStatus::Failed);
        assert_eq!(app.timer.remaining(), 0);
    }

    #[test]
    fn test_tui_app_tick_does_not_overwrite_solved() {
        let mut app = TuiApp::new(2);
        app.mark_solved();

        app.tick();
        app.tick();
        // Should remain solved even though timer expired
        assert_eq!(app.status, GameStatus::Solved);
    }
}
