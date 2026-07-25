//! Governed-memory control plane: ports, adapters, and workflows (T148/T149).

pub mod adapters;
pub mod errors;
pub mod invalidation;
pub mod ports;
pub mod sources;

pub use adapters::{
    AllowAllPolicy, DenyAllPolicy, Sha256FingerprinterPort, StoreEventWriter, StoreGovernedQuery,
    StorePorts, SystemClock,
};
pub use errors::{ControlPlaneError, Result};
pub use invalidation::{
    InvalidationResult, SourceUnavailableRequest, invalidate_dependents_for_changed_source,
    mark_source_unavailable, plan_invalidation_events_for_changed_source,
    revalidate_matching_stale, try_mark_stale_payload,
};
pub use ports::{
    Clock, EventWriter, Fingerprinter, GovernedQueryStore, PolicyEvaluator, StaleFact,
};
pub use sources::{
    ObserveSourceRequest, ObserveSourceResult, SourceContent, normalize_path_locator,
    observe_source, scope_identity_key, source_identity_string,
};
