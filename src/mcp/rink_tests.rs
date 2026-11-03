//! Tests use owned temporary stores/executables, never a live MEM8 manager.
use super::*;

#[test]
fn rink_validation_rejects_injection_and_excess_work() {
    for valid in [
        "1/3 meter to cm",
        "-1 m -> cm",
        "(343 m/s)/(432 Hz) to fraction m",
        "m^2",
        "s^-2",
        "1640 g to lb;oz",
        "(2/3) meter to m;cm fraction 32",
        "#jan 01, 1970 00:00 UTC# -> America/Los_Angeles",
        "#jan 01, 1970 00:00 UTC# -> +02:00",
    ] {
        assert!(validate(valid).is_ok(), "{valid}");
    }
    for invalid in [
        "",
        "\n1 m",
        "1 m;$(id)",
        "#jan 01, 1970#;$(id)",
        "$(id)",
        "`id`",
        "1 | cat",
        "1 & cat",
        "é",
        "1\0",
        "(1",
        ")1",
        "1^99999",
        "1^(2)",
        "1^2.5",
        "2^1e8",
        "2^0x1000",
        "2^16abc",
        "1234567890123456789",
    ] {
        assert!(validate(invalid).is_err(), "{invalid:?}");
    }
    assert!(validate(&"x".repeat(513)).is_err());
    assert!(validate(&format!("{}1{}", "(".repeat(17), ")".repeat(17))).is_err());
}

#[test]
fn rink_interpretation_is_opt_in_and_currency_stays_disabled() {
    let quiet = config_text(false);
    let shown = config_text(true);
    assert!(quiet.contains("show_interpretation = false"), "{quiet}");
    assert!(shown.contains("show_interpretation = true"), "{shown}");
    assert!(shown.contains("[currency]\nenabled = false"), "{shown}");
    assert_eq!(
        definition()["inputSchema"]["properties"]["interpret"]["default"],
        false
    );
}

#[test]
fn rink_definitions_are_explicit_and_do_not_read_home() {
    assert!(validate_definitions("stack 64\n").is_ok());
    assert!(validate_definitions(
        "!category localwood \"Local wood\"\n?? note\nstack 7\n!endcategory\n"
    )
    .is_ok());
    assert!(validate_definitions("widget {\n}\n").is_ok());
    assert!(validate_definitions("1|2 cord\n").is_ok());
    assert!(validate_definitions("stack $(id)\n").is_err());
    assert!(validate_definitions("stack `id`\n").is_err());
    assert!(validate_definitions(&"x".repeat(4097)).is_err());
    let with_defs = QueryDir::create(false, Some("stack 64\n")).unwrap();
    assert_eq!(
        std::fs::read_to_string(with_defs.0.join("definitions.units")).unwrap(),
        "stack 64\n"
    );
    let quiet = QueryDir::create(false, None).unwrap();
    assert!(!quiet.0.join("definitions.units").exists());
}

#[test]
fn rink_wave_presets_preserve_rational_operands_and_units() {
    let examples = [
        (
            json!({"operation":"period", "frequency":"432 Hz"}),
            "1 / (432 Hz) to fraction s",
        ),
        (
            json!({"operation":"wavelength", "frequency":"432 Hz", "speed":"343 m/s"}),
            "(343 m/s) / (432 Hz) to fraction m",
        ),
        (
            json!({"operation":"beat", "frequency":"432 Hz", "other_frequency":"440 Hz"}),
            "(440 Hz) - (432 Hz) to fraction Hz",
        ),
        (
            json!({"operation":"wave", "frequency":"440 Hz", "speed":"343 m/s"}),
            "wave 440 Hz, 343 m/s",
        ),
        (
            json!({"expression":"(1/3) m to fraction cm"}),
            "(1/3) m to fraction cm",
        ),
    ];
    for (args, expected) in examples {
        assert_eq!(
            expression(&serde_json::from_value(args).unwrap()).unwrap(),
            expected
        );
    }
    for (first, second, expected) in [
        ("1/3 kHz", "333 Hz", "((1/3) kHz) - (333 Hz) to fraction Hz"),
        ("1 Hz", "1000 mHz", "(1 Hz) - (1000 mHz) to fraction Hz"),
        ("1 nHz", "1 GHz", "(1 GHz) - (1 nHz) to fraction Hz"),
    ] {
        let request = serde_json::from_value(
            json!({"operation":"beat","frequency":first,"other_frequency":second}),
        )
        .unwrap();
        assert_eq!(expression(&request).unwrap(), expected);
    }
    for invalid in [
        json!({"operation":"period"}),
        json!({"operation":"unknown"}),
        json!({"operation":"wave", "frequency":"440 Hz", "speed":"343 m/s, extra"}),
        json!({"expression":"1", "timeout":999}),
    ] {
        assert!(serde_json::from_value::<Request>(invalid)
            .and_then(|r| expression(&r).map_err(serde::de::Error::custom))
            .is_err());
    }
}

#[test]
fn rink_wave_frequencies_are_strictly_positive_exact_cycle_quantities() {
    for (input, expected) in [
        ("432 Hz", "432 Hz"),
        ("1/3 kHz", "(1/3) kHz"),
        ("(2/7) MHz", "(2/7) MHz"),
    ] {
        assert_eq!(
            frequency(&Some(input.into()), "frequency").unwrap().text,
            expected
        );
    }
    for invalid in [
        "0 Hz", "-1 Hz", "0/3 Hz", "1/0 Hz", "1/-3 Hz", "1 rad/s", "1 meter", "1.5 Hz", "(3-4) Hz",
        "-(-1) Hz", "1e3 Hz", "1/2/3 Hz", "Hz", "1/ Hz",
    ] {
        let request = Request {
            operation: Some("period".into()),
            frequency: Some(invalid.into()),
            ..Request::default()
        };
        assert!(expression(&request).is_err(), "accepted {invalid}");
        let request = Request {
            operation: Some("beat".into()),
            frequency: Some("1 Hz".into()),
            other_frequency: Some(invalid.into()),
            ..Request::default()
        };
        assert!(
            expression(&request).is_err(),
            "accepted other_frequency {invalid}"
        );
    }
    let request = Request {
        operation: Some("period".into()),
        frequency: Some("1/3 Hz".into()),
        ..Request::default()
    };
    assert_eq!(
        expression(&request).unwrap(),
        "1 / ((1/3) Hz) to fraction s"
    );
}

#[test]
fn rink_provenance_reopens_native_mem8_and_preserves_damage() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("provenance.native.m8");
    let id = uuid::Uuid::new_v4().to_string();
    let value = Provenance {
        expression: "(1/3) m to fraction cm".into(),
        output: "100/3 cm".into(),
        executable: "/trusted/rink".into(),
        operator_revision: Some("test-only".into()),
        unix_seconds: 123,
        adapter_version: "test".into(),
    };
    assert_eq!(store(&path, &id, Some(&value)).unwrap(), value);
    assert_eq!(store(&path, &id, None).unwrap(), value);
    let bytes = std::fs::read(&path).unwrap();
    // Existing owner's native MEM8 header, not JSON masquerading as persistence.
    assert_eq!(&bytes[8..12], &0x4d454d38u32.to_le_bytes());
    assert!(store(&path, "../../escape", None).is_err());
    let damaged = temp.path().join("damaged.m8");
    std::fs::write(&damaged, b"not a native MEM8 file").unwrap();
    assert!(store(&damaged, &id, Some(&value)).is_err());
    assert_eq!(std::fs::read(&damaged).unwrap(), b"not a native MEM8 file");
    for length in [1, 4, 11, 63, bytes.len() - 1] {
        let truncated = &bytes[..length];
        std::fs::write(&damaged, truncated).unwrap();
        assert!(store(&damaged, &id, None).is_err());
        assert_eq!(std::fs::read(&damaged).unwrap(), truncated);
        assert!(store(&damaged, &id, Some(&value)).is_err());
        assert_eq!(std::fs::read(&damaged).unwrap(), truncated);
    }
    let missing = temp.path().join("missing.m8");
    assert!(store(&missing, &id, None).is_err());
    assert!(!missing.exists());
}

#[cfg(target_os = "linux")]
fn fixture(body: &str) -> (tempfile::TempDir, PathBuf) {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("rink-fixture");
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    (temp, path)
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn rink_process_contract_isolated_config_and_literal_argument() {
    let (_temp, binary) = fixture(
        r#"
[ "$#" = 4 ] || exit 21
[ "$1" = --config ] || exit 22
[ "$3" = -- ] || exit 23
[ "$4" = '-1 m to cm' ] || exit 24
[ "$HOME" = "$PWD" ] || exit 25
[ "$XDG_CONFIG_HOME" = "$PWD" ] || exit 26
[ -z "$HTTP_PROXY" ] || exit 27
[ -z "$LD_PRELOAD" ] || exit 28
[ ! -e definitions.units ] || exit 29
config=$(/bin/cat "$2")
case "$config" in *'enabled = false'*) ;; *) exit 30 ;; esac
printf '%s\n' "$PWD"
printf '%s\n' '-100 cm'
"#,
    );
    let output = evaluate(&binary, "-1 m to cm", WALL_LIMIT).await.unwrap();
    let cwd = output.lines().next().unwrap();
    assert!(
        !Path::new(cwd).exists(),
        "temporary directory was not cleaned"
    );
    assert!(output.ends_with("-100 cm\n"));
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn rink_process_caps_stdout_stderr_and_rejects_errors() {
    for body in [
        "while :; do printf '0123456789012345678901234567890123456789012345678901234567890123456789'; done",
        "while :; do printf '0123456789012345678901234567890123456789012345678901234567890123456789' >&2; done",
    ] {
        let (_temp, binary) = fixture(body);
        let error = evaluate(&binary, "1", WALL_LIMIT).await.unwrap_err();
        assert!(error.to_string().contains("output exceeds"), "{error:#}");
    }
    for body in [
        "printf 'unit mismatch'; exit 1",
        "printf '\\033[31mred'",
        "exit 0",
    ] {
        let (_temp, binary) = fixture(body);
        assert!(evaluate(&binary, "1", WALL_LIMIT).await.is_err());
    }
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn rink_process_wall_cpu_memory_limits_fail_closed() {
    let (_temp, binary) = fixture("while :; do :; done");
    let error = evaluate(&binary, "1", Duration::from_millis(30))
        .await
        .unwrap_err();
    assert!(error.to_string().contains("wall time"));
    let error = evaluate(&binary, "1", Duration::from_secs(5))
        .await
        .unwrap_err();
    assert!(
        error.to_string().contains("Rink failed"),
        "CPU limit did not terminate: {error:#}"
    );
    let (_temp, binary) = fixture("exec /usr/bin/python3 -c 'bytearray(600 * 1024 * 1024)'");
    let error = evaluate(&binary, "1", WALL_LIMIT).await.unwrap_err();
    assert!(error.to_string().contains("MemoryError"), "{error:#}");
    assert!(evaluate(Path::new("relative-rink"), "1", WALL_LIMIT)
        .await
        .is_err());
    assert!(evaluate(Path::new("/nonexistent/rink"), "1", WALL_LIMIT)
        .await
        .is_err());
}

#[tokio::test]
async fn rink_discovery_and_both_dispatchers_use_same_contract() {
    use super::super::{McpConfig, McpServer};
    let server = McpServer::new(McpConfig::default());
    let ctx = server.context;
    let plain = super::super::tools::handle_tools_list(None, ctx.clone())
        .await
        .unwrap();
    for list in [
        plain["tools"].as_array().unwrap().clone(),
        super::super::tools_consolidated::get_consolidated_tools(),
        super::super::tools_consolidated_enhanced::get_enhanced_consolidated_tools(),
    ] {
        let definitions: Vec<_> = list.iter().filter(|v| v["name"] == "rink").collect();
        assert_eq!(definitions.len(), 1);
        assert_eq!(*definitions[0], definition());
    }
    // Invalid expression fails validation before environment/binary access in both modes.
    let args = json!({"expression":"$(bad)"});
    let consolidated = super::super::tools_consolidated::dispatch_consolidated_tool(
        "rink",
        Some(args.clone()),
        ctx.clone(),
    )
    .await;
    let plain =
        super::super::tools::handle_tools_call(json!({"name":"rink", "arguments":args}), ctx).await;
    assert!(consolidated.unwrap_err().to_string().contains("ASCII"));
    assert!(plain.unwrap_err().to_string().contains("ASCII"));
}

/// Explicit operator-provided binary only; never install or mutate a live service.
#[cfg(target_os = "linux")]
#[tokio::test]
#[ignore = "set ST_RINK_TEST_BIN to an independently built Rink CLI"]
async fn rink_external_end_to_end_quantities_waves_and_provenance() {
    let binary =
        PathBuf::from(std::env::var_os("ST_RINK_TEST_BIN").expect("ST_RINK_TEST_BIN required"));
    let temp = tempfile::tempdir().unwrap();
    for (args, expected) in [
        (json!({"expression":"(1/3) m to fraction cm"}), "100/3"),
        (json!({"operation":"period", "frequency":"4 Hz"}), "1/4"),
        (
            json!({"operation":"period", "frequency":"1/4 Hz"}),
            "4 second",
        ),
        (
            json!({"operation":"wavelength", "speed":"300 m/s", "frequency":"3 Hz"}),
            "100",
        ),
        (
            json!({"operation":"beat", "frequency":"432 Hz", "other_frequency":"440 Hz"}),
            "8",
        ),
    ] {
        let request: Request = serde_json::from_value(args).unwrap();
        let query = expression(&request).unwrap();
        let output = evaluate(&binary, &query, WALL_LIMIT).await.unwrap();
        assert!(output.contains(expected), "{query}: {output}");
        let value = Provenance {
            expression: query,
            output,
            executable: binary.display().to_string(),
            operator_revision: None,
            unix_seconds: 123,
            adapter_version: "test".into(),
        };
        let id = uuid::Uuid::new_v4().to_string();
        let path = temp.path().join("e2e.native.m8");
        assert_eq!(store(&path, &id, Some(&value)).unwrap(), value);
        assert_eq!(store(&path, &id, None).unwrap(), value);
    }
    assert!(evaluate(&binary, "1 m to fraction s", WALL_LIMIT)
        .await
        .is_err());
}

#[cfg(target_os = "linux")]
#[tokio::test]
#[ignore = "set ST_RINK_BIN and private ST_RINK_MEMORY_DIR for actual MCP/MEM8 calls"]
async fn rink_mcp_end_to_end_both_modes_and_reopened_recall() {
    use super::super::{McpConfig, McpServer};
    assert!(std::env::var_os("ST_RINK_BIN").is_some());
    assert!(std::env::var_os("ST_RINK_MEMORY_DIR").is_some());
    for consolidated in [true, false] {
        let config = McpConfig {
            use_consolidated_tools: consolidated,
            ..McpConfig::default()
        };
        let server = McpServer::new(config.clone());
        let response = server.handle_request(&json!({
            "jsonrpc":"2.0", "id":1, "method":"tools/call", "params": {
                "name":"rink", "arguments": {"expression":"(1/3) meter to fraction cm", "remember":true}
            }
        }).to_string()).await.unwrap();
        let response: Value = serde_json::from_str(&response).unwrap();
        assert!(response.get("error").is_none(), "{response}");
        let payload: Value =
            serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap())
                .unwrap();
        assert!(payload["provenance"]["output"]
            .as_str()
            .unwrap()
            .contains("100/3"));
        let id = payload["memory_id"].as_str().unwrap();
        let expression = payload["provenance"]["expression"].clone();
        drop(server);
        // Fresh MCP context, no in-process result cache; RecordStore reopens the same file.
        let server = McpServer::new(config);
        let response = server
            .handle_request(
                &json!({
                    "jsonrpc":"2.0", "id":2, "method":"tools/call", "params": {
                        "name":"rink", "arguments": {"operation":"recall", "memory_id":id}
                    }
                })
                .to_string(),
            )
            .await
            .unwrap();
        let response: Value = serde_json::from_str(&response).unwrap();
        assert!(response.get("error").is_none(), "{response}");
        let recalled: Value =
            serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap())
                .unwrap();
        assert_eq!(recalled["provenance"]["expression"], expression);
        assert_eq!(
            recalled["provenance"]["output"],
            payload["provenance"]["output"]
        );
    }
}
