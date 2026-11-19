//! Startup courtesy for the Ollama emulator: if a real Ollama already answers on the
//! port Smart Tree wants, ask the person at the terminal what to do instead of failing
//! with a bind error.
//!
//! Rules this module keeps:
//! - It only ever asks in an interactive terminal, and the default answer is "leave
//!   everything as it is". Non-interactive starts never stop anything unless
//!   `--takeover always` was passed explicitly.
//! - It only stops something it has positively identified as Ollama. Anything else on
//!   the port (including another Smart Tree) is left alone.
//! - It says plainly what the swap means: the emulator answers with locally gathered
//!   project context and does not run a language model.
//! - Stopping is a polite request (service manager stop, or SIGTERM to the user's own
//!   `ollama` process). Nothing is force-killed.

use anyhow::{bail, Result};
use std::io::{BufRead, IsTerminal, Write};
use std::net::SocketAddr;
use std::process::Command;
use std::time::Duration;

/// What the user asked for on the command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TakeoverMode {
    /// Ask in an interactive terminal; otherwise behave like `Never`.
    Ask,
    /// Never stop Ollama.
    Never,
    /// Stop Ollama without asking.
    Always,
}

impl TakeoverMode {
    /// Parse `ask`, `never`/`no`, `always`/`yes`. Unknown words are an error.
    pub fn parse(word: &str) -> Result<Self> {
        match word.trim().to_ascii_lowercase().as_str() {
            "ask" | "prompt" | "" => Ok(Self::Ask),
            "never" | "no" | "off" => Ok(Self::Never),
            "always" | "yes" | "on" => Ok(Self::Always),
            other => bail!("unknown takeover mode `{other}` (expected ask, never or always)"),
        }
    }
}

/// Who is answering on the port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortOccupant {
    /// A real Ollama server, with the version it reported.
    RealOllama { version: String },
    /// A Smart Tree service (this emulator or the local inference service).
    SmartTree,
    /// Something that is not recognisably either.
    Unknown { detail: String },
}

/// Classify the body returned by `GET /api/version`.
///
/// Real Ollama answers `{"version":"<digits and dots>"}`. Smart Tree services answer
/// with their own markers. Anything else is `Unknown` and will not be touched.
pub fn classify_version_body(body: &str) -> PortOccupant {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(body) else {
        return PortOccupant::Unknown {
            detail: "it did not answer /api/version with JSON".to_string(),
        };
    };
    let version = value.get("version").and_then(|v| v.as_str()).unwrap_or("");
    let engine = value.get("engine").and_then(|v| v.as_str()).unwrap_or("");
    let smart_tree = version.contains("smart-tree")
        || version.starts_with("st-")
        || engine.contains("smart-tree")
        || value.get("inference_arithmetic").is_some();
    if smart_tree {
        return PortOccupant::SmartTree;
    }
    let looks_like_release = version.starts_with(|c: char| c.is_ascii_digit())
        && version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '+');
    if looks_like_release {
        PortOccupant::RealOllama {
            version: version.to_string(),
        }
    } else {
        PortOccupant::Unknown {
            detail: "its /api/version answer does not look like Ollama's".to_string(),
        }
    }
}

/// Model names from the body of `GET /api/ps` (the models Ollama has loaded now).
pub fn loaded_model_names(ps_body: &str) -> Vec<String> {
    serde_json::from_str::<serde_json::Value>(ps_body)
        .ok()
        .and_then(|v| v.get("models").and_then(|m| m.as_array()).cloned())
        .unwrap_or_default()
        .iter()
        .filter_map(|m| {
            m.get("name")
                .or_else(|| m.get("model"))
                .and_then(|n| n.as_str())
        })
        .map(str::to_string)
        .collect()
}

/// The person's answer to the prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TakeoverChoice {
    StopOllama,
    RunBeside,
    LeaveAlone,
}

/// Read an answer. Only an explicit `s`/`stop` stops anything; an empty line or
/// anything unrecognised leaves everything alone.
pub fn parse_choice(input: &str) -> TakeoverChoice {
    match input.trim().to_ascii_lowercase().as_str() {
        "s" | "stop" => TakeoverChoice::StopOllama,
        "p" | "port" | "beside" => TakeoverChoice::RunBeside,
        _ => TakeoverChoice::LeaveAlone,
    }
}

/// How Ollama can be asked to stop on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopMethod {
    /// `systemctl --user stop ollama`
    SystemdUser,
    /// `systemctl stop ollama` (may ask for authorization)
    SystemdSystem,
    /// Quit the macOS menu-bar app.
    MacApp,
    /// SIGTERM to `ollama` processes owned by the current user.
    TermOwnProcesses(Vec<u32>),
}

impl StopMethod {
    /// How to bring Ollama back after this kind of stop.
    pub fn restore_hint(&self) -> &'static str {
        match self {
            Self::SystemdUser => "systemctl --user start ollama",
            Self::SystemdSystem => "sudo systemctl start ollama",
            Self::MacApp => "open -a Ollama",
            Self::TermOwnProcesses(_) => "ollama serve",
        }
    }

    fn describe(&self) -> String {
        match self {
            Self::SystemdUser => "systemctl --user stop ollama".to_string(),
            Self::SystemdSystem => "systemctl stop ollama".to_string(),
            Self::MacApp => "asking the Ollama app to quit".to_string(),
            Self::TermOwnProcesses(pids) => format!("SIGTERM to your ollama process(es) {pids:?}"),
        }
    }
}

/// Pick the gentlest way to stop Ollama given what was observed. Service managers come
/// first, because a supervised process that is merely signalled gets restarted.
/// Returns `None` when Ollama is not something this user can stop.
pub fn choose_stop_method(
    user_unit_active: bool,
    system_unit_active: bool,
    is_macos: bool,
    own_ollama_pids: &[u32],
) -> Option<StopMethod> {
    if user_unit_active {
        Some(StopMethod::SystemdUser)
    } else if system_unit_active {
        Some(StopMethod::SystemdSystem)
    } else if is_macos {
        Some(StopMethod::MacApp)
    } else if !own_ollama_pids.is_empty() {
        Some(StopMethod::TermOwnProcesses(own_ollama_pids.to_vec()))
    } else {
        None
    }
}

/// The text shown before asking.
pub fn takeover_prompt_text(
    addr: SocketAddr,
    version: &str,
    loaded: &[String],
    beside_port: Option<u16>,
    restore_hint: Option<&str>,
) -> String {
    let loaded_line = match loaded.len() {
        0 => "no models loaded right now".to_string(),
        1 => format!("1 model loaded: {}", loaded[0]),
        n => format!("{n} models loaded: {}", loaded.join(", ")),
    };
    let mut text = format!(
        "\n🦙 Ollama {version} is already home on {addr} ({loaded_line}).\n\n\
         Smart Tree can take that port, but here is the honest trade:\n\
         \x20 • Smart Tree's emulator answers with project context it gathers locally.\n\
         \x20   It does not run a language model, so apps on this port would stop\n\
         \x20   getting model-written replies.\n\
         \x20 • Stopping Ollama unloads its models"
    );
    match restore_hint {
        Some(hint) => text.push_str(&format!(". To bring it back: {hint}\n\n")),
        None => text.push_str(".\n\n"),
    }
    text.push_str(&format!(
        "  [s] Stop Ollama and give port {} to Smart Tree\n",
        addr.port()
    ));
    if let Some(port) = beside_port {
        text.push_str(&format!(
            "  [p] Keep Ollama; run Smart Tree beside it on port {port}\n"
        ));
    }
    text.push_str("  [N] Leave everything as it is\n\n");
    text.push_str(if beside_port.is_some() {
        "Your call [s/p/N]: "
    } else {
        "Your call [s/N]: "
    });
    text
}

fn port_answers(addr: SocketAddr) -> bool {
    std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok()
}

fn http_get(addr: SocketAddr, path: &str) -> Option<String> {
    let url = format!("http://{addr}{path}");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .ok()?;
    client.get(url).send().ok()?.text().ok()
}

fn command_succeeds(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// PIDs of processes named exactly `ollama` owned by the current user.
fn own_ollama_pids() -> Vec<u32> {
    // `id -u` avoids an FFI call just to learn our own uid.
    let uid = Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    if uid.is_empty() {
        return Vec::new();
    }
    Command::new("pgrep")
        .args(["-x", "-u", &uid, "ollama"])
        .output()
        .ok()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter_map(|l| l.trim().parse::<u32>().ok())
                .collect()
        })
        .unwrap_or_default()
}

fn detect_stop_method() -> Option<StopMethod> {
    let user_unit = command_succeeds("systemctl", &["--user", "is-active", "--quiet", "ollama"]);
    let system_unit = command_succeeds("systemctl", &["is-active", "--quiet", "ollama"]);
    choose_stop_method(
        user_unit,
        system_unit,
        cfg!(target_os = "macos"),
        &own_ollama_pids(),
    )
}

fn run_stop(method: &StopMethod) -> bool {
    match method {
        StopMethod::SystemdUser => command_succeeds("systemctl", &["--user", "stop", "ollama"]),
        StopMethod::SystemdSystem => command_succeeds("systemctl", &["stop", "ollama"]),
        StopMethod::MacApp => command_succeeds("osascript", &["-e", "quit app \"Ollama\""]),
        StopMethod::TermOwnProcesses(pids) => pids
            .iter()
            .all(|pid| command_succeeds("kill", &["-TERM", &pid.to_string()])),
    }
}

fn wait_until_free(addr: SocketAddr, attempts: u32) -> bool {
    for _ in 0..attempts {
        if !port_answers(addr) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    !port_answers(addr)
}

fn next_free_port(addr: SocketAddr) -> Option<u16> {
    (1..=20u16)
        .filter_map(|step| addr.port().checked_add(step))
        .find(|port| !port_answers(SocketAddr::new(addr.ip(), *port)))
}

/// Decide where the emulator should listen.
///
/// Returns `Ok(Some(addr))` to serve on `addr` (the requested one, or a neighbouring
/// port the person chose), `Ok(None)` when the person chose to leave things alone or
/// Smart Tree is already serving there, and an error when the port is held by
/// something this function will not touch.
///
/// This blocks on a terminal prompt and short local HTTP calls; call it from a
/// blocking context.
pub fn resolve_listen_addr(
    requested: SocketAddr,
    mode: TakeoverMode,
) -> Result<Option<SocketAddr>> {
    if !port_answers(requested) {
        return Ok(Some(requested));
    }
    let occupant = http_get(requested, "/api/version")
        .map(|body| classify_version_body(&body))
        .unwrap_or_else(|| PortOccupant::Unknown {
            detail: "it did not answer /api/version".to_string(),
        });

    let version = match occupant {
        PortOccupant::SmartTree => {
            eprintln!(
                "🌳 Smart Tree is already answering on {requested}. Nothing to start; use --port to run a second one."
            );
            return Ok(None);
        }
        PortOccupant::Unknown { detail } => bail!(
            "{requested} is in use and {detail}, so Smart Tree is leaving it alone. Pick another port with --port."
        ),
        PortOccupant::RealOllama { version } => version,
    };

    let interactive = std::io::stdin().is_terminal() && std::io::stderr().is_terminal();
    let method = detect_stop_method();

    let choice = match mode {
        TakeoverMode::Always => TakeoverChoice::StopOllama,
        TakeoverMode::Ask if interactive => {
            let loaded = http_get(requested, "/api/ps")
                .map(|b| loaded_model_names(&b))
                .unwrap_or_default();
            let beside = next_free_port(requested);
            let text = takeover_prompt_text(
                requested,
                &version,
                &loaded,
                beside,
                method.as_ref().map(StopMethod::restore_hint),
            );
            let mut err = std::io::stderr();
            let _ = err.write_all(text.as_bytes());
            let _ = err.flush();
            let mut line = String::new();
            let _ = std::io::stdin().lock().read_line(&mut line);
            let picked = parse_choice(&line);
            if picked == TakeoverChoice::RunBeside && beside.is_none() {
                TakeoverChoice::LeaveAlone
            } else {
                picked
            }
        }
        TakeoverMode::Ask | TakeoverMode::Never => bail!(
            "Ollama {version} is using {requested}. Smart Tree did not stop it. \
             Run in a terminal to be asked, pass --takeover always, or choose another port with --port."
        ),
    };

    match choice {
        TakeoverChoice::LeaveAlone => {
            eprintln!("👍 Leaving Ollama as it is. Smart Tree did not start.");
            Ok(None)
        }
        TakeoverChoice::RunBeside => match next_free_port(requested) {
            Some(port) => {
                let addr = SocketAddr::new(requested.ip(), port);
                eprintln!("🤝 Ollama keeps {requested}; Smart Tree will listen on {addr}.");
                Ok(Some(addr))
            }
            None => bail!("no free port found next to {requested}"),
        },
        TakeoverChoice::StopOllama => {
            let Some(method) = method else {
                bail!(
                    "Ollama {version} is using {requested}, but it is not running as your user or under a service Smart Tree can stop. Stop it yourself, then start Smart Tree again."
                );
            };
            eprintln!("⏳ Stopping Ollama ({})…", method.describe());
            let asked = run_stop(&method);
            if asked && wait_until_free(requested, 24) {
                eprintln!(
                    "✅ Ollama stopped. To bring it back later: {}",
                    method.restore_hint()
                );
                Ok(Some(requested))
            } else {
                bail!(
                    "Ollama is still answering on {requested} after `{}`. Nothing was force-killed. Stop it yourself, or choose another port with --port.",
                    method.describe()
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_ollama_is_recognised_and_smart_tree_is_never_mistaken_for_it() {
        assert_eq!(
            classify_version_body(r#"{"version":"0.5.7"}"#),
            PortOccupant::RealOllama {
                version: "0.5.7".to_string()
            }
        );
        assert_eq!(
            classify_version_body(r#"{"version":"0.12.3-rc1"}"#),
            PortOccupant::RealOllama {
                version: "0.12.3-rc1".to_string()
            }
        );
        // This emulator's own answer.
        assert_eq!(
            classify_version_body(
                r#"{"version":"st-sovereign-11·0·0","engine":"smart-tree-mem8-context","float_count":0}"#
            ),
            PortOccupant::SmartTree
        );
        // The local inference service's answer.
        assert_eq!(
            classify_version_body(
                r#"{"inference_arithmetic":"backend-defined; not an integer-only claim","version":"smart-tree-local-inference"}"#
            ),
            PortOccupant::SmartTree
        );
        // Something with an Ollama-shaped version but a Smart Tree engine marker.
        assert_eq!(
            classify_version_body(r#"{"version":"1.0.0","engine":"smart-tree-x"}"#),
            PortOccupant::SmartTree
        );
    }

    #[test]
    fn anything_unrecognised_is_unknown_and_so_left_alone() {
        for body in [
            "",
            "<html>nginx</html>",
            "{}",
            r#"{"version":""}"#,
            r#"{"version":"latest"}"#,
            r#"{"version":"1.0 beta"}"#,
            r#"{"version":7}"#,
        ] {
            assert!(
                matches!(classify_version_body(body), PortOccupant::Unknown { .. }),
                "{body:?} should be Unknown"
            );
        }
    }

    #[test]
    fn only_an_explicit_stop_answer_stops_anything() {
        for yes in ["s", "S", " stop ", "STOP"] {
            assert_eq!(parse_choice(yes), TakeoverChoice::StopOllama);
        }
        for beside in ["p", "port", "Beside"] {
            assert_eq!(parse_choice(beside), TakeoverChoice::RunBeside);
        }
        // Enter, "y", "yes", typos and noise all leave Ollama alone: stopping a model
        // server should never happen by reflex.
        for no in [
            "", "\n", "n", "N", "no", "y", "yes", "sure", "sp", "stopp", "1",
        ] {
            assert_eq!(parse_choice(no), TakeoverChoice::LeaveAlone, "{no:?}");
        }
    }

    #[test]
    fn takeover_mode_words() {
        assert_eq!(TakeoverMode::parse("ask").unwrap(), TakeoverMode::Ask);
        assert_eq!(TakeoverMode::parse("Never").unwrap(), TakeoverMode::Never);
        assert_eq!(TakeoverMode::parse("no").unwrap(), TakeoverMode::Never);
        assert_eq!(TakeoverMode::parse("always").unwrap(), TakeoverMode::Always);
        assert_eq!(TakeoverMode::parse("yes").unwrap(), TakeoverMode::Always);
        assert!(TakeoverMode::parse("maybe").is_err());
    }

    #[test]
    fn service_managers_are_preferred_over_signals() {
        assert_eq!(
            choose_stop_method(true, true, false, &[10]),
            Some(StopMethod::SystemdUser)
        );
        assert_eq!(
            choose_stop_method(false, true, false, &[10]),
            Some(StopMethod::SystemdSystem)
        );
        assert_eq!(
            choose_stop_method(false, false, true, &[10]),
            Some(StopMethod::MacApp)
        );
        assert_eq!(
            choose_stop_method(false, false, false, &[10, 11]),
            Some(StopMethod::TermOwnProcesses(vec![10, 11]))
        );
        // Not ours and not under a service we can see: nothing we may stop.
        assert_eq!(choose_stop_method(false, false, false, &[]), None);
    }

    #[test]
    fn loaded_models_are_listed_from_api_ps() {
        let body =
            r#"{"models":[{"name":"llama3:8b","size":1},{"model":"qwen2.5:7b"},{"size":3}]}"#;
        assert_eq!(
            loaded_model_names(body),
            vec!["llama3:8b".to_string(), "qwen2.5:7b".to_string()]
        );
        assert!(loaded_model_names(r#"{"models":[]}"#).is_empty());
        assert!(loaded_model_names(r#"{"error":"nope"}"#).is_empty());
        assert!(loaded_model_names("not json").is_empty());
    }

    #[test]
    fn prompt_states_the_trade_and_defaults_to_no() {
        let addr: SocketAddr = "127.0.0.1:11434".parse().unwrap();
        let text = takeover_prompt_text(
            addr,
            "0.5.7",
            &["llama3:8b".to_string(), "qwen2.5:7b".to_string()],
            Some(11435),
            Some("sudo systemctl start ollama"),
        );
        assert!(text.contains("Ollama 0.5.7"));
        assert!(text.contains("2 models loaded: llama3:8b, qwen2.5:7b"));
        assert!(text.contains("does not run a language model"));
        assert!(text.contains("To bring it back: sudo systemctl start ollama"));
        assert!(text.contains("[p] Keep Ollama; run Smart Tree beside it on port 11435"));
        assert!(text.ends_with("Your call [s/p/N]: "));

        let bare = takeover_prompt_text(addr, "0.5.7", &[], None, None);
        assert!(bare.contains("no models loaded right now"));
        assert!(!bare.contains("[p]"));
        assert!(bare.ends_with("Your call [s/N]: "));
    }

    #[test]
    fn a_free_port_needs_no_question() {
        // Bind to learn a free port, release it, then resolve: no prompt, same address.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        assert_eq!(
            resolve_listen_addr(addr, TakeoverMode::Ask).unwrap(),
            Some(addr)
        );
    }

    /// A tiny HTTP server that answers every request with `body`.
    fn serve_version(body: &'static str) -> SocketAddr {
        use std::io::Read;
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { break };
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf);
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });
        addr
    }

    #[test]
    fn an_existing_smart_tree_is_reported_not_replaced() {
        let addr = serve_version(r#"{"version":"smart-tree-local-inference"}"#);
        assert_eq!(
            resolve_listen_addr(addr, TakeoverMode::Always).unwrap(),
            None
        );
    }

    #[test]
    fn an_unknown_service_is_left_alone_even_with_takeover_always() {
        let addr = serve_version(r#"{"hello":"i am something else"}"#);
        let err = resolve_listen_addr(addr, TakeoverMode::Always).unwrap_err();
        assert!(err.to_string().contains("leaving it alone"), "{err}");
    }

    #[test]
    fn a_real_ollama_is_not_stopped_without_a_terminal_or_explicit_flag() {
        // Tests run without an interactive stdin, so `Ask` must refuse, like `Never`.
        let addr = serve_version(r#"{"version":"0.5.7"}"#);
        for mode in [TakeoverMode::Ask, TakeoverMode::Never] {
            let err = resolve_listen_addr(addr, mode).unwrap_err();
            assert!(err.to_string().contains("did not stop it"), "{err}");
        }
        // The fake server is still up: nothing was signalled.
        assert!(port_answers(addr));
    }
}
