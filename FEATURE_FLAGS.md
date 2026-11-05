# Smart Tree Feature Flags System

## Overview

Smart Tree now includes a comprehensive feature flags system that allows organizations to control and disable features based on their compliance and security requirements. This ensures Smart Tree can be used in enterprise, government, healthcare, and other regulated environments.

## Configuration Methods

Feature flags can be configured through multiple methods (in priority order):

1. **Environment Variables** (highest priority)
2. **Local config file** (`.st/features.toml`)
3. **User config file** (`~/.st/features.toml`)
4. **System config file** (`/etc/smart-tree/features.toml`)
5. **Default values** (lowest priority)

## Environment Variables

Quick control via environment variables:

- `ST_DISABLE_MCP` - Disable MCP server functionality
- `ST_DISABLE_AI` - Disable all AI-related features
- `ST_DISABLE_LOGGING` - Disable activity logging
- `ST_DISABLE_WATCHING` - Disable file watching and context absorption
- `ST_PRIVACY_MODE` - Enable privacy mode (disables telemetry, logging, external connections)
- `ST_COMPLIANCE_MODE` - Set compliance mode (enterprise, government, healthcare, education, financial)

## Compliance Modes

### Government Mode
Maximum restrictions for government use:
- Disables consciousness, memory manager, context absorption
- Disables activity logging and telemetry
- Disables file watching and auto-context
- Disables hooks and external connections
- Restricts home directory access
- Enables privacy mode

### Enterprise Mode
Moderate restrictions for corporate environments:
- Disables consciousness and memory manager
- Disables activity logging and telemetry
- Enables privacy mode
- Disables unified watcher and hooks management

### Healthcare Mode (HIPAA)
- Disables activity logging and telemetry
- Disables context absorption
- Enables privacy mode
- Restricts home directory access

### Education Mode (FERPA)
- Disables activity logging and telemetry
- Enables privacy mode

### Financial Mode (SOC2/PCI)
- Enables activity logging (required for audit)
- Disables telemetry
- Enables privacy mode
- Disables context absorption

## Configuration File Format

Create a `features.toml` file in any of the supported locations:

```toml
# Core features
enable_mcp_server = true
enable_classic_tree = true
enable_formatters = true

# AI/ML features
enable_ai_modes = false
enable_consciousness = false
enable_memory_manager = false
enable_context_absorption = false
enable_smart_search = true

# Data collection
enable_activity_logging = false
enable_telemetry = false
enable_file_watching = true
enable_auto_context = true

# Interactive features
enable_tui = true
enable_hooks = true
enable_tips = true

# Advanced features
enable_quantum_modes = true
enable_wave_signatures = true
enable_mega_sessions = true
enable_q8_caster = true

# Privacy settings
privacy_mode = true
disable_external_connections = false
disable_home_directory_access = false

# Compliance mode
compliance_mode = "enterprise"  # Options: none, enterprise, government, healthcare, education, financial

# Path restrictions
allowed_paths = ["/home/user/projects"]
blocked_paths = ["/etc", "/sys", "/proc"]

# MCP tool controls
[mcp_tools]
enable_find = true
enable_search = true
enable_analyze = true
enable_edit = false
enable_context = true
enable_memory = false
enable_unified_watcher = false
enable_hooks_management = false
enable_sse = true
```

## Testing Feature Flags

Run the included test script to verify feature flags are working:

```bash
./test_feature_flags.sh
```

## Integration in Code

The feature flags are checked throughout the codebase:

1. **CLI Arguments** - Features are disabled with appropriate error messages
2. **MCP Tools** - Tools are filtered based on enabled flags
3. **Context Absorption** - Respects file watching and absorption flags
4. **Activity Logging** - Can be completely disabled for privacy

## Examples

### Disable MCP Server
```bash
export ST_DISABLE_MCP=1
st --mcp  # Will show error: MCP server is disabled
```

### Enable Privacy Mode
```bash
export ST_PRIVACY_MODE=1
st --log  # Will show warning: Activity logging is disabled
```

### Set Government Compliance
```bash
export ST_COMPLIANCE_MODE=government
st --hooks-config list  # Will show error: Hooks are disabled
```

### Custom Configuration File
```bash
# Create local project config
cat > .st/features.toml << EOF
enable_ai_modes = false
privacy_mode = true
compliance_mode = "enterprise"
EOF

# Smart Tree will automatically load and apply these settings
st --mcp
```

## For Enterprises

Organizations like Anthropic or other enterprises can:

1. Deploy a system-wide `/etc/smart-tree/features.toml` to enforce policies
2. Set environment variables in corporate shells
3. Use compliance modes to meet regulatory requirements
4. Disable specific features that conflict with corporate policies
5. Control which MCP tools are available to users

This ensures Smart Tree remains a powerful tool while respecting organizational requirements for security, privacy, and compliance.

## Legal Compliance

The feature flags system ensures Smart Tree can be used in environments with strict legal and regulatory requirements:

- **Data Protection** - Disable logging and telemetry for GDPR compliance
- **Security** - Disable external connections and restrict paths
- **Audit Trail** - Enable logging only when required for compliance
- **Tool Control** - Granularly control which MCP tools are available

"Your tool, your rules!" - Hue