use crate::command_handler::{SessionStopCommand, SessionStopStatus};
use crate::errors::CaptureError;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::{Payload, SessionCompletedPayload, SessionFailedPayload};
use ai_brains_events::{Actor, AggregateType, Envelope, EventKind};

pub fn build_session_stop(command: &SessionStopCommand) -> crate::Result<Envelope> {
    match command.status {
        SessionStopStatus::Completed => EventBuilder::new(
            AggregateType::Session,
            command.session_id.as_uuid(),
            EventKind::SessionCompleted,
            Actor::Harness(command.harness_id),
            command.privacy,
        )
        .build(Payload::SessionCompleted(SessionCompletedPayload {
            session_id: command.session_id,
        }))
        .map_err(Into::into),
        SessionStopStatus::Failed => {
            let reason = command
                .reason
                .clone()
                .ok_or(CaptureError::MissingFailureReason)?;
            EventBuilder::new(
                AggregateType::Session,
                command.session_id.as_uuid(),
                EventKind::SessionFailed,
                Actor::Harness(command.harness_id),
                command.privacy,
            )
            .build(Payload::SessionFailed(SessionFailedPayload {
                session_id: command.session_id,
                reason,
            }))
            .map_err(Into::into)
        }
        SessionStopStatus::Aborted => EventBuilder::new(
            AggregateType::Session,
            command.session_id.as_uuid(),
            EventKind::SessionFailed,
            Actor::Harness(command.harness_id),
            command.privacy,
        )
        .build(Payload::SessionFailed(SessionFailedPayload {
            session_id: command.session_id,
            reason: "aborted".to_string(),
        }))
        .map_err(Into::into),
    }
}
