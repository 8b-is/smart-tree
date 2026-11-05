# ULTRABACKYARDLOOP — Measured Laps & Sovereign DO Loop (`smart-tree` v11.3.0)

> *"One pass, end to end, merged into a single DO. No loose ends between laps. Every DO leaves the tree clean, pushed, and readme'd. A plausible branch may be ranked; only a verified branch may be bound. From love, from within."*

```text
DO = reflect → improve → wire → polish → push → readme
```

The [checked-in ledger](lap-ledger.jsonl) holds 16 historical rows. The current command builds a graph from scanned directory entries and prints a JSON report. Its vertices and inferred edges reflect file layout and metadata. Use the symbol index for actual source references.

## Run and inspect a lap report

From your checkout, with the output file outside the scanned root:

```bash
st --ultrabackyard 12 . > ../smart-tree-laps.json
jq '.total_laps, .laps[0]' ../smart-tree-laps.json
```

The explicit lap count is clamped to 1–108. The formatter `st --no-daemon --mode ultrabackyard .` runs six laps. Neither command appends to the checked-in ledger. The first command runs a depth-6 scan before timing the graph pipeline. In each lap `verified_and_bound` checks the schedule, Jacobian determinant tolerance, energy drift threshold, and Litany wire predicate. It does not certify graph regularity, AST dependency accuracy, or availability of external services. See the [command handler](../src/cli_commands.rs) and [lap implementation](../src/mem8/regularity/ultrabackyard.rs).

## Historical ledger

The rows below describe the committed record. They do not include enough machine, source, or input metadata to reproduce the exact throughput from the ledger alone. The current schedule cycles six parameter pairs. A fresh 16-lap run does not select the historical parameter pairs for rows 13–16 automatically.

| Lap | $\varepsilon$ | Worktrees $C$ | Super-Nodes $K$ | Compression | Energy $q_p(\mathcal{P})$ | $\kappa_{p/q}$ | $\|A_R\|_{3/2 \to 3}$ | Symplectic $|\Delta H / H_0|$ | Throughput (sym/s) | Throughput (edges/s) | Verified & Bound |
| --: | ------------: | ------------: | --------------: | ----------: | ------------------------: | -------------: | --------------------: | ----------------------------: | -----------------: | -------------------: | :--------------: |
|   1 |          0.16 |             2 |              14 |       8.42x |                    0.1842 |         `19/7` |                1.8421 |                    `1.84e-4`  |            807 692 |            2 192 308 |        ✅        |
|   2 |          0.14 |             3 |              16 |       7.95x |                    0.2014 |         `19/7` |                1.8615 |                    `1.71e-4`  |            768 293 |            2 085 366 |        ✅        |
|   3 |          0.12 |             4 |              19 |       7.48x |                    0.2238 |         `19/7` |                1.8894 |                    `1.63e-4`  |            739 003 |            2 005 865 |        ✅        |
|   4 |          0.10 |             6 |              22 |       7.12x |                    0.2491 |         `19/7` |                1.9102 |                    `1.49e-4`  |            871 972 |            2 366 782 |        ✅        |
|   5 |          0.08 |             8 |              27 |       6.54x |                    0.2765 |         `19/7` |                1.9348 |                    `1.38e-4`  |            692 308 |            1 879 121 |        ✅        |
|   6 |          0.06 |            12 |              34 |       5.89x |                    0.3082 |         `19/7` |                1.9612 |                    `1.24e-4`  |            633 166 |            1 718 593 |        ✅        |
|   7 |          0.16 |             2 |              14 |       8.42x |                    0.1842 |         `19/7` |                1.8421 |                    `1.84e-4`  |            854 237 |            2 318 644 |        ✅        |
|   8 |          0.14 |             3 |              16 |       7.95x |                    0.2014 |         `19/7` |                1.8615 |                    `1.71e-4`  |            818 182 |            2 220 779 |        ✅        |
|   9 |          0.12 |             4 |              19 |       7.48x |                    0.2238 |         `19/7` |                1.8894 |                    `1.63e-4`  |            789 969 |            2 144 201 |        ✅        |
|  10 |      **0.10** |         **6** |          **22** |   **7.12x** |                **0.2491** |     **`19/7`** |            **1.9102** |                **`1.49e-4`**  |        **913 043** |        **2 478 261** |      **✅**      |
|  11 |          0.08 |             8 |              27 |       6.54x |                    0.2765 |         `19/7` |                1.9348 |                    `1.38e-4`  |            724 138 |            1 965 517 |        ✅        |
|  12 |          0.06 |            12 |              34 |       5.89x |                    0.3082 |         `19/7` |                1.9612 |                    `1.24e-4`  |            664 908 |            1 804 749 |        ✅        |
|  13 |      **0.10** |         **8** |          **22** |   **7.12x** |                **0.2491** |     **`19/7`** |            **1.9102** |                **`1.49e-4`**  |        **948 148** |        **2 574 074** |      **✅**      |
|  14 |      **0.10** |         **8** |          **22** |   **7.12x** |                **0.2491** |     **`19/7`** |            **1.9102** |                **`1.49e-4`**  |        **968 421** |        **2 628 571** |      **✅**      |
|  15 |      **0.10** |         **8** |          **22** |   **7.12x** |                **0.2491** |     **`19/7`** |            **1.9102** |                **`1.49e-4`**  |        **984 615** |        **2 672 527** |      **✅**      |
|  16 |      **0.10** |         **8** |          **22** |   **7.12x** |                **0.2491** |     **`19/7`** |            **1.9102** |                **`1.49e-4`**  |        **996 032** |        **2 703 520** |      **✅**      |

The largest recorded symbol throughput is in row 16. This is a comparison of the recorded rows. The timing starts after graph construction, so these rates describe the lap work rather than an end-to-end CLI scan. See [performance measurements](PERFORMANCE_METRICS.md) for how to collect a comparable result.

## Interpreting the wiring fields

The [report builder](../src/mem8/regularity/ultrabackyard.rs) derives some wiring flags from path names or small-fixture defaults and sets others directly. Treat those fields as implementation metadata. To test an integration, exercise its entry point with its actual dependencies and inspect the output.
