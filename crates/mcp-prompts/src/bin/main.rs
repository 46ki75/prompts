//! Unified entry point for the MCP server.
//!
//! Subcommands select the transport:
//! - `stdio` — line-buffered JSON-RPC over stdin/stdout
//! - `http`  — streamable HTTP, mounted at `/mcp`
//!
//! The shared `--base-url` option lives at the top level and accepts
//! an env-var fallback so MCP hosts can inject it without rewriting
//! argv.

use clap::{Args, Parser, Subcommand};
use mcp_prompts::{PROMPTS_BASE_URL, Server};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::EnvFilter;

const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1:8000";

#[derive(Debug, Parser)]
#[command(name = "mcp-prompts", version, about, long_about = None)]
struct Cli {
    /// Base URL of the static prompt distribution. Useful for pointing
    /// at a local preview of `dist/` or a wiremock fixture.
    #[arg(
        long,
        env = "MCP_PROMPTS_BASE_URL",
        default_value = PROMPTS_BASE_URL,
        global = true,
    )]
    base_url: String,

    #[command(subcommand)]
    transport: Transport,
}

#[derive(Debug, Subcommand)]
enum Transport {
    /// Serve MCP over stdin/stdout (the transport an MCP host launches
    /// the binary with directly).
    Stdio,

    /// Serve MCP over streamable HTTP at `/mcp`.
    Http(HttpArgs),
}

#[derive(Debug, Args)]
struct HttpArgs {
    /// TCP address to bind the HTTP listener to.
    #[arg(
        long,
        env = "MCP_BIND_ADDRESS",
        default_value = DEFAULT_BIND_ADDRESS,
    )]
    bind: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.transport {
        Transport::Stdio => run_stdio(&cli.base_url).await,
        Transport::Http(args) => run_http(&cli.base_url, &args.bind).await,
    }
}

/// Initialize tracing for the stdio transport.
///
/// stdio servers MUST NOT write anything except JSON-RPC to stdout —
/// the MCP host is parsing every byte. So tracing goes to stderr only,
/// with ANSI escape codes stripped (host log viewers usually render
/// raw text).
fn init_stdio_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();
}

fn init_http_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}

async fn run_stdio(base_url: &str) -> anyhow::Result<()> {
    init_stdio_tracing();

    tracing::info!(%base_url, "starting mcp-prompts over stdio");

    let server = Server::builder().base_url(base_url.to_string()).build()?;

    let service = server.serve(stdio()).await.inspect_err(|err| {
        tracing::error!(error = ?err, "failed to start MCP server");
    })?;

    service.waiting().await?;
    Ok(())
}

async fn run_http(base_url: &str, bind_address: &str) -> anyhow::Result<()> {
    init_http_tracing();

    let cancellation = tokio_util::sync::CancellationToken::new();

    // Build the Server (and its `reqwest::Client`) ONCE, then clone
    // into the per-session factory closure. Server is cheap to clone
    // (the use case is behind an `Arc`), so every session reuses the
    // same HTTP client and connection pool.
    let server_template = Server::builder().base_url(base_url.to_string()).build()?;

    let service = StreamableHttpService::new(
        move || Ok(server_template.clone()),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default().with_cancellation_token(cancellation.child_token()),
    );

    let router = axum::Router::new().nest_service("/mcp", service);
    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    tracing::info!(%bind_address, %base_url, "mcp-prompts listening at /mcp");

    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
            tracing::info!("received Ctrl+C, shutting down");
            cancellation.cancel();
        })
        .await?;

    Ok(())
}
