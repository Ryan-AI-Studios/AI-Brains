//! Shared `--format` resolver for human-vs-json TTY surfaces.
//!
//! Used by scope / retention plan / nightly `--status` (identical token maps).
//! Graph stays local: it resolves to `"pretty"`, not `"human"`.

/// pretty/human/text/markdown/md → human; json → json; auto + TTY → human;
/// auto + pipe → json; unknown → json (fail-closed).
pub(crate) fn resolve_human_json_format(explicit: &str, is_tty: bool) -> &'static str {
    match explicit {
        "pretty" | "human" | "text" | "markdown" | "md" => "human",
        "json" => "json",
        "auto" if is_tty => "human",
        "auto" => "json",
        _ => "json",
    }
}

/// T266 F27: inventory / whoami call site. Does not change resolver behavior.
pub(crate) fn is_json_output(explicit: &str, is_tty: bool) -> bool {
    resolve_human_json_format(explicit, is_tty) == "json"
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::{is_json_output, resolve_human_json_format};

    #[test]
    fn is_json_output__pretty_pipe__false() {
        assert!(!is_json_output("pretty", false));
    }

    #[test]
    fn resolve_human_json_format__auto_tty__human() {
        assert_eq!(resolve_human_json_format("auto", true), "human");
    }

    #[test]
    fn resolve_human_json_format__auto_pipe__json() {
        assert_eq!(resolve_human_json_format("auto", false), "json");
    }

    #[test]
    fn resolve_human_json_format__pretty_aliases__human_regardless_of_tty() {
        for token in ["pretty", "human", "text", "markdown", "md"] {
            assert_eq!(
                resolve_human_json_format(token, true),
                "human",
                "{token} tty"
            );
            assert_eq!(
                resolve_human_json_format(token, false),
                "human",
                "{token} pipe"
            );
        }
    }

    #[test]
    fn resolve_human_json_format__json__json_regardless_of_tty() {
        assert_eq!(resolve_human_json_format("json", true), "json");
        assert_eq!(resolve_human_json_format("json", false), "json");
    }

    #[test]
    fn resolve_human_json_format__unknown__fail_closed_json() {
        assert_eq!(resolve_human_json_format("xml", true), "json");
        assert_eq!(resolve_human_json_format("JSON", false), "json");
        assert_eq!(resolve_human_json_format("Pretty", true), "json");
    }
}
