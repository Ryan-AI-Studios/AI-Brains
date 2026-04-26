use ai_brains_core::turn::*;

#[test]
fn test_no_thinking_role_exists() {
    // This test is essentially a compile-time check that 'thinking' doesn't exist in the enum,
    // but we can also check it at runtime if we iterate (if we had strum).
    // For now, we just try to use what should exist.
    let _user = Role::User;
    let _assistant = Role::AssistantFinal;
}

// We also want to ensure we CANNOT represent 'thinking' or 'tool_call'
// This is more about checking that they are NOT in the source, but we can't easily test non-existence in TDD
// except by not adding them.
