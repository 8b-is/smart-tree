//! Live daemon integration test
//!
//! Starts the daemon, sends protocol frames, validates responses.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

use st_protocol::{Frame, Verb};

/// Get the socket path
fn socket_path() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
        .join("st.sock")
}

/// Start the daemon process
fn start_daemon() -> Child {
    let daemon_path = std::env::current_dir()
        .unwrap()
        .join("target/debug/std");

    Command::new(daemon_path)
        .arg("start")
        .spawn()
        .expect("Failed to start daemon")
}

/// Stop the daemon
fn stop_daemon() {
    let daemon_path = std::env::current_dir()
        .unwrap()
        .join("target/debug/std");

    let _ = Command::new(daemon_path)
        .arg("stop")
        .status();
}

/// Wait for socket to be available
fn wait_for_socket() -> bool {
    for _ in 0..50 {
        if socket_path().exists() {
            // Also try to connect
            if UnixStream::connect(socket_path()).is_ok() {
                return true;
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    false
}

#[test]
#[ignore] // Run with: cargo test test_daemon_ping -- --ignored --nocapture
fn test_daemon_ping() {
    // Ensure clean state
    stop_daemon();
    thread::sleep(Duration::from_millis(200));

    // Start daemon
    let mut daemon = start_daemon();

    // Wait for socket
    assert!(wait_for_socket(), "Daemon socket not available");

    // Connect and send PING
    let mut stream = UnixStream::connect(socket_path())
        .expect("Failed to connect to daemon");

    let ping = Frame::ping();
    stream.write_all(&ping.encode()).expect("Failed to send PING");

    // Read response
    let mut buf = [0u8; 256];
    let n = stream.read(&mut buf).expect("Failed to read response");

    // Parse response
    let response = Frame::decode(&buf[..n]).expect("Failed to decode response");
    assert_eq!(response.verb(), Verb::Ok, "Expected OK response to PING");

    // Clean up
    drop(stream);
    stop_daemon();
    let _ = daemon.wait();

    println!("PING test passed!");
}

#[test]
#[ignore] // Run with: cargo test test_daemon_scan -- --ignored --nocapture
fn test_daemon_scan() {
    // Ensure clean state
    stop_daemon();
    thread::sleep(Duration::from_millis(200));

    // Start daemon
    let mut daemon = start_daemon();

    // Wait for socket
    assert!(wait_for_socket(), "Daemon socket not available");

    // Connect and send SCAN
    let mut stream = UnixStream::connect(socket_path())
        .expect("Failed to connect to daemon");

    let scan = Frame::scan("/tmp", 1);
    stream.write_all(&scan.encode()).expect("Failed to send SCAN");

    // Read response
    let mut buf = vec![0u8; 65536];
    let n = stream.read(&mut buf).expect("Failed to read response");

    // Parse response
    let response = Frame::decode(&buf[..n]).expect("Failed to decode response");
    assert_eq!(response.verb(), Verb::Ok, "Expected OK response to SCAN");

    // Check we got JSON back
    let json_str = response.payload().as_str().expect("Expected string payload");
    assert!(json_str.contains("files"), "Response should contain file count");
    assert!(json_str.contains("dirs"), "Response should contain dir count");

    println!("SCAN response: {} bytes", json_str.len());

    // Clean up
    drop(stream);
    stop_daemon();
    let _ = daemon.wait();

    println!("SCAN test passed!");
}

#[test]
#[ignore] // Run with: cargo test test_daemon_format -- --ignored --nocapture
fn test_daemon_format() {
    // Ensure clean state
    stop_daemon();
    thread::sleep(Duration::from_millis(200));

    // Start daemon
    let mut daemon = start_daemon();

    // Wait for socket
    assert!(wait_for_socket(), "Daemon socket not available");

    // Connect and send FORMAT
    let mut stream = UnixStream::connect(socket_path())
        .expect("Failed to connect to daemon");

    // FORMAT with classic mode on /tmp
    let format = Frame::format_path("classic", "/tmp", 1);
    stream.write_all(&format.encode()).expect("Failed to send FORMAT");

    // Read response
    let mut buf = vec![0u8; 65536];
    let n = stream.read(&mut buf).expect("Failed to read response");

    // Parse response
    let response = Frame::decode(&buf[..n]).expect("Failed to decode response");
    if response.verb() == Verb::Error {
        let err_msg = response.payload().as_str().unwrap_or("unknown error");
        panic!("FORMAT returned error: {}", err_msg);
    }
    assert_eq!(response.verb(), Verb::Ok, "Expected OK response to FORMAT");

    // Check we got formatted output (not empty)
    let output = response.payload().as_str().expect("Expected string payload");
    assert!(!output.is_empty(), "Response should not be empty");

    println!("FORMAT response ({} bytes):\n{}", output.len(), &output[..output.len().min(1000)]);

    // Clean up
    drop(stream);
    stop_daemon();
    let _ = daemon.wait();

    println!("\nFORMAT test passed!");
}

#[test]
#[ignore] // Run with: cargo test test_daemon_format_json -- --ignored --nocapture
fn test_daemon_format_json() {
    // Ensure clean state
    stop_daemon();
    thread::sleep(Duration::from_millis(200));

    // Start daemon
    let mut daemon = start_daemon();

    // Wait for socket
    assert!(wait_for_socket(), "Daemon socket not available");

    // Connect and send FORMAT with JSON mode
    let mut stream = UnixStream::connect(socket_path())
        .expect("Failed to connect to daemon");

    let format = Frame::format_path("json", "/tmp", 1);
    stream.write_all(&format.encode()).expect("Failed to send FORMAT");

    // Read response
    let mut buf = vec![0u8; 65536];
    let n = stream.read(&mut buf).expect("Failed to read response");

    // Parse response
    let response = Frame::decode(&buf[..n]).expect("Failed to decode response");
    assert_eq!(response.verb(), Verb::Ok, "Expected OK response to FORMAT");

    // Check we got valid JSON
    let output = response.payload().as_str().expect("Expected string payload");
    assert!(output.starts_with("[") || output.starts_with("{"),
            "JSON format should start with [ or {{");

    println!("JSON FORMAT response ({} bytes)", output.len());

    // Clean up
    drop(stream);
    stop_daemon();
    let _ = daemon.wait();

    println!("JSON FORMAT test passed!");
}

#[test]
#[ignore] // Run with: cargo test test_daemon_search -- --ignored --nocapture
fn test_daemon_search() {
    // Ensure clean state
    stop_daemon();
    thread::sleep(Duration::from_millis(200));

    // Start daemon
    let mut daemon = start_daemon();

    // Wait for socket
    assert!(wait_for_socket(), "Daemon socket not available");

    // Connect and send SEARCH
    let mut stream = UnixStream::connect(socket_path())
        .expect("Failed to connect to daemon");

    // Search for common text in /tmp
    let search = Frame::search_path("/tmp", "tmp", 10);
    stream.write_all(&search.encode()).expect("Failed to send SEARCH");

    // Set read timeout since search can take time
    stream.set_read_timeout(Some(Duration::from_secs(10))).expect("Failed to set timeout");

    // Read response
    let mut buf = vec![0u8; 65536];
    let n = stream.read(&mut buf).expect("Failed to read response");

    // Parse response
    let response = Frame::decode(&buf[..n]).expect("Failed to decode response");
    if response.verb() == Verb::Error {
        let err_msg = response.payload().as_str().unwrap_or("unknown error");
        panic!("SEARCH returned error: {}", err_msg);
    }
    assert_eq!(response.verb(), Verb::Ok, "Expected OK response to SEARCH");

    // Check we got JSON results
    let output = response.payload().as_str().expect("Expected string payload");
    assert!(output.contains("pattern"), "Response should contain pattern");
    assert!(output.contains("results"), "Response should contain results");

    println!("SEARCH response ({} bytes):\n{}", output.len(), &output[..output.len().min(1000)]);

    // Clean up
    drop(stream);
    stop_daemon();
    let _ = daemon.wait();

    println!("\nSEARCH test passed!");
}
