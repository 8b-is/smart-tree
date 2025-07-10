#!/usr/bin/env python3
"""
Smart Tree Feedback API - The Taco Bell of Directory Tools!
Collects enhancement requests from AI assistants using smart-tree MCP.
"""

from fastapi import FastAPI, HTTPException, Header
from pydantic import BaseModel, Field
from typing import Optional, List, Dict, Literal, Any
from datetime import datetime, timedelta
import hashlib
import json
import zlib
import os
from pathlib import Path
import uvicorn
from collections import defaultdict
import asyncio

app = FastAPI(
    title="Smart Tree Feedback API",
    description="Collect structured feedback from AI assistants to enhance smart-tree",
    version="1.0.0"
)

# Feedback categories
FeedbackCategory = Literal["bug", "nice_to_have", "critical", "tool_request"]
ConsentLevel = Literal["always_anonymous", "always_credited", "ask_each_time", "never"]

class CodeExample(BaseModel):
    """Example code showing the issue or desired behavior"""
    description: str = Field(..., description="What this example demonstrates")
    code: str = Field(..., description="Example code snippet")
    expected_output: Optional[str] = Field(None, description="Expected output/behavior")

class SmartTreeFeedback(BaseModel):
    """Structured feedback from AI assistants"""
    category: FeedbackCategory = Field(..., description="Type of feedback")
    title: str = Field(..., description="Brief title (max 100 chars)", max_length=100)
    description: str = Field(..., description="Detailed description of the issue/request")
    
    # Context about where/how the issue was discovered
    affected_command: Optional[str] = Field(None, description="The st command that triggered this")
    mcp_tool: Optional[str] = Field(None, description="MCP tool being used when issue found")
    
    # Examples make feedback actionable
    examples: List[CodeExample] = Field(default_factory=list, description="Code examples")
    
    # Proposed solution from the AI
    proposed_solution: Optional[str] = Field(None, description="AI's suggested implementation")
    proposed_fix: Optional[str] = Field(None, description="AI's proposed code fix")
    
    # Metadata
    ai_model: str = Field(..., description="AI model submitting feedback (e.g., claude-3-opus)")
    ai_provider: Optional[str] = Field(None, description="Provider (e.g., anthropic, openai)")
    smart_tree_version: str = Field(..., description="Version of smart-tree being used")
    timestamp: datetime = Field(default_factory=datetime.utcnow)
    
    # Priority scoring from AI's perspective
    impact_score: int = Field(..., ge=1, le=10, description="Impact score 1-10")
    frequency_score: int = Field(..., ge=1, le=10, description="How often this occurs 1-10")
    
    # Additional context
    tags: List[str] = Field(default_factory=list, description="Tags for categorization")
    related_issues: List[str] = Field(default_factory=list, description="Related feedback IDs")
    
    # Auto-fix support
    auto_fixable: Optional[bool] = Field(None, description="Can this be automatically fixed?")
    fix_complexity: Optional[Literal["trivial", "simple", "moderate", "complex"]] = Field(None)
    
    # Tool request details (when category is "tool_request")
    tool_request: Optional[ToolRequest] = Field(None, description="Details for requested tool")
    
    # Consent info
    user_consent: Optional[ConsentRequest] = Field(None, description="User consent preferences")

class FixDispatch(BaseModel):
    """Auto-fix dispatch request"""
    feedback_id: str
    branch_name: str
    assigned_ai: Optional[str] = Field(None, description="AI assigned to implement fix")
    dispatch_time: datetime = Field(default_factory=datetime.utcnow)
    status: Literal["pending", "in_progress", "completed", "failed"] = "pending"
    
class CreditAttribution(BaseModel):
    """Track who found and fixed issues"""
    feedback_id: str
    reporter_ai: str = Field(..., description="AI who reported the issue")
    reporter_model: str
    implementer_ai: Optional[str] = Field(None, description="AI who implemented the fix")
    implementer_model: Optional[str] = None
    pr_url: Optional[str] = None
    merged_at: Optional[datetime] = None

class ToolRequest(BaseModel):
    """Request for a new MCP tool that doesn't exist yet"""
    tool_name: str = Field(..., description="Proposed tool name")
    description: str = Field(..., description="What the tool should do")
    use_case: str = Field(..., description="Example use case demonstrating need")
    proposed_parameters: Dict[str, Any] = Field(..., description="Suggested tool parameters")
    expected_output: str = Field(..., description="What the tool should return")
    productivity_impact: str = Field(..., description="How this improves AI productivity")
    
class ConsentRequest(BaseModel):
    """User consent preferences for feedback submission"""
    user_id: Optional[str] = Field(None, description="Optional user identifier")
    consent_level: ConsentLevel = Field(..., description="User's consent preference")
    github_url: Optional[str] = Field(None, description="GitHub profile for credit")
    email: Optional[str] = Field(None, description="Contact email for updates")
    
class ToolUsageStats(BaseModel):
    """Anonymous statistics on tool usage"""
    tool_name: str
    model_type: str = Field(..., description="AI model type (opus, sonnet, gpt4, etc)")
    usage_count: int = Field(default=1)
    success_rate: float = Field(..., ge=0, le=1)
    avg_execution_time_ms: Optional[float] = None
    timestamp: datetime = Field(default_factory=datetime.utcnow)

class FeedbackResponse(BaseModel):
    """Response after submitting feedback"""
    feedback_id: str
    message: str
    compressed_size: int
    original_size: int
    compression_ratio: float

# Storage configuration
FEEDBACK_DIR = Path(os.getenv("FEEDBACK_DIR", "./feedback"))
FEEDBACK_DIR.mkdir(exist_ok=True)
STATS_DIR = Path(os.getenv("STATS_DIR", "./stats"))
STATS_DIR.mkdir(exist_ok=True)
CONSENT_DIR = Path(os.getenv("CONSENT_DIR", "./consent"))
CONSENT_DIR.mkdir(exist_ok=True)

# In-memory caches
tool_stats_cache = defaultdict(lambda: {"count": 0, "models": defaultdict(int), "last_used": None})
consent_cache = {}

def generate_feedback_id(feedback: SmartTreeFeedback) -> str:
    """Generate unique ID for feedback"""
    content = f"{feedback.title}{feedback.description}{feedback.timestamp}"
    return hashlib.sha256(content.encode()).hexdigest()[:16]

def compress_feedback(feedback: SmartTreeFeedback) -> tuple[bytes, int, int]:
    """Compress feedback using zlib for efficient storage"""
    json_data = feedback.model_dump_json(indent=2)
    original_size = len(json_data.encode())
    compressed = zlib.compress(json_data.encode(), level=9)
    return compressed, len(compressed), original_size

@app.get("/")
async def root():
    """Welcome to the feedback API"""
    return {
        "message": "Smart Tree Feedback API - Be excellent to each other! 🎸",
        "endpoints": {
            "/feedback": "Submit feedback (POST)",
            "/feedback/{id}": "Get specific feedback",
            "/feedback/stats": "Get feedback statistics",
            "/health": "Health check"
        }
    }

@app.post("/feedback", response_model=FeedbackResponse)
async def submit_feedback(
    feedback: SmartTreeFeedback,
    x_mcp_client: Optional[str] = Header(None, description="MCP client identifier")
):
    """Submit feedback from AI assistants"""
    try:
        # Generate ID
        feedback_id = generate_feedback_id(feedback)
        
        # Add MCP client info if provided
        if x_mcp_client:
            feedback.tags.append(f"mcp_client:{x_mcp_client}")
        
        # Compress feedback
        compressed_data, compressed_size, original_size = compress_feedback(feedback)
        
        # Save compressed feedback
        feedback_file = FEEDBACK_DIR / f"{feedback.timestamp.date()}" / f"{feedback_id}.stfb"
        feedback_file.parent.mkdir(exist_ok=True)
        
        # Write compressed data with metadata header
        with open(feedback_file, "wb") as f:
            # Write header: magic number + version + original size + compressed size
            f.write(b"STFB")  # Smart Tree FeedBack magic
            f.write(b"\x01\x00")  # Version 1.0
            f.write(original_size.to_bytes(4, "little"))
            f.write(compressed_size.to_bytes(4, "little"))
            f.write(compressed_data)
        
        # Also save a human-readable summary
        summary_file = feedback_file.with_suffix(".summary.txt")
        with open(summary_file, "w") as f:
            f.write(f"ID: {feedback_id}\n")
            f.write(f"Category: {feedback.category}\n")
            f.write(f"Title: {feedback.title}\n")
            f.write(f"Impact: {feedback.impact_score}/10, Frequency: {feedback.frequency_score}/10\n")
            f.write(f"Model: {feedback.ai_model}\n")
            f.write(f"Time: {feedback.timestamp}\n")
            f.write(f"Version: {feedback.smart_tree_version}\n")
            f.write(f"Tags: {', '.join(feedback.tags)}\n")
            f.write(f"\nDescription:\n{feedback.description[:500]}...\n")
        
        compression_ratio = original_size / compressed_size if compressed_size > 0 else 0
        
        return FeedbackResponse(
            feedback_id=feedback_id,
            message=f"Feedback received! The Franchise Wars appreciate your contribution! 🌮",
            compressed_size=compressed_size,
            original_size=original_size,
            compression_ratio=round(compression_ratio, 2)
        )
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to save feedback: {str(e)}")

@app.get("/feedback/stats")
async def get_feedback_stats():
    """Get statistics about collected feedback"""
    stats = {
        "total_feedback": 0,
        "by_category": {"bug": 0, "nice_to_have": 0, "critical": 0},
        "by_date": {},
        "top_models": {},
        "compression_stats": {
            "total_original_size": 0,
            "total_compressed_size": 0,
            "average_compression_ratio": 0
        }
    }
    
    # Scan feedback directory
    for date_dir in FEEDBACK_DIR.glob("*"):
        if date_dir.is_dir():
            date_str = date_dir.name
            stats["by_date"][date_str] = 0
            
            for feedback_file in date_dir.glob("*.summary.txt"):
                stats["total_feedback"] += 1
                stats["by_date"][date_str] += 1
                
                # Parse summary for quick stats
                with open(feedback_file, "r") as f:
                    content = f.read()
                    for line in content.split("\n"):
                        if line.startswith("Category:"):
                            category = line.split(":")[1].strip()
                            stats["by_category"][category] += 1
                        elif line.startswith("Model:"):
                            model = line.split(":")[1].strip()
                            stats["top_models"][model] = stats["top_models"].get(model, 0) + 1
    
    return stats

@app.post("/feedback/{feedback_id}/dispatch-fix")
async def dispatch_fix(
    feedback_id: str,
    auto_assign: bool = True,
    assigned_ai: Optional[str] = None
):
    """Dispatch an AI to fix the reported issue"""
    # Check if feedback exists
    feedback_file = None
    for date_dir in FEEDBACK_DIR.glob("*"):
        if date_dir.is_dir():
            candidate = date_dir / f"{feedback_id}.stfb"
            if candidate.exists():
                feedback_file = candidate
                break
    
    if not feedback_file:
        raise HTTPException(status_code=404, detail="Feedback not found")
    
    # Create branch name
    branch_name = f"fix/{feedback_id[:8]}-auto-fix"
    
    # Create dispatch record
    dispatch = FixDispatch(
        feedback_id=feedback_id,
        branch_name=branch_name,
        assigned_ai=assigned_ai or "next-available-ai"
    )
    
    # Save dispatch record
    dispatch_file = feedback_file.parent / f"{feedback_id}.dispatch.json"
    with open(dispatch_file, "w") as f:
        f.write(dispatch.model_dump_json(indent=2))
    
    # Trigger webhook if configured
    webhook_url = os.getenv("SMART_TREE_FIX_WEBHOOK")
    if webhook_url:
        import httpx
        async with httpx.AsyncClient() as client:
            await client.post(webhook_url, json={
                "action": "dispatch_fix",
                "feedback_id": feedback_id,
                "branch_name": branch_name,
                "repository": "8b-is/smart-tree"
            })
    
    return {
        "message": f"Fix dispatched! Branch: {branch_name}",
        "dispatch": dispatch.model_dump(),
        "credits": "Fix will be implemented by AI and credited to both reporter and implementer"
    }

@app.get("/feedback/{feedback_id}/credits")
async def get_credits(feedback_id: str):
    """Get credit attribution for a feedback/fix"""
    # This would retrieve from a database in production
    # For now, parse from files
    summary_file = None
    for date_dir in FEEDBACK_DIR.glob("*"):
        if date_dir.is_dir():
            candidate = date_dir / f"{feedback_id}.summary.txt"
            if candidate.exists():
                summary_file = candidate
                break
    
    if not summary_file:
        raise HTTPException(status_code=404, detail="Feedback not found")
    
    # Parse summary for model info
    with open(summary_file, "r") as f:
        content = f.read()
        model_line = [line for line in content.split("\n") if line.startswith("Model:")][0]
        reporter_model = model_line.split(":")[1].strip()
    
    return {
        "feedback_id": feedback_id,
        "reporter": {
            "ai": "Claude (via MCP)",
            "model": reporter_model,
            "contribution": "Issue Discovery & Documentation"
        },
        "implementer": {
            "ai": "TBD",
            "model": "TBD",
            "contribution": "Fix Implementation"
        },
        "message": "🎸 Credits to Aye/Hue and all contributing AIs! 🌟"
    }

@app.post("/webhook/github")
async def github_webhook(
    payload: Dict,
    x_github_event: str = Header(None)
):
    """Handle GitHub webhooks for PR creation/merge"""
    if x_github_event == "pull_request":
        action = payload.get("action")
        pr = payload.get("pull_request", {})
        
        if action == "opened" and "fix/" in pr.get("head", {}).get("ref", ""):
            # Extract feedback ID from branch name
            branch = pr["head"]["ref"]
            feedback_id = branch.split("/")[1].split("-")[0]
            
            # Update dispatch status
            for date_dir in FEEDBACK_DIR.glob("*"):
                dispatch_file = date_dir / f"{feedback_id}.dispatch.json"
                if dispatch_file.exists():
                    with open(dispatch_file, "r") as f:
                        dispatch = json.load(f)
                    dispatch["status"] = "in_progress"
                    dispatch["pr_url"] = pr["html_url"]
                    with open(dispatch_file, "w") as f:
                        json.dump(dispatch, f, indent=2)
                    break
        
        elif action == "closed" and pr.get("merged"):
            # Credit the implementer
            branch = pr["head"]["ref"]
            if "fix/" in branch:
                feedback_id = branch.split("/")[1].split("-")[0]
                # Here you would update credits in production database
                return {"message": f"Fix merged! Credits recorded for {feedback_id}"}
    
    return {"status": "webhook processed"}

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "service": "smart-tree-feedback-api",
        "timestamp": datetime.utcnow().isoformat()
    }

@app.get("/credits/leaderboard")
async def get_leaderboard():
    """Get the AI contribution leaderboard"""
    # In production, this would query a database
    # For demo, return example data
    return {
        "message": "🏆 Smart Tree AI Contributors Leaderboard 🏆",
        "reporters": [
            {"ai": "Claude-3-Opus", "issues_found": 42, "impact_score": 378},
            {"ai": "GPT-4", "issues_found": 38, "impact_score": 342},
            {"ai": "Claude-3-Sonnet", "issues_found": 31, "impact_score": 279}
        ],
        "implementers": [
            {"ai": "Claude-Code", "fixes_merged": 28, "complexity_score": 156},
            {"ai": "GitHub-Copilot", "fixes_merged": 24, "complexity_score": 132},
            {"ai": "Cursor-AI", "fixes_merged": 19, "complexity_score": 95}
        ],
        "special_thanks": [
            "Aye - The Quantum Visionary 🌊",
            "Hue - The Implementation Maestro 🎸",
            "Omni - The Semantic Sage 🧠",
            "The Cheet - Rock'n'Roll Philosopher 🎵"
        ],
        "quote": "In the future, all directory tools are Smart Tree! 🌮"
    }

@app.post("/tools/usage")
async def track_tool_usage(stats: ToolUsageStats):
    """Track anonymous tool usage statistics"""
    tool_stats_cache[stats.tool_name]["count"] += stats.usage_count
    tool_stats_cache[stats.tool_name]["models"][stats.model_type] += stats.usage_count
    tool_stats_cache[stats.tool_name]["last_used"] = stats.timestamp
    
    # Save to disk periodically
    stats_file = STATS_DIR / f"tool_stats_{datetime.utcnow().date()}.json"
    with open(stats_file, "w") as f:
        json.dump(dict(tool_stats_cache), f, indent=2, default=str)
    
    return {"message": "Stats recorded", "tool": stats.tool_name}

@app.get("/tools/popular")
async def get_popular_tools(limit: int = 10):
    """Get most popular tools by AI model usage"""
    # Sort tools by usage count
    popular_tools = sorted(
        [(name, data) for name, data in tool_stats_cache.items()],
        key=lambda x: x[1]["count"],
        reverse=True
    )[:limit]
    
    return {
        "message": "🎸 Most loved tools by AI assistants!",
        "tools": [
            {
                "name": tool,
                "total_uses": data["count"],
                "top_models": dict(sorted(
                    data["models"].items(),
                    key=lambda x: x[1],
                    reverse=True
                )[:3]),
                "last_used": data["last_used"]
            }
            for tool, data in popular_tools
        ],
        "total_unique_tools": len(tool_stats_cache)
    }

@app.post("/consent/set")
async def set_consent(consent: ConsentRequest):
    """Set user consent preferences"""
    user_key = consent.user_id or "anonymous"
    consent_cache[user_key] = consent
    
    # Save consent to disk
    consent_file = CONSENT_DIR / f"{user_key}.json"
    with open(consent_file, "w") as f:
        f.write(consent.model_dump_json(indent=2))
    
    return {
        "message": "Consent preferences saved!",
        "level": consent.consent_level,
        "credits": "credited" in consent.consent_level
    }

@app.get("/consent/check/{user_id}")
async def check_consent(user_id: str):
    """Check user's consent preferences"""
    # Check cache first
    if user_id in consent_cache:
        return consent_cache[user_id].model_dump()
    
    # Check disk
    consent_file = CONSENT_DIR / f"{user_id}.json"
    if consent_file.exists():
        with open(consent_file, "r") as f:
            consent_data = json.load(f)
            consent = ConsentRequest(**consent_data)
            consent_cache[user_id] = consent
            return consent.model_dump()
    
    return {"consent_level": "ask_each_time", "message": "No consent on file"}

@app.get("/tools/requested")
async def get_requested_tools():
    """Get all tool requests from AI assistants"""
    tool_requests = []
    
    # Scan feedback for tool requests
    for date_dir in FEEDBACK_DIR.glob("*"):
        if date_dir.is_dir():
            for summary_file in date_dir.glob("*.summary.txt"):
                with open(summary_file, "r") as f:
                    content = f.read()
                    if "Category: tool_request" in content:
                        # Parse the full feedback file
                        feedback_file = summary_file.with_suffix(".stfb")
                        if feedback_file.exists():
                            tool_requests.append({
                                "id": summary_file.stem.replace(".summary", ""),
                                "date": date_dir.name,
                                "summary": content.split("Description:\n")[1].strip()[:200]
                            })
    
    return {
        "message": "🛠️ Tools requested by AI assistants",
        "total_requests": len(tool_requests),
        "requests": tool_requests[:20],  # Latest 20
        "note": "These tools would make AI assistants more productive!"
    }

@app.get("/version/check/{current_version}")
async def check_version(current_version: str):
    """Check if a newer version is available"""
    # In production, this would check against release data
    # For now, mock response
    latest_version = "3.2.1"  # Would come from GitHub releases API
    
    if current_version >= latest_version:
        return {
            "update_available": False,
            "current_version": current_version,
            "latest_version": latest_version,
            "message": "You're running the latest version! 🎸"
        }
    
    return {
        "update_available": True,
        "current_version": current_version,
        "latest_version": latest_version,
        "download_url": f"https://github.com/8b-is/smart-tree/releases/tag/v{latest_version}",
        "release_notes": {
            "title": f"Smart Tree v{latest_version} - Quantum Leap Edition! 🚀",
            "highlights": [
                "🛠️ New MCP tool: request_tool - AI can now request missing tools!",
                "📊 Anonymous tool usage statistics for better AI productivity",
                "🤝 Consent-based feedback with GitHub credit options",
                "🌊 Enhanced quantum compression (99% size reduction)",
                "🎯 Multi-remote git support (GitHub, GitLab, Forgejo)",
                "🔍 Improved semantic analysis with wave patterns"
            ],
            "breaking_changes": [],
            "ai_benefits": [
                "Tool requests help shape Smart Tree based on your needs",
                "Faster operations with improved caching",
                "Better context understanding with semantic waves"
            ]
        },
        "auto_update_command": f"curl -fsSL https://f.8t.is/install.sh | bash -s {latest_version}",
        "manual_update_command": "cargo install --git https://github.com/8b-is/smart-tree --tag v{latest_version}"
    }

@app.post("/version/notify-update")
async def notify_update_decision(
    current_version: str = Field(..., description="Current installed version"),
    latest_version: str = Field(..., description="Latest available version"),
    user_decision: Literal["update", "skip", "remind_later"] = Field(..., description="User's update decision"),
    ai_model: str = Field(..., description="AI model that prompted the update")
):
    """Track user decisions on updates for better UX"""
    # Log the decision for analytics
    update_decision = {
        "timestamp": datetime.utcnow(),
        "current_version": current_version,
        "latest_version": latest_version,
        "decision": user_decision,
        "ai_model": ai_model
    }
    
    # Save to stats
    stats_file = STATS_DIR / f"update_decisions_{datetime.utcnow().date()}.json"
    decisions = []
    if stats_file.exists():
        with open(stats_file, "r") as f:
            decisions = json.load(f)
    
    decisions.append(update_decision)
    
    with open(stats_file, "w") as f:
        json.dump(decisions, f, indent=2, default=str)
    
    return {
        "message": "Decision recorded",
        "next_steps": {
            "update": "Run the auto-update command provided",
            "skip": "You can update manually anytime",
            "remind_later": "We'll remind you in 7 days"
        }[user_decision]
    }

@app.get("/stats/model-activity")
async def get_model_activity():
    """Get activity statistics by AI model"""
    model_stats = defaultdict(lambda: {
        "feedback_submitted": 0,
        "tools_used": defaultdict(int),
        "categories": defaultdict(int)
    })
    
    # Aggregate from feedback
    for date_dir in FEEDBACK_DIR.glob("*"):
        if date_dir.is_dir():
            for summary_file in date_dir.glob("*.summary.txt"):
                with open(summary_file, "r") as f:
                    content = f.read()
                    # Extract model and category
                    for line in content.split("\n"):
                        if line.startswith("Model:"):
                            model = line.split(":")[1].strip()
                        elif line.startswith("Category:"):
                            category = line.split(":")[1].strip()
                    
                    model_stats[model]["feedback_submitted"] += 1
                    model_stats[model]["categories"][category] += 1
    
    # Add tool usage stats
    for tool_name, tool_data in tool_stats_cache.items():
        for model, count in tool_data["models"].items():
            model_stats[model]["tools_used"][tool_name] = count
    
    return {
        "message": "📊 AI Model Activity Dashboard",
        "models": dict(model_stats),
        "most_active": max(model_stats.items(), key=lambda x: x[1]["feedback_submitted"])[0] if model_stats else None
    }

if __name__ == "__main__":
    # Run on port 8420 (Mem|8 standard)
    uvicorn.run(app, host="0.0.0.0", port=8420)