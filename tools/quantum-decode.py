#!/usr/bin/env python3
"""
Quantum Format Decoder - Visualize Smart Tree's quantum compressed output
"""

import sys
import struct

# Control codes
TRAVERSE_SAME = 0x0B      # Vertical Tab
TRAVERSE_DEEPER = 0x0E   # Shift Out
TRAVERSE_BACK = 0x0F     # Shift In
TRAVERSE_SUMMARY = 0x0C  # Form Feed

# Header bits
SIZE_BIT = 0x01
PERMS_BIT = 0x02
TIME_BIT = 0x04
OWNER_BIT = 0x08
DIR_BIT = 0x10
LINK_BIT = 0x20
XATTR_BIT = 0x40
SUMMARY_BIT = 0x80

def decode_size(data, offset):
    """Decode variable-length size encoding"""
    prefix = data[offset]
    if prefix == 0x00:
        return data[offset + 1], offset + 2
    elif prefix == 0x01:
        return struct.unpack_from('<H', data, offset + 1)[0], offset + 3
    elif prefix == 0x02:
        return struct.unpack_from('<I', data, offset + 1)[0], offset + 5
    else:
        return struct.unpack_from('<Q', data, offset + 1)[0], offset + 9

def decode_entry(data, offset):
    """Decode a single quantum entry"""
    if offset >= len(data):
        return None, offset
        
    header = data[offset]
    offset += 1
    
    entry = {'header': header}
    
    # Decode size if present
    if header & SIZE_BIT:
        entry['size'], offset = decode_size(data, offset)
    
    # Decode permissions delta if present
    if header & PERMS_BIT:
        if offset + 2 <= len(data):
            entry['perms_delta'] = (data[offset] << 8) | data[offset + 1]
            offset += 2
    
    # Decode time delta if present
    if header & TIME_BIT:
        # For now, skip time encoding
        pass
    
    # Decode owner/group if present
    if header & OWNER_BIT:
        # For now, skip owner/group encoding
        pass
    
    # Check directory bit
    if header & DIR_BIT:
        entry['is_dir'] = True
    
    # Check symlink bit
    if header & LINK_BIT:
        entry['is_link'] = True
    
    # Find name (ends with null byte or control character)
    name_start = offset
    while offset < len(data):
        if data[offset] == 0 or data[offset] in [TRAVERSE_SAME, TRAVERSE_DEEPER, TRAVERSE_BACK, TRAVERSE_SUMMARY]:
            break
        offset += 1
    
    entry['name'] = data[name_start:offset].decode('utf-8', errors='replace')
    
    # Skip null terminator if present
    if offset < len(data) and data[offset] == 0:
        offset += 1
    
    if offset < len(data):
        entry['traverse'] = data[offset]
        offset += 1
    
    return entry, offset

def decode_quantum(data):
    """Decode quantum format stream"""
    # Find the BEGIN_DATA marker
    begin_marker = b'---BEGIN_DATA---\n'
    end_marker = b'\n---END_DATA---'
    
    begin_pos = data.find(begin_marker)
    if begin_pos == -1:
        # Old format
        lines = data.split(b'\n', 3)
        if len(lines) < 4:
            return
        print(f"Format: {lines[0].decode()}")
        print(f"Key: {lines[1].decode()}")
        print(f"Tokens: {lines[2].decode()}")
        print("-" * 60)
        binary_data = lines[3]
    else:
        # New format with markers
        header = data[:begin_pos].decode()
        print(header.strip())
        print("-" * 60)
        
        end_pos = data.find(end_marker)
        if end_pos == -1:
            binary_data = data[begin_pos + len(begin_marker):]
        else:
            binary_data = data[begin_pos + len(begin_marker):end_pos]
    
    # Process binary data
    offset = 0
    depth = 0
    
    while offset < len(binary_data):
        entry, new_offset = decode_entry(binary_data, offset)
        if not entry:
            break
            
        offset = new_offset
        
        # Display entry first (before handling traversal)
        indent = "  " * depth
        type_char = 'D' if entry.get('is_dir', False) else 'F'
        size = entry.get('size', 0)
        name = entry.get('name', '???')
        
        # Show permission delta if present
        perms_info = ""
        if 'perms_delta' in entry:
            perms_info = f" (Δperms: 0x{entry['perms_delta']:04x})"
        
        print(f"{indent}{type_char} {name:30} {size:>10} bytes{perms_info}")
        
        # Then handle traversal
        traverse = entry.get('traverse', 0)
        if traverse == TRAVERSE_DEEPER:
            depth += 1
        elif traverse == TRAVERSE_BACK:
            depth -= 1
            if depth < 0:
                depth = 0

if __name__ == '__main__':
    # Read from stdin or file
    if len(sys.argv) > 1:
        with open(sys.argv[1], 'rb') as f:
            data = f.read()
    else:
        data = sys.stdin.buffer.read()
    
    decode_quantum(data)