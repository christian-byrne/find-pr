use chrono::{DateTime, Utc};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::time::SystemTime;

use crate::model::PullRequestInfo;

pub struct SearchEngine {
    matcher: SkimMatcherV2,
}

pub struct RankedResult<'a> {
    pub pr: &'a PullRequestInfo,
    pub score: i64,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn rank<'a>(&'a self, query: &str, items: &'a [PullRequestInfo]) -> Vec<RankedResult<'a>> {
        let tokens: Vec<String> = if query.trim().is_empty() {
            Vec::new()
        } else {
            query.split_whitespace().map(|t| t.to_lowercase()).collect()
        };
        let now: DateTime<Utc> = SystemTime::now().into();

        let mut ranked = Vec::new();
        for pr in items {
            if let Some(score) = self.score_item(pr, &tokens, now) {
                ranked.push(RankedResult { pr, score });
            }
        }
        ranked.sort_by(|a, b| b.score.cmp(&a.score));
        ranked
    }

    fn score_item(
        &self,
        pr: &PullRequestInfo,
        tokens: &[String],
        now: DateTime<Utc>,
    ) -> Option<i64> {
        let mut total = 0i64;
        for token in tokens {
            let token_score = self.best_token_score(pr, token)?;
            total += token_score;
        }

        // Recency bonus: prefer fresher merges within ~30 days
        let age_hours = (now - pr.committed_at).num_hours().max(0);
        let recency_bonus = (720 - age_hours).max(0) as i64; // fades after 30 days
        total += recency_bonus;

        Some(total)
    }

    fn best_token_score(&self, pr: &PullRequestInfo, token: &str) -> Option<i64> {
        if token.is_empty() {
            return Some(0);
        }

        let views = [
            pr.title.as_str(),
            pr.author.as_str(),
            pr.source_branch.as_deref().unwrap_or(""),
            pr.commit_id.as_str(),
        ];

        let mut best: Option<i64> = None;
        for text in views {
            if text.is_empty() {
                continue;
            }
            if let Some(score) = self.matcher.fuzzy_match(text, token) {
                best = Some(best.map_or(score, |current| current.max(score)));
            }
            if text.to_lowercase().contains(token) {
                best = Some(best.map_or(150, |current| current.max(150)));
            }
        }

        for file in &pr.files {
            if file.to_lowercase().contains(token) {
                best = Some(best.map_or(180, |current| current.max(180)));
                break;
            }
        }

        if token.chars().all(|c| c.is_ascii_digit()) {
            if let Some(number) = pr.pr_number.map(|n| n.to_string()) {
                if number.contains(token) {
                    best = Some(best.map_or(220, |current| current.max(220)));
                }
            }
        }

        best
    }
}
