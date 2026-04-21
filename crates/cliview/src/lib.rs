//! # cliview
//!
//! Turn your Rust CLI into a local web app. Register pages, actions, and
//! streams as typed async functions; `cliview` serves them over REST + SSE
//! with a batteries-included Svelte SPA.
//!
//! ```no_run
//! use cliview::WebApp;
//!
//! async fn list_users() -> anyhow::Result<Vec<String>> {
//!     Ok(vec!["alice".into(), "bob".into()])
//! }
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     WebApp::new("mycli")
//!         .page("users", "Users", list_users)
//!         .bind("127.0.0.1:0")
//!         .open_browser(true)
//!         .serve()
//!         .await
//! }
//! ```

mod assets;
mod error;
mod handler;
mod registry;
mod server;

pub use error::{Error, Result};
pub use handler::StreamTx;

use std::future::Future;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};

use crate::handler::{ActionFn, PageFn, StreamFn};
use crate::registry::{ActionEntry, PageEntry, Registry, StreamEntry};

/// Builder for a local web app backed by your CLI.
pub struct WebApp {
    registry: Registry,
    bind: String,
    open_browser: bool,
    frontend_dir: Option<PathBuf>,
    dev_proxy: Option<String>,
}

impl WebApp {
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            registry: Registry::new(app_name),
            bind: "127.0.0.1:0".into(),
            open_browser: false,
            frontend_dir: None,
            dev_proxy: std::env::var("CLIVIEW_DEV").ok(),
        }
    }

    /// Forward non-API requests to this URL (e.g. `http://127.0.0.1:5173`).
    /// Overrides both `frontend_dir` and the embedded SPA. Automatically
    /// set from `CLIVIEW_DEV` env var.
    pub fn dev_proxy(mut self, url: impl Into<String>) -> Self {
        self.dev_proxy = Some(url.into());
        self
    }

    /// Address to bind to. Use port `0` to let the OS pick a free port.
    pub fn bind(mut self, addr: impl Into<String>) -> Self {
        self.bind = addr.into();
        self
    }

    /// Open the system default browser once the server is ready.
    pub fn open_browser(mut self, yes: bool) -> Self {
        self.open_browser = yes;
        self
    }

    /// Override the embedded SPA with assets from a directory on disk.
    pub fn frontend_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.frontend_dir = Some(path.into());
        self
    }

    /// Register a read-only page. The handler is called on GET `/api/pages/:id`.
    pub fn page<F, Fut, T>(mut self, id: impl Into<String>, title: impl Into<String>, f: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<T>> + Send + 'static,
        T: Serialize + Send + 'static,
    {
        let entry = PageEntry {
            title: title.into(),
            handler: Arc::new(PageFn { f, _phantom: PhantomData }),
        };
        self.registry.pages.insert(id.into(), entry);
        self
    }

    /// Register an action. Called on POST `/api/actions/:id` with a JSON body
    /// matching the handler's input type.
    pub fn action<F, Fut, I, O>(
        mut self,
        id: impl Into<String>,
        title: impl Into<String>,
        f: F,
    ) -> Self
    where
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<O>> + Send + 'static,
        I: DeserializeOwned + JsonSchema + Send + 'static,
        O: Serialize + Send + 'static,
    {
        let entry = ActionEntry {
            title: title.into(),
            handler: Arc::new(ActionFn { f, _phantom: PhantomData }),
        };
        self.registry.actions.insert(id.into(), entry);
        self
    }

    /// Register a streaming handler. Called on GET `/api/streams/:id`; each
    /// `tx.send(...)` emits one SSE event.
    pub fn stream<F, Fut>(mut self, id: impl Into<String>, title: impl Into<String>, f: F) -> Self
    where
        F: Fn(StreamTx) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        let entry = StreamEntry {
            title: title.into(),
            handler: Arc::new(StreamFn {
                f: Arc::new(f),
                _phantom: PhantomData,
            }),
        };
        self.registry.streams.insert(id.into(), entry);
        self
    }

    /// Bind, start serving, and block until the server stops.
    pub async fn serve(self) -> anyhow::Result<()> {
        let state = server::AppState {
            registry: Arc::new(self.registry),
            frontend_dir: self.frontend_dir,
            dev_proxy: self.dev_proxy,
        };
        let app = server::router(state);

        let addr: SocketAddr = self.bind.parse()?;
        let listener = tokio::net::TcpListener::bind(addr).await?;
        let local = listener.local_addr()?;
        let url = format!("http://{local}");
        tracing::info!("cliview serving on {url}");
        eprintln!("cliview: serving on {url}");

        if self.open_browser {
            let _ = open::that(&url);
        }

        axum::serve(listener, app).await?;
        Ok(())
    }
}
