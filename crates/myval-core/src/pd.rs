use std::collections::{HashMap, HashSet};

use chrono::Utc;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Deserializer, Serialize};

fn null_to_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    let opt = Option::<T>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

use crate::cache::{
    LocalCache, MATCH_HISTORY_TTL_SECS, MMR_CACHE_TTL_SECS,
};
use crate::error::MyValError;
use crate::models::assets::SeasonInfo;
use crate::models::auth::RiotSession;
use crate::models::mmr::{PlayerMmrSummary, RankInfo};
use crate::models::store::{
    AccessoryOffer, BundleItem, DailyOffer, FeaturedBundle, NightMarketOffer, NightMarketSection,
    StorefrontSnapshot, Wallet,
};
use crate::rate_limit::RateLimiter;

pub const CURRENCY_VP: &str = "85ad13f7-3d1b-5128-9eb2-7cd8ee0b5741";
pub const CURRENCY_RADIANITE: &str = "e59aa87c-4cbf-517a-5983-6e81511be9b7";
pub const CURRENCY_KC: &str = "85ca954a-41f2-ce94-9b45-8ca3dd39a00d";

pub const ITEM_TYPE_AGENT: &str = "01bb38e1-da47-4e6a-9b3d-945fe4655707";
pub const ITEM_TYPE_SKIN_LEVEL: &str = "e7c63390-eda7-46e0-bb7a-a6abdacd2433";
pub const ITEM_TYPE_SKIN_CHROMA: &str = "3ad1b2b2-acdb-4524-852f-954a76ddae0a";
pub const ITEM_TYPE_BUDDY: &str = "dd3bf334-87f3-40bd-b043-682a57a8dc3a";
pub const ITEM_TYPE_SPRAY: &str = "d5f120f8-ff8c-4aac-92ea-f2b5acbe9475";
pub const ITEM_TYPE_PLAYER_CARD: &str = "3f296c07-64c3-494c-923b-fe692a4fa1bd";
pub const ITEM_TYPE_TITLE: &str = "de7caa6b-adf7-4588-bbd1-143831e786c6";
pub const ITEM_TYPE_FLEX: &str = "03a572de-4234-31ed-d344-ababa488f981";

pub const STARTER_AGENT_UUIDS: [&str; 5] = [
    "9f0d8ba9-4140-b941-57d3-a7ad57c6b417", // Brimstone
    "add6443a-41bd-e414-f6ad-e58d267f4e95", // Jett
    "eb93336a-449b-9c1b-0a54-a891f7921d69", // Phoenix
    "569fdd95-4d10-43ab-ca70-79becc718b46", // Sage
    "320b2a48-4d9b-a075-30f1-1f93a9b638fa", // Sova
];

pub fn pd_base(shard: &str) -> String {
    format!("https://pd.{shard}.a.pvp.net")
}

// ---- Raw Riot PD DTOs ----

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawStorefrontResponse {
    pub featured_bundle: Option<RawFeaturedBundleSection>,
    pub skins_panel_layout: Option<RawSkinsPanelLayout>,
    pub bonus_store: Option<RawBonusStore>,
    pub accessory_store: Option<RawAccessoryStore>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawFeaturedBundleSection {
    pub bundle: Option<RawBundle>,
    pub bundles: Vec<RawBundle>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawBundle {
    #[serde(rename = "DataAssetID")]
    pub data_asset_id: String,
    pub duration_remaining_in_seconds: i64,
    pub total_base_cost: Option<HashMap<String, i64>>,
    pub total_discounted_cost: Option<HashMap<String, i64>>,
    pub items: Vec<RawBundleItemEntry>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawBundleItemEntry {
    pub item: RawBundleInnerItem,
    pub base_price: i64,
    pub discounted_price: i64,
    pub discount_percent: f64,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawBundleInnerItem {
    #[serde(rename = "ItemTypeID")]
    pub item_type_id: String,
    #[serde(rename = "ItemID")]
    pub item_id: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawSkinsPanelLayout {
    pub single_item_offers: Vec<String>,
    pub single_item_store_offers: Vec<RawStoreOffer>,
    pub single_item_offers_remaining_duration_in_seconds: i64,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawStoreOffer {
    #[serde(rename = "OfferID")]
    pub offer_id: String,
    pub cost: HashMap<String, i64>,
    pub rewards: Vec<RawOfferReward>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawOfferReward {
    #[serde(rename = "ItemTypeID")]
    pub item_type_id: String,
    #[serde(rename = "ItemID")]
    pub item_id: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawBonusStore {
    pub bonus_store_offers: Vec<RawBonusStoreOffer>,
    pub bonus_store_remaining_duration_in_seconds: i64,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawBonusStoreOffer {
    #[serde(rename = "BonusOfferID")]
    pub bonus_offer_id: String,
    pub offer: RawStoreOffer,
    pub discount_percent: i32,
    pub discount_costs: HashMap<String, i64>,
    pub is_seen: bool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawAccessoryStore {
    pub accessory_store_offers: Vec<RawAccessoryStoreOffer>,
    pub accessory_store_remaining_duration_in_seconds: i64,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawAccessoryStoreOffer {
    pub offer: RawStoreOffer,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawWalletResponse {
    pub balances: HashMap<String, i64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawMmrResponse {
    pub latest_competitive_update: Option<RawCompetitiveUpdate>,
    pub queue_skills: HashMap<String, RawQueueSkill>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawCompetitiveUpdate {
    #[serde(rename = "MatchID", deserialize_with = "null_to_default")]
    pub match_id: String,
    pub tier_after_update: i32,
    pub tier_before_update: i32,
    pub ranked_rating_after_update: i32,
    pub ranked_rating_before_update: i32,
    pub ranked_rating_earned: i32,
    pub ranked_rating_performance_bonus: i32,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawCompUpdatesLightResponse {
    #[serde(deserialize_with = "null_to_default")]
    pub matches: Vec<RawCompetitiveUpdate>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawQueueSkill {
    #[serde(rename = "SeasonalInfoBySeasonID")]
    pub seasonal_info_by_season_id: Option<HashMap<String, RawSeasonInfo>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawSeasonInfo {
    #[serde(rename = "SeasonID")]
    pub season_id: String,
    pub competitive_tier: i32,
    pub ranked_rating: i32,
    pub leaderboard_rank: u32,
    pub number_of_games: u32,
    pub wins_by_tier: Option<HashMap<String, u32>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawMatchHistoryResponse {
    pub history: Vec<RawMatchHistoryEntry>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawMatchHistoryEntry {
    #[serde(rename = "MatchID")]
    pub match_id: String,
    pub game_start_time: i64,
    #[serde(rename = "QueueID")]
    pub queue_id: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawNameEntry {
    #[serde(deserialize_with = "null_to_default")]
    pub display_name: String,
    #[serde(deserialize_with = "null_to_default")]
    pub subject: String,
    #[serde(deserialize_with = "null_to_default")]
    pub game_name: String,
    #[serde(deserialize_with = "null_to_default")]
    pub tag_line: String,
}

// ---- Network & Enrichment Functions ----

pub async fn fetch_storefront_raw(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
) -> Result<RawStorefrontResponse, MyValError> {
    let url = format!("{}/store/v3/storefront/{}", pd_base(&session.shard), session.puuid);
    limiter
        .request_json(|| http.post(&url).headers(headers.clone()).body("{}"))
        .await
}

pub async fn fetch_wallet(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
) -> Result<Wallet, MyValError> {
    let url = format!("{}/store/v1/wallet/{}", pd_base(&session.shard), session.puuid);
    let raw: RawWalletResponse = limiter
        .request_json(|| http.get(&url).headers(headers.clone()))
        .await?;
    Ok(Wallet {
        vp: *raw.balances.get(CURRENCY_VP).unwrap_or(&0),
        radianite: *raw.balances.get(CURRENCY_RADIANITE).unwrap_or(&0),
        kingdom_credits: *raw.balances.get(CURRENCY_KC).unwrap_or(&0),
    })
}

pub fn build_rank_info(cache: &LocalCache, tier: i32, rr: i32) -> RankInfo {
    if tier < 3 {
        return RankInfo {
            tier: 0,
            tier_name: "Unranked".to_string(),
            division_name: "Unranked".to_string(),
            rr: 0,
            icon_url: cache.lookup_competitive_tier(0).and_then(|c| c.large_icon),
            color_hex: "#9CA3AF".to_string(),
        };
    }
    if let Some(meta) = cache.lookup_competitive_tier(tier) {
        RankInfo {
            tier,
            tier_name: meta.tier_name,
            division_name: meta.division_name,
            rr,
            icon_url: meta.large_icon,
            color_hex: meta.color,
        }
    } else {
        RankInfo {
            tier,
            tier_name: format!("Tier {tier}"),
            division_name: "Ranked".to_string(),
            rr,
            icon_url: None,
            color_hex: "#5A9FE2".to_string(),
        }
    }
}

fn season_best_tier(info: &RawSeasonInfo) -> i32 {
    let mut best = if info.competitive_tier >= 3 {
        info.competitive_tier
    } else {
        0
    };
    if let Some(wins) = &info.wins_by_tier {
        for (t_str, count) in wins {
            if *count > 0 {
                if let Ok(t) = t_str.parse::<i32>() {
                    if t >= 3 && t > best {
                        best = t;
                    }
                }
            }
        }
    }
    best
}

fn format_season_label(season: &SeasonInfo) -> String {
    season
        .title
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| season.display_name.trim().to_string())
}

pub fn resolve_current_act_info(cache: &LocalCache) -> (String, String) {
    let now = Utc::now();
    let now_iso = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let mut acts: Vec<SeasonInfo> = cache
        .list_category::<SeasonInfo>("season")
        .unwrap_or_default();
    if acts.is_empty() {
        return (String::new(), String::new());
    }
    acts.sort_by(|a, b| b.start_time.cmp(&a.start_time));
    let current = acts
        .iter()
        .find(|s| !s.start_time.is_empty() && !s.end_time.is_empty() && s.start_time <= now_iso && s.end_time >= now_iso)
        .or_else(|| acts.iter().find(|s| !s.start_time.is_empty() && s.start_time <= now_iso))
        .or_else(|| acts.first());
    let Some(cur) = current else {
        return (String::new(), String::new());
    };
    let act_name = format_season_label(cur);
    let end_label = chrono::DateTime::parse_from_rfc3339(&cur.end_time)
        .map(|dt| {
            let end_utc = dt.with_timezone(&Utc);
            let date_str = end_utc.format("%d %b %Y");
            if end_utc > now {
                let days = end_utc.signed_duration_since(now).num_days().max(0);
                format!("Ends {date_str} ({days}d left)")
            } else {
                format!("Ended {date_str}")
            }
        })
        .unwrap_or_default();
    (act_name, end_label)
}

pub fn summarize_mmr(cache: &LocalCache, puuid: &str, raw: &RawMmrResponse) -> PlayerMmrSummary {
    let latest_tier = raw
        .latest_competitive_update
        .as_ref()
        .map(|u| u.tier_after_update)
        .unwrap_or(0);
    let latest_rr = raw
        .latest_competitive_update
        .as_ref()
        .map(|u| u.ranked_rating_after_update)
        .unwrap_or(0);

    let mut peak_tier = if latest_tier >= 3 { latest_tier } else { 0 };
    let mut prev_tier = 0;
    let mut peak_rank_act_name = String::new();
    let mut leaderboard_rank = None;

    if let Some(comp) = raw.queue_skills.get("competitive") {
        if let Some(seasons) = &comp.seasonal_info_by_season_id {
            for info in seasons.values() {
                if info.competitive_tier >= 3 && info.competitive_tier > peak_tier {
                    peak_tier = info.competitive_tier;
                }
                if let Some(wins) = &info.wins_by_tier {
                    for (t_str, count) in wins {
                        if *count > 0 {
                            if let Ok(t) = t_str.parse::<i32>() {
                                if t >= 3 && t > peak_tier {
                                    peak_tier = t;
                                }
                            }
                        }
                    }
                }
                if info.leaderboard_rank > 0 {
                    leaderboard_rank = Some(info.leaderboard_rank);
                }
            }

            let now_iso = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
            let mut cached_acts: Vec<SeasonInfo> = cache
                .list_category::<SeasonInfo>("season")
                .unwrap_or_default()
                .into_iter()
                .filter(|s| s.start_time <= now_iso)
                .collect();
            cached_acts.sort_by(|a, b| b.start_time.cmp(&a.start_time));

            let current_act_uuid = cached_acts
                .iter()
                .find(|s| s.start_time <= now_iso && s.end_time >= now_iso)
                .or_else(|| {
                    cached_acts
                        .iter()
                        .find(|s| seasons.contains_key(&s.uuid) || seasons.contains_key(&s.uuid.to_lowercase()))
                })
                .map(|s| s.uuid.to_lowercase());

            if peak_tier >= 3 {
                for act in &cached_acts {
                    let info_opt = seasons.get(&act.uuid).or_else(|| {
                        seasons
                            .iter()
                            .find(|(k, _)| k.eq_ignore_ascii_case(&act.uuid))
                            .map(|(_, v)| v)
                    });
                    if let Some(info) = info_opt {
                        if season_best_tier(info) == peak_tier {
                            peak_rank_act_name = format_season_label(act);
                            break;
                        }
                    }
                }
                if peak_rank_act_name.is_empty() && peak_tier == latest_tier {
                    if let Some(cur_uuid) = current_act_uuid.as_deref() {
                        if let Some(act) = cached_acts.iter().find(|a| a.uuid.eq_ignore_ascii_case(cur_uuid)) {
                            peak_rank_act_name = format_season_label(act);
                        }
                    }
                }
            }

            for act in &cached_acts {
                if current_act_uuid.as_deref() == Some(&act.uuid.to_lowercase()) {
                    continue;
                }
                let info_opt = seasons
                    .get(&act.uuid)
                    .or_else(|| {
                        seasons
                            .iter()
                            .find(|(k, _)| k.eq_ignore_ascii_case(&act.uuid))
                            .map(|(_, v)| v)
                    });
                if let Some(info) = info_opt {
                    let act_best = season_best_tier(info);
                    if act_best >= 3 {
                        prev_tier = act_best;
                        break;
                    }
                }
            }

            if prev_tier == 0 {
                let mut sorted_keys: Vec<&String> = seasons.keys().collect();
                sorted_keys.sort();
                for k in sorted_keys {
                    if current_act_uuid.as_deref() == Some(&k.to_lowercase()) {
                        continue;
                    }
                    if let Some(info) = seasons.get(k) {
                        let t = season_best_tier(info);
                        if t >= 3 && t != latest_tier {
                            prev_tier = t;
                            break;
                        }
                    }
                }
            }
        }
    }

    PlayerMmrSummary {
        puuid: puuid.to_string(),
        current_rank: build_rank_info(cache, latest_tier, latest_rr),
        peak_rank: build_rank_info(cache, peak_tier, 0),
        peak_rank_act_name,
        previous_act_rank: build_rank_info(cache, prev_tier, 0),
        leaderboard_rank,
        recent_results: Vec::new(),
        recent_outcomes: Vec::new(),
        recent_results_loaded: false,
    }
}

/// Converts Riot `competitiveupdates` entries (newest-first) into signed outcomes:
/// `1` = Win, `-1` = Loss, `0` = Draw. Skips unranked/placement placeholder entries.
pub fn extract_recent_comp_outcomes(
    entries: &[RawCompetitiveUpdate],
    max_count: usize,
) -> Vec<i8> {
    let mut outcomes = Vec::with_capacity(max_count);
    for entry in entries {
        if entry.tier_after_update < 3
            && entry.tier_before_update < 3
            && entry.ranked_rating_earned == 0
        {
            continue;
        }
        let outcome = if entry.ranked_rating_earned > 0 {
            1i8
        } else if entry.ranked_rating_earned < 0 {
            -1i8
        } else if entry.tier_after_update > entry.tier_before_update && entry.tier_before_update >= 3 {
            1i8
        } else if entry.tier_after_update < entry.tier_before_update
            && entry.tier_before_update >= 3
            && entry.tier_after_update >= 3
        {
            -1i8
        } else {
            0i8
        };
        outcomes.push(outcome);
        if outcomes.len() >= max_count {
            break;
        }
    }
    outcomes
}

pub fn summarize_mmr_with_outcomes(
    cache: &LocalCache,
    puuid: &str,
    raw: &RawMmrResponse,
    recent_outcomes: Vec<i8>,
    recent_results_loaded: bool,
) -> PlayerMmrSummary {
    let mut summary = summarize_mmr(cache, puuid, raw);
    summary.recent_results = recent_outcomes.iter().map(|&o| o > 0).collect();
    summary.recent_outcomes = recent_outcomes;
    summary.recent_results_loaded = recent_results_loaded;
    summary
}

pub async fn fetch_player_mmr(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    cache: &LocalCache,
    shard: &str,
    headers: &HeaderMap,
    puuid: &str,
) -> PlayerMmrSummary {
    if let Some(cached) = cache.get_cached_mmr(puuid, MMR_CACHE_TTL_SECS) {
        return cached;
    }

    let mmr_url = format!("{}/mmr/v1/players/{puuid}", pd_base(shard));
    let comp_url = format!(
        "{}/mmr/v1/players/{puuid}/competitiveupdates?startIndex=0&endIndex=8&queue=competitive",
        pd_base(shard)
    );

    let (raw_res, comp_res) = tokio::join!(
        limiter.request_json_opt::<RawMmrResponse, _>(|| http.get(&mmr_url).headers(headers.clone())),
        limiter.request_json_opt::<RawCompUpdatesLightResponse, _>(|| http.get(&comp_url).headers(headers.clone())),
    );
    let raw_opt = raw_res.ok().flatten();
    let comp_opt = comp_res.ok().flatten();

    let (recent_outcomes, recent_loaded) = match comp_opt {
        Some(resp) => (extract_recent_comp_outcomes(&resp.matches, 5), true),
        None => (Vec::new(), false),
    };

    match raw_opt {
        Some(raw) => {
            let s = summarize_mmr_with_outcomes(cache, puuid, &raw, recent_outcomes, recent_loaded);
            if recent_loaded {
                let _ = cache.put_cached_mmr(&s);
            }
            s
        }
        None => {
            let recent_results = recent_outcomes.iter().map(|&o| o > 0).collect();
            PlayerMmrSummary {
                puuid: puuid.to_string(),
                current_rank: build_rank_info(cache, 0, 0),
                peak_rank: build_rank_info(cache, 0, 0),
                peak_rank_act_name: String::new(),
                previous_act_rank: build_rank_info(cache, 0, 0),
                leaderboard_rank: None,
                recent_results,
                recent_outcomes,
                recent_results_loaded: false,
            }
        }
    }
}

pub async fn fetch_recent_match_ids(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    cache: &LocalCache,
    shard: &str,
    headers: &HeaderMap,
    puuid: &str,
    count: usize,
) -> Vec<String> {
    if let Some(cached) = cache.get_cached_match_history(puuid, MATCH_HISTORY_TTL_SECS) {
        return cached;
    }
    let url = format!(
        "{}/match-history/v1/history/{puuid}?startIndex=0&endIndex={count}",
        pd_base(shard)
    );
    let raw_opt: Option<RawMatchHistoryResponse> = limiter
        .request_json_opt(|| http.get(&url).headers(headers.clone()))
        .await
        .ok()
        .flatten();

    let ids: Vec<String> = raw_opt
        .map(|r| {
            r.history
                .into_iter()
                .filter(|e| {
                    let q = e.queue_id.trim();
                    !q.eq_ignore_ascii_case("deathmatch") && !q.eq_ignore_ascii_case("custom")
                })
                .map(|e| e.match_id)
                .collect()
        })
        .unwrap_or_default();
    if !ids.is_empty() {
        let _ = cache.put_cached_match_history(puuid, &ids);
    }
    ids
}

fn parse_name_entry(e: RawNameEntry) -> Option<(String, String, String)> {
    let key = e.subject.trim().to_lowercase();
    if key.is_empty() {
        return None;
    }
    let gn = e.game_name.trim();
    let tl = e.tag_line.trim();
    if !gn.is_empty() {
        return Some((key, gn.to_string(), tl.to_string()));
    }
    let dn = e.display_name.trim();
    if !dn.is_empty() {
        if let Some((left, right)) = dn.split_once('#') {
            if !left.trim().is_empty() {
                return Some((key, left.trim().to_string(), right.trim().to_string()));
            }
        }
        return Some((key, dn.to_string(), String::new()));
    }
    if !tl.is_empty() {
        return Some((key, format!("#{tl}"), String::new()));
    }
    None
}

pub fn fallback_name_service_shard(primary_shard: &str) -> &'static str {
    if primary_shard.trim().eq_ignore_ascii_case("na") {
        "eu"
    } else {
        "na"
    }
}

pub async fn fetch_names_batched(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    cache: &LocalCache,
    shard: &str,
    headers: &HeaderMap,
    puuids: &[String],
) -> HashMap<String, (String, String)> {
    let mut result = cache.get_cached_names(puuids);
    let mut seen_missing = HashSet::new();
    let missing: Vec<String> = puuids
        .iter()
        .filter(|p| {
            let low = p.trim().to_lowercase();
            !low.is_empty() && !result.contains_key(&low) && seen_missing.insert(low)
        })
        .cloned()
        .collect();
    if missing.is_empty() {
        return result;
    }

    let primary_url = format!("{}/name-service/v2/players", pd_base(shard));
    let mut to_cache: Vec<(String, String, String)> = Vec::new();
    if let Ok(Some(entries)) = limiter
        .request_json_opt::<Vec<RawNameEntry>, _>(|| {
            http.put(&primary_url).headers(headers.clone()).json(&missing)
        })
        .await
    {
        for e in entries {
            if let Some((key, gn, tl)) = parse_name_entry(e) {
                result.insert(key.clone(), (gn.clone(), tl.clone()));
                to_cache.push((key, gn, tl));
            }
        }
    }

    // If the home shard returned blank GameName/TagLine for any player, query an alternate PD shard in a single batch.
    let still_missing: Vec<String> = missing
        .into_iter()
        .filter(|p| !result.contains_key(&p.trim().to_lowercase()))
        .collect();
    if !still_missing.is_empty() {
        let alt_url = format!(
            "{}/name-service/v2/players",
            pd_base(fallback_name_service_shard(shard))
        );
        if let Ok(Some(entries)) = limiter
            .request_json_opt::<Vec<RawNameEntry>, _>(|| {
                http.put(&alt_url).headers(headers.clone()).json(&still_missing)
            })
            .await
        {
            for e in entries {
                if let Some((key, gn, tl)) = parse_name_entry(e) {
                    result.insert(key.clone(), (gn.clone(), tl.clone()));
                    to_cache.push((key, gn, tl));
                }
            }
        }
    }

    if !to_cache.is_empty() {
        let _ = cache.put_cached_names(&to_cache);
    }
    result
}

pub fn resolve_item_metadata(
    cache: &LocalCache,
    item_type_id: &str,
    item_id: &str,
) -> (String, String, Option<String>) {
    match item_type_id {
        ITEM_TYPE_SKIN_LEVEL => {
            if let Ok(Some(skin)) = cache.lookup_skin_by_any_uuid(item_id) {
                return ("Skin".to_string(), skin.display_name, skin.display_icon);
            }
            ("Skin".to_string(), "Weapon Skin".to_string(), None)
        }
        ITEM_TYPE_PLAYER_CARD => {
            if let Some(c) = cache.lookup_player_card(item_id) {
                return (
                    "Player Card".to_string(),
                    c.display_name,
                    c.large_art.or(c.wide_art).or(c.small_art),
                );
            }
            ("Player Card".to_string(), "Player Card".to_string(), None)
        }
        ITEM_TYPE_SPRAY => {
            if let Some(s) = cache.lookup_spray(item_id) {
                return ("Spray".to_string(), s.display_name, s.display_icon);
            }
            ("Spray".to_string(), "Spray".to_string(), None)
        }
        ITEM_TYPE_BUDDY => {
            if let Some(b) = cache.lookup_buddy(item_id) {
                return ("Buddy".to_string(), b.display_name, b.display_icon);
            }
            ("Buddy".to_string(), "Gun Buddy".to_string(), None)
        }
        ITEM_TYPE_TITLE => {
            if let Some(t) = cache.lookup_title(item_id) {
                let name = if !t.title_text.is_empty() {
                    t.title_text
                } else {
                    t.display_name
                };
                return ("Title".to_string(), name, None);
            }
            ("Title".to_string(), "Player Title".to_string(), None)
        }
        _ => ("Item".to_string(), "Store Item".to_string(), None),
    }
}

pub fn enrich_storefront(
    cache: &LocalCache,
    raw: &RawStorefrontResponse,
    wallet: Wallet,
) -> StorefrontSnapshot {
    let wishlist: HashSet<String> = cache.get_wishlist_set().unwrap_or_default();
    let mut daily_offers = Vec::new();
    let mut daily_remaining_secs = 0;

    if let Some(panel) = &raw.skins_panel_layout {
        daily_remaining_secs = panel.single_item_offers_remaining_duration_in_seconds;
        if !panel.single_item_store_offers.is_empty() {
            for offer in &panel.single_item_store_offers {
                let item_id = offer
                    .rewards
                    .first()
                    .map(|r| r.item_id.as_str())
                    .unwrap_or(&offer.offer_id);
                let cost_vp = *offer.cost.get(CURRENCY_VP).unwrap_or(&0);
                daily_offers.push(build_daily_offer(cache, &wishlist, &offer.offer_id, item_id, cost_vp));
            }
        } else {
            for item_id in &panel.single_item_offers {
                daily_offers.push(build_daily_offer(cache, &wishlist, item_id, item_id, 0));
            }
        }
    }

    let mut featured_bundles = Vec::new();
    if let Some(fb) = &raw.featured_bundle {
        let bundles_source: Vec<&RawBundle> = if !fb.bundles.is_empty() {
            fb.bundles.iter().collect()
        } else if let Some(single) = &fb.bundle {
            vec![single]
        } else {
            Vec::new()
        };
        for b in bundles_source {
            let asset = cache.lookup_bundle(&b.data_asset_id);
            let items = b
                .items
                .iter()
                .map(|entry| {
                    let (item_type, name, icon) = resolve_item_metadata(
                        cache,
                        &entry.item.item_type_id,
                        &entry.item.item_id,
                    );
                    BundleItem {
                        item_type,
                        item_uuid: entry.item.item_id.clone(),
                        name,
                        display_icon: icon,
                        base_price_vp: entry.base_price,
                        discounted_price_vp: entry.discounted_price,
                        discount_percent: (entry.discount_percent * 100.0).round() as i32,
                    }
                })
                .collect::<Vec<_>>();
            let total_base = b
                .total_base_cost
                .as_ref()
                .and_then(|m| m.get(CURRENCY_VP))
                .copied()
                .unwrap_or_else(|| items.iter().map(|i| i.base_price_vp).sum());
            let total_disc = b
                .total_discounted_cost
                .as_ref()
                .and_then(|m| m.get(CURRENCY_VP))
                .copied()
                .unwrap_or_else(|| items.iter().map(|i| i.discounted_price_vp).sum());
            featured_bundles.push(FeaturedBundle {
                bundle_uuid: b.data_asset_id.clone(),
                name: asset
                    .as_ref()
                    .map(|a| a.display_name.clone())
                    .unwrap_or_else(|| "Featured Bundle".to_string()),
                description: asset.as_ref().map(|a| a.description.clone()).unwrap_or_default(),
                display_icon: asset.and_then(|a| a.display_icon.or(a.vertical_promo_image)),
                remaining_secs: b.duration_remaining_in_seconds,
                total_base_price_vp: total_base,
                total_discounted_price_vp: total_disc,
                items,
            });
        }
    }

    let night_market = raw.bonus_store.as_ref().and_then(|bs| {
        if bs.bonus_store_offers.is_empty() {
            return None;
        }
        let offers = bs
            .bonus_store_offers
            .iter()
            .map(|o| {
                let item_id = o
                    .offer
                    .rewards
                    .first()
                    .map(|r| r.item_id.as_str())
                    .unwrap_or(&o.bonus_offer_id);
                let base_price = *o.offer.cost.get(CURRENCY_VP).unwrap_or(&0);
                let discount_price = *o.discount_costs.get(CURRENCY_VP).unwrap_or(&base_price);
                let skin = cache.lookup_skin_by_any_uuid(item_id).ok().flatten();
                let skin_uuid = skin
                    .as_ref()
                    .map(|s| s.uuid.clone())
                    .unwrap_or_else(|| item_id.to_string());
                NightMarketOffer {
                    offer_id: o.bonus_offer_id.clone(),
                    skin_uuid: skin_uuid.clone(),
                    skin_name: skin
                        .as_ref()
                        .map(|s| s.display_name.clone())
                        .unwrap_or_else(|| "Night Market Skin".to_string()),
                    weapon_name: skin
                        .as_ref()
                        .map(|s| s.weapon_name.clone())
                        .unwrap_or_default(),
                    display_icon: skin.as_ref().and_then(|s| s.display_icon.clone()),
                    tier_color: skin
                        .as_ref()
                        .map(|s| s.content_tier_color.clone())
                        .unwrap_or_else(|| "#E95F8E".to_string()),
                    base_price_vp: base_price,
                    discount_percent: o.discount_percent,
                    discount_price_vp: discount_price,
                    is_seen: o.is_seen,
                    is_wishlisted: wishlist.contains(&skin_uuid.to_lowercase()),
                }
            })
            .collect();
        Some(NightMarketSection {
            remaining_secs: bs.bonus_store_remaining_duration_in_seconds,
            offers,
        })
    });

    let mut accessory_remaining_secs = 0;
    let mut accessory_offers = Vec::new();
    if let Some(acc) = &raw.accessory_store {
        accessory_remaining_secs = acc.accessory_store_remaining_duration_in_seconds;
        for entry in &acc.accessory_store_offers {
            let reward = entry.offer.rewards.first();
            let item_type_id = reward.map(|r| r.item_type_id.as_str()).unwrap_or("");
            let item_id = reward
                .map(|r| r.item_id.as_str())
                .unwrap_or(&entry.offer.offer_id);
            let (item_type, name, display_icon) =
                resolve_item_metadata(cache, item_type_id, item_id);
            let cost_kc = *entry.offer.cost.get(CURRENCY_KC).unwrap_or(&0);
            accessory_offers.push(AccessoryOffer {
                offer_id: entry.offer.offer_id.clone(),
                item_type,
                item_uuid: item_id.to_string(),
                name,
                display_icon,
                cost_kc,
            });
        }
    }

    StorefrontSnapshot {
        fetched_at_epoch_secs: LocalCache::now_secs(),
        daily_remaining_secs,
        daily_offers,
        featured_bundles,
        night_market,
        accessory_remaining_secs,
        accessory_offers,
        wallet,
    }
}

fn build_daily_offer(
    cache: &LocalCache,
    wishlist: &HashSet<String>,
    offer_id: &str,
    level_or_skin_id: &str,
    cost_vp: i64,
) -> DailyOffer {
    let skin = cache.lookup_skin_by_any_uuid(level_or_skin_id).ok().flatten();
    if let Some(s) = skin {
        let video = s
            .levels
            .iter()
            .rev()
            .find_map(|l| l.streamed_video.clone())
            .or_else(|| s.chromas.iter().find_map(|c| c.streamed_video.clone()));
        DailyOffer {
            offer_id: offer_id.to_string(),
            skin_uuid: s.uuid.clone(),
            skin_name: s.display_name,
            weapon_name: s.weapon_name,
            display_icon: s.display_icon,
            tier_name: s.content_tier_name,
            tier_color: s.content_tier_color,
            cost_vp,
            preview_video_url: video,
            chromas: s.chromas,
            is_wishlisted: wishlist.contains(&s.uuid.to_lowercase()),
        }
    } else {
        DailyOffer {
            offer_id: offer_id.to_string(),
            skin_uuid: level_or_skin_id.to_string(),
            skin_name: "Unknown Skin".to_string(),
            weapon_name: String::new(),
            display_icon: None,
            tier_name: None,
            tier_color: "#5A9FE2".to_string(),
            cost_vp,
            preview_video_url: None,
            chromas: Vec::new(),
            is_wishlisted: wishlist.contains(&level_or_skin_id.to_lowercase()),
        }
    }
}
