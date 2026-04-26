#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    Likely,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretKind {
    BearerToken,
    PrivateKey,
    ConnectionString,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub kind: SecretKind,
    pub confidence: Confidence,
    pub start: usize,
    pub end: usize,
}

impl Finding {
    pub fn new(kind: SecretKind, confidence: Confidence, start: usize, end: usize) -> Self {
        Self {
            kind,
            confidence,
            start,
            end,
        }
    }
}
