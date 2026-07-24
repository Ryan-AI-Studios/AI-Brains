//! Governed-memory control plane: **ports only** (T148).
//!
//! No workflows, SQLite adapters, or CLI wiring in this crate.

pub mod errors;
pub mod ports;

pub use errors::{ControlPlaneError, Result};
pub use ports::{Clock, EventWriter, Fingerprinter, GovernedQueryStore, PolicyEvaluator};
