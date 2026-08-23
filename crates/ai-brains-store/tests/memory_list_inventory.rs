//! T216 — QueryStore memory inventory list/count (AC16, limit+1, by_project, count_forgotten).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::DataKey;
use ai_brains_events::{
    Actor, AggregateType, Payload,
    constructors::EventBuilder,
    payload::{MemoryForgottenPayload, MemoryPinnedPayload, ProjectRegisteredPayload},
};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use ai_brains_store::{MemoryListFilter, MemoryListStatus, QueryStore};
use tempfile::NamedTempFile;

fn open_store() -> SqliteEventStore {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap().to_string();
    std::mem::forget(temp_file);

    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(&db_path, &sql_key).unwrap();
    conn.migrate().unwrap();
    SqliteEventStore::new(conn)
}

fn register_project(store: &SqliteEventStore, project_id: ProjectId, name: &str) {
    let reg = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
        project_id,
        name: name.to_string(),
        tx_id: None,
    }))
    .unwrap();
    store.append_event(&reg).expect("register project");
}

fn pin_memory(store: &SqliteEventStore, project_id: ProjectId, content: &str) -> MemoryId {
    let memory_id = MemoryId::new();
    let envelope = EventBuilder::new(
        AggregateType::Memory,
        memory_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::MemoryPinned(MemoryPinnedPayload {
        memory_id,
        content: content.to_string(),
        session_id: None,
        project_id: Some(project_id),
        tx_id: None,
        rank: None,
        source_tag: None,
        query_text: None,
    }))
    .unwrap();
    store.append_event(&envelope).expect("pin memory");
    memory_id
}

fn forget_memory(store: &SqliteEventStore, memory_id: MemoryId) {
    let envelope = EventBuilder::new(
        AggregateType::Memory,
        memory_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::MemoryForgotten(MemoryForgottenPayload {
        memory_id,
    }))
    .unwrap();
    store.append_event(&envelope).expect("forget memory");
}

#[test]
fn list_memories__limit_plus_one__returns_extra_row_for_more_available() {
    let store = open_store();
    let a = ProjectId::new();
    register_project(&store, a, "A");
    for i in 0..5 {
        pin_memory(&store, a, &format!("DECISION: pin {i}"));
    }
    let conn = store.connection();
    // Request limit+1 = 4 when page size is 3.
    let page = MemoryListFilter {
        status: MemoryListStatus::Pinned,
        project_id: Some(a),
        tag: None,
        limit: 4,
    };
    let rows = conn.list_memories(&page).unwrap();
    assert_eq!(
        rows.len(),
        4,
        "limit+1 must surface more_available probe row"
    );
    let total = conn
        .count_memories(&MemoryListFilter {
            status: MemoryListStatus::Pinned,
            project_id: Some(a),
            tag: None,
            limit: 3,
        })
        .unwrap();
    assert_eq!(total, 5);
    // Deterministic ORDER BY updated_at DESC, memory_id ASC
    for w in rows.windows(2) {
        let ok = w[0].updated_at > w[1].updated_at
            || (w[0].updated_at == w[1].updated_at && w[0].memory_id <= w[1].memory_id);
        assert!(
            ok,
            "must be updated_at DESC, memory_id ASC: {} ({}) vs {} ({})",
            w[0].memory_id, w[0].updated_at, w[1].memory_id, w[1].updated_at
        );
    }
}

#[test]
fn count_forgotten_memories__mirrors_pinned_scope() {
    let store = open_store();
    let a = ProjectId::new();
    let b = ProjectId::new();
    register_project(&store, a, "A");
    register_project(&store, b, "B");
    let m1 = pin_memory(&store, a, "pin A1");
    let m2 = pin_memory(&store, a, "pin A2");
    let m3 = pin_memory(&store, b, "pin B1");
    forget_memory(&store, m1);
    forget_memory(&store, m3);

    let conn = store.connection();
    assert_eq!(conn.count_forgotten_memories(None).unwrap(), 2);
    assert_eq!(conn.count_forgotten_memories(Some(&a)).unwrap(), 1);
    assert_eq!(conn.count_forgotten_memories(Some(&b)).unwrap(), 1);
    assert_eq!(conn.count_pinned_memories(Some(&a)).unwrap(), 1); // m2 still pinned
    let _ = m2;
}

#[test]
fn count_memories_by_project__orders_and_omits_zeros() {
    let store = open_store();
    let a = ProjectId::new();
    let b = ProjectId::new();
    let c = ProjectId::new(); // registered only — turn-only style, no memories
    register_project(&store, a, "A");
    register_project(&store, b, "B");
    register_project(&store, c, "C");
    let m = pin_memory(&store, a, "only A forgotten later");
    pin_memory(&store, a, "A pin stays");
    pin_memory(&store, b, "B pin");
    pin_memory(&store, b, "B pin two");
    pin_memory(&store, b, "B pin three");
    forget_memory(&store, m);

    let conn = store.connection();
    let rows = conn.count_memories_by_project().unwrap();
    // C has no memories → excluded
    assert!(
        !rows.iter().any(|(pid, _, _)| pid == &c.to_string()),
        "turn-only / zero-memory projects excluded; got {rows:?}"
    );
    // B has 3 pinned; A has 1 pinned + 1 forgotten = 2 total activity
    assert!(rows.len() >= 2);
    // Ordered by (pinned+forgotten) DESC, project_id ASC
    let totals: Vec<(String, u64)> = rows
        .iter()
        .map(|(pid, p, f)| (pid.clone(), p + f))
        .collect();
    for w in totals.windows(2) {
        assert!(
            w[0].1 >= w[1].1,
            "must be total DESC: {}={} vs {}={}",
            w[0].0,
            w[0].1,
            w[1].0,
            w[1].1
        );
    }
    let a_row = rows
        .iter()
        .find(|(pid, _, _)| pid == &a.to_string())
        .expect("A present");
    assert_eq!(a_row.1, 1, "A pinned");
    assert_eq!(a_row.2, 1, "A forgotten");
    let b_row = rows
        .iter()
        .find(|(pid, _, _)| pid == &b.to_string())
        .expect("B present");
    assert_eq!(b_row.1, 3, "B pinned");
    assert_eq!(b_row.2, 0, "B forgotten");
}

/// T230 AC8 / F29: store-level orphan inject — pin without register_project.
/// `count_memories_by_project` groups memory_projection only (no project JOIN),
/// so orphan project_ids surface for CLI display_label empty-name fill.
#[test]
fn count_memories_by_project__orphan_pin_without_register__includes_project_id() {
    let store = open_store();
    let orphan = ProjectId::new();
    // Deliberately do NOT call register_project — orphan project_id.
    pin_memory(
        &store,
        orphan,
        "DECISION: orphan pin without project_projection",
    );

    let conn = store.connection();
    let rows = conn.count_memories_by_project().unwrap();
    let orphan_s = orphan.to_string();
    let found = rows
        .iter()
        .find(|(pid, _, _)| pid == &orphan_s)
        .expect("orphan project_id must appear in count_memories_by_project");
    assert_eq!(found.1, 1, "orphan pinned count");
    assert_eq!(found.2, 0, "orphan forgotten count");
    // No project_projection row for orphan (get would be None at CLI layer).
    assert!(
        conn.get_project_by_id(&orphan).unwrap().is_none(),
        "orphan must lack project_projection for AC8 display path"
    );
}

#[test]
fn list_memories__tag_prefix_sql__start_anchored_only() {
    let store = open_store();
    let a = ProjectId::new();
    register_project(&store, a, "A");
    pin_memory(&store, a, "TAGS: foo, bar\nbody");
    pin_memory(&store, a, "TAGS: foobar\nbody");
    // Capture-shaped storage (role prefix before TAGS:)
    pin_memory(&store, a, "ASSISTANT: TAGS: foo\nbody from capture shape");
    pin_memory(&store, a, "body with mid TAGS: foo elsewhere");

    let conn = store.connection();
    // SQL stage (tag Some) — TAGS: / role+TAGS: prefix rows, not mid-body.
    let filter = MemoryListFilter {
        status: MemoryListStatus::Pinned,
        project_id: Some(a),
        tag: Some("foo".to_string()),
        limit: 50,
    };
    let rows = conn.list_memories(&filter).unwrap();
    assert_eq!(
        rows.len(),
        3,
        "SQL stage returns TAGS: prefix rows only (not mid-body); got {:?}",
        rows.iter().map(|r| &r.content).collect::<Vec<_>>()
    );
    // Two-stage total for tag foo: exact token "foo" (not foobar).
    let total = conn.count_memories(&filter).unwrap();
    assert_eq!(
        total, 2,
        "token match: foo matches TAGS: foo, bar and ASSISTANT: TAGS: foo"
    );
}

#[test]
fn list_forgotten_memories__thin_wraps_list_memories() {
    let store = open_store();
    let a = ProjectId::new();
    register_project(&store, a, "A");
    let m = pin_memory(&store, a, "to forget");
    forget_memory(&store, m);
    let conn = store.connection();
    let legacy = conn.list_forgotten_memories(Some(a)).unwrap();
    assert_eq!(legacy.len(), 1);
    assert_eq!(legacy[0].0, m.to_string());
    let via_list = conn
        .list_memories(&MemoryListFilter {
            status: MemoryListStatus::Forgotten,
            project_id: Some(a),
            tag: None,
            limit: 50,
        })
        .unwrap();
    assert_eq!(via_list.len(), 1);
    assert_eq!(via_list[0].memory_id, legacy[0].0);
}

#[test]
fn list_memories__parameterized_sql__no_id_interpolation_smoke() {
    // AC16: project id with quote-like characters still binds safely (uuid form is safe;
    // this asserts the call path accepts binds without panic / SQL error).
    let store = open_store();
    let a = ProjectId::new();
    register_project(&store, a, "A");
    pin_memory(&store, a, "safe pin");
    let conn = store.connection();
    let filter = MemoryListFilter {
        status: MemoryListStatus::Pinned,
        project_id: Some(a),
        tag: None,
        limit: 10,
    };
    let rows = conn.list_memories(&filter).unwrap();
    assert_eq!(rows.len(), 1);
    let n = conn.count_memories(&filter).unwrap();
    assert_eq!(n, 1);
}

#[test]
fn list_authority_memories__older_tagged_decision__returned_at_limit_1() {
    let store = open_store();
    let a = ProjectId::new();
    register_project(&store, a, "A");
    let pin_id = pin_memory(
        &store,
        a,
        "ASSISTANT: TAGS: t287\nDECISION: T287a-store-needle",
    );
    pin_memory(&store, a, "## Objective newer dump for T287");
    let conn = store.connection();
    let rows = conn
        .list_authority_memories(&MemoryListFilter {
            status: MemoryListStatus::Pinned,
            project_id: Some(a),
            tag: None,
            limit: 1,
        })
        .unwrap();
    assert_eq!(rows.len(), 1, "pass-1 limit 1 must return the pin");
    assert_eq!(rows[0].memory_id, pin_id.to_string());
    assert!(
        rows[0].content.contains("DECISION:"),
        "authority row content; got {}",
        rows[0].content
    );

    let src = include_str!("../src/query_store.rs");
    assert!(src.contains("GLOB 'TAGS:*'"), "SQL extra must GLOB TAGS:*");
    assert!(
        src.contains("GLOB 'ASSISTANT: TAGS:*'"),
        "SQL extra must GLOB ASSISTANT: TAGS:*"
    );
    assert!(
        src.contains("GLOB 'DECISION:*'"),
        "SQL extra must GLOB DECISION:*"
    );
    assert!(
        src.contains("GLOB 'HOTSPOT:*'"),
        "SQL extra must GLOB HOTSPOT:*"
    );
    let glob_idx = src
        .find("GLOB 'DECISION:*'")
        .expect("DECISION GLOB present");
    let window = src.get(glob_idx.saturating_sub(80)..glob_idx.saturating_add(900));
    let extra = window.unwrap_or(src);
    let and_groups = extra.matches("AND (").count();
    assert!(
        extra.contains(" OR "),
        "authority extra is a single AND ( … OR … ) group"
    );
    assert!(
        and_groups <= 1,
        "must not stack two AND ( groups; window={extra}"
    );
}
