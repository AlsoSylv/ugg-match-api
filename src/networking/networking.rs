use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::graphql::structs::{GetOverallPlayerRanking, PlayerInfoSuggestions};
use crate::{
    graphql::structs::{
        FetchMatch, FetchMatchSummaries, GetSummonerProfile,
        UpdatePlayerProfile,
    },
    structs,
};

const BASE_URL: &str = "https://u.gg/api";

// Season 13 = 21
// Season 14 = 22
const SEASON_ID: u8 = 22;

const MATCH_SUMMARIES: &str = include_str!("../graphql/match_query.graphql");

pub async fn fetch_match_summaries(
    name: &str,
    tag_line: &str,
    region_id: &str,
    role: &[u8],
    page: u8,
    client: &reqwest::Client,
) -> Result<structs::PlayerMatchSummaries, reqwest::Error> {
    request(
        MATCH_SUMMARIES,
        FetchMatchSummaries {
            champion_id: &[],
            page,
            queue_type: &[],
            duo_riot_user_name: "",
            duo_riot_tag_line: "",
            region_id,
            role,
            season_ids: &[SEASON_ID],
            riot_user_name: name,
            riot_tag_line: tag_line,
        },
        client,
        BASE_URL,
    )
    .await
}

const PLAYER_SUGGESTIONS: &str = include_str!("../graphql/player_suggestion_query.graphql");

pub async fn player_suggestions(
    name: Arc<String>,
    client: &reqwest::Client,
) -> Result<structs::PlayerSuggestions, reqwest::Error> {
    request(
        PLAYER_SUGGESTIONS,
        PlayerInfoSuggestions {
            query: name.to_lowercase(),
            region_id: "na1",
        },
        client,
        BASE_URL,
    )
    .await
}

const UPDATE_PLAYER: &str = include_str!("../graphql/update_profile_query.graphql");

pub async fn update_player(
    name: &str,
    tag_line: &str,
    client: &reqwest::Client,
    region_id: &str,
) -> Result<structs::UpdatePlayer, reqwest::Error> {
    request(
        UPDATE_PLAYER,
        UpdatePlayerProfile {
            region_id,
            riot_user_name: name,
            riot_tag_line: tag_line,
        },
        client,
        BASE_URL,
    )
    .await
}

const PLAYER_RANKING: &str = include_str!("../graphql/overall_player_ranking.graphql");

pub async fn player_ranking(
    riot_user_name: &str,
    riot_tag_line: &str,
    client: &reqwest::Client,
    region_id: &'static str,
) -> Result<structs::PlayerRanking, reqwest::Error> {
    request(
        PLAYER_RANKING,
        GetOverallPlayerRanking {
            region_id,
            riot_user_name,
            riot_tag_line,
            queue_type: 420,
        },
        client,
        BASE_URL,
    )
        .await
}

const PLAYER_INFO: &str = include_str!("../graphql/profile_player_info.graphql");

pub async fn player_info(
    name: Arc<String>,
    tag_line: Arc<String>,
    region_id: &'static str,
    client: &reqwest::Client,
) -> Result<structs::PlayerInfo, reqwest::Error> {
    request(
        PLAYER_INFO,
        GetSummonerProfile {
            region_id,
            riot_user_name: name,
            riot_tag_line: tag_line,
            season_id: SEASON_ID,
        },
        client,
        BASE_URL,
    )
    .await
}

const FETCH_MATCH: &str = include_str!("../graphql/fetch_match.graphql");

pub async fn fetch_match(
    name: &str,
    tag_line: &str,
    region_id: &str,
    id: &str,
    version: &str,
    client: &reqwest::Client,
) -> Result<structs::GetMatch, reqwest::Error> {
    request(
        FETCH_MATCH,
        FetchMatch {
            region_id,
            riot_user_name: name,
            riot_tag_line: tag_line,
            match_id: id,
            version,
        },
        client,
        BASE_URL,
    )
    .await
}

async fn request<Data>(
    query: &str,
    variables: impl Serialize,
    client: &reqwest::Client,
    url: &str,
) -> Result<Data, reqwest::Error>
where
    Data: DeserializeOwned,
{
    let res = client
        .post(url)
        .json(&GQLQuery { variables, query })
        .send()
        .await?;

    let value: serde_json::Value = res.json().await?;

    println!("{}", value);

    Ok(serde_json::from_value(value).unwrap())
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct GQLQuery<'a, T> {
    variables: T,
    query: &'a str,
}
