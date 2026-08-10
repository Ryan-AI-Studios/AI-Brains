//! T218 hermetic: injection seam + synthetic embedding BLOBs (AC10 / AC20).
//!
//! No network — query vectors and stored f32 LE BLOBs are synthetic.

#![allow(non_snake_case)]

mod common;

use ai_brains_core::privacy::Privacy;
use ai_brains_retrieval::{
    SEMANTIC_MIN_COSINE, apply_dual_semantic_floor, filter_by_cosine_floor, has_fts_arm, rrf_fuse,
    semantic_search_with_embedding,
};
use ai_brains_store::QueryStore;

/// Pack f32 LE bytes for a unit-ish vector.
fn f32_le_blob(v: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * 4);
    for x in v {
        out.extend_from_slice(&x.to_le_bytes());
    }
    out
}

/// AC20: injection seam callable without HTTP (compile + hermetic).
#[test]
fn semantic_search_with_embedding__synthetic_blob__no_network__ac20()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::store_with_memory(
        "DECISION: path TOCTOU openat cap-std hardening",
        Privacy::CloudOk,
    )?;
    let conn = store.connection();

    // Discover the pinned memory_id from projection.
    let memory_id: String = {
        let db = conn.lock()?;
        db.query_row(
            "SELECT memory_id FROM memory_projection WHERE status = 'pinned' LIMIT 1",
            [],
            |row| row.get(0),
        )?
    };

    // Identical 4-d vectors → cosine ≈ 1.0 (well above floors).
    let vec = vec![1.0f32, 0.0, 0.0, 0.0];
    conn.store_embedding(&memory_id, &f32_le_blob(&vec))?;

    let outcome = semantic_search_with_embedding(conn, &vec, 15, None, None, None)?;
    assert_eq!(
        outcome.embedding.status, "ok",
        "synthetic BLOBs must score without network; detail={:?}",
        outcome.embedding.detail
    );
    assert_eq!(outcome.hits.len(), 1);
    assert_eq!(outcome.hits[0].memory_id, memory_id);
    assert_eq!(outcome.hits[0].source, "semantic");
    let cos = outcome.hits[0].cosine.expect("cosine set");
    assert!(
        (cos - 1.0).abs() < 1e-5,
        "identical vectors → cosine≈1; got {cos}"
    );
    assert_eq!(outcome.hits[0].score, Some(cos));
    Ok(())
}

/// AC10: hermetic hybrid fuse path via injection seam + synthetic BLOBs.
#[test]
fn hybrid_fuse__injection_seam_synthetic_blobs__ac10() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::store_with_memory(
        "DECISION: RRF hybrid cosine floor ScoreKind",
        Privacy::CloudOk,
    )?;
    let conn = store.connection();

    let memory_id: String = {
        let db = conn.lock()?;
        db.query_row(
            "SELECT memory_id FROM memory_projection WHERE status = 'pinned' LIMIT 1",
            [],
            |row| row.get(0),
        )?
    };

    // Query vec aligns with stored → high cosine.
    let stored = vec![0.6f32, 0.8, 0.0, 0.0];
    conn.store_embedding(&memory_id, &f32_le_blob(&stored))?;

    let outcome =
        semantic_search_with_embedding(conn, &stored, 15, None, None, Some(SEMANTIC_MIN_COSINE))?;
    assert_eq!(outcome.embedding.status, "ok");
    assert!(!outcome.hits.is_empty());
    let sem = &outcome.hits;
    assert!(
        sem.iter().all(|h| h.cosine.is_some()),
        "injection seam must set cosine"
    );

    // Synthetic FTS arm with same id → hybrid source + cosine preserved (F4).
    let fts = vec![ai_brains_retrieval::RecallHit::fts(
        memory_id.clone(),
        "FROM_FTS lexical".into(),
        Some(-3.0),
        None,
        None,
    )];
    let fused = rrf_fuse(&fts, sem, 60.0);
    assert_eq!(fused.len(), 1);
    assert_eq!(fused[0].source, "hybrid");
    assert_eq!(fused[0].content, "FROM_FTS lexical");
    assert!(
        fused[0].cosine.is_some_and(|c| c > 0.9),
        "fused must preserve pre-fuse cosine; got {:?}",
        fused[0].cosine
    );
    // RRF score is rank contribution (~0.0328 for both-list rank1), not cosine.
    let rrf = fused[0].score.expect("rrf score");
    assert!(
        rrf < 0.1 && rrf > 0.01,
        "RRF score expected ~0.03; got {rrf}"
    );

    // Dual floor: with FTS arm, weak cosine in [0.55,0.60) stays eligible.
    assert!(has_fts_arm(&fts));
    let weak_sem = filter_by_cosine_floor(
        vec![ai_brains_retrieval::RecallHit::semantic(
            "weak".into(),
            "weak".into(),
            Some(0.57),
            None,
            None,
            None,
        )],
        SEMANTIC_MIN_COSINE,
    );
    let after = apply_dual_semantic_floor(&fts, weak_sem, None);
    assert_eq!(after.len(), 1, "FTS arm keeps [0.55,0.60) for RRF");

    Ok(())
}

/// Hybrid-arm floor via injection seam: below 0.55 dropped.
#[test]
fn semantic_search_with_embedding__below_hybrid_floor_dropped()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::store_with_memory("noise pin content", Privacy::CloudOk)?;
    let conn = store.connection();
    let memory_id: String = {
        let db = conn.lock()?;
        db.query_row(
            "SELECT memory_id FROM memory_projection WHERE status = 'pinned' LIMIT 1",
            [],
            |row| row.get(0),
        )?
    };

    // Orthogonal-ish vectors: e1 vs e2 → cosine 0.
    conn.store_embedding(&memory_id, &f32_le_blob(&[1.0, 0.0, 0.0, 0.0]))?;
    let outcome =
        semantic_search_with_embedding(conn, &[0.0, 1.0, 0.0, 0.0], 15, None, None, None)?;
    assert_eq!(outcome.embedding.status, "ok");
    assert!(
        outcome.hits.is_empty(),
        "cosine 0 must be below hybrid floor; got {:?}",
        outcome.hits
    );
    Ok(())
}
