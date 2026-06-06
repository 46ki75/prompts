//! HTTP repository over the static distribution on GitHub Pages.
//!
//! The distribution contract (see the repository README) is two kinds
//! of files under `<base>/resources/`:
//!
//! - `list.json` — JSON array of `{ name, title, path }` entries
//! - `<path>` — one markdown file per prompt, referenced by `list.json`

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::Deserialize;

/// Boxed future used to keep the repository trait dyn-compatible.
///
/// See the org standards' _Async traits with `Arc<dyn>`_ section for
/// why we hand-roll this instead of using `#[async_trait]`.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// One entry of `list.json`, mirroring the builder's output shape.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PromptEntry {
    /// Directory name of the prompt, e.g. `information-retrieval-policy`.
    pub name: String,
    /// Human-readable title (the prompt's first `#` heading).
    pub title: String,
    /// Path of the markdown file relative to `resources/`.
    pub path: String,
}

/// Errors surfaced by the repository layer.
#[derive(Debug, thiserror::Error)]
pub enum PromptsRepositoryError {
    /// The HTTP request itself failed (DNS, TLS, timeout, ...).
    #[error("request to {url} failed: {source}")]
    Request {
        /// URL the failing request targeted.
        url: String,
        /// Underlying transport error.
        source: reqwest::Error,
    },
    /// The upstream returned 404 for the requested file.
    #[error("not found upstream: {url}")]
    NotFound {
        /// URL that returned 404.
        url: String,
    },
    /// The upstream returned a non-2xx, non-404 status.
    #[error("upstream returned {status} for {url}")]
    UpstreamStatus {
        /// HTTP status returned.
        status: reqwest::StatusCode,
        /// URL that returned the status.
        url: String,
    },
    /// The response body exceeded the configured size cap.
    #[error("response from {url} exceeded the {limit_bytes}-byte cap")]
    PayloadTooLarge {
        /// URL whose body blew the cap.
        url: String,
        /// The cap that fired.
        limit_bytes: usize,
    },
    /// `list.json` did not parse as the expected entry array.
    #[error("invalid list.json from {url}: {source}")]
    InvalidList {
        /// URL the malformed index was fetched from.
        url: String,
        /// Underlying deserialization error.
        source: serde_json::Error,
    },
    /// A prompt body was not valid UTF-8.
    #[error("non-UTF-8 prompt body from {url}")]
    InvalidUtf8 {
        /// URL the malformed body was fetched from.
        url: String,
    },
}

/// Convenience alias for the index fetch result.
pub type FetchListResult = Result<Vec<PromptEntry>, PromptsRepositoryError>;

/// Convenience alias for the per-prompt fetch result.
pub type FetchPromptResult = Result<String, PromptsRepositoryError>;

/// Repository abstraction over the static prompt distribution.
///
/// Held as `Arc<dyn PromptsRepository>` by the use case so a stub can
/// be swapped in for unit tests without touching the real HTTP client.
pub trait PromptsRepository: Send + Sync + 'static {
    /// Fetch and parse `<base>/resources/list.json`.
    fn fetch_list(&self) -> BoxFuture<'_, FetchListResult>;

    /// Fetch `<base>/resources/<path>` as UTF-8 markdown. `path` MUST
    /// come from a [`PromptEntry`] returned by
    /// [`fetch_list`](Self::fetch_list) — the repository does not
    /// validate it beyond URL-joining.
    fn fetch_prompt(&self, path: String) -> BoxFuture<'_, FetchPromptResult>;
}

/// Real implementation backed by `reqwest`, talking to GitHub Pages
/// (or any static host serving the same layout) over HTTPS.
pub struct PromptsRepositoryImpl {
    http: reqwest::Client,
    base_url: Arc<str>,
    max_body_bytes: usize,
}

impl PromptsRepositoryImpl {
    /// Wrap a pre-built `reqwest::Client` and the base URL of the
    /// distribution (e.g. `https://46ki75.github.io/prompts`). A
    /// trailing slash on `base_url` is tolerated.
    pub fn new(http: reqwest::Client, base_url: impl Into<Arc<str>>) -> Self {
        Self {
            http,
            base_url: base_url.into(),
            max_body_bytes: crate::router::DEFAULT_UPSTREAM_BODY_BYTES,
        }
    }

    /// Override the per-response body-size cap. Above the cap the
    /// request errors with [`PromptsRepositoryError::PayloadTooLarge`].
    pub fn with_max_body_bytes(mut self, limit: usize) -> Self {
        self.max_body_bytes = limit;
        self
    }

    fn resource_url(&self, file: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        format!("{base}/resources/{file}")
    }

    async fn fetch_bytes(&self, url: &str) -> Result<Vec<u8>, PromptsRepositoryError> {
        let response =
            self.http
                .get(url)
                .send()
                .await
                .map_err(|source| PromptsRepositoryError::Request {
                    url: url.to_string(),
                    source,
                })?;

        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(PromptsRepositoryError::NotFound {
                url: url.to_string(),
            });
        }
        if !status.is_success() {
            return Err(PromptsRepositoryError::UpstreamStatus {
                status,
                url: url.to_string(),
            });
        }

        read_body_bounded(response, url, self.max_body_bytes).await
    }
}

impl PromptsRepository for PromptsRepositoryImpl {
    fn fetch_list(&self) -> BoxFuture<'_, FetchListResult> {
        Box::pin(async move {
            let url = self.resource_url("list.json");
            let body = self.fetch_bytes(&url).await?;
            serde_json::from_slice(&body)
                .map_err(|source| PromptsRepositoryError::InvalidList { url, source })
        })
    }

    fn fetch_prompt(&self, path: String) -> BoxFuture<'_, FetchPromptResult> {
        Box::pin(async move {
            let url = self.resource_url(&path);
            let body = self.fetch_bytes(&url).await?;
            String::from_utf8(body).map_err(|_| PromptsRepositoryError::InvalidUtf8 { url })
        })
    }
}

/// Stream a response body into memory, aborting once the running size
/// exceeds `limit_bytes` — so a misbehaving host cannot pin the
/// process with an inflated body regardless of its `Content-Length`.
async fn read_body_bounded(
    mut response: reqwest::Response,
    url: &str,
    limit_bytes: usize,
) -> Result<Vec<u8>, PromptsRepositoryError> {
    let mut acc: Vec<u8> = Vec::new();
    while let Some(chunk) =
        response
            .chunk()
            .await
            .map_err(|source| PromptsRepositoryError::Request {
                url: url.to_string(),
                source,
            })?
    {
        if acc.len().saturating_add(chunk.len()) > limit_bytes {
            return Err(PromptsRepositoryError::PayloadTooLarge {
                url: url.to_string(),
                limit_bytes,
            });
        }
        acc.extend_from_slice(&chunk);
    }
    Ok(acc)
}

/// In-memory stub used by unit tests across the crate. Gated on
/// `cfg(test)` so it never ships in release builds; integration tests
/// in `tests/` exercise the real repository through a wiremock-backed
/// upstream instead.
#[cfg(test)]
#[derive(Default)]
pub(crate) struct PromptsRepositoryStub {
    list_queue: tokio::sync::Mutex<Vec<FetchListResult>>,
    prompt_queue: tokio::sync::Mutex<Vec<FetchPromptResult>>,
    /// Captured `path` arguments for every `fetch_prompt` call, in
    /// arrival order. Lets unit tests pin that the use case resolves
    /// names through `list.json` rather than guessing URLs.
    seen_paths: tokio::sync::Mutex<Vec<String>>,
}

#[cfg(test)]
impl PromptsRepositoryStub {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) async fn enqueue_list(&self, result: FetchListResult) {
        self.list_queue.lock().await.push(result);
    }

    pub(crate) async fn enqueue_prompt(&self, result: FetchPromptResult) {
        self.prompt_queue.lock().await.push(result);
    }

    pub(crate) async fn seen_paths(&self) -> Vec<String> {
        self.seen_paths.lock().await.clone()
    }
}

#[cfg(test)]
impl PromptsRepository for PromptsRepositoryStub {
    fn fetch_list(&self) -> BoxFuture<'_, FetchListResult> {
        Box::pin(async move {
            self.list_queue
                .lock()
                .await
                .pop()
                .unwrap_or_else(|| Ok(vec![]))
        })
    }

    fn fetch_prompt(&self, path: String) -> BoxFuture<'_, FetchPromptResult> {
        Box::pin(async move {
            self.seen_paths.lock().await.push(path.clone());
            self.prompt_queue.lock().await.pop().unwrap_or_else(|| {
                Err(PromptsRepositoryError::NotFound {
                    url: format!("stub://{path}"),
                })
            })
        })
    }
}
