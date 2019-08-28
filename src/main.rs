pub mod template;

use std::env;
use std::collections::HashMap;

use tokio::runtime::Runtime;

use hubcaps::{Credentials, Github, Result};
use hubcaps::issues::IssueOptions;
use hubcaps::repositories::Repository;

fn main () -> Result<()> {
    match env::var("INPUT_GITHUB-TOKEN").ok() {
        Some(token) => {
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
    let repo = env::var("INPUT_PROJECT").expect("GitHub project not provided");

    let mut runtime = Runtime::new()?;
    let repo = github.repo(org, repo);

    runtime.block_on(
        match build_issue(&repo).ok() {
            Some(issue) => repo.issues().create(&issue),
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
            let mut issue = template::process(repo, &template, subs);
            issue.title = title;
            issue.assignee = assignee;
            issue.milestone = milestone;

            issue
        },
        None => {
            let body = env::var("INPUT_BODY").ok();
            let labels: Vec<String> = vec!();
            
            IssueOptions::new(title, body, assignee, milestone, labels)
        }
    };

    Ok(issue)
}

fn substitutions(subs: &mut HashMap<String, String>) {
    for (key, value) in env::vars() {
        match key.starts_with("INPUT_SUBSTITUTION") {
            true => {
                let mut match_parts: Vec<&str> = key.as_str().split('_').collect();

                match match_parts.pop() {
                    Some(target) => {
                        subs.insert(target.to_string(), value);
                    },
                    None => ()
                }
            },
            false => ()
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