use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::MyValError;
use crate::models::assets::{
    Agent, AssetSummary, Buddy, BundleAsset, CompetitiveTier, ContentTier, FlexAsset, MapInfo,
    MissionAsset, PlayerCard, PlayerTitle, Skin, Spray, Weapon,
};
use crate::models::career::{
    CareerSnapshot, KnownPlayerSummary, MatchDetailSnapshot, PlayerProfileSnapshot,
};
use crate::models::loadout::LoadoutPreset;
use crate::models::mmr::PlayerMmrSummary;
use crate::models::store::StorefrontSnapshot;

pub const DEFAULT_FALLBACK_CLIENT_VERSION: &str = "release-10.00-shipping-1-3000000";
pub const MMR_CACHE_TTL_SECS: i64 = 2 * 60;
pub const MATCH_HISTORY_TTL_SECS: i64 = 3 * 60;
pub const NAME_CACHE_TTL_SECS: i64 = 24 * 60 * 60;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CachedMatchPartyMap {
    pub queue_id: String,
    /// lowercase puuid -> lowercase valid party_id
    pub player_parties: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CachedLiveMatchGuns {
    pub match_id: String,
    /// lowercase weapon_uuid -> (skin_uuid, skin_level_uuid, chroma_uuid)
    pub guns: HashMap<String, (String, String, String)>,
    /// Ordered/captured live spray & flex pairs: (slot_uuid, asset_uuid)
    #[serde(default)]
    pub sprays: Vec<(String, String)>,
}

pub struct LocalCache {
    conn: Mutex<Connection>,
}

impl LocalCache {
    pub fn open(db_path: &str) -> Result<Self, MyValError> {
        if db_path != ":memory:" {
            if let Some(parent) = Path::new(db_path).parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| MyValError::Storage(format!("Failed to create db dir: {e}")))?;
                }
            }
        }
        let conn = if db_path == ":memory:" {
            Connection::open_in_memory()?
        } else {
            Connection::open(db_path)?
        };
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA busy_timeout = 5000;
            PRAGMA cache_size = -16000;
            PRAGMA temp_store = MEMORY;

            CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS assets_kv (
                category TEXT NOT NULL,
                id TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                json TEXT NOT NULL,
                PRIMARY KEY (category, id)
            );
            CREATE INDEX IF NOT EXISTS idx_assets_category ON assets_kv(category, sort_order);
            CREATE INDEX IF NOT EXISTS idx_assets_cat_lower_id ON assets_kv(category, LOWER(id));

            CREATE TABLE IF NOT EXISTS skin_level_index (
                level_uuid TEXT PRIMARY KEY,
                skin_uuid TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS mmr_cache (
                puuid TEXT PRIMARY KEY,
                json TEXT NOT NULL,
                cached_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS match_history_cache (
                puuid TEXT PRIMARY KEY,
                json TEXT NOT NULL,
                cached_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS match_details_cache (
                match_id TEXT PRIMARY KEY,
                json TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS match_party_cache (
                match_id TEXT PRIMARY KEY,
                json TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS name_cache (
                puuid TEXT PRIMARY KEY,
                game_name TEXT NOT NULL,
                tag_line TEXT NOT NULL,
                cached_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS store_cache (
                puuid TEXT PRIMARY KEY,
                json TEXT NOT NULL,
                cached_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS wishlist (
                skin_uuid TEXT PRIMARY KEY,
                added_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS loadout_presets (
                preset_id TEXT PRIMARY KEY,
                puuid TEXT NOT NULL DEFAULT '',
                name TEXT NOT NULL,
                accent_color_hex TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                weapon_count INTEGER NOT NULL,
                preview_json TEXT NOT NULL,
                raw_loadout_json TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS owned_entitlements (
                puuid TEXT PRIMARY KEY,
                json TEXT NOT NULL,
                cached_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS career_cache (
                puuid TEXT PRIMARY KEY,
                json TEXT NOT NULL,
                cached_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS player_profile_cache (
                puuid TEXT PRIMARY KEY,
                json TEXT NOT NULL,
                cached_at INTEGER NOT NULL
            );
            ",
        )?;
        let _ = conn.execute(
            "ALTER TABLE loadout_presets ADD COLUMN puuid TEXT NOT NULL DEFAULT ''",
            [],
        );
        let inst = Self {
            conn: Mutex::new(conn),
        };
        if inst.get_meta("match_history_schema_version").ok().flatten().as_deref() != Some("2") {
            if let Ok(c) = inst.conn.lock() {
                let _ = c.execute("DELETE FROM match_history_cache", []);
            }
            let _ = inst.set_meta("match_history_schema_version", "2");
        }
        Ok(inst)
    }

    pub fn now_secs() -> i64 {
        Utc::now().timestamp()
    }

    pub fn set_meta(&self, key: &str, value: &str) -> Result<(), MyValError> {
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        conn.execute(
            "INSERT INTO meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_meta(&self, key: &str) -> Result<Option<String>, MyValError> {
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        conn.query_row("SELECT value FROM meta WHERE key = ?1", params![key], |r| r.get(0))
            .optional()
            .map_err(Into::into)
    }

    pub fn riot_client_version(&self) -> String {
        self.get_meta("riot_client_version")
            .ok()
            .flatten()
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| DEFAULT_FALLBACK_CLIENT_VERSION.to_string())
    }

    pub fn replace_category<T: Serialize, F>(
        &self,
        category: &str,
        items: &[T],
        id_fn: F,
    ) -> Result<(), MyValError>
    where
        F: Fn(&T) -> String,
    {
        let mut conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM assets_kv WHERE category = ?1", params![category])?;
        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO assets_kv (category, id, sort_order, json) VALUES (?1, ?2, ?3, ?4)",
            )?;
            for (idx, item) in items.iter().enumerate() {
                let id = id_fn(item);
                let json = serde_json::to_string(item)?;
                stmt.execute(params![category, id, idx as i64, json])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn rebuild_skin_level_index(&self, skins: &[Skin]) -> Result<(), MyValError> {
        let mut conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM skin_level_index", [])?;
        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO skin_level_index (level_uuid, skin_uuid) VALUES (?1, ?2)",
            )?;
            for skin in skins {
                stmt.execute(params![skin.uuid.to_lowercase(), skin.uuid])?;
                for level in &skin.levels {
                    if !level.uuid.is_empty() {
                        stmt.execute(params![level.uuid.to_lowercase(), skin.uuid])?;
                    }
                }
                for chroma in &skin.chromas {
                    if !chroma.uuid.is_empty() {
                        stmt.execute(params![chroma.uuid.to_lowercase(), skin.uuid])?;
                    }
                }
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn list_category<T: DeserializeOwned>(&self, category: &str) -> Result<Vec<T>, MyValError> {
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT json FROM assets_kv WHERE category = ?1 ORDER BY sort_order ASC",
        )?;
        let rows = stmt.query_map(params![category], |row| row.get::<_, String>(0))?;
        let mut out = Vec::new();
        for row in rows {
            let json = row?;
            out.push(serde_json::from_str::<T>(&json)?);
        }
        Ok(out)
    }

    pub fn get_asset<T: DeserializeOwned>(
        &self,
        category: &str,
        id: &str,
    ) -> Result<Option<T>, MyValError> {
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        let row: Option<String> = conn
            .query_row(
                "SELECT json FROM assets_kv WHERE category = ?1 AND LOWER(id) = LOWER(?2)",
                params![category, id],
                |r| r.get(0),
            )
            .optional()?;
        match row {
            Some(j) => Ok(Some(serde_json::from_str(&j)?)),
            None => Ok(None),
        }
    }

    pub fn lookup_skin_by_any_uuid_fast(&self, uuid: &str) -> Result<Option<Skin>, MyValError> {
        let normalized = uuid.to_lowercase();
        let skin_uuid: Option<String> = {
            let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
            conn.query_row(
                "SELECT skin_uuid FROM skin_level_index WHERE level_uuid = ?1",
                params![normalized],
                |r| r.get(0),
            )
            .optional()?
        };
        let target_id = skin_uuid.unwrap_or_else(|| uuid.to_string());
        self.get_asset("skin", &target_id)
    }

    pub fn lookup_skin_by_any_uuid(&self, uuid: &str) -> Result<Option<Skin>, MyValError> {
        let mut skin: Option<Skin> = self.lookup_skin_by_any_uuid_fast(uuid)?;
        if let Some(ref mut s) = skin {
            let wishlisted = self.get_wishlist_set()?;
            s.is_wishlisted = wishlisted.contains(&s.uuid.to_lowercase());
        }
        Ok(skin)
    }

    pub fn lookup_agent(&self, uuid: &str) -> Option<Agent> {
        self.get_asset("agent", uuid).ok().flatten()
    }

    pub fn lookup_weapon(&self, uuid: &str) -> Option<Weapon> {
        self.get_asset("weapon", uuid).ok().flatten()
    }

    pub fn lookup_player_card(&self, uuid: &str) -> Option<PlayerCard> {
        self.get_asset("playercard", uuid).ok().flatten()
    }

    pub fn lookup_bundle(&self, uuid: &str) -> Option<BundleAsset> {
        self.get_asset("bundle", uuid).ok().flatten()
    }

    pub fn lookup_spray(&self, uuid: &str) -> Option<Spray> {
        self.get_asset("spray", uuid).ok().flatten()
    }

    pub fn lookup_buddy(&self, uuid: &str) -> Option<Buddy> {
        self.get_asset("buddy", uuid).ok().flatten()
    }

    pub fn lookup_title(&self, uuid: &str) -> Option<PlayerTitle> {
        self.get_asset("title", uuid).ok().flatten()
    }

    pub fn lookup_flex(&self, uuid: &str) -> Option<FlexAsset> {
        self.get_asset("flex", uuid).ok().flatten()
    }

    pub fn lookup_mission(&self, uuid: &str) -> Option<MissionAsset> {
        self.get_asset("mission", uuid).ok().flatten()
    }

    pub fn list_all_skins(&self) -> Result<Vec<Skin>, MyValError> {
        self.list_category("skin")
    }

    pub fn lookup_content_tier(&self, uuid: &str) -> Option<ContentTier> {
        self.get_asset("contenttier", uuid).ok().flatten()
    }

    pub fn lookup_competitive_tier(&self, tier: i32) -> Option<CompetitiveTier> {
        self.get_asset("competitivetier", &tier.to_string())
            .ok()
            .flatten()
    }

    pub fn lookup_map_by_url_or_id(&self, map_url_or_id: &str) -> Option<MapInfo> {
        let needle = map_url_or_id.trim().to_lowercase();
        let maps: Vec<MapInfo> = self.list_category("map").ok()?;
        maps.into_iter().find(|m| {
            m.map_url.to_lowercase() == needle
                || m.uuid.to_lowercase() == needle
                || m.display_name.to_lowercase() == needle
        })
    }

    pub fn count_category(&self, category: &str) -> Result<u32, MyValError> {
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM assets_kv WHERE category = ?1",
            params![category],
            |r| r.get(0),
        )?;
        Ok(count.max(0) as u32)
    }

    pub fn get_asset_summary(&self) -> Result<AssetSummary, MyValError> {
        let synced_at = self
            .get_meta("assets_synced_at")?
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        Ok(AssetSummary {
            riot_client_version: self.riot_client_version(),
            synced_at_epoch_secs: synced_at,
            agent_count: self.count_category("agent")?,
            map_count: self.count_category("map")?,
            weapon_count: self.count_category("weapon")?,
            skin_count: self.count_category("skin")?,
            bundle_count: self.count_category("bundle")?,
            competitive_tier_count: self.count_category("competitivetier")?,
        })
    }

    pub fn search_skins(
        &self,
        query: &str,
        wishlist_only: bool,
        limit: u32,
    ) -> Result<Vec<Skin>, MyValError> {
        let mut all: Vec<Skin> = self.list_category("skin")?;
        let wishlist = self.get_wishlist_set()?;
        let q = query.trim().to_lowercase();
        let max_items = if limit == 0 { 100 } else { limit as usize };
        let mut filtered = Vec::new();
        for mut skin in all.drain(..) {
            skin.is_wishlisted = wishlist.contains(&skin.uuid.to_lowercase());
            if wishlist_only && !skin.is_wishlisted {
                continue;
            }
            if skin.display_name.starts_with("Standard ")
                || skin.display_name.contains("Random Favorite")
            {
                if q.is_empty() && !wishlist_only {
                    continue;
                }
            }
            if !q.is_empty() {
                let matches_name = skin.display_name.to_lowercase().contains(&q);
                let matches_weapon = skin.weapon_name.to_lowercase().contains(&q);
                if !matches_name && !matches_weapon {
                    continue;
                }
            }
            filtered.push(skin);
        }

        if q.is_empty() && !wishlist_only {
            let weapon_rank = |w: &str| -> u8 {
                match w {
                    "Vandal" => 0,
                    "Phantom" => 1,
                    "Melee" => 2,
                    "Operator" => 3,
                    "Sheriff" => 4,
                    "Ghost" => 5,
                    "Classic" => 6,
                    "Spectre" => 7,
                    "Guardian" => 8,
                    "Marshal" => 9,
                    "Outlaw" => 10,
                    "Bulldog" => 11,
                    "Bandit" => 12,
                    "Judge" => 13,
                    "Frenzy" => 14,
                    "Shorty" => 15,
                    "Stinger" => 16,
                    "Bucky" => 17,
                    "Ares" => 18,
                    "Odin" => 19,
                    _ => 25,
                }
            };
            let tier_rank = |t: Option<&str>| -> u8 {
                match t.unwrap_or_default() {
                    "Ultra Edition" | "Ultra" => 0,
                    "Exclusive Edition" | "Exclusive" => 1,
                    "Premium Edition" | "Premium" => 2,
                    "Deluxe Edition" | "Deluxe" => 3,
                    "Select Edition" | "Select" => 4,
                    _ => 5,
                }
            };
            filtered.sort_by(|a, b| {
                weapon_rank(&a.weapon_name)
                    .cmp(&weapon_rank(&b.weapon_name))
                    .then_with(|| {
                        tier_rank(a.content_tier_name.as_deref())
                            .cmp(&tier_rank(b.content_tier_name.as_deref()))
                    })
                    .then_with(|| a.display_name.cmp(&b.display_name))
            });
        }

        filtered.truncate(max_items);
        Ok(filtered)
    }

    pub fn toggle_wishlist(&self, skin_uuid: &str) -> Result<bool, MyValError> {
        let normalized = skin_uuid.trim().to_lowercase();
        if normalized.is_empty() {
            return Err(MyValError::InvalidInput("Empty skin_uuid".to_string()));
        }
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM wishlist WHERE skin_uuid = ?1",
                params![normalized],
                |_| Ok(true),
            )
            .optional()?
            .unwrap_or(false);
        if exists {
            conn.execute("DELETE FROM wishlist WHERE skin_uuid = ?1", params![normalized])?;
            Ok(false)
        } else {
            conn.execute(
                "INSERT INTO wishlist (skin_uuid, added_at) VALUES (?1, ?2)",
                params![normalized, Self::now_secs()],
            )?;
            Ok(true)
        }
    }

    pub fn get_wishlist_uuids(&self) -> Result<Vec<String>, MyValError> {
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT skin_uuid FROM wishlist ORDER BY added_at DESC")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn get_wishlist_set(&self) -> Result<HashSet<String>, MyValError> {
        Ok(self.get_wishlist_uuids()?.into_iter().collect())
    }

    // ---- MMR / Match History / Name / Storefront Caches ----

    pub fn get_cached_mmr(&self, puuid: &str, max_age_secs: i64) -> Option<PlayerMmrSummary> {
        let key = puuid.trim().to_lowercase();
        let conn = self.conn.lock().ok()?;
        let row: Option<(String, i64)> = conn
            .query_row(
                "SELECT json, cached_at FROM mmr_cache WHERE LOWER(puuid) = ?1",
                params![key],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .ok()?;
        let (json, cached_at) = row?;
        if Self::now_secs() - cached_at > max_age_secs {
            return None;
        }
        let summary: PlayerMmrSummary = serde_json::from_str(&json).ok()?;
        // Auto-heal legacy rows cached before competitiveupdates W/L/D tracking existed.
        if !summary.recent_results_loaded {
            return None;
        }
        Some(summary)
    }

    pub fn put_cached_mmr(&self, summary: &PlayerMmrSummary) -> Result<(), MyValError> {
        let key = summary.puuid.trim().to_lowercase();
        let json = serde_json::to_string(summary)?;
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        conn.execute(
            "INSERT INTO mmr_cache (puuid, json, cached_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(puuid) DO UPDATE SET json = excluded.json, cached_at = excluded.cached_at",
            params![key, json, Self::now_secs()],
        )?;
        Ok(())
    }

    pub fn get_cached_match_history(
        &self,
        puuid: &str,
        max_age_secs: i64,
    ) -> Option<Vec<String>> {
        let key = puuid.trim().to_lowercase();
        let conn = self.conn.lock().ok()?;
        let row: Option<(String, i64)> = conn
            .query_row(
                "SELECT json, cached_at FROM match_history_cache WHERE LOWER(puuid) = ?1",
                params![key],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .ok()?;
        let (json, cached_at) = row?;
        if Self::now_secs() - cached_at > max_age_secs {
            return None;
        }
        serde_json::from_str(&json).ok()
    }

    pub fn put_cached_match_history(
        &self,
        puuid: &str,
        match_ids: &[String],
    ) -> Result<(), MyValError> {
        let key = puuid.trim().to_lowercase();
        let json = serde_json::to_string(match_ids)?;
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        conn.execute(
            "INSERT INTO match_history_cache (puuid, json, cached_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(puuid) DO UPDATE SET json = excluded.json, cached_at = excluded.cached_at",
            params![key, json, Self::now_secs()],
        )?;
        Ok(())
    }

    pub fn get_cached_names(&self, puuids: &[String]) -> HashMap<String, (String, String)> {
        let mut out = HashMap::new();
        let Ok(conn) = self.conn.lock() else {
            return out;
        };
        let now = Self::now_secs();
        let Ok(mut stmt) = conn.prepare(
            "SELECT game_name, tag_line, cached_at FROM name_cache WHERE LOWER(puuid) = LOWER(?1)",
        ) else {
            return out;
        };
        for p in puuids {
            let key = p.to_lowercase();
            if let Ok(Some((g, t, ts))) = stmt
                .query_row(params![&key], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, i64>(2)?,
                    ))
                })
                .optional()
            {
                if now - ts <= NAME_CACHE_TTL_SECS && !g.trim().is_empty() {
                    out.insert(key, (g, t));
                }
            }
        }
        out
    }

    pub fn put_cached_names(&self, entries: &[(String, String, String)]) -> Result<(), MyValError> {
        let mut conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        let now = Self::now_secs();
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO name_cache (puuid, game_name, tag_line, cached_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(puuid) DO UPDATE SET
                   game_name = excluded.game_name,
                   tag_line = excluded.tag_line,
                   cached_at = excluded.cached_at",
            )?;
            for (puuid, game_name, tag_line) in entries {
                if game_name.trim().is_empty() {
                    continue;
                }
                stmt.execute(params![puuid.to_lowercase(), game_name.trim(), tag_line.trim(), now])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn put_cached_storefront(
        &self,
        puuid: &str,
        snapshot: &StorefrontSnapshot,
    ) -> Result<(), MyValError> {
        let key = puuid.trim().to_lowercase();
        let json = serde_json::to_string(snapshot)?;
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        conn.execute(
            "INSERT INTO store_cache (puuid, json, cached_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(puuid) DO UPDATE SET json = excluded.json, cached_at = excluded.cached_at",
            params![key, json, Self::now_secs()],
        )?;
        Ok(())
    }

    pub fn get_cached_storefront(&self, puuid: &str) -> Result<Option<StorefrontSnapshot>, MyValError> {
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        let row: Option<String> = conn
            .query_row(
                "SELECT json FROM store_cache WHERE LOWER(puuid) = LOWER(?1)",
                params![puuid.trim()],
                |r| r.get(0),
            )
            .optional()?;
        match row {
            Some(j) => {
                let mut snap: StorefrontSnapshot = serde_json::from_str(&j)?;
                let elapsed = (Self::now_secs() - snap.fetched_at_epoch_secs).max(0);
                if snap.daily_remaining_secs > 0 && elapsed >= snap.daily_remaining_secs {
                    return Ok(None);
                }
                if elapsed > 0 {
                    snap.daily_remaining_secs = (snap.daily_remaining_secs - elapsed).max(0);
                    snap.accessory_remaining_secs = (snap.accessory_remaining_secs - elapsed).max(0);
                    for bundle in &mut snap.featured_bundles {
                        bundle.remaining_secs = (bundle.remaining_secs - elapsed).max(0);
                    }
                    if let Some(ref mut nm) = snap.night_market {
                        nm.remaining_secs = (nm.remaining_secs - elapsed).max(0);
                    }
                }
                Ok(Some(snap))
            }
            None => Ok(None),
        }
    }

    // ---- Match Details, Career & Loadout Presets ----

    pub fn get_cached_match_detail(&self, match_id: &str, viewer_puuid: &str) -> Option<MatchDetailSnapshot> {
        let row: Option<String> = {
            let conn = self.conn.lock().ok()?;
            conn.query_row(
                "SELECT json FROM match_details_cache WHERE match_id = ?1",
                params![match_id],
                |r| r.get(0),
            )
            .optional()
            .ok()?
        };
        let detail: MatchDetailSnapshot = serde_json::from_str(&row?).ok()?;
        // Heal legacy cached rows where Riot match-details omitted player display names
        // OR predated full 10-player replay position snapshots
        if detail
            .scoreboard
            .iter()
            .any(|p| p.display_name.trim().is_empty() || p.display_name == p.agent_name)
        {
            return None;
        }
        let mut total_kills = 0usize;
        let mut any_has_positions = false;
        let mut missing_weapon_icons = false;
        for r in &detail.rounds {
            for k in &r.kills {
                total_kills += 1;
                if !k.player_positions.is_empty() {
                    any_has_positions = true;
                }
                if k.weapon_icon.is_none() {
                    missing_weapon_icons = true;
                }
            }
        }
        if total_kills > 0 && !any_has_positions {
            return None;
        }
        let mut detail = detail;
        if missing_weapon_icons {
            if let Ok(weapons) = self.list_category::<Weapon>("weapon") {
                let by_name: HashMap<String, String> = weapons
                    .into_iter()
                    .filter_map(|w| {
                        w.kill_stream_icon_url()
                            .map(|icon| (w.display_name.trim().to_lowercase(), icon))
                    })
                    .collect();
                for r in &mut detail.rounds {
                    for k in &mut r.kills {
                        if k.weapon_icon.is_none() {
                            k.weapon_icon = by_name
                                .get(&k.weapon_name.trim().to_lowercase())
                                .cloned();
                        }
                    }
                }
            }
        }
        Some(reperspective_match_detail(detail, viewer_puuid))
    }

    pub fn clear_mmr_and_career_cache(&self) -> Result<(), MyValError> {
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        conn.execute("DELETE FROM mmr_cache", [])?;
        conn.execute("DELETE FROM career_cache", [])?;
        conn.execute("DELETE FROM player_profile_cache", [])?;
        Ok(())
    }

    pub fn put_owned_skin_uuids(&self, puuid: &str, skin_uuids: &[String]) -> Result<(), MyValError> {
        let json = serde_json::to_string(skin_uuids)?;
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO owned_entitlements (puuid, json, cached_at) VALUES (?1, ?2, ?3)",
            params![puuid, json, Self::now_secs()],
        )?;
        Ok(())
    }

    pub fn get_owned_skin_uuids(&self, puuid: &str) -> HashSet<String> {
        let Ok(conn) = self.conn.lock() else {
            return HashSet::new();
        };
        let row: Option<String> = conn
            .query_row(
                "SELECT json FROM owned_entitlements WHERE LOWER(puuid) = LOWER(?1)",
                params![puuid],
                |r| r.get(0),
            )
            .optional()
            .ok()
            .flatten();
        row.and_then(|j| serde_json::from_str::<Vec<String>>(&j).ok())
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.to_lowercase())
            .collect()
    }

    pub fn put_owned_skin_level_uuids(&self, puuid: &str, level_uuids: &[String]) -> Result<(), MyValError> {
        let normalized: Vec<String> = level_uuids.iter().map(|s| s.to_lowercase()).collect();
        let json = serde_json::to_string(&normalized)?;
        self.set_meta(&format!("owned_skin_levels_{}", puuid.to_lowercase()), &json)
    }

    pub fn get_owned_skin_level_uuids(&self, puuid: &str) -> HashSet<String> {
        self.get_meta(&format!("owned_skin_levels_{}", puuid.to_lowercase()))
            .ok()
            .flatten()
            .and_then(|j| serde_json::from_str::<Vec<String>>(&j).ok())
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.to_lowercase())
            .collect()
    }

    pub fn put_owned_agent_uuids(&self, puuid: &str, agent_uuids: &[String]) -> Result<(), MyValError> {
        let normalized: Vec<String> = agent_uuids.iter().map(|s| s.to_lowercase()).collect();
        let json = serde_json::to_string(&normalized)?;
        self.set_meta(&format!("owned_agents_{}", puuid.to_lowercase()), &json)
    }

    pub fn get_owned_agent_uuids(&self, puuid: &str) -> HashSet<String> {
        self.get_meta(&format!("owned_agents_{}", puuid.to_lowercase()))
            .ok()
            .flatten()
            .and_then(|j| serde_json::from_str::<Vec<String>>(&j).ok())
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.to_lowercase())
            .collect()
    }

    pub fn put_owned_chroma_uuids(&self, puuid: &str, chroma_uuids: &[String]) -> Result<(), MyValError> {
        let normalized: Vec<String> = chroma_uuids.iter().map(|s| s.to_lowercase()).collect();
        let json = serde_json::to_string(&normalized)?;
        self.set_meta(&format!("owned_skin_chromas_{}", puuid.to_lowercase()), &json)
    }

    pub fn get_owned_chroma_uuids(&self, puuid: &str) -> HashSet<String> {
        self.get_meta(&format!("owned_skin_chromas_{}", puuid.to_lowercase()))
            .ok()
            .flatten()
            .and_then(|j| serde_json::from_str::<Vec<String>>(&j).ok())
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.to_lowercase())
            .collect()
    }

    fn put_owned_meta_uuids(&self, prefix: &str, puuid: &str, uuids: &[String]) -> Result<(), MyValError> {
        let normalized: Vec<String> = uuids.iter().map(|s| s.to_lowercase()).collect();
        let json = serde_json::to_string(&normalized)?;
        self.set_meta(&format!("{prefix}_{}", puuid.to_lowercase()), &json)
    }

    fn get_owned_meta_uuids(&self, prefix: &str, puuid: &str) -> HashSet<String> {
        self.get_meta(&format!("{prefix}_{}", puuid.to_lowercase()))
            .ok()
            .flatten()
            .and_then(|j| serde_json::from_str::<Vec<String>>(&j).ok())
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.to_lowercase())
            .collect()
    }

    pub fn put_owned_spray_uuids(&self, puuid: &str, uuids: &[String]) -> Result<(), MyValError> {
        self.put_owned_meta_uuids("owned_sprays", puuid, uuids)
    }

    pub fn get_owned_spray_uuids(&self, puuid: &str) -> HashSet<String> {
        self.get_owned_meta_uuids("owned_sprays", puuid)
    }

    pub fn put_owned_card_uuids(&self, puuid: &str, uuids: &[String]) -> Result<(), MyValError> {
        self.put_owned_meta_uuids("owned_cards", puuid, uuids)
    }

    pub fn get_owned_card_uuids(&self, puuid: &str) -> HashSet<String> {
        self.get_owned_meta_uuids("owned_cards", puuid)
    }

    pub fn put_owned_title_uuids(&self, puuid: &str, uuids: &[String]) -> Result<(), MyValError> {
        self.put_owned_meta_uuids("owned_titles", puuid, uuids)
    }

    pub fn get_owned_title_uuids(&self, puuid: &str) -> HashSet<String> {
        self.get_owned_meta_uuids("owned_titles", puuid)
    }

    pub fn put_live_match_guns(&self, puuid: &str, guns: &CachedLiveMatchGuns) -> Result<(), MyValError> {
        let json = serde_json::to_string(guns)?;
        self.set_meta(&format!("live_match_guns_{}", puuid.to_lowercase()), &json)
    }

    pub fn get_live_match_guns(&self, puuid: &str) -> Option<CachedLiveMatchGuns> {
        let raw = self
            .get_meta(&format!("live_match_guns_{}", puuid.to_lowercase()))
            .ok()
            .flatten()?;
        if raw.trim().is_empty() {
            return None;
        }
        serde_json::from_str::<CachedLiveMatchGuns>(&raw).ok()
    }

    pub fn clear_live_match_guns(&self, puuid: &str) {
        let key = format!("live_match_guns_{}", puuid.to_lowercase());
        if let Ok(conn) = self.conn.lock() {
            let _ = conn.execute("DELETE FROM meta WHERE key = ?1", params![key]);
        }
    }

    pub fn needs_asset_resync(&self) -> bool {
        let ver = self.get_meta("assets_schema_version").ok().flatten();
        if ver.as_deref() != Some("4") {
            return true;
        }
        let synced_at = self
            .get_meta("assets_synced_at")
            .ok()
            .flatten()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        if synced_at == 0 || (Self::now_secs() - synced_at) > 86_400 {
            return true;
        }
        self.count_category("skin").unwrap_or(0) == 0
            || self.count_category("season").unwrap_or(0) == 0
    }

    pub fn put_cached_match_detail(&self, detail: &MatchDetailSnapshot) -> Result<(), MyValError> {
        let json = serde_json::to_string(detail)?;
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO match_details_cache (match_id, json) VALUES (?1, ?2)",
            params![detail.match_id, json],
        )?;
        Ok(())
    }

    pub fn get_cached_match_party_map(&self, match_id: &str) -> Option<CachedMatchPartyMap> {
        let key = match_id.trim().to_lowercase();
        if key.is_empty() {
            return None;
        }
        let conn = self.conn.lock().ok()?;
        let json: String = conn
            .query_row(
                "SELECT json FROM match_party_cache WHERE match_id = ?1",
                params![key],
                |r| r.get(0),
            )
            .optional()
            .ok()??;
        serde_json::from_str(&json).ok()
    }

    pub fn put_cached_match_party_map(
        &self,
        match_id: &str,
        data: &CachedMatchPartyMap,
    ) -> Result<(), MyValError> {
        let key = match_id.trim().to_lowercase();
        if key.is_empty() {
            return Ok(());
        }
        let json = serde_json::to_string(data)?;
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO match_party_cache (match_id, json) VALUES (?1, ?2)",
            params![key, json],
        )?;
        Ok(())
    }

    pub fn get_cached_career(&self, puuid: &str, max_age_secs: i64) -> Option<CareerSnapshot> {
        let conn = self.conn.lock().ok()?;
        let (json, ts): (String, i64) = conn
            .query_row(
                "SELECT json, cached_at FROM career_cache WHERE LOWER(puuid) = LOWER(?1)",
                params![puuid.trim()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .ok()??;
        if Self::now_secs() - ts > max_age_secs {
            return None;
        }
        serde_json::from_str(&json).ok()
    }

    pub fn put_cached_career(&self, puuid: &str, career: &CareerSnapshot) -> Result<(), MyValError> {
        let json = serde_json::to_string(career)?;
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO career_cache (puuid, json, cached_at) VALUES (?1, ?2, ?3)",
            params![puuid.trim().to_lowercase(), json, Self::now_secs()],
        )?;
        Ok(())
    }

    pub fn get_cached_player_profile(
        &self,
        puuid: &str,
        max_age_secs: i64,
    ) -> Option<PlayerProfileSnapshot> {
        let conn = self.conn.lock().ok()?;
        let (json, ts): (String, i64) = conn
            .query_row(
                "SELECT json, cached_at FROM player_profile_cache WHERE LOWER(puuid) = LOWER(?1)",
                params![puuid],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .ok()??;
        if Self::now_secs() - ts > max_age_secs {
            return None;
        }
        serde_json::from_str(&json).ok()
    }

    pub fn put_cached_player_profile(
        &self,
        puuid: &str,
        profile: &PlayerProfileSnapshot,
    ) -> Result<(), MyValError> {
        let json = serde_json::to_string(profile)?;
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO player_profile_cache (puuid, json, cached_at) VALUES (?1, ?2, ?3)",
            params![puuid.to_lowercase(), json, Self::now_secs()],
        )?;
        Ok(())
    }

    pub fn cache_player_identity_meta(
        &self,
        puuid: &str,
        card_id: Option<&str>,
        title_id: Option<&str>,
        account_level: Option<i32>,
    ) {
        let key = puuid.trim().to_lowercase();
        if key.is_empty() {
            return;
        }
        if let Some(c) = card_id.map(str::trim).filter(|s| !s.is_empty()) {
            let _ = self.set_meta(&format!("player_card_{key}"), c);
        }
        if let Some(t) = title_id.map(str::trim).filter(|s| !s.is_empty()) {
            let _ = self.set_meta(&format!("player_title_{key}"), t);
        }
        if let Some(lvl) = account_level.filter(|&l| l > 0) {
            let _ = self.set_meta(&format!("player_level_{key}"), &lvl.to_string());
        }
    }

    pub fn get_cached_player_identity_meta(
        &self,
        puuid: &str,
    ) -> (Option<String>, Option<String>, Option<i32>) {
        let key = puuid.trim().to_lowercase();
        if key.is_empty() {
            return (None, None, None);
        }
        let card = self
            .get_meta(&format!("player_card_{key}"))
            .ok()
            .flatten()
            .or_else(|| self.get_meta(&format!("player_card_{puuid}")).ok().flatten());
        let title = self
            .get_meta(&format!("player_title_{key}"))
            .ok()
            .flatten()
            .or_else(|| self.get_meta(&format!("player_title_{puuid}")).ok().flatten());
        let level = self
            .get_meta(&format!("player_level_{key}"))
            .ok()
            .flatten()
            .and_then(|s| s.parse::<i32>().ok())
            .filter(|&l| l > 0);
        (card, title, level)
    }

    pub fn search_known_players(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<Vec<KnownPlayerSummary>, MyValError> {
        let q = query.trim().to_lowercase();
        let cap = limit.clamp(1, 100) as usize;
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT n.puuid, n.game_name, n.tag_line, m.json
             FROM name_cache n
             LEFT JOIN mmr_cache m ON LOWER(m.puuid) = LOWER(n.puuid)
             WHERE TRIM(n.game_name) != ''
             ORDER BY n.cached_at DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })?;
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        for row in rows {
            let (puuid, gn, tl, mmr_json) = row?;
            let display_name = if tl.trim().is_empty() {
                gn.trim().to_string()
            } else {
                format!("{}#{}", gn.trim(), tl.trim())
            };
            if !q.is_empty() && !display_name.to_lowercase().contains(&q) {
                continue;
            }
            if !seen.insert(puuid.to_lowercase()) {
                continue;
            }
            let current_rank = mmr_json
                .and_then(|j| serde_json::from_str::<PlayerMmrSummary>(&j).ok())
                .map(|m| m.current_rank)
                .filter(|r| r.tier >= 3);
            out.push(KnownPlayerSummary {
                puuid,
                display_name,
                current_rank,
            });
            if out.len() >= cap {
                break;
            }
        }
        Ok(out)
    }

    pub fn find_cached_puuid_by_riot_id(&self, game_name: &str, tag_line: &str) -> Option<String> {
        let gn = game_name.trim();
        let tl = tag_line.trim();
        if gn.is_empty() || tl.is_empty() {
            return None;
        }
        let conn = self.conn.lock().ok()?;
        let direct: Option<String> = conn
            .query_row(
                "SELECT puuid FROM name_cache
                 WHERE LOWER(TRIM(game_name)) = LOWER(?1) AND LOWER(TRIM(tag_line)) = LOWER(?2)
                 ORDER BY cached_at DESC LIMIT 1",
                params![gn, tl],
                |r| r.get(0),
            )
            .optional()
            .ok()
            .flatten();
        if direct.is_some() || (gn.is_ascii() && tl.is_ascii()) {
            return direct;
        }
        // Unicode fallback: SQLite's built-in LOWER() is ASCII-only (misses 'Ø', 'Ξ', etc.)
        let gn_lower = gn.to_lowercase();
        let tl_lower = tl.to_lowercase();
        let mut stmt = conn
            .prepare("SELECT puuid, game_name, tag_line FROM name_cache ORDER BY cached_at DESC")
            .ok()?;
        let mut rows = stmt.query([]).ok()?;
        while let Ok(Some(row)) = rows.next() {
            let p: String = row.get(0).ok()?;
            let g: String = row.get(1).ok()?;
            let t: String = row.get(2).ok()?;
            if g.trim().to_lowercase() == gn_lower && t.trim().to_lowercase() == tl_lower {
                return Some(p);
            }
        }
        None
    }

    pub fn set_lookup_api_key(&self, key: &str) -> Result<(), MyValError> {
        self.set_meta("lookup_api_key", key.trim())
    }

    pub fn get_lookup_api_key(&self) -> Option<String> {
        self.get_meta("lookup_api_key")
            .ok()
            .flatten()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    pub fn save_loadout_preset(
        &self,
        puuid: &str,
        name: &str,
        accent_color_hex: &str,
        weapon_count: u32,
        preview_skins: &[String],
        raw_loadout_json: &str,
    ) -> Result<LoadoutPreset, MyValError> {
        let clean_name = name.trim();
        if clean_name.is_empty() {
            return Err(MyValError::InvalidInput("Preset name cannot be empty".to_string()));
        }
        static PRESET_SEQ: AtomicU64 = AtomicU64::new(1);
        let now = Self::now_secs();
        let seq = PRESET_SEQ.fetch_add(1, Ordering::Relaxed);
        let preset_id = format!("preset-{}-{seq}", Utc::now().timestamp_millis());
        let preview_json = serde_json::to_string(preview_skins)?;
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO loadout_presets
             (preset_id, puuid, name, accent_color_hex, created_at, weapon_count, preview_json, raw_loadout_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                preset_id,
                puuid.to_lowercase(),
                clean_name,
                accent_color_hex,
                now,
                weapon_count as i64,
                preview_json,
                raw_loadout_json
            ],
        )?;
        Ok(LoadoutPreset {
            preset_id,
            name: clean_name.to_string(),
            accent_color_hex: accent_color_hex.to_string(),
            created_at_epoch_secs: now,
            weapon_count,
            preview_skins: preview_skins.to_vec(),
        })
    }

    pub fn list_loadout_presets(&self, puuid: Option<&str>) -> Result<Vec<LoadoutPreset>, MyValError> {
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT preset_id, puuid, name, accent_color_hex, created_at, weapon_count, preview_json, raw_loadout_json
             FROM loadout_presets ORDER BY created_at DESC",
        )?;
        let target_puuid = puuid.map(|p| p.to_lowercase()).filter(|p| !p.is_empty());
        let rows = stmt.query_map([], |r| {
            let preview_json: String = r.get(6)?;
            let preview_skins: Vec<String> = serde_json::from_str(&preview_json).unwrap_or_default();
            Ok((
                r.get::<_, String>(1)?,
                r.get::<_, String>(7)?,
                LoadoutPreset {
                    preset_id: r.get(0)?,
                    name: r.get(2)?,
                    accent_color_hex: r.get(3)?,
                    created_at_epoch_secs: r.get(4)?,
                    weapon_count: r.get::<_, i64>(5)?.max(0) as u32,
                    preview_skins,
                },
            ))
        })?;
        let mut out = Vec::new();
        for r in rows {
            let (row_puuid, raw_json, preset) = r?;
            if let Some(ref target) = target_puuid {
                let row_lower = row_puuid.to_lowercase();
                if !row_lower.is_empty() && &row_lower != target {
                    continue;
                }
                if row_lower.is_empty() && !raw_json.to_lowercase().contains(target) && raw_json.len() > 4 {
                    continue;
                }
            }
            out.push(preset);
        }
        Ok(out)
    }

    pub fn get_loadout_preset_raw(&self, preset_id: &str) -> Result<Option<(String, String)>, MyValError> {
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        conn.query_row(
            "SELECT puuid, raw_loadout_json FROM loadout_presets WHERE preset_id = ?1",
            params![preset_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn delete_loadout_preset(&self, preset_id: &str) -> Result<(), MyValError> {
        let conn = self.conn.lock().map_err(|e| MyValError::Storage(e.to_string()))?;
        conn.execute(
            "DELETE FROM loadout_presets WHERE preset_id = ?1",
            params![preset_id],
        )?;
        Ok(())
    }
}

pub fn reperspective_match_detail(
    mut detail: MatchDetailSnapshot,
    viewer_puuid: &str,
) -> MatchDetailSnapshot {
    let trimmed = viewer_puuid.trim();
    if trimmed.is_empty() {
        return detail;
    }
    if let Some(target_team) = detail
        .scoreboard
        .iter()
        .find(|p| p.puuid.eq_ignore_ascii_case(trimmed))
        .map(|p| p.team_id.clone())
    {
        if !target_team.eq_ignore_ascii_case(&detail.my_team_id) {
            std::mem::swap(&mut detail.my_team_score, &mut detail.enemy_team_score);
            detail.my_team_id = target_team;
            detail.won = detail.my_team_score > detail.enemy_team_score;
        }
        for p in &mut detail.scoreboard {
            p.is_me = p.puuid.eq_ignore_ascii_case(trimmed);
        }
    }
    detail
}
