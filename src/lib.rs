//! **Rustle** — a terminal Wordle clone that pulls the daily NYT solution.
//!
//! # Modules
//!
//! | Module | Purpose |
//! |---|---|
//! | [`celebrate`] | Optional end-of-game effects (confetti, loss sound). |
//! | [`cli`] | `--help`/`--version` argument parsing ([`cli::parse_args`]). |
//! | [`dictionary`] | Two-list guess validation ([`dictionary::is_allowed_guess`]). |
//! | [`error`] | Unified error type [`error::RustleError`]. |
//! | [`feedback`] | Two-pass letter-state evaluation ([`feedback::evaluate`]). |
//! | [`game`] | Core game state machine ([`game::Game`]). |
//! | [`nyt`] | NYT Wordle API client ([`nyt::fetch_solution`]). |
//! | [`persistence`] | Streak/history save/load ([`persistence::GameStore`]). |
//! | [`tui`] | Terminal lifecycle and event loops ([`tui::run_game`]). |
//! | [`ui`] | Ratatui rendering of the board, history, and keyboard ([`ui::draw`]). |

pub mod celebrate;
pub mod cli;
pub mod dictionary;
pub mod error;
pub mod feedback;
pub mod game;
pub mod nyt;
pub mod persistence;
pub mod tui;
pub mod ui;
