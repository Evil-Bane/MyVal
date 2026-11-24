use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RankInfo {
    pub tier: i32,
    pub tier_name: String,
    pub division_name: String,
    pub rr: i32,
    pub icon_url: Option<String>,
    pub color_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PlayerMmrSummary {
    pub puuid: String,
    pub current_rank: RankInfo,
    pub peak_rank: RankInfo,
    pub peak_rank_act_name: String,
    pub previous_act_rank: RankInfo,
    pub leaderboard_rank: Option<u32>,
    pub recent_results: Vec<bool>,
    /// Signed recent competitive outcomes (newest first): 1 = Win, -1 = Loss, 0 = Draw.
    pub recent_outcomes: Vec<i8>,
    /// Marks rows fetched with the competitiveupdates endpoint so legacy cached rows auto-heal.
    pub recent_results_loaded: bool,
}
