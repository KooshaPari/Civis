use crate::Error;
use aws_config::BehaviorVersion;
use aws_sdk_s3::{config::Credentials, primitives::ByteStream, Client};

/// MinIO/S3 client wrapper.
pub struct MinioClient {
    client: Client,
    bucket: String,
}

impl MinioClient {
    /// Construct a client for a MinIO-compatible endpoint.
    pub async fn new(
        endpoint: &str,
        access_key: &str,
        secret_key: &str,
    ) -> Result<Self, Error> {
        let credentials = Credentials::new(access_key, secret_key, None, None, "minio");
        let config = aws_config::defaults(BehaviorVersion::latest())
            .endpoint_url(endpoint)
            .credentials_provider(credentials)
            .load()
            .await;
        Ok(Self {
            client: Client::new(&config),
            bucket: "civis".to_string(),
        })
    }

    /// Put an object into the default bucket.
    pub async fn put_object(&self, key: &str, body: Vec<u8>) -> Result<(), Error> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(body))
            .send()
            .await
            .map_err(|err| Error::S3(err.to_string()))?;
        Ok(())
    }

    /// Fetch an object from the default bucket.
    pub async fn get_object(&self, key: &str) -> Result<Vec<u8>, Error> {
        let out = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|err| Error::S3(err.to_string()))?;
        let data = out
            .body
            .collect()
            .await
            .map_err(|err| Error::S3(err.to_string()))?;
        Ok(data.into_bytes().to_vec())
    }
}
