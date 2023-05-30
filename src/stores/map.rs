use async_trait::async_trait;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, SystemTimeError, UNIX_EPOCH},
};
use tokio::sync::Mutex;

use crate::Store;

#[derive(Clone, Debug)]
struct MapValue {
    value: usize,
    expiry_s: usize,
    unix: u64,
}

#[derive(Clone)]
pub struct MapStore {
    hash_map: Arc<Mutex<HashMap<String, MapValue>>>,
}

impl MapStore {
    pub fn new() -> Self {
        return Self {
            hash_map: Arc::new(Mutex::new(HashMap::new())),
        };
    }
}

impl Default for MapStore {
    fn default() -> Self {
        return Self::new();
    }
}

#[async_trait]
impl Store for MapStore {
    type Error = SystemTimeError;

    async fn get(&mut self, key: &str) -> Result<Option<usize>, Self::Error> {
        let mut hash_map = self.hash_map.lock().await;

        let MapValue {
            value,
            expiry_s,
            unix,
        } = match hash_map.get(key).cloned() {
            Some(x) => x,
            None => return Ok(None),
        };

        let remove_at = unix + expiry_s as u64;
        let now_unix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        if remove_at <= now_unix {
            hash_map.remove(key);
            return Ok(None);
        }

        return Ok(Some(value));
    }

    async fn set(&mut self, key: &str, value: usize, expiry_s: usize) -> Result<(), Self::Error> {
        let mut hash_map = self.hash_map.lock().await;

        if let Some(already) = hash_map.get(key) {
            let already = already.clone();
            hash_map.insert(key.to_string(), MapValue { value, ..already });

            return Ok(());
        }

        hash_map.insert(
            key.to_string(),
            MapValue {
                expiry_s,
                value,
                unix: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            },
        );

        return Ok(());
    }
}
