#!/usr/bin/env python3
"""
Smart Tree Quantum Format Visualization
Decode and visualize directory structures from Smart Tree's quantum format

This example shows how to:
1. Decode Smart Tree's quantum/claude format
2. Parse the hierarchical structure
3. Create beautiful visualizations

Based on Omni's brilliant example from the Hot Tub session! 🛁
"""

import os
import sys
import base64
import zlib
import matplotlib.pyplot as plt
import networkx as nx
import subprocess

def run_smart_tree(path, mode="claude"):
    """Run Smart Tree and capture quantum/claude format output"""
    try:
        result = subprocess.run(
            ["st", "-m", mode, path],
            capture_output=True,
            text=True,
            check=True
        )
        return result.stdout
    except subprocess.CalledProcessError as e:
        print(f"Error running Smart Tree: {e}")
        sys.exit(1)

def decode_quantum_format(content):
    """Decode Smart Tree's quantum/claude format"""
    lines = content.strip().split('\n')
    
    # Check if it's compressed claude format
    if lines[0].startswith("CLAUDE_V1:"):
        # Extract base64 encoded data
        header_end = lines[0].index(':') + 1
        b64_data = lines[0][header_end:]
        
        # Decode base64 and decompress
        compressed = base64.b64decode(b64_data)
        decompressed = zlib.decompress(compressed)
        
        # The decompressed data is the tree structure
        return decompressed.decode('utf-8')
    
    # Otherwise return as-is
    return content

def parse_tree_structure(content):
    """Parse the tree structure into a graph"""
    G = nx.DiGraph()
    lines = content.strip().split('\n')
    
    # Skip header if present
    start_idx = 0
    for i, line in enumerate(lines):
        if not line.strip() or line.startswith('TREE_') or line.startswith('END_'):
            continue
        # Found first real content line
        start_idx = i
        break
    
    parent_stack = [("root", -1)]  # (name, indent_level)
    
    for line in lines[start_idx:]:
        if not line.strip() or line.startswith('END_'):
            break
            
        # Detect indentation (Smart Tree uses various formats)
        indent = 0
        
        # Classic format with tree characters
        if '├' in line or '└' in line or '│' in line:
            # Count leading spaces and tree chars
            for char in line:
                if char in ' │':
                    indent += 1
                else:
                    break
            indent = indent // 4  # Normalize
            
        # AI/Hex format (no indentation, uses depth in data)
        elif line.strip() and not line[0].isspace():
            # Try to parse hex depth if available
            parts = line.split()
            if parts and all(c in '0123456789abcdef' for c in parts[0]):
                indent = int(parts[0], 16)
            else:
                indent = 0
        else:
            # Standard space indentation
            indent = len(line) - len(line.lstrip())
            indent = indent // 2  # Normalize (assuming 2-space indent)
        
        # Extract name (remove tree characters and metadata)
        name = line.strip()
        for char in ['├', '└', '─', '│', ' ']:
            name = name.replace(char, '')
        
        # Remove file metadata if present (size, date, etc)
        if ' (' in name:
            name = name[:name.index(' (')]
        
        # Skip empty names
        if not name:
            continue
            
        # Find parent based on indentation
        while len(parent_stack) > 1 and parent_stack[-1][1] >= indent:
            parent_stack.pop()
        
        parent = parent_stack[-1][0]
        
        # Add to graph
        if parent == "root":
            G.add_node(name)
        else:
            G.add_edge(parent, name)
        
        # Add to stack for potential children
        parent_stack.append((name, indent))
    
    return G

def visualize_tree(G, title="Smart Tree Quantum Visualization"):
    """Create a beautiful visualization of the tree structure"""
    plt.figure(figsize=(20, 12))
    
    # Use hierarchical layout for tree-like structure
    try:
        # Try to use graphviz layout for better tree visualization
        pos = nx.nx_agraph.graphviz_layout(G, prog='dot')
    except:
        # Fallback to spring layout if graphviz not available
        pos = nx.spring_layout(G, k=2, iterations=50, seed=42)
    
    # Color nodes by type
    node_colors = []
    for node in G.nodes():
        if '.' not in node or node.endswith('/'):
            node_colors.append('lightblue')  # Directories
        elif node.endswith(('.py', '.rs', '.js', '.ts')):
            node_colors.append('lightgreen')  # Code files
        elif node.endswith(('.md', '.txt', '.rst')):
            node_colors.append('lightyellow')  # Docs
        elif node.endswith(('.json', '.toml', '.yaml', '.yml')):
            node_colors.append('lightcoral')  # Config
        else:
            node_colors.append('lightgray')  # Other files
    
    # Draw the graph
    nx.draw(G, pos, 
            with_labels=True,
            node_size=3000,
            node_color=node_colors,
            font_size=8,
            font_family="monospace",
            font_weight="bold",
            edge_color="gray",
            arrows=True,
            arrowsize=10,
            4.0.0=0.9)
    
    plt.title(title, fontsize=16, fontweight='bold')
    plt.tight_layout()
    plt.axis('off')
    
    # Add legend
    from matplotlib.patches import Patch
    legend_elements = [
        Patch(facecolor='lightblue', label='Directories'),
        Patch(facecolor='lightgreen', label='Code'),
        Patch(facecolor='lightyellow', label='Documentation'),
        Patch(facecolor='lightcoral', label='Config'),
        Patch(facecolor='lightgray', label='Other')
    ]
    plt.legend(handles=legend_elements, loc='upper right')
    
    return plt

def main():
    """Main function to run the visualization"""
    if len(sys.argv) > 1:
        path = sys.argv[1]
    else:
        path = "."
    
    print(f"🌳 Visualizing Smart Tree quantum format for: {path}")
    
    # Run Smart Tree in claude mode (maximum compression)
    print("📊 Running Smart Tree in claude mode...")
    content = run_smart_tree(path, mode="claude")
    
    # Decode the quantum format
    print("🔓 Decoding quantum format...")
    decoded = decode_quantum_format(content)
    
    # Parse into graph structure
    print("🔨 Building graph structure...")
    G = parse_tree_structure(decoded)
    
    print(f"📈 Created graph with {G.number_of_nodes()} nodes and {G.number_of_edges()} edges")
    
    # Visualize
    print("🎨 Creating visualization...")
    plt = visualize_tree(G, f"Smart Tree Quantum Visualization: {os.path.basename(os.path.abspath(path))}")
    
    # Save and show
    output_file = "smart_tree_quantum_viz.png"
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"💾 Saved visualization to {output_file}")
    
    plt.show()

if __name__ == "__main__":
    main()