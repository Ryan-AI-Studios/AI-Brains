//! T271 — `sync query` ledger pane: query forwarder, miss copy, token rescue.
//!
//! Ledgerful `ledger search` already phrase-wraps the whole argv. Do not apply
//! T90 `sanitize_fts_query` here (that is vault MATCH only).

use std::path::Path;

/// Cap on token-rescue subprocesses after an empty phrase probe (F6 / F22).
pub(crate) const LEDGER_RESCUE_TOKEN_CAP: usize = 3;

/// First-line stderr budget (F19; same 140 as T250 `PRETTY_LINE_MAX`).
pub(crate) const LEDGER_STDERR_LINE_MAX: usize = 140;

/// F2 never-ran reason when cwd is System32 / SysWOW64.
pub(crate) const SYSTEM32_NEVER_RAN: &str =
    "cwd is a Windows system directory (not a git worktree). cd to the repo.";

/// Probe class after spawn / exit (F19). Success still splits Hits vs RanEmpty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LedgerOutcomeClass {
    NeverRan,
    Failed,
    Success,
}

/// Result of a `ledgerful ledger search --json` probe (T211 F12 / T271 / T313).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LedgerProbeResult {
    pub non_empty: bool,
    /// Human table, or a named miss line (F1).
    pub display: Option<String>,
    /// F7 banner only when token rescue produced hits.
    pub banner: Option<String>,
    /// T313: token that produced a rescue hit; drives the section heading.
    pub rescued_token: Option<String>,
}

/// T313 F1: ledger pane heading names the rescued token when present and non-blank.
pub(crate) fn ledger_section_heading(rescued_token: Option<&str>) -> String {
    match rescued_token {
        Some(tok) if !tok.trim().is_empty() => {
            format!("--- Ledgerful Ledger Search (rescued token: '{tok}') ---")
        }
        _ => "--- Ledgerful Ledger Search ---".to_string(),
    }
}

/// AC8 unit helper: heading, optional banner, optional display (no leading blank).
#[cfg(test)]
pub(crate) fn format_ledger_section_lines(section: &LedgerProbeResult) -> Vec<String> {
    let mut lines = vec![ledger_section_heading(section.rescued_token.as_deref())];
    if let Some(ref banner) = section.banner {
        lines.push(banner.clone());
    }
    if let Some(ref text) = section.display {
        lines.push(text.clone());
    }
    lines
}

/// Print the ledger pane (T313). Three `println!` matching today's spacing.
pub(crate) fn print_ledger_section(section: &LedgerProbeResult) {
    println!(
        "\n{}",
        ledger_section_heading(section.rescued_token.as_deref())
    );
    if let Some(ref banner) = section.banner {
        println!("{}", banner);
    }
    if let Some(ref text) = section.display {
        println!("{}", text);
    }
}

/// Strip ANSI and trim. Never FTS-quote (F5).
pub(crate) fn ledger_forward_query(raw: &str) -> String {
    ai_brains_retrieval::strip_ansi(raw).trim().to_string()
}

/// Argv for `ledgerful ledger search` (T273 F1/F2). Always inserts POSIX `--`
/// before QUERY so dash-leading needles are not Ledgerful flags.
pub(crate) fn ledger_search_argv(query: &str, json: bool) -> Vec<String> {
    let mut args = vec!["ledger".to_string(), "search".to_string()];
    if json {
        args.push("--json".to_string());
    }
    args.push("--".to_string());
    args.push(query.to_string());
    args
}

/// First-seen contentful tokens for rescue (F6). Not length-sorted.
pub(crate) fn ledger_rescue_tokens(user: &str) -> Vec<String> {
    use ai_brains_core::{contentful_tokens, extract_fts_tokens};
    contentful_tokens(&extract_fts_tokens(user))
}

/// First rescued token whose JSON is non-empty; None if phrase already hits.
/// Probe uses the same rule sequentially (stop at first hit; F6 / F22).
#[cfg(test)]
pub(crate) fn ledger_rescue_pick<'a>(
    phrase_json: &str,
    token_jsons: &[(&'a str, &'a str)],
) -> Option<&'a str> {
    if ledger_json_non_empty(phrase_json) || token_jsons.len() < 2 {
        return None;
    }
    for (token, json) in token_jsons.iter().take(LEDGER_RESCUE_TOKEN_CAP) {
        if ledger_json_non_empty(json) {
            return Some(*token);
        }
    }
    None
}

/// Ran-empty miss copy (F1). Must quote the **user** string, not T90 AND.
pub(crate) fn ledger_miss_copy_ran_empty(user: &str) -> String {
    format!("No ledger entries found matching '{user}'.")
}

/// Never-ran miss copy (F1 / F8 / F18).
pub(crate) fn ledger_miss_copy_never_ran(reason: &str) -> String {
    format!("Ledger search did not run: {reason}")
}

/// Failed miss copy (F19). Empty detail → locked `Ledger search failed.`
pub(crate) fn ledger_miss_copy_failed(detail: Option<&str>) -> String {
    match detail {
        Some(d) if !d.trim().is_empty() => format!("Ledger search failed: {d}"),
        _ => "Ledger search failed.".to_string(),
    }
}

/// Path suffix looks like Windows System32 / SysWOW64 (case-insensitive).
pub(crate) fn path_is_windows_system_dir(path: &Path) -> bool {
    let raw = path.to_string_lossy();
    let normalized = raw
        .trim_end_matches(['/', '\\'])
        .replace('/', "\\")
        .to_ascii_lowercase();
    normalized.ends_with(r"\windows\system32") || normalized.ends_with(r"\windows\syswow64")
}

/// F2: production guard. Always false on non-Windows (skip this arm).
pub(crate) fn is_windows_system_cwd(path: &Path) -> bool {
    cfg!(windows) && path_is_windows_system_dir(path)
}

/// F19 pure classifier (no subprocess).
pub(crate) fn ledger_classify_outcome(
    spawn_ok: bool,
    exit_success: bool,
    stderr: &str,
) -> LedgerOutcomeClass {
    if !spawn_ok {
        return LedgerOutcomeClass::NeverRan;
    }
    if exit_success {
        return LedgerOutcomeClass::Success;
    }
    let lower = stderr.to_ascii_lowercase();
    if lower.contains("git") || lower.contains("work directory") || lower.contains("layout") {
        LedgerOutcomeClass::NeverRan
    } else {
        LedgerOutcomeClass::Failed
    }
}

/// F8: `--quiet` omits never-ran/failed; hits and ran-empty still print.
pub(crate) fn ledger_quiet_omits_pane(quiet: bool, class: LedgerOutcomeClass) -> bool {
    quiet
        && matches!(
            class,
            LedgerOutcomeClass::NeverRan | LedgerOutcomeClass::Failed
        )
}

/// First stderr line, then cap at [`LEDGER_STDERR_LINE_MAX`] chars (F19).
pub(crate) fn ledger_first_stderr_line(stderr: &str) -> String {
    let first = match stderr.lines().next() {
        Some(line) => line.trim(),
        None => return String::new(),
    };
    if first.is_empty() {
        return String::new();
    }
    if first.chars().count() <= LEDGER_STDERR_LINE_MAX {
        first.to_string()
    } else {
        first.chars().take(LEDGER_STDERR_LINE_MAX).collect()
    }
}

/// F7 banner. User/token are stripped strings with no added FTS quotes.
pub(crate) fn ledger_rescue_banner(user: &str, token: &str) -> String {
    format!("Note: no phrase match for '{user}'; showing hits for '{token}'.")
}

#[allow(clippy::disallowed_methods)]
fn run_ledger_search(
    query: &str,
    json: bool,
    quiet: bool,
    is_tty: bool,
) -> std::io::Result<std::process::Output> {
    let mut cmd = std::process::Command::new("ledgerful");
    cmd.args(ledger_search_argv(query, json));
    if !is_tty {
        cmd.env("NO_COLOR", "1");
    }
    if quiet {
        cmd.stderr(std::process::Stdio::null());
    }
    cmd.output()
}

fn human_ledger_display(
    query: &str,
    quiet: bool,
    is_tty: bool,
    json_stdout: &str,
) -> Option<String> {
    match run_ledger_search(query, false, quiet, is_tty) {
        Ok(out) if out.status.success() => {
            let s = String::from_utf8_lossy(&out.stdout).into_owned();
            let s = if is_tty {
                s
            } else {
                ai_brains_retrieval::strip_ansi(&s)
            };
            if s.trim().is_empty() { None } else { Some(s) }
        }
        _ => {
            let s = if is_tty {
                json_stdout.to_string()
            } else {
                ai_brains_retrieval::strip_ansi(json_stdout)
            };
            if s.trim().is_empty() { None } else { Some(s) }
        }
    }
}

fn miss_from_nonzero(stderr: &str) -> String {
    let class = ledger_classify_outcome(true, false, stderr);
    let line = ledger_first_stderr_line(stderr);
    match class {
        LedgerOutcomeClass::NeverRan => {
            if line.is_empty() {
                ledger_miss_copy_failed(None)
            } else {
                ledger_miss_copy_never_ran(line.as_str())
            }
        }
        LedgerOutcomeClass::Failed | LedgerOutcomeClass::Success => {
            let detail = if line.is_empty() {
                None
            } else {
                Some(line.as_str())
            };
            ledger_miss_copy_failed(detail)
        }
    }
}

/// Probe ledger JSON; rescue tokens on empty phrase; named miss otherwise (T271).
#[allow(clippy::disallowed_methods)]
pub(crate) fn probe_ledger_search(query: &str, quiet: bool) -> Option<LedgerProbeResult> {
    use std::io::IsTerminal;
    let is_tty = std::io::stdout().is_terminal();

    if let Ok(cwd) = std::env::current_dir()
        && is_windows_system_cwd(&cwd)
    {
        if ledger_quiet_omits_pane(quiet, LedgerOutcomeClass::NeverRan) {
            return None;
        }
        return Some(LedgerProbeResult {
            non_empty: false,
            display: Some(ledger_miss_copy_never_ran(SYSTEM32_NEVER_RAN)),
            banner: None,
            rescued_token: None,
        });
    }

    let forward = ledger_forward_query(query);
    if forward.is_empty() {
        if ledger_quiet_omits_pane(quiet, LedgerOutcomeClass::NeverRan) {
            return None;
        }
        return Some(LedgerProbeResult {
            non_empty: false,
            display: Some(ledger_miss_copy_never_ran("query is empty.")),
            banner: None,
            rescued_token: None,
        });
    }

    let json_output = match run_ledger_search(&forward, true, quiet, is_tty) {
        Ok(out) => out,
        Err(_) => {
            if !quiet {
                tracing::info!("ledgerful CLI not found or failed to execute.");
            }
            if ledger_quiet_omits_pane(quiet, LedgerOutcomeClass::NeverRan) {
                return None;
            }
            return Some(LedgerProbeResult {
                non_empty: false,
                display: Some(ledger_miss_copy_never_ran("ledgerful CLI not found.")),
                banner: None,
                rescued_token: None,
            });
        }
    };

    if !json_output.status.success() {
        let stderr = String::from_utf8_lossy(&json_output.stderr);
        let class = ledger_classify_outcome(true, false, &stderr);
        if ledger_quiet_omits_pane(quiet, class) {
            return None;
        }
        return Some(LedgerProbeResult {
            non_empty: false,
            display: Some(miss_from_nonzero(&stderr)),
            banner: None,
            rescued_token: None,
        });
    }

    let stdout = String::from_utf8_lossy(&json_output.stdout);
    if ledger_json_non_empty(&stdout) {
        return Some(LedgerProbeResult {
            non_empty: true,
            display: human_ledger_display(&forward, quiet, is_tty, &stdout),
            banner: None,
            rescued_token: None,
        });
    }

    let tokens = ledger_rescue_tokens(&forward);
    if tokens.len() >= 2 {
        // Spec §5.2: only a JSON hit rescues. Token spawn/nonzero continues;
        // phrase already succeeded empty, so the pane class stays ran-empty.
        for token in tokens.iter().take(LEDGER_RESCUE_TOKEN_CAP) {
            let tok_out = match run_ledger_search(token, true, quiet, is_tty) {
                Ok(out) if out.status.success() => out,
                _ => continue,
            };
            let tok_stdout = String::from_utf8_lossy(&tok_out.stdout);
            if ledger_json_non_empty(&tok_stdout) {
                return Some(LedgerProbeResult {
                    non_empty: true,
                    display: human_ledger_display(token, quiet, is_tty, &tok_stdout),
                    banner: Some(ledger_rescue_banner(&forward, token)),
                    rescued_token: Some(token.clone()),
                });
            }
        }
    }

    Some(LedgerProbeResult {
        non_empty: false,
        display: Some(ledger_miss_copy_ran_empty(&forward)),
        banner: None,
        rescued_token: None,
    })
}

/// F12 non-empty detection: JSON array/object with ≥1 entry OR ≥1 NDJSON line.
pub(crate) fn ledger_json_non_empty(stdout: &str) -> bool {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return false;
    }

    if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
        return match v {
            serde_json::Value::Array(a) => !a.is_empty(),
            serde_json::Value::Object(o) => !o.is_empty(),
            _ => false,
        };
    }

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            match v {
                serde_json::Value::Array(a) if !a.is_empty() => return true,
                serde_json::Value::Object(o) if !o.is_empty() => return true,
                serde_json::Value::Null
                | serde_json::Value::Bool(_)
                | serde_json::Value::Number(_) => {
                    continue;
                }
                serde_json::Value::String(s) if s.is_empty() => continue,
                serde_json::Value::String(_) => return true,
                serde_json::Value::Array(_) | serde_json::Value::Object(_) => continue,
            }
        }
    }
    false
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::{
        LEDGER_STDERR_LINE_MAX, LedgerOutcomeClass, LedgerProbeResult, SYSTEM32_NEVER_RAN,
        format_ledger_section_lines, is_windows_system_cwd, ledger_classify_outcome,
        ledger_first_stderr_line, ledger_forward_query, ledger_json_non_empty,
        ledger_miss_copy_failed, ledger_miss_copy_never_ran, ledger_miss_copy_ran_empty,
        ledger_quiet_omits_pane, ledger_rescue_banner, ledger_rescue_pick, ledger_rescue_tokens,
        ledger_search_argv, ledger_section_heading, path_is_windows_system_dir,
    };
    use std::path::Path;

    /// T313 AC1: rescued token appears in the ledger section heading.
    #[test]
    #[allow(non_snake_case)]
    fn ledger_section_heading__rescued_token__names_token() {
        let heading = ledger_section_heading(Some("graph"));
        assert_eq!(
            heading,
            "--- Ledgerful Ledger Search (rescued token: 'graph') ---"
        );
        assert!(heading.contains("rescued token"));
        assert!(heading.contains("'graph'"));
    }

    /// T313 AC2: phrase-hit / miss heading stays generic (no rescued).
    #[test]
    #[allow(non_snake_case)]
    fn ledger_section_heading__phrase_hit__generic() {
        let heading = ledger_section_heading(None);
        assert_eq!(heading, "--- Ledgerful Ledger Search ---");
        assert!(!heading.contains("rescued"));
    }

    /// T313 AC3 / F25: empty or whitespace-only rescued token → generic heading.
    #[test]
    #[allow(non_snake_case)]
    fn ledger_section_heading__empty_token__generic() {
        assert_eq!(
            ledger_section_heading(Some("")),
            "--- Ledgerful Ledger Search ---"
        );
        assert_eq!(
            ledger_section_heading(Some("   ")),
            "--- Ledgerful Ledger Search ---"
        );
        assert!(!ledger_section_heading(Some("")).contains("rescued"));
        assert!(!ledger_section_heading(Some("   ")).contains("rescued"));
    }

    /// T313 AC8: rescued lines are heading → F7 banner → display; phrase-hit has no banner.
    #[test]
    #[allow(non_snake_case)]
    fn format_ledger_section_lines__rescued__heading_then_banner() {
        let rescued = LedgerProbeResult {
            non_empty: true,
            display: Some("10 matching entries for 'graph':".to_string()),
            banner: Some(ledger_rescue_banner("graph backend", "graph")),
            rescued_token: Some("graph".to_string()),
        };
        let lines = format_ledger_section_lines(&rescued);
        assert_eq!(
            lines[0],
            "--- Ledgerful Ledger Search (rescued token: 'graph') ---"
        );
        assert_eq!(
            lines[1],
            "Note: no phrase match for 'graph backend'; showing hits for 'graph'."
        );
        assert_eq!(lines[2], "10 matching entries for 'graph':");
        assert_eq!(lines.len(), 3);

        let phrase = LedgerProbeResult {
            non_empty: true,
            display: Some("3 matching entries for 'T314':".to_string()),
            banner: None,
            rescued_token: None,
        };
        let phrase_lines = format_ledger_section_lines(&phrase);
        assert_eq!(phrase_lines[0], "--- Ledgerful Ledger Search ---");
        assert!(!phrase_lines.iter().any(|l| l.contains("no phrase match")));
        assert_eq!(phrase_lines[1], "3 matching entries for 'T314':");
        assert_eq!(phrase_lines.len(), 2);
    }

    /// T273 AC1: JSON argv always inserts `--` immediately before a dash needle.
    #[test]
    #[allow(non_snake_case)]
    fn ledger_search_argv__json_dash_limit__end_of_options_before_query() {
        assert_eq!(
            ledger_search_argv("--limit", true),
            vec!["ledger", "search", "--json", "--", "--limit"]
        );
    }

    /// T273 AC2: human re-run omits `--json` but still ends options before QUERY.
    #[test]
    #[allow(non_snake_case)]
    fn ledger_search_argv__human_dash_limit__no_json_flag() {
        assert_eq!(
            ledger_search_argv("--limit", false),
            vec!["ledger", "search", "--", "--limit"]
        );
    }

    /// T273 AC3: non-dash phrases still get always-on `--`.
    #[test]
    #[allow(non_snake_case)]
    fn ledger_search_argv__plain_phrase__still_emits_double_dash() {
        let got = ledger_search_argv("capture independence", true);
        assert_eq!(
            got,
            vec!["ledger", "search", "--json", "--", "capture independence"]
        );
        let query_at = got.len() - 1;
        assert_eq!(
            got.get(query_at.saturating_sub(1)).map(String::as_str),
            Some("--")
        );
        assert_eq!(got.last().map(String::as_str), Some("capture independence"));
    }

    /// T273 AC4: `--days` is QUERY, not Ledgerful `-d/--days`.
    #[test]
    #[allow(non_snake_case)]
    fn ledger_search_argv__json_dash_days__needle_after_terminator() {
        assert_ledger_search_argv_needle("--days");
    }

    /// T273 AC4: `--breaking` is QUERY, not Ledgerful `-b/--breaking`.
    #[test]
    #[allow(non_snake_case)]
    fn ledger_search_argv__json_dash_breaking__needle_after_terminator() {
        assert_ledger_search_argv_needle("--breaking");
    }

    /// T273 AC4: `--json` as QUERY is not a second Ledgerful `--json`.
    #[test]
    #[allow(non_snake_case)]
    fn ledger_search_argv__json_dash_json__needle_after_terminator() {
        assert_ledger_search_argv_needle("--json");
    }

    /// T273 AC4: short `-l` is QUERY, not Ledgerful `--limit`.
    #[test]
    #[allow(non_snake_case)]
    fn ledger_search_argv__json_short_l__needle_after_terminator() {
        assert_ledger_search_argv_needle("-l");
    }

    /// T273 AC4: short `-d` is QUERY, not Ledgerful `--days`.
    #[test]
    #[allow(non_snake_case)]
    fn ledger_search_argv__json_short_d__needle_after_terminator() {
        assert_ledger_search_argv_needle("-d");
    }

    /// T273 AC4: short `-b` is QUERY, not Ledgerful `--breaking`.
    #[test]
    #[allow(non_snake_case)]
    fn ledger_search_argv__json_short_b__needle_after_terminator() {
        assert_ledger_search_argv_needle("-b");
    }

    /// T273 AC4 / F19: needle `"--"` is last; terminator sits immediately before it.
    #[test]
    #[allow(non_snake_case)]
    fn ledger_search_argv__json_double_dash_needle__terminator_then_needle() {
        let got = ledger_search_argv("--", true);
        assert_eq!(got, vec!["ledger", "search", "--json", "--", "--"]);
        assert_eq!(got.last().map(String::as_str), Some("--"));
        assert_ne!(
            got.get(got.len().saturating_sub(2)).map(String::as_str),
            Some("search"),
            "needle must not sit next to search without a terminator"
        );
    }

    fn assert_ledger_search_argv_needle(needle: &str) {
        let got = ledger_search_argv(needle, true);
        assert_eq!(
            got.last().map(String::as_str),
            Some(needle),
            "last argv must be the needle {needle:?}; got {got:?}"
        );
        let term_at = got.len().saturating_sub(2);
        assert_eq!(
            got.get(term_at).map(String::as_str),
            Some("--"),
            "option-terminator must sit immediately before {needle:?}; got {got:?}"
        );
        assert_ne!(
            got.get(got.len().saturating_sub(2))
                .map(String::as_str)
                .filter(|s| *s != "--"),
            Some("search"),
            "needle must not be adjacent to search without terminator; got {got:?}"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn ledger_forward_query__user_phrase__not_fts_quoted() {
        let got = ledger_forward_query("capture independence");
        assert_eq!(got, "capture independence");
        assert_ne!(got, r#""capture" "independence""#);
        assert!(!got.contains('"'));
    }

    #[test]
    #[allow(non_snake_case)]
    fn ledger_forward_query__empty__returns_empty() {
        assert_eq!(ledger_forward_query(""), "");
        assert_eq!(ledger_forward_query("   "), "");
        assert_eq!(ledger_forward_query("\n"), "");
        assert_eq!(ledger_forward_query(" \t\n "), "");
    }

    #[test]
    #[allow(non_snake_case)]
    fn ledger_forward_query__ansi_stripped() {
        let colored = "\u{1b}[31mcapture independence\u{1b}[0m";
        assert_eq!(ledger_forward_query(colored), "capture independence");
    }

    #[test]
    #[allow(non_snake_case)]
    fn ledger_rescue_tokens__capture_independence__first_seen_capture() {
        assert_eq!(
            ledger_rescue_tokens("capture independence"),
            vec!["capture".to_string(), "independence".to_string()]
        );
        assert_ne!(
            ledger_rescue_tokens("capture independence"),
            vec!["independence".to_string(), "capture".to_string()]
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn ledger_rescue_pick__first_token_empty_second_hits__selects_second() {
        let picked = ledger_rescue_pick(
            "[]",
            &[("capture", "[]"), ("independence", r#"[{"id":1}]"#)],
        );
        assert_eq!(picked, Some("independence"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn ledger_miss_copy__ran_empty__uses_user_query_not_quotes() {
        let copy = ledger_miss_copy_ran_empty("capture independence");
        assert!(
            copy.contains("capture independence"),
            "ran-empty must quote the user query; got {copy}"
        );
        assert!(
            !copy.contains(r#"'"capture" "independence"'"#),
            "must not print T90-quoted needle; got {copy}"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn is_windows_system_cwd__system32_and_syswow64__true() {
        assert!(path_is_windows_system_dir(Path::new(
            r"C:\Windows\System32"
        )));
        assert!(path_is_windows_system_dir(Path::new(
            r"C:\Windows\SysWOW64"
        )));
        assert!(path_is_windows_system_dir(Path::new(
            r"c:\windows\system32"
        )));
        assert!(path_is_windows_system_dir(Path::new(
            r"C:\Windows\System32\"
        )));
        assert!(!path_is_windows_system_dir(Path::new(r"C:\dev\AI-Brains")));
        assert!(!path_is_windows_system_dir(Path::new(
            r"C:\Windows\System32\Wbem"
        )));
        assert_eq!(
            is_windows_system_cwd(Path::new(r"C:\Windows\System32")),
            cfg!(windows)
        );
        assert_eq!(
            ledger_miss_copy_never_ran(SYSTEM32_NEVER_RAN),
            "Ledger search did not run: cwd is a Windows system directory (not a git worktree). cd to the repo."
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn ledger_quiet_omits_pane__never_ran_failed_only() {
        assert!(ledger_quiet_omits_pane(true, LedgerOutcomeClass::NeverRan));
        assert!(ledger_quiet_omits_pane(true, LedgerOutcomeClass::Failed));
        assert!(!ledger_quiet_omits_pane(true, LedgerOutcomeClass::Success));
        assert!(!ledger_quiet_omits_pane(
            false,
            LedgerOutcomeClass::NeverRan
        ));
        assert!(!ledger_quiet_omits_pane(false, LedgerOutcomeClass::Failed));
    }

    #[test]
    #[allow(non_snake_case)]
    fn ledger_rescue_pick__single_token_hit__does_not_rescue() {
        assert_eq!(
            ledger_rescue_pick("[]", &[("capture", r#"[{"id":1}]"#)]),
            None
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn ledger_miss_copy__never_ran__did_not_run() {
        let copy = ledger_miss_copy_never_ran("ledgerful CLI not found.");
        assert!(
            copy.to_ascii_lowercase().contains("did not run"),
            "never-ran must say did not run; got {copy}"
        );
        assert!(
            !copy.contains("No ledger entries found matching"),
            "never-ran must not look like ran-empty; got {copy}"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn ledger_miss_copy__empty_query__did_not_run() {
        let copy = ledger_miss_copy_never_ran("query is empty.");
        assert!(
            copy.contains("query is empty"),
            "empty query copy must name the reason; got {copy}"
        );
        assert!(
            copy.to_ascii_lowercase().contains("did not run"),
            "empty query is never-ran; got {copy}"
        );
        assert!(
            !copy.contains("No ledger entries found matching"),
            "must not look like ran-empty; got {copy}"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn ledger_classify_outcome__nonzero_git_stderr__never_ran() {
        let class =
            ledger_classify_outcome(true, false, "Failed to find work directory for repository");
        assert_eq!(class, LedgerOutcomeClass::NeverRan);
    }

    #[test]
    #[allow(non_snake_case)]
    fn ledger_classify_outcome__nonzero_other_stderr__failed() {
        let stderr = "fts5: syntax error near \".\"";
        let class = ledger_classify_outcome(true, false, stderr);
        assert_eq!(class, LedgerOutcomeClass::Failed);
        let line = ledger_first_stderr_line(stderr);
        assert_eq!(line, stderr);
        assert!(line.chars().count() <= LEDGER_STDERR_LINE_MAX);

        let long: String = std::iter::repeat_n('x', 200).collect();
        let capped = ledger_first_stderr_line(&format!("{long}\nsecond line"));
        assert_eq!(capped.chars().count(), LEDGER_STDERR_LINE_MAX);
        assert_eq!(capped, "x".repeat(LEDGER_STDERR_LINE_MAX));
        let copy = ledger_miss_copy_failed(Some(&capped));
        assert!(copy.contains("failed") || copy.contains("Failed"));
        assert!(!copy.contains(&"x".repeat(141)));
    }

    #[test]
    #[allow(non_snake_case)]
    fn ledger_rescue_banner__phrase_empty_token_hit__locked_sentence() {
        assert_eq!(
            ledger_rescue_banner("capture independence", "capture"),
            "Note: no phrase match for 'capture independence'; showing hits for 'capture'."
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn ledger_json_non_empty__array_with_item() {
        assert!(ledger_json_non_empty(r#"[{"id":1}]"#));
    }

    #[test]
    #[allow(non_snake_case)]
    fn ledger_json_non_empty__empty_array() {
        assert!(!ledger_json_non_empty("[]"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn ledger_json_non_empty__ndjson_object_line() {
        assert!(ledger_json_non_empty("{\"a\":1}\n"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn ledger_json_non_empty__blank() {
        assert!(!ledger_json_non_empty("  \n"));
    }
}
