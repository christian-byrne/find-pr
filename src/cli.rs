use chrono::Duration;
use clap::Parser;
use dialoguer::Input;

#[derive(Debug, Parser, Clone)]
#[command(about = "Semantic finder for recent pull requests", version)]
pub struct Cli {
    /// Free-form query describing the PR (author, title, files, etc.)
    #[arg(short, long)]
    pub query: Option<String>,

    /// How many ranked candidates to show before selection
    #[arg(short = 'n', long, default_value_t = 3, value_parser = clap::value_parser!(usize))]
    pub results: usize,

    /// How many merge commits to scan from HEAD backwards
    #[arg(long, default_value_t = 400, value_parser = clap::value_parser!(usize))]
    pub max_merges: usize,

    /// Automatically pick a specific 1-based index (used for testing)
    #[arg(long, hide = true)]
    pub auto_select: Option<usize>,

    /// Only consider PRs merged in the last N days (0 to disable)
    #[arg(long, default_value_t = 31, value_parser = clap::value_parser!(u32))]
    pub max_age_days: u32,

    /// Skip clipboard writes (useful in headless environments)
    #[arg(long)]
    pub no_clipboard: bool,

    /// Disable the interactive selector and just print the ranked list
    #[arg(long)]
    pub non_interactive: bool,
}

impl Cli {
    pub fn resolve_query(&self) -> String {
        self.query
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                if self.non_interactive {
                    String::new()
                } else {
                    Input::<String>::new()
                        .with_prompt("Describe the PR (author, title, files)")
                        .interact_text()
                        .unwrap_or_default()
                }
            })
    }

    pub fn bounded_results(&self) -> usize {
        self.results.clamp(1, 10)
    }

    pub fn bounded_merges(&self) -> usize {
        self.max_merges.clamp(10, 5000)
    }

    pub fn max_age_duration(&self) -> Option<Duration> {
        if self.max_age_days == 0 {
            None
        } else {
            Some(Duration::days(self.max_age_days as i64))
        }
    }

    pub fn should_select(&self) -> bool {
        !self.non_interactive
    }
}
