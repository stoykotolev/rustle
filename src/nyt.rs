use serde::Deserialize;

use crate::error::RustleError;

#[derive(Deserialize)]
struct WordleData {
    solution: String,
}

/// Parses a raw JSON response body from the NYT Wordle API.
///
/// Returns the `solution` field as an owned `String`.
///
/// # Errors
///
/// Returns [`crate::error::RustleError::Utf8`] if `bytes` is not valid UTF-8,
/// [`crate::error::RustleError::Json`] if the bytes are not valid JSON or lack
/// the `solution` field, and [`crate::error::RustleError::EmptySolution`] if
/// the `solution` field is an empty string.
pub(crate) fn parse_solution(bytes: &[u8]) -> Result<String, RustleError> {
    let data = std::str::from_utf8(bytes)?;
    let WordleData { solution } = serde_json::from_str::<WordleData>(data)?;
    if solution.is_empty() {
        return Err(RustleError::EmptySolution);
    }
    Ok(solution)
}

/// Fetches today's Wordle solution from the NYT API.
///
/// Constructs the request URL using the current local date and sends a
/// blocking HTTP GET with a 10-second timeout.
///
/// # Errors
///
/// Returns [`crate::error::RustleError::Http`] on a transport failure,
/// [`crate::error::RustleError::HttpStatus`] when the server responds with a
/// non-2xx status, or any error that parsing the response body can return
/// (see the [`crate::error::RustleError`] `Utf8`, `Json`, and `EmptySolution`
/// variants).
pub fn fetch_solution() -> Result<String, RustleError> {
    let current_date = chrono::Local::now().date_naive();
    let url = format!("https://www.nytimes.com/svc/wordle/v2/{current_date}.json");
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let response = client.get(&url).send()?;
    if !response.status().is_success() {
        return Err(RustleError::HttpStatus(response.status()));
    }
    let bytes = response.bytes()?;
    parse_solution(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_solution() {
        let result = parse_solution(b"{\"solution\":\"tests\"}");
        assert_eq!(result.unwrap(), "tests");
    }

    #[test]
    fn parse_empty_solution() {
        let result = parse_solution(b"{\"solution\":\"\"}");
        assert!(matches!(result, Err(RustleError::EmptySolution)));
    }

    #[test]
    fn parse_invalid_json() {
        let result = parse_solution(b"<html>not json</html>");
        assert!(matches!(result, Err(RustleError::Json(_))));
    }
}
