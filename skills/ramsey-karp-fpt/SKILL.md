---
name: ramsey-karp-fpt
description: Ramsey-Theoretic 256-Bit SIMD Bitboard Bron-Kerbosch, Fox-Sudakov/Rödl Erdős-Hajnal Bi-Clique API Conformance Bug Hunter, and Karp's 21 NP-Complete Fixed-Parameter Tractable (FPT) Reduction Engine for Smart Tree across Gemini, ChatGPT, DeepSeek, and Claude on Antigravity (agy), OpenCode, Crush, and Claude Code.
---

# Ramsey, Erdős–Hajnal & Karp-21 FPT Engine (`smart-tree` v10.10.0+)

> *"Complete disorder is impossible."* — **Ramsey Theory (Paul Erdős, 1935–1989)**

`ramsey-karp-fpt` (`src/mem8/regularity/ramsey_erdos_hajnal.rs` & `salience_karp_manybody.rs`) turns NP-complete graph problems into sub-millisecond static analysis over the 5-color AST symbol graph (`Import`, `CallRef`, `TraitImpl`, `TypeBound`, `MutableShared`) plus the 6th anti-color $c_\emptyset$.

---

## 1. 256-Bit Bitboard Bron–Kerbosch & Erdős–Hajnal Bi-Cliques

1. **Matula–Beck Degeneracy Ordering ($d \ll n$):**
   Computes $(v_1, \dots, v_n)$ in $O(|V| + |E|)$ bucket-queue time so every forward monochromatic neighborhood $N_c^+(v_i)$ has size $\le d \le 255$.
2. **`BitSet256([u64; 4])` Hardware Bitboard:**
   Executes Tomita-pivoted Bron–Kerbosch set intersections ($P \cap N(u)$, $X \cap N(u)$) in four CPU `AND` + `felix_popcnt_u64` / `felix_tzcnt_u64` instructions with **zero heap allocations** inside the recursion.
3. **Erdős–Moser Tournament vs. Cyclic Deadlock Knot (`CliqueTopology`):**
   - `TransitiveTournament` ($\vec{T}_k^{(c)}$): Clean acyclic compilation/call pipeline.
   - `CyclicDeadlockKnot`: Mutual lock/call cycle; emits `feedback_back_edges` (Minimum Feedback Arc Set) and `RefactorPrescription::BreakDeadlockCycle` or `FlattenCircularImports`.
4. **Fox–Sudakov / Rödl Pure & Almost-Pure Bi-Cliques (`ErdosHajnalBiClique`):**
   - **Pure Bi-Clique ($d_c(A, B) = 1.0$):** $|A|$ structs all depend on the exact same $|B|$ targets $\rightarrow$ `ExtractSharedTraitFacade`.
   - **Almost-Pure Bi-Clique ($1 - \delta \le d_c(A, B) < 1.0$):** Pinpoints **API Conformance Bugs** via `missing_anomaly_edges` — the exact pair $(a^*, b^*)$ where struct $a^*$ forgot to implement or call sibling method $b^*$.

---

## 2. Taming Karp's 21 NP-Complete Problems in `smart-tree`

| Karp NP-Complete Problem | `smart-tree` Reduction & Algorithm | Complexity |
| :--- | :--- | :--- |
| **#3 Clique ($\omega(G)$)** | Degeneracy-ordered `BitSet256` Tomita Bron–Kerbosch | $O(d \cdot |V| \cdot 3^{d/3})$ FPT |
| **#4 Independent Set ($\alpha(G)$)** | Caro–Wei greedy minimum-degree anti-color $c_\emptyset$ | $O(|V| \log |V| + |E|)$ |
| **#2 Vertex Cover** | Gavril maximal-matching 2-Approximation (`vertex_cover_2approx_size`) | $O(|V| + |E|)$ |
| **#7 Feedback Arc Set** | Tournament degree-rank back-edge isolation (`feedback_back_edges`) | $O(k^2)$ per clique |
| **#11 Chromatic Number ($\chi$)** | Moser–Tardos Constructive LLL worktree solver (`moser_tardos.rs`) | $O(|\mathcal{U}| / \text{poly})$ |
| **#12 Clique Cover** | Szemerédi $\varepsilon$-regular partition (`super_nodes_k`) | $O(K^2 \cdot |E|)$ |
| **#15 Steiner Tree** | Mehlhorn Voronoi Dijkstra + Kruskal MST 2-Approximation | $O(|E| + |V|\log|V|)$ |
| **#18 Knapsack** | Erdős-distance radial shell packer (`knapsack_budget_utilization`) | $O(K \log K)$ |

---

## 3. CLI Invocation

```bash
# Run Ramsey & Erdős–Hajnal 256-Bit Bitboard + Bi-Clique + Karp-21 Audit:
st --ramsey-cliques .
```
