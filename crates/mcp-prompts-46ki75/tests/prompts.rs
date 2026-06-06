//! Hermetic integration tests — drive the server through an in-process
//! MCP client (`tokio::io::duplex`) against a wiremock static host.

use mcp_prompts_46ki75::Server;
use rmcp::model::{GetPromptRequestParams, PromptMessage, PromptMessageContent};
use rmcp::{ClientHandler, ServiceExt};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

/// Mount the standard two-prompt fixture on `mock`. The markdown files
/// mirror the builder's output: frontmatter already stripped, metadata
/// carried by `list.json`.
async fn mount_fixture(mock: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/prompts/list.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "name": "alpha",
                "title": "Alpha Title",
                "description": "Alpha description.",
                "arguments": ["topic", "audience"],
                "path": "alpha.md",
            },
            { "name": "beta", "title": "Beta Title", "path": "beta.md" },
        ])))
        .mount(mock)
        .await;
    Mock::given(method("GET"))
        .and(path("/prompts/alpha.md"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string("# Alpha Title\n\nResearch $1 for $2.\n"),
        )
        .mount(mock)
        .await;
    Mock::given(method("GET"))
        .and(path("/prompts/beta.md"))
        .respond_with(ResponseTemplate::new(200).set_body_string("# Beta Title\n\nbeta body\n"))
        .mount(mock)
        .await;
}

fn server_for(mock: &MockServer) -> Server {
    Server::builder()
        .base_url(mock.uri())
        .build()
        .expect("server should build")
}

fn first_text(messages: &[PromptMessage]) -> Option<&str> {
    messages.iter().find_map(|message| match &message.content {
        PromptMessageContent::Text { text } => Some(text.as_str()),
        _ => None,
    })
}

#[tokio::test]
async fn server_advertises_prompts_capability() -> anyhow::Result<()> {
    let mock = MockServer::start().await;
    let (client, server_handle) = spawn(server_for(&mock)).await;

    let info = client.peer_info().expect("server info after initialize");
    assert!(
        info.capabilities.prompts.is_some(),
        "prompts capability missing: {info:?}"
    );

    client.cancel().await?;
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
async fn list_prompts_projects_list_json_entries() -> anyhow::Result<()> {
    let mock = MockServer::start().await;
    mount_fixture(&mock).await;
    let (client, server_handle) = spawn(server_for(&mock)).await;

    let listed = client.list_prompts(None).await?;

    assert_eq!(
        listed.prompts.len(),
        2,
        "expected both fixture prompts: {:?}",
        listed.prompts
    );

    let alpha = listed
        .prompts
        .iter()
        .find(|p| p.name == "alpha")
        .expect("alpha entry");
    assert_eq!(alpha.title.as_deref(), Some("Alpha Title"));
    assert_eq!(alpha.description.as_deref(), Some("Alpha description."));
    let arguments = alpha.arguments.as_deref().expect("alpha arguments");
    let names: Vec<&str> = arguments.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, ["topic", "audience"]);

    let beta = listed
        .prompts
        .iter()
        .find(|p| p.name == "beta")
        .expect("beta entry");
    assert_eq!(beta.title.as_deref(), Some("Beta Title"));
    assert_eq!(beta.description, None);
    assert_eq!(beta.arguments, None);

    client.cancel().await?;
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
async fn get_prompt_substitutes_arguments() -> anyhow::Result<()> {
    let mock = MockServer::start().await;
    mount_fixture(&mock).await;
    let (client, server_handle) = spawn(server_for(&mock)).await;

    let arguments = json!({ "topic": "rust", "audience": "beginners" })
        .as_object()
        .cloned()
        .expect("arguments object");
    let result = client
        .get_prompt(GetPromptRequestParams::new("alpha").with_arguments(arguments))
        .await?;

    assert_eq!(result.description.as_deref(), Some("Alpha description."));
    let text = first_text(&result.messages).expect("text message");
    assert_eq!(text, "# Alpha Title\n\nResearch rust for beginners.\n");

    client.cancel().await?;
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
async fn get_prompt_without_arguments_returns_body() -> anyhow::Result<()> {
    let mock = MockServer::start().await;
    mount_fixture(&mock).await;
    let (client, server_handle) = spawn(server_for(&mock)).await;

    let result = client
        .get_prompt(GetPromptRequestParams::new("beta"))
        .await?;

    assert_eq!(result.description, None);
    let text = first_text(&result.messages).expect("text message");
    assert_eq!(text, "# Beta Title\n\nbeta body\n");

    client.cancel().await?;
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
async fn get_prompt_rejects_unlisted_names() -> anyhow::Result<()> {
    let mock = MockServer::start().await;
    mount_fixture(&mock).await;
    // `gamma.md` exists upstream but is NOT in list.json — it must
    // stay unreachable, proving gets resolve through the index.
    Mock::given(method("GET"))
        .and(path("/prompts/gamma.md"))
        .respond_with(ResponseTemplate::new(200).set_body_string("unlisted"))
        .mount(&mock)
        .await;
    let (client, server_handle) = spawn(server_for(&mock)).await;

    let err = client
        .get_prompt(GetPromptRequestParams::new("gamma"))
        .await
        .expect_err("unlisted prompt must not be readable");

    let message = err.to_string();
    assert!(
        message.contains("prompt not found"),
        "expected invalid-params message, got: {message}"
    );

    client.cancel().await?;
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
async fn upstream_failure_surfaces_as_internal_error() -> anyhow::Result<()> {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/prompts/list.json"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock)
        .await;
    let (client, server_handle) = spawn(server_for(&mock)).await;

    let err = client
        .list_prompts(None)
        .await
        .expect_err("upstream 500 must surface as an error");

    let message = err.to_string();
    assert!(
        message.contains("500"),
        "expected upstream status in message, got: {message}"
    );

    client.cancel().await?;
    let _ = server_handle.await;
    Ok(())
}
