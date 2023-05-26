pub mod map;
pub mod redis;

#[async_trait]
pub trait Store {
    type Error: Debug;
    async fn get(&mut self, key: &str) -> Result<Option<usize>, Self::Error>;
    async fn set(&mut self, key: &str, value: usize, expiry_ms: usize) -> Result<(), Self::Error>;
}