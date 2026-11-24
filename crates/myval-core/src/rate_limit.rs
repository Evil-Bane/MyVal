use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use tokio::sync::Semaphore;

use crate::error::MyValError;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RiotErrorPayload {
    #[serde(rename = "errorCode")]
    error_code: String,
    message: String,
}

pub fn format_riot_api_error(status: u16, url_path: &str, body: &str) -> String {
    if let Ok(parsed) = serde_json::from_str::<RiotErrorPayload>(body) {
        let code = parsed.error_code.trim().to_uppercase();
        let friendly = match code.as_str() {
            "ERR_INVALID_PARTY_STATE" | "PARTY_IN_MATCHMAKING" => {
                Some("Action isn't allowed in the current party state (leave queue first).")
            }
            "ERR_NOT_PARTY_OWNER" | "NOT_PARTY_OWNER" => {
                Some("Only the party leader can do that.")
            }
            "ERR_PARTY_NOT_READY" | "PARTY_NOT_READY" => {
                Some("All party members must be ready before starting queue.")
            }
            "ERR_QUEUE_DISABLED" | "INELIGIBLE_QUEUE" | "ERR_INELIGIBLE_QUEUE" => {
                Some("That queue isn't available for your current party.")
            }
            "ERR_PARTY_FULL" => Some("That party is already full."),
            "ERR_PARTY_CLOSED" => Some("That party is closed to invite codes."),
            "RESOURCE_NOT_FOUND" | "PLAYER_DOES_NOT_EXIST" => {
                if url_path.contains("/parties/")
                    || url_path.contains("/core-game/")
                    || url_path.contains("/pregame/")
                {
                    Some("No active Valorant PC session or party found — open Valorant on PC first.")
                } else {
                    None
                }
            }
            "PREGAME_NOT_FOUND" | "PLAYER_NOT_IN_PREGAME" => {
                Some("Agent select is no longer active.")
            }
            "CHARACTER_ALREADY_LOCKED" => Some("That agent is already locked."),
            _ => None,
        };
        if let Some(msg) = friendly {
            return msg.to_string();
        }
        let msg = parsed.message.trim();
        if !msg.is_empty() {
            return if code.is_empty() {
                msg.to_string()
            } else {
                format!("{msg} ({code})")
            };
        }
        if !code.is_empty() {
            return format!("Riot rejected request ({code})");
        }
    }
    let snippet: String = body.trim().chars().take(180).collect();
    if snippet.is_empty() {
        format!("HTTP {status} on {url_path}")
    } else {
        format!("HTTP {status} on {url_path}: {snippet}")
    }
}

#[derive(Debug)]
pub enum RateAttempt<T> {
    Done(Result<T, MyValError>),
    Retry429 { retry_after_secs: Option<u64> },
}

#[derive(Clone, Debug)]
pub struct RateLimiter {
    semaphore: Arc<Semaphore>,
}

impl RateLimiter {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent.max(1))),
        }
    }

    /// Executes an async Riot HTTP operation with bounded concurrency and exponential backoff on HTTP 429.
    pub async fn execute_with_retry<T, F, Fut>(&self, mut op: F) -> Result<T, MyValError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = RateAttempt<T>>,
    {
        let mut backoff_secs = 1u64;
        for attempt in 0..3 {
            let _permit = self
                .semaphore
                .acquire()
                .await
                .map_err(|_| MyValError::Network("Rate limiter closed".to_string()))?;

            match op().await {
                RateAttempt::Done(res) => return res,
                RateAttempt::Retry429 { retry_after_secs } => {
                    drop(_permit);
                    let wait = retry_after_secs.unwrap_or(backoff_secs).clamp(1, 15);
                    if attempt == 2 {
                        return Err(MyValError::RateLimited {
                            retry_after_secs: wait,
                        });
                    }
                    tokio::time::sleep(Duration::from_secs(wait)).await;
                    backoff_secs = (backoff_secs * 2).min(12);
                }
            }
        }
        Err(MyValError::RateLimited {
            retry_after_secs: 5,
        })
    }

    async fn request_json_internal<T, F>(
        &self,
        mut build_req: F,
        allow_404_none: bool,
    ) -> Result<Option<T>, MyValError>
    where
        T: DeserializeOwned,
        F: FnMut() -> reqwest::RequestBuilder,
    {
        self.execute_with_retry(|| {
            let req = build_req();
            async move {
                let resp = match req.send().await {
                    Ok(r) => r,
                    Err(e) => return RateAttempt::Done(Err(MyValError::from(e))),
                };
                let url_str = resp.url().path().to_string();
                let status = resp.status().as_u16();
                if status == 404 && allow_404_none {
                    return RateAttempt::Done(Ok(None));
                }
                if status == 429 {
                    return RateAttempt::Retry429 {
                        retry_after_secs: parse_retry_after_secs(resp.headers()),
                    };
                }
                if status == 401 {
                    return RateAttempt::Done(Err(MyValError::Unauthorized(
                        "Riot access token expired or invalid".to_string(),
                    )));
                }
                if !resp.status().is_success() {
                    let text = resp.text().await.unwrap_or_default();
                    return RateAttempt::Done(Err(MyValError::Api {
                        status,
                        message: format_riot_api_error(status, &url_str, &text),
                    }));
                }
                let text = match resp.text().await {
                    Ok(t) => t,
                    Err(e) => return RateAttempt::Done(Err(MyValError::from(e))),
                };
                match serde_json::from_str::<T>(&text) {
                    Ok(val) => RateAttempt::Done(Ok(Some(val))),
                    Err(e) => {
                        let snippet: String = text.chars().take(240).collect();
                        RateAttempt::Done(Err(MyValError::Parse(format!(
                            "{url_str}: {e} | body={snippet}"
                        ))))
                    }
                }
            }
        })
        .await
    }

    pub async fn request_empty<F>(&self, mut build_req: F) -> Result<(), MyValError>
    where
        F: FnMut() -> reqwest::RequestBuilder,
    {
        self.execute_with_retry(|| {
            let req = build_req();
            async move {
                let resp = match req.send().await {
                    Ok(r) => r,
                    Err(e) => return RateAttempt::Done(Err(MyValError::from(e))),
                };
                let url_str = resp.url().path().to_string();
                let status = resp.status().as_u16();
                if status == 429 {
                    return RateAttempt::Retry429 {
                        retry_after_secs: parse_retry_after_secs(resp.headers()),
                    };
                }
                if status == 401 {
                    return RateAttempt::Done(Err(MyValError::Unauthorized(
                        "Riot access token expired or invalid".to_string(),
                    )));
                }
                if !resp.status().is_success() {
                    let text = resp.text().await.unwrap_or_default();
                    return RateAttempt::Done(Err(MyValError::Api {
                        status,
                        message: format_riot_api_error(status, &url_str, &text),
                    }));
                }
                RateAttempt::Done(Ok(()))
            }
        })
        .await
    }

    /// Shared Riot JSON request helper handling 401, 404 -> Ok(None) (for GLZ lobby probes), and 429 exponential backoff.
    pub async fn request_json_opt<T, F>(
        &self,
        build_req: F,
    ) -> Result<Option<T>, MyValError>
    where
        T: DeserializeOwned,
        F: FnMut() -> reqwest::RequestBuilder,
    {
        self.request_json_internal(build_req, true).await
    }

    pub async fn request_json<T, F>(&self, build_req: F) -> Result<T, MyValError>
    where
        T: DeserializeOwned,
        F: FnMut() -> reqwest::RequestBuilder,
    {
        self.request_json_internal(build_req, false)
            .await?
            .ok_or(MyValError::NotInSession)
    }
}

pub fn parse_retry_after_secs(headers: &HeaderMap) -> Option<u64> {
    headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.trim().parse::<u64>().ok())
}
