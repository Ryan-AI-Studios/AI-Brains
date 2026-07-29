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
    /// Historical AI-Brains event-log content imported into governed ECD (T167).
    LegacyAiBrains,
    Other(String),
}
