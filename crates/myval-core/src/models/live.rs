use serde::{Deserialize, Serialize};

use super::mmr::RankInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiveState {
    Idle {
        message: String,
        recommended_poll_secs: u64,
    },
    InParty(PartySnapshot),
    PreGame(PreGameSnapshot),
    CoreGame(CoreGameSnapshot),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EquippedWeaponSkin {
    pub weapon_name: String,
    pub skin_name: String,
    pub skin_icon: Option<String>,
    pub tier_name: Option<String>,
    pub tier_color: String,
    pub is_animated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(default)]
pub struct PartyGroup {
    pub group_id: String,
    pub label: String,
    pub team_id: String,
    pub member_puuids: Vec<String>,
    pub confidence_shared_matches: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
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
