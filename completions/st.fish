complete -c st -l completions -d 'Generate shell completion scripts' -r -f -a "bash\t''
elvish\t''
fish\t''
powershell\t''
zsh\t''"
complete -c st -l rename-project -d 'Rename project - elegant identity transition (format: "OldName" "NewName")' -r
complete -c st -l input -d 'Specify input type explicitly (filesystem, qcp, sse, openapi, mem8)' -r
complete -c st -s m -l mode -d 'Choose your adventure! Selects the output format. From classic human-readable to AI-optimized hex, we\'ve got options' -r -f -a "auto\t'Auto mode - smart default selection based on context'
classic\t'Classic tree format, human-readable with metadata and emojis (unless disabled). Our beloved default'
hex\t'Hexadecimal format with fixed-width fields. Excellent for AI parsing or detailed analysis'
json\t'JSON output. Structured data for easy programmatic use'
ls\t'Unix ls -Alh format. Familiar directory listing with human-readable sizes and permissions'
ai\t'AI-optimized format. A special blend of hex tree and statistics, designed for LLMs'
stats\t'Directory statistics only. Get a summary without the full tree'
csv\t'CSV (Comma Separated Values) format. Spreadsheet-friendly'
tsv\t'TSV (Tab Separated Values) format. Another spreadsheet favorite'
digest\t'Super compact digest format. A single line with a hash and minimal stats, perfect for quick AI pre-checks'
quantum\t'MEM|8 Quantum format. The ultimate compression with bitfield headers and tokenization'
semantic\t'Semantic grouping format. Groups files by conceptual similarity (inspired by Omni!)'
mermaid\t'Mermaid diagram format. Perfect for embedding in documentation!'
markdown\t'Markdown report format. Combines mermaid, tables, and charts for beautiful documentation!'
summary\t'Interactive summary mode (default for humans in terminal)'
summary-ai\t'AI-optimized summary mode (default for AI/piped output)'
relations\t'Code relationship analysis'
quantum-semantic\t'Quantum compression with semantic understanding (Omni\'s nuclear option!)'
waste\t'Waste detection and optimization analysis (Marie Kondo mode!)'
marqant\t'Marqant - Quantum-compressed markdown format (.mq)'
sse\t'SSE - Server-Sent Events streaming format for real-time monitoring'
function-markdown\t'Function documentation in markdown format - living blueprints of your code!'"
complete -c st -l find -d 'Feeling like a detective? Find files/directories matching this regex pattern. Example: --find "README\\\\.md"' -r
complete -c st -l type -d 'Filter by file extension. Show only files of this type (e.g., "rs", "txt"). No leading dot needed, just the extension itself' -r
complete -c st -l entry-type -d 'Filter to show only files (f) or directories (d)' -r -f -a "f\t''
d\t''"
complete -c st -l min-size -d 'Only show files larger than this size. Accepts human-readable sizes like "1M" (1 Megabyte), "500K" (500 Kilobytes), "100B" (100 Bytes)' -r
complete -c st -l max-size -d 'Only show files smaller than this size. Same format as --min-size. Let\'s find those tiny files!' -r
complete -c st -l newer-than -d 'Time traveler? Show files newer than this date (YYYY-MM-DD format)' -r
complete -c st -l older-than -d 'Or perhaps you prefer antiques? Show files older than this date (YYYY-MM-DD format)' -r
complete -c st -s d -l depth -d 'How deep should we dig? Limits the traversal depth. Default is 0 (auto) which lets each mode pick its ideal depth. Set explicitly to override: 1 for shallow, 10 for deep exploration' -r
complete -c st -l path-mode -d 'Controls how file paths are displayed in the output' -r -f -a "off\t'Show only filenames (default). Clean and simple'
relative\t'Show paths relative to the scan root. Good for context within the project'
full\t'Show full absolute paths. Leaves no doubt where things are'"
complete -c st -l color -d 'When should we splash some color on the output? `auto` (default) uses colors if outputting to a terminal' -r -f -a "always\t'Always use colors, no matter what. Go vibrant!'
never\t'Never use colors. For the minimalists'
auto\t'Use colors if the output is a terminal (tty), otherwise disable. This is the default smart behavior'"
complete -c st -l sse-port -d 'Port for SSE server mode (default: 8420)' -r
complete -c st -l search -d 'Search for a keyword within file contents. Best used with `--type` to limit search to specific file types (e.g., `--type rs --search "TODO"`). This is like having X-ray vision for your files!' -r
complete -c st -l mermaid-style -d 'Mermaid diagram style (only used with --mode mermaid). Options: flowchart (default), mindmap, gitgraph' -r -f -a "flowchart\t'Traditional flowchart (default)'
mindmap\t'Mind map style'
gitgraph\t'Git graph style'
treemap\t'Treemap style (shows file sizes visually)'"
complete -c st -l focus -d 'Focus analysis on specific file (for relations mode). Shows all relationships for a particular file' -r -F
complete -c st -l relations-filter -d 'Filter relationships by type (for relations mode). Options: imports, calls, types, tests, coupled' -r
complete -c st -l sort -d 'Sort results by: a-to-z, z-to-a, largest, smallest, newest, oldest, type Examples: --sort largest (biggest files first), --sort newest (recent files first) Use with --top to get "top 10 largest files" or "20 newest files"' -r -f -a "a-to-z\t'Sort alphabetically A to Z'
z-to-a\t'Sort alphabetically Z to A'
largest\t'Sort by size, largest files first'
smallest\t'Sort by size, smallest files first'
newest\t'Sort by modification date, newest first'
oldest\t'Sort by modification date, oldest first'
type\t'Sort by file type/extension'
name\t'Legacy aliases for backward compatibility'
size\t''
date\t''"
complete -c st -l top -d 'Show only the top N results (useful with --sort) Examples: --sort size --top 10 (10 largest files) --sort date --top 20 (20 most recent files)' -r
complete -c st -l cleanup-diffs -d 'Clean up old diffs in .st folder, keeping only last N per file Example: --cleanup-diffs 5 (keep last 5 diffs per file)' -r
complete -c st -l cheet -d 'Show the cheatsheet'
complete -c st -l man -d 'Generate the man page'
complete -c st -l mcp -d 'Run `st` as an MCP (Model Context Protocol) server'
complete -c st -l mcp-tools -d 'List the tools `st` provides when running as an MCP server'
complete -c st -l mcp-config -d 'Show the configuration snippet for the MCP server'
complete -c st -s V -l version -d 'Show version information and check for updates'
complete -c st -l terminal -d 'Launch Smart Tree Terminal Interface (STTI) - Your coding companion! This starts an interactive terminal that anticipates your needs'
complete -c st -l no-ignore -d 'Daredevil mode: Ignores `.gitignore` files. See everything, even what Git tries to hide!'
complete -c st -l no-default-ignore -d 'Double daredevil: Ignores our built-in default ignore patterns too (like `node_modules`, `__pycache__`). Use with caution, or you might see more than you bargained for!'
complete -c st -s a -l all -d 'Show all files, including hidden ones (those starting with a `.`). The `-a` is for "all", naturally'
complete -c st -l show-ignored -d 'Want to see what\'s being ignored? This flag shows ignored directories in brackets `[dirname]`. Useful for debugging your ignore patterns or just satisfying curiosity'
complete -c st -l everything -d 'SHOW ME EVERYTHING! The nuclear option that combines --all, --no-ignore, and --no-default-ignore. This reveals absolutely everything: hidden files, git directories, node_modules, the works! Warning: May produce overwhelming output in large codebases'
complete -c st -l show-filesystems -d 'Show filesystem type indicators in output (e.g., X=XFS, 4=ext4, B=Btrfs). Each file/directory gets a single character showing what filesystem it\'s on. Great for understanding storage layout and mount points!'
complete -c st -l no-emoji -d 'Not a fan of emojis? This flag disables them for a plain text experience. (But Trish loves the emojis, just saying!) 🌳✨'
complete -c st -s z -l compress -d 'Compress the output using zlib. Great for sending large tree structures over the wire or for AI models that appreciate smaller inputs. Output will be base64 encoded'
complete -c st -l mcp-optimize -d 'MCP/API optimization mode. Automatically enables compression, disables colors/emoji, and optimizes output for machine consumption. Perfect for MCP servers, LLM APIs, and tools. Works with any output mode to make it API-friendly!'
complete -c st -l compact -d 'For JSON output, this makes it compact (one line) instead of pretty-printed. Saves space, but might make Trish\'s eyes water if she tries to read it directly'
complete -c st -l ai-json -d 'For AI mode, wraps the output in a JSON structure. Makes it easier for programmatic consumption by our AI overlords (just kidding... mostly)'
complete -c st -l stream -d 'Stream output as files are scanned. This is a game-changer for very large directories! You\'ll see results trickling in, rather than waiting for the whole scan to finish. Note: Compression is disabled in stream mode for now'
complete -c st -l sse-server -d 'Start SSE server mode for real-time directory monitoring (experimental). This starts an HTTP server that streams directory changes as Server-Sent Events. Example: st --sse-server --sse-port 8420 /path/to/watch'
complete -c st -l semantic -d 'Group files by semantic similarity (inspired by Omni\'s wisdom!). Uses content-aware tokenization to identify conceptually related files. Perfect for understanding project structure at a higher level. Example groups: "tests", "documentation", "configuration", "source code"'
complete -c st -l no-markdown-mermaid -d 'Exclude mermaid diagrams from markdown report (only used with --mode markdown)'
complete -c st -l no-markdown-tables -d 'Exclude tables from markdown report (only used with --mode markdown)'
complete -c st -l no-markdown-pie-charts -d 'Exclude pie charts from markdown report (only used with --mode markdown)'
complete -c st -l show-private -d 'Include private functions in function documentation (for function-markdown mode) By default, only public functions are shown'
complete -c st -l view-diffs -d 'View diffs stored in the .st folder (Smart Edit history) Shows all diffs for files modified by Smart Edit operations'
complete -c st -s h -l help -d 'Print help (see more with \'--help\')'
