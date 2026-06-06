//! MCP resource handlers — the thin adapter from the use case onto
//! `resources/list` and `resources/read`.
//!
//! Prompts are addressed as `prompts://<name>` where `<name>` is the
//! `name` field of the `list.json` entry. The custom scheme (rather
//! than the raw GitHub Pages URL) keeps resource URIs stable if the
//! hosting ever moves.

use rmcp::ErrorData as McpError;
use rmcp::model::{
    AnnotateAble, ListResourcesResult, RawResource, ReadResourceRequestParams, ReadResourceResult,
    ResourceContents,
};
use serde_json::json;

use crate::Server;
use crate::use_case::PromptsUseCaseError;

/// URI scheme under which prompts are exposed.
pub const PROMPT_URI_SCHEME: &str = "prompts://";

/// MIME type reported for every prompt resource.
pub const PROMPT_MIME_TYPE: &str = "text/markdown";

/// Build the canonical resource URI for a prompt name.
pub fn prompt_uri(name: &str) -> String {
    format!("{PROMPT_URI_SCHEME}{name}")
}

/// Handle `resources/list` by projecting `list.json` entries onto
/// MCP [`Resource`]s.
pub async fn list_resources(server: &Server) -> Result<ListResourcesResult, McpError> {
    let entries = server
        .prompts_use_case()
        .list_prompts()
        .await
        .map_err(to_mcp_error)?;

    let resources = entries
        .into_iter()
        .map(|entry| {
            let mut raw = RawResource::new(prompt_uri(&entry.name), entry.name);
            raw.title = Some(entry.title);
            raw.mime_type = Some(PROMPT_MIME_TYPE.to_string());
            raw.no_annotation()
        })
        .collect();

    Ok(ListResourcesResult {
        resources,
        next_cursor: None,
        meta: None,
    })
}

/// Handle `resources/read` for `prompts://<name>` URIs.
pub async fn read_resource(
    server: &Server,
    request: ReadResourceRequestParams,
) -> Result<ReadResourceResult, McpError> {
    let Some(name) = request.uri.strip_prefix(PROMPT_URI_SCHEME) else {
        return Err(McpError::invalid_params(
            format!("unsupported URI scheme; expected {PROMPT_URI_SCHEME}<name>"),
            Some(json!({ "uri": request.uri })),
        ));
    };
    if name.is_empty() {
        return Err(McpError::invalid_params(
            format!("{PROMPT_URI_SCHEME} requires a non-empty prompt name"),
            Some(json!({ "uri": request.uri })),
        ));
    }

    let document = server
        .prompts_use_case()
        .read_prompt(name)
        .await
        .map_err(to_mcp_error)?;

    Ok(ReadResourceResult::new(vec![
        // Keep the contents' MIME type consistent with what
        // `resources/list` advertises (the `::text` helper would
        // default it to the bare "text").
        ResourceContents::text(document.content, request.uri).with_mime_type(PROMPT_MIME_TYPE),
    ]))
}

/// Map use case errors onto JSON-RPC error codes: an unknown name is
/// `resource_not_found`; everything else (network, bad deployment) is
/// an internal error carrying the upstream detail.
fn to_mcp_error(error: PromptsUseCaseError) -> McpError {
    match &error {
        PromptsUseCaseError::NotFound { name } => McpError::resource_not_found(
            "resource_not_found",
            Some(json!({ "uri": prompt_uri(name) })),
        ),
        PromptsUseCaseError::Repository(_) => McpError::internal_error(error.to_string(), None),
    }
}
