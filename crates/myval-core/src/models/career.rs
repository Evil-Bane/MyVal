use serde::{Deserialize, Serialize};

use super::mmr::RankInfo;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ContractProgressSnapshot {
    pub active_contract_name: String,
    pub current_level: u32,
    pub max_level: u32,
    pub xp_in_level: i64,
    pub xp_required_for_next: i64,
    pub total_xp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AccountPenalty {
    pub reason: String,
    pub expires_at: Option<String>,
    pub games_remaining: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MissionProgress {
    pub mission_id: String,
    pub title: String,
    pub current_progress: i32,
    pub target_progress: i32,
    pub xp_grant: i32,
    pub completed: bool,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ReplayPlayerSnapshot {
    pub puuid: String,
    pub display_name: String,
    pub team_id: String,
    pub agent_icon: Option<String>,
    /// Normalized [0.0, 1.0] minimap coordinates
    pub norm_x: f64,
    pub norm_y: f64,
    /// Projected facing angle in radians on the 2D minimap canvas (0 = right, +pi/2 = down)
    pub view_angle_rad: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
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
    /// Normalized [0.0, 1.0] minimap coordinates
    pub killer_norm_x: f64,
    pub killer_norm_y: f64,
    pub victim_norm_x: f64,
    pub victim_norm_y: f64,
    /// Every alive player's exact minimap position + view direction at the moment of this kill
    pub player_positions: Vec<ReplayPlayerSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct KnownPlayerSummary {
    pub puuid: String,
    pub display_name: String,
    pub current_rank: Option<RankInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
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
