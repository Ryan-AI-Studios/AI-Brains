//! T203 governed discovery list ports (sources / evidence).
#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

use ai_brains_control_plane::{
    AllowAllPolicy, DEFAULT_LIST_LIMIT, EvidenceListRow, GovernedQueryStore, MAX_LIST_LIMIT,
    ObserveSourceRequest, Sha256FingerprinterPort, SourceContent, StorePorts, SystemClock,
    clamp_list_limit, observe_source, scope_identity_key,
};
use ai_brains_core::ids::{PrincipalId, ProjectId, UserId};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_core::source::SourceKind;
use ai_brains_crypto::DataKey;
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use tempfile::NamedTempFile;
use uuid::Uuid;

fn open_ports() -> (NamedTempFile, StorePorts) {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(db_path, &sql_key).unwrap();
    conn.migrate().unwrap();
    let store = SqliteEventStore::new(conn);
    (temp_file, StorePorts::from_store(store))
}

fn repo_scope(n: u128) -> ScopeRef {
    ScopeRef::Repository(ProjectId::from_uuid(Uuid::from_u128(n)))
}

fn observe_file(ports: &StorePorts, scope: ScopeRef, name: &str, locator: &str, content: &[u8]) {
    let clock = SystemClock;
    let fp = Sha256FingerprinterPort::new();
    let policy = AllowAllPolicy;
    let req = ObserveSourceRequest {
        principal: PrincipalId::from_uuid(Uuid::from_u128(1)),
        scope,
        kind: SourceKind::File,
        display_name: name.into(),
        locator: Some(locator.into()),
        content: SourceContent::Bytes(content.to_vec()),
        privacy: Privacy::LocalOnly,
        run_invalidation: false,
    };
    observe_source(&ports.writer, &ports.query, &clock, &fp, &policy, req).expect("observe");
}

#[test]
fn clamp_list_limit__none_or_zero__defaults_to_50() {
    assert_eq!(clamp_list_limit(None), DEFAULT_LIST_LIMIT);
    assert_eq!(clamp_list_limit(Some(0)), DEFAULT_LIST_LIMIT);
}

#[test]
fn clamp_list_limit__over_max__clamps_to_200() {
    assert_eq!(clamp_list_limit(Some(9999)), MAX_LIST_LIMIT);
    assert_eq!(clamp_list_limit(Some(200)), MAX_LIST_LIMIT);
    assert_eq!(clamp_list_limit(Some(10)), 10);
}

#[test]
fn list_sources_for_scope__empty__returns_empty_vec() {
    let (_tmp, ports) = open_ports();
    let scope = repo_scope(0xA1);
    let key = scope_identity_key(&scope);
    let rows = ports.query.list_sources_for_scope(&key, 50).expect("list");
    assert!(rows.is_empty());
}

#[test]
fn list_sources_for_scope__happy__returns_active_in_scope() {
    let (_tmp, ports) = open_ports();
    let scope = repo_scope(0xA2);
    observe_file(&ports, scope.clone(), "readme", "/tmp/r.md", b"hello\n");
    let key = scope_identity_key(&scope);
    let rows = ports.query.list_sources_for_scope(&key, 50).expect("list");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].display_name, "readme");
    assert_eq!(rows[0].scope, key);
}

#[test]
fn list_sources_for_scope__cross_scope__isolated() {
    let (_tmp, ports) = open_ports();
    let a = repo_scope(0xB1);
    let b = repo_scope(0xB2);
    observe_file(&ports, a.clone(), "a-src", "/a", b"a\n");
    observe_file(&ports, b.clone(), "b-src", "/b", b"b\n");
    let rows_a = ports
        .query
        .list_sources_for_scope(&scope_identity_key(&a), 50)
        .expect("a");
    let rows_b = ports
        .query
        .list_sources_for_scope(&scope_identity_key(&b), 50)
        .expect("b");
    assert_eq!(rows_a.len(), 1);
    assert_eq!(rows_a[0].display_name, "a-src");
    assert_eq!(rows_b.len(), 1);
    assert_eq!(rows_b[0].display_name, "b-src");
}

#[test]
fn list_sources_for_scope__non_active__excluded() {
    let (_tmp, ports) = open_ports();
    let scope = repo_scope(0xC1);
    observe_file(&ports, scope.clone(), "live", "/live", b"x\n");
    let key = scope_identity_key(&scope);
    // Insert a non-Active row directly into projection.
    {
        let store = ports.store();
        let conn = store.connection().lock().unwrap();
        conn.execute(
            "INSERT INTO source_projection (
                source_id, scope, kind, display_name, locator, status,
                last_observed_at, recorded_at, updated_at
             ) VALUES (?, ?, '\"File\"', 'dead', '/dead', 'Unavailable',
                '2020-01-01T00:00:00Z', '2020-01-01T00:00:00Z', '2020-01-01T00:00:00Z')",
            rusqlite::params![Uuid::from_u128(0xDEAD).to_string(), key],
        )
        .unwrap();
    }
    let rows = ports.query.list_sources_for_scope(&key, 50).expect("list");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].display_name, "live");
}

#[test]
fn list_sources_for_scope__limit_plus_one__more_available_pattern() {
    let (_tmp, ports) = open_ports();
    let scope = repo_scope(0xD1);
    for i in 0..5 {
        observe_file(
            &ports,
            scope.clone(),
            &format!("s{i}"),
            &format!("/s{i}"),
            format!("c{i}\n").as_bytes(),
        );
    }
    let key = scope_identity_key(&scope);
    let page = 3usize;
    let mut rows = ports
        .query
        .list_sources_for_scope(&key, page + 1)
        .expect("list");
    let more = rows.len() > page;
    assert!(more, "expected more_available pattern");
    rows.truncate(page);
    assert_eq!(rows.len(), page);
}

#[test]
fn list_evidence_for_scope__empty__returns_empty() {
    let (_tmp, ports) = open_ports();
    let key = scope_identity_key(&repo_scope(0xE1));
    let rows = ports
        .query
        .list_evidence_for_scope(&key, None, 50)
        .expect("list");
    assert!(rows.is_empty());
}

#[test]
fn list_evidence_for_scope__happy_and_fts() {
    let (_tmp, ports) = open_ports();
    let scope = repo_scope(0xE2);
    // Observe creates evidence summary "Observed {display_name}".
    observe_file(
        &ports,
        scope.clone(),
        "unique_keyword_alpha",
        "/doc.md",
        b"content for fts\n",
    );
    let key = scope_identity_key(&scope);
    let all = ports
        .query
        .list_evidence_for_scope(&key, None, 50)
        .expect("plain list");
    assert_eq!(all.len(), 1);
    assert!(
        all[0].summary.contains("unique_keyword_alpha") || !all[0].summary.is_empty(),
        "summary={}",
        all[0].summary
    );

    let hits: Vec<EvidenceListRow> = ports
        .query
        .list_evidence_for_scope(&key, Some("unique_keyword_alpha"), 50)
        .expect("fts");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, all[0].id);

    let miss = ports
        .query
        .list_evidence_for_scope(&key, Some("zzznomatchtoken"), 50)
        .expect("miss");
    assert!(miss.is_empty());
}

#[test]
fn list_evidence_for_scope__empty_sanitized_query__plain_list() {
    let (_tmp, ports) = open_ports();
    let scope = repo_scope(0xE3);
    observe_file(&ports, scope.clone(), "n", "/n", b"body\n");
    let key = scope_identity_key(&scope);
    // Punctuation-only sanitizes to empty → plain list path.
    let rows = ports
        .query
        .list_evidence_for_scope(&key, Some("***"), 50)
        .expect("list");
    assert_eq!(rows.len(), 1);
}

#[test]
fn list_evidence_for_scope__cross_scope__isolated() {
    let (_tmp, ports) = open_ports();
    let a = repo_scope(0xF1);
    let b = repo_scope(0xF2);
    observe_file(&ports, a.clone(), "scope_a_only_token", "/a", b"a\n");
    observe_file(&ports, b.clone(), "scope_b_only_token", "/b", b"b\n");
    let rows = ports
        .query
        .list_evidence_for_scope(&scope_identity_key(&a), Some("scope_a_only_token"), 50)
        .expect("a");
    assert_eq!(rows.len(), 1);
    let leak = ports
        .query
        .list_evidence_for_scope(&scope_identity_key(&a), Some("scope_b_only_token"), 50)
        .expect("no leak");
    assert!(leak.is_empty());
}

#[test]
fn list_evidence_for_scope__non_active__excluded() {
    let (_tmp, ports) = open_ports();
    let scope = repo_scope(0xF3);
    observe_file(&ports, scope.clone(), "live", "/live", b"live evidence\n");
    let key = scope_identity_key(&scope);
    let live = ports
        .query
        .list_evidence_for_scope(&key, None, 50)
        .expect("live");
    assert_eq!(live.len(), 1);
    let live_id = live[0].id.to_string();
    let source_id = live[0].source_id.to_string();
    {
        let store = ports.store();
        let conn = store.connection().lock().unwrap();
        let dead_id = Uuid::from_u128(0xBEEF).to_string();
        conn.execute(
            "INSERT INTO evidence_projection (
                evidence_id, source_id, source_version_id, status, summary,
                privacy, model_provenance_json, fingerprint, recorded_at
             ) VALUES (?, ?, NULL, 'Erased', 'should not list', '\"LocalOnly\"', NULL, NULL,
                '2020-01-01T00:00:00Z')",
            rusqlite::params![dead_id, source_id],
        )
        .unwrap();
        let _ = live_id;
    }
    let rows = ports
        .query
        .list_evidence_for_scope(&key, None, 50)
        .expect("active only");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].status, "Active");
}

#[test]
fn list_evidence_for_scope__personal_scope_shape() {
    let (_tmp, ports) = open_ports();
    let scope = ScopeRef::Personal(UserId::from_uuid(Uuid::from_u128(9)));
    observe_file(&ports, scope.clone(), "p", "/p", b"personal\n");
    let rows = ports
        .query
        .list_evidence_for_scope(&scope_identity_key(&scope), None, 10)
        .expect("personal");
    assert_eq!(rows.len(), 1);
}
