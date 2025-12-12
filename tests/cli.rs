use assert_cmd::Command;
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
        let status = StdCommand::new("git")
            .args(args)
            .current_dir(self.path())
            .status()
            .expect("git command");
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
    cmd.current_dir(repo.path()).args([
        "--query",
        "inventory",
        "--auto-select",
        "1",
        "--no-clipboard",
    ]);
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
