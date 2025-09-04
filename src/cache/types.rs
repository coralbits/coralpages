use async_trait::async_trait;

#[async_trait]
pub trait Cache: Send + Sync {
    async fn get(&self, key: &str) -> Option<String>;
    async fn set(&self, key: &str, value: &str);
    async fn delete(&self, key: &str) -> Option<()>;
}
