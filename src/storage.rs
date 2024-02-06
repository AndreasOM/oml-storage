use crate::StorageItem;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;
use color_eyre::eyre::Result;
use serde::Deserialize;
use serde::Serialize;

#[async_trait]
pub trait Storage<ITEM: StorageItem + Sized>: Send + Sync + std::fmt::Debug {
    async fn ensure_storage_exists(&mut self) -> Result<()>;
    async fn create(&self) -> Result<String>;
    async fn exists(&self, id: &str) -> Result<bool>;
    async fn load(&self, id: &str) -> Result<ITEM>;
    async fn save(&self, id: &str, item: &ITEM, lock: &StorageLock) -> Result<()>;

    async fn lock(&self, id: &str, who: &str) -> Result<LockResult<ITEM>>;
    async fn unlock(&self, id: &str, lock: StorageLock) -> Result<()>;

    async fn force_unlock(&self, id: &str) -> Result<()>;
    async fn verify_lock(&self, id: &str, lock: &StorageLock) -> Result<bool>;
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct StorageLock {
    who: String,
    when: DateTime<Utc>,
}

impl StorageLock {
    pub fn new(who: &str) -> Self {
        Self {
            who: who.to_string(),
            when: Utc::now(),
        }
    }
}

#[derive(Debug)]
pub enum LockResult<ITEM> {
    Success { lock: StorageLock, item: ITEM },
    AlreadyLocked { who: String },
}
