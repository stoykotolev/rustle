use std::io;

use strustle::cli::{self, CliAction};
use strustle::game::Game;
use strustle::nyt::fetch_solution;
use strustle::persistence::GameStore;
use strustle::tui;

fn main() {
    // Handle `--help`/`--version` before any I/O so they work offline and never
    // touch the terminal or the save file.
    match cli::parse_args(std::env::args()) {
        CliAction::Run => {}
        CliAction::Help => {
            println!("{}", cli::help_text());
            return;
        }
        CliAction::Version => {
            println!("{}", cli::version_line());
            return;
        }
        CliAction::Unknown(arg) => {
            eprintln!("error: unrecognized argument '{arg}'");
            eprintln!("Try '{} --help' for usage.", env!("CARGO_PKG_NAME"));
            std::process::exit(2);
        }
    }

    // Use the same date source as the NYT client so the saved record lines up
    // with the puzzle that would otherwise be fetched.
    let today = chrono::Local::now().date_naive();
    let mut store = GameStore::load();

    match run(today, &mut store) {
        Ok(outcome) => {
            // Confetti / loss sound run *after* the terminal is restored so they
            // never interfere with the alternate screen. Replays are read-only:
            // they never re-trigger effects, but a replayed loss still exits 1.
            match outcome {
                Outcome::Won => strustle::celebrate::confetti(),
                Outcome::Lost => {
                    strustle::celebrate::play_sad_sound();
                    std::process::exit(1);
                }
                Outcome::Replayed { won: false } => std::process::exit(1),
                Outcome::Quit | Outcome::Replayed { won: true } => {}
            }
        }
        Err(err) => {
            // Make sure the terminal is usable before printing the error.
            let _ = tui::restore();
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    }
}

/// How a session ended, used to pick the right post-game effect and exit code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Won,
    Lost,
    /// The player quit a fresh game before finishing it.
    Quit,
    /// Today was already played; the board was replayed read-only. A lost replay
    /// still exits non-zero but triggers no effects.
    Replayed {
        won: bool,
    },
}

/// Orchestrates one launch: replay if already played, otherwise fetch the
/// solution, run the TUI, and persist the result. The terminal is always
/// restored before returning.
fn run(today: chrono::NaiveDate, store: &mut GameStore) -> io::Result<Outcome> {
    // If today's game is already finished, replay it read-only and exit without
    // contacting NYT.
    if let Some(played) = store.played_today(today) {
        let won = played.won;
        let mut terminal = tui::init()?;
        let result = tui::run_replay(&mut terminal, &played, &store.history(), store.stats());
        tui::restore()?;
        result?;
        return Ok(Outcome::Replayed { won });
    }

    let solution = fetch_solution().map_err(|err| io::Error::other(err.to_string()))?;
    let mut game = Game::new(solution.chars().collect::<Vec<char>>())
        .map_err(|err| io::Error::other(err.to_string()))?;

    let history = store.history();
    let stats = store.stats();
    let mut terminal = tui::init()?;
    let result = tui::run_game(&mut terminal, &mut game, &history, stats);
    tui::restore()?;
    result?;

    // Only a finished game is persisted; quitting mid-game records nothing so
    // the player can resume a fresh attempt later that day.
    if game.is_over() {
        let won = game.is_won();
        store.record_today(today, game.solution_string(), game.guessed_words(), won);
        Ok(if won { Outcome::Won } else { Outcome::Lost })
    } else {
        Ok(Outcome::Quit)
    }
}
