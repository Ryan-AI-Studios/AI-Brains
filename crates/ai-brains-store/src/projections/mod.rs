use crate::errors::Result;

use ai_brains_events::Envelope;
use rusqlite::Transaction;

pub mod briefing;
pub mod claim_conflict;
pub mod conclusion;
pub mod conflict;
pub mod content_envelope;
pub mod decision;
pub mod dependency;
pub mod evidence;
pub mod grant;
pub mod memory;
pub mod policy_log;
pub mod principal;
pub mod project;
pub mod recipe;
pub mod repository_identity;
pub mod review;
pub mod session;
pub mod source;
pub mod turn;
pub mod workspace;

pub trait Projection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()>;
}

pub fn apply_all(tx: &Transaction, envelope: &Envelope) -> Result<()> {
    project::ProjectProjection.apply(tx, envelope)?;
    session::SessionProjection.apply(tx, envelope)?;
    turn::TurnProjection.apply(tx, envelope)?;
    memory::MemoryProjection.apply(tx, envelope)?;
    conflict::ConflictProjection.apply(tx, envelope)?;
    recipe::RecipeProjection.apply(tx, envelope)?;
    // Governed source/evidence/dependency projections (T149).
    // Order: source first (FK parent), then evidence, then dependency edges.
    source::SourceProjection.apply(tx, envelope)?;
    evidence::EvidenceProjection.apply(tx, envelope)?;
    dependency::DependencyProjection.apply(tx, envelope)?;
    // Epistemic lifecycle (T150) — after dependency so evidence FKs exist.
    conclusion::ConclusionProjection.apply(tx, envelope)?;
    decision::DecisionProjection.apply(tx, envelope)?;
    review::ReviewProjection.apply(tx, envelope)?;
    claim_conflict::ClaimConflictProjection.apply(tx, envelope)?;
    // Scopes / principals / grants (T151) — no FK to epistemic tables.
    workspace::WorkspaceProjection.apply(tx, envelope)?;
    principal::PrincipalProjection.apply(tx, envelope)?;
    grant::GrantProjection.apply(tx, envelope)?;
    repository_identity::RepositoryIdentityProjection.apply(tx, envelope)?;
    policy_log::PolicyLogProjection.apply(tx, envelope)?;
    // Briefings + progressive query traces (T152).
    briefing::BriefingProjection.apply(tx, envelope)?;
    // Content-envelope erasure / tombstone event projections (T163).
    // Side stores (content_key_store, encrypted_content_blob) are not written here.
    content_envelope::ContentEnvelopeProjection.apply(tx, envelope)?;
    Ok(())
}
