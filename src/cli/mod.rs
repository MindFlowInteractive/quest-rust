use std::io::{self, BufRead, Write};

// --- Mock Domain Traits for Framework Context ---
// Replace these stubs with your actual module dependencies as needed.
pub struct PuzzleState {
    pub description: String,
    pub hint: String,
    pub current_score: u32,
    pub is_solved: bool,
}

pub struct GameEngine {
    pub puzzle: PuzzleState,
    pub unlocked_achievements: Vec<String>,
}

impl GameEngine {
    pub fn process_input(&mut self, input: &str) {
        if input.trim().to_lowercase() == "stellar" {
            self.puzzle.is_solved = true;
            self.puzzle.current_score += 100;
            self.unlocked_achievements
                .push("Soroban Pioneer".to_string());
        }
    }
}
// ------------------------------------------------

pub struct CliModule<'a, R: BufRead, W: Write> {
    reader: R,
    writer: &'a mut W,
}

impl<'a, R: BufRead, W: Write> CliModule<'a, R, W> {
    /// Instantiates a new CLI wrapper around arbitrary input/output streams
    pub fn new(reader: R, writer: &'a mut W) -> Self {
        CliModule { reader, writer }
    }

    /// Renders the current layout coordinates to stdout
    pub fn render_frame(&mut self, state: &PuzzleState) -> io::Result<()> {
        writeln!(self.writer, "\n========================================")?;
        writeln!(
            self.writer,
            "  PUZZLE SESSION | Score: {} pts",
            state.current_score
        )?;
        writeln!(self.writer, "========================================")?;
        writeln!(self.writer, "Description: {}", state.description)?;
        writeln!(self.writer, "Hint:        {}", state.hint)?;
        writeln!(self.writer, "----------------------------------------")?;
        write!(self.writer, "Enter your solution > ")?;
        self.writer.flush()
    }

    /// Renders a specialized banner when achievements or rewards unlock
    pub fn display_unlocks(&mut self, achievements: &[String]) -> io::Result<()> {
        for achievement in achievements {
            writeln!(
                self.writer,
                "\n🎉 ACHIEVEMENT UNLOCKED: [{}] 🎉",
                achievement
            )?;
            writeln!(
                self.writer,
                "🏆 +100 Base XP credited to profile registers."
            )?;
        }
        self.writer.flush()
    }

    /// Starts the blocking standard IO event capture loop
    pub fn start_game_loop(&mut self, engine: &mut GameEngine) -> io::Result<()> {
        let mut input_buffer = String::new();

        loop {
            // Render current frame snapshot
            self.render_frame(&engine.puzzle)?;

            // Clear buffer and read next line from terminal context stream
            input_buffer.clear();
            let bytes_read = self.reader.read_line(&mut input_buffer)?;

            // Handle EOF/exit signals safely
            if bytes_read == 0 || input_buffer.trim() == "exit" {
                writeln!(self.writer, "\nSession terminated. Goodbye!")?;
                break;
            }

            // Route input through parsing engines
            let sanitized_input = input_buffer.trim();
            engine.process_input(sanitized_input);

            // Check for unlocks immediately after processing input
            if !engine.unlocked_achievements.is_empty() {
                self.display_unlocks(&engine.unlocked_achievements)?;
            }

            // Exit state condition checks
            if engine.puzzle.is_solved {
                writeln!(self.writer, "\n✨ Puzzle Completed Successfully! ✨")?;
                break;
            }
        }
        Ok(())
    }
}
