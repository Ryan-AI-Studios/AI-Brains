use crate::envelope::Envelope;
use crate::errors::EventError;
use crate::version::CURRENT_SCHEMA_VERSION;

pub trait Upcast {
    fn upcast(self) -> Result<Envelope, EventError>;
}

impl Upcast for Envelope {
    fn upcast(self) -> Result<Envelope, EventError> {
        if self.schema_version == CURRENT_SCHEMA_VERSION {
            return Ok(self);
        }

        if self.schema_version > CURRENT_SCHEMA_VERSION {
            // Future version, we might want to degrade or fail
            // For now, let's allow it but maybe the payload is already unknown
            return Ok(self);
        }

        // Handle migration from v0 -> v1, etc.
        let mut current = self;
        while current.schema_version < CURRENT_SCHEMA_VERSION {
            current = upcast_once(current)?;
        }

        Ok(current)
    }
}

fn upcast_once(envelope: Envelope) -> Result<Envelope, EventError> {
    #[allow(clippy::match_single_binding)]
    match envelope.schema_version {
        // Example: 0 => { ... }
        _ => Err(EventError::UnknownVersion(envelope.envelope_version())),
    }
}

impl Envelope {
    pub fn envelope_version(&self) -> u32 {
        self.schema_version
    }
}
