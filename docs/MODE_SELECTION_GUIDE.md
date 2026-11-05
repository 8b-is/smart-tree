# Choose an output mode

Decide what the consumer needs to recover from the result. A summary can answer “what kind of project is this?” even when it omits the one path you need for a code change.

| Question | CLI mode | Information available |
| --- | --- | --- |
| What paths are visible? | `classic` | A human-readable tree of entries admitted by the scan. |
| What should a program parse? | `json` | Structured nodes. Limit the root or depth to control output size. |
| What metadata does each entry carry? | `ai`, `hex` | Compact text records for inspected entries. |
| Which parts deserve a closer look? | `summary-ai` | Aggregate project information and selected paths. Follow up with a listing or targeted search. |
| How many entries and bytes were scanned? | `stats` | Aggregates over the scan. |
| Has the *structure* changed? | `digest` | A short metadata fingerprint plus counts and summaries. Same-length file edits can leave it unchanged. |
| What does the bounded projection retain? | `louds` | JSON containing the [SMTR capsule](SMTR_CAPSULE.md), raw/projected node counts, and a subtree digest. |
| What does graph analysis find? | `the-book` | A report on the graph built from scanned file nodes. See the [architecture review](ARCHITECTURE_REVIEW_V10_8.md) for limits. |

```sh
st --no-daemon --mode classic --depth 2 .
st --no-daemon --mode json --depth 2 .
st --no-daemon --mode louds --depth 2 .
```

In the default MCP interface, `analyze` takes `mode: "directory"` to choose the operation and `format: "json"` to choose the formatter. CLI mode names and MCP operation names serve different purposes. Older examples using `st --mode claude` predate the current CLI. See `st --help` for available modes.

For comparisons, keep root, depth, ignores, and code revision fixed. Measure the [actual bytes or model tokens](PERFORMANCE_METRICS.md) and record what information each output omitted.
