//! Shared infrastructure client wrappers for Civis services.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Redis-compatible cache wrapper.
#[cfg(feature = "cache")]
pub mod cache;
/// Local error type.
pub mod error;
/// S3-compatible client wrapper.
#[cfg(feature = "s3")]
pub mod minio;
/// NATS client wrapper.
#[cfg(feature = "nats")]
pub mod nats;
/// PostgreSQL client wrapper.
#[cfg(feature = "pg")]
pub mod pg;

pub use error::Error;

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn missing_config_error_is_actionable() {
        let err = Error::MissingConfig("DATABASE_URL".into());
        let message = err.to_string();
        assert!(message.contains("DATABASE_URL"));
        assert!(message.contains("missing configuration"));
    }

    #[cfg(feature = "nats")]
    #[test]
    fn nats_error_includes_detail() {
        let err = Error::Nats("connection refused".into());
        let message = err.to_string();
        assert!(message.contains("nats error"));
        assert!(message.contains("connection refused"));
    }

    #[cfg(feature = "s3")]
    #[test]
    fn s3_error_includes_detail() {
        let err = Error::S3("bucket missing".into());
        let message = err.to_string();
        assert!(message.contains("s3 error"));
        assert!(message.contains("bucket missing"));
    }

    #[cfg(feature = "cache")]
    #[test]
    fn cache_error_includes_detail() {
        let err = Error::Cache("broken pipe".into());
        let message = err.to_string();
        assert!(message.contains("cache error"));
        assert!(message.contains("broken pipe"));
    }

    #[cfg(feature = "cache")]
    #[test]
    fn redis_error_converts_to_cache_variant() {
        let redis_err = redis::RedisError::from((redis::ErrorKind::IoError, "broken pipe"));
        let err: Error = redis_err.into();
        assert!(err.to_string().contains("cache error"));
    }

    #[cfg(feature = "pg")]
    #[test]
    fn postgres_error_includes_detail() {
        let err = Error::Postgres(sqlx::Error::PoolClosed);
        let message = err.to_string();
        assert!(message.contains("postgres error"));
    }

    #[cfg(feature = "pg")]
    #[test]
    fn sqlx_error_converts_to_postgres_variant() {
        let err: Error = sqlx::Error::PoolClosed.into();
        assert!(matches!(err, Error::Postgres(_)));
        assert!(err.to_string().contains("postgres error"));
    }
}
