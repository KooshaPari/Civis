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

/// Desire-path emergence tracker (FR-CIV-ROAD-900). Pure-logic, no Bevy
/// dependency; tracks accumulated traversal weight between world cells and
/// decays unused paths back to bare ground.
pub mod desire_paths;
pub use desire_paths::{
    DesireEdge, DesireEdgeKey, DesirePathConfig, DesirePathTracker, PathState,
};

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
    Nats(String),
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

#[cfg(test)]
mod tests {
    use super::InfraError;

    #[test]
    fn missing_config_error_is_actionable() {
        let err = InfraError::MissingConfig("DATABASE_URL".into());
        let message = err.to_string();
        assert!(message.contains("DATABASE_URL"));
        assert!(message.contains("missing configuration"));
    }

    #[cfg(feature = "nats")]
    #[test]
    fn nats_error_includes_detail() {
        let err = InfraError::Nats("connection refused".into());
        let message = err.to_string();
        assert!(message.contains("nats error"));
        assert!(message.contains("connection refused"));
    }

    #[cfg(feature = "s3")]
    #[test]
    fn s3_error_includes_detail() {
        let err = InfraError::S3("bucket missing".into());
        let message = err.to_string();
        assert!(message.contains("s3 error"));
        assert!(message.contains("bucket missing"));
    }

    #[cfg(feature = "cache")]
    #[test]
    fn cache_error_includes_detail() {
        let err = InfraError::Cache("broken pipe".into());
        let message = err.to_string();
        assert!(message.contains("cache error"));
        assert!(message.contains("broken pipe"));
    }

    #[cfg(feature = "cache")]
    #[test]
    fn redis_error_converts_to_cache_variant() {
        let redis_err = redis::RedisError::from((redis::ErrorKind::IoError, "broken pipe"));
        let err: InfraError = redis_err.into();
        assert!(err.to_string().contains("cache error"));
    }

    #[cfg(feature = "pg")]
    #[test]
    fn postgres_error_includes_detail() {
        let err = InfraError::Postgres(sqlx::Error::PoolClosed);
        let message = err.to_string();
        assert!(message.contains("postgres error"));
    }

    #[cfg(feature = "pg")]
    #[test]
    fn sqlx_error_converts_to_postgres_variant() {
        let err: InfraError = sqlx::Error::PoolClosed.into();
        assert!(matches!(err, InfraError::Postgres(_)));
        assert!(err.to_string().contains("postgres error"));
    }
}
