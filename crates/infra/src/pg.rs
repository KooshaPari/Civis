use crate::InfraError;
use sqlx::{postgres::PgPoolOptions, PgPool};

/// Resolve and validate a PostgreSQL URL from an optional environment value.
///
/// Used by services that read `DATABASE_URL` before opening a pool.
pub fn resolve_database_url(from_env: Option<&str>) -> Result<String, InfraError> {
    let url = from_env
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| InfraError::MissingConfig("DATABASE_URL".into()))?;
    validate_postgres_url(url)?;
    Ok(url.to_owned())
}

/// Validate a PostgreSQL connection URL before connecting.
pub fn validate_postgres_url(url: &str) -> Result<(), InfraError> {
    let url = url.trim();
    if url.is_empty() {
        return Err(InfraError::MissingConfig("DATABASE_URL is empty".into()));
    }

    let scheme_end = url.find("://").ok_or_else(|| {
        InfraError::MissingConfig(format!("invalid DATABASE_URL (missing scheme): {url}"))
    })?;
    let scheme = &url[..scheme_end];
    if scheme != "postgres" && scheme != "postgresql" {
        return Err(InfraError::MissingConfig(format!(
            "DATABASE_URL scheme must be postgres or postgresql, got {scheme}"
        )));
    }

    let after_scheme = &url[scheme_end + 3..];
    if after_scheme.is_empty() || after_scheme.starts_with('/') {
        return Err(InfraError::MissingConfig(format!(
            "invalid DATABASE_URL (missing host): {url}"
        )));
    }

    Ok(())
}

/// SQL used by replay persistence (stable for offline checks and unit tests).
pub(crate) mod replay_sql {
    /// Insert a replay row and return its id.
    pub const INSERT: &str = r#"insert into replays (name, blob) values ($1, $2) returning id"#;
    /// Load replay bytes by row id.
    pub const SELECT_BLOB_BY_ID: &str = r#"select blob from replays where id = $1"#;
}

/// Relative path to the initial replays migration from the crate root.
pub const MIGRATION_0001: &str = "migrations/0001_create_replays.sql";

/// PostgreSQL connection wrapper.
pub struct PgConn {
    pool: PgPool,
}

impl PgConn {
    /// Connect to a PostgreSQL database.
    pub async fn connect(url: &str) -> Result<Self, InfraError> {
        validate_postgres_url(url)?;
        let pool = PgPoolOptions::new().connect(url).await?;
        Ok(Self { pool })
    }

    /// Save a replay blob and return its row id.
    pub async fn save_replay(&self, name: &str, blob: &[u8]) -> Result<i64, InfraError> {
        let row: (i64,) = sqlx::query_as(replay_sql::INSERT)
            .bind(name)
            .bind(blob)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    /// Load a replay blob by row id.
    pub async fn load_replay(&self, id: i64) -> Result<Vec<u8>, InfraError> {
        let row: (Vec<u8>,) = sqlx::query_as(replay_sql::SELECT_BLOB_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{replay_sql, resolve_database_url, validate_postgres_url, MIGRATION_0001};
    use crate::InfraError;
    use std::path::Path;

    #[test]
    fn migration_path_is_relative_to_crate_root() {
        assert_eq!(MIGRATION_0001, "migrations/0001_create_replays.sql");
    }

    #[test]
    fn validate_postgres_url_accepts_postgres_and_postgresql_schemes() {
        for url in [
            "postgres://user:pass@localhost:5432/db",
            "postgresql://user@127.0.0.1/db",
            "postgres://host/db?sslmode=disable",
        ] {
            validate_postgres_url(url).expect(url);
        }
    }

    #[test]
    fn validate_postgres_url_rejects_empty_and_bad_schemes() {
        let empty = validate_postgres_url("").unwrap_err();
        assert!(matches!(empty, InfraError::MissingConfig(_)));
        assert!(empty.to_string().contains("empty"));

        let mysql = validate_postgres_url("mysql://localhost/db").unwrap_err();
        assert!(mysql.to_string().contains("postgres or postgresql"));

        let missing_scheme = validate_postgres_url("localhost/db").unwrap_err();
        assert!(missing_scheme.to_string().contains("missing scheme"));
    }

    #[test]
    fn validate_postgres_url_rejects_missing_host() {
        let err = validate_postgres_url("postgres:///db").unwrap_err();
        assert!(err.to_string().contains("missing host"));

        let err = validate_postgres_url("postgresql://").unwrap_err();
        assert!(err.to_string().contains("missing host"));
    }

    #[test]
    fn resolve_database_url_requires_non_empty_value() {
        let err = resolve_database_url(None).unwrap_err();
        assert!(matches!(err, InfraError::MissingConfig(_)));
        assert!(err.to_string().contains("DATABASE_URL"));

        let err = resolve_database_url(Some("   ")).unwrap_err();
        assert!(err.to_string().contains("DATABASE_URL"));
    }

    #[test]
    fn resolve_database_url_trims_and_validates() {
        let url = resolve_database_url(Some("  postgres://localhost/db  ")).expect("resolve");
        assert_eq!(url, "postgres://localhost/db");
    }

    #[test]
    fn save_replay_sql_uses_parameterized_insert() {
        let sql = replay_sql::INSERT;
        assert!(sql.contains("insert into replays"));
        assert!(sql.contains("(name, blob)"));
        assert!(sql.contains("values ($1, $2)"));
        assert!(sql.contains("returning id"));
        assert!(
            !sql.contains(';'),
            "query_as should not include a trailing semicolon"
        );
    }

    #[test]
    fn load_replay_sql_selects_blob_by_id() {
        let sql = replay_sql::SELECT_BLOB_BY_ID;
        assert_eq!(sql, "select blob from replays where id = $1");
        assert!(!sql.contains(';'));
    }

    #[test]
    fn migration_file_exists() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(MIGRATION_0001);
        assert!(path.is_file(), "expected migration at {}", path.display());
    }

    #[test]
    fn migration_defines_replays_table() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(MIGRATION_0001);
        let sql = std::fs::read_to_string(&path).expect("read migration");
        let normalized: String = sql
            .split_whitespace()
            .map(str::to_ascii_lowercase)
            .collect::<Vec<_>>()
            .join(" ");
        assert!(normalized.contains("create table"));
        assert!(normalized.contains("replays"));
        assert!(normalized.contains("id bigserial primary key"));
        assert!(normalized.contains("name text not null"));
        assert!(normalized.contains("created_at timestamptz not null"));
        assert!(normalized.contains("blob bytea not null"));
    }

    #[test]
    fn replay_sql_has_no_string_interpolation() {
        for sql in [replay_sql::INSERT, replay_sql::SELECT_BLOB_BY_ID] {
            assert!(
                !sql.contains('\''),
                "SQL must use bind params, not literals: {sql}"
            );
            assert!(
                !sql.contains('"'),
                "SQL must use bind params, not literals: {sql}"
            );
        }
    }

    #[tokio::test]
    async fn connect_rejects_invalid_url_before_pool() {
        let err = match super::PgConn::connect("not-a-url").await {
            Err(err) => err,
            Ok(_) => panic!("expected invalid URL to fail"),
        };
        assert!(err.to_string().contains("missing scheme"));

        let err = match super::PgConn::connect("mysql://localhost/db").await {
            Err(err) => err,
            Ok(_) => panic!("expected bad scheme to fail"),
        };
        assert!(err.to_string().contains("postgres or postgresql"));
    }
}
