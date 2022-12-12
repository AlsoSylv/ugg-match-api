#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInfoSuggestions {
    pub query: String,
    pub region_id: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchMatchSummaries {
    pub champion_id: Vec<i64>,
    pub page: i64,
    pub queue_type: Vec<i64>,
    pub duo_name: String,
    pub region_id: String,
    pub role: Vec<i64>,
    pub season_ids: Vec<i64>,
    pub summoner_name: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlayerProfile {
    pub region_id: String,
    pub summoner_name: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchProfileRanks {
    pub region_id: String,
    pub summoner_name: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetOverallPlayerRanking {
    pub queue_type: i64,
    pub summoner_name: String,
    pub region_id: String,
}
