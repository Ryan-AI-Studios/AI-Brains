//! T204 — CLI help information architecture hermetic locks (AC1–AC7, AC11).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use std::process::Output;

fn help_stdout(args: &[&str]) -> String {
    let mut cmd = common::hermetic_bin();
    for a in args {
        cmd.arg(a);
    }
    let out: Output = cmd.output().expect("help command must spawn");
    assert!(
        out.status.success(),
        "help must exit 0; args={args:?} stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

// ---------------------------------------------------------------------------
// AC1 / AC4 — long help group labels
// ---------------------------------------------------------------------------

#[test]
fn long_help__contains_required_group_labels() {
    let help = help_stdout(&["--help"]);
    for label in ["Daily", "Operator", "Governed", "Dangerous", "Harness"] {
        assert!(
            help.contains(label),
            "long --help must contain group label {label:?}\n--- help ---\n{help}"
        );
    }
    // Soft F34
    assert!(
        help.contains("Setup"),
        "long --help should contain Setup (soft)"
    );
}

// ---------------------------------------------------------------------------
// AC11 — stop-session under Daily inventory
// ---------------------------------------------------------------------------

#[test]
fn long_help__daily_inventory_includes_stop_session() {
    let help = help_stdout(&["--help"]);
    assert!(
        help.contains("stop-session"),
        "long help must mention stop-session"
    );
    // Prefer the Daily line shape from ROOT_AFTER_LONG_HELP
    assert!(
        help.contains("Daily:") && help.contains("stop-session"),
        "stop-session should appear with Daily group context\n--- help ---\n{help}"
    );
}

// ---------------------------------------------------------------------------
// AC2 + F33 — [dangerous] markers on full erase/rotate/apply class set
// ---------------------------------------------------------------------------

#[test]
fn subcommand_help__f33_dangerous_markers_present() {
    // F33: markers at the depth where danger lives (not appendix-only).
    let surfaces: &[(&[&str], &str)] = &[
        (&["forget", "--help"], "forget"),
        (&["erasure", "--help"], "erasure"),
        (&["erasure", "wipe", "--help"], "erasure wipe"),
        (&["retention", "apply", "--help"], "retention apply"),
        (&["vault", "encrypt", "--help"], "vault encrypt"),
        (
            &["vault", "rotate-datakey", "--help"],
            "vault rotate-datakey",
        ),
        (&["migrate", "governed", "--help"], "migrate governed"),
        (&["daemon", "install", "--help"], "daemon install"),
        (&["daemon", "uninstall", "--help"], "daemon uninstall"),
        (&["daemon", "update", "--help"], "daemon update"),
    ];
    for (args, label) in surfaces {
        let help = help_stdout(args);
        assert!(
            help.contains("[dangerous]"),
            "F33: {label} help must contain [dangerous]\n--- help ---\n{help}"
        );
    }
}

// ---------------------------------------------------------------------------
// AC5 — no rename: recall / doctor / erasure still parse
// ---------------------------------------------------------------------------

#[test]
fn known_commands__still_parse_via_help() {
    for cmd in ["recall", "doctor", "erasure"] {
        let help = help_stdout(&[cmd, "--help"]);
        assert!(
            help.contains("Usage:") || help.to_ascii_lowercase().contains(cmd),
            "command {cmd} must still parse and render help\n--- help ---\n{help}"
        );
    }
}

// ---------------------------------------------------------------------------
// AC6 — query progressive / expand / parent after_help project-id ceremony
// ---------------------------------------------------------------------------

#[test]
fn query_help__includes_project_id_ceremony() {
    for args in [
        &["query", "--help"][..],
        &["query", "progressive", "--help"][..],
        &["query", "expand", "--help"][..],
    ] {
        let help = help_stdout(args);
        let has_flag = help.contains("--project-id");
        let has_env = help.contains("AI_BRAINS_PROJECT_ID");
        assert!(
            has_flag || has_env,
            "query help {args:?} must mention --project-id and/or AI_BRAINS_PROJECT_ID\n--- help ---\n{help}"
        );
    }
}

// ---------------------------------------------------------------------------
// AC7 / F31 — Daily appears before early-declared Harness (`ingest`) in Commands list
// ---------------------------------------------------------------------------

#[test]
fn long_help__daily_commands_before_harness_ingest() {
    let help = help_stdout(&["--help"]);
    // F31 reordering proof: without display_order, enum order puts `ingest` before
    // Daily verbs. Assert Commands-list style lines (`  recall`) so appendix text
    // (which also names these verbs) does not satisfy the lock.
    let commands_idx = help
        .find("Commands:")
        .expect("long help must contain Commands:");
    let commands = &help[commands_idx..];
    // Stop before after_long_help appendix if present
    let commands = commands.split("Command groups").next().unwrap_or(commands);

    let daily_pos = ["  recall", "  doctor", "  preflight"]
        .iter()
        .filter_map(|s| commands.find(s))
        .min();
    let ingest_pos = commands.find("  ingest");
    let evaluate_pos = commands.find("  evaluate");

    let daily = daily_pos.unwrap_or_else(|| {
        panic!("could not locate Daily command line in Commands list\n--- commands ---\n{commands}")
    });
    let ingest = ingest_pos.unwrap_or_else(|| {
        panic!("could not locate `  ingest` in Commands list\n--- commands ---\n{commands}")
    });
    assert!(
        daily < ingest,
        "Daily (pos {daily}) must appear before Harness ingest (pos {ingest}) — F31 display_order"
    );
    if let Some(ev) = evaluate_pos {
        assert!(
            daily < ev,
            "Daily (pos {daily}) must appear before evaluate (pos {ev})"
        );
    }
}

// ---------------------------------------------------------------------------
// Soft AC10 — short -h tip only; no full group inventory wall
// ---------------------------------------------------------------------------

#[test]
fn short_help__tip_without_full_group_wall() {
    let short = help_stdout(&["-h"]);
    let long = help_stdout(&["--help"]);
    assert!(
        short.contains("command groups") || short.contains("Daily"),
        "short help should tip toward long help groups\n--- short ---\n{short}"
    );
    // F5/M5: full after_long_help group appendix is long-only. Commands list still
    // enumerates all verbs on both -h and --help (expected).
    assert!(
        !short.contains("Command groups (presentation only"),
        "short -h must not embed full after_long_help inventory wall"
    );
    assert!(
        !short.contains("Start here:"),
        "short -h must not embed Start here block from after_long_help"
    );
    assert!(
        long.contains("Command groups (presentation only") && long.contains("Start here:"),
        "long --help must include full group appendix"
    );
    assert!(
        short.len() < long.len(),
        "short -h ({} bytes) should be shorter than long --help ({} bytes)",
        short.len(),
        long.len()
    );
}
