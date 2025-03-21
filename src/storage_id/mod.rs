use color_eyre::eyre::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::{Debug, Display};
use std::hash::Hash;
/// Trait for storage item identifiers
pub trait StorageId:
    ToString
    + Sync
    + Send
    + Debug
    + Display
    + PartialOrd
    + Clone
    + Default
    + Serialize
    + DeserializeOwned
    + Hash
    + std::cmp::Eq
{
    /// Create an ID from its string representation
    fn from_string(s: &str) -> Result<Self>
    where
        Self: Sized;

    /// Generate a new unique ID
    fn generate_new(previous: Option<&Self>) -> Self
    where
        Self: Sized;

    /// Validate if a given string is a valid ID format
    fn is_valid_format(s: &str) -> bool
    where
        Self: Sized;
}

mod external_id;
mod random_id;
mod sequential_id;
mod simple_external_id;

pub use external_id::ExternalId;
pub use random_id::RandomId;
pub use sequential_id::SequentialId;
pub use simple_external_id::SimpleExternalId;
