pub fn normalize_role(role: &str) -> String {
    role.trim().to_ascii_lowercase()
}

/// Heuristic: token looks like an env/secret key name (or key=value head).
pub fn looks_like_secret_key(token: &str) -> bool {
    let t = token
        .trim()
        .trim_matches(|c: char| matches!(c, '"' | '\'' | '`' | '{' | '}' | '[' | ']' | '(' | ')'));
    if t.is_empty() {
        return false;
    }
    // key=value form
    let key = t.split_once('=').map(|(k, _)| k).unwrap_or(t);
    let upper = key.to_ascii_uppercase();
    const KEYWORDS: &[&str] = &[
        "API_KEY",
        "SECRET",
        "PASSWORD",
        "TOKEN",
        "ACCESS_KEY",
        "PRIVATE_KEY",
        "AUTH",
        "CREDENTIAL",
        "BEARER",
        "DATABASE_PASSWORD",
        "DB_PASSWORD",
    ];
    KEYWORDS.iter().any(|k| upper.contains(k))
        || upper.ends_with("_KEY")
        || upper.ends_with("_TOKEN")
        || upper.ends_with("_SECRET")
        || upper.ends_with("_PASSWORD")
}

/// Redact secret-looking tokens and key=value pairs from free text.
pub fn redact_sensitive_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for (i, part) in input.split_whitespace().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        if looks_like_secret_key(part) || part.contains("sk-") {
            // Replace entire token / key=value with redaction marker.
            if let Some((key, _)) = part.split_once('=')
                && (looks_like_secret_key(key) || looks_like_secret_key(part))
            {
                out.push_str(key);
                out.push_str("=[REDACTED]");
                continue;
            }
            out.push_str("[REDACTED]");
        } else {
            out.push_str(part);
        }
    }
    out
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn looks_like_secret_key__api_key__true() {
        assert!(looks_like_secret_key("API_KEY"));
        assert!(looks_like_secret_key("API_KEY=sk-abc"));
        assert!(looks_like_secret_key("TOKEN"));
    }

    #[test]
    fn redact_sensitive_text__key_value__redacted() {
        let redacted = redact_sensitive_text("down API_KEY=sk-super-secret TOKEN=abc");
        assert!(!redacted.contains("sk-super-secret"));
        assert!(redacted.contains("[REDACTED]"));
    }
}
