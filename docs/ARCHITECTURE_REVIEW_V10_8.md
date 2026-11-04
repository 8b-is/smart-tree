# Smart Tree architecture review

The v10.8 Oni-Felix work added bounded directory projections and the `SMTR` capsule. The v10.9 work added Book mode and FELIX kernels. This review describes how those paths behave in the [11.3.1 source snapshot](https://github.com/8b-is/smart-tree/tree/f7dacdcaa8ecd466b1b74b71f3972f32769f892a). The [original review](https://github.com/8b-is/smart-tree/blob/8f043b7ee38b6394ea5e0862393de4e348bfc069/docs/ARCHITECTURE_REVIEW_V10_8.md) is available for historical comparison.

## From request to result

| Entry point | How it produces a result | What the caller receives |
| --- | --- | --- |
| `st --no-daemon --mode classic .` | The [scanner](../src/scanner.rs) walks eligible entries under the requested root. The [formatters](../src/formatters/) consume those nodes and their statistics. | A directory listing, scoped by depth, ignores, and safety budgets. |
| `st --mcp` | The [stdio server](../src/mcp/mod.rs) exposes consolidated tools by default. [Directory analysis](../src/mcp/tools/directory.rs) validates the root and formats the scan. | MCP content blocks. A safety-budget truncation adds a notice and skips the cache write. |
| `st symbol …` | The [CrabCC index](../src/mcp/symbol.rs) indexes source for symbol queries. | Definitions, references, outlines, or graph statistics for the requested operation. |
| `st --no-daemon --mode louds .` | The [SMTR formatter](../src/formatters/digest.rs) projects a bounded prefix of scanned nodes. | JSON containing counts, a subtree digest, and one hex-encoded 64-byte capsule. |
| `st --no-daemon --mode the-book .` | The [Book pipeline](../src/mem8/regularity/mod.rs) analyzes a graph derived from scanned file nodes. | A graph report. Its dependency-like edge labels come from directory relationships and file categories. Use the symbol index for source-level references. |

## What a scan can establish

A depth or ignore rule defines the scope requested by the caller. A safety budget can cut that scope short. The scanner records the reason in `TreeStats::truncation_reason`. The [daemon response](../src/daemon_cli.rs), [CLI](../src/main.rs), [SSE formatter](../src/formatters/sse.rs), and [statistics formatter](../src/formatters/stats.rs) each carry or display this signal. The MCP directory handler returns a separate text notice. Consume every content block before interpreting a missing result.

The capsule has a separate limit. Even a complete walk can contain more than the first 64 nodes admitted to `LoudsSmartTree`. Its projection drops ignored categories and, by default in the formatter, test entries and descendants whose parent was dropped. This is category-based pruning. File names, kinds, sizes, permissions, and tree relationships influence the digest. A same-size source edit can leave the capsule unchanged. To answer a question about file contents, retrieve the file or symbol behind the summary.

## Two different binary contracts

| Representation | What it preserves | Verification boundary |
| --- | --- | --- |
| [SMTR v2 capsule](SMTR_CAPSULE.md) | Node count after projection, one topology word, a topology checksum fragment, and the projected root digest. | The formatter verifies a tag using a key embedded in the binary. Anyone with the source can recompute it. The tag alone establishes neither author identity nor coverage of a whole repository. |
| [MEM8 block file](../src/mem8_binary.rs) | A 30-byte header and 4,096-byte blocks. Each block has a previous-hash field and up to 4,024 payload bytes. | Consecutive appends on one open writer link selected fields. Reopening an existing file resets the remembered hash to zero. The reader parses links without checking continuity. |

The block writer hashes index, wave signature, timestamp, previous hash, and stored payload. Importance, token ID, serialized length, and padding are outside that calculation. The header CRC checks the header when nonzero. Treat uninterrupted-writer linkage and verified persistence across restarts as separate guarantees. A regression test for the latter would need to append, close, reopen, append again, and verify the new link against the earlier block.

## Book mode and regularity

[Graph construction](../src/mem8/regularity/types.rs) derives vertices from file nodes, features from names and metadata, and edges from parent and neighboring entries. The [compressor](../src/mem8/regularity/compressor.rs) uses bounded refinement. Its [oracle](../src/mem8/regularity/oracle.rs) looks for a witness to irregularity. A witness can be checked by comparing subset densities. A returned `Regular` can also mean this search missed a witness.

For example, consider a 4 × 4 bipartite adjacency matrix with ones in the upper-left and lower-right 2 × 2 blocks and zeros elsewhere. The full pair has density ½. Either occupied block has density 1 and meets the minimum subset size at ε = 0.1. The oracle's alternating initial vector cancels in every row, so the zero-norm branch returns `Regular`. This matters when deciding whether to collapse a cluster or discard its exceptions.

Book mode also includes [Steiner context pruning](../src/mem8/regularity/erdos_steiner.rs), [worktree scheduling](../src/mem8/regularity/moser_tardos.rs), [graph diagnostics](../src/mem8/regularity/erdos_renyi.rs), and a separate [MEM8BOOK archive layout](../src/mem8/regularity/wire.rs). Their outputs answer different questions from the SMTR capsule.

## Performance evidence

The scanner reuses a line buffer during search. MEM8 header serialization and block I/O use stack buffers, while block payloads still allocate on append and read. The [FELIX kernels](../src/felix_asm.rs) contain architecture-specific implementations. These source-level properties can be inspected directly. Cycle counts, allocation totals, and scaling across cores require measurements on a named workload and machine. See the [measurement procedure and sample](PERFORMANCE_METRICS.md).
