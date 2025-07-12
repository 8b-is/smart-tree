#!/usr/bin/env python3
"""
🌟 Unified MCP Server for Tmux AI Assistant 🌟
Supports both OpenAI MCP (for deep research) and standard MCP protocols!

This beauty serves tmux monitoring data through:
1. OpenAI MCP: search & fetch for deep research in ChatGPT
2. Standard MCP: full toolkit for Claude, Cursor, and other MCP clients

Aye, Hue, and Trisha's masterpiece! 🎉
"""

import json
import asyncio
import logging
from datetime import datetime
from typing import List, Dict, Any, Optional

# Standard MCP imports
from mcp.server.fastmcp import FastMCP

# For OpenAI MCP (HTTP/SSE)
from fastapi import FastAPI, Request
from sse_starlette.sse import EventSourceResponse
import uvicorn

# Our tmux monitoring imports
from tmux_monitor import TmuxAIMonitor
import libtmux
from dotenv import load_dotenv  # For loading API keys from .env
from colorama import Fore  # For the colorful error message

# Setup logging - keeping it colorful for Trish! 🌈
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)

# Load environment variables - because secrets are like Trish's favorite snacks, best kept hidden!
load_dotenv()

# Standard MCP server instance
mcp = FastMCP("Tmux AI Assistant MCP")

# OpenAI MCP FastAPI app
openai_app = FastAPI(title="Tmux AI Assistant - OpenAI MCP")

# Shared tmux monitoring instance
tmux_server = libtmux.Server()
monitor: Optional[TmuxAIMonitor] = (
    None  # Will be initialized with session name and AI provider
)

# Store session activity for search/fetch
session_history: List[Dict[str, Any]] = []
max_history_items = (
    100  # Keep last 100 activities, because too much history can be overwhelming!
)

# ===== Standard MCP Tools (for Claude, Cursor, etc.) =====


@mcp.tool()
def list_tmux_sessions() -> List[str]:
    """List all active tmux sessions. Perfect for finding what's running!
    This helps other AIs understand the current terminal landscape.
    """
    try:
        sessions = tmux_server.sessions
        return [session.name for session in sessions]
    except Exception as e:
        logger.error(f"Failed to list sessions: {e}")
        return []


@mcp.tool()
def monitor_session(session_name: str, duration: int = 5) -> str:
    """
    Monitor a tmux session for activity.
    Returns captured output from the session. This is how we feed the AI fresh data!

    Args:
        session_name: Name of the tmux session to monitor
        duration: How many seconds to monitor (default: 5) - a quick peek!
    """
    try:
        session = tmux_server.find_where({"session_name": session_name})
        if not session:
            return f"Session '{session_name}' not found!"

        # Capture current pane content
        pane = session.attached_pane
        output = pane.capture_pane()

        # Store in history for OpenAI/Gemini search - because memory is key!
        activity = {
            "id": f"activity_{datetime.now().isoformat()}",
            "session": session_name,
            "timestamp": datetime.now().isoformat(),
            "content": "\n".join(output),
            "title": f"Tmux activity from {session_name}",
        }
        session_history.append(activity)
        if len(session_history) > max_history_items:
            session_history.pop(0)

        return "\n".join(output[-50:])  # Return last 50 lines
    except Exception as e:
        logger.error(f"Failed to monitor session: {e}")
        return f"Error monitoring session: {str(e)}"


@mcp.tool()
async def get_next_steps(session_activity: str) -> str:
    """
    Analyze session activity and suggest next steps.
    Uses the same AI logic as our tmux monitor! This is where the magic happens!

    Args:
        session_activity: Recent activity from the tmux session
    """
    if not monitor:
        return "Monitor not initialized. Please set up with a session first."

    try:
        # Use our existing AI logic from the TmuxAIMonitor instance
        # The monitor now handles which AI (OpenAI/Gemini) to use internally
        summary = await monitor.summarize_activity(session_activity)
        next_steps = await monitor.generate_next_step(summary)
        return next_steps
    except Exception as e:
        logger.error(f"Failed to get next steps: {e}")
        return f"Error generating next steps: {str(e)}"


@mcp.resource("session://{name}/current")
def get_session_state(name: str) -> str:
    """Get current state of a tmux session. Non-invasive read!
    Useful for other AIs to quickly peek at a session's current state.
    """
    try:
        session = tmux_server.find_where({"session_name": name})
        if not session:
            return f"Session '{name}' not found"

        pane = session.attached_pane
        return "\n".join(pane.capture_pane()[-30:])
    except Exception as e:
        return f"Error: {str(e)}"


# ===== OpenAI MCP Endpoints (search & fetch for deep research) =====


@openai_app.get("/")
async def root():
    """Welcome endpoint - Trisha insisted on a friendly greeting!
    A warm welcome to anyone connecting to our awesome server.
    """
    return {
        "message": "🎉 Tmux AI Assistant MCP Server is running!",
        "openai_mcp": "Use /sse for ChatGPT deep research",
        "standard_mcp": "Connect with Claude/Cursor using the mcp CLI",
        "made_with": "💖 by Aye, Hue, and Trisha",
    }


@openai_app.get("/sse")
async def openai_mcp_sse(request: Request):
    """
    OpenAI MCP SSE endpoint for ChatGPT deep research.
    Provides search and fetch tools for terminal session data.
    This is the gateway for ChatGPT to access our tmux insights!
    """

    async def event_generator():
        # Send initial handshake - like a secret club greeting!
        yield {
            "event": "connection",
            "data": json.dumps(
                {
                    "protocol": "openai-mcp",
                    "version": "1.0",
                    "capabilities": ["search", "fetch"],
                }
            ),
        }

        # Send tool definitions - telling ChatGPT what it can do!
        tools_definition = {
            "tools": [
                {
                    "name": "search",
                    "description": "Search through tmux session history. Query can include: session names, commands, output text, timestamps. Examples: 'error in mysession', 'git commands', 'python output'",
                    "input_schema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Search query for tmux session activity",
                            }
                        },
                        "required": ["query"],
                    },
                    "output_schema": {
                        "type": "object",
                        "properties": {
                            "results": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "id": {"type": "string"},
                                        "title": {"type": "string"},
                                        "text": {"type": "string"},
                                        "url": {"type": ["string", "null"]},
                                        "timestamp": {"type": "string"},
                                        "session": {"type": "string"},
                                    },
                                    "required": ["id", "title", "text"],
                                },
                            }
                        },
                        "required": ["results"],
                    },
                },
                {
                    "name": "fetch",
                    "description": "Fetch full content of a specific tmux activity by ID",
                    "input_schema": {
                        "type": "object",
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": "ID of the activity to fetch",
                            }
                        },
                        "required": ["id"],
                    },
                    "output_schema": {
                        "type": "object",
                        "properties": {
                            "content": {"type": "string"},
                            "metadata": {
                                "type": "object",
                                "properties": {
                                    "session": {"type": "string"},
                                    "timestamp": {"type": "string"},
                                },
                            },
                        },
                        "required": ["content"],
                    },
                },
            ]
        }

        yield {"event": "tools", "data": json.dumps(tools_definition)}

        # Keep connection alive and handle tool calls - like a steady heartbeat!
        try:
            while True:
                # Check for disconnection - don't want to talk to thin air!
                if await request.is_disconnected():
                    break

                # Send heartbeat - "Are you still there, ChatGPT?"
                yield {
                    "event": "heartbeat",
                    "data": json.dumps({"timestamp": datetime.now().isoformat()}),
                }

                await asyncio.sleep(30)  # Heartbeat every 30 seconds
        except asyncio.CancelledError:
            logger.info("SSE connection cancelled")

    return EventSourceResponse(event_generator())


@openai_app.post("/tools/{tool_name}")
async def execute_tool(tool_name: str, request: Request):
    """Execute OpenAI MCP tools (search or fetch)
    This is where ChatGPT's requests come to life!
    """
    body = await request.json()

    if tool_name == "search":
        query = body.get("query", "").lower()
        results = []

        # Search through session history - finding those golden nuggets of info!
        for activity in session_history:
            content_lower = activity["content"].lower()
            if (
                query in content_lower
                or query in activity["session"].lower()
                or query in activity["title"].lower()
            ):

                # Extract relevant snippet - just the juicy bits!
                lines = activity["content"].split("\n")
                snippet_lines = [line for line in lines if query in line.lower()]
                if not snippet_lines:
                    snippet_lines = lines[-5:]  # Last 5 lines if no direct match

                results.append(
                    {
                        "id": activity["id"],
                        "title": activity["title"],
                        "text": "\n".join(snippet_lines[:3]),  # First 3 matching lines
                        "url": None,  # No URL for terminal sessions (yet!)
                        "timestamp": activity["timestamp"],
                        "session": activity["session"],
                    }
                )

        return {
            "results": results[:10]
        }  # Return top 10 results - quality over quantity!

    elif tool_name == "fetch":
        activity_id = body.get("id")

        # Find the activity - like finding a specific receipt for Trish!
        for activity in session_history:
            if activity["id"] == activity_id:
                return {
                    "content": activity["content"],
                    "metadata": {
                        "session": activity["session"],
                        "timestamp": activity["timestamp"],
                    },
                }

        return {"error": f"Activity {activity_id} not found"}

    return {"error": f"Unknown tool: {tool_name}"}


# ===== Unified Server Runner =====


async def run_unified_server(
    session_name: Optional[str] = None,
    ai_provider: str = "openai",
    openai_api_key: Optional[str] = None,
    gemini_api_key: Optional[str] = None,
    port: int = 8000,
):
    """
    Run both MCP servers simultaneously!
    Like a DJ mixing two awesome tracks, with a third one waiting in the wings! 🎵
    """
    global monitor

    # Initialize monitor with the chosen AI provider and keys
    if session_name:
        try:
            monitor = TmuxAIMonitor(
                session_name, ai_provider, openai_api_key, gemini_api_key
            )
            logger.info(
                f"Monitoring tmux session: {session_name} with {ai_provider.upper()} AI."
            )
        except ValueError as e:
            logger.error(f"Failed to initialize TmuxAIMonitor: {e}")
            return  # Exit if monitor cannot be initialized

    # Start OpenAI MCP server (HTTP/SSE)
    logger.info(f"Starting OpenAI MCP server on http://localhost:{port}")
    logger.info("Connect ChatGPT to http://localhost:{port}/sse for deep research!")

    # Start standard MCP server
    logger.info("Standard MCP server ready for Claude/Cursor connections!")
    logger.info(f"Use: mcp connect stdio -- python {__file__} --stdio")

    # Run the FastAPI server
    config = uvicorn.Config(openai_app, host="0.0.0.0", port=port, log_level="info")
    server = uvicorn.Server(config)
    await server.serve()


# ===== Main Entry Point =====

if __name__ == "__main__":
    import sys
    import argparse  # For better argument parsing, because sys.argv can be a bit messy!

    parser = argparse.ArgumentParser(description="Tmux AI Assistant MCP Server")
    parser.add_argument(
        "--stdio",
        action="store_true",
        help="Run in standard MCP stdio mode for Claude/Cursor",
    )
    parser.add_argument(
        "session_name",
        nargs="?",
        default=None,
        help="Name of the tmux session to monitor (required for HTTP mode)",
    )
    parser.add_argument(
        "--ai-provider",
        default="openai",
        choices=["openai", "gemini", "ollama"],
        help="AI provider to use (openai, gemini, or ollama)",
    )
    parser.add_argument(
        "--openai-api-key",
        env_var="OPENAI_API_KEY",
        help="OpenAI API key (if using openai provider)",
    )
    parser.add_argument(
        "--gemini-api-key",
        env_var="GEMINI_API_KEY",
        help="Google Gemini API key (if using gemini provider)",
    )
    parser.add_argument(
        "--port",
        type=int,
        default=8000,
        help="Port for the OpenAI MCP server (if running in HTTP mode)",
    )

    args = parser.parse_args()

    # Check if running as standard MCP (stdio mode)
    if args.stdio:
        # Standard MCP mode for Claude/Cursor - they love a direct connection!
        mcp.run()
    else:
        # HTTP server mode for OpenAI MCP and others
        if not args.session_name:
            print(
                f"{Fore.RED}Error: Session name is required when not running in --stdio mode."
            )
            sys.exit(1)

        print("🚀 Tmux AI Assistant MCP Server")
        print("=" * 40)
        print(f"Session: {args.session_name or 'Not specified'}")
        print(f"AI Provider: {args.ai_provider.upper()}")
        print(f"Port: {args.port}")
        print("\n📌 Connection Instructions:")
        print(f"  - ChatGPT: http://localhost:{args.port}/sse")
        print(f"  - Claude: mcp connect stdio -- python {__file__} --stdio")
        print("\nPress Ctrl+C to stop")
        print("=" * 40)

        asyncio.run(
            run_unified_server(
                args.session_name,
                args.ai_provider,
                args.openai_api_key,
                args.gemini_api_key,
                args.port,
            )
        )
