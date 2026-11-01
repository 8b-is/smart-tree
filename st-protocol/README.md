# st-protocol

Binary wire protocol for Smart Tree daemon communication.

## Overview

A tight, 6502-inspired binary protocol using control ASCII (0x00-0x1F) as opcodes.
No JSON in the core path. Every byte means something.

## Frame Format

```
┌──────┬─────────────────┬──────┐
│ verb │     payload     │ 0x00 │
│ 1B   │   N bytes       │ END  │
└──────┴─────────────────┴──────┘
```

## Escape Sequences

- `0x1B 0x1B` = literal `0x1B` in payload
- `0x1B 0x00` = literal `0x00` in payload

## Verb Map

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

## Network Addressing

Single byte prefix for routing:
- `0x00` = local daemon (Unix socket)
- `0x01-0x7F` = cached host index
- `0x80-0xFE` = inline address (len = byte - 0x80)
- `0xFF` = broadcast/discover

## Security Levels

- Level 0x00: Read-only (SCAN, SEARCH, STATS) - no auth required
- Level 0x01: Local write (FORMAT, temp files) - session required
- Level 0x02: Mutate (EDIT, DELETE) - requires FIDO
- Level 0x03: Admin (PERMIT, config) - requires FIDO + PIN

## Usage

```rust
use st_protocol::{Frame, Verb, Payload};

// Create a PING frame (2 bytes)
let ping = Frame::ping();
let bytes = ping.encode(); // [0x05, 0x00]

// Create a SCAN frame
let scan = Frame::scan("/home/hue", 3);
let bytes = scan.encode();

// Decode a frame
let frame = Frame::decode(&bytes)?;
println!("{} {}", frame.verb().name(), frame.payload().as_str().unwrap_or(""));
```

## License

MIT
