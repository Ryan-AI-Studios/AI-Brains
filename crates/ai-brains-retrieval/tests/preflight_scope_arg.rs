mod common;

use ai_brains_core::privacy::Privacy;
use ai_brains_retrieval::build_preflight;

/// Test that build_preflight accepts scope_paths and produces output
/// even when Ledgerful is unavailable (fail-open behavior).
#[test]
fn preflight_with_scope_paths_does_not_crash() -> Result<(), Box<dyn std::error::Error>> {
    let content = "CONSTRAINT: All public APIs must be versioned.";
    let store = common::store_with_memory(content, Privacy::CloudOk)?;

    let project_id = ai_brains_core::ids::ProjectId::from_uuid(uuid::Uuid::nil());
    let scope_paths = Some(vec![
        "crates/api/src/lib.rs".to_string(),
        "crates/api/src/handlers.rs".to_string(),
    ]);

    let context = build_preflight(
        store.connection(),
        None,
        1500,
        Some(project_id),
        scope_paths,
        false,
    )?;

    // Should produce output even without Ledgerful available (fail-open)
    assert!(!context.text.is_empty());
    assert!(context.word_count > 0);

    Ok(())
}

/// Test that preflight with None scope works identically to before.
#[test]
fn preflight_without_scope_still_works() -> Result<(), Box<dyn std::error::Error>> {
    let content = "INVARIANT: Database queries must use parameterized statements.";
    let store = common::store_with_memory(content, Privacy::CloudOk)?;

    let project_id = ai_brains_core::ids::ProjectId::from_uuid(uuid::Uuid::nil());

    let context = build_preflight(
        store.connection(),
        None,
        1500,
        Some(project_id),
        None,
        false,
    )?;

    assert!(!context.text.is_empty());
    assert!(context.word_count > 0);
    assert!(context.text.contains("INVARIANT"));

    Ok(())
}

/// Test that empty scope vec is treated the same as None.
#[test]
fn preflight_with_empty_scope_vec() -> Result<(), Box<dyn std::error::Error>> {
    let content = "Important memory content that should appear in preflight.";
    let store = common::store_with_memory(content, Privacy::CloudOk)?;

    let project_id = ai_brains_core::ids::ProjectId::from_uuid(uuid::Uuid::nil());

    // Empty vec should behave the same as None
    let context_none = build_preflight(
        store.connection(),
        None,
        1500,
        Some(project_id),
        None,
        false,
    )?;

    let context_empty = build_preflight(
        store.connection(),
        None,
        1500,
        Some(project_id),
        Some(vec![]),
        false,
    )?;

    // Both should contain the same memory content. Do not require byte-identical
    // relative ages ("just now" vs "1 min ago") — wall clock can advance between
    // the two build_preflight calls under load (unrelated flake under full suite).
    assert!(
        context_none.text.contains(content),
        "None scope missing memory: {}",
        context_none.text
    );
    assert!(
        context_empty.text.contains(content),
        "empty scope missing memory: {}",
        context_empty.text
    );
    let strip_ages = |s: &str| {
        s.replace("just now", "<age>")
            .replace("1 min ago", "<age>")
            .replace("2 min ago", "<age>")
    };
    assert_eq!(
        strip_ages(&context_none.text),
        strip_ages(&context_empty.text),
        "None vs empty scope should match aside from relative age wording"
    );

    Ok(())
}
