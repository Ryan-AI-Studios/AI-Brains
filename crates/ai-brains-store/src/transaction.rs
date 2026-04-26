use crate::errors::Result;
use rusqlite::Transaction as SqliteTx;

pub struct Transaction<'conn> {
    inner: SqliteTx<'conn>,
}

impl<'conn> Transaction<'conn> {
    pub fn new(inner: SqliteTx<'conn>) -> Self {
        Self { inner }
    }

    pub fn commit(self) -> Result<()> {
        self.inner.commit()?;
        Ok(())
    }

    pub fn rollback(self) -> Result<()> {
        self.inner.rollback()?;
        Ok(())
    }
}

impl<'conn> std::ops::Deref for Transaction<'conn> {
    type Target = SqliteTx<'conn>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
