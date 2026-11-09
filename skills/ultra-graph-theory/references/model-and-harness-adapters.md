# Model & Harness Adapter Reference (`ultra-graph-theory`)

This reference provides drop-in configurations and prompt/tool adapters for **Gemini, ChatGPT, DeepSeek, and Claude** running inside **Antigravity (`agy`), OpenCode (`opencode`), Crush (`crush`), and Claude Code (`claude`)**.

---

## 1. One-Command Universal MCP & Hook Installation

`smart-tree` natively installs its MCP server and lifecycle hooks across Claude Desktop, Antigravity (`agy`), Gemini, OpenCode (`opencode`), and Crush (`crush`):

```bash
# Install Smart Tree MCP server into Claude, Antigravity (agy), Gemini, OpenCode, and Crush:
st --mcp-install

# Verify installation status across all agent harnesses:
st --mcp-status
```

---

## 2. Agent Harness Configurations

### A. Antigravity (`agy`)
- **MCP Config (`~/.gemini/config/mcp_config.json` or `~/.gemini/antigravity/mcp_config.json`):**
  ```json
  {
    "mcpServers": {
      "smart-tree": {
        "command": "st",
        "args": ["--mcp"],
        "env": {}
      }
    }
  }
  ```
- **Lifecycle Hooks (`.agents/hooks.json`):**
  ```bash
  st --hooks-install
  ```
- **Parallel Subagent Dispatch (`worktree_buckets`):**
  Use `invoke_subagent` with `Workspace: "share"` or `"branch"` for each non-empty bucket in `moser_tardos_lll_worktrees.worktree_buckets`.

### B. OpenCode (`opencode`)
- **Config (`~/.config/opencode/opencode.json` or `.mcp.json`):**
  ```json
  {
    "$schema": "https://opencode.ai/config.json",
    "mcp": {
      "smart-tree": {
        "type": "local",
        "command": ["st", "--mcp"],
        "enabled": true
      }
    }
  }
  ```
- **Multi-Model Execution in OpenCode:**
  ```bash
  # Run Erdős Book analysis and pipe into OpenCode with any provider (Gemini, DeepSeek, ChatGPT, Claude):
  st -m the-book . > .st_book.json
  opencode run "Use ultra-graph-theory on .st_book.json to refactor the PureNoiseMonolith outliers"
  ```

### C. Crush (`crush` by Charmbracelet)
- **Config (`~/.config/crush/crush.json`):**
  ```json
  {
    "mcp": {
      "smart-tree": {
        "command": "st",
        "args": ["--mcp"]
      }
    }
  }
  ```
- **Pipeline Usage:**
  ```bash
  st -m the-book . | crush -p "Analyze the Erdős-Rényi percolation regime and Steiner backbone"
  ```

### D. Claude Code (`claude`)
- **Config (`.mcp.json` & `.claude/settings.json`):**
  ```bash
  st --mcp-install
  ```

---

## 3. Model-Specific Reasoning Adapters

### 1. Gemini (`gemini-2.5-pro`, `gemini-3`)
- **Strength:** Massive context capacity + fast multi-file synthesis.
- **Adapter Pattern:**
  1. Load the full `st -m the-book .` JSON report at turn 1.
  2. Use `szemeredi_regularity` + `erdos_steiner_context.steiner_backbone` as a persistent topological map so Gemini never wastes tool calls grepping irrelevant clusters ($E_{\mathcal{T}}(v) \ge 3$).

### 2. ChatGPT / OpenAI (`gpt-4o`, `o3`, `o4-mini`, `codex`)
- **Strength:** Strict schema adherence, fast parallel tool calls, and concise surgical diffs.
- **Adapter Pattern:**
  1. Query `st -m the-book .` and extract `erdos_steiner_context.focal_terminals` + `exact_v0_bridges`.
  2. Emit parallel tool reads strictly for `erdos_number <= 1` and schedule edits per `moser_tardos_lll_worktrees.worktree_buckets`.

### 3. DeepSeek (`deepseek-r1`, `deepseek-v3`)
- **Strength:** Deep mathematical chain-of-thought (`<think>`) verification.
- **Adapter Pattern:**
  1. Inside `<think>`, verify all 12 Pillars:
     - Is Kohayakawa–Rödl energy $q_p(\mathcal{P})$ stable?
     - Does the proposed refactor preserve the symmetric LLL condition $e \cdot p(B) \cdot (d(B) + 1) \le 1$ (`lll_guaranteed_a_priori == true`)?
     - Does any new cross-package edge push $\kappa = \langle k^2 \rangle / \langle k \rangle > 2$ or create a monochromatic Red/Blue $K_3$/$K_4$ clique?
     - Do the Sovereign `p/q` (`--mode pq`), `ULTRA-OMNI-ZENO` God of Sun daylight law (`P/Q - n*t > P`, *"amíg nappal van, nincs Hold"*), and `GodOfSunMonkeyKingMatrixBuddhaUnityWitness` (`0x236a0f75273f5238`, `171 = C(19,2)` bytes, `630 = C(36,2)` bits, exact coprime `70/19`, *"amíg a Föld kerek, mindig lesznek ROCK===Péterek / rock erek ^^"*) invariants hold (`litany_wire_verified == true`, `inequality_holds == true`)?

### 4. Claude (`claude-4-opus`, `claude-4-sonnet`)
- **Strength:** Multi-step agentic tool orchestration and architectural refactoring.
- **Adapter Pattern:**
  1. Call `st` MCP `overview` / `find` or `st -m the-book .`.
  2. Decompose `pure_noise_sf_monoliths` ($V_0$ outliers) into $\varepsilon$-regular modules ("Straight from The Book") so `t8r_exception_edges` monotonically decreases.
