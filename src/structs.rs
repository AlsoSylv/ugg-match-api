use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Deserialize Player Matches
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerMatchSummeries {
    pub data: Data,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub fetch_player_match_summaries: FetchPlayerMatchSummaries,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchPlayerMatchSummaries {
    pub finished_match_summaries: bool,
    pub match_summaries: Vec<MatchSummary>,
    pub total_num_matches: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchSummary {
    pub assists: i64,
    pub champion_id: i64,
    pub cs: i64,
    pub damage: i64,
    pub deaths: i64,
    pub gold: i64,
    pub items: Vec<i64>,
    pub jungle_cs: i64,
    pub kill_participation: i64,
    pub kills: i64,
    pub level: i64,
    pub match_creation_time: i64,
    pub match_duration: i64,
    pub match_id: i64,
    pub maximum_kill_streak: i64,
    pub primary_style: i64,
    pub ps_hard_carry: i64,
    pub ps_team_play: i64,
    pub queue_type: String,
    pub region_id: String,
    pub role: i64,
    pub runes: Vec<i64>,
    pub sub_style: i64,
    pub summoner_name: String,
    pub summoner_spells: Vec<i64>,
    pub team_a: Vec<Team>,
    pub team_b: Vec<Team>,
    pub version: String,
    pub vision_score: i64,
    pub win: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub champion_id: i64,
    pub hard_carry: f64,
    pub role: i64,
    pub summoner_name: String,
    pub teamplay: f64,
}

/// Deserialize Player Suggestions
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSuggestions {
    pub data: PlayerData,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerData {
    pub player_info_suggestions: Vec<PlayerInfoSuggestion>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInfoSuggestion {
    // #[serde(rename = "__typename")]
    // pub typename: String,
    pub icon_id: i64,
    pub puuid_v4: String,
    pub summoner_level: i64,
    pub summoner_name: String,
}

/// Deserialize Player Updates
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlayer {
    pub data: UpdateData,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateData {
    pub update_player_profile: UpdatePlayerProfile,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlayerProfile {
    pub error_reason: String,
    pub success: bool,
}

/// Deserealize PlayerRanks
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRank {
    pub data: RankData,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankData {
    pub fetch_profile_ranks: FetchProfileRanks,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchProfileRanks {
    pub rank_scores: Vec<RankScore>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankScore {
    pub last_updated_at: Option<i64>,
    pub losses: i64,
    pub lp: i64,
    pub promo_progress: Option<String>,
    pub queue_type: String,
    pub rank: String,
    pub role: Option<String>,
    pub season_id: i64,
    pub tier: String,
    pub wins: i64,
}

/// Deserialize Player Overview
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRanking {
    pub data: RankingData,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankingData {
    pub overall_ranking: Option<OverallRanking>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverallRanking {
    pub overall_ranking: i64,
    pub total_player_count: i64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInfo {
    pub data: ProfilePlayerInfo,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePlayerInfo {
    pub profile_player_info: Option<ProfileInfo>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileInfo {
    pub icon_id: i64,
    pub summoner_level: i64,
    pub summoner_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampionJson {
    #[serde(rename = "type")]
    pub _type: String,
    pub format: String,
    pub version: String,
    pub data: HashMap<String, Datum>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Datum {
    pub version: String,
    pub id: String,
    pub key: String,
    pub name: String,
    pub title: String,
    pub blurb: String,
    pub info: Info,
    pub image: Image,
    pub tags: Vec<Tag>,
    pub partype: String,
    pub stats: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub full: String,
    pub sprite: String,
    pub group: String,
    pub x: i64,
    pub y: i64,
    pub w: i64,
    pub h: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub attack: i64,
    pub defense: i64,
    pub magic: i64,
    pub difficulty: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Tag {
    Assassin,
    Fighter,
    Mage,
    Marksman,
    Support,
    Tank,
}
