#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T164 — store integration: real AEAD → side-store → open; destroy wrap → fail closed.

use ai_brains_core::ids::ContentKeyId;
use ai_brains_crypto::content_envelope::{
    ALGORITHM_LABEL, ENVELOPE_SCHEMA_VERSION, SealAad, generate_wrap_and_seal, open,
    unwrap_and_open,
};
use ai_brains_crypto::content_key_store::{
    ContentDek, WRAP_SCHEMA_VERSION, WrappedContentDek, parse_nonce, unwrap_content_dek,
    wrap_content_dek,
};
use ai_brains_crypto::{CryptoError, DataKey};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::SqliteEventStore;
use ai_brains_store::projections::content_envelope::{
    self, ALGORITHM_AES_256_GCM, ContentKeyWrapRow,
    ENVELOPE_SCHEMA_VERSION as STORE_ENVELOPE_SCHEMA_VERSION, EncryptedBlobRow,
};
use tempfile::NamedTempFile;
use uuid::Uuid;

const CREATED_AT: &str = "2026-07-28T12:00:00Z";
const DESTROYED_AT: &str = "2026-07-28T13:00:00Z";

fn open_store() -> (NamedTempFile, SqliteEventStore) {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(db_path, &sql_key).unwrap();
    conn.migrate().unwrap();
    (temp_file, SqliteEventStore::new(conn))
}

/// Store-layer open adapter (C14): maps destroyed / missing wrap material to
/// [`CryptoError::AuthenticationFailed`] without inventing Option-shaped crypto types.
///
/// Only constructs [`WrappedContentDek`] when wrap status is active and nonce/ciphertext
/// are present and non-empty; otherwise fails closed before AEAD.
fn open_from_store_rows(
    data_key: &DataKey,
    content_key_id: &ContentKeyId,
    wrap_row: &ContentKeyWrapRow,
    sealed: &ai_brains_crypto::SealedContent,
    blob_id: Uuid,
) -> Result<Vec<u8>, CryptoError> {
    let wrap = match wrap_row {
        row if row.status == "active" => row,
        _ => return Err(CryptoError::AuthenticationFailed),
    };
    let (nonce_bytes, ciphertext) = match (&wrap.wrap_nonce, &wrap.wrap_ciphertext) {
        (Some(n), Some(c)) if !n.is_empty() && !c.is_empty() => (n.as_slice(), c.as_slice()),
        _ => return Err(CryptoError::AuthenticationFailed),
    };
    let nonce = parse_nonce(nonce_bytes)?;
    let wrapped = WrappedContentDek {
        wrap_schema_version: wrap.wrap_schema_version as u32,
        nonce,
        ciphertext: ciphertext.to_vec(),
    };
    let opened = unwrap_and_open(data_key, content_key_id, &wrapped, sealed, blob_id)?;
    Ok(opened.to_vec())
}

#[test]
fn content_envelope_crypto__persist_and_open__round_trip() {
    let (_tmp, store) = open_store();
    let data_key = DataKey::generate();
    let content_key_id = ContentKeyId::new();
    let blob_id = Uuid::new_v4();
    let plaintext = b"persist-and-open round trip UTF-8: hello";

    let env = generate_wrap_and_seal(&data_key, content_key_id, blob_id, plaintext)
        .expect("generate_wrap_and_seal");

    {
        let conn = store.connection().lock().unwrap();
        content_envelope::insert_content_key_wrap(
            &conn,
            &content_key_id.to_string(),
            i64::from(env.wrapped_dek.wrap_schema_version),
            &env.wrapped_dek.nonce,
            &env.wrapped_dek.ciphertext,
            CREATED_AT,
        )
        .expect("insert wrap");

        let row = EncryptedBlobRow {
            blob_id: blob_id.to_string(),
            content_key_id: content_key_id.to_string(),
            envelope_schema_version: i64::from(env.sealed.envelope_schema_version),
            algorithm: ALGORITHM_AES_256_GCM.to_string(),
            nonce: env.sealed.nonce.to_vec(),
            ciphertext: env.sealed.ciphertext.clone(),
            content_class: None,
            subject_kind: None,
            subject_id: None,
            size_bytes: env.sealed.ciphertext.len() as i64,
            created_at: CREATED_AT.to_string(),
        };
        content_envelope::insert_encrypted_blob(&conn, &row).expect("insert blob");
    }

    // Reload from store and open via fail-closed store adapter (success path).
    let conn = store.connection().lock().unwrap();
    let wrap_row = content_envelope::get_content_key_wrap(&conn, &content_key_id.to_string())
        .expect("get wrap")
        .expect("wrap exists");
    assert_eq!(wrap_row.status, "active");

    let blob = content_envelope::get_encrypted_blob(&conn, &blob_id.to_string())
        .expect("get blob")
        .expect("blob exists");
    assert_eq!(blob.algorithm, ALGORITHM_LABEL);
    assert_eq!(blob.envelope_schema_version, STORE_ENVELOPE_SCHEMA_VERSION);
    assert_eq!(
        blob.envelope_schema_version,
        i64::from(ENVELOPE_SCHEMA_VERSION)
    );
    assert_eq!(blob.size_bytes, blob.ciphertext.len() as i64);

    let sealed = ai_brains_crypto::SealedContent {
        envelope_schema_version: blob.envelope_schema_version as u32,
        nonce: parse_nonce(&blob.nonce).expect("blob nonce"),
        ciphertext: blob.ciphertext,
    };

    let opened = open_from_store_rows(&data_key, &content_key_id, &wrap_row, &sealed, blob_id)
        .expect("open_from_store_rows");
    assert_eq!(opened.as_slice(), plaintext);

    // Also path: unwrap DEK then open with SealAad (direct crypto, wrap fields present).
    let wrap_nonce =
        parse_nonce(wrap_row.wrap_nonce.as_ref().expect("nonce")).expect("12-byte nonce");
    let wrap_ct = wrap_row.wrap_ciphertext.as_ref().expect("ciphertext");
    let wrapped = WrappedContentDek {
        wrap_schema_version: wrap_row.wrap_schema_version as u32,
        nonce: wrap_nonce,
        ciphertext: wrap_ct.clone(),
    };
    let dek = unwrap_content_dek(&data_key, &wrapped, &content_key_id).expect("unwrap dek");
    let aad = SealAad {
        envelope_schema_version: sealed.envelope_schema_version,
        content_key_id,
        blob_id,
    };
    let opened2 = open(&dek, &sealed, &aad).expect("open");
    assert_eq!(opened2.as_slice(), plaintext);
}

#[test]
fn content_envelope_crypto__destroy_wrap__cannot_open() {
    let (_tmp, store) = open_store();
    let data_key = DataKey::generate();
    let content_key_id = ContentKeyId::new();
    let blob_id = Uuid::new_v4();
    let plaintext = b"will become unrecoverable";

    let dek = ContentDek::generate().expect("dek");
    let wrapped = wrap_content_dek(&data_key, &dek, &content_key_id).expect("wrap");
    let aad = SealAad {
        envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
        content_key_id,
        blob_id,
    };
    let sealed = ai_brains_crypto::seal(&dek, plaintext, &aad).expect("seal");
    // Drop plaintext DEK (zeroize on drop).
    drop(dek);

    {
        let conn = store.connection().lock().unwrap();
        content_envelope::insert_content_key_wrap(
            &conn,
            &content_key_id.to_string(),
            i64::from(WRAP_SCHEMA_VERSION),
            &wrapped.nonce,
            &wrapped.ciphertext,
            CREATED_AT,
        )
        .expect("insert wrap");
        let row = EncryptedBlobRow {
            blob_id: blob_id.to_string(),
            content_key_id: content_key_id.to_string(),
            envelope_schema_version: i64::from(ENVELOPE_SCHEMA_VERSION),
            algorithm: ALGORITHM_AES_256_GCM.to_string(),
            nonce: sealed.nonce.to_vec(),
            ciphertext: sealed.ciphertext.clone(),
            content_class: Some("memory".to_string()),
            subject_kind: None,
            subject_id: None,
            size_bytes: sealed.ciphertext.len() as i64,
            created_at: CREATED_AT.to_string(),
        };
        content_envelope::insert_encrypted_blob(&conn, &row).expect("insert blob");
    }

    // Prove open works before destroy via the same store adapter.
    {
        let conn = store.connection().lock().unwrap();
        let wrap_row = content_envelope::get_content_key_wrap(&conn, &content_key_id.to_string())
            .unwrap()
            .unwrap();
        let blob = content_envelope::get_encrypted_blob(&conn, &blob_id.to_string())
            .unwrap()
            .unwrap();
        let sealed_live = ai_brains_crypto::SealedContent {
            envelope_schema_version: blob.envelope_schema_version as u32,
            nonce: parse_nonce(&blob.nonce).unwrap(),
            ciphertext: blob.ciphertext,
        };
        let opened =
            open_from_store_rows(&data_key, &content_key_id, &wrap_row, &sealed_live, blob_id)
                .expect("open before destroy");
        assert_eq!(opened.as_slice(), plaintext);
    }

    // Destroy wrap (CE primitive at store layer).
    {
        let conn = store.connection().lock().unwrap();
        content_envelope::destroy_content_key_wrap(
            &conn,
            &content_key_id.to_string(),
            DESTROYED_AT,
        )
        .expect("destroy");
        assert!(
            content_envelope::is_content_key_destroyed(&conn, &content_key_id.to_string()).unwrap()
        );
    }

    // Fail closed: destroyed row has None wraps — open path maps via real branch.
    let conn = store.connection().lock().unwrap();
    let wrap_row = content_envelope::get_content_key_wrap(&conn, &content_key_id.to_string())
        .unwrap()
        .expect("row still present as destroyed");
    assert_eq!(wrap_row.status, "destroyed");
    assert!(wrap_row.wrap_nonce.is_none());
    assert!(wrap_row.wrap_ciphertext.is_none());

    // Ciphertext blob may remain (undecryptable garbage) — CE destroys keys, not media.
    let blob = content_envelope::get_encrypted_blob(&conn, &blob_id.to_string())
        .unwrap()
        .expect("blob retained");
    assert!(!blob.ciphertext.is_empty());
    let sealed_orphan = ai_brains_crypto::SealedContent {
        envelope_schema_version: blob.envelope_schema_version as u32,
        nonce: parse_nonce(&blob.nonce).expect("blob nonce"),
        ciphertext: blob.ciphertext,
    };

    // Realistic open path: destroyed/missing wrap → AuthenticationFailed from the
    // actual fail-closed branch (no hardcoded Err literal; no fake wrap material).
    let open_result = open_from_store_rows(
        &data_key,
        &content_key_id,
        &wrap_row,
        &sealed_orphan,
        blob_id,
    );
    assert!(
        matches!(open_result, Err(CryptoError::AuthenticationFailed)),
        "destroyed wrap must fail closed via store adapter, got: {open_result:?}"
    );
}
