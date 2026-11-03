---
name: manybody-strang-mem8
description: Quantum Many-Body Kohn-Sham DFT Super-Node Orbital Compression, Local Natural Orbital (T8R) Correlations, and Second-Order Strang Split-Operator Salience Wave Propagation for Smart Tree across Gemini, ChatGPT, DeepSeek, and Claude on Antigravity (agy), OpenCode, Crush, and Claude Code.
---

# Quantum Many-Body Kohn–Sham DFT & Strang Wave Engine (`smart-tree` v10.10.0+)

`manybody-strang-mem8` (`src/mem8/regularity/salience_karp_manybody.rs`) bridges quantum computational chemistry and MEM8 wave dynamics to model $N$-symbol codebases without hitting the exponential $O(2^N)$ many-body wave-function wall.

---

## 1. Kohn–Sham DFT Super-Orbitals & Local Natural Orbital (`T8R`) Correlations

In quantum chemistry, storing an exact $N$-electron wave function $\Psi(\mathbf{r}_1, \dots, \mathbf{r}_N)$ scales as $O(2^N)$. **Kohn–Sham Density Functional Theory (DFT)** and **Linear-Scaling CCSD(T) (Local Natural Orbitals / QM-MM)** break this curse of dimensionality by separating the mean-field one-particle density $\rho(\mathbf{r})$ from localized strong correlations:

| Quantum Many-Body Concept | `smart-tree` MEM8 Equivalent | Complexity Reduction |
| :--- | :--- | :--- |
| **One-Particle Kohn–Sham Density $\rho(\mathbf{r})$** | `SuperNodeCentroidPod` (`[f32; 64]` L2-normalized wave centroid per Szemerédi cluster $V_i$) | $N$ symbols $\rightarrow K \ll N$ orbitals |
| **Hartree–Fock / Exchange Integrals $\langle \psi_i \vert \hat{V}_{\text{XC}} \vert \psi_j \rangle$** | `RegularSuperEdgePod` density $d_p(V_i, V_j) \times$ `felix_dot64_f32` SIMD wave overlap | $O(N^2) \rightarrow O(K^2)$ |
| **Local Natural Orbitals (LNO) / QM-MM Active Space** | `T8R` LEB128 Exception Ledger (`OutlierVertex`, `IrregularPair`, `ArchitecturalLeak`) | Exact $O(\vert E_{\text{T8R}}\vert)$ local correlation |

---

## 2. Second-Order Strang Split-Operator Salience Wave Propagation

Because the observable temporal drift operator $A_{\text{temporal}}$ ($d_T$ decay) and the relational Kohn–Sham coupling operator $B_{\text{relational}}$ ($d_R$ super-edge + `T8R` wave exchange) do not commute ($[A, B] \neq 0$), `smart-tree` evolves the $K$-orbital salience state $\Psi(t)$ via **Second-Order Strang Operator Splitting**:

$$\Psi(t + h) = e^{\frac{h}{2} A_{\text{temporal}}} \, e^{h B_{\text{relational}}} \, e^{\frac{h}{2} A_{\text{temporal}}} \Psi(t) + O(h^3)$$

- **`kohn_sham_exchange_energy`**: Total mean-field + LNO wave coupling energy across the reduced graph.
- **`strang_split_coherence`**: Inner-product phase stability $|\langle \Psi(0) \mid \Psi(h) \rangle| \in [0, 1]$.
- **`dominant_salience_orbitals`**: Top Szemerédi super-node clusters ranked by propagated Born probability $|\Psi_i(t + h)|^2$.

---

## 3. CLI Invocation

```bash
# Inspect Quantum Many-Body Kohn-Sham DFT & Strang Wave Orbitals:
st --erdos-center .
st --mode the-book .
```
