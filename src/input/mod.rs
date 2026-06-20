//! Input module.
//!
//! Captures and normalises player input for puzzle interaction. Maps
//! command-line/keyboard input streams into structured [`GameAction`] values.

use std::fmt;
use std::io::BufRead;

/// Structured representation of actions a player can take in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameAction {
    /// Selecting a specific puzzle option or index.
    Select(usize),
    /// Confirming an action or accepting a choice.
    Confirm,
    /// Returning to the previous screen or canceling.
    Back,
    /// Exiting the game or puzzle session.
    Exit,
    /// Asking for help or instructions.
    Help,
}

/// Errors returned during input capture and parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputError {
    /// The input was not recognized as a valid game action.
    InvalidInput(String),
    /// No input was provided.
    EmptyInput,
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputError::InvalidInput(s) => write!(f, "Invalid input: '{}'", s),
            InputError::EmptyInput => write!(f, "Input cannot be empty"),
        }
    }
}

impl std::error::Error for InputError {}

/// Normalises and parses a raw string input into a [`GameAction`].
///
/// Leading and trailing whitespaces are trimmed, and the text is converted
/// to lowercase before matching.
pub fn parse_input(input: &str) -> Result<GameAction, InputError> {
    let normalised = input.trim().to_lowercase();
    if normalised.is_empty() {
        return Err(InputError::EmptyInput);
    }

    match normalised.as_str() {
        "confirm" | "yes" | "y" | "ok" | "agree" | "submit" => Ok(GameAction::Confirm),
        "back" | "no" | "n" | "cancel" | "return" => Ok(GameAction::Back),
        "exit" | "quit" | "q" => Ok(GameAction::Exit),
        "help" | "h" | "info" | "?" => Ok(GameAction::Help),
        other => {
            // Attempt to parse standalone numeric selection
            if let Ok(num) = other.parse::<usize>() {
                return Ok(GameAction::Select(num));
            }

            // Attempt to parse option prefixed with 'select'
            if let Some(stripped) = other.strip_prefix("select ")
                && let Ok(num) = stripped.trim().parse::<usize>()
            {
                return Ok(GameAction::Select(num));
            }

            Err(InputError::InvalidInput(input.to_string()))
        }
    }
}

/// Captures a single line of input from a [`BufRead`] reader source,
/// normalises it, and parses it into a [`GameAction`].
pub fn read_action<R: BufRead>(reader: &mut R) -> Result<GameAction, InputError> {
    let mut buffer = String::new();
    reader
        .read_line(&mut buffer)
        .map_err(|e| InputError::InvalidInput(e.to_string()))?;

    parse_input(&buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_valid_confirm() {
        assert_eq!(parse_input("confirm"), Ok(GameAction::Confirm));
        assert_eq!(parse_input("  YES  "), Ok(GameAction::Confirm));
        assert_eq!(parse_input("y"), Ok(GameAction::Confirm));
        assert_eq!(parse_input("ok"), Ok(GameAction::Confirm));
        assert_eq!(parse_input("submit"), Ok(GameAction::Confirm));
    }

    #[test]
    fn test_parse_valid_back() {
        assert_eq!(parse_input("back"), Ok(GameAction::Back));
        assert_eq!(parse_input("  no  "), Ok(GameAction::Back));
        assert_eq!(parse_input("n"), Ok(GameAction::Back));
        assert_eq!(parse_input("cancel"), Ok(GameAction::Back));
        assert_eq!(parse_input("return"), Ok(GameAction::Back));
    }

    #[test]
    fn test_parse_valid_exit() {
        assert_eq!(parse_input("exit"), Ok(GameAction::Exit));
        assert_eq!(parse_input("QUIT"), Ok(GameAction::Exit));
        assert_eq!(parse_input("q"), Ok(GameAction::Exit));
    }

    #[test]
    fn test_parse_valid_help() {
        assert_eq!(parse_input("help"), Ok(GameAction::Help));
        assert_eq!(parse_input("?"), Ok(GameAction::Help));
        assert_eq!(parse_input("info"), Ok(GameAction::Help));
    }

    #[test]
    fn test_parse_valid_select() {
        assert_eq!(parse_input("1"), Ok(GameAction::Select(1)));
        assert_eq!(parse_input("42"), Ok(GameAction::Select(42)));
        assert_eq!(parse_input("select 3"), Ok(GameAction::Select(3)));
        assert_eq!(parse_input("  select   5  "), Ok(GameAction::Select(5)));
    }

    #[test]
    fn test_parse_empty_input() {
        assert_eq!(parse_input(""), Err(InputError::EmptyInput));
        assert_eq!(parse_input("   \n\r "), Err(InputError::EmptyInput));
    }

    #[test]
    fn test_parse_invalid_inputs() {
        assert_eq!(
            parse_input("invalid"),
            Err(InputError::InvalidInput("invalid".to_string()))
        );
        assert_eq!(
            parse_input("select abc"),
            Err(InputError::InvalidInput("select abc".to_string()))
        );
        assert_eq!(
            parse_input("confirm please"),
            Err(InputError::InvalidInput("confirm please".to_string()))
        );
    }

    #[test]
    fn test_read_action_from_stream() {
        let mut input = Cursor::new("yes\n  select 4  \nquit\ninvalid\n");

        assert_eq!(read_action(&mut input), Ok(GameAction::Confirm));
        assert_eq!(read_action(&mut input), Ok(GameAction::Select(4)));
        assert_eq!(read_action(&mut input), Ok(GameAction::Exit));
        assert_eq!(
            read_action(&mut input),
            Err(InputError::InvalidInput("invalid\n".to_string()))
        );
        assert_eq!(read_action(&mut input), Err(InputError::EmptyInput));
    }
}
