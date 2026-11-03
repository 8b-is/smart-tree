---
name: ultra-graph-theory
description: Model-agnostic Paul Erdős ("Straight from The Book") extremal, probabilistic, and structural graph-theory skill for codebase compression (Szemerédi Regularity), Steiner context pruning, Moser–Tardos LLL parallel worktree scheduling, Erdős–Rényi supply-chain security scanning, and Ramsey clique detection across Gemini, ChatGPT, DeepSeek, and Claude on Antigravity (agy), OpenCode (opencode), Crush (crush), and Claude Code.
---

# Ultra Graph Theory — "Straight from The Book" (`smart-tree` v10.9.0+)

> *"Another roof, another proof. In God's Book are the most elegant proofs — and the cleanest architectures."*
> — **Paul Erdős (1913–1996), The Magician from Budapest**

`ultra-graph-theory` is a **100% model-agnostic and harness-agnostic** architectural intelligence skill powered by `smart-tree` (`st --mode the-book` / `src/mem8/regularity/`). It replaces naive $O(V + E)$ directory dumps with **five rigorous theorems from extremal, probabilistic, and structural graph theory**.

---

## 1. First-Class Citizens: Models & Agent Harnesses

This skill treats **all four frontier model families** and **all four terminal/IDE agent harnesses** as first-class citizens without vendor lock-in.

### A. First-Class Frontier Models

| Model Family | Optimal `st` Invocation | Context & Reasoning Calibration |
| :--- | :--- | :--- |
| **Gemini** (`gemini-2.5-pro`, `gemini-3`) | `st -m the-book .` + `ZeroCopyMem8Book::top_k_super_nodes` | Pair Gemini's deep multimodal/long-context window with Szemerédi super-node maps (`K <= 256`) as a global structural index, expanding Erdős Number `0..=2` shells (`max_erdos_radius = 2`). |
| **ChatGPT / Codex** (`gpt-4o`, `o3`, `o4-mini`, `codex`) | `st -m the-book --compact .` | Feed the deterministic `ERDOS-THE-BOOK-v1.0` JSON schema directly into structured tool outputs; use `worktree_buckets` for parallel function calls and `token_budget = 8000` for tight latency. |
| **DeepSeek** (`deepseek-r1`, `deepseek-v3`) | `st -m the-book .` + `st --security-scan .` | Inject the 4 Erdős–Rényi invariants ($\kappa, s(C), \Lambda(C), \lambda_1$), Kohayakawa–Rödl energy $q_p(\mathcal{P})$, and Moser Witness Trees into `<think>` verification blocks before generating patches. |
| **Claude** (`claude-4-opus`, `claude-4-sonnet`) | `st -m the-book .` + `st --mcp` | Combine `ZeroCopyMem8Book` (`.m8b` mmap) + `T8R` LEB128 exception streams with MCP tool calls so Claude expands only Erdős Number `0` terminals and `1` Steiner bridges. |

### B. First-Class Agent Harnesses / CLIs

| Agent Harness | CLI Binary | MCP Config Location | Native Integration & Parallel Dispatch |
| :--- | :--- | :--- | :--- |
| **Antigravity** | `agy` | `~/.gemini/config/mcp_config.json` & `.agents/hooks.json` | Native `--agy-hook` lifecycle (`session-start`, `pre-invocation`, `pre-tool-use`, `post-tool-use`, `stop`) + subagent spawning per `worktree_buckets[c]`. |
| **OpenCode** | `opencode` | `~/.config/opencode/opencode.json` & `.mcp.json` | First-class stdio/SSE MCP server (`st --mcp` / `st --http-daemon`) + multi-session worktree dispatch across Gemini, ChatGPT, DeepSeek, and Claude providers. |
| **Crush** | `crush` | `~/.config/crush/crush.json` & `.mcp.json` | Charmbracelet `crush` fast terminal workflow: pipe `st -m the-book . \| jq .erdos_steiner_context` or invoke `smart-tree` MCP tools across any configured LLM provider. |
| **Claude Code** | `claude` | `~/.claude/settings.json` & `.mcp.json` | Native `SessionStart`/`SessionEnd` consciousness hooks (`st --agent-restore` / `st --agent-save`) and `git worktree` wave execution. |

> See [references/model-and-harness-adapters.md](references/model-and-harness-adapters.md) for copy-paste configuration files and harness-specific dispatch recipes.

---

## 2. The Five Pillars of Erdősian Codebase Intelligence

### Pillar I: Szemerédi Regularity Compression ("MEM8 Book Mode")
- **Theorem (Szemerédi 1975; Kohayakawa–Rödl 2003 for Sparse Graphs):** Any arbitrarily large graph $G = (V, E)$ can be partitioned into a small, bounded number of $\varepsilon$-regular cluster pairs $(V_i, V_j)$ that behave like random bipartite graphs of density $d_p(V_i, V_j)$, plus an outlier set $V_0$ of size $\le \varepsilon |V|$.
- **Implementation (`src/mem8/regularity/{compressor.rs,oracle.rs,wire.rs,mmap_view.rs}`):**
  1. Maximizes the **Kohayakawa–Rödl Potential Energy**:
     $$q_p(\mathcal{P}) = \sum_{1 \le i < j \le k} \frac{|V_i| |V_j|}{|V|^2} d_p(V_i, V_j)^2$$
  2. Uses the **Degrees-of-Common-Neighbors ($L_2$ Codegree) Spectral Oracle** (`oracle.rs`) accelerated by **FELIX Assembly / NEON / SSE2** (`felix_normalize_f64_slice`) to verify $(\varepsilon, p)$-regularity in $O(|E(A, B)|)$ time without $O(2^{|A|})$ subset enumeration.
  3. Serializes into a **64-byte cache-line aligned zero-copy `.m8b` frame** (`Mem8BookHeader` 128B + `SuperNodeCentroidPod` 256B + `SuperNodeMetaPod` 32B + `RegularSuperEdgePod` 8B + columnar delta-LEB128 `T8R` exception stream at ~2.1 bytes/edge).

### Pillar II: Erdős Number & Mehlhorn Voronoi Steiner Tree Context Pruner
- **Theorem (Mehlhorn 1988 2-Approximation Steiner Tree):** Given terminal symbols $\mathcal{T} \subset V$ (the focal roots of a bug or feature, assigned **Erdős Number 0**), compute the minimum-weight subgraph $T_{\text{Steiner}}$ connecting all $t \in \mathcal{T}$ plus radial shells of Erdős distance $E_{\mathcal{T}}(v) \le R_{\max}$.
- **Implementation (`src/mem8/regularity/erdos_steiner.rs`):**
  1. Runs a **single multi-source Voronoi Dijkstra pass** from all terminal clusters simultaneously, weighted by resistance distance $w_{ij} = 1 / (1 + \ln(1 + d_p(V_i, V_j)))$ and modulated by `felix_dot64_f32` 64-lane SIMD wave-centroid cosine similarity.
  2. Injects virtual bridges from the `T8R` exception stream (`extract_t8r_virtual_bridges`) so $V_0$ outlier hubs and architectural leaks are never missed.
  3. Extracts the Kruskal MST over Voronoi boundary bridges and expands clusters strictly by `(erdos_number ASC, resistance_dist ASC)` up to `token_budget`.

### Pillar III: Constructive Lovász Local Lemma (Moser–Tardos) Parallel Worktree Splitter
- **Theorem (Erdős–Lovász 1975; Moser–Tardos 2010):** If bad events $\mathcal{B}$ (merge/type/lock collisions across $C$ parallel agent worktrees) satisfy the symmetric LLL condition:
  $$e \cdot p(B) \cdot (d(B) + 1) \le 1$$
  then a conflict-free worktree coloring exists and the parallel **Moser–Tardos Maximal Independent Set (MIS) Resampling Algorithm** finds it in $O(|\mathcal{U}| / \text{poly})$ expected steps.
- **Implementation (`src/mem8/regularity/moser_tardos.rs`):**
  1. Builds a zero-allocation Dual-CSR hypergraph (`LllHypergraph`) encoding `PairwiseExclusive`, `MonochromaticHyperedge`, and `CongestionThreshold` constraints.
  2. Verifies `check_lll_criterion(c)` a priori and **auto-scales worktree count $C$** (`solve_moser_tardos_auto_scale`) if the LLL threshold is violated.
  3. Uses the 4-lane superscalar `felix_splitmix64_x4` PRNG seeded deterministically from the `.m8b` BLAKE3 digest (`seed_from_mem8_header`).
  4. If an architectural monolith violates Shearer's threshold ($p \cdot \Delta > 1/e$), returns the **minimal Moser Witness Tree** (`witness_bottleneck_events`) pinpointing the exact coupling bottleneck.

### Pillar IV: Erdős–Rényi Phase-Transition & Shadow-Graph Supply-Chain Scanner
- **Theorem (Erdős–Rényi 1960; Molloy–Reed 1995; Füredi–Komlós 1981):** In $G(n, p)$, a giant component of size $\Theta(n)$ emerges when $\kappa = \langle k^2 \rangle / \langle k \rangle > 2$ (with critical window scale $\Theta(n^{2/3})$ at $c = 1$).
- **Implementation (`src/mem8/regularity/erdos_renyi.rs`):**
  1. Filters `CsrSymbolGraph` down to the **Cross-Boundary Shadow Graph $G_{\text{shadow}}$** (`scan_erdos_renyi_shadow_graph`): edges that cross Szemerédi cluster boundaries (`symbol_to_cluster[u] != symbol_to_cluster[v]`), touch $V_0$ outliers, or touch capability hazards (`NET_IPFS_EGRESS`, `PROC_EXEC`, `SECRET_ENV_READ`, `DYNAMIC_EVAL`, `MCP_HOOK_MUTATE`, `HIGH_ENTROPY_OBFUSCATED`).
  2. Tracks **Component Excess / Cyclomatic Surplus** $s(C) = |E(C)| - |V(C)| + 1$ in $O(\alpha(n))$ time via Path-Compressing Union-Find (`uf_union_with_excess`).
  3. Computes **Chernoff–Erdős KL Log-Surprise** $\Lambda(C)$ in nats, **Füredi–Komlós Spectral Radius** $\lambda_1(A_{\text{shadow}})$ vs. `expected_random_lambda1` ($\max(c, \sqrt{\Delta_{\max}})$), **Top-5 Eigenvector Centrality `pivot_symbols`**, and iterative Tarjan `directed_scc_cycles`.

### Pillar V: Ramsey-Theoretic Monochromatic Clique Detector
- **Theorem (Ramsey 1930):** Since $R(3, 3) = 6$ and $R(3, 3, 3) = 17$, any sufficiently dense multi-relational subsystem inevitably contains monochromatic $K_3$ or $K_4$ cliques.
- **Implementation (`scan_ramsey_monochromatic_cliques`):**
  - Colors edges into **Red (`RedSyncLock`: blocking calls/locks)**, **Blue (`BlueSharedState`: shared mutable state)**, and **Green (`GreenModuleCoupling`: circular trait/type bounds)** and extracts $K_3$/$K_4$ cliques via sorted neighbor-list binary intersection.

---

## 3. Step-by-Step Agent Workflow

Whenever you are asked to **comprehend a large repo, plan a multi-agent refactor, prune LLM context, or audit supply-chain security**:

1. **Generate the Complete Erdős Book Diagnostic (`ERDOS-THE-BOOK-v1.0`):**
   ```bash
   st --mode the-book .
   ```
2. **Inspect the Paul Erdős Lexicon Classification (`erdos_lexicon`):**
   - **Straight from The Book**: Clean modules inside $\varepsilon$-regular Szemerédi clusters.
   - **Epsilons ($\varepsilon$)**: Small utility helpers (`< 1 KiB`).
   - **Legally Dead (L.D.)**: Orphaned, ignored, or zero-byte artifacts ready for pruning.
   - **Pure Noise / S.F. Monoliths**: Oversized $V_0$ outlier hubs (`> 128 KiB` or high T8R exception degree) that should be decomposed first.
3. **Prune Context via the Erdős-Steiner Backbone (`erdos_steiner_context`):**
   - Expand full source code **only** for `erdos_number == 0` (`focal_terminals`) and `exact_v0_bridges`.
   - Keep `erdos_number == 1` and `2` clusters as 18-token Szemerédi super-node summaries (`anchor_symbols`).
4. **Dispatch Parallel Agents via Moser–Tardos Worktree Buckets (`moser_tardos_lll_worktrees`):**
   - Read `worktree_buckets` (`0..num_worktrees`).
   - Verify `is_conflict_free == true` (and `lll_guaranteed_a_priori`). If `false`, refactor the `witness_bottleneck_events` sequentially before spawning parallel `agy` / `opencode` / `crush` / `claude` workers.
5. **Enforce Erdős–Rényi & Ramsey Security Gates (`erdos_renyi_percolation` & `ramsey_monochromatic_cliques`):**
   - Flag any component where `phase_regime == "AnomalousC2Clique"`, `erdos_log_surprise_nats > 15.0`, `spectral_radius_lambda1 > 2.2 * expected_random_lambda1`, or `red_sync_lock_cliques` / `blue_mutable_state_cliques` spike after a dependency update.
