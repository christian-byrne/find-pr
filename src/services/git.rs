use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::model::PullRequestInfo;

pub struct GitService {
    root: PathBuf,
    remote_http_url: Option<String>,
}

impl GitService {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let root = run_git_in(path.as_ref(), ["rev-parse", "--show-toplevel"])?;
        let root = PathBuf::from(root.trim());
        let remote_http_url = run_git(&root, ["config", "--get", "remote.origin.url"])
            .ok()
            .and_then(|url| normalize_remote(url.trim()));

        Ok(Self {
            root,
            remote_http_url,
        })
    }

    pub fn recent_pr_merges(&self, max_items: usize) -> Result<Vec<PullRequestInfo>> {
        let max_arg = max_items.to_string();
        let revs = run_git(
            &self.root,
            [
                "rev-list",
                "--merges",
                "--max-count",
                max_arg.as_str(),
                "HEAD",
            ],
        )
        .context("unable to walk merge history")?;
        let mut prs = Vec::new();
        for line in revs.lines() {
            let commit_id = line.trim();
            if commit_id.is_empty() {
                continue;
            }
            let pr = self.commit_info(commit_id)?;
            prs.push(pr);
        }
        Ok(prs)
    }

    fn commit_info(&self, commit: &str) -> Result<PullRequestInfo> {
        const FORMAT: &str = "%H%x00%an%x00%ae%x00%cI%x00%P%x00%B%x00";
        let format_arg = format!("--format={FORMAT}");
        let raw = run_git(&self.root, ["log", "-1", format_arg.as_str(), commit])?;
        let mut parts = raw.split('\0');
        let commit_id = parts
            .next()
            .ok_or_else(|| anyhow!("commit id missing"))?
            .trim()
            .to_string();
        let author = parts
            .next()
            .ok_or_else(|| anyhow!("author missing"))?
            .trim()
            .to_string();
        let email = parts
            .next()
            .ok_or_else(|| anyhow!("email missing"))?
            .trim()
            .to_string();
        let date_raw = parts.next().ok_or_else(|| anyhow!("date missing"))?.trim();
        let committed_at = DateTime::parse_from_rfc3339(date_raw)
            .map(|dt| dt.with_timezone(&Utc))
            .context("invalid commit timestamp")?;
        let parents_raw = parts.next().unwrap_or_default().to_string();
        let first_parent = parents_raw.split_whitespace().next().map(|s| s.to_string());
        let message = parts
            .next()
            .unwrap_or_default()
            .replace('\r', "")
            .trim()
            .to_string();

        let files = self.changed_files(commit, first_parent.as_deref())?;

        Ok(PullRequestInfo {
            commit_id,
            title: message.lines().next().unwrap_or("").to_string(),
            author,
            author_email: email,
            committed_at,
            pr_number: parse_pr_number(&message),
            source_branch: parse_source_branch(&message),
            files,
            repo_http_url: self.remote_http_url.clone(),
        })
    }

    fn changed_files(&self, commit: &str, parent: Option<&str>) -> Result<Vec<String>> {
        let data = if let Some(parent_oid) = parent {
            run_git(&self.root, ["diff", "--name-only", parent_oid, commit])?
        } else {
            run_git(
                &self.root,
                ["show", "--name-only", "--pretty=format:", commit],
            )?
        };

        let mut files = Vec::new();
        for line in data.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            files.push(trimmed.to_string());
            if files.len() >= 40 {
                break;
            }
        }
        Ok(files)
    }
}

fn run_git<I, S>(root: &Path, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .context("git command failed")?;
    if !output.status.success() {
        return Err(anyhow!(
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn run_git_in<I, S>(path: &Path, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("git")
        .args(args)
        .current_dir(path)
        .output()
        .context("git command failed")?;
    if !output.status.success() {
        return Err(anyhow!(
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn normalize_remote(input: &str) -> Option<String> {
    if input.starts_with("http") {
        let without_git = input.trim_end_matches(".git");
        let cleaned = without_git.trim_end_matches('/');
        return Some(cleaned.to_string());
    }

    if input.starts_with("git@") {
        let without_suffix = input.trim_end_matches(".git");
        let parts: Vec<&str> = without_suffix.split(':').collect();
        if parts.len() == 2 {
            return Some(format!(
                "https://{}/{}",
                parts[0].trim_start_matches("git@"),
                parts[1]
            ));
        }
    }

    if input.starts_with("ssh://") {
        let without_suffix = input.trim_end_matches(".git");
        if let Some(rest) = without_suffix.strip_prefix("ssh://git@") {
            return Some(format!("https://{}", rest));
        }
    }

    None
}

fn parse_pr_number(message: &str) -> Option<u64> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"#(\d+)").unwrap());
    RE.captures(message)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

fn parse_source_branch(message: &str) -> Option<String> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"from ([^\s]+)").unwrap());
    RE.captures(message).and_then(|caps| caps.get(1)).map(|m| {
        m.as_str()
            .split('/')
            .last()
            .unwrap_or(m.as_str())
            .to_string()
    })
}
