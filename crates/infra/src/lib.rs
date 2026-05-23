//! Shared infrastructure client wrappers for Civis services.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Redis-compatible cache wrapper.
#[cfg(feature = "cache")]
pub mod cache;
/// S3-compatible client wrapper.
#[cfg(feature = "s3")]
pub mod minio;
/// NATS client wrapper.
#[cfg(feature = "nats")]
pub mod nats;
/// PostgreSQL client wrapper.
#[cfg(feature = "pg")]
pub mod pg;

/// Unified infrastructure error.
#[derive(Debug, thiserror::Error)]
pub enum InfraError {
    /// PostgreSQL error.
    #[cfg(feature = "pg")]
    #[error("postgres error: {0}")]
    Postgres(#[from] sqlx::Error),
    /// NATS error.
    #[cfg(feature = "nats")]
    #[error("nats error: {0}")]
    Nats(#[from] async_nats::Error),
    /// S3 error.
    #[cfg(feature = "s3")]
    #[error("s3 error: {0}")]
    S3(String),
    /// Redis error.
    #[cfg(feature = "cache")]
    #[error("cache error: {0}")]
    Cache(String),
    /// Missing runtime configuration.
    #[error("missing configuration: {0}")]
    MissingConfig(String),
}

#[cfg(feature = "cache")]
impl From<redis::RedisError> for InfraError {
    fn from(value: redis::RedisError) -> Self {
        Self::Cache(value.to_string())
    }
}
