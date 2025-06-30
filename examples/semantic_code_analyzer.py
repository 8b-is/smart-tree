#!/usr/bin/env python3
"""
Semantic Code Structure Analysis
Analyze code semantics using Smart Tree's quantum tokenization

This demonstrates:
1. Leveraging quantum format's token mapping for semantic analysis
2. Identifying code patterns and structures
3. Finding similar code blocks across projects
4. Visualizing semantic relationships

"Code flows like water, patterns emerge like waves" - Omni 🌊
"""

import os
import sys
import json
import base64
import zlib
import subprocess
import re
from typing import Dict, List, Set, Tuple, Optional
from collections import defaultdict, Counter
import matplotlib.pyplot as plt
import networkx as nx
from dataclasses import dataclass
import numpy as np
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.metrics.pairwise import cosine_similarity

@dataclass
class CodeToken:
    """Represents a semantic token in code"""
    value: str
    token_id: int
    frequency: int
    semantic_type: str  # 'keyword', 'identifier', 'literal', 'operator', 'structure'
    
@dataclass
class CodeBlock:
    """Represents a semantic code block"""
    path: str
    tokens: List[CodeToken]
    depth: int
    block_type: str  # 'function', 'class', 'module', 'config'
    fingerprint: str  # Semantic fingerprint
    
    def token_vector(self) -> List[int]:
        """Get token frequency vector"""
        vector = defaultdict(int)
        for token in self.tokens:
            vector[token.token_id] += 1
        return vector

class SemanticAnalyzer:
    """Analyze code semantics using quantum tokenization"""
    
    def __init__(self):
        self.token_map: Dict[str, int] = {}
        self.reverse_token_map: Dict[int, str] = {}
        self.code_blocks: List[CodeBlock] = []
        self.semantic_graph = nx.DiGraph()
        
        # Common programming patterns
        self.patterns = {
            'function': re.compile(r'(def|function|func|fn)\s+(\w+)'),
            'class': re.compile(r'(class|struct|interface)\s+(\w+)'),
            'import': re.compile(r'(import|require|use|include)\s+(.+)'),
            'variable': re.compile(r'(const|let|var|val)\s+(\w+)'),
            'loop': re.compile(r'(for|while|loop)\s*\('),
            'condition': re.compile(r'(if|else|elif|when)\s*\('),
            'error': re.compile(r'(try|catch|except|error|throw)'),
            'async': re.compile(r'(async|await|promise|future)'),
        }
        
    def analyze_quantum_output(self, quantum_data: str) -> Dict[str, any]:
        """Extract semantic information from quantum format"""
        # Check for quantum format with token map
        if quantum_data.startswith("QUANTUM_V1:"):
            header_end = quantum_data.find('\n')
            header = quantum_data[:header_end]
            
            # Extract token map from header
            if '"tokens":' in header:
                header_json = header[header.index('{'):header.index('}')+1]
                data = json.loads(header_json)
                self.token_map = data.get('tokens', {})
                self.reverse_token_map = {v: k for k, v in self.token_map.items()}
                
                return self._analyze_with_tokens(quantum_data[header_end+1:])
        
        # Fallback to claude format
        elif quantum_data.startswith("CLAUDE_V1:"):
            return self._analyze_claude_format(quantum_data)
        
        return {}
    
    def _analyze_claude_format(self, data: str) -> Dict[str, any]:
        """Analyze claude compressed format"""
        # Decode
        b64_data = data.split(':', 1)[1]
        compressed = base64.b64decode(b64_data)
        decompressed = zlib.decompress(compressed)
        content = decompressed.decode('utf-8')
        
        # Build token map from content
        self._build_token_map_from_content(content)
        
        return self._analyze_tree_content(content)
    
    def _build_token_map_from_content(self, content: str):
        """Build token map from file content"""
        # Extract all identifiers and keywords
        tokens = set()
        
        for line in content.split('\n'):
            if not line.strip():
                continue
            
            # Extract filename
            parts = line.split()
            if len(parts) > 7:  # Hex format
                filename = parts[-1]
                if '.' in filename:
                    ext = filename.split('.')[-1]
                    tokens.add(ext)
                
                # Extract name parts
                name_parts = filename.replace('/', ' ').replace('-', ' ').replace('_', ' ').split()
                tokens.update(name_parts)
        
        # Assign token IDs
        for i, token in enumerate(sorted(tokens)):
            self.token_map[token] = i
            self.reverse_token_map[i] = token
    
    def _analyze_tree_content(self, content: str) -> Dict[str, any]:
        """Analyze tree content for semantic patterns"""
        blocks_by_type = defaultdict(list)
        file_relationships = []
        
        for line in content.split('\n'):
            if not line.strip():
                continue
            
            # Parse hex format line
            parts = line.split(None, 7)
            if len(parts) >= 7:
                try:
                    depth = int(parts[0], 16)
                    filename = parts[-1].strip()
                    
                    # Clean filename
                    if filename.startswith(('d ', 'f ')):
                        is_dir = filename.startswith('d ')
                        filename = filename[2:]
                    
                    if not is_dir and '.' in filename:
                        # Analyze file semantics
                        ext = filename.split('.')[-1].lower()
                        name = filename.rsplit('.', 1)[0]
                        
                        # Determine semantic type
                        semantic_type = self._get_semantic_type(filename, ext)
                        
                        # Create code block
                        tokens = self._tokenize_filename(filename)
                        block = CodeBlock(
                            path=filename,
                            tokens=tokens,
                            depth=depth,
                            block_type=semantic_type,
                            fingerprint=self._generate_fingerprint(tokens)
                        )
                        
                        self.code_blocks.append(block)
                        blocks_by_type[semantic_type].append(block)
                        
                        # Find relationships
                        if semantic_type in ['test', 'spec']:
                            # Find related source file
                            source_name = name.replace('_test', '').replace('.test', '').replace('_spec', '').replace('.spec', '')
                            file_relationships.append((source_name, filename, 'tests'))
                        
                except:
                    continue
        
        return {
            'blocks_by_type': dict(blocks_by_type),
            'relationships': file_relationships,
            'token_count': len(self.token_map),
            'block_count': len(self.code_blocks)
        }
    
    def _get_semantic_type(self, filename: str, ext: str) -> str:
        """Determine semantic type of file"""
        name_lower = filename.lower()
        
        # Test files
        if any(pattern in name_lower for pattern in ['test', 'spec', '_test.', '.test.', '_spec.', '.spec.']):
            return 'test'
        
        # Configuration
        if ext in ['json', 'yaml', 'yml', 'toml', 'ini', 'env', 'conf', 'config']:
            return 'config'
        
        # Documentation
        if ext in ['md', 'rst', 'txt', 'doc', 'pdf']:
            return 'documentation'
        
        # Build files
        if filename in ['Makefile', 'CMakeLists.txt', 'package.json', 'Cargo.toml', 'pom.xml', 'build.gradle']:
            return 'build'
        
        # Source code by extension
        code_types = {
            'py': 'python',
            'js': 'javascript',
            'ts': 'typescript',
            'rs': 'rust',
            'go': 'go',
            'java': 'java',
            'cpp': 'cpp',
            'c': 'c',
            'rb': 'ruby',
            'php': 'php'
        }
        
        return code_types.get(ext, 'source')
    
    def _tokenize_filename(self, filename: str) -> List[CodeToken]:
        """Tokenize filename into semantic tokens"""
        tokens = []
        
        # Split by common separators
        parts = re.split(r'[/\-_.]+', filename.lower())
        
        for part in parts:
            if part in self.token_map:
                token_id = self.token_map[part]
            else:
                token_id = len(self.token_map)
                self.token_map[part] = token_id
                self.reverse_token_map[token_id] = part
            
            # Determine semantic type
            if part in ['test', 'spec', 'mock']:
                sem_type = 'test'
            elif part in ['config', 'settings', 'env']:
                sem_type = 'config'
            elif part in ['main', 'index', 'app']:
                sem_type = 'entry'
            elif part in ['util', 'helper', 'common']:
                sem_type = 'utility'
            else:
                sem_type = 'identifier'
            
            tokens.append(CodeToken(
                value=part,
                token_id=token_id,
                frequency=1,
                semantic_type=sem_type
            ))
        
        return tokens
    
    def _generate_fingerprint(self, tokens: List[CodeToken]) -> str:
        """Generate semantic fingerprint for code block"""
        # Sort tokens by ID and create a fingerprint
        token_ids = sorted([t.token_id for t in tokens])
        fingerprint = '-'.join(map(str, token_ids))
        return fingerprint
    
    def find_similar_blocks(self, threshold: float = 0.7) -> List[Tuple[CodeBlock, CodeBlock, float]]:
        """Find semantically similar code blocks"""
        similar_pairs = []
        
        if len(self.code_blocks) < 2:
            return similar_pairs
        
        # Create TF-IDF vectors
        documents = []
        for block in self.code_blocks:
            # Create document from tokens
            doc = ' '.join([t.value for t in block.tokens])
            documents.append(doc)
        
        # Compute TF-IDF
        vectorizer = TfidfVectorizer()
        tfidf_matrix = vectorizer.fit_transform(documents)
        
        # Compute pairwise similarities
        similarities = cosine_similarity(tfidf_matrix)
        
        # Find similar pairs
        for i in range(len(self.code_blocks)):
            for j in range(i + 1, len(self.code_blocks)):
                similarity = similarities[i][j]
                if similarity >= threshold:
                    similar_pairs.append((
                        self.code_blocks[i],
                        self.code_blocks[j],
                        similarity
                    ))
        
        return sorted(similar_pairs, key=lambda x: x[2], reverse=True)
    
    def build_semantic_graph(self):
        """Build graph of semantic relationships"""
        # Add nodes for each code block
        for block in self.code_blocks:
            self.semantic_graph.add_node(
                block.path,
                block_type=block.block_type,
                depth=block.depth,
                tokens=len(block.tokens)
            )
        
        # Add edges for similar blocks
        similar_blocks = self.find_similar_blocks(threshold=0.5)
        for block1, block2, similarity in similar_blocks:
            self.semantic_graph.add_edge(
                block1.path,
                block2.path,
                weight=similarity,
                relationship='similar'
            )
        
        # Add edges for test relationships
        for block in self.code_blocks:
            if block.block_type == 'test':
                # Find related source file
                base_name = block.path.replace('_test', '').replace('.test', '').replace('_spec', '').replace('.spec', '')
                for other in self.code_blocks:
                    if other.path != block.path and base_name in other.path:
                        self.semantic_graph.add_edge(
                            other.path,
                            block.path,
                            relationship='tested_by'
                        )
    
    def visualize_semantic_network(self, output_file: str = "semantic_network.png"):
        """Visualize semantic code relationships"""
        if not self.semantic_graph.nodes():
            print("No semantic graph to visualize")
            return
        
        plt.figure(figsize=(20, 16))
        
        # Layout using spring algorithm
        pos = nx.spring_layout(self.semantic_graph, k=3, iterations=50, seed=42)
        
        # Color nodes by type
        color_map = {
            'python': '#3776AB',
            'javascript': '#F7DF1E',
            'typescript': '#3178C6',
            'rust': '#CE442C',
            'go': '#00ADD8',
            'java': '#007396',
            'config': '#6DB33F',
            'test': '#FF6B6B',
            'documentation': '#4ECDC4',
            'build': '#95E1D3',
            'source': '#A8A8A8'
        }
        
        node_colors = []
        node_sizes = []
        for node in self.semantic_graph.nodes():
            node_data = self.semantic_graph.nodes[node]
            block_type = node_data.get('block_type', 'source')
            node_colors.append(color_map.get(block_type, '#A8A8A8'))
            node_sizes.append(1000 + node_data.get('tokens', 1) * 100)
        
        # Draw nodes
        nx.draw_networkx_nodes(
            self.semantic_graph, pos,
            node_color=node_colors,
            node_size=node_sizes,
            alpha=0.8
        )
        
        # Draw edges with different styles for different relationships
        edge_colors = []
        edge_styles = []
        edge_widths = []
        
        for u, v, data in self.semantic_graph.edges(data=True):
            rel = data.get('relationship', 'similar')
            if rel == 'tested_by':
                edge_colors.append('green')
                edge_styles.append('dashed')
                edge_widths.append(2)
            else:
                weight = data.get('weight', 0.5)
                edge_colors.append('gray')
                edge_styles.append('solid')
                edge_widths.append(weight * 3)
        
        # Draw edges
        for i, (u, v) in enumerate(self.semantic_graph.edges()):
            nx.draw_networkx_edges(
                self.semantic_graph, pos,
                [(u, v)],
                edge_color=edge_colors[i],
                style=edge_styles[i],
                width=edge_widths[i],
                alpha=0.5,
                arrows=True,
                arrowsize=10
            )
        
        # Draw labels
        labels = {}
        for node in self.semantic_graph.nodes():
            # Shorten long paths
            label = node.split('/')[-1] if '/' in node else node
            if len(label) > 20:
                label = label[:17] + '...'
            labels[node] = label
        
        nx.draw_networkx_labels(
            self.semantic_graph, pos,
            labels,
            font_size=8,
            font_family='monospace'
        )
        
        plt.title("Semantic Code Network Analysis", fontsize=20, fontweight='bold', pad=20)
        plt.axis('off')
        
        # Add legend
        from matplotlib.patches import Patch, Line2D
        legend_elements = [
            Patch(facecolor='#3776AB', label='Python'),
            Patch(facecolor='#F7DF1E', label='JavaScript'),
            Patch(facecolor='#CE442C', label='Rust'),
            Patch(facecolor='#6DB33F', label='Config'),
            Patch(facecolor='#FF6B6B', label='Tests'),
            Line2D([0], [0], color='green', linestyle='dashed', label='Test Relationship'),
            Line2D([0], [0], color='gray', label='Semantic Similarity')
        ]
        plt.legend(handles=legend_elements, loc='upper left', bbox_to_anchor=(0, 1))
        
        plt.tight_layout()
        plt.savefig(output_file, dpi=300, bbox_inches='tight')
        print(f"💾 Saved semantic network to {output_file}")
        plt.show()
    
    def generate_semantic_report(self) -> str:
        """Generate semantic analysis report"""
        report = []
        report.append("🧠 Semantic Code Analysis Report")
        report.append("=" * 50)
        
        # Token statistics
        report.append(f"\n📊 Token Statistics:")
        report.append(f"  Unique tokens: {len(self.token_map)}")
        report.append(f"  Code blocks analyzed: {len(self.code_blocks)}")
        
        # Block type distribution
        type_counts = Counter(block.block_type for block in self.code_blocks)
        report.append(f"\n📁 Code Distribution:")
        for block_type, count in type_counts.most_common():
            report.append(f"  {block_type}: {count}")
        
        # Semantic patterns
        similar_blocks = self.find_similar_blocks(threshold=0.8)
        if similar_blocks:
            report.append(f"\n🔍 Highly Similar Code Blocks (>80% similarity):")
            for block1, block2, similarity in similar_blocks[:5]:
                report.append(f"  {block1.path}")
                report.append(f"  ≈ {block2.path}")
                report.append(f"  Similarity: {similarity:.1%}\n")
        
        # Test coverage insights
        test_blocks = [b for b in self.code_blocks if b.block_type == 'test']
        if test_blocks:
            report.append(f"\n🧪 Test Coverage Insights:")
            report.append(f"  Test files: {len(test_blocks)}")
            report.append(f"  Test ratio: {len(test_blocks) / len(self.code_blocks):.1%}")
        
        # Complexity indicators
        deep_blocks = [b for b in self.code_blocks if b.depth > 5]
        if deep_blocks:
            report.append(f"\n⚠️  Deep Nesting Detected:")
            for block in deep_blocks[:5]:
                report.append(f"  {block.path} (depth: {block.depth})")
        
        return '\n'.join(report)

def main():
    """Example usage"""
    if len(sys.argv) > 1:
        path = sys.argv[1]
    else:
        path = "."
    
    print("🧠 Semantic Code Structure Analyzer")
    print("=" * 50)
    
    # Run Smart Tree in claude mode for maximum compression
    print(f"📊 Analyzing {path}...")
    try:
        result = subprocess.run(
            ["st", "-m", "claude", path],
            capture_output=True,
            text=True,
            check=True
        )
        
        analyzer = SemanticAnalyzer()
        analysis = analyzer.analyze_quantum_output(result.stdout)
        
        # Build semantic graph
        analyzer.build_semantic_graph()
        
        # Generate report
        print("\n" + analyzer.generate_semantic_report())
        
        # Find code duplication
        similar = analyzer.find_similar_blocks(threshold=0.9)
        if similar:
            print("\n⚠️  Potential Code Duplication (>90% similar):")
            for block1, block2, sim in similar[:3]:
                print(f"  {block1.path} ≈ {block2.path} ({sim:.0%})")
        
        # Visualize semantic network
        if len(analyzer.code_blocks) >= 2:
            print("\n🎨 Creating semantic visualization...")
            analyzer.visualize_semantic_network()
        
        # Show token efficiency
        print("\n💡 Quantum Tokenization Benefits:")
        print(f"   Traditional: ~{len(analyzer.code_blocks) * 100} tokens")
        print(f"   Quantum: ~{len(analyzer.token_map)} unique tokens")
        print(f"   Compression: {(1 - len(analyzer.token_map) / (len(analyzer.code_blocks) * 100)) * 100:.1f}%")
        
    except subprocess.CalledProcessError:
        print("❌ Error: Smart Tree (st) not found. Please install it first.")
    except Exception as e:
        print(f"❌ Error: {e}")

if __name__ == "__main__":
    main()