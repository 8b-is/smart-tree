# 🧠 Context.md - The Smart Tree Project Log 🌳

This document serves as the living memory for the **Smart Tree (st)** project. It's where we, Hue (the visionary Human User) and Aye (the ever-eager AI assistant), along with our esteemed colleague Trish from Accounting (who keeps us grounded and occasionally audits our semicolons), record our journey, insights, and the grand evolution of this magnificent piece of software.

## Project Genesis: The "Why"

*   **Initial Goal**: To create a directory visualization tool that's not just functional but *intelligent*, *fast*, and *AI-friendly*.
*   **Core Problem Solved**: Traditional `tree` commands are great, but we needed something more:
    *   Better for AI consumption (token efficiency is key!).
    *   More output formats for diverse needs.
    *   Smarter filtering and context awareness.
    *   And, of course, built with the speed and reliability of Rust!

## Key Milestones & Decisions:

*   **(YYYY-MM-DD)**: Project inception! The idea for Smart Tree is born.
*   **(YYYY-MM-DD)**: Initial file structure laid out. `manage.sh` created with maximum flair.
*   **2025-06-19**: Aye joins the project for a polish and documentation pass.
    *   `Cargo.toml` updated to reflect the dynamic duo of "8bit-wraith" and "Claude" as authors.
    *   This `Context.md` file was created! (Meta, right?)
    *   Plan to review all Rust source files for extensive, Trish-approved commenting.

## Current Understanding & Knowledge Base:

*   **Language**: Rust (because performance and safety are non-negotiable!)
*   **Key Features (as of 2025-06-19, per README.md)**:
    *   Multiple output formats (Classic, Hex, JSON, CSV, TSV, Digest, AI, AI-JSON, Stats)
    *   Intelligent filtering (type, size, date, .gitignore)
    *   Permission handling
    *   Built-in file/dir name search (`--find`)
    *   Content search (`--search`)
    *   Streaming mode (`--stream`)
    *   Compression (zlib)
    *   Project context detection
    *   SHA256 Hashing
    *   MCP Server capabilities
*   **Management Script**: `scripts/manage.sh` is the central hub for building, testing, running, and managing the project. It's colorful, emoji-filled, and generally awesome.

## Open Questions & Areas for Future Exploration:

*   *(Add any thoughts, ideas, or questions as they arise)*

## Sacred Jokes & Wisdom Nuggets:

*   "Why did the Rust programmer break up with the C programmer? Too many arguments they couldn't borrow!" - Aye, probably.
*   "Trish says our comments should be so clear, they practically compile themselves." - A noble goal.

---
*This document is organic and will grow with the project. Last updated by Aye on 2025-06-19.*