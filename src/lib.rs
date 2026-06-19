//! **Rustle** — a terminal Wordle clone that pulls the daily NYT solution.
//!
//! # Modules
//!
//! | Module | Purpose |
//! |---|---|
//! | [`celebrate`] | Optional end-of-game effects (confetti, loss sound). |
//! | [`dictionary`] | Bundled word-list lookup via [`dictionary::is_valid_word`]. |
//! | [`error`] | Unified error type [`error::RustleError`]. |
//! | [`feedback`] | Two-pass letter-state evaluation ([`feedback::evaluate`]). |
//! | [`game`] | Core game loop ([`game::Game`]). |
//! | [`nyt`] | NYT Wordle API client ([`nyt::fetch_solution`]). |
//! | [`persistence`] | Daily-result save/load ([`persistence::load_today`]). |
//! | [`render`] | ANSI colour rendering ([`render::render_board`]). |

pub mod celebrate;
pub mod dictionary;
pub mod error;
pub mod feedback;
pub mod game;
pub mod nyt;
pub mod persistence;
pub mod render;
