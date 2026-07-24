use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum Privacy {
    #[serde(
        rename = "Public",
        alias = "cloudok",
        alias = "cloud_ok",
        alias = "CloudOk",
        alias = "public"
    )]
    CloudOk = 0,
    #[serde(
        rename = "ProjectLocal",
        alias = "localonly",
        alias = "local_only",
        alias = "LocalOnly",
        alias = "projectlocal",
        alias = "project_local"
    )]
    LocalOnly = 1,
    #[serde(
        rename = "Private",
        alias = "neverinject",
        alias = "never_inject",
        alias = "NeverInject",
        alias = "private"
    )]
    NeverInject = 2,
    #[default]
    #[serde(rename = "Sealed", alias = "sealed", alias = "Sealed")]
    Sealed = 3,
}

impl Privacy {
    pub fn combine(&self, other: Privacy) -> Privacy {
        if *self > other { *self } else { other }
    }
}
