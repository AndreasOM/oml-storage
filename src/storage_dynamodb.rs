use crate::storage::LockNewResult;
use crate::LockResult;
#[cfg(feature = "metadata")]
use crate::Metadata;
use crate::Storage;
use crate::StorageItem;
use crate::StorageLock;
use async_trait::async_trait;
use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::describe_table::DescribeTableError::ResourceNotFoundException;
use aws_sdk_dynamodb::operation::get_item::GetItemOutput;
use aws_sdk_dynamodb::operation::scan::ScanOutput;
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
    #[cfg(feature = "metadata")]
    metadata: Metadata<ITEM>,
}

#[cfg(feature = "metadata")]
impl<ITEM: StorageItem> StorageDynamoDb<ITEM> {
    fn update_highest_seen_id(&self, id: &ITEM::ID) {
        self.metadata.update_highest_seen_id(id);
    }
}

#[cfg(not(feature = "metadata"))]
impl<ITEM: StorageItem> StorageDynamoDb<ITEM> {
    fn update_highest_seen_id(&self, _id: &ITEM::ID) {}
}

impl<ITEM: StorageItem> StorageDynamoDb<ITEM> {
    pub async fn new(table_name: &str) -> Self {
        Self {
            table_name: String::from(table_name),
            endpoint_url: None,
            item_type: PhantomData,
            #[cfg(feature = "metadata")]
            metadata: Metadata::default(),
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
        self.ensure_table_exists().await
    }
    async fn create(&self) -> Result<ITEM::ID> {
        let mut tries = 10;
        loop {
            //let id = nanoid::nanoid!();
            let id = ITEM::generate_next_id(None);
            if !self.exists(&id).await? {
                return Ok(id);
            }

            tries -= 1;
            if tries <= 0 {
                todo!();
            }
        }
    }
    async fn exists(&self, id: &ITEM::ID) -> Result<bool> {
        tracing::info!("Checking if {id} exists");
        let client = self.client().await?;
        match client
            .get_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .projection_expression("#Id")
            .expression_attribute_names("#Id", "id")
            .send()
            .await
        {
            Ok(o) => {
                tracing::info!("Check - GetItem {id} success {o:?}");
                let Some(_item) = o.item else {
                    return Ok(false);
                };
                self.update_highest_seen_id(id);
                Ok(true)
            }
            Err(e) => {
                tracing::warn!("Check - GetItem {id} failure {e:?}");
                Err(eyre!(":TODO:"))
            }
        }
        //Ok(false) // :TODO:
    }

    async fn load(&self, _id: &ITEM::ID) -> Result<ITEM> {
        todo!();
    }

    async fn save(&self, id: &ITEM::ID, item: &ITEM, lock: &StorageLock) -> Result<()> {
        tracing::info!("Saving: {id} -> {item:?} with lock {lock:?}");
        let lock_json = serde_json::to_string_pretty(&lock)?;
        let client = self.client().await?;
        let data = item.serialize()?;
        let data = String::from_utf8_lossy(&data);
        match client
            .update_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .update_expression("SET #Data = :data")
            .expression_attribute_names("#Data", "data")
            .expression_attribute_values(
                ":data",
                aws_sdk_dynamodb::types::AttributeValue::S(data.to_string()),
            )
            .condition_expression("#Lock = :lock")
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
                tracing::info!("Save - UpdateItem {id} success {o:?}");
                self.update_highest_seen_id(id);
                Ok(())
            }
            Err(e) => {
                tracing::warn!("Save - UpdateItem {id} failure {e:?}");
                // :TODO: check if it was actually the lock that failed
                Err(eyre!("Lock invalid!"))
            }
        }
    }
    async fn lock(&self, id: &ITEM::ID, who: &str) -> Result<LockResult<ITEM>> {
        let lock = StorageLock::new(who);
        let lock_json = serde_json::to_string_pretty(&lock)?;

        // write lock
        let client = self.client().await?;

        match client
            .update_item()
            .table_name(&self.table_name)
            //.key("id", AttributeValue::S(String::from(id)))
            .key("id", AttributeValue::S(id.to_string()))
            //.expression_attribute_names()
            //.update_expression("SET #Count = if_not_exists(#Count, :zero) + :one, Images = list_append(if_not_exists(Images, :empty), :image)")
            .update_expression("SET #Lock = :lock")
            .expression_attribute_names("#Lock", "lock")
            .expression_attribute_values(
                ":lock",
                aws_sdk_dynamodb::types::AttributeValue::S(lock_json),
            )
            .condition_expression("attribute_not_exists(#Lock)")
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
                                        self.update_highest_seen_id(id);
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
                return Ok(LockResult::AlreadyLocked {
                    who: String::from(":TODO:"),
                });
            }
        }
    }

    async fn lock_new(&self, _id: &ITEM::ID, _who: &str) -> Result<LockNewResult<ITEM>> {
        todo!("lock_new is not implemented for DynamoDB");
    }

    async fn unlock(&self, id: &ITEM::ID, lock: StorageLock) -> Result<()> {
        tracing::info!("Unlocking: {id} with lock {lock:?}");
        let lock_json = serde_json::to_string_pretty(&lock)?;
        let client = self.client().await?;
        match client
            .update_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .update_expression("REMOVE #Lock")
            .expression_attribute_names("#Lock", "lock")
            .condition_expression("#Lock = :lock")
            .expression_attribute_values(
                ":lock",
                aws_sdk_dynamodb::types::AttributeValue::S(lock_json),
            )
            .return_values(ReturnValue::None)
            .send()
            .await
        {
            Ok(o) => {
                tracing::info!("Unlock - UpdateItem {id} success {o:?}");
                self.update_highest_seen_id(id);
                Ok(())
            }
            Err(e) => {
                tracing::warn!("Unlock - UpdateItem {id} failure {e:?}");
                // :TODO: check if it was actually the lock that failed
                Err(eyre!("Lock invalid!"))
            }
        }
    }

    async fn force_unlock(&self, id: &ITEM::ID) -> Result<()> {
        tracing::info!("Force Unlocking: {id}");
        let client = self.client().await?;
        match client
            .update_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .update_expression("REMOVE #Lock")
            .expression_attribute_names("#Lock", "lock")
            .return_values(ReturnValue::None)
            .send()
            .await
        {
            Ok(o) => {
                tracing::info!("Force Unlock - UpdateItem {id} success {o:?}");
                self.update_highest_seen_id(id);
                Ok(())
            }
            Err(e) => {
                tracing::warn!("Force Unlock - UpdateItem {id} failure {e:?}");
                // :TODO: check
                Err(eyre!("Lock invalid!"))
            }
        }
    }
    async fn verify_lock(&self, id: &ITEM::ID, lock: &StorageLock) -> Result<bool> {
        tracing::info!("Checking if lock {lock:?} is correct for {id}");
        let client = self.client().await?;
        match client
            .get_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .projection_expression("#Id, #Lock")
            .expression_attribute_names("#Id", "id")
            .expression_attribute_names("#Lock", "lock")
            .send()
            .await
        {
            Ok(o) => {
                let Some(item) = o.item else {
                    // item does not exist so lock can't be valid
                    return Ok(false);
                };
                // tracing::info!("{item:#?}");
                self.update_highest_seen_id(id);
                let Some(lock_json) = item.get("lock") else {
                    // item has no lock so lock can't be valid
                    return Ok(false);
                };
                let Ok(lock_json) = lock_json.as_s() else {
                    // item lock has wrong type so lock can't be valid
                    return Ok(false);
                };

                let Ok(db_lock) = serde_json::from_str::<StorageLock>(lock_json) else {
                    // item lock has wrong content so lock can't be valid
                    return Ok(false);
                };

                Ok(*lock == db_lock)
            }
            Err(e) => {
                tracing::warn!("Check - GetItem {id} failure {e:?}");
                Err(eyre!(":TODO:"))
            }
        }
    }
    async fn all_ids(&self) -> Result<Vec<ITEM::ID>> {
        todo!();
        // Ok(Vec::default())
    }
    async fn scan_ids(
        &self,
        start: Option<&str>,
        limit: Option<usize>,
    ) -> Result<(Vec<ITEM::ID>, Option<String>)> {
        // tracing::info!("Scanning Ids: {start:?} {limit:?}");
        let client = self.client().await?;
        let mut scan = client
            .scan()
            .table_name(&self.table_name)
            .projection_expression("#Id")
            .expression_attribute_names("#Id", "id");
        if let Some(start) = start {
            scan = scan.exclusive_start_key("id", AttributeValue::S(start.to_string()));
        }
        if let Some(limit) = limit {
            scan = scan.limit(limit as i32);
        }
        match scan.send().await {
            Ok(ScanOutput {
                items,
                last_evaluated_key,
                ..
            }) => {
                // tracing::info!("Scanning Ids - Scan success {items:?} {last_evaluated_key:?}");

                // :TODO: flatten
                let scan_pos = match last_evaluated_key {
                    None => None,
                    Some(k) => {
                        if let Some(last_id) = k.get("id") {
                            if let Ok(last_id_s) = last_id.as_s() {
                                Some(last_id_s.to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                };
                // :TODO: map and collect ?
                let mut ids = Vec::default();
                if let Some(items) = items {
                    for item in items {
                        if let Some(ida) = item.get("id") {
                            if let Ok(id_s) = ida.as_s() {
                                let id: ITEM::ID = ITEM::make_id(id_s)?;
                                // :LATER: self.update_highest_seen_id(&id);
                                ids.push(id);
                            }
                        }
                    }
                };
                Ok((ids, scan_pos))
            }
            Err(e) => {
                tracing::warn!("Scanning Ids - Scan failure {e:?}");
                // :TODO: check
                Err(eyre!("I don't know what happened ;) {e:?}!"))
            }
        }
    }

    async fn display_lock(&self, id: &ITEM::ID) -> Result<String> {
        let client = self.client().await?;
        match client
            .get_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .projection_expression("#Lock")
            .expression_attribute_names("#Lock", "lock")
            .send()
            .await
        {
            Ok(GetItemOutput { mut item, .. }) => {
                // tracing::info!("Display Lock - GetItem {id} success {item:?}");
                if let Some(item) = item.take() {
                    // locked
                    let Some(lock_json) = item.get("lock") else {
                        // not locked
                        return Ok(String::default());
                    };
                    let lock_json = lock_json.as_s().map_err(|e| eyre!(":TODO: {e:?}"))?;
                    let lock: StorageLock = serde_json::from_str(lock_json)?;
                    let lock_string = format!("Locked by {} at {:?}", lock.who(), lock.when());

                    Ok(lock_string)
                } else {
                    // not locked
                    Ok(String::default())
                }
            }
            Err(e) => {
                tracing::warn!("Display Lock  - GetItem {id} failure {e:?}");
                Err(eyre!(":TODO: {e:?}"))
            }
        }
    }
    #[cfg(feature = "metadata")]
    async fn metadata_highest_seen_id(&self) -> Option<ITEM::ID> {
        self.metadata.highest_seen_id()
    }

    #[cfg(feature = "wipe")]
    async fn wipe(&self, confirmation: &str) -> Result<()> {
        if confirmation != "Yes, I know what I am doing!" {
            tracing::error!("Please confirm you know what you are doing");
            return Err(eyre!("Unconfirmed wipe attempt"));
        }

        let mut count = 0;
        let mut scan_pos: Option<String> = None;
        loop {
            let (ids, new_scan_pos) = self.scan_ids(scan_pos.as_deref(), Some(3)).await?;
            scan_pos = new_scan_pos;

            for id in ids {
                tracing::info!("Deleting {id}");
                let client = self.client().await?;
                match client
                    .delete_item()
                    .table_name(&self.table_name)
                    .key("id", AttributeValue::S(id.to_string()))
                    .return_values(ReturnValue::None)
                    .send()
                    .await
                {
                    Ok(o) => {
                        tracing::info!("Deleting - UpdateItem {id} success {o:?}");
                        self.update_highest_seen_id(&id);
                        count += 1;
                    }
                    Err(e) => {
                        tracing::warn!("Deleting - UpdateItem {id} failure {e:?}");
                    }
                }
            }

            if scan_pos.is_none() {
                break;
            }
        }

        tracing::warn!("Deleted {count} items");
        Ok(())
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
        type ID = String;
        fn serialize(&self) -> Result<Vec<u8>> {
            todo!()
        }
        fn deserialize(_: &[u8]) -> Result<Self> {
            todo!()
        }
        fn generate_next_id(a_previous_id: Option<&Self::ID>) -> Self::ID {
            todo!();
        }
        fn make_id(id: &str) -> Result<Self::ID> {
            let id = id.parse::<Self::ID>()?;
            Ok(id)
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
