use std::env;

use tokio::runtime::Runtime;

use hubcaps::issues::IssueOptions;
use hubcaps::{Credentials, Github, Result};

use hyper::client::connect::Connect;

fn main () -> Result<()> {
    match env::var("INPUT_GITHUB-TOKEN").ok() {
        Some(token) => {
            let github = Github::new(
                "ChildIssueAction",
                Credentials::Token(token)
            );

            create_issue(github)
        },
        _ => Err("No GitHub Token provided".into()),
    }
}

fn create_issue<C: Clone + Connect + 'static>(github: Github<C>) -> Result<()> {
    let org = env::var("INPUT_ORG").expect("GitHub org not provided");
    let repo = env::var("INPUT_PROJECT").expect("GitHub project not provided");

    let issue = build_issue().expect("Failed to create Issue metadata");

    let mut runtime = Runtime::new()?;

    runtime.block_on(
        github.repo(org, repo)
            .issues()
            .create(&issue)
    )?;

    Ok(())
}

fn build_issue() -> Result<IssueOptions> {
    let title = env::var("INPUT_TITLE").expect("Issue Title not set");
    let body = env::var("INPUT_BODY").ok();
    let assignee = env::var("INPUT_ASSIGNEE").ok();
    let milestone = match env::var("INPUT_MILESTONE").ok() {
        Some(milestone) => milestone.parse::<u64>().ok(),
        None => None
    };
    let labels: Vec<String> = vec!();

    Ok(IssueOptions::new(title, body, assignee, milestone, labels))
}