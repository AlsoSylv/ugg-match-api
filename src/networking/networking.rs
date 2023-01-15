use graphql_client::GraphQLQuery;
use serde::Deserialize;

use crate::{
    graphql::structs::{
        fetch_match_summaries, fetch_profile_ranks, player_info_suggestions, update_player_profile,
        FetchMatchSummaries, FetchProfileRanks, PlayerInfoSuggestions, UpdatePlayerProfile,
    },
    structs,
};

pub async fn fetch_match_summaries(
    mut name: String,
    region_id: &str,
    role: Vec<Option<i64>>,
    page: i64,
    client: &reqwest::Client,
) -> Result<structs::PlayerMatchSummeries, reqwest::Error> {
    remove_whitespace(&mut name);
    let vars = fetch_match_summaries::Variables {
        champion_id: Some(Vec::new()),
        page: Some(page),
        queue_type: Some(Vec::new()),
        duo_name: Some("".to_string()),
        region_id: region_id.to_string(),
        role: Some(role),
        season_ids: vec![Some(18)],
        summoner_name: name.to_string(),
    };

    request::<FetchMatchSummaries, structs::PlayerMatchSummeries>(vars, client).await
}

pub async fn player_suggestiosn(
    mut name: String,
    client: &reqwest::Client,
) -> Result<structs::PlayerSuggestions, reqwest::Error> {
    remove_whitespace(&mut name);
    let vars = player_info_suggestions::Variables {
        query: name.to_lowercase(),
        region_id: "na1".to_string(),
    };

    request::<PlayerInfoSuggestions, structs::PlayerSuggestions>(vars, client).await
}

pub async fn update_player(
    mut name: String,
    client: &reqwest::Client,
) -> Result<structs::UpdatePlayer, reqwest::Error> {
    remove_whitespace(&mut name);
    let vars = update_player_profile::Variables {
        region_id: "na1".to_string(),
        summoner_name: name.to_lowercase(),
    };

    request::<UpdatePlayerProfile, structs::UpdatePlayer>(vars, client).await
}

pub async fn profile_ranks(
    mut name: String,
    client: &reqwest::Client,
) -> Result<structs::PlayerRank, reqwest::Error> {
    remove_whitespace(&mut name);
    let vars = fetch_profile_ranks::Variables {
        region_id: "na1".to_string(),
        summoner_name: name.to_lowercase(),
    };

    request::<FetchProfileRanks, structs::PlayerRank>(vars, client).await
}

async fn request<T: GraphQLQuery, R: for<'de> Deserialize<'de>>(
    vars: T::Variables,
    client: &reqwest::Client,
) -> Result<R, reqwest::Error> {
    let request_body = T::build_query(vars);
    let request = client
        .post("https://u.gg/api")
        .json(&request_body)
        .send()
        .await;

    match request {
        Ok(response) => {
            let json = response.json::<R>().await;
            match json {
                Ok(json) => Ok(json),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

fn remove_whitespace(s: &mut String) {
    s.retain(|c| !c.is_whitespace());
}
