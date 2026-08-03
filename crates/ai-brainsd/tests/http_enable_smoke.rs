//! T161: daemon HTTP adapter unit smoke (enable flags + dispatch port).
#![allow(clippy::disallowed_methods, non_snake_case)]

use std::sync::Arc;

use ai_brains_api_server::dispatch::HttpDispatch;
use ai_brains_api_server::{is_loopback_addr, resolve_bind_addr};
use ai_brains_crypto::SqlCipherKey;
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::SqliteEventStore;
use ai_brainsd::DaemonWriter;
use ai_brainsd::http_adapter::{
    DaemonHttpDispatch, http_enabled_from_env_and_args, parse_http_bind_arg,
};
use ai_brainsd::services::GovernedServices;
use tempfile::tempdir;

#[test]
fn http_enable__http_flag__true() {
    assert!(http_enabled_from_env_and_args(&[
        "ai-brainsd".into(),
        "--http".into()
    ]));
}

#[test]
fn http_bind_arg__parses_explicit() {
    let args = vec![
        "ai-brainsd".into(),
        "--http".into(),
        "--http-bind".into(),
        "127.0.0.1:0".into(),
    ];
    assert_eq!(parse_http_bind_arg(&args).as_deref(), Some("127.0.0.1:0"));
    let addr = resolve_bind_addr(parse_http_bind_arg(&args).as_deref(), None).unwrap();
    assert!(is_loopback_addr(addr.ip()));
    assert_eq!(addr.port(), 0);
}

#[tokio::test]
async fn http_dispatch__ping__returns_pong() {
    let _allow = ai_brains_core::temp_env::TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    let spool = dir.path().join("spool");
    let key = SqlCipherKey::from_raw(
        "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
    );
    let conn = VaultConnection::open(&vault, &key).expect("open vault");
    conn.migrate().expect("migrate");
    let store = Arc::new(SqliteEventStore::new(conn));
    let writer = DaemonWriter::start(spool, store.clone())
        .await
        .expect("writer");
    let services = GovernedServices::new(store);
    let dispatch = DaemonHttpDispatch::new(writer, services);
    let resp = dispatch
        .dispatch(DaemonRequest::Ping)
        .await
        .expect("dispatch");
    assert!(matches!(resp, DaemonResponse::Pong));
}
