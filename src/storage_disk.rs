use crate::Storage;
use crate::StorageItem;
use async_trait::async_trait;
use color_eyre::eyre::Result;
use core::marker::PhantomData;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
pub struct StorageDisk<ITEM: StorageItem> {
    base_path: PathBuf,
    extension: PathBuf,
    item_type: PhantomData<ITEM>,
}

impl<ITEM: StorageItem> StorageDisk<ITEM> {
    pub async fn new(base_path: &Path, extension: &Path) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
            extension: extension.to_path_buf(),
            item_type: PhantomData,
        }
    }

    fn file_path(&self, id: &str) -> PathBuf {
        let mut p = PathBuf::new();
        p.push(&self.base_path);
        let idp = Path::new(id);
        p.push(idp);
        p.set_extension(&self.extension);

        p
    }
}

#[async_trait]
impl<ITEM: StorageItem + std::marker::Send> Storage<ITEM> for StorageDisk<ITEM> {
    async fn create(&self) -> Result<String> {
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
    async fn exists(&self, id: &str) -> Result<bool> {
        let p = self.file_path(id);
        tracing::debug!("{p:?}");
        Ok(fs::metadata(p).is_ok())
    }

    async fn load(&self, id: &str) -> Result<ITEM> {
        let p = self.file_path(id);
        let b = fs::read(p)?;
        let i = ITEM::deserialize(&b)?;

        Ok(i)
    }

    async fn save(&self, id: &str, item: &ITEM) -> Result<()> {
        let p = self.file_path(id);
        let b = item.serialize()?;
        fs::write(p, b)?;
        Ok(())
    }
}
