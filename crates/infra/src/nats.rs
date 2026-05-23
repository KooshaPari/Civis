use crate::InfraError;

/// NATS client wrapper.
pub struct NatsClient {
    client: async_nats::Client,
}

impl NatsClient {
    /// Connect to NATS.
    pub async fn connect(url: &str) -> Result<Self, InfraError> {
        Ok(Self {
            client: async_nats::connect(url).await?,
        })
    }

    /// Publish a payload to a topic.
    pub async fn publish(
        &self,
        topic: &str,
        payload: impl Into<bytes::Bytes>,
    ) -> Result<(), InfraError> {
        self.client
            .publish(topic.to_owned(), payload.into())
            .await?;
        Ok(())
    }

    /// Subscribe to a topic.
    pub async fn subscribe(&self, topic: &str) -> Result<async_nats::Subscriber, InfraError> {
        Ok(self.client.subscribe(topic.to_owned()).await?)
    }
}
