pub fn apply_idempotent_patch(existing: &str, patch: &str) -> String {
    if existing.contains(patch) {
        existing.to_string()
    } else if existing.is_empty() {
        patch.to_string()
    } else {
        format!("{existing}\n{patch}")
    }
}
