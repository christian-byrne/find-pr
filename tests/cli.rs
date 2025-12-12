use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::TempDir;

struct RepoFixture {
    dir: TempDir,
}

impl RepoFixture {
    fn new() -> Self {
        let dir = TempDir::new().expect("temp dir");
        let fixture = Self { dir };
        fixture.run_git(&["init"]);
        fixture.run_git(&["config", "user.name", "Integration Tester"]);
        fixture.run_git(&["config", "user.email", "tester@example.com"]);
        fixture.write_file("README.md", "seed repo");
        fixture.run_git(&["add", "README.md"]);
        fixture.run_git(&["commit", "-m", "chore: init"]);
        fixture.run_git(&["branch", "-M", "main"]);
        fixture.run_git(&[
            "remote",
            "add",
            "origin",
            "git@github.com:example/find-pr-semantic-search.git",
        ]);
        fixture
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn create_pr(&self, number: u32, branch: &str, file: &str, contents: &str) {
        self.run_git(&["checkout", "-b", branch]);
        self.write_file(file, contents);
        self.run_git(&["add", "."]);
        self.run_git(&["commit", "-m", &format!("feat: {branch}")]);
        self.run_git(&["checkout", "main"]);
        self.run_git(&[
            "merge",
            "--no-ff",
            branch,
            "-m",
            &format!("Merge pull request #{} from example/{}", number, branch),
        ]);
    }

    fn run_git(&self, args: &[&str]) {
        self.run_git_env(args, &[]);
    }

    fn run_git_env(&self, args: &[&str], env: &[(&str, &str)]) {
        let mut cmd = StdCommand::new("git");
        cmd.args(args).current_dir(self.path());
        for (key, value) in env {
            cmd.env(key, value);
        }
        let status = cmd.status().expect("git command");
        assert!(status.success(), "git {:?} failed", args);
    }

    fn write_file<P: AsRef<Path>>(&self, relative: P, contents: &str) {
        let path = self.path().join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("dir");
        }
        let mut file = fs::File::create(path).expect("file");
        file.write_all(contents.as_bytes()).expect("write");
    }

    fn create_pr_with_date(
        &self,
        number: u32,
        branch: &str,
        file: &str,
        contents: &str,
        iso_date: &str,
    ) {
        let date_env = [
            ("GIT_AUTHOR_DATE", iso_date),
            ("GIT_COMMITTER_DATE", iso_date),
        ];
        self.run_git(&["checkout", "-b", branch]);
        self.write_file(file, contents);
        self.run_git(&["add", "."]);
        self.run_git_env(&["commit", "-m", &format!("feat: {branch}")], &date_env);
        self.run_git(&["checkout", "main"]);
        self.run_git_env(
            &[
                "merge",
                "--no-ff",
                branch,
                "-m",
                &format!("Merge pull request #{} from example/{}", number, branch),
            ],
            &date_env,
        );
    }

    fn create_squash_pr(
        &self,
        number: u32,
        branch: &str,
        file: &str,
        contents: &str,
        iso_date: &str,
    ) {
        let date_env = [
            ("GIT_AUTHOR_DATE", iso_date),
            ("GIT_COMMITTER_DATE", iso_date),
        ];
        self.run_git(&["checkout", "-b", branch]);
        self.write_file(file, contents);
        self.run_git(&["add", "."]);
        self.run_git_env(&["commit", "-m", &format!("feat: {branch}")], &date_env);
        self.run_git(&["checkout", "main"]);
        self.run_git(&["merge", "--squash", branch]);
        self.run_git_env(
            &["commit", "-m", &format!("{} (#{})", branch, number)],
            &date_env,
        );
    }
}

#[test]
fn finds_pr_by_branch_name() {
    let repo = RepoFixture::new();
    repo.create_pr(
        42,
        "feature/payments-search",
        "src/payments.rs",
        "fn pay() {}",
    );
    repo.create_pr(
        77,
        "feature/inventory-sync",
        "src/inventory.rs",
        "fn sync() {}",
    );

    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("find-pr-semantic-search").unwrap();
    cmd.current_dir(repo.path())
        .args(["inventory", "--auto-select", "1", "--no-clipboard"]);
    cmd.assert()
        .success()
        .stdout(contains("inventory-sync"))
        .stdout(contains("pull request #77"))
        .stdout(contains("src/inventory.rs"));
}

#[test]
fn lists_candidates_when_non_interactive() {
    let repo = RepoFixture::new();
    repo.create_pr(10, "bugfix/logging", "logging.txt", "fix logging");
    repo.create_pr(11, "feature/metrics", "metrics.txt", "add metrics");

    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("find-pr-semantic-search").unwrap();
    cmd.current_dir(repo.path()).args([
        "--query",
        "",
        "--results",
        "2",
        "--no-clipboard",
        "--non-interactive",
    ]);

    cmd.assert()
        .success()
        .stdout(contains("1:"))
        .stdout(contains("2:"));
}

#[test]
fn filters_out_old_merges_by_default() {
    let repo = RepoFixture::new();
    repo.create_pr_with_date(
        15,
        "feature/legacy-credits",
        "legacy.txt",
        "legacy",
        "2018-01-01T00:00:00Z",
    );
    repo.create_pr(16, "feature/new-credits", "new.txt", "new hotness");

    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("find-pr-semantic-search").unwrap();
    cmd.current_dir(repo.path()).args([
        "--query",
        "legacy",
        "--auto-select",
        "1",
        "--no-clipboard",
    ]);

    cmd.assert()
        .failure()
        .stderr(contains("no matches for query"));

    #[allow(deprecated)]
    let mut legacy_cmd = Command::cargo_bin("find-pr-semantic-search").unwrap();
    legacy_cmd.current_dir(repo.path()).args([
        "--query",
        "legacy",
        "--auto-select",
        "1",
        "--no-clipboard",
        "--max-age-days",
        "0",
    ]);

    legacy_cmd
        .assert()
        .success()
        .stdout(contains("legacy-credits"));

    #[allow(deprecated)]
    let mut list_cmd = Command::cargo_bin("find-pr-semantic-search").unwrap();
    list_cmd.current_dir(repo.path()).args([
        "--query",
        "",
        "--non-interactive",
        "--results",
        "2",
        "--no-clipboard",
    ]);

    list_cmd
        .assert()
        .success()
        .stdout(contains("new-credits"))
        .stdout(contains("legacy-credits").not());
}

#[test]
fn finds_squash_style_prs() {
    let repo = RepoFixture::new();
    repo.create_squash_pr(
        55,
        "feature/subscription-panel",
        "ui/subscription.tsx",
        "fn subscription() {}",
        "2025-11-20T00:00:00Z",
    );
    repo.create_squash_pr(
        56,
        "feature/credits-refresh",
        "ui/credits.tsx",
        "fn credits() {}",
        "2025-11-25T00:00:00Z",
    );

    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("find-pr-semantic-search").unwrap();
    cmd.current_dir(repo.path()).args([
        "--query",
        "subscription",
        "--auto-select",
        "1",
        "--no-clipboard",
    ]);

    cmd.assert()
        .success()
        .stdout(contains("subscription-panel"))
        .stdout(contains("ui/subscription.tsx"));
}
