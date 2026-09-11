# Smart Tree Hub on 8s.is

`8s.is` provides repository archives, cited recall, and private Smart Tree feedback.
`i1.is` remains the universal installer, with browser pages and shell responses
served by the existing sibling `i1-is` application. The hub follows its visual style.

The installer offers a small tool menu and direct commands. Smart Tree offers
precompiled binaries or compilation from the matching current source snapshot.
Both include the supported `full` feature set and install `st`, `std`, `m8`, and
`n8x`. `--features full` includes local LLM inference; `--all-features` also enables
unfinished Google/voice placeholders and is deliberately excluded. Model weights
are downloaded separately when needed. Glow uses its upstream release binaries.

```sh
curl -fsSL https://i1.is/tools/smart-tree | sh -s -- --precompiled
curl -fsSL https://i1.is/tools/smart-tree | sh -s -- --compile
curl -fsSL https://i1.is | sh -s -- fzf
```

The current catalogue is at `https://i1.is/releases/smart-tree/latest.json`.
`latest.txt` resolves to the immutable build ID. The browser, installer, CLI
updater, and public hub version endpoint share this catalogue, independently of
GitHub's older release tags. The manifest records source provenance, feature
selection, available platforms, and SHA-256 hashes. These are explicitly identified
source snapshots; publishing one does not create a GitHub release.

The terminal prompts for a mode; noninteractive installs default to precompiled.
`--prefix`/`I1_INSTALL_DIR` selects the destination (default `~/.local/bin`), while
`--version`/`I1_VERSION` selects a version or exact build ID. Both source and binary
downloads require matching checksums. An unavailable current binary produces a
compile instruction, without substituting an older release. Source compilation
uses the included lockfile and reuses `~/.cache/i1/smart-tree/target` (override with
`I1_BUILD_CACHE`). It requires C/C++ tools, plus pkg-config and D-Bus headers on
Linux; the script installs a minimal Rust toolchain if Cargo is missing.
The current precompiled targets are Apple Silicon macOS (deployment target 11+)
and Linux x86-64 (glibc 2.34+ and OpenSSL 3, including Ubuntu 22.04 and Debian 12).
Intel Macs and Linux ARM64 can use the source option. `st --update` currently
checks availability; use the installer commands above to perform an upgrade.

The public [API and coverage guide](https://8s.is/docs) and
[agent guide](https://8s.is/llms.txt) describe the supported requests and limits.

## Storage and permissions

Git is the archive format. Exact source passages and feedback use compressed T8R
records. Native MEM8 stores semantic vectors and their model identity. SQLite WAL
stores the catalogue, permissions, index generations, and persistent FTS5 tokens.
Restarting loads saved vectors and token indexes; it does not rebuild embeddings.
Changed commits publish a new generation atomically. Unchanged commits are reused.

New requests accept public GitHub HTTPS sources, default to private archives with
recall disabled, and wait for operator approval before cloning. A receipt token
controls each archive. Visibility and recall permission are independent. Disabling
recall immediately removes it from retrieval and logically deletes indexed records;
this is not secure erasure of old disk blocks or backups. Feedback is operator-only.

Existing collections are registered in place, reading committed Git objects. They
are not extra backup copies. The initial collection is `/data/git/standardgalactic`.
`/data/git/google` is allowlisted for a later explicit import.

The September 2026 server inventory found ten 16 TB hard drives. One is already
mounted at `/data/git`; the other nine are unformatted and remain untouched. The
two 960 GB NVMe drives already mirror the root filesystem using Linux RAID1. Hub
state and model files use that SSD-backed filesystem. Public capacity figures come
from mounted filesystems, so unused raw disks are not counted as available archives.

| Purpose | Location |
| --- | --- |
| New Git mirrors | `/data/git/smart-tree-hub` |
| Catalogue, T8R passages, MEM8 recall | `/var/lib/smart-tree-hub` |
| Local embedding model cache | `/var/lib/smart-tree-encoder` |
| Private configuration and administrator token | `/etc/smart-tree-hub` |
| Hub releases and current symlink | `/usr/local/lib/smart-tree-hub` |
| Installer releases and current symlink | `/usr/local/lib/i1-installer` |
| Installer state | `/var/lib/i1-is` |
| Immutable CLI bundles and current pointer | `/srv/i1-releases/smart-tree` |

## Services and routing

Connect with the existing SSH entry: `ssh 8s.is` (port 2552). Services start on boot:

```sh
sudo systemctl status smart-tree-hub smart-tree-encoder i1-installer
sudo journalctl -u smart-tree-hub -n 50 --no-pager
curl -fsS http://127.0.0.1:8428/health
curl -fsS http://127.0.0.1:8429/health
curl -fsS http://127.0.0.1:8431/health
```

The hub and installer run as separate unprivileged users with restricted writable
paths. The encoder container runs without capabilities, with a read-only root,
bounded memory/CPU, and its model directory mounted writable. All three services
listen on loopback. Caddy handles public HTTPS and preserves `boot.ayeos.com`.

The Caddy fragment is [deploy/hub/Caddyfile.fragment](../deploy/hub/Caddyfile.fragment).
Known 8s names and both i1 names receive automatic certificates. Other single-label
`*.8s.is` names use on-demand certificates. A loopback-only approval route accepts
at most 35 new names per rolling seven days; established names remain eligible for
renewal. This uses individual certificates, not a DNS wildcard certificate. Public
access to `/internal/*` is blocked at Caddy. Arbitrary domains are refused.

Keep `/etc/caddy/Caddyfile`'s existing global options and sites. The fragment needs
the commented `on_demand_tls` options added to that global block and an import of
`/etc/caddy/conf.d/smart-tree-hub.caddy`. Validate before reloading:

```sh
sudo caddy validate --config /etc/caddy/Caddyfile
sudo systemctl reload caddy
```

## Operator actions

The installed `st-hub-admin` helper reads the protected token file locally. It never
puts the token in command arguments and refuses HTTP redirects. Run it through sudo.

```sh
# Includes private requests for the operator; paginate with offset as needed.
sudo st-hub-admin '/api/v1/repositories?limit=100'
sudo st-hub-admin /api/v1/admin/feedback

# Approve one reviewed request; replace ARCHIVE_ID with its catalogue ID.
sudo st-hub-admin /api/v1/admin/repositories/ARCHIVE_ID/approve --json '{}'

# Register an allowed collection, explicitly enabling public recall.
sudo st-hub-admin /api/v1/admin/import --json \
  '{"root":"/data/git/standardgalactic","public":true,"recall_opt_in":true}'

# Recheck local HEAD after an operator updates the underlying repository.
sudo st-hub-admin /api/v1/admin/repositories/ARCHIVE_ID/reindex --json '{}'

# Withdraw public access and recall permission immediately.
sudo st-hub-admin /api/v1/repositories/ARCHIVE_ID --method PATCH --json \
  '{"public":false,"recall_opt_in":false}'
```

The worker retries interrupted indexing on startup. Failed repositories keep their
last published generation and show an error in the catalogue. Inspect the failure
and use the reindex endpoint to retry. Imports skip already registered paths.

There is no automatic upstream fetch, push support, user account recovery, archive
deletion API, or Git LFS/submodule download. Keep the receipt token to manage an
archive; an operator can intervene with the administrator credential.

## Builds, checks, and updates

```sh
cargo build --release --no-default-features --bin st-hub
./scripts/manage.sh test
python3 scripts/test_hub.py --binary target/release/st-hub

# Optional real-browser checks; point at an installed playwright-core module.
PLAYWRIGHT_MODULE=/path/to/node_modules/playwright-core \
  python3 scripts/test_hub.py --binary target/release/st-hub --browser
```

The integration test starts isolated temporary Git repositories and an encoder
fixture. It tests private access, Git cloning, source evidence, consent revocation,
TLS approval limits, persistence, and unchanged-commit reuse. The browser test
covers desktop/mobile layouts, themes, search, receipts, feedback, and dialogs.
The fixture validates retrieval plumbing; live-model quality requires separate
queries against the deployed encoder.

Linux build templates and service units are in [deploy/hub](../deploy/hub).
`Dockerfile.build` uses a pinned Rust image and the locked dependency graph. The
build context needs Cargo files, `src`, `st-protocol`, `expert_prompt_engineer`, and
`systemd`, with local secrets and build outputs excluded. Extract the release
binary, run the integration test on Linux, install it in a new release directory,
switch the `current` symlink, and restart the hub. Keep the previous release for
rollback. No archive or index rebuild is needed for a compatible binary update.

The encoder uses local CPU inference with `BAAI/bge-small-en-v1.5` through pinned
FastEmbed. Model downloads occur at startup; document and query text stays on this
server. Its response includes a digest of the model/tokenizer artifacts. Retrieval
only compares vectors with the matching model identity.

Smart Tree feedback now defaults to `https://8s.is`. `SMART_TREE_HUB_URL` can select
a different hub; the MCP's existing `SMART_TREE_FEEDBACK_API` full-URL override is
also supported. Cached feedback files require an explicit resubmission. Rebuild or
release the CLI to distribute the changed default to existing installations.

### Publishing a current CLI build

Run the pre-commit checks and `python3 scripts/test_i1_release.py` first.
`scripts/package_i1_release.py snapshot --output
/path/to/releases` creates an allowlisted source archive, includes `Cargo.lock`,
and fingerprints its contents. Build the extracted source on each supported
platform with `ST_BUILD_ID` set to the generated ID:

```sh
ST_BUILD_ID=BUILD_ID cargo build --locked --release --features full \
  --bin st --bin std --bin m8 --bin n8x
python3 scripts/package_i1_release.py binaries \
  --release /path/to/releases/BUILD_ID --binaries /path/to/target/release \
  --target aarch64-apple-darwin
```

The binary packager runs `st --version` and rejects a mismatched build ID. Use
`x86_64-unknown-linux-gnu` for the Linux bundle. The compatible Linux build base is
in `deploy/hub/Dockerfile.cli`; build it with an empty context using
`docker build -t smart-tree-cli-build - < deploy/hub/Dockerfile.cli`. Mount the
extracted source and a persistent Cargo target directory when compiling. Refresh
extracted source timestamps before reusing a target directory: the source archive
has deterministic timestamps. The public compile installer does this automatically.

Combine the verified platform asset entries into one manifest, run native smoke
checks, and transfer only the packaged release files to the server. Then publish:

```sh
sudo python3 scripts/publish_i1_release.py --release /path/to/verified/BUILD_ID
```

The publisher checks hashes, expected filenames, metadata, and feature selection;
it refuses to change an existing immutable build. It atomically switches the
`current` symlink after installing the complete bundle. Keep old build directories
for pinned installs and rollback. Bump the Cargo package version for subsequent
functional releases: CLI update notifications compare semantic versions.
Caddy serves release files directly, with uncached current/version pointers and
immutable caching for successful build-specific downloads. The hub's public
`/api/smart-tree/latest` route serves the same manifest without restarting indexing.
After changing routing, run `python3 scripts/test_hub_routing.py --alias
https://api.8s.is --alias https://agents.8s.is --existing-site
https://boot.ayeos.com/`. Internal routes must be blocked in a `handle` before the
catch-all proxy handle; a top-level `respond` executes too late in Caddy's ordering.

## Backups and current scale

No unattended backup job or disk-formatting job is installed. Before copying hub
state, stop `smart-tree-hub` and back up its entire state directory together,
including SQLite WAL files and all T8R/MEM8 files. Protect the token and environment
files separately. Archive drives and existing Git collections need their own
backup policy. Restart after the copy, then verify health and a known recall.
Restores must use a consistent set of catalogue and record files. Do not restore
older consent records without reconciling subsequent permission withdrawals.

The first import indexes in the background. Text coverage is current committed
HEAD, selected UTF-8 files up to 1 MiB, and bounded files/passages per repository;
the public guide documents exclusions. Semantic retrieval currently scans saved
vectors with cosine similarity, with eight concurrent recall slots. This is not
an ANN index or a full-capacity performance claim. Old generations may occupy
space after publication; compaction and large-corpus performance work remain.
