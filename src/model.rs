use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct PullRequestInfo {
    pub commit_id: String,
    pub title: String,
    pub author: String,
    pub author_email: String,
    pub committed_at: DateTime<Utc>,
    pub pr_number: Option<u64>,
    pub source_branch: Option<String>,
    pub files: Vec<String>,
    pub repo_http_url: Option<String>,
}

impl PullRequestInfo {
    pub fn pull_request_url(&self) -> Option<String> {
        match (self.repo_http_url.as_ref(), self.pr_number) {
            (Some(base), Some(number)) => Some(format!("{}/pull/{}", base, number)),
            _ => None,
        }
    }

    pub fn fetch_command(&self) -> Option<String> {
        self.pr_number.map(|number| {
            format!("git fetch origin pull/{number}/head:pr-{number} && git checkout pr-{number}")
        })
    }
}
