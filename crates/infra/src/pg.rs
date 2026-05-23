use crate::InfraError;
use sqlx::{postgres::PgPoolOptions, PgPool};

/// PostgreSQL connection wrapper.
pub struct PgConn {
    pool: PgPool,
}

impl PgConn {
    /// Connect to a PostgreSQL database.
    pub async fn connect(url: &str) -> Result<Self, InfraError> {
        let pool = PgPoolOptions::new().connect(url).await?;
        Ok(Self { pool })
    }

    /// Save a replay blob and return its row id.
    pub async fn save_replay(&self, name: &str, blob: &[u8]) -> Result<i64, InfraError> {
        let row: (i64,) =
            sqlx::query_as(r#"insert into replays (name, blob) values ($1, $2) returning id"#)
                .bind(name)
                .bind(blob)
                .fetch_one(&self.pool)
                .await?;
        Ok(row.0)
    }

    /// Load a replay blob by row id.
    pub async fn load_replay(&self, id: i64) -> Result<Vec<u8>, InfraError> {
        let row: (Vec<u8>,) = sqlx::query_as(r#"select blob from replays where id = $1"#)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }
}
