use std::collections::HashMap;

use reqwest::header::HeaderMap;
use serde::Deserialize;
use url::Url;

use crate::auth::region_to_shard;
use crate::cache::LocalCache;
use crate::error::MyValError;
use crate::models::assets::MapInfo;
use crate::models::auth::RiotSession;
use crate::models::career::{
    AccountPenalty, AgentPerformanceStat, CareerSnapshot, CompetitiveMatchUpdate,
    ContractProgressSnapshot, MatchDetailSnapshot, MatchHistoryItem, MissionProgress,
    PlayerProfileSnapshot, ReplayPlayerSnapshot, RoundKillEvent, RoundReplayData, ScoreboardPlayer,
};
use crate::pd::{
    build_rank_info, fetch_names_batched, fetch_player_mmr, fetch_recent_match_ids, pd_base,
    resolve_current_act_info,
};
use crate::rate_limit::RateLimiter;

pub const PLAYER_PROFILE_CACHE_TTL_SECS: i64 = 5 * 60;

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
struct RawCompUpdatesResp {
    pub matches: Vec<RawCompUpdateEntry>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
struct RawCompUpdateEntry {
    #[serde(rename = "MatchID")]
    pub match_id: String,
    #[serde(rename = "MapID")]
    pub map_id: String,
    pub match_start_time: i64,
    pub tier_after_update: i32,
    pub ranked_rating_after_update: i32,
    pub ranked_rating_earned: i32,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawMatchDetailResp {
    pub match_info: RawMatchInfo,
    pub players: Vec<RawMatchPlayer>,
    pub teams: Vec<RawMatchTeam>,
    pub round_results: Vec<RawRoundResult>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawMatchInfo {
    pub match_id: String,
    pub map_id: String,
    pub game_length_millis: i64,
    pub game_start_millis: i64,
    #[serde(rename = "queueID")]
    pub queue_id: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawMatchPlayer {
    pub subject: String,
    pub game_name: String,
    pub tag_line: String,
    pub team_id: String,
    pub party_id: String,
    pub character_id: String,
    pub competitive_tier: i32,
    pub player_card: String,
    pub player_title: String,
    pub account_level: i32,
    pub stats: Option<RawPlayerStats>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawMatchPartyResp {
    pub match_info: RawMatchInfo,
    pub players: Vec<RawMatchPartyPlayer>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawMatchPartyPlayer {
    pub subject: String,
    pub party_id: String,
}

pub fn is_valid_riot_party_id(party_id: &str) -> bool {
    let clean = party_id.trim();
    !clean.is_empty() && !clean.chars().all(|c| c == '0' || c == '-')
}

pub fn build_cached_match_party_map<'a, I>(queue_id: &str, players: I) -> crate::cache::CachedMatchPartyMap
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let q = queue_id.trim().to_lowercase();
    let mut player_parties = HashMap::new();
    if !q.eq_ignore_ascii_case("deathmatch") && !q.eq_ignore_ascii_case("custom") {
        for (subject, party_id) in players {
            let sub = subject.trim().to_lowercase();
            let pid = party_id.trim().to_lowercase();
            if !sub.is_empty() && is_valid_riot_party_id(&pid) {
                player_parties.insert(sub, pid);
            }
        }
    }
    crate::cache::CachedMatchPartyMap {
        queue_id: q,
        player_parties,
    }
}

pub async fn fetch_match_party_map(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    cache: &LocalCache,
    shard: &str,
    headers: &HeaderMap,
    match_id: &str,
) -> Option<crate::cache::CachedMatchPartyMap> {
    if let Some(cached) = cache.get_cached_match_party_map(match_id) {
        return Some(cached);
    }
    let url = format!("{}/match-details/v1/matches/{match_id}", pd_base(shard));
    let raw: RawMatchPartyResp = limiter
        .request_json(|| http.get(&url).headers(headers.clone()))
        .await
        .ok()?;
    let party_map = build_cached_match_party_map(
        &raw.match_info.queue_id,
        raw.players
            .iter()
            .map(|p| (p.subject.as_str(), p.party_id.as_str())),
    );
    let _ = cache.put_cached_match_party_map(match_id, &party_map);
    Some(party_map)
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawPlayerStats {
    pub score: i32,
    pub rounds_played: i32,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawMatchTeam {
    pub team_id: String,
    pub won: bool,
    pub rounds_won: i32,
}

fn null_to_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawRoundResult {
    pub round_num: u32,
    #[serde(deserialize_with = "null_to_default")]
    pub round_result: String,
    #[serde(deserialize_with = "null_to_default")]
    pub winning_team: String,
    pub plant_round_time: i64,
    #[serde(deserialize_with = "null_to_default")]
    pub plant_site: String,
    #[serde(deserialize_with = "null_to_default")]
    pub bomb_planter: String,
    pub plant_location: Option<RawGameCoords>,
    #[serde(deserialize_with = "null_to_default")]
    pub plant_player_locations: Vec<RawPlayerLocation>,
    pub defuse_round_time: i64,
    #[serde(deserialize_with = "null_to_default")]
    pub bomb_defuser: String,
    pub defuse_location: Option<RawGameCoords>,
    #[serde(deserialize_with = "null_to_default")]
    pub defuse_player_locations: Vec<RawPlayerLocation>,
    #[serde(deserialize_with = "null_to_default")]
    pub player_stats: Vec<RawRoundPlayerStat>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawGameCoords {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawRoundPlayerStat {
    #[serde(deserialize_with = "null_to_default")]
    pub subject: String,
    #[serde(deserialize_with = "null_to_default")]
    pub damage: Vec<RawRoundDamage>,
    #[serde(deserialize_with = "null_to_default")]
    pub kills: Vec<RawRoundKill>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawRoundDamage {
    pub damage: i32,
    pub headshots: i32,
    pub bodyshots: i32,
    pub legshots: i32,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawRoundKill {
    pub round_time: i64,
    #[serde(deserialize_with = "null_to_default")]
    pub killer: String,
    #[serde(deserialize_with = "null_to_default")]
    pub victim: String,
    #[serde(deserialize_with = "null_to_default")]
    pub victim_location: RawGameCoords,
    #[serde(deserialize_with = "null_to_default")]
    pub player_locations: Vec<RawPlayerLocation>,
    #[serde(deserialize_with = "null_to_default")]
    pub finishing_damage: RawFinishingDamage,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawPlayerLocation {
    #[serde(deserialize_with = "null_to_default")]
    pub subject: String,
    pub view_radians: f64,
    #[serde(deserialize_with = "null_to_default")]
    pub location: RawGameCoords,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawFinishingDamage {
    pub damage_type: String,
    pub damage_item: String,
}

fn resolve_finishing_weapon(
    cache: &LocalCache,
    finishing: &RawFinishingDamage,
    killer_agent: Option<&crate::models::assets::Agent>,
) -> (String, Option<String>) {
    let item = finishing.damage_item.trim();
    if let Some(w) = cache.lookup_weapon(item) {
        let icon = w.kill_stream_icon_url();
        return (w.display_name, icon);
    }
    if finishing.damage_type.eq_ignore_ascii_case("Bomb") || item.eq_ignore_ascii_case("Bomb") {
        return ("Spike".to_string(), None);
    }
    if finishing.damage_type.eq_ignore_ascii_case("Ability")
        || item.eq_ignore_ascii_case("Ability1")
        || item.eq_ignore_ascii_case("Ability2")
        || item.eq_ignore_ascii_case("GrenadeAbility")
        || item.eq_ignore_ascii_case("Ultimate")
    {
        if let Some(agent) = killer_agent {
            let target_slot = if item.eq_ignore_ascii_case("GrenadeAbility") {
                "Grenade"
            } else {
                item
            };
            if let Some(ab) = agent
                .abilities
                .iter()
                .find(|a| a.slot.eq_ignore_ascii_case(target_slot))
            {
                let name = if ab.display_name.trim().is_empty() {
                    "Ability".to_string()
                } else {
                    ab.display_name.clone()
                };
                return (name, ab.display_icon.clone());
            }
        }
        return ("Ability".to_string(), None);
    }
    ("Ability / Rifle".to_string(), None)
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
struct RawContractsResp {
    pub contracts: Vec<RawContractEntry>,
    #[serde(deserialize_with = "null_to_default")]
    pub missions: Vec<RawMissionEntry>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
struct RawMissionEntry {
    #[serde(rename = "ID", deserialize_with = "null_to_default")]
    pub id: String,
    #[serde(deserialize_with = "null_to_default")]
    pub objectives: HashMap<String, i32>,
    #[serde(deserialize_with = "null_to_default")]
    pub complete: bool,
    pub expiration_time: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
struct RawContractEntry {
    #[serde(rename = "ContractDefinitionID")]
    pub contract_definition_id: String,
    pub progression_level_reached: u32,
    pub progression_towards_next_level: i64,
    pub contract_progression: RawContractProg,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
struct RawContractProg {
    pub total_earned: i64,
}

/// Converts Valorant in-engine world `(x, y)` coordinates into normalized `[0.0, 1.0]`
/// minimap coordinates using `valorant-api.com` map multipliers & scalars.
pub fn transform_minimap_coords(game_x: f64, game_y: f64, map: &MapInfo) -> (f64, f64) {
    if map.x_multiplier.abs() < 1e-9 || map.y_multiplier.abs() < 1e-9 {
        return (0.5, 0.5);
    }
    let norm_x = (game_y * map.x_multiplier + map.x_scalar_to_add).clamp(0.02, 0.98);
    let norm_y = (game_x * map.y_multiplier + map.y_scalar_to_add).clamp(0.02, 0.98);
    (norm_x, norm_y)
}

/// Projects Unreal Engine world `viewRadians` onto the 2D minimap screen angle
/// (0 = right, +pi/2 = down), accounting for the swapped X/Y axes and multiplier signs.
pub fn project_view_angle_rad(view_radians: f64, map: &MapInfo) -> f64 {
    let sx = if map.x_multiplier >= 0.0 { 1.0 } else { -1.0 };
    let sy = if map.y_multiplier >= 0.0 { 1.0 } else { -1.0 };
    let dx = view_radians.sin() * sx;
    let dy = view_radians.cos() * sy;
    dy.atan2(dx)
}

fn build_replay_player_snapshots(
    raw_locs: &[RawPlayerLocation],
    player_lookup: &HashMap<String, (String, String, Option<String>)>,
    map_meta: &MapInfo,
) -> Vec<ReplayPlayerSnapshot> {
    raw_locs
        .iter()
        .filter(|pl| !pl.subject.trim().is_empty())
        .map(|pl| {
            let (nx, ny) = transform_minimap_coords(pl.location.x, pl.location.y, map_meta);
            let angle = project_view_angle_rad(pl.view_radians, map_meta);
            let (name, team, icon) = player_lookup
                .get(&pl.subject.to_lowercase())
                .cloned()
                .unwrap_or_else(|| ("Player".into(), "Blue".into(), None));
            ReplayPlayerSnapshot {
                puuid: pl.subject.clone(),
                display_name: name,
                team_id: team,
                agent_icon: icon,
                norm_x: nx,
                norm_y: ny,
                view_angle_rad: angle,
            }
        })
        .collect()
}

pub async fn fetch_match_detail(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    cache: &LocalCache,
    session: &RiotSession,
    headers: &HeaderMap,
    match_id: &str,
) -> Result<MatchDetailSnapshot, MyValError> {
    fetch_match_detail_for_player(http, limiter, cache, session, headers, &session.puuid, match_id).await
}

pub async fn fetch_match_detail_for_player(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    cache: &LocalCache,
    session: &RiotSession,
    headers: &HeaderMap,
    perspective_puuid: &str,
    match_id: &str,
) -> Result<MatchDetailSnapshot, MyValError> {
    let shard = cache
        .get_meta(&format!("player_shard_{}", perspective_puuid.trim().to_lowercase()))
        .ok()
        .flatten()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| session.shard.clone());
    fetch_match_detail_on_shard(
        http,
        limiter,
        cache,
        &shard,
        headers,
        perspective_puuid,
        match_id,
    )
    .await
}

pub async fn fetch_match_detail_on_shard(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    cache: &LocalCache,
    shard: &str,
    headers: &HeaderMap,
    perspective_puuid: &str,
    match_id: &str,
) -> Result<MatchDetailSnapshot, MyValError> {
    if let Some(cached) = cache.get_cached_match_detail(match_id, perspective_puuid) {
        return Ok(cached);
    }
    let url = format!("{}/match-details/v1/matches/{match_id}", pd_base(shard));
    let raw: RawMatchDetailResp = limiter
        .request_json(|| http.get(&url).headers(headers.clone()))
        .await?;

    let mut cached_match_names: Vec<(String, String, String)> = Vec::new();
    for p in &raw.players {
        if !p.game_name.trim().is_empty() && !p.subject.trim().is_empty() {
            cached_match_names.push((
                p.subject.to_lowercase(),
                p.game_name.trim().to_string(),
                p.tag_line.trim().to_string(),
            ));
        }
    }
    if !cached_match_names.is_empty() {
        let _ = cache.put_cached_names(&cached_match_names);
    }

    let puuids: Vec<String> = raw.players.iter().map(|p| p.subject.clone()).collect();
    let names = fetch_names_batched(http, limiter, cache, shard, headers, &puuids).await;

    let party_map = build_cached_match_party_map(
        &raw.match_info.queue_id,
        raw.players
            .iter()
            .map(|p| (p.subject.as_str(), p.party_id.as_str())),
    );
    let _ = cache.put_cached_match_party_map(match_id, &party_map);

    let detail = parse_match_detail(cache, perspective_puuid, &names, raw);
    let _ = cache.put_cached_match_detail(&detail);
    Ok(detail)
}

fn resolve_match_player_name(
    p: &RawMatchPlayer,
    names: &HashMap<String, (String, String)>,
    agent_name: &str,
) -> String {
    let lookup = names
        .get(&p.subject.to_lowercase())
        .or_else(|| names.get(&p.subject));
    if let Some((gn, tl)) = lookup {
        let g = gn.trim();
        let t = tl.trim();
        if !g.is_empty() {
            return if t.is_empty() {
                g.to_string()
            } else {
                format!("{g}#{t}")
            };
        }
    }
    let g = p.game_name.trim();
    let t = p.tag_line.trim();
    if !g.is_empty() {
        return if t.is_empty() {
            g.to_string()
        } else {
            format!("{g}#{t}")
        };
    }
    agent_name.to_string()
}

fn parse_match_detail(
    cache: &LocalCache,
    my_puuid: &str,
    names: &HashMap<String, (String, String)>,
    raw: RawMatchDetailResp,
) -> MatchDetailSnapshot {
    let map_meta = cache
        .lookup_map_by_url_or_id(&raw.match_info.map_id)
        .unwrap_or_else(|| MapInfo {
            display_name: "Match Map".to_string(),
            x_multiplier: 0.00007,
            y_multiplier: -0.00007,
            x_scalar_to_add: 0.5,
            y_scalar_to_add: 0.5,
            ..Default::default()
        });

    let my_team_id = raw
        .players
        .iter()
        .find(|p| p.subject.eq_ignore_ascii_case(my_puuid))
        .map(|p| p.team_id.clone())
        .unwrap_or_else(|| "Blue".to_string());

    let mut my_team_score = 0;
    let mut enemy_team_score = 0;
    let mut won = false;
    for t in &raw.teams {
        if t.team_id.eq_ignore_ascii_case(&my_team_id) {
            my_team_score = t.rounds_won;
            won = t.won;
        } else {
            enemy_team_score = enemy_team_score.max(t.rounds_won);
        }
    }

    let mut damage_by_puuid: HashMap<String, (i32, i32, i32)> = HashMap::new();
    let mut first_bloods_by_puuid: HashMap<String, i32> = HashMap::new();
    let mut player_lookup: HashMap<String, (String, String, Option<String>)> = HashMap::new();
    let mut killer_agent_by_puuid: HashMap<String, crate::models::assets::Agent> = HashMap::new();

    for p in &raw.players {
        cache.cache_player_identity_meta(
            &p.subject,
            Some(&p.player_card),
            Some(&p.player_title),
            Some(p.account_level),
        );
        let ag = cache.lookup_agent(&p.character_id);
        let agent_name = ag
            .as_ref()
            .map(|a| a.display_name.clone())
            .unwrap_or_else(|| "Agent".to_string());
        let resolved_name = resolve_match_player_name(p, names, &agent_name);
        if let Some(agent_model) = ag.as_ref() {
            killer_agent_by_puuid.insert(p.subject.to_lowercase(), agent_model.clone());
        }
        player_lookup.insert(
            p.subject.to_lowercase(),
            (
                resolved_name,
                p.team_id.clone(),
                ag.and_then(|a| a.display_icon),
            ),
        );
    }

    let mut rounds = Vec::with_capacity(raw.round_results.len());
    for rr in &raw.round_results {
        let mut round_kills = Vec::new();
        for ps in &rr.player_stats {
            let entry = damage_by_puuid
                .entry(ps.subject.to_lowercase())
                .or_insert((0, 0, 0));
            for d in &ps.damage {
                entry.0 += d.damage;
                entry.1 += d.headshots;
                entry.2 += d.headshots + d.bodyshots + d.legshots;
            }
            for k in &ps.kills {
                let (v_nx, v_ny) =
                    transform_minimap_coords(k.victim_location.x, k.victim_location.y, &map_meta);
                let killer_loc = k
                    .player_locations
                    .iter()
                    .find(|pl| pl.subject.eq_ignore_ascii_case(&k.killer))
                    .map(|pl| pl.location)
                    .unwrap_or(k.victim_location);
                let (k_nx, k_ny) = transform_minimap_coords(killer_loc.x, killer_loc.y, &map_meta);

                let (k_name, k_team, k_icon) = player_lookup
                    .get(&k.killer.to_lowercase())
                    .cloned()
                    .unwrap_or_else(|| ("Player".into(), "Blue".into(), None));
                let (v_name, v_team, v_icon) = player_lookup
                    .get(&k.victim.to_lowercase())
                    .cloned()
                    .unwrap_or_else(|| ("Player".into(), "Red".into(), None));
                let (weapon_name, weapon_icon) = resolve_finishing_weapon(
                    cache,
                    &k.finishing_damage,
                    killer_agent_by_puuid.get(&k.killer.to_lowercase()),
                );

                let player_positions =
                    build_replay_player_snapshots(&k.player_locations, &player_lookup, &map_meta);

                round_kills.push(RoundKillEvent {
                    round_num: rr.round_num + 1,
                    time_in_round_secs: (k.round_time as f64 / 1000.0).max(0.0),
                    killer_puuid: k.killer.clone(),
                    killer_name: k_name,
                    killer_team: k_team,
                    killer_agent_icon: k_icon,
                    victim_puuid: k.victim.clone(),
                    victim_name: v_name,
                    victim_team: v_team,
                    victim_agent_icon: v_icon,
                    weapon_name,
                    weapon_icon,
                    killer_norm_x: k_nx,
                    killer_norm_y: k_ny,
                    victim_norm_x: v_nx,
                    victim_norm_y: v_ny,
                    player_positions,
                });
            }
        }
        round_kills.sort_by(|a, b| {
            a.time_in_round_secs
                .partial_cmp(&b.time_in_round_secs)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        if let Some(fb) = round_kills.first() {
            *first_bloods_by_puuid
                .entry(fb.killer_puuid.to_lowercase())
                .or_insert(0) += 1;
        }
        let (p_nx, p_ny) = match rr.plant_location {
            Some(loc) if loc.x.abs() > 1.0 || loc.y.abs() > 1.0 => {
                let (nx, ny) = transform_minimap_coords(loc.x, loc.y, &map_meta);
                (Some(nx), Some(ny))
            }
            _ => (None, None),
        };
        let plant_time_secs = if p_nx.is_some() && rr.plant_round_time > 0 {
            Some(rr.plant_round_time as f64 / 1000.0)
        } else {
            None
        };
        let plant_site = if !rr.plant_site.trim().is_empty() {
            Some(rr.plant_site.trim().to_string())
        } else {
            None
        };
        let planter_name = if !rr.bomb_planter.trim().is_empty() {
            player_lookup
                .get(&rr.bomb_planter.to_lowercase())
                .map(|(n, _, _)| n.clone())
        } else {
            None
        };
        let plant_player_positions =
            build_replay_player_snapshots(&rr.plant_player_locations, &player_lookup, &map_meta);

        let (d_nx, d_ny) = match rr.defuse_location {
            Some(loc) if loc.x.abs() > 1.0 || loc.y.abs() > 1.0 => {
                let (nx, ny) = transform_minimap_coords(loc.x, loc.y, &map_meta);
                (Some(nx), Some(ny))
            }
            _ => (None, None),
        };
        let defuse_time_secs = if d_nx.is_some() && rr.defuse_round_time > 0 {
            Some(rr.defuse_round_time as f64 / 1000.0)
        } else {
            None
        };
        let defuser_name = if !rr.bomb_defuser.trim().is_empty() {
            player_lookup
                .get(&rr.bomb_defuser.to_lowercase())
                .map(|(n, _, _)| n.clone())
        } else {
            None
        };
        let defuse_player_positions =
            build_replay_player_snapshots(&rr.defuse_player_locations, &player_lookup, &map_meta);

        rounds.push(RoundReplayData {
            round_num: rr.round_num + 1,
            winning_team: rr.winning_team.clone(),
            end_type: rr.round_result.clone(),
            plant_time_secs,
            plant_site,
            planter_name,
            plant_norm_x: p_nx,
            plant_norm_y: p_ny,
            plant_player_positions,
            defuse_time_secs,
            defuser_name,
            defuse_norm_x: d_nx,
            defuse_norm_y: d_ny,
            defuse_player_positions,
            kills: round_kills,
        });
    }

    let total_rounds = (my_team_score + enemy_team_score).max(1);
    let mut scoreboard: Vec<ScoreboardPlayer> = raw
        .players
        .into_iter()
        .map(|p| {
            let st = p.stats.clone().unwrap_or_default();
            let r_played = st.rounds_played.max(total_rounds).max(1);
            let (dmg, hs, shots) = damage_by_puuid
                .get(&p.subject.to_lowercase())
                .copied()
                .unwrap_or((0, 0, 0));
            let hs_pct = if shots > 0 {
                ((hs as f64 / shots as f64) * 1000.0).round() / 10.0
            } else {
                0.0
            };
            let agent = cache.lookup_agent(&p.character_id);
            let agent_name = agent
                .as_ref()
                .map(|a| a.display_name.clone())
                .unwrap_or_else(|| "Agent".to_string());
            let display_name = resolve_match_player_name(&p, names, &agent_name);
            let rank = build_rank_info(cache, p.competitive_tier, 0);
            ScoreboardPlayer {
                puuid: p.subject.clone(),
                display_name,
                team_id: p.team_id,
                is_me: p.subject.eq_ignore_ascii_case(my_puuid),
                agent_name,
                agent_icon: agent.and_then(|a| a.display_icon),
                rank_name: rank.tier_name,
                rank_color: rank.color_hex,
                kills: st.kills,
                deaths: st.deaths,
                assists: st.assists,
                acs: st.score / r_played,
                adr: dmg / r_played,
                hs_percent: hs_pct,
                first_bloods: *first_bloods_by_puuid
                    .get(&p.subject.to_lowercase())
                    .unwrap_or(&0),
            }
        })
        .collect();

    scoreboard.sort_by(|a, b| b.acs.cmp(&a.acs));

    MatchDetailSnapshot {
        match_id: raw.match_info.match_id,
        map_name: map_meta.display_name,
        map_splash: map_meta.list_view_icon.or(map_meta.splash),
        map_minimap_icon: map_meta.display_icon,
        queue_id: if raw.match_info.queue_id.is_empty() {
            "competitive".to_string()
        } else {
            raw.match_info.queue_id
        },
        start_time_epoch_secs: raw.match_info.game_start_millis / 1000,
        duration_secs: raw.match_info.game_length_millis / 1000,
        my_team_id,
        my_team_score,
        enemy_team_score,
        won,
        scoreboard,
        rounds,
    }
}

fn humanize_penalty_token(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    trimmed
        .split(['_', '-'])
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    format!("{}{}", first.to_uppercase(), chars.as_str().to_lowercase())
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn find_json_str(val: &serde_json::Value, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(s) = val.get(*k).and_then(|v| v.as_str()) {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn find_json_i32(val: &serde_json::Value, keys: &[&str]) -> Option<i32> {
    for k in keys {
        if let Some(n) = val.get(*k).and_then(|v| v.as_i64()) {
            return Some(n as i32);
        }
    }
    None
}

pub async fn fetch_account_penalties(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
) -> Vec<AccountPenalty> {
    let url = format!("{}/restrictions/v3/penalties", pd_base(&session.shard));
    let Ok(Some(json)) = limiter
        .request_json_opt::<serde_json::Value, _>(|| http.get(&url).headers(headers.clone()))
        .await
    else {
        return Vec::new();
    };
    let Some(arr) = json.get("Penalties").and_then(|v| v.as_array()) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for item in arr {
        let mut raw_reason = find_json_str(item, &["Reason", "AutomatedDetectionID", "RestrictionType"]);
        let mut games_remaining = find_json_i32(item, &["GamesRemaining", "RemainingGames"]);
        if let Some(obj) = item.as_object() {
            for (k, sub) in obj {
                if sub.is_object() {
                    if raw_reason.is_none() {
                        raw_reason = find_json_str(
                            sub,
                            &["AutomatedDetectionID", "RestrictionReason", "Reason", "Effect"],
                        );
                    }
                    if games_remaining.is_none() {
                        games_remaining = find_json_i32(sub, &["GamesRemaining", "RemainingGames"]);
                    }
                    if raw_reason.is_none() && k.contains("Effect") {
                        raw_reason = Some(k.clone());
                    }
                }
            }
        }
        let reason = raw_reason
            .map(|r| humanize_penalty_token(&r))
            .filter(|r| !r.is_empty())
            .unwrap_or_else(|| "Account Restriction".to_string());
        let expires_at = find_json_str(item, &["Expiry", "ExpiresAt"]).map(|raw_exp| {
            chrono::DateTime::parse_from_rfc3339(&raw_exp)
                .map(|dt| dt.with_timezone(&chrono::Utc).format("%d %b %Y %H:%M UTC").to_string())
                .unwrap_or(raw_exp)
        });
        out.push(AccountPenalty {
            reason,
            expires_at,
            games_remaining,
        });
    }
    out
}

pub async fn fetch_career_snapshot(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    cache: &LocalCache,
    session: &RiotSession,
    headers: &HeaderMap,
) -> Result<CareerSnapshot, MyValError> {
    let comp_url = format!(
        "{}/mmr/v1/players/{}/competitiveupdates?startIndex=0&endIndex=12&queue=competitive",
        pd_base(&session.shard),
        session.puuid
    );
    let contracts_url = format!(
        "{}/contracts/v1/contracts/{}",
        pd_base(&session.shard),
        session.puuid
    );

    let (mmr_summary, comp_opt, contracts_opt, recent_ids, penalties) = tokio::join!(
        fetch_player_mmr(http, limiter, cache, &session.shard, headers, &session.puuid),
        limiter.request_json_opt::<RawCompUpdatesResp, _>(|| http.get(&comp_url).headers(headers.clone())),
        limiter.request_json_opt::<RawContractsResp, _>(|| http.get(&contracts_url).headers(headers.clone())),
        fetch_recent_match_ids(http, limiter, cache, &session.shard, headers, &session.puuid, 10),
        fetch_account_penalties(http, limiter, session, headers),
    );

    let comp_entries = comp_opt.ok().flatten().map(|r| r.matches).unwrap_or_default();
    let mut rr_trend = Vec::new();
    let mut rr_by_match: HashMap<String, i32> = HashMap::new();

    for entry in &comp_entries {
        if entry.tier_after_update == 0 && entry.ranked_rating_earned == 0 {
            continue;
        }
        rr_by_match.insert(entry.match_id.to_lowercase(), entry.ranked_rating_earned);
        let map_meta = cache.lookup_map_by_url_or_id(&entry.map_id);
        let rank_meta = build_rank_info(cache, entry.tier_after_update, entry.ranked_rating_after_update);
        rr_trend.push(CompetitiveMatchUpdate {
            match_id: entry.match_id.clone(),
            map_name: map_meta
                .as_ref()
                .map(|m| m.display_name.clone())
                .unwrap_or_else(|| "Competitive".to_string()),
            map_splash: map_meta.and_then(|m| m.list_view_icon.or(m.splash)),
            start_time_epoch_secs: entry.match_start_time / 1000,
            tier_after: entry.tier_after_update,
            tier_name: rank_meta.tier_name,
            tier_color: rank_meta.color_hex,
            rr_after: entry.ranked_rating_after_update,
            rr_delta: entry.ranked_rating_earned,
            won: entry.ranked_rating_earned > 0,
        });
    }

    let mut match_ids: Vec<String> = Vec::with_capacity(10);
    for id in recent_ids {
        if !match_ids.iter().any(|existing| existing.eq_ignore_ascii_case(&id)) {
            match_ids.push(id);
            if match_ids.len() >= 10 {
                break;
            }
        }
    }
    for entry in &comp_entries {
        if match_ids.len() >= 10 {
            break;
        }
        if !match_ids.iter().any(|existing| existing.eq_ignore_ascii_case(&entry.match_id)) {
            match_ids.push(entry.match_id.clone());
        }
    }

    let detail_tasks = match_ids.iter().map(|mid| {
        let hdr = headers.clone();
        let mid_str = mid.clone();
        async move {
            fetch_match_detail(http, limiter, cache, session, &hdr, &mid_str)
                .await
                .ok()
        }
    });
    let details: Vec<Option<MatchDetailSnapshot>> = futures_util::future::join_all(detail_tasks).await;

    let mut recent_matches = Vec::new();
    for detail in details.into_iter().flatten() {
        if let Some(me) = detail.scoreboard.iter().find(|p| p.is_me) {
            recent_matches.push(MatchHistoryItem {
                match_id: detail.match_id.clone(),
                map_name: detail.map_name.clone(),
                map_splash: detail.map_splash.clone(),
                queue_id: detail.queue_id.clone(),
                start_time_epoch_secs: detail.start_time_epoch_secs,
                duration_secs: detail.duration_secs,
                my_team_rounds: detail.my_team_score,
                enemy_team_rounds: detail.enemy_team_score,
                won: detail.won,
                my_agent_name: me.agent_name.clone(),
                my_agent_icon: me.agent_icon.clone(),
                my_kills: me.kills,
                my_deaths: me.deaths,
                my_assists: me.assists,
                my_acs: me.acs,
                my_hs_percent: me.hs_percent,
                my_adr: me.adr,
                rr_delta: rr_by_match.get(&detail.match_id.to_lowercase()).copied(),
            });
        }
    }

    let agent_stats = compute_agent_stats_from_history(&recent_matches);
    let net_rr: i32 = rr_trend.iter().take(10).map(|u| u.rr_delta).sum();
    let wins_10 = rr_trend.iter().take(10).filter(|u| u.won).count();
    let total_10 = rr_trend.iter().take(10).count().max(1);
    let win_rate = ((wins_10 as f64 / total_10 as f64) * 1000.0).round() / 10.0;

    let streak_label = compute_streak_label(&rr_trend);
    let contracts_parsed = contracts_opt.ok().flatten();
    let missions: Vec<MissionProgress> = contracts_parsed
        .as_ref()
        .map(|c| {
            c.missions
                .iter()
                .filter(|m| !m.id.trim().is_empty())
                .map(|m| {
                    let meta = cache.lookup_mission(&m.id);
                    let current_progress: i32 = m.objectives.values().copied().sum();
                    let target_progress = meta
                        .as_ref()
                        .map(|a| a.progress_to_complete)
                        .filter(|&t| t > 0)
                        .unwrap_or_else(|| current_progress.max(1));
                    let title = meta
                        .as_ref()
                        .map(|a| a.title.clone())
                        .filter(|t| !t.trim().is_empty())
                        .unwrap_or_else(|| "Weekly Mission".to_string());
                    let xp_grant = meta.as_ref().map(|a| a.xp_grant).unwrap_or(0);
                    let expires_at = m.expiration_time.as_deref().map(|raw_exp| {
                        chrono::DateTime::parse_from_rfc3339(raw_exp)
                            .map(|dt| dt.with_timezone(&chrono::Utc).format("%d %b").to_string())
                            .unwrap_or_else(|_| raw_exp.to_string())
                    });
                    MissionProgress {
                        mission_id: m.id.clone(),
                        title,
                        current_progress: if m.complete {
                            target_progress.max(current_progress)
                        } else {
                            current_progress
                        },
                        target_progress,
                        xp_grant,
                        completed: m.complete || current_progress >= target_progress,
                        expires_at,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let contract = contracts_parsed
        .and_then(|c| {
            c.contracts.into_iter().max_by_key(|e| {
                (
                    e.progression_level_reached,
                    e.progression_towards_next_level,
                    e.contract_progression.total_earned,
                )
            })
        })
        .map(|best| ContractProgressSnapshot {
            active_contract_name: "Active Battlepass & Agent Gear".to_string(),
            current_level: best.progression_level_reached,
            max_level: 50,
            xp_in_level: best.progression_towards_next_level,
            xp_required_for_next: 25_000,
            total_xp: best.contract_progression.total_earned,
        })
        .unwrap_or_else(|| ContractProgressSnapshot {
            active_contract_name: "Season Battlepass".to_string(),
            current_level: 0,
            max_level: 50,
            xp_in_level: 0,
            xp_required_for_next: 25_000,
            total_xp: 0,
        });

    let (current_act_name, current_act_end_label) = resolve_current_act_info(cache);

    let snap = CareerSnapshot {
        puuid: session.puuid.clone(),
        current_rank: mmr_summary.current_rank,
        peak_rank: mmr_summary.peak_rank,
        previous_act_rank: mmr_summary.previous_act_rank,
        current_act_name,
        current_act_end_label,
        peak_rank_act_name: mmr_summary.peak_rank_act_name,
        net_rr_last_10: net_rr,
        win_rate_last_10: win_rate,
        current_streak_label: streak_label,
        rr_trend,
        recent_matches,
        agent_stats,
        contract,
        penalties,
        missions,
    };
    let _ = cache.put_cached_career(&session.puuid, &snap);
    Ok(snap)
}

fn compute_streak_label(updates: &[CompetitiveMatchUpdate]) -> String {
    let Some(first) = updates.first() else {
        return "EVEN".to_string();
    };
    let target = first.won;
    let count = updates.iter().take_while(|u| u.won == target).count();
    if target {
        format!("{count}W STREAK")
    } else {
        format!("{count}L STREAK")
    }
}

pub fn compute_agent_stats_from_history(matches: &[MatchHistoryItem]) -> Vec<AgentPerformanceStat> {
    let mut grouped: HashMap<String, (Option<String>, u32, u32, i32, i32, i32, f64)> = HashMap::new();
    for m in matches {
        let e = grouped.entry(m.my_agent_name.clone()).or_insert((
            m.my_agent_icon.clone(),
            0,
            0,
            0,
            0,
            0,
            0.0,
        ));
        e.1 += 1;
        if m.won {
            e.2 += 1;
        }
        e.3 += m.my_acs;
        e.4 += m.my_kills;
        e.5 += m.my_deaths;
        e.6 += m.my_hs_percent;
    }
    let mut stats: Vec<AgentPerformanceStat> = grouped
        .into_iter()
        .map(|(name, (icon, count, wins, sum_acs, sum_k, sum_d, sum_hs))| {
            let c = count.max(1) as f64;
            AgentPerformanceStat {
                agent_uuid: name.to_lowercase(),
                agent_name: name,
                agent_icon: icon,
                matches_played: count,
                wins,
                win_rate_percent: ((wins as f64 / c) * 1000.0).round() / 10.0,
                avg_acs: ((sum_acs as f64 / c) * 10.0).round() / 10.0,
                kd_ratio: ((sum_k as f64 / (sum_d.max(1) as f64)) * 100.0).round() / 100.0,
                hs_percent: ((sum_hs / c) * 10.0).round() / 10.0,
            }
        })
        .collect();
    stats.sort_by(|a, b| b.matches_played.cmp(&a.matches_played));
    stats
}

pub async fn fetch_player_profile(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    cache: &LocalCache,
    session: &RiotSession,
    headers: &HeaderMap,
    target_puuid: &str,
    force_refresh: bool,
) -> Result<PlayerProfileSnapshot, MyValError> {
    let saved_shard = cache
        .get_meta(&format!("player_shard_{}", target_puuid.trim().to_lowercase()))
        .ok()
        .flatten()
        .filter(|s| !s.is_empty());
    fetch_player_profile_on_shard(
        http,
        limiter,
        cache,
        session,
        headers,
        target_puuid,
        saved_shard.as_deref(),
        force_refresh,
    )
    .await
}

pub async fn fetch_player_profile_on_shard(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    cache: &LocalCache,
    session: &RiotSession,
    headers: &HeaderMap,
    target_puuid: &str,
    shard_override: Option<&str>,
    force_refresh: bool,
) -> Result<PlayerProfileSnapshot, MyValError> {
    let target = target_puuid.trim();
    if target.is_empty() {
        return Err(MyValError::InvalidInput("Player PUUID cannot be empty".to_string()));
    }
    if !force_refresh {
        if let Some(cached) = cache.get_cached_player_profile(target, PLAYER_PROFILE_CACHE_TTL_SECS) {
            return Ok(cached);
        }
    }

    let active_shard = shard_override
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(session.shard.as_str());

    let comp_url = format!(
        "{}/mmr/v1/players/{target}/competitiveupdates?startIndex=0&endIndex=10&queue=competitive",
        pd_base(active_shard)
    );
    let puuid_vec = vec![target.to_string()];

    let (mut names, mmr_summary, comp_opt, recent_ids) = tokio::join!(
        fetch_names_batched(http, limiter, cache, active_shard, headers, &puuid_vec),
        fetch_player_mmr(http, limiter, cache, active_shard, headers, target),
        limiter.request_json_opt::<RawCompUpdatesResp, _>(|| http.get(&comp_url).headers(headers.clone())),
        fetch_recent_match_ids(http, limiter, cache, active_shard, headers, target, 8),
    );
    if target.eq_ignore_ascii_case(&session.puuid) && !session.game_name.trim().is_empty() {
        names
            .entry(session.puuid.to_lowercase())
            .or_insert_with(|| (session.game_name.clone(), session.tag_line.clone()));
    }

    let display_name = names
        .get(&target.to_lowercase())
        .or_else(|| names.get(target))
        .map(|(g, t)| {
            if t.trim().is_empty() {
                g.trim().to_string()
            } else {
                format!("{}#{}", g.trim(), t.trim())
            }
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Valorant Player".to_string());

    let comp_entries = comp_opt.ok().flatten().map(|r| r.matches).unwrap_or_default();
    let mut rr_trend = Vec::new();
    let mut rr_by_match: HashMap<String, i32> = HashMap::new();
    for entry in &comp_entries {
        if entry.tier_after_update == 0 && entry.ranked_rating_earned == 0 {
            continue;
        }
        rr_by_match.insert(entry.match_id.to_lowercase(), entry.ranked_rating_earned);
        let map_meta = cache.lookup_map_by_url_or_id(&entry.map_id);
        let rank_meta = build_rank_info(cache, entry.tier_after_update, entry.ranked_rating_after_update);
        rr_trend.push(CompetitiveMatchUpdate {
            match_id: entry.match_id.clone(),
            map_name: map_meta
                .as_ref()
                .map(|m| m.display_name.clone())
                .unwrap_or_else(|| "Competitive".to_string()),
            map_splash: map_meta.and_then(|m| m.list_view_icon.or(m.splash)),
            start_time_epoch_secs: entry.match_start_time / 1000,
            tier_after: entry.tier_after_update,
            tier_name: rank_meta.tier_name,
            tier_color: rank_meta.color_hex,
            rr_after: entry.ranked_rating_after_update,
            rr_delta: entry.ranked_rating_earned,
            won: entry.ranked_rating_earned > 0,
        });
    }

    let mut match_ids: Vec<String> = comp_entries
        .iter()
        .take(6)
        .map(|m| m.match_id.clone())
        .collect();
    if match_ids.len() < 6 {
        for id in recent_ids {
            if !match_ids.iter().any(|existing| existing.eq_ignore_ascii_case(&id)) {
                match_ids.push(id);
                if match_ids.len() >= 6 {
                    break;
                }
            }
        }
    }

    let detail_tasks = match_ids.iter().map(|mid| {
        let hdr = headers.clone();
        let mid_str = mid.clone();
        async move {
            fetch_match_detail_on_shard(http, limiter, cache, active_shard, &hdr, target, &mid_str)
                .await
                .ok()
        }
    });
    let details: Vec<Option<MatchDetailSnapshot>> = futures_util::future::join_all(detail_tasks).await;

    let mut recent_matches = Vec::new();
    for detail in details.into_iter().flatten() {
        if let Some(me) = detail
            .scoreboard
            .iter()
            .find(|p| p.puuid.eq_ignore_ascii_case(target))
        {
            recent_matches.push(MatchHistoryItem {
                match_id: detail.match_id.clone(),
                map_name: detail.map_name.clone(),
                map_splash: detail.map_splash.clone(),
                queue_id: detail.queue_id.clone(),
                start_time_epoch_secs: detail.start_time_epoch_secs,
                duration_secs: detail.duration_secs,
                my_team_rounds: detail.my_team_score,
                enemy_team_rounds: detail.enemy_team_score,
                won: detail.won,
                my_agent_name: me.agent_name.clone(),
                my_agent_icon: me.agent_icon.clone(),
                my_kills: me.kills,
                my_deaths: me.deaths,
                my_assists: me.assists,
                my_acs: me.acs,
                my_hs_percent: me.hs_percent,
                my_adr: me.adr,
                rr_delta: rr_by_match.get(&detail.match_id.to_lowercase()).copied(),
            });
        }
    }

    let matches_analyzed = recent_matches.len() as u32;
    let wins = recent_matches.iter().filter(|m| m.won).count() as u32;
    let losses = matches_analyzed.saturating_sub(wins);
    let total_kills: i32 = recent_matches.iter().map(|m| m.my_kills).sum();
    let total_deaths: i32 = recent_matches.iter().map(|m| m.my_deaths).sum();
    let total_assists: i32 = recent_matches.iter().map(|m| m.my_assists).sum();
    let total_acs: i32 = recent_matches.iter().map(|m| m.my_acs).sum();
    let total_adr: i32 = recent_matches.iter().map(|m| m.my_adr).sum();
    let total_hs: f64 = recent_matches.iter().map(|m| m.my_hs_percent).sum();

    let denom = matches_analyzed.max(1) as f64;
    let kd_ratio = if matches_analyzed > 0 {
        ((total_kills as f64 / total_deaths.max(1) as f64) * 100.0).round() / 100.0
    } else {
        0.0
    };
    let avg_acs = if matches_analyzed > 0 {
        ((total_acs as f64 / denom) * 10.0).round() / 10.0
    } else {
        0.0
    };
    let avg_adr = if matches_analyzed > 0 {
        ((total_adr as f64 / denom) * 10.0).round() / 10.0
    } else {
        0.0
    };
    let hs_percent = if matches_analyzed > 0 {
        ((total_hs / denom) * 10.0).round() / 10.0
    } else {
        0.0
    };
    let avg_kills = if matches_analyzed > 0 {
        ((total_kills as f64 / denom) * 10.0).round() / 10.0
    } else {
        0.0
    };
    let avg_deaths = if matches_analyzed > 0 {
        ((total_deaths as f64 / denom) * 10.0).round() / 10.0
    } else {
        0.0
    };
    let avg_assists = if matches_analyzed > 0 {
        ((total_assists as f64 / denom) * 10.0).round() / 10.0
    } else {
        0.0
    };
    let win_rate_percent = if matches_analyzed > 0 {
        ((wins as f64 / denom) * 1000.0).round() / 10.0
    } else if !rr_trend.is_empty() {
        let w = rr_trend.iter().filter(|u| u.won).count() as f64;
        ((w / rr_trend.len() as f64) * 1000.0).round() / 10.0
    } else {
        0.0
    };

    let agent_stats = compute_agent_stats_from_history(&recent_matches);
    let net_rr_last_10: i32 = rr_trend.iter().take(10).map(|u| u.rr_delta).sum();
    let current_streak_label = compute_streak_label(&rr_trend);

    let (card_id, title_id, account_level) = cache.get_cached_player_identity_meta(target);
    let card_asset = card_id.as_deref().and_then(|id| cache.lookup_player_card(id));
    let player_card_small = card_asset.as_ref().and_then(|c| c.small_art.clone());
    let player_card_wide = card_asset
        .as_ref()
        .and_then(|c| c.wide_art.clone().or_else(|| c.large_art.clone()));
    let player_title = title_id
        .as_deref()
        .and_then(|id| cache.lookup_title(id))
        .map(|t| t.title_text)
        .filter(|t| !t.trim().is_empty());

    let profile = PlayerProfileSnapshot {
        puuid: target.to_string(),
        display_name,
        account_level,
        player_card_small,
        player_card_wide,
        player_title,
        current_rank: mmr_summary.current_rank,
        peak_rank: mmr_summary.peak_rank,
        previous_act_rank: mmr_summary.previous_act_rank,
        leaderboard_rank: mmr_summary.leaderboard_rank,
        matches_analyzed,
        wins,
        losses,
        win_rate_percent,
        kd_ratio,
        avg_acs,
        avg_adr,
        hs_percent,
        avg_kills,
        avg_deaths,
        avg_assists,
        net_rr_last_10,
        current_streak_label,
        rr_trend,
        agent_stats,
        recent_matches,
    };

    let _ = cache.put_cached_player_profile(target, &profile);
    Ok(profile)
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawRiotAliasEntry {
    pub puuid: String,
    pub active: bool,
    pub alias: Option<RawRiotAliasName>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawRiotAliasName {
    #[serde(alias = "gameName")]
    pub game_name: String,
    #[serde(alias = "tagLine")]
    pub tag_line: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawOfficialAccountResp {
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawHenrikAccountEnvelope {
    pub status: u16,
    pub data: Option<RawHenrikAccountData>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawHenrikAccountData {
    pub puuid: String,
    pub region: String,
    pub account_level: i32,
    pub name: String,
    pub tag: String,
    pub card: Option<RawHenrikCard>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawHenrikCard {
    pub id: String,
}

pub fn parse_riot_id(input: &str) -> Result<(String, String), MyValError> {
    let trimmed = input.trim();
    let Some((name_part, tag_part)) = trimmed.rsplit_once('#') else {
        return Err(MyValError::InvalidInput(
            "Enter a full Riot ID including tag (e.g. PlayerName#TAG)".to_string(),
        ));
    };
    let name = name_part.trim();
    let tag = tag_part.trim();
    if name.is_empty() || tag.is_empty() {
        return Err(MyValError::InvalidInput(
            "Both Name and #TAG are required (e.g. PlayerName#TAG)".to_string(),
        ));
    }
    Ok((name.to_string(), tag.to_string()))
}

pub struct ResolvedRiotAccount {
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
    pub shard_hint: Option<String>,
}

/// Resolves ANY Riot ID (GameName#TagLine) to a PUUID via:
/// 1. Local SQLite name_cache (instant)
/// 2. Riot's RSO-authenticated Account Alias service (api.account.riotgames.com/aliases/v1/aliases)
/// 3. Optional user-configured free HenrikDev (api.henrikdev.xyz) or official Riot Dev (RGAPI-) key
pub async fn resolve_riot_id_to_puuid(
    http: &reqwest::Client,
    cache: &LocalCache,
    session: &RiotSession,
    game_name: &str,
    tag_line: &str,
) -> Result<ResolvedRiotAccount, MyValError> {
    // Tier 1: Local exact cache
    if let Some(puuid) = cache.find_cached_puuid_by_riot_id(game_name, tag_line) {
        let shard_hint = cache
            .get_meta(&format!("player_shard_{}", puuid.to_lowercase()))
            .ok()
            .flatten()
            .filter(|s| !s.is_empty());
        return Ok(ResolvedRiotAccount {
            puuid,
            game_name: game_name.to_string(),
            tag_line: tag_line.to_string(),
            shard_hint,
        });
    }

    // Tier 2: Riot RSO Account Alias lookup (uses user's own Riot Bearer token)
    let mut rso_not_found = false;
    if let Ok(url) = Url::parse_with_params(
        "https://api.account.riotgames.com/aliases/v1/aliases",
        &[("gameName", game_name), ("tagLine", tag_line)],
    ) {
        if let Ok(resp) = http
            .get(url)
            .header("Authorization", format!("Bearer {}", session.access_token))
            .send()
            .await
        {
            let status = resp.status().as_u16();
            if status == 200 {
                if let Ok(entries) = resp.json::<Vec<RawRiotAliasEntry>>().await {
                    let chosen = entries
                        .iter()
                        .find(|e| e.active && !e.puuid.trim().is_empty())
                        .or_else(|| entries.iter().find(|e| !e.puuid.trim().is_empty()));
                    if let Some(entry) = chosen {
                        let gn = entry
                            .alias
                            .as_ref()
                            .map(|a| a.game_name.trim())
                            .filter(|s| !s.is_empty())
                            .unwrap_or(game_name)
                            .to_string();
                        let tl = entry
                            .alias
                            .as_ref()
                            .map(|a| a.tag_line.trim())
                            .filter(|s| !s.is_empty())
                            .unwrap_or(tag_line)
                            .to_string();
                        let _ = cache.put_cached_names(&[(entry.puuid.to_lowercase(), gn.clone(), tl.clone())]);
                        return Ok(ResolvedRiotAccount {
                            puuid: entry.puuid.clone(),
                            game_name: gn,
                            tag_line: tl,
                            shard_hint: None,
                        });
                    }
                    rso_not_found = true;
                }
            } else if status == 404 {
                rso_not_found = true;
            }
        }
    }

    // Tier 3: Optional user-provided HenrikDev or Official Riot Dev API key
    if let Some(api_key) = cache.get_lookup_api_key() {
        if api_key.to_uppercase().starts_with("RGAPI-") {
            let mut url = Url::parse("https://asia.api.riotgames.com/riot/account/v1/accounts/by-riot-id")
                .map_err(|e| MyValError::Parse(e.to_string()))?;
            if let Ok(mut segs) = url.path_segments_mut() {
                segs.push(game_name);
                segs.push(tag_line);
            }
            let resp = http
                .get(url)
                .header("X-Riot-Token", &api_key)
                .send()
                .await?;
            let status = resp.status().as_u16();
            if status == 200 {
                let parsed: RawOfficialAccountResp = resp.json().await?;
                if !parsed.puuid.trim().is_empty() {
                    let gn = if parsed.game_name.trim().is_empty() {
                        game_name.to_string()
                    } else {
                        parsed.game_name
                    };
                    let tl = if parsed.tag_line.trim().is_empty() {
                        tag_line.to_string()
                    } else {
                        parsed.tag_line
                    };
                    let _ = cache.put_cached_names(&[(parsed.puuid.to_lowercase(), gn.clone(), tl.clone())]);
                    return Ok(ResolvedRiotAccount {
                        puuid: parsed.puuid,
                        game_name: gn,
                        tag_line: tl,
                        shard_hint: None,
                    });
                }
            } else if status == 404 {
                return Err(MyValError::InvalidInput(format!(
                    "No Riot account found for {game_name}#{tag_line}"
                )));
            } else if status == 401 || status == 403 {
                return Err(MyValError::Unauthorized(
                    "Saved Riot API key expired or invalid (Riot Dev keys expire after 24h) — update it in Lookup Settings."
                        .to_string(),
                ));
            }
        } else {
            // HenrikDev API key (free from api.henrikdev.xyz/dashboard)
            let mut url = Url::parse("https://api.henrikdev.xyz/valorant/v1/account")
                .map_err(|e| MyValError::Parse(e.to_string()))?;
            if let Ok(mut segs) = url.path_segments_mut() {
                segs.push(game_name);
                segs.push(tag_line);
            }
            let resp = http
                .get(url)
                .header("Authorization", &api_key)
                .send()
                .await?;
            let status = resp.status().as_u16();
            if status == 200 {
                let env: RawHenrikAccountEnvelope = resp.json().await?;
                if let Some(data) = env.data {
                    if !data.puuid.trim().is_empty() {
                        let gn = if data.name.trim().is_empty() {
                            game_name.to_string()
                        } else {
                            data.name
                        };
                        let tl = if data.tag.trim().is_empty() {
                            tag_line.to_string()
                        } else {
                            data.tag
                        };
                        let shard_hint = if data.region.trim().is_empty() {
                            None
                        } else {
                            Some(region_to_shard(&data.region).shard)
                        };
                        let _ = cache.put_cached_names(&[(data.puuid.to_lowercase(), gn.clone(), tl.clone())]);
                        if let Some(ref sh) = shard_hint {
                            let _ = cache.set_meta(&format!("player_shard_{}", data.puuid.to_lowercase()), sh);
                        }
                        cache.cache_player_identity_meta(
                            &data.puuid,
                            data.card.as_ref().map(|c| c.id.as_str()),
                            None,
                            Some(data.account_level),
                        );
                        return Ok(ResolvedRiotAccount {
                            puuid: data.puuid,
                            game_name: gn,
                            tag_line: tl,
                            shard_hint,
                        });
                    }
                }
            } else if status == 404 {
                return Err(MyValError::InvalidInput(format!(
                    "No Valorant account found for {game_name}#{tag_line}"
                )));
            } else if status == 401 || status == 403 {
                return Err(MyValError::Unauthorized(
                    "Saved HenrikDev API key was rejected — check your key at api.henrikdev.xyz/dashboard."
                        .to_string(),
                ));
            }
        }
    }

    if rso_not_found {
        return Err(MyValError::InvalidInput(format!(
            "No Riot account found for {game_name}#{tag_line}"
        )));
    }

    Err(MyValError::InvalidInput(
        "Riot requires an API key to look up players you haven't played with yet. Add a free HenrikDev key (api.henrikdev.xyz/dashboard) or Riot Dev key below to search any player globally."
            .to_string(),
    ))
}

pub async fn lookup_player_profile_by_riot_id(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    cache: &LocalCache,
    session: &RiotSession,
    headers: &HeaderMap,
    riot_id_input: &str,
) -> Result<PlayerProfileSnapshot, MyValError> {
    let (game_name, tag_line) = parse_riot_id(riot_id_input)?;
    let resolved = resolve_riot_id_to_puuid(http, cache, session, &game_name, &tag_line).await?;

    let initial_shard = resolved
        .shard_hint
        .as_deref()
        .unwrap_or(session.shard.as_str());
    let mut profile = fetch_player_profile_on_shard(
        http,
        limiter,
        cache,
        session,
        headers,
        &resolved.puuid,
        Some(initial_shard),
        false,
    )
    .await?;

    // If shard wasn't known and home shard had zero data, probe the other Valorant shards automatically
    if resolved.shard_hint.is_none()
        && profile.current_rank.tier == 0
        && profile.recent_matches.is_empty()
        && profile.rr_trend.is_empty()
    {
        for alt_shard in ["na", "eu", "ap", "kr"] {
            if alt_shard.eq_ignore_ascii_case(initial_shard) {
                continue;
            }
            if let Ok(alt_profile) = fetch_player_profile_on_shard(
                http,
                limiter,
                cache,
                session,
                headers,
                &resolved.puuid,
                Some(alt_shard),
                true,
            )
            .await
            {
                if alt_profile.current_rank.tier > 0
                    || !alt_profile.recent_matches.is_empty()
                    || !alt_profile.rr_trend.is_empty()
                {
                    let _ = cache.set_meta(
                        &format!("player_shard_{}", resolved.puuid.to_lowercase()),
                        alt_shard,
                    );
                    profile = alt_profile;
                    break;
                }
            }
        }
    }

    if profile.display_name == "Valorant Player" || profile.display_name.trim().is_empty() {
        profile.display_name = format!("{}#{}", resolved.game_name, resolved.tag_line);
        let _ = cache.put_cached_player_profile(&resolved.puuid, &profile);
    }

    Ok(profile)
}
