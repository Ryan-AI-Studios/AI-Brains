//! T256 — Root clap help must not echo `AI_BRAINS_KEY` (AC1–AC6, AC12).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use ai_brains_crypto::test_support::assert_no_secret_leakage;
use std::process::Output;

/// Distinctive product-form dummy (F8). Must not be `ZERO_SQLCIPHER_KEY`.
const DUMMY_KEY: &str = "x'deadbeefcafebabe0123456789abcdefdeadbeefcafebabe0123456789abcdef'";

const DUMMY_KEY_BYTES: [u8; 32] = [
    0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
    0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
];

const ENV_SLOT: &str = "[env: AI_BRAINS_KEY]";
const KEY_ASSIGN_PREFIX: &str = "AI_BRAINS_KEY=x'";

fn combined_output(output: &Output) -> String {
    let mut s = String::new();
    s.push_str(&String::from_utf8_lossy(&output.stdout));
    s.push_str(&String::from_utf8_lossy(&output.stderr));
    s
}

fn hermetic_with_dummy() -> assert_cmd::Command {
    let mut cmd = common::hermetic_bin();
    cmd.env("AI_BRAINS_KEY", DUMMY_KEY);
    cmd
}

fn assert_key_not_echoed(combined: &str) {
    assert_no_secret_leakage(combined, &DUMMY_KEY_BYTES);
    assert!(
        !combined.contains(KEY_ASSIGN_PREFIX),
        "combined output must not contain {KEY_ASSIGN_PREFIX}"
    );
}

fn assert_root_help_redacted(args: &[&str]) {
    let mut cmd = hermetic_with_dummy();
    for a in args {
        cmd.arg(a);
    }
    let out = cmd.output().expect("help command must spawn");
    assert!(
        out.status.success(),
        "help must exit 0; args={args:?} stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let combined = combined_output(&out);
    // Leak first (the red AC). Exact `[env: AI_BRAINS_KEY]` only appears after
    // hide_env_values (today clap emits `[env: AI_BRAINS_KEY=…]`).
    assert_key_not_echoed(&combined);
    assert!(
        combined.contains(ENV_SLOT),
        "root help must contain exact {ENV_SLOT}; args={args:?}"
    );
}

#[test]
fn root_long_help__dummy_key_env__does_not_echo_payload() {
    assert_root_help_redacted(&["--help"]);
}

#[test]
fn root_short_help__dummy_key_env__does_not_echo_payload() {
    assert_root_help_redacted(&["-h"]);
}

#[test]
fn root_help_subcommand__dummy_key_env__does_not_echo_payload() {
    assert_root_help_redacted(&["help"]);
}

#[test]
fn root_long_help__key_unset__still_names_env() {
    let out = common::hermetic_bin_no_key()
        .arg("--help")
        .output()
        .expect("help command must spawn");
    assert!(
        out.status.success(),
        "unset-key --help must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let combined = combined_output(&out);
    assert_key_not_echoed(&combined);
    // Same clap slot as AC1: `[env: AI_BRAINS_KEY]` only after hide_env_values
    // (unset today is `[env: AI_BRAINS_KEY=]`, so this is green-after-F1).
    assert!(
        combined.contains(ENV_SLOT),
        "unset-key --help must still name {ENV_SLOT}"
    );
}

#[test]
fn doctor_help__dummy_key_env__does_not_echo_payload() {
    let out = hermetic_with_dummy()
        .args(["doctor", "--help"])
        .output()
        .expect("doctor --help must spawn");
    assert!(
        out.status.success(),
        "doctor --help must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let combined = combined_output(&out);
    assert_key_not_echoed(&combined);
}

#[test]
fn recall_help__dummy_key_env__does_not_echo_payload() {
    let out = hermetic_with_dummy()
        .args(["recall", "--help"])
        .output()
        .expect("recall --help must spawn");
    assert!(
        out.status.success(),
        "recall --help must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let combined = combined_output(&out);
    assert_key_not_echoed(&combined);
}

#[test]
fn unknown_flag__dummy_key_env__does_not_echo_payload() {
    let out = hermetic_with_dummy()
        .arg("--not-a-real-flag")
        .output()
        .expect("unknown flag must spawn");
    assert!(
        !out.status.success(),
        "unknown flag must be non-zero; stdout={}",
        String::from_utf8_lossy(&out.stdout)
    );
    let combined = combined_output(&out);
    assert_key_not_echoed(&combined);
}
