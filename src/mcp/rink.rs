//! Opt-in process boundary to Rink. No Rink code or unit definitions are linked.
use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt};

#[cfg(test)]
#[path = "rink_tests.rs"]
mod tests;

const MAX_EXPRESSION: usize = 512;
const MAX_OUTPUT: u64 = 16 * 1024;
const WALL_LIMIT: Duration = Duration::from_secs(3);
const MAX_STORE: u64 = 16 * 1024 * 1024;
static QUERIES: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(2);
fn config_text(interpret: bool) -> String {
    format!(
        "[rink]\nshow_interpretation = {}\nlong_output = false\n[currency]\nenabled = false\n[colors]\nenabled = false\n[limits]\nenabled = false\n",
        if interpret { "true" } else { "false" }
    )
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Request {
    operation: Option<String>,
    expression: Option<String>,
    frequency: Option<String>,
    other_frequency: Option<String>,
    speed: Option<String>,
    remember: bool,
    memory_id: Option<String>,
    interpret: bool,
}

/// The same schema is used by every MCP discovery path.
pub fn definition() -> Value {
    json!({
        "name": "rink",
        "description": "Offline, opt-in external Rink quantities and classical wave math. Operations: query (expression), period (frequency), wavelength (speed/frequency), beat (frequency/other_frequency), recall (memory_id). Requires operator ST_RINK_BIN absolute path and Linux /usr/bin/prlimit. Fixed CPU/memory/time/output limits; no shell or currency fetch. Results retain Rink text, not a universal exactness guarantee (Rink has float fallbacks). remember:true explicitly stores provenance using Smart Tree native MEM8 in ST_RINK_MEMORY_DIR. No quantum hardware claim.",
        "inputSchema": {
            "type": "object", "additionalProperties": false,
            "properties": {
                "operation": {"type": "string", "enum": ["query", "period", "wavelength", "beat", "recall"], "default": "query"},
                "expression": {"type": "string", "maxLength": MAX_EXPRESSION, "description": "Rink expression, e.g. (1/3) meter to fraction cm, 1640 g to lb;oz, or #jan 01, 1970# -> UTC. Semicolon, hash, comma, and colon are Rink syntax, not shell commands."},
                "frequency": {"type": "string", "maxLength": 128, "description": "Strictly positive ordinary cycle frequency: integer or p/q plus Hz, kHz, MHz, GHz, mHz, uHz or nHz; e.g. 432 Hz or (1/3) kHz. Not rad/s; angular omega = 2*pi*f."},
                "other_frequency": {"type": "string", "maxLength": 128},
                "speed": {"type": "string", "maxLength": 128, "description": "Propagation speed, e.g. 343 m/s"},
                "remember": {"type": "boolean", "default": false, "description": "Explicitly consent to durable provenance storage; requires ST_RINK_MEMORY_DIR"},
                "interpret": {"type": "boolean", "default": false, "description": "Show Rink's input interpretation. Default false keeps the answer compact."},
                "memory_id": {"type": "string", "description": "UUID from a prior remembered result"}
            }
        }
    })
}

fn validate(expression: &str) -> Result<()> {
    ensure!(!expression.trim().is_empty(), "Empty Rink expression");
    ensure!(
        expression.len() <= MAX_EXPRESSION,
        "Rink expression exceeds 512 bytes"
    );
    let mut depth = 0u8;
    let mut digits = 0u8;
    for byte in expression.bytes() {
        ensure!(
            byte.is_ascii_alphanumeric() || b" +-*/^().>;#,:_".contains(&byte),
            "Only bounded ASCII quantity arithmetic is supported"
        );
        if byte.is_ascii_digit() {
            digits += 1;
            ensure!(
                digits <= 18,
                "Numeric literals are limited to 18 consecutive digits"
            );
        } else {
            digits = 0;
        }
        match byte {
            b'(' => {
                depth += 1;
                ensure!(depth <= 16, "Rink nesting exceeds 16");
            }
            b')' => {
                ensure!(depth > 0, "Unbalanced parentheses");
                depth -= 1;
            }
            _ => {}
        }
    }
    ensure!(depth == 0, "Unbalanced parentheses");
    // Resource limits remain authoritative; this also rejects obvious exponent bombs.
    for tail in expression
        .match_indices('^')
        .map(|(i, _)| &expression[i + 1..])
    {
        let tail = tail.trim_start().trim_start_matches(['+', '-']);
        let number: String = tail.chars().take_while(char::is_ascii_digit).collect();
        ensure!(
            !number.is_empty(),
            "Powers require a literal integer exponent"
        );
        ensure!(
            number.parse::<u32>().context("Exponent too large")? <= 16,
            "Exponent magnitude exceeds 16"
        );
        ensure!(
            tail[number.len()..].bytes().next().map(|byte| b" +-*/^)>	".contains(&byte)).unwrap_or(true),
            "Powers require a complete decimal integer token, not a scientific, radix or fractional exponent"
        );
    }
    Ok(())
}

fn field<'a>(value: &'a Option<String>, name: &str) -> Result<&'a str> {
    let value = value
        .as_deref()
        .with_context(|| format!("Missing {name}"))?;
    ensure!(value.len() <= 128, "Wave operands are limited to 128 bytes");
    validate(value)?;
    Ok(value)
}

// Presets accept a deliberately narrower positive rational lane. Arbitrary
// expressions remain available through query, without a physical-sign claim.
struct Frequency {
    text: String,
    numerator_hz: u128,
    denominator: u128,
}

fn frequency(value: &Option<String>, name: &str) -> Result<Frequency> {
    let value = field(value, name)?.trim();
    let (coefficient, unit) = value.rsplit_once(' ').context(
        "Wave frequency needs a positive integer or p/q coefficient and a Hz-family unit",
    )?;
    ensure!(
        ["Hz", "kHz", "MHz", "GHz", "mHz", "uHz", "nHz"].contains(&unit),
        "Wave presets require ordinary cycle frequency in Hz, kHz, MHz, GHz, mHz, uHz or nHz; not angular frequency in rad/s"
    );
    let coefficient = coefficient.trim();
    let coefficient = if coefficient.starts_with('(') {
        coefficient
            .strip_prefix('(')
            .and_then(|v| v.strip_suffix(')'))
            .context("Invalid rational frequency parentheses")?
    } else {
        coefficient
    };
    let parts: Vec<_> = coefficient.split('/').collect();
    ensure!(
        parts.len() <= 2,
        "Frequency must be an integer or p/q rational"
    );
    for part in &parts {
        ensure!(
            !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()),
            "Frequency must use unsigned integer or p/q notation, not an unchecked expression"
        );
        ensure!(
            part.parse::<u64>()
                .context("Frequency coefficient too large")?
                > 0,
            "Wave frequencies and rational denominators must be strictly positive"
        );
    }
    let text = if parts.len() == 2 {
        // Parenthesization avoids Rink parsing 1/3 Hz as reciprocal frequency.
        format!("({}/{}) {unit}", parts[0], parts[1])
    } else {
        format!("{} {unit}", parts[0])
    };
    let (scale_num, scale_den) = match unit {
        "kHz" => (1_000u128, 1),
        "MHz" => (1_000_000, 1),
        "GHz" => (1_000_000_000, 1),
        "mHz" => (1, 1_000),
        "uHz" => (1, 1_000_000),
        "nHz" => (1, 1_000_000_000),
        _ => (1, 1),
    };
    let numerator = parts[0].parse::<u128>()?;
    let denominator = parts
        .get(1)
        .map(|p| p.parse::<u128>())
        .transpose()?
        .unwrap_or(1);
    Ok(Frequency {
        text,
        numerator_hz: numerator
            .checked_mul(scale_num)
            .context("Frequency scale overflow")?,
        denominator: denominator
            .checked_mul(scale_den)
            .context("Frequency scale overflow")?,
    })
}

fn expression(request: &Request) -> Result<String> {
    let operation = request.operation.as_deref().unwrap_or("query");
    let result = match operation {
        "query" => request.expression.clone().context("Missing expression")?,
        "period" => format!(
            "1 / ({}) to fraction s",
            frequency(&request.frequency, "frequency")?.text
        ),
        "wavelength" => format!(
            "({}) / ({}) to fraction m",
            field(&request.speed, "speed")?,
            frequency(&request.frequency, "frequency")?.text
        ),
        "beat" => {
            let first = frequency(&request.frequency, "frequency")?;
            let second = frequency(&request.other_frequency, "other_frequency")?;
            let left = first
                .numerator_hz
                .checked_mul(second.denominator)
                .context("Beat comparison exceeds the checked integer range")?;
            let right = second
                .numerator_hz
                .checked_mul(first.denominator)
                .context("Beat comparison exceeds the checked integer range")?;
            let (high, low) = if left >= right {
                (first.text, second.text)
            } else {
                (second.text, first.text)
            };
            // Subtraction of exact rationals stays exact; sqrt(square) uses Rink's float fallback.
            format!("({high}) - ({low}) to fraction Hz")
        }
        _ => bail!("Unknown Rink operation: {operation}"),
    };
    validate(&result)?;
    Ok(result)
}

/// Private empty working/config directories prevent ambient user unit overrides.
struct QueryDir(PathBuf);
impl QueryDir {
    fn create(interpret: bool) -> Result<Self> {
        let path = std::env::temp_dir().join(format!("st-rink-{}", uuid::Uuid::new_v4()));
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&path)?;
        let directory = Self(path);
        std::fs::write(directory.0.join("config.toml"), config_text(interpret))?;
        Ok(directory)
    }
}
impl Drop for QueryDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

async fn limited_read(reader: impl AsyncRead + Unpin) -> Result<String> {
    let mut bytes = Vec::new();
    reader.take(MAX_OUTPUT + 1).read_to_end(&mut bytes).await?;
    ensure!(
        bytes.len() as u64 <= MAX_OUTPUT,
        "Rink output exceeds 16384 bytes per stream"
    );
    let text = String::from_utf8(bytes).context("Rink output is not UTF-8")?;
    ensure!(
        !text
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t' && c != '\r'),
        "Rink emitted terminal control characters"
    );
    Ok(text)
}

async fn evaluate(binary: &Path, expression: &str, wall: Duration) -> Result<String> {
    evaluate_with(binary, expression, wall, false).await
}

async fn evaluate_with(
    binary: &Path,
    expression: &str,
    wall: Duration,
    interpret: bool,
) -> Result<String> {
    ensure!(
        cfg!(target_os = "linux"),
        "Bounded Rink adapter currently requires Linux"
    );
    ensure!(
        binary.is_absolute() && binary.is_file(),
        "ST_RINK_BIN must name an existing absolute executable"
    );
    let directory = QueryDir::create(interpret)?;
    let mut command = tokio::process::Command::new("/usr/bin/prlimit");
    command
        .args([
            "--as=536870912",
            "--cpu=2",
            "--fsize=0",
            "--core=0",
            "--nofile=64",
            "--",
        ])
        .arg(binary)
        .arg("--config")
        .arg(directory.0.join("config.toml"))
        .arg("--")
        .arg(expression)
        .current_dir(&directory.0)
        .env_clear()
        .env("HOME", &directory.0)
        .env("XDG_CONFIG_HOME", &directory.0)
        .env("XDG_CACHE_HOME", &directory.0)
        .env("NO_COLOR", "1")
        .env("LANG", "C.UTF-8")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = command
        .spawn()
        .context("Cannot start Rink with /usr/bin/prlimit")?;
    let stdout = child.stdout.take().context("Missing Rink stdout")?;
    let stderr = child.stderr.take().context("Missing Rink stderr")?;
    let result = tokio::time::timeout(wall, async {
        tokio::try_join!(
            async { child.wait().await.context("Cannot wait for Rink") },
            limited_read(stdout),
            limited_read(stderr)
        )
    })
    .await;
    match result {
        Ok(Ok((status, stdout, stderr))) => {
            ensure!(status.success(), "Rink failed ({status}): {stdout}{stderr}");
            ensure!(!stdout.trim().is_empty(), "Rink returned empty output");
            Ok(stdout)
        }
        failure => {
            // Explicitly kill AND reap on errors; kill_on_drop also handles cancellation.
            let _ = child.kill().await;
            match failure {
                Ok(Err(error)) => Err(error),
                _ => bail!("Rink exceeded wall time limit"),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Provenance {
    expression: String,
    output: String,
    executable: String,
    operator_revision: Option<String>,
    unix_seconds: i64,
    adapter_version: String,
}

fn memory_path() -> Result<PathBuf> {
    let path = std::env::var_os("ST_RINK_MEMORY_DIR")
        .context("MEM8 provenance requires operator ST_RINK_MEMORY_DIR opt-in")?;
    let path = PathBuf::from(path);
    ensure!(path.is_absolute(), "ST_RINK_MEMORY_DIR must be absolute");
    Ok(path.join("rink-provenance.native.m8"))
}

fn store(path: &Path, id: &str, provenance: Option<&Provenance>) -> Result<Provenance> {
    use crate::mem8::record_store::RecordStore;
    let id = uuid::Uuid::parse_str(id).context("Invalid memory UUID")?;
    ensure!(
        provenance.is_some() || path.is_file(),
        "Rink provenance store does not exist"
    );
    if let Ok(metadata) = std::fs::metadata(path) {
        ensure!(
            metadata.len() <= MAX_STORE,
            "Rink provenance store reached 16 MiB; preserve/archive it before further use"
        );
    }
    let mut store = RecordStore::open_memory_preserving(path)?;
    let key = format!("rink/provenance/v1/{id}");
    if let Some(value) = provenance {
        store.put(&key, value)?;
    }
    drop(store);
    // Reopen through the owning reader: acknowledgement includes durable read-back.
    RecordStore::open_memory_preserving(path)?
        .get(&key)?
        .context("Rink memory not found")
}

pub async fn handle(args: Value) -> Result<Value> {
    let request: Request = serde_json::from_value(args).context("Invalid Rink arguments")?;
    let _permit = QUERIES
        .try_acquire()
        .context("Rink busy: at most two concurrent requests")?;
    if request.operation.as_deref() == Some("recall") {
        ensure!(!request.remember, "recall does not accept remember");
        let id = request.memory_id.context("Missing memory_id")?;
        let path = memory_path()?;
        let value = tokio::task::spawn_blocking(move || store(&path, &id, None)).await??;
        return envelope(
            json!({"provenance": value, "storage": "Smart Tree native MEM8/RAW8 via RecordStore"}),
        );
    }
    let expression = expression(&request)?;
    // Check memory configuration before running; never silently downgrade a requested write.
    let memory = request.remember.then(memory_path).transpose()?;
    let binary = std::env::var_os("ST_RINK_BIN").context(
        "Rink disabled: operator must set ST_RINK_BIN to a trusted absolute Rink CLI executable",
    )?;
    let binary = PathBuf::from(binary);
    ensure!(binary.is_absolute(), "ST_RINK_BIN must be absolute");
    let binary = binary
        .canonicalize()
        .context("Cannot resolve ST_RINK_BIN")?;
    let output = evaluate_with(&binary, &expression, WALL_LIMIT, request.interpret).await?;
    let provenance = Provenance {
        expression,
        output,
        executable: binary.display().to_string(),
        operator_revision: std::env::var("ST_RINK_REVISION")
            .ok()
            .map(|s| s.chars().take(128).collect()),
        unix_seconds: chrono::Utc::now().timestamp(),
        adapter_version: env!("CARGO_PKG_VERSION").to_owned(),
    };
    let memory_id = if let Some(path) = memory {
        let id = uuid::Uuid::new_v4().to_string();
        let copy = provenance.clone();
        let key = id.clone();
        tokio::task::spawn_blocking(move || store(&path, &key, Some(&copy))).await??;
        Some(id)
    } else {
        None
    };
    envelope(json!({
        "provenance": provenance, "memory_id": memory_id,
        "arithmetic": "Rink text retained without numeric coercion; rational operations may be exact; external Rink has floating-point fallbacks",
        "limits": {"wall_seconds": 3, "cpu_seconds": 2, "address_space_bytes": 536870912u64, "output_bytes_per_stream": MAX_OUTPUT},
        "scope": "quantity conversion and classical wave relationships, not quantum hardware",
        "wave_convention": "Presets require positive ordinary cycle frequency f in Hz-family units: T=1/f, wavelength=v/f; angular omega=2*pi*f, so angular inputs need an explicit conversion and are rejected by presets.",
        "offline": "currency disabled; no network operation requested; trusted external binary, not a network sandbox"
    }))
}

fn envelope(value: Value) -> Result<Value> {
    Ok(json!({"content": [{"type": "text", "text": serde_json::to_string_pretty(&value)?}]}))
}
