use thiserror::Error;

#[derive(Debug, Error)]
pub enum RustleError {
    #[error("Could not reach the NYT Wordle API - check your internet connection ({0})")]
    Http(#[from] reqwest::Error),

    #[error("NYT Wordle API returned an unexpected status: {0}")]
    HttpStatus(reqwest::StatusCode),

    #[error("NYT Wordle API returned an empty solution string")]
    EmptySolution,

    #[error("UTF-8 decoding failed: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("JSON parsing failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("expected a 5-letter word, got {0} characters")]
    InvalidWordLength(usize),
}
