//! Local error type for civis-mcp.

/// Crate-local error type.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Filesystem or process I/O failed.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON serialization or deserialization failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    /// A Tokio task failed to join.
    #[error("task join error: {0}")]
    Join(#[from] tokio::task::JoinError),
}

#[cfg(test)]
mod tests {
    use super::Error;

    fn assert_std_error(err: &dyn std::error::Error) -> String {
        err.to_string()
    }

    #[test]
    fn converts_io_and_json_errors() {
        let io_err: Error = std::io::Error::new(std::io::ErrorKind::Other, "disk full").into();
        assert!(assert_std_error(&io_err).contains("disk full"));

        let json_err: Error = serde_json::from_str::<serde_json::Value>("{")
            .unwrap_err()
            .into();
        assert!(assert_std_error(&json_err).contains("json error"));
    }

    #[test]
    fn converts_tokio_join_error() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let join_err = runtime.block_on(async {
            tokio::spawn(async { panic!("join failure") })
                .await
                .unwrap_err()
        });
        let err: Error = join_err.into();
        assert!(assert_std_error(&err).contains("task join error"));
    }
}
