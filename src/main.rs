use dotenv::dotenv;
use futures::future::BoxFuture;
use reqwest::{
    header::{ACCEPT, AUTHORIZATION, USER_AGENT},
    Response,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct User {
    login: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct PullRequest {} // Empty because we don't use it.

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Issue {
    number: usize,
    title: String,
    pull_request: Option<PullRequest>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct IssueReaction {
    content: String,
    user: User,
}

fn get_issues_wrapper(url: Option<String>) -> BoxFuture<'static, Vec<Issue>> {
    Box::pin(get_issues(url))
}

fn construct_new_url(response: &Response) -> Option<String> {
    response.headers().get("link").and_then(|link_header| {
        link_header.to_str().ok().and_then(|link_value| {
            link_value.contains("rel=\"next\"").then(|| {
                link_value
                    .split(';')
                    .collect::<Vec<&str>>()
                    .get(0)
                    .expect("could not find new url with page")
                    .to_string()
            })
        })
    })
}

async fn get_issues(url: Option<String>) -> Vec<Issue> {
    let token = std::env::var("GITHUB_PAT").expect("GITHUB_PAT must be set");
    let request_url = url.unwrap_or(format!(
        "https://api.github.com/repos/{owner}/{repo}/issues?state=open&page=1&per_page=100",
        owner = "FlorianWoelki",
        repo = "obsidian-iconize",
    ));
    let client = reqwest::Client::new();
    let response = client
        .get(&request_url)
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(USER_AGENT, "FlorianWoelki")
        .header(ACCEPT, "application/vnd.github+json")
        .send()
        .await;
    match response {
        Ok(response) => {
            if response.status() != 200 {
                return Vec::new();
            }

            let new_request_url = construct_new_url(&response);

            let resolved_response = response
                .json::<Vec<Issue>>()
                .await
                .expect("something went wrong while parsing")
                .into_iter()
                .filter(|issue| issue.pull_request.is_none())
                .collect::<Vec<_>>();

            match new_request_url {
                Some(request_url) => {
                    let issues_next_page = get_issues_wrapper(Some(request_url)).await;

                    return resolved_response
                        .into_iter()
                        .chain(issues_next_page)
                        .collect();
                }
                None => {
                    return resolved_response;
                }
            }
        }
        Err(_) => {
            return Vec::new();
        }
    }
}

async fn get_issue_reactions(issue_id: usize) -> Vec<IssueReaction> {
    let request_url = format!(
        "https://api.github.com/repos/{owner}/{repo}/issues/{issue_id}/reactions",
        owner = "FlorianWoelki",
        repo = "obsidian-iconize",
        issue_id = issue_id
    );
    let client = reqwest::Client::new();
    let response = client
        .get(&request_url)
        .header(USER_AGENT, "rust web-api")
        .send()
        .await
        .expect("something went wrong while fetching");
    let resolved_response = response
        .json::<Vec<IssueReaction>>()
        .await
        .expect("something went wrong while parsing");
    resolved_response
}

fn main() {
    dotenv().ok();

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let issues = runtime.block_on(get_issues(None));

    for issue in &issues {
        println!("Issue: {}", issue.title)
    }

    println!("Amount of issues: {}", issues.len())
}
