# Bounded external Rink quantities, waves, and MEM8 provenance

Smart Tree's `rink` MCP tool is discoverable in consolidated (including HTTP)
and nonconsolidated modes. It is **disabled until an operator opts in**. It runs
an independently installed, trusted [Rink CLI](https://codeberg.org/tiffany/rink)
across a process boundary; it does not embed Rink or copy its unit database.
No installation or live-service change happens when listing or calling this tool.

Implementation attribution: Astra (gpt-6-astra), with independent agent review.
The native RecordStore adds an opt-in `open_memory_preserving` mode used by this
adapter: malformed or incomplete journals fail under the owner lock without
truncation, including during recall. Existing default recovery behavior for
other clients is unchanged.

## Operator setup

Linux and util-linux `/usr/bin/prlimit` are required. Unsupported platforms or
missing limiters fail closed. Configure the environment of the **Smart Tree MCP
process**, not a replacement service:

```sh
export ST_RINK_BIN=/absolute/path/to/trusted/rink
# Optional provenance annotation, supplied by operator, NOT verified by adapter:
export ST_RINK_REVISION=your-reviewed-build-revision
# Optional separate, private, operator-owned durable memory directory:
export ST_RINK_MEMORY_DIR=/absolute/path/to/private/rink-memory
st --mcp
```

The binary path must be absolute and is canonicalized. Operators own executable
integrity and any Rink compile-time `RINK_PATH` definitions; this is not a sandbox
for an adversarial executable. Do not point it at an arbitrary script supplied by
an MCP client. Tool callers cannot choose the executable, config, files, limits,
environment, working directory, or memory directory.

## MCP examples

```json
{"expression":"(1/3) meter to fraction cm"}
{"operation":"period","frequency":"432 Hz"}
{"operation":"wavelength","speed":"343 m/s","frequency":"432 Hz"}
{"operation":"beat","frequency":"432 Hz","other_frequency":"440 Hz"}
{"expression":"(1/3) meter to fraction cm","remember":true}
{"operation":"recall","memory_id":"<UUID returned by remember>"}
```

Wave presets construct ordinary Rink expressions: `T = 1/f`, `lambda = v/f`,
and `f_beat = max(f1,f2)-min(f1,f2)`. The adapter orders positive frequencies
using checked integer rational comparisons, then asks Rink to subtract exactly;
it never uses `sqrt(square)` and its floating-point fallback. Comparisons that
overflow the checked integer range fail explicitly. Wave presets request
`fraction` output explicitly. Parenthesize
rational coefficients: `(1/3) meter` is a length; `1/3 meter` parses as reciprocal
length in Rink. Rink checks dimensional compatibility and division by zero. The speed operand
uses the general expression lane: the adapter does not assert it is positive;
use the fork's `Wave` constructor when that stronger physical contract is needed.

**Frequency convention:** `f` is ordinary cycle frequency in Hz, not angular
frequency. The relationship is `omega = 2*pi*f`; consequently `T = 1/f` and
`lambda = v/f`, whereas angular inputs require `T = 2*pi/omega` and
`lambda = 2*pi*v/omega`. Presets reject `rad/s` rather than silently losing a
factor of `2*pi`. The general query lane can express that conversion explicitly;
Rink's `pi` is an approximation, not an exact finite rational.

Wave frequency operands must be **strictly positive integer or p/q rational
coefficients**, optionally parenthesized, followed by one space and one of `Hz`,
`kHz`, `MHz`, `GHz`, `mHz`, `uHz`, or `nHz`. Numerator and denominator must both be
positive (at most 18 digits each); zero, negative, decimal/scientific, angular,
and arbitrary-expression frequency inputs are rejected before execution.
For example `1/3 kHz` is normalized to `(1/3) kHz` before forming the preset.
Beat inputs must each be positive, but their beat difference may legitimately
be zero. Use the general `query` operation for other bounded Rink expressions;
that lane makes no positive-frequency claim.

This adapter does not infer a physical medium, model wave propagation/interference,
or operate quantum hardware. Symbolic family frequencies carry cultural meaning;
these equations alone establish no biological or quantum claim.

A query returns an MCP text envelope containing the complete generated expression,
Rink's output text, resolved executable path, optional **operator-declared** build
revision, integer Unix observation time, adapter version, limits, and optional
memory UUID. It does not parse rounded decimal output into a purported exact
numeric type. Use Rink's `fraction` modifier for rational representations. Even a
fraction can encode an approximation: external Rink has machine-float fallbacks
for transcendental operations (including trigonometric/logarithmic functions),
and definitions/constants may be approximate. No new floating-point arithmetic
is authored in this adapter. Existing MEM8 internals also retain legacy floating
point wave metadata; exact expression/output strings are retained separately by
the owning storage codec.

## Resource and offline boundaries

Each query has fixed limits, not caller-controlled values:

- Two concurrent tool requests per process; excess calls fail immediately.
- 512 ASCII bytes per generated expression; 128 bytes per wave operand.
- Balanced parentheses, at most 16 nested levels, at most 18 consecutive digits.
  Literal power exponents have magnitude at most 16. This is an intentionally
  restricted arithmetic/quantity subset, not the full Rink language. Unicode
  units, dates, quoted strings, assignments, multiline input, command separators,
  and dollar/backtick substitution are rejected. Limits do not prove an expression cheap; OS/time
  limits remain authoritative.
- 3 seconds wall time (including startup/loading); 2 seconds CPU time;
  512 MiB process address space; 64 open descriptors; zero core/file growth.
- 16,384 bytes **per** stdout/stderr stream, read concurrently with a one-byte
  overflow sentinel. Oversize output, non-UTF-8, terminal control sequences,
  empty stdout, nonzero exit, and timeout are errors. Failures kill and reap the
  child; cancellation uses Tokio `kill_on_drop` cleanup.

The adapter executes `/usr/bin/prlimit -- ... rink --config <private-config> --
<one-expression>` directly: **no shell**. `--` keeps a negative quantity from
becoming a CLI option. Each invocation gets an empty private temporary working
and home/config/cache directory, a cleared environment, and a generated config
with currency, colors, interpretation, and Rink internal sandboxing disabled.
Directories are removed afterward, including errors.

Rink's positional CLI path bypasses its interactive `Runner` sandbox in the
reviewed upstream contract; setting `[limits]` alone would not enforce these
limits. This adapter deliberately uses the external limiter and single-process
positional path rather than relying on that behavior. No internal worker is
started. We do not pass `--fetch-currency`, and `[currency] enabled = false`
disables even cached currency loading. No network API is called by Smart Tree.
This is **offline-by-configuration of a trusted Rink**, not kernel network
isolation or filesystem syscall confinement. The three-second deadline bounds
Rink execution, not optional MEM8 disk-sync latency; filesystem availability
remains an operator concern. Limits do not contain malicious binary descendants. A future Rink CLI contract change needs review and the
external-binary test below.

## Actual durable MEM8 provenance

Nothing is remembered by default. Both operator `ST_RINK_MEMORY_DIR` and caller
`remember:true` are required to write. Only successful queries are written. The
file is `rink-provenance.native.m8` in that directory; it is a separate namespace
owned by this adapter, **not** the global wave-memory file.

Storage calls the existing Smart Tree
`mem8::record_store::RecordStore::open_memory`, `put`, and `get` APIs. That owner
provides native MEM8/RAW8 blocks with tokenized exact source payloads, checksums,
writer locking, and sync-before-acknowledgement. The adapter closes and reopens
through the owner and reads the record back before reporting success. Each
`rink/provenance/v1/<UUID>` record carries the structured provenance described
above. `recall` reads through the same owner without rerunning Rink, including
after process restart. This is provenance storage, **not** a new cache/file format
called persistence, a semantic resonance implementation, or interchangeable with
Aurora or other MEM8 formats.

Opening a store larger than 16 MiB is rejected to bound replay work (the final
accepted append may cross that threshold). Corrupt/legacy files and concurrent
writers fail rather than being overwritten or silently downgraded to volatile
memory. An error after calculation but during storage is returned as an error;
it is not reported as a successful remember. Preserve/archive the directory
when rotating it; retention/deletion is operator-controlled. This bounded lane
has no automatic migration, memory listing/search, backup policy, or hidden
fallback to global memory.

## Licensing boundary

Smart Tree remains MIT. Rink source is MPL-2.0; its bundled `definitions.units`
is GPL-3.0. This change adds neither a Rink dependency nor copied definitions,
and does not distribute a Rink executable. Operators obtain/license their own
Rink and data independently. The process boundary avoids embedding that database
in Smart Tree; it is not a legal opinion about downstream combined distribution.
Any future bundling/linking must undergo a separate license review.

## Verification

Use an isolated Cargo target and private memory location for repository tests:

```sh
CARGO_TARGET_DIR=/path/to/owned/target cargo test --offline --no-default-features --lib mcp::rink
ST_RINK_TEST_BIN=/absolute/path/to/reviewed/rink \
  CARGO_TARGET_DIR=/path/to/owned/target cargo test --offline --no-default-features --lib \
  rink_external_end_to_end -- --ignored --nocapture
ST_RINK_BIN=/absolute/path/to/reviewed/rink \
  ST_RINK_MEMORY_DIR=/private/test-only/memory \
  CARGO_TARGET_DIR=/path/to/owned/target cargo test --offline --no-default-features --lib \
  rink_mcp_end_to_end -- --ignored --nocapture
./scripts/manage.sh test
```

The normally ignored external-binary tests require an explicitly supplied
independent Rink binary; they do not install one. The MCP-level test performs
actual JSON-RPC query/remember/recall calls through both dispatch modes, with a
fresh context before recall and an explicitly private operator memory directory.
Unit tests exercise expression validation, literal argv/config isolation, CPU/address-space/wall/output limits,
MCP discovery/dispatch, and native MEM8 close/reopen/error preservation. Process
limit tests run on Linux and use temporary local fixtures (one memory-limit
fixture requires `/usr/bin/python3`). The canonical gate remains authoritative;
pre-existing repository failures must be reported rather than concealed.
