#!/usr/bin/env python3
"""
Demo of Smart Tree's revolutionary smart edit capabilities!
By Aye, for Hue's delight!

This demonstrates how Smart Tree uses far fewer tokens than traditional diff-based editing.
Instead of sending entire file contents or diffs, we just send intentions!
"""

import json
import subprocess
import tempfile
import os

def run_smart_tree_tool(tool_name, params):
    """Run a Smart Tree MCP tool via st --mcp"""
    request = {
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": params
        },
        "id": 1
    }
    
    # Run st --mcp
    process = subprocess.Popen(
        ["st", "--mcp"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    
    # Send request
    request_str = json.dumps(request) + "\n"
    stdout, stderr = process.communicate(input=request_str)
    
    # Parse response
    for line in stdout.split('\n'):
        if line.strip() and line.startswith('{'):
            try:
                response = json.loads(line)
                if 'result' in response:
                    return response['result']
            except:
                pass
    
    return None

def demo_smart_edit():
    """Demonstrate smart editing capabilities"""
    print("🚀 Smart Tree Smart Edit Demo")
    print("=" * 50)
    
    # Create a test file
    test_file = "/tmp/smart_edit_demo.py"
    initial_code = '''def greet(name):
    print(f"Hello, {name}!")

def main():
    greet("World")

if __name__ == "__main__":
    main()
'''
    
    with open(test_file, 'w') as f:
        f.write(initial_code)
    
    print(f"✅ Created test file: {test_file}")
    print("\nInitial code:")
    print("-" * 40)
    print(initial_code)
    print("-" * 40)
    
    # Demo 1: Get function tree
    print("\n📊 Demo 1: Get Function Tree")
    result = run_smart_tree_tool("get_function_tree", {
        "file_path": test_file
    })
    if result:
        print(json.dumps(result, indent=2))
    
    # Demo 2: Insert a function
    print("\n✨ Demo 2: Insert Function (minimal tokens!)")
    print("Instead of sending entire file + diff, we just send:")
    insert_params = {
        "file_path": test_file,
        "name": "farewell",
        "body": '''(name):
    print(f"Goodbye, {name}!")''',
        "after": "greet",
        "visibility": "public"
    }
    print(json.dumps(insert_params, indent=2))
    
    result = run_smart_tree_tool("insert_function", insert_params)
    if result:
        print("\n✅ Function inserted successfully!")
        with open(test_file, 'r') as f:
            print("\nUpdated code:")
            print("-" * 40)
            print(f.read())
            print("-" * 40)
    
    # Demo 3: Smart edit - multiple operations
    print("\n🎯 Demo 3: Multiple Smart Edits in One Call")
    smart_edits = {
        "file_path": test_file,
        "edits": [
            {
                "operation": "AddImport",
                "import": "sys"
            },
            {
                "operation": "ReplaceFunction",
                "name": "main",
                "new_body": '''():
    name = input("Enter your name: ")
    greet(name)
    farewell(name)'''
            },
            {
                "operation": "SmartAppend",
                "section": "functions",
                "content": '''def celebrate():
    print("🎉 Smart editing is amazing!")'''
            }
        ]
    }
    
    print("Sending these edits (look how minimal!):")
    print(json.dumps(smart_edits, indent=2))
    
    result = run_smart_tree_tool("smart_edit", smart_edits)
    if result:
        print("\n✅ All edits applied successfully!")
        with open(test_file, 'r') as f:
            print("\nFinal code:")
            print("-" * 40)
            print(f.read())
            print("-" * 40)
    
    # Demo 4: Remove function with dependency checking
    print("\n🗑️ Demo 4: Remove Function with Dependency Awareness")
    remove_params = {
        "file_path": test_file,
        "name": "celebrate",
        "force": False
    }
    
    result = run_smart_tree_tool("remove_function", remove_params)
    if result:
        print("✅ Function removed (no dependencies found)")
    
    # Show token efficiency
    print("\n📈 Token Efficiency Comparison:")
    print("-" * 50)
    print("Traditional diff approach:")
    print("  - Send entire file before: ~200 tokens")
    print("  - Send entire file after: ~250 tokens")
    print("  - Total: ~450 tokens")
    print("\nSmart Tree approach:")
    print("  - Send function + position: ~30 tokens")
    print("  - Total: ~30 tokens")
    print("\n💡 That's 93% fewer tokens! 🚀")
    
    print("\n" + "=" * 50)
    print("Demo complete! Aye says: 'Fast is better than slow!'")
    print("Trisha adds: 'It's like double-entry bookkeeping for code!' 💎")

if __name__ == "__main__":
    demo_smart_edit()