//! [`ServerBuilder`] and the constants it defaults to.

use std::sync::Arc;
use std::time::Duration;

use crate::Server;
use crate::error::Error;
use crate::repository::{PromptsRepository, PromptsRepositoryImpl};
use crate::use_case::PromptsUseCase;

/// Default distribution upstream — this repository's GitHub Pages site.
pub const PROMPTS_BASE_URL: &str = "https://46ki75.github.io/prompts";

/// Default overall HTTP request timeout applied to the built-in
/// `reqwest::Client`. Prompt files are small; 30s only matters when
/// the upstream is genuinely unreachable, in which case we want an
/// error rather than a hang.
pub const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(30);

/// Default TCP-connect timeout applied to the built-in
/// `reqwest::Client`. Distinct from the overall request timeout so a
/// dead upstream fails fast at the SYN-ACK stage.
pub const DEFAULT_HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Default per-response body size cap.
///
/// Prompts are markdown documents of a few KB; `list.json` is smaller
/// still. 8 MB leaves generous headroom while keeping a misbehaving
/// host from pinning the process with an arbitrarily large body.
pub const DEFAULT_UPSTREAM_BODY_BYTES: usize = 8 * 1024 * 1024;

/// Default `User-Agent` header sent by the built-in HTTP client,
/// embedding the crate version and source URL.
pub const DEFAULT_USER_AGENT: &str = concat!(
    "mcp-prompts/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/46ki75/prompts)",
);

/// Fluent builder for [`Server`].
///
/// All fields are optional; calling [`build`](Self::build) with no
/// overrides reproduces [`Server::new`]. Tests point
/// [`base_url`](Self::base_url) at a wiremock upstream.
pub struct ServerBuilder {
    base_url: String,
    user_agent: String,
    http: Option<reqwest::Client>,
    http_timeout: Duration,
    http_connect_timeout: Duration,
    upstream_body_size_limit: usize,
    repository: Option<Arc<dyn PromptsRepository>>,
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self {
            base_url: PROMPTS_BASE_URL.to_string(),
            user_agent: DEFAULT_USER_AGENT.to_string(),
            http: None,
            http_timeout: DEFAULT_HTTP_TIMEOUT,
            http_connect_timeout: DEFAULT_HTTP_CONNECT_TIMEOUT,
            upstream_body_size_limit: DEFAULT_UPSTREAM_BODY_BYTES,
            repository: None,
        }
    }
}

impl ServerBuilder {
    /// Override the distribution base URL. Defaults to
    /// [`PROMPTS_BASE_URL`]. Pass a wiremock URL in tests, or a local
    /// static server when previewing an unpublished `dist/`.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Override the `User-Agent` header sent by the built-in HTTP
    /// client. Ignored when [`http_client`](Self::http_client) is also
    /// set (the supplied client owns its own headers).
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Supply a pre-built `reqwest::Client`.
    ///
    /// **Caveat:** when a client is supplied, the
    /// [`http_timeout`](Self::http_timeout) and
    /// [`http_connect_timeout`](Self::http_connect_timeout) settings
    /// are ignored — the injected client owns its own timeouts.
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.http = Some(client);
        self
    }

    /// Override the overall HTTP request timeout applied to the
    /// built-in client. Defaults to [`DEFAULT_HTTP_TIMEOUT`].
    pub fn http_timeout(mut self, timeout: Duration) -> Self {
        self.http_timeout = timeout;
        self
    }

    /// Override the TCP-connect timeout applied to the built-in
    /// client. Defaults to [`DEFAULT_HTTP_CONNECT_TIMEOUT`].
    pub fn http_connect_timeout(mut self, timeout: Duration) -> Self {
        self.http_connect_timeout = timeout;
        self
    }

    /// Override the per-response body-size cap. Defaults to
    /// [`DEFAULT_UPSTREAM_BODY_BYTES`]. Ignored when
    /// [`repository`](Self::repository) is supplied.
    pub fn upstream_body_size_limit(mut self, limit: usize) -> Self {
        self.upstream_body_size_limit = limit;
        self
    }

    /// Inject a fully-formed repository, short-circuiting HTTP client
    /// setup entirely.
    pub fn repository(mut self, repository: Arc<dyn PromptsRepository>) -> Self {
        self.repository = Some(repository);
        self
    }

    /// Finalize the builder: construct the HTTP client (only when no
    /// repository was injected), then the repository, use case, and
    /// server in that order.
    pub fn build(self) -> Result<Server, Error> {
        let repository: Arc<dyn PromptsRepository> = match self.repository {
            Some(repository) => repository,
            None => {
                let http = match self.http {
                    Some(client) => client,
                    None => reqwest::Client::builder()
                        .user_agent(self.user_agent)
                        .timeout(self.http_timeout)
                        .connect_timeout(self.http_connect_timeout)
                        .build()?,
                };
                Arc::new(
                    PromptsRepositoryImpl::new(http, self.base_url)
                        .with_max_body_bytes(self.upstream_body_size_limit),
                )
            }
        };

        let use_case = Arc::new(PromptsUseCase::new(repository));
        Ok(Server::with_use_case(use_case))
    }
}
