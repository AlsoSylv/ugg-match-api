use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    graphql::structs::{
        FetchMatchSummaries, FetchProfileRanks, GetOverallPlayerRanking, PlayerInfoSuggestions,
        UpdatePlayerProfile,
    },
    structs,
};

const URL: &str = "https://u.gg/api";

const MATCH_SUMMERIES: &str = include_str!("../graphql/match_query.graphql");

pub async fn fetch_match_summaries(
    mut name: String,
    region_id: &'static str,
    role: Vec<i64>,
    page: i64,
    client: &reqwest::Client,
) -> Result<structs::PlayerMatchSummeries, reqwest::Error> {
    remove_whitespace(&mut name);
    let vars = FetchMatchSummaries {
        champion_id: Vec::new(),
        page,
        queue_type: Vec::new(),
        duo_name: "".to_string(),
        region_id: region_id,
        role,
        season_ids: vec![18],
        summoner_name: name.to_string(),
    };

    request::<FetchMatchSummaries, structs::PlayerMatchSummeries>(
        MATCH_SUMMERIES,
        vars,
        client,
        URL,
    )
    .await
}

const PLAYER_SUGGESTIONS: &str = include_str!("../graphql/player_suggestion_query.graphql");

pub async fn player_suggestiosn(
    mut name: String,
    client: &reqwest::Client,
) -> Result<structs::PlayerSuggestions, reqwest::Error> {
    remove_whitespace(&mut name);
    let vars = PlayerInfoSuggestions {
        query: name.to_lowercase(),
        region_id: "na1",
    };

    request::<PlayerInfoSuggestions, structs::PlayerSuggestions>(
        PLAYER_SUGGESTIONS,
        vars,
        client,
        URL,
    )
    .await
}

const UPDATE_PLAYER: &str = include_str!("../graphql/update_profile_query.graphql");

pub async fn update_player(
    mut name: String,
    client: &reqwest::Client,
) -> Result<structs::UpdatePlayer, reqwest::Error> {
    remove_whitespace(&mut name);
    let vars = UpdatePlayerProfile {
        region_id: "na1",
        summoner_name: name.to_lowercase(),
    };

    request::<UpdatePlayerProfile, structs::UpdatePlayer>(UPDATE_PLAYER, vars, client, URL).await
}

const PROFILE_RANKS: &str = include_str!("../graphql/fetch_profile_rank_queries.graphql");

pub async fn profile_ranks(
    mut name: String,
    client: &reqwest::Client,
) -> Result<structs::PlayerRank, reqwest::Error> {
    remove_whitespace(&mut name);
    let vars = FetchProfileRanks {
        region_id: "na1",
        summoner_name: name.to_lowercase(),
    };

    request(PROFILE_RANKS, vars, client, URL).await
}

const PLAYER_RANKING: &str = include_str!("../graphql/overall_player_ranking.graphql");

pub async fn player_ranking(
    mut name: String,
    client: &reqwest::Client,
) -> Result<structs::PlayerRanking, reqwest::Error> {
    remove_whitespace(&mut name);
    let vars = GetOverallPlayerRanking {
        region_id: "na1",
        summoner_name: name.to_lowercase(),
        queue_type: 420,
    };

    request(PLAYER_RANKING, vars, client, URL).await
}

const PLAYER_INFO: &str = include_str!("../graphql/profile_player_info.graphql");

pub async fn player_info(
    mut name: String,
    region_id: &'static str,
    client: &reqwest::Client,
) -> Result<structs::PlayerInfo, reqwest::Error> {
    remove_whitespace(&mut name);
    let vars = json!({
        "regionId": region_id,
        "summonerName": name
    });

    request(PLAYER_INFO, vars, client, URL).await
}

fn remove_whitespace(s: &mut String) {
    s.retain(|c| !c.is_whitespace());
}

async fn request<Vars: Serialize, Data: for<'de> Deserialize<'de>>(
    query: &str,
    variables: Vars,
    client: &reqwest::Client,
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
