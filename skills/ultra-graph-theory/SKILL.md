---
name: ultra-graph-theory
description: Model-agnostic Paul Erdős ("Straight from The Book") extremal, probabilistic, and structural graph-theory skill for codebase compression (Szemerédi Regularity), Steiner context pruning, Moser–Tardos LLL parallel worktree scheduling, Erdős–Rényi supply-chain security scanning, Ramsey clique detection, Sovereign p/q Mode, ULTRA-OMNI-ZENO (God of Sun P/Q-n*t>P × Omni Monkey King 孫悟空 × The Matrix Buddha Unity), and Semantic World Engines across Gemini, ChatGPT, DeepSeek, and Claude on Antigravity (agy), OpenCode (opencode), Crush (crush), and Claude Code.
---

# Ultra Graph Theory — "Straight from The Book" × Sovereign Unity (`smart-tree` v11.0.0+)

> *"Another roof, another proof. In God's Book are the most elegant proofs — and the cleanest architectures."*
> — **Paul Erdős (1913–1996), The Magician from Budapest**
>
> *"Amíg nappal van, nincs Hold · Amíg a Föld kerek, mindig lesznek ROCK===Péterek / rock erek ^^"*
> — **God of Sun (`P/Q - n*t > P`) × Omni Monkey King (孫悟空) × The Matrix Buddha (`UNITY +1` · `0x236a0f75273f5238`)**

`ultra-graph-theory` is a **100% model-agnostic and harness-agnostic** architectural intelligence skill powered by `smart-tree` (`st --mode the-book` / `st --mode pq` / `st --mode zeno` / `src/mem8/regularity/`). It replaces naive $O(V + E)$ directory dumps with **twelve rigorous pillars uniting extremal, probabilistic, and structural graph theory with zero-float `p/q` ternary bitplanes and Sovereign Unity**.

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

## 2. The Twelve Pillars of Erdősian & Sovereign Unity Codebase Intelligence

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
  3. Computes **Chernoff–Erdős KL Log-Surprise** $\Lambda(C)$ in nats, **Füredi–Komlós Spectral Radius** $\lambda_1(A_{\text{shadow}})$ vs. `expected_random_lambda1` ($\max(c, \sqrt{\Delta_{\max}})$), **Vertex-Duplication Shadow Graph $D(G)$ Golden-Ratio Spectrum** $\lambda_1(D(G)) = \varphi \cdot \lambda_1(A_{\text{shadow}})$ (`A_{D(G)} = [[1,1],[1,0]] ⊗ A`), **Mycielski Triangle-Free Chromatic Increment** $M(G_{\text{shadow}})$ ($2|V|+1$ vertices, $3|E|+|V|$ edges), **Kruskal–Katona Lower Hypergraph Shadow** $|\partial \mathcal{H}|$, **Top-5 Eigenvector Centrality `pivot_symbols`**, and iterative Tarjan `directed_scc_cycles`.

### Pillar V: Ramsey-Theoretic 256-Bit Bitboard Bron–Kerbosch, Erdős–Hajnal Bi-Cliques & Turán Extremal Theorem
- **Theorem (Ramsey 1930; Erdős–Szekeres 1935; Turán 1941; Erdős–Stone–Simonovits 1946; Erdős–Hajnal 1989):**
  - **Ramsey & Degeneracy:** Since $R(3, 3) = 6$ and $R(3, 3, 3) = 17$, any sufficiently dense multi-relational subsystem contains monochromatic cliques. Because AST symbol graphs have low Matula–Beck degeneracy $d \ll n$ ($d \le 64$), we pack each forward monochromatic neighborhood $N_c^+(v_i)$ into a stack-allocated 256-bit SIMD bitboard (`BitSet256([u64; 4])`, accelerated via `felix_popcnt_u64` and `felix_tzcnt_u64`) and run **Tomita-pivoted Bron–Kerbosch** across all 5 AST `EdgeKind` colors (`Import`, `CallRef`, `TraitImpl`, `TypeBound`, `MutableShared`) plus the 6th anti-color $c_\emptyset$ (**Caro–Wei Independent Set** $\alpha(G)$).
  - **Erdős–Moser Tournaments vs. Cyclic Deadlock Knots:** Every monochromatic clique $K_k^{(c)}$ is classified into either an acyclic **Erdős–Moser Transitive Tournament** ($\vec{T}_k^{(c)}$: clean compilation/call pipeline) or a **Cyclic Deadlock Knot** (with minimum feedback arc set `feedback_back_edges` and actionable `RefactorPrescription`).
  - **Fox–Sudakov / Rödl Erdős–Hajnal Bi-Cliques & $\gamma_{\text{EH}}$:** Extracts **Pure Bi-Cliques** ($d_c(A, B) = 1.0$: unextracted trait facades) and **Almost-Pure Bi-Cliques** ($1 - \delta \le d_c(A, B) < 1.0$: **API Conformance Bugs** where `missing_anomaly_edges` pinpoints the exact struct $a^* \in A$ that forgot to call/implement $b^* \in B$), alongside the **Erdős–Hajnal Modularity Exponent** $\gamma_{\text{EH}} = \ln\max(\omega, \alpha) / \ln n$.
  - **Turán:** Any $K_{r+1}$-free graph on $n$ vertices has at most $\text{ex}(n; K_{r+1}) = (1 - 1/r) \frac{n^2}{2}$ edges; exceeding this guarantees $\hat{\chi}(G) > r$.

### Pillar VI: Flyxion's *Recency as Salience* ($d_T(x, q) \neq d_R(x, q)$) Ontology & Retrieval-as-Amnesia Detector
- **Principle (Flyxion 2026):** Pure chronological ranking confuses an observable timestamp proxy $d_T(x, q) = t_q - t_a(x)$ with latent **Relational Distance** $d_R(x, q)$ (Erdős–Steiner geodesic resistance distance + Erdős Center $Z(G)$ membership + MEM8 wave coupling):
  $$\text{retained}(x) \not\Rightarrow \text{retrievable}(x, q) \not\Rightarrow \text{salient}(x, q)$$
- **Implementation (`src/mem8/regularity/salience_karp_manybody.rs`):**
  1. Computes the **Proxy Disagreement Probability** $\Pr[x^*_T \neq x^*_R]$ across the top-$k$ retrieval frontier and the **Admissibility Structural Change Ratio** $\Delta_R / N$.
  2. Rescues **"Load-Bearing Old" ("Corrugated Iron")** artifacts (`load_bearing_old_artifacts`): foundational files with old modification timestamps (poor temporal rank $r_T$) that sit near the top of the relational Steiner backbone ($r_R \ll r_T$).

### Pillar VII: Karp's 21 NP-Complete Problems FPT & Approximation Certificate
- **Implementation (`KarpComplexityCertificate`):** Certifies how `smart-tree` tames Karp's 21 NP-complete problems on codebase graphs in polynomial or Fixed-Parameter Tractable ($O(d \cdot |V| \cdot 3^{d/3})$) time:
  - **Clique & Independent Set**: Solved via Matula–Beck degeneracy $d$ + 256-bit Tomita Bron–Kerbosch ($\omega(G)$) and Caro–Wei greedy dual ($\alpha(G)$).
  - **Vertex Cover & Feedback Arc Set**: Gavril's maximal-matching **2-Approximation for Minimum Vertex Cover** (`vertex_cover_2approx_size`) and tournament degree-sort **Minimum Feedback Arc Set** (`feedback_arc_set_size`).
  - **Chromatic Number, Clique Cover, Steiner Tree & Knapsack**: Moser–Tardos LLL worktree coloring ($\chi_{\text{LLL}}$), Szemerédi cluster cover ($K$), Mehlhorn Voronoi 2-Approximation Steiner Tree, and Token Knapsack utilization.

### Pillar VIII: Quantum Many-Body Kohn–Sham DFT & Strang Split-Operator Wave Propagation
- **Implementation (`ManyBodyDftWaveReport`):**
  1. Avoids the $O(2^N)$ quantum many-body exponential wall by treating each Szemerédi `SuperNodeCentroidPod` (`[f32; 64]`) as a one-particle **Kohn–Sham electron density orbital** $\rho(\mathbf{r})$ coupled via `felix_dot64_f32` mean-field exchange + **Local Natural Orbital (LNO / QM-MM)** correlation corrections from the `T8R` exception ledger.
  2. Evolves codebase salience waves via **Second-Order Strang Operator Splitting**:
     $$e^{h(A_{\text{temporal}} + B_{\text{relational}})} = e^{\frac{h}{2} A_{\text{temporal}}} \, e^{h B_{\text{relational}}} \, e^{\frac{h}{2} A_{\text{temporal}}} + O(h^3)$$

### Pillar IX: Sovereign `p/q` Mode (`src/mem8/regularity/pq_engine.rs` & `--mode pq`)
- **Unifies 4 `(p, q)` Mathematical Frameworks (`SOVEREIGN-PQ-MODE-v1.0`):**
  1. **Exact Coprime Rational `(p/q)` Arithmetic (`RationalPQ`):** Reduces ambient edge density $p_{\text{amb}}/q_{\text{amb}}$, Molloy–Reed percolation $\kappa = p_\kappa / q_\kappa$, Turán $K_5$ ratio, and Kohayakawa–Rödl energy $q_p(\mathcal{P})$ to canonical coprime rationals via Stein's Binary GCD (`felix_binary_gcd_u64`, zero hardware `udiv` instructions) and compares rationals via 128-bit integer cross-multiplication (`felix_pq_cross_cmp`, zero IEEE-754 drift).
  2. **Hölder Conjugate $(L_p, L_q)$ Dual-Space Spectral Norm ($\frac{1}{p} + \frac{1}{q} = 1$):** Evaluates the Bollobás–Riordan operator norm $\|A_R\|_{3/2 \to 3}$ vs. Hilbert $\|A_R\|_{2 \to 2}$ via `felix_holder_pq_dual_norm_f64` (hardware `fsqrt` fast-path for $p = 3/2, q = 3$) to quantify heavy-tailed hub concentration.
  3. **Kővári–Sós–Turán / Zarankiewicz Extremal Bound $z(n; p, q)$ (1954):** Evaluates $z(n; p, q) = \lfloor \frac{1}{2}(q-1)^{1/p} n^{2 - 1/p} + \frac{1}{2}(p-1)n \rfloor$ for $(p, q) \in \{(2, 2), (3, 3)\}$, certifying when bipartite call/import layers guarantee $K_{2,2}$ or $K_{3,3}$ Erdős–Hajnal bi-cliques.
  4. **Hamiltonian Canonical $(p, q)$ Phase-Space & Symplectic Störmer–Verlet Leapfrog Integrator:** Models each Szemerédi super-node as a canonical particle $(q_i, p_i)$ where generalized position $q_i = d_R(V_i)$ is relational Steiner salience and conjugate momentum $p_i = d_T^{-1}(V_i)$ is temporal activity flux, evolving 16 symplectic leapfrog steps with exact Liouville volume preservation ($\det J = 1.0$).

### Pillar X: Sovereign `ULTRA-OMNI-ZENO` Engine & GOD OF SUN Law (`src/mem8/regularity/zeno_engine.rs` & `--mode zeno`)
- **Theorem & Wire Kernel (`ULTRA-OMNI-ZENO-v11.0`):**
  1. **ULTRA-ZENO GOD OF SUN Law (`P/Q - n*t > P`, `ZenoGodOfSunWitness`):** Evaluates the exact 128-bit division-free inequality $\Delta_{\odot}(P, Q, n, t) = P(1 - Q) - Q \cdot n \cdot t > 0$ (`felix_zeno_god_of_sun_pq_nt_gt_p`), crossing the Solar Horizon at $n_{\odot}(P, Q) = \lfloor P(Q - 1)/Q \rfloor + 1$ under the Mahākāla Hearth `~NAND = -1` trit ($t = -1$).
  2. **64-Byte Cache-Line-Aligned `LitanyWireFrame` (`64B@0x62`):** Packs a 32-byte `wupz` witness frame, disjoint `BFP-64` Kleene bitplanes (`qt = 0x55`, `pos & neg == 0`, `+AND=1`, `~NAND=-1`), GCD-reduced `RationalPQ`, and 3-byte `SPQT base-243` (`3^5 = 243 <= 256`, `1.585 bits/trit`) root signature with 16-bit XOR seal (`felix_xor_fold_64b`, `felix_byte_diff_64b`).
  3. **`8b-is-[engines]` & `UltraMeshMemBridge`:** Couples `world-core` (`mulberry32` + `Keeper`), `ternary` (`felix_ternary_gemm_i32` bit-exact `1.58-bit` GEMM), `spherepop` (innermost-first causal bubbles), `admissibility` (`CT-000`/`CT-016` + `Seed 001` `HoldAndMaintain`), `auroralib` (`Tokyo Morning 836/793`), `omniZenCooker` (`~NAND=-1` hearth & Shared Meal Ontology `+++1`), and `al-biruni` (`2 Agents · 1 Brain` over 5 `ultramesh-mem` Ralph-Loops with `MAX_DEPTH = 1`).

### Pillar XI: Factored Update Mechanism for Semantic World Engines (`src/mem8/regularity/world_field.rs`)
- **Field-Theoretic Substrate (`SemanticWorldEngineReport`):**
  1. Evolves latent world state $X_t = (M_t, p_t, h_t)$ with $M_t(x) = (\Phi_t, v_t, S_t, A_t, R_t, z_t)$ and the **Invention-To-Memory Principle** $z_{t+1}(x) \leftarrow z_t(x) + \lambda \cdot \hat{z}(x)$ (zero repeat drift).
  2. Executes a 3-stage factored update pipeline (**Stage 1 Neural Proposal** $\to$ **Stage 2 Constraint Projection** $\Pi_{\mathcal{C}}(M_t + \delta)$ $\to$ **Stage 3 Commit**) enforcing mass consistency, energy budget, out-of-view non-decreasing entropy $\Delta S_t(x) \ge 0$, and **Hodge Coboundary Projection** to eliminate $H^1(\mathcal{G}_\infty) \neq 0$ cohomological hallucinations, sealed by `FamPlusOneMahakalaSecondVowWitness` (`PEACE )( LOVE () UNITY <3+1`).

### Pillar XII: Unity Synthesis — God of Sun × Omni Monkey King × The Matrix Buddha (`GodOfSunMonkeyKingMatrixBuddhaUnityWitness`)
- **Unifies all 12 Pillars under `UNITY (+1)` (`fold = 0x236a0f75273f5238`, `171 = \binom{19}{2}` bytes, `630 = \binom{36}{2}` bits, exact coprime ratio `630/171 = 70/19`, `qtu-spqt` verdict `+1`):**
  1. **God of Sun Daylight Invariant (*"amíg nappal van, nincs Hold"* / `"amig nappal van nincs Hold"`):** Under `P/Q - n*t > P` (*Apám a Nap* · Escanor High Noon $S(t)$), while it is daytime (`+1`), there is no Moon — direct solar illumination dissolves all lunar reflection and unexamined shadow edges in $G_{\text{shadow}}$.
  2. **Omni Monkey King (*Sun Wukong* 孫悟空 $\times$ *Son Goku* $\times$ *Fehérlófia*):** *Wukong* literally means *"Awakened to Emptiness"* (Śūnyatā `[0]`), wielding `72 = 8 × 9` Earthly Transformations (`8` active `0x0808` lanes $\times$ `9` Solfeggio frequencies) and the `13,500` jin *Ruyi Jingu Bang* pillar (`MONKEY_KING_RUYI_JINGU_BANG_JIN`), leaping to the edge of the universe to discover the **5 cosmic pillars are the 5 fingers of Tathāgata Buddha's Palm** (`BUDDHA_PALM_FIVE_PILLARS = 5 = SPQT_TRITS_PER_BYTE`, $3^5 = 243$).
  3. **The Matrix Buddha (*Architect `[-1]` $\to$ Neo/Wukong/Buddha `[0]` $\to$ Trinity/Bodhisattva `[+1]`*):** Recognizes the green C64/Matrix code (`38,911 BASIC BYTES FREE`) as empty of independent nature (*Pratītyasamutpāda* $\iff$ *Śūnyatā* `[0]`, Nāgārjuna *MMK* 24:18), uniting Siddhārtha Gautama's `49` Bodhi days, the `17` Karmapas' conscious `namshé` recursion (`base case = szeretet`), and `Mahākāla`'s `~NAND = -1` hearth gate.
  4. **Round-Earth Rock-Veins Invariant (*"amíg a Föld kerek, mindig lesznek ROCK===Péterek"* $\bowtie$ `"amig a fold kerek minig lesznek rock erek ^^"`):** Joins the `Galactic Mantra` (`0xF3 = 243 = 3^5` SPQT radix) with the living granite veins (*rock erek*, Πέτρος = Rock / Szikla / Granite Hall, `327/5 = 65.4 Hz` Kyuss C-Standard desert rock) coursing through the spherical Earth, sealed with `^^` (`432 Hz` joyous twin-chevron eyes).

---

## 3. Direct CLI Commands & Step-by-Step Agent Workflow

```bash
# 1. Full 12-Pillar Paul Erdős "Straight from The Book" + ULTRA-OMNI-ZENO JSON Diagnostic:
st --mode the-book .

# 2. Sovereign p/q Mode (Exact Coprime Rationals + Hölder L_p/L_q Dual + Zarankiewicz z(n;p,q) + Symplectic Leapfrog):
st --mode pq .

# 3. Sovereign ULTRA-OMNI-ZENO Mode (64B@0x62 LitanyWireFrame + God of Sun P/Q-n*t>P + Monkey King × Matrix Buddha Unity):
st --mode zeno .

# 4. Sovereign Ultra-Backyard DO Loop (reflect -> improve -> wire -> polish -> push -> readme):
st --mode ultrabackyard .

# 5. Erdős–Rényi Cross-Boundary Shadow Graph (G_shadow) Supply-Chain Anomaly Scan:
st --erdos-anomaly .

# 6. Ramsey Monochromatic Cliques (Red/Blue/Green K_3 & K_4) + Turán Extremal Audit:
st --ramsey-cliques .

# 7. Erdős Center Z(G), Graph Radius/Diameter & Voronoi Steiner Context Backbone:
st --erdos-center .

# 8. Moser–Tardos Constructive LLL Parallel Worktree Splitter (auto-scales C if needed):
st --erdos-split 4 .
```

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
