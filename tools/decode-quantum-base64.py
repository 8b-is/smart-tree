#!/usr/bin/env python3
"""
Decode base64-encoded quantum format from MCP output
"""

import sys
import base64

def decode_quantum_base64(data):
    """Decode QUANTUM_BASE64 format"""
    if data.startswith("QUANTUM_BASE64:"):
        # Extract base64 data
        b64_data = data[15:]  # Skip "QUANTUM_BASE64:"
        
        # Decode from base64
        try:
            binary_data = base64.b64decode(b64_data)
            
            # Write to stdout as binary (for piping) or save to file
            if sys.stdout.isatty():
                # Terminal - save to file
                with open("quantum_decoded.bin", "wb") as f:
                    f.write(binary_data)
                print(f"Decoded {len(binary_data)} bytes to quantum_decoded.bin")
                print("You can now use: python3 quantum-decode.py quantum_decoded.bin")
            else:
                # Pipe - write binary to stdout
                sys.stdout.buffer.write(binary_data)
                
        except Exception as e:
            print(f"Error decoding base64: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        print("Input doesn't start with QUANTUM_BASE64:", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    if len(sys.argv) > 1:
        # Read from file
        with open(sys.argv[1], 'r') as f:
            data = f.read().strip()
    else:
        # Read from stdin
        data = sys.stdin.read().strip()
    
    decode_quantum_base64(data)