use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Granular endpoint class for model providers (authoritative for deployment derivation).
///
/// PascalCase serde names. Used to derive [`ModelProvenance::deployment`] consistently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EndpointClass {
    /// Loopback host only: `localhost`, `127.0.0.1`, `::1`.
    LocalLoopback,
    /// In-process or Unix-socket style local process (no network egress).
    LocalProcess,
    /// Any networked non-loopback endpoint (including private LAN).
    CloudApi,
    /// Unparseable or unclassified endpoint.
    Unknown,
}

/// Optional token usage metrics when a provider exposes them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ModelUsage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u64>,
}

/// Model/provider/workflow version fields for model-derived evidence and conclusions.
///
/// Additive optional fields (schema v1): never store chain-of-thought, prompts, or API keys.
///
/// **`deployment` is derived from `endpoint_class`** via [`ModelProvenance::with_endpoint_class`]
/// / [`deployment_from_endpoint_class`]. Do not set them independently to disagreeing values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelProvenance {
    pub provider: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_version: Option<String>,
    /// Deployment class when known: `"local"` or `"cloud"`.
    /// Derived from [`Self::endpoint_class`]; omit when class is Unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployment: Option<String>,
    /// Authoritative endpoint class; drives [`Self::deployment`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint_class: Option<EndpointClass>,
    /// Token usage when available from the provider response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<ModelUsage>,
    /// Optional template / prompt-template identifier (never the prompt body).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    /// Input evidence/memory ids as strings (no CoT or raw tool logs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_ids: Option<Vec<String>>,
    /// SHA-256 hex of model output text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(with = "time::serde::rfc3339::option")]
    pub started_at: Option<OffsetDateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(with = "time::serde::rfc3339::option")]
    pub completed_at: Option<OffsetDateTime>,
}

impl ModelProvenance {
    /// Build provenance from provider identity and endpoint class.
    ///
    /// Sets `endpoint_class` and derives `deployment` via [`Self::with_endpoint_class`].
    /// Optional fields (`workflow_version`, `input_ids`, `output_hash`, timestamps, `usage`)
    /// start as `None` and may be filled by the caller.
    pub fn from_provider(
        provider_name: impl Into<String>,
        model: impl Into<String>,
        endpoint_class: EndpointClass,
    ) -> Self {
        Self {
            provider: provider_name.into(),
            model: model.into(),
            model_version: None,
            workflow_version: None,
            deployment: None,
            endpoint_class: None,
            usage: None,
            template_id: None,
            input_ids: None,
            output_hash: None,
            started_at: None,
            completed_at: None,
        }
        .with_endpoint_class(endpoint_class)
    }

    /// Set `endpoint_class` and derive `deployment` consistently.
    ///
    /// LocalLoopback | LocalProcess → `"local"`; CloudApi → `"cloud"`; Unknown → `None`.
    pub fn with_endpoint_class(mut self, class: EndpointClass) -> Self {
        self.endpoint_class = Some(class);
        self.deployment = deployment_from_endpoint_class(class).map(str::to_string);
        self
    }
}

/// Derive deployment string from endpoint class.
///
/// Unknown yields `None` (omit / freeze: not `"unknown"`).
pub fn deployment_from_endpoint_class(class: EndpointClass) -> Option<&'static str> {
    match class {
        EndpointClass::LocalLoopback | EndpointClass::LocalProcess => Some("local"),
        EndpointClass::CloudApi => Some("cloud"),
        EndpointClass::Unknown => None,
    }
}

/// True when the endpoint class counts as local for privacy routing.
pub fn endpoint_class_is_local(class: EndpointClass) -> bool {
    matches!(
        class,
        EndpointClass::LocalLoopback | EndpointClass::LocalProcess
    )
}

// ── Cloud route gate (shared by registry + synthesis) ────────────────────────

/// Stable reason codes for model cloud-routing denials.
///
/// Align with control-plane `policy::reason` where names overlap.
/// Messages MUST contain only these codes — never prompts, API keys, or claim bodies.
pub mod reason {
    /// Sealed / LocalOnly / NeverInject routed to a non-local provider.
    pub const PRIVACY_ROUTE_MISMATCH: &str = "privacy_route_mismatch";
    /// Global/config `allow_cloud_extraction` is off and provider is non-local.
    pub const CLOUD_EXTRACTION_DISABLED: &str = "cloud_extraction_disabled";
    /// Local provider required but none registered / none viable.
    pub const NO_LOCAL_PROVIDER: &str = "no_local_provider";
}

/// Denial from [`cloud_route_allowed`]. Display is reason_code only (no secrets).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudRouteDenial {
    pub reason_code: &'static str,
}

impl std::fmt::Display for CloudRouteDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.reason_code)
    }
}

impl std::error::Error for CloudRouteDenial {}

/// Pure cloud-route gate used by provider registry and synthesis.
///
/// Rules:
/// 1. Local-strict privacy (`LocalOnly` | `NeverInject` | `Sealed`) + non-local provider
///    → `privacy_route_mismatch`
/// 2. Non-local provider + `allow_cloud_extraction == false` → `cloud_extraction_disabled`
/// 3. Local provider always allowed regardless of flag
/// 4. Otherwise `Ok`
pub fn cloud_route_allowed(
    privacy: crate::privacy::Privacy,
    provider_is_local: bool,
    allow_cloud_extraction: bool,
) -> Result<(), CloudRouteDenial> {
    use crate::privacy::privacy_is_local_strict;

    if privacy_is_local_strict(privacy) && !provider_is_local {
        return Err(CloudRouteDenial {
            reason_code: reason::PRIVACY_ROUTE_MISMATCH,
        });
    }
    if !provider_is_local && !allow_cloud_extraction {
        return Err(CloudRouteDenial {
            reason_code: reason::CLOUD_EXTRACTION_DISABLED,
        });
    }
    Ok(())
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use crate::privacy::Privacy;

    #[test]
    fn model_provenance__legacy_json_without_new_fields__deserializes() {
        let json = r#"{
            "provider": "ollama",
            "model": "qwen",
            "model_version": "3.5",
            "workflow_version": null,
            "deployment": "local",
            "input_ids": ["id-1"],
            "output_hash": "abc",
            "started_at": null,
            "completed_at": null
        }"#;
        let p: ModelProvenance = serde_json::from_str(json).expect("legacy JSON must deserialize");
        assert_eq!(p.provider, "ollama");
        assert_eq!(p.model, "qwen");
        assert_eq!(p.deployment.as_deref(), Some("local"));
        assert!(p.endpoint_class.is_none());
        assert!(p.usage.is_none());
        assert!(p.template_id.is_none());
    }

    #[test]
    fn model_provenance__endpoint_class_sets_deployment_consistently() {
        let base = ModelProvenance {
            provider: "mock".into(),
            model: "m".into(),
            model_version: None,
            workflow_version: None,
            deployment: Some("cloud".into()), // stale; helper must overwrite
            endpoint_class: None,
            usage: None,
            template_id: None,
            input_ids: None,
            output_hash: None,
            started_at: None,
            completed_at: None,
        };

        let local = base
            .clone()
            .with_endpoint_class(EndpointClass::LocalLoopback);
        assert_eq!(local.endpoint_class, Some(EndpointClass::LocalLoopback));
        assert_eq!(local.deployment.as_deref(), Some("local"));

        let process = base
            .clone()
            .with_endpoint_class(EndpointClass::LocalProcess);
        assert_eq!(process.deployment.as_deref(), Some("local"));

        let cloud = base.clone().with_endpoint_class(EndpointClass::CloudApi);
        assert_eq!(cloud.deployment.as_deref(), Some("cloud"));

        let unknown = base.with_endpoint_class(EndpointClass::Unknown);
        assert_eq!(unknown.endpoint_class, Some(EndpointClass::Unknown));
        assert!(unknown.deployment.is_none());
    }

    #[test]
    fn model_provenance__from_provider__sets_endpoint_class_and_deployment() {
        let local = ModelProvenance::from_provider("ollama", "qwen", EndpointClass::LocalLoopback);
        assert_eq!(local.provider, "ollama");
        assert_eq!(local.model, "qwen");
        assert_eq!(local.endpoint_class, Some(EndpointClass::LocalLoopback));
        assert_eq!(local.deployment.as_deref(), Some("local"));
        assert!(local.workflow_version.is_none());
        assert!(local.input_ids.is_none());

        let cloud = ModelProvenance::from_provider("openai", "gpt", EndpointClass::CloudApi);
        assert_eq!(cloud.endpoint_class, Some(EndpointClass::CloudApi));
        assert_eq!(cloud.deployment.as_deref(), Some("cloud"));

        let unknown = ModelProvenance::from_provider("unknown-p", "m", EndpointClass::Unknown);
        assert_eq!(unknown.endpoint_class, Some(EndpointClass::Unknown));
        assert!(unknown.deployment.is_none());
    }

    #[test]
    fn model_provenance__endpoint_class_and_usage__roundtrip() {
        let p = ModelProvenance {
            provider: "ollama".into(),
            model: "qwen".into(),
            model_version: None,
            workflow_version: Some("hierarchical-synthesis/v1".into()),
            deployment: None,
            endpoint_class: None,
            usage: Some(ModelUsage {
                prompt_tokens: Some(10),
                completion_tokens: Some(20),
                total_tokens: Some(30),
            }),
            template_id: Some("tpl-a".into()),
            input_ids: Some(vec!["m1".into()]),
            output_hash: Some("deadbeef".into()),
            started_at: None,
            completed_at: None,
        }
        .with_endpoint_class(EndpointClass::LocalLoopback);

        let v = serde_json::to_value(&p).expect("serialize");
        let back: ModelProvenance = serde_json::from_value(v).expect("deserialize");
        assert_eq!(back, p);
        assert_eq!(back.endpoint_class, Some(EndpointClass::LocalLoopback));
        assert_eq!(back.deployment.as_deref(), Some("local"));
        assert_eq!(back.usage.as_ref().and_then(|u| u.total_tokens), Some(30));
        assert_eq!(back.template_id.as_deref(), Some("tpl-a"));
    }

    #[test]
    fn cloud_route__sealed_plus_cloud_provider__denied() {
        let err = cloud_route_allowed(Privacy::Sealed, false, true).unwrap_err();
        assert_eq!(err.reason_code, reason::PRIVACY_ROUTE_MISMATCH);
    }

    #[test]
    fn cloud_route__never_inject_plus_cloud__denied() {
        let err = cloud_route_allowed(Privacy::NeverInject, false, true).unwrap_err();
        assert_eq!(err.reason_code, reason::PRIVACY_ROUTE_MISMATCH);
    }

    #[test]
    fn cloud_route__local_only_plus_cloud__denied() {
        let err = cloud_route_allowed(Privacy::LocalOnly, false, true).unwrap_err();
        assert_eq!(err.reason_code, reason::PRIVACY_ROUTE_MISMATCH);
    }

    #[test]
    fn cloud_route__cloud_ok_plus_cloud__allowed_when_flag_on() {
        assert!(cloud_route_allowed(Privacy::CloudOk, false, true).is_ok());
    }

    #[test]
    fn cloud_route__cloud_ok_plus_cloud__denied_when_flag_off() {
        let err = cloud_route_allowed(Privacy::CloudOk, false, false).unwrap_err();
        assert_eq!(err.reason_code, reason::CLOUD_EXTRACTION_DISABLED);
    }

    #[test]
    fn cloud_route__sealed_plus_local__allowed() {
        assert!(cloud_route_allowed(Privacy::Sealed, true, false).is_ok());
        assert!(cloud_route_allowed(Privacy::LocalOnly, true, false).is_ok());
        assert!(cloud_route_allowed(Privacy::NeverInject, true, false).is_ok());
    }

    #[test]
    fn cloud_route_denial__reason_code_no_prompt_content() {
        let denial = CloudRouteDenial {
            reason_code: reason::PRIVACY_ROUTE_MISMATCH,
        };
        let s = denial.to_string();
        assert_eq!(s, reason::PRIVACY_ROUTE_MISMATCH);
        assert!(!s.contains("prompt"));
        assert!(!s.contains("api_key"));
        assert!(!s.contains("sk-"));
        // Ensure Display never embeds Privacy Debug dumps
        assert!(!s.contains("Sealed"));
        assert!(!s.contains("LocalOnly"));
    }

    #[test]
    fn endpoint_class_is_local__loopback_and_process__true() {
        assert!(endpoint_class_is_local(EndpointClass::LocalLoopback));
        assert!(endpoint_class_is_local(EndpointClass::LocalProcess));
        assert!(!endpoint_class_is_local(EndpointClass::CloudApi));
        assert!(!endpoint_class_is_local(EndpointClass::Unknown));
    }
}
