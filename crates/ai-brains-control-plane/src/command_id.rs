//! Deterministic command_id → domain-id derivation (T159 / T160).
//!
//! Daemon and CLI local mutation paths must produce **byte-identical** pre-assigned
//! domain ids for the same `(namespace, command_id)` so control-plane
//! detect-already-done and spool replay stay consistent across paths.
//!
//! Algorithm: uuid v5 over DNS namespace of the frozen string, then v5 of that
//! namespace with the command_id bytes (double-hash).

use uuid::Uuid;

/// UUID v5 namespace seed for `propose_conclusion` command_id → conclusion_id.
pub const NS_PROPOSE_CONCLUSION: &str = "ai-brains.command.propose_conclusion";

/// UUID v5 namespace seed for `propose_decision` command_id → decision_id.
pub const NS_PROPOSE_DECISION: &str = "ai-brains.command.propose_decision";

/// UUID v5 namespace seed for `request_erasure` command_id → ticket request_id.
pub const NS_REQUEST_ERASURE: &str = "ai-brains.command.request_erasure";

/// Derive a deterministic UUID from a frozen DNS-style namespace + command_id.
///
/// ```text
/// ns = uuid_v5(NAMESPACE_DNS, namespace_name.as_bytes())
/// id = uuid_v5(ns, command_id.as_bytes())
/// ```
pub fn id_from_command(namespace_name: &str, command_id: &str) -> Uuid {
    let ns = Uuid::new_v5(&Uuid::NAMESPACE_DNS, namespace_name.as_bytes());
    Uuid::new_v5(&ns, command_id.as_bytes())
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn id_from_command__same_inputs__same_uuid() {
        let a = id_from_command(NS_PROPOSE_CONCLUSION, "cmd-1");
        let b = id_from_command(NS_PROPOSE_CONCLUSION, "cmd-1");
        let c = id_from_command(NS_PROPOSE_CONCLUSION, "cmd-2");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn id_from_command__different_namespaces__different_uuid() {
        let a = id_from_command(NS_PROPOSE_CONCLUSION, "same");
        let b = id_from_command(NS_PROPOSE_DECISION, "same");
        let c = id_from_command(NS_REQUEST_ERASURE, "same");
        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(b, c);
    }

    #[test]
    fn namespace_strings__byte_stable() {
        assert_eq!(
            NS_PROPOSE_CONCLUSION.as_bytes(),
            b"ai-brains.command.propose_conclusion"
        );
        assert_eq!(
            NS_PROPOSE_DECISION.as_bytes(),
            b"ai-brains.command.propose_decision"
        );
        assert_eq!(
            NS_REQUEST_ERASURE.as_bytes(),
            b"ai-brains.command.request_erasure"
        );
    }

    #[test]
    fn id_from_command__golden_double_v5() {
        // Frozen algorithm: v5(DNS, ns_name) then v5(that, command_id).
        let expected_ns = Uuid::new_v5(&Uuid::NAMESPACE_DNS, NS_PROPOSE_CONCLUSION.as_bytes());
        let expected = Uuid::new_v5(&expected_ns, b"golden-cmd");
        assert_eq!(
            id_from_command(NS_PROPOSE_CONCLUSION, "golden-cmd"),
            expected
        );
    }

    #[test]
    fn command_id_derivation__cp_matches_legacy_daemon_namespaces() {
        // Byte-stable namespace strings locked in T159 services (now shared).
        assert_eq!(
            NS_PROPOSE_CONCLUSION,
            "ai-brains.command.propose_conclusion"
        );
        assert_eq!(NS_PROPOSE_DECISION, "ai-brains.command.propose_decision");
        assert_eq!(NS_REQUEST_ERASURE, "ai-brains.command.request_erasure");
        let cmd = "idempotent-cmd-42";
        let c = id_from_command(NS_PROPOSE_CONCLUSION, cmd);
        let d = id_from_command(NS_PROPOSE_DECISION, cmd);
        let e = id_from_command(NS_REQUEST_ERASURE, cmd);
        assert_ne!(c, d);
        assert_ne!(c, e);
        assert_eq!(c, id_from_command(NS_PROPOSE_CONCLUSION, cmd));
    }

    #[test]
    fn cli_local_propose__same_command_id__same_conclusion_id_as_daemon_helper() {
        // CLI local path and daemon both call this helper for pre-assigned ids.
        let command_id = "shared-command-id-t160";
        let conclusion_uuid = id_from_command(NS_PROPOSE_CONCLUSION, command_id);
        assert_eq!(
            conclusion_uuid,
            id_from_command(NS_PROPOSE_CONCLUSION, command_id)
        );
        assert_ne!(
            conclusion_uuid,
            id_from_command(NS_PROPOSE_DECISION, command_id)
        );
    }
}
