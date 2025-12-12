use crate::model::PullRequestInfo;

pub fn candidate_line(pr: &PullRequestInfo, score: i64) -> String {
    let date = pr.committed_at.format("%Y-%m-%d %H:%M UTC");
    let title = pr.title.trim();
    let who = &pr.author;
    let pr_part = pr
        .pr_number
        .map(|n| format!("PR #{n}"))
        .unwrap_or_else(|| "merge".to_string());
    format!("[{pr_part}] {title} — {who} — {date} (score {score})")
}

pub fn detailed_output(pr: &PullRequestInfo) -> String {
    let date = pr.committed_at.format("%Y-%m-%d %H:%M:%S UTC");
    let files = if pr.files.is_empty() {
        "  (no tracked files)".to_string()
    } else {
        pr.files
            .iter()
            .map(|f| format!("  - {f}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let mut lines = vec![
        format!("Title     : {}", pr.title.trim()),
        format!("Author    : {} <{}>", pr.author, pr.author_email),
        format!("Merged    : {date}"),
        format!("Commit    : {}", pr.commit_id),
        format!(
            "Branch    : {}",
            pr.source_branch.clone().unwrap_or_else(|| "unknown".into())
        ),
    ];

    if let Some(url) = pr.pull_request_url() {
        lines.push(format!("URL       : {url}"));
    }
    if let Some(cmd) = pr.fetch_command() {
        lines.push(format!("Fetch cmd : {cmd}"));
    }

    lines.push("Files     :".into());
    lines.push(files);

    lines.join("\n")
}
