
pub mod mock;


#[async_trait::async_trait]
pub trait Storage: Send + Sync {
    async fn get(&self, id: &String) -> anyhow::Result<&crate::profile::Profile>;
    async fn put(&mut self, profile: crate::profile::Profile) -> anyhow::Result<()>;
    async fn delete(&mut self, id: &String) -> anyhow::Result<crate::profile::Profile>;
}



