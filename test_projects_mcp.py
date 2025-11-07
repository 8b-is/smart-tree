#!/usr/bin/env python3
"""Test the find_projects MCP tool"""

import json
import subprocess
import sys

# Test request for find_projects tool
test_request = {
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "find_projects",
        "arguments": {
            "path": ".",
            "depth": 3
        }
    },
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
            if "result" in response:
                print("✅ MCP tool find_projects works!")
                print(f"Found {response['result'].get('count', 0)} projects")
                if "projects" in response['result']:
                    for proj in response['result']['projects'][:3]:
                        print(f"  - {proj.get('name', 'Unknown')}")
            elif "error" in response:
                print("❌ Error:", response['error'])
        except json.JSONDecodeError:
            pass

if stderr:
    print("Stderr:", stderr, file=sys.stderr)