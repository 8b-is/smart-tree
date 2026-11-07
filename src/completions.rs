//! Self-Maintaining Shell Autocomplete Engine for Smart Tree (`st`)
//!
//! Provides:
//! 1. **Zero-Friction Auto-Maintenance**: Automatically installs and keeps Fish, Zsh, and Bash
//!    completion scripts up to date (`~/.config/fish/completions/st.fish`, etc., plus system
//!    vendor dirs when running as the `smart-tree` daemon/service).
//! 2. **Rich Descriptions Everywhere**: Subcommands, aliases (`mesh`, `chirp`, `crab`, `foken`,
//!    `salient`), flags, and enum values all include human-friendly descriptions.
//! 3. **Dynamic Folder, Consciousness & `MEM|8` Memory Completions**:
//!    - Completes local & remembered directories annotated with `.m8` wave frequency (`Hz`),
//!      `MEM|8` memory summaries, `MemNet` share quotas, language/manifest type, and git branch.
//!    - Completes `MEM|8` memory keywords, anchor types, origins, and `MemNet` Wave Q&A packets.

use crate::cli::Cli;
use crate::memnet::MemnetNodeState;
use crate::memory_manager::{Memory, MemoryManager};
use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Schema version embedded on line 1 of auto-maintained completion files.
/// Whenever the CLI surface or completion engine changes, bumping this string
/// triggers an automatic, atomic refresh on the next `st` run or daemon tick.
pub const COMPLETIONS_SCHEMA_STAMP: &str = concat!(
    "# @st-completions-version: ",
    env!("CARGO_PKG_VERSION"),
    "-sovereign-v2 (auto-maintained by smart-tree service)"
);

/// Cached folder & memory metadata maintained by the Smart Tree service/daemon
/// at `~/.st/completion_cache.json` for sub-millisecond `<TAB>` completion.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompletionCache {
    pub version: String,
    pub updated_unix: u64,
    pub folders: Vec<CachedFolderEntry>,
    pub sync_orgs: Vec<CachedOrgEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedFolderEntry {
    pub path: String,
    pub display_path: String,
    pub name: String,
    pub description: String,
    pub frequency_hz: u16,
    pub is_memnet_share: bool,
    pub is_remembered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedOrgEntry {
    pub name: String,
    pub description: String,
    pub repo_count: usize,
}

/// Fast-path check called on CLI startup (`src/main.rs`).
/// If `~/.config/fish/completions/st.fish` is missing or outdated, atomically
/// writes the latest completions and refreshes the local folder/memory cache.
pub fn ensure_user_completions_fresh() {
    let Some(home) = dirs::home_dir() else {
        return;
    };
    let fish_path = user_fish_completion_path(&home);
    let needs_script_update = !file_has_current_stamp(&fish_path);

    if needs_script_update {
        let _ = install_user_completions_for_home(&home, None);
    }

    // If the warm completion cache doesn't exist yet, or is older than 10 minutes, refresh it lightly
    let cache_path = home.join(".st").join("completion_cache.json");
    let cache_stale = match fs::metadata(&cache_path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
    {
        Some(age) => age.as_secs() > 600,
        None => true,
    };

    if needs_script_update || cache_stale {
        let _ = refresh_completion_cache_for_home(&home);
    }
}

/// Service-level maintenance called by the Smart Tree daemon (`src/daemon.rs`),
/// `st service install` (`src/service_manager.rs`), and `st completions sync`.
pub fn maintain_service_completions(force: bool) -> Result<MaintenanceReport> {
    let mut report = MaintenanceReport::default();

    // 1. Maintain for current $HOME if present
    if let Some(home) = dirs::home_dir() {
        if force || !file_has_current_stamp(&user_fish_completion_path(&home)) {
            if let Ok(paths) = install_user_completions_for_home(&home, None) {
                report.updated_files.extend(paths);
            }
        } else {
            report
                .up_to_date_files
                .push(user_fish_completion_path(&home));
        }
        if let Ok(cache) = refresh_completion_cache_for_home(&home) {
            report.cached_folders = cache.folders.len();
            report.cached_orgs = cache.sync_orgs.len();
        }
    }

    // 2. If running under sudo or as root systemd service, also maintain for real users in /home/*
    //    and system-wide vendor completion directories.
    #[cfg(unix)]
    {
        let is_root = unsafe { libc::geteuid() == 0 };
        if is_root {
            // System-wide vendor completion directories
            let vendor_targets = [
                (
                    Shell::Fish,
                    PathBuf::from("/usr/share/fish/vendor_completions.d/st.fish"),
                ),
                (
                    Shell::Bash,
                    PathBuf::from("/usr/share/bash-completion/completions/st"),
                ),
                (
                    Shell::Zsh,
                    PathBuf::from("/usr/share/zsh/site-functions/_st"),
                ),
            ];
            for (shell, target) in vendor_targets {
                if let Some(parent) = target.parent() {
                    if parent.is_dir() && (force || !file_has_current_stamp(&target)) {
                        let content = generate_completion_script(shell);
                        if atomic_write_file(&target, content.as_bytes(), None).is_ok() {
                            report.updated_files.push(target);
                        }
                    }
                }
            }

            // Discover user home directories under /home/*
            if let Ok(entries) = fs::read_dir("/home") {
                for entry in entries.flatten() {
                    let user_home = entry.path();
                    if !user_home.is_dir() {
                        continue;
                    }
                    use std::os::unix::fs::MetadataExt;
                    let owner = fs::metadata(&user_home)
                        .ok()
                        .map(|m| (m.uid(), m.gid()))
                        .filter(|(uid, _)| *uid >= 500);
                    if owner.is_none() {
                        continue;
                    }
                    let fish_file = user_fish_completion_path(&user_home);
                    if force || !file_has_current_stamp(&fish_file) {
                        if let Ok(paths) = install_user_completions_for_home(&user_home, owner) {
                            report.updated_files.extend(paths);
                        }
                    }
                    if let Ok(cache) = refresh_completion_cache_for_home_owned(&user_home, owner) {
                        report.cached_folders = report.cached_folders.max(cache.folders.len());
                        report.cached_orgs = report.cached_orgs.max(cache.sync_orgs.len());
                    }
                }
            }
        }
    }

    if let Ok(mgr) = MemoryManager::new() {
        report.memory_count = mgr.memories().len();
    }
    let mesh = MemnetNodeState::load_or_default();
    report.memnet_waves = mesh.wave_qa_cache.len();
    report.memnet_shares = mesh.shared_folders.len();

    Ok(report)
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct MaintenanceReport {
    pub schema_stamp: String,
    pub updated_files: Vec<PathBuf>,
    pub up_to_date_files: Vec<PathBuf>,
    pub cached_folders: usize,
    pub cached_orgs: usize,
    pub memory_count: usize,
    pub memnet_waves: usize,
    pub memnet_shares: usize,
}

fn user_fish_completion_path(home: &Path) -> PathBuf {
    home.join(".config")
        .join("fish")
        .join("completions")
        .join("st.fish")
}

fn file_has_current_stamp(path: &Path) -> bool {
    use std::io::Read;
    let Ok(mut f) = fs::File::open(path) else {
        return false;
    };
    let mut buf = [0u8; 160];
    let Ok(n) = f.read(&mut buf) else {
        return false;
    };
    let header = String::from_utf8_lossy(&buf[..n]);
    header.starts_with(COMPLETIONS_SCHEMA_STAMP)
}

fn install_user_completions_for_home(
    home: &Path,
    owner: Option<(u32, u32)>,
) -> Result<Vec<PathBuf>> {
    let mut written = Vec::new();

    // 1. Fish (~/.config/fish/completions/st.fish) — always maintain for Fish users
    let fish_path = user_fish_completion_path(home);
    let fish_script = generate_completion_script(Shell::Fish);
    if atomic_write_file(&fish_path, fish_script.as_bytes(), owner).is_ok() {
        written.push(fish_path);
    }

    // 2. Bash (~/.local/share/bash-completion/completions/st)
    let bash_path = home
        .join(".local")
        .join("share")
        .join("bash-completion")
        .join("completions")
        .join("st");
    let bash_script = generate_completion_script(Shell::Bash);
    if atomic_write_file(&bash_path, bash_script.as_bytes(), owner).is_ok() {
        written.push(bash_path);
    }

    // 3. Zsh (~/.config/zsh/completions/_st)
    let zsh_path = home
        .join(".config")
        .join("zsh")
        .join("completions")
        .join("_st");
    let zsh_script = generate_completion_script(Shell::Zsh);
    if atomic_write_file(&zsh_path, zsh_script.as_bytes(), owner).is_ok() {
        written.push(zsh_path);
    }

    // 4. Sovereign Companion: Lightpanda Fish completions (~/.config/fish/completions/lightpanda.fish)
    if Path::new("/usr/bin/lightpanda").exists() {
        let lp_fish_path = home
            .join(".config")
            .join("fish")
            .join("completions")
            .join("lightpanda.fish");
        if let Ok(output) = std::process::Command::new("/usr/bin/lightpanda")
            .args(["completion", "fish"])
            .output()
        {
            if output.status.success() && !output.stdout.is_empty() {
                if atomic_write_file(&lp_fish_path, &output.stdout, owner).is_ok() {
                    written.push(lp_fish_path);
                }
            }
        }
    }

    Ok(written)
}

fn atomic_write_file(dst: &Path, content: &[u8], owner: Option<(u32, u32)>) -> Result<()> {
    let Some(parent) = dst.parent() else {
        anyhow::bail!("No parent directory for {}", dst.display());
    };
    fs::create_dir_all(parent)?;
    let tmp = parent.join(format!(
        ".{}.tmp.{}",
        dst.file_name().and_then(|s| s.to_str()).unwrap_or("st"),
        std::process::id()
    ));
    fs::write(&tmp, content)?;
    #[cfg(unix)]
    if let Some((uid, gid)) = owner {
        use std::os::unix::ffi::OsStrExt;
        if let Ok(c_path) = std::ffi::CString::new(tmp.as_os_str().as_bytes()) {
            unsafe {
                let _ = libc::chown(c_path.as_ptr(), uid, gid);
            }
        }
        if let Ok(c_parent) = std::ffi::CString::new(parent.as_os_str().as_bytes()) {
            unsafe {
                let _ = libc::chown(c_parent.as_ptr(), uid, gid);
            }
        }
    }
    fs::rename(&tmp, dst)?;
    Ok(())
}

/// Generate the completion script for `shell` with Smart Tree's enhanced
/// descriptions and dynamic folder + `MEM|8` memory completion hooks.
pub fn generate_completion_script(shell: Shell) -> String {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    let mut raw = Vec::new();
    generate(shell, &mut cmd, bin_name, &mut raw);
    let base = String::from_utf8_lossy(&raw).to_string();

    match shell {
        Shell::Fish => enrich_fish_script(&base),
        Shell::Zsh => format!("{}\n{}", COMPLETIONS_SCHEMA_STAMP, base),
        Shell::Bash => format!("{}\n{}", COMPLETIONS_SCHEMA_STAMP, base),
        _ => format!("{}\n{}", COMPLETIONS_SCHEMA_STAMP, base),
    }
}

/// Print the enhanced completion script for `shell` to stdout (used by `st --completions <SHELL>`).
pub fn print_completions(shell: Shell) {
    let script = generate_completion_script(shell);
    print!("{script}");
    let _ = io::stdout().flush();
}

use std::io::Write;

fn enrich_fish_script(base_clap_fish: &str) -> String {
    // 1. Upgrade empty descriptions in clap_complete's base output & fix clap_complete's trailing-option argparse bug
    let enriched_base = base_clap_fish
        .replace(
            "    set -l cmd (commandline -opc)\n    set -e cmd[1]",
            "    set -l cmd (commandline -opc)\n    set -e cmd[1]\n    if test (count $cmd) -ge 1; and string match -q -- '-*' $cmd[-1]\n        set -e cmd[-1]\n    end",
        )
        .replace("\"Straight from The Book\"", "Straight from The Book")
        .replace(
            "bash\\t''\nelvish\\t''\nfish\\t''\npowershell\\t''\nzsh\\t''",
            "bash\\t'Bourne Again SHell completions'\nelvish\\t'Elvish shell completions'\nfish\\t'Friendly Interactive Shell (dynamic folders + MEM|8)'\npowershell\\t'PowerShell completions'\nzsh\\t'Zsh shell completions'",
        )
        .replace(
            "enable\\t''\ndisable\\t''\nstatus\\t''",
            "enable\\t'Enable AI agent lifecycle hooks'\ndisable\\t'Disable AI agent lifecycle hooks'\nstatus\\t'Show installed AI agent hooks status'",
        )
        .replace(
            "on\\t''\noff\\t''",
            "on\\t'Enable helpful Smart Tree tips'\noff\\t'Disable startup tips'",
        )
        .replace(
            "f\\t''\nd\\t''",
            "f\\t'Regular files only'\nd\\t'Directories (folders) only'",
        );

    let mut out = String::with_capacity(enriched_base.len() + 8192);
    out.push_str(COMPLETIONS_SCHEMA_STAMP);
    out.push('\n');
    out.push_str("# Automatically maintained by Smart Tree (`st`).\n");
    out.push_str("# Dynamic completions query folders, .m8 consciousness, MEM|8 memories & MemNet waves.\n\n");
    out.push_str(&enriched_base);

    // 2. Append the Sovereign Dynamic Completion Engine for Fish
    out.push_str(
        r#"
# =============================================================================
# SMART TREE SOVEREIGN DYNAMIC AUTOCOMPLETE (Folders + MEM|8 Memories + MemNet)
# =============================================================================

function __fish_st_positional_tokens
    set -l tokens (commandline -opc)
    set -e tokens[1]
    for t in $tokens
        if not string match -q -- '-*' $t
            echo $t
        end
    end
end

function __fish_st_pos_count
    count (__fish_st_positional_tokens)
end

function __fish_st_pos_is -a idx
    set -l pos (__fish_st_positional_tokens)
    set -e argv[1]
    if test (count $pos) -ge $idx
        contains -- $pos[$idx] $argv
        return $status
    end
    return 1
end

function __fish_st_at_root_arg
    set -l cur (commandline -ct)
    string match -q -- '-*' "$cur"; and return 1
    set -l tokens (commandline -opc)
    if test (count $tokens) -ge 2
        set -l prev $tokens[-1]
        if string match -q -- '-*' "$prev"
            switch "$prev"
                case -a --all -z --compress --smart --everything --no-ignore --no-default-ignore --show-ignored --changes-only --no-security --show-filesystems --no-emoji --mcp-optimize --compact --ai-json --stream --sse-server --semantic --show-private --view-diffs --refresh-all --quarantine --no-daemon --yolo
                    # Boolean scan flags: next token is still root positional
                case '*'
                    return 1
            end
        end
    end
    test (__fish_st_pos_count) -eq 0
end

function __fish_st_complete_folders
    command st --complete-dynamic folders (commandline -ct) 2>/dev/null
end

function __fish_st_complete_memories
    command st --complete-dynamic memories (commandline -ct) 2>/dev/null
end

function __fish_st_complete_memory_types
    command st --complete-dynamic memory-types (commandline -ct) 2>/dev/null
end

function __fish_st_complete_memnet_ask
    command st --complete-dynamic memnet-ask (commandline -ct) 2>/dev/null
end

function __fish_st_complete_memnet_peers
    command st --complete-dynamic memnet-peers (commandline -ct) 2>/dev/null
end

function __fish_st_complete_sync_users
    command st --complete-dynamic sync-users (commandline -ct) 2>/dev/null
end

function __fish_st_complete_symbols
    command st --complete-dynamic symbols (commandline -ct) 2>/dev/null
end

# 1. Root positional completion: Local folders + Remembered MEM|8 projects + MemNet shares
complete -c st -n "__fish_st_at_root_arg" -f -a "(__fish_st_complete_folders)"

# 2. Subcommand aliases with descriptions
complete -c st -n "__fish_st_at_root_arg" -f -a "mesh" -d '🌐 Alias for memnet: local folder/drive sharing, ESP32/LoRa/BLE routing & Wave Q&A'
complete -c st -n "__fish_st_at_root_arg" -f -a "chirp" -d '🐦 Alias for memnet: 48kHz Ultrasonic Chirp acoustic speaker->mic modem'
complete -c st -n "__fish_st_at_root_arg" -f -a "crab" -d '🦀 Alias for symbol: CrabCC AST definitions, references, outlines & call graphs'
complete -c st -n "__fish_st_at_root_arg" -f -a "foken" -d '🪙 Alias for g8t: G8T × Smart-Tree Speed & Foken Scoreboard'
complete -c st -n "__fish_st_at_root_arg" -f -a "capsela" -d '⚡ Alias for g8t: Verified pure-Rust function capsules on g8t.is'
complete -c st -n "__fish_st_at_root_arg" -f -a "salient" -d '🧠 Alias for remember: Ingest git forge or folder into salient MEM|8 waves'

# 3. `st memnet` / `st mesh` / `st chirp` actions & arguments
complete -c st -n "__fish_st_pos_is 1 memnet mesh chirp; and test (__fish_st_pos_count) -eq 1" -f -a "status" -d '🌐 Show Sovereign MemNet node, shared folders, routes, RTC τ & MEM|8 waves'
complete -c st -n "__fish_st_pos_is 1 memnet mesh chirp; and test (__fish_st_pos_count) -eq 1" -f -a "share" -d '📂 Share a local folder & drive space quota over MemNet'
complete -c st -n "__fish_st_pos_is 1 memnet mesh chirp; and test (__fish_st_pos_count) -eq 1" -f -a "ls" -d '📁 List all shared MemNet folders, quotas & semantic tags'
complete -c st -n "__fish_st_pos_is 1 memnet mesh chirp; and test (__fish_st_pos_count) -eq 1" -f -a "routes" -d '🛰️ Show multi-medium route table & Peter Real Time Constant (RTC τ)'
complete -c st -n "__fish_st_pos_is 1 memnet mesh chirp; and test (__fish_st_pos_count) -eq 1" -f -a "add-route" -d '➕ Add/update a route (ESP32-NOW, LoRa SX1262, BLE, WiFi, Ethernet, Ultrasonic)'
complete -c st -n "__fish_st_pos_is 1 memnet mesh chirp; and test (__fish_st_pos_count) -eq 1" -f -a "escape" -d '☀️ 3rd+ Escape: Solar Fusion + Ultra-Graph + Constructive LLL + Language LLL 4D'
complete -c st -n "__fish_st_pos_is 1 memnet mesh chirp; and test (__fish_st_pos_count) -eq 1" -f -a "rtc" -d '⏱️ Peter Real Time Constant (RTC τ) continuous-time relaxation & settling audit'
complete -c st -n "__fish_st_pos_is 1 memnet mesh chirp; and test (__fish_st_pos_count) -eq 1" -f -a "lll" -d '📐 Language LLL 4D Lattice Basis Reduction (Lenstra-Lenstra-Lovász + Moser-Tardos)'
complete -c st -n "__fish_st_pos_is 1 memnet mesh chirp; and test (__fish_st_pos_count) -eq 1" -f -a "glassdoor" -d '🪟 Sovereign Glassdoor: memnet + mem|8 + space-bender + demoscene (100% NO-FLOAT)'
complete -c st -n "__fish_st_pos_is 1 memnet mesh chirp; and test (__fish_st_pos_count) -eq 1" -f -a "ask" -d '🧠 Query local area MEM|8 Wave Packet bank before waking an LLM'
complete -c st -n "__fish_st_pos_is 1 memnet mesh chirp; and test (__fish_st_pos_count) -eq 1" -f -a "teach" -d '⚡ Anchor a Q&A wave packet into local MemNet for zero-LLM-cost recall'
complete -c st -n "__fish_st_pos_is 1 memnet mesh chirp; and test (__fish_st_pos_count) -eq 1" -f -a "stream-chunk" -d '🎬 Frame audio/video media into 64B ESP32-NOW & 79B WaveFrame79 packets'
complete -c st -n "__fish_st_pos_is 1 memnet mesh chirp; and test (__fish_st_pos_count) -eq 1" -f -a "chirp-encode" -d '🐦 Encode message/file into 48kHz Ultrasonic Chirp WAV (speaker->mic modem)'
complete -c st -n "__fish_st_pos_is 1 memnet mesh chirp; and test (__fish_st_pos_count) -eq 1" -f -a "chirp-decode" -d '🎧 Decode 48kHz Ultrasonic Chirp WAV via Goertzel DFT & Hamming(12,8) FEC'

# `st memnet share <TAB>` -> complete folders with consciousness descriptions
complete -c st -n "__fish_st_pos_is 1 memnet mesh chirp; and __fish_st_pos_is 2 share stream-chunk; and test (__fish_st_pos_count) -eq 2" -f -a "(__fish_st_complete_folders)"
# `st memnet ask <TAB>` / `teach <TAB>` -> complete MEM|8 Wave Q&A & memories
complete -c st -n "__fish_st_pos_is 1 memnet mesh chirp; and __fish_st_pos_is 2 ask query teach learn; and test (__fish_st_pos_count) -eq 2" -f -a "(__fish_st_complete_memnet_ask)"
# `st memnet add-route <TAB>` -> complete peers & SSH hosts
complete -c st -n "__fish_st_pos_is 1 memnet mesh chirp; and __fish_st_pos_is 2 add-route; and test (__fish_st_pos_count) -eq 2" -f -a "(__fish_st_complete_memnet_peers)"
# `st memnet --medium <TAB>` -> transport mediums with Peter's RTC τ₀ floor
complete -c st -n "__fish_st_pos_is 1 memnet mesh chirp" -l medium -r -f -a "
wifi\t'WiFi UDP/mDNS mesh (RTC τ₀=8ms, 1400B MTU)'
ethernet\t'Gigabit Ethernet LAN (RTC τ₀=3ms, 1472B MTU)'
esp32\t'ESP32-NOW 64B MemnetCapsule64 (RTC τ₀=10ms, 250B MTU)'
bluetooth\t'BLE 8BM8 96B Extended / 31B Legacy Adv (RTC τ₀=18ms)'
lora\t'LoRa SX1262 923.2MHz long-range sub-GHz (RTC τ₀=60ms, 222B MTU)'
usb\t'USB CDC/Bulk direct tether (RTC τ₀=2ms, 4096B MTU)'
ultrasonic\t'48kHz Ultrasonic Chirp 18.5-21kHz speaker->mic (RTC τ₀=90ms)'
infrasound\t'Sub-20Hz Infrasound long-horizon beacon (RTC τ₀=250ms)'
vpn\t'WireGuard/Tailscale encrypted tunnel (RTC τ₀=25ms)'
"

# 4. `st g8t` / `st foken` / `st capsela` actions
complete -c st -n "__fish_st_pos_is 1 g8t foken capsela; and test (__fish_st_pos_count) -eq 1" -f -a "leaderboard" -d '📊 Show live G8T × Smart-Tree Speed & Foken Scoreboard'
complete -c st -n "__fish_st_pos_is 1 g8t foken capsela; and test (__fish_st_pos_count) -eq 1" -f -a "best" -d '🏆 Query g8t.is for fastest verified Rust functions by keyword or signature'
complete -c st -n "__fish_st_pos_is 1 g8t foken capsela; and test (__fish_st_pos_count) -eq 1" -f -a "yolo" -d '⚡ Return ONLY the #1 fastest verified Rust function (raw drop-in code + receipt)'
complete -c st -n "__fish_st_pos_is 1 g8t foken capsela; and test (__fish_st_pos_count) -eq 1" -f -a "submit" -d '🚀 Submit pure-Rust function file/snippet to g8t.is with hardware cycle count'

# 5. `st remember` / `st salient` / `st ingest` targets, levels & queries
complete -c st -n "__fish_st_pos_is 1 remember salient ingest; and test (__fish_st_pos_count) -eq 1" -f -a "(__fish_st_complete_folders)"
complete -c st -n "__fish_st_pos_is 1 remember salient ingest; and test (__fish_st_pos_count) -eq 1" -f -a "(__fish_st_complete_sync_users)"
complete -c st -n "__fish_st_pos_is 1 remember salient ingest" -l level -r -f -a "
extreme\t'🌌 Full Nexus depth (complete code + thoughts + wave synthesis)'
professor\t'🎓 Deep academic & architectural extraction'
expert\t'🔬 High-salience AST, diffs & wave compression (default)'
high\t'⚡ Detailed structural & semantic ingestion'
medium\t'📐 Balanced summary & key symbols'
low\t'🌱 Lightweight repository overview'
bullshitter\t'🎭 Fast surface-level gist'
"
complete -c st -n "__fish_st_pos_is 1 remember salient ingest" -s q -l query -r -f -a "(__fish_st_complete_memories)"

# 6. `st sync` / `st gh-sync` GitHub user/org completions
complete -c st -n "__fish_st_pos_is 1 sync gh-sync sync-user sync-github; and test (__fish_st_pos_count) -eq 1" -f -a "(__fish_st_complete_sync_users)"

# 7. `st symbol` / `st crab` AST symbol & file completions
complete -c st -n "__fish_st_pos_is 1 symbol crab; and __fish_st_pos_is 2 sym refs; and test (__fish_st_pos_count) -eq 2" -f -a "(__fish_st_complete_symbols)"
complete -c st -n "__fish_st_pos_is 1 symbol crab; and __fish_st_pos_is 2 outline; and test (__fish_st_pos_count) -eq 2" -f -a "(__fish_st_complete_folders)"

# 8. `st completions` / `st autocomplete` subcommand actions
complete -c st -n "__fish_st_pos_is 1 completions autocomplete complete; and test (__fish_st_pos_count) -eq 1" -f -a "status" -d '📊 Show auto-maintained shell completion status, cached folders & MEM|8 memories'
complete -c st -n "__fish_st_pos_is 1 completions autocomplete complete; and test (__fish_st_pos_count) -eq 1" -f -a "sync" -d '🔄 Force-refresh shell completion scripts and folder/memory completion cache'
complete -c st -n "__fish_st_pos_is 1 completions autocomplete complete; and test (__fish_st_pos_count) -eq 1" -f -a "folders" -d '📂 Preview live folder & remembered project completions with descriptions'
complete -c st -n "__fish_st_pos_is 1 completions autocomplete complete; and test (__fish_st_pos_count) -eq 1" -f -a "memories" -d '🧠 Preview live MEM|8 memory & Wave Q&A completions with descriptions'
complete -c st -n "__fish_st_pos_is 1 completions autocomplete complete; and test (__fish_st_pos_count) -eq 1" -f -a "fish" -d '🐟 Emit enhanced Fish shell completion script to stdout'
complete -c st -n "__fish_st_pos_is 1 completions autocomplete complete; and test (__fish_st_pos_count) -eq 1" -f -a "zsh" -d '⚡ Emit Zsh shell completion script to stdout'
complete -c st -n "__fish_st_pos_is 1 completions autocomplete complete; and test (__fish_st_pos_count) -eq 1" -f -a "bash" -d '🐚 Emit Bash shell completion script to stdout'

# 9. Global flags backed by live MEM|8 memories, folders, and value descriptions
complete -c st -l memory-find -d 'Find memories by keywords (comma-separated)' -r -f -a "(__fish_st_complete_memories)"
complete -c st -l recall -d 'Recall indexed files and remembered context through the daemon' -r -f -a "(__fish_st_complete_memories)"
complete -c st -l memory-anchor -d 'Anchor a memory: --memory-anchor <TYPE> <KEYWORDS> <CONTEXT>' -r -f -a "(__fish_st_complete_memory_types)"
complete -c st -l sync-user -d 'Sync all GitHub repositories for a user/org' -r -f -a "(__fish_st_complete_sync_users)"
complete -c st -l update-consciousness -d 'Update .m8 consciousness files for a directory' -r -f -a "(__fish_st_complete_folders)"
complete -c st -l security-scan -d 'Scan codebase for supply chain attack patterns' -r -f -a "(__fish_st_complete_folders)"
complete -c st -l integrity-scan -d 'Deep integrity scan (exploit detection, hash memory, cert analysis)' -r -f -a "(__fish_st_complete_folders)"
complete -c st -l cert-scan -d 'Inspect PEM/DER certificates embedded in files' -r -f -a "(__fish_st_complete_folders)"
complete -c st -l erdos-anomaly -d 'Run Paul Erdős–Rényi Shadow Graph Supply-Chain Anomaly Detector' -r -f -a "(__fish_st_complete_folders)"
complete -c st -l token-stats -d 'Show tokenization statistics for a path' -r -f -a "(__fish_st_complete_folders)"
complete -c st -l get-frequency -d 'Get wave frequency for a directory' -r -f -a "(__fish_st_complete_folders)"
complete -c st -l ramsey-cliques -d 'Detect Ramsey-Theoretic Monochromatic Cliques & Turán Density' -r -f -a "(__fish_st_complete_folders)"
complete -c st -l erdos-center -d 'Compute Erdős Center Z(G), Graph Radius/Diameter & Steiner Backbone' -r -f -a "(__fish_st_complete_folders)"
complete -c st -l zeno -d 'Execute Sovereign ULTRA-OMNI-ZENO v11 engine on a path' -r -f -a "(__fish_st_complete_folders)"
complete -c st -l gdrive -d 'Google Drive / Gmail operations' -r -f -a "
status\t'☁️ Show Google Drive / Gmail authentication & sync status'
list\t'📋 List recent files in Google Drive'
sync\t'🔄 Synchronize directory with Google Drive'
login\t'🔑 Authenticate with Google OAuth2'
logout\t'🚪 Clear stored Google credentials'
"
complete -c st -l voice -d 'Voice & audio operations' -r -f -a "
status\t'🎙️ Show voice synthesis & Marine VAD audio status'
test\t'🔊 Play voice test phrase'
speak\t'🗣️ Speak text aloud via Smart Tree voice engine'
"
complete -c st -l input -d 'Specify input type explicitly' -r -f -a "
filesystem\t'📂 Local directory or file path (default)'
mem8\t'🧠 MEM|8 binary consciousness (.m8) wave store'
qcp\t'🌊 Quantum Context Protocol stream'
sse\t'📡 Server-Sent Events live stream'
openapi\t'📜 OpenAPI / Swagger specification'
"
complete -c st -l relations-filter -d 'Filter code relationships' -r -f -a "
imports\t'📦 Module & package import edges'
calls\t'📞 Function & method call-graph edges'
types\t'🧬 Struct, enum & trait type dependencies'
tests\t'🧪 Test-to-implementation coverage links'
coupled\t'🔗 Tightly coupled co-changing files'
"
complete -c st -l type -d 'Filter by file extension' -r -f -a "
rs\t'🦀 Rust source files (.rs)'
py\t'🐍 Python source files (.py)'
ts\t'📘 TypeScript files (.ts)'
tsx\t'⚛️ TypeScript JSX files (.tsx)'
js\t'🟨 JavaScript files (.js)'
go\t'🐹 Go source files (.go)'
c\t'⚙️ C source files (.c)'
cpp\t'⚙️ C++ source files (.cpp)'
md\t'📝 Markdown documentation (.md)'
toml\t'🔧 TOML manifest/config files (.toml)'
json\t'📋 JSON data files (.json)'
yaml\t'📐 YAML configuration files (.yaml)'
fish\t'🐟 Fish shell scripts (.fish)'
sh\t'🐚 POSIX/Bash shell scripts (.sh)'
m8\t'🧠 MEM|8 binary wave files (.m8)'
"
"#,
    );

    out
}

// =============================================================================
// DYNAMIC COMPLETION ENGINE (`st --complete-dynamic <KIND> [PREFIX]`)
// =============================================================================

/// Execute a fast dynamic completion query and print `value\tdescription` lines to stdout.
pub fn handle_dynamic_completion(args: &[String]) -> Result<()> {
    let kind = args.first().map(|s| s.as_str()).unwrap_or("folders");
    let prefix = args.get(1).map(|s| s.as_str()).unwrap_or("");

    let items = match kind {
        "folders" | "paths" | "dirs" => complete_folders(prefix),
        "memories" | "memory" | "recall" => complete_memories(prefix),
        "memory-types" | "anchor-types" => complete_memory_types(prefix),
        "memnet-ask" | "waves" => complete_memnet_ask(prefix),
        "memnet-peers" | "routes" => complete_memnet_peers(prefix),
        "sync-users" | "orgs" | "users" => complete_sync_users(prefix),
        "symbols" | "sym" => complete_symbols(prefix),
        _ => complete_folders(prefix),
    };

    let mut stdout = io::stdout().lock();
    for (val, desc) in items {
        let clean_val = sanitize_completion_field(&val);
        let clean_desc = truncate_clean(&sanitize_completion_field(&desc), 78);
        if !clean_val.is_empty() {
            let _ = writeln!(stdout, "{clean_val}\t{clean_desc}");
        }
    }
    let _ = stdout.flush();
    Ok(())
}

fn sanitize_completion_field(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\t' | '\n' | '\r' => ' ',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

fn truncate_clean(s: &str, max_chars: usize) -> String {
    let mut chars = s.chars();
    let collected: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{collected}…")
    } else {
        collected
    }
}

/// Complete folders based on:
/// 1. Local directories matching `prefix` (annotated with `.m8` consciousness, `MEM|8` memories,
///    `MemNet` share quotas, Rust/Node/Python/Git manifests)
/// 2. Remembered `MEM|8` project folders (`~/.st/memories.m8` & `~/.st/completion_cache.json`)
/// 3. Shared `MemNet` folders (`~/.mem8/memnet/mesh_state.json`)
pub fn complete_folders(prefix: &str) -> Vec<(String, String)> {
    let mut results: Vec<(String, String)> = Vec::new();
    let mut seen_paths: HashSet<String> = HashSet::new();

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Load remembered directory map: canonical path -> (frequency_hz, summary)
    let mut memory_map: BTreeMap<String, (u16, String)> = BTreeMap::new();
    if let Ok(mgr) = MemoryManager::new() {
        for mem in mgr.memories() {
            if mem.origin.starts_with('/') {
                let hz = mem.frequency.round().clamp(1.0, 9999.0) as u16;
                memory_map.insert(
                    mem.origin.clone(),
                    (hz, summarize_memory_context(&mem.context)),
                );
            }
        }
    }

    // Load warm completion cache if available
    let cache = load_completion_cache(&home);
    for entry in &cache.folders {
        memory_map
            .entry(entry.path.clone())
            .or_insert_with(|| (entry.frequency_hz, entry.description.clone()));
    }

    // Load MemNet shared folders: path -> (quota_mb, semantic_essence)
    let mut memnet_shares: BTreeMap<String, (u64, String)> = BTreeMap::new();
    let mesh = MemnetNodeState::load_or_default();
    for share in mesh.shared_folders.values() {
        memnet_shares.insert(
            share.host_path.clone(),
            (share.offered_drive_space_mb, share.semantic_essence.clone()),
        );
    }

    // Determine target base directory and entry filter from `prefix`
    let expanded_prefix = if let Some(rest) = prefix.strip_prefix("~/") {
        home.join(rest).to_string_lossy().to_string()
    } else if prefix == "~" {
        format!("{}/", home.display())
    } else {
        prefix.to_string()
    };

    let (search_dir, dir_prefix_for_output, name_filter) =
        if let Some((parent_part, file_part)) = expanded_prefix.rsplit_once('/') {
            let base = if parent_part.is_empty() {
                PathBuf::from("/")
            } else {
                PathBuf::from(parent_part)
            };
            let out_prefix = if let Some((orig_parent, _)) = prefix.rsplit_once('/') {
                format!("{orig_parent}/")
            } else {
                String::new()
            };
            (base, out_prefix, file_part.to_lowercase())
        } else {
            (cwd.clone(), String::new(), expanded_prefix.to_lowercase())
        };

    let show_hidden = name_filter.starts_with('.');

    // 1. Scan immediate subdirectories of `search_dir`
    if let Ok(entries) = fs::read_dir(&search_dir) {
        let mut local_dirs: Vec<(String, PathBuf)> = entries
            .flatten()
            .filter_map(|e| {
                let ft = e.file_type().ok()?;
                let is_dir = ft.is_dir() || (ft.is_symlink() && e.path().is_dir());
                if !is_dir {
                    return None;
                }
                let name = e.file_name().to_string_lossy().to_string();
                if !show_hidden && name.starts_with('.') {
                    return None;
                }
                if !name_filter.is_empty() && !name.to_lowercase().starts_with(&name_filter) {
                    return None;
                }
                Some((name, e.path()))
            })
            .collect();

        local_dirs.sort_by(|a, b| a.0.cmp(&b.0));

        for (name, full_path) in local_dirs.into_iter().take(40) {
            let canon = full_path
                .canonicalize()
                .unwrap_or_else(|_| full_path.clone())
                .to_string_lossy()
                .to_string();
            let candidate = format!("{dir_prefix_for_output}{name}/");
            let desc = describe_directory(&full_path, &canon, &memory_map, &memnet_shares);
            seen_paths.insert(canon);
            results.push((candidate, desc));
        }
    }

    // 2. Offer Remembered MEM|8 Project Folders & MemNet Shared Folders (even when in another directory!)
    let lower_prefix = prefix.to_lowercase();
    for entry in &cache.folders {
        if seen_paths.contains(&entry.path) {
            continue;
        }
        let matches = lower_prefix.is_empty()
            || entry.display_path.to_lowercase().contains(&lower_prefix)
            || entry.name.to_lowercase().contains(&lower_prefix)
            || entry.description.to_lowercase().contains(&lower_prefix);
        if !matches {
            continue;
        }
        if !Path::new(&entry.path).is_dir() {
            continue;
        }
        seen_paths.insert(entry.path.clone());
        let badge = if entry.is_memnet_share {
            format!("🌐 MemNet • {}Hz", entry.frequency_hz)
        } else if entry.is_remembered {
            format!("🧠 MEM|8 • {}Hz", entry.frequency_hz)
        } else {
            format!("📂 Workspace • {}Hz", entry.frequency_hz)
        };
        results.push((
            entry.display_path.clone(),
            format!("{badge} — {}", entry.description),
        ));
        if results.len() >= 55 {
            break;
        }
    }

    // Also include any direct `memories.m8` project origins not yet in cache
    for (origin, (hz, summary)) in &memory_map {
        if seen_paths.contains(origin) || results.len() >= 60 {
            continue;
        }
        let p = Path::new(origin);
        if !p.is_dir() {
            continue;
        }
        let display = shorten_home(p, &home);
        if !lower_prefix.is_empty()
            && !display.to_lowercase().contains(&lower_prefix)
            && !summary.to_lowercase().contains(&lower_prefix)
        {
            continue;
        }
        seen_paths.insert(origin.clone());
        results.push((display, format!("🧠 MEM|8 [{hz}Hz] — {summary}")));
    }

    results
}

fn describe_directory(
    dir: &Path,
    canon: &str,
    memory_map: &BTreeMap<String, (u16, String)>,
    memnet_shares: &BTreeMap<String, (u64, String)>,
) -> String {
    let mut badges: Vec<String> = Vec::new();

    if let Some((quota_mb, essence)) = memnet_shares.get(canon) {
        if essence.is_empty() {
            badges.push(format!("🌐 MemNet {quota_mb}MB"));
        } else {
            badges.push(format!(
                "🌐 MemNet {quota_mb}MB [{}]",
                truncate_clean(essence, 24)
            ));
        }
    }

    if let Some((hz, mem_summary)) = memory_map.get(canon) {
        badges.push(format!("🧠 {hz}Hz"));
        badges.push(mem_summary.clone());
        return badges.join(" • ");
    }

    // Check .m8 consciousness file
    if dir.join(".m8").is_file() {
        if let Ok(c) = crate::m8_consciousness::DirectoryConsciousness::load(dir) {
            let hz = c.frequency.round() as u16;
            badges.push(format!("🌊 {hz}Hz"));
            if !c.essence.is_empty() {
                badges.push(c.essence);
                return badges.join(" • ");
            }
        }
    }

    // Fast manifest & git inspection
    let branch = fs::read_to_string(dir.join(".git").join("HEAD"))
        .ok()
        .and_then(|s| {
            s.trim()
                .strip_prefix("ref: refs/heads/")
                .map(str::to_string)
        });

    if let Some((_, summary)) = MemoryManager::synthesize_directory_memory(dir) {
        let icon = if dir.join("Cargo.toml").is_file() {
            "🦀"
        } else if dir.join("package.json").is_file() {
            "📦"
        } else if dir.join("pyproject.toml").is_file() {
            "🐍"
        } else if dir.join("go.mod").is_file() {
            "🐹"
        } else if branch.is_some() {
            "📂"
        } else {
            "📁"
        };
        badges.push(format!("{icon} {summary}"));
        return badges.join(" • ");
    }

    let count = fs::read_dir(dir).map(|rd| rd.count()).unwrap_or(0);
    if let Some(b) = branch {
        format!("📂 Git repo [{b}] ({count} items)")
    } else {
        format!("📁 Directory ({count} items)")
    }
}

fn summarize_memory_context(ctx: &str) -> String {
    let first_segment = ctx.split(" | ").next().unwrap_or(ctx).trim();
    truncate_clean(first_segment, 64)
}

fn shorten_home(path: &Path, home: &Path) -> String {
    if let Ok(rel) = path.strip_prefix(home) {
        let s = rel.to_string_lossy();
        if s.is_empty() {
            "~/".to_string()
        } else {
            format!("~/{s}/")
        }
    } else {
        format!("{}/", path.display())
    }
}

/// Complete `MEM|8` memories (keywords, anchor types, origins, and `MemNet` Wave Q&A packets).
pub fn complete_memories(prefix: &str) -> Vec<(String, String)> {
    let lower = prefix.to_lowercase();
    let mut results: Vec<(String, String)> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    if let Ok(mgr) = MemoryManager::new() {
        let mut memories: Vec<&Memory> = mgr.memories().iter().collect();
        memories.sort_by_key(|b| std::cmp::Reverse(b.timestamp));

        // 1. Individual keywords from anchored memories with context + Hz + origin
        for mem in &memories {
            let hz = mem.frequency.round() as u16;
            let origin_short = Path::new(&mem.origin)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&mem.origin);
            let ctx_short = truncate_clean(&mem.context, 48);

            for kw in &mem.keywords {
                let kw_clean = kw.trim();
                if kw_clean.is_empty() || seen.contains(kw_clean) {
                    continue;
                }
                if !lower.is_empty()
                    && !kw_clean.to_lowercase().contains(&lower)
                    && !mem.context.to_lowercase().contains(&lower)
                {
                    continue;
                }
                seen.insert(kw_clean.to_string());
                results.push((
                    kw_clean.to_string(),
                    format!(
                        "🧠 [{} @ {}Hz] {} ({})",
                        mem.anchor_type, hz, ctx_short, origin_short
                    ),
                ));
            }
        }

        // 2. Anchor types with memory count
        let mut type_counts: BTreeMap<String, usize> = BTreeMap::new();
        for mem in &memories {
            *type_counts.entry(mem.anchor_type.clone()).or_insert(0) += 1;
        }
        for (t, count) in type_counts {
            if !seen.contains(&t) && (lower.is_empty() || t.to_lowercase().contains(&lower)) {
                seen.insert(t.clone());
                results.push((
                    t.clone(),
                    format!("🏷️ Memory anchor type ({count} anchored memories)"),
                ));
            }
        }
    }

    // 3. Also include MemNet Wave Q&A semantic tokens
    let mesh = MemnetNodeState::load_or_default();
    for wave in &mesh.wave_qa_cache {
        for kw in &wave.semantic_tokens {
            if !seen.contains(kw) && (lower.is_empty() || kw.to_lowercase().contains(&lower)) {
                seen.insert(kw.clone());
                results.push((
                    kw.clone(),
                    format!(
                        "🌊 [MemNet Wave {}] {}",
                        wave.wave_b3,
                        truncate_clean(&wave.answer, 48)
                    ),
                ));
            }
        }
    }

    results.truncate(50);
    results
}

/// Complete `--memory-anchor <TYPE>` with descriptions and live counts.
pub fn complete_memory_types(prefix: &str) -> Vec<(String, String)> {
    let lower = prefix.to_lowercase();
    let types = [
        (
            "insight",
            "💡 Architectural, performance, or debugging insight worth preserving",
        ),
        (
            "decision",
            "⚖️ Design or trade-off decision with clear rationale",
        ),
        (
            "pattern",
            "🧬 Reusable code or protocol pattern (e.g. NO-FLOAT Q16, RTC τ)",
        ),
        (
            "gotcha",
            "⚠️ Subtle trap, edge case, or pitfall to avoid in future sessions",
        ),
        ("todo", "📌 Pending follow-up task or roadmap milestone"),
        (
            "project",
            "📂 Directory or repository consciousness summary",
        ),
        (
            "infrastructure",
            "🛰️ Server, SSH host, or MemNet mesh topology memory",
        ),
    ];

    types
        .into_iter()
        .filter(|(k, _)| lower.is_empty() || k.starts_with(&lower))
        .map(|(k, d)| (k.to_string(), d.to_string()))
        .collect()
}

/// Complete `st memnet ask <TAB>` from `MemNet` Wave Q&A packets and `MEM|8` memories.
pub fn complete_memnet_ask(prefix: &str) -> Vec<(String, String)> {
    let lower = prefix.to_lowercase();
    let mut results = Vec::new();
    let mut seen = HashSet::new();

    let mesh = MemnetNodeState::load_or_default();
    for wave in &mesh.wave_qa_cache {
        for kw in &wave.semantic_tokens {
            if !seen.contains(kw)
                && (lower.is_empty()
                    || kw.to_lowercase().contains(&lower)
                    || wave.question.to_lowercase().contains(&lower))
            {
                seen.insert(kw.clone());
                results.push((
                    kw.clone(),
                    format!(
                        "🌊 [{} • {} hits] Q: {} -> {}",
                        wave.wave_b3,
                        wave.hit_count,
                        truncate_clean(&wave.question, 26),
                        truncate_clean(&wave.answer, 34)
                    ),
                ));
            }
        }
    }

    for (kw, desc) in complete_memories(prefix) {
        if !seen.contains(&kw) {
            seen.insert(kw.clone());
            results.push((kw, desc));
        }
    }

    results.truncate(40);
    results
}

/// Complete `st memnet add-route <TAB>` from known MemNet routes and SSH config hosts.
pub fn complete_memnet_peers(prefix: &str) -> Vec<(String, String)> {
    let lower = prefix.to_lowercase();
    let mut results = Vec::new();
    let mut seen = HashSet::new();

    let mesh = MemnetNodeState::load_or_default();
    for r in &mesh.routes {
        if lower.is_empty() || r.node_name.to_lowercase().contains(&lower) {
            seen.insert(r.node_name.clone());
            results.push((
                r.node_name.clone(),
                format!(
                    "🛰️ {} via {} (RTC τ={}ms, 3τ={}ms)",
                    r.medium,
                    r.endpoint,
                    r.effective_rtc_ms(),
                    r.settling_3tau_ms()
                ),
            ));
        }
    }

    if let Some(home) = dirs::home_dir() {
        if let Ok(cfg) = fs::read_to_string(home.join(".ssh").join("config")) {
            if let Some((keywords, _)) = MemoryManager::parse_ssh_config_summary(&cfg) {
                for kw in keywords {
                    if !matches!(
                        kw.as_str(),
                        "ssh" | "connections" | "hosts" | "servers" | "config"
                    ) && !seen.contains(&kw)
                        && (lower.is_empty() || kw.contains(&lower))
                    {
                        seen.insert(kw.clone());
                        results.push((kw, "🔑 Remembered SSH host from ~/.ssh/config".to_string()));
                    }
                }
            }
        }
    }

    results
}

/// Complete GitHub users/organizations for `st sync <TAB>` and `--sync-user <TAB>`.
pub fn complete_sync_users(prefix: &str) -> Vec<(String, String)> {
    let lower = prefix.to_lowercase();
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let cache = load_completion_cache(&home);
    let mut results = Vec::new();
    let mut seen = HashSet::new();

    for org in cache.sync_orgs {
        if lower.is_empty() || org.name.to_lowercase().contains(&lower) {
            seen.insert(org.name.clone());
            results.push((org.name, org.description));
        }
    }

    // Fallback well-known sovereign orgs
    for (name, desc) in [
        (
            "8b-is",
            "🐙 8b-is Sovereign Organization (smart-tree, mem8, demoscene, space-bender)",
        ),
        ("8bit-wraith", "🐙 8bit-wraith GitHub Repositories"),
        (
            "standardgalactic",
            "🌌 StandardGalactic Full Nexus Collection",
        ),
        ("flyxion", "🔬 Flyxion Research & Multi-Dimensional Systems"),
    ] {
        if !seen.contains(name) && (lower.is_empty() || name.contains(&lower)) {
            results.push((name.to_string(), desc.to_string()));
        }
    }

    results
}

/// Complete Rust/source symbols in the current workspace for `st symbol sym <TAB>` / `refs <TAB>`.
pub fn complete_symbols(prefix: &str) -> Vec<(String, String)> {
    let lower = prefix.to_lowercase();
    let mut results = Vec::new();
    let mut seen = HashSet::new();

    let src_dir = Path::new("src");
    if src_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(src_dir) {
            for entry in entries.flatten().take(40) {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                    continue;
                }
                let fname = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("src.rs");
                if let Ok(content) = fs::read_to_string(&path) {
                    for (line_idx, line) in content.lines().enumerate() {
                        let t = line.trim();
                        let (kind, rest) = if let Some(r) = t.strip_prefix("pub fn ") {
                            ("fn", r)
                        } else if let Some(r) = t.strip_prefix("pub async fn ") {
                            ("async fn", r)
                        } else if let Some(r) = t.strip_prefix("pub const fn ") {
                            ("const fn", r)
                        } else if let Some(r) = t.strip_prefix("pub struct ") {
                            ("struct", r)
                        } else if let Some(r) = t.strip_prefix("pub enum ") {
                            ("enum", r)
                        } else {
                            continue;
                        };
                        let sym: String = rest
                            .chars()
                            .take_while(|c| c.is_alphanumeric() || *c == '_')
                            .collect();
                        if sym.len() >= 3
                            && !seen.contains(&sym)
                            && (lower.is_empty() || sym.to_lowercase().contains(&lower))
                        {
                            seen.insert(sym.clone());
                            results.push((
                                sym,
                                format!("🦀 pub {kind} in src/{fname}:{}", line_idx + 1),
                            ));
                            if results.len() >= 45 {
                                return results;
                            }
                        }
                    }
                }
            }
        }
    }

    results
}

// =============================================================================
// WARM COMPLETION CACHE MAINTENANCE (`~/.st/completion_cache.json`)
// =============================================================================

fn load_completion_cache(home: &Path) -> CompletionCache {
    let cache_path = home.join(".st").join("completion_cache.json");
    if let Ok(data) = fs::read_to_string(&cache_path) {
        if let Ok(cache) = serde_json::from_str::<CompletionCache>(&data) {
            return cache;
        }
    }
    refresh_completion_cache_for_home(home).unwrap_or_default()
}

pub fn refresh_completion_cache_for_home(home: &Path) -> Result<CompletionCache> {
    refresh_completion_cache_for_home_owned(home, None)
}

fn refresh_completion_cache_for_home_owned(
    home: &Path,
    owner: Option<(u32, u32)>,
) -> Result<CompletionCache> {
    let mut folders: Vec<CachedFolderEntry> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    // Ensure current directory is remembered if we're in a project
    let mut mem_mgr = MemoryManager::new().ok();
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(mgr) = mem_mgr.as_mut() {
            let _ = mgr.ensure_directory_remembered(&cwd);
        }
    }

    // Map of MemNet shared paths
    let mut memnet_paths: HashSet<String> = HashSet::new();
    let mesh = MemnetNodeState::load_or_default();
    for share in mesh.shared_folders.values() {
        memnet_paths.insert(share.host_path.clone());
        let p = Path::new(&share.host_path);
        if p.is_dir() && seen.insert(share.host_path.clone()) {
            let name = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&share.share_name)
                .to_string();
            let desc = format!(
                "Shared on MemNet ({} MB quota, {} files) — {}",
                share.offered_drive_space_mb,
                share.total_files,
                truncate_clean(&share.semantic_essence, 40)
            );
            folders.push(CachedFolderEntry {
                path: share.host_path.clone(),
                display_path: shorten_home(p, home),
                name,
                description: desc,
                frequency_hz: (share.consciousness_freq_hz.clamp(20, 9999)) as u16,
                is_memnet_share: true,
                is_remembered: true,
            });
        }
    }

    // 1. Harvest remembered directories from `memories.m8`
    if let Some(mgr) = mem_mgr.as_ref() {
        for mem in mgr.memories() {
            if !mem.origin.starts_with('/') {
                continue;
            }
            let p = Path::new(&mem.origin);
            if !p.is_dir() || !seen.insert(mem.origin.clone()) {
                continue;
            }
            let name = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("project")
                .to_string();
            folders.push(CachedFolderEntry {
                path: mem.origin.clone(),
                display_path: shorten_home(p, home),
                name,
                description: summarize_memory_context(&mem.context),
                frequency_hz: mem.frequency.round().clamp(20.0, 999.0) as u16,
                is_memnet_share: memnet_paths.contains(&mem.origin),
                is_remembered: true,
            });
        }
    }

    // 2. Discover workspace directories & GitHub orgs under ~/source
    let mut sync_orgs: Vec<CachedOrgEntry> = Vec::new();
    let source_root = home.join("source");
    if source_root.is_dir() {
        if let Ok(entries) = fs::read_dir(&source_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let dir_name = entry.file_name().to_string_lossy().to_string();
                if dir_name.starts_with('.') {
                    continue;
                }

                // Count child git repos or projects
                let mut child_repos = 0usize;
                let mut child_projects = Vec::new();
                if let Ok(children) = fs::read_dir(&path) {
                    for child in children.flatten().take(250) {
                        let cp = child.path();
                        if !cp.is_dir() {
                            continue;
                        }
                        if cp.join(".git").exists() || cp.join("Cargo.toml").is_file() {
                            child_repos += 1;
                            if child_projects.len() < 12 {
                                child_projects.push(cp);
                            }
                        }
                    }
                }

                if child_repos > 0 {
                    sync_orgs.push(CachedOrgEntry {
                        name: dir_name.clone(),
                        description: format!(
                            "🐙 GitHub Org/Workspace • {child_repos} local repos in ~/source/{dir_name}"
                        ),
                        repo_count: child_repos,
                    });
                }

                // Index notable projects inside ~/source/<org>/<repo>
                for proj in child_projects {
                    let canon = proj
                        .canonicalize()
                        .unwrap_or_else(|_| proj.clone())
                        .to_string_lossy()
                        .to_string();
                    if !seen.insert(canon.clone()) {
                        continue;
                    }
                    if let Some((_, summary)) = MemoryManager::synthesize_directory_memory(&proj) {
                        let name = proj
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("repo")
                            .to_string();
                        let freq = 40
                            + ((canon.bytes().fold(0u32, |a, b| a.wrapping_add(b as u32)) % 180)
                                as u16);
                        folders.push(CachedFolderEntry {
                            path: canon.clone(),
                            display_path: shorten_home(&proj, home),
                            name,
                            description: truncate_clean(&summary, 64),
                            frequency_hz: freq,
                            is_memnet_share: memnet_paths.contains(&canon),
                            is_remembered: false,
                        });
                    }
                }
            }
        }
    }

    sync_orgs.sort_by_key(|b| std::cmp::Reverse(b.repo_count));

    let now_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let cache = CompletionCache {
        version: COMPLETIONS_SCHEMA_STAMP.to_string(),
        updated_unix: now_unix,
        folders,
        sync_orgs,
    };

    let cache_path = home.join(".st").join("completion_cache.json");
    if let Ok(json) = serde_json::to_vec_pretty(&cache) {
        let _ = atomic_write_file(&cache_path, &json, owner);
    }

    Ok(cache)
}

/// CLI handler for `st completions [status|sync|folders|memories|fish|zsh|bash]`
pub fn handle_completions_command(cmd: &crate::cli::CompletionsCommand) -> Result<()> {
    let action = cmd.action.to_ascii_lowercase();
    let prefix = cmd.prefix.as_deref().unwrap_or("");

    match action.as_str() {
        "fish" => {
            print_completions(Shell::Fish);
            return Ok(());
        }
        "zsh" => {
            print_completions(Shell::Zsh);
            return Ok(());
        }
        "bash" => {
            print_completions(Shell::Bash);
            return Ok(());
        }
        "powershell" | "ps" => {
            print_completions(Shell::PowerShell);
            return Ok(());
        }
        "folders" | "dirs" | "paths" => {
            return handle_dynamic_completion(&["folders".to_string(), prefix.to_string()]);
        }
        "memories" | "memory" | "waves" => {
            return handle_dynamic_completion(&["memories".to_string(), prefix.to_string()]);
        }
        _ => {}
    }

    let force = matches!(action.as_str(), "sync" | "install" | "maintain" | "refresh");
    let report = maintain_service_completions(force)?;

    println!("🌳 Smart Tree Sovereign Autocomplete Service Status");
    println!("════════════════════════════════════════════════════════════════════════");
    println!("  Schema Stamp:     {}", COMPLETIONS_SCHEMA_STAMP);
    println!("  Service Policy:   Self-Maintaining (Daemon + Service Install + CLI Auto-Sync)");
    if !report.updated_files.is_empty() {
        println!("  Updated Scripts:");
        for p in &report.updated_files {
            println!("    ✅ {} (refreshed)", p.display());
        }
    }
    if !report.up_to_date_files.is_empty() {
        println!("  Active Scripts:");
        for p in &report.up_to_date_files {
            println!("    🟢 {} (up-to-date)", p.display());
        }
    }
    println!();
    println!("  Live Dynamic Completion Index (~/.st/completion_cache.json):");
    println!(
        "    📂 Indexed Folders & Projects: {} (with .m8 Hz, Git branch & manifest summary)",
        report.cached_folders
    );
    println!(
        "    🧠 Anchored MEM|8 Memories:    {} (keywords, types, origins & frequencies)",
        report.memory_count
    );
    println!(
        "    🌊 MemNet Wave Q&A Packets:    {} (zero-LLM-cost local wave answers)",
        report.memnet_waves
    );
    println!(
        "    🌐 MemNet Shared Folders:      {}",
        report.memnet_shares
    );
    println!("    🐙 GitHub Sync Orgs/Users:     {}", report.cached_orgs);

    println!();
    println!("  Sample Folder Completions (`st <TAB>`):");
    for (cand, desc) in complete_folders(prefix).into_iter().take(6) {
        println!("    {:28} -> {}", cand, desc);
    }

    println!();
    println!("  Sample Memory Completions (`st --memory-find <TAB>` / `st memnet ask <TAB>`):");
    for (cand, desc) in complete_memories(prefix).into_iter().take(6) {
        println!("    {:28} -> {}", cand, desc);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fish_completion_script_includes_descriptions_and_dynamic_hooks() {
        let script = generate_completion_script(Shell::Fish);
        assert!(script.starts_with(COMPLETIONS_SCHEMA_STAMP));
        assert!(script.contains("__fish_st_complete_folders"));
        assert!(script.contains("__fish_st_complete_memories"));
        assert!(script.contains("__fish_st_complete_memnet_ask"));
        assert!(script.contains("glassdoor"));
        assert!(script.contains("Sovereign Glassdoor"));
        assert!(script.contains("RTC τ₀=10ms"));
    }

    #[test]
    fn test_dynamic_memory_types_have_descriptions() {
        let types = complete_memory_types("");
        assert!(types.len() >= 6);
        assert!(types
            .iter()
            .any(|(k, d)| k == "pattern" && d.contains("NO-FLOAT")));
    }

    #[test]
    fn test_dynamic_folders_returns_annotated_entries() {
        let folders = complete_folders("");
        assert!(!folders.is_empty());
        for (val, desc) in &folders {
            assert!(!val.is_empty());
            assert!(!desc.is_empty());
            assert!(!desc.contains('\t'));
        }
    }
}
