use std::future::Future;
use std::sync::Arc;

use myval_core::models as cm;
use myval_core::{MyValEngineCore, MyValError};

uniffi::setup_scaffolding!();

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FfiError {
    #[error("Network error: {detail}")]
    Network { detail: String },
    #[error("Unauthorized: {detail}")]
    Unauthorized { detail: String },
    #[error("Rate limited ({retry_after_secs}s)")]
    RateLimited { retry_after_secs: u64 },
    #[error("Not in an active Valorant session")]
    NotInSession,
    #[error("API {status}: {detail}")]
    Api { status: u16, detail: String },
    #[error("Parse error: {detail}")]
    Parse { detail: String },
    #[error("Storage error: {detail}")]
    Storage { detail: String },
    #[error("Invalid input: {detail}")]
    InvalidInput { detail: String },
}

impl From<MyValError> for FfiError {
    fn from(err: MyValError) -> Self {
        match err {
            MyValError::Network(detail) => Self::Network { detail },
            MyValError::Unauthorized(detail) => Self::Unauthorized { detail },
            MyValError::RateLimited { retry_after_secs } => Self::RateLimited { retry_after_secs },
            MyValError::NotInSession => Self::NotInSession,
            MyValError::Api { status, message } => Self::Api { status, detail: message },
            MyValError::Parse(detail) => Self::Parse { detail },
            MyValError::Storage(detail) => Self::Storage { detail },
            MyValError::InvalidInput(detail) => Self::InvalidInput { detail },
        }
    }
}

// ---- UniFFI Exported Records & Enums ----

#[derive(Debug, Clone, uniffi::Record)]
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

impl From<cm::assets::AssetSummary> for AssetSummary {
    fn from(v: cm::assets::AssetSummary) -> Self {
        Self {
            riot_client_version: v.riot_client_version,
            synced_at_epoch_secs: v.synced_at_epoch_secs,
            agent_count: v.agent_count,
            map_count: v.map_count,
            weapon_count: v.weapon_count,
            skin_count: v.skin_count,
            bundle_count: v.bundle_count,
            competitive_tier_count: v.competitive_tier_count,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct AgentAbility {
    pub slot: String,
    pub display_name: String,
    pub description: String,
    pub display_icon: Option<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
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

impl From<cm::assets::Agent> for Agent {
    fn from(a: cm::assets::Agent) -> Self {
        Self {
            uuid: a.uuid,
            display_name: a.display_name,
            description: a.description,
            role_name: a.role_name,
            role_icon: a.role_icon,
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
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct MapInfo {
    pub uuid: String,
    pub display_name: String,
    pub map_url: String,
    pub coordinates: Option<String>,
    pub splash: Option<String>,
    pub display_icon: Option<String>,
    pub list_view_icon: Option<String>,
    pub callout_count: u32,
}

impl From<cm::assets::MapInfo> for MapInfo {
    fn from(m: cm::assets::MapInfo) -> Self {
        Self {
            uuid: m.uuid,
            display_name: m.display_name,
            map_url: m.map_url,
            coordinates: m.coordinates,
            splash: m.splash,
            display_icon: m.display_icon,
            list_view_icon: m.list_view_icon,
            callout_count: m.callout_count,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
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

impl From<cm::assets::Weapon> for Weapon {
    fn from(w: cm::assets::Weapon) -> Self {
        Self {
            uuid: w.uuid,
            display_name: w.display_name,
            category: w.category,
            display_icon: w.display_icon,
            kill_stream_icon: w.kill_stream_icon,
            cost: w.cost,
            fire_rate: w.fire_rate,
            magazine_size: w.magazine_size,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SkinChroma {
    pub uuid: String,
    pub display_name: String,
    pub display_icon: Option<String>,
    pub swatch: Option<String>,
    pub streamed_video: Option<String>,
}

impl From<cm::assets::SkinChroma> for SkinChroma {
    fn from(c: cm::assets::SkinChroma) -> Self {
        Self {
            uuid: c.uuid,
            display_name: c.display_name,
            display_icon: c.display_icon,
            swatch: c.swatch,
            streamed_video: c.streamed_video,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SkinLevel {
    pub uuid: String,
    pub display_name: String,
    pub display_icon: Option<String>,
    pub streamed_video: Option<String>,
}

impl From<cm::assets::SkinLevel> for SkinLevel {
    fn from(l: cm::assets::SkinLevel) -> Self {
        Self {
            uuid: l.uuid,
            display_name: l.display_name,
            display_icon: l.display_icon,
            streamed_video: l.streamed_video,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct Skin {
    pub uuid: String,
    pub display_name: String,
    pub weapon_uuid: String,
    pub weapon_name: String,
    pub content_tier_name: Option<String>,
    pub content_tier_color: String,
    pub display_icon: Option<String>,
    pub preview_video_url: Option<String>,
    pub chromas: Vec<SkinChroma>,
    pub levels: Vec<SkinLevel>,
    pub is_wishlisted: bool,
}

impl From<cm::assets::Skin> for Skin {
    fn from(s: cm::assets::Skin) -> Self {
        let video = s
            .levels
            .iter()
            .rev()
            .find_map(|l| l.streamed_video.clone())
            .or_else(|| s.chromas.iter().find_map(|c| c.streamed_video.clone()));
        Self {
            uuid: s.uuid,
            display_name: s.display_name,
            weapon_uuid: s.weapon_uuid,
            weapon_name: s.weapon_name,
            content_tier_name: s.content_tier_name,
            content_tier_color: s.content_tier_color,
            display_icon: s.display_icon,
            preview_video_url: video,
            chromas: s.chromas.into_iter().map(Into::into).collect(),
            levels: s.levels.into_iter().map(Into::into).collect(),
            is_wishlisted: s.is_wishlisted,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct PlayerCard {
    pub uuid: String,
    pub display_name: String,
    pub small_art: Option<String>,
    pub wide_art: Option<String>,
    pub large_art: Option<String>,
}

impl From<cm::assets::PlayerCard> for PlayerCard {
    fn from(c: cm::assets::PlayerCard) -> Self {
        Self {
            uuid: c.uuid,
            display_name: c.display_name,
            small_art: c.small_art,
            wide_art: c.wide_art,
            large_art: c.large_art,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct PlayerTitle {
    pub uuid: String,
    pub display_name: String,
    pub title_text: String,
}

impl From<cm::assets::PlayerTitle> for PlayerTitle {
    fn from(t: cm::assets::PlayerTitle) -> Self {
        Self {
            uuid: t.uuid,
            display_name: t.display_name,
            title_text: t.title_text,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct Spray {
    pub uuid: String,
    pub display_name: String,
    pub display_icon: Option<String>,
    pub full_transparent_icon: Option<String>,
}

impl From<cm::assets::Spray> for Spray {
    fn from(s: cm::assets::Spray) -> Self {
        Self {
            uuid: s.uuid,
            display_name: s.display_name,
            display_icon: s.display_icon,
            full_transparent_icon: s.full_transparent_icon,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SessionSummary {
    pub puuid: String,
    pub riot_id: String,
    pub region: String,
    pub shard: String,
    pub expires_at_epoch_secs: i64,
}

impl From<cm::auth::RiotSession> for SessionSummary {
    fn from(s: cm::auth::RiotSession) -> Self {
        Self {
            riot_id: s.riot_id(),
            puuid: s.puuid,
            region: s.region,
            shard: s.shard,
            expires_at_epoch_secs: s.expires_at_epoch_secs,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct Wallet {
    pub vp: i64,
    pub radianite: i64,
    pub kingdom_credits: i64,
}

impl From<cm::store::Wallet> for Wallet {
    fn from(w: cm::store::Wallet) -> Self {
        Self {
            vp: w.vp,
            radianite: w.radianite,
            kingdom_credits: w.kingdom_credits,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
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

impl From<cm::store::DailyOffer> for DailyOffer {
    fn from(d: cm::store::DailyOffer) -> Self {
        Self {
            offer_id: d.offer_id,
            skin_uuid: d.skin_uuid,
            skin_name: d.skin_name,
            weapon_name: d.weapon_name,
            display_icon: d.display_icon,
            tier_name: d.tier_name,
            tier_color: d.tier_color,
            cost_vp: d.cost_vp,
            preview_video_url: d.preview_video_url,
            chromas: d.chromas.into_iter().map(Into::into).collect(),
            is_wishlisted: d.is_wishlisted,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct BundleItem {
    pub item_type: String,
    pub item_uuid: String,
    pub name: String,
    pub display_icon: Option<String>,
    pub base_price_vp: i64,
    pub discounted_price_vp: i64,
    pub discount_percent: i32,
}

impl From<cm::store::BundleItem> for BundleItem {
    fn from(b: cm::store::BundleItem) -> Self {
        Self {
            item_type: b.item_type,
            item_uuid: b.item_uuid,
            name: b.name,
            display_icon: b.display_icon,
            base_price_vp: b.base_price_vp,
            discounted_price_vp: b.discounted_price_vp,
            discount_percent: b.discount_percent,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
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

impl From<cm::store::FeaturedBundle> for FeaturedBundle {
    fn from(b: cm::store::FeaturedBundle) -> Self {
        Self {
            bundle_uuid: b.bundle_uuid,
            name: b.name,
            description: b.description,
            display_icon: b.display_icon,
            remaining_secs: b.remaining_secs,
            total_base_price_vp: b.total_base_price_vp,
            total_discounted_price_vp: b.total_discounted_price_vp,
            items: b.items.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
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

impl From<cm::store::NightMarketOffer> for NightMarketOffer {
    fn from(n: cm::store::NightMarketOffer) -> Self {
        Self {
            offer_id: n.offer_id,
            skin_uuid: n.skin_uuid,
            skin_name: n.skin_name,
            weapon_name: n.weapon_name,
            display_icon: n.display_icon,
            tier_color: n.tier_color,
            base_price_vp: n.base_price_vp,
            discount_percent: n.discount_percent,
            discount_price_vp: n.discount_price_vp,
            is_seen: n.is_seen,
            is_wishlisted: n.is_wishlisted,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct NightMarketSection {
    pub remaining_secs: i64,
    pub offers: Vec<NightMarketOffer>,
}

impl From<cm::store::NightMarketSection> for NightMarketSection {
    fn from(s: cm::store::NightMarketSection) -> Self {
        Self {
            remaining_secs: s.remaining_secs,
            offers: s.offers.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct AccessoryOffer {
    pub offer_id: String,
    pub item_type: String,
    pub item_uuid: String,
    pub name: String,
    pub display_icon: Option<String>,
    pub cost_kc: i64,
}

impl From<cm::store::AccessoryOffer> for AccessoryOffer {
    fn from(a: cm::store::AccessoryOffer) -> Self {
        Self {
            offer_id: a.offer_id,
            item_type: a.item_type,
            item_uuid: a.item_uuid,
            name: a.name,
            display_icon: a.display_icon,
            cost_kc: a.cost_kc,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
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

impl From<cm::store::StorefrontSnapshot> for StorefrontSnapshot {
    fn from(s: cm::store::StorefrontSnapshot) -> Self {
        Self {
            fetched_at_epoch_secs: s.fetched_at_epoch_secs,
            daily_remaining_secs: s.daily_remaining_secs,
            daily_offers: s.daily_offers.into_iter().map(Into::into).collect(),
            featured_bundles: s.featured_bundles.into_iter().map(Into::into).collect(),
            night_market: s.night_market.map(Into::into),
            accessory_remaining_secs: s.accessory_remaining_secs,
            accessory_offers: s.accessory_offers.into_iter().map(Into::into).collect(),
            wallet: s.wallet.into(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct RankInfo {
    pub tier: i32,
    pub tier_name: String,
    pub division_name: String,
    pub rr: i32,
    pub icon_url: Option<String>,
    pub color_hex: String,
}

impl From<cm::mmr::RankInfo> for RankInfo {
    fn from(r: cm::mmr::RankInfo) -> Self {
        Self {
            tier: r.tier,
            tier_name: r.tier_name,
            division_name: r.division_name,
            rr: r.rr,
            icon_url: r.icon_url,
            color_hex: r.color_hex,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct PartyMemberInfo {
    pub puuid: String,
    pub display_name: String,
    pub is_ready: bool,
    pub is_owner: bool,
    pub is_me: bool,
    pub account_level: Option<i32>,
    pub player_card_icon: Option<String>,
    pub platform: String,
    pub current_rank: RankInfo,
}

impl From<cm::live::PartyMemberInfo> for PartyMemberInfo {
    fn from(m: cm::live::PartyMemberInfo) -> Self {
        Self {
            puuid: m.puuid,
            display_name: m.display_name,
            is_ready: m.is_ready,
            is_owner: m.is_owner,
            is_me: m.is_me,
            account_level: m.account_level,
            player_card_icon: m.player_card_icon,
            platform: m.platform,
            current_rank: m.current_rank.into(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct PartySnapshot {
    pub party_id: String,
    pub state: String,
    pub accessibility: String,
    pub queue_id: String,
    pub invite_code: Option<String>,
    pub is_in_queue: bool,
    pub is_match_starting: bool,
    pub queue_elapsed_secs: i64,
    pub error_message: Option<String>,
    pub members: Vec<PartyMemberInfo>,
    pub available_queues: Vec<String>,
    pub eligible_queues: Vec<String>,
    pub can_change_queue: bool,
    pub change_queue_disabled_reason: Option<String>,
    pub can_start_queue: bool,
    pub can_leave_queue: bool,
    pub queue_action_disabled_reason: Option<String>,
    pub can_toggle_ready: bool,
    pub can_toggle_accessibility: bool,
    pub can_generate_invite_code: bool,
    pub can_disable_invite_code: bool,
    pub can_join_by_code: bool,
}

impl From<cm::live::PartySnapshot> for PartySnapshot {
    fn from(p: cm::live::PartySnapshot) -> Self {
        Self {
            party_id: p.party_id,
            state: p.state,
            accessibility: p.accessibility,
            queue_id: p.queue_id,
            invite_code: p.invite_code,
            is_in_queue: p.is_in_queue,
            is_match_starting: p.is_match_starting,
            queue_elapsed_secs: p.queue_elapsed_secs,
            error_message: p.error_message,
            members: p.members.into_iter().map(Into::into).collect(),
            available_queues: p.available_queues,
            eligible_queues: p.eligible_queues,
            can_change_queue: p.can_change_queue,
            change_queue_disabled_reason: p.change_queue_disabled_reason,
            can_start_queue: p.can_start_queue,
            can_leave_queue: p.can_leave_queue,
            queue_action_disabled_reason: p.queue_action_disabled_reason,
            can_toggle_ready: p.can_toggle_ready,
            can_toggle_accessibility: p.can_toggle_accessibility,
            can_generate_invite_code: p.can_generate_invite_code,
            can_disable_invite_code: p.can_disable_invite_code,
            can_join_by_code: p.can_join_by_code,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct PreGameAlly {
    pub puuid: String,
    pub display_name: String,
    pub agent_uuid: String,
    pub agent_name: String,
    pub agent_icon: Option<String>,
    pub selection_state: String,
    pub account_level: Option<i32>,
    pub hide_account_level: bool,
    pub incognito: bool,
    pub is_me: bool,
    pub current_rank: RankInfo,
    pub peak_rank: RankInfo,
    pub recent_outcomes: Vec<i8>,
    pub party_group_id: Option<String>,
    pub party_group_label: Option<String>,
    pub party_group_size: u32,
    pub equipped_skins: Vec<EquippedWeaponSkin>,
}

impl From<cm::live::PreGameAlly> for PreGameAlly {
    fn from(a: cm::live::PreGameAlly) -> Self {
        Self {
            puuid: a.puuid,
            display_name: a.display_name,
            agent_uuid: a.agent_uuid,
            agent_name: a.agent_name,
            agent_icon: a.agent_icon,
            selection_state: a.selection_state,
            account_level: a.account_level,
            hide_account_level: a.hide_account_level,
            incognito: a.incognito,
            is_me: a.is_me,
            current_rank: a.current_rank.into(),
            peak_rank: a.peak_rank.into(),
            recent_outcomes: a.recent_outcomes,
            party_group_id: a.party_group_id,
            party_group_label: a.party_group_label,
            party_group_size: a.party_group_size,
            equipped_skins: a.equipped_skins.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct PreGameSnapshot {
    pub match_id: String,
    pub map_name: String,
    pub map_splash: Option<String>,
    pub queue_id: String,
    pub pregame_state: String,
    pub ally_team_id: String,
    pub ally_side: String,
    pub phase_time_remaining_secs: i64,
    pub enemy_team_size: i32,
    pub enemy_team_lock_count: i32,
    pub can_select_agent: bool,
    pub can_quit_match: bool,
    pub locked_agent_uuids: Vec<String>,
    pub my_agent_uuid: Option<String>,
    pub my_selection_state: String,
    pub allies: Vec<PreGameAlly>,
    pub detected_stacks: Vec<PartyGroup>,
}

impl From<cm::live::PreGameSnapshot> for PreGameSnapshot {
    fn from(p: cm::live::PreGameSnapshot) -> Self {
        Self {
            match_id: p.match_id,
            map_name: p.map_name,
            map_splash: p.map_splash,
            queue_id: p.queue_id,
            pregame_state: p.pregame_state,
            ally_team_id: p.ally_team_id,
            ally_side: p.ally_side,
            phase_time_remaining_secs: p.phase_time_remaining_secs,
            enemy_team_size: p.enemy_team_size,
            enemy_team_lock_count: p.enemy_team_lock_count,
            can_select_agent: p.can_select_agent,
            can_quit_match: p.can_quit_match,
            locked_agent_uuids: p.locked_agent_uuids,
            my_agent_uuid: p.my_agent_uuid,
            my_selection_state: p.my_selection_state,
            allies: p.allies.into_iter().map(Into::into).collect(),
            detected_stacks: p.detected_stacks.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct EquippedWeaponSkin {
    pub weapon_name: String,
    pub skin_name: String,
    pub skin_icon: Option<String>,
    pub tier_name: Option<String>,
    pub tier_color: String,
    pub is_animated: bool,
}

impl From<cm::live::EquippedWeaponSkin> for EquippedWeaponSkin {
    fn from(e: cm::live::EquippedWeaponSkin) -> Self {
        Self {
            weapon_name: e.weapon_name,
            skin_name: e.skin_name,
            skin_icon: e.skin_icon,
            tier_name: e.tier_name,
            tier_color: e.tier_color,
            is_animated: e.is_animated,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ScoutedPlayer {
    pub puuid: String,
    pub display_name: String,
    pub team_id: String,
    pub agent_uuid: String,
    pub agent_name: String,
    pub agent_icon: Option<String>,
    pub player_card_icon: Option<String>,
    pub account_level: Option<i32>,
    pub incognito: bool,
    pub hide_account_level: bool,
    pub is_me: bool,
    pub current_rank: RankInfo,
    pub peak_rank: RankInfo,
    pub recent_results: Vec<bool>,
    pub recent_outcomes: Vec<i8>,
    pub party_group_id: Option<String>,
    pub party_group_label: Option<String>,
    pub party_group_size: u32,
    pub equipped_skins: Vec<EquippedWeaponSkin>,
}

impl From<cm::live::ScoutedPlayer> for ScoutedPlayer {
    fn from(s: cm::live::ScoutedPlayer) -> Self {
        Self {
            puuid: s.puuid,
            display_name: s.display_name,
            team_id: s.team_id,
            agent_uuid: s.agent_uuid,
            agent_name: s.agent_name,
            agent_icon: s.agent_icon,
            player_card_icon: s.player_card_icon,
            account_level: s.account_level,
            incognito: s.incognito,
            hide_account_level: s.hide_account_level,
            is_me: s.is_me,
            current_rank: s.current_rank.into(),
            peak_rank: s.peak_rank.into(),
            recent_results: s.recent_results,
            recent_outcomes: s.recent_outcomes,
            party_group_id: s.party_group_id,
            party_group_label: s.party_group_label,
            party_group_size: s.party_group_size,
            equipped_skins: s.equipped_skins.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct PartyGroup {
    pub group_id: String,
    pub label: String,
    pub team_id: String,
    pub member_puuids: Vec<String>,
    pub confidence_shared_matches: u32,
}

impl From<cm::live::PartyGroup> for PartyGroup {
    fn from(g: cm::live::PartyGroup) -> Self {
        Self {
            group_id: g.group_id,
            label: g.label,
            team_id: g.team_id,
            member_puuids: g.member_puuids,
            confidence_shared_matches: g.confidence_shared_matches,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct CoreGameSnapshot {
    pub match_id: String,
    pub map_name: String,
    pub map_splash: Option<String>,
    pub queue_id: String,
    pub provisioning_flow: String,
    pub server_name: String,
    pub my_starting_side: String,
    pub average_lobby_rank_name: String,
    pub my_team: Vec<ScoutedPlayer>,
    pub enemy_team: Vec<ScoutedPlayer>,
    pub detected_stacks: Vec<PartyGroup>,
}

impl From<cm::live::CoreGameSnapshot> for CoreGameSnapshot {
    fn from(c: cm::live::CoreGameSnapshot) -> Self {
        Self {
            match_id: c.match_id,
            map_name: c.map_name,
            map_splash: c.map_splash,
            queue_id: c.queue_id,
            provisioning_flow: c.provisioning_flow,
            server_name: c.server_name,
            my_starting_side: c.my_starting_side,
            average_lobby_rank_name: c.average_lobby_rank_name,
            my_team: c.my_team.into_iter().map(Into::into).collect(),
            enemy_team: c.enemy_team.into_iter().map(Into::into).collect(),
            detected_stacks: c.detected_stacks.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum LiveState {
    Idle {
        message: String,
        recommended_poll_secs: u64,
    },
    InParty {
        snapshot: PartySnapshot,
    },
    PreGame {
        snapshot: PreGameSnapshot,
    },
    CoreGame {
        snapshot: CoreGameSnapshot,
    },
}

impl From<cm::live::LiveState> for LiveState {
    fn from(state: cm::live::LiveState) -> Self {
        match state {
            cm::live::LiveState::Idle {
                message,
                recommended_poll_secs,
            } => Self::Idle {
                message,
                recommended_poll_secs,
            },
            cm::live::LiveState::InParty(s) => Self::InParty { snapshot: s.into() },
            cm::live::LiveState::PreGame(s) => Self::PreGame { snapshot: s.into() },
            cm::live::LiveState::CoreGame(s) => Self::CoreGame { snapshot: s.into() },
        }
    }
}

// ---- Loadout, Collection, Career & 2D Replay UniFFI Records ----

#[derive(Debug, Clone, uniffi::Record)]
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

impl From<cm::loadout::LoadoutGunSlot> for LoadoutGunSlot {
    fn from(v: cm::loadout::LoadoutGunSlot) -> Self {
        Self {
            weapon_uuid: v.weapon_uuid,
            weapon_name: v.weapon_name,
            category: v.category,
            skin_uuid: v.skin_uuid,
            skin_name: v.skin_name,
            skin_icon: v.skin_icon,
            tier_color: v.tier_color,
            chroma_uuid: v.chroma_uuid,
            level_uuid: v.level_uuid,
            buddy_uuid: v.buddy_uuid,
            buddy_name: v.buddy_name,
            buddy_icon: v.buddy_icon,
            is_live_match_skin: v.is_live_match_skin,
            next_match_skin_name: v.next_match_skin_name,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct LoadoutSpraySlot {
    pub slot_uuid: String,
    pub slot_label: String,
    pub spray_uuid: String,
    pub spray_name: String,
    pub spray_icon: Option<String>,
    pub is_flex: bool,
}

impl From<cm::loadout::LoadoutSpraySlot> for LoadoutSpraySlot {
    fn from(v: cm::loadout::LoadoutSpraySlot) -> Self {
        Self {
            slot_uuid: v.slot_uuid,
            slot_label: v.slot_label,
            spray_uuid: v.spray_uuid,
            spray_name: v.spray_name,
            spray_icon: v.spray_icon,
            is_flex: v.is_flex,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
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

impl From<cm::loadout::PlayerLoadoutSnapshot> for PlayerLoadoutSnapshot {
    fn from(v: cm::loadout::PlayerLoadoutSnapshot) -> Self {
        Self {
            puuid: v.puuid,
            version: v.version,
            player_card_uuid: v.player_card_uuid,
            player_card_name: v.player_card_name,
            player_card_art: v.player_card_art,
            player_title_uuid: v.player_title_uuid,
            player_title_text: v.player_title_text,
            incognito: v.incognito,
            hide_account_level: v.hide_account_level,
            is_live_match_synced: v.is_live_match_synced,
            guns: v.guns.into_iter().map(Into::into).collect(),
            sprays: v.sprays.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct LoadoutPreset {
    pub preset_id: String,
    pub name: String,
    pub accent_color_hex: String,
    pub created_at_epoch_secs: i64,
    pub weapon_count: u32,
    pub preview_skins: Vec<String>,
}

impl From<cm::loadout::LoadoutPreset> for LoadoutPreset {
    fn from(v: cm::loadout::LoadoutPreset) -> Self {
        Self {
            preset_id: v.preset_id,
            name: v.name,
            accent_color_hex: v.accent_color_hex,
            created_at_epoch_secs: v.created_at_epoch_secs,
            weapon_count: v.weapon_count,
            preview_skins: v.preview_skins,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct CollectionRarityBreakdown {
    pub tier_name: String,
    pub tier_color: String,
    pub count: u32,
}

impl From<cm::loadout::CollectionRarityBreakdown> for CollectionRarityBreakdown {
    fn from(v: cm::loadout::CollectionRarityBreakdown) -> Self {
        Self {
            tier_name: v.tier_name,
            tier_color: v.tier_color,
            count: v.count,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct CollectionSummary {
    pub owned_skin_count: u32,
    pub estimated_total_vp: i64,
    pub estimated_usd: f64,
    pub wishlisted_in_store_names: Vec<String>,
    pub rarity_breakdown: Vec<CollectionRarityBreakdown>,
}

impl From<cm::loadout::CollectionSummary> for CollectionSummary {
    fn from(v: cm::loadout::CollectionSummary) -> Self {
        Self {
            owned_skin_count: v.owned_skin_count,
            estimated_total_vp: v.estimated_total_vp,
            estimated_usd: v.estimated_usd,
            wishlisted_in_store_names: v.wishlisted_in_store_names,
            rarity_breakdown: v.rarity_breakdown.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct CompetitiveMatchUpdate {
    pub match_id: String,
    pub map_name: String,
    pub map_splash: Option<String>,
    pub start_time_epoch_secs: i64,
    pub tier_after: i32,
    pub tier_name: String,
    pub tier_color: String,
    pub rr_after: i32,
    pub rr_delta: i32,
    pub won: bool,
}

impl From<cm::career::CompetitiveMatchUpdate> for CompetitiveMatchUpdate {
    fn from(v: cm::career::CompetitiveMatchUpdate) -> Self {
        Self {
            match_id: v.match_id,
            map_name: v.map_name,
            map_splash: v.map_splash,
            start_time_epoch_secs: v.start_time_epoch_secs,
            tier_after: v.tier_after,
            tier_name: v.tier_name,
            tier_color: v.tier_color,
            rr_after: v.rr_after,
            rr_delta: v.rr_delta,
            won: v.won,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct AgentPerformanceStat {
    pub agent_uuid: String,
    pub agent_name: String,
    pub agent_icon: Option<String>,
    pub matches_played: u32,
    pub wins: u32,
    pub win_rate_percent: f64,
    pub avg_acs: f64,
    pub kd_ratio: f64,
    pub hs_percent: f64,
}

impl From<cm::career::AgentPerformanceStat> for AgentPerformanceStat {
    fn from(v: cm::career::AgentPerformanceStat) -> Self {
        Self {
            agent_uuid: v.agent_uuid,
            agent_name: v.agent_name,
            agent_icon: v.agent_icon,
            matches_played: v.matches_played,
            wins: v.wins,
            win_rate_percent: v.win_rate_percent,
            avg_acs: v.avg_acs,
            kd_ratio: v.kd_ratio,
            hs_percent: v.hs_percent,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ContractProgressSnapshot {
    pub active_contract_name: String,
    pub current_level: u32,
    pub max_level: u32,
    pub xp_in_level: i64,
    pub xp_required_for_next: i64,
    pub total_xp: i64,
}

impl From<cm::career::ContractProgressSnapshot> for ContractProgressSnapshot {
    fn from(v: cm::career::ContractProgressSnapshot) -> Self {
        Self {
            active_contract_name: v.active_contract_name,
            current_level: v.current_level,
            max_level: v.max_level,
            xp_in_level: v.xp_in_level,
            xp_required_for_next: v.xp_required_for_next,
            total_xp: v.total_xp,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct MatchHistoryItem {
    pub match_id: String,
    pub map_name: String,
    pub map_splash: Option<String>,
    pub queue_id: String,
    pub start_time_epoch_secs: i64,
    pub duration_secs: i64,
    pub my_team_rounds: i32,
    pub enemy_team_rounds: i32,
    pub won: bool,
    pub my_agent_name: String,
    pub my_agent_icon: Option<String>,
    pub my_kills: i32,
    pub my_deaths: i32,
    pub my_assists: i32,
    pub my_acs: i32,
    pub my_hs_percent: f64,
    pub my_adr: i32,
    pub rr_delta: Option<i32>,
}

impl From<cm::career::MatchHistoryItem> for MatchHistoryItem {
    fn from(v: cm::career::MatchHistoryItem) -> Self {
        Self {
            match_id: v.match_id,
            map_name: v.map_name,
            map_splash: v.map_splash,
            queue_id: v.queue_id,
            start_time_epoch_secs: v.start_time_epoch_secs,
            duration_secs: v.duration_secs,
            my_team_rounds: v.my_team_rounds,
            enemy_team_rounds: v.enemy_team_rounds,
            won: v.won,
            my_agent_name: v.my_agent_name,
            my_agent_icon: v.my_agent_icon,
            my_kills: v.my_kills,
            my_deaths: v.my_deaths,
            my_assists: v.my_assists,
            my_acs: v.my_acs,
            my_hs_percent: v.my_hs_percent,
            my_adr: v.my_adr,
            rr_delta: v.rr_delta,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct AccountPenalty {
    pub reason: String,
    pub expires_at: Option<String>,
    pub games_remaining: Option<i32>,
}

impl From<cm::career::AccountPenalty> for AccountPenalty {
    fn from(v: cm::career::AccountPenalty) -> Self {
        Self {
            reason: v.reason,
            expires_at: v.expires_at,
            games_remaining: v.games_remaining,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct MissionProgress {
    pub mission_id: String,
    pub title: String,
    pub current_progress: i32,
    pub target_progress: i32,
    pub xp_grant: i32,
    pub completed: bool,
    pub expires_at: Option<String>,
}

impl From<cm::career::MissionProgress> for MissionProgress {
    fn from(v: cm::career::MissionProgress) -> Self {
        Self {
            mission_id: v.mission_id,
            title: v.title,
            current_progress: v.current_progress,
            target_progress: v.target_progress,
            xp_grant: v.xp_grant,
            completed: v.completed,
            expires_at: v.expires_at,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct CareerSnapshot {
    pub puuid: String,
    pub current_rank: RankInfo,
    pub peak_rank: RankInfo,
    pub previous_act_rank: RankInfo,
    pub current_act_name: String,
    pub current_act_end_label: String,
    pub peak_rank_act_name: String,
    pub net_rr_last_10: i32,
    pub win_rate_last_10: f64,
    pub current_streak_label: String,
    pub rr_trend: Vec<CompetitiveMatchUpdate>,
    pub recent_matches: Vec<MatchHistoryItem>,
    pub agent_stats: Vec<AgentPerformanceStat>,
    pub contract: ContractProgressSnapshot,
    pub penalties: Vec<AccountPenalty>,
    pub missions: Vec<MissionProgress>,
}

impl From<cm::career::CareerSnapshot> for CareerSnapshot {
    fn from(v: cm::career::CareerSnapshot) -> Self {
        Self {
            puuid: v.puuid,
            current_rank: v.current_rank.into(),
            peak_rank: v.peak_rank.into(),
            previous_act_rank: v.previous_act_rank.into(),
            current_act_name: v.current_act_name,
            current_act_end_label: v.current_act_end_label,
            peak_rank_act_name: v.peak_rank_act_name,
            net_rr_last_10: v.net_rr_last_10,
            win_rate_last_10: v.win_rate_last_10,
            current_streak_label: v.current_streak_label,
            rr_trend: v.rr_trend.into_iter().map(Into::into).collect(),
            recent_matches: v.recent_matches.into_iter().map(Into::into).collect(),
            agent_stats: v.agent_stats.into_iter().map(Into::into).collect(),
            contract: v.contract.into(),
            penalties: v.penalties.into_iter().map(Into::into).collect(),
            missions: v.missions.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ScoreboardPlayer {
    pub puuid: String,
    pub display_name: String,
    pub team_id: String,
    pub is_me: bool,
    pub agent_name: String,
    pub agent_icon: Option<String>,
    pub rank_name: String,
    pub rank_color: String,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub acs: i32,
    pub adr: i32,
    pub hs_percent: f64,
    pub first_bloods: i32,
}

impl From<cm::career::ScoreboardPlayer> for ScoreboardPlayer {
    fn from(v: cm::career::ScoreboardPlayer) -> Self {
        Self {
            puuid: v.puuid,
            display_name: v.display_name,
            team_id: v.team_id,
            is_me: v.is_me,
            agent_name: v.agent_name,
            agent_icon: v.agent_icon,
            rank_name: v.rank_name,
            rank_color: v.rank_color,
            kills: v.kills,
            deaths: v.deaths,
            assists: v.assists,
            acs: v.acs,
            adr: v.adr,
            hs_percent: v.hs_percent,
            first_bloods: v.first_bloods,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ReplayPlayerSnapshot {
    pub puuid: String,
    pub display_name: String,
    pub team_id: String,
    pub agent_icon: Option<String>,
    pub norm_x: f64,
    pub norm_y: f64,
    pub view_angle_rad: f64,
}

impl From<cm::career::ReplayPlayerSnapshot> for ReplayPlayerSnapshot {
    fn from(v: cm::career::ReplayPlayerSnapshot) -> Self {
        Self {
            puuid: v.puuid,
            display_name: v.display_name,
            team_id: v.team_id,
            agent_icon: v.agent_icon,
            norm_x: v.norm_x,
            norm_y: v.norm_y,
            view_angle_rad: v.view_angle_rad,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct RoundKillEvent {
    pub round_num: u32,
    pub time_in_round_secs: f64,
    pub killer_puuid: String,
    pub killer_name: String,
    pub killer_team: String,
    pub killer_agent_icon: Option<String>,
    pub victim_puuid: String,
    pub victim_name: String,
    pub victim_team: String,
    pub victim_agent_icon: Option<String>,
    pub weapon_name: String,
    pub weapon_icon: Option<String>,
    pub killer_norm_x: f64,
    pub killer_norm_y: f64,
    pub victim_norm_x: f64,
    pub victim_norm_y: f64,
    pub player_positions: Vec<ReplayPlayerSnapshot>,
}

impl From<cm::career::RoundKillEvent> for RoundKillEvent {
    fn from(v: cm::career::RoundKillEvent) -> Self {
        Self {
            round_num: v.round_num,
            time_in_round_secs: v.time_in_round_secs,
            killer_puuid: v.killer_puuid,
            killer_name: v.killer_name,
            killer_team: v.killer_team,
            killer_agent_icon: v.killer_agent_icon,
            victim_puuid: v.victim_puuid,
            victim_name: v.victim_name,
            victim_team: v.victim_team,
            victim_agent_icon: v.victim_agent_icon,
            weapon_name: v.weapon_name,
            weapon_icon: v.weapon_icon,
            killer_norm_x: v.killer_norm_x,
            killer_norm_y: v.killer_norm_y,
            victim_norm_x: v.victim_norm_x,
            victim_norm_y: v.victim_norm_y,
            player_positions: v.player_positions.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct RoundReplayData {
    pub round_num: u32,
    pub winning_team: String,
    pub end_type: String,
    pub plant_time_secs: Option<f64>,
    pub plant_site: Option<String>,
    pub planter_name: Option<String>,
    pub plant_norm_x: Option<f64>,
    pub plant_norm_y: Option<f64>,
    pub plant_player_positions: Vec<ReplayPlayerSnapshot>,
    pub defuse_time_secs: Option<f64>,
    pub defuser_name: Option<String>,
    pub defuse_norm_x: Option<f64>,
    pub defuse_norm_y: Option<f64>,
    pub defuse_player_positions: Vec<ReplayPlayerSnapshot>,
    pub kills: Vec<RoundKillEvent>,
}

impl From<cm::career::RoundReplayData> for RoundReplayData {
    fn from(v: cm::career::RoundReplayData) -> Self {
        Self {
            round_num: v.round_num,
            winning_team: v.winning_team,
            end_type: v.end_type,
            plant_time_secs: v.plant_time_secs,
            plant_site: v.plant_site,
            planter_name: v.planter_name,
            plant_norm_x: v.plant_norm_x,
            plant_norm_y: v.plant_norm_y,
            plant_player_positions: v.plant_player_positions.into_iter().map(Into::into).collect(),
            defuse_time_secs: v.defuse_time_secs,
            defuser_name: v.defuser_name,
            defuse_norm_x: v.defuse_norm_x,
            defuse_norm_y: v.defuse_norm_y,
            defuse_player_positions: v.defuse_player_positions.into_iter().map(Into::into).collect(),
            kills: v.kills.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct MatchDetailSnapshot {
    pub match_id: String,
    pub map_name: String,
    pub map_splash: Option<String>,
    pub map_minimap_icon: Option<String>,
    pub queue_id: String,
    pub start_time_epoch_secs: i64,
    pub duration_secs: i64,
    pub my_team_id: String,
    pub my_team_score: i32,
    pub enemy_team_score: i32,
    pub won: bool,
    pub scoreboard: Vec<ScoreboardPlayer>,
    pub rounds: Vec<RoundReplayData>,
}

impl From<cm::career::MatchDetailSnapshot> for MatchDetailSnapshot {
    fn from(v: cm::career::MatchDetailSnapshot) -> Self {
        Self {
            match_id: v.match_id,
            map_name: v.map_name,
            map_splash: v.map_splash,
            map_minimap_icon: v.map_minimap_icon,
            queue_id: v.queue_id,
            start_time_epoch_secs: v.start_time_epoch_secs,
            duration_secs: v.duration_secs,
            my_team_id: v.my_team_id,
            my_team_score: v.my_team_score,
            enemy_team_score: v.enemy_team_score,
            won: v.won,
            scoreboard: v.scoreboard.into_iter().map(Into::into).collect(),
            rounds: v.rounds.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct KnownPlayerSummary {
    pub puuid: String,
    pub display_name: String,
    pub current_rank: Option<RankInfo>,
}

impl From<cm::career::KnownPlayerSummary> for KnownPlayerSummary {
    fn from(v: cm::career::KnownPlayerSummary) -> Self {
        Self {
            puuid: v.puuid,
            display_name: v.display_name,
            current_rank: v.current_rank.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct PlayerProfileSnapshot {
    pub puuid: String,
    pub display_name: String,
    pub account_level: Option<i32>,
    pub player_card_small: Option<String>,
    pub player_card_wide: Option<String>,
    pub player_title: Option<String>,
    pub current_rank: RankInfo,
    pub peak_rank: RankInfo,
    pub previous_act_rank: RankInfo,
    pub leaderboard_rank: Option<u32>,
    pub matches_analyzed: u32,
    pub wins: u32,
    pub losses: u32,
    pub win_rate_percent: f64,
    pub kd_ratio: f64,
    pub avg_acs: f64,
    pub avg_adr: f64,
    pub hs_percent: f64,
    pub avg_kills: f64,
    pub avg_deaths: f64,
    pub avg_assists: f64,
    pub net_rr_last_10: i32,
    pub current_streak_label: String,
    pub rr_trend: Vec<CompetitiveMatchUpdate>,
    pub agent_stats: Vec<AgentPerformanceStat>,
    pub recent_matches: Vec<MatchHistoryItem>,
}

impl From<cm::career::PlayerProfileSnapshot> for PlayerProfileSnapshot {
    fn from(v: cm::career::PlayerProfileSnapshot) -> Self {
        Self {
            puuid: v.puuid,
            display_name: v.display_name,
            account_level: v.account_level,
            player_card_small: v.player_card_small,
            player_card_wide: v.player_card_wide,
            player_title: v.player_title,
            current_rank: v.current_rank.into(),
            peak_rank: v.peak_rank.into(),
            previous_act_rank: v.previous_act_rank.into(),
            leaderboard_rank: v.leaderboard_rank,
            matches_analyzed: v.matches_analyzed,
            wins: v.wins,
            losses: v.losses,
            win_rate_percent: v.win_rate_percent,
            kd_ratio: v.kd_ratio,
            avg_acs: v.avg_acs,
            avg_adr: v.avg_adr,
            hs_percent: v.hs_percent,
            avg_kills: v.avg_kills,
            avg_deaths: v.avg_deaths,
            avg_assists: v.avg_assists,
            net_rr_last_10: v.net_rr_last_10,
            current_streak_label: v.current_streak_label,
            rr_trend: v.rr_trend.into_iter().map(Into::into).collect(),
            agent_stats: v.agent_stats.into_iter().map(Into::into).collect(),
            recent_matches: v.recent_matches.into_iter().map(Into::into).collect(),
        }
    }
}

// ---- UniFFI Engine Object ----

#[derive(uniffi::Object)]
pub struct MyValEngine {
    core: Arc<MyValEngineCore>,
    runtime: tokio::runtime::Runtime,
}

impl MyValEngine {
    async fn run_on_tokio<T, F, Fut>(&self, f: F) -> Result<T, FfiError>
    where
        T: Send + 'static,
        F: FnOnce(Arc<MyValEngineCore>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, MyValError>> + Send + 'static,
    {
        let core = Arc::clone(&self.core);
        let handle = self.runtime.spawn(async move { f(core).await });
        match handle.await {
            Ok(res) => res.map_err(Into::into),
            Err(e) => Err(FfiError::Network {
                detail: format!("Async task failure: {e}"),
            }),
        }
    }
}

#[uniffi::export]
impl MyValEngine {
    #[uniffi::constructor]
    pub fn new(db_path: String) -> Result<Arc<Self>, FfiError> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("myval-tokio")
            .build()
            .map_err(|e| FfiError::Storage {
                detail: format!("Failed to start runtime: {e}"),
            })?;
        let core = Arc::new(MyValEngineCore::open(&db_path)?);
        Ok(Arc::new(Self { core, runtime }))
    }

    pub fn get_riot_login_url(&self) -> String {
        self.core.get_riot_login_url()
    }

    pub async fn complete_auth_from_redirect_url(
        &self,
        redirect_url: String,
    ) -> Result<SessionSummary, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.complete_auth_from_redirect_url(&redirect_url)
                .await
                .map(Into::into)
        })
        .await
    }

    pub async fn refresh_session_from_cookie(
        &self,
        cookie_header: String,
    ) -> Result<SessionSummary, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.refresh_session_from_cookie(&cookie_header)
                .await
                .map(Into::into)
        })
        .await
    }

    pub async fn restore_session_json(&self, session_json: String) -> Result<SessionSummary, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.restore_session_json(&session_json).await.map(Into::into)
        })
        .await
    }

    pub async fn export_session_json(&self) -> Result<Option<String>, FfiError> {
        self.run_on_tokio(|core| async move { core.export_session_json().await })
            .await
    }

    pub async fn current_session(&self) -> Result<Option<SessionSummary>, FfiError> {
        self.run_on_tokio(|core| async move { Ok(core.current_session().await.map(Into::into)) })
            .await
    }

    pub async fn clear_session(&self) -> Result<(), FfiError> {
        self.run_on_tokio(|core| async move {
            core.clear_session().await;
            Ok(())
        })
        .await
    }

    pub fn needs_asset_resync(&self) -> bool {
        self.core.needs_asset_resync()
    }

    pub async fn sync_assets(&self) -> Result<AssetSummary, FfiError> {
        self.run_on_tokio(|core| async move { core.sync_assets().await.map(Into::into) })
            .await
    }

    pub fn get_asset_summary(&self) -> Result<AssetSummary, FfiError> {
        Ok(self.core.get_asset_summary()?.into())
    }

    pub fn list_agents(&self) -> Result<Vec<Agent>, FfiError> {
        Ok(self.core.list_agents()?.into_iter().map(Into::into).collect())
    }

    pub fn list_weapons(&self) -> Result<Vec<Weapon>, FfiError> {
        Ok(self.core.list_weapons()?.into_iter().map(Into::into).collect())
    }

    pub fn list_maps(&self) -> Result<Vec<MapInfo>, FfiError> {
        Ok(self.core.list_maps()?.into_iter().map(Into::into).collect())
    }

    pub fn list_all_weapon_skins(&self) -> Result<Vec<Skin>, FfiError> {
        Ok(self
            .core
            .list_all_weapon_skins()?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub fn list_player_cards(&self) -> Result<Vec<PlayerCard>, FfiError> {
        Ok(self
            .core
            .list_player_cards()?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub fn list_player_titles(&self) -> Result<Vec<PlayerTitle>, FfiError> {
        Ok(self
            .core
            .list_player_titles()?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub fn list_sprays(&self) -> Result<Vec<Spray>, FfiError> {
        Ok(self
            .core
            .list_sprays()?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub fn search_skins(
        &self,
        query: String,
        wishlist_only: bool,
        limit: u32,
    ) -> Result<Vec<Skin>, FfiError> {
        Ok(self
            .core
            .search_skins(&query, wishlist_only, limit)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub fn toggle_wishlist(&self, skin_uuid: String) -> Result<bool, FfiError> {
        Ok(self.core.toggle_wishlist(&skin_uuid)?)
    }

    pub fn get_wishlist_uuids(&self) -> Result<Vec<String>, FfiError> {
        Ok(self.core.get_wishlist_uuids()?)
    }

    pub async fn get_owned_skin_uuids(&self) -> Result<Vec<String>, FfiError> {
        self.run_on_tokio(|core| async move { Ok(core.get_owned_skin_uuids().await) })
            .await
    }

    pub async fn get_owned_chroma_uuids(&self) -> Result<Vec<String>, FfiError> {
        self.run_on_tokio(|core| async move { Ok(core.get_owned_chroma_uuids().await) })
            .await
    }

    pub async fn get_owned_level_uuids(&self) -> Result<Vec<String>, FfiError> {
        self.run_on_tokio(|core| async move { Ok(core.get_owned_level_uuids().await) })
            .await
    }

    pub async fn get_owned_agent_uuids(&self) -> Result<Vec<String>, FfiError> {
        self.run_on_tokio(|core| async move { Ok(core.get_owned_agent_uuids().await) })
            .await
    }

    pub async fn get_owned_spray_uuids(&self) -> Result<Vec<String>, FfiError> {
        self.run_on_tokio(|core| async move { Ok(core.get_owned_spray_uuids().await) })
            .await
    }

    pub async fn get_owned_card_uuids(&self) -> Result<Vec<String>, FfiError> {
        self.run_on_tokio(|core| async move { Ok(core.get_owned_card_uuids().await) })
            .await
    }

    pub async fn get_owned_title_uuids(&self) -> Result<Vec<String>, FfiError> {
        self.run_on_tokio(|core| async move { Ok(core.get_owned_title_uuids().await) })
            .await
    }

    pub async fn fetch_storefront(&self) -> Result<StorefrontSnapshot, FfiError> {
        self.run_on_tokio(|core| async move { core.fetch_storefront().await.map(Into::into) })
            .await
    }

    pub async fn get_cached_storefront(&self) -> Result<Option<StorefrontSnapshot>, FfiError> {
        self.run_on_tokio(|core| async move {
            Ok(core.get_cached_storefront().await?.map(Into::into))
        })
        .await
    }

    pub async fn poll_live_state(&self) -> Result<LiveState, FfiError> {
        self.run_on_tokio(|core| async move { core.poll_live_state().await.map(Into::into) })
            .await
    }

    pub async fn party_set_ready(&self, party_id: String, ready: bool) -> Result<PartySnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.party_set_ready(&party_id, ready).await.map(Into::into)
        })
        .await
    }

    pub async fn party_join_queue(&self, party_id: String) -> Result<PartySnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.party_join_queue(&party_id).await.map(Into::into)
        })
        .await
    }

    pub async fn party_leave_queue(&self, party_id: String) -> Result<PartySnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.party_leave_queue(&party_id).await.map(Into::into)
        })
        .await
    }

    pub async fn party_change_queue(
        &self,
        party_id: String,
        queue_id: String,
    ) -> Result<PartySnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.party_change_queue(&party_id, &queue_id)
                .await
                .map(Into::into)
        })
        .await
    }

    pub async fn party_set_accessibility(
        &self,
        party_id: String,
        open: bool,
    ) -> Result<PartySnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.party_set_accessibility(&party_id, open)
                .await
                .map(Into::into)
        })
        .await
    }

    pub async fn party_generate_invite_code(&self, party_id: String) -> Result<PartySnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.party_generate_invite_code(&party_id)
                .await
                .map(Into::into)
        })
        .await
    }

    pub async fn party_disable_invite_code(&self, party_id: String) -> Result<PartySnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.party_disable_invite_code(&party_id)
                .await
                .map(Into::into)
        })
        .await
    }

    pub async fn party_join_by_code(&self, invite_code: String) -> Result<PartySnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.party_join_by_code(&invite_code).await.map(Into::into)
        })
        .await
    }

    pub async fn pregame_select_agent(
        &self,
        match_id: String,
        agent_uuid: String,
    ) -> Result<PreGameSnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.pregame_select_agent(&match_id, &agent_uuid)
                .await
                .map(Into::into)
        })
        .await
    }

    pub async fn pregame_lock_agent(
        &self,
        match_id: String,
        agent_uuid: String,
    ) -> Result<PreGameSnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.pregame_lock_agent(&match_id, &agent_uuid)
                .await
                .map(Into::into)
        })
        .await
    }

    pub async fn pregame_quit_match(&self, match_id: String) -> Result<(), FfiError> {
        self.run_on_tokio(move |core| async move { core.pregame_quit_match(&match_id).await })
            .await
    }

    // ---- Collection, Loadout, Presets & Randomizer ----

    pub async fn fetch_collection_summary(&self) -> Result<CollectionSummary, FfiError> {
        self.run_on_tokio(|core| async move { core.fetch_collection_summary().await.map(Into::into) })
            .await
    }

    pub async fn fetch_player_loadout(&self) -> Result<PlayerLoadoutSnapshot, FfiError> {
        self.run_on_tokio(|core| async move { core.fetch_player_loadout().await.map(Into::into) })
            .await
    }

    pub async fn equip_weapon_skin(
        &self,
        weapon_uuid: String,
        skin_uuid: String,
    ) -> Result<PlayerLoadoutSnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.equip_weapon_skin(&weapon_uuid, &skin_uuid)
                .await
                .map(Into::into)
        })
        .await
    }

    pub async fn equip_weapon_skin_chroma(
        &self,
        weapon_uuid: String,
        skin_uuid: String,
        chroma_uuid: Option<String>,
    ) -> Result<PlayerLoadoutSnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.equip_weapon_skin_chroma(&weapon_uuid, &skin_uuid, chroma_uuid.as_deref())
                .await
                .map(Into::into)
        })
        .await
    }

    pub async fn equip_weapon_skin_full(
        &self,
        weapon_uuid: String,
        skin_uuid: String,
        chroma_uuid: Option<String>,
        level_uuid: Option<String>,
    ) -> Result<PlayerLoadoutSnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.equip_weapon_skin_full(
                &weapon_uuid,
                &skin_uuid,
                chroma_uuid.as_deref(),
                level_uuid.as_deref(),
            )
            .await
            .map(Into::into)
        })
        .await
    }

    pub async fn equip_spray(
        &self,
        slot_uuid: String,
        spray_uuid: String,
    ) -> Result<PlayerLoadoutSnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.equip_spray(&slot_uuid, &spray_uuid)
                .await
                .map(Into::into)
        })
        .await
    }

    pub async fn equip_player_card(
        &self,
        card_uuid: String,
    ) -> Result<PlayerLoadoutSnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.equip_player_card(&card_uuid).await.map(Into::into)
        })
        .await
    }

    pub async fn equip_player_title(
        &self,
        title_uuid: String,
    ) -> Result<PlayerLoadoutSnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.equip_player_title(&title_uuid).await.map(Into::into)
        })
        .await
    }

    pub async fn set_identity_privacy(
        &self,
        incognito: bool,
        hide_account_level: bool,
    ) -> Result<PlayerLoadoutSnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.set_identity_privacy(incognito, hide_account_level)
                .await
                .map(Into::into)
        })
        .await
    }

    pub async fn randomize_loadout(&self, seed: u64) -> Result<PlayerLoadoutSnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.randomize_loadout(seed).await.map(Into::into)
        })
        .await
    }

    pub async fn save_current_loadout_preset(
        &self,
        name: String,
        accent_color_hex: String,
    ) -> Result<LoadoutPreset, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.save_current_loadout_preset(&name, &accent_color_hex)
                .await
                .map(Into::into)
        })
        .await
    }

    pub async fn list_loadout_presets(&self) -> Result<Vec<LoadoutPreset>, FfiError> {
        self.run_on_tokio(|core| async move {
            Ok(core
                .list_loadout_presets()
                .await?
                .into_iter()
                .map(Into::into)
                .collect())
        })
        .await
    }

    pub async fn apply_loadout_preset(
        &self,
        preset_id: String,
    ) -> Result<PlayerLoadoutSnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.apply_loadout_preset(&preset_id).await.map(Into::into)
        })
        .await
    }

    pub fn delete_loadout_preset(&self, preset_id: String) -> Result<(), FfiError> {
        Ok(self.core.delete_loadout_preset(&preset_id)?)
    }

    // ---- Career & 2D Match Replay ----

    pub async fn get_cached_career(&self) -> Option<CareerSnapshot> {
        let core = Arc::clone(&self.core);
        self.runtime
            .spawn(async move { core.get_cached_career().await.map(Into::into) })
            .await
            .ok()
            .flatten()
    }

    pub async fn fetch_career(&self) -> Result<CareerSnapshot, FfiError> {
        self.run_on_tokio(|core| async move { core.fetch_career().await.map(Into::into) })
            .await
    }

    pub async fn fetch_match_detail(
        &self,
        match_id: String,
    ) -> Result<MatchDetailSnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.fetch_match_detail(&match_id).await.map(Into::into)
        })
        .await
    }

    pub async fn fetch_match_detail_for_player(
        &self,
        match_id: String,
        perspective_puuid: String,
    ) -> Result<MatchDetailSnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.fetch_match_detail_for_player(&match_id, &perspective_puuid)
                .await
                .map(Into::into)
        })
        .await
    }

    pub async fn fetch_player_profile(
        &self,
        target_puuid: String,
        force_refresh: bool,
    ) -> Result<PlayerProfileSnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.fetch_player_profile(&target_puuid, force_refresh)
                .await
                .map(Into::into)
        })
        .await
    }

    pub fn search_known_players(
        &self,
        query: String,
        limit: u32,
    ) -> Result<Vec<KnownPlayerSummary>, FfiError> {
        Ok(self
            .core
            .search_known_players(&query, limit)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub async fn lookup_player_profile_by_riot_id(
        &self,
        riot_id: String,
    ) -> Result<PlayerProfileSnapshot, FfiError> {
        self.run_on_tokio(move |core| async move {
            core.lookup_player_profile_by_riot_id(&riot_id)
                .await
                .map(Into::into)
        })
        .await
    }

    pub fn set_lookup_api_key(&self, key: String) -> Result<(), FfiError> {
        Ok(self.core.set_lookup_api_key(&key)?)
    }

    pub fn has_lookup_api_key(&self) -> bool {
        self.core.has_lookup_api_key()
    }
}
