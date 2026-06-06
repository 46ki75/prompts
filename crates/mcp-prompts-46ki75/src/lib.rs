//! MCP server exposing the prompts published to GitHub Pages as MCP
//! resources.
//!
//! The static distribution (built by `crates/builder`, deployed by
//! `.github/workflows/deploy.yml`) serves two kinds of files:
//!
//! - `resources/list.json` — index of `{ name, title, path }` entries
//! - `resources/<name>.md` — one markdown document per prompt
//!
//! This server reads that hosting and re-exposes it over MCP:
//! `resources/list` projects `list.json` onto `prompts://<name>`
//! resources, and `resources/read` resolves a name through the index
//! and returns the markdown body.
//!
//! A single binary, `46ki75-prompts`, adapts the library to the two MCP
//! transports an editor host cares about, selected by subcommand:
//!
//! - `46ki75-prompts stdio` — line-buffered JSON-RPC over stdin/stdout
//! - `46ki75-prompts http`  — streamable HTTP at `/mcp`
//!
//! Both accept `--base-url` (env `MCP_PROMPTS_BASE_URL`) to override
//! the upstream, useful for hermetic tests or previewing an
//! unpublished `dist/` from a local static server.
//!
//! Internally the crate follows the repository / use case layering
//! documented in the org-wide Rust standards; the MCP adapter lives in
//! [`resources`].

#![deny(missing_docs)]

/// Crate-wide error type used during [`Server`] construction.
pub mod error;
/// HTTP repository over the static distribution.
pub mod repository;
/// MCP resource handlers (`resources/list`, `resources/read`).
pub mod resources;
/// [`ServerBuilder`] and the constants it defaults to.
pub mod router;
/// Name-resolution use case between handler and repository.
pub mod use_case;

use std::sync::Arc;

use rmcp::ErrorData as McpError;
use rmcp::ServerHandler;
use rmcp::model::{
    Implementation, ListResourcesResult, PaginatedRequestParams, ReadResourceRequestParams,
    ReadResourceResult, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};

pub use crate::error::Error;
pub use crate::router::{DEFAULT_USER_AGENT, PROMPTS_BASE_URL, ServerBuilder};
pub use crate::use_case::PromptsUseCase;

/// MCP server handler exposing the prompt distribution as resources.
///
/// Cheap to clone — the use case is behind an `Arc` — which is what
/// lets the streamable-HTTP transport hand out a fresh `Server` per
/// session without rebuilding any state.
#[derive(Clone)]
pub struct Server {
    prompts_use_case: Arc<PromptsUseCase>,
}

impl Server {
    /// Build a server backed by a freshly constructed HTTP client
    /// pointed at the public GitHub Pages distribution. Equivalent to
    /// `Self::builder().build()`.
    pub fn new() -> Result<Self, Error> {
        Self::builder().build()
    }

    /// Start a [`ServerBuilder`] for customizing the upstream URL,
    /// HTTP client, or repository implementation.
    pub fn builder() -> ServerBuilder {
        ServerBuilder::default()
    }

    /// Wrap a pre-built use case. Useful when callers want to share
    /// one HTTP connection pool across many `Server` clones, as the
    /// HTTP transport binary does.
    pub fn with_use_case(prompts_use_case: Arc<PromptsUseCase>) -> Self {
        Self { prompts_use_case }
    }

    pub(crate) fn prompts_use_case(&self) -> &PromptsUseCase {
        &self.prompts_use_case
    }
}

impl ServerHandler for Server {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_resources().build();
        info.server_info = Implementation::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        info.instructions = Some(
            "Prompt library served from static hosting. `resources/list` \
             enumerates the published prompts as `prompts://<name>` \
             resources; `resources/read` returns a prompt's markdown body."
                .to_string(),
        );
        info
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        resources::list_resources(self).await
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        resources::read_resource(self, request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cloning a [`Server`] MUST share its use case `Arc` with the
    /// source. The streamable-HTTP transport relies on this: its
    /// per-session factory hands out a fresh `Server` per session by
    /// cloning a pre-built template, so all sessions share the
    /// underlying `reqwest::Client` (and its connection pool).
    #[test]
    fn cloning_server_shares_use_case_arc() {
        let server = Server::new().expect("server should build");
        let clone = server.clone();
        assert!(
            Arc::ptr_eq(&server.prompts_use_case, &clone.prompts_use_case),
            "prompts_use_case must be shared between clones",
        );
    }
}
