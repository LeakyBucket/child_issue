pub mod template;

use std::env;
use std::collections::HashMap;

use tokio::runtime::Runtime;

use hubcaps::{Credentials, Github, Result};
use hubcaps::issues::IssueOptions;
use hubcaps::repositories::Repository;

fn main () -> Result<()> {
    println!("Starting!");

    match env::var("INPUT_GITHUB_TOKEN").ok() {
        Some(token) => {
            println!("Found Auth Token");

            let github = Github::new(
                "ChildIssueAction",
                Credentials::Token(token)
            )?;

            create_issue(github)
        },
        _ => Err("No GitHub Token provided".into()),
    }
}

fn create_issue(github: Github) -> Result<()> {
    let org = env::var("INPUT_ORG").expect("GitHub org not provided");
    let repo = env::var("INPUT_REPO").expect("GitHub repo not provided");

    let mut runtime = Runtime::new()?;
    let repo = github.repo(org, repo);

    println!("About to build issue");

    runtime.block_on(
        match build_issue(&repo).ok() {
            Some(issue) => {
                println!("Creating issue");
                repo.issues().create(&issue)
            },
            None => panic!("Failed to build issue")
        }
    )?;

    Ok(())
}

fn build_issue(repo: &Repository) -> Result<IssueOptions> {
    let mut subs = HashMap::new();
    substitutions(&mut subs);
    let title = env::var("INPUT_TITLE").expect("Issue Title is Required!");
    let assignee = env::var("INPUT_ASSIGNEE").ok();
    let milestone = match env::var("INPUT_MILESTONE").ok() {
        Some(milestone) => milestone.parse::<u64>().ok(),
        None => None
    };

    let issue = match env::var("INPUT_TEMPLATE").ok() {
        Some(template) => {
            println!("Template specified");
            let mut issue = template::process(repo, &template, subs);
            issue.title = title;
            issue.assignee = assignee;
            issue.milestone = milestone;

            println!("Template processed");

            issue
        },
        None => {
            println!("No template specified");
            let body = env::var("INPUT_BODY").ok();
            let labels: Vec<String> = vec!();
            
            IssueOptions::new(title, body, assignee, milestone, labels)
        }
    };

    Ok(issue)
}

fn substitutions(subs: &mut HashMap<String, String>) {
    for (key, value) in env::vars() {
        if key.starts_with("INPUT_SUBSTITUTION") {
            subs.insert(key.trim_start_matches("INPUT_SUBSTITUTION_").to_string(), value);
        }
    }

    ()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_env() {
        env::set_var("INPUT_SUBSTITUTION_TARGET", "value");
    }

    #[test]
    fn substitution_test() {
        setup_env();
        let mut subs = HashMap::new();

        substitutions(&mut subs);

        assert_eq!(subs.get("TARGET"), Some(&"value".to_string()));
    }
}