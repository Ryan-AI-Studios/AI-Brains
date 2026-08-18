//! T236 F17 / AC13 — sessions with new turns after summary reappear in unsummarized set.
#![allow(clippy::disallowed_methods, non_snake_case)]

use ai_brains_core::ids::{MemoryId, ProjectId, SessionId};
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::{DataKey, SqlCipherKey};
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{
    Actor, AggregateType, Payload, ProjectRegisteredPayload, SessionCompletedPayload,
    SessionStartedPayload, SessionSummaryCreatedPayload, UserPromptRecordedPayload,
};
use ai_brains_store::{EventStore, QueryStore, SqliteEventStore, VaultConnection};
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn get_unsummarized__new_turns_after_summary__relisted() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("v.db");
    let key = DataKey::generate();
    let sql_key = SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(&db, &sql_key).unwrap();
    conn.migrate().unwrap();
    let store = SqliteEventStore::new(conn.clone());

    let project_id = ProjectId::new();
    let session_id = SessionId::new();
    let actor = Actor::User(ai_brains_core::ids::UserId::new());

    let events = vec![
        EventBuilder::new(
            AggregateType::Project,
            project_id.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
            project_id,
            name: "p".into(),
            tx_id: None,
        }))
        .unwrap(),
        EventBuilder::new(
            AggregateType::Session,
            session_id.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::SessionStarted(SessionStartedPayload {
            session_id,
            project_id,
            tx_id: None,
        }))
        .unwrap(),
        EventBuilder::new(
            AggregateType::Session,
            session_id.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::UserPromptRecorded(UserPromptRecordedPayload {
            session_id,
            content: "first".into(),
            tx_id: None,
            turn_id: None,
        }))
        .unwrap(),
        EventBuilder::new(
            AggregateType::Session,
            session_id.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::SessionCompleted(SessionCompletedPayload {
            session_id,
        }))
        .unwrap(),
    ];
    for e in events {
        store.append_event(&e).unwrap();
    }

    // Before summary → listed
    let listed = conn.get_unsummarized_sessions().unwrap();
    assert!(
        listed.iter().any(|s| s == &session_id.to_string()),
        "unsummarized before summary: {listed:?}"
    );

    // Small sleep so summarized_at is strictly before later turn occurred_at
    thread::sleep(Duration::from_millis(20));

    let mem = MemoryId::new();
    store
        .append_event(
            &EventBuilder::new(
                AggregateType::Session,
                session_id.as_uuid(),
                actor.clone(),
                Privacy::LocalOnly,
            )
            .build(Payload::SessionSummaryCreated(
                SessionSummaryCreatedPayload {
                    session_id,
                    project_id: Some(project_id),
                    memory_id: mem,
                    summary: "sum".into(),
                },
            ))
            .unwrap(),
        )
        .unwrap();

    let after_sum = conn.get_unsummarized_sessions().unwrap();
    assert!(
        !after_sum.iter().any(|s| s == &session_id.to_string()),
        "should not list after summary with no new turns: {after_sum:?}"
    );

    thread::sleep(Duration::from_millis(20));

    // New turn after summary
    store
        .append_event(
            &EventBuilder::new(
                AggregateType::Session,
                session_id.as_uuid(),
                actor,
                Privacy::LocalOnly,
            )
            .build(Payload::UserPromptRecorded(UserPromptRecordedPayload {
                session_id,
                content: "second after summary".into(),
                tx_id: None,
                turn_id: None,
            }))
            .unwrap(),
        )
        .unwrap();

    let relisted = conn.get_unsummarized_sessions().unwrap();
    assert!(
        relisted.iter().any(|s| s == &session_id.to_string()),
        "session with new turns after summary must reappear: {relisted:?}"
    );
}
