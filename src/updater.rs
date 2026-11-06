// -----------------------------------------------------------------------------
// Self-Update Module for Smart Tree
// Checks the i1 release catalogue and installs verified Smart Tree builds.
// -----------------------------------------------------------------------------

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use ring::signature::{UnparsedPublicKey, ED25519};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::net::UdpSocket;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// The installer and CLI use the same published build catalogue.
const RELEASES_API: &str = "https://i1.is/releases/smart-tree/latest.json";

/// HTTPS fallback for the Ed25519-signed Secure DNS TXT record if UDP/53 is blocked.
const SECURE_DNS_HTTPS_TXT: &str = "https://i1.is/releases/smart-tree/dns-txt";

/// Authoritative Harmony DNS nameservers (`ns3.i1.is`, `ns1.i1.is`, `ns2.i1.is`, `ns4.i1.is`)
const HARMONY_DNS_SERVERS: &[&str] = &[
    "5.78.220.57:53",
    "2.31.35.192:53",
    "5.161.188.178:53",
    "5.223.89.144:53",
];

/// Pinned RFC 8080 Ed25519 (Alg 15, KeyTag 3313) KSK Public Key for `i1.is.` (`BSHQDZCIBiYH4eOU+z/9vC7GQLxJXSsTxjd/WOiYpnQ=`)
pub const I1_HARMONY_ED25519_PUBKEY_HEX: &str =
    "0521D00D9088062607E1E394FB3FFDBC2EC640BC495D2B13C6377F58E898A674";
pub const I1_HARMONY_KEY_TAG: u16 = 3313;

/// Parsed & Ed25519-verified Secure DNS version advertisement (`_smart-tree.i1.is` TXT + DNSSEC `RRSIG`)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecureDnsRelease {
    pub version: String,
    pub build_id: String,
    pub sha256: String,
    pub key_tag: u16,
    pub signature_b64: String,
    /// True when the DNS wire response also carried a valid RFC 4034 / RFC 8080 DNSSEC `RRSIG`
    /// (`rtype=46`, `type_covered=16`, `alg=15` Ed25519, `keytag=3313`) verified against the pinned `i1.is.` KSK.
    #[serde(default)]
    pub dnssec_rrsig_verified: bool,
}

/// Parse a `v=ST1; ver=...; build=...; sha256=...; keytag=3313; alg=Ed25519; sig=...` TXT record
/// and cryptographically verify its Ed25519 signature against `I1_HARMONY_ED25519_PUBKEY_HEX`.
pub fn verify_dns_version_txt(txt: &str) -> Result<SecureDnsRelease> {
    let clean = txt.trim().trim_matches('"');
    let mut v_proto = None;
    let mut version = None;
    let mut build_id = None;
    let mut sha256 = None;
    let mut key_tag = None;
    let mut alg = None;
    let mut sig_b64 = None;

    for part in clean.split(';') {
        let kv = part.trim();
        if let Some((k, val)) = kv.split_once('=') {
            let k = k.trim();
            let val = val.trim();
            match k {
                "v" => v_proto = Some(val.to_string()),
                "ver" => version = Some(val.trim_start_matches('v').to_string()),
                "build" => build_id = Some(val.to_string()),
                "sha256" => sha256 = Some(val.to_lowercase()),
                "keytag" => key_tag = val.parse::<u16>().ok(),
                "alg" => alg = Some(val.to_string()),
                "sig" => sig_b64 = Some(val.to_string()),
                _ => {}
            }
        }
    }

    if v_proto.as_deref() != Some("ST1") {
        bail!("Unsupported Secure DNS TXT version tag");
    }
    if alg.as_deref() != Some("Ed25519") {
        bail!("Unsupported Secure DNS signature algorithm");
    }
    let key_tag = key_tag.context("Missing keytag in Secure DNS TXT")?;
    if key_tag != I1_HARMONY_KEY_TAG {
        bail!(
            "Unexpected Secure DNS keytag {} (expected {})",
            key_tag,
            I1_HARMONY_KEY_TAG
        );
    }

    let version = version.context("Missing ver in Secure DNS TXT")?;
    let build_id = build_id.context("Missing build in Secure DNS TXT")?;
    let sha256 = sha256.context("Missing sha256 in Secure DNS TXT")?;
    if sha256.len() != 64 || !sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!("Invalid SHA-256 hex in Secure DNS TXT");
    }
    let sig_b64 = sig_b64.context("Missing sig in Secure DNS TXT")?;
    let sig_bytes = BASE64
        .decode(&sig_b64)
        .context("Invalid base64 signature in Secure DNS TXT")?;

    let pubkey_bytes =
        hex::decode(I1_HARMONY_ED25519_PUBKEY_HEX).expect("valid pinned Ed25519 pubkey hex");
    let canonical_msg = format!(
        "ST-DNS-v1|ver={}|build={}|sha256={}",
        version, build_id, sha256
    );
    let peer_pub = UnparsedPublicKey::new(&ED25519, &pubkey_bytes);
    peer_pub
        .verify(canonical_msg.as_bytes(), &sig_bytes)
        .map_err(|_| anyhow::anyhow!("Secure DNS Ed25519 signature verification failed"))?;

    Ok(SecureDnsRelease {
        version,
        build_id,
        sha256,
        key_tag,
        signature_b64: sig_b64,
        dnssec_rrsig_verified: false,
    })
}

/// Encode a domain name into canonical lowercase RFC 4034 §6.1 DNS wire format.
fn encode_canonical_dns_name(qname: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(qname.len() + 2);
    for label in qname.trim_end_matches('.').split('.') {
        if label.is_empty() {
            continue;
        }
        let lower = label.to_ascii_lowercase();
        out.push(lower.len() as u8);
        out.extend_from_slice(lower.as_bytes());
    }
    out.push(0);
    out
}

/// Build an RFC 1035 + RFC 6891/3225 EDNS0 (`DO=1` DNSSEC OK, `AD=1`) DNS TXT query packet
/// for `qname` (e.g. `_smart-tree.i1.is`) so authoritative and validating servers return `RRSIG` (type 46).
fn build_dns_txt_query_packet(qname: &str, tx_id: u16) -> Vec<u8> {
    let mut pkt = Vec::with_capacity(64);
    pkt.extend_from_slice(&tx_id.to_be_bytes());
    pkt.extend_from_slice(&0x0120u16.to_be_bytes()); // Flags: RD=1 (0x0100) | AD=1 (0x0020)
    pkt.extend_from_slice(&1u16.to_be_bytes()); // QDCOUNT = 1
    pkt.extend_from_slice(&0u16.to_be_bytes()); // ANCOUNT = 0
    pkt.extend_from_slice(&0u16.to_be_bytes()); // NSCOUNT = 0
    pkt.extend_from_slice(&1u16.to_be_bytes()); // ARCOUNT = 1 (EDNS0 OPT with DO=1)
    pkt.extend_from_slice(&encode_canonical_dns_name(qname));
    pkt.extend_from_slice(&16u16.to_be_bytes()); // QTYPE = 16 (TXT)
    pkt.extend_from_slice(&1u16.to_be_bytes()); // QCLASS = 1 (IN)
                                                // EDNS0 OPT pseudo-RR (RFC 6891 + RFC 3225 DO=1 bit):
    pkt.push(0); // Root domain name
    pkt.extend_from_slice(&41u16.to_be_bytes()); // TYPE = 41 (OPT)
    pkt.extend_from_slice(&1232u16.to_be_bytes()); // UDP payload size = 1232
    pkt.push(0); // EXTENDED-RCODE = 0
    pkt.push(0); // VERSION = 0
    pkt.extend_from_slice(&0x8000u16.to_be_bytes()); // Z flags: DO=1 (DNSSEC OK)
    pkt.extend_from_slice(&0u16.to_be_bytes()); // RDLENGTH = 0
    pkt
}

/// Skip a DNS wire name (handling RFC 1035 label compression pointers `0xC0`)
fn skip_dns_name(pkt: &[u8], mut pos: usize) -> Option<usize> {
    let mut jumps = 0;
    loop {
        let b = *pkt.get(pos)?;
        if b == 0 {
            return Some(pos + 1);
        }
        if (b & 0xC0) == 0xC0 {
            return Some(pos + 2);
        }
        pos = pos.checked_add(1 + (b as usize))?;
        jumps += 1;
        if jumps > 64 {
            return None;
        }
    }
}

/// Verify an RFC 4034 / RFC 8080 DNSSEC `RRSIG` (`rtype=46`, `type_covered=16`, `alg=15` Ed25519, `keytag=3313`)
/// over a raw `TXT` RDATA slice for `qname` against `I1_HARMONY_ED25519_PUBKEY_HEX`.
pub fn verify_dnssec_rrsig_for_txt(
    qname: &str,
    txt_rdata: &[u8],
    rrsig_rdata: &[u8],
    now_epoch_secs: Option<u64>,
) -> Result<()> {
    if rrsig_rdata.len() < 19 {
        bail!("DNSSEC RRSIG RDATA too short");
    }
    let type_covered = u16::from_be_bytes([rrsig_rdata[0], rrsig_rdata[1]]);
    let algorithm = rrsig_rdata[2];
    let _labels = rrsig_rdata[3];
    let orig_ttl = u32::from_be_bytes([
        rrsig_rdata[4],
        rrsig_rdata[5],
        rrsig_rdata[6],
        rrsig_rdata[7],
    ]);
    let sig_exp = u32::from_be_bytes([
        rrsig_rdata[8],
        rrsig_rdata[9],
        rrsig_rdata[10],
        rrsig_rdata[11],
    ]);
    let sig_inc = u32::from_be_bytes([
        rrsig_rdata[12],
        rrsig_rdata[13],
        rrsig_rdata[14],
        rrsig_rdata[15],
    ]);
    let key_tag = u16::from_be_bytes([rrsig_rdata[16], rrsig_rdata[17]]);

    if type_covered != 16 {
        bail!("DNSSEC RRSIG type_covered {} != TXT (16)", type_covered);
    }
    if algorithm != 15 {
        bail!("DNSSEC RRSIG algorithm {} != Ed25519 (15)", algorithm);
    }
    if key_tag != I1_HARMONY_KEY_TAG {
        bail!(
            "DNSSEC RRSIG keytag {} != expected {}",
            key_tag,
            I1_HARMONY_KEY_TAG
        );
    }
    if let Some(now) = now_epoch_secs {
        // Only enforce wall-clock bounds when clock is plausibly synchronized and inside uint32 epoch range
        if now >= 1_700_000_000 && now <= u64::from(u32::MAX) {
            let now32 = now as u32;
            if now32 < sig_inc || now32 > sig_exp {
                bail!(
                    "DNSSEC RRSIG outside validity window [{}..{}] (now={})",
                    sig_inc,
                    sig_exp,
                    now32
                );
            }
        }
    }

    let signer_end =
        skip_dns_name(rrsig_rdata, 18).context("Invalid signer_name in DNSSEC RRSIG")?;
    let rrsig_header = &rrsig_rdata[..signer_end];
    let signature = &rrsig_rdata[signer_end..];
    if signature.len() != 64 {
        bail!(
            "Invalid Ed25519 DNSSEC RRSIG signature length {}",
            signature.len()
        );
    }

    // RFC 4034 §3.1.8.1: signed_data = RRSIG_RDATA[..signer_end] || RR(1)
    // where RR(1) = canonical_owner_name || type(16) || class(1) || orig_ttl || rdlength || txt_rdata
    let owner_wire = encode_canonical_dns_name(qname);
    let mut signed_data =
        Vec::with_capacity(rrsig_header.len() + owner_wire.len() + 10 + txt_rdata.len());
    signed_data.extend_from_slice(rrsig_header);
    signed_data.extend_from_slice(&owner_wire);
    signed_data.extend_from_slice(&16u16.to_be_bytes()); // TYPE = TXT (16)
    signed_data.extend_from_slice(&1u16.to_be_bytes()); // CLASS = IN (1)
    signed_data.extend_from_slice(&orig_ttl.to_be_bytes());
    signed_data.extend_from_slice(&(txt_rdata.len() as u16).to_be_bytes());
    signed_data.extend_from_slice(txt_rdata);

    let pubkey_bytes =
        hex::decode(I1_HARMONY_ED25519_PUBKEY_HEX).expect("valid pinned Ed25519 pubkey hex");
    let peer_pub = UnparsedPublicKey::new(&ED25519, &pubkey_bytes);
    peer_pub.verify(&signed_data, signature).map_err(|_| {
        anyhow::anyhow!("RFC 4034 / RFC 8080 DNSSEC RRSIG Ed25519 verification failed")
    })
}

/// Parse TXT record strings (and optional DNSSEC RRSIG records) from a raw RFC 1035 DNS response packet
fn parse_dns_txt_and_rrsig_response(
    pkt: &[u8],
    expected_tx_id: u16,
) -> Option<(Vec<(String, Vec<u8>)>, Vec<Vec<u8>>)> {
    if pkt.len() < 12 {
        return None;
    }
    let tx_id = u16::from_be_bytes([pkt[0], pkt[1]]);
    if tx_id != expected_tx_id {
        return None;
    }
    let flags = u16::from_be_bytes([pkt[2], pkt[3]]);
    if (flags & 0x8000) == 0 || (flags & 0x000F) != 0 {
        return None;
    }
    let qdcount = u16::from_be_bytes([pkt[4], pkt[5]]) as usize;
    let ancount = u16::from_be_bytes([pkt[6], pkt[7]]) as usize;
    let mut pos = 12;
    for _ in 0..qdcount {
        pos = skip_dns_name(pkt, pos)?;
        pos = pos.checked_add(4)?;
    }
    let mut txts = Vec::new();
    let mut rrsigs = Vec::new();
    for _ in 0..ancount {
        pos = skip_dns_name(pkt, pos)?;
        if pos + 10 > pkt.len() {
            break;
        }
        let rtype = u16::from_be_bytes([pkt[pos], pkt[pos + 1]]);
        let rdlength = u16::from_be_bytes([pkt[pos + 8], pkt[pos + 9]]) as usize;
        pos += 10;
        if pos + rdlength > pkt.len() {
            break;
        }
        let rdata = &pkt[pos..pos + rdlength];
        if rtype == 16 {
            let mut rpos = 0;
            let mut joined = String::new();
            while rpos < rdata.len() {
                let slen = rdata[rpos] as usize;
                rpos += 1;
                if rpos + slen <= rdata.len() {
                    if let Ok(s) = std::str::from_utf8(&rdata[rpos..rpos + slen]) {
                        joined.push_str(s);
                    }
                }
                rpos += slen;
            }
            if !joined.is_empty() {
                txts.push((joined, rdata.to_vec()));
            }
        } else if rtype == 46 {
            rrsigs.push(rdata.to_vec());
        }
        pos += rdlength;
    }
    if txts.is_empty() {
        None
    } else {
        Some((txts, rrsigs))
    }
}

/// Query `_smart-tree.i1.is` (and `_smart-tree.8b.is`) via EDNS0 DNSSEC (`DO=1`) UDP DNS
/// and verify both the RFC 4034/8080 DNSSEC `RRSIG` (`rtype=46`, `alg=15`, `keytag=3313`)
/// and the inner `ST-DNS-v1` Ed25519 signature (`keytag=3313`).
/// Falls back to HTTPS `/releases/smart-tree/dns-txt` if UDP/53 is blocked (with `dnssec_rrsig_verified = false`).
pub async fn query_secure_dns_version() -> Result<SecureDnsRelease> {
    let tx_id: u16 = 0x5431;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs());
    for qname in ["_smart-tree.i1.is", "_smart-tree.8b.is"] {
        let query_pkt = build_dns_txt_query_packet(qname, tx_id);
        for &ns in HARMONY_DNS_SERVERS {
            if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
                let _ = sock.set_read_timeout(Some(Duration::from_millis(750)));
                let _ = sock.set_write_timeout(Some(Duration::from_millis(750)));
                if sock.send_to(&query_pkt, ns).is_ok() {
                    let mut buf = [0u8; 1500];
                    if let Ok((n, _)) = sock.recv_from(&mut buf) {
                        if let Some((txts, rrsigs)) =
                            parse_dns_txt_and_rrsig_response(&buf[..n], tx_id)
                        {
                            for (txt, txt_rdata) in txts {
                                if txt.contains("v=ST1") {
                                    if let Ok(mut verified) = verify_dns_version_txt(&txt) {
                                        let rrsig_ok = rrsigs.iter().any(|rrsig| {
                                            verify_dnssec_rrsig_for_txt(
                                                qname, &txt_rdata, rrsig, now,
                                            )
                                            .is_ok()
                                        });
                                        verified.dnssec_rrsig_verified = rrsig_ok;
                                        return Ok(verified);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback to HTTPS tiny TXT endpoint (~237 bytes) if outbound UDP/53 is blocked by firewall
    let client = reqwest::Client::builder()
        .user_agent("smart-tree-dns-updater")
        .timeout(Duration::from_secs(4))
        .build()?;
    let txt = client
        .get(SECURE_DNS_HTTPS_TXT)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    verify_dns_version_txt(&txt)
}

/// Strictly require an RFC 4034 / RFC 8080 DNSSEC `RRSIG`-verified AND `ST-DNS-v1` Ed25519-verified
/// response (`keytag=3313`, `alg=15` Ed25519) before allowing unattended automatic updates.
pub async fn query_secure_dns_version_strict_dnssec() -> Result<SecureDnsRelease> {
    let rel = query_secure_dns_version().await?;
    if !rel.dnssec_rrsig_verified {
        bail!(
            "Refusing unattended update: response for v{} lacked a verified RFC 4034/8080 DNSSEC RRSIG (keytag={})",
            rel.version,
            rel.key_tag
        );
    }
    Ok(rel)
}

/// Rate limit: check for updates at most once per 24 hours
const UPDATE_CHECK_INTERVAL_SECS: u64 = 86400;

/// Binaries included in the release tarball
/// Note: "n8x" replaces "tree" to avoid shadowing the real tree command
const BINARIES: &[&str] = &["st", "std", "m8", "n8x"];

/// Current version from Cargo.toml
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// GitHub release response (partial)
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    digest: String,
}

/// Update check cache
#[derive(Debug, Default, Serialize, Deserialize)]
struct UpdateCache {
    #[serde(default)]
    last_check: u64,
    #[serde(default)]
    latest_version: Option<String>,
}

/// Get the cache file path (~/.st/update_check.json)
fn get_cache_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let st_dir = home.join(".st");
    fs::create_dir_all(&st_dir)?;
    Ok(st_dir.join("update_check.json"))
}

/// Load the update cache
fn load_cache() -> UpdateCache {
    let cache_path = match get_cache_path() {
        Ok(p) => p,
        Err(_) => return UpdateCache::default(),
    };

    match fs::read_to_string(&cache_path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => UpdateCache::default(),
    }
}

/// Save the update cache
fn save_cache(cache: &UpdateCache) -> Result<()> {
    let cache_path = get_cache_path()?;
    let contents = serde_json::to_string_pretty(cache)?;
    fs::write(&cache_path, contents)?;
    Ok(())
}

/// Get current timestamp in seconds
fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Check if we should perform an update check (rate limiting)
pub fn should_check_update() -> bool {
    let cache = load_cache();
    let now = now_secs();
    now.saturating_sub(cache.last_check) > UPDATE_CHECK_INTERVAL_SECS
}

/// Compare version strings (semver-like)
pub fn is_newer_version(current: &str, latest: &str) -> bool {
    // Strip 'v' prefix if present
    let current = current.strip_prefix('v').unwrap_or(current);
    let latest = latest.strip_prefix('v').unwrap_or(latest);

    let parse_version = |v: &str| -> (u32, u32, u32) {
        let parts: Vec<u32> = v.split('.').filter_map(|p| p.parse().ok()).collect();
        (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    };

    let (curr_major, curr_minor, curr_patch) = parse_version(current);
    let (lat_major, lat_minor, lat_patch) = parse_version(latest);

    (lat_major, lat_minor, lat_patch) > (curr_major, curr_minor, curr_patch)
}

/// Check for available updates (network call) - async version.
/// First queries Secure DNS (`_smart-tree.i1.is` TXT with Ed25519 signature verification, ~290 bytes)
/// to avoid full HTTPS JSON fetches when already up to date.
pub async fn check_for_update() -> Result<Option<String>> {
    if let Ok(dns_rel) = query_secure_dns_version().await {
        let tag = format!("v{}", dns_rel.version.trim_start_matches('v'));
        let mut cache = load_cache();
        cache.last_check = now_secs();
        cache.latest_version = Some(tag.clone());
        let _ = save_cache(&cache);

        if is_newer_version(CURRENT_VERSION, &tag) {
            return Ok(Some(tag));
        } else {
            return Ok(None);
        }
    }

    let client = reqwest::Client::builder()
        .user_agent("smart-tree-updater")
        .timeout(Duration::from_secs(10))
        .build()?;

    let response: GitHubRelease = client
        .get(RELEASES_API)
        .send()
        .await
        .context("Failed to connect to the i1 release catalogue")?
        .error_for_status()?
        .json()
        .await
        .context("Failed to parse release metadata")?;

    // Update cache
    let mut cache = load_cache();
    cache.last_check = now_secs();
    cache.latest_version = Some(response.tag_name.clone());
    let _ = save_cache(&cache);

    let latest = response.tag_name;
    if is_newer_version(CURRENT_VERSION, &latest) {
        Ok(Some(latest))
    } else {
        Ok(None)
    }
}

/// Check for update using cache if within rate limit - async version
pub async fn check_for_update_cached() -> Option<String> {
    let cache = load_cache();

    if should_check_update() {
        // Perform actual check
        match check_for_update().await {
            Ok(Some(version)) => Some(version),
            Ok(None) => None,
            Err(_) => None, // Silently fail on network errors
        }
    } else {
        // Use cached result
        cache
            .latest_version
            .filter(|v| is_newer_version(CURRENT_VERSION, v))
    }
}

/// Print update available banner
pub fn print_update_banner(latest_version: &str) {
    let current = format!("v{}", CURRENT_VERSION);
    eprintln!();
    eprintln!("\x1b[36m╭─────────────────────────────────────────────────────╮\x1b[0m");
    eprintln!(
        "\x1b[36m│\x1b[0m \x1b[32m🌳 Smart Tree {} is available!\x1b[0m (you have {})",
        latest_version, current
    );
    eprintln!("\x1b[36m│\x1b[0m    Run '\x1b[1mst --update\x1b[0m' to upgrade");
    eprintln!("\x1b[36m╰─────────────────────────────────────────────────────╯\x1b[0m");
    eprintln!();
}

/// Detect the current platform for download
fn get_platform() -> Result<(&'static str, &'static str)> {
    let os = if cfg!(target_os = "macos") {
        "apple-darwin"
    } else if cfg!(target_os = "linux") {
        "unknown-linux-gnu"
    } else if cfg!(target_os = "windows") {
        "pc-windows-msvc"
    } else {
        bail!("Unsupported operating system");
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        bail!("Unsupported architecture");
    };

    Ok((arch, os))
}

/// Create a temporary directory for the update
fn create_temp_dir() -> Result<PathBuf> {
    let base = env::temp_dir();
    let unique_name = format!("st-update-{}", uuid::Uuid::new_v4());
    let temp_dir = base.join(unique_name);
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder
        .create(&temp_dir)
        .context("Failed to create temp directory")?;
    Ok(temp_dir)
}

/// Clean up a temporary directory
fn cleanup_temp_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

/// Find where the current binary is installed
fn find_install_dir() -> Result<PathBuf> {
    // Try to find where 'st' is installed
    let current_exe = env::current_exe().context("Could not determine current executable path")?;
    let install_dir = current_exe
        .parent()
        .context("Could not determine installation directory")?
        .to_path_buf();

    Ok(install_dir)
}

/// Check if we need elevated permissions
fn needs_sudo(install_dir: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let Ok(meta) = install_dir.metadata() {
            // Check if we're the owner or if we can write
            let uid = unsafe { libc::getuid() };
            if meta.uid() != uid {
                // Not owner, check if writable
                return fs::metadata(install_dir)
                    .and_then(|_| {
                        fs::OpenOptions::new()
                            .write(true)
                            .open(install_dir.join(".test_write"))
                    })
                    .is_err();
            }
        }
        false
    }
    #[cfg(not(unix))]
    {
        false
    }
}

fn read_confirmation(input: &mut impl BufRead) -> Result<bool> {
    let mut answer = String::new();
    if input.read_line(&mut answer)? == 0 {
        bail!("Update confirmation requires interactive input; nothing was installed");
    }
    let answer = answer.trim().to_lowercase();
    Ok(answer.is_empty() || answer == "y" || answer == "yes")
}

/// Download and install the update - async version
pub async fn download_and_install(version: &str, yes: bool) -> Result<()> {
    let (arch, os) = get_platform()?;
    let install_dir = find_install_dir()?;
    if install_dir.join(".auto-update/enabled").exists() {
        bail!("Managed automatic updates are enabled. Run the updater service, or disable it before a manual update.");
    }

    println!("\x1b[36m🌳 Smart Tree Updater\x1b[0m");
    println!();
    println!("Current version: v{}", CURRENT_VERSION);
    println!("Latest version:  {}", version);
    println!("Install path:    {}", install_dir.display());
    println!("Binaries:        {}", BINARIES.join(", "));
    println!();

    if !yes {
        print!("Proceed with update? [Y/n] ");
        io::stdout().flush()?;

        if !read_confirmation(&mut io::stdin().lock())? {
            println!("Update cancelled.");
            return Ok(());
        }
    }

    let use_sudo = needs_sudo(&install_dir);
    if use_sudo {
        println!("\x1b[33m⚠ Installation directory requires elevated permissions.\x1b[0m");
        println!("  You may be prompted for your password.\n");
    }

    // Construct download URL
    let ext = if cfg!(target_os = "windows") {
        "zip"
    } else {
        "tar.gz"
    };
    let archive_name = format!("st-{}-{}.{}", arch, os, ext);
    let client = reqwest::Client::builder()
        .user_agent("smart-tree-updater")
        .timeout(Duration::from_secs(300))
        .build()?;
    let release: GitHubRelease = client
        .get(RELEASES_API)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    if release.tag_name.trim_start_matches('v') != version.trim_start_matches('v') {
        bail!("The current release changed. Run st --update again.");
    }
    let asset = release.assets.iter().find(|asset| asset.name == archive_name)
        .context("No current binary for this platform. Compile with: curl -fsSL https://i1.is/tools/smart-tree | sh -s -- --compile")?;
    let download_url = &asset.browser_download_url;
    let url = reqwest::Url::parse(download_url)?;
    if url.scheme() != "https"
        || url.host_str() != Some("i1.is")
        || !url.path().starts_with("/releases/smart-tree/")
        || !url.username().is_empty()
        || url.password().is_some()
    {
        bail!("Invalid release asset URL");
    }

    println!("Downloading {}...", archive_name);

    // Create temp directory
    let temp_dir = create_temp_dir()?;
    let archive_path = temp_dir.join(&archive_name);

    // Download
    let response = client
        .get(download_url)
        .send()
        .await
        .context("Failed to download release")?;

    if !response.status().is_success() {
        bail!("Download failed: HTTP {}", response.status());
    }

    let bytes = response.bytes().await?;
    verify_asset_digest(&bytes, &asset.digest)?;
    if archive_name == "st-x86_64-unknown-linux-gnu.tar.gz" {
        if let Ok(dns_rel) = query_secure_dns_version().await {
            if dns_rel.version == version.trim_start_matches('v') {
                verify_asset_digest(&bytes, &format!("sha256:{}", dns_rel.sha256))?;
                println!(
                    "🔐 Verified Ed25519 Secure DNS signature (keytag={}) & SHA-256 digest",
                    dns_rel.key_tag
                );
            }
        }
    }
    fs::write(&archive_path, &bytes)?;

    println!("Extracting...");

    // Extract archive
    #[cfg(unix)]
    {
        let output = Command::new("tar")
            .args(["-xzf", archive_path.to_str().unwrap()])
            .current_dir(&temp_dir)
            .output()
            .context("Failed to extract archive")?;

        if !output.status.success() {
            bail!(
                "Failed to extract archive: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    #[cfg(windows)]
    {
        // On Windows, use powershell to extract zip
        let output = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    archive_path.display(),
                    &temp_dir.display()
                ),
            ])
            .output()
            .context("Failed to extract archive")?;

        if !output.status.success() {
            bail!(
                "Failed to extract archive: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    // Require the primary executable before replacing any installed companion.
    find_binary_in_dir(&temp_dir, if cfg!(windows) { "st.exe" } else { "st" })
        .context("Release archive is missing the primary st executable")?;

    // Install binaries
    println!("Installing binaries...");

    let mut installed_count = 0;
    for binary in BINARIES {
        let binary_name = if cfg!(windows) {
            format!("{}.exe", binary)
        } else {
            binary.to_string()
        };

        // Find binary in temp dir (might be at root or in subdirectory)
        let src_path = match find_binary_in_dir(&temp_dir, &binary_name) {
            Ok(path) => path,
            Err(_) => {
                // Binary not in archive - skip it (older releases may not have all binaries)
                println!("  \x1b[33m⚠\x1b[0m {} (not in archive, skipping)", binary);
                continue;
            }
        };
        let dest_path = install_dir.join(&binary_name);

        // IMPORTANT: Replace binary atomically (avoid unlinking before copy completes
        // and avoid macOS Mach-O code-signature / zombie process issues, FR-ST-02).
        #[cfg(unix)]
        {
            if use_sudo {
                let installed = Command::new("sudo")
                    .args([
                        "install",
                        "-m",
                        "0755",
                        src_path.to_str().unwrap(),
                        dest_path.to_str().unwrap(),
                    ])
                    .status()
                    .context(format!("Failed to install {}", binary))?;

                if !installed.success() {
                    bail!(
                        "Failed to install {}: sudo install exited with {}",
                        binary,
                        installed
                    );
                }
            } else {
                use std::os::unix::fs::PermissionsExt;
                let tmp_dest = install_dir.join(format!(".{}.new", binary_name));
                let _ = fs::remove_file(&tmp_dest);
                fs::copy(&src_path, &tmp_dest).context(format!("Failed to stage {}", binary))?;
                let mut perms = fs::metadata(&tmp_dest)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&tmp_dest, perms)?;
                fs::rename(&tmp_dest, &dest_path)
                    .context(format!("Failed to atomically replace {}", binary))?;
            }
        }

        #[cfg(windows)]
        {
            // On Windows, rename old binary first (can't delete while running)
            let old_path = install_dir.join(format!("{}.old", binary_name));
            let _ = fs::remove_file(&old_path);
            let _ = fs::rename(&dest_path, &old_path);

            fs::copy(&src_path, &dest_path).context(format!("Failed to install {}", binary))?;
        }

        println!("  \x1b[32m✓\x1b[0m {}", binary);
        installed_count += 1;
    }

    // Ensure at least the main binary was installed
    if installed_count == 0 {
        bail!("No binaries were installed from the archive");
    }

    // Update cache
    let mut cache = load_cache();
    cache.latest_version = Some(version.to_string());
    let _ = save_cache(&cache);

    // Clean up temp directory
    cleanup_temp_dir(&temp_dir);

    println!();
    println!("\x1b[32m✨ Successfully updated to {}!\x1b[0m", version);

    #[cfg(windows)]
    {
        println!();
        println!(
            "\x1b[33mNote: Please restart your terminal for the update to take effect.\x1b[0m"
        );
    }

    Ok(())
}

/// Find a binary file within a directory (handles nested extraction)
fn find_binary_in_dir(dir: &Path, binary_name: &str) -> Result<PathBuf> {
    // Check root
    let root_path = dir.join(binary_name);
    if root_path.exists() {
        return Ok(root_path);
    }

    // Search subdirectories
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let nested = path.join(binary_name);
            if nested.exists() {
                return Ok(nested);
            }
        }
    }

    bail!("Could not find {} in downloaded archive", binary_name)
}

/// Run the update command - async version
pub async fn run_update(yes: bool) -> Result<()> {
    println!("Checking for updates...");

    match check_for_update().await? {
        Some(version) => {
            download_and_install(&version, yes).await?;
        }
        None => {
            println!(
                "\x1b[32m✓\x1b[0m Already up to date! (v{})",
                CURRENT_VERSION
            );
        }
    }

    Ok(())
}

/// Get current version string
pub fn current_version() -> &'static str {
    CURRENT_VERSION
}

fn verify_asset_digest(bytes: &[u8], digest: &str) -> Result<()> {
    let expected = digest
        .strip_prefix("sha256:")
        .context("Missing SHA-256 release digest")?;
    let expected = hex::decode(expected).context("Invalid release digest")?;
    if expected.as_slice() != Sha256::digest(bytes).as_slice() {
        bail!("Release checksum did not match; nothing was installed");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_confirmation_rejects_closed_input() {
        assert!(read_confirmation(&mut io::Cursor::new(b"")).is_err());
        assert!(!read_confirmation(&mut io::Cursor::new(b"n\n")).unwrap());
        assert!(read_confirmation(&mut io::Cursor::new(b"yes\n")).unwrap());
        assert!(read_confirmation(&mut io::Cursor::new(b"\n")).unwrap());
    }

    #[test]
    fn archive_companions_do_not_substitute_for_primary_executable() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("m8"), b"companion").unwrap();
        assert!(find_binary_in_dir(dir.path(), "st").is_err());
        fs::write(dir.path().join("st"), b"primary").unwrap();
        assert_eq!(
            find_binary_in_dir(dir.path(), "st").unwrap(),
            dir.path().join("st")
        );
    }

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("5.5.0", "5.5.1"));
        assert!(is_newer_version("5.5.1", "5.6.0"));
        assert!(is_newer_version("5.5.1", "6.0.0"));
        assert!(is_newer_version("v5.5.0", "v5.5.1"));
        assert!(!is_newer_version("5.5.1", "5.5.1"));
        assert!(!is_newer_version("5.5.1", "5.5.0"));
        assert!(!is_newer_version("6.0.0", "5.5.1"));
    }

    #[test]
    fn test_platform_detection() {
        let result = get_platform();
        assert!(result.is_ok());
    }

    #[test]
    fn release_checksum_rejects_modified_downloads() {
        let digest = format!("sha256:{}", hex::encode(Sha256::digest(b"release")));
        assert!(verify_asset_digest(b"release", &digest).is_ok());
        assert!(verify_asset_digest(b"modified", &digest).is_err());
        assert!(verify_asset_digest(b"release", "sha256:bad").is_err());
        assert!(!is_newer_version("8.1.0", "6.5.2"));
    }

    #[test]
    fn secure_dns_ed25519_signature_verifies_and_rejects_tampering() {
        use ring::digest::{digest as ring_digest, SHA256 as RING_SHA256};
        use ring::signature::{Ed25519KeyPair, KeyPair};

        let seed_digest = ring_digest(
            &RING_SHA256,
            b"8b-is::harmony-dns::i1.is::sovereign-ed25519-ksk-v1::i1.is.",
        );
        let keypair =
            Ed25519KeyPair::from_seed_unchecked(seed_digest.as_ref()).expect("valid seed");
        assert_eq!(
            hex::encode_upper(keypair.public_key().as_ref()),
            I1_HARMONY_ED25519_PUBKEY_HEX
        );

        let sha = "a3f5c8e90123456789abcdef0123456789abcdef0123456789abcdef01234567";
        let msg = format!(
            "ST-DNS-v1|ver=10.4.0|build=10.4.0-a3f5c8e90123|sha256={}",
            sha
        );
        let sig_b64 = BASE64.encode(keypair.sign(msg.as_bytes()).as_ref());

        let txt = format!(
            "v=ST1; ver=10.4.0; build=10.4.0-a3f5c8e90123; sha256={}; keytag=3313; alg=Ed25519; sig={}",
            sha, sig_b64
        );
        let verified = verify_dns_version_txt(&txt).expect("Ed25519 DNS TXT should verify");
        assert_eq!(verified.version, "10.4.0");
        assert_eq!(verified.build_id, "10.4.0-a3f5c8e90123");
        assert_eq!(verified.sha256, sha);
        assert_eq!(verified.key_tag, 3313);

        // Tampering with version or sha256 must fail Ed25519 verification
        let tampered_ver = txt.replace("ver=10.4.0", "ver=10.4.1");
        assert!(verify_dns_version_txt(&tampered_ver).is_err());

        let tampered_sha = txt.replace("a3f5c8e9", "deadbeef");
        assert!(verify_dns_version_txt(&tampered_sha).is_err());

        // Also test full RFC 4034 / RFC 8080 DNSSEC RRSIG (rtype=46, alg=15, keytag=3313) wire verification
        let mut txt_rdata = Vec::new();
        txt_rdata.push(txt.len() as u8);
        txt_rdata.extend_from_slice(txt.as_bytes());

        let mut rrsig_header = Vec::new();
        rrsig_header.extend_from_slice(&16u16.to_be_bytes()); // type_covered = TXT (16)
        rrsig_header.push(15); // algorithm = Ed25519 (15)
        rrsig_header.push(3); // labels = 3 (_smart-tree.i1.is)
        rrsig_header.extend_from_slice(&60u32.to_be_bytes()); // orig_ttl = 60
        rrsig_header.extend_from_slice(&2_000_000_000u32.to_be_bytes()); // sig_exp
        rrsig_header.extend_from_slice(&1_700_000_000u32.to_be_bytes()); // sig_inc
        rrsig_header.extend_from_slice(&I1_HARMONY_KEY_TAG.to_be_bytes()); // keytag = 3313
        rrsig_header.extend_from_slice(&encode_canonical_dns_name("i1.is")); // signer_name = i1.is.

        let owner_wire = encode_canonical_dns_name("_smart-tree.i1.is");
        let mut signed_wire = Vec::new();
        signed_wire.extend_from_slice(&rrsig_header);
        signed_wire.extend_from_slice(&owner_wire);
        signed_wire.extend_from_slice(&16u16.to_be_bytes());
        signed_wire.extend_from_slice(&1u16.to_be_bytes());
        signed_wire.extend_from_slice(&60u32.to_be_bytes());
        signed_wire.extend_from_slice(&(txt_rdata.len() as u16).to_be_bytes());
        signed_wire.extend_from_slice(&txt_rdata);

        let rrsig_sig = keypair.sign(&signed_wire);
        let mut rrsig_rdata = rrsig_header.clone();
        rrsig_rdata.extend_from_slice(rrsig_sig.as_ref());

        verify_dnssec_rrsig_for_txt(
            "_smart-tree.i1.is",
            &txt_rdata,
            &rrsig_rdata,
            Some(1_800_000_000),
        )
        .expect("RFC 4034/8080 DNSSEC RRSIG should verify");

        // Expired or tampered RRSIG must fail
        assert!(verify_dnssec_rrsig_for_txt(
            "_smart-tree.i1.is",
            &txt_rdata,
            &rrsig_rdata,
            Some(2_100_000_000),
        )
        .is_err());
        let mut bad_txt_rdata = txt_rdata.clone();
        let last_idx = bad_txt_rdata.len() - 1;
        bad_txt_rdata[last_idx] ^= 0x01;
        assert!(verify_dnssec_rrsig_for_txt(
            "_smart-tree.i1.is",
            &bad_txt_rdata,
            &rrsig_rdata,
            Some(1_800_000_000),
        )
        .is_err());
    }
}
