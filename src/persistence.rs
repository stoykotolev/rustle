//! Persistence of the player's daily result.
//!
//! The game records the outcome of each completed daily puzzle so that a second
//! launch on the same day replays the saved board instead of re-fetching the
//! NYT solution. The schema is deliberately minimal: only the raw guesses,
//! solution, and outcome are stored. The colour-coded grid is *reconstructed*
//! on load by re-running [`crate::feedback::evaluate`], keeping that function
//! the single source of truth for letter states.
//!
//! In the spirit of the [`crate::celebrate`] module, every read failure is
//! treated as "not played yet" and writes are best-effort: a corrupt, missing,
//! or unwritable save file must never crash or block the game.

use std::fs;
use std::path::PathBuf;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::feedback::{self, LetterState};
use crate::game::WORD_LEN;

/// The current on-disk schema version. Records written by older or newer
/// versions are rejected on load (treated as "not played yet").
const SCHEMA_VERSION: u32 = 1;

/// The date format used for the `date` field, matching `Local::now().date_naive()`.
const DATE_FORMAT: &str = "%Y-%m-%d";

/// The outcome of a completed daily game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    /// The player guessed the solution.
    Won,
    /// The player exhausted all guesses without finding the solution.
    Lost,
}

/// The serialised record of a single day's completed game.
///
/// The grid is *not* stored; it is rebuilt from `guesses` and `solution` via
/// [`crate::feedback::evaluate`] on load.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyRecord {
    /// The schema version this record was written with.
    pub version: u32,
    /// The play date in ISO `%Y-%m-%d` form.
    pub date: String,
    /// The solution, five lowercase characters.
    pub solution: String,
    /// The guesses in play order, each five lowercase characters.
    pub guesses: Vec<String>,
    /// Whether the player won or lost.
    pub outcome: Outcome,
}

impl DailyRecord {
    /// Builds a record for `date` from a finished game's data.
    ///
    /// `won` selects the [`Outcome`]; `solution` and `guesses` are stored
    /// verbatim and re-evaluated on load.
    pub fn new(date: NaiveDate, solution: String, guesses: Vec<String>, won: bool) -> Self {
        DailyRecord {
            version: SCHEMA_VERSION,
            date: date.format(DATE_FORMAT).to_string(),
            solution,
            guesses,
            outcome: if won { Outcome::Won } else { Outcome::Lost },
        }
    }
}

/// A reconstructed view of a previously completed daily game, ready to render.
pub struct PlayedToday {
    /// The evaluated rows, each pairing a guess with its per-letter states.
    pub rows: Vec<([char; WORD_LEN], [LetterState; WORD_LEN])>,
    /// Whether the game was won.
    pub won: bool,
    /// The solution word.
    pub solution: [char; WORD_LEN],
}

/// Resolves the path of the save file.
///
/// Prefers the platform data directory (`dirs::data_dir()`), falling back to
/// `$HOME/.local/share` when it is unavailable. Returns `None` if neither can
/// be resolved, in which case persistence becomes a no-op.
fn save_path() -> Option<PathBuf> {
    let base = dirs::data_dir().or_else(|| {
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("share"))
    })?;
    Some(base.join("strustle").join("state.json"))
}

/// Parses raw bytes into a [`DailyRecord`].
///
/// Any failure to decode, deserialise, or a record whose `version` does not
/// match [`SCHEMA_VERSION`] yields `None`.
fn parse_record(bytes: &[u8]) -> Option<DailyRecord> {
    let record: DailyRecord = serde_json::from_slice(bytes).ok()?;
    if record.version != SCHEMA_VERSION {
        return None;
    }
    Some(record)
}

/// Converts a five-character lowercase word into a `[char; WORD_LEN]`.
///
/// Returns `None` unless the word is exactly [`WORD_LEN`] ASCII-alphabetic
/// characters.
fn word_to_array(word: &str) -> Option<[char; WORD_LEN]> {
    let chars: Vec<char> = word.chars().collect();
    if chars.len() != WORD_LEN || !chars.iter().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    chars.try_into().ok()
}

/// Reconstructs a [`PlayedToday`] from a record, if it is valid and for `today`.
///
/// Returns `None` when the record is for a different date, when any stored guess
/// or the solution is not exactly five ASCII-alphabetic characters. Otherwise
/// the grid rows are rebuilt via [`crate::feedback::evaluate`].
fn record_for_today(record: DailyRecord, today: NaiveDate) -> Option<PlayedToday> {
    if record.date != today.format(DATE_FORMAT).to_string() {
        return None;
    }

    let solution = word_to_array(&record.solution)?;

    let mut rows = Vec::with_capacity(record.guesses.len());
    for guess in &record.guesses {
        let arr = word_to_array(guess)?;
        let states = feedback::evaluate(&arr, &solution);
        rows.push((arr, states));
    }

    Some(PlayedToday {
        rows,
        won: record.outcome == Outcome::Won,
        solution,
    })
}

/// Loads today's saved game, if one exists and is valid.
///
/// Composes the file read with the pure parsing functions. Every I/O error, a
/// missing file, a corrupt payload, or a record for a different day yields
/// `None`, meaning "not played yet".
pub fn load_today(today: NaiveDate) -> Option<PlayedToday> {
    let path = save_path()?;
    let bytes = fs::read(path).ok()?;
    let record = parse_record(&bytes)?;
    record_for_today(record, today)
}

/// Persists the result of today's game.
///
/// Creates the parent directory if needed, then writes atomically by
/// serialising to a sibling `.tmp` file and renaming it into place.
///
/// # Errors
///
/// Returns any underlying [`std::io::Error`]. Callers should treat persistence
/// as best-effort and ignore the result so a write failure never disrupts play.
pub fn save_result(record: &DailyRecord) -> std::io::Result<()> {
    let path = save_path().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "could not resolve a save directory",
        )
    })?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(record)?;
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, json)?;
    // On a rename failure, remove the temporary file so a stray `.tmp` is not
    // left behind; the cleanup itself is best-effort.
    if let Err(err) = fs::rename(&tmp, &path) {
        let _ = fs::remove_file(&tmp);
        return Err(err);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feedback::LetterState::{Absent as A, Correct as C};

    fn sample_record() -> DailyRecord {
        DailyRecord {
            version: SCHEMA_VERSION,
            date: "2026-06-19".to_string(),
            solution: "crane".to_string(),
            guesses: vec!["slate".to_string(), "crane".to_string()],
            outcome: Outcome::Won,
        }
    }

    fn date(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, DATE_FORMAT).expect("valid test date")
    }

    #[test]
    fn parse_record_valid() {
        let bytes = serde_json::to_vec(&sample_record()).unwrap();
        assert_eq!(parse_record(&bytes), Some(sample_record()));
    }

    #[test]
    fn parse_record_corrupt() {
        assert_eq!(parse_record(b"not json"), None);
    }

    #[test]
    fn parse_record_unknown_version() {
        let mut record = sample_record();
        record.version = 99;
        let bytes = serde_json::to_vec(&record).unwrap();
        assert_eq!(parse_record(&bytes), None);
    }

    #[test]
    fn record_for_today_matches() {
        let record = sample_record();
        let played = record_for_today(record, date("2026-06-19")).expect("matches today");
        assert_eq!(played.rows.len(), 2);
        assert!(played.won);
        assert_eq!(played.solution, ['c', 'r', 'a', 'n', 'e']);
    }

    #[test]
    fn record_for_today_stale_date() {
        let record = sample_record();
        assert!(record_for_today(record, date("2026-06-18")).is_none());
    }

    #[test]
    fn record_malformed_word() {
        let mut record = sample_record();
        record.guesses = vec!["slat".to_string()];
        assert!(record_for_today(record, date("2026-06-19")).is_none());
    }

    #[test]
    fn rows_reconstruction_matches_feedback() {
        let record = sample_record();
        let played = record_for_today(record, date("2026-06-19")).expect("matches today");

        let guess = ['s', 'l', 'a', 't', 'e'];
        let solution = ['c', 'r', 'a', 'n', 'e'];
        let expected = feedback::evaluate(&guess, &solution);
        assert_eq!(played.rows[0].1, expected);
        // Sanity-check the concrete pattern: slate vs crane.
        assert_eq!(played.rows[0].1, [A, A, C, A, C]);
        // The winning row is all correct.
        assert_eq!(played.rows[1].1, [C, C, C, C, C]);
    }

    #[test]
    fn new_builds_won_and_lost_records() {
        let won = DailyRecord::new(
            date("2026-06-19"),
            "crane".to_string(),
            vec!["slate".to_string()],
            true,
        );
        assert_eq!(won.version, SCHEMA_VERSION);
        assert_eq!(won.date, "2026-06-19");
        assert_eq!(won.outcome, Outcome::Won);

        let lost = DailyRecord::new(date("2026-06-19"), "crane".to_string(), vec![], false);
        assert_eq!(lost.outcome, Outcome::Lost);
    }

    #[test]
    fn record_for_today_reconstructs_loss() {
        // A lost game: six wrong guesses, solution never reached.
        let record = DailyRecord::new(
            date("2026-06-19"),
            "crane".to_string(),
            vec![
                "slate".to_string(),
                "moist".to_string(),
                "dough".to_string(),
                "bumpy".to_string(),
                "wrong".to_string(),
                "fizzy".to_string(),
            ],
            false,
        );
        let played = record_for_today(record, date("2026-06-19")).expect("matches today");
        assert!(!played.won);
        assert_eq!(played.rows.len(), 6);
        // The final row is the last guess, not the solution.
        assert_ne!(played.rows[5].0, played.solution);
    }

    #[test]
    fn record_for_today_empty_guesses() {
        // A record with no guesses (e.g. an interrupted game) still
        // reconstructs into an empty board rather than failing.
        let mut record = sample_record();
        record.guesses = vec![];
        let played = record_for_today(record, date("2026-06-19")).expect("matches today");
        assert!(played.rows.is_empty());
    }

    #[test]
    fn record_for_today_rejects_non_ascii() {
        let mut record = sample_record();
        // Five Unicode scalar values, but not ASCII-alphabetic.
        record.guesses = vec!["crÄne".to_string()];
        assert!(record_for_today(record, date("2026-06-19")).is_none());
    }

    #[test]
    fn record_for_today_rejects_bad_solution() {
        let mut record = sample_record();
        record.solution = "toolong".to_string();
        assert!(record_for_today(record, date("2026-06-19")).is_none());
    }

    #[test]
    fn serialize_parse_record_for_today_roundtrip() {
        // Exercises the full pure path a real save/load takes, without touching
        // the user's data directory: build -> serialize -> parse -> rebuild.
        let record = DailyRecord::new(
            date("2026-06-19"),
            "crane".to_string(),
            vec!["slate".to_string(), "crane".to_string()],
            true,
        );
        let bytes = serde_json::to_vec(&record).expect("serialize");
        let parsed = parse_record(&bytes).expect("parse");
        assert_eq!(parsed, record);
        let played = record_for_today(parsed, date("2026-06-19")).expect("matches today");
        assert!(played.won);
        assert_eq!(played.rows.len(), 2);
        assert_eq!(played.rows[1].1, [C, C, C, C, C]);
    }

    #[test]
    fn outcome_serde_roundtrip() {
        assert_eq!(serde_json::to_string(&Outcome::Won).unwrap(), "\"won\"");
        assert_eq!(serde_json::to_string(&Outcome::Lost).unwrap(), "\"lost\"");
        assert_eq!(
            serde_json::from_str::<Outcome>("\"won\"").unwrap(),
            Outcome::Won
        );
        assert_eq!(
            serde_json::from_str::<Outcome>("\"lost\"").unwrap(),
            Outcome::Lost
        );
    }
}
