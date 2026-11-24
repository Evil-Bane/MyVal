use std::collections::{HashMap, HashSet};

use reqwest::header::HeaderMap;
use serde::{Deserialize, Deserializer, Serialize};

use crate::cache::LocalCache;
use crate::error::MyValError;
use crate::models::assets::Skin;
use crate::models::auth::RiotSession;
use crate::models::loadout::{
    CollectionRarityBreakdown, CollectionSummary, LoadoutGunSlot, LoadoutSpraySlot,
    PlayerLoadoutSnapshot,
};
use crate::models::store::StorefrontSnapshot;
use crate::pd::{
    pd_base, CURRENCY_VP, ITEM_TYPE_AGENT, ITEM_TYPE_FLEX, ITEM_TYPE_PLAYER_CARD,
    ITEM_TYPE_SKIN_CHROMA, ITEM_TYPE_SKIN_LEVEL, ITEM_TYPE_SPRAY, ITEM_TYPE_TITLE,
    STARTER_AGENT_UUIDS,
};
use crate::rate_limit::RateLimiter;

fn null_to_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    let opt = Option::<T>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawLoadoutExpression {
    #[serde(rename = "TypeID", deserialize_with = "null_to_default")]
    pub type_id: String,
    #[serde(rename = "AssetID", deserialize_with = "null_to_default")]
    pub asset_id: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawPlayerLoadout {
    #[serde(deserialize_with = "null_to_default")]
    pub subject: String,
    #[serde(deserialize_with = "null_to_default")]
    pub version: i32,
    #[serde(deserialize_with = "null_to_default")]
    pub guns: Vec<RawLoadoutGun>,
    #[serde(deserialize_with = "null_to_default")]
    pub sprays: Vec<RawLoadoutSpray>,
    #[serde(deserialize_with = "null_to_default")]
    pub active_expressions: Vec<RawLoadoutExpression>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_options: Option<serde_json::Value>,
    #[serde(deserialize_with = "null_to_default")]
    pub identity: RawLoadoutIdentity,
    #[serde(deserialize_with = "null_to_default")]
    pub incognito: bool,
    #[serde(default)]
    pub api_version: u8,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawLoadoutPutBody {
    pub guns: Vec<RawLoadoutGun>,
    pub sprays: Vec<RawLoadoutSpray>,
    pub identity: RawLoadoutIdentity,
    pub incognito: bool,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawLoadoutPutBodyV3 {
    pub guns: Vec<RawLoadoutGun>,
    pub active_expressions: Vec<RawLoadoutExpression>,
    pub dynamic_options: serde_json::Value,
    pub identity: RawLoadoutIdentity,
    pub incognito: bool,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawLoadoutGun {
    #[serde(rename = "ID", deserialize_with = "null_to_default")]
    pub id: String,
    #[serde(rename = "CharmInstanceID", skip_serializing_if = "Option::is_none")]
    pub charm_instance_id: Option<String>,
    #[serde(rename = "CharmID", skip_serializing_if = "Option::is_none")]
    pub charm_id: Option<String>,
    #[serde(rename = "CharmLevelID", skip_serializing_if = "Option::is_none")]
    pub charm_level_id: Option<String>,
    #[serde(rename = "SkinID", deserialize_with = "null_to_default")]
    pub skin_id: String,
    #[serde(rename = "SkinLevelID", deserialize_with = "null_to_default")]
    pub skin_level_id: String,
    #[serde(rename = "ChromaID", deserialize_with = "null_to_default")]
    pub chroma_id: String,
    #[serde(deserialize_with = "null_to_default")]
    pub attachments: Vec<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawLoadoutSpray {
    #[serde(rename = "EquipSlotID", alias = "SocketID", deserialize_with = "null_to_default")]
    pub equip_slot_id: String,
    #[serde(rename = "SprayID", alias = "AssetID", deserialize_with = "null_to_default")]
    pub spray_id: String,
    #[serde(rename = "SprayLevelID")]
    pub spray_level_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawLoadoutIdentity {
    #[serde(rename = "PlayerCardID", deserialize_with = "null_to_default")]
    pub player_card_id: String,
    #[serde(rename = "PlayerTitleID", deserialize_with = "null_to_default")]
    pub player_title_id: String,
    #[serde(deserialize_with = "null_to_default")]
    pub account_level: i32,
    #[serde(rename = "PreferredLevelBorderID", deserialize_with = "null_to_default")]
    pub preferred_level_border_id: String,
    #[serde(deserialize_with = "null_to_default")]
    pub hide_account_level: bool,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawEntitlementsResp {
    #[serde(deserialize_with = "null_to_default")]
    pub entitlements: Vec<RawEntitlementItem>,
    #[serde(deserialize_with = "null_to_default")]
    pub entitlements_by_types: Vec<RawEntitlementsByType>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawEntitlementsByType {
    #[serde(rename = "ItemTypeID", deserialize_with = "null_to_default")]
    pub item_type_id: String,
    #[serde(deserialize_with = "null_to_default")]
    pub entitlements: Vec<RawEntitlementItem>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct RawEntitlementItem {
    #[serde(rename = "TypeID", deserialize_with = "null_to_default")]
    pub type_id: String,
    #[serde(rename = "ItemID", deserialize_with = "null_to_default")]
    pub item_id: String,
    #[serde(rename = "InstanceID")]
    pub instance_id: Option<String>,
}

impl RawEntitlementsResp {
    pub fn into_items(self, filter_type_id: Option<&str>) -> Vec<RawEntitlementItem> {
        if !self.entitlements.is_empty() {
            return self.entitlements;
        }
        let mut out = Vec::new();
        for group in self.entitlements_by_types {
            if let Some(tid) = filter_type_id {
                if !group.item_type_id.is_empty() && !group.item_type_id.eq_ignore_ascii_case(tid) {
                    continue;
                }
            }
            out.extend(group.entitlements);
        }
        out
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
struct RawOffersResp {
    #[serde(deserialize_with = "null_to_default")]
    pub offers: Vec<RawCatalogOffer>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
struct RawCatalogOffer {
    #[serde(rename = "OfferID", deserialize_with = "null_to_default")]
    pub offer_id: String,
    #[serde(deserialize_with = "null_to_default")]
    pub cost: HashMap<String, i64>,
}

pub async fn fetch_raw_loadout(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
) -> Result<RawPlayerLoadout, MyValError> {
    let v3_url = format!(
        "{}/personalization/v3/players/{}/playerloadout",
        pd_base(&session.shard),
        session.puuid
    );
    match limiter
        .request_json::<RawPlayerLoadout, _>(|| http.get(&v3_url).headers(headers.clone()))
        .await
    {
        Ok(mut raw) => {
            raw.api_version = 3;
            Ok(raw)
        }
        Err(MyValError::Api { status: 404, .. }) => {
            let v2_url = format!(
                "{}/personalization/v2/players/{}/playerloadout",
                pd_base(&session.shard),
                session.puuid
            );
            let mut raw: RawPlayerLoadout = limiter
                .request_json(|| http.get(&v2_url).headers(headers.clone()))
                .await?;
            raw.api_version = 2;
            Ok(raw)
        }
        Err(e) => Err(e),
    }
}

pub async fn put_raw_loadout(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    session: &RiotSession,
    headers: &HeaderMap,
    raw: &RawPlayerLoadout,
) -> Result<RawPlayerLoadout, MyValError> {
    let mut extra = raw.extra.clone();
    extra.retain(|k, _| {
        !k.eq_ignore_ascii_case("Subject")
            && !k.eq_ignore_ascii_case("Version")
            && !k.eq_ignore_ascii_case("Sprays")
            && !k.eq_ignore_ascii_case("ActiveExpressions")
            && !k.eq_ignore_ascii_case("DynamicOptions")
            && !k.eq_ignore_ascii_case("ApiVersion")
    });

    if raw.api_version != 2 {
        let v3_url = format!(
            "{}/personalization/v3/players/{}/playerloadout",
            pd_base(&session.shard),
            session.puuid
        );
        let body_v3 = RawLoadoutPutBodyV3 {
            guns: raw.guns.clone(),
            active_expressions: raw.active_expressions.clone(),
            dynamic_options: raw
                .dynamic_options
                .clone()
                .unwrap_or_else(|| serde_json::json!({})),
            identity: raw.identity.clone(),
            incognito: raw.incognito,
            extra: extra.clone(),
        };
        match limiter
            .request_json::<RawPlayerLoadout, _>(|| {
                http.put(&v3_url).headers(headers.clone()).json(&body_v3)
            })
            .await
        {
            Ok(mut updated) => {
                updated.api_version = 3;
                return Ok(updated);
            }
            Err(MyValError::Api { status: 404, .. }) if raw.api_version == 0 => {}
            Err(e) => return Err(e),
        }
    }

    let v2_url = format!(
        "{}/personalization/v2/players/{}/playerloadout",
        pd_base(&session.shard),
        session.puuid
    );
    let body_v2 = RawLoadoutPutBody {
        guns: raw.guns.clone(),
        sprays: raw.sprays.clone(),
        identity: raw.identity.clone(),
        incognito: raw.incognito,
        extra,
    };
    let mut updated: RawPlayerLoadout = limiter
        .request_json(|| http.put(&v2_url).headers(headers.clone()).json(&body_v2))
        .await?;
    updated.api_version = 2;
    Ok(updated)
}

pub fn save_cached_loadout(cache: &LocalCache, puuid: &str, raw: &RawPlayerLoadout) {
    if let Ok(json) = serde_json::to_string(raw) {
        let _ = cache.set_meta(&format!("active_loadout_{puuid}"), &json);
    }
}

pub fn load_cached_loadout(cache: &LocalCache, puuid: &str) -> Option<RawPlayerLoadout> {
    let json = cache.get_meta(&format!("active_loadout_{puuid}")).ok()??;
    let parsed: RawPlayerLoadout = serde_json::from_str(&json).ok()?;
    if parsed.guns.is_empty() {
        return None;
    }
    Some(parsed)
}

pub async fn sync_or_persist_loadout(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    cache: &LocalCache,
    session: &RiotSession,
    headers: &HeaderMap,
    raw: &RawPlayerLoadout,
) -> Result<RawPlayerLoadout, MyValError> {
    let updated = put_raw_loadout(http, limiter, session, headers, raw).await?;
    save_cached_loadout(cache, &session.puuid, &updated);
    Ok(updated)
}

pub async fn fetch_or_build_player_loadout(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    cache: &LocalCache,
    session: &RiotSession,
    headers: &HeaderMap,
) -> Result<RawPlayerLoadout, MyValError> {
    match fetch_raw_loadout(http, limiter, session, headers).await {
        Ok(raw) if !raw.guns.is_empty() => {
            save_cached_loadout(cache, &session.puuid, &raw);
            return Ok(raw);
        }
        Ok(_) | Err(MyValError::Api { status: 404, .. }) | Err(MyValError::NotInSession) => {}
        Err(e) => return Err(e),
    }

    // Never fabricate a fake inventory loadout; return the last real server-synced loadout or NotInSession.
    load_cached_loadout(cache, &session.puuid).ok_or(MyValError::NotInSession)
}

pub fn fetch_or_build_offline_loadout(
    cache: &LocalCache,
    puuid: &str,
) -> Result<RawPlayerLoadout, MyValError> {
    load_cached_loadout(cache, puuid).ok_or(MyValError::NotInSession)
}

const CANONICAL_SPRAY_SLOTS: [(&str, &str); 4] = [
    ("0814b2fe-4512-60a4-5288-1fbdcec6ca48", "Pre-Round"),
    ("04af080a-4071-487b-61c0-5b9c0cfaac74", "Mid-Round"),
    ("5863985e-43ac-b05d-cb2d-139e72970014", "Post-Round"),
    ("7cdc908e-4f69-9140-a604-899bd879eed1", "Extra"),
];

fn spray_slot_meta(slot_uuid: &str, fallback_idx: usize) -> (usize, String) {
    let clean = slot_uuid.trim().to_lowercase();
    for (idx, (id, label)) in CANONICAL_SPRAY_SLOTS.iter().enumerate() {
        if clean == *id {
            return (idx, (*label).to_string());
        }
    }
    (10 + fallback_idx, format!("Slot {}", fallback_idx + 1))
}

fn push_slot_asset(
    map: &mut HashMap<String, (usize, String, String)>,
    next_order: &mut usize,
    slot_uuid: &str,
    asset_uuid: &str,
) {
    let slot_clean = slot_uuid.trim();
    if slot_clean.is_empty() {
        return;
    }
    let key = slot_clean.to_lowercase();
    let asset_clean = asset_uuid.trim().to_string();
    if let Some((_, _, existing_asset)) = map.get_mut(&key) {
        if existing_asset.is_empty() && !asset_clean.is_empty() {
            *existing_asset = asset_clean;
        }
    } else {
        let ord = *next_order;
        *next_order += 1;
        map.insert(key, (ord, slot_clean.to_string(), asset_clean));
    }
}

pub fn enrich_loadout(cache: &LocalCache, raw: &RawPlayerLoadout) -> PlayerLoadoutSnapshot {
    let card = cache.lookup_player_card(&raw.identity.player_card_id);
    let title = cache.lookup_title(&raw.identity.player_title_id);
    let live_guns = cache.get_live_match_guns(&raw.subject);
    let is_live_match_synced = live_guns
        .as_ref()
        .map(|lg| !lg.guns.is_empty())
        .unwrap_or(false);

    let mut guns: Vec<LoadoutGunSlot> = raw
        .guns
        .iter()
        .map(|g| {
            let weapon = cache.lookup_weapon(&g.id);
            let base_skin = cache
                .lookup_skin_by_any_uuid(&g.skin_id)
                .ok()
                .flatten()
                .or_else(|| cache.lookup_skin_by_any_uuid(&g.skin_level_id).ok().flatten());
            let mut skin = base_skin.clone();
            let mut effective_chroma_id = g.chroma_id.clone();
            let mut effective_level_id = g.skin_level_id.clone();
            let mut is_live_match_skin = false;
            let mut next_match_skin_name = None;

            if let Some((live_skin_id, live_level_id, live_chroma_id)) = live_guns
                .as_ref()
                .and_then(|lg| lg.guns.get(&g.id.trim().to_lowercase()))
            {
                let live_skin = cache
                    .lookup_skin_by_any_uuid(live_skin_id)
                    .ok()
                    .flatten()
                    .or_else(|| cache.lookup_skin_by_any_uuid(live_level_id).ok().flatten())
                    .or_else(|| cache.lookup_skin_by_any_uuid(live_chroma_id).ok().flatten());
                if let Some(resolved) = live_skin {
                    if !resolved.display_name.contains("Random Favorite") {
                        if let Some(ref base) = base_skin {
                            if !base.uuid.eq_ignore_ascii_case(&resolved.uuid)
                                && !base.display_name.contains("Random Favorite")
                            {
                                next_match_skin_name = Some(base.display_name.clone());
                            }
                        }
                        skin = Some(resolved);
                        is_live_match_skin = true;
                        if !live_chroma_id.trim().is_empty() {
                            effective_chroma_id = live_chroma_id.clone();
                        }
                        if !live_level_id.trim().is_empty() {
                            effective_level_id = live_level_id.clone();
                        }
                    }
                }
            }

            let buddy = g
                .charm_id
                .as_deref()
                .and_then(|id| cache.lookup_buddy(id))
                .or_else(|| g.charm_level_id.as_deref().and_then(|id| cache.lookup_buddy(id)));

            let chroma_pos = if !effective_chroma_id.trim().is_empty() {
                skin.as_ref().and_then(|s| {
                    s.chromas
                        .iter()
                        .position(|c| c.uuid.eq_ignore_ascii_case(&effective_chroma_id))
                })
            } else {
                None
            };
            let level_icon = if !effective_level_id.trim().is_empty() {
                skin.as_ref().and_then(|s| {
                    s.levels
                        .iter()
                        .find(|l| l.uuid.eq_ignore_ascii_case(&effective_level_id))
                        .and_then(|l| l.display_icon.clone())
                        .filter(|u| !u.trim().is_empty())
                })
            } else {
                None
            };
            let chroma_variant_icon = chroma_pos.and_then(|idx| {
                if idx > 0 {
                    skin.as_ref()
                        .and_then(|s| s.chromas.get(idx))
                        .and_then(|c| c.display_icon.clone())
                        .filter(|u| !u.trim().is_empty())
                } else {
                    None
                }
            });
            let base_chroma_icon = chroma_pos.and_then(|idx| {
                if idx == 0 {
                    skin.as_ref()
                        .and_then(|s| s.chromas.first())
                        .and_then(|c| c.display_icon.clone())
                        .filter(|u| !u.trim().is_empty())
                } else {
                    None
                }
            });

            let skin_icon = chroma_variant_icon
                .or(level_icon)
                .or(base_chroma_icon)
                .or_else(|| {
                    skin.as_ref()
                        .and_then(|s| s.display_icon.clone())
                        .filter(|u| !u.trim().is_empty())
                })
                .or_else(|| weapon.as_ref().and_then(|w| w.display_icon.clone()));

            LoadoutGunSlot {
                weapon_uuid: g.id.clone(),
                weapon_name: weapon
                    .as_ref()
                    .map(|w| w.display_name.clone())
                    .or_else(|| skin.as_ref().map(|s| s.weapon_name.clone()))
                    .unwrap_or_else(|| "Weapon".to_string()),
                category: weapon
                    .as_ref()
                    .map(|w| w.category.clone())
                    .unwrap_or_else(|| "Rifle".to_string()),
                skin_uuid: skin
                    .as_ref()
                    .map(|s| s.uuid.clone())
                    .unwrap_or_else(|| g.skin_id.clone()),
                skin_name: skin
                    .as_ref()
                    .map(|s| s.display_name.clone())
                    .unwrap_or_else(|| "Standard".to_string()),
                skin_icon,
                tier_color: skin
                    .as_ref()
                    .map(|s| s.content_tier_color.clone())
                    .unwrap_or_else(|| "#5A9FE2".to_string()),
                chroma_uuid: effective_chroma_id,
                level_uuid: effective_level_id,
                buddy_uuid: g.charm_id.clone(),
                buddy_name: buddy.as_ref().map(|b| b.display_name.clone()),
                buddy_icon: buddy.and_then(|b| b.display_icon),
                is_live_match_skin,
                next_match_skin_name,
            }
        })
        .collect();

    let order = |name: &str| -> u8 {
        match name {
            "Vandal" => 0,
            "Phantom" => 1,
            "Melee" => 2,
            "Operator" => 3,
            "Sheriff" => 4,
            "Ghost" => 5,
            "Bandit" => 6,
            "Classic" => 7,
            "Spectre" => 8,
            "Guardian" => 9,
            "Marshal" => 10,
            "Warden" => 11,
            "Outlaw" => 12,
            "Bulldog" => 13,
            "Judge" => 14,
            "Bucky" => 15,
            "Stinger" => 16,
            "Frenzy" => 17,
            "Shorty" => 18,
            "Odin" => 19,
            "Ares" => 20,
            _ => 25,
        }
    };
    guns.sort_by(|a, b| {
        order(&a.weapon_name)
            .cmp(&order(&b.weapon_name))
            .then_with(|| a.weapon_name.cmp(&b.weapon_name))
    });

    let sprays: Vec<LoadoutSpraySlot> = if raw.api_version == 3 || !raw.active_expressions.is_empty() {
        let mut entries: Vec<(usize, usize, LoadoutSpraySlot)> = Vec::new();
        let mut spray_ord = 0usize;
        let mut flex_ord = 0usize;
        for (idx, expr) in raw.active_expressions.iter().enumerate() {
            let asset_uuid = expr.asset_id.trim().to_string();
            let type_is_flex = expr.type_id.eq_ignore_ascii_case(ITEM_TYPE_FLEX);
            let flex_hit = (!asset_uuid.is_empty())
                .then(|| cache.lookup_flex(&asset_uuid))
                .flatten();
            let spray_hit = (!asset_uuid.is_empty())
                .then(|| cache.lookup_spray(&asset_uuid))
                .flatten();

            if type_is_flex || (spray_hit.is_none() && flex_hit.is_some()) {
                let (name, icon) = if let Some(fx) = flex_hit {
                    let n = if fx.display_name.trim().is_empty() {
                        "Flex".to_string()
                    } else {
                        fx.display_name
                    };
                    (n, fx.display_icon)
                } else {
                    ("Flex".to_string(), None)
                };
                entries.push((
                    20 + flex_ord,
                    idx,
                    LoadoutSpraySlot {
                        slot_uuid: format!("expr:{idx}"),
                        slot_label: if flex_ord == 0 {
                            "Flex".to_string()
                        } else {
                            format!("Flex {}", flex_ord + 1)
                        },
                        spray_uuid: asset_uuid,
                        spray_name: name,
                        spray_icon: icon,
                        is_flex: true,
                    },
                ));
                flex_ord += 1;
            } else {
                let label = match spray_ord {
                    0 => "Pre-Round".to_string(),
                    1 => "Mid-Round".to_string(),
                    2 => "Post-Round".to_string(),
                    3 => "Extra".to_string(),
                    n => format!("Slot {}", n + 1),
                };
                let (name, icon) = if let Some(sp) = spray_hit {
                    let n = if sp.display_name.trim().is_empty() {
                        "Spray".to_string()
                    } else {
                        sp.display_name
                    };
                    (n, sp.full_transparent_icon.or(sp.display_icon))
                } else {
                    ("Empty".to_string(), None)
                };
                entries.push((
                    spray_ord,
                    idx,
                    LoadoutSpraySlot {
                        slot_uuid: format!("expr:{idx}"),
                        slot_label: label,
                        spray_uuid: asset_uuid,
                        spray_name: name,
                        spray_icon: icon,
                        is_flex: false,
                    },
                ));
                spray_ord += 1;
            }
        }
        while spray_ord < 4 {
            let label = match spray_ord {
                0 => "Pre-Round",
                1 => "Mid-Round",
                2 => "Post-Round",
                _ => "Extra",
            }
            .to_string();
            entries.push((
                spray_ord,
                100 + spray_ord,
                LoadoutSpraySlot {
                    slot_uuid: format!("spray:{spray_ord}"),
                    slot_label: label,
                    spray_uuid: String::new(),
                    spray_name: "Empty".to_string(),
                    spray_icon: None,
                    is_flex: false,
                },
            ));
            spray_ord += 1;
        }
        entries.sort_by_key(|(rank, ord, _)| (*rank, *ord));
        entries.into_iter().take(6).map(|(_, _, s)| s).collect()
    } else {
        let mut slot_assets: HashMap<String, (usize, String, String)> = HashMap::new();
        let mut next_order = 0usize;
        if let Some(ref lg) = live_guns {
            for (slot_uuid, asset_uuid) in &lg.sprays {
                push_slot_asset(&mut slot_assets, &mut next_order, slot_uuid, asset_uuid);
            }
        }
        for s in &raw.sprays {
            push_slot_asset(&mut slot_assets, &mut next_order, &s.equip_slot_id, &s.spray_id);
        }
        if let Some(selections) = raw
            .extra
            .get("Expressions")
            .and_then(|e| e.get("AESSelections"))
            .and_then(|a| a.as_array())
        {
            for sel in selections {
                let slot_uuid = sel
                    .get("SocketID")
                    .or_else(|| sel.get("EquipSlotID"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let asset_uuid = sel
                    .get("AssetID")
                    .or_else(|| sel.get("SprayID"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                push_slot_asset(&mut slot_assets, &mut next_order, slot_uuid, asset_uuid);
            }
        }
        for (slot_uuid, _) in &CANONICAL_SPRAY_SLOTS {
            if !slot_assets.contains_key(*slot_uuid) && slot_assets.len() < 4 {
                push_slot_asset(&mut slot_assets, &mut next_order, slot_uuid, "");
            }
        }

        let mut ordered_sprays: Vec<(usize, usize, LoadoutSpraySlot)> = slot_assets
            .into_values()
            .map(|(orig_ord, slot_uuid, asset_uuid)| {
                let (rank, label) = spray_slot_meta(&slot_uuid, orig_ord);
                let spray_opt = if !asset_uuid.is_empty() {
                    cache.lookup_spray(&asset_uuid)
                } else {
                    None
                };
                let (spray_name, spray_icon, is_flex) = if let Some(sp) = spray_opt {
                    let name = if sp.display_name.trim().is_empty() {
                        "Spray".to_string()
                    } else {
                        sp.display_name
                    };
                    (name, sp.full_transparent_icon.or(sp.display_icon), false)
                } else if let Some(fx) = (!asset_uuid.is_empty())
                    .then(|| cache.lookup_flex(&asset_uuid))
                    .flatten()
                {
                    let name = if fx.display_name.trim().is_empty() {
                        "Flex".to_string()
                    } else {
                        fx.display_name
                    };
                    (name, fx.display_icon, true)
                } else {
                    ("Empty".to_string(), None, false)
                };

                (
                    rank,
                    orig_ord,
                    LoadoutSpraySlot {
                        slot_uuid,
                        slot_label: label,
                        spray_uuid: asset_uuid,
                        spray_name,
                        spray_icon,
                        is_flex,
                    },
                )
            })
            .collect();
        ordered_sprays.sort_by_key(|(rank, orig_ord, _)| (*rank, *orig_ord));
        ordered_sprays.into_iter().take(5).map(|(_, _, s)| s).collect()
    };

    PlayerLoadoutSnapshot {
        puuid: raw.subject.clone(),
        version: raw.version,
        player_card_uuid: raw.identity.player_card_id.clone(),
        player_card_name: card
            .as_ref()
            .map(|c| c.display_name.clone())
            .unwrap_or_else(|| "VALORANT Card".to_string()),
        player_card_art: card.and_then(|c| c.wide_art.or(c.large_art).or(c.small_art)),
        player_title_uuid: raw.identity.player_title_id.clone(),
        player_title_text: title
            .map(|t| {
                if !t.title_text.is_empty() {
                    t.title_text
                } else {
                    t.display_name
                }
            })
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "No Title".to_string()),
        incognito: raw.incognito,
        hide_account_level: raw.identity.hide_account_level,
        is_live_match_synced,
        guns,
        sprays,
    }
}

pub fn resolve_best_owned_level_uuid(skin: &Skin, owned_level_uuids: &HashSet<String>) -> String {
    if !owned_level_uuids.is_empty() {
        if let Some(best) = skin
            .levels
            .iter()
            .rev()
            .find(|l| owned_level_uuids.contains(&l.uuid.to_lowercase()))
        {
            return best.uuid.clone();
        }
    }
    skin.levels
        .first()
        .map(|l| l.uuid.clone())
        .unwrap_or_else(|| skin.uuid.clone())
}

pub fn apply_skin_to_raw_loadout(
    cache: &LocalCache,
    raw: &mut RawPlayerLoadout,
    weapon_uuid: &str,
    skin_uuid: &str,
    owned_skin_uuids: &HashSet<String>,
    owned_level_uuids: &HashSet<String>,
    desired_chroma_uuid: Option<&str>,
    owned_chroma_uuids: &HashSet<String>,
    desired_level_uuid: Option<&str>,
) -> Result<(), MyValError> {
    let skin: Skin = cache
        .lookup_skin_by_any_uuid(skin_uuid)?
        .ok_or_else(|| MyValError::InvalidInput(format!("Skin not found: {skin_uuid}")))?;

    if !skin.weapon_uuid.is_empty() && !skin.weapon_uuid.eq_ignore_ascii_case(weapon_uuid) {
        return Err(MyValError::InvalidInput(format!(
            "'{}' cannot be equipped on this weapon slot",
            skin.display_name
        )));
    }

    let is_standard = skin.display_name.starts_with("Standard ");
    let already_equipped = raw.guns.iter().any(|g| {
        g.id.eq_ignore_ascii_case(weapon_uuid) && g.skin_id.eq_ignore_ascii_case(&skin.uuid)
    });
    if !is_standard
        && !already_equipped
        && !owned_skin_uuids.is_empty()
        && !owned_skin_uuids.contains(&skin.uuid.to_lowercase())
    {
        return Err(MyValError::InvalidInput(format!(
            "You do not own '{}'",
            skin.display_name
        )));
    }

    let requested_level = desired_level_uuid
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let level_uuid = if let Some(req_lvl) = requested_level {
        let (idx, lvl) = skin
            .levels
            .iter()
            .enumerate()
            .find(|(_, l)| l.uuid.eq_ignore_ascii_case(req_lvl))
            .ok_or_else(|| {
                MyValError::InvalidInput(format!(
                    "Upgrade level not found for '{}'",
                    skin.display_name
                ))
            })?;
        if idx > 0
            && !is_standard
            && !owned_level_uuids.is_empty()
            && !owned_level_uuids.contains(&lvl.uuid.to_lowercase())
        {
            return Err(MyValError::InvalidInput(
                "You do not own this upgrade level".to_string(),
            ));
        }
        lvl.uuid.clone()
    } else {
        resolve_best_owned_level_uuid(&skin, owned_level_uuids)
    };

    let requested_chroma = desired_chroma_uuid
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let existing_slot = raw
        .guns
        .iter()
        .find(|g| g.id.eq_ignore_ascii_case(weapon_uuid));

    let chosen_chroma_uuid = if let Some(req) = requested_chroma {
        let (idx, chroma) = skin
            .chromas
            .iter()
            .enumerate()
            .find(|(_, c)| c.uuid.eq_ignore_ascii_case(req))
            .ok_or_else(|| {
                MyValError::InvalidInput(format!(
                    "Variant not found for '{}'",
                    skin.display_name
                ))
            })?;
        let already_equipped_chroma = existing_slot
            .map(|g| g.chroma_id.eq_ignore_ascii_case(&chroma.uuid))
            .unwrap_or(false);
        if idx > 0
            && !is_standard
            && !already_equipped_chroma
            && !owned_chroma_uuids.is_empty()
            && !owned_chroma_uuids.contains(&chroma.uuid.to_lowercase())
        {
            return Err(MyValError::InvalidInput(
                "You do not own this variant/chroma".to_string(),
            ));
        }
        chroma.uuid.clone()
    } else if let Some(slot) = existing_slot {
        let keep_existing_chroma = !slot.chroma_id.trim().is_empty()
            && slot.skin_id.eq_ignore_ascii_case(&skin.uuid)
            && skin
                .chromas
                .iter()
                .any(|c| c.uuid.eq_ignore_ascii_case(&slot.chroma_id));
        if keep_existing_chroma {
            slot.chroma_id.clone()
        } else {
            skin.chromas
                .first()
                .map(|c| c.uuid.clone())
                .unwrap_or_else(|| skin.uuid.clone())
        }
    } else {
        skin.chromas
            .first()
            .map(|c| c.uuid.clone())
            .unwrap_or_else(|| skin.uuid.clone())
    };

    if let Some(slot) = raw
        .guns
        .iter_mut()
        .find(|g| g.id.eq_ignore_ascii_case(weapon_uuid))
    {
        slot.skin_id = skin.uuid;
        slot.skin_level_id = level_uuid;
        slot.chroma_id = chosen_chroma_uuid;
    } else {
        raw.guns.push(RawLoadoutGun {
            id: weapon_uuid.to_string(),
            charm_instance_id: None,
            charm_id: None,
            charm_level_id: None,
            skin_id: skin.uuid,
            skin_level_id: level_uuid,
            chroma_id: chosen_chroma_uuid,
            attachments: Vec::new(),
            extra: HashMap::new(),
        });
    }
    Ok(())
}

pub fn apply_spray_to_raw_loadout(
    raw: &mut RawPlayerLoadout,
    slot_uuid: &str,
    spray_uuid: &str,
    owned_spray_uuids: &HashSet<String>,
) -> Result<(), MyValError> {
    let clean_slot = slot_uuid.trim();
    if clean_slot.is_empty() {
        return Err(MyValError::InvalidInput("Spray slot cannot be empty".to_string()));
    }
    let clean_spray = spray_uuid.trim();

    if raw.api_version == 3 || !raw.active_expressions.is_empty() {
        let already_equipped = if let Some(idx_str) = clean_slot.strip_prefix("expr:") {
            idx_str
                .parse::<usize>()
                .ok()
                .and_then(|i| raw.active_expressions.get(i))
                .map(|e| e.asset_id.eq_ignore_ascii_case(clean_spray))
                .unwrap_or(false)
        } else {
            let spray_ord = clean_slot
                .strip_prefix("spray:")
                .and_then(|s| s.parse::<usize>().ok())
                .or_else(|| {
                    CANONICAL_SPRAY_SLOTS
                        .iter()
                        .position(|(id, _)| clean_slot.eq_ignore_ascii_case(id))
                })
                .unwrap_or(0);
            raw.active_expressions
                .iter()
                .filter(|e| !e.type_id.eq_ignore_ascii_case(ITEM_TYPE_FLEX))
                .nth(spray_ord)
                .map(|e| e.asset_id.eq_ignore_ascii_case(clean_spray))
                .unwrap_or(false)
        };
        if !clean_spray.is_empty()
            && !already_equipped
            && !owned_spray_uuids.is_empty()
            && !owned_spray_uuids.contains(&clean_spray.to_lowercase())
        {
            return Err(MyValError::InvalidInput("You do not own this spray".to_string()));
        }

        if let Some(idx_str) = clean_slot.strip_prefix("expr:") {
            let idx = idx_str
                .parse::<usize>()
                .map_err(|_| MyValError::InvalidInput("Invalid expression slot".to_string()))?;
            let expr = raw
                .active_expressions
                .get_mut(idx)
                .ok_or_else(|| MyValError::InvalidInput("Expression slot not found".to_string()))?;
            if expr.type_id.eq_ignore_ascii_case(ITEM_TYPE_FLEX) {
                return Err(MyValError::InvalidInput("Cannot equip spray in Flex slot".to_string()));
            }
            if expr.type_id.is_empty() {
                expr.type_id = ITEM_TYPE_SPRAY.to_string();
            }
            expr.asset_id = clean_spray.to_string();
        } else {
            let spray_ord = clean_slot
                .strip_prefix("spray:")
                .and_then(|s| s.parse::<usize>().ok())
                .or_else(|| {
                    CANONICAL_SPRAY_SLOTS
                        .iter()
                        .position(|(id, _)| clean_slot.eq_ignore_ascii_case(id))
                })
                .unwrap_or(0);
            let target_idx = raw
                .active_expressions
                .iter()
                .enumerate()
                .filter(|(_, e)| !e.type_id.eq_ignore_ascii_case(ITEM_TYPE_FLEX))
                .nth(spray_ord)
                .map(|(i, _)| i);
            if let Some(idx) = target_idx {
                raw.active_expressions[idx].asset_id = clean_spray.to_string();
            } else {
                raw.active_expressions.push(RawLoadoutExpression {
                    type_id: ITEM_TYPE_SPRAY.to_string(),
                    asset_id: clean_spray.to_string(),
                    extra: HashMap::new(),
                });
            }
        }
        return Ok(());
    }

    let already_equipped = raw.sprays.iter().any(|s| {
        s.equip_slot_id.eq_ignore_ascii_case(clean_slot) && s.spray_id.eq_ignore_ascii_case(clean_spray)
    });
    if !clean_spray.is_empty()
        && !already_equipped
        && !owned_spray_uuids.is_empty()
        && !owned_spray_uuids.contains(&clean_spray.to_lowercase())
    {
        return Err(MyValError::InvalidInput("You do not own this spray".to_string()));
    }

    if let Some(slot) = raw
        .sprays
        .iter_mut()
        .find(|s| s.equip_slot_id.eq_ignore_ascii_case(clean_slot))
    {
        slot.spray_id = clean_spray.to_string();
        slot.spray_level_id = None;
    } else {
        raw.sprays.push(RawLoadoutSpray {
            equip_slot_id: clean_slot.to_string(),
            spray_id: clean_spray.to_string(),
            spray_level_id: None,
            extra: HashMap::new(),
        });
    }

    if let Some(selections) = raw
        .extra
        .get_mut("Expressions")
        .and_then(|e| e.get_mut("AESSelections"))
        .and_then(|a| a.as_array_mut())
    {
        for sel in selections.iter_mut() {
            let matches = sel
                .get("SocketID")
                .or_else(|| sel.get("EquipSlotID"))
                .and_then(|v| v.as_str())
                .map(|s| s.eq_ignore_ascii_case(clean_slot))
                .unwrap_or(false);
            if matches {
                if let Some(obj) = sel.as_object_mut() {
                    obj.insert("AssetID".to_string(), serde_json::Value::String(clean_spray.to_string()));
                }
            }
        }
    }
    Ok(())
}

pub fn apply_player_card_to_raw_loadout(
    raw: &mut RawPlayerLoadout,
    card_uuid: &str,
    owned_card_uuids: &HashSet<String>,
) -> Result<(), MyValError> {
    let clean = card_uuid.trim();
    if clean.is_empty() {
        return Err(MyValError::InvalidInput("Player card cannot be empty".to_string()));
    }
    if !raw.identity.player_card_id.eq_ignore_ascii_case(clean)
        && !owned_card_uuids.is_empty()
        && !owned_card_uuids.contains(&clean.to_lowercase())
    {
        return Err(MyValError::InvalidInput("You do not own this player card".to_string()));
    }
    raw.identity.player_card_id = clean.to_string();
    Ok(())
}

pub fn apply_player_title_to_raw_loadout(
    raw: &mut RawPlayerLoadout,
    title_uuid: &str,
    owned_title_uuids: &HashSet<String>,
) -> Result<(), MyValError> {
    let clean = title_uuid.trim();
    if !clean.is_empty()
        && !raw.identity.player_title_id.eq_ignore_ascii_case(clean)
        && !owned_title_uuids.is_empty()
        && !owned_title_uuids.contains(&clean.to_lowercase())
    {
        return Err(MyValError::InvalidInput("You do not own this player title".to_string()));
    }
    raw.identity.player_title_id = clean.to_string();
    Ok(())
}

pub fn apply_identity_privacy_to_raw_loadout(
    raw: &mut RawPlayerLoadout,
    incognito: bool,
    hide_account_level: bool,
) {
    raw.incognito = incognito;
    raw.identity.hide_account_level = hide_account_level;
}

pub fn randomize_raw_loadout(
    cache: &LocalCache,
    raw: &mut RawPlayerLoadout,
    owned_skin_uuids: &HashSet<String>,
    owned_level_uuids: &HashSet<String>,
    seed: u64,
) -> Result<(), MyValError> {
    if owned_skin_uuids.is_empty() {
        return Err(MyValError::InvalidInput(
            "No owned weapon skins found to randomize".to_string(),
        ));
    }
    let all_skins: Vec<Skin> = cache.list_category("skin")?;
    let wishlist = cache.get_wishlist_set()?;
    let mut by_weapon: HashMap<String, Vec<&Skin>> = HashMap::new();
    for s in &all_skins {
        if s.weapon_uuid.is_empty() || s.display_name.contains("Random Favorite") {
            continue;
        }
        let is_owned = owned_skin_uuids.contains(&s.uuid.to_lowercase());
        if !is_owned {
            continue;
        }
        by_weapon
            .entry(s.weapon_uuid.to_lowercase())
            .or_default()
            .push(s);
    }

    if by_weapon.is_empty() {
        return Err(MyValError::InvalidInput(
            "No owned weapon skins matched cached weapon catalog".to_string(),
        ));
    }

    for (idx, gun) in raw.guns.iter_mut().enumerate() {
        let Some(candidates) = by_weapon.get(&gun.id.to_lowercase()) else {
            continue;
        };
        if candidates.is_empty() {
            continue;
        }
        let wish_candidates: Vec<&&Skin> = candidates
            .iter()
            .filter(|s| wishlist.contains(&s.uuid.to_lowercase()))
            .collect();
        let chosen: &Skin = if !wish_candidates.is_empty() {
            let pick = ((seed as usize).wrapping_add(idx * 17)) % wish_candidates.len();
            wish_candidates[pick]
        } else {
            let pick = ((seed as usize).wrapping_add(idx * 31 + 7)) % candidates.len();
            candidates[pick]
        };
        gun.skin_id = chosen.uuid.clone();
        gun.skin_level_id = resolve_best_owned_level_uuid(chosen, owned_level_uuids);
        gun.chroma_id = chosen
            .chromas
            .first()
            .map(|c| c.uuid.clone())
            .unwrap_or_else(|| chosen.uuid.clone());
    }
    Ok(())
}

pub async fn fetch_owned_agent_uuids(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    cache: &LocalCache,
    session: &RiotSession,
    headers: &HeaderMap,
) -> Result<Vec<String>, MyValError> {
    let url = format!(
        "{}/store/v1/entitlements/{}/{}",
        pd_base(&session.shard),
        session.puuid,
        ITEM_TYPE_AGENT
    );
    let ent: RawEntitlementsResp = limiter
        .request_json(|| http.get(&url).headers(headers.clone()))
        .await?;
    let mut set: HashSet<String> = STARTER_AGENT_UUIDS
        .iter()
        .map(|s| (*s).to_lowercase())
        .collect();
    for item in ent.into_items(Some(ITEM_TYPE_AGENT)) {
        let id = item.item_id.trim().to_lowercase();
        if !id.is_empty() {
            set.insert(id);
        }
    }
    let mut list: Vec<String> = set.into_iter().collect();
    list.sort();
    let _ = cache.put_owned_agent_uuids(&session.puuid, &list);
    Ok(list)
}

pub async fn fetch_collection_summary(
    http: &reqwest::Client,
    limiter: &RateLimiter,
    cache: &LocalCache,
    session: &RiotSession,
    headers: &HeaderMap,
    current_store: Option<&StorefrontSnapshot>,
) -> Result<CollectionSummary, MyValError> {
    let ent_url = format!(
        "{}/store/v1/entitlements/{}/{}",
        pd_base(&session.shard),
        session.puuid,
        ITEM_TYPE_SKIN_LEVEL
    );
    let chroma_ent_url = format!(
        "{}/store/v1/entitlements/{}/{}",
        pd_base(&session.shard),
        session.puuid,
        ITEM_TYPE_SKIN_CHROMA
    );
    let spray_ent_url = format!(
        "{}/store/v1/entitlements/{}/{}",
        pd_base(&session.shard),
        session.puuid,
        ITEM_TYPE_SPRAY
    );
    let card_ent_url = format!(
        "{}/store/v1/entitlements/{}/{}",
        pd_base(&session.shard),
        session.puuid,
        ITEM_TYPE_PLAYER_CARD
    );
    let title_ent_url = format!(
        "{}/store/v1/entitlements/{}/{}",
        pd_base(&session.shard),
        session.puuid,
        ITEM_TYPE_TITLE
    );
    let offers_url = format!("{}/store/v1/offers", pd_base(&session.shard));

    let (ent_res, chroma_ent_res, spray_ent_res, card_ent_res, title_ent_res, offers_res) = tokio::join!(
        limiter.request_json::<RawEntitlementsResp, _>(|| http.get(&ent_url).headers(headers.clone())),
        limiter.request_json_opt::<RawEntitlementsResp, _>(|| http.get(&chroma_ent_url).headers(headers.clone())),
        limiter.request_json_opt::<RawEntitlementsResp, _>(|| http.get(&spray_ent_url).headers(headers.clone())),
        limiter.request_json_opt::<RawEntitlementsResp, _>(|| http.get(&card_ent_url).headers(headers.clone())),
        limiter.request_json_opt::<RawEntitlementsResp, _>(|| http.get(&title_ent_url).headers(headers.clone())),
        limiter.request_json_opt::<RawOffersResp, _>(|| http.get(&offers_url).headers(headers.clone())),
    );
    let ent = ent_res?;
    if let Ok(Some(chroma_ent)) = chroma_ent_res {
        let chroma_ids: Vec<String> = chroma_ent
            .into_items(Some(ITEM_TYPE_SKIN_CHROMA))
            .into_iter()
            .map(|e| e.item_id)
            .collect();
        let _ = cache.put_owned_chroma_uuids(&session.puuid, &chroma_ids);
    }
    if let Ok(Some(spray_ent)) = spray_ent_res {
        let ids: Vec<String> = spray_ent
            .into_items(Some(ITEM_TYPE_SPRAY))
            .into_iter()
            .map(|e| e.item_id)
            .collect();
        let _ = cache.put_owned_spray_uuids(&session.puuid, &ids);
    }
    if let Ok(Some(card_ent)) = card_ent_res {
        let ids: Vec<String> = card_ent
            .into_items(Some(ITEM_TYPE_PLAYER_CARD))
            .into_iter()
            .map(|e| e.item_id)
            .collect();
        let _ = cache.put_owned_card_uuids(&session.puuid, &ids);
    }
    if let Ok(Some(title_ent)) = title_ent_res {
        let ids: Vec<String> = title_ent
            .into_items(Some(ITEM_TYPE_TITLE))
            .into_iter()
            .map(|e| e.item_id)
            .collect();
        let _ = cache.put_owned_title_uuids(&session.puuid, &ids);
    }
    let price_by_level: HashMap<String, i64> = offers_res
        .ok()
        .flatten()
        .map(|o| {
            o.offers
                .into_iter()
                .filter_map(|offer| {
                    let vp = *offer.cost.get(CURRENCY_VP)?;
                    Some((offer.offer_id.to_lowercase(), vp))
                })
                .collect()
        })
        .unwrap_or_default();

    let level_ids: Vec<String> = ent
        .into_items(Some(ITEM_TYPE_SKIN_LEVEL))
        .into_iter()
        .map(|e| e.item_id)
        .collect();
    let _ = cache.put_owned_skin_level_uuids(&session.puuid, &level_ids);
    let (summary, owned_skin_uuids) =
        compute_collection_summary_from_levels(cache, &level_ids, &price_by_level, current_store)?;
    let _ = cache.put_owned_skin_uuids(&session.puuid, &owned_skin_uuids);
    Ok(summary)
}

pub fn compute_collection_summary_from_levels(
    cache: &LocalCache,
    owned_level_ids: &[String],
    price_by_level: &HashMap<String, i64>,
    current_store: Option<&StorefrontSnapshot>,
) -> Result<(CollectionSummary, Vec<String>), MyValError> {
    let mut seen_skins: HashSet<String> = HashSet::new();
    let mut owned_skin_uuids: Vec<String> = Vec::new();
    let mut total_vp = 0i64;
    let mut tier_counts: HashMap<String, (String, u32)> = HashMap::new();

    for level_id in owned_level_ids {
        if let Ok(Some(skin)) = cache.lookup_skin_by_any_uuid(level_id) {
            let key = skin.uuid.to_lowercase();
            if !seen_skins.insert(key.clone()) {
                continue;
            }
            owned_skin_uuids.push(key);
            let level1 = skin
                .levels
                .first()
                .map(|l| l.uuid.to_lowercase())
                .unwrap_or_else(|| level_id.to_lowercase());
            let tier_name = skin
                .content_tier_name
                .clone()
                .unwrap_or_else(|| "Deluxe".to_string());
            let fallback_vp = match tier_name.as_str() {
                "Select" | "Select Edition" => 875,
                "Deluxe" | "Deluxe Edition" => 1275,
                "Premium" | "Premium Edition" => 1775,
                "Exclusive" | "Exclusive Edition" => 2175,
                "Ultra" | "Ultra Edition" => 2475,
                _ => 1275,
            };
            let vp = price_by_level
                .get(&level1)
                .copied()
                .or_else(|| price_by_level.get(&level_id.to_lowercase()).copied())
                .unwrap_or(fallback_vp);
            total_vp += vp;

            let entry = tier_counts
                .entry(tier_name)
                .or_insert_with(|| (skin.content_tier_color.clone(), 0));
            entry.1 += 1;
        }
    }

    let mut rarity_breakdown: Vec<CollectionRarityBreakdown> = tier_counts
        .into_iter()
        .map(|(tier_name, (tier_color, count))| CollectionRarityBreakdown {
            tier_name,
            tier_color,
            count,
        })
        .collect();
    rarity_breakdown.sort_by(|a, b| b.count.cmp(&a.count));

    let wishlisted_in_store_names = extract_wishlisted_store_hits(current_store);

    Ok((
        CollectionSummary {
            owned_skin_count: seen_skins.len() as u32,
            estimated_total_vp: total_vp,
            estimated_usd: ((total_vp as f64) * 0.0099 * 100.0).round() / 100.0,
            wishlisted_in_store_names,
            rarity_breakdown,
        },
        owned_skin_uuids,
    ))
}

pub fn extract_wishlisted_store_hits(store: Option<&StorefrontSnapshot>) -> Vec<String> {
    let Some(s) = store else {
        return Vec::new();
    };
    let mut hits = Vec::new();
    for d in &s.daily_offers {
        if d.is_wishlisted {
            hits.push(format!("{} (Daily Shop)", d.skin_name));
        }
    }
    if let Some(nm) = &s.night_market {
        for o in &nm.offers {
            if o.is_wishlisted {
                hits.push(format!("{} (Night Market -{}%)", o.skin_name, o.discount_percent));
            }
        }
    }
    hits
}
