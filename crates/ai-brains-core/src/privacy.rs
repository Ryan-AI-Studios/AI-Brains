use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum Privacy {
    CloudOk = 0,
    LocalOnly = 1,
    NeverInject = 2,
    #[default]
    Sealed = 3,
}

impl Privacy {
    pub fn combine(&self, other: Privacy) -> Privacy {
        if *self > other {
            *self
        } else {
            other
        }
    }
}
