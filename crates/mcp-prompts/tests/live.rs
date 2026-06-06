//! Live tests — hit the real GitHub Pages distribution.
//!
//! Skipped by default via `#[ignore]`. Run with `just test-live`
//! (or `cargo test -- --ignored`). Failures here may reflect upstream
//! state (network, Pages outage, an empty deployment) rather than this
//! diff, so per the org standards they do not gate PR merges.

use mcp_prompts::Server;
use rmcp::model::ReadResourceRequestParams;
use rmcp::{ClientHandler, ServiceExt};

#[derive(Default, Clone)]
struct TestClient;

impl ClientHandler for TestClient {}

async fn spawn(
    server: Server,
) -> (
    rmcp::service::RunningService<rmcp::RoleClient, TestClient>,
    tokio::task::JoinHandle<anyhow::Result<()>>,
) {
    let (server_io, client_io) = tokio::io::duplex(64 * 1024);

    let server_handle = tokio::spawn(async move {
        let svc = server.serve(server_io).await?;
        svc.waiting().await?;
        anyhow::Ok(())
    });

    let client = TestClient
        .serve(client_io)
        .await
        .expect("client failed to connect");

    (client, server_handle)
}

#[tokio::test]
#[ignore = "live: hits the real GitHub Pages distribution"]
async fn live_list_and_read_first_published_prompt() -> anyhow::Result<()> {
    let server = Server::new()?;
    let (client, server_handle) = spawn(server).await;

    let listed = client.list_resources(None).await?;
    assert!(
        !listed.resources.is_empty(),
        "published distribution should contain at least one prompt"
    );

    // Read whichever prompt is listed first — the test must not pin a
    // specific prompt name, only the list → read contract.
    let first = &listed.resources[0].raw;
    assert!(
        first.uri.starts_with("prompts://"),
        "unexpected URI shape: {}",
        first.uri
    );

    let result = client
        .read_resource(ReadResourceRequestParams::new(first.uri.clone()))
        .await?;

    let text = result
        .contents
        .iter()
        .find_map(|c| match c {
            rmcp::model::ResourceContents::TextResourceContents { text, .. } => Some(text.clone()),
            _ => None,
        })
        .expect("text contents");
    assert!(
        !text.trim().is_empty(),
        "prompt body should be non-empty markdown"
    );

    client.cancel().await?;
    let _ = server_handle.await;
    Ok(())
}
