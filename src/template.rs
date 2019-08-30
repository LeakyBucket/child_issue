use hubcaps::content::File;
use hubcaps::errors::Error;
use hubcaps::issues::IssueOptions;
use hubcaps::repositories::Repository;
use std::collections::HashMap;
use std::env;
use tokio::runtime::Runtime;

fn populate_metadata(issue: &mut IssueOptions, template: &mut str) {
    let mut lines = template.lines();

    lines.next(); // Swallow the leading `----`
    lines.next(); // Swallow the name
    lines.next(); // Swallow the about meta

    // template title
    match lines.next() {
        Some(line) => {
            match line.split(':').next_back() {
                Some(title) => issue.title = title.trim_start().to_string(),
                None => ()
            }
        },
        None => ()
    };

    // template labels
    match lines.next() {
        Some(line) => {
            match line.split(':').next_back() {
                Some(labels) => {
                    let trimmed = labels.trim_start();

                    if trimmed != "''" {
                        issue.labels = labels.trim_start()
                                             .trim_matches('\'')
                                             .split(',')
                                             .map(|tag| tag.to_string())
                                             .collect::<Vec<String>>()
                    }
                },
                None => ()
            }
        },
        None => ()
    };

    // template assignees
    match lines.next() {
        Some(line) => {
            match line.split(':').next_back() {
                Some(assignees) => issue.assignee = Some(assignees.trim_start().to_string()),
                None => ()
            }
        },
        None => ()
    };

    lines.next(); // Swallow the blank line after the metadata
    lines.next(); // Swallow the ---- line separating the metadata from the body
    lines.next(); // Swallow the blank line at the top of the body

    issue.body = Some(lines.collect::<Vec<&str>>().join("\n"));
}

fn fetch(repo: &Repository, name: &str) -> Result<File, Error> {
    let mut runtime = Runtime::new()?;

    runtime.block_on(
        repo.content()
            .file(name)
    )
}

pub fn process(repo: &Repository, template_name: &str, subs: HashMap<String, String>) -> IssueOptions {
    let title = match env::var("INPUT_TITLE").ok() {
        Some(title) => title,
        None => "".to_string()
    };
    let labels: Vec<String> = vec!();
    let body: Option<String> = None;
    let assignee: Option<String> = None;
    let milestone: Option<u64> = None;

    let mut issue = IssueOptions::new(title, body, assignee, milestone, labels);

    match fetch(repo, template_name) {
        Ok(file) => {
            let template = std::str::from_utf8(file.content.as_ref()).expect("Non UTF8 contents in template");
            populate_metadata(&mut issue, &mut template.to_string().clone());
            match issue.body {
                Some(mut content) => {
                    issue.body = Some(substitute(&mut content, subs));
                    ()
                },
                None => ()
            }
        },
        Err(error) => {
            dbg!(error);
            panic!("Template could not be fetched!");
        }
    };

    issue
}

fn substitute(body: &mut str, subs: HashMap<String, String>) -> String {
    let mut subbed = body.to_string();

    for (target, replacement) in subs.iter() {
        let wrapped = format!("{{{{ {} }}}}", target);

        subbed = subbed.replace(&wrapped, replacement);
    }

    subbed
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEMPLATE_NAME: &str = ".github/ISSUE_TEMPLATE/testing-template.md";

    fn template() -> String {
        let t = r#"---
        name: Template
        about: This is a template
        title: Test title
        labels: new,bug
        assignees: ''

        ----

        # Sample Template
        ## Subs
        {{ first }}
        {{ second }}
        ## Other Stuff
        Hi Mom!
        "#;

        t.to_string()
    }

    fn raw_body() -> String {
        r#"# Sample Template
        ## Subs
        {{ first }}
        {{ second }}
        ## Other Stuff
        Hi Mom!
        "#.to_string()
    }

    fn subbed_body() -> String {
        let t = r#"# Sample Template
        ## Subs
        FIRST
        SECOND
        ## Other Stuff
        Hi Mom!
        "#;

        t.to_string()
    }

    fn partial_sub() -> String {
        let t = r#"# Sample Template
        ## Subs
        FIRST
        {{ second }}
        ## Other Stuff
        Hi Mom!
        "#;

        t.to_string()
    }

    #[test]
    fn full_template_substitution() {
        let mut subs = HashMap::new();

        subs.insert("{{ first }}".to_string(), "FIRST".to_string());
        subs.insert("{{ second }}".to_string(), "SECOND".to_string());

        assert_eq!(subbed_body(), substitute(&mut raw_body(), subs));
    }

    #[test]
    fn sub_without_matches() {
        assert_eq!(raw_body(), substitute(&mut raw_body(), HashMap::new()));
    }

    #[test]
    fn partial_substitution() {
        let mut subs = HashMap::new();

        subs.insert("{{ first }}".to_string(), "FIRST".to_string());

        assert_eq!(partial_sub(), substitute(&mut raw_body(), subs));
    }

    #[test]
    fn template_decomposition() {
        let title = "";
        let labels: Vec<String> = vec!();
        let body: Option<String> = None;
        let assignee: Option<String> = None;
        let milestone: Option<u64> = None;
        let mut issue = IssueOptions::new(title, body, assignee, milestone, labels);

        populate_metadata(&mut issue, &mut template());

        assert_eq!("Test title", issue.title);
        assert_eq!(vec!["new", "bug"], issue.labels);
        assert_eq!(Some(raw_body()), issue.body);
    }

    #[test]
    fn template_fetch() {
        let token = match std::env::var("AUTH_TOKEN").ok() {
            Some(token) => hubcaps::Credentials::Token(token),
            _ => panic!("No Auth Token Found")
        };

        let github = hubcaps::Github::new(
            "ChildIssueActionTest",
            token
        );
        
        let repo = match github {
            Ok(github) => github.repo("LeakyBucket", "child_issue"),
            Err(_) => panic!("Couldn't build Github struct")
        };

        let file = match fetch(&repo, TEMPLATE_NAME) {
            Ok(file) => file,
            Err(error) => {
                dbg!(error);
                panic!("Download failed")
            }
        };

        let contents = match std::str::from_utf8(file.content.as_ref()) {
            Ok(content) => content,
            Err(_) => panic!("Could not convert content")
        };

        assert!(contents.chars().count() > 0);
    }
}