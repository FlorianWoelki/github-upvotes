use reqwest::header::USER_AGENT;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct User {
    login: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Issue {
    content: String,
    user: User,
}

async fn get_issue_reactions() -> Vec<Issue> {
    let request_url = format!(
        "https://api.github.com/repos/{owner}/{repo}/issues/76/reactions",
        owner = "FlorianWoelki",
        repo = "obsidian-iconize"
    );
    let client = reqwest::Client::new();
    let response = client
        .get(&request_url)
        .header(USER_AGENT, "rust web-api")
        .send()
        .await
        .expect("something went wrong while fetching");
    let resolved_response = response
        .json::<Vec<Issue>>()
        .await
        .expect("something went wrong while parsing");
    resolved_response
}

fn main() {
    let issues = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(get_issue_reactions());
    println!("{:?}", issues);
}
