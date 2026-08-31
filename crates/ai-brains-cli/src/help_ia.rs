//! T204 — CLI help information architecture (presentation only).
//!
//! Root long/short help appendix strings and shared labels. Command names are
//! unchanged; grouping is via `after_long_help` + `display_order` only.

/// Normative long-help appendix (F32 / §12.1). Shown only with `--help`.
pub const ROOT_AFTER_LONG_HELP: &str = "\
Command groups (presentation only — names unchanged):

  Setup:     init
  Daily:     recall, preflight, doctor, status, project, pin, memory, context, stop-session, daemon
  Operator:  backup, recovery, vault, retention, device, replicate, nightly, safety
  Governed:  scope, briefing, query, evidence, source, review, policy, conclusion, decision
  Dangerous: forget, erasure; also retention apply, vault encrypt|rotate-datakey, migrate governed --confirm, daemon install|uninstall|update
  Harness:   ingest, harness, antigravity-import, agy-hook, grok-import, grok-hook, opencode-import, opencode-hook, claude-import, claude-hook, codex-import, codex-hook, cursor-import, sync, shadow, evaluate, dogfood, graph, migrate

Start here:
  ai-brains doctor
  ai-brains doctor --summary
  ai-brains status
  ai-brains recall \"what did we decide\"
  ai-brains search \"what did we decide\"  # alias of recall
  ai-brains scope resolve
  ai-brains scope resolve --format json

Docs: Docs/INSTALL.md | Docs/CLI-EXIT-CODES.md | CONTRIBUTING.md
Tip: use --help on a subcommand for examples (e.g. recall \"what did we decide\").
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
                "Daily:     recall, preflight, doctor, status, project, pin, memory, context, stop-session, daemon"
            ),
            "Daily inventory must include status + memory + stop-session in group text"
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
    fn root_after_long_help__harness_inventory_includes_harness_cmd() {
        // T235 F18 / AC10
        assert!(
            ROOT_AFTER_LONG_HELP.contains("Harness:")
                && ROOT_AFTER_LONG_HELP.contains("harness")
                && ROOT_AFTER_LONG_HELP.contains("agy-hook")
                && ROOT_AFTER_LONG_HELP.contains("grok-hook")
                && ROOT_AFTER_LONG_HELP.contains("grok-import")
                && ROOT_AFTER_LONG_HELP.contains("opencode-hook")
                && ROOT_AFTER_LONG_HELP.contains("opencode-import")
                && ROOT_AFTER_LONG_HELP.contains("claude-hook")
                && ROOT_AFTER_LONG_HELP.contains("claude-import")
                && ROOT_AFTER_LONG_HELP.contains("codex-hook")
                && ROOT_AFTER_LONG_HELP.contains("codex-import")
                && ROOT_AFTER_LONG_HELP.contains("cursor-import")
                && !ROOT_AFTER_LONG_HELP.contains("cursor-hook"),
            "Harness inventory must include harness/grok/opencode/claude/codex/cursor-import (not cursor-hook); got:\n{ROOT_AFTER_LONG_HELP}"
        );
    }

    #[test]
    fn root_after_long_help__keeps_scope_resolve_format_json() {
        assert!(
            ROOT_AFTER_LONG_HELP.contains("ai-brains scope resolve --format json"),
            "T204/T249 Start-here json lock must remain; got:\n{ROOT_AFTER_LONG_HELP}"
        );
    }

    #[test]
    fn root_after_long_help__tip_names_recall_not_progressive() {
        // T263 AC9 / F10
        assert!(
            ROOT_AFTER_LONG_HELP.contains("recall \"what did we decide\""),
            "Start-here must keep recall what-did-we-decide; got:\n{ROOT_AFTER_LONG_HELP}"
        );
        let tip_line = ROOT_AFTER_LONG_HELP
            .lines()
            .find(|l| l.starts_with("Tip:"))
            .unwrap_or("");
        assert!(
            !tip_line.is_empty(),
            "ROOT_AFTER_LONG_HELP must have a Tip line; got:\n{ROOT_AFTER_LONG_HELP}"
        );
        assert!(
            !tip_line.contains("query progressive"),
            "Tip must not exemplify query progressive; got {tip_line}"
        );
        assert!(
            tip_line.contains("recall"),
            "Tip should exemplify recall; got {tip_line}"
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
