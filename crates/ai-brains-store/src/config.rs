use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct StoreConfig {
    pub path: PathBuf,
}

impl StoreConfig {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn in_memory() -> Self {
        Self {
            path: PathBuf::from(":memory:"),
        }
    }
}
