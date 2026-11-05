# Local build and validation

Validated on 2026-09-11 on Apple Silicon, macOS 27.0, with Rust/Cargo 1.98.0.

| Check | Result |
| --- | --- |
| Optimized workspace build with default/full features | Passed |
| Workspace tests, including documentation examples | 457 passed, 35 existing ignored |
| Workspace library tests without default features | 350 passed, 1 existing ignored |
| Strict workspace Clippy and formatting | Passed |
| Release-binary persistence/scanning smoke test | Passed |
| `st`, `std`, `m8`, and `n8x` help commands | Passed |

Validation repaired CLI rendering through symlinked paths, glob handling
(including Unicode), and summary/quantum-semantic formatter routing. Tool
wrappers now report subprocess failures, and integration tests select Cargo's
built binary explicitly. Documentation examples compile, and the management
script checks every workspace member.

Run from the repository root:

```sh
cargo test --workspace --no-default-features --lib
./scripts/manage.sh test
cargo build --release --workspace
python3 scripts/test_local_persistence.py --bin-dir target/release
```

The management script runs workspace tests, strict workspace Clippy, and
formatting checks. The default build includes `full` features and Candle local
LLM support. Binaries are written to `target/release/`.

The Python smoke test requires Python 3, OpenSSL, and Unix sockets. It starts
the built `std` with temporary configuration, memory, certificates, a private
socket directory, and an available loopback port. It checks:

- Markdown concerning-file findings and machine-readable certificate reports.
- PEM and embedded DER fingerprints, plus malformed-certificate exit status.
- Authenticated HTTP and CLI people/time recall.
- Native MEM8 memory over the Unix socket and incremental file indexing.
- Memory and scan-history recovery across daemon restarts.
- Reuse of unchanged index records and persistence of memory/watch deletions.
- MEM8/T8R file headers and private memory-file permissions.

The test shuts down its own HTTP service, terminates its socket process, and
removes its temporary files. It does not install binaries or call an LLM provider.

Existing ignored tests remain opt-in: private-session placeholders, tests
requiring a separately running daemon or external `mq`, and a legacy
consciousness-loop test. The process smoke test exercises the new daemon
persistence independently. Local model inference and remote provider calls are
outside this validation.

The SIMD test checks numerical accuracy and reports timing diagnostically;
machine load does not determine whether correctness tests pass.
