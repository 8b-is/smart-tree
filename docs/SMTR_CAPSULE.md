# SMTR v2 capsule

`st --no-daemon --mode louds DIR` emits a JSON report. Its `smtr_m8_capsule_64b_hex` field contains exactly 128 hexadecimal characters, representing 64 bytes. The in-memory `LoudsSmartTree` is 64-byte aligned and contains four topology words and separate arrays for up to 64 nodes. The [serializer and verifier](../src/mem8_binary.rs) define the wire layout.

| Byte range (start inclusive, end exclusive) | Size | Contents |
| --- | ---: | --- |
| `[0, 4)` | 4 | ASCII `SMTR` |
| `[4, 5)` | 1 | Version 2 |
| `[5, 6)` | 1 | Projected node count, 0 through 64 |
| `[6, 8)` | 2 | Low 16 bits of IEEE CRC32 over all four little-endian topology words, stored little-endian |
| `[8, 16)` | 8 | First topology word, little-endian |
| `[16, 48)` | 32 | Projected root subtree digest |
| `[48, 64)` | 16 | First 16 bytes of the domain-prefixed keyed SHA-256 calculation over bytes 0 through 47 |

The three remaining topology words are absent from the wire. The checksum fragment covers them but cannot reconstruct them. The [CLI formatter](../src/formatters/digest.rs) also exposes the root digest and first topology word in separate JSON fields. Its key is a public constant. The verifier checks magic, version and tag. It does not independently reconstruct the tree, verify repository contents, or establish who sent the capsule.

## Check an emitted capsule

Run this with Python 3 and `st` on your PATH. It cross-checks the wire fields against the JSON report and computes the tag independently of the Rust verifier. Run in a directory you want to scan.

```python
import hashlib
import json
import subprocess

report = json.loads(subprocess.check_output(["st", "--no-daemon", "--mode", "louds", "."]))
wire = bytes.fromhex(report["smtr_m8_capsule_64b_hex"])
assert len(wire) == 64 and wire[:5] == b"SMTR\x02"
assert wire[5] == report["louds_sufficient_nodes"] <= report["louds_raw_nodes"] <= 64
assert int.from_bytes(wire[8:16], "little") == int(report["louds_bits_w0_hex"], 16)
assert wire[16:48].hex() == report["indra_net_root_hex"]
key = b"8b-is::smart-tree::smtr-key::v21"
domain = b"|8b-is::louds-smart-tree::v2.1|"
assert hashlib.sha256(key + domain + wire[:48]).digest()[:16] == wire[48:]
```

## What changes the capsule?

`from_file_nodes` takes the first 64 scanned nodes. It encodes the basename, kind, parent relation, and a signature derived from size and permissions. The formatter calls `project_sufficient(false)` before packing. Entries classified as ignored noise or tests, and descendants of removed parents, drop out of that projection. Compare `louds_raw_nodes`, `louds_sufficient_nodes`, and scan totals when judging coverage.

Try replacing `cat` with `dog` in a scanned file while preserving its path, size, and permissions. The source bytes change but the capsule can stay identical. A consumer needing a content identity should hash the original bytes.
