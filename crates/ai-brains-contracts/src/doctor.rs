//! Doctor health report contracts (T192).
//!
//! JSON schema_version = 1. No secrets in report fields.

use serde::{Deserialize, Serialize};

/// Overall doctor roll-up status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoctorStatus {
    Ok,
    Degraded,
    Fail,
}

/// Per-check severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckSeverity {
    Ok,
    Warn,
    Fail,
    Skip,
}

impl CheckSeverity {
    /// `ok` is true iff severity is [`Ok`](Self::Ok) or [`Skip`](Self::Skip).
    pub fn is_ok_flag(self) -> bool {
        matches!(self, Self::Ok | Self::Skip)
    }
}

/// Full doctor report emitted as JSON (or summarized as human text).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoctorReport {
    pub schema_version: u32,
    pub status: DoctorStatus,
    pub checks: Vec<HealthCheck>,
    pub vault_path: String,
    /// RFC3339 timestamp when the report was generated.
    pub generated_at: String,
}

impl DoctorReport {
    pub const SCHEMA_VERSION: u32 = 1;

    /// Pure roll-up: any `fail` → Fail; else any `warn` → Degraded; else Ok.
    /// Skipped checks do not degrade.
    pub fn roll_up(checks: &[HealthCheck]) -> DoctorStatus {
        if checks.iter().any(|c| c.severity == CheckSeverity::Fail) {
            DoctorStatus::Fail
        } else if checks.iter().any(|c| c.severity == CheckSeverity::Warn) {
            DoctorStatus::Degraded
        } else {
            DoctorStatus::Ok
        }
    }
}

/// One health check result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthCheck {
    pub name: String,
    pub severity: CheckSeverity,
    /// True iff severity is Ok or Skip.
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

impl HealthCheck {
    pub fn new(
        name: impl Into<String>,
        severity: CheckSeverity,
        message: Option<String>,
        remediation: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            severity,
            ok: severity.is_ok_flag(),
            message,
            remediation,
        }
    }

    pub fn ok_msg(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(name, CheckSeverity::Ok, Some(message.into()), None)
    }

    pub fn warn(
        name: impl Into<String>,
        message: impl Into<String>,
        remediation: Option<String>,
    ) -> Self {
        Self::new(name, CheckSeverity::Warn, Some(message.into()), remediation)
    }

    pub fn fail(
        name: impl Into<String>,
        message: impl Into<String>,
        remediation: Option<String>,
    ) -> Self {
        Self::new(name, CheckSeverity::Fail, Some(message.into()), remediation)
    }

    pub fn skip(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(name, CheckSeverity::Skip, Some(message.into()), None)
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn doctor_report__serde__roundtrip_schema_v1() {
        let report = DoctorReport {
            schema_version: DoctorReport::SCHEMA_VERSION,
            status: DoctorStatus::Degraded,
            checks: vec![
                HealthCheck::ok_msg("vault_exists", "present"),
                HealthCheck::warn(
                    "backup_recent",
                    "no backups",
                    Some("ai-brains backup create".into()),
                ),
                HealthCheck::skip("integrity", "pass --full"),
            ],
            vault_path: "/tmp/vault.db".into(),
            generated_at: "2026-08-02T00:00:00Z".into(),
        };

        let json = serde_json::to_string(&report).expect("serialize");
        assert!(json.contains("\"schema_version\":1"));
        assert!(json.contains("\"status\":\"degraded\""));
        assert!(json.contains("\"severity\":\"ok\""));
        assert!(json.contains("\"severity\":\"warn\""));
        assert!(json.contains("\"severity\":\"skip\""));
        // skip_serializing_if: ok_msg has no remediation key
        assert!(
            !json.contains("\"name\":\"vault_exists\"") || !json.contains("\"remediation\":null")
        );
        let parsed: DoctorReport = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, report);
        assert_eq!(parsed.schema_version, 1);
    }

    #[test]
    fn doctor_status__roll_up__fail_beats_degraded() {
        let checks = vec![
            HealthCheck::warn("backup_recent", "stale", None),
            HealthCheck::fail("vault_open", "wrong key", None),
            HealthCheck::ok_msg("daemon_reachable", "down"),
        ];
        assert_eq!(DoctorReport::roll_up(&checks), DoctorStatus::Fail);
    }

    #[test]
    fn doctor_status__roll_up__warn_is_degraded() {
        let checks = vec![
            HealthCheck::ok_msg("vault_exists", "present"),
            HealthCheck::warn("zero_key_escape", "zero key", None),
            HealthCheck::skip("recovery_kit_file", "pass --kit-path"),
        ];
        assert_eq!(DoctorReport::roll_up(&checks), DoctorStatus::Degraded);
    }

    #[test]
    fn doctor_status__roll_up__skip_does_not_degrade() {
        let checks = vec![
            HealthCheck::ok_msg("vault_exists", "present"),
            HealthCheck::skip("integrity", "not requested"),
            HealthCheck::skip("recovery_kit_file", "pass --kit-path"),
        ];
        assert_eq!(DoctorReport::roll_up(&checks), DoctorStatus::Ok);
    }

    #[test]
    fn health_check__ok_flag__matches_severity() {
        assert!(CheckSeverity::Ok.is_ok_flag());
        assert!(CheckSeverity::Skip.is_ok_flag());
        assert!(!CheckSeverity::Warn.is_ok_flag());
        assert!(!CheckSeverity::Fail.is_ok_flag());
        assert!(HealthCheck::ok_msg("a", "m").ok);
        assert!(HealthCheck::skip("a", "m").ok);
        assert!(!HealthCheck::warn("a", "m", None).ok);
        assert!(!HealthCheck::fail("a", "m", None).ok);
    }

    #[test]
    fn doctor_report__serde__omits_none_optionals() {
        let check = HealthCheck::ok_msg("cipher_page", "4.5.0");
        let json = serde_json::to_string(&check).expect("serialize");
        assert!(!json.contains("remediation"));
        // message is Some — present
        assert!(json.contains("\"message\""));
    }
}
