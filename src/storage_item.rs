use async_trait::async_trait;
use color_eyre::eyre::Result;

#[async_trait]
pub trait StorageItem: core::fmt::Debug + std::marker::Sync {
    fn serialize(&self) -> Result<Vec<u8>>;
    fn deserialize(data: &[u8]) -> Result<Self>
    where
        Self: Sized;
}
