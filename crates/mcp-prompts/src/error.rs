//! Crate-wide error type used during [`Server`](crate::Server)
//! construction.

/// Errors that can occur while building a [`Server`](crate::Server).
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Constructing the built-in `reqwest::Client` failed.
    #[error("failed to build HTTP client: {0}")]
    HttpClient(#[from] reqwest::Error),
}
