# Foken Network Integration

**Status**: 🚧 Under Development

Smart Tree now includes integration with the [Foken GPU Sharing Network](https://st.foken.ai), enabling users to earn Fokens by sharing idle GPU resources while maintaining enterprise-grade security.

## Overview

The Foken integration transforms Smart Tree into an intelligent system daemon that:

1. **Monitors system security** - Tracks browser sessions, extensions, and network activity
2. **Shares idle resources** - Contributes GPU/CPU when you're not using them
3. **Earns Fokens** - Get compensated for compute contributions
4. **Maintains trust** - Multi-layer cryptographic verification ensures safety

## Architecture

```
┌─────────────────────────────────────┐
│      Foken Network (Trust Root)     │
│  - Compiles unique auditors         │
│  - Manages node reputation          │
│  - Deploys honeypots                │
└──────────────┬──────────────────────┘
               │
     ┌─────────▼─────────┐
     │ Auditor (Per-Job) │
     │ - Read-only       │
     │ - Time-limited    │
     │ - Cryptosigned    │
     └─────────┬─────────┘
               │ Validates
     ┌─────────▼─────────┐
     │  Smart Tree Node  │
     │ - SHA256 verified │
     │ - Open source     │
     └─────────┬─────────┘
               │ Executes
     ┌─────────▼─────────┐
     │ Secure Sandbox    │
     │ - Encrypted       │
     │ - CRC monitored   │
     │ - Network locked  │
     └───────────────────┘
```

## Security Model

### Stage 1: Public Data Sharing

Start with **zero-risk jobs** where data exposure has no consequence:

- "How to calculate SUM in Excel"
- "Convert PNG to JPG"
- Public dataset processing

**Goal**: Build trust incrementally without exposing sensitive data.

### Stage 2: Verified Execution

Each job runs in a **secure sandbox** with:

- ✅ Encrypted container images
- ✅ CRC verification (before/after execution)
- ✅ Network activity monitoring (only Foken orchestrator allowed)
- ✅ Disk operation logging
- ✅ Honeypot canaries (fake data to catch malicious nodes)
- ✅ Secure wipe after completion

**Honeypot System**: Random jobs contain irresistible fake data (Bitcoin wallets, AWS keys, database credentials). These canaries are monitored 24/7 across dark web, Tor, paste sites. If leaked, the exact node is identified and reputation destroyed.

### Stage 3: Cryptographic Auditing

The **killer feature** - a uniquely compiled auditor for each job:

```rust
// Compiled on Foken network for each job
pub struct CompiledAuditor {
    job_id: String,        // Unique job identifier
    binary: Vec<u8>,       // Uniquely compiled for this job
    signature: Signature,  // Cryptographically signed
    valid_until: Time,     // Time-limited (1 hour)
}
```

**Auditor capabilities** (audit-only rights):
- ✅ Read Smart Tree binary and verify SHA256 hash
- ✅ Read configuration and detect tampering
- ✅ Monitor execution behavior in real-time
- ✅ Log anomalies and suspicious activity
- ❌ **Cannot write** (audit-only!)
- ❌ **Cannot execute** arbitrary code

**Why this works**:
1. **Unique per job** - Can't be pre-analyzed or reverse engineered in advance
2. **Signed by network** - Tampering invalidates signature
3. **Time-limited** - Expires after 1 hour
4. **Validates Smart Tree** - Checks binary hash against known releases
5. **Catches tampering** - If node modifies Smart Tree, auditor detects it

### Stage 4: Data Obfuscation (Future)

Advanced privacy techniques:
- Homomorphic encryption (compute on encrypted data)
- Differential privacy (add statistical noise)
- Zero-knowledge proofs (prove without revealing)
- Secret sharing (split across multiple nodes)

## Hardware Requirements

Smart Tree only asks users with **worthwhile hardware** to share:

| Hardware | Worth Sharing? | Est. Daily Fokens |
|----------|----------------|-------------------|
| RTX 4090 (24GB) | ✅ Yes | 100+ |
| RTX 3080 (12GB) | ✅ Yes | 50-80 |
| RTX 3060 (8GB) | ✅ Yes | 25-40 |
| Apple M1+ (NPU) | ✅ Yes | 30-50 |
| 32GB+ RAM (CPU) | ✅ Yes | 10-20 |
| 8GB RAM, no GPU | ❌ No | Not recommended |

Detection is automatic - Smart Tree profiles your system and only prompts if sharing makes sense.

## User Permissions

Smart Tree requests explicit user consent for each capability:

```
Smart Tree would like to:

 ☑️ Monitor file system activity (for security alerts)
 ☑️ Track installed browser extensions
 ☐ Share your GPU when idle (earn Fokens)
 ☐ Auto-quarantine suspicious software
 ☐ Monitor network connections for threats

 Your hardware: RTX 4090, 64GB RAM
 Estimated earnings: ~100 Fokens/day (8 hours idle)

 [Enable Selected] [Learn More]
```

**Security monitoring example**:

```
⚠️  SECURITY ALERT

Detected: Chrome extension "SuperVPN" contacted Nigerian IP
          30 minutes after Bank of America session ended

Recommendation: Quarantine extension

[Quarantine] [Ignore] [Learn More]
```

Users can choose their protection level:
- **Level 1**: Alerts only (no auto-action)
- **Level 2**: Quarantine suspicious software
- **Level 3**: Full protection (auto-quarantine + network blocking)

## How Malicious Nodes are Caught

Even if a criminal **recompiles Smart Tree** to bypass security:

1. **Auditor checks binary hash** - Detects modified Smart Tree
2. **Job rejected immediately** - No execution
3. **Reputation destroyed** - Node blacklisted
4. **Honeypots catch data theft** - Canaries trace leaks
5. **Legal action possible** - Provable breach with cryptographic proof

## Usage

### As a CLI Tool

```bash
# Check if your hardware is worth sharing
st --foken-check

# Start daemon with Foken enabled
st --daemon --foken-enable

# Check earnings
st --foken-earnings

# View security threats
st --foken-threats
```

### As a System Service

```bash
# Install as systemd service (Linux)
sudo st --install-service --foken-enable

# Start service
sudo systemctl start smart-tree

# Check status
sudo systemctl status smart-tree
```

### As a Library

```rust
use st::foken::{FokenDaemon, ServiceConfig, SharingPreferences};

#[tokio::main]
async fn main() -> Result<()> {
    let config = ServiceConfig {
        sharing: SharingPreferences {
            level: SharingLevel::Moderate,
            idle_threshold_minutes: 15,
            ..Default::default()
        },
        ..Default::default()
    };

    let mut daemon = FokenDaemon::new(config)?;
    daemon.start().await?;

    Ok(())
}
```

## Transparency

**Smart Tree is open source.** The Foken integration code is in `src/foken/`:

- `auditor.rs` - Auditor compilation and runtime
- `honeypot.rs` - Canary system for leak detection
- `sandbox.rs` - Secure execution environment
- `daemon.rs` - System service layer
- `types.rs` - Common types

You can **audit every line of code**. Nothing hidden.

## FAQ

### Q: Can Foken see my private files?

**A:** No. Jobs run in isolated sandboxes with no file system access. The auditor has read-only rights to verify Smart Tree's integrity, but cannot access your files.

### Q: What if my internet goes out during a job?

**A:** The job fails gracefully. Sandbox is cleaned up, no data leaves your system.

### Q: Can I share only specific hours?

**A:** Yes. Configure idle threshold and schedule in preferences.

### Q: How are Fokens converted to real money?

**A:** Through the Foken marketplace. Exchange rate varies based on demand.

### Q: What if I don't trust this?

**A:** Don't enable it! Foken is **completely optional**. Smart Tree works great without it.

## Roadmap

- [ ] Basic daemon implementation
- [ ] Auditor compilation system
- [ ] Honeypot deployment and monitoring
- [ ] Sandbox with CRC verification
- [ ] Security monitoring (browser, extensions, network)
- [ ] Foken network integration
- [ ] Auto-quarantine system
- [ ] Homomorphic encryption support
- [ ] Zero-knowledge proofs
- [ ] Multi-node secret sharing

## Learn More

- Website: [st.foken.ai](https://st.foken.ai)
- Network: [foken.ai](https://foken.ai)
- Docs: [docs.foken.ai](https://docs.foken.ai)
- GitHub: [github.com/8b-is/smart-tree](https://github.com/8b-is/smart-tree)

## License

The Foken integration follows Smart Tree's license (see LICENSE file).

---

**Built with love by Hue & Claude** 🎮💰🔒
