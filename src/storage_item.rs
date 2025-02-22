use async_trait::async_trait;
use color_eyre::eyre::Result;

/// The `trait` your items need to implement to be storable
///
/// If your item is serialisable and deserialisable via serde you can use something like:
/// ```
/// use color_eyre::eyre::Result;
/// use serde::Serialize;
/// use serde::Deserialize;
/// use oml_storage::StorageItem;
///
/// #[derive(Debug,Default,Serialize,Deserialize)]
/// pub struct TestItem {}
/// impl StorageItem for TestItem {
///     type ID = String;
///     fn serialize(&self) -> Result<Vec<u8>> {
///         let json = serde_json::to_string_pretty(&self)?;
///     
///         Ok(json.into())
///     }
///     fn deserialize(data: &[u8]) -> Result<Self>
///     where
///         Self: Sized,
///     {
///         let i = serde_json::from_slice(&data)?;
///     
///         Ok(i)
///     }
///     fn generate_next_id(a_previous_id: Option<&Self::ID>) -> Self::ID {
///         nanoid::nanoid!()
///     }
///     fn make_id(id: &str) -> Result<Self::ID> {
///         let id = id.parse::<Self::ID>()?;
///         Ok(id)
///     }
/// }
/// ```
///

#[async_trait]
pub trait StorageItem: core::fmt::Debug + std::default::Default + std::marker::Sync {
    type ID: ToString
        // + for<'a> From<&'a str> -> use make_id
        + Sync
        + Send
        + core::fmt::Debug
        + std::fmt::Display
        + PartialOrd
        + Clone
        + Default;
    fn serialize(&self) -> Result<Vec<u8>>;
    fn deserialize(data: &[u8]) -> Result<Self>
    where
        Self: Sized;

    /// Experimental. Might be gone soon, or not.
    fn generate_next_id(a_previous_id: Option<&Self::ID>) -> Self::ID;

    fn make_id(id: &str) -> Result<Self::ID>;
}
/*
pub trait StorageItemId {

}
*/
