use crate::storage::LockNewResult;
use crate::LockResult;
#[cfg(feature = "metadata")]
use crate::Metadata;
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
    #[cfg(feature = "metadata")]
    metadata: Metadata<ITEM>,
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
            #[cfg(feature = "metadata")]
            metadata: Metadata::default(),
        }
    }

    fn file_path(&self, id: &ITEM::ID) -> PathBuf {
        let mut p = PathBuf::new();
        p.push(&self.base_path);
        let id = format!("{id}");
        let idp = Path::new(&id);
        p.push(idp);
        p.set_extension(&self.extension);

        p
    }
    fn lock_path(&self, id: &ITEM::ID) -> PathBuf {
        let mut p = PathBuf::new();
        p.push(&self.base_path);
        let id = format!("{id}");
        let idp = Path::new(&id);
        p.push(idp);
        p.set_extension("lock");

        p
    }
}

#[cfg(feature = "metadata")]
impl<ITEM: StorageItem> StorageDisk<ITEM> {
    fn update_highest_seen_id(&self, id: &ITEM::ID) {
        self.metadata.update_highest_seen_id(id);
    }
}

#[cfg(not(feature = "metadata"))]
impl<ITEM: StorageItem> StorageDisk<ITEM> {
    fn update_highest_seen_id(&self, _id: &ITEM::ID) {}
}

#[async_trait]
impl<ITEM: StorageItem + std::marker::Send> Storage<ITEM> for StorageDisk<ITEM> {
    async fn ensure_storage_exists(&mut self) -> Result<()> {
        self.ensure_folder_exists().await
    }
    async fn create(&self) -> Result<ITEM::ID> {
        let mut tries = 10;
        loop {
            //let id = nanoid::nanoid!();
            let id = ITEM::generate_next_id(None);
            if !self.exists(&id).await? {
                return Ok(id);
            }

            tries -= 1;
            if tries <= 0 {
                todo!();
            }
        }
    }
    async fn exists(&self, id: &ITEM::ID) -> Result<bool> {
        //let p = self.file_path(id.into());
        //let p = self.file_path(&format!("{id}"));
        let p = self.file_path(id);
        tracing::debug!("{p:?}");

        if fs::metadata(p).is_ok() {
            self.update_highest_seen_id(&id);
            Ok(true)
        } else {
            // the lockfile already exists, but the data file doesn't
            // might happen when somebody crashed during creation
            // or is in the middle of creation
            let p = self.lock_path(id);
            if fs::metadata(p).is_ok() {
                self.update_highest_seen_id(&id);
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    async fn load(&self, id: &ITEM::ID) -> Result<ITEM> {
        let p = self.file_path(id);
        let b = fs::read(p.clone()).map_err(|e| eyre!("Can't load from {p:?} -> {e}"))?;
        let i = ITEM::deserialize(&b)?;
        self.update_highest_seen_id(&id);

        Ok(i)
    }

    async fn save(&self, id: &ITEM::ID, item: &ITEM, lock: &StorageLock) -> Result<()> {
        if !self.verify_lock(id, lock).await? {
            Err(eyre!("Lock invalid!"))
        } else {
            let p = self.file_path(id);
            let b = item.serialize()?;
            fs::write(p.clone(), b).map_err(|e| eyre!("Can't save to {p:?}: {e:?}"))?;
            self.update_highest_seen_id(&id);
            Ok(())
        }
    }
    async fn lock(&self, id: &ITEM::ID, who: &str) -> Result<LockResult<ITEM>> {
        let l = self.lock_path(id);
        let (lock, item) = {
            let sem = self.lock_semaphore.acquire().await?;
            tracing::debug!("Lock[{who}]: Got Semaphore");

            tracing::debug!("Lock[{who}]: Does {l:?} exist");

            if fs::metadata(&l).is_ok() {
                tracing::warn!("lock: Lockfile {l:?} already exists");
                drop(sem);
                tracing::debug!("Lock[{who}]: Dropped Semaphore"); // close enough
                                                                   //return Err(eyre!("Already locked"));
                                                                   // :TODO: load lock
                self.update_highest_seen_id(&id);
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
        self.update_highest_seen_id(&id);
        Ok(LockResult::Success { lock, item })
    }

    async fn lock_new(&self, id: &ITEM::ID, who: &str) -> Result<LockNewResult<ITEM>> {
        let l = self.lock_path(id);
        let (lock, item) = {
            let sem = self.lock_semaphore.acquire().await?;
            tracing::debug!("Lock[{who}]: Got Semaphore");

            if self.exists(id).await? {
                tracing::warn!("lock_new: Item {id:?} already exists");
                drop(sem);
                tracing::debug!("Lock[{who}]: Dropped Semaphore"); // close enough
                return Ok(LockNewResult::AlreadyExists);
            }

            tracing::debug!("Lock[{who}]: Does {l:?} exist");

            if fs::metadata(&l).is_ok() {
                tracing::warn!("lock_new: Lockfile {l:?} already exists");
                drop(sem);
                tracing::debug!("Lock[{who}]: Dropped Semaphore"); // close enough
                                                                   //return Err(eyre!("Already locked"));
                                                                   // :TODO: load lock
                self.update_highest_seen_id(&id);
                return Ok(LockNewResult::AlreadyLocked {
                    who: String::from(":TODO:"),
                });
            }

            let lock = StorageLock::new(who);
            let lock_json = serde_json::to_string_pretty(&lock)?;

            tracing::debug!("Lock[{who}]: Write lock to {l:?}");
            fs::write(l.clone(), lock_json)
                .map_err(|e| eyre!("Can't lock {l:?} for {who}: {e:?}"))?;

            tracing::debug!("Lock[{who}]: Load {id}");
            let item_path = self.file_path(id);
            tracing::debug!("{item_path:?}");

            if fs::metadata(item_path).is_ok() {
                tracing::warn!("lock_new: Item {id:?} already exists -- after creating lock");
                self.unlock(id, lock).await.inspect_err(|e| {
                    tracing::error!("Can't unlock {id}: {e:?}");
                })?;
                drop(sem);
                tracing::debug!("Lock[{who}]: Dropped Semaphore"); // close enough
                return Ok(LockNewResult::AlreadyExists);
            }

            tracing::debug!("Lock[{who}]: Dropped Semaphore"); // close enough
            let item = ITEM::default();
            self.save(id, &item, &lock).await.inspect_err(|e| {
                tracing::error!("Failed saving new item {id}: {e:?}");
            })?;
            // :TODO: could probably be done earlier
            drop(sem);
            (lock, item)
        };
        self.update_highest_seen_id(&id);
        Ok(LockNewResult::Success { lock, item })
    }

    async fn unlock(&self, id: &ITEM::ID, lock: StorageLock) -> Result<()> {
        if !self.verify_lock(id, &lock).await? {
            Err(eyre!("Lock invalid!"))
        } else {
            let l = self.lock_path(id);
            std::fs::remove_file(l.clone()).map_err(|e| eyre!("Can't unlock {l:?}: {e:?}"))?;
            Ok(())
        }
    }

    async fn force_unlock(&self, id: &ITEM::ID) -> Result<()> {
        let l = self.lock_path(id);
        if !fs::metadata(&l).is_ok() {
            tracing::warn!("Lockfile {l:?} doesn't exists");
            return Err(eyre!("Not locked"));
        }

        std::fs::remove_file(l.clone()).map_err(|e| eyre!("Can't force unlock {l:?}: {e:?}"))?;
        Ok(())
    }
    async fn verify_lock(&self, id: &ITEM::ID, lock: &StorageLock) -> Result<bool> {
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
    async fn all_ids(&self) -> Result<Vec<ITEM::ID>> {
        //tracing::debug!("all_ids");
        let mut ids = Vec::default();
        let extension = self.extension.to_string_lossy(); //.to_string();
        let extension = format!(".{}", extension);
        let mut highest_id = ITEM::ID::default();
        for entry in fs::read_dir(&self.base_path)? {
            if let Ok(entry) = &entry {
                match entry.file_type() {
                    Ok(file_type) if file_type.is_file() => {
                        //tracing::debug!("{entry:?}");
                        //let p = entry.path();
                        let f = entry.file_name();
                        let f = f.to_string_lossy().to_string();
                        if let Some(id) = f.strip_suffix(&extension) {
                            //tracing::debug!("{f} -> {id:?}");
                            //let id: ITEM::ID = id.try_into().map_err(|e| eyre!("Can not convert {id} into ITEM::ID -> {e:?}") )?;
                            let id: ITEM::ID = ITEM::make_id(id)?;
                            if id > highest_id {
                                highest_id = id.to_owned(); // :TODO: decide if we want to keep this
                            } else {
                                tracing::debug!("{id} < {highest_id}");
                            }
                            ids.push(id);
                        }
                    }
                    _ => {} // skip
                }
            }
        }
        self.update_highest_seen_id(&highest_id);
        Ok(ids)
    }
    async fn scan_ids(
        &self,
        start: Option<&str>,
        limit: Option<usize>,
    ) -> Result<(Vec<ITEM::ID>, Option<String>)> {
        // :HACK: just scan all and filter after
        let mut all_ids = self.all_ids().await?;

        let skip_count = if let Some(start) = start {
            let skip_count = start.parse::<usize>()?;
            let skip_count = skip_count.min(all_ids.len());
            all_ids.drain(0..skip_count);
            skip_count
        } else {
            0
        };

        if let Some(limit) = limit {
            let limit = limit.min(all_ids.len());
            all_ids.resize_with(limit, || {
                /* :TODO: trace? */
                unimplemented!() /* ITEM::ID::default() */
            });
        }

        let scan_pos = skip_count + all_ids.len();

        let scan_pos = if scan_pos <= all_ids.len() {
            Some(format!("{scan_pos}"))
        } else {
            None
        };

        Ok((all_ids, scan_pos))
    }

    async fn display_lock(&self, id: &ITEM::ID) -> Result<String> {
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
    #[cfg(feature = "metadata")]
    async fn metadata_highest_seen_id(&self) -> Option<ITEM::ID> {
        self.metadata.highest_seen_id()
    }

    #[cfg(feature = "wipe")]
    async fn wipe(&self, confirmation: &str) -> Result<()> {
        if confirmation != "Yes, I know what I am doing!" {
            tracing::error!("Please confirm you know what you are doing");
            return Err(eyre!("Unconfirmed wipe attempt"));
        }

        let _sem = self.lock_semaphore.acquire().await?;

        // we know all_ids doesn't use the semaphore
        let ids = self.all_ids().await?;

        tracing::warn!("Wiping {} items.", ids.len());
        for id in ids {
            let l = self.lock_path(&id);
            if fs::metadata(&l).is_ok() {
                let _ =
                    std::fs::remove_file(l.clone()).map_err(|e| eyre!("Can't remove {l:?}: {e:?}"));
            }
            let f = self.file_path(&id);
            if fs::metadata(&f).is_ok() {
                let _ =
                    std::fs::remove_file(f.clone()).map_err(|e| eyre!("Can't remove {f:?}: {e:?}"));
            }
        }
        Ok(())
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

        assert!(all_ids.len() >= ids.len());
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
