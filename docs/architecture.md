Architecture Snapshot
=====================

Flow
----
1. `cli` parses flags (including the `--max-age-days` window) and prompts for a query when needed.
2. `app` wires everything together: opens the repo, asks `GitService` for recent PR-like commits (merge or squash), trims anything older than the requested window, hands the rest to `SearchEngine`, then delegates to the selector / clipboard pipeline.
3. `GitService` shells out to `git` for `rev-list`, `log`, `diff`, and config queries, stitching the results into `PullRequestInfo` structs.
4. `SearchEngine` tokenizes the query, combines fuzzy matching with substring and recency bonuses, and returns sorted candidates.
5. `presentation` builds both the compact list rows and the detailed payload, which `clipboard` optionally copies via the first available system tool.

Modules
-------
- `app.rs` – orchestration, interactive selection, and high-level error messaging.
- `cli.rs` – `clap` parser plus helpers that clamp unsafe argument ranges.
- `model.rs` – shared data structures for pull-request metadata.
- `services/git.rs` – repo discovery, metadata extraction, file diffing, and remote URL normalization.
- `services/scoring.rs` – fuzzy matcher wrapper and deterministic ranking logic.
- `presentation.rs` – string formatting for candidates and full details.
- `services/clipboard.rs` – best-effort clipboard integration targeting macOS, Linux (Wayland/X11), and Windows.

Extension Points
----------------
- add semantic embeddings / vector search by storing a small cache file beside `.git` and swapping the `SearchEngine` scoring method
- layer in remote providers (GitHub, GitLab, Bitbucket) by implementing a new service that augments `PullRequestInfo`
- capture frequently used filters (author, labels, directories) and expose them as `--filter author=alice` style options without touching `GitService`
