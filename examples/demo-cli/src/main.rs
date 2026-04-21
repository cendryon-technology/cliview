use cliview::{StreamTx, WebApp};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

async fn list_users() -> anyhow::Result<Vec<User>> {
    Ok(vec![
        User { id: 1, name: "Alice".into(), email: "alice@example.com".into() },
        User { id: 2, name: "Bob".into(), email: "bob@example.com".into() },
        User { id: 3, name: "Carol".into(), email: "carol@example.com".into() },
    ])
}

#[derive(Serialize)]
struct SystemInfo {
    version: &'static str,
    hostname: String,
}

async fn system_info() -> anyhow::Result<SystemInfo> {
    Ok(SystemInfo {
        version: env!("CARGO_PKG_VERSION"),
        hostname: hostname().unwrap_or_else(|| "unknown".into()),
    })
}

fn hostname() -> Option<String> {
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

#[derive(Deserialize, schemars::JsonSchema)]
struct GreetInput {
    /// Name of the person to greet
    name: String,
    /// Whether to shout
    #[serde(default)]
    loud: bool,
}

#[derive(Serialize)]
struct GreetOutput {
    message: String,
}

async fn greet(input: GreetInput) -> anyhow::Result<GreetOutput> {
    let mut msg = format!("Hello, {}!", input.name);
    if input.loud {
        msg = msg.to_uppercase();
    }
    Ok(GreetOutput { message: msg })
}

async fn tail_ticks(tx: StreamTx) -> anyhow::Result<()> {
    for i in 1..=20 {
        tx.send(format!("tick {i}")).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|s| s.as_str()) != Some("web") {
        eprintln!("usage: demo-cli web [path]");
        std::process::exit(1);
    }

    let bind = std::env::var("CLIVIEW_BIND").unwrap_or_else(|_| "127.0.0.1:0".into());
    let open = std::env::var("CLIVIEW_NO_OPEN").is_err();

    WebApp::new("demo-cli")
        .page("users", "Users", list_users)
        .page("system", "System", system_info)
        .action("greet", "Greet", greet)
        .stream("ticks", "Ticks", tail_ticks)
        .bind(bind)
        .open_browser(open)
        .serve()
        .await
}
