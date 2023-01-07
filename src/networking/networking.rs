use serde::{Deserialize, Serialize};

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
    region_id: &str,
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
        region_id: region_id.to_string(),
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
        region_id: "na1".to_string(),
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
        region_id: "na1".to_string(),
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
        region_id: "na1".to_string(),
        summoner_name: name.to_lowercase(),
    };

    request::<FetchProfileRanks, structs::PlayerRank>(PROFILE_RANKS, vars, client, URL).await
}

const PLAYER_RANKING: &str = include_str!("../graphql/overall_player_ranking.graphql");

pub async fn player_ranking(
    mut name: String,
    client: &reqwest::Client,
) -> Result<structs::PlayerRanking, reqwest::Error> {
    remove_whitespace(&mut name);
    let vars = GetOverallPlayerRanking {
        region_id: "na1".to_string(),
        summoner_name: name.to_lowercase(),
        queue_type: 420,
    };

    request::<GetOverallPlayerRanking, structs::PlayerRanking>(PLAYER_RANKING, vars, client, URL)
        .await
}

fn remove_whitespace(s: &mut String) {
    s.retain(|c| !c.is_whitespace());
}

async fn request<Vars: Serialize, Data: for<'de> Deserialize<'de>>(
    query: &str,
    vars: Vars,
    client: &reqwest::Client,
    url: &str,
) -> Result<Data, reqwest::Error> {
    let json = GQLQuery {
        variables: vars,
        query: query.to_string(),
    };

    let client = client.post(url).json(&json).send().await;
    match client {
        Ok(response) => {
            let json = response.json::<Data>().await;
            match json {
                Ok(json) => Ok(json),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct GQLQuery<T> {
    variables: T,
    query: String
}