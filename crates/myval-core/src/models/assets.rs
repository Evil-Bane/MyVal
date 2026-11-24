use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct VersionInfo {
    pub riot_client_version: String,
    pub version: String,
    pub build_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AgentAbility {
    pub slot: String,
    pub display_name: String,
    pub description: String,
    pub display_icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Agent {
    pub uuid: String,
    pub display_name: String,
    pub description: String,
    pub role_name: Option<String>,
    pub role_icon: Option<String>,
    pub display_icon: Option<String>,
    pub full_portrait: Option<String>,
    pub background_gradient_colors: Vec<String>,
    pub abilities: Vec<AgentAbility>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MapInfo {
    pub uuid: String,
    pub display_name: String,
    pub map_url: String,
    pub coordinates: Option<String>,
    pub splash: Option<String>,
    pub display_icon: Option<String>,
    pub list_view_icon: Option<String>,
    pub x_multiplier: f64,
    pub y_multiplier: f64,
    pub x_scalar_to_add: f64,
    pub y_scalar_to_add: f64,
    pub callout_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Weapon {
    pub uuid: String,
    pub display_name: String,
    pub category: String,
    pub display_icon: Option<String>,
    pub kill_stream_icon: Option<String>,
    pub cost: i32,
    pub fire_rate: f64,
    pub magazine_size: i32,
}

impl Weapon {
    /// Returns the horizontal kill-feed silhouette URL, deriving it from `display_icon`
    /// for older synced asset rows that predate `kill_stream_icon`.
    pub fn kill_stream_icon_url(&self) -> Option<String> {
        self.kill_stream_icon.clone().or_else(|| {
            self.display_icon
                .as_deref()
                .map(|u| u.replace("/displayicon.png", "/killstreamicon.png"))
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SkinChroma {
    pub uuid: String,
    pub display_name: String,
    pub display_icon: Option<String>,
    pub swatch: Option<String>,
    pub streamed_video: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SkinLevel {
    pub uuid: String,
    pub display_name: String,
    pub display_icon: Option<String>,
    pub streamed_video: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Skin {
    pub uuid: String,
    pub display_name: String,
    pub weapon_uuid: String,
    pub weapon_name: String,
    pub content_tier_uuid: Option<String>,
    pub content_tier_name: Option<String>,
    pub content_tier_color: String,
    pub display_icon: Option<String>,
    pub wallpaper: Option<String>,
    pub chromas: Vec<SkinChroma>,
    pub levels: Vec<SkinLevel>,
    pub is_wishlisted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ContentTier {
    pub uuid: String,
    pub display_name: String,
    pub dev_name: String,
    pub rank: i32,
    pub highlight_color: String,
    pub display_icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CompetitiveTier {
    pub tier: i32,
    pub tier_name: String,
    pub division_name: String,
    pub color: String,
    pub background_color: String,
    pub large_icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BundleAsset {
    pub uuid: String,
    pub display_name: String,
    pub description: String,
    pub display_icon: Option<String>,
    pub vertical_promo_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PlayerCard {
    pub uuid: String,
    pub display_name: String,
    pub small_art: Option<String>,
    pub wide_art: Option<String>,
    pub large_art: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Spray {
    pub uuid: String,
    pub display_name: String,
    pub display_icon: Option<String>,
    pub full_transparent_icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Buddy {
    pub uuid: String,
    pub display_name: String,
    pub display_icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PlayerTitle {
    pub uuid: String,
    pub display_name: String,
    pub title_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SeasonInfo {
    pub uuid: String,
    pub display_name: String,
    pub title: Option<String>,
    pub season_type: Option<String>,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct FlexAsset {
    pub uuid: String,
    pub display_name: String,
    pub display_icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MissionAsset {
    pub uuid: String,
    pub title: String,
    pub progress_to_complete: i32,
    pub xp_grant: i32,
    pub mission_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AssetSummary {
    pub riot_client_version: String,
    pub synced_at_epoch_secs: i64,
    pub agent_count: u32,
    pub map_count: u32,
    pub weapon_count: u32,
    pub skin_count: u32,
    pub bundle_count: u32,
    pub competitive_tier_count: u32,
}
