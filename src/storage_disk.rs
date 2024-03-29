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
    pub async fn ensure_folder_exists(&mut self) -> Result<()> {
        std::fs::create_dir_all(&self.base_path)
            .map_err(|e| eyre!("Could not create folder {:?} -> {e}", &self.base_path))?;

        Ok(())
    }
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
    async fn ensure_storage_exists(&mut self) -> Result<()> {
        self.ensure_folder_exists().await
    }
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

        if fs::metadata(p).is_ok() {
            Ok(true)
        } else {
            // the lockfile already exists, but the data file doesn't
            // might happen when somebody crashed during creation
            // or is in the middle of creation
            let p = self.lock_path(id);
            Ok(fs::metadata(p).is_ok())
        }
    }

    async fn load(&self, id: &str) -> Result<ITEM> {
        let p = self.file_path(id);
        let b = fs::read(p.clone()).map_err(|e| eyre!("Can't load from {p:?} -> {e}"))?;
        let i = ITEM::deserialize(&b)?;

        Ok(i)
    }

    async fn save(&self, id: &str, item: &ITEM, lock: &StorageLock) -> Result<()> {
        if !self.verify_lock(id, lock).await? {
            Err(eyre!("Lock invalid!"))
        } else {
            let p = self.file_path(id);
            let b = item.serialize()?;
            fs::write(p.clone(), b).map_err(|e| eyre!("Can't save to {p:?}: {e:?}"))?;
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
            fs::write(l.clone(), lock_json)
                .map_err(|e| eyre!("Can't lock {l:?} for {who}: {e:?}"))?;

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
            std::fs::remove_file(l.clone()).map_err(|e| eyre!("Can't unlock {l:?}: {e:?}"))?;
            Ok(())
        }
    }

    async fn force_unlock(&self, id: &str) -> Result<()> {
        let l = self.lock_path(id);
        if !fs::metadata(&l).is_ok() {
            tracing::warn!("Lockfile {l:?} doesn't exists");
            return Err(eyre!("Not locked"));
        }

        std::fs::remove_file(l.clone()).map_err(|e| eyre!("Can't force unlock {l:?}: {e:?}"))?;
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
    async fn all_ids(&self) -> Result<Vec<String>> {
        let mut ids = Vec::default();
        let extension = self.extension.to_string_lossy(); //.to_string();
        let extension = format!(".{}", extension);
        for entry in fs::read_dir(&self.base_path)? {
            if let Ok(entry) = &entry {
                match entry.file_type() {
                    Ok(file_type) if file_type.is_file() => {
                        //println!("{entry:?}");
                        //let p = entry.path();
                        let f = entry.file_name();
                        let f = f.to_string_lossy().to_string();
                        if let Some(id) = f.strip_suffix(&extension) {
                            //println!("{f} -> {id:?}");
                            ids.push(String::from(id));
                        }
                    }
                    _ => {} // skip
                }
            }
        }
        Ok(ids)
    }

    async fn display_lock(&self, id: &str) -> Result<String> {
        let l = self.lock_path(id);
        if !fs::metadata(&l).is_ok() {
            return Ok(String::default());
        } else {
            let lock_json = fs::read(&l)?;
            let lock: StorageLock = serde_json::from_slice(&lock_json)?;
            let lock_string = format!("Locked by {} at {:?}", lock.who(), lock.when());
            //            let lock_string = format!("{:?}", lock);

            Ok(lock_string)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::LockResult;
    use crate::Storage;
    use crate::StorageDisk;
    use crate::StorageItem;
    use color_eyre::Result;
    use serde::Deserialize;
    use serde::Serialize;
    use std::env;
    use std::path::Path;

    #[derive(Default, Debug, Serialize, Deserialize)]
    struct TestItem {}

    impl StorageItem for TestItem {
        fn serialize(&self) -> Result<Vec<u8>> {
            let json = serde_json::to_string_pretty(&self)?;

            Ok(json.into())
        }
        fn deserialize(data: &[u8]) -> Result<Self>
        where
            Self: Sized,
        {
            let i = serde_json::from_slice(&data)?;

            Ok(i)
        }
    }

    #[tokio::test]
    async fn it_debugs() -> Result<()> {
        let mut path = env::current_dir()?;
        path.push("data");
        path.push("test_items");
        let extension = Path::new("test_item");

        let storage = StorageDisk::<TestItem>::new(&path, &extension).await;
        println!("{storage:?}");

        let storage: Box<dyn Storage<TestItem>> = Box::new(storage);
        println!("{storage:?}");

        Ok(())
    }
    #[tokio::test]
    async fn it_gives_all_ids() -> Result<()> {
        let mut path = env::current_dir()?;
        path.push("data");
        path.push("test_items");
        let extension = Path::new("test_item.json");

        let storage = StorageDisk::<TestItem>::new(&path, &extension).await;
        //println!("{storage:?}");

        let storage: Box<dyn Storage<TestItem>> = Box::new(storage);
        //println!("{storage:?}");

        let us = "TEST";

        let mut ids = Vec::default();

        for _ in 0..5 {
            let item_id = storage.create().await.unwrap();
            //println!("{item_id:?}");

            let (lock, item) = match storage.lock(&item_id, &us).await? {
                LockResult::Success { lock, item } => (lock, item),
                LockResult::AlreadyLocked { .. } => {
                    todo!();
                }
            };
            storage.save(&item_id, &item, &lock).await?;
            storage.unlock(&item_id, lock).await?;

            ids.push(item_id);
        }
        let all_ids = storage.all_ids().await?;

        //println!("{all_ids:#?}");

        assert!(all_ids.len() > ids.len());
        assert!(ids.iter().all(|id| all_ids.contains(id)));

        Ok(())
    }

    #[tokio::test]
    async fn it_displays_locks() -> Result<()> {
        let mut path = env::current_dir()?;
        path.push("data");
        path.push("test_items");
        let extension = Path::new("test_item");

        let storage = StorageDisk::<TestItem>::new(&path, &extension).await;
        // println!("{storage:?}");

        let storage: Box<dyn Storage<TestItem>> = Box::new(storage);
        // println!("{storage:?}");

        let us = "TEST";

        let item_id = storage.create().await.unwrap();
        //println!("{item_id:?}");

        let (lock, item) = match storage.lock(&item_id, &us).await? {
            LockResult::Success { lock, item } => (lock, item),
            LockResult::AlreadyLocked { .. } => {
                todo!();
            }
        };
        storage.save(&item_id, &item, &lock).await?;
        let l = storage.display_lock(&item_id).await?;
        println!("{l:?}");
        storage.unlock(&item_id, lock).await?;
        let l = storage.display_lock(&item_id).await?;
        println!("{l:?}");

        Ok(())
    }

    #[tokio::test]
    async fn exists_works_during_creation() -> Result<()> {
        let mut path = env::current_dir()?;
        path.push("data");
        path.push("test_items");
        let extension = Path::new("test_item");

        let storage = StorageDisk::<TestItem>::new(&path, &extension).await;
        // println!("{storage:?}");

        let storage: Box<dyn Storage<TestItem>> = Box::new(storage);
        // println!("{storage:?}");

        let us = "TEST";

        let item_id = nanoid::nanoid!();

        let (lock, item) = match storage.lock(&item_id, &us).await? {
            LockResult::Success { lock, item } => (lock, item),
            LockResult::AlreadyLocked { .. } => {
                todo!();
            }
        };
        let exists_during_creation = storage.exists(&item_id).await?;

        // storage.save(&item_id, &item, &lock).await?;
        let l = storage.display_lock(&item_id).await?;
        // println!("{l:?}");
        storage.unlock(&item_id, lock).await?;
        // let l = storage.display_lock(&item_id).await?;
        // println!("{l:?}");

        assert_eq!(true, exists_during_creation);
        Ok(())
    }

    //ensure_storage_exists
}
