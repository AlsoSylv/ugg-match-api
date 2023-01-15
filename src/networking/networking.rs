use graphql_client::GraphQLQuery;

use crate::{
    graphql::structs::{
        fetch_match_summaries, player_info_suggestions, update_player_profile, FetchMatchSummaries,
        PlayerInfoSuggestions, UpdatePlayerProfile,
    },
    structs,
};

pub async fn fetch_match_summaries(
    mut name: String,
    region_id: &str,
    role: Vec<Option<i64>>,
    page: i64,
    client: reqwest::Client,
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
    let request_body = FetchMatchSummaries::build_query(vars);
    let request = client
        .post("https://u.gg/api")
        .json(&request_body)
        .send()
        .await;
    match request {
        Ok(response) => {
            let json = response.json::<structs::PlayerMatchSummeries>().await;
            match json {
                Ok(json) => Ok(json),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
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
    let request_body = PlayerInfoSuggestions::build_query(vars);
    let request = client
        .post("https://u.gg/api")
        .json(&request_body)
        .send()
        .await;
    match request {
        Ok(response) => {
            let json = response.json::<structs::PlayerSuggestions>().await;
            match json {
                Ok(json) => Ok(json),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
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
    let request_body = UpdatePlayerProfile::build_query(vars);
    let request = client
        .post("https://u.gg/api")
        .json(&request_body)
        .send()
        .await;
    match request {
        Ok(response) => {
            let json = response.json::<structs::UpdatePlayer>().await;
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
