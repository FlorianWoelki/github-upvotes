use std::collections::HashMap;

use dotenv::dotenv;
use futures::{future::BoxFuture, stream::FuturesUnordered, StreamExt};
use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION, USER_AGENT};
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
    content: String, // could be '+1'
    user: User,
}

fn get_issues_wrapper(url: Option<String>) -> BoxFuture<'static, Vec<Issue>> {
    Box::pin(get_issues(url))
}

fn construct_new_url(headers: &HeaderMap) -> Option<String> {
    headers.get("link").and_then(|link_header| {
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

    let response = match response {
        Ok(res) if res.status().is_success() => res,
        _ => return Vec::new(),
    };
    // TODO: Maybe remove the clone here.
    let response_headers = response.headers().clone();

    let issues = response
        .json::<Vec<Issue>>()
        .await
        .expect("something went wrong while parsing")
        .into_iter()
        .filter(|issue| issue.pull_request.is_none())
        .collect::<Vec<_>>();

    if let Some(new_url) = construct_new_url(&response_headers) {
        let more_issues = get_issues_wrapper(Some(new_url)).await;
        return issues.into_iter().chain(more_issues).collect();
    }

    issues
}

async fn get_issue_reactions(issue_id: usize) -> Vec<IssueReaction> {
    let token = std::env::var("GITHUB_PAT").expect("GITHUB_PAT must be set");
    let request_url = format!(
        "https://api.github.com/repos/{owner}/{repo}/issues/{issue_id}/reactions",
        owner = "FlorianWoelki",
        repo = "obsidian-iconize",
        issue_id = issue_id
    );
    let client = reqwest::Client::new();
    let response = client
        .get(&request_url)
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(USER_AGENT, "rust web-api")
        .header(ACCEPT, "application/vnd.github+json")
        .send()
        .await;
    let response = match response {
        Ok(res) if res.status().is_success() => res,
        _ => return Vec::new(),
    };
    let reactions = response
        .json::<Vec<IssueReaction>>()
        .await
        .expect("something went wrong while parsing");
    reactions
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let issues = get_issues(None).await;
    let mut futures = FuturesUnordered::new();

    for issue in &issues {
        let issue_number = issue.number;
        futures.push(async move {
            let reactions = get_issue_reactions(issue_number).await;
            (issue_number, reactions)
        });
    }

    let mut results: HashMap<usize, usize> = HashMap::new();
    while let Some((number, reactions)) = futures.next().await {
        let reactions_count = reactions.iter().filter(|r| r.content == "+1").count();
        results
            .entry(number)
            .and_modify(|e| *e += reactions_count)
            .or_insert(reactions_count);
    }

    let mut sorted_result: Vec<_> = results
        .into_iter()
        .filter(|&(_, count)| count > 0)
        .collect();
    sorted_result.sort_by(|a, b| b.1.cmp(&a.1));

    let now = chrono::Utc::now();
    println!("*Updated on {} (UTC)*", now.to_rfc2822());

    for (index, (issue_number, upvotes)) in sorted_result.iter().enumerate() {
        println!("{}. #{} ({} 👍)", index + 1, issue_number, upvotes)
    }
}
