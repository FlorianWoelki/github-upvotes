use futures::future::BoxFuture;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct User {
    login: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Issue {
    number: usize,
    title: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct IssueReaction {
    content: String,
    user: User,
}

fn get_issues_wrapper(page: Option<usize>) -> BoxFuture<'static, Vec<Issue>> {
    Box::pin(get_issues(page))
}

async fn get_issues(page: Option<usize>) -> Vec<Issue> {
    let page = page.unwrap_or(1);
    let request_url = format!(
        "https://api.github.com/repos/{owner}/{repo}/issues?state=open&page={page}",
        owner = "FlorianWoelki",
        repo = "obsidian-iconize",
        page = page,
    );
    let client = reqwest::Client::new();
    let response = client
        .get(&request_url)
        .header(USER_AGENT, "FlorianWoelki")
        .header(ACCEPT, "application/vnd.github+json")
        .send()
        .await;
    match response {
        Ok(response) => {
            println!("{:?}", response);
            let resolved_response = response
                .json::<Vec<Issue>>()
                .await
                .expect("something went wrong while parsing");

            let issues_next_page = get_issues_wrapper(Some(page + 1)).await;

            return resolved_response
                .into_iter()
                .chain(issues_next_page)
                .collect();
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
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let issues = runtime.block_on(get_issues(None));

    for issue in &issues {
        println!("Issue: {}", issue.title)
    }

    println!("Amount of issues: {}", issues.len())
}
