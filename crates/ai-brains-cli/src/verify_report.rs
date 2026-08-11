//! T225 — pure formatters for quiet-by-default `backup verify` human output.
//!
//! Integrity and exit codes stay in `commands/backup.rs`. This module only
//! shapes counts, FAIL previews, create-nudge predicate, and optional class rollup.

/// Default cap for FAIL detail lines under human quiet mode (not CLI-configurable).
pub const VERIFY_FAIL_PREVIEW_CAP: usize = 5;

/// Summary counts line for default human verify output.
///
/// SOOT: `Verified {total} backup(s): {ok} OK, {fail} FAIL.`
/// Uses singular `backup` when `total == 1`.
pub fn format_verify_counts(total: usize, ok: usize, fail: usize) -> String {
    let unit = if total == 1 { "backup" } else { "backups" };
    format!("Verified {total} {unit}: {ok} OK, {fail} FAIL.")
}

/// First `cap` FAIL detail lines plus optional trailer when more remain.
///
/// Each fail is `(filename, reason)`. Lines: `{name}: FAIL — {reason}`.
/// Trailer (when `fails.len() > cap`) contains `and`, `more`, and `--verbose`.
pub fn format_fail_preview(
    fails: &[(String, String)],
    cap: usize,
) -> (Vec<String>, Option<String>) {
    let preview_n = fails.len().min(cap);
    let lines: Vec<String> = fails
        .iter()
        .take(preview_n)
        .map(|(name, reason)| format!("{name}: FAIL — {reason}"))
        .collect();

    let trailer = if fails.len() > cap {
        let more = fails.len() - cap;
        Some(format!(
            "… and {more} more FAIL (use --verbose for full list)."
        ))
    } else {
        None
    };

    (lines, trailer)
}

/// Verify human-default create nudge: only when zero OK among discovered backups.
///
/// Does **not** age-check; doctor owns stale-usable (T225 F8/M4).
pub fn should_emit_create_nudge(ok: usize, total: usize) -> bool {
    ok == 0 && total >= 1
}

/// Create-nudge body for human verify when [`should_emit_create_nudge`] is true.
///
/// Must contain the substring `ai-brains backup create`.
pub fn format_create_nudge() -> String {
    "No usable encrypted backup under current key. Run: ai-brains backup create".to_string()
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn format_verify_counts__zero_fail__includes_zero_fail_and_ok() {
        let s = format_verify_counts(2, 2, 0);
        assert!(s.contains("2"), "total: {s}");
        assert!(s.contains("2 OK"), "{s}");
        assert!(s.contains("0 FAIL"), "all-OK SOOT must include 0 FAIL: {s}");
        assert!(s.contains("backups"), "{s}");
    }

    #[test]
    fn format_verify_counts__total_one__singular_backup() {
        let s = format_verify_counts(1, 1, 0);
        assert!(s.contains("1 backup:"), "singular polish expected: {s}");
        assert!(!s.contains("1 backups"), "{s}");
        assert!(s.contains("1 OK"), "{s}");
        assert!(s.contains("0 FAIL"), "{s}");
    }

    #[test]
    fn format_fail_preview__zero_fail__empty_lines_no_trailer() {
        let (lines, trailer) = format_fail_preview(&[], VERIFY_FAIL_PREVIEW_CAP);
        assert!(lines.is_empty());
        assert!(trailer.is_none());
    }

    #[test]
    fn format_fail_preview__three_fail__three_lines_no_trailer() {
        let fails: Vec<(String, String)> = (1..=3)
            .map(|i| (format!("vault-{i}.db.bak"), format!("reason-{i}")))
            .collect();
        let (lines, trailer) = format_fail_preview(&fails, VERIFY_FAIL_PREVIEW_CAP);
        assert_eq!(lines.len(), 3, "{lines:?}");
        assert!(trailer.is_none(), "no trailer when fail <= cap");
        assert!(
            lines[0].contains("FAIL —"),
            "T138 reason form: {}",
            lines[0]
        );
        assert!(lines[0].contains("reason-1"), "{}", lines[0]);
    }

    #[test]
    fn format_fail_preview__six_fail__five_detail_plus_trailer() {
        let fails: Vec<(String, String)> = (1..=6)
            .map(|i| (format!("vault-{i}.db.bak"), format!("reason-{i}")))
            .collect();
        let (lines, trailer) = format_fail_preview(&fails, VERIFY_FAIL_PREVIEW_CAP);
        assert_eq!(
            lines.len(),
            VERIFY_FAIL_PREVIEW_CAP,
            "preview must cap at {VERIFY_FAIL_PREVIEW_CAP}"
        );
        let t = trailer.expect("trailer when fail > cap");
        assert!(t.contains("and"), "trailer must contain 'and': {t}");
        assert!(t.contains("more"), "trailer must contain 'more': {t}");
        assert!(
            t.contains("--verbose"),
            "trailer must contain '--verbose': {t}"
        );
        assert!(t.contains('1'), "one more fail: {t}");
    }

    #[test]
    fn should_emit_create_nudge__ok_zero_total_positive__true() {
        assert!(should_emit_create_nudge(0, 1));
        assert!(should_emit_create_nudge(0, 21));
    }

    #[test]
    fn should_emit_create_nudge__ok_positive__false() {
        assert!(!should_emit_create_nudge(1, 1));
        assert!(!should_emit_create_nudge(1, 5));
        assert!(!should_emit_create_nudge(5, 5));
    }

    #[test]
    fn should_emit_create_nudge__empty_total__false() {
        assert!(!should_emit_create_nudge(0, 0));
    }

    #[test]
    fn format_create_nudge__contains_backup_create() {
        let s = format_create_nudge();
        assert!(
            s.contains("ai-brains backup create"),
            "nudge must cite create SOOT: {s}"
        );
    }

    #[test]
    fn verify_fail_preview_cap__is_five() {
        assert_eq!(VERIFY_FAIL_PREVIEW_CAP, 5);
    }
}
