use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Deserialize Player Matches
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerMatchSummeries {
    pub data: Data,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub fetch_player_match_summaries: FetchPlayerMatchSummaries,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchPlayerMatchSummaries {
    pub finished_match_summaries: bool,
    pub match_summaries: Vec<MatchSummary>,
    pub total_num_matches: i64,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub champion_id: i64,
    pub hard_carry: f64,
    pub role: i64,
    pub summoner_name: String,
    pub teamplay: f64,
}

/// Deserialize Player Suggestions
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSuggestions {
    pub data: PlayerData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerData {
    pub player_info_suggestions: Vec<PlayerInfoSuggestion>,
}

#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlayer {
    pub data: UpdateData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateData {
    pub update_player_profile: UpdatePlayerProfile,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlayerProfile {
    pub error_reason: Option<String>,
    pub success: bool,
}

/// Deserealize PlayerRanks
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRank {
    pub data: RankData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankData {
    pub fetch_profile_ranks: FetchProfileRanks,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchProfileRanks {
    pub rank_scores: Vec<RankScore>,
}

#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRanking {
    pub data: RankingData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankingData {
    pub overall_ranking: Option<OverallRanking>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverallRanking {
    pub overall_ranking: i64,
    pub total_player_count: i64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInfo {
    pub data: ProfilePlayerInfo,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePlayerInfo {
    pub profile_player_info: Option<ProfileInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileInfo {
    pub icon_id: i64,
    pub summoner_level: i64,
    pub summoner_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChampionJson {
    #[serde(rename = "type")]
    pub _type: String,
    pub format: String,
    pub version: String,
    pub data: HashMap<String, ChampData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChampData {
    pub version: String,
    #[serde(rename = "id")]
    pub key: String,
    #[serde(rename = "key")]
    pub id: String,
    pub name: String,
    pub title: String,
    pub blurb: String,
    pub info: Info,
    pub image: Image,
    pub tags: Vec<Tag>,
    pub partype: String,
    pub stats: HashMap<String, f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub full: String,
    pub sprite: String,
    pub group: String,
    pub x: i64,
    pub y: i64,
    pub w: i64,
    pub h: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    pub attack: i64,
    pub defense: i64,
    pub magic: i64,
    pub difficulty: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Tag {
    Assassin,
    Fighter,
    Mage,
    Marksman,
    Support,
    Tank,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetMatch {
    pub data: MatchData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MatchData {
    #[serde(rename = "match")]
    pub data_match: Match,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Match {
    pub match_summary: FullMatchSummary,
    pub performance_score: Vec<PerformanceScore>,
    pub winning_team: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FullMatchSummary {
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
    pub team_a: Vec<MatchTeam>,
    pub team_b: Vec<MatchTeam>,
    pub version: String,
    pub vision_score: i64,
    pub win: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceScore {
    pub hard_carry: i64,
    pub summoner_name: String,
    pub teamplay: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchTeam {
    pub champion_id: i64,
    pub damage: i64,
    pub role: i64,
    pub summoner_name: String,
    pub team_id: i64,
}
