// lib.rs - Main library file

pub mod user_service;
pub mod auth_handler;

pub use user_service::{UserService, User};
pub use auth_handler::AuthHandler;