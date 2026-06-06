//! Hermetic integration tests — drive the server through an in-process
//! MCP client (`tokio::io::duplex`) against a wiremock static host.

use mcp_prompts_46ki75::Server;
use rmcp::model::{RawResource, ReadResourceRequestParams, ResourceContents};
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

/// Markdown body of the `alpha` fixture prompt — carries frontmatter.
const ALPHA_MARKDOWN: &str = "---\nname: Alpha Frontmatter Name\ndescription: Alpha frontmatter description.\n---\n\n# Alpha Title\n\nalpha body\n";

/// Mount the standard two-prompt fixture on `mock`.
///
/// `alpha.md` is served with YAML frontmatter; `beta.md` is listed but
/// deliberately not mounted (its fetch 404s), exercising the fallback
/// to `list.json` metadata.
async fn mount_fixture(mock: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/resources/list.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "name": "alpha", "title": "Alpha Title", "path": "alpha.md" },
            { "name": "beta", "title": "Beta Title", "path": "beta.md" },
        ])))
        .mount(mock)
        .await;
    Mock::given(method("GET"))
        .and(path("/resources/alpha.md"))
        .respond_with(ResponseTemplate::new(200).set_body_string(ALPHA_MARKDOWN))
        .mount(mock)
        .await;
}

fn server_for(mock: &MockServer) -> Server {
    Server::builder()
        .base_url(mock.uri())
        .build()
        .expect("server should build")
}

fn first_text(contents: &[ResourceContents]) -> Option<(&str, Option<&str>, &str)> {
    contents.iter().find_map(|c| match c {
        ResourceContents::TextResourceContents {
            uri,
            mime_type,
            text,
            ..
        } => Some((uri.as_str(), mime_type.as_deref(), text.as_str())),
        _ => None,
    })
}

#[tokio::test]
async fn server_advertises_resources_capability() -> anyhow::Result<()> {
    let mock = MockServer::start().await;
    let (client, server_handle) = spawn(server_for(&mock)).await;

    let info = client.peer_info().expect("server info after initialize");
    assert!(
        info.capabilities.resources.is_some(),
        "resources capability missing: {info:?}"
    );

    client.cancel().await?;
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
async fn list_resources_projects_list_json_entries() -> anyhow::Result<()> {
    let mock = MockServer::start().await;
    mount_fixture(&mock).await;
    let (client, server_handle) = spawn(server_for(&mock)).await;

    let listed = client.list_resources(None).await?;

    let raws: Vec<&RawResource> = listed.resources.iter().map(|r| &r.raw).collect();
    assert_eq!(raws.len(), 2, "expected both fixture prompts: {raws:?}");

    // Frontmatter wins for title/description when present...
    let alpha = raws
        .iter()
        .find(|r| r.name == "alpha")
        .expect("alpha entry");
    assert_eq!(alpha.uri, "prompts://alpha");
    assert_eq!(alpha.title.as_deref(), Some("Alpha Frontmatter Name"));
    assert_eq!(
        alpha.description.as_deref(),
        Some("Alpha frontmatter description.")
    );
    assert_eq!(alpha.mime_type.as_deref(), Some("text/markdown"));

    // ...and an unreadable body falls back to the `list.json` title.
    let beta = raws.iter().find(|r| r.name == "beta").expect("beta entry");
    assert_eq!(beta.uri, "prompts://beta");
    assert_eq!(beta.title.as_deref(), Some("Beta Title"));
    assert_eq!(beta.description, None);

    client.cancel().await?;
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
async fn read_resource_returns_prompt_markdown() -> anyhow::Result<()> {
    let mock = MockServer::start().await;
    mount_fixture(&mock).await;
    let (client, server_handle) = spawn(server_for(&mock)).await;

    let result = client
        .read_resource(ReadResourceRequestParams::new(
            "prompts://alpha".to_string(),
        ))
        .await?;

    let (uri, mime_type, text) = first_text(&result.contents).expect("text contents");
    assert_eq!(uri, "prompts://alpha");
    // Must match what `resources/list` advertises.
    assert_eq!(mime_type, Some("text/markdown"));
    // The frontmatter is metadata for `resources/list` — contents
    // carry only the document body.
    assert_eq!(text, "# Alpha Title\n\nalpha body\n");

    client.cancel().await?;
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
async fn read_resource_rejects_unlisted_names_with_resource_not_found() -> anyhow::Result<()> {
    let mock = MockServer::start().await;
    mount_fixture(&mock).await;
    // `gamma.md` exists upstream but is NOT in list.json — it must
    // stay unreachable, proving reads resolve through the index.
    Mock::given(method("GET"))
        .and(path("/resources/gamma.md"))
        .respond_with(ResponseTemplate::new(200).set_body_string("unlisted"))
        .mount(&mock)
        .await;
    let (client, server_handle) = spawn(server_for(&mock)).await;

    let err = client
        .read_resource(ReadResourceRequestParams::new(
            "prompts://gamma".to_string(),
        ))
        .await
        .expect_err("unlisted prompt must not be readable");

    let message = err.to_string();
    assert!(
        message.contains("resource_not_found"),
        "expected resource_not_found, got: {message}"
    );

    client.cancel().await?;
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
async fn read_resource_rejects_foreign_uri_schemes() -> anyhow::Result<()> {
    let mock = MockServer::start().await;
    let (client, server_handle) = spawn(server_for(&mock)).await;

    let err = client
        .read_resource(ReadResourceRequestParams::new(
            "file:///etc/passwd".to_string(),
        ))
        .await
        .expect_err("foreign scheme must be rejected");

    let message = err.to_string();
    assert!(
        message.contains("unsupported URI scheme"),
        "expected invalid-params message, got: {message}"
    );

    client.cancel().await?;
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
async fn read_resource_rejects_empty_prompt_name() -> anyhow::Result<()> {
    let mock = MockServer::start().await;
    let (client, server_handle) = spawn(server_for(&mock)).await;

    let err = client
        .read_resource(ReadResourceRequestParams::new("prompts://".to_string()))
        .await
        .expect_err("empty name must be rejected");

    let message = err.to_string();
    assert!(
        message.contains("non-empty prompt name"),
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
        .and(path("/resources/list.json"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock)
        .await;
    let (client, server_handle) = spawn(server_for(&mock)).await;

    let err = client
        .list_resources(None)
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
