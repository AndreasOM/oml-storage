use crate::StorageItem;
use core::marker::PhantomData;
use std::sync::Arc;
use std::sync::RwLock;

#[cfg(feature = "metadata")]
#[derive(Debug, Default)]
pub(crate) struct Metadata<ITEM: StorageItem> {
    item_type: PhantomData<ITEM>,
    highest_seen_id: Arc<RwLock<Option<ITEM::ID>>>,
}
#[cfg(feature = "metadata")]
impl<ITEM: StorageItem> Metadata<ITEM> {
    pub fn highest_seen_id(&self) -> Option<ITEM::ID> {
        self.highest_seen_id.read().expect("can read lock").clone()
    }

    pub fn update_highest_seen_id(&self, id: &ITEM::ID) {
        let highest_seen_id = self.highest_seen_id.read().expect("can read lock");
        tracing::debug!("update_highest_seen_id: '{id}' >? '{highest_seen_id:?}'");
        let higher = if let Some(highest_seen_id) = &*highest_seen_id {
            // :HACK to ensure we compare numbers correctly
            let higher = *id > *highest_seen_id;
            /*
            let higher = match (id.parse::<u64>(), highest_seen_id.parse::<u64>()) {
                (Ok(a), Ok(b)) => a > b,
                _ => *id > **highest_seen_id,
            };
            */
            tracing::debug!("update_highest_seen_id: '{id}' >? '{highest_seen_id:?}'");
            higher
        } else {
            true
        };

        if higher {
            drop(highest_seen_id);
            tracing::debug!("Updating to {id}");
            let mut highest_seen_id = self.highest_seen_id.write().expect("can write lock");
            //*highest_seen_id = id.to_string();
            *highest_seen_id = Some(id.to_owned());
        }
    }
}
