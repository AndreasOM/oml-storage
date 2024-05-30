use crate::LockResult;
use crate::Storage;
use crate::StorageItem;
use crate::StorageLock;
use async_trait::async_trait;
use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::describe_table::DescribeTableError::ResourceNotFoundException;
use aws_sdk_dynamodb::operation::update_item::UpdateItemOutput;
use aws_sdk_dynamodb::types::AttributeDefinition;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::types::KeySchemaElement;
use aws_sdk_dynamodb::types::KeyType;
use aws_sdk_dynamodb::types::ProvisionedThroughput;
use aws_sdk_dynamodb::types::ReturnValue;
use aws_sdk_dynamodb::types::ScalarAttributeType;
use color_eyre::eyre::eyre;
use color_eyre::eyre::Result;

use core::marker::PhantomData;

#[derive(Debug)]
pub struct StorageDynamoDb<ITEM: StorageItem> {
    table_name: String,
    endpoint_url: Option<String>,
    item_type: PhantomData<ITEM>,
}

impl<ITEM: StorageItem> StorageDynamoDb<ITEM> {
    pub async fn new(table_name: &str) -> Self {
        Self {
            table_name: String::from(table_name),
            endpoint_url: None,
            item_type: PhantomData,
        }
    }

    pub fn set_endpoint_url(&mut self, url: &str) -> Result<()> {
        self.endpoint_url = Some(String::from(url));

        Ok(())
    }
    async fn client(&self) -> Result<aws_sdk_dynamodb::Client> {
        // let config = aws_config::load_from_env().await;
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest());
        let config = if let Some(endpoint_url) = &self.endpoint_url {
            config.endpoint_url(endpoint_url)
        } else {
            config
        };
        let config = config.load().await;
        let client = aws_sdk_dynamodb::Client::new(&config);

        Ok(client)
    }
    pub async fn ensure_table_exists(&mut self) -> Result<()> {
        /*
        // let config = aws_config::load_from_env().await;
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest());
        let config = if let Some(endpoint_url) = &self.endpoint_url {
            config.endpoint_url(endpoint_url)
        } else {
            config
        };
        let config = config.load().await;
        let client = aws_sdk_dynamodb::Client::new(&config);
        */
        let client = self.client().await?;

        match client
            .describe_table()
            .table_name(&self.table_name)
            .send()
            .await
        {
            Ok(_o) => {
                // :TODO: verify table format?
                // tracing::info!("Table {} exists -> {o:#?}", &self.table_name);
                tracing::info!("Table {} exists", &self.table_name);
            }
            Err(e) => {
                // tracing::debug!("Err {e:?}");
                match e {
                    SdkError::ServiceError(se) => {
                        match se.err() {
                            ResourceNotFoundException(_nf) => {
                                // tracing::debug!("{nf:?}");
                                tracing::info!("Table {} not found. Creating...", &self.table_name);

                                // :TODO:

                                let ad_id = AttributeDefinition::builder()
                                    .attribute_name("id")
                                    .attribute_type(ScalarAttributeType::S)
                                    .build()?;

                                let key_id = KeySchemaElement::builder()
                                    .attribute_name("id")
                                    .key_type(KeyType::Hash)
                                    .build()?;

                                let pt = ProvisionedThroughput::builder()
                                    .read_capacity_units(1)
                                    .write_capacity_units(1)
                                    .build()?;

                                let r = client
                                    .create_table()
                                    .table_name(&self.table_name)
                                    .attribute_definitions(ad_id)
                                    //.attribute_definitions(ad_lock)
                                    //.attribute_definitions(ad_data)
                                    .key_schema(key_id)
                                    //.key_schema(key_lock)
                                    //.key_schema(key_data)
                                    .provisioned_throughput(pt);
                                r.send().await?;
                            }
                            oe => return Err(eyre!("Error describing table {oe:?}")),
                        }
                    }
                    _o => {
                        todo!();
                    }
                }
            }
        };

        // tracing::debug!("{client:?}");

        // insert test data
        /*
        let request = client
            .put_item()
            .table_name(&self.table_name)
            .item("id", AttributeValue::S(nanoid::nanoid!()))
            .item("lock", AttributeValue::S(String::from("")))
            .item("data", AttributeValue::S(String::from("{}")))
            .send()
            .await?;
        */
        Ok(())
    }
}

#[async_trait]
impl<ITEM: StorageItem + std::marker::Send> Storage<ITEM> for StorageDynamoDb<ITEM> {
    async fn ensure_storage_exists(&mut self) -> Result<()> {
        Ok(())
    }
    async fn create(&self) -> Result<String> {
        let mut tries = 10;
        loop {
            let id = nanoid::nanoid!();
            if !self.exists(&id).await? {
                return Ok(id);
            }

            tries -= 1;
            if tries <= 0 {
                todo!();
            }
        }
    }
    async fn exists(&self, _id: &str) -> Result<bool> {
        Ok(false) // :TODO:
    }

    async fn load(&self, _id: &str) -> Result<ITEM> {
        todo!();
    }

    async fn save(&self, id: &str, item: &ITEM, lock: &StorageLock) -> Result<()> {
        tracing::info!("Saving: {id} -> {item:?} with lock {lock:?}");
        let client = self.client().await?;
        let data = item.serialize()?;
        let data = String::from_utf8_lossy(&data);
        match client
            .update_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(String::from(id)))
            //.expression_attribute_names()
            //.update_expression("SET #Count = if_not_exists(#Count, :zero) + :one, Images = list_append(if_not_exists(Images, :empty), :image)")
            .update_expression("SET #Data = :data")
            .expression_attribute_names("#Data", "data")
            .expression_attribute_values(
                ":data",
                aws_sdk_dynamodb::types::AttributeValue::S(data.to_string()),
            )
            .return_values(ReturnValue::AllOld)
            .send()
            .await
        {
            Ok(o) => {
                tracing::info!("Save - UpdateItem {id} success {o:?}");
                Ok(())
            }
            Err(e) => {
                tracing::warn!("Save - UpdateItem {id} failure {e:?}");
                todo!();
            }
        }
    }
    async fn lock(&self, id: &str, who: &str) -> Result<LockResult<ITEM>> {
        let lock = StorageLock::new(who);
        let lock_json = serde_json::to_string_pretty(&lock)?;

        // write lock
        let client = self.client().await?;

        match client
            .update_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(String::from(id)))
            //.expression_attribute_names()
            //.update_expression("SET #Count = if_not_exists(#Count, :zero) + :one, Images = list_append(if_not_exists(Images, :empty), :image)")
            .update_expression("SET #Lock = :lock")
            .expression_attribute_names("#Lock", "lock")
            .expression_attribute_values(
                ":lock",
                aws_sdk_dynamodb::types::AttributeValue::S(lock_json),
            )
            .return_values(ReturnValue::AllOld)
            .send()
            .await
        {
            Ok(o) => {
                tracing::info!("Lock - UpdateItem {id} success {o:?}");
                let item = match o {
                    UpdateItemOutput { ref attributes, .. } => {
                        if let Some(attributes) = &attributes {
                            if let Some(data) = attributes.get("data") {
                                match data {
                                    AttributeValue::S(data) => {
                                        let item = ITEM::deserialize(data.as_bytes())?;
                                        tracing::info!("Lock - Got item {item:?}");
                                        item
                                    }
                                    o => {
                                        tracing::warn!(
                                            "No data attribute for item is not a string {o:?}"
                                        );
                                        ITEM::default()
                                    }
                                }
                            } else {
                                tracing::warn!("No data attribute for item");
                                ITEM::default()
                            }
                        } else {
                            tracing::warn!("No attributes for item");
                            ITEM::default()
                        }
                    }
                };

                //let item = ITEM::default();
                Ok(LockResult::Success { lock, item })
            }
            Err(e) => {
                tracing::warn!("Lock - UpdateItem {id} failure {e:?}");
                todo!();
            }
        }
    }

    async fn unlock(&self, _id: &str, _lock: StorageLock) -> Result<()> {
        todo!();
    }

    async fn force_unlock(&self, _id: &str) -> Result<()> {
        todo!();
    }
    async fn verify_lock(&self, _id: &str, _lock: &StorageLock) -> Result<bool> {
        todo!();
    }
    async fn all_ids(&self) -> Result<Vec<String>> {
        todo!();
        // Ok(Vec::default())
    }

    async fn display_lock(&self, _id: &str) -> Result<String> {
        todo!();
    }
    #[cfg(feature = "metadata")]
    async fn metadata_highest_seen_id(&self) -> String {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use crate::Storage;
    use crate::StorageDynamoDb;
    use crate::StorageItem;
    use color_eyre::Result;
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Default, Debug, Serialize, Deserialize)]
    struct TestItem {}

    impl StorageItem for TestItem {
        fn serialize(&self) -> Result<Vec<u8>> {
            todo!()
        }
        fn deserialize(_: &[u8]) -> Result<Self> {
            todo!()
        }
    }

    #[tokio::test]
    async fn it_debugs() -> Result<()> {
        let table_name = "test_items";
        let storage = StorageDynamoDb::<TestItem>::new(&table_name).await;
        println!("{storage:?}");

        let storage: Box<dyn Storage<TestItem>> = Box::new(storage);
        println!("{storage:?}");

        Ok(())
    }
}
