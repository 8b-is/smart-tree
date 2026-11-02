# Atomic-on-operation-error Smart Edit batches

Select `"batch_mode": "atomic"` in `smart_edit`, or in consolidated `edit` with
`"operation": "smart_edit"`. The default remains `"best_effort"` for compatibility.

```json
{
  "operation": "smart_edit",
  "file_path": "src/example.rs",
  "batch_mode": "atomic",
  "edits": [
    {"operation": "replace_function", "name": "first", "new_body": "42"},
    {"operation": "replace_function", "name": "second", "new_body": "first()"}
  ]
}
```

All operations run sequentially in an owned, temporary SmartEditor. Later edits
can refer to functions inserted earlier. If an operation fails, the handler
returns an error naming its zero-based index, drops the staged content and exits
before initializing DiffStorage or writing the target. Malformed operations and
invalid mode values also fail before persistence. No successful partial result
is returned. Successful responses identify the selected mode.

This contract covers reported operation errors. It does not add semantic code
validation, crash-safe disk replacement, concurrent-writer protection or a
transaction spanning diff storage and file writes. The existing successful-write
path and its best-effort diff recording are retained. `best_effort` continues to
report individual operation errors and write accumulated edits.

## Evidence and provenance

`tests/smart_edit_atomic.rs` exercises unchanged target bytes, absent and existing
diff/ignore state, malformed requests, sequential insert-then-replace, multibyte
source, compiled result execution and legacy partial-success behavior. It runs
in its own integration-test process because DiffStorage uses process cwd.

Independently authored by OpenAI GPT-6 Astra (`openai/gpt-6-astra`). The owned
staging/fail-on-error pattern was identified in Pascal Hertleif and Oliver
Schneider's [rustfix](https://github.com/rust-lang/rustfix), reviewed at
`0886aef003b31f882116e1b32282dcba2473618a` (MIT OR Apache-2.0), research `review-17`.
No rustfix source or dependency is incorporated.
