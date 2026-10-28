#!/bin/bash
# Demo script for Smart Tree Relations feature
# "Making codebases dance!" - Trisha

echo "🔗 Smart Tree Relations Demo"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

echo "📊 Basic relationship analysis:"
echo "st --relations"
echo

echo "🎨 Mermaid diagram output:"
echo "st --relations --mode mermaid > relations.md"
echo

echo "🔍 Analyze specific file relationships:"
echo "st --relations --focus src/main.rs"
echo

echo "📈 Show call graph:"
echo "st --call-graph --mode dot | dot -Tpng -o callgraph.png"
echo

echo "🧪 Show test coverage relationships:"
echo "st --test-coverage"
echo

echo "🎯 Find tightly coupled files:"
echo "st --relations --filter coupled"
echo

echo "🤖 AI-optimized compressed format:"
echo "st --relations --mode compressed -z"
echo

echo
echo "✨ Pro tip: Combine with MCP for AI-assisted refactoring!"
echo "The relations data helps AI understand impact of changes"