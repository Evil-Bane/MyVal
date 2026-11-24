use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::auth::{
    build_riot_headers, build_riot_login_url, complete_auth_from_redirect_url,
    refresh_session_from_cookie,
};
use crate::cache::{CachedLiveMatchGuns, LocalCache};
use crate::career::{
    fetch_career_snapshot, fetch_match_detail, fetch_match_detail_for_player, fetch_player_profile,
    lookup_player_profile_by_riot_id,
};
use crate::error::MyValError;
use crate::glz::{self, RawPartyResp, RawPreGameMatchResp};
use crate::live::{
    build_party_snapshot, build_pregame_snapshot, extract_live_gun_map, extract_live_spray_list,
    poll_live_state, resolve_coregame_loadout_for_player, validate_party_change_queue,
    validate_party_disable_invite, validate_party_generate_invite, validate_party_join_queue,
    validate_party_leave_queue, validate_party_set_accessibility, validate_party_set_ready,
    validate_pregame_agent_action, validate_pregame_action_delay, validate_pregame_quit,
};
use crate::loadout::{
    apply_identity_privacy_to_raw_loadout, apply_player_card_to_raw_loadout,
    apply_player_title_to_raw_loadout, apply_skin_to_raw_loadout, apply_spray_to_raw_loadout,
    compute_collection_summary_from_levels, enrich_loadout, fetch_collection_summary,
    fetch_or_build_offline_loadout, fetch_or_build_player_loadout, fetch_owned_agent_uuids,
    randomize_raw_loadout, sync_or_persist_loadout, RawPlayerLoadout,
};
use crate::models::assets::{Agent, AssetSummary, MapInfo, PlayerCard, PlayerTitle, Skin, Spray, Weapon};
use crate::models::auth::RiotSession;
use crate::models::career::{
    CareerSnapshot, KnownPlayerSummary, MatchDetailSnapshot, PlayerProfileSnapshot,
};
use crate::models::live::{LiveState, PartySnapshot, PreGameSnapshot};
use crate::models::loadout::{CollectionSummary, LoadoutPreset, PlayerLoadoutSnapshot};
use crate::models::store::StorefrontSnapshot;
use crate::pd::{enrich_storefront, fetch_storefront_raw, fetch_wallet};
use crate::rate_limit::RateLimiter;
use crate::valorant_api::sync_all_assets;

pub struct MyValEngineCore {
    http: reqwest::Client,
    pub cache: LocalCache,
    limiter: RateLimiter,
    session: RwLock<Option<RiotSession>>,
    active_raw_loadout: RwLock<Option<RawPlayerLoadout>>,
    last_party_snapshot: RwLock<Option<PartySnapshot>>,
    last_pregame_snapshot: RwLock<Option<PreGameSnapshot>>,
    pregame_first_seen: RwLock<Option<(String, std::time::Instant)>>,
}

impl MyValEngineCore {
    pub fn open(db_path: &str) -> Result<Self, MyValError> {
        let http = reqwest::Client::builder()
            .user_agent("MyVal/1.0.0")
            .timeout(std::time::Duration::from_secs(20))
            .build()?;
        let cache = LocalCache::open(db_path)?;
        Ok(Self {
            http,
            cache,
            limiter: RateLimiter::new(3),
            session: RwLock::new(None),
            active_raw_loadout: RwLock::new(None),
            last_party_snapshot: RwLock::new(None),
            last_pregame_snapshot: RwLock::new(None),
            pregame_first_seen: RwLock::new(None),
        })
    }

    async fn record_pregame_seen(&self, match_id: &str) {
        let clean = match_id.trim();
        if clean.is_empty() {
            return;
        }
        let mut guard = self.pregame_first_seen.write().await;
        let same_match = guard
            .as_ref()
            .map(|(id, _)| id.eq_ignore_ascii_case(clean))
            .unwrap_or(false);
        if !same_match {
            *guard = Some((clean.to_string(), std::time::Instant::now()));
        }
    }

    async fn clear_pregame_seen(&self) {
        *self.pregame_first_seen.write().await = None;
    }

    async fn pregame_elapsed_secs(&self, match_id: &str) -> u64 {
        let clean = match_id.trim();
        let guard = self.pregame_first_seen.read().await;
        match guard.as_ref() {
            Some((id, seen_at)) if !clean.is_empty() && id.eq_ignore_ascii_case(clean) => seen_at.elapsed().as_secs(),
            _ => 0,
        }
    }

    pub fn get_riot_login_url(&self) -> String {
        build_riot_login_url()
    }

    async fn set_active_session(&self, s: RiotSession) {
        let switched_user = self
            .session
            .read()
            .await
            .as_ref()
            .map(|prev| !prev.puuid.eq_ignore_ascii_case(&s.puuid))
            .unwrap_or(true);
        if switched_user {
            *self.active_raw_loadout.write().await = None;
            *self.last_party_snapshot.write().await = None;
            *self.last_pregame_snapshot.write().await = None;
            self.clear_pregame_seen().await;
        }
        *self.session.write().await = Some(s);
    }

    pub async fn complete_auth_from_redirect_url(
        &self,
        redirect_url: &str,
    ) -> Result<RiotSession, MyValError> {
        let s = complete_auth_from_redirect_url(&self.http, redirect_url).await?;
        self.set_active_session(s.clone()).await;
        Ok(s)
    }

    pub async fn refresh_session_from_cookie(
        &self,
        cookie_header: &str,
    ) -> Result<RiotSession, MyValError> {
        let s = refresh_session_from_cookie(&self.http, cookie_header).await?;
        self.set_active_session(s.clone()).await;
        Ok(s)
    }

    pub async fn restore_session_json(&self, session_json: &str) -> Result<RiotSession, MyValError> {
        let s: RiotSession = serde_json::from_str(session_json)?;
        self.set_active_session(s.clone()).await;
        Ok(s)
    }

    pub async fn export_session_json(&self) -> Result<Option<String>, MyValError> {
        let guard = self.session.read().await;
        match &*guard {
            Some(s) => Ok(Some(serde_json::to_string(s)?)),
            None => Ok(None),
        }
    }

    pub async fn current_session(&self) -> Option<RiotSession> {
        self.session.read().await.clone()
    }

    pub async fn clear_session(&self) {
        *self.session.write().await = None;
        *self.active_raw_loadout.write().await = None;
        *self.last_party_snapshot.write().await = None;
        *self.last_pregame_snapshot.write().await = None;
        self.clear_pregame_seen().await;
    }

    async fn require_session(&self) -> Result<RiotSession, MyValError> {
        let s = self
            .session
            .read()
            .await
            .clone()
            .ok_or_else(|| MyValError::Unauthorized("Not signed in".to_string()))?;
        if s.is_expired(LocalCache::now_secs()) {
            return Err(MyValError::Unauthorized(
                "Riot session expired — please sign in again".to_string(),
            ));
        }
        Ok(s)
    }

    async fn require_session_allow_expired(&self) -> Result<(RiotSession, bool), MyValError> {
        let s = self
            .session
            .read()
            .await
            .clone()
            .ok_or_else(|| MyValError::Unauthorized("Not signed in".to_string()))?;
        let expired = s.is_expired(LocalCache::now_secs());
        Ok((s, expired))
    }

    pub fn needs_asset_resync(&self) -> bool {
        self.cache.needs_asset_resync()
    }

    pub async fn sync_assets(&self) -> Result<AssetSummary, MyValError> {
        sync_all_assets(&self.http, &self.cache).await
    }

    pub fn get_asset_summary(&self) -> Result<AssetSummary, MyValError> {
        self.cache.get_asset_summary()
    }

    pub fn list_agents(&self) -> Result<Vec<Agent>, MyValError> {
        self.cache.list_category("agent")
    }

    pub fn list_weapons(&self) -> Result<Vec<Weapon>, MyValError> {
        self.cache.list_category("weapon")
    }

    pub fn list_maps(&self) -> Result<Vec<MapInfo>, MyValError> {
        self.cache.list_category("map")
    }

    pub fn list_all_weapon_skins(&self) -> Result<Vec<Skin>, MyValError> {
        self.cache.list_all_skins()
    }

    pub fn list_player_cards(&self) -> Result<Vec<PlayerCard>, MyValError> {
        self.cache.list_category("playercard")
    }

    pub fn list_player_titles(&self) -> Result<Vec<PlayerTitle>, MyValError> {
        self.cache.list_category("title")
    }

    pub fn list_sprays(&self) -> Result<Vec<Spray>, MyValError> {
        self.cache.list_category("spray")
    }

    pub fn search_skins(
        &self,
        query: &str,
        wishlist_only: bool,
        limit: u32,
    ) -> Result<Vec<Skin>, MyValError> {
        self.cache.search_skins(query, wishlist_only, limit)
    }

    pub fn toggle_wishlist(&self, skin_uuid: &str) -> Result<bool, MyValError> {
        self.cache.toggle_wishlist(skin_uuid)
    }

    pub fn get_wishlist_uuids(&self) -> Result<Vec<String>, MyValError> {
        self.cache.get_wishlist_uuids()
    }

    pub async fn get_owned_skin_uuids(&self) -> Vec<String> {
        let puuid = self
            .session
            .read()
            .await
            .as_ref()
            .map(|s| s.puuid.clone())
            .unwrap_or_default();
        if puuid.is_empty() {
            return Vec::new();
        }
        self.cache
            .get_owned_skin_uuids(&puuid)
            .into_iter()
            .collect()
    }

    pub async fn get_owned_agent_uuids(&self) -> Vec<String> {
        let Ok((session, expired)) = self.require_session_allow_expired().await else {
            return Vec::new();
        };
        if !expired {
            if let Ok(headers) = build_riot_headers(&session, &self.cache.riot_client_version()) {
                if let Ok(live) = fetch_owned_agent_uuids(
                    &self.http,
                    &self.limiter,
                    &self.cache,
                    &session,
                    &headers,
                )
                .await
                {
                    return live;
                }
            }
        }
        self.cache
            .get_owned_agent_uuids(&session.puuid)
            .into_iter()
            .collect()
    }

    pub async fn fetch_storefront(&self) -> Result<StorefrontSnapshot, MyValError> {
        let (session, expired) = self.require_session_allow_expired().await?;
        if expired {
            return Err(MyValError::Unauthorized(
                "Riot session expired — please sign in again".to_string(),
            ));
        }
        let headers = build_riot_headers(&session, &self.cache.riot_client_version())?;
        let (raw_res, wallet_res) = tokio::join!(
            fetch_storefront_raw(&self.http, &self.limiter, &session, &headers),
            fetch_wallet(&self.http, &self.limiter, &session, &headers),
        );
        match raw_res {
            Ok(raw) => {
                let wallet = wallet_res.unwrap_or_default();
                let snap = enrich_storefront(&self.cache, &raw, wallet);
                self.cache.put_cached_storefront(&session.puuid, &snap)?;
                Ok(snap)
            }
            Err(err @ MyValError::Unauthorized(_)) => Err(err),
            Err(live_err) => {
                if let Some(cached) = self.cache.get_cached_storefront(&session.puuid)? {
                    return Ok(cached);
                }
                Err(live_err)
            }
        }
    }

    pub async fn get_cached_storefront(&self) -> Result<Option<StorefrontSnapshot>, MyValError> {
        let puuid = self
            .session
            .read()
            .await
            .as_ref()
            .map(|s| s.puuid.clone())
            .unwrap_or_default();
        if puuid.is_empty() {
            return Ok(None);
        }
        self.cache.get_cached_storefront(&puuid)
    }

    pub async fn fetch_collection_summary(&self) -> Result<CollectionSummary, MyValError> {
        let (session, expired) = self.require_session_allow_expired().await?;
        if expired {
            return Err(MyValError::Unauthorized(
                "Riot session expired — please sign in again".to_string(),
            ));
        }
        let store_opt = self.cache.get_cached_storefront(&session.puuid).ok().flatten();
        let headers = build_riot_headers(&session, &self.cache.riot_client_version())?;
        match fetch_collection_summary(
            &self.http,
            &self.limiter,
            &self.cache,
            &session,
            &headers,
            store_opt.as_ref(),
        )
        .await
        {
            Ok(summary) => return Ok(summary),
            Err(err @ MyValError::Unauthorized(_)) => return Err(err),
            Err(_) => {}
        }
        let owned: Vec<String> = self
            .cache
            .get_owned_skin_uuids(&session.puuid)
            .into_iter()
            .collect();
        let (summary, _) = compute_collection_summary_from_levels(
            &self.cache,
            &owned,
            &HashMap::new(),
            store_opt.as_ref(),
        )?;
        Ok(summary)
    }

    // ---- Active Loadout, Remote Skin Changer, Presets & Randomizer ----

    async fn resolve_current_raw_loadout(&self, force_fresh: bool) -> Result<(RiotSession, bool, RawPlayerLoadout), MyValError> {
        let (session, expired) = self.require_session_allow_expired().await?;
        if !force_fresh {
            if let Some(existing) = self.active_raw_loadout.read().await.clone() {
                if !existing.guns.is_empty() && existing.subject.eq_ignore_ascii_case(&session.puuid) {
                    return Ok((session, expired, existing));
                }
            }
        }
        if expired {
            return Err(MyValError::Unauthorized(
                "Riot session expired — refresh your session to sync loadout".to_string(),
            ));
        }
        let headers = build_riot_headers(&session, &self.cache.riot_client_version())?;
        let raw = match fetch_or_build_player_loadout(
            &self.http,
            &self.limiter,
            &self.cache,
            &session,
            &headers,
        )
        .await
        {
            Ok(r) => r,
            Err(e @ MyValError::Unauthorized(_)) => return Err(e),
            Err(_) => fetch_or_build_offline_loadout(&self.cache, &session.puuid)?,
        };
        Ok((session, expired, raw))
    }

    async fn persist_updated_raw_loadout(
        &self,
        session: &RiotSession,
        expired: bool,
        raw: &RawPlayerLoadout,
    ) -> Result<RawPlayerLoadout, MyValError> {
        if expired {
            return Err(MyValError::Unauthorized(
                "Riot session expired — refresh your session before editing loadout".to_string(),
            ));
        }
        let headers = build_riot_headers(session, &self.cache.riot_client_version())?;
        sync_or_persist_loadout(
            &self.http,
            &self.limiter,
            &self.cache,
            session,
            &headers,
            raw,
        )
        .await
    }

    async fn sync_live_match_guns_if_in_coregame(
        &self,
        session: &RiotSession,
        headers: &reqwest::header::HeaderMap,
    ) {
        let Ok(cg_player_opt) =
            glz::get_coregame_player(&self.http, &self.limiter, session, headers).await
        else {
            return;
        };
        let Some(cg_player) = cg_player_opt else {
            self.cache.clear_live_match_guns(&session.puuid);
            return;
        };
        let match_id = cg_player.match_id;
        let (cg_match_res, cg_loadouts_res) = tokio::join!(
            glz::get_coregame_match(&self.http, &self.limiter, session, headers, &match_id),
            glz::get_coregame_loadouts(&self.http, &self.limiter, session, headers, &match_id),
        );
        let Ok(cg_match) = cg_match_res else {
            return;
        };
        let raw_loadouts = cg_loadouts_res
            .ok()
            .flatten()
            .map(|r| r.loadouts)
            .unwrap_or_default();
        let total = cg_match.players.len();
        if let Some((idx, me)) = cg_match
            .players
            .iter()
            .enumerate()
            .find(|(_, p)| p.subject.eq_ignore_ascii_case(&session.puuid))
        {
            let entry = resolve_coregame_loadout_for_player(
                &raw_loadouts,
                &me.subject,
                &me.character_id,
                idx,
                total,
            );
            let live_guns = extract_live_gun_map(entry);
            let live_sprays = extract_live_spray_list(entry);
            if !live_guns.is_empty() || !live_sprays.is_empty() {
                let _ = self.cache.put_live_match_guns(
                    &session.puuid,
                    &CachedLiveMatchGuns {
                        match_id,
                        guns: live_guns,
                        sprays: live_sprays,
                    },
                );
            }
        }
    }

    pub async fn fetch_player_loadout(&self) -> Result<PlayerLoadoutSnapshot, MyValError> {
        let (session, expired, raw) = self.resolve_current_raw_loadout(true).await?;
        if !expired {
            if let Ok(headers) = build_riot_headers(&session, &self.cache.riot_client_version()) {
                self.sync_live_match_guns_if_in_coregame(&session, &headers).await;
            }
        }
        let snap = enrich_loadout(&self.cache, &raw);
        *self.active_raw_loadout.write().await = Some(raw);
        Ok(snap)
    }

    async fn ensure_collection_entitlements_cached(&self, puuid: &str) {
        if self.cache.get_owned_skin_uuids(puuid).is_empty()
            || self.cache.get_owned_skin_level_uuids(puuid).is_empty()
            || self.cache.get_owned_chroma_uuids(puuid).is_empty()
            || self.cache.get_owned_spray_uuids(puuid).is_empty()
            || self.cache.get_owned_card_uuids(puuid).is_empty()
            || self.cache.get_owned_title_uuids(puuid).is_empty()
        {
            let _ = self.fetch_collection_summary().await;
        }
    }

    pub async fn equip_weapon_skin_full(
        &self,
        weapon_uuid: &str,
        skin_uuid: &str,
        chroma_uuid: Option<&str>,
        level_uuid: Option<&str>,
    ) -> Result<PlayerLoadoutSnapshot, MyValError> {
        let (session, expired, mut raw) = self.resolve_current_raw_loadout(true).await?;
        if expired {
            return Err(MyValError::Unauthorized(
                "Riot session expired — refresh your session to equip skins".to_string(),
            ));
        }
        self.ensure_collection_entitlements_cached(&session.puuid).await;
        let owned_skins = self.cache.get_owned_skin_uuids(&session.puuid);
        let owned_levels = self.cache.get_owned_skin_level_uuids(&session.puuid);
        let owned_chromas = self.cache.get_owned_chroma_uuids(&session.puuid);
        apply_skin_to_raw_loadout(
            &self.cache,
            &mut raw,
            weapon_uuid,
            skin_uuid,
            &owned_skins,
            &owned_levels,
            chroma_uuid,
            &owned_chromas,
            level_uuid,
        )?;
        let updated = self.persist_updated_raw_loadout(&session, expired, &raw).await?;
        let snap = enrich_loadout(&self.cache, &updated);
        *self.active_raw_loadout.write().await = Some(updated);
        Ok(snap)
    }

    pub async fn equip_weapon_skin_chroma(
        &self,
        weapon_uuid: &str,
        skin_uuid: &str,
        chroma_uuid: Option<&str>,
    ) -> Result<PlayerLoadoutSnapshot, MyValError> {
        self.equip_weapon_skin_full(weapon_uuid, skin_uuid, chroma_uuid, None)
            .await
    }

    pub async fn equip_weapon_skin(
        &self,
        weapon_uuid: &str,
        skin_uuid: &str,
    ) -> Result<PlayerLoadoutSnapshot, MyValError> {
        self.equip_weapon_skin_full(weapon_uuid, skin_uuid, None, None)
            .await
    }

    pub async fn equip_spray(
        &self,
        slot_uuid: &str,
        spray_uuid: &str,
    ) -> Result<PlayerLoadoutSnapshot, MyValError> {
        let (session, expired, mut raw) = self.resolve_current_raw_loadout(true).await?;
        if expired {
            return Err(MyValError::Unauthorized(
                "Riot session expired — refresh your session to equip sprays".to_string(),
            ));
        }
        self.ensure_collection_entitlements_cached(&session.puuid).await;
        let owned_sprays = self.cache.get_owned_spray_uuids(&session.puuid);
        apply_spray_to_raw_loadout(&mut raw, slot_uuid, spray_uuid, &owned_sprays)?;
        let updated = self.persist_updated_raw_loadout(&session, expired, &raw).await?;
        let snap = enrich_loadout(&self.cache, &updated);
        *self.active_raw_loadout.write().await = Some(updated);
        Ok(snap)
    }

    async fn refresh_party_identity_best_effort(
        &self,
        session: &RiotSession,
        headers: &reqwest::header::HeaderMap,
    ) {
        if let Ok(Some(party_player)) =
            glz::get_party_player(&self.http, &self.limiter, session, headers).await
        {
            if !party_player.current_party_id.is_empty() {
                let _ = glz::party_refresh_player_identity(
                    &self.http,
                    &self.limiter,
                    session,
                    headers,
                    &party_player.current_party_id,
                )
                .await;
            }
        }
    }

    pub async fn test_loadout_v3_noop_put(&self) -> Result<(u8, i32, usize, usize), MyValError> {
        let session = self.require_session().await?;
        let headers = build_riot_headers(&session, &self.cache.riot_client_version())?;
        let raw = crate::loadout::fetch_raw_loadout(&self.http, &self.limiter, &session, &headers).await?;
        let updated = crate::loadout::put_raw_loadout(&self.http, &self.limiter, &session, &headers, &raw).await?;
        crate::loadout::save_cached_loadout(&self.cache, &session.puuid, &updated);
        let api_version = updated.api_version;
        let version = updated.version;
        let guns_len = updated.guns.len();
        let expr_len = updated.active_expressions.len();
        *self.active_raw_loadout.write().await = Some(updated);
        Ok((api_version, version, guns_len, expr_len))
    }

    pub async fn equip_player_card(
        &self,
        card_uuid: &str,
    ) -> Result<PlayerLoadoutSnapshot, MyValError> {
        let (session, expired, mut raw) = self.resolve_current_raw_loadout(true).await?;
        if expired {
            return Err(MyValError::Unauthorized(
                "Riot session expired — refresh your session to update player card".to_string(),
            ));
        }
        self.ensure_collection_entitlements_cached(&session.puuid).await;
        let owned_cards = self.cache.get_owned_card_uuids(&session.puuid);
        apply_player_card_to_raw_loadout(&mut raw, card_uuid, &owned_cards)?;
        let updated = self.persist_updated_raw_loadout(&session, expired, &raw).await?;
        if let Ok(headers) = build_riot_headers(&session, &self.cache.riot_client_version()) {
            self.refresh_party_identity_best_effort(&session, &headers).await;
        }
        let snap = enrich_loadout(&self.cache, &updated);
        *self.active_raw_loadout.write().await = Some(updated);
        Ok(snap)
    }

    pub async fn equip_player_title(
        &self,
        title_uuid: &str,
    ) -> Result<PlayerLoadoutSnapshot, MyValError> {
        let (session, expired, mut raw) = self.resolve_current_raw_loadout(true).await?;
        if expired {
            return Err(MyValError::Unauthorized(
                "Riot session expired — refresh your session to update player title".to_string(),
            ));
        }
        self.ensure_collection_entitlements_cached(&session.puuid).await;
        let owned_titles = self.cache.get_owned_title_uuids(&session.puuid);
        apply_player_title_to_raw_loadout(&mut raw, title_uuid, &owned_titles)?;
        let updated = self.persist_updated_raw_loadout(&session, expired, &raw).await?;
        if let Ok(headers) = build_riot_headers(&session, &self.cache.riot_client_version()) {
            self.refresh_party_identity_best_effort(&session, &headers).await;
        }
        let snap = enrich_loadout(&self.cache, &updated);
        *self.active_raw_loadout.write().await = Some(updated);
        Ok(snap)
    }

    pub async fn set_identity_privacy(
        &self,
        incognito: bool,
        hide_account_level: bool,
    ) -> Result<PlayerLoadoutSnapshot, MyValError> {
        let (session, expired, mut raw) = self.resolve_current_raw_loadout(true).await?;
        if expired {
            return Err(MyValError::Unauthorized(
                "Riot session expired — refresh your session to update privacy".to_string(),
            ));
        }
        apply_identity_privacy_to_raw_loadout(&mut raw, incognito, hide_account_level);
        let updated = self.persist_updated_raw_loadout(&session, expired, &raw).await?;
        if let Ok(headers) = build_riot_headers(&session, &self.cache.riot_client_version()) {
            self.refresh_party_identity_best_effort(&session, &headers).await;
        }
        let snap = enrich_loadout(&self.cache, &updated);
        *self.active_raw_loadout.write().await = Some(updated);
        Ok(snap)
    }

    pub async fn get_owned_chroma_uuids(&self) -> Vec<String> {
        let session = self.session.read().await;
        let Some(s) = session.as_ref() else {
            return Vec::new();
        };
        self.cache.get_owned_chroma_uuids(&s.puuid).into_iter().collect()
    }

    pub async fn get_owned_level_uuids(&self) -> Vec<String> {
        let session = self.session.read().await;
        let Some(s) = session.as_ref() else {
            return Vec::new();
        };
        self.cache.get_owned_skin_level_uuids(&s.puuid).into_iter().collect()
    }

    pub async fn get_owned_spray_uuids(&self) -> Vec<String> {
        let session = self.session.read().await;
        let Some(s) = session.as_ref() else {
            return Vec::new();
        };
        self.cache.get_owned_spray_uuids(&s.puuid).into_iter().collect()
    }

    pub async fn get_owned_card_uuids(&self) -> Vec<String> {
        let session = self.session.read().await;
        let Some(s) = session.as_ref() else {
            return Vec::new();
        };
        self.cache.get_owned_card_uuids(&s.puuid).into_iter().collect()
    }

    pub async fn get_owned_title_uuids(&self) -> Vec<String> {
        let session = self.session.read().await;
        let Some(s) = session.as_ref() else {
            return Vec::new();
        };
        self.cache.get_owned_title_uuids(&s.puuid).into_iter().collect()
    }

    pub async fn randomize_loadout(&self, seed: u64) -> Result<PlayerLoadoutSnapshot, MyValError> {
        let (session, expired, mut raw) = self.resolve_current_raw_loadout(true).await?;
        if expired {
            return Err(MyValError::Unauthorized(
                "Riot session expired — refresh your session to randomize loadout".to_string(),
            ));
        }
        let mut owned_skins = self.cache.get_owned_skin_uuids(&session.puuid);
        let mut owned_levels = self.cache.get_owned_skin_level_uuids(&session.puuid);
        if owned_skins.is_empty() || owned_levels.is_empty() {
            let _ = self.fetch_collection_summary().await;
            owned_skins = self.cache.get_owned_skin_uuids(&session.puuid);
            owned_levels = self.cache.get_owned_skin_level_uuids(&session.puuid);
        }
        randomize_raw_loadout(&self.cache, &mut raw, &owned_skins, &owned_levels, seed)?;
        let updated = self.persist_updated_raw_loadout(&session, expired, &raw).await?;
        let snap = enrich_loadout(&self.cache, &updated);
        *self.active_raw_loadout.write().await = Some(updated);
        Ok(snap)
    }

    pub async fn save_current_loadout_preset(
        &self,
        name: &str,
        accent_color_hex: &str,
    ) -> Result<LoadoutPreset, MyValError> {
        let (session, _expired, raw) = self.resolve_current_raw_loadout(false).await?;
        let enriched = enrich_loadout(&self.cache, &raw);
        let preview_skins: Vec<String> = enriched
            .guns
            .iter()
            .take(4)
            .map(|g| g.skin_name.clone())
            .collect();
        let raw_json = serde_json::to_string(&raw)?;
        self.cache.save_loadout_preset(
            &session.puuid,
            name,
            accent_color_hex,
            enriched.guns.len() as u32,
            &preview_skins,
            &raw_json,
        )
    }

    pub async fn list_loadout_presets(&self) -> Result<Vec<LoadoutPreset>, MyValError> {
        let puuid = self
            .session
            .read()
            .await
            .as_ref()
            .map(|s| s.puuid.clone());
        self.cache.list_loadout_presets(puuid.as_deref())
    }

    pub async fn apply_loadout_preset(
        &self,
        preset_id: &str,
    ) -> Result<PlayerLoadoutSnapshot, MyValError> {
        let (session, expired) = self.require_session_allow_expired().await?;
        let (preset_puuid, raw_json) = self
            .cache
            .get_loadout_preset_raw(preset_id)?
            .ok_or_else(|| MyValError::InvalidInput("Preset not found".to_string()))?;
        if !preset_puuid.is_empty() && !preset_puuid.eq_ignore_ascii_case(&session.puuid) {
            return Err(MyValError::InvalidInput(
                "This loadout preset belongs to a different Riot account".to_string(),
            ));
        }
        let raw: RawPlayerLoadout = serde_json::from_str(&raw_json)?;
        if !raw.subject.is_empty() && !raw.subject.eq_ignore_ascii_case(&session.puuid) {
            return Err(MyValError::InvalidInput(
                "This loadout preset belongs to a different Riot account".to_string(),
            ));
        }
        let updated = self.persist_updated_raw_loadout(&session, expired, &raw).await?;
        let snap = enrich_loadout(&self.cache, &updated);
        *self.active_raw_loadout.write().await = Some(updated);
        Ok(snap)
    }

    pub fn delete_loadout_preset(&self, preset_id: &str) -> Result<(), MyValError> {
        self.cache.delete_loadout_preset(preset_id)
    }

    // ---- Career, Match History & Interactive 2D Round Replay ----

    pub async fn get_cached_career(&self) -> Option<CareerSnapshot> {
        let puuid = self
            .session
            .read()
            .await
            .as_ref()
            .map(|s| s.puuid.clone())
            .unwrap_or_default();
        if puuid.is_empty() {
            return None;
        }
        self.cache.get_cached_career(&puuid, 86_400 * 30)
    }

    pub async fn fetch_career(&self) -> Result<CareerSnapshot, MyValError> {
        let (session, expired) = self.require_session_allow_expired().await?;
        if expired {
            return Err(MyValError::Unauthorized(
                "Riot session expired — please sign in again".to_string(),
            ));
        }
        let headers = build_riot_headers(&session, &self.cache.riot_client_version())?;
        match fetch_career_snapshot(&self.http, &self.limiter, &self.cache, &session, &headers).await {
            Ok(live) => {
                let _ = self.cache.put_cached_career(&session.puuid, &live);
                Ok(live)
            }
            Err(err @ MyValError::Unauthorized(_)) => Err(err),
            Err(live_err) => {
                if let Some(cached) = self.cache.get_cached_career(&session.puuid, 86_400 * 30) {
                    return Ok(cached);
                }
                Err(live_err)
            }
        }
    }

    pub async fn fetch_match_detail(&self, match_id: &str) -> Result<MatchDetailSnapshot, MyValError> {
        let (session, expired) = self.require_session_allow_expired().await?;
        if let Some(cached) = self.cache.get_cached_match_detail(match_id, &session.puuid) {
            return Ok(cached);
        }
        if expired {
            return Err(MyValError::Unauthorized(
                "Riot session expired — please sign in again".to_string(),
            ));
        }
        let headers = build_riot_headers(&session, &self.cache.riot_client_version())?;
        fetch_match_detail(
            &self.http,
            &self.limiter,
            &self.cache,
            &session,
            &headers,
            match_id,
        )
        .await
    }

    pub async fn fetch_match_detail_for_player(
        &self,
        match_id: &str,
        perspective_puuid: &str,
    ) -> Result<MatchDetailSnapshot, MyValError> {
        let (session, expired) = self.require_session_allow_expired().await?;
        let target = if perspective_puuid.trim().is_empty() {
            session.puuid.as_str()
        } else {
            perspective_puuid.trim()
        };
        if let Some(cached) = self.cache.get_cached_match_detail(match_id, target) {
            return Ok(cached);
        }
        if expired {
            return Err(MyValError::Unauthorized(
                "Riot session expired — please sign in again".to_string(),
            ));
        }
        let headers = build_riot_headers(&session, &self.cache.riot_client_version())?;
        fetch_match_detail_for_player(
            &self.http,
            &self.limiter,
            &self.cache,
            &session,
            &headers,
            target,
            match_id,
        )
        .await
    }

    pub async fn fetch_player_profile(
        &self,
        target_puuid: &str,
        force_refresh: bool,
    ) -> Result<PlayerProfileSnapshot, MyValError> {
        let (session, expired) = self.require_session_allow_expired().await?;
        if !force_refresh {
            if let Some(cached) = self
                .cache
                .get_cached_player_profile(target_puuid, crate::career::PLAYER_PROFILE_CACHE_TTL_SECS)
            {
                return Ok(cached);
            }
        }
        if expired {
            return Err(MyValError::Unauthorized(
                "Riot session expired — please sign in again".to_string(),
            ));
        }
        let headers = build_riot_headers(&session, &self.cache.riot_client_version())?;
        fetch_player_profile(
            &self.http,
            &self.limiter,
            &self.cache,
            &session,
            &headers,
            target_puuid,
            force_refresh,
        )
        .await
    }

    pub fn search_known_players(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<Vec<KnownPlayerSummary>, MyValError> {
        self.cache.search_known_players(query, limit)
    }

    pub async fn lookup_player_profile_by_riot_id(
        &self,
        riot_id: &str,
    ) -> Result<PlayerProfileSnapshot, MyValError> {
        let session = self.require_session().await?;
        let headers = build_riot_headers(&session, &self.cache.riot_client_version())?;
        lookup_player_profile_by_riot_id(
            &self.http,
            &self.limiter,
            &self.cache,
            &session,
            &headers,
            riot_id,
        )
        .await
    }

    pub fn set_lookup_api_key(&self, key: &str) -> Result<(), MyValError> {
        self.cache.set_lookup_api_key(key)
    }

    pub fn has_lookup_api_key(&self) -> bool {
        self.cache.get_lookup_api_key().is_some()
    }

    // ---- Live Hub ----

    pub async fn poll_live_state(&self) -> Result<LiveState, MyValError> {
        let session = self.require_session().await?;
        let state = poll_live_state(&self.http, &self.limiter, &self.cache, &session).await?;
        match &state {
            LiveState::InParty(snap) => {
                *self.last_party_snapshot.write().await = Some(snap.clone());
                *self.last_pregame_snapshot.write().await = None;
            }
            LiveState::PreGame(snap) => {
                *self.last_party_snapshot.write().await = None;
                *self.last_pregame_snapshot.write().await = Some(snap.clone());
                self.record_pregame_seen(&snap.match_id).await;
            }
            LiveState::CoreGame(_) | LiveState::Idle { .. } => {
                *self.last_party_snapshot.write().await = None;
                *self.last_pregame_snapshot.write().await = None;
            }
        }
        Ok(state)
    }

    async fn finish_party_action(
        &self,
        session: &RiotSession,
        raw: RawPartyResp,
    ) -> Result<PartySnapshot, MyValError> {
        let headers = build_riot_headers(session, &self.cache.riot_client_version())?;
        let snap = build_party_snapshot(
            &self.http,
            &self.limiter,
            &self.cache,
            session,
            &headers,
            raw,
        )
        .await;
        *self.last_party_snapshot.write().await = Some(snap.clone());
        Ok(snap)
    }

    async fn finish_pregame_action(
        &self,
        session: &RiotSession,
        match_id: String,
        raw: RawPreGameMatchResp,
    ) -> Result<PreGameSnapshot, MyValError> {
        let headers = build_riot_headers(session, &self.cache.riot_client_version())?;
        let snap = build_pregame_snapshot(
            &self.http,
            &self.limiter,
            &self.cache,
            session,
            &headers,
            match_id,
            raw,
        )
        .await;
        *self.last_pregame_snapshot.write().await = Some(snap.clone());
        self.record_pregame_seen(&snap.match_id).await;
        Ok(snap)
    }

    pub async fn party_set_ready(&self, party_id: &str, ready: bool) -> Result<PartySnapshot, MyValError> {
        let session = self.require_session().await?;
        if let Some(ref snap) = *self.last_party_snapshot.read().await {
            if snap.party_id.eq_ignore_ascii_case(party_id) {
                validate_party_set_ready(snap)?;
            }
        }
        let headers = build_riot_headers(&session, &self.cache.riot_client_version())?;
        let raw = glz::party_set_ready(&self.http, &self.limiter, &session, &headers, party_id, ready).await?;
        self.finish_party_action(&session, raw).await
    }

    pub async fn party_join_queue(&self, party_id: &str) -> Result<PartySnapshot, MyValError> {
        let session = self.require_session().await?;
        if let Some(ref snap) = *self.last_party_snapshot.read().await {
            if snap.party_id.eq_ignore_ascii_case(party_id) {
                validate_party_join_queue(snap)?;
            }
        }
        let headers = build_riot_headers(&session, &self.cache.riot_client_version())?;
        let raw = glz::party_join_queue(&self.http, &self.limiter, &session, &headers, party_id).await?;
        self.finish_party_action(&session, raw).await
    }

    pub async fn party_leave_queue(&self, party_id: &str) -> Result<PartySnapshot, MyValError> {
        let session = self.require_session().await?;
        if let Some(ref snap) = *self.last_party_snapshot.read().await {
            if snap.party_id.eq_ignore_ascii_case(party_id) {
                validate_party_leave_queue(snap)?;
            }
        }
        let headers = build_riot_headers(&session, &self.cache.riot_client_version())?;
        let raw = glz::party_leave_queue(&self.http, &self.limiter, &session, &headers, party_id).await?;
        self.finish_party_action(&session, raw).await
    }

    pub async fn party_change_queue(&self, party_id: &str, queue_id: &str) -> Result<PartySnapshot, MyValError> {
        let session = self.require_session().await?;
        if let Some(ref snap) = *self.last_party_snapshot.read().await {
            if snap.party_id.eq_ignore_ascii_case(party_id) {
                validate_party_change_queue(snap, queue_id)?;
            }
        }
        let headers = build_riot_headers(&session, &self.cache.riot_client_version())?;
        let raw = glz::party_change_queue(&self.http, &self.limiter, &session, &headers, party_id, queue_id).await?;
        self.finish_party_action(&session, raw).await
    }

    pub async fn party_set_accessibility(&self, party_id: &str, open: bool) -> Result<PartySnapshot, MyValError> {
        let session = self.require_session().await?;
        if let Some(ref snap) = *self.last_party_snapshot.read().await {
            if snap.party_id.eq_ignore_ascii_case(party_id) {
                validate_party_set_accessibility(snap)?;
            }
        }
        let headers = build_riot_headers(&session, &self.cache.riot_client_version())?;
        let raw = glz::party_set_accessibility(&self.http, &self.limiter, &session, &headers, party_id, open).await?;
        self.finish_party_action(&session, raw).await
    }

    pub async fn party_generate_invite_code(&self, party_id: &str) -> Result<PartySnapshot, MyValError> {
        let session = self.require_session().await?;
        if let Some(ref snap) = *self.last_party_snapshot.read().await {
            if snap.party_id.eq_ignore_ascii_case(party_id) {
                validate_party_generate_invite(snap)?;
            }
        }
        let headers = build_riot_headers(&session, &self.cache.riot_client_version())?;
        let raw = glz::party_generate_invite_code(&self.http, &self.limiter, &session, &headers, party_id).await?;
        self.finish_party_action(&session, raw).await
    }

    pub async fn party_disable_invite_code(&self, party_id: &str) -> Result<PartySnapshot, MyValError> {
        let session = self.require_session().await?;
        if let Some(ref snap) = *self.last_party_snapshot.read().await {
            if snap.party_id.eq_ignore_ascii_case(party_id) {
                validate_party_disable_invite(snap)?;
            }
        }
        let headers = build_riot_headers(&session, &self.cache.riot_client_version())?;
        let raw = glz::party_disable_invite_code(&self.http, &self.limiter, &session, &headers, party_id).await?;
        self.finish_party_action(&session, raw).await
    }

    pub async fn party_join_by_code(&self, invite_code: &str) -> Result<PartySnapshot, MyValError> {
        let session = self.require_session().await?;
        let headers = build_riot_headers(&session, &self.cache.riot_client_version())?;
        let cached_party = self.last_party_snapshot.read().await.clone();
        if let Some(ref snap) = cached_party {
            if !snap.can_join_by_code {
                return Err(MyValError::InvalidInput(
                    "Leave queue before joining another party".to_string(),
                ));
            }
        } else {
            // Joining a party by code only works when Valorant PC client already has an active session/party
            let active = glz::get_party_player(&self.http, &self.limiter, &session, &headers).await?;
            if active.is_none() {
                return Err(MyValError::InvalidInput(
                    "Open Valorant on PC first to join a party by invite code.".to_string(),
                ));
            }
        }
        let raw = glz::party_join_by_code(&self.http, &self.limiter, &session, &headers, invite_code).await?;
        self.finish_party_action(&session, raw).await
    }

    async fn execute_pregame_agent_pick(
        &self,
        match_id: &str,
        agent_uuid: &str,
        lock: bool,
    ) -> Result<PreGameSnapshot, MyValError> {
        let session = self.require_session().await?;
        self.record_pregame_seen(match_id).await;
        let elapsed = self.pregame_elapsed_secs(match_id).await;
        validate_pregame_action_delay(elapsed, lock)?;
        if let Some(ref snap) = *self.last_pregame_snapshot.read().await {
            if snap.match_id.eq_ignore_ascii_case(match_id) {
                validate_pregame_agent_action(snap, agent_uuid)?;
            }
        }
        let headers = build_riot_headers(&session, &self.cache.riot_client_version())?;
        let raw = if lock {
            glz::pregame_lock_agent(
                &self.http,
                &self.limiter,
                &session,
                &headers,
                match_id,
                agent_uuid,
            )
            .await?
        } else {
            glz::pregame_select_agent(
                &self.http,
                &self.limiter,
                &session,
                &headers,
                match_id,
                agent_uuid,
            )
            .await?
        };
        self.finish_pregame_action(&session, match_id.to_string(), raw)
            .await
    }

    pub async fn pregame_select_agent(
        &self,
        match_id: &str,
        agent_uuid: &str,
    ) -> Result<PreGameSnapshot, MyValError> {
        self.execute_pregame_agent_pick(match_id, agent_uuid, false)
            .await
    }

    pub async fn pregame_lock_agent(
        &self,
        match_id: &str,
        agent_uuid: &str,
    ) -> Result<PreGameSnapshot, MyValError> {
        self.execute_pregame_agent_pick(match_id, agent_uuid, true)
            .await
    }

    pub async fn pregame_quit_match(&self, match_id: &str) -> Result<(), MyValError> {
        let session = self.require_session().await?;
        if let Some(ref snap) = *self.last_pregame_snapshot.read().await {
            if snap.match_id.eq_ignore_ascii_case(match_id) {
                validate_pregame_quit(snap)?;
            }
        }
        let headers = build_riot_headers(&session, &self.cache.riot_client_version())?;
        glz::pregame_quit_match(&self.http, &self.limiter, &session, &headers, match_id).await?;
        *self.last_pregame_snapshot.write().await = None;
        self.clear_pregame_seen().await;
        Ok(())
    }
}
