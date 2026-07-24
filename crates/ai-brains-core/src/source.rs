use serde::{Deserialize, Serialize};

/// Kind of external or local source feeding governed evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SourceKind {
    GitRepository,
    File,
    ObsidianVault,
    Ledgerful,
    HermesSession,
    Honcho,
    Manual,
    Other(String),
}
