use super::Graphql;
use anyhow::Error;
use chrono::{Duration, Utc};
use fehler::throws;
use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "gql/schema.docs.graphql",
    query_path = "gql/organization_repos.graphql",
    response_derives = "Serialize,Debug"
)]
struct OrgRepos;

#[throws]
pub(super) async fn all_repos(graphql: &Graphql, org: &str) -> Vec<String> {
    let org_name = format!("{}", org);
    let mut repos: Vec<String> = vec![];
    let mut after_cursor = None;

    loop {
        let res = graphql
            .query(OrgRepos)
            .execute(org_repos::Variables {
                org_name: org_name.to_owned(),
                after_cursor,
            })
            .await?;

        let response_data = res.data.expect("missing response data");
        let repos_data = if let Some(org_data) = response_data.organization {
            org_data.repositories
        } else {
            break;
        };

        if let Some(edges) = repos_data.edges {
            for edge in edges.iter() {
                if let Some(Some(name)) = edge
                    .as_ref()
                    .map(|e| e.node.as_ref().map(|n| n.name.to_owned()))
                {
                    repos.push(name);
                }
            }
        }

        if repos_data.page_info.has_next_page {
            after_cursor = repos_data.page_info.end_cursor;
        } else {
            break;
        }
    }

    repos
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "gql/schema.docs.graphql",
    query_path = "gql/count_pull_requests.graphql",
    response_derives = "Serialize,Debug"
)]
struct CountPullRequests;

/// count the number of pull requests created in the given time period for the given repository within the given GitHub organization
///
/// # Arguments
/// - `org_name` — The name of the github organization that owns the specified repository
/// - `repo_name` — The name of the repository to count pull requests for. **Note:** repository should exist within the `org_name` Github Organization
/// - `time_period` — The relevant time period to search within
#[throws]
pub(super) async fn count_pull_requests(
    graphql: &Graphql,
    org_name: &str,
    repo_name: &str,
    time_period: Duration,
) -> usize {
    // get date string to match GitHub's PR query format for `created` field
    // i.e., "2021-05-18UTC" turns into "2021-05-18"
    let date_str = chrono::NaiveDate::parse_from_str(
        &format!("{}", (Utc::now() - time_period).date())[..],
        "%FUTC",
    )
    .unwrap();

    let query_string = format!(
        r#"repo:{}/{} is:pr created:>{}"#,
        org_name, repo_name, date_str,
    );

    let response = graphql
        .query(CountPullRequests)
        .execute(count_pull_requests::Variables { query_string })
        .await?;
    let response_data = response.data.expect("missing response data");
    let count = response_data.search.issue_count;
    count as usize
}
