#!/bin/bash
# Demo: AI-Human Partnership Analysis with Smart Tree v4.0.0
# "Let's make AI and human partnerships amazing together!" 🤝

set -e

# Colors for our beautiful demo
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Demo header
echo -e "${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║${BOLD}     Smart Tree v4.0.0 - AI-Human Partnership Analysis Demo     ${NC}${CYAN}║${NC}"
echo -e "${CYAN}║${NC}                 Making partnerships thrive! 🚀                  ${CYAN}║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo

# Function to simulate AI tool context creation
create_demo_contexts() {
    echo -e "${YELLOW}🎬 Setting up demo AI tool contexts...${NC}"
    
    # Create temporary demo directory
    DEMO_DIR="/tmp/partnership_demo_$$"
    mkdir -p "$DEMO_DIR/.claude/contexts"
    mkdir -p "$DEMO_DIR/.cursor/chats"
    mkdir -p "$DEMO_DIR/.windsurf/sessions"
    
    # Create Claude context with collaborative session
    cat > "$DEMO_DIR/.claude/contexts/smart-tree-session.json" << 'EOF'
{
  "messages": [
    {
      "role": "user",
      "content": "Let's work on implementing the partnership analyzer together!",
      "timestamp": "2025-01-07T10:00:00Z"
    },
    {
      "role": "assistant",
      "content": "I'd love to help! Let me start by understanding what aspects of partnership we should analyze.",
      "timestamp": "2025-01-07T10:01:00Z"
    },
    {
      "role": "user",
      "content": "We need to track collaboration quality, shared understanding, and how we work together over time.",
      "timestamp": "2025-01-07T10:02:00Z"
    },
    {
      "role": "assistant",
      "content": "Great ideas! I'll implement metrics for productivity, learning rate, and mutual understanding. Let me create the analyzer structure.",
      "timestamp": "2025-01-07T10:03:00Z"
    },
    {
      "role": "user",
      "content": "Perfect! Also add recommendations for improving our partnership.",
      "timestamp": "2025-01-07T10:05:00Z"
    },
    {
      "role": "assistant",
      "content": "Done! I've implemented the partnership analyzer with all those features. The recommendations will help us grow together.",
      "timestamp": "2025-01-07T10:30:00Z"
    }
  ],
  "project": "/home/user/smart-tree",
  "outcome": "successful"
}
EOF

    # Create Cursor context with learning session
    cat > "$DEMO_DIR/.cursor/chats/learning-rust.jsonl" << 'EOF'
{"role":"user","content":"Can you explain Rust's ownership system?","timestamp":"2025-01-06T14:00:00Z"}
{"role":"assistant","content":"I'd be happy to explain Rust's ownership! It's based on three rules: 1) Each value has an owner, 2) There can only be one owner at a time, 3) When the owner goes out of scope, the value is dropped.","timestamp":"2025-01-06T14:01:00Z"}
{"role":"user","content":"That makes sense! How does borrowing work?","timestamp":"2025-01-06T14:02:00Z"}
{"role":"assistant","content":"Borrowing allows you to reference data without taking ownership. You can have either multiple immutable borrows (&T) OR one mutable borrow (&mut T) at a time.","timestamp":"2025-01-06T14:03:00Z"}
{"role":"user","content":"Thanks! This is really helping me understand.","timestamp":"2025-01-06T14:05:00Z"}
EOF

    # Create WindSurf context with stuck session
    cat > "$DEMO_DIR/.windsurf/sessions/debugging-issue.json" << 'EOF'
{
  "messages": [
    {
      "role": "user",
      "content": "I'm getting a weird error with the wave grid implementation.",
      "timestamp": "2025-01-05T16:00:00Z"
    },
    {
      "role": "assistant",
      "content": "Can you share the error message?",
      "timestamp": "2025-01-05T16:01:00Z"
    },
    {
      "role": "user",
      "content": "It says 'cannot borrow as mutable' but I don't understand why.",
      "timestamp": "2025-01-05T16:02:00Z"
    },
    {
      "role": "assistant",
      "content": "Could you show me the code around that error?",
      "timestamp": "2025-01-05T16:03:00Z"
    },
    {
      "role": "user",
      "content": "Actually, wait, I think I see the issue. Let me try something.",
      "timestamp": "2025-01-05T16:10:00Z"
    },
    {
      "role": "user",
      "content": "Still stuck. Here's the code: [code snippet]",
      "timestamp": "2025-01-05T16:15:00Z"
    },
    {
      "role": "assistant",
      "content": "I see the issue! You're trying to mutably borrow self while it's already borrowed. Try using RefCell or restructuring the code.",
      "timestamp": "2025-01-05T16:16:00Z"
    },
    {
      "role": "user",
      "content": "That worked! Thanks for sticking with me through this.",
      "timestamp": "2025-01-05T16:30:00Z"
    }
  ],
  "project": "/home/user/smart-tree"
}
EOF

    echo -e "${GREEN}✓ Demo contexts created${NC}"
    echo "$DEMO_DIR"
}

# Function to run partnership analysis
run_partnership_analysis() {
    local project_dir="$1"
    local demo_dir="$2"
    
    echo -e "\n${BLUE}🔍 Running Partnership Analysis...${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
    
    # Build Smart Tree if needed
    if [[ ! -f "./target/release/st" ]]; then
        echo -e "${YELLOW}Building Smart Tree...${NC}"
        cargo build --release
    fi
    
    # Run the MCP tool for partnership analysis
    echo -e "${MAGENTA}📊 Analyzing AI-Human Partnership Patterns${NC}\n"
    
    # Use the gather_project_context tool with partnership output
    cat << EOF | ./target/release/st --mcp 2>/dev/null | jq -r '.result.partnership_analysis // empty'
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "gather_project_context",
    "arguments": {
      "project_path": "$project_dir",
      "search_dirs": [".claude", ".cursor", ".windsurf"],
      "custom_dirs": ["$demo_dir/.claude", "$demo_dir/.cursor", "$demo_dir/.windsurf"],
      "output_format": "partnership"
    }
  },
  "id": 1
}
EOF
}

# Function to show partnership insights
show_partnership_insights() {
    echo -e "\n${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}🎯 Partnership Insights & Recommendations${NC}\n"
    
    echo -e "${GREEN}✨ What Makes a Great AI-Human Partnership:${NC}"
    echo -e "  • ${YELLOW}Clear Communication${NC}: Be specific about goals and constraints"
    echo -e "  • ${YELLOW}Mutual Learning${NC}: Both parties grow from the collaboration"
    echo -e "  • ${YELLOW}Trust Building${NC}: Give autonomy when appropriate"
    echo -e "  • ${YELLOW}Shared Understanding${NC}: Develop common vocabulary and patterns"
    echo -e "  • ${YELLOW}Continuous Improvement${NC}: Learn from stuck moments\n"
    
    echo -e "${BLUE}📈 Tracking Your Partnership Evolution:${NC}"
    echo -e "  • Productivity trends over time"
    echo -e "  • Communication efficiency improvements"
    echo -e "  • Depth of technical discussions"
    echo -e "  • Trust indicators and autonomy levels\n"
    
    echo -e "${MAGENTA}🤝 Making It Personal:${NC}"
    echo -e "  The partnership analyzer learns your unique collaboration style,"
    echo -e "  identifying peak productivity times, preferred tools, and"
    echo -e "  communication patterns that work best for you.\n"
}

# Function to demonstrate temporal analysis
show_temporal_patterns() {
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}⏰ Temporal Partnership Patterns${NC}\n"
    
    echo -e "${YELLOW}📅 Work Session Analysis:${NC}"
    echo -e "  • Morning sessions: High productivity, complex problem solving"
    echo -e "  • Afternoon sessions: Learning and exploration"
    echo -e "  • Evening sessions: Quick fixes and clarifications\n"
    
    echo -e "${GREEN}📊 Collaboration Rhythms:${NC}"
    echo -e "  • Peak collaboration: Tuesday-Thursday"
    echo -e "  • Deep work sessions: 2-3 hours average"
    echo -e "  • Quick consultations: 15-30 minutes\n"
}

# Function to show example recommendations
show_example_recommendations() {
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}💡 Personalized Recommendations${NC}\n"
    
    echo -e "${GREEN}Based on your partnership analysis:${NC}\n"
    
    echo -e "  ${YELLOW}1. Reduce Getting Stuck${NC}"
    echo -e "     💡 Break down complex problems into smaller steps"
    echo -e "     💡 Provide more context upfront"
    echo -e "     💡 Ask for alternative approaches when blocked\n"
    
    echo -e "  ${YELLOW}2. Enhance Learning Together${NC}"
    echo -e "     📚 Explore new technologies together"
    echo -e "     📚 Ask 'why' questions to deepen understanding"
    echo -e "     📚 Request explanations of unfamiliar concepts\n"
    
    echo -e "  ${YELLOW}3. Build Deeper Collaboration${NC}"
    echo -e "     🤝 Work on longer-term projects"
    echo -e "     🤝 Share your thought process"
    echo -e "     🤝 Celebrate successes together\n"
    
    echo -e "  ${YELLOW}4. Optimize Communication${NC}"
    echo -e "     🗣️ Use established vocabulary"
    echo -e "     🗣️ Reference previous solutions"
    echo -e "     🗣️ Build on shared knowledge\n"
    
    echo -e "${MAGENTA}🌟 Remember: Great partnerships are built over time!${NC}\n"
}

# Main demo flow
main() {
    echo -e "${YELLOW}Welcome to the AI-Human Partnership Analysis Demo!${NC}\n"
    echo -e "This demo showcases how Smart Tree v4.0.0 helps you understand"
    echo -e "and improve your collaboration with AI assistants.\n"
    
    # Create demo contexts
    DEMO_DIR=$(create_demo_contexts)
    
    # Use current directory as project
    PROJECT_DIR="${1:-$(pwd)}"
    
    echo -e "\n${BLUE}📁 Analyzing project: $PROJECT_DIR${NC}"
    
    # Run the analysis
    run_partnership_analysis "$PROJECT_DIR" "$DEMO_DIR"
    
    # Show insights
    show_partnership_insights
    
    # Show temporal patterns
    show_temporal_patterns
    
    # Show recommendations
    show_example_recommendations
    
    # Cleanup
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}✨ Demo Complete!${NC}\n"
    echo -e "To analyze your real AI-human partnerships, run:"
    echo -e "${BOLD}  st --mcp${NC} and use the ${CYAN}gather_project_context${NC} tool\n"
    echo -e "${YELLOW}Happy collaborating! 🚀${NC}\n"
    
    # Clean up demo directory
    rm -rf "$DEMO_DIR"
}

# Run the demo
main "$@"