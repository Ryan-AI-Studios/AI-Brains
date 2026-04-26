use crate::finding::{Confidence, Finding, SecretKind};
use once_cell::sync::Lazy;
use regex::Regex;

static BEARER_TOKEN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\bbearer\s+[A-Za-z0-9._\-]{12,}\b").unwrap_or_else(|_| unreachable!())
});
static PRIVATE_KEY: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----")
        .unwrap_or_else(|_| unreachable!())
});
static CONNECTION_STRING: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(?:server|host|data source)=.+?(?:;|$).+?\b(?:password|pwd)=.+?(?:;|$)")
        .unwrap_or_else(|_| unreachable!())
});

pub fn detect_patterns(input: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    findings.extend(BEARER_TOKEN.find_iter(input).map(|m| {
        Finding::new(
            SecretKind::BearerToken,
            Confidence::Likely,
            m.start(),
            m.end(),
        )
    }));
    findings.extend(
        PRIVATE_KEY
            .find_iter(input)
            .map(|m| Finding::new(SecretKind::PrivateKey, Confidence::High, m.start(), m.end())),
    );
    findings.extend(CONNECTION_STRING.find_iter(input).map(|m| {
        Finding::new(
            SecretKind::ConnectionString,
            Confidence::Likely,
            m.start(),
            m.end(),
        )
    }));

    findings.sort_by_key(|finding| finding.start);
    findings
}
