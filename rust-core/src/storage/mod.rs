//! Local encrypted storage helpers
use sled::Db;
use std::path::Path;

pub struct Store {
    db: Db,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, sled::Error> {
        Ok(Self { db: sled::open(path)? })
    }

    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<(), sled::Error> {
        self.db.insert(key, value)?;
        self.db.flush()?;
        Ok(())
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, sled::Error> {
        Ok(self.db.get(key)?.map(|v| v.to_vec()))
    }

    pub fn remove(&self, key: &[u8]) -> Result<(), sled::Error> {
        self.db.remove(key)?;
        Ok(())
    }
}
