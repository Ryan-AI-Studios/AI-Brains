use crate::errors::Result;

use ai_brains_events::Envelope;
use rusqlite::Transaction;

pub mod conflict;
pub mod dependency;
pub mod evidence;
pub mod memory;
pub mod project;
pub mod recipe;
pub mod session;
pub mod source;
pub mod turn;

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
    Ok(())
}
