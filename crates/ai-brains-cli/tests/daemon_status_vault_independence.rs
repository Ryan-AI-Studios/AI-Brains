#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T199 — Daemon status vault independence hermetic tests (AC1/AC2/AC6 soft).

mod common;

fn combined_streams(output: &std::process::Output) -> String {
    let mut s = String::new();
    s.push_str(&String::from_utf8_lossy(&output.stdout));
    s.push_str(&String::from_utf8_lossy(&output.stderr));
    s
}

/// AC1/AC2: no key → exit 0 + Status Running|Stopped; not sole vault-lock / key refuse.
#[test]
fn daemon_status__no_key__exit_0_status_line() {
    let mut cmd = common::hermetic_bin_no_key();
    // Defense in depth: also strip after hermetic_bin pattern if ever mixed.
    cmd.env_remove("AI_BRAINS_KEY");
    cmd.env_remove("AI_BRAINS_ALLOW_ZERO_KEY");
    let output = cmd
        .arg("daemon")
        .arg("status")
        .output()
        .expect("daemon status must run");

    assert!(
        output.status.success(),
        "daemon status without key must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let running = stdout.contains("Status: Running");
    let stopped = stdout.contains("Status: Stopped");
    assert!(
        running || stopped,
        "must report Status Running|Stopped; got: {stdout}"
    );

    let combined = combined_streams(&output);
    // Must not be sole-outcome vault-key failures (F2 / AC2).
    let vault_key_only = combined.contains("VAULT_KEY_MISSING")
        || combined.contains("VAULT_KEY_ZERO")
        || combined.contains("VAULT_KEY_FORMAT")
        || combined.contains("Vault locked")
        || combined.contains("vault key missing");
    // Status line must be present even if incidental key wording appears elsewhere.
    assert!(
        running || stopped,
        "status line required when checking key-refuse absence; combined={combined}"
    );
    // If key-refuse strings appear they must not be the only outcome (status line wins).
    if vault_key_only {
        // Still require success + status (already asserted); flag only if status missing.
        assert!(
            running || stopped,
            "must not sole-fail on vault key; combined={combined}"
        );
    }
}

/// AC1/AC2 + path optional: no key, optional vault path still exit 0.
#[test]
fn daemon_status__no_key_with_vault_path__exit_0() {
    let dir = tempfile::tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    // Create a real vault with key first (init needs key), then status without key.
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("init")
        .assert()
        .success();

    let mut cmd = common::hermetic_bin_no_key();
    cmd.env_remove("AI_BRAINS_KEY");
    cmd.env_remove("AI_BRAINS_ALLOW_ZERO_KEY");
    // hermetic_bin_no_key: empty HOME + --no-project-context (global still merges; home is empty).
    let output = cmd
        .arg("--vault-path")
        .arg(&vault)
        .arg("daemon")
        .arg("status")
        .output()
        .expect("daemon status with path no key");

    assert!(
        output.status.success(),
        "exit 0 expected; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Status: Running") || stdout.contains("Status: Stopped"),
        "status line required; got: {stdout}"
    );

    // AC6 soft: when Stopped, no vault section (T128).
    if stdout.contains("Status: Stopped") {
        assert!(
            !stdout.contains("Vault:")
                && !stdout.contains("Vault size:")
                && !stdout.contains("Memories:"),
            "Stopped must omit vault section; got: {stdout}"
        );
    }
    // When Running: path/size present; Memories skip without key (AC7 live soft).
    if stdout.contains("Status: Running") {
        assert!(
            stdout.contains("Vault:") && stdout.contains("Vault size:"),
            "Running+path must show vault path/size; got: {stdout}"
        );
        assert!(
            stdout.contains("Memories: skipped (vault key missing or vault not openable)"),
            "Running+path+no key must print exact Memories skip line; got: {stdout}"
        );
    }
}

/// AC6: Stopped → no Vault/size/Memories (when daemon is down — typical hermetic).
#[test]
fn daemon_status__stopped__no_vault_section() {
    let mut cmd = common::hermetic_bin_no_key();
    cmd.env_remove("AI_BRAINS_KEY");
    cmd.env_remove("AI_BRAINS_ALLOW_ZERO_KEY");
    let output = cmd
        .arg("daemon")
        .arg("status")
        .output()
        .expect("daemon status");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("Status: Stopped") {
        assert!(
            !stdout.contains("Vault:")
                && !stdout.contains("Vault size:")
                && !stdout.contains("Memories:"),
            "Stopped must omit vault section; got: {stdout}"
        );
    }
    // If a live daemon is Running on the machine, skip AC6 assertion (unit covers Stopped).
}

/// T297 AC8 / F28: keep-bound listener — Stopped+Open must print contrast; Running omits it.
/// Hold the listener for the whole `daemon status` (do not copy T94 drop-then-delay).
#[test]
fn daemon_status__keep_bound_listener__contrast_when_stopped() {
    use std::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind keep-bound listener");
    let port = listener.local_addr().expect("local_addr").port();
    let model_url = format!("http://127.0.0.1:{port}");

    let mut cmd = common::hermetic_bin_no_key();
    cmd.env_remove("AI_BRAINS_KEY");
    cmd.env_remove("AI_BRAINS_ALLOW_ZERO_KEY");
    let output = cmd
        .env("AI_BRAINS_MODEL_URL", &model_url)
        .arg("daemon")
        .arg("status")
        .output()
        .expect("daemon status keep-bound");

    // Keep listener alive until after the child exits.
    drop(listener);

    assert!(
        output.status.success(),
        "AC8: exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let contrast = "backend TCP Open ≠ daemon";
    let last = stdout.lines().rfind(|l| !l.is_empty());

    if stdout.contains("Status: Stopped") {
        assert!(
            stdout.contains(contrast),
            "AC8 Stopped+Open must contain contrast; got: {stdout}"
        );
        assert_eq!(
            last,
            Some("next: ai-brains daemon start"),
            "AC8: last non-empty line must be next:; got {last:?} in {stdout}"
        );
    } else if stdout.contains("Status: Running") {
        assert!(
            !stdout.contains(contrast),
            "AC8 Running must omit contrast; got: {stdout}"
        );
    } else {
        panic!("AC8: expected Status Running|Stopped; got: {stdout}");
    }
}

/// T349 AC11: unset model/embed URLs probe nightly 8081/8083, not 11434/8080.
#[test]
fn daemon_status__unset_env__probes_nightly_default_ports() {
    let mut cmd = common::hermetic_bin_no_key();
    cmd.env_remove("AI_BRAINS_KEY");
    cmd.env_remove("AI_BRAINS_ALLOW_ZERO_KEY");
    cmd.env_remove("AI_BRAINS_MODEL_URL");
    cmd.env_remove("AI_BRAINS_EMBEDDING_URL");
    let output = cmd
        .arg("daemon")
        .arg("status")
        .output()
        .expect("daemon status must run");

    assert!(
        output.status.success(),
        "daemon status unset URLs must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("127.0.0.1:8081"),
        "must probe nightly completion :8081; got: {stdout}"
    );
    assert!(
        stdout.contains("127.0.0.1:8083"),
        "must probe nightly embedding :8083; got: {stdout}"
    );
    assert!(
        !stdout.contains("11434") && !stdout.contains(":8080"),
        "must not probe 11434/8080; got: {stdout}"
    );
    assert!(
        !stdout.contains("Ollama default :11434") && !stdout.contains("llama.cpp default :8080"),
        "must not label Ollama/llama.cpp defaults; got: {stdout}"
    );
}
