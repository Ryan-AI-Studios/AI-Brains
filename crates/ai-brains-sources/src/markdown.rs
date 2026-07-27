//! Hand-rolled Markdown frontmatter split and bounded preview helpers (T154).
//!
//! No YAML crate: frontmatter between `---` fences is treated as opaque text
//! for preview; full-file bytes remain the fingerprint input.

use crate::connector::Preview;

/// Split optional YAML frontmatter from a Markdown document.
///
/// Recognizes a leading fence of `---\n` or `---\r\n`. The closing fence is a
/// line that is exactly `---` (optional surrounding CR). Returns
/// `(Some(frontmatter), body)` when both fences are found; otherwise
/// `(None, full text)`.
pub fn split_frontmatter(text: &str) -> (Option<&str>, &str) {
    let rest = if let Some(r) = text.strip_prefix("---\r\n") {
        r
    } else if let Some(r) = text.strip_prefix("---\n") {
        r
    } else {
        return (None, text);
    };

    // Find closing fence as a whole line starting at a line boundary.
    let mut search_from = 0usize;
    while search_from <= rest.len() {
        let slice = &rest[search_from..];
        // Match "---" at start of remaining text or after newline.
        let at_line_start =
            search_from == 0 || rest.as_bytes().get(search_from.wrapping_sub(1)) == Some(&b'\n');
        if at_line_start && let Some(after) = slice.strip_prefix("---") {
            // Fence line ends: end of string, \n, or \r\n
            if after.is_empty() {
                return (Some(&rest[..search_from]), "");
            }
            if let Some(body) = after.strip_prefix('\n') {
                let fm_end = search_from;
                return (Some(&rest[..fm_end]), body);
            }
            if let Some(body) = after.strip_prefix("\r\n") {
                let fm_end = search_from;
                return (Some(&rest[..fm_end]), body);
            }
        }
        // Advance to next character (simple scan).
        if search_from >= rest.len() {
            break;
        }
        search_from += 1;
    }

    // Opening fence without closing → treat entire text as body.
    (None, text)
}

/// Build a bounded preview of Markdown body with 1-based line anchors.
///
/// When frontmatter is present, anchors cover the **body** region in the full
/// document (body starts after the closing fence). Preview text is taken from
/// the body only, capped to `max_chars` Unicode scalar values.
pub fn preview_from_markdown(text: &str, max_chars: usize) -> Preview {
    let (fm, body) = split_frontmatter(text);

    let body_line_start: u32 = if fm.is_some() {
        // Opening --- line (1) + frontmatter lines + closing --- line.
        // Count lines before body in the original text.
        let body_offset = text.len().saturating_sub(body.len());
        let prefix = &text[..body_offset];
        let lines_before = prefix.lines().count();
        // If prefix ends with newline, lines() already counted fence lines correctly.
        // body starts on the next line after the closing fence.
        u32::try_from(lines_before.saturating_add(1)).unwrap_or(u32::MAX)
    } else {
        1
    };

    let preview_text: String = body.chars().take(max_chars).collect();
    let preview_line_count = if preview_text.is_empty() {
        0u32
    } else {
        u32::try_from(preview_text.lines().count()).unwrap_or(u32::MAX)
    };

    let line_end = if preview_line_count == 0 {
        None
    } else {
        Some(
            body_line_start
                .saturating_add(preview_line_count)
                .saturating_sub(1),
        )
    };

    Preview {
        text: preview_text,
        line_start: Some(body_line_start),
        line_end,
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod unit_tests {
    use super::*;

    #[test]
    fn split_frontmatter__yaml_fence__separates_body() {
        let text = "---\ntitle: Alpha\n---\n# Body\nhello\n";
        let (fm, body) = split_frontmatter(text);
        assert_eq!(fm, Some("title: Alpha\n"));
        assert_eq!(body, "# Body\nhello\n");
    }

    #[test]
    fn split_frontmatter__crlf_fence__separates_body() {
        let text = "---\r\ntitle: X\r\n---\r\nbody\r\n";
        let (fm, body) = split_frontmatter(text);
        assert_eq!(fm, Some("title: X\r\n"));
        assert_eq!(body, "body\r\n");
    }

    #[test]
    fn split_frontmatter__no_fence__body_is_all() {
        let text = "# Just a note\nno frontmatter\n";
        let (fm, body) = split_frontmatter(text);
        assert_eq!(fm, None);
        assert_eq!(body, text);
    }

    #[test]
    fn preview_from_markdown__line_anchors__cover_body() {
        let text = "---\ntitle: A\n---\nline1\nline2\nline3\n";
        let p = preview_from_markdown(text, 4096);
        assert!(p.text.starts_with("line1"));
        assert_eq!(p.line_start, Some(4)); // 1:--- 2:title 3:--- 4:line1
        assert_eq!(p.line_end, Some(6));
    }

    #[test]
    fn preview_from_markdown__max_chars__bounds() {
        let text = "abcdefghijklmnopqrstuvwxyz";
        let p = preview_from_markdown(text, 5);
        assert_eq!(p.text, "abcde");
        assert_eq!(p.line_start, Some(1));
    }
}
