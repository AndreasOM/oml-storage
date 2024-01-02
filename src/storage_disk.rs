use crate::LockResult;
use crate::Storage;
use crate::StorageItem;
use crate::StorageLock;
use async_trait::async_trait;
use color_eyre::eyre::eyre;
use color_eyre::eyre::Result;
use tokio::sync::Semaphore;

use core::marker::PhantomData;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
pub struct StorageDisk<ITEM: StorageItem> {
    base_path: PathBuf,
    extension: PathBuf,
    item_type: PhantomData<ITEM>,
    lock_semaphore: Semaphore,
}

impl<ITEM: StorageItem> StorageDisk<ITEM> {
    pub async fn new(base_path: &Path, extension: &Path) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
            extension: extension.to_path_buf(),
            item_type: PhantomData,
            lock_semaphore: Semaphore::new(1),
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
    fn lock_path(&self, id: &str) -> PathBuf {
        let mut p = PathBuf::new();
        p.push(&self.base_path);
        let idp = Path::new(id);
        p.push(idp);
        p.set_extension("lock");

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
        let b = fs::read(p.clone()).map_err(|e| eyre!("{e} -> {p:?}"))?;
        let i = ITEM::deserialize(&b)?;

        Ok(i)
    }

    async fn save(&self, id: &str, item: &ITEM, lock: &StorageLock) -> Result<()> {
        if !self.verify_lock(id, lock).await? {
            Err(eyre!("Lock invalid!"))
        } else {
            let p = self.file_path(id);
            let b = item.serialize()?;
            fs::write(p, b)?;
            Ok(())
        }
    }
    async fn lock(&self, id: &str, who: &str) -> Result<LockResult<ITEM>> {
        let l = self.lock_path(id);
        let (lock, item) = {
            let sem = self.lock_semaphore.acquire().await?;
            tracing::debug!("Lock[{who}]: Got Semaphore");

            tracing::debug!("Lock[{who}]: Does {l:?} exist");

            if fs::metadata(&l).is_ok() {
                tracing::warn!("Lockfile {l:?} already exists");
                drop(sem);
                tracing::debug!("Lock[{who}]: Dropped Semaphore"); // close enough
                                                                   //return Err(eyre!("Already locked"));
                                                                   // :TODO: load lock
                return Ok(LockResult::AlreadyLocked {
                    who: String::from(":TODO:"),
                });
            }

            let lock = StorageLock::new(who);
            let lock_json = serde_json::to_string_pretty(&lock)?;

            tracing::debug!("Lock[{who}]: Write lock to {l:?}");
            fs::write(l, lock_json)?;

            tracing::debug!("Lock[{who}]: Load {id}");
            let item = self.load(id).await.unwrap_or_default();

            drop(sem);
            tracing::debug!("Lock[{who}]: Dropped Semaphore"); // close enough
            (lock, item)
        };
        Ok(LockResult::Success { lock, item })
    }

    async fn unlock(&self, id: &str, lock: StorageLock) -> Result<()> {
        if !self.verify_lock(id, &lock).await? {
            Err(eyre!("Lock invalid!"))
        } else {
            let l = self.lock_path(id);
            std::fs::remove_file(l)?;
            Ok(())
        }
    }

    async fn force_unlock(&self, id: &str) -> Result<()> {
        let l = self.lock_path(id);
        if !fs::metadata(&l).is_ok() {
            tracing::warn!("Lockfile {l:?} doesn't exists");
            return Err(eyre!("Not locked"));
        }

        std::fs::remove_file(l)?;
        Ok(())
    }
    async fn verify_lock(&self, id: &str, lock: &StorageLock) -> Result<bool> {
        let l = self.lock_path(id);
        if !fs::metadata(&l).is_ok() {
            tracing::warn!("Lockfile {l:?} doesn't exists");
            return Ok(false);
        }

        let expected_lock_json = fs::read(&l)?;
        let expected_lock: StorageLock = serde_json::from_slice(&expected_lock_json)?;

        if expected_lock != *lock {
            tracing::warn!("Lock mismatch for {id} {lock:?} != {expected_lock:?}");
            return Ok(false);
        }
        Ok(true)
    }
}
