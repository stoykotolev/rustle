//! Terminal lifecycle and the interactive event loops.
//!
//! This is the stateful counterpart to [`crate::ui`]: it owns raw-mode setup and
//! teardown, the alternate screen, and the blocking key-event loops that drive a
//! live game ([`run_game`]) or replay a finished one ([`run_replay`]). All the
//! actual drawing is delegated to [`crate::ui::draw`].

use std::io::{self, Stdout};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, ExecutableCommand};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::game::{Game, TurnResult, WORD_LEN};
use crate::persistence::{HistoryEntry, PlayedToday, Stats};
use crate::ui::{self, View};

/// The concrete terminal type used throughout the app.
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Enters the alternate screen in raw mode and installs a panic hook that
/// restores the terminal first, so a crash never leaves the user's shell
/// garbled.
pub fn init() -> io::Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Best-effort restore; ignore errors since we are already panicking.
        let _ = restore();
        original_hook(info);
    }));

    Terminal::new(CrosstermBackend::new(stdout))
}

/// Leaves the alternate screen and disables raw mode. Safe to call more than
/// once; each step is independent and best-effort from the caller's side.
pub fn restore() -> io::Result<()> {
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

/// Runs the interactive game until the player quits or the game ends.
///
/// Mutates `game` via [`Game::apply_turn`] as guesses are submitted. The loop
/// keeps running after the game is over so the player can admire the final
/// board; pressing Enter, Esc, or Ctrl-C then exits.
pub fn run_game(
    terminal: &mut Tui,
    game: &mut Game,
    history: &[HistoryEntry],
    stats: Stats,
) -> io::Result<()> {
    let mut input = String::new();
    let mut message = String::from("Guess the word!");

    loop {
        let over = game.is_over();
        let footer = if over {
            "Enter / Esc / q to quit"
        } else {
            "a–z to type · Enter to submit · Backspace to delete · Esc to quit"
        };
        terminal.draw(|frame| {
            ui::draw(
                frame,
                &View {
                    rows: game.evaluated_rows(),
                    current_input: &input,
                    message: &message,
                    history,
                    stats,
                    footer,
                },
            );
        })?;

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        if is_quit(&key) {
            break;
        }

        if over {
            if matches!(key.code, KeyCode::Enter | KeyCode::Char('q')) {
                break;
            }
            continue;
        }

        match key.code {
            KeyCode::Char(c) if c.is_ascii_alphabetic() => {
                if input.chars().count() < WORD_LEN {
                    input.push(c.to_ascii_lowercase());
                }
            }
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Enter => {
                message = match game.apply_turn(&input) {
                    TurnResult::Rejected(reason) => reason.to_string(),
                    TurnResult::Accepted => "Nice — keep going!".to_string(),
                    TurnResult::GameOver if game.is_won() => "You got it! 🎉".to_string(),
                    TurnResult::GameOver => {
                        format!("The word was {}.", game.solution_string().to_uppercase())
                    }
                };
                input.clear();
            }
            _ => {}
        }
    }

    Ok(())
}

/// Renders a finished game's board and waits for the player to quit.
///
/// No input is accepted: the day is already over, so the board, history, and
/// stats are shown read-only until Enter, Esc, q, or Ctrl-C is pressed.
pub fn run_replay(
    terminal: &mut Tui,
    played: &PlayedToday,
    history: &[HistoryEntry],
    stats: Stats,
) -> io::Result<()> {
    let message = if played.won {
        "You already played today — you won! 🎉".to_string()
    } else {
        let solution: String = played.solution.iter().collect();
        format!(
            "You already played today. The word was {}.",
            solution.to_uppercase()
        )
    };

    loop {
        terminal.draw(|frame| {
            ui::draw(
                frame,
                &View {
                    rows: &played.rows,
                    current_input: "",
                    message: &message,
                    history,
                    stats,
                    footer: "Enter / Esc / q to quit · come back tomorrow",
                },
            );
        })?;

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        if is_quit(&key) || matches!(key.code, KeyCode::Enter | KeyCode::Char('q')) {
            break;
        }
    }

    Ok(())
}

/// Returns `true` for the global quit chords: Esc or Ctrl-C.
fn is_quit(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Esc)
        || (key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')))
}
