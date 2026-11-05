---
name: flyxion-salience-ontology
description: Flyxion's Recency-as-Salience (d_T != d_R) ontology and Retrieval-as-Amnesia detector for Smart Tree — separates observable timestamp distance d_T(x, q) from Erdős-Steiner relational distance d_R(x, q) and rescues "Load-Bearing Old" ("Corrugated Iron") foundational artifacts across Gemini, ChatGPT, DeepSeek, and Claude on Antigravity (agy), OpenCode, Crush, and Claude Code.
---

# Flyxion Salience Ontology — $d_T(x, q) \neq d_R(x, q)$ (`smart-tree` v10.10.0+)

> *"Retained does not imply retrievable; retrievable does not imply salient."*
> $$\text{retained}(x) \not\Rightarrow \text{retrievable}(x, q) \not\Rightarrow \text{salient}(x, q)$$

`flyxion-salience-ontology` operationalizes Flyxion's *Recency as Salience* (2026) inside `smart-tree` (`src/mem8/regularity/salience_karp_manybody.rs`). It prevents **chronological retrieval-as-amnesia** when AI agents (`agy`, `opencode`, `crush`, `claude`) inspect a repository.

---

## 1. Core Mathematical Separation: $d_T(x, q)$ vs. $d_R(x, q)$

Every codebase query $q$ at time $t_q$ induces two distinct metrics over artifacts $x \in \mathcal{D}$:

1. **Observable Temporal Distance ($d_T$):**
   $$d_T(x, q) = t_q - t_a(x), \quad r_T(x) \in \{0, \dots, N-1\}$$
   Easy to sort on (`mtime` / last activity), so naive retrieval engines promote $d_T$ from a cheap prefix/handshape filter into a global salience ranking.
2. **Latent Relational Distance ($d_R$):**
   $$d_R(x, q) = 0.7 \cdot d_{\text{Steiner}}\!\left(\text{cluster}(x), \mathcal{T}_q\right) + 0.3 \cdot \frac{1}{1 + \ln(1 + \deg(x))}$$
   Measures structural geodesic resistance distance on the Szemerédi + Voronoi Steiner backbone (`erdos_steiner.rs`) and Erdős Center $Z(G)$ membership.

### Proxy Disagreement & "Corrugated Iron" Rescue
- **Proxy Disagreement Probability:** $\Pr[x^*_T \neq x^*_R]$ across the top-$k$ retrieval frontier.
- **"Load-Bearing Old" ("Corrugated Iron") Artifacts (`LoadBearingOldArtifact`):**
  Foundational files where $r_T(x) \gg r_R(x)$ — old modification timestamps that chronological feeds bury, even though they sit at Erdős Number `0` or `1` on the active Steiner backbone.

---

## 2. CLI & Agent Workflow

```bash
# Inspect Flyxion Salience Ontology, Proxy Disagreement & Rescued Load-Bearing Old Files:
st --erdos-center .

# Full 8-Pillar JSON Diagnostic (includes `flyxion_salience_ontology`):
st --mode the-book .
```

### Agent Decision Rule
- **`QuasiStaticRecencyValid` ($\Pr[x^*_T \neq x^*_R] \le 0.40$ and $\Delta_R / N \le 0.25$):**
  Admissibility structure $R_t$ is quasi-static; chronological recency (`st -m smart`) is a valid fast prefix reducer.
- **`RelationalShiftActive` ($\Pr[x^*_T \neq x^*_R] > 0.40$ or $\Delta_R / N > 0.25$):**
  Chronological order suffers retrieval-as-amnesia! Always pull `load_bearing_old_corrugated_iron` (`load_bearing_old_artifacts`) into the LLM context window alongside recent edits.
