//! Endpoint classification for model provider locality (T157).
//!
//! Pure, dependency-light host parsing — no `url` crate required.
//! **Local for privacy** = loopback only (`localhost`, `127.0.0.1`, `::1`).
//! Private LAN and remote hostnames are **CloudApi** (network egress).

use ai_brains_core::model_provenance::{EndpointClass, endpoint_class_is_local};

/// Classify a configured endpoint URL or bare host for privacy routing.
///
/// Accepts forms such as:
/// - `http://127.0.0.1:11434`
/// - `https://gpu.example.com:11434/path`
/// - `localhost:11434`
/// - `127.0.0.1`
/// - `[::1]:11434`
///
/// Empty / whitespace-only → [`EndpointClass::Unknown`].
/// Unparseable host → [`EndpointClass::Unknown`].
/// Loopback hosts → [`EndpointClass::LocalLoopback`].
/// All other parseable hosts (incl. private LAN, `0.0.0.0`) → [`EndpointClass::CloudApi`].
pub fn classify_endpoint(endpoint: &str) -> EndpointClass {
    let trimmed = endpoint.trim();
    if trimmed.is_empty() {
        return EndpointClass::Unknown;
    }

    let Some(host) = extract_host(trimmed) else {
        return EndpointClass::Unknown;
    };

    if is_loopback_host(&host) {
        EndpointClass::LocalLoopback
    } else {
        // Private LAN, 0.0.0.0, remote hostnames — not local for privacy.
        EndpointClass::CloudApi
    }
}

/// True when the classified endpoint is local for privacy (`LocalLoopback` | `LocalProcess`).
pub fn endpoint_is_local(endpoint: &str) -> bool {
    endpoint_class_is_local(classify_endpoint(endpoint))
}

/// Extract host from `scheme://host:port/path`, `host:port`, `[ipv6]:port`, or bare host.
fn extract_host(input: &str) -> Option<String> {
    let without_scheme = strip_scheme(input);
    // Drop path/query after host[:port]
    let authority = without_scheme.split(['/', '?', '#']).next()?.trim();
    if authority.is_empty() {
        return None;
    }

    // Bracketed IPv6: [::1] or [::1]:11434
    if let Some(rest) = authority.strip_prefix('[') {
        let end = rest.find(']')?;
        let host = &rest[..end];
        if host.is_empty() {
            return None;
        }
        return Some(host.to_string());
    }

    // IPv6 without brackets is uncommon in URLs; if multiple colons, treat whole
    // authority as host when no scheme was present and no path — else try last-colon port split
    // only for IPv4 / hostname forms (single trailing :port).
    if authority.matches(':').count() > 1 {
        // Likely bare IPv6 (::1, 2001:db8::1) without brackets.
        return Some(authority.to_string());
    }

    // host or host:port
    if let Some((host, port)) = authority.rsplit_once(':') {
        // Ensure port looks numeric; otherwise treat full string as host.
        if !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) {
            if host.is_empty() {
                return None;
            }
            return Some(host.to_string());
        }
    }

    Some(authority.to_string())
}

fn strip_scheme(input: &str) -> &str {
    // Hand-parse common schemes only; avoid pulling url crate.
    for prefix in ["https://", "http://", "HTTPS://", "HTTP://"] {
        if let Some(rest) = input.strip_prefix(prefix) {
            return rest;
        }
        // Case-insensitive fallback for mixed case scheme
    }
    // Generic scheme:// (case-insensitive scheme name)
    if let Some(idx) = input.find("://") {
        let scheme = &input[..idx];
        if !scheme.is_empty()
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '.' || c == '-')
        {
            return &input[idx + 3..];
        }
    }
    input
}

fn is_loopback_host(host: &str) -> bool {
    let h = host.trim();
    if h.eq_ignore_ascii_case("localhost") {
        return true;
    }
    if h == "127.0.0.1" {
        return true;
    }
    // IPv6 loopback forms
    if h == "::1" || h.eq_ignore_ascii_case("[::1]") {
        return true;
    }
    // Normalized expanded form sometimes seen
    if h == "0:0:0:0:0:0:0:1" {
        return true;
    }
    false
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn classify_endpoint__loopback_127__local_loopback() {
        assert_eq!(
            classify_endpoint("http://127.0.0.1:11434"),
            EndpointClass::LocalLoopback
        );
        assert_eq!(
            classify_endpoint("127.0.0.1:11434"),
            EndpointClass::LocalLoopback
        );
        assert_eq!(
            classify_endpoint("http://127.0.0.1"),
            EndpointClass::LocalLoopback
        );
    }

    #[test]
    fn classify_endpoint__localhost__local_loopback() {
        assert_eq!(
            classify_endpoint("http://localhost:11434"),
            EndpointClass::LocalLoopback
        );
        assert_eq!(
            classify_endpoint("LOCALHOST:8080"),
            EndpointClass::LocalLoopback
        );
        assert_eq!(
            classify_endpoint("https://LocalHost/v1"),
            EndpointClass::LocalLoopback
        );
    }

    #[test]
    fn classify_endpoint__ipv6_loopback__local_loopback() {
        assert_eq!(
            classify_endpoint("http://[::1]:11434"),
            EndpointClass::LocalLoopback
        );
        assert_eq!(classify_endpoint("[::1]"), EndpointClass::LocalLoopback);
        assert_eq!(classify_endpoint("::1"), EndpointClass::LocalLoopback);
    }

    #[test]
    fn classify_endpoint__remote_https_host__cloud_api() {
        assert_eq!(
            classify_endpoint("https://gpu-box.example.com:11434"),
            EndpointClass::CloudApi
        );
        assert_eq!(
            classify_endpoint("https://api.openai.com/v1"),
            EndpointClass::CloudApi
        );
    }

    #[test]
    fn classify_endpoint__private_lan_ip__not_local() {
        assert_eq!(
            classify_endpoint("http://192.168.1.10:11434"),
            EndpointClass::CloudApi
        );
        assert_eq!(
            classify_endpoint("http://10.0.0.5:8080"),
            EndpointClass::CloudApi
        );
        assert_eq!(
            classify_endpoint("http://172.16.0.1:11434"),
            EndpointClass::CloudApi
        );
        assert!(!endpoint_is_local("http://192.168.1.10:11434"));
    }

    #[test]
    fn classify_endpoint__zero_bind__not_local() {
        assert_eq!(
            classify_endpoint("http://0.0.0.0:11434"),
            EndpointClass::CloudApi
        );
        assert!(!endpoint_is_local("http://0.0.0.0:11434"));
    }

    #[test]
    fn classify_endpoint__empty__unknown() {
        assert_eq!(classify_endpoint(""), EndpointClass::Unknown);
        assert_eq!(classify_endpoint("   "), EndpointClass::Unknown);
    }
}
