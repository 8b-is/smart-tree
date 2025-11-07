#!/usr/bin/env python3
"""List all available MCP tools"""

import json
import subprocess

# Test request to list tools
test_request = {
    "jsonrpc": "2.0",
    "method": "tools/list",
    "params": {},
    "id": 1
}

# Run st with MCP mode and send the request
proc = subprocess.Popen(
    ["./target/release/st", "--mcp"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True
)

# Send the request
request_line = json.dumps(test_request) + "\n"
stdout, stderr = proc.communicate(input=request_line)

# Parse and display the response
for line in stdout.strip().split('\n'):
    if line:
        try:
            response = json.loads(line)
            if "result" in response and "tools" in response['result']:
                tools = response['result']['tools']
                print(f"Found {len(tools)} tools:")
                for tool in tools:
                    name = tool.get('name', 'Unknown')
                    if 'project' in name.lower():
                        print(f"  ✅ {name} - {tool.get('description', '')[:100]}")

                # Check if find_projects is there
                tool_names = [t.get('name') for t in tools]
                if 'find_projects' in tool_names:
                    print("\n✅ find_projects tool is available!")
                else:
                    print("\n❌ find_projects tool is NOT in the list")
                    print("Available project-related tools:")
                    for name in tool_names:
                        if 'project' in name.lower():
                            print(f"  - {name}")
        except json.JSONDecodeError:
            pass