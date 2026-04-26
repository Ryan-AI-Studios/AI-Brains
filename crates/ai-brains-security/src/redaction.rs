use crate::finding::Finding;

pub fn redact_text(input: &str, findings: &[Finding]) -> String {
    if findings.is_empty() {
        return input.to_string();
    }

    let mut redacted = String::new();
    let mut cursor = 0usize;

    for finding in findings {
        if finding.start > cursor {
            redacted.push_str(&input[cursor..finding.start]);
        }

        let original = &input[finding.start..finding.end];
        let prefix = original.chars().take(4).collect::<String>();
        let suffix = original
            .chars()
            .rev()
            .take(4)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<String>();
        let replacement = if original.chars().count() <= 10 {
            "[REDACTED]".to_string()
        } else {
            format!("{prefix}...[REDACTED]...{suffix}")
        };
        redacted.push_str(&replacement);
        cursor = finding.end;
    }

    if cursor < input.len() {
        redacted.push_str(&input[cursor..]);
    }

    redacted
}
