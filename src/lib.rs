//#![warn(missing_docs)]

//! A very simple wrapper to handle locked storage of items.
//!
//! Provides an abstraction over storage backends.
//! The core idea is that storage items will be locked in storage and stay hot in memory for a while.
//!
//! Note:
//! The documentation is still work-in-progress.

mod storage;
pub use storage::LockResult;
pub use storage::Storage;
pub use storage::StorageLock;

mod storage_item;
pub use storage_item::StorageItem;

mod storage_disk;
pub use storage_disk::StorageDisk;
mod storage_dynamodb;
pub use storage_dynamodb::StorageDynamoDb;
mod storage_null;
pub use storage_null::StorageNull;

#[cfg(feature = "metadata")]
mod metadata;
pub(crate) use metadata::Metadata;
