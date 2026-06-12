use crate::Error;

/// NATS client wrapper.
pub struct NatsClient {
    client: async_nats::Client,
}

impl NatsClient {
    /// Connect to NATS.
    pub async fn connect(url: &str) -> Result<Self, Error> {
        Ok(Self {
            client: async_nats::connect(url)
                .await
                .map_err(|err| Error::Nats(err.to_string()))?,
        })
    }

    /// Publish a payload to a topic.
    pub async fn publish(
        &self,
        topic: &str,
        payload: impl Into<bytes::Bytes>,
    ) -> Result<(), Error> {
        self.client
            .publish(topic.to_owned(), payload.into())
            .await
            .map_err(|err| Error::Nats(err.to_string()))?;
        Ok(())
    }

    /// Subscribe to a topic.
    pub async fn subscribe(&self, topic: &str) -> Result<async_nats::Subscriber, Error> {
        self.client
            .subscribe(topic.to_owned())
            .await
            .map_err(|err| Error::Nats(err.to_string()))
    }
}
