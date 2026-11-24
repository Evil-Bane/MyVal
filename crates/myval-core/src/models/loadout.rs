use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LoadoutGunSlot {
    pub weapon_uuid: String,
    pub weapon_name: String,
    pub category: String,
    pub skin_uuid: String,
    pub skin_name: String,
    pub skin_icon: Option<String>,
    pub tier_color: String,
    pub chroma_uuid: String,
    pub level_uuid: String,
    pub buddy_uuid: Option<String>,
    pub buddy_name: Option<String>,
    pub buddy_icon: Option<String>,
    pub is_live_match_skin: bool,
    pub next_match_skin_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LoadoutSpraySlot {
    pub slot_uuid: String,
    pub slot_label: String,
    pub spray_uuid: String,
    pub spray_name: String,
    pub spray_icon: Option<String>,
    pub is_flex: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PlayerLoadoutSnapshot {
    pub puuid: String,
    pub version: i32,
    pub player_card_uuid: String,
    pub player_card_name: String,
    pub player_card_art: Option<String>,
    pub player_title_uuid: String,
    pub player_title_text: String,
    pub incognito: bool,
    pub hide_account_level: bool,
    pub is_live_match_synced: bool,
    pub guns: Vec<LoadoutGunSlot>,
    pub sprays: Vec<LoadoutSpraySlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LoadoutPreset {
    pub preset_id: String,
    pub name: String,
    pub accent_color_hex: String,
    pub created_at_epoch_secs: i64,
    pub weapon_count: u32,
    pub preview_skins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CollectionRarityBreakdown {
    pub tier_name: String,
    pub tier_color: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CollectionSummary {
    pub owned_skin_count: u32,
    pub estimated_total_vp: i64,
    pub estimated_usd: f64,
    pub wishlisted_in_store_names: Vec<String>,
    pub rarity_breakdown: Vec<CollectionRarityBreakdown>,
}
