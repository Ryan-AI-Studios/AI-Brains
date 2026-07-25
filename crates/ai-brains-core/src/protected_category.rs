use serde::{Deserialize, Serialize};

/// Categories of conclusions that require human approval for Confirmed/Approved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ProtectedCategory {
    SecurityPolicy,
    PrivacyScope,
    Deletion,
    Spending,
    LegalCompliance,
    DeploymentAuthorization,
    IrreversibleArchitecture,
}

impl ProtectedCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            ProtectedCategory::SecurityPolicy => "SecurityPolicy",
            ProtectedCategory::PrivacyScope => "PrivacyScope",
            ProtectedCategory::Deletion => "Deletion",
            ProtectedCategory::Spending => "Spending",
            ProtectedCategory::LegalCompliance => "LegalCompliance",
            ProtectedCategory::DeploymentAuthorization => "DeploymentAuthorization",
            ProtectedCategory::IrreversibleArchitecture => "IrreversibleArchitecture",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "SecurityPolicy" => Some(ProtectedCategory::SecurityPolicy),
            "PrivacyScope" => Some(ProtectedCategory::PrivacyScope),
            "Deletion" => Some(ProtectedCategory::Deletion),
            "Spending" => Some(ProtectedCategory::Spending),
            "LegalCompliance" => Some(ProtectedCategory::LegalCompliance),
            "DeploymentAuthorization" => Some(ProtectedCategory::DeploymentAuthorization),
            "IrreversibleArchitecture" => Some(ProtectedCategory::IrreversibleArchitecture),
            _ => None,
        }
    }
}

impl std::fmt::Display for ProtectedCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
