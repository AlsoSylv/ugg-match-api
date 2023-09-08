use std::sync::Arc;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInfoSuggestions {
    pub query: String,
    pub region_id: &'static str,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchMatchSummaries {
    pub champion_id: Vec<i64>,
    pub page: i64,
    pub queue_type: Vec<i64>,
    pub duo_name: String,
    pub region_id: &'static str,
    pub role: Vec<u8>,
    pub season_ids: Vec<i32>,
    pub summoner_name: Arc<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlayerProfile {
    pub region_id: &'static str,
    pub summoner_name: Arc<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchProfileRanks {
    pub region_id: &'static str,
    pub summoner_name: Arc<String>,
    pub season_id: i32,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetOverallPlayerRanking {
    pub queue_type: i64,
    pub summoner_name: Arc<String>,
    pub region_id: &'static str,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchProfilePlayerInfo {
    pub region_id: &'static str,
    pub summoner_name: Arc<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchMatch {
    pub summoner_name: Arc<String>,
    pub region_id: &'static str,
    pub match_id: String,
    pub version: Arc<String>,
}
