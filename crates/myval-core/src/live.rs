use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use reqwest::header::HeaderMap;
use tokio::sync::Mutex;

use crate::auth::build_riot_headers;
use crate::cache::{CachedLiveMatchGuns, CachedMatchPartyMap, LocalCache};
use crate::career::{fetch_match_party_map, is_valid_riot_party_id};
use crate::error::MyValError;
use crate::glz::{
    get_coregame_loadouts, get_coregame_match, get_coregame_player, get_party, get_party_player,
    get_pregame_loadouts, get_pregame_match, get_pregame_player, RawCoreGameLoadoutEntry,
    RawCoreLoadoutDetail, RawCoreLoadoutItem, RawCoreLoadoutSocket, RawPartyResp,
    RawPreGameMatchResp,
};
use crate::models::assets::Skin;
use crate::models::auth::RiotSession;
use crate::models::career::{CompetitiveMatchUpdate, MatchHistoryItem};
use crate::models::live::{
    CoreGameSnapshot, EquippedWeaponSkin, LiveState, PartyGroup, PartyMemberInfo, PartySnapshot,
    PreGameAlly, PreGameSnapshot, ScoutedPlayer,
};
use crate::models::mmr::PlayerMmrSummary;
use crate::pd::{
    build_rank_info, fetch_names_batched, fetch_player_mmr, fetch_recent_match_ids,
};
use crate::rate_limit::RateLimiter;

const SKIN_SOCKET_UUID: &str = "bcef87d6-209b-46c6-8b19-fbe40bd95abc";
const SKIN_LEVEL_SOCKET_UUID: &str = "e7c63390-eda7-46e0-bb7a-a6abdacd2433";
const SKIN_CHROMA_SOCKET_UUID: &str = "3ad1b2b2-acdb-4524-852f-954a77ddae05";

// Valorant top 3 content tiers (Premium = Purple/Magenta, Exclusive = Orange/Gold & Limited Editions, Ultra = Gold)
pub const TIER_PREMIUM_UUID: &str = "60bca009-4182-7998-dee7-b8a2558dc369";
pub const TIER_EXCLUSIVE_UUID: &str = "e046854e-406c-37f4-6607-19a9ba8426fc";
pub const TIER_ULTRA_UUID: &str = "411e4a55-4e59-7757-41f0-86a53f101bb5";

pub fn format_display_name(
    puuid: &str,
    names: &HashMap<String, (String, String)>,
    _incognito: bool,
    _is_me: bool,
    fallback_agent: &str,
) -> String {
    let resolved = names
        .get(&puuid.to_lowercase())
        .or_else(|| names.get(puuid));
    if let Some((g, t)) = resolved {
        let g = g.trim();
        let t = t.trim();
        if !g.is_empty() {
            return if t.is_empty() {
                g.to_string()
            } else {
                format!("{g}#{t}")
            };
        }
    }
    let fb = fallback_agent.trim();
    if fb.is_empty() || fb.eq_ignore_ascii_case("Selecting...") || fb.eq_ignore_ascii_case("Selecting agent...") {
        "Player".to_string()
    } else {
        fb.to_string()
    }
}

pub fn visible_account_level(account_level: i32, hide_account_level: bool, is_me: bool) -> Option<i32> {
    if (hide_account_level && !is_me) || account_level <= 0 {
        None
    } else {
        Some(account_level)
    }
}

/// Tracker.gg-style historical Party/Stack detector.
const MAX_CANDIDATE_MATCH_PARTY_FETCHES: usize = 10;

fn is_non_party_queue(queue_id: &str) -> bool {
    let q = queue_id.trim();
    q.eq_ignore_ascii_case("deathmatch") || q.eq_ignore_ascii_case("custom")
}

fn party_for_player<'a>(map: &'a CachedMatchPartyMap, puuid_lower: &str) -> Option<&'a str> {
    if let Some(pid) = map.player_parties.get(puuid_lower) {
        if is_valid_riot_party_id(pid) {
            return Some(pid.as_str());
        }
    }
    map.player_parties.iter().find_map(|(k, v)| {
        if k.trim().eq_ignore_ascii_case(puuid_lower) && is_valid_riot_party_id(v) {
            Some(v.as_str())
        } else {
            None
        }
    })
}

fn cleaned_histories_by_player(
    players: &[(String, String)],
    recent_matches_by_puuid: &HashMap<String, Vec<String>>,
    exclude_match_id: Option<&str>,
) -> Vec<Vec<String>> {
    let lookup: HashMap<String, &[String]> = recent_matches_by_puuid
        .iter()
        .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.as_slice()))
        .collect();
    let excluded = exclude_match_id
        .map(|id| id.trim().to_ascii_lowercase())
        .filter(|id| !id.is_empty());

    players
        .iter()
        .map(|(puuid, _)| {
            lookup
                .get(&puuid.trim().to_ascii_lowercase())
                .map(|list| {
                    let mut seen: HashSet<String> = HashSet::new();
                    let mut out = Vec::with_capacity(list.len());
                    for raw in *list {
                        let m = raw.trim().to_ascii_lowercase();
                        if m.is_empty() || excluded.as_ref() == Some(&m) {
                            continue;
                        }
                        if seen.insert(m.clone()) {
                            out.push(m);
                        }
                    }
                    out
                })
                .unwrap_or_default()
        })
        .collect()
}

async fn fetch_my_live_party_puuids(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
) -> Option<Vec<String>> {
    let party_player = get_party_player(http, limiter, session, headers)
        .await
        .ok()
        .flatten()?;
    let party_id = party_player.current_party_id.trim();
    if party_id.is_empty() || !is_valid_riot_party_id(party_id) {
        return None;
    }
    let party = get_party(http, limiter, session, headers, party_id)
        .await
        .ok()?;
    let members: Vec<String> = party
        .members
        .into_iter()
        .map(|m| m.subject.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    if members.is_empty() {
        None
    } else {
        Some(members)
    }
}

pub fn collect_shared_candidate_match_ids(
    players: &[(String, String)],
    recent_matches_by_puuid: &HashMap<String, Vec<String>>,
    exclude_match_id: Option<&str>,
    max_candidates: usize,
) -> Vec<String> {
    if players.len() < 2 || max_candidates == 0 {
        return Vec::new();
    }

    let cleaned = cleaned_histories_by_player(players, recent_matches_by_puuid, exclude_match_id);

    struct CandidateStats {
        players_count: usize,
        latest_count: usize,
        first_pos: usize,
    }

    let mut per_team_counts: HashMap<(String, String), CandidateStats> = HashMap::new();
    for (player_idx, (_, team_id)) in players.iter().enumerate() {
        let team_key = team_id.trim().to_ascii_lowercase();
        for (pos, mid) in cleaned[player_idx].iter().enumerate() {
            let entry = per_team_counts
                .entry((team_key.clone(), mid.clone()))
                .or_insert(CandidateStats {
                    players_count: 0,
                    latest_count: 0,
                    first_pos: pos,
                });
            entry.players_count += 1;
            if pos == 0 {
                entry.latest_count += 1;
            }
            if pos < entry.first_pos {
                entry.first_pos = pos;
            }
        }
    }

    let mut best_by_match: HashMap<String, (bool, usize, usize)> = HashMap::new();
    for ((_, mid), stats) in per_team_counts {
        if stats.players_count < 2 {
            continue;
        }
        let is_latest_pair = stats.latest_count >= 2;
        best_by_match
            .entry(mid)
            .and_modify(|e| {
                e.0 = e.0 || is_latest_pair;
                e.1 = e.1.max(stats.players_count);
                e.2 = e.2.min(stats.first_pos);
            })
            .or_insert((is_latest_pair, stats.players_count, stats.first_pos));
    }

    let mut scored: Vec<(String, bool, usize, usize)> = best_by_match
        .into_iter()
        .map(|(mid, (is_latest, count, pos))| (mid, is_latest, count, pos))
        .collect();
    scored.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| b.2.cmp(&a.2))
            .then_with(|| a.3.cmp(&b.3))
            .then_with(|| a.0.cmp(&b.0))
    });
    scored
        .into_iter()
        .take(max_candidates)
        .map(|(mid, _, _, _)| mid)
        .collect()
}

async fn fetch_candidate_match_parties(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    cache: &LocalCache,
    shard: &str,
    headers: &HeaderMap,
    players: &[(String, String)],
    recent_matches_by_puuid: &HashMap<String, Vec<String>>,
    exclude_match_id: Option<&str>,
) -> HashMap<String, CachedMatchPartyMap> {
    let candidates = collect_shared_candidate_match_ids(
        players,
        recent_matches_by_puuid,
        exclude_match_id,
        MAX_CANDIDATE_MATCH_PARTY_FETCHES,
    );
    if candidates.is_empty() {
        return HashMap::new();
    }
    let futs = candidates.into_iter().map(|mid| async move {
        let map = fetch_match_party_map(http, limiter, cache, shard, headers, &mid).await;
        (mid, map)
    });
    futures_util::future::join_all(futs)
        .await
        .into_iter()
        .filter_map(|(mid, opt)| opt.map(|m| (mid.trim().to_ascii_lowercase(), m)))
        .collect()
}

async fn resolve_detected_party_groups(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    cache: &LocalCache,
    session: &RiotSession,
    headers: &HeaderMap,
    players: &[(String, String)],
    recent_matches_by_puuid: &HashMap<String, Vec<String>>,
    my_live_party_puuids: Option<&[String]>,
    exclude_match_id: Option<&str>,
) -> Vec<PartyGroup> {
    let match_parties = fetch_candidate_match_parties(
        http,
        limiter,
        cache,
        &session.shard,
        headers,
        players,
        recent_matches_by_puuid,
        exclude_match_id,
    )
    .await;
    let my_live_party = my_live_party_puuids.map(|members| (session.puuid.as_str(), members));
    detect_party_groups(
        players,
        recent_matches_by_puuid,
        &match_parties,
        my_live_party,
        exclude_match_id,
    )
}

/// Groups same-team players into premade party stacks using three verified layers:
/// 1. Authoritative GLZ live party roster (`my_live_party`) for `is_me` and live party mates.
/// 2. PD `match-details` `partyId` confirmation on >= 2 non-DM/custom shared matches.
/// 3. PD `match-details` `partyId` confirmation on both players' single most-recent match (`index 0`).
pub fn detect_party_groups(
    players: &[(String, String)], // (puuid, team_id)
    recent_matches_by_puuid: &HashMap<String, Vec<String>>,
    match_parties: &HashMap<String, CachedMatchPartyMap>,
    my_live_party: Option<(&str, &[String])>,
    exclude_match_id: Option<&str>,
) -> Vec<PartyGroup> {
    let n = players.len();
    if n < 2 {
        return Vec::new();
    }

    let cleaned_history =
        cleaned_histories_by_player(players, recent_matches_by_puuid, exclude_match_id);
    let normalized_match_parties: HashMap<String, &CachedMatchPartyMap> = match_parties
        .iter()
        .map(|(k, v)| (k.trim().to_ascii_lowercase(), v))
        .collect();
    let live_party_puuids: HashSet<String> = match my_live_party {
        Some((self_puuid, members)) => {
            let mut set: HashSet<String> = members
                .iter()
                .map(|m| m.trim().to_ascii_lowercase())
                .filter(|m| !m.is_empty())
                .collect();
            let self_lower = self_puuid.trim().to_ascii_lowercase();
            if !self_lower.is_empty() && !set.is_empty() {
                set.insert(self_lower);
            }
            set
        }
        None => HashSet::new(),
    };

    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }
    fn union(parent: &mut [usize], a: usize, b: usize) {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra != rb {
            parent[rb] = ra;
        }
    }

    let mut pair_shared_counts: HashMap<(usize, usize), u32> = HashMap::new();

    for i in 0..n {
        for j in (i + 1)..n {
            // Never group across opposing teams
            if !players[i].1.eq_ignore_ascii_case(&players[j].1) {
                continue;
            }

            let puuid_i = players[i].0.trim().to_ascii_lowercase();
            let puuid_j = players[j].0.trim().to_ascii_lowercase();

            let both_in_live_party = if !live_party_puuids.is_empty() {
                let i_in_live = live_party_puuids.contains(&puuid_i);
                let j_in_live = live_party_puuids.contains(&puuid_j);
                if i_in_live != j_in_live {
                    continue;
                }
                i_in_live && j_in_live
            } else {
                false
            };

            let hist_i = &cleaned_history[i];
            let hist_j = &cleaned_history[j];
            let mut confirmed_shared_count = 0u32;
            let mut latest_confirmed = false;

            if !hist_i.is_empty() && !hist_j.is_empty() {
                let set_j: HashSet<&str> = hist_j.iter().map(|s| s.as_str()).collect();
                for (pos_i, mid) in hist_i.iter().enumerate() {
                    if !set_j.contains(mid.as_str()) {
                        continue;
                    }
                    let Some(pmap) = normalized_match_parties.get(mid) else {
                        continue;
                    };
                    if is_non_party_queue(&pmap.queue_id) {
                        continue;
                    }
                    let Some(pid_i) = party_for_player(pmap, &puuid_i) else {
                        continue;
                    };
                    let Some(pid_j) = party_for_player(pmap, &puuid_j) else {
                        continue;
                    };
                    if pid_i.eq_ignore_ascii_case(pid_j) {
                        confirmed_shared_count += 1;
                        if pos_i == 0 && hist_j[0] == *mid {
                            latest_confirmed = true;
                        }
                    }
                }
            }

            if both_in_live_party || confirmed_shared_count >= 2 || latest_confirmed {
                union(&mut parent, i, j);
                pair_shared_counts.insert((i, j), confirmed_shared_count.max(1));
            }
        }
    }

    let mut components: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..n {
        let root = find(&mut parent, i);
        components.entry(root).or_default().push(i);
    }

    let mut raw_groups: Vec<(usize, Vec<usize>)> = components
        .into_iter()
        .filter(|(_, members)| members.len() >= 2)
        .collect();
    raw_groups.sort_by_key(|(_, m)| m[0]);

    let labels = ["A", "B", "C", "D", "E"];
    raw_groups
        .into_iter()
        .enumerate()
        .map(|(idx, (_, indices))| {
            let group_id = labels
                .get(idx)
                .map(|l| (*l).to_string())
                .unwrap_or_else(|| format!("G{}", idx + 1));
            let label = stack_label_for_size(indices.len(), &group_id);
            let mut max_shared = 1u32;
            for &a in &indices {
                for &b in &indices {
                    if a < b {
                        if let Some(&c) = pair_shared_counts.get(&(a, b)) {
                            max_shared = max_shared.max(c);
                        }
                    }
                }
            }
            PartyGroup {
                group_id,
                label,
                team_id: players[indices[0]].1.clone(),
                member_puuids: indices.iter().map(|&i| players[i].0.clone()).collect(),
                confidence_shared_matches: max_shared,
            }
        })
        .collect()
}

pub fn stack_label_for_size(size: usize, group_id: &str) -> String {
    let prefix = match size {
        2 => "DUO",
        3 => "TRIO",
        4 => "4-STACK",
        5 => "5-STACK",
        _ => "STACK",
    };
    format!("{prefix} {group_id}")
}

pub fn skin_is_animated(skin: &Skin) -> bool {
    skin.levels.len() > 1
        || skin.levels.iter().any(|l| l.streamed_video.is_some())
        || skin.chromas.iter().any(|c| c.streamed_video.is_some())
}

fn skin_tier_rank(skin: &Skin) -> u8 {
    if let Some(ref tid) = skin.content_tier_uuid {
        if tid.eq_ignore_ascii_case(TIER_ULTRA_UUID) {
            return 4;
        }
        if tid.eq_ignore_ascii_case(TIER_EXCLUSIVE_UUID) {
            return 3;
        }
        if tid.eq_ignore_ascii_case(TIER_PREMIUM_UUID) {
            return 2;
        }
    }
    if let Some(ref name) = skin.content_tier_name {
        let low = name.to_lowercase();
        if low.contains("ultra") {
            return 4;
        }
        if low.contains("exclusive") {
            return 3;
        }
        if low.contains("premium") {
            return 2;
        }
    }
    if skin_is_animated(skin) {
        1
    } else {
        0
    }
}

/// Returns true for all Premium (purple), Exclusive (orange/limited edition),
/// Ultra (golden), and any animated/upgradable skins (e.g. levels > 1 or preview video).
pub fn is_featured_live_skin(skin: &Skin) -> bool {
    let name_low = skin.display_name.trim().to_lowercase();
    if name_low.is_empty()
        || name_low.starts_with("standard ")
        || name_low.starts_with("random favorite")
    {
        return false;
    }
    skin_tier_rank(skin) >= 2 || skin_is_animated(skin)
}

fn short_tier_label(skin: &Skin, is_animated: bool) -> Option<String> {
    match skin_tier_rank(skin) {
        4 => Some("Ultra".to_string()),
        3 => Some("Exclusive".to_string()),
        2 => Some("Premium".to_string()),
        _ if is_animated => Some("Animated".to_string()),
        _ => skin.content_tier_name.as_deref().and_then(|n| {
            let low = n.to_lowercase();
            if low.contains("deluxe") {
                Some("Deluxe".to_string())
            } else if low.contains("select") {
                Some("Select".to_string())
            } else {
                None
            }
        }),
    }
}

fn find_socket_ci<'a>(
    sockets: &'a HashMap<String, RawCoreLoadoutSocket>,
    target_uuid: &str,
) -> Option<&'a RawCoreLoadoutSocket> {
    sockets
        .iter()
        .find(|(k, v)| {
            k.eq_ignore_ascii_case(target_uuid)
                || v.id.eq_ignore_ascii_case(target_uuid)
                || v.item.type_id.eq_ignore_ascii_case(target_uuid)
        })
        .map(|(_, v)| v)
}

pub fn extract_skins_from_items(
    cache: &LocalCache,
    items: &HashMap<String, RawCoreLoadoutItem>,
) -> Vec<EquippedWeaponSkin> {
    let mut seen_skins: HashSet<String> = HashSet::new();
    let mut featured: Vec<(u8, EquippedWeaponSkin)> = Vec::new();
    let mut fallback_non_standard: Vec<(u8, EquippedWeaponSkin)> = Vec::new();

    for item in items.values() {
        let mut skin_opt = None;
        for preferred_socket in [SKIN_SOCKET_UUID, SKIN_LEVEL_SOCKET_UUID, SKIN_CHROMA_SOCKET_UUID] {
            if let Some(sock) = find_socket_ci(&item.sockets, preferred_socket) {
                if !sock.item.id.trim().is_empty() {
                    if let Ok(Some(s)) = cache.lookup_skin_by_any_uuid_fast(&sock.item.id) {
                        skin_opt = Some(s);
                        break;
                    }
                }
            }
        }
        if skin_opt.is_none() {
            for sock in item.sockets.values() {
                if !sock.item.id.trim().is_empty() {
                    if let Ok(Some(s)) = cache.lookup_skin_by_any_uuid_fast(&sock.item.id) {
                        skin_opt = Some(s);
                        break;
                    }
                }
            }
        }
        if skin_opt.is_none() && !item.id.trim().is_empty() {
            if let Ok(Some(s)) = cache.lookup_skin_by_any_uuid_fast(&item.id) {
                skin_opt = Some(s);
            }
        }
        let Some(skin) = skin_opt else {
            continue;
        };
        let name_low = skin.display_name.trim().to_lowercase();
        if name_low.is_empty()
            || name_low.starts_with("standard ")
            || name_low.starts_with("random favorite")
        {
            continue;
        }
        if !seen_skins.insert(skin.uuid.to_lowercase()) {
            continue;
        }

        let equipped_chroma_icon = find_socket_ci(&item.sockets, SKIN_CHROMA_SOCKET_UUID)
            .and_then(|sock| {
                skin.chromas
                    .iter()
                    .find(|c| c.uuid.eq_ignore_ascii_case(&sock.item.id))
                    .and_then(|c| c.display_icon.clone())
            })
            .or_else(|| {
                item.sockets.values().find_map(|sock| {
                    skin.chromas
                        .iter()
                        .find(|c| c.uuid.eq_ignore_ascii_case(&sock.item.id))
                        .and_then(|c| c.display_icon.clone())
                })
            });
        let is_animated = skin_is_animated(&skin);
        let tier_rank = skin_tier_rank(&skin);
        let tier_name = short_tier_label(&skin, is_animated);

        let equipped = EquippedWeaponSkin {
            weapon_name: skin.weapon_name.clone(),
            skin_name: skin.display_name.clone(),
            skin_icon: equipped_chroma_icon.or_else(|| skin.display_icon.clone()),
            tier_name,
            tier_color: skin.content_tier_color.clone(),
            is_animated,
        };

        if is_featured_live_skin(&skin) {
            featured.push((tier_rank, equipped));
        } else {
            fallback_non_standard.push((tier_rank, equipped));
        }
    }

    let mut out = if !featured.is_empty() {
        featured
    } else {
        fallback_non_standard
    };

    let priority = |w: &str| -> u8 {
        match w {
            "Vandal" => 0,
            "Phantom" => 1,
            "Operator" => 2,
            "Sheriff" => 3,
            "Ghost" => 4,
            "Melee" => 5,
            "Classic" => 6,
            "Guardian" => 7,
            "Spectre" => 8,
            "Bulldog" => 9,
            "Marshal" => 10,
            "Outlaw" => 11,
            "Frenzy" => 12,
            "Shorty" => 13,
            "Stinger" => 14,
            "Bucky" => 15,
            "Judge" => 16,
            "Ares" => 17,
            "Odin" => 18,
            _ => 25,
        }
    };
    out.sort_by(|(rank_a, a), (rank_b, b)| {
        priority(&a.weapon_name)
            .cmp(&priority(&b.weapon_name))
            .then_with(|| rank_b.cmp(rank_a))
            .then_with(|| a.skin_name.cmp(&b.skin_name))
    });
    out.into_iter().map(|(_, skin)| skin).collect()
}

fn entry_items(entry: &RawCoreGameLoadoutEntry) -> &HashMap<String, RawCoreLoadoutItem> {
    if !entry.loadout.items.is_empty() {
        &entry.loadout.items
    } else {
        &entry.items
    }
}

pub fn extract_skins_for_subject(
    cache: &LocalCache,
    loadout_entry: Option<&RawCoreGameLoadoutEntry>,
) -> Vec<EquippedWeaponSkin> {
    let Some(entry) = loadout_entry else {
        return Vec::new();
    };
    extract_skins_from_items(cache, entry_items(entry))
}

pub fn extract_live_gun_map(
    loadout_entry: Option<&RawCoreGameLoadoutEntry>,
) -> HashMap<String, (String, String, String)> {
    let Some(entry) = loadout_entry else {
        return HashMap::new();
    };
    let mut guns = HashMap::new();
    for (item_key, item) in entry_items(entry) {
        let weapon_uuid = if !item.id.trim().is_empty() {
            item.id.trim().to_lowercase()
        } else {
            item_key.trim().to_lowercase()
        };
        if weapon_uuid.is_empty() {
            continue;
        }
        let skin_id = find_socket_ci(&item.sockets, SKIN_SOCKET_UUID)
            .map(|s| s.item.id.trim().to_string())
            .unwrap_or_default();
        let level_id = find_socket_ci(&item.sockets, SKIN_LEVEL_SOCKET_UUID)
            .map(|s| s.item.id.trim().to_string())
            .unwrap_or_default();
        let chroma_id = find_socket_ci(&item.sockets, SKIN_CHROMA_SOCKET_UUID)
            .map(|s| s.item.id.trim().to_string())
            .unwrap_or_default();
        if !skin_id.is_empty() || !level_id.is_empty() || !chroma_id.is_empty() {
            guns.insert(weapon_uuid, (skin_id, level_id, chroma_id));
        }
    }
    guns
}

pub fn extract_live_spray_list(
    loadout_entry: Option<&RawCoreGameLoadoutEntry>,
) -> Vec<(String, String)> {
    let Some(entry) = loadout_entry else {
        return Vec::new();
    };
    let mut seen_slots: HashSet<String> = HashSet::new();
    let mut out = Vec::new();
    for s in &entry.loadout.sprays.spray_selections {
        let slot = s.socket_id.trim().to_string();
        let asset = s.spray_id.trim().to_string();
        if !slot.is_empty() && !asset.is_empty() && seen_slots.insert(slot.to_lowercase()) {
            out.push((slot, asset));
        }
    }
    for e in &entry.loadout.expressions.aes_selections {
        let slot = e.socket_id.trim().to_string();
        let asset = e.asset_id.trim().to_string();
        if !slot.is_empty() && !asset.is_empty() && seen_slots.insert(slot.to_lowercase()) {
            out.push((slot, asset));
        }
    }
    out
}

pub fn friendly_server_pod(game_pod_id: &str) -> String {
    let pod = game_pod_id.trim().to_lowercase();
    if pod.is_empty() {
        return String::new();
    }
    if pod.contains("bom1") || pod.contains("mumbai") {
        "Mumbai"
    } else if pod.contains("sgp1") || pod.contains("ap-southeast-1") || pod.contains("singapore") {
        "Singapore"
    } else if pod.contains("tok1") || pod.contains("ap-northeast-1") || pod.contains("tokyo") {
        "Tokyo"
    } else if pod.contains("seo1") || pod.contains("ap-northeast-2") || pod.contains("seoul") {
        "Seoul"
    } else if pod.contains("syd1") || pod.contains("ap-southeast-2") || pod.contains("sydney") {
        "Sydney"
    } else if pod.contains("hkg1") || pod.contains("ap-east-1") || pod.contains("hongkong") {
        "Hong Kong"
    } else if pod.contains("fra1") || pod.contains("eu-central-1") || pod.contains("frankfurt") {
        "Frankfurt"
    } else if pod.contains("lon1") || pod.contains("eu-west-2") || pod.contains("london") {
        "London"
    } else if pod.contains("par1") || pod.contains("eu-west-3") || pod.contains("paris") {
        "Paris"
    } else if pod.contains("sto1") || pod.contains("eu-north-1") || pod.contains("stockholm") {
        "Stockholm"
    } else if pod.contains("mad1") || pod.contains("eu-south-2") || pod.contains("madrid") {
        "Madrid"
    } else if pod.contains("waw1") || pod.contains("warsaw") {
        "Warsaw"
    } else if pod.contains("ist1") || pod.contains("istanbul") {
        "Istanbul"
    } else if pod.contains("bah1") || pod.contains("bahrain") {
        "Bahrain"
    } else if pod.contains("dub1") || pod.contains("dubai") {
        "Dubai"
    } else if pod.contains("cpt1") || pod.contains("capetown") {
        "Cape Town"
    } else if pod.contains("ash1") || pod.contains("us-east-1") {
        "N. Virginia"
    } else if pod.contains("chi1") || pod.contains("us-east-2") || pod.contains("ord") {
        "Illinois"
    } else if pod.contains("atl1") {
        "Georgia"
    } else if pod.contains("dfw1") {
        "Texas"
    } else if pod.contains("mia1") {
        "Miami"
    } else if pod.contains("sjc1") || pod.contains("us-west-1") {
        "N. California"
    } else if pod.contains("pdx1") || pod.contains("us-west-2") {
        "Oregon"
    } else if pod.contains("mex1") {
        "Mexico City"
    } else if pod.contains("bog1") {
        "Bogota"
    } else if pod.contains("scl1") {
        "Santiago"
    } else if pod.contains("gru1") || pod.contains("sa-east-1") {
        "Sao Paulo"
    } else {
        ""
    }
    .to_string()
}

pub fn resolve_coregame_loadout_for_player<'a>(
    loadouts: &'a [RawCoreGameLoadoutEntry],
    player_subject: &str,
    player_character_id: &str,
    player_index: usize,
    total_players: usize,
) -> Option<&'a RawCoreGameLoadoutEntry> {
    let sub_low = player_subject.trim().to_lowercase();
    if !sub_low.is_empty() {
        if let Some(found) = loadouts.iter().find(|e| {
            e.subject.trim().eq_ignore_ascii_case(&sub_low)
                || e.loadout.subject.trim().eq_ignore_ascii_case(&sub_low)
        }) {
            return Some(found);
        }
    }
    let char_low = player_character_id.trim().to_lowercase();
    if !char_low.is_empty() {
        let mut char_matches = loadouts
            .iter()
            .filter(|e| e.character_id.trim().eq_ignore_ascii_case(&char_low));
        if let Some(first) = char_matches.next() {
            if char_matches.next().is_none() {
                return Some(first);
            }
        }
        if loadouts.len() == total_players {
            if let Some(entry) = loadouts.get(player_index) {
                if entry.character_id.trim().eq_ignore_ascii_case(&char_low) {
                    return Some(entry);
                }
            }
        }
    }
    None
}

pub fn friendly_queue_label(queue_id: &str) -> String {
    match queue_id.trim().to_lowercase().as_str() {
        "competitive" => "Competitive".to_string(),
        "unrated" => "Unrated".to_string(),
        "swiftplay" => "Swiftplay".to_string(),
        "spikerush" => "Spike Rush".to_string(),
        "deathmatch" => "Deathmatch".to_string(),
        "hurm" => "Team Deathmatch".to_string(),
        "ggteam" => "Escalation".to_string(),
        "premier" => "Premier".to_string(),
        "newmap" => "New Map".to_string(),
        "custom" => "Custom Game".to_string(),
        other if !other.is_empty() => {
            let mut chars = other.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => "Match".to_string(),
            }
        }
        _ => "Match".to_string(),
    }
}

fn format_party_error_notification(error_type: &str) -> Option<String> {
    let trimmed = error_type.trim();
    if trimmed.is_empty() {
        return None;
    }
    let msg = match trimmed.to_uppercase().as_str() {
        "COMPETITIVE_RANK_DISPARITY" | "RANK_DISPARITY" => {
            "Rank disparity in party is too wide for Competitive."
        }
        "QUEUE_PENALTY" | "RESTRICTED_FROM_QUEUE" | "PLAYER_RESTRICTED" => {
            "A party member currently has a queue restriction."
        }
        "PARTY_SIZE_INVALID" | "INVALID_PARTY_SIZE" => {
            "Current party size is not allowed for this queue."
        }
        "ACCOUNT_LEVEL_TOO_LOW" => "A party member has not reached Level 20 for Competitive.",
        "INELIGIBLE_QUEUE" => "Party is not eligible for the selected queue.",
        _ => return Some(trimmed.replace('_', " ")),
    };
    Some(msg.to_string())
}

fn parse_queue_elapsed_secs(queue_entry_time: &str, is_in_queue: bool) -> i64 {
    if !is_in_queue {
        return 0;
    }
    let trimmed = queue_entry_time.trim();
    if trimmed.is_empty() || trimmed.starts_with("0001-") {
        return 0;
    }
    DateTime::parse_from_rfc3339(trimmed)
        .map(|dt| (Utc::now().timestamp() - dt.timestamp()).max(0))
        .unwrap_or(0)
}

pub fn build_party_snapshot_from_parts(
    raw: RawPartyResp,
    members: Vec<PartyMemberInfo>,
) -> PartySnapshot {
    let state_upper = raw.state.trim().to_uppercase();
    let is_in_queue = matches!(state_upper.as_str(), "MATCHMAKING" | "STARTING_MATCHMAKING");
    let is_match_starting = matches!(
        state_upper.as_str(),
        "MATCHMADE_GAME_STARTING" | "CUSTOM_GAME_STARTING"
    );
    let queue_id = raw
        .matchmaking_data
        .as_ref()
        .map(|d| d.queue_id.trim().to_lowercase())
        .filter(|q| !q.is_empty())
        .unwrap_or_else(|| {
            if state_upper == "CUSTOM_GAME_SETUP" {
                "custom".to_string()
            } else {
                "unrated".to_string()
            }
        });
    let is_custom_game = state_upper == "CUSTOM_GAME_SETUP" || queue_id == "custom";
    let is_lobby_idle = state_upper.is_empty() || state_upper == "DEFAULT";

    let me = members.iter().find(|m| m.is_me);
    let am_i_owner = me.map(|m| m.is_owner).unwrap_or(false);
    let all_ready = !members.is_empty() && members.iter().all(|m| m.is_ready);
    let party_size = members.len();
    let is_open = raw.accessibility.trim().eq_ignore_ascii_case("OPEN");
    let invite_code = raw.invite_code.and_then(|c| {
        let t = c.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    let error_message = raw
        .error_notification
        .as_ref()
        .and_then(|e| format_party_error_notification(&e.error_type));

    let raw_eligible: Vec<String> = {
        let mut seen = HashSet::new();
        raw.eligible_queues
            .into_iter()
            .map(|q| q.trim().to_lowercase())
            .filter(|q| !q.is_empty() && q != "custom" && seen.insert(q.clone()))
            .collect()
    };

    let standard_order = [
        "competitive",
        "unrated",
        "swiftplay",
        "spikerush",
        "deathmatch",
        "hurm",
    ];
    let mut available_queues: Vec<String> = Vec::new();
    let mut avail_seen = HashSet::new();
    for q in standard_order {
        avail_seen.insert(q.to_string());
        available_queues.push(q.to_string());
    }
    for q in &raw_eligible {
        if avail_seen.insert(q.clone()) {
            available_queues.push(q.clone());
        }
    }
    if !is_custom_game && !queue_id.is_empty() && avail_seen.insert(queue_id.clone()) {
        available_queues.push(queue_id.clone());
    }

    let eligible_queues: Vec<String> = if !raw_eligible.is_empty() {
        raw_eligible
    } else if error_message.is_some() {
        Vec::new()
    } else {
        available_queues
            .iter()
            .filter(|q| match q.as_str() {
                "deathmatch" => party_size <= 1,
                "competitive" => party_size != 4 && party_size <= 5,
                _ => party_size <= 5,
            })
            .cloned()
            .collect()
    };

    let can_change_queue = am_i_owner && is_lobby_idle && !is_custom_game;
    let change_queue_disabled_reason = if is_match_starting {
        Some("Match found — loading Agent Select".to_string())
    } else if is_in_queue {
        Some("Cancel queue first to switch mode".to_string())
    } else if is_custom_game {
        Some("Leave custom lobby to switch matchmade mode".to_string())
    } else if !am_i_owner {
        Some("Only the party leader can switch queue mode".to_string())
    } else {
        None
    };

    let current_queue_eligible =
        !is_custom_game && eligible_queues.iter().any(|q| q.eq_ignore_ascii_case(&queue_id));
    let can_leave_queue = is_in_queue && !is_match_starting;
    let can_start_queue = am_i_owner
        && is_lobby_idle
        && !is_custom_game
        && all_ready
        && current_queue_eligible
        && error_message.is_none();

    let queue_action_disabled_reason = if is_match_starting {
        Some("Match found — launching Agent Select...".to_string())
    } else if is_in_queue {
        None
    } else if is_custom_game {
        Some("Start custom games from the Valorant PC client".to_string())
    } else if !am_i_owner {
        Some("Only the party leader can start matchmaking".to_string())
    } else if !all_ready {
        Some("Waiting for all party members to ready up".to_string())
    } else if let Some(ref err) = error_message {
        Some(err.clone())
    } else if !current_queue_eligible {
        Some(format!(
            "Party is not eligible for {} right now",
            friendly_queue_label(&queue_id)
        ))
    } else {
        None
    };

    let can_toggle_ready = !is_in_queue && !is_match_starting;
    let can_toggle_accessibility = am_i_owner && !is_in_queue && !is_match_starting;
    let can_generate_invite_code =
        am_i_owner && is_open && !is_in_queue && !is_match_starting && invite_code.is_none();
    let can_disable_invite_code =
        am_i_owner && !is_in_queue && !is_match_starting && invite_code.is_some();
    let can_join_by_code = !is_in_queue && !is_match_starting;
    let queue_elapsed_secs = parse_queue_elapsed_secs(&raw.queue_entry_time, is_in_queue);

    PartySnapshot {
        party_id: raw.id,
        state: raw.state,
        accessibility: raw.accessibility,
        queue_id,
        invite_code,
        is_in_queue,
        is_match_starting,
        queue_elapsed_secs,
        error_message,
        members,
        available_queues,
        eligible_queues,
        can_change_queue,
        change_queue_disabled_reason,
        can_start_queue,
        can_leave_queue,
        queue_action_disabled_reason,
        can_toggle_ready,
        can_toggle_accessibility,
        can_generate_invite_code,
        can_disable_invite_code,
        can_join_by_code,
    }
}

pub fn validate_party_change_queue(
    snap: &PartySnapshot,
    target_queue_id: &str,
) -> Result<(), MyValError> {
    let target = target_queue_id.trim().to_lowercase();
    if target.is_empty() {
        return Err(MyValError::InvalidInput("Queue ID cannot be empty".to_string()));
    }
    if !snap.can_change_queue {
        return Err(MyValError::InvalidInput(
            snap.change_queue_disabled_reason
                .clone()
                .unwrap_or_else(|| "Cannot switch queue in current party state".to_string()),
        ));
    }
    if snap.queue_id.eq_ignore_ascii_case(&target) {
        return Err(MyValError::InvalidInput(format!(
            "Already set to {}",
            friendly_queue_label(&target)
        )));
    }
    if !snap.eligible_queues.is_empty()
        && !snap
            .eligible_queues
            .iter()
            .any(|q| q.eq_ignore_ascii_case(&target))
    {
        return Err(MyValError::InvalidInput(format!(
            "{} is not eligible for your current party",
            friendly_queue_label(&target)
        )));
    }
    Ok(())
}

pub fn validate_party_join_queue(snap: &PartySnapshot) -> Result<(), MyValError> {
    if !snap.can_start_queue {
        return Err(MyValError::InvalidInput(
            snap.queue_action_disabled_reason
                .clone()
                .unwrap_or_else(|| "Cannot start matchmaking right now".to_string()),
        ));
    }
    Ok(())
}

pub fn validate_party_leave_queue(snap: &PartySnapshot) -> Result<(), MyValError> {
    if !snap.can_leave_queue {
        return Err(MyValError::InvalidInput(
            if snap.is_match_starting {
                "Match already found — cannot cancel queue".to_string()
            } else {
                "Party is not currently searching in queue".to_string()
            },
        ));
    }
    Ok(())
}

pub fn validate_party_set_ready(snap: &PartySnapshot) -> Result<(), MyValError> {
    if !snap.can_toggle_ready {
        return Err(MyValError::InvalidInput(
            if snap.is_match_starting {
                "Match found — cannot change ready state".to_string()
            } else {
                "Leave queue before changing ready state".to_string()
            },
        ));
    }
    Ok(())
}

pub fn validate_party_set_accessibility(snap: &PartySnapshot) -> Result<(), MyValError> {
    if !snap.can_toggle_accessibility {
        return Err(MyValError::InvalidInput(
            if snap.is_in_queue || snap.is_match_starting {
                "Leave queue before changing party privacy".to_string()
            } else {
                "Only the party leader can change party privacy".to_string()
            },
        ));
    }
    Ok(())
}

pub fn validate_party_generate_invite(snap: &PartySnapshot) -> Result<(), MyValError> {
    if !snap.can_generate_invite_code {
        return Err(MyValError::InvalidInput(
            if !snap.accessibility.eq_ignore_ascii_case("OPEN") {
                "Set party to OPEN before generating an invite code".to_string()
            } else if snap.is_in_queue || snap.is_match_starting {
                "Leave queue before managing invite codes".to_string()
            } else {
                "Only the party leader can generate an invite code".to_string()
            },
        ));
    }
    Ok(())
}

pub fn validate_party_disable_invite(snap: &PartySnapshot) -> Result<(), MyValError> {
    if !snap.can_disable_invite_code {
        return Err(MyValError::InvalidInput(
            if snap.is_in_queue || snap.is_match_starting {
                "Leave queue before disabling invite codes".to_string()
            } else {
                "Only the party leader can disable invite codes".to_string()
            },
        ));
    }
    Ok(())
}

pub const FAIR_PLAY_LOCK_DELAY_SECS: u64 = 15;

pub fn validate_pregame_action_delay(elapsed_secs: u64, lock: bool) -> Result<(), MyValError> {
    if elapsed_secs < FAIR_PLAY_LOCK_DELAY_SECS {
        let wait = FAIR_PLAY_LOCK_DELAY_SECS - elapsed_secs;
        let action = if lock { "locking" } else { "hovering" };
        return Err(MyValError::InvalidInput(format!(
            "Wait {}s before {} — MyVal strictly disables instapicking and instalocking for any unfair advantage.",
            wait, action
        )));
    }
    Ok(())
}

pub fn validate_pregame_lock_delay(elapsed_secs: u64) -> Result<(), MyValError> {
    validate_pregame_action_delay(elapsed_secs, true)
}

pub fn validate_pregame_agent_action(
    snap: &PreGameSnapshot,
    agent_uuid: &str,
) -> Result<(), MyValError> {
    let target = agent_uuid.trim();
    if target.is_empty() {
        return Err(MyValError::InvalidInput("Agent ID cannot be empty".to_string()));
    }
    if !snap.can_select_agent {
        return Err(MyValError::InvalidInput(
            if snap.my_selection_state.eq_ignore_ascii_case("locked") {
                "You have already locked in your agent".to_string()
            } else {
                "Agent select is no longer accepting picks".to_string()
            },
        ));
    }
    if snap
        .locked_agent_uuids
        .iter()
        .any(|id| id.eq_ignore_ascii_case(target))
    {
        return Err(MyValError::InvalidInput(
            "That agent is already locked by a teammate".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_pregame_quit(snap: &PreGameSnapshot) -> Result<(), MyValError> {
    if !snap.can_quit_match {
        return Err(MyValError::InvalidInput(
            "Cannot quit match — agent select has already finished".to_string(),
        ));
    }
    Ok(())
}

pub async fn build_party_snapshot(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    cache: &LocalCache,
    session: &RiotSession,
    headers: &HeaderMap,
    raw: RawPartyResp,
) -> PartySnapshot {
    let puuids: Vec<String> = raw.members.iter().map(|m| m.subject.clone()).collect();
    let mut names = fetch_names_batched(http, limiter, cache, &session.shard, headers, &puuids).await;
    if !session.game_name.trim().is_empty() {
        names
            .entry(session.puuid.to_lowercase())
            .or_insert_with(|| (session.game_name.clone(), session.tag_line.clone()));
    }

    let members = raw
        .members
        .iter()
        .map(|m| {
            let is_me = m.subject.eq_ignore_ascii_case(&session.puuid);
            let incognito = m.player_identity.incognito;
            cache.cache_player_identity_meta(
                &m.subject,
                Some(&m.player_identity.player_card_id),
                None,
                Some(m.player_identity.account_level),
            );
            let display_name =
                format_display_name(&m.subject, &names, incognito, is_me, "Party Player");
            let card_icon = cache
                .lookup_player_card(&m.player_identity.player_card_id)
                .and_then(|c| c.small_art.or(c.wide_art));
            let tier = if m.competitive_tier >= 3 {
                m.competitive_tier
            } else {
                0
            };
            PartyMemberInfo {
                puuid: m.subject.clone(),
                display_name,
                is_ready: m.is_ready,
                is_owner: m.is_owner,
                is_me,
                account_level: visible_account_level(
                    m.player_identity.account_level,
                    m.player_identity.hide_account_level,
                    is_me,
                ),
                player_card_icon: card_icon,
                platform: if m.platform_info.platform_type.is_empty() {
                    "PC".to_string()
                } else {
                    m.platform_info.platform_type.to_uppercase()
                },
                current_rank: build_rank_info(cache, tier, 0),
            }
        })
        .collect();

    build_party_snapshot_from_parts(raw, members)
}

fn infer_pregame_queue_id(raw: &RawPreGameMatchResp) -> String {
    let q = raw.queue_id.trim();
    if !q.is_empty() {
        return q.to_lowercase();
    }
    if raw.is_ranked {
        return "competitive".to_string();
    }
    let mode_lower = raw.mode.to_lowercase();
    if mode_lower.contains("swiftplay") {
        "swiftplay".to_string()
    } else if mode_lower.contains("quickbomb") {
        "spikerush".to_string()
    } else if mode_lower.contains("deathmatch") {
        "deathmatch".to_string()
    } else if mode_lower.contains("hurm") {
        "hurm".to_string()
    } else if mode_lower.contains("ggteam") {
        "ggteam".to_string()
    } else if mode_lower.contains("bomb") {
        "unrated".to_string()
    } else {
        "match".to_string()
    }
}

pub async fn build_pregame_snapshot(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    cache: &LocalCache,
    session: &RiotSession,
    headers: &HeaderMap,
    match_id: String,
    pg_match: RawPreGameMatchResp,
) -> PreGameSnapshot {
    let ally_team_id = pg_match
        .ally_team
        .as_ref()
        .map(|t| t.team_id.clone())
        .unwrap_or_default();
    let ally_side = match ally_team_id.trim().to_lowercase().as_str() {
        "red" => "Attacking".to_string(),
        "blue" => "Defending".to_string(),
        _ => String::new(),
    };
    let queue_id = infer_pregame_queue_id(&pg_match);
    let phase_time_remaining_secs = (pg_match.phase_time_remaining_ns / 1_000_000_000).max(0);
    let enemy_team_size = pg_match.enemy_team_size.max(0);
    let enemy_team_lock_count = pg_match.enemy_team_lock_count.max(0);
    let map_meta = cache.lookup_map_by_url_or_id(&pg_match.map_id);
    let pregame_state = pg_match.pregame_state;

    let ally_players = pg_match
        .ally_team
        .map(|t| t.players)
        .unwrap_or_default();
    let puuids: Vec<String> = ally_players.iter().map(|p| p.subject.clone()).collect();

    let mmr_map: Mutex<HashMap<String, PlayerMmrSummary>> = Mutex::new(HashMap::new());
    let history_map: Mutex<HashMap<String, Vec<String>>> = Mutex::new(HashMap::new());
    let (mut names, pg_loadouts_res, my_live_party_puuids, _) = tokio::join!(
        fetch_names_batched(http, limiter, cache, &session.shard, headers, &puuids),
        get_pregame_loadouts(http, limiter, session, headers, &match_id),
        fetch_my_live_party_puuids(http, limiter, session, headers),
        async {
            let tasks = puuids.iter().map(|puuid| {
                let mmr_ref = &mmr_map;
                let hist_ref = &history_map;
                let hdr = headers.clone();
                async move {
                    let (mmr, hist) = tokio::join!(
                        fetch_player_mmr(http, limiter, cache, &session.shard, &hdr, puuid),
                        fetch_recent_match_ids(http, limiter, cache, &session.shard, &hdr, puuid, 8)
                    );
                    mmr_ref.lock().await.insert(puuid.to_lowercase(), mmr);
                    hist_ref.lock().await.insert(puuid.clone(), hist);
                }
            });
            futures_util::future::join_all(tasks).await;
        }
    );
    if !session.game_name.trim().is_empty() {
        names
            .entry(session.puuid.to_lowercase())
            .or_insert_with(|| (session.game_name.clone(), session.tag_line.clone()));
    }
    let mmr_done = mmr_map.into_inner();
    let hist_done = history_map.into_inner();

    let team_key = if ally_team_id.trim().is_empty() {
        "Ally".to_string()
    } else {
        ally_team_id.clone()
    };
    let ally_teams: Vec<(String, String)> = ally_players
        .iter()
        .map(|p| (p.subject.clone(), team_key.clone()))
        .collect();
    let detected_stacks = resolve_detected_party_groups(
        http,
        limiter,
        cache,
        session,
        headers,
        &ally_teams,
        &hist_done,
        my_live_party_puuids.as_deref(),
        Some(&match_id),
    )
    .await;
    let mut stack_by_puuid: HashMap<String, &PartyGroup> = HashMap::new();
    for g in &detected_stacks {
        for p in &g.member_puuids {
            stack_by_puuid.insert(p.to_lowercase(), g);
        }
    }

    let pg_loadouts_vec: Vec<RawCoreLoadoutDetail> = pg_loadouts_res
        .ok()
        .flatten()
        .filter(|r| r.loadouts_valid || r.loadouts.iter().any(|l| !l.items.is_empty()))
        .map(|r| r.loadouts)
        .unwrap_or_default();

    let total_allies = ally_players.len();
    let mut allies = Vec::with_capacity(total_allies);
    let mut locked_agent_uuids = Vec::new();
    let mut my_agent_uuid = None;
    let mut my_selection_state = "selecting".to_string();

    for (idx, p) in ally_players.into_iter().enumerate() {
        let is_me = p.subject.eq_ignore_ascii_case(&session.puuid);
        cache.cache_player_identity_meta(
            &p.subject,
            Some(&p.player_identity.player_card_id),
            None,
            Some(p.player_identity.account_level),
        );
        let has_char = !p.character_id.trim().is_empty();
        let agent = if has_char {
            cache.lookup_agent(&p.character_id)
        } else {
            None
        };
        let agent_name = agent
            .as_ref()
            .map(|a| a.display_name.clone())
            .unwrap_or_else(|| "Selecting agent...".to_string());
        let raw_sel = p.character_selection_state.trim().to_lowercase();
        let selection_state = if raw_sel == "locked" {
            "locked".to_string()
        } else if has_char {
            "hovering".to_string()
        } else {
            "selecting".to_string()
        };
        if selection_state == "locked" && has_char {
            locked_agent_uuids.push(p.character_id.clone());
        }
        if is_me {
            my_selection_state = selection_state.clone();
            if has_char {
                my_agent_uuid = Some(p.character_id.clone());
            }
        }
        let mmr = mmr_done
            .get(&p.subject.to_lowercase())
            .cloned()
            .unwrap_or_default();
        let recent_outcomes = resolve_recent_outcomes(cache, &p.subject, is_me, &mmr);
        let stack_opt = stack_by_puuid.get(&p.subject.to_lowercase());
        let pg_loadout_opt = pg_loadouts_vec
            .iter()
            .find(|l| !l.subject.trim().is_empty() && l.subject.eq_ignore_ascii_case(&p.subject))
            .or_else(|| {
                if pg_loadouts_vec.len() == total_allies {
                    pg_loadouts_vec.get(idx)
                } else {
                    None
                }
            });
        let equipped_skins = pg_loadout_opt
            .map(|l| extract_skins_from_items(cache, &l.items))
            .unwrap_or_default();

        allies.push(PreGameAlly {
            puuid: p.subject.clone(),
            display_name: format_display_name(
                &p.subject,
                &names,
                p.player_identity.incognito,
                is_me,
                &agent_name,
            ),
            agent_uuid: p.character_id,
            agent_name,
            agent_icon: agent.and_then(|a| a.display_icon),
            selection_state,
            account_level: visible_account_level(
                p.player_identity.account_level,
                p.player_identity.hide_account_level,
                is_me,
            ),
            hide_account_level: p.player_identity.hide_account_level,
            incognito: p.player_identity.incognito,
            is_me,
            current_rank: if mmr.current_rank.tier >= 3 {
                mmr.current_rank
            } else {
                build_rank_info(cache, p.competitive_tier, 0)
            },
            peak_rank: mmr.peak_rank,
            recent_outcomes,
            party_group_id: stack_opt.map(|g| g.group_id.clone()),
            party_group_label: stack_opt.map(|g| g.label.clone()),
            party_group_size: stack_opt.map(|g| g.member_puuids.len() as u32).unwrap_or(1),
            equipped_skins,
        });
    }

    let select_active = pregame_state.is_empty()
        || pregame_state.eq_ignore_ascii_case("character_select_active");
    let can_select_agent = select_active && !my_selection_state.eq_ignore_ascii_case("locked");
    let can_quit_match = select_active;

    PreGameSnapshot {
        match_id,
        map_name: map_meta
            .as_ref()
            .map(|m| m.display_name.clone())
            .unwrap_or_else(|| "Agent Select".to_string()),
        map_splash: map_meta.and_then(|m| m.list_view_icon.or(m.splash)),
        queue_id,
        pregame_state,
        ally_team_id,
        ally_side,
        phase_time_remaining_secs,
        enemy_team_size,
        enemy_team_lock_count,
        can_select_agent,
        can_quit_match,
        locked_agent_uuids,
        my_agent_uuid,
        my_selection_state,
        allies,
        detected_stacks,
    }
}

fn outcomes_from_rr_trend(trend: &[CompetitiveMatchUpdate]) -> Vec<i8> {
    trend
        .iter()
        .take(5)
        .map(|u| {
            if u.rr_delta > 0 {
                1
            } else if u.rr_delta < 0 {
                -1
            } else {
                0
            }
        })
        .collect()
}

fn outcomes_from_competitive_matches(matches: &[MatchHistoryItem]) -> Vec<i8> {
    matches
        .iter()
        .filter(|m| m.queue_id.trim().eq_ignore_ascii_case("competitive"))
        .take(5)
        .map(|m| {
            if m.my_team_rounds == m.enemy_team_rounds {
                0
            } else if m.won {
                1
            } else {
                -1
            }
        })
        .collect()
}

/// Resolves up to 5 recent Competitive match outcomes (`1` win, `-1` loss, `0` draw).
/// Never falls back to Unrated, Swiftplay, Deathmatch, or other non-competitive queues.
pub fn resolve_recent_outcomes(
    cache: &LocalCache,
    puuid: &str,
    is_me: bool,
    mmr: &PlayerMmrSummary,
) -> Vec<i8> {
    if !mmr.recent_outcomes.is_empty() {
        return mmr.recent_outcomes.iter().take(5).copied().collect();
    }
    if !mmr.recent_results.is_empty() {
        return mmr
            .recent_results
            .iter()
            .take(5)
            .map(|&won| if won { 1 } else { -1 })
            .collect();
    }
    if is_me {
        if let Some(career) = cache.get_cached_career(puuid, 10 * 60) {
            if !career.rr_trend.is_empty() {
                return outcomes_from_rr_trend(&career.rr_trend);
            }
            let comp_only = outcomes_from_competitive_matches(&career.recent_matches);
            if !comp_only.is_empty() {
                return comp_only;
            }
        }
    }
    if let Some(prof) = cache.get_cached_player_profile(puuid, 15 * 60) {
        if !prof.rr_trend.is_empty() {
            return outcomes_from_rr_trend(&prof.rr_trend);
        }
        let comp_only = outcomes_from_competitive_matches(&prof.recent_matches);
        if !comp_only.is_empty() {
            return comp_only;
        }
    }
    Vec::new()
}

pub async fn poll_live_state(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    cache: &LocalCache,
    session: &RiotSession,
) -> Result<LiveState, MyValError> {
    let client_version = cache.riot_client_version();
    let headers = build_riot_headers(session, &client_version)?;

    // 1. Core-Game first
    if let Some(cg_player) = get_coregame_player(http, limiter, session, &headers).await? {
        let match_id = cg_player.match_id;
        let (cg_match, cg_loadouts) = tokio::join!(
            get_coregame_match(http, limiter, session, &headers, &match_id),
            get_coregame_loadouts(http, limiter, session, &headers, &match_id)
        );
        let cg_match = cg_match?;
        let raw_loadouts: Vec<RawCoreGameLoadoutEntry> = cg_loadouts
            .ok()
            .flatten()
            .map(|r| r.loadouts)
            .unwrap_or_default();

        let puuids: Vec<String> = cg_match.players.iter().map(|p| p.subject.clone()).collect();
        let mut names = fetch_names_batched(http, limiter, cache, &session.shard, &headers, &puuids).await;
        if !session.game_name.trim().is_empty() {
            names
                .entry(session.puuid.to_lowercase())
                .or_insert_with(|| (session.game_name.clone(), session.tag_line.clone()));
        }

        let mmr_map: Mutex<HashMap<String, PlayerMmrSummary>> = Mutex::new(HashMap::new());
        let history_map: Mutex<HashMap<String, Vec<String>>> = Mutex::new(HashMap::new());

        let tasks = puuids.iter().map(|puuid| {
            let mmr_ref = &mmr_map;
            let hist_ref = &history_map;
            let hdr = headers.clone();
            async move {
                let (mmr, hist) = tokio::join!(
                    fetch_player_mmr(http, limiter, cache, &session.shard, &hdr, puuid),
                    fetch_recent_match_ids(http, limiter, cache, &session.shard, &hdr, puuid, 8)
                );
                mmr_ref.lock().await.insert(puuid.to_lowercase(), mmr);
                hist_ref.lock().await.insert(puuid.clone(), hist);
            }
        });
        let (_, my_live_party_puuids) = tokio::join!(
            futures_util::future::join_all(tasks),
            fetch_my_live_party_puuids(http, limiter, session, &headers),
        );

        let mmr_done = mmr_map.into_inner();
        let hist_done = history_map.into_inner();

        let player_teams: Vec<(String, String)> = cg_match
            .players
            .iter()
            .map(|p| (p.subject.clone(), p.team_id.clone()))
            .collect();
        let detected_stacks = resolve_detected_party_groups(
            http,
            limiter,
            cache,
            session,
            &headers,
            &player_teams,
            &hist_done,
            my_live_party_puuids.as_deref(),
            Some(&match_id),
        )
        .await;

        let mut stack_by_puuid: HashMap<String, &PartyGroup> = HashMap::new();
        for g in &detected_stacks {
            for p in &g.member_puuids {
                stack_by_puuid.insert(p.to_lowercase(), g);
            }
        }

        let my_team_id = cg_match
            .players
            .iter()
            .find(|p| p.subject.eq_ignore_ascii_case(&session.puuid))
            .map(|p| p.team_id.clone())
            .unwrap_or_else(|| "Blue".to_string());

        let map_meta = cache.lookup_map_by_url_or_id(&cg_match.map_id);
        let mut my_team = Vec::new();
        let mut enemy_team = Vec::new();
        let mut tier_sum = 0i32;
        let mut ranked_count = 0i32;

        let total_players = cg_match.players.len();
        for (idx, p) in cg_match.players.into_iter().enumerate() {
            let is_me = p.subject.eq_ignore_ascii_case(&session.puuid);
            cache.cache_player_identity_meta(
                &p.subject,
                Some(&p.player_identity.player_card_id),
                None,
                Some(p.player_identity.account_level),
            );
            let agent = cache.lookup_agent(&p.character_id);
            let agent_name = agent
                .as_ref()
                .map(|a| a.display_name.clone())
                .unwrap_or_else(|| "Agent".to_string());
            let agent_icon = agent.and_then(|a| a.display_icon);
            let card_icon = cache
                .lookup_player_card(&p.player_identity.player_card_id)
                .and_then(|c| c.small_art.or(c.wide_art));
            let mmr = mmr_done
                .get(&p.subject.to_lowercase())
                .cloned()
                .unwrap_or_default();
            let recent_outcomes = resolve_recent_outcomes(cache, &p.subject, is_me, &mmr);
            let recent_results: Vec<bool> = recent_outcomes.iter().map(|&o| o > 0).collect();
            if mmr.current_rank.tier >= 3 {
                tier_sum += mmr.current_rank.tier;
                ranked_count += 1;
            }
            let stack_opt = stack_by_puuid.get(&p.subject.to_lowercase());
            let loadout_entry = resolve_coregame_loadout_for_player(
                &raw_loadouts,
                &p.subject,
                &p.character_id,
                idx,
                total_players,
            );
            if is_me {
                let live_guns = extract_live_gun_map(loadout_entry);
                let live_sprays = extract_live_spray_list(loadout_entry);
                if !live_guns.is_empty() || !live_sprays.is_empty() {
                    let _ = cache.put_live_match_guns(
                        &session.puuid,
                        &CachedLiveMatchGuns {
                            match_id: match_id.clone(),
                            guns: live_guns,
                            sprays: live_sprays,
                        },
                    );
                }
            }
            let scouted = ScoutedPlayer {
                puuid: p.subject.clone(),
                display_name: format_display_name(
                    &p.subject,
                    &names,
                    p.player_identity.incognito,
                    is_me,
                    &agent_name,
                ),
                team_id: p.team_id.clone(),
                agent_uuid: p.character_id,
                agent_name,
                agent_icon,
                player_card_icon: card_icon,
                account_level: visible_account_level(
                    p.player_identity.account_level,
                    p.player_identity.hide_account_level,
                    is_me,
                ),
                incognito: p.player_identity.incognito,
                hide_account_level: p.player_identity.hide_account_level,
                is_me,
                current_rank: mmr.current_rank,
                peak_rank: mmr.peak_rank,
                recent_results,
                recent_outcomes,
                party_group_id: stack_opt.map(|g| g.group_id.clone()),
                party_group_label: stack_opt.map(|g| g.label.clone()),
                party_group_size: stack_opt.map(|g| g.member_puuids.len() as u32).unwrap_or(1),
                equipped_skins: extract_skins_for_subject(cache, loadout_entry),
            };
            if scouted.team_id.eq_ignore_ascii_case(&my_team_id) {
                my_team.push(scouted);
            } else {
                enemy_team.push(scouted);
            }
        }

        let avg_tier = if ranked_count > 0 {
            (tier_sum as f64 / ranked_count as f64).round() as i32
        } else {
            0
        };
        let avg_rank_name = build_rank_info(cache, avg_tier, 0).tier_name;
        let server_name = friendly_server_pod(&cg_match.game_pod_id);
        let my_starting_side = if my_team_id.eq_ignore_ascii_case("Red") {
            "Attacking".to_string()
        } else if my_team_id.eq_ignore_ascii_case("Blue") {
            "Defending".to_string()
        } else {
            String::new()
        };

        return Ok(LiveState::CoreGame(CoreGameSnapshot {
            match_id,
            map_name: map_meta
                .as_ref()
                .map(|m| m.display_name.clone())
                .unwrap_or_else(|| "Active Match".to_string()),
            map_splash: map_meta.and_then(|m| m.list_view_icon.or(m.splash)),
            queue_id: cg_match
                .matchmaking_data
                .map(|d| d.queue_id)
                .filter(|q| !q.is_empty())
                .unwrap_or_else(|| "competitive".to_string()),
            provisioning_flow: cg_match.provisioning_flow,
            server_name,
            my_starting_side,
            average_lobby_rank_name: avg_rank_name,
            my_team,
            enemy_team,
            detected_stacks,
        }));
    }

    // 2. Pre-Game (Allies ONLY)
    if let Some(pg_player) = get_pregame_player(http, limiter, session, &headers).await? {
        let match_id = pg_player.match_id;
        let pg_match = get_pregame_match(http, limiter, session, &headers, &match_id).await?;
        let snap = build_pregame_snapshot(
            http,
            limiter,
            cache,
            session,
            &headers,
            match_id,
            pg_match,
        )
        .await;
        return Ok(LiveState::PreGame(snap));
    }

    // 3. Party
    if let Some(party_player) = get_party_player(http, limiter, session, &headers).await? {
        cache.clear_live_match_guns(&session.puuid);
        let raw_party = get_party(http, limiter, session, &headers, &party_player.current_party_id).await?;
        let snap = build_party_snapshot(http, limiter, cache, session, &headers, raw_party).await;
        return Ok(LiveState::InParty(snap));
    }

    // 4. Idle
    cache.clear_live_match_guns(&session.puuid);
    Ok(LiveState::Idle {
        message: "Valorant PC client is not in an active party or match".to_string(),
        recommended_poll_secs: 10,
    })
}
