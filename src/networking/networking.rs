use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    graphql::structs::{
        FetchMatchSummaries, FetchProfilePlayerInfo, FetchProfileRanks, GetOverallPlayerRanking,
        UpdatePlayerProfile,
    },
    structs,
};

const BASE_URL: &str = "https://u.gg/api";

const SEASON_ID: i32 = 21;

const MATCH_SUMMERIES: &str = include_str!("../graphql/match_query.graphql");

pub async fn fetch_match_summaries(
    name: Arc<String>,
    region_id: &'static str,
    role: Vec<i8>,
    page: i64,
    client: reqwest::Client,
) -> Result<structs::PlayerMatchSummeries, reqwest::Error> {
    request(
        MATCH_SUMMERIES,
        FetchMatchSummaries {
            champion_id: Vec::new(),
            page,
            queue_type: Vec::new(),
            duo_name: "".to_string(),
            region_id,
            role,
            season_ids: vec![SEASON_ID],
            summoner_name: name.to_lowercase(),
        },
        client,
        BASE_URL,
    )
    .await
}

// const PLAYER_SUGGESTIONS: &str = include_str!("../graphql/player_suggestion_query.graphql");
//
// pub async fn player_suggestiosn(
//     name: Arc<String>,
//     client: &reqwest::Client,
// ) -> Result<structs::PlayerSuggestions, reqwest::Error> {
//     request(
//         PLAYER_SUGGESTIONS,
//         PlayerInfoSuggestions {
//             query: name.to_lowercase(),
//             region_id: "na1",
//         },
//         client,
//         BASE_URL,
//     )
//     .await
// }

const UPDATE_PLAYER: &str = include_str!("../graphql/update_profile_query.graphql");

pub async fn update_player(
    name: Arc<String>,
    client: reqwest::Client,
) -> Result<structs::UpdatePlayer, reqwest::Error> {
    request(
        UPDATE_PLAYER,
        UpdatePlayerProfile {
            region_id: "na1",
            summoner_name: name.to_lowercase(),
        },
        client,
        BASE_URL,
    )
    .await
}

const PROFILE_RANKS: &str = include_str!("../graphql/fetch_profile_rank_queries.graphql");

pub async fn profile_ranks(
    name: Arc<String>,
    client: reqwest::Client,
) -> Result<structs::PlayerRank, reqwest::Error> {
    request(
        PROFILE_RANKS,
        FetchProfileRanks {
            region_id: "na1",
            summoner_name: name.to_lowercase(),
            season_id: SEASON_ID,
        },
        client,
        BASE_URL,
    )
    .await
}

const PLAYER_RANKING: &str = include_str!("../graphql/overall_player_ranking.graphql");

pub async fn player_ranking(
    name: Arc<String>,
    client: reqwest::Client,
) -> Result<structs::PlayerRanking, reqwest::Error> {
    request(
        PLAYER_RANKING,
        GetOverallPlayerRanking {
            region_id: "na1",
            summoner_name: name.to_lowercase(),
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
    region_id: &'static str,
    client: reqwest::Client,
) -> Result<structs::PlayerInfo, reqwest::Error> {
    request(
        PLAYER_INFO,
        FetchProfilePlayerInfo {
            region_id,
            summoner_name: name.to_lowercase(),
        },
        client,
        BASE_URL,
    )
    .await
}

async fn request<Vars: Serialize, Data: for<'de> Deserialize<'de>>(
    query: &str,
    variables: Vars,
    client: reqwest::Client,
    url: &str,
) -> Result<Data, reqwest::Error> {
    client
        .post(url)
        .json(&GQLQuery { variables, query })
        .send()
        .await?
        .json()
        .await
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct GQLQuery<'a, T> {
    variables: T,
    query: &'a str,
}
