#!/usr/bin/env python3
"""
MCP Explorer - Interactive MCP Tool Explorer for Humans
========================================================
A universal MCP client that lets you explore and interact with any MCP server.
Perfect for understanding what AI tools do and how they work!

Author: Aye & Hue 🚢
"""

import json
import subprocess
import sys
import os
from typing import Dict, List, Any, Optional
from dataclasses import dataclass
import argparse
from pathlib import Path
import textwrap
import re

# Try to import rich for beautiful terminal output
try:
    from rich.console import Console
    from rich.table import Table
    from rich.panel import Panel
    from rich.syntax import Syntax
    from rich.prompt import Prompt, Confirm
    from rich.markdown import Markdown
    from rich import print as rprint
    RICH_AVAILABLE = True
except ImportError:
    RICH_AVAILABLE = False
    print("💡 Tip: Install 'rich' for a better experience: pip install rich")

@dataclass
class MCPTool:
    """Represents an MCP tool with its metadata"""
    name: str
    description: str
    parameters: Dict[str, Any]
    lane: Optional[str] = None  # EXPLORE, ANALYZE, or ACT
    
    def get_emoji(self) -> str:
        """Get emoji based on tool lane"""
        if not self.lane:
            return "🔧"
        lane_emojis = {
            "EXPLORE": "🔍",
            "ANALYZE": "🧪", 
            "ACT": "⚡"
        }
        return lane_emojis.get(self.lane, "🔧")

class MCPExplorer:
    """Interactive MCP Explorer - Learn by doing!"""
    
    def __init__(self, server_command: List[str], verbose: bool = False):
        self.server_command = server_command
        self.verbose = verbose
        self.console = Console() if RICH_AVAILABLE else None
        self.tools: Dict[str, MCPTool] = {}
        self.server_info: Dict[str, Any] = {}
        self.history: List[Dict] = []
        
    def print(self, *args, **kwargs):
        """Print with rich if available, otherwise standard print"""
        if self.console:
            self.console.print(*args, **kwargs)
        else:
            print(*args, **kwargs)
    
    def send_request(self, method: str, params: Optional[Dict] = None) -> Dict:
        """Send a JSON-RPC request to the MCP server"""
        request = {
            "jsonrpc": "2.0",
            "id": len(self.history) + 1,
            "method": method
        }
        if params:
            request["params"] = params
            
        if self.verbose:
            self.print(f"[dim]→ Sending: {json.dumps(request, indent=2)}[/dim]")
        
        try:
            # Run the MCP server with the request
            process = subprocess.Popen(
                self.server_command,
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )
            
            # Send request and get response
            stdout, stderr = process.communicate(input=json.dumps(request))
            
            if self.verbose and stderr:
                self.print(f"[dim yellow]Server logs: {stderr}[/dim yellow]")
            
            # Parse response - handle shell escaping
            stdout = stdout.strip()
            if stdout.startswith("'") and stdout.endswith("'"):
                stdout = stdout[1:-1]
            response = json.loads(stdout)
            
            if self.verbose:
                self.print(f"[dim]← Received: {json.dumps(response, indent=2)}[/dim]")
                
            self.history.append({"request": request, "response": response})
            return response
            
        except json.JSONDecodeError as e:
            self.print(f"[red]Error parsing response: {e}[/red]")
            if stdout:
                self.print(f"Raw output: {stdout}")
            return {}
        except Exception as e:
            self.print(f"[red]Error communicating with server: {e}[/red]")
            return {}
    
    def initialize(self) -> bool:
        """Initialize connection to MCP server"""
        self.print("\n🚀 Connecting to MCP server...")
        
        # Try to get server info
        response = self.send_request("initialize", {
            "protocolVersion": "0.1.0",
            "capabilities": {
                "tools": {}
            },
            "clientInfo": {
                "name": "MCP Explorer",
                "version": "1.0.0"
            }
        })
        
        if "result" in response:
            self.server_info = response["result"].get("serverInfo", {})
            self.print(f"✅ Connected to: [bold]{self.server_info.get('name', 'Unknown')}[/bold] v{self.server_info.get('version', '?')}")
            
            # Get available tools
            self.discover_tools()
            return True
        else:
            self.print("[red]Failed to initialize MCP connection[/red]")
            return False
    
    def discover_tools(self):
        """Discover available tools from the server"""
        response = self.send_request("tools/list")
        
        if "result" in response:
            tools_data = response["result"].get("tools", [])
            
            for tool_data in tools_data:
                # Extract lane from description if present
                desc = tool_data.get("description", "")
                lane = None
                if "EXPLORE:" in desc:
                    lane = "EXPLORE"
                elif "ANALYZE:" in desc:
                    lane = "ANALYZE"
                elif "ACT:" in desc:
                    lane = "ACT"
                
                tool = MCPTool(
                    name=tool_data["name"],
                    description=desc,
                    parameters=tool_data.get("inputSchema", {}),
                    lane=lane
                )
                self.tools[tool.name] = tool
            
            self.print(f"📦 Discovered [green]{len(self.tools)}[/green] tools")
    
    def display_tools(self, filter_lane: Optional[str] = None):
        """Display available tools in a nice format"""
        if RICH_AVAILABLE:
            table = Table(title="Available MCP Tools", show_lines=True)
            table.add_column("Tool", style="cyan", no_wrap=True)
            table.add_column("Lane", style="magenta")
            table.add_column("Description", style="white")
            
            for tool in self.tools.values():
                if filter_lane and tool.lane != filter_lane:
                    continue
                    
                # Clean up description
                desc = re.sub(r'^(🔍 EXPLORE:|🧪 ANALYZE:|⚡ ACT:)\s*', '', tool.description)
                desc = textwrap.fill(desc, width=60)
                
                table.add_row(
                    f"{tool.get_emoji()} {tool.name}",
                    tool.lane or "General",
                    desc
                )
            
            self.console.print(table)
        else:
            # Simple text output
            print("\n=== Available MCP Tools ===\n")
            for tool in self.tools.values():
                if filter_lane and tool.lane != filter_lane:
                    continue
                print(f"{tool.get_emoji()} {tool.name}")
                print(f"   Lane: {tool.lane or 'General'}")
                print(f"   {textwrap.fill(tool.description, width=70, subsequent_indent='   ')}")
                print()
    
    def get_tool_params(self, tool: MCPTool) -> Dict[str, Any]:
        """Interactively get parameters for a tool"""
        params = {}
        schema = tool.parameters.get("properties", {})
        required = tool.parameters.get("required", [])
        
        self.print(f"\n📝 Parameters for [cyan]{tool.name}[/cyan]:")
        
        for param_name, param_schema in schema.items():
            param_type = param_schema.get("type", "string")
            description = param_schema.get("description", "")
            default = param_schema.get("default")
            is_required = param_name in required
            
            # Build prompt
            prompt = f"  {param_name}"
            if description:
                prompt += f" ({description})"
            if not is_required:
                prompt += " [optional]"
            if default is not None:
                prompt += f" [default: {default}]"
            prompt += ": "
            
            # Get value based on type
            if param_type == "boolean":
                if RICH_AVAILABLE:
                    value = Confirm.ask(prompt, default=default if default is not None else False)
                else:
                    response = input(prompt + "(y/n) ").lower()
                    value = response in ['y', 'yes', 'true', '1']
            elif param_type == "integer":
                value_str = input(prompt)
                if value_str:
                    try:
                        value = int(value_str)
                    except ValueError:
                        self.print(f"[yellow]Invalid integer, using default[/yellow]")
                        value = default
                else:
                    value = default
            elif param_type == "array":
                value_str = input(prompt + "(comma-separated) ")
                if value_str:
                    value = [v.strip() for v in value_str.split(",")]
                else:
                    value = default
            else:  # string or other
                value = input(prompt)
                if not value and default is not None:
                    value = default
            
            # Only add if we have a value or it's required
            if value or is_required:
                params[param_name] = value
        
        return params
    
    def call_tool(self, tool_name: str, params: Optional[Dict] = None):
        """Call a specific tool and display results"""
        if tool_name not in self.tools:
            self.print(f"[red]Tool '{tool_name}' not found[/red]")
            return
        
        tool = self.tools[tool_name]
        
        # Get parameters interactively if not provided
        if params is None:
            params = self.get_tool_params(tool)
        
        self.print(f"\n🔄 Calling [cyan]{tool_name}[/cyan]...")
        
        response = self.send_request("tools/call", {
            "name": tool_name,
            "arguments": params
        })
        
        if "result" in response:
            self.display_result(response["result"])
        elif "error" in response:
            self.print(f"[red]Error: {response['error'].get('message', 'Unknown error')}[/red]")
    
    def display_result(self, result: Any):
        """Display tool result in a nice format"""
        if isinstance(result, dict) and "content" in result:
            # MCP standard response format
            for content_item in result["content"]:
                if content_item.get("type") == "text":
                    text = content_item.get("text", "")
                    
                    # Try to parse as JSON for better display
                    try:
                        data = json.loads(text)
                        if RICH_AVAILABLE:
                            self.console.print(Panel(
                                Syntax(json.dumps(data, indent=2), "json"),
                                title="Result",
                                border_style="green"
                            ))
                        else:
                            print("\n=== Result ===")
                            print(json.dumps(data, indent=2))
                    except json.JSONDecodeError:
                        # Display as plain text
                        if RICH_AVAILABLE:
                            self.console.print(Panel(text, title="Result", border_style="green"))
                        else:
                            print("\n=== Result ===")
                            print(text)
        else:
            # Raw result
            if RICH_AVAILABLE:
                self.console.print(Panel(
                    Syntax(json.dumps(result, indent=2), "json"),
                    title="Result",
                    border_style="green"
                ))
            else:
                print("\n=== Result ===")
                print(json.dumps(result, indent=2))
    
    def interactive_mode(self):
        """Run interactive exploration mode"""
        self.print("\n🎮 [bold]Interactive MCP Explorer[/bold]")
        self.print("Type 'help' for commands, 'quit' to exit\n")
        
        while True:
            try:
                if RICH_AVAILABLE:
                    command = Prompt.ask("[bold blue]mcp[/bold blue]>").strip()
                else:
                    command = input("mcp> ").strip()
                
                if not command:
                    continue
                
                parts = command.split()
                cmd = parts[0].lower()
                
                if cmd in ['quit', 'exit', 'q']:
                    self.print("👋 Goodbye!")
                    break
                    
                elif cmd == 'help':
                    self.show_help()
                    
                elif cmd == 'tools':
                    filter_lane = parts[1].upper() if len(parts) > 1 else None
                    self.display_tools(filter_lane)
                    
                elif cmd == 'info':
                    if len(parts) > 1:
                        self.show_tool_info(parts[1])
                    else:
                        self.show_server_info()
                        
                elif cmd == 'call':
                    if len(parts) > 1:
                        self.call_tool(parts[1])
                    else:
                        self.print("[yellow]Usage: call <tool_name>[/yellow]")
                        
                elif cmd == 'lanes':
                    self.show_lanes()
                    
                elif cmd == 'history':
                    self.show_history()
                    
                elif cmd == 'guided':
                    self.guided_exploration()
                    
                else:
                    self.print(f"[yellow]Unknown command: {cmd}[/yellow]")
                    self.print("Type 'help' for available commands")
                    
            except KeyboardInterrupt:
                self.print("\n[yellow]Use 'quit' to exit[/yellow]")
            except Exception as e:
                self.print(f"[red]Error: {e}[/red]")
    
    def show_help(self):
        """Display help information"""
        help_text = """
[bold]Available Commands:[/bold]

  [cyan]tools [lane][/cyan]     - List all tools (optionally filtered by lane)
  [cyan]info <tool>[/cyan]      - Show detailed info about a tool
  [cyan]call <tool>[/cyan]      - Call a tool interactively
  [cyan]lanes[/cyan]            - Show tool lanes (EXPLORE/ANALYZE/ACT)
  [cyan]guided[/cyan]           - Start guided exploration
  [cyan]history[/cyan]          - Show command history
  [cyan]help[/cyan]             - Show this help
  [cyan]quit[/cyan]             - Exit the explorer

[bold]Tool Lanes:[/bold]
  🔍 [green]EXPLORE[/green] - Discovery and overview tools
  🧪 [green]ANALYZE[/green] - Deep analysis and search tools  
  ⚡ [green]ACT[/green]     - Tools that modify or create

[bold]Examples:[/bold]
  tools explore     - Show only EXPLORE tools
  info quick_tree   - Get details about quick_tree
  call search_in_files - Search for content in files
  guided            - Let me guide you through the tools!
        """
        
        if RICH_AVAILABLE:
            self.console.print(Markdown(help_text))
        else:
            print(help_text)
    
    def show_tool_info(self, tool_name: str):
        """Show detailed information about a specific tool"""
        if tool_name not in self.tools:
            self.print(f"[red]Tool '{tool_name}' not found[/red]")
            return
        
        tool = self.tools[tool_name]
        
        if RICH_AVAILABLE:
            panel_content = f"""
[bold]Description:[/bold]
{tool.description}

[bold]Lane:[/bold] {tool.lane or 'General'} {tool.get_emoji()}

[bold]Parameters:[/bold]
"""
            schema = tool.parameters.get("properties", {})
            required = tool.parameters.get("required", [])
            
            for param_name, param_schema in schema.items():
                param_type = param_schema.get("type", "string")
                description = param_schema.get("description", "")
                is_required = param_name in required
                
                panel_content += f"\n  • [cyan]{param_name}[/cyan] ({param_type})"
                if is_required:
                    panel_content += " [red]*required[/red]"
                if description:
                    panel_content += f"\n    {description}"
            
            self.console.print(Panel(
                panel_content,
                title=f"Tool: {tool.name}",
                border_style="cyan"
            ))
        else:
            print(f"\n=== Tool: {tool.name} ===")
            print(f"Description: {tool.description}")
            print(f"Lane: {tool.lane or 'General'} {tool.get_emoji()}")
            print("\nParameters:")
            
            schema = tool.parameters.get("properties", {})
            required = tool.parameters.get("required", [])
            
            for param_name, param_schema in schema.items():
                param_type = param_schema.get("type", "string")
                description = param_schema.get("description", "")
                is_required = param_name in required
                
                print(f"  • {param_name} ({param_type})", end="")
                if is_required:
                    print(" *required", end="")
                print()
                if description:
                    print(f"    {description}")
    
    def show_lanes(self):
        """Show tools organized by lanes"""
        lanes = {"EXPLORE": [], "ANALYZE": [], "ACT": [], "General": []}
        
        for tool in self.tools.values():
            lane = tool.lane or "General"
            lanes[lane].append(tool)
        
        if RICH_AVAILABLE:
            for lane_name, tools in lanes.items():
                if not tools:
                    continue
                    
                emoji = {"EXPLORE": "🔍", "ANALYZE": "🧪", "ACT": "⚡", "General": "🔧"}[lane_name]
                
                self.console.print(f"\n[bold]{emoji} {lane_name} Lane[/bold] ({len(tools)} tools)")
                for tool in tools:
                    self.console.print(f"  • {tool.name}")
        else:
            for lane_name, tools in lanes.items():
                if not tools:
                    continue
                    
                print(f"\n=== {lane_name} Lane ({len(tools)} tools) ===")
                for tool in tools:
                    print(f"  • {tool.name}")
    
    def show_server_info(self):
        """Display server information"""
        if RICH_AVAILABLE:
            info = f"""
[bold]Server:[/bold] {self.server_info.get('name', 'Unknown')}
[bold]Version:[/bold] {self.server_info.get('version', 'Unknown')}
[bold]Total Tools:[/bold] {len(self.tools)}
"""
            self.console.print(Panel(info, title="MCP Server Info", border_style="blue"))
        else:
            print("\n=== MCP Server Info ===")
            print(f"Server: {self.server_info.get('name', 'Unknown')}")
            print(f"Version: {self.server_info.get('version', 'Unknown')}")
            print(f"Total Tools: {len(self.tools)}")
    
    def show_history(self):
        """Display command history"""
        if not self.history:
            self.print("[yellow]No history yet[/yellow]")
            return
        
        for i, item in enumerate(self.history[-10:], 1):  # Show last 10
            req = item["request"]
            self.print(f"{i}. {req.get('method', 'unknown')} ", end="")
            if "params" in req and "name" in req["params"]:
                self.print(f"- {req['params']['name']}")
            else:
                self.print()
    
    def guided_exploration(self):
        """Guided exploration for beginners"""
        self.print("\n🎯 [bold]Guided Exploration Mode[/bold]")
        self.print("Let me guide you through the tools step by step!\n")
        
        # Step 1: Choose a lane
        self.print("[bold]Step 1: Choose your exploration path[/bold]")
        self.print("1. 🔍 EXPLORE - Start with overview and discovery")
        self.print("2. 🧪 ANALYZE - Deep dive into code and content")
        self.print("3. ⚡ ACT - Make changes and modifications")
        
        choice = input("\nYour choice (1-3): ").strip()
        
        if choice == "1":
            lane = "EXPLORE"
            self.print("\n[green]Great choice! Let's start exploring.[/green]")
            recommended_tool = "quick_tree"
        elif choice == "2":
            lane = "ANALYZE"
            self.print("\n[green]Perfect! Let's analyze some code.[/green]")
            recommended_tool = "search_in_files"
        elif choice == "3":
            lane = "ACT"
            self.print("\n[green]Powerful! Let's make some changes.[/green]")
            recommended_tool = "smart_edit"
        else:
            self.print("[yellow]Invalid choice, starting with EXPLORE[/yellow]")
            lane = "EXPLORE"
            recommended_tool = "quick_tree"
        
        # Step 2: Show relevant tools
        self.print(f"\n[bold]Step 2: Available {lane} tools:[/bold]")
        lane_tools = [t for t in self.tools.values() if t.lane == lane]
        
        for i, tool in enumerate(lane_tools[:5], 1):  # Show first 5
            self.print(f"{i}. {tool.get_emoji()} {tool.name}")
            desc = tool.description[:100] + "..." if len(tool.description) > 100 else tool.description
            self.print(f"   {desc}")
        
        # Step 3: Recommend a tool
        if recommended_tool in self.tools:
            self.print(f"\n[bold]Step 3: I recommend starting with '{recommended_tool}'[/bold]")
            if input("Would you like to try it? (y/n): ").lower() == 'y':
                self.call_tool(recommended_tool)
        
        self.print("\n[green]Great job! You can now explore other tools with 'tools' or 'call <tool>'[/green]")

def main():
    parser = argparse.ArgumentParser(
        description="MCP Explorer - Interactive tool explorer for humans",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Explore Smart Tree MCP tools
  python mcp_explorer.py --command "st --mcp"
  
  # Explore any MCP server
  python mcp_explorer.py --command "your-mcp-server"
  
  # With verbose output
  python mcp_explorer.py --command "st --mcp" --verbose
  
  # Use a specific config file
  python mcp_explorer.py --config ~/.config/mcp/servers.json --server smart-tree
"""
    )
    
    parser.add_argument(
        "--command",
        type=str,
        help="Command to run the MCP server (e.g., 'st --mcp')"
    )
    
    parser.add_argument(
        "--config",
        type=str,
        help="Path to MCP config file (for Claude Desktop compatibility)"
    )
    
    parser.add_argument(
        "--server",
        type=str,
        help="Server name from config file"
    )
    
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Show detailed request/response logs"
    )
    
    args = parser.parse_args()
    
    # Determine server command
    if args.command:
        server_command = args.command.split()
    elif args.config and args.server:
        # Read from config file (Claude Desktop format)
        try:
            with open(args.config) as f:
                config = json.load(f)
            
            if args.server in config.get("mcpServers", {}):
                server_config = config["mcpServers"][args.server]
                server_command = server_config["command"].split()
                if "args" in server_config:
                    server_command.extend(server_config["args"])
            else:
                print(f"Server '{args.server}' not found in config")
                sys.exit(1)
        except Exception as e:
            print(f"Error reading config: {e}")
            sys.exit(1)
    else:
        # Default to Smart Tree if available
        server_command = ["st", "--mcp"]
        print("No command specified, trying default: st --mcp")
        print("Use --command to specify a different MCP server\n")
    
    # Create and run explorer
    explorer = MCPExplorer(server_command, verbose=args.verbose)
    
    if explorer.initialize():
        explorer.interactive_mode()
    else:
        print("Failed to connect to MCP server")
        print("Make sure the server is installed and the command is correct")
        sys.exit(1)

if __name__ == "__main__":
    main()