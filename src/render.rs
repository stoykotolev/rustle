use crate::feedback::LetterState;
use crate::game::WORD_LEN;

const GREEN: &str = "\x1b[1;32m";
const YELLOW: &str = "\x1b[0;33m";
const GRAY: &str = "\x1b[0;37m";
const RESET: &str = "\x1b[0m";

/// Renders a single evaluated guess row to `out`.
///
/// Each character is wrapped in the ANSI colour sequence that corresponds to
/// its [`crate::feedback::LetterState`]: green for `Correct`, yellow for
/// `Present`, and gray for `Absent`.
pub fn render_guess(
    guess: &[char; WORD_LEN],
    states: &[LetterState; WORD_LEN],
    out: &mut dyn std::io::Write,
) -> std::io::Result<()> {
    for i in 0..WORD_LEN {
        let color = match states[i] {
            LetterState::Correct => GREEN,
            LetterState::Present => YELLOW,
            LetterState::Absent => GRAY,
        };
        write!(out, "{}{}{}", color, guess[i], RESET)?;
    }
    writeln!(out)
}

/// Clears the terminal and renders every evaluated guess row in order.
///
/// Calls [`render_guess`] for each `(guess, states)` pair in `rows`. The
/// terminal is cleared with a standard ANSI escape sequence before drawing so
/// that each game tick replaces the previous board rather than appending to it.
pub fn render_board(
    rows: &[([char; WORD_LEN], [LetterState; WORD_LEN])],
    out: &mut dyn std::io::Write,
) -> std::io::Result<()> {
    write!(out, "\x1b[2J\x1b[H")?;
    for (guess, states) in rows {
        render_guess(guess, states, out)?;
    }
    Ok(())
}
