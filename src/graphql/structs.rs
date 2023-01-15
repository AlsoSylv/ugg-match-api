use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/player_suggestion_query.graphql",
    response_derives = "Debug, Serialize",
    variables_derives = "Debug"
)]
pub struct PlayerInfoSuggestions;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/match_query.graphql",
    response_derives = "Debug, Serialize",
    variables_derives = "Debug"
)]
pub struct FetchMatchSummaries;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/update_profile_query.graphql",
    response_derives = "Debug, Serialize",
    variables_derives = "Debug"
)]
pub struct UpdatePlayerProfile;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/fetch_profile_rank_queries.graphql",
    response_derives = "Debug, Serialize",
    variables_derives = "Debug"
)]
pub struct FetchProfileRanks;

pub type UnixTimestamp = i64;
