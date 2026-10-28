#!/usr/bin/env python3
"""Test the find tool with projects type"""

import json
import subprocess
import sys

# Test request for find tool with projects type
test_request = {
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "find",
        "arguments": {
            "type": "projects",
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
print(f"Got {len(stdout.strip().split('\\n'))} lines of output")
for line in stdout.strip().split('\n'):
    if line:
        print(f"Processing line: {line[:100]}...")
        try:
            response = json.loads(line)
            if "result" in response:
                result = response['result']
                print(f"✅ Find tool with projects type works!")
                print(f"Result keys: {list(result.keys())}")
                if "projects" in result:
                    print(f"Found {result.get('count', len(result.get('projects', [])))} projects")
                    for proj in result.get('projects', [])[:5]:
                        if isinstance(proj, dict):
                            print(f"  - {proj.get('name', 'Unknown')}: {proj.get('info', '')[:50]}")
                else:
                    print("Raw result:", json.dumps(result, indent=2)[:1000])
            elif "error" in response:
                print("❌ Error:", response['error'])
        except json.JSONDecodeError:
            pass

if stderr:
    print("Stderr output available (set RUST_LOG=error to suppress)", file=sys.stderr)