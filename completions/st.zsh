#compdef st

autoload -U is-at-least

_st() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" : \
'--completions=[Generate shell completion scripts]:SHELL:(bash elvish fish powershell zsh)' \
'*--rename-project=[Rename project - elegant identity transition (format\: "OldName" "NewName")]:OLD:_default:OLD:_default' \
'--input=[Specify input type explicitly (filesystem, qcp, sse, openapi, mem8)]:TYPE:_default' \
'-m+[Choose your adventure! Selects the output format. From classic human-readable to AI-optimized hex, we'\''ve got options]:MODE:((auto\:"Auto mode - smart default selection based on context"
classic\:"Classic tree format, human-readable with metadata and emojis (unless disabled). Our beloved default"
hex\:"Hexadecimal format with fixed-width fields. Excellent for AI parsing or detailed analysis"
json\:"JSON output. Structured data for easy programmatic use"
ls\:"Unix ls -Alh format. Familiar directory listing with human-readable sizes and permissions"
ai\:"AI-optimized format. A special blend of hex tree and statistics, designed for LLMs"
stats\:"Directory statistics only. Get a summary without the full tree"
csv\:"CSV (Comma Separated Values) format. Spreadsheet-friendly"
tsv\:"TSV (Tab Separated Values) format. Another spreadsheet favorite"
digest\:"Super compact digest format. A single line with a hash and minimal stats, perfect for quick AI pre-checks"
quantum\:"MEM|8 Quantum format. The ultimate compression with bitfield headers and tokenization"
semantic\:"Semantic grouping format. Groups files by conceptual similarity (inspired by Omni!)"
mermaid\:"Mermaid diagram format. Perfect for embedding in documentation!"
markdown\:"Markdown report format. Combines mermaid, tables, and charts for beautiful documentation!"
summary\:"Interactive summary mode (default for humans in terminal)"
summary-ai\:"AI-optimized summary mode (default for AI/piped output)"
relations\:"Code relationship analysis"
quantum-semantic\:"Quantum compression with semantic understanding (Omni'\''s nuclear option!)"
waste\:"Waste detection and optimization analysis (Marie Kondo mode!)"
marqant\:"Marqant - Quantum-compressed markdown format (.mq)"
sse\:"SSE - Server-Sent Events streaming format for real-time monitoring"
function-markdown\:"Function documentation in markdown format - living blueprints of your code!"))' \
'--mode=[Choose your adventure! Selects the output format. From classic human-readable to AI-optimized hex, we'\''ve got options]:MODE:((auto\:"Auto mode - smart default selection based on context"
classic\:"Classic tree format, human-readable with metadata and emojis (unless disabled). Our beloved default"
hex\:"Hexadecimal format with fixed-width fields. Excellent for AI parsing or detailed analysis"
json\:"JSON output. Structured data for easy programmatic use"
ls\:"Unix ls -Alh format. Familiar directory listing with human-readable sizes and permissions"
ai\:"AI-optimized format. A special blend of hex tree and statistics, designed for LLMs"
stats\:"Directory statistics only. Get a summary without the full tree"
csv\:"CSV (Comma Separated Values) format. Spreadsheet-friendly"
tsv\:"TSV (Tab Separated Values) format. Another spreadsheet favorite"
digest\:"Super compact digest format. A single line with a hash and minimal stats, perfect for quick AI pre-checks"
quantum\:"MEM|8 Quantum format. The ultimate compression with bitfield headers and tokenization"
semantic\:"Semantic grouping format. Groups files by conceptual similarity (inspired by Omni!)"
mermaid\:"Mermaid diagram format. Perfect for embedding in documentation!"
markdown\:"Markdown report format. Combines mermaid, tables, and charts for beautiful documentation!"
summary\:"Interactive summary mode (default for humans in terminal)"
summary-ai\:"AI-optimized summary mode (default for AI/piped output)"
relations\:"Code relationship analysis"
quantum-semantic\:"Quantum compression with semantic understanding (Omni'\''s nuclear option!)"
waste\:"Waste detection and optimization analysis (Marie Kondo mode!)"
marqant\:"Marqant - Quantum-compressed markdown format (.mq)"
sse\:"SSE - Server-Sent Events streaming format for real-time monitoring"
function-markdown\:"Function documentation in markdown format - living blueprints of your code!"))' \
'--find=[Feeling like a detective? Find files/directories matching this regex pattern. Example\: --find "README\\\\.md"]:FIND:_default' \
'--type=[Filter by file extension. Show only files of this type (e.g., "rs", "txt"). No leading dot needed, just the extension itself]:FILTER_TYPE:_default' \
'--entry-type=[Filter to show only files (f) or directories (d)]:ENTRY_TYPE:(f d)' \
'--min-size=[Only show files larger than this size. Accepts human-readable sizes like "1M" (1 Megabyte), "500K" (500 Kilobytes), "100B" (100 Bytes)]:MIN_SIZE:_default' \
'--max-size=[Only show files smaller than this size. Same format as --min-size. Let'\''s find those tiny files!]:MAX_SIZE:_default' \
'--newer-than=[Time traveler? Show files newer than this date (YYYY-MM-DD format)]:NEWER_THAN:_default' \
'--older-than=[Or perhaps you prefer antiques? Show files older than this date (YYYY-MM-DD format)]:OLDER_THAN:_default' \
'-d+[How deep should we dig? Limits the traversal depth. Default is 0 (auto) which lets each mode pick its ideal depth. Set explicitly to override\: 1 for shallow, 10 for deep exploration]:DEPTH:_default' \
'--depth=[How deep should we dig? Limits the traversal depth. Default is 0 (auto) which lets each mode pick its ideal depth. Set explicitly to override\: 1 for shallow, 10 for deep exploration]:DEPTH:_default' \
'--path-mode=[Controls how file paths are displayed in the output]:PATH_MODE:((off\:"Show only filenames (default). Clean and simple"
relative\:"Show paths relative to the scan root. Good for context within the project"
full\:"Show full absolute paths. Leaves no doubt where things are"))' \
'--color=[When should we splash some color on the output? \`auto\` (default) uses colors if outputting to a terminal]:COLOR:((always\:"Always use colors, no matter what. Go vibrant!"
never\:"Never use colors. For the minimalists"
auto\:"Use colors if the output is a terminal (tty), otherwise disable. This is the default smart behavior"))' \
'--sse-port=[Port for SSE server mode (default\: 8420)]:SSE_PORT:_default' \
'--search=[Search for a keyword within file contents. Best used with \`--type\` to limit search to specific file types (e.g., \`--type rs --search "TODO"\`). This is like having X-ray vision for your files!]:SEARCH:_default' \
'--mermaid-style=[Mermaid diagram style (only used with --mode mermaid). Options\: flowchart (default), mindmap, gitgraph]:MERMAID_STYLE:((flowchart\:"Traditional flowchart (default)"
mindmap\:"Mind map style"
gitgraph\:"Git graph style"
treemap\:"Treemap style (shows file sizes visually)"))' \
'--focus=[Focus analysis on specific file (for relations mode). Shows all relationships for a particular file]:FILE:_files' \
'--relations-filter=[Filter relationships by type (for relations mode). Options\: imports, calls, types, tests, coupled]:TYPE:_default' \
'--sort=[Sort results by\: a-to-z, z-to-a, largest, smallest, newest, oldest, type Examples\: --sort largest (biggest files first), --sort newest (recent files first) Use with --top to get "top 10 largest files" or "20 newest files"]:SORT:((a-to-z\:"Sort alphabetically A to Z"
z-to-a\:"Sort alphabetically Z to A"
largest\:"Sort by size, largest files first"
smallest\:"Sort by size, smallest files first"
newest\:"Sort by modification date, newest first"
oldest\:"Sort by modification date, oldest first"
type\:"Sort by file type/extension"
name\:"Legacy aliases for backward compatibility"
size\:""
date\:""))' \
'--top=[Show only the top N results (useful with --sort) Examples\: --sort size --top 10 (10 largest files) --sort date --top 20 (20 most recent files)]:N:_default' \
'--cleanup-diffs=[Clean up old diffs in .st folder, keeping only last N per file Example\: --cleanup-diffs 5 (keep last 5 diffs per file)]:N:_default' \
'--cheet[Show the cheatsheet]' \
'--man[Generate the man page]' \
'--mcp[Run \`st\` as an MCP (Model Context Protocol) server]' \
'--mcp-tools[List the tools \`st\` provides when running as an MCP server]' \
'--mcp-config[Show the configuration snippet for the MCP server]' \
'-V[Show version information and check for updates]' \
'--version[Show version information and check for updates]' \
'--terminal[Launch Smart Tree Terminal Interface (STTI) - Your coding companion! This starts an interactive terminal that anticipates your needs]' \
'--no-ignore[Daredevil mode\: Ignores \`.gitignore\` files. See everything, even what Git tries to hide!]' \
'--no-default-ignore[Double daredevil\: Ignores our built-in default ignore patterns too (like \`node_modules\`, \`__pycache__\`). Use with caution, or you might see more than you bargained for!]' \
'-a[Show all files, including hidden ones (those starting with a \`.\`). The \`-a\` is for "all", naturally]' \
'--all[Show all files, including hidden ones (those starting with a \`.\`). The \`-a\` is for "all", naturally]' \
'--show-ignored[Want to see what'\''s being ignored? This flag shows ignored directories in brackets \`\[dirname\]\`. Useful for debugging your ignore patterns or just satisfying curiosity]' \
'--everything[SHOW ME EVERYTHING! The nuclear option that combines --all, --no-ignore, and --no-default-ignore. This reveals absolutely everything\: hidden files, git directories, node_modules, the works! Warning\: May produce overwhelming output in large codebases]' \
'--show-filesystems[Show filesystem type indicators in output (e.g., X=XFS, 4=ext4, B=Btrfs). Each file/directory gets a single character showing what filesystem it'\''s on. Great for understanding storage layout and mount points!]' \
'--no-emoji[Not a fan of emojis? This flag disables them for a plain text experience. (But Trish loves the emojis, just saying!) 🌳✨]' \
'-z[Compress the output using zlib. Great for sending large tree structures over the wire or for AI models that appreciate smaller inputs. Output will be base64 encoded]' \
'--compress[Compress the output using zlib. Great for sending large tree structures over the wire or for AI models that appreciate smaller inputs. Output will be base64 encoded]' \
'--mcp-optimize[MCP/API optimization mode. Automatically enables compression, disables colors/emoji, and optimizes output for machine consumption. Perfect for MCP servers, LLM APIs, and tools. Works with any output mode to make it API-friendly!]' \
'--compact[For JSON output, this makes it compact (one line) instead of pretty-printed. Saves space, but might make Trish'\''s eyes water if she tries to read it directly]' \
'--ai-json[For AI mode, wraps the output in a JSON structure. Makes it easier for programmatic consumption by our AI overlords (just kidding... mostly)]' \
'--stream[Stream output as files are scanned. This is a game-changer for very large directories! You'\''ll see results trickling in, rather than waiting for the whole scan to finish. Note\: Compression is disabled in stream mode for now]' \
'--sse-server[Start SSE server mode for real-time directory monitoring (experimental). This starts an HTTP server that streams directory changes as Server-Sent Events. Example\: st --sse-server --sse-port 8420 /path/to/watch]' \
'--semantic[Group files by semantic similarity (inspired by Omni'\''s wisdom!). Uses content-aware tokenization to identify conceptually related files. Perfect for understanding project structure at a higher level. Example groups\: "tests", "documentation", "configuration", "source code"]' \
'--no-markdown-mermaid[Exclude mermaid diagrams from markdown report (only used with --mode markdown)]' \
'--no-markdown-tables[Exclude tables from markdown report (only used with --mode markdown)]' \
'--no-markdown-pie-charts[Exclude pie charts from markdown report (only used with --mode markdown)]' \
'--show-private[Include private functions in function documentation (for function-markdown mode) By default, only public functions are shown]' \
'--view-diffs[View diffs stored in the .st folder (Smart Edit history) Shows all diffs for files modified by Smart Edit operations]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'::path -- Path to the directory or file you want to analyze. Can also be a URL (http\://), QCP query (qcp\://), SSE stream, or MEM8 stream (mem8\://):_default' \
&& ret=0
}

(( $+functions[_st_commands] )) ||
_st_commands() {
    local commands; commands=()
    _describe -t commands 'st commands' commands "$@"
}

if [ "$funcstack[1]" = "_st" ]; then
    _st "$@"
else
    compdef _st st
fi
