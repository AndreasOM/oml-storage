use async_trait::async_trait;
use color_eyre::eyre::Result;

#[async_trait]
pub trait Storage<ITEM> {
    async fn create(&self) -> Result<String>;
    async fn exists(&self, id: &str) -> Result<bool>;
    async fn load(&self, id: &str) -> Result<ITEM>;
    async fn save(&self, id: &str, item: &ITEM) -> Result<()>;
}
