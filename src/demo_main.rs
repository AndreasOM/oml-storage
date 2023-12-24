use std::env;
use std::path::Path;

use color_eyre::eyre::Result;
use oml_storage::Storage;
use oml_storage::StorageDisk;
use oml_storage::StorageItem;

use serde::Deserialize;
use serde::Serialize;

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

    let id = String::from("1");

    if storage.exists(&id).await? {
        tracing::debug!("Item {id} exists");
        let mut item = storage.load(&id).await?;
        tracing::debug!("{item:?}");
        item.increment_counter();
        storage.save(&id, &item).await?;
    } else {
        tracing::debug!("Item {id} doesn't exists");
        let item = TestItem::default();
        storage.save(&id, &item).await?;
    }

    let id = storage.create().await?;

    if storage.exists(&id).await? {
        tracing::debug!("Item {id} exists");
        let mut item = storage.load(&id).await?;
        tracing::debug!("{item:?}");
        item.increment_counter();
        storage.save(&id, &item).await?;
    } else {
        tracing::debug!("Item {id} doesn't exists");
        let item = TestItem::default();
        storage.save(&id, &item).await?;
    }

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
}

impl TestItem {
    fn increment_counter(&mut self) {
        self.counter += 1;
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
