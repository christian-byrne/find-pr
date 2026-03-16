# ⛔ DEPRECATED

> **This repository has been archived and absorbed into the [ticket-to-pr-pipeline](https://github.com/christian-byrne/ticket-to-pr-pipeline) monorepo.**
>
> **New location:** [`tools/find-pr/`](https://github.com/christian-byrne/ticket-to-pr-pipeline/tree/main/tools/find-pr)
>
> This repo will receive no further updates. All new development happens in the monorepo.

---

find-pr-semantic-search
=======================

Fast Rust CLI for ranking and previewing recently merged pull requests in the git repository you are currently inside. Enter what you remember (author, branch, file names, loose keywords) and get back a short ranked list, pick one, and have the full context copied to your clipboard.

Features
--------
- walks PR-like commits (merge or squash) locally via `git` (no network round-trips) and caches nothing
- hybrid fuzzy/sub-string scoring across authors, branch names, commit titles, file paths, and PR numbers
- recency boosting so the freshest matches surface first even with thin queries
- interactive picker (default) plus automation flags for scripting and tests
- detailed payload includes URL, merge commit, author, files touched, and a ready-to-run `git fetch` command
- clipboard helper works with `pbcopy`, `wl-copy`, `xclip`, `xsel`, or `clip.exe`, and can be disabled

Quickstart
----------
```
cargo install --path .
```
Run inside any git repository:
```
find-pr-semantic-search "alice payment checkout"
```
You will see the top three matches; use the arrow keys or number keys to pick one. The formatted block is printed and copied to your clipboard.

Useful Flags
------------
- `QUERY` positional argument or `--query TEXT` free-form description (leave empty to be prompted)
- `--results N` number of ranked candidates to show (1-10, default 8)
- `--max-merges N` how many recent merge commits to scan (10-5000, default 400)
- `--max-age-days N` only consider merges from the last N days (default 31, pass 0 to disable)
- `--non-interactive` print the ranked list without prompting (great for scripts)
- `--auto-select INDEX` automatically choose the INDEX-th candidate (1-based)
- `--no-clipboard` skip copying (CI, SSH boxes)

Testing
-------
```
cargo test
```
Integration tests spin up throwaway git repositories with fake pull requests to exercise the ranking and formatting pipeline end to end.

Design Notes
------------
- `GitService` shells out to the locally installed `git` binary for reliability and speed and normalizes remote URLs into HTTPS form.
- `SearchEngine` performs tokenized fuzzy matching plus substring bonuses and a recency decay so the scoring is deterministic and extendable.
- `presentation` handles both the picker labels and the final clipboard payload so future format tweaks remain localized.
- `clipboard` attempts the usual OS utilities in order and fails fast with a helpful error when none are installed.

See `docs/architecture.md` for module boundaries and extension points.
