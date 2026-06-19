use thiserror::Error;

#[derive(Debug, Error)]
pub enum RustleError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("UTF-8 decoding failed: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("JSON parsing failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("expected a 5-letter word, got {0} characters")]
    InvalidWordLength(usize),
}
