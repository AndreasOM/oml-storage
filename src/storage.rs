use crate::StorageItem;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;
use color_eyre::eyre::Result;
use serde::Deserialize;
use serde::Serialize;

/// The interface to all storage backends.
///
/// Note:
/// ```
/// // 'life0, 'life1, 'async_trait
/// ```
/// Are mostly just noise in the documentation, and I didn't figure out how to remove it yet.
///
/// You can just ignore them. In the end the `fn` are just `async` and return a [color_eyre::eyre::Result]
#[async_trait]
pub trait Storage<ITEM: StorageItem + Sized>: Send + Sync + std::fmt::Debug {
    /// Ensure the storage layer actually exists
    async fn ensure_storage_exists(&mut self) -> Result<()>;

    /// Creates a new item with a random id.
    /// If you want a specific it use [Storage::lock] instead.
    /// Warning: `id` creation is still work-in-progress.
    async fn create(&self) -> Result<String>;
    async fn exists(&self, id: &str) -> Result<bool>;
    async fn load(&self, id: &str) -> Result<ITEM>;
    async fn save(&self, id: &str, item: &ITEM, lock: &StorageLock) -> Result<()>;

    /// Tries to lock an (existing or new) item
    async fn lock(&self, id: &str, who: &str) -> Result<LockResult<ITEM>>;
    async fn unlock(&self, id: &str, lock: StorageLock) -> Result<()>;

    async fn force_unlock(&self, id: &str) -> Result<()>;
    async fn verify_lock(&self, id: &str, lock: &StorageLock) -> Result<bool>;

    // Experimental
    /// Returns all ids. This is a :HACK: and we will probably switch to an iterator at some point
    async fn all_ids(&self) -> Result<Vec<String>>;
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
