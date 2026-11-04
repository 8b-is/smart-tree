# Smart Tree 10.5.0 validation

Author: OpenAI GPT-6 Astra (`openai/gpt-6-astra`), 2026-10-28.
Research references: `rustfix` owned staging, `bfs` budget admission, and `gfold`
Git state distinctions. These are independently authored adaptations; see the
feature-specific documents for source revisions and attribution.

## Local checks

- `cargo test --offline --locked --workspace --no-fail-fast -j 4 -- --test-threads=1`
  passes across library, binary, integration and doc targets. The primary library
  has 400 passing tests and three existing subprocess/ignored entries.
- `cargo clippy --offline --locked --workspace -j 4 -- -D warnings` passes.
- `cargo fmt --all -- --check` and `git diff --check` pass.
- `python3 scripts/test_i1_release.py`: all five packaging tests pass.
- `bash -n scripts/install-auto-update.sh` passes.
- Atomic-edit integration verifies unchanged target/history on errors and compiles
  and executes successful sequential edits. Existing sync and scanner regression
  suites are included in the workspace run.

## Fixture repairs required for reliable checks

The original full suite exposed seven library failures. Existing hook unit tests
changed process cwd to a temporary directory and deleted it, breaking unrelated
tests. They now exercise the real hook CLI in subprocesses with the same contract
assertions. Scanner fixtures now use the checkout filesystem because production
scanning intentionally skips tmpfs. Related integration fixtures were repaired
the same way. Git timeline import now creates its own committed fixture instead
of depending on cwd history or skipping CI. No production scanner exclusion or
test assertion was disabled to obtain these results. The pre-change main CI also
reported the cwd failures on Linux and macOS (run `40507410575`).

## Scanner's self-scan

`st --security-scan . --no-update-check` ran and returned exit 1 with pattern
matches, rather than a clean scan. The report includes the scanner's own
signatures, documentation, retained legacy files, and the moved hook test's
literal risky-command fixture. The test supplies that string to the classifier;
it does not execute npm. No new runtime dependency is introduced by the feature
patches. A pattern scan is not a comprehensive security audit.

## Release evidence

Version checking, artifact production, feed promotion and installed upgrade
verification are separate steps. The build/publishing process and machine-level
verification procedure are documented in [HUB_OPERATIONS.md](HUB_OPERATIONS.md).
Platform build receipts and deployment results belong with the release artifacts;
this source-validation receipt alone does not establish deployment.
