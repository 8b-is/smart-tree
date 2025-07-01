#!/usr/bin/env python3
"""
Smart Tree Quantum Format Decoder
Parse and analyze Smart Tree's quantum encoded output

This demonstrates:
1. Decoding various Smart Tree formats (quantum, claude, ai)
2. Extracting structured data
3. Analyzing directory statistics
4. Converting between formats

Inspired by Omni's insight: "Every format tells a story, quantum tells it efficiently" 🌊
"""

import json
import base64
import zlib
import re
from dataclasses import dataclass
from typing import List, Dict, Optional, Tuple
from collections import defaultdict

@dataclass
class FileEntry:
    """Represents a file or directory in the tree"""
    name: str
    depth: int
    size: Optional[int] = None
    permissions: Optional[str] = None
    is_directory: bool = False
    file_type: Optional[str] = None
    
    @property
    def extension(self) -> str:
        """Get file extension"""
        if '.' in self.name and not self.name.startswith('.'):
            return self.name.split('.')[-1].lower()
        return ''

class QuantumDecoder:
    """Decode Smart Tree's quantum/compressed formats"""
    
    def __init__(self, content: str):
        self.raw_content = content
        self.entries: List[FileEntry] = []
        self.metadata: Dict[str, any] = {}
        
    def decode(self) -> 'QuantumDecoder':
        """Decode the content based on format detection"""
        lines = self.raw_content.strip().split('\n')
        
        if not lines:
            return self
            
        first_line = lines[0]
        
        # Detect format
        if first_line.startswith("CLAUDE_V1:"):
            self._decode_claude_format(first_line)
        elif first_line.startswith("QUANTUM_V1:"):
            self._decode_quantum_format(lines)
        elif first_line.startswith("TREE_HEX_V1:"):
            self._decode_ai_format(lines)
        elif first_line.startswith("COMPRESSED_V1:"):
            self._decode_compressed_format(first_line)
        else:
            # Try hex format detection
            if lines and all(c in '0123456789abcdef ' for c in lines[0].split()[0]):
                self._decode_hex_format(lines)
            else:
                self._decode_classic_format(lines)
                
        return self
    
    def _decode_claude_format(self, line: str):
        """Decode claude format (base64 + zlib compression)"""
        header_end = line.index(':') + 1
        b64_data = line[header_end:]
        
        compressed = base64.b64decode(b64_data)
        decompressed = zlib.decompress(compressed)
        content = decompressed.decode('utf-8')
        
        # Now decode the decompressed content
        self.raw_content = content
        return self.decode()
    
    def _decode_compressed_format(self, line: str):
        """Decode compressed format"""
        header_end = line.index(':') + 1
        hex_data = line[header_end:]
        
        compressed = bytes.fromhex(hex_data)
        decompressed = zlib.decompress(compressed)
        content = decompressed.decode('utf-8')
        
        self.raw_content = content
        return self.decode()
    
    def _decode_quantum_format(self, lines: List[str]):
        """Decode native quantum format with tokenization"""
        # Extract token map from header
        header = lines[0]
        if '"tokens":' in header:
            import json
            header_data = header[header.index('{'):header.index('}')+1]
            tokens = json.loads(header_data).get('tokens', {})
            self.metadata['tokens'] = tokens
        
        # Parse entries (simplified - full implementation would handle all quantum features)
        for line in lines[1:]:
            if line.startswith('[') or not line.strip():
                continue
            # Quantum format parsing would go here
            # This is a simplified version
            pass
    
    def _decode_hex_format(self, lines: List[str]):
        """Decode hex format entries"""
        for line in lines:
            if not line.strip():
                continue
                
            parts = line.split(None, 7)  # Split into max 8 parts
            if len(parts) < 7:
                continue
                
            try:
                depth = int(parts[0], 16)
                perms = int(parts[1], 16)
                size = int(parts[4], 16)
                name = parts[-1] if len(parts) > 7 else parts[6]
                
                # Clean name
                name = name.strip()
                if name.startswith(('d ', 'f ')):
                    is_dir = name.startswith('d ')
                    name = name[2:]
                else:
                    is_dir = name.endswith('/')
                
                entry = FileEntry(
                    name=name.rstrip('/'),
                    depth=depth,
                    size=size if not is_dir else None,
                    permissions=f"{perms:03x}",
                    is_directory=is_dir
                )
                self.entries.append(entry)
            except:
                continue
    
    def _decode_ai_format(self, lines: List[str]):
        """Decode AI format with hex tree and stats"""
        in_stats = False
        
        for line in lines:
            if line.startswith("END_AI"):
                break
            elif line.strip().startswith(("F:", "D:", "S:")):
                in_stats = True
                self._parse_stats_line(line)
            elif not in_stats and line.strip():
                # It's a hex format line
                self._decode_hex_format([line])
    
    def _parse_stats_line(self, line: str):
        """Parse statistics from AI format"""
        if line.startswith("F:"):
            match = re.search(r'F:(\d+)', line)
            if match:
                self.metadata['total_files'] = int(match.group(1))
        elif line.startswith("D:"):
            match = re.search(r'D:(\d+)', line)
            if match:
                self.metadata['total_dirs'] = int(match.group(1))
        elif line.startswith("TYPES:"):
            types = {}
            for type_count in line[6:].split():
                if ':' in type_count:
                    ext, count = type_count.split(':')
                    types[ext] = int(count)
            self.metadata['file_types'] = types
    
    def _decode_classic_format(self, lines: List[str]):
        """Decode classic tree format"""
        depth_chars = {'├': 1, '└': 1, '│': 0}
        
        for line in lines:
            if not line.strip():
                continue
            
            # Calculate depth from tree characters
            depth = 0
            for char in line:
                if char == ' ':
                    depth += 0.25
                elif char in depth_chars:
                    depth += depth_chars[char]
                else:
                    break
            
            depth = int(depth)
            
            # Extract name
            name = line.strip()
            for char in ['├', '└', '─', '│', ' ']:
                name = name.replace(char, '')
            
            name = name.strip()
            if not name:
                continue
            
            # Parse size/metadata if present
            size = None
            if ' (' in name and ')' in name:
                try:
                    meta_start = name.rindex(' (')
                    meta = name[meta_start+2:-1]
                    name = name[:meta_start]
                    
                    # Try to parse size
                    size_match = re.search(r'([\d.]+)\s*([KMGT]B)', meta)
                    if size_match:
                        size_val = float(size_match.group(1))
                        unit = size_match.group(2)
                        multipliers = {'KB': 1024, 'MB': 1024**2, 'GB': 1024**3, 'TB': 1024**4}
                        size = int(size_val * multipliers.get(unit, 1))
                except:
                    pass
            
            is_dir = name.endswith('/') or '📁' in line
            name = name.rstrip('/')
            
            entry = FileEntry(
                name=name,
                depth=depth,
                size=size,
                is_directory=is_dir
            )
            self.entries.append(entry)
    
    def get_statistics(self) -> Dict[str, any]:
        """Calculate statistics from decoded entries"""
        stats = {
            'total_files': 0,
            'total_dirs': 0,
            'total_size': 0,
            'file_types': defaultdict(int),
            'largest_files': [],
            'depth_distribution': defaultdict(int)
        }
        
        # Use metadata if available
        if self.metadata:
            stats.update(self.metadata)
        
        # Calculate from entries
        file_sizes = []
        
        for entry in self.entries:
            stats['depth_distribution'][entry.depth] += 1
            
            if entry.is_directory:
                stats['total_dirs'] += 1
            else:
                stats['total_files'] += 1
                if entry.size:
                    stats['total_size'] += entry.size
                    file_sizes.append((entry.name, entry.size))
                
                if entry.extension:
                    stats['file_types'][entry.extension] += 1
        
        # Get largest files
        file_sizes.sort(key=lambda x: x[1], reverse=True)
        stats['largest_files'] = file_sizes[:10]
        
        return dict(stats)
    
    def to_json(self) -> str:
        """Convert to JSON format"""
        data = {
            'entries': [
                {
                    'name': e.name,
                    'depth': e.depth,
                    'size': e.size,
                    'is_directory': e.is_directory,
                    'extension': e.extension
                }
                for e in self.entries
            ],
            'statistics': self.get_statistics()
        }
        return json.dumps(data, indent=2)
    
    def to_simple_tree(self) -> str:
        """Convert to simple indented tree"""
        lines = []
        for entry in self.entries:
            indent = '  ' * entry.depth
            suffix = '/' if entry.is_directory else ''
            size_str = f" ({entry.size:,} bytes)" if entry.size else ""
            lines.append(f"{indent}{entry.name}{suffix}{size_str}")
        return '\n'.join(lines)


def main():
    """Example usage"""
    import sys
    import subprocess
    
    # Get path from command line or use current directory
    path = sys.argv[1] if len(sys.argv) > 1 else "."
    
    print(f"🔬 Analyzing Smart Tree output for: {path}\n")
    
    # Try different formats
    formats = ['claude', 'ai', 'hex', 'classic']
    
    for fmt in formats:
        print(f"\n{'='*60}")
        print(f"📊 Format: {fmt}")
        print('='*60)
        
        try:
            # Run Smart Tree
            result = subprocess.run(
                ["st", "-m", fmt, path, "--depth", "3"],
                capture_output=True,
                text=True,
                check=True
            )
            
            # Decode
            decoder = QuantumDecoder(result.stdout).decode()
            
            # Show statistics
            stats = decoder.get_statistics()
            print(f"📁 Directories: {stats.get('total_dirs', len([e for e in decoder.entries if e.is_directory]))}")
            print(f"📄 Files: {stats.get('total_files', len([e for e in decoder.entries if not e.is_directory]))}")
            print(f"💾 Total size: {stats.get('total_size', 0):,} bytes")
            
            # Show file type distribution
            if stats.get('file_types'):
                print(f"\n📊 File types:")
                for ext, count in sorted(stats['file_types'].items(), key=lambda x: x[1], reverse=True)[:5]:
                    print(f"   .{ext}: {count}")
            
            # Show largest files
            if stats.get('largest_files'):
                print(f"\n📈 Largest files:")
                for name, size in stats['largest_files'][:3]:
                    print(f"   {name}: {size:,} bytes")
            
            # Show compression ratio
            raw_size = len(result.stdout)
            print(f"\n🗜️  Output size: {raw_size:,} bytes")
            
        except subprocess.CalledProcessError:
            print(f"❌ Format {fmt} not available")
        except Exception as e:
            print(f"⚠️  Error: {e}")

if __name__ == "__main__":
    main()