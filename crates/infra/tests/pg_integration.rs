//! Round-trip replay persistence against a real PostgreSQL instance (Docker required).
#![cfg(feature = "pg")]

use civ_infra::pg::PgConn;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

#[tokio::test]
#[ignore = "requires Docker and a running container runtime"]
async fn save_replay_load_replay_roundtrip() {
    let container = Postgres::default().start().await.expect("start postgres");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("postgres port");
    let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");

    let pool = sqlx::PgPool::connect(&url)
        .await
        .expect("connect for migrations");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("apply migrations");
    pool.close().await;

    let conn = PgConn::connect(&url).await.expect("connect");
    let blob = b"\xde\xad\xbe\xef replay bytes";
    let id = conn
        .save_replay("integration-test", blob)
        .await
        .expect("save_replay");
    assert!(id > 0);

    let loaded = conn.load_replay(id).await.expect("load_replay");
    assert_eq!(loaded.as_slice(), blob);
}
