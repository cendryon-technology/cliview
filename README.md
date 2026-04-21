# cliview

Turn your Rust CLI into a local web app with one line.

```rust
use cliview::WebApp;

async fn list_users() -> anyhow::Result<Vec<String>> {
    Ok(vec!["alice".into(), "bob".into()])
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    WebApp::new("mycli")
        .page("users", "Users", list_users)
        .bind("127.0.0.1:0")
        .open_browser(true)
        .serve()
        .await
}
```

Then:

```
mycli web .
```

…spins up a localhost server with a browser UI that lists your pages,
invokes your actions, and streams long-running output.

## Workspace layout

```
.
├── crates/cliview/        # the library crate (published to crates.io)
│   └── frontend/dist/     # prebuilt SPA, embedded via rust-embed
├── frontend/              # SvelteKit source (built with `just build-ui`)
└── examples/demo-cli/     # end-to-end example
```

## Concepts

- **Page**  (`GET /api/pages/:id`)  — read-only view; handler returns JSON
- **Action** (`POST /api/actions/:id`) — mutation; typed JSON input → JSON output
- **Stream** (`GET /api/streams/:id`) — SSE stream for long-running output

## Try the demo

```
cargo run -p demo-cli -- web .
```

## Commands

```
mise run check       # cargo check
mise run demo        # run the demo CLI
mise run build-ui    # rebuild the Svelte SPA into crates/cliview/frontend/dist
mise run test        # run all tests
```

## Status

Early scaffold. The embedded SPA is currently a placeholder that renders
`/api/meta`; the real SvelteKit frontend lives in `frontend/` and gets built
into `crates/cliview/frontend/dist/` via `just build-ui`.
