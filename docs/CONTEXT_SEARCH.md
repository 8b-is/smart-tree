# Persistent context search

Smart Tree's daemon keeps a searchable file index, conversation history, and
explicit associations between files, people, notes, and time. Queries use saved
data without calling an LLM or walking the filesystem.

```sh
std start  # Keep running; use another terminal for the next command
st --recall "document with my daughter last week"
```

`--recall` returns JSON through the running daemon. Add roots with `POST /watch`;
`std start` also registers its working directory. Roots can be directories on any
mounted filesystem accessible to the daemon. Indexing is limited to registered
roots; this does not automatically crawl every disk or infer personal relationships.
An explicitly removed root stays removed across restarts; `POST /watch` adds it
again when needed.

## HTTP API

Use the daemon bearer token from `~/.st/daemon.token` (or `ST_TOKEN_PATH`) for
these authenticated endpoints. The
default base URL is `http://127.0.0.1:28428`.

| Request | Purpose |
| --- | --- |
| `POST /watch` with `{"path":"/absolute/directory"}` | Index a root and persist its watch |
| `DELETE /watch` with the same body | Remove its watch and index entries, preserving overlapping roots |
| `POST /context/remember` | Persist a contextual association in native MEM8 |
| `POST /context/recall` | Search the saved index and associations |
| `GET /context` | Read restored project and directory counts |

Record the evidence that makes a personal query answerable:

```json
{
  "path": "/Users/me/Documents/moon-homework.pdf",
  "people": ["daughter"],
  "notes": "Worked on lunar homework with my 10 year old daughter",
  "occurred_at": "2026-09-03T16:00:00Z"
}
```

Send that body to `/context/remember`; it returns an ID and `stored: true` only
after persistence succeeds. Then send a query to `/context/recall`:

```json
{
  "query": "document with my daughter",
  "after": "2026-08-31T00:00:00Z",
  "before": "2026-09-07T00:00:00Z",
  "limit": 10
}
```

Results include the file path, a match score, modification time when indexed,
recorded evidence, and a bounded text preview. Time bounds are inclusive at the
start and exclusive at the end. The phrase `last week` supplies the previous
Monday-to-Monday calendar week in UTC when explicit bounds are absent. Limits
default to 10 and are capped at 100.

Retrieval currently uses keyword postings and time filters. Recorded context
matches receive greater weight than file text matches. Scores are ranking weights,
not probabilities. People and collaboration context must be recorded explicitly;
file modification time alone does not establish who worked on a document.

The index records filenames and metadata for regular files. Supported text
extensions also contribute terms from at most the first 64 KiB. PDF, Office,
encrypted, and other binary files can be found by path and recorded context;
their document contents are not extracted by this implementation. Recursive
indexing skips `.git`, `node_modules`, `target`, `.cache`, the daemon's own memory
directory, and symlinks. Inaccessible roots retain previously saved context.

## Storage and restart behavior

| Data | Store |
| --- | --- |
| Proxy conversation scopes | `~/.st/proxy_memory.m8` |
| Watched roots, directory context, contextual associations | `~/.st/directory_context.m8` |
| File metadata and keyword index source records | `~/.st/file_index.t8` |
| Unix-socket wave memory | `~/.mem8/wave_memory.native.m8` |
| Integrity and certificate reports | `~/.st/security.scans.t8` |

`ST_MEMORY_DIR` overrides conversation, directory, file-index, and socket-memory
locations. Project-specific MCP memories remain under the project's `.st/mem8`.
Security reports follow `security.database.path`.

`ST_CONFIG_PATH` selects an alternate configuration file (default:
`~/.st/config.toml`). Set it together with `ST_MEMORY_DIR`, `ST_TOKEN_PATH`,
`ST_DAEMON_PORT`, and `XDG_RUNTIME_DIR` when running an isolated test daemon.

Memory stores use Aye's native MEM8/RAW8 v1.1.1 block layouts: a 32-byte fixed-point
WaveAtom in the MEM8 lane and exact application data in the paired RAW8 lane.
The safe byte codec follows `mem8-core/src/wave_atom.rs` and
`mem8-storage/src/block_v1.rs` from Aye commit
`9d052392a673d6aeebc69f1846f4ed1fb728e3f4`. Smart Tree's `ST8` application envelope
inside RAW8 is versioned separately. This is not the older M8C/WaveInt container.
Wave values are quantized to the native fixed-point fields; timestamps and exact
source content remain in the paired application data.

Exact data uses lossless dictionary token IDs, varints, and fast zlib compression.
Standalone `.t8` files use the T8R v1 frame format for data without a memory wave.
Token dictionaries are scoped to individual records, so replay needs no model,
external vocabulary, or separately regenerated tokenizer state.

Each update is appended and synced before acknowledgement. Stores have exclusive
writer locks and checksums. Replay recovers an incomplete final append; detected
corruption fails explicitly. Compaction retains current records and atomically
replaces the journal after syncing it. Files are created with mode 0600 on Unix.
Individual records are capped at 64 MiB before and after compression. The current
native writer uses one wave per pair and additional pairs for larger payloads,
so small memories have a minimum 8 KiB physical footprint. The token store avoids
that block overhead for file facts. No production throughput or competitor
performance claim has been established.

Existing `proxy_memory.json` and legacy JSON wave-memory files are imported once.
Imports resume after interruption, preserve newer records and deletions, and
leave the original files untouched. Native stores are authoritative afterward;
there is no automatic fallback to temporary conversation memory. Clearing a
memory is a logical deletion; retained legacy files and obsolete journal records
are not securely erased.

The daemon restores saved context and watches before serving. File events update
affected index entries; metadata reconciliation after startup and every five
minutes covers changes missed while offline. Unchanged files are not reread or
rewritten. Queries use an immutable index view and remain available while the
writer works. Changed directory context is checkpointed every 30 seconds and
on graceful HTTP shutdown. Pending context updates are journaled with file-index
changes and replayed if a checkpoint is interrupted. An abrupt termination can
lose recent wave activity. Idle wave decay does not trigger storage writes or gate
retrieval. The proxy retains the latest 20 messages per conversation scope.

This supplies persistent local retrieval and an API foundation. General semantic
embedding generation, ANN indexing, automatic collaboration capture, and broad
document extraction remain separate work before positioning it as a general
replacement for a vector database.
