//! Embedded default SPA. The `frontend/dist/` directory is committed to the
//! crate so downstream users need no Node toolchain.

use axum::body::Body;
use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "frontend/dist/"]
struct EmbeddedAssets;

pub(crate) fn serve_embedded(uri: &Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match EmbeddedAssets::get(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(file.data.into_owned()))
                .unwrap()
        }
        None => {
            // SPA fallback: serve index.html for unknown routes
            match EmbeddedAssets::get("index.html") {
                Some(file) => Response::builder()
                    .header(header::CONTENT_TYPE, "text/html")
                    .body(Body::from(file.data.into_owned()))
                    .unwrap(),
                None => (StatusCode::NOT_FOUND, "frontend assets missing").into_response(),
            }
        }
    }
}

pub(crate) fn serve_from_dir(root: &std::path::Path, uri: &Uri) -> Response {
    let rel = uri.path().trim_start_matches('/');
    let rel = if rel.is_empty() { "index.html" } else { rel };
    let candidate = root.join(rel);

    let final_path = if candidate.is_file() {
        candidate
    } else {
        root.join("index.html")
    };

    match std::fs::read(&final_path) {
        Ok(bytes) => {
            let mime = mime_guess::from_path(&final_path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(bytes))
                .unwrap()
        }
        Err(_) => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}
