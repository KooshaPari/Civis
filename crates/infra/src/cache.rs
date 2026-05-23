use crate::InfraError;

/// Redis-compatible cache wrapper.
pub struct Cache {
    client: redis::Client,
}

impl Cache {
    /// Connect to a Redis-compatible cache.
    pub async fn connect(url: &str) -> Result<Self, InfraError> {
        Ok(Self {
            client: redis::Client::open(url)?,
        })
    }

    async fn connection(&self) -> Result<redis::aio::MultiplexedConnection, InfraError> {
        Ok(self.client.get_multiplexed_tokio_connection().await?)
    }

    /// Set a key to a binary payload.
    pub async fn set(&self, key: &[u8], value: &[u8]) -> Result<(), InfraError> {
        let mut conn = self.connection().await?;
        redis::cmd("SET")
            .arg(key)
            .arg(value)
            .query_async::<_, ()>(&mut conn)
            .await?;
        Ok(())
    }

    /// Fetch a binary payload.
    pub async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, InfraError> {
        let mut conn = self.connection().await?;
        Ok(redis::cmd("GET")
            .arg(key)
            .query_async::<_, Option<Vec<u8>>>(&mut conn)
            .await?)
    }

    /// Delete a key.
    pub async fn del(&self, key: &[u8]) -> Result<(), InfraError> {
        let mut conn = self.connection().await?;
        redis::cmd("DEL")
            .arg(key)
            .query_async::<_, ()>(&mut conn)
            .await?;
        Ok(())
    }
}
