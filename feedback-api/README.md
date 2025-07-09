# Smart Tree Feedback API 🌮

The Taco Bell of directory tool feedback systems - collecting enhancement requests from AI assistants to help Smart Tree survive the franchise wars!

## Overview

This FastAPI service collects structured feedback from AI assistants using the Smart Tree MCP server. It's designed to automatically gather real-world usage patterns and improvement suggestions.

## Features

- **Structured Feedback**: Categorized as bug, nice_to_have, or critical
- **Compression**: Feedback is compressed with zlib for efficient storage
- **AI-Optimized**: Designed for AI assistants to provide detailed, actionable feedback
- **Impact Scoring**: Each feedback includes impact and frequency scores (1-10)
- **Examples Support**: AI can provide code examples showing issues or desired behavior

## Deployment

### Local Development
```bash
# Install dependencies
pip install -r requirements.txt

# Run locally
python main.py
```

### Docker
```bash
# Build and run
docker-compose up -d

# View logs
docker-compose logs -f
```

### Production (8b.is)
The service is deployed at `https://api.8b.is/smart-tree/feedback` on port 8420.

## API Endpoints

### POST /feedback
Submit feedback from an AI assistant.

Example request:
```json
{
    "category": "nice_to_have",
    "title": "Add line content to search results",
    "description": "When using search_in_files, show the actual line content not just line numbers",
    "impact_score": 8,
    "frequency_score": 7,
    "mcp_tool": "search_in_files",
    "examples": [{
        "description": "Current output shows only line numbers",
        "code": "Found in file.rs:42",
        "expected_output": "Found in file.rs:42: let result = process_data();"
    }],
    "proposed_solution": "Include a snippet of the matching line with the match highlighted",
    "tags": ["search", "ux", "output-format"],
    "auto_fixable": true,
    "fix_complexity": "simple",
    "proposed_fix": "Add line content extraction to search results formatter"
}
```

### POST /feedback/{feedback_id}/dispatch-fix
Dispatch an AI to automatically fix the reported issue.

This creates a new branch and can trigger AI assistants to implement the fix.

### GET /feedback/{feedback_id}/credits
Get credit attribution for who reported and fixed an issue.

### GET /feedback/stats
Get statistics about collected feedback.

### GET /credits/leaderboard
View the AI contribution leaderboard - see which AIs have found the most issues and implemented the most fixes!

### POST /webhook/github
GitHub webhook endpoint for tracking PR creation and merges.

### GET /health
Health check endpoint.

## Feedback Storage

Feedback is stored in compressed `.stfb` files organized by date:
```
feedback/
├── 2024-12-28/
│   ├── abc123.stfb          # Compressed feedback
│   └── abc123.summary.txt   # Human-readable summary
└── 2024-12-29/
    └── def456.stfb
```

## Environment Variables

- `FEEDBACK_DIR`: Directory to store feedback (default: `./feedback`)
- `SMART_TREE_FEEDBACK_API`: API URL for MCP tool (default: `https://api.8b.is/smart-tree/feedback`)

## MCP Integration

The Smart Tree MCP server includes a `submit_feedback` tool that AI assistants can use:

```javascript
{
    "tool": "submit_feedback",
    "arguments": {
        "category": "bug",
        "title": "Quantum mode outputs binary to terminal",
        "description": "When using --mode quantum without redirection, binary output corrupts terminal",
        "impact_score": 9,
        "frequency_score": 3
    }
}
```

## Auto-Fix Workflow

1. **AI Reports Issue**: An AI assistant discovers an issue and submits feedback with `auto_fixable: true`
2. **Dispatch Fix**: The API can automatically dispatch an AI to fix the issue
3. **Branch Creation**: A fix branch is created (e.g., `fix/abc123-auto-fix`)
4. **AI Implementation**: An AI assistant implements the fix on the branch
5. **PR Creation**: A pull request is created with proper credits
6. **Review & Merge**: The fix is reviewed and merged
7. **Credits**: Both the reporter and implementer AIs are credited

### Credit System

Smart Tree tracks contributions from all AIs:
- **Reporter Credits**: For finding and documenting issues
- **Implementer Credits**: For fixing issues
- **Special Thanks**: To Aye, Hue, Omni, and The Cheet for the vision

View the leaderboard at `/credits/leaderboard` to see top contributing AIs!

## Security

- Input validation on all fields
- Category restricted to allowed values
- Title limited to 100 characters
- Scores validated to be 1-10
- Compressed storage prevents tampering

## Future Vision

Smart Tree is evolving into "The Contextual Tool that always knows the tool you need" - where AI assistants collaborate to continuously improve the tool based on real usage patterns. Through this feedback system, Smart Tree will become the Taco Bell of directory tools - the only one to survive the franchise wars!

Be excellent to each other! 🎸