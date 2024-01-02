use crate::LockResult;
use crate::Storage;
use crate::StorageItem;
use crate::StorageLock;
use async_trait::async_trait;
use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::describe_table::DescribeTableError::ResourceNotFoundException;
use aws_sdk_dynamodb::types::AttributeDefinition;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::types::KeySchemaElement;
use aws_sdk_dynamodb::types::KeyType;
use aws_sdk_dynamodb::types::ProvisionedThroughput;
use aws_sdk_dynamodb::types::ScalarAttributeType;
use color_eyre::eyre::eyre;
use color_eyre::eyre::Result;
use tokio::sync::Semaphore;

use core::marker::PhantomData;

#[derive(Debug)]
pub struct StorageDynamoDb<ITEM: StorageItem> {
    table_name: String,
    endpoint_url: Option<String>,
    item_type: PhantomData<ITEM>,
    lock_semaphore: Semaphore,
}

impl<ITEM: StorageItem> StorageDynamoDb<ITEM> {
    pub async fn new(table_name: &str) -> Self {
        Self {
            table_name: String::from(table_name),
            endpoint_url: None,
            item_type: PhantomData,
            lock_semaphore: Semaphore::new(1),
        }
    }

    pub fn set_endpoint_url(&mut self, url: &str) -> Result<()> {
        self.endpoint_url = Some(String::from(url));

        Ok(())
    }
    pub async fn ensure_table_exists(&mut self) -> Result<()> {
        // let config = aws_config::load_from_env().await;
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest());
        let config = if let Some(endpoint_url) = &self.endpoint_url {
            config.endpoint_url(endpoint_url)
        } else {
            config
        };
        let config = config.load().await;
        let client = aws_sdk_dynamodb::Client::new(&config);
        match client
            .describe_table()
            .table_name(&self.table_name)
            .send()
            .await
        {
            Ok(_o) => {
                // :TODO: verify table format?
            }
            Err(e) => {
                // tracing::debug!("Err {e:?}");
                match e {
                    SdkError::ServiceError(se) => {
                        match se.err() {
                            ResourceNotFoundException(nf) => {
                                // tracing::debug!("{nf:?}");
                                tracing::info!("Table {} not found. Creating...", &self.table_name);

                                // :TODO:

                                let ad_id = AttributeDefinition::builder()
                                    .attribute_name("id")
                                    .attribute_type(ScalarAttributeType::S)
                                    .build()?;
                                let ad_lock = AttributeDefinition::builder()
                                    .attribute_name("lock")
                                    .attribute_type(ScalarAttributeType::S)
                                    .build()?;
                                let ad_data = AttributeDefinition::builder()
                                    .attribute_name("data")
                                    .attribute_type(ScalarAttributeType::S)
                                    .build()?;

                                let key_id = KeySchemaElement::builder()
                                    .attribute_name("id")
                                    .key_type(KeyType::Hash)
                                    .build()?;

                                let key_lock = KeySchemaElement::builder()
                                    .attribute_name("lock")
                                    .key_type(KeyType::Range)
                                    .build()?;

                                let key_data = KeySchemaElement::builder()
                                    .attribute_name("data")
                                    .key_type(KeyType::Range)
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
                                // add schema
                                // id | lock | data
                                // string | string | string

                                /*
                                    let ad = AttributeDefinition::builder()
                                        .attribute_name(&a_name)
                                        .attribute_type(ScalarAttributeType::S)
                                        .build()
                                        .map_err(Error::BuildError)?;

                                    let ks = KeySchemaElement::builder()
                                        .attribute_name(&a_name)
                                        .key_type(KeyType::Hash)
                                        .build()
                                        .map_err(Error::BuildError)?;
                                */
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
        let request = client
            .put_item()
            .table_name(&self.table_name)
            .item("id", AttributeValue::S(nanoid::nanoid!()))
            .item("lock", AttributeValue::S(String::from("")))
            .item("data", AttributeValue::S(String::from("{}")))
            .send()
            .await?;

        Ok(())
    }
}

#[async_trait]
impl<ITEM: StorageItem + std::marker::Send> Storage<ITEM> for StorageDynamoDb<ITEM> {
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
    async fn exists(&self, id: &str) -> Result<bool> {
        todo!();
    }

    async fn load(&self, id: &str) -> Result<ITEM> {
        todo!();
    }

    async fn save(&self, id: &str, item: &ITEM, lock: &StorageLock) -> Result<()> {
        todo!();
    }
    async fn lock(&self, id: &str, who: &str) -> Result<LockResult<ITEM>> {
        todo!();
    }

    async fn unlock(&self, id: &str, lock: StorageLock) -> Result<()> {
        todo!();
    }

    async fn force_unlock(&self, id: &str) -> Result<()> {
        todo!();
    }
    async fn verify_lock(&self, id: &str, lock: &StorageLock) -> Result<bool> {
        todo!();
    }
}
