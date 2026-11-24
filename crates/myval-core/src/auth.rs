use std::collections::HashMap;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::Utc;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use url::Url;

use crate::error::MyValError;
use crate::models::auth::{RegionShard, RiotSession};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRedirectTokens {
    pub access_token: String,
    pub id_token: String,
    pub expires_in_secs: i64,
}

#[derive(Debug, Deserialize)]
struct EntitlementsResp {
    entitlements_token: String,
}

#[derive(Debug, Deserialize)]
struct UserInfoResp {
    sub: String,
    #[serde(default)]
    acct: Option<UserInfoAcct>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct UserInfoAcct {
    game_name: String,
    tag_line: String,
}

#[derive(Debug, Deserialize)]
struct RiotGeoResp {
    #[serde(default)]
    affinities: RiotGeoAffinities,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RiotGeoAffinities {
    live: String,
}

pub fn build_riot_login_url() -> String {
    "https://auth.riotgames.com/authorize?redirect_uri=https%3A%2F%2Fplayvalorant.com%2Fopt_in&client_id=play-valorant-web-prod&response_type=token%20id_token&nonce=1&scope=account%20openid".to_string()
}

pub fn parse_redirect_url(redirect_url: &str) -> Result<ParsedRedirectTokens, MyValError> {
    let parsed = Url::parse(redirect_url.trim())
        .map_err(|e| MyValError::InvalidInput(format!("Invalid redirect URL: {e}")))?;

    let mut params: HashMap<String, String> = HashMap::new();
    if let Some(fragment) = parsed.fragment() {
        for (k, v) in url::form_urlencoded::parse(fragment.as_bytes()) {
            params.insert(k.into_owned(), v.into_owned());
        }
    }
    for (k, v) in parsed.query_pairs() {
        params.entry(k.into_owned()).or_insert_with(|| v.into_owned());
    }

    if let Some(err) = params.get("error") {
        let desc = params
            .get("error_description")
            .cloned()
            .unwrap_or_else(|| err.clone());
        return Err(MyValError::Unauthorized(desc));
    }

    let access_token = params
        .get("access_token")
        .filter(|s| !s.is_empty())
        .cloned()
        .ok_or_else(|| MyValError::InvalidInput("Missing access_token in redirect URL".to_string()))?;

    let id_token = params
        .get("id_token")
        .filter(|s| !s.is_empty())
        .cloned()
        .ok_or_else(|| MyValError::InvalidInput("Missing id_token in redirect URL".to_string()))?;

    let expires_in_secs = params
        .get("expires_in")
        .and_then(|s| s.parse::<i64>().ok())
        .filter(|v| (60..=86_400 * 30).contains(v))
        .unwrap_or(3600);

    Ok(ParsedRedirectTokens {
        access_token,
        id_token,
        expires_in_secs,
    })
}

pub fn region_to_shard(region: &str) -> RegionShard {
    let norm = region.trim().to_lowercase();
    let region_clean = if norm.is_empty() {
        "na".to_string()
    } else {
        norm.clone()
    };
    let shard = match region_clean.as_str() {
        "latam" | "br" => "na".to_string(),
        "eu" | "ap" | "kr" | "na" => region_clean.clone(),
        _ => "na".to_string(),
    };
    RegionShard {
        region: region_clean,
        shard,
    }
}

pub fn build_client_platform_b64() -> String {
    let json = r#"{"platformType":"PC","platformOS":"Windows","platformOSVersion":"10.0.19042.1.256.64bit","platformChipset":"Unknown"}"#;
    STANDARD.encode(json.as_bytes())
}

pub fn build_riot_headers(
    session: &RiotSession,
    riot_client_version: &str,
) -> Result<HeaderMap, MyValError> {
    let mut headers = HeaderMap::new();
    let bearer = format!("Bearer {}", session.access_token);
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&bearer).map_err(|e| MyValError::InvalidInput(e.to_string()))?,
    );
    headers.insert(
        "X-Riot-Entitlements-JWT",
        HeaderValue::from_str(&session.entitlements_token)
            .map_err(|e| MyValError::InvalidInput(e.to_string()))?,
    );
    headers.insert(
        "X-Riot-ClientPlatform",
        HeaderValue::from_str(&build_client_platform_b64())
            .map_err(|e| MyValError::InvalidInput(e.to_string()))?,
    );
    headers.insert(
        "X-Riot-ClientVersion",
        HeaderValue::from_str(riot_client_version)
            .map_err(|e| MyValError::InvalidInput(e.to_string()))?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    Ok(headers)
}

/// Completes on-device Riot Sign-On after the visible WebView redirects with tokens.
pub async fn complete_auth_from_redirect_url(
    http: &reqwest::Client,
    redirect_url: &str,
) -> Result<RiotSession, MyValError> {
    let tokens = parse_redirect_url(redirect_url)?;

    let (ent_res, user_res, geo_res) = tokio::join!(
        http.post("https://entitlements.auth.riotgames.com/api/token/v1")
            .bearer_auth(&tokens.access_token)
            .header(CONTENT_TYPE, "application/json")
            .body("{}")
            .send(),
        http.get("https://auth.riotgames.com/userinfo")
            .bearer_auth(&tokens.access_token)
            .send(),
        http.put("https://riot-geo.pas.si.riotgames.com/pas/v1/product/valorant")
            .bearer_auth(&tokens.access_token)
            .json(&serde_json::json!({ "id_token": tokens.id_token }))
            .send(),
    );

    let ent_resp = ent_res?;
    if ent_resp.status().as_u16() == 401 {
        return Err(MyValError::Unauthorized("Access token rejected by entitlements".to_string()));
    }
    if !ent_resp.status().is_success() {
        return Err(MyValError::Api {
            status: ent_resp.status().as_u16(),
            message: "Failed to fetch Riot entitlements JWT".to_string(),
        });
    }
    let entitlements_token = ent_resp.json::<EntitlementsResp>().await?.entitlements_token;

    let user_resp = user_res?;
    if !user_resp.status().is_success() {
        return Err(MyValError::Api {
            status: user_resp.status().as_u16(),
            message: "Failed to fetch Riot userinfo".to_string(),
        });
    }
    let user_info: UserInfoResp = user_resp.json().await?;
    let acct = user_info.acct.unwrap_or_default();

    let geo_resp = geo_res?;
    let region_raw = if geo_resp.status().is_success() {
        geo_resp
            .json::<RiotGeoResp>()
            .await
            .map(|g| g.affinities.live)
            .unwrap_or_else(|_| "na".to_string())
    } else {
        "na".to_string()
    };
    let rs = region_to_shard(&region_raw);

    Ok(RiotSession {
        puuid: user_info.sub,
        game_name: acct.game_name,
        tag_line: acct.tag_line,
        region: rs.region,
        shard: rs.shard,
        access_token: tokens.access_token,
        id_token: tokens.id_token,
        entitlements_token,
        expires_at_epoch_secs: Utc::now().timestamp().saturating_add(tokens.expires_in_secs),
    })
}

/// Silently refreshes an existing Riot session using the WebView's `auth.riotgames.com` cookies.
pub async fn refresh_session_from_cookie(
    http: &reqwest::Client,
    cookie_header: &str,
) -> Result<RiotSession, MyValError> {
    let trimmed = cookie_header.trim();
    if trimmed.is_empty() {
        return Err(MyValError::Unauthorized(
            "No Riot SSO cookies present".to_string(),
        ));
    }
    let no_redirect = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("Mozilla/5.0 (Linux; Android 15; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36")
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let resp = no_redirect
        .get(build_riot_login_url())
        .header(reqwest::header::COOKIE, trimmed)
        .send()
        .await?;

    if let Some(loc) = resp
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|v| v.to_str().ok())
    {
        if loc.contains("access_token=") {
            return complete_auth_from_redirect_url(http, loc).await;
        }
    }

    Err(MyValError::Unauthorized(
        "Riot SSO cookie requires re-authentication".to_string(),
    ))
}
