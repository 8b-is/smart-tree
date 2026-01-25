# ST Daemon Vision

**Date**: 2026-01-25
**Status**: Design Phase
**Authors**: Hue, Claude

## Overview

Split smart-tree into a client/daemon architecture with a custom binary protocol. The daemon (`std`) runs persistently, providing context, security, and API services. The client (`st`) becomes a thin binary that communicates with local or remote daemons.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Your Fleet                             │
├────────────────┬────────────────┬────────────────────────────┤
│  workstation   │    server1     │       server2              │
│                │                │                            │
│  std (daemon)  │   std (daemon) │      std (daemon)          │
│  ├─ MCP API    │   ├─ MCP API   │      ├─ MCP API            │
│  ├─ M8 Memory  │   ├─ M8 Memory │      ├─ M8 Memory          │
│  ├─ Security   │   ├─ Security  │      ├─ Security           │
│  └─ Dashboard  │   └─ (headless)│      └─ (headless)         │
└───────▲────────┴───────▲────────┴───────────▲────────────────┘
        │                │                    │
        └────────────────┼────────────────────┘
                         │
                   ┌─────┴─────┐
                   │    st     │  ← universal client
                   │   (cli)   │
                   └───────────┘
```

## Binary Protocol (6502 Edition)

Native protocol uses control ASCII (0x00-0x1F) as opcodes. No JSON in the core path.

### Frame Format

```
┌──────┬─────────────────┬──────┐
│ verb │     payload     │ 0x00 │
│ 1B   │   N bytes       │ END  │
└──────┴─────────────────┴──────┘

Escape sequences:
  0x1B 0x1B = literal 0x1B in payload
  0x1B 0x00 = literal 0x00 in payload
```

### Verb Map (Control ASCII)

```
0x01 SOH  SCAN         0x11 DC1  PERMIT
0x02 STX  FORMAT       0x12 DC2  DENY
0x03 ETX  SEARCH       0x13 DC3  ELEVATE
0x04 EOT  END_STREAM   0x14 DC4  AUDIT
0x05 ENQ  PING         0x15 NAK  ERROR
0x06 ACK  OK           0x16 SYN  SUBSCRIBE
0x07 BEL  ALERT        0x17 ETB  UNSUBSCRIBE
0x08 BS   BACK/UNDO    0x18 CAN  CANCEL
0x09 HT   CONTEXT      0x19 EM   M8_WAVE
0x0A LF   NEXT         0x1A SUB  REMEMBER
0x0B VT   STATS        0x1B ESC  ESCAPE
0x0C FF   CLEAR        0x1C FS   RECALL
0x0D CR   COMPLETE     0x1D GS   FORGET
0x0E SO   AUTH_START   0x1E RS   SESSION
0x0F SI   AUTH_END     0x1F US   USER
```

### Payload Encoding

```
First byte after verb:
  0x20-0x7E  = ASCII string starts (printable)
  0x80-0xFE  = Length prefix (len = byte - 0x80, max 126)
  0xFF       = Extended length (next 2 bytes = u16 LE)
```

### Network Addressing

Single byte prefix for routing:

```
0x00        = local daemon (Unix socket /run/st.sock)
0x01-0x7F   = cached host index (up to 127 known hosts)
0x80-0xFE   = inline address follows (len = byte - 0x80)
0xFF        = broadcast/discover
```

### Examples

```
SCAN /home/hue depth=3:
  01                      ; SCAN
  8A                      ; length 10
  2F 68 6F 6D 65 2F 68 75 65  ; /home/hue
  03                      ; depth
  00                      ; END
  = 13 bytes total

PING:
  05 00                   ; ENQ + END = 2 bytes

Remote SCAN:
  03                      ; host[3] from cache
  01                      ; SCAN
  ...payload...
  00                      ; END
```

## Security Model

### Auth Block (inline)

```
Protected operation:
  0E                      ; SO = AUTH_START
  [level: 1B]             ; 0x01=pin, 0x02=fido, 0x03=bio
  [session: 16B]          ; UUID
  [sig: 32B]              ; Ed25519
  0F                      ; SI = AUTH_END
  01                      ; SCAN (actual verb)
  ...payload...
  00                      ; END
```

### Security Levels

```
Level 0x00: Read-only (SCAN, SEARCH, STATS) - no auth required
Level 0x01: Local write (FORMAT output, temp files) - session required
Level 0x02: Mutate (EDIT, DELETE) - requires FIDO
Level 0x03: Admin (PERMIT, config changes) - requires FIDO + PIN
```

### Protected Paths

The daemon intercepts writes to sensitive paths:

- `~/.claude/settings.json` - Claude Code config
- `~/.config/*/` - AI tool configs
- System paths

Elevation request triggers FIDO/PIN/biometric based on configured level.

## Protocol Adapters

The core uses binary protocol. Adapters translate at boundaries:

```
┌───────────────┬───────────────┬───────────────┬────────────┐
│  st-protocol  │  MCP Adapter  │ OpenAI Compat │  Raw HTTP  │
│  (native)     │  (JSON-RPC)   │  (/v1/chat)   │  (REST)    │
└───────┬───────┴───────┬───────┴───────┬───────┴─────┬──────┘
        │               │               │             │
        └───────────────┴───────┬───────┴─────────────┘
                                │
                    ┌───────────▼───────────┐
                    │   std core (engine)   │
                    │   Binary protocol     │
                    └───────────────────────┘
```

## Daemon Features

### Context Gathering
- Watches filesystem for changes
- Maintains warm caches
- Provides instant context to AI tools

### M8 Memory
- Persistent memory across sessions
- Wave signatures for temporal context
- Searchable by keywords/tags

### MCP Server
- 30+ tools for AI assistants
- Smart edit with AST awareness
- Git integration

### Web Dashboard
- PTY terminal in browser
- File browser
- Real-time updates via WebSocket

## CLI Usage

```bash
# Local operations
st .                      # scan local via daemon
st --mode quantum .       # quantum compression

# Remote operations
st @server1 .             # remote daemon (cached)
st @192.168.1.5:8420 .    # explicit address
st @* --ping              # discover all daemons

# Daemon control
std start                 # start daemon
std stop                  # stop daemon
std status                # health check
```

## Integration with i1.is

The universal installer bootstraps the ecosystem:

```bash
curl i1.is | bash         # installs i1
i1 st                     # installs st + std
i1 foken                  # installs foken
i1 m8                     # installs m8 tools
```

## File Locations

```
~/.st/
├── hosts                 # cached remote hosts (binary)
├── keys/
│   ├── id_ed25519        # client keypair
│   └── id_ed25519.pub
├── cache/                # response cache
└── config.toml           # client config

/run/st.sock              # local daemon socket (Linux)
~/Library/Application Support/st/st.sock  # macOS

~/.mem8/                  # M8 memory storage
```

## Implementation Phases

### Phase 1: Protocol Crate ✅ COMPLETE
- [x] `st-protocol` crate with binary format
- [x] Encode/decode for all verbs (27 verbs mapped to control ASCII)
- [x] Escape sequence handling (0x1B 0x1B / 0x1B 0x00)
- [x] Tests with known vectors (27 tests passing)
- [x] Network addressing (local/cached/inline/broadcast)
- [x] Auth blocks with security levels (None/Session/FIDO/FIDO+PIN)
- [x] Path protection API

### Phase 2: Daemon Core ✅ COMPLETE
- [x] Unix socket listener (`/run/user/$UID/st.sock`)
- [x] Basic verb handling (PING, SCAN, STATS, SESSION)
- [x] Security context per connection
- [x] Live integration tests (6 tests: PING, SCAN, FORMAT×2, SEARCH, MEMORY)
- [x] FORMAT verb with 7 output modes (classic, ai, json, hex, quantum, stats, digest)
- [x] SEARCH verb integration (content search with line numbers)
- [x] M8 memory verbs (REMEMBER, RECALL, FORGET, M8_WAVE) with compact grid
- [x] Spatial audio processing (wave grid as 256×256 spatial room with stereo sampling)
- [x] WaveGrid corrected to 2D (256×256 cells, Z as value not dimension)

### Phase 3: Client Refactor
- [ ] Thin st client
- [ ] Daemon auto-start
- [ ] Local/remote routing
- [ ] Host cache

### Phase 4: Security Layer
- [ ] Auth block parsing
- [ ] FIDO2 integration
- [ ] Path protection
- [ ] Elevation prompts

### Phase 5: Multi-Daemon
- [ ] mDNS discovery
- [ ] Trust-on-first-use
- [ ] Remote session establishment
- [ ] Cross-daemon memory sync

## Binary Names

- `st` - Client (thin, fast)
- `std` - Daemon (ST Daemon, also "standard" vibes)
- `i1` - Universal installer
- `m8` - Memory tools
- `mq` - Marqant compressor

## Related Documents

- `SMART_COMPRESSION.md` - Compression formats
- `MCP_UPDATE_ANALYTICS.md` - MCP tool usage
- `docs/AI_ASSISTANT_GUIDE.md` - AI integration

---

*"Every byte means something. No ceremony."*
