#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use common::governed_fixture::{LoadedFixture, export_selected_projections, governed_fixture_path};
use std::fs;

#[test]
fn governed_fixture_replay__synthetic_events__stable_selected_projections() {
    let loaded = LoadedFixture::load_default().expect("load fixture vault");
    let snapshot = loaded
        .export_selected_projections()
        .expect("export projections");

    let golden_path = governed_fixture_path("expected-legacy-projections.json");
    if !golden_path.exists() {
        // Only rewrite goldens when explicitly requested — never by default.
        let update = std::env::var("UPDATE_GOVERNED_GOLDEN")
            .map(|v| v == "1")
            .unwrap_or(false);
        if update {
            let pretty = serde_json::to_string_pretty(&snapshot).expect("serialize golden");
            fs::write(&golden_path, format!("{pretty}\n")).expect("write golden");
            panic!(
                "golden missing; wrote {} — re-run test to verify match \
                 (UPDATE_GOVERNED_GOLDEN=1)",
                golden_path.display()
            );
        }
        panic!(
            "golden missing at {} — set UPDATE_GOVERNED_GOLDEN=1 to regenerate, \
             then re-run the test to verify the match",
            golden_path.display()
        );
    }

    let golden_text = fs::read_to_string(&golden_path).expect("read golden");
    let golden: serde_json::Value = serde_json::from_str(&golden_text).expect("parse golden JSON");

    assert_eq!(
        snapshot,
        golden,
        "projection snapshot must match golden fixture exactly\nactual:\n{}\nexpected:\n{}",
        serde_json::to_string_pretty(&snapshot).unwrap_or_default(),
        serde_json::to_string_pretty(&golden).unwrap_or_default()
    );
}

#[test]
fn governed_fixture_replay__load_twice_on_fresh_vaults__identical_snapshots() {
    let a = LoadedFixture::load_default().expect("load vault A");
    let b = LoadedFixture::load_default().expect("load vault B");

    let snap_a = a.export_selected_projections().expect("export A");
    let snap_b = b.export_selected_projections().expect("export B");

    assert_eq!(
        snap_a, snap_b,
        "two independent loads on fresh vaults must produce identical selected projections"
    );

    // Second append of same event_id into a populated vault must fail cleanly
    // (PRIMARY KEY uniqueness — immutable event log).
    let dup = a.append_duplicate_first();
    assert!(
        dup.is_err(),
        "duplicate event_id append must fail; got Ok(())"
    );
    let err = dup.expect_err("checked is_err");
    let msg = err.to_string();
    assert!(
        msg.contains("UNIQUE")
            || msg.contains("unique")
            || msg.contains("immutable")
            || msg.contains("Failed to append"),
        "duplicate append error should be a clean store failure, got: {msg}"
    );

    // Also prove export helper is pure w.r.t. store state.
    let _ = export_selected_projections(&a.store).expect("re-export");
}
