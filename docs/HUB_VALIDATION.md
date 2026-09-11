# Hub deployment validation — 2026-09-11

Deployed to `8s.is` through the existing SSH configuration on port 2552.
`i1.is` remains the installer. The existing `boot.ayeos.com` site still responds.

## Verified

- Smart Tree workspace checks: **462 tests passed, 35 ignored**;
  `cargo clippy --workspace -- -D warnings` and formatting passed through
  `./scripts/manage.sh test`.
- Hub integration checks passed on macOS and on the Linux server under the
  dedicated service user: private retrieval, authenticated Git access, cloning,
  archive receipts, operator approval boundaries, private feedback, exact commit
  and line evidence, excluded credentials, persisted TLS approval limits,
  native MEM8/T8R recovery, unchanged-commit reuse, atomic generation replacement,
  and durable consent revocation.
- Playwright passed desktop and mobile checks against both an isolated test hub
  and the live HTTPS sites. Live checks included semantic recall, source evidence,
  themes, consent defaults, API documentation, and installer command cards.
- Real Smart Tree and Glow release installs passed on Apple Silicon macOS and in
  a clean Linux x86-64 container. Offline Smart Tree checks covered all four
  supported release targets, checksum rejection, unsupported platforms, and the
  noninteractive menu. All six menu installer responses passed shell syntax checks.
- The sibling i1 server's Rust tests and Linux release build passed. Its broader
  existing crate still emits unused/dead-code warnings; this is not a claim that
  its full lint backlog is resolved.
- Live HTTPS checks passed for `8s.is`, API/feedback/Git aliases, `agents.8s.is`,
  `i1.is`, and `www.i1.is`. The public TLS approval route returned 404, and private
  feedback without administrator authorization returned 401.
- The real local embedding model distinguished a child/space-homework query from
  an unrelated repository-storage passage. Live semantic retrieval also worked
  after a service restart. This is a functional check, not a retrieval-quality or
  full-capacity benchmark.
- An existing operator-owned StandardGalactic repository served Git refs after
  adding a per-process exception for its exact registered Git directory. Existing
  repositories were not re-owned or made writable by the hub.
- A labelled deployment feedback entry was acknowledged over public HTTPS and
  retrieved through the private operator endpoint.

## Security scan interpretation

The focused `st --no-daemon --security-scan src/hub` scan reported no findings.
The required repository-wide scan completed with 17 critical, 21 high, 1 medium,
and 50 low matches and returned a nonzero exit status. All reported files were
unchanged from the existing commit: detector patterns and tests in
`src/security_scan.rs`, cleanup patterns in `src/ai_install.rs`, the persistence
test fixture, a trust-analyzer comment, security disclosure examples, and a catch
expression in the existing vendored Markdown renderer. Those matches were reviewed;
the scan should not be described as globally clean or as an independent audit.

## Operational state and limits

All three application services and Caddy are active and enabled at boot. Release
directories retain the previous binaries and include SHA-256 manifests. The hub
release installed during this validation is `091555296345cd2c`; the installer is
now `b150f12579a2be1f` (previously `daa6aac520d347df`). Application source snapshots
remain in the server's private build directory. The current CLI source and binary
bundles are public on i1.is, as described below. No GitHub release was created.

The initial import registered **469 StandardGalactic repositories**. Indexing is
still running; catalogue and passage counts are live. No failed repositories were
reported at the final deployment check. New archive requests require operator
approval. Existing collection registrations do not create independent backups.

The nine unused hard drives remain unformatted. Archives currently use the mounted
16 TB drive, with roughly 15.5 TB available at deployment. Indexes and model files
use the existing mirrored NVMe root filesystem. No unattended backup job is installed.
On-demand TLS admits up to 35 new subdomain names per rolling week. See
[operations and remaining scale limits](HUB_OPERATIONS.md) for the storage paths,
operator commands, backup procedure, and current retrieval/indexing limits.

## Current CLI installer update

The promoted CLI build is **8.1.0-f2dbac1f5a90**, built from one frozen source
snapshot with a locked dependency graph. Precompiled and source modes both enable
`--features full`; unfinished Google and voice placeholders are excluded. The
source archive, platform binaries, provenance, and SHA-256 hashes are published
under `https://i1.is/releases/smart-tree/8.1.0-f2dbac1f5a90/`.

- Full release builds passed on Apple Silicon macOS and Linux x86-64. The Linux
  binary was rebuilt on Ubuntu 22.04 to reduce its minimum glibc requirement from
  2.39 to 2.34; it also requires OpenSSL 3. All four Linux binaries ran in that
  Ubuntu environment and on the Arch server. macOS's deployment target is 11.0.
- Native CLI/daemon checks passed on both systems: concerning-file Markdown,
  PEM and embedded DER certificates, malformed certificate diagnostics, HTTP and
  socket memory, live index updates, restart recovery, unchanged-index reuse,
  and durable deletion. The Linux fixture was moved out of `/tmp`, which the
  scanner intentionally excludes by default; production filtering was retained.
- Public HTTPS precompiled installs passed on both platforms. Installed Linux
  binaries match the verified build byte-for-byte. The active Mac installation
  was upgraded from 6.5.2 to this build; its previous `st` binary is retained in
  `~/.local/share/smart-tree/install-backups/before-8.1.0-f2dbac1f5a90/`.
- The public `--compile` installer also passed end-to-end in the Ubuntu build
  environment: verified source download, locked full-feature compilation using
  the retained Cargo cache, all four installed binaries, and matching build ID.
- Installer tests passed for both terminal choices, noninteractive defaults,
  version/build pinning, four platform mappings, checksums, unsafe source archive
  rejection, the Cargo feature arguments/cache, and root-menu argument forwarding.
- Five publication regression tests passed, covering immutable builds, checksum
  rejection, metadata consistency, path validation, and omission of build files.
- Live Playwright passed both copy buttons, displayed version, feature details,
  failed-metadata fallback, light/dark themes, and 320/390-pixel mobile layouts.
  The broader live hub recall/citation and installer browser checks also passed.
- The installed MCP `feedback` tool's `check_updates` operation reports 8.1.0 as
  current. The public i1 and hub version endpoints serve the same manifest with
  `Cache-Control: no-store`; successful immutable downloads use long-term caching.
- Final routing checks caught and corrected a Caddy ordering issue: the internal
  route block must be a `handle` before the generic proxy. Canonical, API, and
  wildcard hub names now return 404 for internal routes and 401 for private
  feedback. `scripts/test_hub_routing.py` preserves this regression check.
- Post-install `st --cleanup` found no malicious AI integrations on this Mac;
  the focused hub security scan also reported no findings. The repository-wide
  scan interpretation above still applies.

The web installer release built successfully, and its five Rust tests passed.
Its existing unused/dead-code warning backlog remains. Public health checks passed
for i1, the hub, and the unchanged boot site; all four services remain active.
