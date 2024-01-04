use crate::LockResult;
use crate::Storage;
use crate::StorageItem;
use crate::StorageLock;
use async_trait::async_trait;

use color_eyre::eyre::Result;

use core::marker::PhantomData;

#[derive(Debug, Default)]
pub struct StorageNull<ITEM: StorageItem> {
    item_type: PhantomData<ITEM>,
    warnings_on_use: bool,
}

impl<ITEM: StorageItem> StorageNull<ITEM> {
    pub fn enable_warnings_on_use(&mut self) {
        self.warnings_on_use = true;
    }
}

#[async_trait]
impl<ITEM: StorageItem + std::marker::Send> Storage<ITEM> for StorageNull<ITEM> {
    async fn create(&self) -> Result<String> {
        if self.warnings_on_use {
            tracing::warn!("StorageNull create used!");
        }
        let mut tries = 10;
        loop {
            let id = nanoid::nanoid!();
            if !self.exists(&id).await? {
                return Ok(id);
            }

            tries -= 1;
            if tries <= 0 {
                todo!();
            }
        }
    }
    async fn exists(&self, _id: &str) -> Result<bool> {
        if self.warnings_on_use {
            tracing::warn!("StorageNull exists used!");
        }
        Ok(false)
    }

    async fn load(&self, _id: &str) -> Result<ITEM> {
        if self.warnings_on_use {
            tracing::warn!("StorageNull load used!");
        }
        let i = ITEM::default();

        Ok(i)
    }

    async fn save(&self, _id: &str, _item: &ITEM, _lock: &StorageLock) -> Result<()> {
        if self.warnings_on_use {
            tracing::warn!("StorageNull save used!");
        }
        Ok(())
    }
    async fn lock(&self, id: &str, who: &str) -> Result<LockResult<ITEM>> {
        if self.warnings_on_use {
            tracing::warn!("StorageNull lock used!");
        }
        let (lock, item) = {
            let lock = StorageLock::new(who);

            tracing::debug!("Lock[{who}]: Load {id}");
            let item = self.load(id).await.unwrap_or_default();

            (lock, item)
        };
        Ok(LockResult::Success { lock, item })
    }

    async fn unlock(&self, _id: &str, _lock: StorageLock) -> Result<()> {
        if self.warnings_on_use {
            tracing::warn!("StorageNull unlock used!");
        }

        Ok(())
    }

    async fn force_unlock(&self, _id: &str) -> Result<()> {
        if self.warnings_on_use {
            tracing::warn!("StorageNull force_unlock used!");
        }
        Ok(())
    }
    async fn verify_lock(&self, _id: &str, _lock: &StorageLock) -> Result<bool> {
        if self.warnings_on_use {
            tracing::warn!("StorageNull verify_lock used!");
        }
        Ok(true)
    }
}
