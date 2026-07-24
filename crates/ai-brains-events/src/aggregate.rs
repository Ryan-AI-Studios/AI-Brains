use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregateType {
    System,
    Project,
    Session,
    Memory,
    Job,
    Conflict,
    Recipe,
    /// Legacy decision facts (`DecisionRecorded`) and new governed decision events
    /// (`DecisionProposed` / `DecisionApproved` / …) share this aggregate type.
    Decision,
    // Governed memory aggregates (T148)
    Source,
    Evidence,
    Conclusion,
    Workspace,
    Principal,
    Grant,
    ReviewItem,
    Briefing,
    QueryTrace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aggregate {
    pub aggregate_type: AggregateType,
    pub aggregate_id: Uuid,
}

impl Aggregate {
    pub fn new(aggregate_type: AggregateType, aggregate_id: Uuid) -> Self {
        Self {
            aggregate_type,
            aggregate_id,
        }
    }

    pub fn system() -> Self {
        Self::new(AggregateType::System, Uuid::nil())
    }
}
