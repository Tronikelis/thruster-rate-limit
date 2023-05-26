use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands, RedisError, RedisResult};

use super::Store;

#[derive(Clone)]
pub struct RedisStore {
    connection_manager: ConnectionManager,
}

impl RedisStore {
    pub async fn new(url: &str) -> RedisResult<Self> {
        let client = redis::Client::open(url)?;
        let connection_manager = ConnectionManager::new(client).await?;

        return Ok(Self { connection_manager });
    }
}

#[async_trait]
impl Store for RedisStore {
    type Error = RedisError;

    async fn get(&mut self, key: &str) -> Result<Option<usize>, Self::Error> {
        let current: Option<usize> = self.connection_manager.get(key).await?;
        return Ok(current);
    }

    async fn set(&mut self, key: &str, value: usize, expiry_s: usize) -> Result<(), Self::Error> {
        let _: () = self.connection_manager.set_ex(key, value, expiry_s).await?;
        return Ok(());
    }
}
