#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectPath {
    canonical: String,
    display: String,
}

impl ProjectPath {
    pub(crate) fn new(canonical: String, display: String) -> Self {
        Self { canonical, display }
    }

    pub fn canonical(&self) -> &str {
        &self.canonical
    }

    pub fn display(&self) -> &str {
        &self.display
    }
}
