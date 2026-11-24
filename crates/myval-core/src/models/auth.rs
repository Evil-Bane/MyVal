use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegionShard {
    pub region: String,
    pub shard: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiotSession {
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
    pub region: String,
    pub shard: String,
    pub access_token: String,
    pub id_token: String,
    pub entitlements_token: String,
    pub expires_at_epoch_secs: i64,
}

impl RiotSession {
    pub fn riot_id(&self) -> String {
        if self.tag_line.is_empty() {
            self.game_name.clone()
        } else {
            format!("{}#{}", self.game_name, self.tag_line)
        }
    }

    pub fn is_expired(&self, now_epoch_secs: i64) -> bool {
        self.expires_at_epoch_secs > 0 && now_epoch_secs >= self.expires_at_epoch_secs.saturating_sub(30)
    }
}
