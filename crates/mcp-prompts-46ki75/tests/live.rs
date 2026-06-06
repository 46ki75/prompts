//! Live tests — hit the real GitHub Pages distribution.
//!
//! Skipped by default via `#[ignore]`. Run with `just test-live`
//! (or `cargo test -- --ignored`). Failures here may reflect upstream
//! state (network, Pages outage, an empty or not-yet-migrated
//! deployment) rather than this diff, so per the org standards they do
//! not gate PR merges.

use mcp_prompts_46ki75::Server;
use rmcp::model::GetPromptRequestParams;
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
async fn live_list_and_get_first_published_prompt() -> anyhow::Result<()> {
    let server = Server::new()?;
    let (client, server_handle) = spawn(server).await;

    let listed = client.list_prompts(None).await?;
    assert!(
        !listed.prompts.is_empty(),
        "published distribution should contain at least one prompt"
    );

    // Get whichever prompt is listed first — the test must not pin a
    // specific prompt name, only the list → get contract.
    let first = &listed.prompts[0];

    let result = client
        .get_prompt(GetPromptRequestParams::new(first.name.clone()))
        .await?;

    let text = result
        .messages
        .iter()
        .find_map(|message| match &message.content {
            rmcp::model::PromptMessageContent::Text { text } => Some(text.clone()),
            _ => None,
        })
        .expect("text message");
    assert!(
        !text.trim().is_empty(),
        "prompt body should be non-empty markdown"
    );

    client.cancel().await?;
    let _ = server_handle.await;
    Ok(())
}
