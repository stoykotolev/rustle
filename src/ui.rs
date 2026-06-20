//! Ratatui rendering for the game.
//!
//! This module is deliberately free of terminal setup, event handling, and I/O:
//! it exposes a plain [`View`] data struct and a single pure [`draw`] function
//! that paints it into a [`Frame`]. That keeps the layout testable with
//! ratatui's `TestBackend` (see the tests at the bottom) and leaves all the
//! stateful terminal wiring to [`crate::tui`].

use std::collections::HashMap;

use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::feedback::LetterState;
use crate::game::{EvaluatedGuess, MAX_GUESSES, WORD_LEN};
use crate::persistence::{HistoryEntry, Stats};

/// Everything needed to paint one frame. Borrowed, so building a view is cheap
/// and allocation-free for the hot path.
pub struct View<'a> {
    /// The evaluated guess rows already played.
    pub rows: &'a [EvaluatedGuess],
    /// The letters the player is currently typing (empty when not interactive).
    pub current_input: &'a str,
    /// A short status line shown beneath the board.
    pub message: &'a str,
    /// Past games, most-recent-first, for the history panel.
    pub history: &'a [HistoryEntry],
    /// Streak counters for the stats panel.
    pub stats: Stats,
    /// The key-hint line pinned to the bottom of the screen.
    pub footer: &'a str,
}

const GREEN: Color = Color::Rgb(83, 141, 78);
const YELLOW: Color = Color::Rgb(181, 159, 59);
const GRAY: Color = Color::Rgb(58, 58, 60);

const KEYBOARD_ROWS: [&str; 3] = ["qwertyuiop", "asdfghjkl", "zxcvbnm"];

/// Aggregates the best-known state of every letter the player has guessed.
///
/// When a letter appears in multiple guesses its strongest state wins
/// (`Correct` > `Present` > `Absent`), mirroring how a real Wordle keyboard
/// only ever upgrades a key's colour.
pub fn keyboard_states(rows: &[EvaluatedGuess]) -> HashMap<char, LetterState> {
    let mut map: HashMap<char, LetterState> = HashMap::new();
    for (word, states) in rows {
        for i in 0..WORD_LEN {
            let entry = map.entry(word[i]).or_insert(LetterState::Absent);
            if rank(states[i]) > rank(*entry) {
                *entry = states[i];
            }
        }
    }
    map
}

/// Orders letter states so the strongest survives aggregation.
fn rank(state: LetterState) -> u8 {
    match state {
        LetterState::Absent => 1,
        LetterState::Present => 2,
        LetterState::Correct => 3,
    }
}

/// Paints the full game screen for `view` into `frame`.
pub fn draw(frame: &mut Frame, view: &View) {
    let root = Layout::vertical([
        Constraint::Min(0),    // board + side panels
        Constraint::Length(5), // on-screen keyboard
        Constraint::Length(1), // footer
    ])
    .split(frame.area());

    let content = Layout::horizontal([
        Constraint::Length(24), // board column
        Constraint::Min(22),    // history + stats column
    ])
    .split(root[0]);

    let side = Layout::vertical([
        Constraint::Min(3),    // history
        Constraint::Length(4), // stats
    ])
    .split(content[1]);

    draw_board(frame, content[0], view);
    draw_history(frame, side[0], view.history);
    draw_stats(frame, side[1], view.stats);
    draw_keyboard(frame, root[1], view.rows);
    draw_footer(frame, root[2], view.footer);
}

/// Draws the six-row board plus the status message.
fn draw_board(frame: &mut Frame, area: Rect, view: &View) {
    let input: Vec<char> = view.current_input.chars().collect();
    let mut lines: Vec<Line> = Vec::with_capacity(MAX_GUESSES + 2);

    for row in 0..MAX_GUESSES {
        if let Some((word, states)) = view.rows.get(row) {
            lines.push(board_row(word.iter().map(|c| Some(*c)), Some(states)));
        } else if row == view.rows.len() && !input.is_empty() {
            // The row currently being typed: letters with no colour yet.
            let cells = (0..WORD_LEN).map(|i| input.get(i).copied());
            lines.push(board_row(cells, None));
        } else {
            lines.push(board_row((0..WORD_LEN).map(|_| None), None));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        view.message.to_string(),
        Style::default().add_modifier(Modifier::BOLD),
    )));

    let board = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(" rustle "));
    frame.render_widget(board, area);
}

/// Builds one board line from up to [`WORD_LEN`] cells and optional states.
fn board_row(
    cells: impl Iterator<Item = Option<char>>,
    states: Option<&[LetterState; WORD_LEN]>,
) -> Line<'static> {
    let mut spans: Vec<Span> = Vec::with_capacity(WORD_LEN * 2);
    for (i, cell) in cells.enumerate() {
        if i > 0 {
            spans.push(Span::raw(" "));
        }
        spans.push(cell_span(cell, states.map(|s| s[i])));
    }
    Line::from(spans)
}

/// Renders a single board cell as a coloured ` X ` tile.
fn cell_span(letter: Option<char>, state: Option<LetterState>) -> Span<'static> {
    let glyph = match letter {
        Some(c) => format!(" {} ", c.to_ascii_uppercase()),
        None => "   ".to_string(),
    };
    let style = match state {
        Some(LetterState::Correct) => Style::default().bg(GREEN).fg(Color::White),
        Some(LetterState::Present) => Style::default().bg(YELLOW).fg(Color::White),
        Some(LetterState::Absent) => Style::default().bg(GRAY).fg(Color::White),
        // A typed-but-unsubmitted letter, or an empty slot.
        None if letter.is_some() => Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
        None => Style::default().fg(GRAY).add_modifier(Modifier::DIM),
    };
    let glyph = if letter.is_none() && state.is_none() {
        " _ ".to_string()
    } else {
        glyph
    };
    Span::styled(glyph, style)
}

/// Draws the history panel of past solutions, most recent first.
fn draw_history(frame: &mut Frame, area: Rect, history: &[HistoryEntry]) {
    let items: Vec<ListItem> = history
        .iter()
        .map(|entry| {
            let mark = if entry.won { "✓" } else { "✗" };
            let mark_style = if entry.won {
                Style::default().fg(GREEN).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)
            };
            let line = Line::from(vec![
                Span::raw(format!("{}  ", short_date(&entry.date))),
                Span::styled(
                    entry.solution.to_ascii_uppercase(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(mark, mark_style),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = if items.is_empty() {
        List::new(vec![ListItem::new(Line::from(Span::styled(
            "No past games yet",
            Style::default().add_modifier(Modifier::DIM),
        )))])
    } else {
        List::new(items)
    };

    frame.render_widget(
        list.block(Block::default().borders(Borders::ALL).title(" history ")),
        area,
    );
}

/// Renders an ISO `YYYY-MM-DD` date as a compact `Mon DD`, falling back to the
/// raw string if it cannot be parsed.
fn short_date(date: &str) -> String {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| d.format("%b %d").to_string())
        .unwrap_or_else(|_| date.to_string())
}

/// Draws the streak stats panel.
fn draw_stats(frame: &mut Frame, area: Rect, stats: Stats) {
    let lines = vec![
        from_streak("Streak", stats.current_streak, true),
        from_streak("Best", stats.max_streak, false),
    ];
    let stats_widget =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" stats "));
    frame.render_widget(stats_widget, area);
}

/// Builds a `label: value` stats line, flagging an active streak with a flame.
fn from_streak(label: &str, value: u32, flame: bool) -> Line<'static> {
    let suffix = if flame && value > 0 { " 🔥" } else { "" };
    Line::from(vec![
        Span::styled(
            format!("{label}: "),
            Style::default().add_modifier(Modifier::DIM),
        ),
        Span::styled(
            format!("{value}{suffix}"),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ])
}

/// Draws the QWERTY keyboard, colouring each key by its aggregated state.
fn draw_keyboard(frame: &mut Frame, area: Rect, rows: &[EvaluatedGuess]) {
    let states = keyboard_states(rows);
    let lines: Vec<Line> = KEYBOARD_ROWS
        .iter()
        .map(|row| {
            let mut spans: Vec<Span> = Vec::new();
            for ch in row.chars() {
                let style = match states.get(&ch) {
                    Some(LetterState::Correct) => Style::default().bg(GREEN).fg(Color::White),
                    Some(LetterState::Present) => Style::default().bg(YELLOW).fg(Color::White),
                    Some(LetterState::Absent) => {
                        Style::default().fg(GRAY).add_modifier(Modifier::DIM)
                    }
                    None => Style::default().fg(Color::White),
                };
                spans.push(Span::styled(
                    format!(" {} ", ch.to_ascii_uppercase()),
                    style,
                ));
            }
            Line::from(spans)
        })
        .collect();

    let keyboard = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(" keys "));
    frame.render_widget(keyboard, area);
}

/// Draws the bottom key-hint line.
fn draw_footer(frame: &mut Frame, area: Rect, footer: &str) {
    let widget = Paragraph::new(Line::from(Span::styled(
        footer.to_string(),
        Style::default().add_modifier(Modifier::DIM),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(widget, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feedback::LetterState::{Absent as A, Correct as C, Present as P};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn row(word: &str, states: [LetterState; WORD_LEN]) -> EvaluatedGuess {
        let arr: [char; WORD_LEN] = word.chars().collect::<Vec<_>>().try_into().unwrap();
        (arr, states)
    }

    #[test]
    fn keyboard_states_takes_strongest_state() {
        // "crane" vs "crane" would be all correct; build a mixed history where
        // 'a' is Absent in one guess but Correct in another.
        let rows = vec![row("aaxyz", [A, A, A, A, A]), row("baker", [A, C, A, A, A])];
        let states = keyboard_states(&rows);
        // 'a' was Absent then Correct -> Correct wins.
        assert_eq!(states.get(&'a'), Some(&C));
        // 'b' only ever Absent.
        assert_eq!(states.get(&'b'), Some(&A));
        // A never-guessed letter has no entry.
        assert_eq!(states.get(&'q'), None);
    }

    #[test]
    fn keyboard_states_present_does_not_downgrade_correct() {
        let rows = vec![row("scare", [A, A, C, A, A]), row("acrid", [P, A, A, A, A])];
        let states = keyboard_states(&rows);
        // 'a' is Correct in the first guess, Present in the second: stays Correct.
        assert_eq!(states.get(&'a'), Some(&C));
    }

    fn render_to_string(view: &View) -> String {
        let backend = TestBackend::new(60, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| draw(f, view)).unwrap();
        let buffer = terminal.backend().buffer().clone();
        buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    }

    #[test]
    fn draw_renders_titles_streak_and_history() {
        let rows = vec![row("slate", [A, A, C, A, C])];
        let history = vec![HistoryEntry {
            date: "2026-06-18".to_string(),
            solution: "ghost".to_string(),
            won: true,
            guess_count: 3,
        }];
        let view = View {
            rows: &rows,
            current_input: "cr",
            message: "Keep going!",
            history: &history,
            stats: Stats {
                current_streak: 4,
                max_streak: 9,
            },
            footer: "type a-z",
        };
        let text = render_to_string(&view);
        assert!(text.contains("rustle"), "board title missing");
        assert!(text.contains("history"), "history title missing");
        assert!(text.contains("stats"), "stats title missing");
        assert!(text.contains("keys"), "keyboard title missing");
        // The streak value and a past solution both render.
        assert!(text.contains('4'), "streak value missing");
        assert!(text.contains("GHOST"), "history solution missing");
        assert!(text.contains("Keep going!"), "message missing");
    }

    #[test]
    fn draw_handles_empty_history() {
        let view = View {
            rows: &[],
            current_input: "",
            message: "Guess the word!",
            history: &[],
            stats: Stats::default(),
            footer: "type a-z",
        };
        let text = render_to_string(&view);
        assert!(text.contains("No past games yet"));
    }
}
