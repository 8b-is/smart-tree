#!/usr/bin/env python3
"""
Send Smart Tree quantum format to Claude via Anthropic's API
This demonstrates how to efficiently transmit directory structures to LLMs
"""

import os
import sys
import base64
import json
import subprocess
from anthropic import Anthropic

def get_quantum_output(path="."):
    """Run Smart Tree in quantum mode and capture output"""
    try:
        result = subprocess.run(
            ["st", path, "-m", "quantum"],
            capture_output=True,
            check=True
        )
        return result.stdout
    except subprocess.CalledProcessError as e:
        print(f"Error running Smart Tree: {e}")
        sys.exit(1)

def prepare_quantum_for_api(quantum_data):
    """Prepare quantum format for API transmission"""
    # Find the data section markers
    begin_marker = b'---BEGIN_DATA---\n'
    end_marker = b'\n---END_DATA---'
    
    begin_pos = quantum_data.find(begin_marker)
    end_pos = quantum_data.find(end_marker)
    
    if begin_pos == -1 or end_pos == -1:
        # Fallback for old format
        lines = quantum_data.split(b'\n', 3)
        if len(lines) >= 4:
            header = b'\n'.join(lines[:3]).decode('utf-8')
            binary = lines[3]
        else:
            raise ValueError("Invalid quantum format")
    else:
        # Extract header and binary sections
        header = quantum_data[:begin_pos].decode('utf-8').strip()
        binary = quantum_data[begin_pos + len(begin_marker):end_pos]
    
    # Encode binary data as base64 for safe transmission
    binary_b64 = base64.b64encode(binary).decode('ascii')
    
    return {
        "format": "smart-tree-quantum-v1",
        "header": header,
        "data_base64": binary_b64,
        "data_size": len(binary),
        "compression_ratio": f"{len(binary) / len(str(quantum_data)):.1%}"
    }

def send_to_claude(quantum_payload, user_prompt="What can you tell me about this codebase?"):
    """Send quantum format to Claude API"""
    
    # Initialize Anthropic client
    api_key = os.environ.get("ANTHROPIC_API_KEY")
    if not api_key:
        print("Please set ANTHROPIC_API_KEY environment variable")
        sys.exit(1)
    
    client = Anthropic(api_key=api_key)
    
    # Construct the message
    system_prompt = """You are receiving a Smart Tree quantum format directory structure.
This is an ultra-compressed format that uses:
- Bitfield headers for metadata
- Token substitution for common patterns (e.g., 0x80=node_modules, 0x91=.rs)
- ASCII control codes for tree traversal (0x0E=deeper, 0x0F=back)
- Delta encoding for permissions (0x0049 = difference between 755 and 644)

The format provides complete directory structure information in minimal bytes."""

    message_content = f"""I'm sending you a directory structure in Smart Tree quantum format:

{quantum_payload['header']}

Binary data (base64 encoded, {quantum_payload['data_size']} bytes):
{quantum_payload['data_base64']}

Compression ratio: {quantum_payload['compression_ratio']}

{user_prompt}"""

    try:
        response = client.messages.create(
            model="claude-3-opus-20240229",  # or claude-3-sonnet-20240229
            system=system_prompt,
            messages=[{
                "role": "user",
                "content": message_content
            }],
            max_tokens=4096
        )
        
        return response.content[0].text
    
    except Exception as e:
        print(f"Error calling Claude API: {e}")
        return None

def decode_quantum_for_display(quantum_data):
    """Decode quantum format for human viewing (optional)"""
    # This would use the quantum decoder logic
    # For now, just show a summary
    lines = quantum_data.decode('utf-8', errors='replace').split('\n')
    return '\n'.join(lines[:10]) + '\n...'

def main():
    # Get path from command line or use current directory
    path = sys.argv[1] if len(sys.argv) > 1 else "."
    
    print(f"Scanning {path} with Smart Tree quantum format...")
    quantum_data = get_quantum_output(path)
    
    print(f"Raw quantum output size: {len(quantum_data)} bytes")
    
    # Prepare for API
    quantum_payload = prepare_quantum_for_api(quantum_data)
    
    print(f"Prepared payload:")
    print(f"  Header lines: {len(quantum_payload['header'].split(chr(10)))}")
    print(f"  Binary data: {quantum_payload['data_size']} bytes")
    print(f"  Base64 size: {len(quantum_payload['data_base64'])} chars")
    print(f"  Compression: {quantum_payload['compression_ratio']}")
    
    # Example prompts
    example_prompts = [
        "What can you tell me about this codebase structure?",
        "What are the main components of this project?",
        "Are there any potential issues with the file organization?",
        "What type of project is this and what does it do?",
        "Can you identify any patterns or best practices in the structure?"
    ]
    
    print("\nExample prompts:")
    for i, prompt in enumerate(example_prompts, 1):
        print(f"{i}. {prompt}")
    
    # Get user prompt
    prompt_choice = input("\nSelect a prompt (1-5) or enter custom prompt: ").strip()
    
    if prompt_choice.isdigit() and 1 <= int(prompt_choice) <= len(example_prompts):
        user_prompt = example_prompts[int(prompt_choice) - 1]
    else:
        user_prompt = prompt_choice if prompt_choice else example_prompts[0]
    
    print(f"\nSending to Claude with prompt: {user_prompt}")
    
    # Send to Claude
    response = send_to_claude(quantum_payload, user_prompt)
    
    if response:
        print("\nClaude's response:")
        print("-" * 80)
        print(response)
    else:
        print("\nFailed to get response from Claude")
        
        # Show the payload that would have been sent
        print("\nPayload that would have been sent:")
        print(json.dumps(quantum_payload, indent=2)[:500] + "...")

if __name__ == "__main__":
    main()