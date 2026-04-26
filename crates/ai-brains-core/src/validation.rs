use crate::errors::{Error, Result};

pub fn validate_content(content: &str) -> Result<()> {
    if content.trim().is_empty() {
        return Err(Error::EmptyContent);
    }
    Ok(())
}
