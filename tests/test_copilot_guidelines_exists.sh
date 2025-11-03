#!/usr/bin/env bash
set -euo pipefail

# Quick test: verify Copilot/MCP guideline files exist and contain expected hints
GFILE=".github/COPILOT_MCP_GUIDELINES.md"
RFILE=".github/COPILOT_REPO_INSTRUCTIONS.md"

if [[ ! -f "$GFILE" ]]; then
  echo "ERROR: $GFILE not found"
  exit 2
fi

if [[ ! -f "$RFILE" ]]; then
  echo "ERROR: $RFILE not found"
  exit 3
fi

# Basic content checks
grep -q "keyword" "$GFILE" || { echo "ERROR: 'keyword' sample not found in $GFILE"; exit 4; }
grep -q "include_content" "$GFILE" || { echo "ERROR: 'include_content' sample not found in $GFILE"; exit 5; }

echo "OK: guidelines present"
exit 0
