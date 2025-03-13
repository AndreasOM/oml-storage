use crate::StorageItem;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;
use color_eyre::eyre::eyre;
use color_eyre::eyre::Result;
use serde::Deserialize;
use serde::Serialize;

/// Storage is the core trait for interacting with stored items.
///
/// This trait provides a comprehensive API for creating, reading, updating, and deleting items,
/// with built-in locking mechanisms to handle concurrent access.
///
/// # Usage Pattern
///
/// The typical workflow for using Storage is:
///
/// 1. **Creating/Accessing Items**:
///    - Call `create()` to generate a new ID
///    - Or use an existing ID
///
/// 2. **Locking Items**:
///    - Call `lock()` or `lock_new()` to get exclusive access
///    - These return a StorageLock and the item (if it exists)
///
/// 3. **Modifying Items**:
///    - Update the item as needed
///
/// 4. **Saving Items**:
///    - Call `save()` with the ID, modified item, and lock
///
/// 5. **Releasing Locks**:
///    - Call `unlock()` to release the lock when done
///
/// # Example
///
/// ```rust,no_run
/// # use color_eyre::eyre::Result;
/// # use oml_storage::{Storage, StorageItem, StorageLock};
/// # async fn example<S, I>(storage: &S, id: &I::ID) -> Result<()>
/// # where
/// #     S: Storage<I>,
/// #     I: StorageItem,
/// # {
/// // Lock an item for exclusive access
/// let lock_result = storage.lock(id, "user-123").await?;
/// let (lock, mut item) = lock_result.success()?;
///
/// // Modify the item
/// // item.update_something();
///
/// // Save the modified item
/// storage.save(id, &item, &lock).await?;
///
/// // Release the lock when done
/// storage.unlock(id, lock).await?;
/// # Ok(())
/// # }
/// ```
///
/// Note: Due to async_trait, you might see lifetime parameters like `'life0, 'life1, 'async_trait`
/// in the documentation. These can be ignored - all methods are simply `async` and return
/// a [color_eyre::eyre::Result].
#[async_trait]
pub trait Storage<ITEM: StorageItem + Sized>: Send + Sync + std::fmt::Debug {
    /// Initializes the storage backend, creating any necessary resources.
    ///
    /// This method should be called before any other operations to ensure
    /// the storage system is properly set up (e.g., creating directories
    /// or database tables).
    async fn ensure_storage_exists(&mut self) -> Result<()>;

    /// Creates a new item with a randomly generated ID.
    ///
    /// # Returns
    /// * `Result<ITEM::ID>` - The newly created ID on success
    ///
    /// # Notes
    /// * This only creates an ID, not an actual item
    /// * To create an item with a specific ID, use [`lock_new`](#method.lock_new) instead
    /// * ID generation strategy depends on the [`StorageItem::generate_next_id`] implementation
    async fn create(&self) -> Result<ITEM::ID>;

    /// Checks if an item with the given ID exists in storage.
    ///
    /// # Parameters
    /// * `id` - The ID to check
    ///
    /// # Returns
    /// * `Result<bool>` - `true` if the item exists, `false` otherwise
    async fn exists(&self, id: &ITEM::ID) -> Result<bool>;

    /// Loads an item from storage without obtaining a lock.
    ///
    /// # Parameters
    /// * `id` - The ID of the item to load
    ///
    /// # Returns
    /// * `Result<ITEM>` - The loaded item on success
    ///
    /// # Notes
    /// * This operation does not acquire a lock, so the item could be modified
    ///   by other processes while you're working with it
    /// * For exclusive access, use [`lock`](#method.lock) instead
    async fn load(&self, id: &ITEM::ID) -> Result<ITEM>;

    /// Saves an item to storage with lock verification.
    ///
    /// # Parameters
    /// * `id` - The ID of the item to save
    /// * `item` - The item to save
    /// * `lock` - The lock that grants permission to save this item
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Notes
    /// * You must provide a valid lock obtained via [`lock`](#method.lock) or [`lock_new`](#method.lock_new)
    /// * The save will fail if the lock is invalid or expired
    async fn save(&self, id: &ITEM::ID, item: &ITEM, lock: &StorageLock) -> Result<()>;

    /// Acquires an exclusive lock on an item for modification.
    ///
    /// # Parameters
    /// * `id` - The ID of the item to lock
    /// * `who` - An identifier for the lock owner (e.g., username or process ID)
    ///
    /// # Returns
    /// * `Result<LockResult<ITEM>>` - A result enum that can be:
    ///   * `LockResult::Success` - Contains the lock and the item
    ///   * `LockResult::AlreadyLocked` - Item is already locked by another owner
    ///
    /// # Notes
    /// * If the item doesn't exist, a new default item is created
    /// * Use the [`success`](#method.success) method on the result to get the lock and item
    /// * The lock must be released with [`unlock`](#method.unlock) when done
    /// * You must explicitly call [`save`](#method.save) before unlocking to persist any changes
    async fn lock(&self, id: &ITEM::ID, who: &str) -> Result<LockResult<ITEM>>;

    /// Locks a new item, failing if it already exists.
    ///
    /// # Parameters
    /// * `id` - The ID for the new item
    /// * `who` - An identifier for the lock owner (e.g., username or process ID)
    ///
    /// # Returns
    /// * `Result<LockNewResult<ITEM>>` - A result enum that can be:
    ///   * `LockNewResult::Success` - Contains the lock and the new default item
    ///   * `LockNewResult::AlreadyLocked` - Item is already locked by another owner
    ///   * `LockNewResult::AlreadyExists` - An item with this ID already exists
    ///
    /// # Notes
    /// * Use the [`success`](#method.success) method on the result to get the lock and item
    /// * The lock must be released with [`unlock`](#method.unlock) when done
    /// * You must explicitly call [`save`](#method.save) before unlocking to persist the new item
    async fn lock_new(&self, id: &ITEM::ID, who: &str) -> Result<LockNewResult<ITEM>>;

    /// Releases a previously acquired lock.
    ///
    /// # Parameters
    /// * `id` - The ID of the item to unlock
    /// * `lock` - The lock object to release (returned from `lock` or `lock_new`)
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Notes
    /// * The provided lock must match the current lock on the item
    /// * After unlocking, other processes can acquire a lock on this item
    /// * IMPORTANT: Unlock does NOT automatically save changes to the item
    /// * You must explicitly call [`save`](#method.save) before unlocking to persist changes
    async fn unlock(&self, id: &ITEM::ID, lock: StorageLock) -> Result<()>;

    /// Forces the release of a lock regardless of ownership.
    ///
    /// # Parameters
    /// * `id` - The ID of the item to forcibly unlock
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Warning
    /// * This should only be used in emergency situations, such as when
    ///   a process crashes without releasing its locks
    /// * Using this risks data corruption if the original lock owner is still active
    async fn force_unlock(&self, id: &ITEM::ID) -> Result<()>;

    /// Verifies if a lock is still valid for an item.
    ///
    /// # Parameters
    /// * `id` - The ID of the item to check
    /// * `lock` - The lock to verify
    ///
    /// # Returns
    /// * `Result<bool>` - `true` if the lock is valid, `false` otherwise
    async fn verify_lock(&self, id: &ITEM::ID, lock: &StorageLock) -> Result<bool>;

    /// Returns all item IDs in the storage.
    ///
    /// # Returns
    /// * `Result<Vec<ITEM::ID>>` - All item IDs on success
    ///
    /// # Notes
    /// * This is an experimental API that may change in future versions
    /// * For large datasets, consider using [`scan_ids`](#method.scan_ids) instead
    /// * This method may be inefficient for storages with many items
    async fn all_ids(&self) -> Result<Vec<ITEM::ID>>;

    /// Scans for item IDs with pagination support.
    ///
    /// # Parameters
    /// * `start` - Optional start position for the scan (continuation token)
    /// * `limit` - Optional maximum number of IDs to return
    ///
    /// # Returns
    /// * `Result<(Vec<ITEM::ID>, Option<String>)>` - A tuple containing:
    ///   * The list of IDs found in this scan
    ///   * An optional continuation token for the next page (None if end reached)
    ///
    /// # Notes
    /// * This is an experimental API that may change in future versions
    async fn scan_ids(
        &self,
        _start: Option<&str>,
        _limit: Option<usize>,
    ) -> Result<(Vec<ITEM::ID>, Option<String>)> {
        todo!("Implement scan position for ...");
    }

    /// Returns a human-readable description of the current lock status.
    ///
    /// # Parameters
    /// * `id` - The ID of the item to check
    ///
    /// # Returns
    /// * `Result<String>` - A description of the lock status on success
    ///
    /// # Notes
    /// * This is primarily intended for debugging purposes
    async fn display_lock(&self, id: &ITEM::ID) -> Result<String>;

    /// Returns the highest ID that has been seen by this storage.
    ///
    /// # Returns
    /// * `Option<ITEM::ID>` - The highest ID if available, or None if no items exist
    ///
    /// # Notes
    /// * This method is only available when the "metadata" feature is enabled
    /// * Useful for sequential ID generation strategies
    #[cfg(feature = "metadata")]
    async fn metadata_highest_seen_id(&self) -> Option<ITEM::ID>;

    /// Wipes all items from the storage.
    ///
    /// # Parameters
    /// * `confirmation` - A confirmation string to prevent accidental wipes
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Warning
    /// * This is a destructive operation that permanently deletes all data
    /// * Only available when the "wipe" feature is enabled
    /// * Use with extreme caution
    #[cfg(feature = "wipe")]
    async fn wipe(&self, confirmation: &str) -> Result<()>;
}

/// Represents an exclusive lock on a storage item.
///
/// A StorageLock provides exclusive access to an item for modification. It records:
/// - Who acquired the lock (typically a user ID or process identifier)
/// - When the lock was acquired
///
/// Locks are used to prevent concurrent modifications to the same item.
/// You must acquire a lock before saving changes to an item, and release
/// the lock when done to allow others to modify the item.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct StorageLock {
    /// Identifier of who acquired the lock
    who: String,
    /// Timestamp when the lock was acquired
    when: DateTime<Utc>,
}

impl StorageLock {
    /// Creates a new lock for the specified owner.
    ///
    /// # Parameters
    /// * `who` - An identifier for the lock owner (e.g., username or process ID)
    ///
    /// # Returns
    /// * A new StorageLock instance with the current timestamp
    pub fn new(who: &str) -> Self {
        Self {
            who: who.to_string(),
            when: Utc::now(),
        }
    }

    /// Returns the identifier of who owns this lock.
    ///
    /// # Returns
    /// * A string reference to the lock owner's identifier
    pub fn who(&self) -> &str {
        &self.who
    }

    /// Returns when this lock was acquired.
    ///
    /// # Returns
    /// * A reference to the timestamp when the lock was created
    pub fn when(&self) -> &DateTime<Utc> {
        &self.when
    }
}

/// Result type for lock operations on existing or new items.
///
/// This enum represents the two possible outcomes when trying to lock an item:
/// - Successfully acquired the lock and loaded/created the item
/// - Failed to acquire the lock because the item is already locked
#[derive(Debug)]
pub enum LockResult<ITEM> {
    /// Lock was successfully acquired
    Success {
        /// The lock that grants exclusive access
        lock: StorageLock,
        /// The item that was loaded or created
        item: ITEM,
    },
    /// Item is already locked by someone else
    AlreadyLocked {
        /// Identifier of who currently holds the lock
        who: String,
    },
}

impl<ITEM> LockResult<ITEM> {
    /// Converts the result into a simple (lock, item) tuple or an error.
    ///
    /// # Returns
    /// * `Result<(StorageLock, ITEM)>` - A tuple with the lock and item on success
    ///
    /// # Errors
    /// * Returns an error if the item was already locked
    ///
    /// # Example
    /// ```rust,no_run
    /// # async fn example<S, I>(storage: &S, id: &I::ID) -> color_eyre::eyre::Result<()>
    /// # where
    /// #     S: oml_storage::Storage<I>,
    /// #     I: oml_storage::StorageItem,
    /// # {
    /// let lock_result = storage.lock(id, "user-123").await?;
    /// let (lock, item) = lock_result.success()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn success(self) -> Result<(StorageLock, ITEM)> {
        match self {
            LockResult::Success { lock, item } => Ok((lock, item)),
            LockResult::AlreadyLocked { who } => Err(eyre!("Already locked by {who:?}")),
        }
    }
}

/// Result type for lock operations specifically on new items.
///
/// This enum represents the three possible outcomes when trying to lock a new item:
/// - Successfully acquired the lock and created the item
/// - Failed because the item is already locked by someone else
/// - Failed because an item with this ID already exists (but is not locked)
#[derive(Debug)]
pub enum LockNewResult<ITEM> {
    /// Lock was successfully acquired on a new item
    Success {
        /// The lock that grants exclusive access
        lock: StorageLock,
        /// The newly created item
        item: ITEM,
    },
    /// Item is already locked by someone else
    AlreadyLocked {
        /// Identifier of who currently holds the lock
        who: String,
    },
    /// Item with this ID already exists but is not locked
    AlreadyExists,
}

impl<ITEM> LockNewResult<ITEM> {
    /// Converts the result into a simple (lock, item) tuple or an error.
    ///
    /// # Returns
    /// * `Result<(StorageLock, ITEM)>` - A tuple with the lock and item on success
    ///
    /// # Errors
    /// * Returns an error if the item was already locked or already exists
    ///
    /// # Example
    /// ```rust,no_run
    /// # async fn example<S, I>(storage: &S, id: &I::ID) -> color_eyre::eyre::Result<()>
    /// # where
    /// #     S: oml_storage::Storage<I>,
    /// #     I: oml_storage::StorageItem,
    /// # {
    /// let lock_result = storage.lock_new(id, "user-123").await?;
    /// let (lock, item) = lock_result.success()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn success(self) -> Result<(StorageLock, ITEM)> {
        match self {
            LockNewResult::Success { lock, item } => Ok((lock, item)),
            LockNewResult::AlreadyLocked { who } => Err(eyre!("Already locked by {who:?}")),
            LockNewResult::AlreadyExists => Err(eyre!("Already exists")),
        }
    }
}
