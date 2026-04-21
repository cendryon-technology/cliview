use axum::extract::{Path, State};
use axum::http::Uri;
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures::stream::{Stream, StreamExt};
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tower_http::cors::CorsLayer;

use crate::assets;
use crate::error::{Error, Result};
use crate::handler::StreamTx;
use crate::registry::Registry;

#[derive(Clone)]
pub(crate) struct AppState {
    pub registry: Arc<Registry>,
    pub frontend_dir: Option<PathBuf>,
    /// Dev-mode: forward non-API requests to this URL (e.g. http://127.0.0.1:5173)
    pub dev_proxy: Option<String>,
}

pub(crate) fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/meta", get(meta))
        .route("/api/pages/:id", get(page))
        .route("/api/actions/:id", post(action))
        .route("/api/streams/:id", get(stream))
        .fallback(static_fallback)
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn meta(State(s): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::to_value(s.registry.meta()).unwrap())
}

async fn page(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let entry = s
        .registry
        .pages
        .get(&id)
        .ok_or_else(|| Error::NotFound(format!("page '{id}'")))?;
    Ok(Json(entry.handler.call().await?))
}

async fn action(
    State(s): State<AppState>,
    Path(id): Path<String>,
    body: Option<Json<serde_json::Value>>,
) -> Result<Json<serde_json::Value>> {
    let entry = s
        .registry
        .actions
        .get(&id)
        .ok_or_else(|| Error::NotFound(format!("action '{id}'")))?;
    let input = body.map(|Json(v)| v).unwrap_or(serde_json::Value::Null);
    Ok(Json(entry.handler.call(input).await?))
}

async fn stream(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Sse<impl Stream<Item = std::result::Result<Event, Infallible>>>> {
    let entry = s
        .registry
        .streams
        .get(&id)
        .ok_or_else(|| Error::NotFound(format!("stream '{id}'")))?
        .handler
        .clone();

    let (tx, rx) = mpsc::channel::<serde_json::Value>(64);
    let stream_tx = StreamTx::new(tx);

    tokio::spawn(async move {
        if let Err(e) = entry.call(stream_tx).await {
            tracing::warn!("stream handler error: {e:#}");
        }
    });

    let sse_stream = ReceiverStream::new(rx).map(|v| {
        Ok::<_, Infallible>(
            Event::default().data(serde_json::to_string(&v).unwrap_or_default()),
        )
    });

    Ok(Sse::new(sse_stream))
}

async fn static_fallback(State(s): State<AppState>, uri: Uri) -> Response {
    if uri.path().starts_with("/api/") {
        return (axum::http::StatusCode::NOT_FOUND, "not found").into_response();
    }
    if let Some(target) = &s.dev_proxy {
        return proxy_to(target, &uri).await;
    }
    if let Some(dir) = &s.frontend_dir {
        assets::serve_from_dir(dir, &uri)
    } else {
        assets::serve_embedded(&uri)
    }
}

async fn proxy_to(base: &str, uri: &Uri) -> Response {
    let url = format!("{}{}", base.trim_end_matches('/'), uri.path());
    match reqwest::get(&url).await {
        Ok(resp) => {
            let status = resp.status();
            let headers = resp.headers().clone();
            match resp.bytes().await {
                Ok(body) => {
                    let mut builder = Response::builder().status(status);
                    if let Some(h) = builder.headers_mut() {
                        for (k, v) in headers.iter() {
                            if k.as_str().to_ascii_lowercase() != "transfer-encoding" {
                                h.insert(k.clone(), v.clone());
                            }
                        }
                    }
                    builder
                        .body(axum::body::Body::from(body))
                        .unwrap_or_else(|_| {
                            (
                                axum::http::StatusCode::BAD_GATEWAY,
                                "bad response from dev server",
                            )
                                .into_response()
                        })
                }
                Err(e) => (
                    axum::http::StatusCode::BAD_GATEWAY,
                    format!("dev proxy body error: {e}"),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            axum::http::StatusCode::BAD_GATEWAY,
            format!("dev proxy error ({url}): {e}"),
        )
            .into_response(),
    }
}

