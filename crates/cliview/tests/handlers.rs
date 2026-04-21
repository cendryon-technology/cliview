//! Smoke tests for page / action / stream handler wiring. We exercise the
//! public builder API and reach into the server by starting it on an
//! ephemeral port, then hit it with reqwest.

use cliview::{StreamTx, WebApp};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize)]
struct Row {
    id: u64,
    name: String,
}

async fn list_rows() -> anyhow::Result<Vec<Row>> {
    Ok(vec![
        Row { id: 1, name: "a".into() },
        Row { id: 2, name: "b".into() },
    ])
}

#[derive(Deserialize, schemars::JsonSchema)]
struct EchoIn {
    message: String,
}

#[derive(Serialize)]
struct EchoOut {
    echoed: String,
}

async fn echo(i: EchoIn) -> anyhow::Result<EchoOut> {
    Ok(EchoOut { echoed: i.message })
}

async fn ticks(tx: StreamTx) -> anyhow::Result<()> {
    for i in 0..3u32 {
        tx.send(i).await?;
    }
    Ok(())
}

/// Start a test server bound to 127.0.0.1:0 on a background task and return
/// the base URL once it's listening.
async fn start_server() -> String {
    use tokio::net::TcpListener;

    // Bind first so we know the real port, then hand the listener to axum
    // via a custom serve loop. Easiest path: use the public API which binds
    // internally — but we need the port. Workaround: bind a probe to find a
    // free port, drop it, then WebApp binds to that port. Race-prone but
    // acceptable for tests.
    let probe = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = probe.local_addr().unwrap().port();
    drop(probe);

    let addr = format!("127.0.0.1:{port}");
    let url = format!("http://{addr}");

    tokio::spawn(async move {
        WebApp::new("test-app")
            .page("rows", "Rows", list_rows)
            .action("echo", "Echo", echo)
            .stream("ticks", "Ticks", ticks)
            .bind(addr)
            .serve()
            .await
            .unwrap();
    });

    // Wait for the server to become reachable.
    for _ in 0..50 {
        if reqwest::get(format!("{url}/api/meta")).await.is_ok() {
            return url;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("server did not start in time");
}

#[tokio::test]
async fn meta_lists_registered_handlers() {
    let url = start_server().await;
    let meta: serde_json::Value = reqwest::get(format!("{url}/api/meta"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(meta["name"], "test-app");
    assert_eq!(meta["pages"][0]["id"], "rows");
    assert_eq!(meta["actions"][0]["id"], "echo");
    assert_eq!(meta["streams"][0]["id"], "ticks");
    // schemars JSON Schema should expose the "message" field
    let schema = &meta["actions"][0]["input_schema"];
    assert!(schema.to_string().contains("message"));
}

#[tokio::test]
async fn page_returns_handler_output() {
    let url = start_server().await;
    let rows: serde_json::Value = reqwest::get(format!("{url}/api/pages/rows"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(rows[0]["id"], 1);
    assert_eq!(rows[1]["name"], "b");
}

#[tokio::test]
async fn action_roundtrips_typed_input() {
    let url = start_server().await;
    let client = reqwest::Client::new();
    let body = serde_json::json!({ "message": "hi" });
    let out: serde_json::Value = client
        .post(format!("{url}/api/actions/echo"))
        .json(&body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(out["echoed"], "hi");
}

#[tokio::test]
async fn action_bad_input_returns_400() {
    let url = start_server().await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{url}/api/actions/echo"))
        .json(&serde_json::json!({ "wrong": 1 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn unknown_page_returns_404() {
    let url = start_server().await;
    let resp = reqwest::get(format!("{url}/api/pages/nope")).await.unwrap();
    assert_eq!(resp.status(), 404);
}
