//! Persistence of the player's results, streak, and history.
//!
//! All state lives in a single JSON file (`state.json`) in the platform data
//! directory. It records every completed daily game in play order, plus the
//! current and best win streaks. From that the game can:
//!
//! * detect that today's puzzle was already finished and replay its board
//!   (the colour-coded grid is *reconstructed* on load by re-running
//!   [`crate::feedback::evaluate`], keeping that the single source of truth);
//! * show a history panel of past solutions; and
//! * show how many days in a row the player has guessed the word.
//!
//! In the spirit of the [`crate::celebrate`] module, every read failure is
//! treated as "fresh" and writes are best-effort: a corrupt, missing, or
//! unwritable save file must never crash or block the game.

use std::fs;
use std::path::PathBuf;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::feedback::{self, LetterState};
use crate::game::WORD_LEN;

/// The current on-disk schema version. A file written by an older or newer
/// version is ignored on load (the player simply starts fresh).
const SCHEMA_VERSION: u32 = 2;

/// The date format used for the `date` field, matching `Local::now().date_naive()`.
const DATE_FORMAT: &str = "%Y-%m-%d";

/// A single completed day's game, stored verbatim. The colour grid is rebuilt
/// from `solution` + `guesses` on demand rather than serialised.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameRecord {
    /// The play date in ISO `%Y-%m-%d` form.
    pub date: String,
    /// The solution, five lowercase characters.
    pub solution: String,
    /// The guesses in play order, each five lowercase characters.
    pub guesses: Vec<String>,
    /// Whether the player guessed the solution.
    pub won: bool,
}

/// The complete persisted state: schema version, streak counters, and the full
/// list of completed games in chronological order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SavedState {
    version: u32,
    current_streak: u32,
    max_streak: u32,
    games: Vec<GameRecord>,
}

impl Default for SavedState {
    fn default() -> Self {
        SavedState {
            version: SCHEMA_VERSION,
            current_streak: 0,
            max_streak: 0,
            games: Vec::new(),
        }
    }
}

/// The player's streak counters, shown in the stats panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stats {
    /// Consecutive days, ending at the most recently completed game, that were won.
    pub current_streak: u32,
    /// The longest `current_streak` ever reached.
    pub max_streak: u32,
}

/// A past game summarised for the history panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    /// The play date in ISO `%Y-%m-%d` form.
    pub date: String,
    /// The solution word.
    pub solution: String,
    /// Whether the player won.
    pub won: bool,
    /// How many guesses the player used.
    pub guess_count: usize,
}

/// A reconstructed view of a completed daily game, ready to render.
pub struct PlayedToday {
    /// The evaluated rows, each pairing a guess with its per-letter states.
    pub rows: Vec<([char; WORD_LEN], [LetterState; WORD_LEN])>,
    /// Whether the game was won.
    pub won: bool,
    /// The solution word.
    pub solution: [char; WORD_LEN],
}

/// An in-memory handle to the saved state, loaded once at startup.
///
/// Reads are pure accessors; [`GameStore::record_today`] mutates the streak and
/// history then persists best-effort.
pub struct GameStore {
    state: SavedState,
}

impl GameStore {
    /// Loads the saved state from disk, falling back to a fresh, empty store on
    /// any error (missing file, corrupt payload, or unknown schema version).
    pub fn load() -> Self {
        GameStore {
            state: load_state().unwrap_or_default(),
        }
    }

    /// Builds a store directly from a state, used in tests.
    #[cfg(test)]
    fn from_state(state: SavedState) -> Self {
        GameStore { state }
    }

    /// Returns the current and best streak counters.
    pub fn stats(&self) -> Stats {
        Stats {
            current_streak: self.state.current_streak,
            max_streak: self.state.max_streak,
        }
    }

    /// Returns past games most-recent-first for the history panel.
    pub fn history(&self) -> Vec<HistoryEntry> {
        self.state
            .games
            .iter()
            .rev()
            .map(|g| HistoryEntry {
                date: g.date.clone(),
                solution: g.solution.clone(),
                won: g.won,
                guess_count: g.guesses.len(),
            })
            .collect()
    }

    /// If today's game is already recorded, reconstructs its board for replay.
    ///
    /// Returns `None` when today has not been completed, or when the stored
    /// record fails reconstruction (a guess or the solution is not exactly five
    /// ASCII-alphabetic characters).
    pub fn played_today(&self, today: NaiveDate) -> Option<PlayedToday> {
        let target = today.format(DATE_FORMAT).to_string();
        let record = self.state.games.iter().find(|g| g.date == target)?;
        reconstruct(record)
    }

    /// Records the outcome of today's freshly completed game.
    ///
    /// Updates the streak (a win extends it when yesterday was also won, else it
    /// starts a new run of one; a loss resets it to zero), refreshes the best
    /// streak, appends the game to history, and persists best-effort.
    ///
    /// Calling this when today is already recorded is a no-op, so a streak can
    /// never be double-counted.
    pub fn record_today(
        &mut self,
        today: NaiveDate,
        solution: String,
        guesses: Vec<String>,
        won: bool,
    ) {
        let today_str = today.format(DATE_FORMAT).to_string();
        if self.state.games.iter().any(|g| g.date == today_str) {
            return;
        }

        self.state.current_streak = next_streak(
            self.state.games.last(),
            self.state.current_streak,
            today,
            won,
        );
        self.state.max_streak = self.state.max_streak.max(self.state.current_streak);
        self.state.games.push(GameRecord {
            date: today_str,
            solution,
            guesses,
            won,
        });

        // Best-effort: a write failure must never disrupt play.
        let _ = save_state(&self.state);
    }
}

/// Computes the streak after a freshly completed game.
///
/// A loss always resets the streak to zero. A win extends the previous streak by
/// one only when the immediately preceding game was *yesterday* and was itself a
/// win; any gap in days (or a preceding loss) starts a new run of one.
fn next_streak(
    prev_last: Option<&GameRecord>,
    prev_streak: u32,
    today: NaiveDate,
    won: bool,
) -> u32 {
    if !won {
        return 0;
    }
    let extends = prev_last
        .filter(|g| g.won)
        .and_then(|g| NaiveDate::parse_from_str(&g.date, DATE_FORMAT).ok())
        .and_then(|d| d.succ_opt())
        .is_some_and(|day_after_prev| day_after_prev == today);
    if extends {
        prev_streak + 1
    } else {
        1
    }
}

/// Rebuilds a renderable [`PlayedToday`] from a stored record.
fn reconstruct(record: &GameRecord) -> Option<PlayedToday> {
    let solution = word_to_array(&record.solution)?;
    let mut rows = Vec::with_capacity(record.guesses.len());
    for guess in &record.guesses {
        let arr = word_to_array(guess)?;
        let states = feedback::evaluate(&arr, &solution);
        rows.push((arr, states));
    }
    Some(PlayedToday {
        rows,
        won: record.won,
        solution,
    })
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

/// Reads and parses the saved state, returning `None` on any failure or a
/// version mismatch.
fn load_state() -> Option<SavedState> {
    let path = save_path()?;
    let bytes = fs::read(path).ok()?;
    parse_state(&bytes)
}

/// Parses raw bytes into a [`SavedState`], rejecting unknown schema versions.
fn parse_state(bytes: &[u8]) -> Option<SavedState> {
    let state: SavedState = serde_json::from_slice(bytes).ok()?;
    if state.version != SCHEMA_VERSION {
        return None;
    }
    Some(state)
}

/// Persists the state atomically (write to a sibling `.tmp` file, then rename).
fn save_state(state: &SavedState) -> std::io::Result<()> {
    let path = save_path().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "could not resolve a save directory",
        )
    })?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(state)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feedback::LetterState::{Absent as A, Correct as C};

    fn date(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, DATE_FORMAT).expect("valid test date")
    }

    fn game(date: &str, solution: &str, guesses: &[&str], won: bool) -> GameRecord {
        GameRecord {
            date: date.to_string(),
            solution: solution.to_string(),
            guesses: guesses.iter().map(|g| g.to_string()).collect(),
            won,
        }
    }

    #[test]
    fn parse_state_roundtrip() {
        let state = SavedState {
            version: SCHEMA_VERSION,
            current_streak: 3,
            max_streak: 5,
            games: vec![game("2026-06-19", "crane", &["slate", "crane"], true)],
        };
        let bytes = serde_json::to_vec(&state).unwrap();
        assert_eq!(parse_state(&bytes), Some(state));
    }

    #[test]
    fn parse_state_corrupt() {
        assert_eq!(parse_state(b"not json"), None);
    }

    #[test]
    fn parse_state_unknown_version() {
        let state = SavedState {
            version: 99,
            ..SavedState::default()
        };
        let bytes = serde_json::to_vec(&state).unwrap();
        assert_eq!(parse_state(&bytes), None);
    }

    #[test]
    fn parse_state_rejects_v1_schema() {
        // A v1 record had a different shape; it must be ignored, not partially
        // read, so the player simply starts fresh after an upgrade.
        let v1 =
            br#"{"version":1,"date":"2026-06-19","solution":"crane","guesses":[],"outcome":"won"}"#;
        assert_eq!(parse_state(v1), None);
    }

    #[test]
    fn load_falls_back_to_fresh() {
        // `from_state(default)` mirrors what `load` produces when the file is
        // missing or unreadable.
        let store = GameStore::from_state(SavedState::default());
        assert_eq!(store.stats(), Stats::default());
        assert!(store.history().is_empty());
    }

    #[test]
    fn streak_first_win_is_one() {
        assert_eq!(next_streak(None, 0, date("2026-06-19"), true), 1);
    }

    #[test]
    fn streak_first_game_loss_is_zero() {
        assert_eq!(next_streak(None, 0, date("2026-06-19"), false), 0);
    }

    #[test]
    fn streak_consecutive_win_extends() {
        let prev = game("2026-06-18", "ghost", &["ghost"], true);
        assert_eq!(next_streak(Some(&prev), 4, date("2026-06-19"), true), 5);
    }

    #[test]
    fn streak_loss_resets_to_zero() {
        let prev = game("2026-06-18", "ghost", &["ghost"], true);
        assert_eq!(next_streak(Some(&prev), 4, date("2026-06-19"), false), 0);
    }

    #[test]
    fn streak_gap_day_restarts_at_one() {
        // Last win was two days ago: the missed day breaks the streak.
        let prev = game("2026-06-17", "ghost", &["ghost"], true);
        assert_eq!(next_streak(Some(&prev), 9, date("2026-06-19"), true), 1);
    }

    #[test]
    fn streak_after_previous_loss_restarts_at_one() {
        // Yesterday was a loss, so today's win starts a new run.
        let prev = game("2026-06-18", "ghost", &["wrong", "wrong", "wrong"], false);
        assert_eq!(next_streak(Some(&prev), 0, date("2026-06-19"), true), 1);
    }

    #[test]
    fn record_today_updates_streak_and_history() {
        let mut store = GameStore::from_state(SavedState {
            version: SCHEMA_VERSION,
            current_streak: 2,
            max_streak: 2,
            games: vec![game("2026-06-18", "ghost", &["ghost"], true)],
        });
        // Persistence is keyed to the test machine's data dir; we only assert on
        // the in-memory state, which `record_today` updates before saving.
        store.record_today(
            date("2026-06-19"),
            "crane".to_string(),
            vec!["slate".to_string(), "crane".to_string()],
            true,
        );
        let stats = store.stats();
        assert_eq!(stats.current_streak, 3);
        assert_eq!(stats.max_streak, 3);
        assert_eq!(store.history().len(), 2);
        // History is most-recent-first.
        assert_eq!(store.history()[0].solution, "crane");
        assert_eq!(store.history()[0].guess_count, 2);
    }

    #[test]
    fn record_today_is_idempotent_for_same_day() {
        let mut store = GameStore::from_state(SavedState {
            version: SCHEMA_VERSION,
            current_streak: 1,
            max_streak: 1,
            games: vec![game("2026-06-19", "crane", &["crane"], true)],
        });
        store.record_today(date("2026-06-19"), "crane".to_string(), vec![], false);
        // The pre-existing record is untouched: no double count, no overwrite.
        assert_eq!(store.stats().current_streak, 1);
        assert_eq!(store.history().len(), 1);
        assert!(store.history()[0].won);
    }

    #[test]
    fn record_today_loss_breaks_max_streak_untouched() {
        let mut store = GameStore::from_state(SavedState {
            version: SCHEMA_VERSION,
            current_streak: 7,
            max_streak: 7,
            games: vec![game("2026-06-18", "ghost", &["ghost"], true)],
        });
        store.record_today(
            date("2026-06-19"),
            "crane".to_string(),
            vec!["wrong".to_string()],
            false,
        );
        assert_eq!(store.stats().current_streak, 0);
        // Best streak is preserved across a loss.
        assert_eq!(store.stats().max_streak, 7);
    }

    #[test]
    fn played_today_reconstructs_board() {
        let store = GameStore::from_state(SavedState {
            version: SCHEMA_VERSION,
            current_streak: 1,
            max_streak: 1,
            games: vec![game("2026-06-19", "crane", &["slate", "crane"], true)],
        });
        let played = store
            .played_today(date("2026-06-19"))
            .expect("today is recorded");
        assert!(played.won);
        assert_eq!(played.solution, ['c', 'r', 'a', 'n', 'e']);
        assert_eq!(played.rows.len(), 2);
        // slate vs crane: s,l absent, a correct, t absent, e correct.
        assert_eq!(played.rows[0].1, [A, A, C, A, C]);
        // The winning row is all correct.
        assert_eq!(played.rows[1].1, [C, C, C, C, C]);
    }

    #[test]
    fn played_today_none_for_other_day() {
        let store = GameStore::from_state(SavedState {
            version: SCHEMA_VERSION,
            current_streak: 1,
            max_streak: 1,
            games: vec![game("2026-06-19", "crane", &["crane"], true)],
        });
        assert!(store.played_today(date("2026-06-20")).is_none());
    }

    #[test]
    fn played_today_rejects_malformed_record() {
        let store = GameStore::from_state(SavedState {
            version: SCHEMA_VERSION,
            current_streak: 0,
            max_streak: 0,
            games: vec![game("2026-06-19", "crane", &["slat"], true)],
        });
        assert!(store.played_today(date("2026-06-19")).is_none());
    }

    #[test]
    fn word_to_array_rejects_non_ascii_and_wrong_length() {
        assert!(word_to_array("crane").is_some());
        assert!(word_to_array("crÄne").is_none());
        assert!(word_to_array("toolong").is_none());
        assert!(word_to_array("abc").is_none());
    }
}
