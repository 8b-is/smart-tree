//! MagiSCanner — deep file security scanning, hash memory, and certificate trust.
//!
//! Ported from the standalone magiscanner project into Smart Tree's daemon-backed
//! security sentinel.

pub mod analyzers;
pub mod cert_neutralize;
pub mod config;
pub mod db;
pub mod dish;
pub mod finding;
pub mod http;
pub mod operation;
pub mod quarantine;
pub mod recipe;
pub mod scanner;
pub mod service;

pub use config::SecurityConfig;
pub use db::Database;
pub use finding::{Finding, ScanReport, Severity};
pub use scanner::Scanner;
