use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::str::FromStr for $name {
            type Err = uuid::Error;
            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                Ok(Self(Uuid::parse_str(s)?))
            }
        }
    };
}

define_id!(ProjectId);
define_id!(UserId);
define_id!(DeviceId);
define_id!(HarnessId);
define_id!(SessionId);
define_id!(TurnId);
define_id!(MemoryId);
define_id!(ConflictId);
define_id!(RecipeId);
define_id!(KnowledgeId);
// Governed memory IDs (T148) — DecisionId is distinct from MemoryId / KnowledgeId
define_id!(SourceId);
define_id!(SourceVersionId);
define_id!(EvidenceId);
define_id!(ConclusionId);
define_id!(DecisionId);
define_id!(WorkspaceId);
define_id!(PrincipalId);
define_id!(GrantId);
define_id!(ReviewItemId);
define_id!(BriefingId);
define_id!(QueryTraceId);
define_id!(ContentKeyId);
define_id!(TombstoneId);
define_id!(ReplicationEventId);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionId(String);

impl TransactionId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TransactionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for TransactionId {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}
