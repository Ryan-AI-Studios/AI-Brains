use crate::actor::Actor;
use crate::aggregate::{Aggregate, AggregateType};
use crate::event_kind::EventKind;
use crate::payload::Payload;
use ai_brains_core::privacy::Privacy;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    pub event_id: Uuid,
    pub schema_version: u32,
    pub aggregate_type: AggregateType,
    pub aggregate_id: Uuid,
    pub event_type: EventKind,
    #[serde(with = "time::serde::rfc3339")]
    pub occurred_at: OffsetDateTime,
    pub actor: Actor,
    pub causation_id: Option<Uuid>,
    pub correlation_id: Option<Uuid>,
    pub privacy: Privacy,
    pub payload: Payload,
    pub payload_hash: String,
}

impl Envelope {
    pub fn aggregate(&self) -> Aggregate {
        Aggregate {
            aggregate_type: self.aggregate_type,
            aggregate_id: self.aggregate_id,
        }
    }
}
