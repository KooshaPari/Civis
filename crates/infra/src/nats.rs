use crate::InfraError;

/// NATS client wrapper.
pub struct NatsClient {
    client: async_nats::Client,
}

impl NatsClient {
    /// Connect to NATS.
    pub async fn connect(url: &str) -> Result<Self, InfraError> {
        Ok(Self {
            client: async_nats::connect(url)
                .await
                .map_err(|err| InfraError::Nats(err.to_string()))?,
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
            .await
            .map_err(|err| InfraError::Nats(err.to_string()))?;
        Ok(())
    }

    /// Subscribe to a topic.
    pub async fn subscribe(&self, topic: &str) -> Result<async_nats::Subscriber, InfraError> {
        self.client
            .subscribe(topic.to_owned())
            .await
            .map_err(|err| InfraError::Nats(err.to_string()))
    }
}
