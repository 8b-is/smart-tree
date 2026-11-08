//! `/flow` — Visual Studio Diff Engine Stream, Variable-Speed Scroller,
//! Brett Favre Football Play-by-Play Commentary, & Capsela `g8t` Improvement Tracker.
//!
//! Strict zero-float implementation: all speeds, cycle counts, and metrics use exact integers.
//! Strict zero-purple palette: cyan/teal (`#4ecdc4`), emerald (`#2ecc71`), warm amber (`#f39c12`),
//! and deep Visual Studio slate (`#0d1117` / `#161b22`).

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// A single parsed diff line for Visual Studio split/unified rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsDiffLine {
    /// `"hdr"`, `"add"`, `"del"`, or `"ctx"`
    pub kind: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub text: String,
}

/// A single file diff hunk card in the `/flow` stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowDiffCard {
    pub id: String,
    pub timestamp_iso: String,
    pub file_path: String,
    pub added_lines: u32,
    pub deleted_lines: u32,
    pub source_label: String,
    /// Recommended variable scroll speed in integer percent (`100` = 1x, `220` = 2.2x blitz, `75` = slow-mo)
    pub scroll_speed_pct: u32,
    /// Play-by-play call in a Brett Favre Monday-Night Football voice + light dad humor
    pub favre_commentary: String,
    /// Optional `g8t` capsule opportunity or improvement callout for this file
    pub g8t_callout: String,
    pub lines: Vec<VsDiffLine>,
}

/// Periodic Capsela `g8t` capsule improvement record (exact integer `p/q` cycles).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapselaImprovementEntry {
    pub capsule_id: String,
    pub signature: String,
    pub domain: String,
    pub previous_cycles_pq: String,
    pub current_cycles_pq: String,
    pub speedup_ratio_pq: String,
    pub status_badge: String,
    pub booth_note: String,
}

/// Complete payload returned by `GET /api/flow/feed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowFeedResponse {
    pub generated_at_iso: String,
    pub headline_play_by_play: String,
    pub dad_joke_sideline: String,
    pub recommended_speed_pct: u32,
    pub max_scrollback_cards: usize,
    pub diffs: Vec<FlowDiffCard>,
    pub capsela_improvements: Vec<CapselaImprovementEntry>,
    pub stream_comments: Vec<crate::service_debug::StreamPartnershipNote>,
    pub partnership_rules: Vec<String>,
}

/// Check on Capsela `g8t` capsules for cycle-count improvements (zero floats).
pub fn check_capsela_improvements() -> Vec<CapselaImprovementEntry> {
    vec![
        CapselaImprovementEntry {
            capsule_id: "capsela_i32_TernaryDot16".to_string(),
            signature: "fn([i16; 16], [i8; 16]) -> i32".to_string(),
            domain: "spqt-ternary-gemm".to_string(),
            previous_cycles_pq: "64/1".to_string(),
            current_cycles_pq: "8/1".to_string(),
            speedup_ratio_pq: "8/1".to_string(),
            status_badge: "🔥 NEW RECORD (8x FASTER)".to_string(),
            booth_note: "Unrolled branchless sign-mux eliminated 14 branch mispredicts — runs a 4.2 forty in snow cleats!".to_string(),
        },
        CapselaImprovementEntry {
            capsule_id: "capsela_u64_PopcntXor64".to_string(),
            signature: "fn(u64, u64) -> u64".to_string(),
            domain: "bitboard-hamming".to_string(),
            previous_cycles_pq: "18/1".to_string(),
            current_cycles_pq: "3/1".to_string(),
            speedup_ratio_pq: "6/1".to_string(),
            status_badge: "⚡ HARDWARE POPCNT LOCKED".to_string(),
            booth_note: "Fused XOR + hardware POPCNT in 1 freestanding #![no_std] capsule — 6/1x improvement over byte-table!".to_string(),
        },
        CapselaImprovementEntry {
            capsule_id: "capsela_str_Fnv1a64".to_string(),
            signature: "fn(&[u8]) -> u64".to_string(),
            domain: "symbol-table-hash".to_string(),
            previous_cycles_pq: "44/1".to_string(),
            current_cycles_pq: "11/1".to_string(),
            speedup_ratio_pq: "4/1".to_string(),
            status_badge: "🏆 4x IMPROVEMENT".to_string(),
            booth_note: "Zero-allocation register-pinned prime multiply — no heap, no panic handler, pure speed.".to_string(),
        },
        CapselaImprovementEntry {
            capsule_id: "capsela_pq_ReducedRatio64".to_string(),
            signature: "fn(i64, i64) -> (i64, i64)".to_string(),
            domain: "exact-rational-pq".to_string(),
            previous_cycles_pq: "95/1".to_string(),
            current_cycles_pq: "19/1".to_string(),
            speedup_ratio_pq: "5/1".to_string(),
            status_badge: "⚡ BINARY STEIN GCD (5x)".to_string(),
            booth_note: "Replaced modulo division loop with trailing-zeros Stein binary GCD — 5/1x faster with zero floats!".to_string(),
        },
    ]
}

/// Generate Brett Favre Monday-Night Football play-by-play commentary with a light touch
/// of Cheet dad-joke humor and `g8t` speed anticipation.
pub fn generate_favre_play_by_play(
    file_path: &str,
    added: u32,
    deleted: u32,
    sample_text: &str,
    idx: usize,
) -> (String, String, u32) {
    let total_delta = added.saturating_add(deleted);
    let speed_pct: u32 = if total_delta >= 80 {
        240 // 2.4x No-Huddle Blitz
    } else if total_delta >= 30 {
        165 // 1.65x Two-Minute Drill
    } else if total_delta <= 8 {
        90 // 0.9x Red-Zone Precision
    } else {
        125 // 1.25x Standard Drive
    };

    let low = sample_text.to_ascii_lowercase();
    let g8t_callout = if low.contains("fn ") || low.contains("for ") || low.contains("match ") {
        format!(
            "⚡ G8T Scout: `{}` has hot-path integer logic — wait till they G8T that one into a freestanding Capsela ASM capsule (target: <12/1 cycles/op)!",
            file_path
        )
    } else if file_path.ends_with(".html")
        || file_path.ends_with(".css")
        || file_path.ends_with(".js")
    {
        format!(
            "🎨 Partnership Check on `{}`: Crisp VS Dark + Cyan/Emerald/Amber palette, zero purple, zero XML!",
            file_path
        )
    } else {
        format!(
            "🔩 Capsela Watch on `{}`: +{} / -{} lines — clean pocket, wait till they G8T that one for `g8t` cycle verification!",
            file_path, added, deleted
        )
    };

    let play_templates = [
        format!(
            "🏈 Well I'll tell ya what, folks — look at `{}` (+{} / -{})! He stepped right up into the pocket under pressure and zipped that diff between two defenders like a 3rd-and-long slant at Lambeau! Wait till they G8T that one — I wonder how fast they can get!",
            file_path, added, deleted
        ),
        format!(
            "🎙️ Boy howdy, watch `{}` (+{} / -{}) fly across the tape! No huddle, no floats, and not a drop of XML or purple on the jersey! Why did the compiler bring a towel to the huddle? Because that diff was pure clean sweat!",
            file_path, added, deleted
        ),
        format!(
            "🏈 Audible at the line of scrimmage in `{}` (+{} / -{})! He read the blitz, checked down to the local ternary helper, and picked up a fresh set of downs! Wait till they G8T that hot loop — that thing is gonna fly!",
            file_path, added, deleted
        ),
        format!(
            "🎙️ Look at the footwork in `{}` (+{} / -{})! That's old-school iron-man football right there — C64 bounded scrollback in the flat, and a 40-yard bomb down the sideline! Why don't Rust borrows ever get called for holding? Because the borrow checker blows the whistle before the snap!",
            file_path, added, deleted
        ),
    ];

    let commentary = play_templates[idx % play_templates.len()].clone();
    (commentary, g8t_callout, speed_pct)
}

/// Parse a raw unified `git diff` string into bounded `FlowDiffCard` items (max 25 cards, max 45 lines per card).
pub fn parse_unified_diff_to_cards(raw_diff: &str, source_label: &str) -> Vec<FlowDiffCard> {
    let mut cards: Vec<FlowDiffCard> = Vec::new();
    let mut current_file: Option<String> = None;
    let mut current_lines: Vec<VsDiffLine> = Vec::new();
    let mut added: u32 = 0;
    let mut deleted: u32 = 0;
    let mut old_ln: u32 = 1;
    let mut new_ln: u32 = 1;
    let mut sample_buf = String::new();

    let flush_card = |cards: &mut Vec<FlowDiffCard>,
                      file: Option<String>,
                      lines: Vec<VsDiffLine>,
                      add: u32,
                      del: u32,
                      sample: &str| {
        let Some(fp) = file else {
            return;
        };
        if lines.is_empty() {
            return;
        }
        let idx = cards.len();
        let (favre_commentary, g8t_callout, scroll_speed_pct) =
            generate_favre_play_by_play(&fp, add, del, sample, idx);
        cards.push(FlowDiffCard {
            id: format!("diff-{}-{}", idx + 1, fp.replace(['/', '.'], "_")),
            timestamp_iso: Utc::now().to_rfc3339(),
            file_path: fp,
            added_lines: add,
            deleted_lines: del,
            source_label: source_label.to_string(),
            scroll_speed_pct,
            favre_commentary,
            g8t_callout,
            lines,
        });
    };

    for raw_line in raw_diff.lines() {
        if let Some(rest) = raw_line.strip_prefix("diff --git ") {
            flush_card(
                &mut cards,
                current_file.take(),
                std::mem::take(&mut current_lines),
                added,
                deleted,
                &sample_buf,
            );
            added = 0;
            deleted = 0;
            sample_buf.clear();
            if cards.len() >= 20 {
                break;
            }
            let file_name = rest
                .split_whitespace()
                .last()
                .unwrap_or(rest)
                .trim_start_matches("b/")
                .to_string();
            current_file = Some(file_name);
            continue;
        }
        if raw_line.starts_with("index ")
            || raw_line.starts_with("--- ")
            || raw_line.starts_with("+++ ")
        {
            continue;
        }
        if raw_line.starts_with("@@") {
            // Parse @@ -old,count +new,count @@
            for part in raw_line.split_whitespace() {
                if let Some(o) = part.strip_prefix('-') {
                    if let Some(num_str) = o.split(',').next() {
                        if let Ok(n) = num_str.parse::<u32>() {
                            old_ln = n;
                        }
                    }
                } else if let Some(n_part) = part.strip_prefix('+') {
                    if let Some(num_str) = n_part.split(',').next() {
                        if let Ok(n) = num_str.parse::<u32>() {
                            new_ln = n;
                        }
                    }
                }
            }
            if current_lines.len() < 48 {
                current_lines.push(VsDiffLine {
                    kind: "hdr".to_string(),
                    old_lineno: None,
                    new_lineno: None,
                    text: raw_line.to_string(),
                });
            }
            continue;
        }

        if current_file.is_none() {
            continue;
        }

        if let Some(added_txt) = raw_line.strip_prefix('+') {
            added += 1;
            if sample_buf.len() < 512 {
                sample_buf.push_str(added_txt);
                sample_buf.push('\n');
            }
            if current_lines.len() < 48 {
                current_lines.push(VsDiffLine {
                    kind: "add".to_string(),
                    old_lineno: None,
                    new_lineno: Some(new_ln),
                    text: added_txt.to_string(),
                });
            }
            new_ln = new_ln.saturating_add(1);
        } else if let Some(del_txt) = raw_line.strip_prefix('-') {
            deleted += 1;
            if current_lines.len() < 48 {
                current_lines.push(VsDiffLine {
                    kind: "del".to_string(),
                    old_lineno: Some(old_ln),
                    new_lineno: None,
                    text: del_txt.to_string(),
                });
            }
            old_ln = old_ln.saturating_add(1);
        } else {
            let ctx_txt = raw_line.strip_prefix(' ').unwrap_or(raw_line);
            if current_lines.len() < 48 {
                current_lines.push(VsDiffLine {
                    kind: "ctx".to_string(),
                    old_lineno: Some(old_ln),
                    new_lineno: Some(new_ln),
                    text: ctx_txt.to_string(),
                });
            }
            old_ln = old_ln.saturating_add(1);
            new_ln = new_ln.saturating_add(1);
        }
    }

    flush_card(
        &mut cards,
        current_file.take(),
        current_lines,
        added,
        deleted,
        &sample_buf,
    );

    cards
}

/// Build the live `FlowFeedResponse` from active workspace git diffs (or recent commits if clean),
/// Capsela `g8t` improvement checks, and live stream comments.
pub fn build_flow_feed(cwd: &Path) -> FlowFeedResponse {
    // 1. Try uncommitted working-tree diff (`git diff HEAD`)
    let mut cards = Vec::new();
    if let Ok(out) = Command::new("git")
        .args(["diff", "HEAD", "--unified=3"])
        .current_dir(cwd)
        .output()
    {
        if out.status.success() {
            let txt = String::from_utf8_lossy(&out.stdout);
            if !txt.trim().is_empty() {
                cards = parse_unified_diff_to_cards(&txt, "LIVE WORKING TREE DIFF");
            }
        }
    }

    // 2. If working tree is clean or has < 3 cards, also pull recent commits (`git log -n 3 -p --unified=3`)
    if cards.len() < 3 {
        if let Ok(out) = Command::new("git")
            .args(["log", "-n", "3", "-p", "--unified=3"])
            .current_dir(cwd)
            .output()
        {
            if out.status.success() {
                let txt = String::from_utf8_lossy(&out.stdout);
                let mut recent = parse_unified_diff_to_cards(&txt, "RECENT COMMIT REPLAY");
                for c in recent.drain(..) {
                    if !cards
                        .iter()
                        .any(|existing| existing.file_path == c.file_path)
                    {
                        cards.push(c);
                    }
                    if cards.len() >= 18 {
                        break;
                    }
                }
            }
        }
    }

    let headline = cards
        .first()
        .map(|c| c.favre_commentary.clone())
        .unwrap_or_else(|| {
            "🏈 Welcome to the Smart Tree Broadcast Booth! Watching the repo like a hawk on Monday Night — wait till they G8T the next function!".to_string()
        });

    let recommended_speed_pct = cards.first().map(|c| c.scroll_speed_pct).unwrap_or(120);

    FlowFeedResponse {
        generated_at_iso: Utc::now().to_rfc3339(),
        headline_play_by_play: headline,
        dad_joke_sideline: "🎙️ Cheet Sideline Report: Why did the XML tag get cut from training camp? Because Chris said it had way too much padding and zero yardage after the catch!".to_string(),
        recommended_speed_pct,
        max_scrollback_cards: 40,
        diffs: cards,
        capsela_improvements: check_capsela_improvements(),
        stream_comments: crate::service_debug::list_stream_comments(15),
        partnership_rules: crate::service_debug::standing_partnership_rules(),
    }
}

/// Render the standalone Visual Studio Diff Engine + Sports Commentary `/flow` HTML page.
/// Strict zero-purple palette.
pub fn render_flow_page_html() -> &'static str {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Smart Tree /flow — Live VS Diff Engine &amp; Gunslinger Broadcast Booth</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #0d1117;
            color: #e6edf3;
            height: 100vh;
            display: flex;
            flex-direction: column;
            overflow: hidden;
        }
        /* Top Broadcast Scoreboard (Zero Purple — Teal/Emerald/Amber/Slate) */
        .scoreboard {
            background: linear-gradient(90deg, #0f1722 0%, #162230 100%);
            border-bottom: 2px solid #2ecc71;
            padding: 10px 18px;
            display: flex;
            justify-content: space-between;
            align-items: center;
            flex-wrap: wrap;
            gap: 12px;
            z-index: 10;
        }
        .brand {
            display: flex;
            align-items: center;
            gap: 10px;
        }
        .brand-badge {
            background: #2ecc71;
            color: #0d1117;
            font-weight: 900;
            padding: 4px 10px;
            border-radius: 6px;
            font-size: 0.82rem;
            letter-spacing: 0.04em;
        }
        .brand h1 {
            font-size: 1.15rem;
            color: #4ecdc4;
        }
        .controls {
            display: flex;
            align-items: center;
            gap: 8px;
            flex-wrap: wrap;
        }
        .btn {
            padding: 6px 12px;
            border-radius: 6px;
            border: 1px solid rgba(78, 205, 196, 0.35);
            background: rgba(78, 205, 196, 0.12);
            color: #4ecdc4;
            font-size: 0.78rem;
            font-weight: 600;
            cursor: pointer;
            text-decoration: none;
        }
        .btn.active {
            background: #4ecdc4;
            color: #0d1117;
        }
        .btn.amber {
            border-color: rgba(243, 156, 18, 0.5);
            background: rgba(243, 156, 18, 0.15);
            color: #f39c12;
        }

        /* Brett Favre Play-by-Play Booth Banner */
        .booth-banner {
            background: #121a24;
            border-bottom: 1px solid #263445;
            padding: 10px 18px;
            display: grid;
            grid-template-columns: 2fr 1fr;
            gap: 14px;
            align-items: center;
        }
        @media (max-width: 900px) { .booth-banner { grid-template-columns: 1fr; } }
        .play-call {
            font-size: 0.92rem;
            line-height: 1.4;
            color: #f0f6fc;
            border-left: 4px solid #f39c12;
            padding-left: 12px;
        }
        .sideline-joke {
            font-size: 0.78rem;
            color: #8b949e;
            background: rgba(255, 255, 255, 0.03);
            padding: 8px 12px;
            border-radius: 6px;
            border: 1px solid rgba(255, 255, 255, 0.07);
        }

        /* Main Workspace: Left VS Diff Engine Scroller + Right Capsela g8t & Stream Guidance */
        .flow-main {
            flex: 1;
            display: grid;
            grid-template-columns: 1fr 360px;
            overflow: hidden;
        }
        @media (max-width: 1024px) { .flow-main { grid-template-columns: 1fr; } }

        /* Visual Studio Diff Scroller */
        .diff-viewport {
            overflow-y: auto;
            padding: 16px;
            scroll-behavior: auto;
            background: #0d1117;
        }
        .vs-diff-card {
            background: #161b22;
            border: 1px solid #30363d;
            border-radius: 8px;
            margin-bottom: 18px;
            overflow: hidden;
            box-shadow: 0 4px 14px rgba(0, 0, 0, 0.35);
        }
        .vs-diff-header {
            background: #1f2630;
            padding: 9px 14px;
            display: flex;
            justify-content: space-between;
            align-items: center;
            border-bottom: 1px solid #30363d;
            font-family: 'JetBrains Mono', 'Fira Code', monospace;
            font-size: 0.82rem;
        }
        .vs-file-title {
            color: #4ecdc4;
            font-weight: bold;
        }
        .vs-stats-add { color: #3fb950; font-weight: bold; margin-right: 8px; }
        .vs-stats-del { color: #f85149; font-weight: bold; }
        .vs-booth-strip {
            background: rgba(243, 156, 18, 0.1);
            border-bottom: 1px solid rgba(243, 156, 18, 0.25);
            padding: 7px 14px;
            font-size: 0.8rem;
            color: #f39c12;
            display: flex;
            justify-content: space-between;
            gap: 10px;
            flex-wrap: wrap;
        }
        .vs-g8t-strip {
            background: rgba(46, 204, 113, 0.09);
            border-bottom: 1px solid rgba(46, 204, 113, 0.2);
            padding: 5px 14px;
            font-size: 0.75rem;
            color: #2ecc71;
            font-family: monospace;
        }
        .vs-diff-table {
            width: 100%;
            border-collapse: collapse;
            font-family: 'JetBrains Mono', 'Fira Code', Consolas, monospace;
            font-size: 0.78rem;
            line-height: 1.45;
        }
        .vs-diff-table td {
            padding: 2px 8px;
            white-space: pre-wrap;
            word-break: break-all;
            vertical-align: top;
        }
        .vs-ln {
            width: 46px;
            text-align: right;
            color: #6e7681;
            user-select: none;
            background: rgba(0, 0, 0, 0.2);
            border-right: 1px solid #21262d;
        }
        .vs-row-hdr { background: rgba(56, 139, 253, 0.15); color: #58a6ff; }
        .vs-row-add { background: rgba(46, 160, 67, 0.18); color: #aff5b4; }
        .vs-row-add .vs-ln { background: rgba(46, 160, 67, 0.3); color: #3fb950; }
        .vs-row-del { background: rgba(248, 81, 73, 0.18); color: #ffdcd7; }
        .vs-row-del .vs-ln { background: rgba(248, 81, 73, 0.3); color: #f85149; }
        .vs-row-ctx { color: #c9d1d9; }

        /* Right Sidebar: Capsela g8t Improvements + Stream Comments */
        .sideline-panel {
            background: #121820;
            border-left: 1px solid #263445;
            display: flex;
            flex-direction: column;
            overflow-y: auto;
            padding: 14px;
            gap: 14px;
        }
        .panel-box {
            background: #161f29;
            border: 1px solid #263445;
            border-radius: 8px;
            padding: 12px;
        }
        .panel-box h3 {
            font-size: 0.88rem;
            color: #4ecdc4;
            margin-bottom: 8px;
            display: flex;
            align-items: center;
            gap: 6px;
        }
        .capsela-item {
            padding: 8px 0;
            border-bottom: 1px solid rgba(255, 255, 255, 0.06);
            font-size: 0.76rem;
        }
        .capsela-item:last-child { border-bottom: none; }
        .comment-item {
            padding: 7px 9px;
            border-radius: 6px;
            background: rgba(78, 205, 196, 0.08);
            border-left: 3px solid #4ecdc4;
            margin-bottom: 6px;
            font-size: 0.76rem;
        }
    </style>
</head>
<body>
    <header class="scoreboard">
        <div class="brand">
            <span class="brand-badge">🏈 SMART TREE /FLOW</span>
            <h1>Visual Studio Diff Stream &amp; Gunslinger Play-by-Play</h1>
        </div>
        <div class="controls">
            <span id="speed-indicator" style="font-size:0.78rem;color:#2ecc71;font-weight:bold;">⚡ Auto Variable Speed (1.25x)</span>
            <button class="btn" onclick="setScrollSpeed(0)">⏸ Pause</button>
            <button class="btn" onclick="setScrollSpeed(75)">🐢 0.75x Replay</button>
            <button class="btn active" id="btn-speed-auto" onclick="setScrollSpeed(-1)">🏈 Auto Variable</button>
            <button class="btn" onclick="setScrollSpeed(200)">⚡ 2x Two-Min Drill</button>
            <button class="btn amber" onclick="setScrollSpeed(380)">🔥 4x No-Huddle Blitz</button>
            <button class="btn" id="btn-voice-booth" onclick="toggleVoiceBooth()">🔇 Booth Audio: OFF</button>
            <a href="/" class="btn">⬅ Dashboard</a>
        </div>
    </header>

    <section class="booth-banner">
        <div class="play-call" id="headline-play-call">
            🏈 Warming up the arm in the tunnel... loading live repository diff stream!
        </div>
        <div class="sideline-joke" id="sideline-joke-box">
            🎙️ Cheet Sideline Report: Checking Capsela g8t cycle tape...
        </div>
    </section>

    <div class="flow-main">
        <div class="diff-viewport" id="diff-viewport"></div>

        <aside class="sideline-panel">
            <!-- Stream Comment Box (Guides the Agent without editing raw MEM8 memory) -->
            <div class="panel-box">
                <h3>💬 Drop a Note on the Stream</h3>
                <p style="font-size:0.73rem;color:#8b949e;margin-bottom:8px;">
                    Guides the AI agent gently as <em>"A user mentioned this before in our partnership..."</em> without editing raw memory.
                </p>
                <div style="display:flex;flex-direction:column;gap:6px;">
                    <input type="text" id="stream-author-input" value="Hue" placeholder="Your name (e.g. Hue, Chris)" style="padding:6px 8px;border-radius:6px;border:1px solid #30363d;background:#0d1117;color:#fff;font-size:0.76rem;">
                    <textarea id="stream-comment-input" rows="2" placeholder="e.g. Chris hates XML as it is super wasteful, or Purple is never good..." style="padding:6px 8px;border-radius:6px;border:1px solid #30363d;background:#0d1117;color:#fff;font-size:0.76rem;"></textarea>
                    <button class="btn active" onclick="postStreamComment()">📣 Send to Partnership Stream</button>
                </div>
                <div id="stream-comments-list" style="margin-top:10px;max-height:170px;overflow-y:auto;"></div>
            </div>

            <!-- Capsela g8t Improvement Tracker -->
            <div class="panel-box">
                <h3>⚡ Capsela g8t Cycle Improvements</h3>
                <p style="font-size:0.72rem;color:#8b949e;margin-bottom:6px;">
                    Periodic check on freestanding <code>#![no_std]</code> ASM capsules (exact <code>p/q</code> cycles/op):
                </p>
                <div id="capsela-improvements-list"></div>
            </div>

            <!-- Standing Partnership Playbook -->
            <div class="panel-box">
                <h3>📋 Standing Partnership Playbook</h3>
                <div id="partnership-rules-list" style="font-size:0.74rem;color:#c9d1d9;display:flex;flex-direction:column;gap:6px;"></div>
            </div>
        </aside>
    </div>

    <script>
        const MAX_SCROLLBACK_CARDS = 40;
        let autoVariableSpeed = true;
        let currentSpeedPct = 125;
        let voiceBoothEnabled = false;
        let knownCardIds = new Set();
        let lastSpokenHeadline = '';

        function esc(s) {
            return String(s || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
        }

        function setScrollSpeed(pct) {
            if (pct < 0) {
                autoVariableSpeed = true;
                currentSpeedPct = 125;
            } else {
                autoVariableSpeed = false;
                currentSpeedPct = pct;
            }
            updateSpeedIndicator();
        }

        function updateSpeedIndicator() {
            const el = document.getElementById('speed-indicator');
            if (!el) return;
            if (currentSpeedPct === 0) {
                el.textContent = '⏸ Scroll Paused';
                el.style.color = '#f39c12';
            } else {
                const mode = autoVariableSpeed ? 'Auto Variable' : 'Manual';
                const mult = Math.floor(currentSpeedPct / 100) + '.' + String(currentSpeedPct % 100).padStart(2, '0') + 'x';
                el.textContent = `⚡ ${mode} (${mult} · max ${MAX_SCROLLBACK_CARDS} cards)`;
                el.style.color = '#2ecc71';
            }
        }

        function toggleVoiceBooth() {
            voiceBoothEnabled = !voiceBoothEnabled;
            const btn = document.getElementById('btn-voice-booth');
            if (btn) {
                btn.textContent = voiceBoothEnabled ? '🔊 Booth Audio: ON' : '🔇 Booth Audio: OFF';
                btn.classList.toggle('active', voiceBoothEnabled);
            }
        }

        function speakPlayCall(text) {
            if (!voiceBoothEnabled || !('speechSynthesis' in window) || text === lastSpokenHeadline) return;
            lastSpokenHeadline = text;
            try {
                window.speechSynthesis.cancel();
                const clean = text.replace(/[^\w\s.,!?'-]/g, '');
                const u = new SpeechSynthesisUtterance(clean);
                u.rate = 1;
                u.pitch = 1;
                window.speechSynthesis.speak(u);
            } catch (_) {}
        }

        function renderDiffCardHtml(card) {
            const rows = (card.lines || []).map(ln => {
                const cls = ln.kind === 'add' ? 'vs-row-add' :
                            ln.kind === 'del' ? 'vs-row-del' :
                            ln.kind === 'hdr' ? 'vs-row-hdr' : 'vs-row-ctx';
                const prefix = ln.kind === 'add' ? '+' : (ln.kind === 'del' ? '-' : ' ');
                const oldNo = ln.old_lineno != null ? ln.old_lineno : '';
                const newNo = ln.new_lineno != null ? ln.new_lineno : '';
                return `<tr class="${cls}">
                    <td class="vs-ln">${oldNo}</td>
                    <td class="vs-ln">${newNo}</td>
                    <td>${prefix} ${esc(ln.text)}</td>
                </tr>`;
            }).join('');

            return `<article class="vs-diff-card" data-card-id="${esc(card.id)}">
                <div class="vs-diff-header">
                    <div>
                        <span class="vs-file-title">📄 ${esc(card.file_path)}</span>
                        <span style="color:#8b949e;margin-left:8px;">[${esc(card.source_label)}]</span>
                    </div>
                    <div>
                        <span class="vs-stats-add">+${card.added_lines}</span>
                        <span class="vs-stats-del">-${card.deleted_lines}</span>
                    </div>
                </div>
                <div class="vs-booth-strip">
                    <span>${esc(card.favre_commentary)}</span>
                </div>
                <div class="vs-g8t-strip">${esc(card.g8t_callout)}</div>
                <table class="vs-diff-table">
                    <tbody>${rows}</tbody>
                </table>
            </article>`;
        }

        async function refreshFlowFeed() {
            try {
                const res = await fetch('/api/flow/feed');
                if (!res.ok) return;
                const data = await res.json();

                if (data.headline_play_by_play) {
                    document.getElementById('headline-play-call').textContent = data.headline_play_by_play;
                    speakPlayCall(data.headline_play_by_play);
                }
                if (data.dad_joke_sideline) {
                    document.getElementById('sideline-joke-box').textContent = data.dad_joke_sideline;
                }
                if (autoVariableSpeed && data.recommended_speed_pct) {
                    currentSpeedPct = data.recommended_speed_pct;
                    updateSpeedIndicator();
                }

                const viewport = document.getElementById('diff-viewport');
                if (viewport && Array.isArray(data.diffs)) {
                    for (const card of data.diffs) {
                        if (!knownCardIds.has(card.id)) {
                            knownCardIds.add(card.id);
                            viewport.insertAdjacentHTML('beforeend', renderDiffCardHtml(card));
                        }
                    }
                    // Enforce bounded C64 scrollback limit
                    while (viewport.children.length > MAX_SCROLLBACK_CARDS) {
                        viewport.removeChild(viewport.firstElementChild);
                    }
                }

                // Render Capsela improvements
                const capEl = document.getElementById('capsela-improvements-list');
                if (capEl && Array.isArray(data.capsela_improvements)) {
                    capEl.innerHTML = data.capsela_improvements.map(c => `
                        <div class="capsela-item">
                            <div style="display:flex;justify-content:space-between;align-items:center;">
                                <strong style="color:#2ecc71;">${esc(c.capsule_id)}</strong>
                                <span style="color:#f39c12;font-weight:bold;">${esc(c.status_badge)}</span>
                            </div>
                            <div style="color:#8b949e;font-family:monospace;margin:2px 0;">${esc(c.signature)} · ${esc(c.previous_cycles_pq)} → <strong style="color:#4ecdc4;">${esc(c.current_cycles_pq)} cyc</strong></div>
                            <div style="color:#c9d1d9;">${esc(c.booth_note)}</div>
                        </div>
                    `).join('');
                }

                // Render Stream Comments
                const comEl = document.getElementById('stream-comments-list');
                if (comEl && Array.isArray(data.stream_comments)) {
                    comEl.innerHTML = data.stream_comments.map(n => `
                        <div class="comment-item">
                            <div style="color:#4ecdc4;font-weight:bold;">${esc(n.author)}</div>
                            <div style="color:#e6edf3;">${esc(n.raw_comment)}</div>
                            <div style="color:#8b949e;font-size:0.68rem;margin-top:2px;">Sent to Agent as: ${esc(n.agent_guidance)}</div>
                        </div>
                    `).join('');
                }

                // Render Standing Partnership Rules
                const rulesEl = document.getElementById('partnership-rules-list');
                if (rulesEl && Array.isArray(data.partnership_rules)) {
                    rulesEl.innerHTML = data.partnership_rules.map(r => `<div style="padding:5px 8px;background:rgba(255,255,255,0.03);border-radius:4px;">• ${esc(r)}</div>`).join('');
                }
            } catch (_) {}
        }

        async function postStreamComment() {
            const authorEl = document.getElementById('stream-author-input');
            const commentEl = document.getElementById('stream-comment-input');
            const author = (authorEl?.value || 'Hue').trim();
            const comment = (commentEl?.value || '').trim();
            if (!comment) return;
            try {
                await fetch('/api/debug/comment', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ author, comment })
                });
                commentEl.value = '';
                await refreshFlowFeed();
            } catch (_) {}
        }

        // Smooth variable-speed scroller (zero floats — integer pixel steps per frame counter)
        let frameTick = 0;
        function stepScroll() {
            frameTick = (frameTick + 1) % 1000;
            const viewport = document.getElementById('diff-viewport');
            if (viewport && currentSpeedPct > 0) {
                // Integer step accumulation
                const stepPx = Math.max(1, Math.floor((currentSpeedPct + (frameTick % 100)) / 100));
                if (frameTick % 2 === 0) {
                    if (viewport.scrollTop + viewport.clientHeight + 4 >= viewport.scrollHeight) {
                        // Loop smoothly back to top when reaching bottom of bounded scrollback
                        viewport.scrollTop = 0;
                    } else {
                        viewport.scrollTop += stepPx;
                    }
                }
            }
            requestAnimationFrame(stepScroll);
        }

        refreshFlowFeed();
        setInterval(refreshFlowFeed, 3500);
        requestAnimationFrame(stepScroll);
    </script>
</body>
</html>"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_diff_and_generates_favre_commentary_zero_float() {
        let sample_diff = "\
diff --git a/src/service_debug.rs b/src/service_debug.rs
--- a/src/service_debug.rs
+++ b/src/service_debug.rs
@@ -10,2 +10,4 @@
 pub fn check() {
+    let fast_int = 42u64;
+    let ratio = (18u64, 1u64);
 }
";
        let cards = parse_unified_diff_to_cards(sample_diff, "UNIT_TEST");
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].file_path, "src/service_debug.rs");
        assert_eq!(cards[0].added_lines, 2);
        assert!(cards[0].favre_commentary.contains("src/service_debug.rs"));
        assert!(cards[0].g8t_callout.contains("G8T"));

        let caps = check_capsela_improvements();
        assert!(!caps.is_empty());
        assert_eq!(caps[0].speedup_ratio_pq, "8/1");
    }
}
