use thiserror::Error;

#[derive(Debug, Error)]
pub enum MyValError {
    #[error("Network request failed: {0}")]
    Network(String),

    #[error("Unauthorized or expired Riot session: {0}")]
    Unauthorized(String),

    #[error("Rate limited by Riot API (retry after {retry_after_secs}s)")]
    RateLimited { retry_after_secs: u64 },

    #[error("Not currently in an active Valorant session or lobby")]
    NotInSession,

    #[error("Riot API returned HTTP {status}: {message}")]
    Api { status: u16, message: String },

    #[error("Failed to parse response or payload: {0}")]
    Parse(String),

    #[error("Local SQLite storage error: {0}")]
    Storage(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl From<reqwest::Error> for MyValError {
    fn from(err: reqwest::Error) -> Self {
        Self::Network(err.to_string())
    }
}

impl From<serde_json::Error> for MyValError {
    fn from(err: serde_json::Error) -> Self {
        Self::Parse(err.to_string())
    }
}

impl From<rusqlite::Error> for MyValError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Storage(err.to_string())
    }
}
