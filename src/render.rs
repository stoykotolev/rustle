use crate::feedback::LetterState;

const GREEN: &str = "\x1b[1;32m";
const YELLOW: &str = "\x1b[0;33m";
const GRAY: &str = "\x1b[0;37m";
const RESET: &str = "\x1b[0m";

pub fn render_guess(
    guess: &[char; 5],
    states: &[LetterState; 5],
    out: &mut dyn std::io::Write,
) -> std::io::Result<()> {
    for i in 0..5 {
        let color = match states[i] {
            LetterState::Correct => GREEN,
            LetterState::Present => YELLOW,
            LetterState::Absent => GRAY,
        };
        write!(out, "{}{}{}", color, guess[i], RESET)?;
    }
    writeln!(out)
}
