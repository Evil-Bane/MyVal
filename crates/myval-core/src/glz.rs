use std::collections::HashMap;

use reqwest::header::HeaderMap;
use serde::{Deserialize, Deserializer};

use crate::error::MyValError;
use crate::models::auth::RiotSession;
use crate::rate_limit::RateLimiter;

fn null_to_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    let opt = Option::<T>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

pub fn glz_base(region: &str, shard: &str) -> String {
    format!("https://glz-{region}-1.{shard}.a.pvp.net")
}

// ---- Party DTOs ----

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawPartyPlayerResp {
    #[serde(rename = "CurrentPartyID", deserialize_with = "null_to_default")]
    pub current_party_id: String,
    #[serde(rename = "Subject", deserialize_with = "null_to_default")]
    pub subject: String,
    #[serde(deserialize_with = "null_to_default")]
    pub platform_info: RawPlatformInfo,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawPlatformInfo {
    #[serde(deserialize_with = "null_to_default")]
    pub platform_type: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawPartyErrorNotification {
    #[serde(deserialize_with = "null_to_default")]
    pub error_type: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawPartyResp {
    #[serde(rename = "ID", deserialize_with = "null_to_default")]
    pub id: String,
    #[serde(deserialize_with = "null_to_default")]
    pub state: String,
    #[serde(deserialize_with = "null_to_default")]
    pub accessibility: String,
    pub invite_code: Option<String>,
    #[serde(deserialize_with = "null_to_default")]
    pub eligible_queues: Vec<String>,
    #[serde(deserialize_with = "null_to_default")]
    pub queue_entry_time: String,
    pub error_notification: Option<RawPartyErrorNotification>,
    pub matchmaking_data: Option<RawMatchmakingData>,
    #[serde(deserialize_with = "null_to_default")]
    pub members: Vec<RawPartyMember>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawMatchmakingData {
    #[serde(rename = "QueueID", deserialize_with = "null_to_default")]
    pub queue_id: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawPartyMember {
    #[serde(deserialize_with = "null_to_default")]
    pub subject: String,
    #[serde(deserialize_with = "null_to_default")]
    pub is_ready: bool,
    #[serde(deserialize_with = "null_to_default")]
    pub is_owner: bool,
    #[serde(deserialize_with = "null_to_default")]
    pub competitive_tier: i32,
    #[serde(deserialize_with = "null_to_default")]
    pub player_identity: RawPlayerIdentity,
    #[serde(deserialize_with = "null_to_default")]
    pub platform_info: RawPlatformInfo,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawPlayerIdentity {
    #[serde(deserialize_with = "null_to_default")]
    pub subject: String,
    #[serde(rename = "PlayerCardID", deserialize_with = "null_to_default")]
    pub player_card_id: String,
    #[serde(deserialize_with = "null_to_default")]
    pub account_level: i32,
    #[serde(deserialize_with = "null_to_default")]
    pub incognito: bool,
    #[serde(deserialize_with = "null_to_default")]
    pub hide_account_level: bool,
}

// ---- Pre-Game DTOs (Allies ONLY — EnemyTeam players never exposed) ----

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawPreGamePlayerResp {
    #[serde(rename = "MatchID", deserialize_with = "null_to_default")]
    pub match_id: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawPreGameMatchResp {
    #[serde(rename = "ID", deserialize_with = "null_to_default")]
    pub id: String,
    #[serde(rename = "MapID", deserialize_with = "null_to_default")]
    pub map_id: String,
    #[serde(rename = "GamePodID", deserialize_with = "null_to_default")]
    pub game_pod_id: String,
    #[serde(rename = "Mode", deserialize_with = "null_to_default")]
    pub mode: String,
    #[serde(rename = "QueueID", deserialize_with = "null_to_default")]
    pub queue_id: String,
    #[serde(deserialize_with = "null_to_default")]
    pub pregame_state: String,
    #[serde(rename = "PhaseTimeRemainingNS", deserialize_with = "null_to_default")]
    pub phase_time_remaining_ns: i64,
    #[serde(deserialize_with = "null_to_default")]
    pub enemy_team_size: i32,
    #[serde(deserialize_with = "null_to_default")]
    pub enemy_team_lock_count: i32,
    #[serde(deserialize_with = "null_to_default")]
    pub is_ranked: bool,
    pub ally_team: Option<RawPreGameAllyTeam>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawPreGameAllyTeam {
    #[serde(rename = "TeamID", deserialize_with = "null_to_default")]
    pub team_id: String,
    #[serde(deserialize_with = "null_to_default")]
    pub players: Vec<RawPreGameAllyPlayer>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawPreGameAllyPlayer {
    #[serde(deserialize_with = "null_to_default")]
    pub subject: String,
    #[serde(rename = "CharacterID", deserialize_with = "null_to_default")]
    pub character_id: String,
    #[serde(deserialize_with = "null_to_default")]
    pub character_selection_state: String,
    #[serde(deserialize_with = "null_to_default")]
    pub competitive_tier: i32,
    #[serde(deserialize_with = "null_to_default")]
    pub player_identity: RawPlayerIdentity,
}

// ---- Core-Game & Pre-Game Loadout DTOs ----

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawCoreGamePlayerResp {
    #[serde(rename = "MatchID", deserialize_with = "null_to_default")]
    pub match_id: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawCoreGameMatchResp {
    #[serde(rename = "MatchID", deserialize_with = "null_to_default")]
    pub match_id: String,
    #[serde(rename = "MapID", deserialize_with = "null_to_default")]
    pub map_id: String,
    #[serde(rename = "GamePodID", deserialize_with = "null_to_default")]
    pub game_pod_id: String,
    #[serde(deserialize_with = "null_to_default")]
    pub provisioning_flow: String,
    pub matchmaking_data: Option<RawMatchmakingData>,
    #[serde(deserialize_with = "null_to_default")]
    pub players: Vec<RawCoreGamePlayer>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawCoreGamePlayer {
    #[serde(deserialize_with = "null_to_default")]
    pub subject: String,
    #[serde(rename = "TeamID", deserialize_with = "null_to_default")]
    pub team_id: String,
    #[serde(rename = "CharacterID", deserialize_with = "null_to_default")]
    pub character_id: String,
    #[serde(deserialize_with = "null_to_default")]
    pub player_identity: RawPlayerIdentity,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawCoreGameLoadoutsResp {
    #[serde(deserialize_with = "null_to_default")]
    pub loadouts: Vec<RawCoreGameLoadoutEntry>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawCoreGameLoadoutEntry {
    #[serde(rename = "Subject", deserialize_with = "null_to_default")]
    pub subject: String,
    #[serde(rename = "CharacterID", deserialize_with = "null_to_default")]
    pub character_id: String,
    #[serde(deserialize_with = "null_to_default")]
    pub loadout: RawCoreLoadoutDetail,
    /// Fallback when Riot returns Items directly on the loadout entry instead of nested under `Loadout`
    #[serde(deserialize_with = "null_to_default")]
    pub items: HashMap<String, RawCoreLoadoutItem>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawCoreSprays {
    #[serde(deserialize_with = "null_to_default")]
    pub spray_selections: Vec<RawCoreSpraySelection>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawCoreSpraySelection {
    #[serde(rename = "SocketID", deserialize_with = "null_to_default")]
    pub socket_id: String,
    #[serde(rename = "SprayID", deserialize_with = "null_to_default")]
    pub spray_id: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawCoreExpressions {
    #[serde(rename = "AESSelections", deserialize_with = "null_to_default")]
    pub aes_selections: Vec<RawCoreAesSelection>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawCoreAesSelection {
    #[serde(rename = "SocketID", deserialize_with = "null_to_default")]
    pub socket_id: String,
    #[serde(rename = "AssetID", deserialize_with = "null_to_default")]
    pub asset_id: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawCoreLoadoutDetail {
    #[serde(deserialize_with = "null_to_default")]
    pub subject: String,
    #[serde(deserialize_with = "null_to_default")]
    pub items: HashMap<String, RawCoreLoadoutItem>,
    #[serde(deserialize_with = "null_to_default")]
    pub sprays: RawCoreSprays,
    #[serde(deserialize_with = "null_to_default")]
    pub expressions: RawCoreExpressions,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawPreGameLoadoutsResp {
    #[serde(deserialize_with = "null_to_default")]
    pub loadouts: Vec<RawCoreLoadoutDetail>,
    #[serde(deserialize_with = "null_to_default")]
    pub loadouts_valid: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawCoreLoadoutItem {
    #[serde(rename = "ID", alias = "ItemID", deserialize_with = "null_to_default")]
    pub id: String,
    #[serde(rename = "TypeID", alias = "ItemTypeID", deserialize_with = "null_to_default")]
    pub type_id: String,
    #[serde(deserialize_with = "null_to_default")]
    pub sockets: HashMap<String, RawCoreLoadoutSocket>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawCoreLoadoutSocket {
    #[serde(rename = "ID", deserialize_with = "null_to_default")]
    pub id: String,
    #[serde(rename = "Item", deserialize_with = "null_to_default")]
    pub item: RawSocketItem,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawSocketItem {
    #[serde(rename = "ID", alias = "ItemID", deserialize_with = "null_to_default")]
    pub id: String,
    #[serde(rename = "TypeID", alias = "ItemTypeID", deserialize_with = "null_to_default")]
    pub type_id: String,
}

// ---- GLZ Requests ----

pub async fn get_coregame_player(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
) -> Result<Option<RawCoreGamePlayerResp>, MyValError> {
    let url = format!(
        "{}/core-game/v1/players/{}",
        glz_base(&session.region, &session.shard),
        session.puuid
    );
    limiter
        .request_json_opt(|| http.get(&url).headers(headers.clone()))
        .await
}

pub async fn get_coregame_match(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
    match_id: &str,
) -> Result<RawCoreGameMatchResp, MyValError> {
    let url = format!(
        "{}/core-game/v1/matches/{match_id}",
        glz_base(&session.region, &session.shard)
    );
    limiter
        .request_json(|| http.get(&url).headers(headers.clone()))
        .await
}

pub async fn get_coregame_loadouts(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
    match_id: &str,
) -> Result<Option<RawCoreGameLoadoutsResp>, MyValError> {
    let url = format!(
        "{}/core-game/v1/matches/{match_id}/loadouts",
        glz_base(&session.region, &session.shard)
    );
    limiter
        .request_json_opt(|| http.get(&url).headers(headers.clone()))
        .await
}

pub async fn get_pregame_player(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
) -> Result<Option<RawPreGamePlayerResp>, MyValError> {
    let url = format!(
        "{}/pregame/v1/players/{}",
        glz_base(&session.region, &session.shard),
        session.puuid
    );
    limiter
        .request_json_opt(|| http.get(&url).headers(headers.clone()))
        .await
}

pub async fn get_pregame_match(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
    match_id: &str,
) -> Result<RawPreGameMatchResp, MyValError> {
    let url = format!(
        "{}/pregame/v1/matches/{match_id}",
        glz_base(&session.region, &session.shard)
    );
    limiter
        .request_json(|| http.get(&url).headers(headers.clone()))
        .await
}

pub async fn get_pregame_loadouts(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
    match_id: &str,
) -> Result<Option<RawPreGameLoadoutsResp>, MyValError> {
    let url = format!(
        "{}/pregame/v1/matches/{match_id}/loadouts",
        glz_base(&session.region, &session.shard)
    );
    limiter
        .request_json_opt(|| http.get(&url).headers(headers.clone()))
        .await
}

pub async fn pregame_select_agent(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
    match_id: &str,
    agent_id: &str,
) -> Result<RawPreGameMatchResp, MyValError> {
    let url = format!(
        "{}/pregame/v1/matches/{match_id}/select/{agent_id}",
        glz_base(&session.region, &session.shard)
    );
    limiter
        .request_json(|| http.post(&url).headers(headers.clone()).body("{}"))
        .await
}

pub async fn pregame_lock_agent(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
    match_id: &str,
    agent_id: &str,
) -> Result<RawPreGameMatchResp, MyValError> {
    let url = format!(
        "{}/pregame/v1/matches/{match_id}/lock/{agent_id}",
        glz_base(&session.region, &session.shard)
    );
    limiter
        .request_json(|| http.post(&url).headers(headers.clone()).body("{}"))
        .await
}

pub async fn pregame_quit_match(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
    match_id: &str,
) -> Result<(), MyValError> {
    let url = format!(
        "{}/pregame/v1/matches/{match_id}/quit",
        glz_base(&session.region, &session.shard)
    );
    limiter
        .request_empty(|| http.post(&url).headers(headers.clone()).body("{}"))
        .await
}

pub async fn get_party_player(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
) -> Result<Option<RawPartyPlayerResp>, MyValError> {
    let url = format!(
        "{}/parties/v1/players/{}",
        glz_base(&session.region, &session.shard),
        session.puuid
    );
    limiter
        .request_json_opt(|| http.get(&url).headers(headers.clone()))
        .await
}

pub async fn get_party(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
    party_id: &str,
) -> Result<RawPartyResp, MyValError> {
    let url = format!(
        "{}/parties/v1/parties/{party_id}",
        glz_base(&session.region, &session.shard)
    );
    limiter
        .request_json(|| http.get(&url).headers(headers.clone()))
        .await
}

pub async fn party_set_ready(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
    party_id: &str,
    ready: bool,
) -> Result<RawPartyResp, MyValError> {
    let url = format!(
        "{}/parties/v1/parties/{party_id}/members/{}/setReady",
        glz_base(&session.region, &session.shard),
        session.puuid
    );
    limiter
        .request_json(|| {
            http.post(&url)
                .headers(headers.clone())
                .json(&serde_json::json!({ "ready": ready }))
        })
        .await
}

pub async fn party_join_queue(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
    party_id: &str,
) -> Result<RawPartyResp, MyValError> {
    let url = format!(
        "{}/parties/v1/parties/{party_id}/matchmaking/join",
        glz_base(&session.region, &session.shard)
    );
    limiter
        .request_json(|| http.post(&url).headers(headers.clone()).body("{}"))
        .await
}

pub async fn party_leave_queue(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
    party_id: &str,
) -> Result<RawPartyResp, MyValError> {
    let url = format!(
        "{}/parties/v1/parties/{party_id}/matchmaking/leave",
        glz_base(&session.region, &session.shard)
    );
    limiter
        .request_json(|| http.post(&url).headers(headers.clone()).body("{}"))
        .await
}

pub async fn party_change_queue(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
    party_id: &str,
    queue_id: &str,
) -> Result<RawPartyResp, MyValError> {
    let url = format!(
        "{}/parties/v1/parties/{party_id}/queue",
        glz_base(&session.region, &session.shard)
    );
    limiter
        .request_json(|| {
            http.post(&url)
                .headers(headers.clone())
                .json(&serde_json::json!({ "queueID": queue_id }))
        })
        .await
}

pub async fn party_set_accessibility(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
    party_id: &str,
    open: bool,
) -> Result<RawPartyResp, MyValError> {
    let acc = if open { "OPEN" } else { "CLOSED" };
    let url = format!(
        "{}/parties/v1/parties/{party_id}/accessibility",
        glz_base(&session.region, &session.shard)
    );
    limiter
        .request_json(|| {
            http.post(&url)
                .headers(headers.clone())
                .json(&serde_json::json!({ "accessibility": acc }))
        })
        .await
}

pub async fn party_generate_invite_code(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
    party_id: &str,
) -> Result<RawPartyResp, MyValError> {
    let url = format!(
        "{}/parties/v1/parties/{party_id}/invitecode",
        glz_base(&session.region, &session.shard)
    );
    limiter
        .request_json(|| http.post(&url).headers(headers.clone()).body("{}"))
        .await
}

pub async fn party_disable_invite_code(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
    party_id: &str,
) -> Result<RawPartyResp, MyValError> {
    let url = format!(
        "{}/parties/v1/parties/{party_id}/invitecode",
        glz_base(&session.region, &session.shard)
    );
    limiter
        .request_json(|| http.delete(&url).headers(headers.clone()))
        .await
}

pub async fn party_join_by_code(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
    invite_code: &str,
) -> Result<RawPartyResp, MyValError> {
    let clean = invite_code.trim();
    if clean.is_empty() {
        return Err(MyValError::InvalidInput("Invite code cannot be empty".to_string()));
    }
    let url = format!(
        "{}/parties/v1/parties/joincode/{clean}",
        glz_base(&session.region, &session.shard)
    );
    limiter
        .request_json(|| http.post(&url).headers(headers.clone()).body("{}"))
        .await
}

pub async fn party_refresh_player_identity(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
    party_id: &str,
) -> Result<RawPartyResp, MyValError> {
    let url = format!(
        "{}/parties/v1/parties/{party_id}/members/{}/refreshPlayerIdentity",
        glz_base(&session.region, &session.shard),
        session.puuid
    );
    limiter
        .request_json(|| http.post(&url).headers(headers.clone()).body("{}"))
        .await
}
