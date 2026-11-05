# Measuring Smart Tree performance

Choose the question before choosing the metric. Output bytes measure transport size. Model input costs depend on the serialized request and tokenizer. Directory-scan latency also includes traversal and formatting. Each needs a separate measurement.

## A small output-size comparison

These are observed stdout byte counts from the [11.3.1 Linux x86-64 release](https://github.com/8b-is/smart-tree/releases/tag/v11.3.1), build `11.3.1-16835836147a`. The fixture had two files totaling 31 bytes. Every mode scanned the same root at depth 3 with color disabled.

| Mode | Output bytes | What the output contains |
| --- | ---: | --- |
| `classic` | 136 | Human-readable names, sizes, and counts |
| `json` | 331 | Structured directory and file entries |
| `ai` | 330 | Text metadata records and statistics |
| `digest` | 223 | Structural fingerprint, counts, capsule, and selected aggregates |

The digest omits entry detail. On this small fixture even the classic listing is shorter. Workload size, file mix, directory order, metadata, and the information a consumer needs all affect the comparison.

Run the following in a POSIX shell with the release binary on PATH. The temporary directory keeps benchmark output outside the scanned fixture.

```sh
(
  set -eu
  cd "$(mktemp -d)"
  mkdir -p fixture/src
  printf '%s\n' '# Example' > fixture/README.md
  printf '%s\n' 'fn main() {} // TODO' > fixture/src/main.rs
  for mode in classic json ai digest; do
    st --no-daemon --color never --depth 3 --mode "$mode" fixture > "$mode.out"
    wc -c < "$mode.out" | awk -v m="$mode" '{print m, $1}'
  done
)
```

## Timing and allocation claims

For latency or throughput, record the source revision or release build ID, hardware, OS, compiler and features, input tree, scan flags, thread count, and warm/cold cache policy. Compare equivalent tasks, repeat runs, and report the distribution. Time the whole process for CLI latency. Use a kernel benchmark for a claim about a specific instruction path. Allocation counts need an allocator trace or counter.

The [Criterion target](../benches/combinatorics_llm_bench.rs) measures selected graph and bitset workloads. From a source checkout with the project build prerequisites installed, run `cargo bench --bench combinatorics_llm_bench`. The [lap report](ULTRABACKYARDLOOP.md) covers a different workload and records per-lap graph throughput.

For token counts, specify the exact tokenizer and count the text or complete tool response the consumer actually receives. For monetary or emissions estimates, record pricing or energy assumptions separately from measured data.
