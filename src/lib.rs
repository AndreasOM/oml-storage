//#![warn(missing_docs)]

//! A very simple wrapper to handle locked storage of items.
//!
//! Provides an abstraction over storage backends.
//! The core idea is that storage items will be locked in storage and stay hot in memory for a while.
//!
//! Note:
//! The documentation is still work-in-progress.

mod storage;
pub use storage::LockNewResult;
pub use storage::LockResult;
pub use storage::Storage;
pub use storage::StorageLock;

mod storage_item;
pub use storage_item::StorageItem;

mod storage_id;

// New storage ID types
pub use storage_id::StorageId;

// ID implementations
pub use storage_id::ExternalId;
pub use storage_id::RandomId;
pub use storage_id::SequentialId;

mod storage_disk;
pub use storage_disk::StorageDisk;
mod storage_dynamodb;
pub use storage_dynamodb::StorageDynamoDb;
mod storage_null;
pub use storage_null::StorageNull;

#[cfg(feature = "metadata")]
mod metadata;
#[cfg(feature = "metadata")]
pub(crate) use metadata::Metadata;

#[cfg(test)]
mod storage_id_test;
