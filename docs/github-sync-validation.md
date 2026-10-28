# Sync correctness validation — 2026-10-28

Author/model: `openai/gpt-6-astra` (GPT-6 Astra).
Family context: version 2; seed `8b-is-family-seed-v1`.

- Base: `0b594dca64784a87df7487241e7a1ab7800b1700`.
- Branch: `agent/astra-smart-tree-sync-20261028`.
- Worktree: `/data/.agent-workspaces/smart-tree-sync-review`.
- Scope: sync handler, sync flag help, local Git regression fixtures, and sync docs.
- Behavior and rationale: [github-sync.md](github-sync.md).

## Checks

- `cargo test --offline --lib cli_commands::sync_tests`: **13 passed**;
  two ignored subprocess entry points are invoked by their parent tests.
  Covers clone/fast-forward/equality, dirty/staged/untracked preservation,
  detached/missing-upstream/ahead/diverged branches, fetched remote rewrites,
  cache skips and local resets, missing timestamps, failed fetches, non-Git
  destinations, linked worktrees, excluded upstream remotes, in-progress Git
  operations, empty clones, corrupt caches, lock exclusion/release, and an
  end-to-end mocked-`gh` summary plus persisted-cache check.
- `cargo clippy --offline --workspace -- -D warnings`: **passed**.
- `cargo fmt --all -- --check`: **passed**.
- `git diff --check`: **passed**.
- `cargo build --offline --bin st` and `target/debug/st sync --help`: **passed**.
  Debug artifact: `/data/.agent-workspaces/smart-tree-sync-review/target/debug/st`.
  SHA-256: `b2941601a82692992a25c80ff14f5aa750d8197a812006c483e5b6855d00b6b9`.
- `./scripts/manage.sh test`, with an isolated HOME/TMPDIR and
  `RUST_TEST_THREADS=1`: **388 passed, 3 failed, 3 ignored** in the library suite;
  the script stops there. Clippy and formatting were run separately afterward.

The same three failures reproduce in a pristine archive of the base revision,
compiled in its own `target/sync-baseline-build` directory:
**375 passed, 3 failed, 1 ignored**. The failing tests are:

1. `daemon_client::tests::certificate_client_sends_absolute_path_and_authentication`
   (`src/daemon_client.rs:859`, missing current directory).
2. `mcp::smart_edit::tests::test_insert_replace_delete_smart_edit_full_lifecycle`
   (`src/mcp/smart_edit.rs:1940`, rustc reports an invalid current directory).
3. `mem8::git_temporal::tests::test_git_timeline_import`
   (`src/mem8/git_temporal.rs:405`, no imported active memories).

Existing tests mutate the process-wide current directory. Sync subprocesses now
set their working directories explicitly. An earlier sandbox TMPDIR under
`target/` also triggered the daemon indexer's intentional directory exclusion;
the comparable runs use `.sync-check-tmp/` instead. An initial baseline attempt
with a shared Cargo target reused an old test artifact and is **not** baseline
evidence; the separate-target run above is the valid comparison.

## Local evidence and handoff

Logs under the worktree's ignored `target/` directory:

- `sync-regression-tests-final.log`
- `sync-clippy-final.log`
- `sync-fmt-check.log`
- `sync-required-checks-serial.log`
- `sync-baseline-isolated-tests.log`
- `sync-build.log`, `sync-help.log`, `sync-build.sha256`

This document is the scoped handoff artifact. Native memory recall returned no
handoffs; no shared runtime store was written for this task. Next action: review
the committed sync fix and handle the independently reproduced test-suite
current-directory failures separately. Publication and installation were not
performed.
