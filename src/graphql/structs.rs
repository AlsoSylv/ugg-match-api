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

pub type UnixTimestamp = i64;
