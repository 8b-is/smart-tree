# GitHub repository sync

`st sync OWNER --dir BASE` syncs repositories into `BASE/OWNER/REPOSITORY`.

Existing repositories are inspected before any update. Dirty worktrees (including
staged and untracked files), detached HEADs, missing upstreams, in-progress Git
operations, and branches ahead of or diverged from their upstream are **blocked**.
Resolve these states manually before retrying. Nonempty non-Git destinations
produce a helpful failure and are left intact.

Eligible branches are fetched, inspected again, and fast-forwarded to the inspected
upstream commit. A fetch that discovers divergence counts as **blocked** and
**fetched-only**, never updated or up-to-date. Fetched-only is a subset of blocked,
not an additional repository total. Fetch failures count as failed.

Matching GitHub timestamps and a matching clean local HEAD/upstream allow a
**skipped (cached)** result, not a claim of remote equality. `--refresh-all` bypasses
this optimization. Missing timestamps force a refresh. Only verified clones,
updates, and equal branches produce success receipts in `.st-sync-state.json`;
blocked and failed entries lose their cached receipts. Cache skips retain the
previous receipt without changing its timestamp.
Unreadable or malformed caches fail explicitly and are preserved for review.

An OS-backed `.st-sync-lock` excludes concurrent Smart Tree sync processes for
the same destination, including cache writes. The lock file stays in place;
ownership is released when the process exits. This lock does not coordinate with
editors or independent Git commands, so avoid editing repositories during sync.
The cache is replaced atomically after all workers complete. Failed repositories
give a nonzero exit status; blocked repositories are reported separately.

Local regression fixtures: `cargo test --lib cli_commands::sync_tests`.
