//! SSE stream and read-only snapshot/terrain handlers.

use std::convert::Infallible;

use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json, Response,
    },
};
use futures::{stream::Stream, StreamExt};
use tokio_stream::wrappers::BroadcastStream;
use tracing::warn;

use crate::app::{AppState, Snapshot};

pub(crate) async fn snapshot_handler(State(state): State<AppState>) -> Json<Option<Snapshot>> {
    Json(state.latest.read().await.clone())
}

pub(crate) async fn terrain_handler(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let cache = &state.terrain_cache;
    if headers
        .get(header::IF_NONE_MATCH)
        .is_some_and(|value| value == cache.etag)
    {
        return (
            StatusCode::NOT_MODIFIED,
            [(header::ETAG, cache.etag.clone())],
        )
            .into_response();
    }

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ETAG, cache.etag.clone())
        .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
        .body(Body::from(cache.body.clone()))
        .expect("terrain response")
}

pub(crate) async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|item| async move {
        match item {
            Ok(snapshot) => match serde_json::to_string(&snapshot) {
                Ok(json) => Some(Ok(Event::default().event("snapshot").data(json))),
                Err(err) => {
                    warn!(?err, "failed to serialize snapshot");
                    None
                }
            },
            Err(err) => {
                warn!(?err, "snapshot stream closed");
                None
            }
        }
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}
