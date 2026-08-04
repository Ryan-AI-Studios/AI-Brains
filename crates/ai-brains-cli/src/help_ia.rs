//! T204 — CLI help information architecture (presentation only).
//!
//! Root long/short help appendix strings and shared labels. Command names are
//! unchanged; grouping is via `after_long_help` + `display_order` only.

/// Normative long-help appendix (F32 / §12.1). Shown only with `--help`.
pub const ROOT_AFTER_LONG_HELP: &str = "\
Command groups (presentation only — names unchanged):

  Setup:     init
  Daily:     recall, preflight, doctor, project, pin, context, stop-session, daemon
  Operator:  backup, recovery, vault, retention, device, replicate, nightly, safety
  Governed:  scope, briefing, query, evidence, source, review, policy, conclusion, decision
  Dangerous: forget, erasure; also retention apply, vault encrypt|rotate-datakey, migrate governed --confirm, daemon install|uninstall|update
  Harness:   ingest, antigravity-import, agy-hook, sync, shadow, evaluate, dogfood, graph, migrate

Start here:
  ai-brains doctor
  ai-brains recall \"what did we decide\"
  ai-brains scope resolve --format json

Docs: Docs/INSTALL.md | Docs/CLI-EXIT-CODES.md | CONTRIBUTING.md
Tip: use --help on a subcommand for examples (e.g. query progressive --project-id ...).
";

/// One-line tip for short help (F5/M5). clap shows `after_help` on both `-h` and `--help`.
pub const ROOT_AFTER_HELP_TIP: &str = "Tip: run `ai-brains --help` for command groups (Daily / Operator / Governed / Dangerous / Harness).";

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn root_after_long_help__contains_required_group_labels() {
        for label in ["Daily", "Operator", "Governed", "Dangerous", "Harness"] {
            assert!(
                ROOT_AFTER_LONG_HELP.contains(label),
                "ROOT_AFTER_LONG_HELP must contain group label {label:?}"
            );
        }
    }

    #[test]
    fn root_after_long_help__contains_setup_and_stop_session() {
        assert!(
            ROOT_AFTER_LONG_HELP.contains("Setup"),
            "Setup label recommended in appendix"
        );
        assert!(
            ROOT_AFTER_LONG_HELP.contains("stop-session"),
            "AC11: stop-session under Daily inventory"
        );
        assert!(
            ROOT_AFTER_LONG_HELP.contains(
                "Daily:     recall, preflight, doctor, project, pin, context, stop-session, daemon"
            ),
            "Daily inventory must include stop-session in group text"
        );
    }

    #[test]
    fn root_after_long_help__contains_dangerous_appendix() {
        assert!(
            ROOT_AFTER_LONG_HELP.contains("Dangerous:"),
            "Dangerous appendix label"
        );
        assert!(
            ROOT_AFTER_LONG_HELP.contains("forget")
                && ROOT_AFTER_LONG_HELP.contains("erasure")
                && ROOT_AFTER_LONG_HELP.contains("retention apply"),
            "Dangerous appendix should list dual-ops class"
        );
    }

    #[test]
    fn root_after_help_tip__is_one_line_without_full_group_wall() {
        let tip = ROOT_AFTER_HELP_TIP.trim();
        assert!(
            !tip.contains('\n'),
            "after_help tip must be a single line (F5/M5)"
        );
        assert!(
            tip.contains("Daily") && tip.contains("--help"),
            "tip should point at long help groups"
        );
        // Must not embed the full multi-line inventory wall
        assert!(
            !tip.contains("stop-session") && !tip.contains("antigravity-import"),
            "short tip must not include full command inventory"
        );
    }
}
