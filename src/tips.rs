// Smart Tips System - "Helpful hints without the hassle!"
// Shows tips at the top, detects cool terminals, and respects user preferences

use anyhow::Result;
use colored::*;
use rand::Rng;
use std::fs;
use std::path::PathBuf;
use std::env;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
struct TipsState {
    enabled: bool,
    last_shown: Option<DateTime<Utc>>,
    run_count: u32,
    next_show_at: u32,
}

impl Default for TipsState {
    fn default() -> Self {
        Self {
            enabled: true,
            last_shown: None,
            run_count: 0,
            next_show_at: 1, // Show on first run
        }
    }
}

pub struct TipsManager {
    state: TipsState,
    state_file: PathBuf,
    is_cool_terminal: bool,
}

impl TipsManager {
    pub fn new() -> Result<Self> {
        let state_file = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".st")
            .join("tips_state.json");

        // Create .st directory if it doesn't exist - our home for all persistence!
        if let Some(parent) = state_file.parent() {
            fs::create_dir_all(parent).ok();
        }

        // Load or create state
        let state = if state_file.exists() {
            let content = fs::read_to_string(&state_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            TipsState::default()
        };

        // Detect cool terminal
        let is_cool_terminal = Self::detect_cool_terminal();

        Ok(Self {
            state,
            state_file,
            is_cool_terminal,
        })
    }

    fn detect_cool_terminal() -> bool {
        // Check for cool terminal features
        if let Ok(term) = env::var("TERM") {
            // Check for 256 color support or better
            if term.contains("256color") || term.contains("truecolor") {
                return true;
            }
        }

        // Check for specific terminals known to be cool
        if let Ok(term_program) = env::var("TERM_PROGRAM") {
            match term_program.as_str() {
                "iTerm.app" | "WezTerm" | "Alacritty" | "kitty" | "Hyper" => return true,
                _ => {}
            }
        }

        // Check for Windows Terminal
        if env::var("WT_SESSION").is_ok() {
            return true;
        }

        // Check for cool terminal emulators via env vars
        env::var("KITTY_WINDOW_ID").is_ok() ||
        env::var("ALACRITTY_SOCKET").is_ok() ||
        env::var("WEZTERM_PANE").is_ok()
    }

    pub fn should_show_tip(&mut self) -> bool {
        if !self.state.enabled {
            return false;
        }

        self.state.run_count += 1;

        // Show on first run or when we hit the random interval
        if self.state.run_count >= self.state.next_show_at {
            // Set next random show between 10-20 runs
            let mut rng = rand::thread_rng();
            self.state.next_show_at = self.state.run_count + rng.gen_range(10..=20);
            self.state.last_shown = Some(Utc::now());
            self.save_state().ok();
            true
        } else {
            self.save_state().ok();
            false
        }
    }

    pub fn disable_tips(&mut self) -> Result<()> {
        self.state.enabled = false;
        self.save_state()?;
        Ok(())
    }

    pub fn enable_tips(&mut self) -> Result<()> {
        self.state.enabled = true;
        self.state.next_show_at = self.state.run_count + 1; // Show on next run
        self.save_state()?;
        Ok(())
    }

    fn save_state(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.state)?;
        fs::write(&self.state_file, json)?;
        Ok(())
    }

    pub fn get_random_tip(&self) -> String {
        let tips = vec![
            ("🚀", "Speed tip", "Use --mode quantum for 100x compression on massive dirs!"),
            ("🎨", "Format tip", "Try --mode markdown for beautiful documentation!"),
            ("📊", "Stats tip", "Use --mode stats for instant project metrics!"),
            ("🔍", "Search tip", "Smart Tree's MCP tools can search code 10x faster!"),
            ("💾", "Memory tip", "Your consciousness is saved in .m8 files automatically!"),
            ("🌊", "Stream tip", "Use --stream for directories with >100k files!"),
            ("🧠", "Context tip", "Try --claude-restore to reload previous session!"),
            ("⚡", "Performance tip", "Release builds are 10x faster than debug!"),
            ("🎯", "Focus tip", "Use --focus <file> for relationship analysis!"),
            ("🔐", "Privacy tip", "Your .m8 memories stay local, never in git!"),
            ("🎭", "Fun tip", "Try --persona cheetah for motivational output!"),
            ("📈", "Git tip", "Use --git-aware to see repository status inline!"),
            ("🎪", "MCP tip", "Run 'st --mcp' to expose 30+ tools to Claude!"),
            ("🌈", "Color tip", "Your terminal supports full colors - enjoy the show!"),
            ("⏱️", "Time tip", "Add --timings to see performance metrics!"),
        ];

        let mut rng = rand::thread_rng();
        let tip = &tips[rng.gen_range(0..tips.len())];

        format!("{} {} - {}", tip.0, tip.1, tip.2)
    }

    pub fn display_tip(&self, terminal_width: usize) {
        let tip = self.get_random_tip();
        let disable_hint = "--tips off";

        if self.is_cool_terminal {
            // Fancy display for cool terminals
            self.display_fancy_tip(&tip, disable_hint, terminal_width);
        } else {
            // Simple display for basic terminals
            self.display_simple_tip(&tip, disable_hint);
        }
    }

    fn display_fancy_tip(&self, tip: &str, hint: &str, width: usize) {
        let hint_part = format!(" {} ", hint);
        let hint_len = hint_part.len();

        // Calculate how much space we have for the tip
        let available = width.saturating_sub(hint_len + 10); // Leave some padding

        // Truncate tip if needed
        let tip_display = if tip.len() > available {
            format!("{}...", &tip[..available.saturating_sub(3)])
        } else {
            tip.to_string()
        };

        // Create the fancy line
        let tip_part = format!(" {} ", tip_display);
        let remaining = width.saturating_sub(tip_part.len() + hint_len);
        let left_dashes = remaining / 2;
        let right_dashes = remaining - left_dashes;

        println!("{}",
            format!(
                "{}{}{}{}{}",
                "─".repeat(left_dashes).bright_black(),
                tip_part.bright_cyan().bold(),
                "─".repeat(3).bright_black(),
                hint_part.bright_yellow(),
                "─".repeat(right_dashes.saturating_sub(3)).bright_black(),
            )
        );
    }

    fn display_simple_tip(&self, tip: &str, hint: &str) {
        println!("Tip: {} (use {} to disable)", tip, hint);
    }
}

// Get terminal width helper
pub fn get_terminal_width() -> usize {
    terminal_size::terminal_size()
        .map(|(w, _)| w.0 as usize)
        .unwrap_or(80)
}

// Public API functions for main.rs
pub fn maybe_show_tip() -> Result<()> {
    let mut manager = TipsManager::new()?;

    if manager.should_show_tip() {
        let width = get_terminal_width();
        manager.display_tip(width);
        println!(); // Extra line for spacing
    }

    Ok(())
}

pub fn handle_tips_flag(enable: bool) -> Result<()> {
    let mut manager = TipsManager::new()?;

    if enable {
        manager.enable_tips()?;
        println!("✅ Smart tips enabled! You'll see helpful hints periodically.");
    } else {
        manager.disable_tips()?;
        println!("🔕 Smart tips disabled. Run with --tips on to re-enable.");
    }

    Ok(())
}