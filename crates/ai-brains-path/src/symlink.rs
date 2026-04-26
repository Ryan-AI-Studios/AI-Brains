use crate::unc;
use crate::windows;
use std::path::Path;

pub fn resolve_best_effort(input: &str) -> String {
    let path = Path::new(input);
    match path.canonicalize() {
        Ok(resolved) => {
            let resolved_str = resolved.to_string_lossy();
            let stripped = windows::strip_extended_length_prefix(&resolved_str);
            if unc::is_unc_path(&stripped) {
                unc::normalize_unc(&stripped)
            } else {
                windows::normalize_drive_path(&stripped).unwrap_or(stripped)
            }
        }
        Err(_) => input.to_string(),
    }
}
