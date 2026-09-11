# Certificate scanning and daemon scan memory

`st --mode markdown PATH` includes a **Concerning Files** section with risk,
relative file path, line number, pattern, and explanation. Findings are sorted by
risk and come from the normal tree scan. Filters, depth, ignored files, and
`--no-security` affect coverage; an empty section does not mean every file was
inspected. Source excerpts are omitted. The size chart uses actual scanned file
counts.

## Inspect certificates in files

```sh
st --cert-scan ./assets
st --cert-scan ./bundle.pem --mode json
st --no-daemon --cert-scan ./application.bin --mode json
```

The explicit scan reads regular files, including binary files, and reports PEM
certificates and embedded DER X.509 certificates. It handles bundles and
deduplicates identical certificates within a file by SHA-256 fingerprint.
Metadata includes subject, issuer, serial, validity timestamps (Unix seconds in
JSON), alternative names, CA status, self-issuance, encoding, and byte offset.
Offsets identify the first PEM occurrence, or the first DER occurrence when no
PEM copy is present. Keys and raw certificate bytes are not part of the inventory.

Inspection does not verify signatures, chain trust, revocation, or signatures on
executables. Self-issued means issuer and subject names match. Certificates inside
compressed or encrypted containers need to be extracted first. Explicit malformed
PEM blocks are reported; arbitrary binary data that does not parse as a certificate
is not classified as a certificate.

Certificate findings apply the existing configured distrust/approval policy and
check expired or not-yet-valid dates. The explicit command runs even when
`security.certificates.enabled` disables certificate analysis in general integrity
scans. Files are never quarantined by the certificate command.

`security.scan.max_file_size_mb` (default 100) limits each read, including files
that grow during scanning. `security.scan.follow_symlinks` defaults to false.
The command scans directories recursively, independently of the tree formatter's
filters. Unreadable, oversized, and non-regular entries are listed as skipped;
malformed PEM blocks and skipped entries produce a nonzero CLI exit status after
the report is printed. Certificate findings themselves do not change that status.

## Daemon API and persistence

The CLI uses the running HTTP daemon for integrity and certificate scans. If the
daemon cannot be started/reached, it scans locally; `--no-daemon` explicitly uses
the local scanner. Daemon scans are stored before a successful response is sent.
Local scans do not write MEM8 history. Existing SQLite security policy, hash
memory, and integrity history remain in use.

All of these HTTP routes require the daemon's Bearer token:

| Method | Route | Request |
| --- | --- | --- |
| POST | `/security/scan` | `{"path":"/absolute/path","recursive":true}` |
| POST | `/security/certs/scan` | `{"path":"/absolute/path","recursive":true}` |
| GET | `/security/history` | Query parameters `path` and `kind` (`integrity` or `certificates`) |

The certificate response includes `scanned_at`, `files_scanned`, `files`, and
`skipped`. Partial scans retain those diagnostics. History returns the latest
report for a target and scan kind, including its timestamp and recursion setting;
missing history returns 404. History is not automatically reused as a fresh scan:
files, policies, and certificate validity can change. Request a new scan to update
it. Paths in HTTP requests are relative to the daemon's working directory, so use
absolute paths; the CLI converts paths before sending them.

With the default database path, the daemon writes `~/.st/security.scans.t8` and a
writer lock at `~/.st/security.scans.lock`. A custom `security.database.path` places
these files alongside that database using `.scans.t8` and `.scans.lock` extensions.
Only one daemon can open a particular store at a time.

The implementation adapts Aye's `crates/mem8-storage/src/wal.rs` append, durable
flush, replay, and consolidation design. Reports use the **T8R version 1** token
record format. Native MEM8 is reserved for conversation and directory memory;
see [persistent context search](CONTEXT_SEARCH.md). T8R has a versioned header,
bounded variable-length bincode records, lossless dictionary tokens, fast zlib
compression, checksummed frame headers/payloads, and an in-memory index
to the latest record for each key. Each append is flushed to disk before it is
acknowledged. Records are limited to 64 MiB before and after compression.

The journal consolidates when it exceeds 4 MiB and obsolete records occupy more
space than current records. Consolidation writes and syncs a temporary file, then
atomically replaces the journal while retaining the writer lock. Restart recovery
discards incomplete final frames; checksum failures and unsupported versions fail
without replacing the data. Store files are created with mode 0600 on Unix.
This provides compact storage and bounded record reads; no throughput claim has
been established by a representative production benchmark.

## Validation

Run `./scripts/manage.sh test` for tests, strict clippy, and formatting. External
`mq aggregate` tests require a separately installed marqant CLI and are opt-in:

```sh
MQ_TEST_BINARY=/path/to/mq cargo test --test test_mq_aggregate -- --ignored
```

See [local validation](LOCAL_VALIDATION.md) for the release build and isolated
daemon restart/scanning smoke test.
