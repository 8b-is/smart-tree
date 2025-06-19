#!/usr/bin/env python3
"""
SmartTree (stree) - An intelligent file tree visualization tool
"""
import os
import sys
import stat
import json
import base64
import zlib
import struct
from pathlib import Path
from dataclasses import dataclass, field
from typing import Dict, Set, List, Optional, Any, Union, Tuple
import click
from rich.console import Console
from rich.tree import Tree
from rich.markup import escape
from datetime import datetime
import fnmatch

@dataclass
class FileNode:
    """Represents a file or directory in the tree with additional context"""
    path: Path
    is_dir: bool
    stat_info: os.stat_result = None
    references: Set[Path] = field(default_factory=set)
    referenced_by: Set[Path] = field(default_factory=set)
    imports: Set[str] = field(default_factory=set)
    imported_by: Set[Path] = field(default_factory=set)

class SmartTree:
    def __init__(self, roo_path: str, display_mode: str = 'classic', no_emoji: bool = False):
        self.roo = Path(roo_path).resolve()
        self.nodes: Dict[Path, FileNode] = {}
        self.gitignore_patterns = self._load_gitignore()
        self.display_mode = display_mode
        self.use_color = True
        self.console = Console()
        self.no_emoji = no_emoji
        self.stats = {
            'total_files': 0,
            'total_dirs': 0,
            'total_size': 0,
            'file_types': {},
            'largest_files': [],
            'newest_files': [],
            'oldest_files': []
        }
    
    COLORS = {
        'depth': '\033[36m',  # cyan for depth
        'perm': '\033[33m',   # yellow for permissions
        'id': '\033[35m',     # magenta for uid/gid
        'size': '\033[32m',   # green for size
        'time': '\033[34m',   # blue for timestamp
        'reset': '\033[0m'    # reset
    }
    
    def _load_gitignore(self):
        """Load gitignore patterns if available"""
        gitignore_path = self.roo / '.gitignore'
        patterns = []
        if gitignore_path.exists():
            with open(gitignore_path, 'r') as f:
                for line in f:
                    line = line.strip()
                    if line and not line.startswith('#'):
                        patterns.append(line)
        return patterns
    
    def _is_ignored(self, path):
        """Check if a path should be ignored based on gitignore patterns"""
        rel_path = str(path.relative_to(self.roo))
        for pattern in self.gitignore_patterns:
            if fnmatch.fnmatch(rel_path, pattern):
                return True
        return False
    
    def _get_file_emoji(self, mode):
        """Get appropriate emoji for file type"""
        if self.no_emoji:
            if stat.S_ISDIR(mode): return "d"
            elif stat.S_ISLNK(mode): return "l"
            elif mode & stat.S_IXUSR: return "x"
            elif stat.S_ISSOCK(mode): return "s"
            elif stat.S_ISFIFO(mode): return "p"
            elif stat.S_ISBLK(mode): return "b"
            elif stat.S_ISCHR(mode): return "c"
            else: return "f"
        else:
            if stat.S_ISDIR(mode): return "📁"
            elif stat.S_ISLNK(mode): return "🔗"
            elif mode & stat.S_IXUSR: return "⚙️"
            elif stat.S_ISSOCK(mode): return "🔌"
            elif stat.S_ISFIFO(mode): return "📝"
            elif stat.S_ISBLK(mode): return "💾"
            elif stat.S_ISCHR(mode): return "📺"
            else: return "📄"
    
    def format_hex_node(self, path: Path, depth: int = 0) -> str:
        """Format a node in hex format with all metadata"""
        node = self.nodes[path]
        stat_info = node.stat_info
        
        # Convert values to hex with padding for alignment
        perms_hex = f"{stat_info.st_mode & 0o777:03x}"
        uid_hex = f"{stat_info.st_uid:04x}"  # 4 hex digits for uid
        gid_hex = f"{stat_info.st_gid:04x}"  # 4 hex digits for gid
        size_hex = f"{stat_info.st_size:08x}"  # 8 hex digits for size (up to 4GB)
        time_hex = f"{int(stat_info.st_mtime):08x}"  # 8 hex digits for timestamp
        depth_hex = f"{depth:x}"
        
        emoji = self._get_file_emoji(stat_info.st_mode)
        
        if self.use_color:
            return (f"{self.COLORS['depth']}{depth_hex}{self.COLORS['reset']} "
                   f"{self.COLORS['perm']}{perms_hex}{self.COLORS['reset']} "
                   f"{self.COLORS['id']}{uid_hex} {gid_hex}{self.COLORS['reset']} "
                   f"{self.COLORS['size']}{size_hex}{self.COLORS['reset']} "
                   f"{self.COLORS['time']}{time_hex}{self.COLORS['reset']} "
                   f"{emoji} {path.name}")
        else:
            return f"{depth_hex} {perms_hex} {uid_hex} {gid_hex} {size_hex} {time_hex} {emoji} {path.name}"
    
    def format_classic_node(self, path: Path) -> str:
        """Format node in classic tree style"""
        node = self.nodes[path]
        stat_info = node.stat_info
        
        emoji = self._get_file_emoji(stat_info.st_mode)
        size_str = f"{stat_info.st_size:,} bytes" if not stat.S_ISDIR(stat_info.st_mode) else "directory"
        mod_time = datetime.fromtimestamp(stat_info.st_mtime).strftime("%Y-%m-%d %H:%M")
        
        return f"{emoji} {path.name} ({size_str}, modified: {mod_time})"
    
    def scan(self, max_depth=10, filter_ignored=True, filters=None):
        """Scan the directory tree and build node structure"""
        def should_include(item: Path, stat_info) -> bool:
            """Check if item should be included based on filters"""
            if not filters:
                return True
            
            # File type filter
            if filters.get('file_type') and item.suffix.lstrip('.') != filters['file_type']:
                return False
            
            # Size filters (only for files)
            if not stat.S_ISDIR(stat_info.st_mode):
                if filters.get('min_size') and stat_info.st_size < filters['min_size']:
                    return False
                if filters.get('max_size') and stat_info.st_size > filters['max_size']:
                    return False
            
            # Date filters
            if filters.get('newer_than'):
                newer_date = datetime.strptime(filters['newer_than'], '%Y-%m-%d').timestamp()
                if stat_info.st_mtime < newer_date:
                    return False
            if filters.get('older_than'):
                older_date = datetime.strptime(filters['older_than'], '%Y-%m-%d').timestamp()
                if stat_info.st_mtime > older_date:
                    return False
            
            return True
        
        def scan_dir(current: Path, depth=0):
            if depth > max_depth:
                return
            
            try:
                for item in current.iterdir():
                    # Skip if matches gitignore pattern
                    if filter_ignored and self._is_ignored(item):
                        continue
                    
                    try:
                        stat_info = item.stat()
                        is_dir = item.is_dir() and not item.is_symlink()
                        
                        # Apply filters
                        if not should_include(item, stat_info):
                            continue
                        
                        # Create node
                        self.nodes[item] = FileNode(
                            path=item,
                            is_dir=is_dir,
                            stat_info=stat_info
                        )
                        
                        # Update statistics
                        if is_dir:
                            self.stats['total_dirs'] += 1
                        else:
                            self.stats['total_files'] += 1
                            self.stats['total_size'] += stat_info.st_size
                            
                            # Track file types
                            ext = item.suffix.lstrip('.')
                            if ext:
                                self.stats['file_types'][ext] = self.stats['file_types'].get(ext, 0) + 1
                            
                            # Track largest files
                            self.stats['largest_files'].append((stat_info.st_size, item))
                            self.stats['largest_files'].sort(reverse=True, key=lambda x: x[0])
                            self.stats['largest_files'] = self.stats['largest_files'][:10]
                            
                            # Track newest/oldest files
                            self.stats['newest_files'].append((stat_info.st_mtime, item))
                            self.stats['newest_files'].sort(reverse=True, key=lambda x: x[0])
                            self.stats['newest_files'] = self.stats['newest_files'][:10]
                            
                            self.stats['oldest_files'].append((stat_info.st_mtime, item))
                            self.stats['oldest_files'].sort(key=lambda x: x[0])
                            self.stats['oldest_files'] = self.stats['oldest_files'][:10]
                        
                        # Recursively scan directories
                        if is_dir:
                            scan_dir(item, depth + 1)
                    except (PermissionError, FileNotFoundError):
                        # Handle permission errors gracefully
                        pass
            except (PermissionError, FileNotFoundError):
                pass
        
        # Start scanning from roo
        self.nodes[self.roo] = FileNode(
            path=self.roo,
            is_dir=True, 
            stat_info=self.roo.stat()
        )
        scan_dir(self.roo)
    
    def format_json_node(self, path: Path) -> Dict[str, Any]:
        """Format a node for JSON output"""
        node = self.nodes[path]
        stat_info = node.stat_info
        
        # Get file type
        file_type = 'directory' if stat.S_ISDIR(stat_info.st_mode) else \
                   'symlink' if stat.S_ISLNK(stat_info.st_mode) else \
                   'executable' if stat_info.st_mode & stat.S_IXUSR else \
                   'socket' if stat.S_ISSOCK(stat_info.st_mode) else \
                   'pipe' if stat.S_ISFIFO(stat_info.st_mode) else \
                   'block_device' if stat.S_ISBLK(stat_info.st_mode) else \
                   'character_device' if stat.S_ISCHR(stat_info.st_mode) else 'file'
        
        # Format permissions in octal
        perms = f"{stat_info.st_mode & 0o777:o}"
        
        # Create JSON object
        return {
            'name': path.name,
            'path': str(path),
            'type': file_type,
            'size': stat_info.st_size,
            'permissions': perms,
            'uid': stat_info.st_uid,
            'gid': stat_info.st_gid,
            'modified': datetime.fromtimestamp(stat_info.st_mtime).isoformat(),
            'created': datetime.fromtimestamp(stat_info.st_ctime).isoformat(),
        }
    
    def build_json_tree(self) -> Dict[str, Any]:
        """Build a JSON representation of the tree"""
        def add_to_json(path: Path) -> Dict[str, Any]:
            if path not in self.nodes:
                return {}
            
            node_json = self.format_json_node(path)
            
            if self.nodes[path].is_dir:
                child_paths = sorted(
                    [p for p in self.nodes if p.parent == path],
                    key=lambda p: (not self.nodes[p].is_dir, p.name)
                )
                
                children = []
                for child_path in child_paths:
                    child_json = add_to_json(child_path)
                    if child_json:
                        children.append(child_json)
                
                node_json['children'] = children
            
            return node_json
        
        return add_to_json(self.roo)
    
    
    def build_rich_tree(self, compact=False, show_ignored=False, compress=False):
        """Build and display a tree visualization"""
        # Capture output if compression is requested
        if compress:
            import io
            import sys
            old_stdout = sys.stdout
            sys.stdout = buffer = io.StringIO()
        
        try:
            if self.display_mode == 'json':
                # Output JSON format
                json_tree = self.build_json_tree()
                indent = None if compact else 2
                print(json.dumps(json_tree, indent=indent))
            elif self.display_mode == 'ai':
                # Optimal AI format with hex + stats
                self.print_ai()
            elif self.display_mode == 'hex':
                # Custom hex output without tree lines
                self.print_hex_tree(show_ignored=show_ignored)
            elif self.display_mode == 'stats':
                # Output statistics
                self.print_stats()
            elif self.display_mode == 'csv':
                # Output CSV format
                self.print_csv()
            elif self.display_mode == 'tsv':
                # Output TSV format
                self.print_tsv()
            else:
                # Classic mode with rich tree
                self._print_classic_tree()
        finally:
            if compress:
                # Get the output and compress it
                sys.stdout = old_stdout
                output = buffer.getvalue()
                self._print_compressed(output)
    
    def _print_classic_tree(self):
        """Print classic tree format using Rich"""
        def add_to_tree(node: Tree, path: Path, depth=0):
            if path not in self.nodes:
                return
            
            current = self.nodes[path]
            child_paths = sorted(
                [p for p in self.nodes if p.parent == path],
                key=lambda p: (not self.nodes[p].is_dir, p.name)
            )
            
            for child_path in child_paths:
                child = self.nodes[child_path]
                node_text = self.format_classic_node(child_path)
                child_node = node.add(node_text)
                if child.is_dir:
                    add_to_tree(child_node, child_path, depth + 1)
        
        roo_node = Tree(self.format_classic_node(self.roo))
        add_to_tree(roo_node, self.roo)
        self.console.print(roo_node)
    
    def _print_compressed(self, output: str):
        """Compress and print output in hex format"""
        try:
            # Compress with zlib
            compressed = zlib.compress(output.encode('utf-8'))
            
            # Encode to hex
            hex_output = base64.b16encode(compressed).decode('ascii')
            
            # Print with header
            print(f"COMPRESSED_V1:{hex_output}")
            
            # Print stats if DEBUG is set
            if os.environ.get('DEBUG') == '1':
                original_size = len(output)
                compressed_size = len(compressed)
                ratio = original_size / compressed_size if compressed_size > 0 else 0
                print(f"\n# DEBUG: {original_size} bytes -> {compressed_size} bytes (ratio: {ratio:.2f}x)", file=sys.stderr)
        except Exception as e:
            print(f"ERROR_COMPRESSING: {str(e)}")
    
    def print_hex_tree(self, path: Path = None, depth: int = 0, show_ignored: bool = False):
        """Print tree in hex format without indentation"""
        if path is None:
            path = self.roo
            # Print root
            print(self.format_hex_node(path, depth))
        
        if path not in self.nodes:
            return
        
        # Get children
        child_paths = sorted(
            [p for p in self.nodes if p.parent == path],
            key=lambda p: (not self.nodes[p].is_dir, p.name)
        )
        
        # Also check for ignored directories if show_ignored is True
        if show_ignored and self.gitignore_patterns:
            try:
                all_items = list(path.iterdir())
                for item in all_items:
                    if item not in self.nodes and item.is_dir() and self._is_ignored(item):
                        # Format ignored directory with brackets
                        stat_info = item.stat()
                        perms_hex = f"{stat_info.st_mode & 0o777:03x}"
                        uid_hex = f"{stat_info.st_uid:04x}"
                        gid_hex = f"{stat_info.st_gid:04x}"
                        size_hex = f"{0:08x}"  # Dirs shown as 0 size
                        time_hex = f"{int(stat_info.st_mtime):08x}"
                        depth_hex = f"{depth + 1:x}"
                        emoji = self._get_file_emoji(stat_info.st_mode)
                        
                        # Print with brackets to show it's ignored
                        print(f"{depth_hex} {perms_hex} {uid_hex} {gid_hex} {size_hex} {time_hex} {emoji} [{item.name}]")
            except (PermissionError, FileNotFoundError): 
                pass
        
        for child_path in child_paths:
            # No indentation - depth is shown in the hex value
            print(self.format_hex_node(child_path, depth + 1))
            
            # Recurse for directories
            if self.nodes[child_path].is_dir:
                self.print_hex_tree(child_path, depth + 1, show_ignored)
    
    def print_stats(self):
        """Print directory statistics"""
        print(f"{'='*60}")
        print(f"Directory Statistics for: {self.roo}")
        print(f"{'='*60}")
        print(f"Total Files: {self.stats['total_files']:,}")
        print(f"Total Directories: {self.stats['total_dirs']:,}")
        print(f"Total Size: {self.stats['total_size']:,} bytes ({self.stats['total_size'] / (1024**3):.2f} GB)")
        print()
        
        if self.stats['file_types']:
            print("File Types (by count):")
            for ext, count in sorted(self.stats['file_types'].items(), key=lambda x: x[1], reverse=True)[:20]:
                print(f"  .{ext}: {count}")
            print()
        
        if self.stats['largest_files']:
            print("Largest Files:")
            for size, path in self.stats['largest_files'][:10]:
                print(f"  {size:12,} bytes  {path.relative_to(self.roo)}")
            print()
        
        if self.stats['newest_files']:
            print("Newest Files:")
            for mtime, path in self.stats['newest_files'][:5]:
                print(f"  {datetime.fromtimestamp(mtime).strftime('%Y-%m-%d %H:%M')}  {path.relative_to(self.roo)}")
            print()
        
        if self.stats['oldest_files']:
            print("Oldest Files:")
            for mtime, path in self.stats['oldest_files'][:5]:
                print(f"  {datetime.fromtimestamp(mtime).strftime('%Y-%m-%d %H:%M')}  {path.relative_to(self.roo)}")
    
    def print_csv(self):
        """Print in CSV format"""
        import csv
        import sys
        
        writer = csv.writer(sys.stdout)
        writer.writerow(['path', 'type', 'size', 'permissions', 'uid', 'gid', 'modified', 'depth'])
        
        def write_node(path: Path, depth: int = 0):
            if path in self.nodes:
                node = self.nodes[path]
                stat_info = node.stat_info
                writer.writerow([
                    str(path.relative_to(self.roo) if path != self.roo else '.'),
                    'd' if node.is_dir else 'f',
                    stat_info.st_size,
                    oct(stat_info.st_mode)[-3:],
                    stat_info.st_uid,
                    stat_info.st_gid,
                    datetime.fromtimestamp(stat_info.st_mtime).isoformat(),
                    depth
                ])
                
                # Process children
                if node.is_dir:
                    child_paths = sorted(
                        [p for p in self.nodes if p.parent == path],
                        key=lambda p: (not self.nodes[p].is_dir, p.name)
                    )
                    for child_path in child_paths:
                        write_node(child_path, depth + 1)
        
        write_node(self.roo)
    
    def print_tsv(self):
        """Print in TSV format"""
        def write_node(path: Path, depth: int = 0):
            if path in self.nodes:
                node = self.nodes[path]
                stat_info = node.stat_info
                print('\t'.join([
                    str(path.relative_to(self.roo) if path != self.roo else '.'),
                    'd' if node.is_dir else 'f',
                    str(stat_info.st_size),
                    oct(stat_info.st_mode)[-3:],
                    str(stat_info.st_uid),
                    str(stat_info.st_gid),
                    datetime.fromtimestamp(stat_info.st_mtime).isoformat(),
                    str(depth)
                ]))
                
                # Process children
                if node.is_dir:
                    child_paths = sorted(
                        [p for p in self.nodes if p.parent == path],
                        key=lambda p: (not self.nodes[p].is_dir, p.name)
                    )
                    for child_path in child_paths:
                        write_node(child_path, depth + 1)
        
        # Print header
        print('\t'.join(['path', 'type', 'size', 'permissions', 'uid', 'gid', 'modified', 'depth']))
        write_node(self.roo)
    
    def print_ai(self):
        """Print optimal AI format with hex tree and statistics"""
        # First print the hex tree
        print("TREE_HEX_V1:")
        self.print_hex_tree(show_ignored=True)
        
        # Then print compact statistics
        print("\nSTATS:")
        print(f"F:{self.stats['total_files']} D:{self.stats['total_dirs']} S:{self.stats['total_size']:x} ({self.stats['total_size'] / (1024**2):.1f}MB)")
        
        # File type summary
        if self.stats['file_types']:
            types_str = " ".join([f"{ext}:{count}" for ext, count in sorted(self.stats['file_types'].items(), key=lambda x: x[1], reverse=True)[:10]])
            print(f"TYPES: {types_str}")
        
        # Largest files (top 5)
        if self.stats['largest_files']:
            large_str = " ".join([f"{path.name}:{size:x}" for size, path in self.stats['largest_files'][:5]])
            print(f"LARGE: {large_str}")
        
        # Date range
        if self.stats['oldest_files'] and self.stats['newest_files']:
            oldest = int(self.stats['oldest_files'][0][0])
            newest = int(self.stats['newest_files'][0][0])
            print(f"DATES: {oldest:x}-{newest:x}")
        
        print("END_AI")

@click.command()
@click.argument('path', default='.')
@click.option('--mode', '-m', default='classic', 
              type=click.Choice(['classic', 'hex', 'json', 'ai', 'stats', 'csv', 'tsv']), 
              help='Display mode (classic, hex, json, ai for optimal AI format, stats, csv, or tsv)')
@click.option('--compress', '-z', is_flag=True, help='Compress output with zlib (hex encoding)')
@click.option('--compact/--no-compact', default=False, help='Output compact JSON without whitespace for JSON-based modes')
@click.option('--depth', '-d', default=5, help='Maximum directory depth')
@click.option('--ignore/--no-ignore', default=True, help='Respect .gitignore files (default: ON)')
@click.option('--show-ignored', is_flag=True, help='Show ignored directories in brackets')
@click.option('--no-emoji', is_flag=True, help='Disable emoji in output')
@click.option('--filter-type', help='Filter by file type (e.g., "py", "js", "md")')
@click.option('--min-size', type=int, help='Minimum file size in bytes')
@click.option('--max-size', type=int, help='Maximum file size in bytes')
@click.option('--newer-than', help='Show files newer than date (YYYY-MM-DD)')
@click.option('--older-than', help='Show files older than date (YYYY-MM-DD)')
def main(path, mode, depth, ignore, compact, show_ignored, no_emoji, filter_type, min_size, max_size, newer_than, older_than, compress):
    """Smart tree visualization tool
    
    Set SMART_TREE_JSON=1 environment variable to automatically use json mode.
    Set AI_TOOLS=1 environment variable to use optimal AI mode.
    
    Use --compress (-z) flag to compress any output mode with zlib.
    """
    # Check for SMART_TREE_JSON environment variable to automatically use json mode
    if os.environ.get('SMART_TREE_JSON') == '1':
        if mode in ['classic', 'hex']:
            mode = 'json'
    
    # Check for AI_TOOLS environment variable to use optimal AI mode
    if os.environ.get('AI_TOOLS') == '1':
        mode = 'ai'
        # Also enable compression by default for AI mode
        compress = True
    
    # Build filters dictionary
    filters = {}
    if filter_type:
        filters['file_type'] = filter_type
    if min_size is not None:
        filters['min_size'] = min_size
    if max_size is not None:
        filters['max_size'] = max_size
    if newer_than:
        filters['newer_than'] = newer_than
    if older_than:
        filters['older_than'] = older_than
    
    tree = SmartTree(path, display_mode=mode, no_emoji=no_emoji)
    tree.scan(max_depth=depth, filter_ignored=ignore, filters=filters)
    tree.build_rich_tree(compact=compact, show_ignored=show_ignored, compress=compress)

if __name__ == '__main__':
    main()