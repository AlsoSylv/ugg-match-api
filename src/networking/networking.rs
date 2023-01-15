use graphql_client::GraphQLQuery;

use crate::{
    format_time,
    graphql::structs::{
        fetch_match_summaries, player_info_suggestions, FetchMatchSummaries, PlayerInfoSuggestions,
    },
    structs, MatchSummeryTranslated,
};

pub async fn fetch_match_summaries(
    mut name: String,
    region_id: &str,
    role: Vec<Option<i64>>,
    page: i64,
    client: reqwest::Client,
) -> Result<MatchSummeryTranslated, reqwest::Error> {
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
    let res = client
        .post("https://u.gg/api")
        .json(&request_body)
        .send()
        .await;
    match res {
        Ok(yay) => match yay.json::<structs::PlayerMatchSummeries>().await {
            Ok(json) => {
                let summeries = json.data.fetch_player_match_summaries.match_summaries;
                if summeries.is_empty() {
                    // This should be unreachable if the API is impmenented properly
                    unreachable!()
                }
                let last_match = &summeries[0];

                let kda = format!(
                    "{}/{}/{}",
                    last_match.kills, last_match.deaths, last_match.assists
                );
                let _gold = last_match.gold;
                let kp = format!("{}%", last_match.kill_participation);

                let time = format_time(last_match.match_duration);
                Ok(MatchSummeryTranslated { time, kda, kp })
            }
            Err(err) => Err(err),
        },
        Err(boo) => Err(boo),
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
    let res = client
        .post("https://u.gg/api")
        .json(&request_body)
        .send()
        .await;
    match res {
        Ok(yay) => {
            let yay = yay.json::<structs::PlayerSuggestions>().await;
            match yay {
                Ok(json) => Ok(json),
                Err(err) => Err(err),
            }
        }
        Err(boo) => Err(boo),
    }
}

fn remove_whitespace(s: &mut String) {
    s.retain(|c| !c.is_whitespace());
}
