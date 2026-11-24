use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::cache::LocalCache;
use crate::error::MyValError;
use crate::models::assets::{
    Agent, AgentAbility, AssetSummary, Buddy, BundleAsset, CompetitiveTier, ContentTier, FlexAsset,
    MapInfo, MissionAsset, PlayerCard, PlayerTitle, SeasonInfo, Skin, SkinChroma, SkinLevel,
    Spray, Weapon,
};

const BASE_URL: &str = "https://valorant-api.com/v1";

#[derive(Debug, Deserialize)]
struct ApiEnvelope<T> {
    #[serde(default)]
    status: u16,
    data: T,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawVersion {
    riot_client_version: String,
    version: String,
    build_date: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawAgent {
    uuid: String,
    display_name: String,
    description: String,
    display_icon: Option<String>,
    full_portrait: Option<String>,
    background_gradient_colors: Vec<String>,
    role: Option<RawRole>,
    abilities: Vec<RawAbility>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawRole {
    display_name: String,
    display_icon: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawAbility {
    slot: String,
    display_name: String,
    description: String,
    display_icon: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawMap {
    uuid: String,
    display_name: String,
    map_url: String,
    coordinates: Option<String>,
    splash: Option<String>,
    display_icon: Option<String>,
    list_view_icon: Option<String>,
    x_multiplier: f64,
    y_multiplier: f64,
    x_scalar_to_add: f64,
    y_scalar_to_add: f64,
    callouts: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawWeapon {
    uuid: String,
    display_name: String,
    category: String,
    display_icon: Option<String>,
    kill_stream_icon: Option<String>,
    shop_data: Option<RawShopData>,
    weapon_stats: Option<RawWeaponStats>,
    skins: Vec<RawWeaponSkinRef>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawShopData {
    cost: i32,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawWeaponStats {
    fire_rate: f64,
    magazine_size: i32,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawWeaponSkinRef {
    uuid: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawSkin {
    uuid: String,
    display_name: String,
    content_tier_uuid: Option<String>,
    display_icon: Option<String>,
    wallpaper: Option<String>,
    chromas: Vec<RawSkinChroma>,
    levels: Vec<RawSkinLevel>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawSkinChroma {
    uuid: String,
    display_name: String,
    display_icon: Option<String>,
    full_render: Option<String>,
    swatch: Option<String>,
    streamed_video: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawSkinLevel {
    uuid: String,
    display_name: String,
    display_icon: Option<String>,
    streamed_video: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawContentTier {
    uuid: String,
    display_name: String,
    dev_name: String,
    rank: i32,
    highlight_color: String,
    display_icon: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawCompetitiveTierSet {
    tiers: Vec<RawCompetitiveTier>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawCompetitiveTier {
    tier: i32,
    tier_name: String,
    division_name: String,
    color: String,
    background_color: String,
    large_icon: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawBundle {
    uuid: String,
    display_name: String,
    description: String,
    display_icon: Option<String>,
    vertical_promo_image: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawPlayerCard {
    uuid: String,
    display_name: String,
    small_art: Option<String>,
    wide_art: Option<String>,
    large_art: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawSpray {
    uuid: String,
    display_name: String,
    display_icon: Option<String>,
    full_transparent_icon: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawBuddy {
    uuid: String,
    display_name: String,
    display_icon: Option<String>,
    levels: Vec<RawBuddyLevel>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawBuddyLevel {
    uuid: String,
    display_icon: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawPlayerTitle {
    uuid: String,
    display_name: Option<String>,
    title_text: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawSeason {
    uuid: String,
    display_name: String,
    title: Option<String>,
    #[serde(rename = "type")]
    season_type: Option<String>,
    start_time: String,
    end_time: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawFlex {
    uuid: String,
    display_name: String,
    display_icon: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawMission {
    uuid: String,
    title: Option<String>,
    progress_to_complete: i32,
    xp_grant: i32,
    #[serde(rename = "type")]
    mission_type: Option<String>,
}

pub fn normalize_hex_color(raw: &str, fallback: &str) -> String {
    let trimmed = raw.trim().trim_start_matches('#');
    if trimmed.len() >= 6 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        format!("#{}", &trimmed[..6].to_uppercase())
    } else {
        fallback.to_string()
    }
}

async fn fetch_json<T: DeserializeOwned>(
    http: &reqwest::Client,
    path: &str,
) -> Result<T, MyValError> {
    let url = format!("{BASE_URL}/{path}");
    let resp = http.get(&url).send().await?;
    let status = resp.status();
    if !status.is_success() {
        return Err(MyValError::Api {
            status: status.as_u16(),
            message: format!("valorant-api error on {path}"),
        });
    }
    let env: ApiEnvelope<T> = resp.json().await?;
    if env.status != 200 {
        return Err(MyValError::Api {
            status: env.status,
            message: format!("valorant-api envelope status {} on {path}", env.status),
        });
    }
    Ok(env.data)
}

/// Syncs public static Valorant assets + live clientVersion from valorant-api.com into SQLite.
pub async fn sync_all_assets(
    http: &reqwest::Client,
    cache: &LocalCache,
) -> Result<AssetSummary, MyValError> {
    let (
        version_res,
        agents_res,
        maps_res,
        weapons_res,
        skins_res,
        tiers_res,
        comp_tiers_res,
        bundles_res,
        cards_res,
        sprays_res,
        buddies_res,
        titles_res,
        seasons_res,
        flex_res,
        missions_res,
    ) = tokio::join!(
        fetch_json::<RawVersion>(http, "version"),
        fetch_json::<Vec<RawAgent>>(http, "agents?isPlayableCharacter=true"),
        fetch_json::<Vec<RawMap>>(http, "maps"),
        fetch_json::<Vec<RawWeapon>>(http, "weapons"),
        fetch_json::<Vec<RawSkin>>(http, "weapons/skins"),
        fetch_json::<Vec<RawContentTier>>(http, "contenttiers"),
        fetch_json::<Vec<RawCompetitiveTierSet>>(http, "competitivetiers"),
        fetch_json::<Vec<RawBundle>>(http, "bundles"),
        fetch_json::<Vec<RawPlayerCard>>(http, "playercards"),
        fetch_json::<Vec<RawSpray>>(http, "sprays"),
        fetch_json::<Vec<RawBuddy>>(http, "buddies"),
        fetch_json::<Vec<RawPlayerTitle>>(http, "playertitles"),
        fetch_json::<Vec<RawSeason>>(http, "seasons"),
        fetch_json::<Vec<RawFlex>>(http, "flex"),
        fetch_json::<Vec<RawMission>>(http, "missions"),
    );

    let version = version_res?;
    if !version.riot_client_version.is_empty() {
        cache.set_meta("riot_client_version", &version.riot_client_version)?;
    }

    let agents: Vec<Agent> = agents_res?
        .into_iter()
        .map(|a| Agent {
            uuid: a.uuid,
            display_name: a.display_name,
            description: a.description,
            role_name: a.role.as_ref().map(|r| r.display_name.clone()),
            role_icon: a.role.and_then(|r| r.display_icon),
            display_icon: a.display_icon,
            full_portrait: a.full_portrait,
            background_gradient_colors: a.background_gradient_colors,
            abilities: a
                .abilities
                .into_iter()
                .map(|ab| AgentAbility {
                    slot: ab.slot,
                    display_name: ab.display_name,
                    description: ab.description,
                    display_icon: ab.display_icon,
                })
                .collect(),
        })
        .collect();
    cache.replace_category("agent", &agents, |a| a.uuid.clone())?;

    let maps: Vec<MapInfo> = maps_res?
        .into_iter()
        .map(|m| MapInfo {
            uuid: m.uuid,
            display_name: m.display_name,
            map_url: m.map_url,
            coordinates: m.coordinates,
            splash: m.splash,
            display_icon: m.display_icon,
            list_view_icon: m.list_view_icon,
            x_multiplier: m.x_multiplier,
            y_multiplier: m.y_multiplier,
            x_scalar_to_add: m.x_scalar_to_add,
            y_scalar_to_add: m.y_scalar_to_add,
            callout_count: m.callouts.map(|c| c.len() as u32).unwrap_or(0),
        })
        .collect();
    cache.replace_category("map", &maps, |m| m.uuid.clone())?;

    let raw_weapons = weapons_res?;
    let mut skin_to_weapon: HashMap<String, (String, String)> = HashMap::new();
    let weapons: Vec<Weapon> = raw_weapons
        .into_iter()
        .map(|w| {
            for s in &w.skins {
                skin_to_weapon.insert(
                    s.uuid.to_lowercase(),
                    (w.uuid.clone(), w.display_name.clone()),
                );
            }
            Weapon {
                uuid: w.uuid,
                display_name: w.display_name,
                category: w
                    .category
                    .strip_prefix("EEquippableCategory::")
                    .unwrap_or(&w.category)
                    .to_string(),
                display_icon: w.display_icon,
                kill_stream_icon: w.kill_stream_icon,
                cost: w.shop_data.map(|s| s.cost).unwrap_or(0),
                fire_rate: w.weapon_stats.as_ref().map(|s| s.fire_rate).unwrap_or(0.0),
                magazine_size: w.weapon_stats.map(|s| s.magazine_size).unwrap_or(0),
            }
        })
        .collect();
    cache.replace_category("weapon", &weapons, |w| w.uuid.clone())?;

    let content_tiers: Vec<ContentTier> = tiers_res?
        .into_iter()
        .map(|t| ContentTier {
            uuid: t.uuid,
            display_name: t.display_name,
            dev_name: t.dev_name,
            rank: t.rank,
            highlight_color: normalize_hex_color(&t.highlight_color, "#5A9FE2"),
            display_icon: t.display_icon,
        })
        .collect();
    let tier_lookup: HashMap<String, ContentTier> = content_tiers
        .iter()
        .map(|t| (t.uuid.to_lowercase(), t.clone()))
        .collect();
    cache.replace_category("contenttier", &content_tiers, |t| t.uuid.clone())?;

    let skins: Vec<Skin> = skins_res?
        .into_iter()
        .map(|s| {
            let (weapon_uuid, weapon_name) = skin_to_weapon
                .get(&s.uuid.to_lowercase())
                .cloned()
                .unwrap_or_else(|| (String::new(), "Weapon".to_string()));
            let tier_opt = s
                .content_tier_uuid
                .as_deref()
                .and_then(|id| tier_lookup.get(&id.to_lowercase()));
            let fallback_icon = s
                .display_icon
                .clone()
                .or_else(|| s.chromas.first().and_then(|c| c.full_render.clone().or(c.display_icon.clone())))
                .or_else(|| s.levels.first().and_then(|l| l.display_icon.clone()));

            Skin {
                uuid: s.uuid,
                display_name: s.display_name,
                weapon_uuid,
                weapon_name,
                content_tier_uuid: s.content_tier_uuid,
                content_tier_name: tier_opt.map(|t| t.display_name.clone()),
                content_tier_color: tier_opt
                    .map(|t| t.highlight_color.clone())
                    .unwrap_or_else(|| "#5A9FE2".to_string()),
                display_icon: fallback_icon,
                wallpaper: s.wallpaper,
                chromas: s
                    .chromas
                    .into_iter()
                    .map(|c| SkinChroma {
                        uuid: c.uuid,
                        display_name: c.display_name,
                        display_icon: c.full_render.or(c.display_icon),
                        swatch: c.swatch,
                        streamed_video: c.streamed_video,
                    })
                    .collect(),
                levels: s
                    .levels
                    .into_iter()
                    .map(|l| SkinLevel {
                        uuid: l.uuid,
                        display_name: l.display_name,
                        display_icon: l.display_icon,
                        streamed_video: l.streamed_video,
                    })
                    .collect(),
                is_wishlisted: false,
            }
        })
        .collect();
    cache.replace_category("skin", &skins, |s| s.uuid.clone())?;
    cache.rebuild_skin_level_index(&skins)?;

    let comp_tiers: Vec<CompetitiveTier> = comp_tiers_res?
        .into_iter()
        .last()
        .map(|set| {
            set.tiers
                .into_iter()
                .map(|t| CompetitiveTier {
                    tier: t.tier,
                    tier_name: t.tier_name,
                    division_name: t.division_name,
                    color: normalize_hex_color(&t.color, "#9CA3AF"),
                    background_color: normalize_hex_color(&t.background_color, "#1C2633"),
                    large_icon: t.large_icon,
                })
                .collect()
        })
        .unwrap_or_default();
    cache.replace_category("competitivetier", &comp_tiers, |t| t.tier.to_string())?;

    let bundles: Vec<BundleAsset> = bundles_res?
        .into_iter()
        .map(|b| BundleAsset {
            uuid: b.uuid,
            display_name: b.display_name,
            description: b.description,
            display_icon: b.display_icon,
            vertical_promo_image: b.vertical_promo_image,
        })
        .collect();
    cache.replace_category("bundle", &bundles, |b| b.uuid.clone())?;

    let cards: Vec<PlayerCard> = cards_res?
        .into_iter()
        .map(|c| PlayerCard {
            uuid: c.uuid,
            display_name: c.display_name,
            small_art: c.small_art,
            wide_art: c.wide_art,
            large_art: c.large_art,
        })
        .collect();
    cache.replace_category("playercard", &cards, |c| c.uuid.clone())?;

    let sprays: Vec<Spray> = sprays_res?
        .into_iter()
        .map(|s| Spray {
            uuid: s.uuid,
            display_name: s.display_name,
            display_icon: s.full_transparent_icon.or(s.display_icon),
            full_transparent_icon: None,
        })
        .collect();
    cache.replace_category("spray", &sprays, |s| s.uuid.clone())?;

    let mut buddies: Vec<Buddy> = Vec::new();
    for b in buddies_res? {
        let icon = b.display_icon.clone();
        buddies.push(Buddy {
            uuid: b.uuid.clone(),
            display_name: b.display_name.clone(),
            display_icon: icon.clone(),
        });
        for lvl in b.levels {
            if !lvl.uuid.is_empty() && !lvl.uuid.eq_ignore_ascii_case(&b.uuid) {
                buddies.push(Buddy {
                    uuid: lvl.uuid,
                    display_name: b.display_name.clone(),
                    display_icon: lvl.display_icon.or_else(|| icon.clone()),
                });
            }
        }
    }
    cache.replace_category("buddy", &buddies, |b| b.uuid.clone())?;

    let titles: Vec<PlayerTitle> = titles_res?
        .into_iter()
        .map(|t| PlayerTitle {
            uuid: t.uuid,
            display_name: t.display_name.unwrap_or_default(),
            title_text: t.title_text.unwrap_or_default(),
        })
        .collect();
    cache.replace_category("title", &titles, |t| t.uuid.clone())?;

    if let Ok(raw_seasons) = seasons_res {
        let mut seasons: Vec<SeasonInfo> = raw_seasons
            .into_iter()
            .filter(|s| s.season_type.as_deref() == Some("EAresSeasonType::Act"))
            .map(|s| SeasonInfo {
                uuid: s.uuid,
                display_name: s.display_name,
                title: s.title,
                season_type: s.season_type,
                start_time: s.start_time,
                end_time: s.end_time,
            })
            .collect();
        seasons.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        cache.replace_category("season", &seasons, |s| s.uuid.clone())?;
    }

    if let Ok(raw_flex) = flex_res {
        let flex_items: Vec<FlexAsset> = raw_flex
            .into_iter()
            .map(|f| FlexAsset {
                uuid: f.uuid,
                display_name: f.display_name,
                display_icon: f.display_icon,
            })
            .collect();
        let _ = cache.replace_category("flex", &flex_items, |f| f.uuid.clone());
    }

    if let Ok(raw_missions) = missions_res {
        let missions: Vec<MissionAsset> = raw_missions
            .into_iter()
            .filter_map(|m| {
                let title = m.title?.trim().to_string();
                if title.is_empty() {
                    return None;
                }
                Some(MissionAsset {
                    uuid: m.uuid,
                    title,
                    progress_to_complete: m.progress_to_complete,
                    xp_grant: m.xp_grant,
                    mission_type: m.mission_type.unwrap_or_default(),
                })
            })
            .collect();
        let _ = cache.replace_category("mission", &missions, |m| m.uuid.clone());
    }

    let _ = cache.clear_mmr_and_career_cache();
    cache.set_meta("assets_synced_at", &LocalCache::now_secs().to_string())?;
    cache.set_meta("assets_schema_version", "4")?;
    cache.get_asset_summary()
}
