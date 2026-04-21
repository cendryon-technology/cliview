//! Handler traits — type-erased wrappers around async Rust functions so the
//! registry can store heterogeneous handlers behind a single interface.
//!
//! Three flavors:
//! - `PageHandler`  — `async fn() -> Result<T: Serialize>`
//! - `ActionHandler` — `async fn(I: Deserialize) -> Result<O: Serialize>`
//! - `StreamHandler` — `async fn(Stream) -> Result<()>` where Stream emits SSE events

use async_trait::async_trait;
use futures::future::BoxFuture;
use schemars::{schema_for, JsonSchema};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::error::{Error, Result};

// ---------- Page ----------

#[async_trait]
pub trait PageHandler: Send + Sync + 'static {
    async fn call(&self) -> Result<Value>;
}

pub(crate) struct PageFn<F, Fut, T>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = anyhow::Result<T>> + Send + 'static,
    T: Serialize + Send + 'static,
{
    pub f: F,
    pub _phantom: std::marker::PhantomData<fn() -> (Fut, T)>,
}

#[async_trait]
impl<F, Fut, T> PageHandler for PageFn<F, Fut, T>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = anyhow::Result<T>> + Send + 'static,
    T: Serialize + Send + 'static,
{
    async fn call(&self) -> Result<Value> {
        let out = (self.f)().await?;
        Ok(serde_json::to_value(out).map_err(|e| Error::Handler(e.into()))?)
    }
}

// ---------- Action ----------

#[async_trait]
pub trait ActionHandler: Send + Sync + 'static {
    async fn call(&self, input: Value) -> Result<Value>;
    /// JSON Schema for the input type — used by `<AutoForm>` in the SPA.
    fn input_schema(&self) -> Value;
}

pub(crate) struct ActionFn<F, Fut, I, O>
where
    F: Fn(I) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = anyhow::Result<O>> + Send + 'static,
    I: DeserializeOwned + JsonSchema + Send + 'static,
    O: Serialize + Send + 'static,
{
    pub f: F,
    pub _phantom: std::marker::PhantomData<fn(I) -> (Fut, O)>,
}

#[async_trait]
impl<F, Fut, I, O> ActionHandler for ActionFn<F, Fut, I, O>
where
    F: Fn(I) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = anyhow::Result<O>> + Send + 'static,
    I: DeserializeOwned + JsonSchema + Send + 'static,
    O: Serialize + Send + 'static,
{
    async fn call(&self, input: Value) -> Result<Value> {
        let typed: I = serde_json::from_value(input)
            .map_err(|e| Error::BadRequest(e.to_string()))?;
        let out = (self.f)(typed).await?;
        Ok(serde_json::to_value(out).map_err(|e| Error::Handler(e.into()))?)
    }

    fn input_schema(&self) -> Value {
        let schema = schema_for!(I);
        serde_json::to_value(schema).unwrap_or(Value::Null)
    }
}

// ---------- Stream ----------

/// Handle passed to stream handlers; each `send` pushes one SSE `data:` event.
#[derive(Clone)]
pub struct StreamTx {
    tx: mpsc::Sender<Value>,
}

impl StreamTx {
    pub(crate) fn new(tx: mpsc::Sender<Value>) -> Self {
        Self { tx }
    }

    /// Serialize and send a single event. Errors if the client disconnected.
    pub async fn send<T: Serialize>(&self, value: T) -> anyhow::Result<()> {
        let v = serde_json::to_value(value)?;
        self.tx
            .send(v)
            .await
            .map_err(|_| anyhow::anyhow!("stream client disconnected"))
    }
}

pub trait StreamHandler: Send + Sync + 'static {
    fn call(&self, tx: StreamTx) -> BoxFuture<'static, anyhow::Result<()>>;
}

pub(crate) struct StreamFn<F, Fut>
where
    F: Fn(StreamTx) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = anyhow::Result<()>> + Send + 'static,
{
    pub f: Arc<F>,
    pub _phantom: std::marker::PhantomData<fn() -> Fut>,
}

impl<F, Fut> StreamHandler for StreamFn<F, Fut>
where
    F: Fn(StreamTx) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = anyhow::Result<()>> + Send + 'static,
{
    fn call(&self, tx: StreamTx) -> BoxFuture<'static, anyhow::Result<()>> {
        let f = self.f.clone();
        Box::pin(async move { (f)(tx).await })
    }
}
