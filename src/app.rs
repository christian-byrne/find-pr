use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use dialoguer::{theme::ColorfulTheme, Select};

use crate::{
    cli::Cli,
    presentation,
    services::{clipboard, git::GitService, scoring::SearchEngine},
};

pub fn run(args: Cli) -> Result<()> {
    let repo = GitService::open(".").context("failed to open git repository")?;
    let mut merges = repo
        .recent_pr_merges(args.bounded_merges())
        .context("unable to scan merge commits")?;
    if merges.is_empty() {
        return Err(anyhow!("no pull-request commits found"));
    }

    if let Some(window) = args.max_age_duration() {
        let cutoff = Utc::now() - window;
        merges.retain(|pr| pr.committed_at >= cutoff);
        if merges.is_empty() {
            return Err(anyhow!(
                "no merge commits found within the last {} days (adjust --max-age-days)",
                args.max_age_days
            ));
        }
    }

    let engine = SearchEngine::new();
    let query = args.resolve_query();
    let ranked = engine.rank(&query, &merges);
    if ranked.is_empty() {
        return Err(anyhow!("no matches for query"));
    }

    let mut top: Vec<_> = ranked.into_iter().take(args.bounded_results()).collect();
    if top.is_empty() {
        return Err(anyhow!("no candidates to display"));
    }

    if args.non_interactive && args.auto_select.is_none() {
        for (idx, candidate) in top.iter().enumerate() {
            println!(
                "{}: {}",
                idx + 1,
                presentation::candidate_line(candidate.pr, candidate.score)
            );
        }
        return Ok(());
    }

    let selected_index = if let Some(forced) = args.auto_select {
        let idx = forced
            .checked_sub(1)
            .ok_or_else(|| anyhow!("selection must be >= 1"))?;
        if idx >= top.len() {
            return Err(anyhow!("selection {forced} is outside the candidate list"));
        }
        idx
    } else if args.should_select() {
        let items: Vec<String> = top
            .iter()
            .map(|candidate| presentation::candidate_line(candidate.pr, candidate.score))
            .collect();
        Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Pick a PR to copy")
            .items(&items)
            .default(0)
            .interact()?
    } else {
        0
    };

    let chosen = top.swap_remove(selected_index);
    let detail = presentation::detailed_output(chosen.pr);
    println!("{detail}\n");
    clipboard::copy_to_clipboard(&detail, !args.no_clipboard)?;
    eprintln!("Copied details to clipboard");

    Ok(())
}
