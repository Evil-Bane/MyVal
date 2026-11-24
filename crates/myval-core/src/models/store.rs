use serde::{Deserialize, Serialize};

use super::assets::SkinChroma;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Wallet {
    pub vp: i64,
    pub radianite: i64,
    pub kingdom_credits: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DailyOffer {
    pub offer_id: String,
    pub skin_uuid: String,
    pub skin_name: String,
    pub weapon_name: String,
    pub display_icon: Option<String>,
    pub tier_name: Option<String>,
    pub tier_color: String,
    pub cost_vp: i64,
    pub preview_video_url: Option<String>,
    pub chromas: Vec<SkinChroma>,
    pub is_wishlisted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BundleItem {
    pub item_type: String,
    pub item_uuid: String,
    pub name: String,
    pub display_icon: Option<String>,
    pub base_price_vp: i64,
    pub discounted_price_vp: i64,
    pub discount_percent: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct FeaturedBundle {
    pub bundle_uuid: String,
    pub name: String,
    pub description: String,
    pub display_icon: Option<String>,
    pub remaining_secs: i64,
    pub total_base_price_vp: i64,
    pub total_discounted_price_vp: i64,
    pub items: Vec<BundleItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct NightMarketOffer {
    pub offer_id: String,
    pub skin_uuid: String,
    pub skin_name: String,
    pub weapon_name: String,
    pub display_icon: Option<String>,
    pub tier_color: String,
    pub base_price_vp: i64,
    pub discount_percent: i32,
    pub discount_price_vp: i64,
    pub is_seen: bool,
    pub is_wishlisted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct NightMarketSection {
    pub remaining_secs: i64,
    pub offers: Vec<NightMarketOffer>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AccessoryOffer {
    pub offer_id: String,
    pub item_type: String,
    pub item_uuid: String,
    pub name: String,
    pub display_icon: Option<String>,
    pub cost_kc: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct StorefrontSnapshot {
    pub fetched_at_epoch_secs: i64,
    pub daily_remaining_secs: i64,
    pub daily_offers: Vec<DailyOffer>,
    pub featured_bundles: Vec<FeaturedBundle>,
    pub night_market: Option<NightMarketSection>,
    pub accessory_remaining_secs: i64,
    pub accessory_offers: Vec<AccessoryOffer>,
    pub wallet: Wallet,
}
