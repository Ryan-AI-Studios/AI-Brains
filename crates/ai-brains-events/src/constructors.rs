use crate::actor::Actor;
use crate::aggregate::AggregateType;
use crate::envelope::Envelope;
use crate::event_kind::EventKind;
use crate::hash::compute_payload_hash;
use crate::payload::Payload;
use ai_brains_core::clock;
use ai_brains_core::privacy::Privacy;
use uuid::Uuid;

pub struct EventBuilder {
    aggregate_type: AggregateType,
    aggregate_id: Uuid,
    event_type: EventKind,
    actor: Actor,
    privacy: Privacy,
    causation_id: Option<Uuid>,
    correlation_id: Option<Uuid>,
}

impl EventBuilder {
    pub fn new(
        aggregate_type: AggregateType,
        aggregate_id: Uuid,
        event_type: EventKind,
        actor: Actor,
        privacy: Privacy,
    ) -> Self {
        Self {
            aggregate_type,
            aggregate_id,
            event_type,
            actor,
            privacy,
            causation_id: None,
            correlation_id: None,
        }
    }

    pub fn with_causation(mut self, id: Uuid) -> Self {
        self.causation_id = Some(id);
        self
    }

    pub fn with_correlation(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }

    pub fn build(self, payload: Payload) -> Result<Envelope, crate::errors::EventError> {
        let payload_hash = compute_payload_hash(&payload)?;

        Ok(Envelope {
            event_id: Uuid::new_v4(),
            schema_version: crate::version::CURRENT_SCHEMA_VERSION,
            aggregate_type: self.aggregate_type,
            aggregate_id: self.aggregate_id,
            event_type: self.event_type,
            occurred_at: clock::now(),
            actor: self.actor,
            causation_id: self.causation_id,
            correlation_id: self.correlation_id,
            privacy: self.privacy,
            payload,
            payload_hash,
        })
    }
}
