use thiserror::Error;

/// All errors that can be produced by Rustle.
///
/// Variants wrap the underlying library errors where applicable so that
/// callers receive typed, displayable diagnostics.
#[derive(Debug, Error)]
pub enum RustleError {
    /// An HTTP transport error occurred while contacting the NYT Wordle API.
    #[error("Could not reach the NYT Wordle API - check your internet connection ({0})")]
    Http(#[from] reqwest::Error),

    /// The NYT Wordle API responded with a non-2xx HTTP status code.
    #[error("NYT Wordle API returned an unexpected status: {0}")]
    HttpStatus(reqwest::StatusCode),

    /// The NYT Wordle API returned a JSON payload whose `solution` field is
    /// an empty string.
    #[error("NYT Wordle API returned an empty solution string")]
    EmptySolution,

    /// The response body could not be decoded as UTF-8.
    #[error("UTF-8 decoding failed: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    /// The response body was not valid JSON or lacked the expected schema.
    #[error("JSON parsing failed: {0}")]
    Json(#[from] serde_json::Error),

    /// The solution word did not have the expected length of five characters.
    #[error("expected a 5-letter word, got {0} characters")]
    InvalidWordLength(usize),
}
