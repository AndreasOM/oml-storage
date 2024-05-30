use crate::LockResult;
/// This is a *Null* implementation that does nothing.
/// It can be used as a default, and can warn when actually being used.

#[cfg(feature = "metadata")]
use crate::Metadata;
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
    #[cfg(feature = "metadata")]
    metadata: Metadata<ITEM>,
}

impl<ITEM: StorageItem> StorageNull<ITEM> {
    pub fn enable_warnings_on_use(&mut self) {
        self.warnings_on_use = true;
    }
}

#[cfg(feature = "metadata")]
impl<ITEM: StorageItem> StorageNull<ITEM> {
    fn update_highest_seen_id(&self, id: &str) {
        self.metadata.update_highest_seen_id(id);
    }
}

#[cfg(not(feature = "metadata"))]
impl<ITEM: StorageItem> StorageNull<ITEM> {
    fn update_highest_seen_id(&self, _id: &str) {}
}

#[async_trait]
impl<ITEM: StorageItem + std::marker::Send> Storage<ITEM> for StorageNull<ITEM> {
    async fn ensure_storage_exists(&mut self) -> Result<()> {
        Ok(())
    }
    async fn create(&self) -> Result<String> {
        if self.warnings_on_use {
            tracing::warn!("StorageNull create used!");
        }
        let mut tries = 10;
        loop {
            let id = nanoid::nanoid!();
            if !self.exists(&id).await? {
                // NO! self.update_highest_seen_id( &id );
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

    async fn load(&self, id: &str) -> Result<ITEM> {
        if self.warnings_on_use {
            tracing::warn!("StorageNull load used!");
        }
        let i = ITEM::default();
        self.update_highest_seen_id(&id);

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
    async fn all_ids(&self) -> Result<Vec<String>> {
        if self.warnings_on_use {
            tracing::warn!("StorageNull all_ids used!");
        }
        Ok(Vec::default())
    }
    async fn display_lock(&self, _id: &str) -> Result<String> {
        if self.warnings_on_use {
            tracing::warn!("StorageNull all_ids used!");
        }
        Ok(String::default())
    }

    #[cfg(feature = "metadata")]
    async fn metadata_highest_seen_id(&self) -> String {
        if self.warnings_on_use {
            tracing::warn!("StorageNull metadata_highest_seen_id used!");
        }
        self.metadata.highest_seen_id()
    }
}

#[cfg(test)]
mod tests {
    use crate::Storage;
    use crate::StorageItem;
    use crate::StorageNull;
    use color_eyre::Result;
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Default, Debug, Serialize, Deserialize)]
    struct TestItem {}

    impl StorageItem for TestItem {
        fn serialize(&self) -> Result<Vec<u8>> {
            todo!()
        }
        fn deserialize(_: &[u8]) -> Result<Self> {
            todo!()
        }
    }

    #[test]
    fn it_debugs() {
        let storage = StorageNull::<TestItem>::default();
        println!("{storage:?}");

        let storage: Box<dyn Storage<TestItem>> = Box::new(storage);
        println!("{storage:?}");
    }
}
