#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceDecision {
    Proceed,
    AlreadyRunning,
    ProbeFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeOutcome {
    Pong,
    NoResponse,
    ConnectFailed,
}

impl InstanceDecision {
    pub fn from_probe(outcome: ProbeOutcome) -> Self {
        match outcome {
            ProbeOutcome::Pong => InstanceDecision::AlreadyRunning,
            ProbeOutcome::NoResponse => InstanceDecision::Proceed,
            ProbeOutcome::ConnectFailed => InstanceDecision::Proceed,
        }
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn instance_decision__probe_pong__returns_already_running() {
        assert_eq!(
            InstanceDecision::from_probe(ProbeOutcome::Pong),
            InstanceDecision::AlreadyRunning,
        );
    }

    #[test]
    fn instance_decision__probe_no_response__returns_proceed() {
        assert_eq!(
            InstanceDecision::from_probe(ProbeOutcome::NoResponse),
            InstanceDecision::Proceed,
        );
    }

    #[test]
    fn instance_decision__probe_connect_failed__returns_proceed() {
        assert_eq!(
            InstanceDecision::from_probe(ProbeOutcome::ConnectFailed),
            InstanceDecision::Proceed,
        );
    }
}
