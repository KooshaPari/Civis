use civis_cli::census::CensusConfig;
use civis_mcp::{census_sim_status, dispatch_rpc_method};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

fn with_server(frames: Vec<Message>, call: impl FnOnce(&CensusConfig)) {
    with_server_hold(frames, 0, call);
}

fn with_server_hold(frames: Vec<Message>, hold_ms: u64, call: impl FnOnce(&CensusConfig)) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    listener.set_nonblocking(true).unwrap();
    let server = std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async move {
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws = tokio_tungstenite::accept_async(stream).await.unwrap();
            ws.next().await.unwrap().unwrap();
            for frame in frames {
                if ws.send(frame).await.is_err() {
                    break;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(hold_ms)).await;
        });
    });
    call(&CensusConfig {
        host: "127.0.0.1".into(),
        port,
        path: "/ws".into(),
        timeout_ms: 1000,
    });
    server.join().unwrap();
}

#[test]
fn mcp_ignores_pushes_and_unrelated_response_before_matching_id() {
    let frames = vec![
        Message::Binary(vec![0x46, 0x33, 0x44, 0x30]),
        Message::Text(r#"{"jsonrpc":"2.0","method":"scene.reset","params":{}}"#.into()),
        Message::Text(r#"{"jsonrpc":"2.0","id":2,"result":{"tick":999}}"#.into()),
        Message::Text(r#"{"jsonrpc":"2.0","id":1,"result":{"tick":7,"population":4}}"#.into()),
    ];
    with_server(frames.clone(), |config| {
        let result = dispatch_rpc_method(config, "sim.snapshot", serde_json::json!({})).unwrap();
        assert_eq!(result["tick"], 7);
    });
    with_server(frames, |config| {
        let result = census_sim_status(config).unwrap();
        assert_eq!(result.tick, 7);
        assert_eq!(result.population, Some(4));
    });
}

#[test]
fn mcp_rejects_an_unrelated_reply_without_a_matching_response() {
    let frame = Message::Text(r#"{"jsonrpc":"2.0","id":2,"result":{"tick":999}}"#.into());
    with_server(vec![frame.clone()], |config| {
        assert!(dispatch_rpc_method(config, "sim.snapshot", serde_json::json!({})).is_err());
    });
    with_server(vec![frame], |config| {
        assert!(census_sim_status(config).is_err());
    });
}

#[test]
fn mcp_preserves_matching_rpc_errors() {
    let frame = Message::Text(
        r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32603,"message":"test failure"}}"#.into(),
    );
    with_server(vec![frame.clone()], |config| {
        assert!(
            dispatch_rpc_method(config, "sim.snapshot", serde_json::json!({}))
                .unwrap_err()
                .contains("test failure")
        );
    });
    with_server(vec![frame], |config| {
        assert!(census_sim_status(config)
            .unwrap_err()
            .contains("test failure"));
    });
}

#[test]
fn mcp_times_out_when_only_notifications_arrive() {
    let frame = Message::Text(r#"{"jsonrpc":"2.0","method":"scene.reset","params":{}}"#.into());
    with_server_hold(vec![frame], 1200, |config| {
        let mut config = config.clone();
        config.timeout_ms = 100;
        assert!(
            dispatch_rpc_method(&config, "sim.snapshot", serde_json::json!({}))
                .unwrap_err()
                .contains("timed out")
        );
    });
}

#[test]
fn mcp_accepts_matching_json_response_in_binary_frame() {
    let frame = Message::Binary(br#"{"jsonrpc":"2.0","id":1,"result":{"tick":7}}"#.to_vec());
    with_server(vec![frame.clone()], |config| {
        assert_eq!(census_sim_status(config).unwrap().tick, 7);
    });
    with_server(vec![frame], |config| {
        assert_eq!(
            dispatch_rpc_method(config, "sim.snapshot", serde_json::json!({})).unwrap()["tick"],
            7
        );
    });
}

#[test]
fn mcp_times_out_during_a_stalled_handshake() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (_stream, _) = listener.accept().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(400));
    });
    let config = CensusConfig {
        host: "127.0.0.1".into(),
        port,
        path: "/ws".into(),
        timeout_ms: 100,
    };
    assert!(
        dispatch_rpc_method(&config, "sim.snapshot", serde_json::json!({}))
            .unwrap_err()
            .contains("timed out")
    );
    server.join().unwrap();
}

#[test]
fn mcp_rejects_ambiguous_matching_envelope() {
    let frame = Message::Text(
        r#"{"jsonrpc":"2.0","id":1,"result":{"tick":7},"error":{"code":-32603,"message":"bad"}}"#
            .into(),
    );
    with_server(vec![frame], |config| {
        assert!(
            dispatch_rpc_method(config, "sim.snapshot", serde_json::json!({}))
                .unwrap_err()
                .contains("envelope")
        );
    });
}

#[test]
fn mcp_rejects_matching_null_result() {
    let frame = Message::Text(r#"{"jsonrpc":"2.0","id":1,"result":null}"#.into());
    with_server(vec![frame], |config| {
        assert!(dispatch_rpc_method(config, "sim.snapshot", serde_json::json!({})).is_err());
    });
}

#[test]
fn mcp_rejects_invalid_matching_version_and_invalid_text() {
    for frame in [
        Message::Text(r#"{"jsonrpc":"1.0","id":1,"result":{"tick":7}}"#.into()),
        Message::Text("not-json".into()),
    ] {
        with_server(vec![frame], |config| {
            assert!(dispatch_rpc_method(config, "sim.snapshot", serde_json::json!({})).is_err());
        });
    }
}
