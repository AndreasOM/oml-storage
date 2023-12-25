use std::env;
use std::path::Path;
use std::sync::Arc;

use color_eyre::eyre::Result;
use oml_storage::Storage;
use oml_storage::StorageDisk;
use oml_storage::StorageItem;

use serde::Deserialize;
use serde::Serialize;

async fn test(storage: Arc<Box<dyn Storage<TestItem>>>, id: String) -> Result<bool> {
    if storage.exists(&id).await? {
        tracing::debug!("Item {} exists", id);
        let mut item = storage.load(&id).await?;
        tracing::debug!("Load {item:?}");
        item.increment_counter();
        let data = nanoid::nanoid!();
        item.set_data(&data);
        tracing::debug!("Save {item:?}");
        storage.save(&id, &item).await?;

        // wait
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        let item2 = storage.load(&id).await?;
        tracing::debug!("Load {item2:?}");
        if item2.data() == data {
            return Ok(true);
        } else {
            return Ok(false);
        }
    } else {
        tracing::debug!("Item {} doesn't exists", id);
        let item = TestItem::default();
        storage.save(&id, &item).await?;
    }
    Ok(true)
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing();

    color_eyre::install()?;

    tracing::info!("Demo started");

    // ---

    let extension = Path::new("test_item");

    let mut path = env::current_dir()?;
    path.push("data");
    path.push("test_items");
    tracing::debug!("Path {path:?} .{extension:?}");

    let storage = StorageDisk::<TestItem>::new(&path, &extension).await;
    let storage: Box<dyn Storage<TestItem>> = Box::new(storage);
    let storage = Arc::new(storage);

    let id = storage.create().await?;
    test(storage.clone(), id.clone()).await?;

    let id = String::from("1");
    let mut failed = 0;
    let mut succeeded = 0;
    let mut tasks = Vec::new();

    for _i in 0..10 {
        let s = storage.clone();
        let i = id.clone();
        let task = tokio::spawn({ test(s, i) });
        tasks.push(task);
    }

    for task in tasks {
        let f = task.await?;
        if f? {
            succeeded += 1;
        } else {
            failed += 1;
        }
    }

    tracing::info!("Failed {failed} | {succeeded} Succeeded");

    // ---

    tracing::info!("Demo started");
    Ok(())
}

fn setup_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(false);

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct TestItem {
    counter: u32,
    #[serde(default)]
    data: String,
}

impl TestItem {
    fn increment_counter(&mut self) {
        self.counter += 1;
    }

    fn set_data(&mut self, data: &str) {
        self.data = data.to_string();
    }

    fn data(&self) -> &str {
        &self.data
    }
}

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
