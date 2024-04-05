use std::sync::Arc;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInfoSuggestions {
    pub query: String,
    pub region_id: &'static str,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchMatchSummaries<'s> {
    pub riot_user_name: &'s str,
    pub riot_tag_line: &'s str,
    pub duo_riot_user_name: &'s str,
    pub duo_riot_tag_line: &'s str,
    pub queue_type: &'s [i64],
    pub region_id: &'s str,
    pub role: &'s [u8],
    pub season_ids: &'s [i32],
    pub champion_id: &'s [i16],
    pub page: i64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlayerProfile<'s> {
    pub region_id: &'s str,
    pub riot_user_name: &'s str,
    pub riot_tag_line: &'s str,
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
pub struct GetSummonerProfile {
    pub region_id: &'static str,
    pub riot_user_name: Arc<String>,
    pub riot_tag_line: Arc<String>,
    pub season_id: i32,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchMatch<'a> {
    pub summoner_name: Arc<String>,
    pub region_id: &'static str,
    pub match_id: &'a str,
    pub version: &'a str,
}
