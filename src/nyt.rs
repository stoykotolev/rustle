use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::error::RustleError;

#[derive(Debug, Serialize, Deserialize)]
pub struct WordleData<'a> {
    pub solution: &'a str,
}

pub fn get_data() -> Result<Vec<u8>, RustleError> {
    let current_date = Local::now().date_naive();
    let wordle_url = format!("https://www.nytimes.com/svc/wordle/v2/{current_date}.json");

    let resp = reqwest::blocking::get(wordle_url)?.bytes()?;

    Ok(resp.to_vec())
}

pub fn get_word(bytes: &[u8]) -> Result<WordleData<'_>, RustleError> {
    let data = std::str::from_utf8(bytes)?;
    let word = serde_json::from_str::<WordleData>(data)?;

    Ok(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_word() {
        let sample_data = "{\"solution\":\"test\"}";
        let result = get_word(sample_data.as_bytes()).unwrap();
        assert_eq!(result.solution, "test");
    }
}
