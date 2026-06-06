//! Use case sitting between the MCP handler and the repository.
//!
//! Owns the one piece of business logic this server has: prompt names
//! are resolved **through `list.json`**, never mapped to URLs directly.
//! That keeps unlisted files unreadable and makes `list.json` the
//! single source of truth for what exists.

use std::sync::Arc;

use crate::repository::{PromptEntry, PromptsRepository, PromptsRepositoryError};

/// A prompt resolved to both its index entry and its markdown body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptDocument {
    /// The `list.json` entry the prompt was resolved from.
    pub entry: PromptEntry,
    /// The prompt markdown body, verbatim. The builder strips
    /// frontmatter at build time, so this is the document body only.
    pub content: String,
}

/// Errors surfaced by [`PromptsUseCase`].
#[derive(Debug, thiserror::Error)]
pub enum PromptsUseCaseError {
    /// The requested prompt name is not present in `list.json`.
    #[error("prompt not found: {name}")]
    NotFound {
        /// The name that failed to resolve.
        name: String,
    },
    /// The repository failed underneath us.
    #[error(transparent)]
    Repository(#[from] PromptsRepositoryError),
}

/// Lists prompts and resolves prompt names to their markdown bodies.
pub struct PromptsUseCase {
    repository: Arc<dyn PromptsRepository>,
}

impl PromptsUseCase {
    /// Wrap a repository.
    pub fn new(repository: Arc<dyn PromptsRepository>) -> Self {
        Self { repository }
    }

    /// All published prompts, in the order `list.json` declares
    /// (the builder sorts it by name). The index already carries the
    /// frontmatter-derived metadata, so listing costs one fetch.
    pub async fn list_prompts(&self) -> Result<Vec<PromptEntry>, PromptsUseCaseError> {
        Ok(self.repository.fetch_list().await?)
    }

    /// Resolve `name` through `list.json` and fetch the prompt body.
    ///
    /// A name absent from the index returns
    /// [`PromptsUseCaseError::NotFound`] without issuing a second
    /// upstream request — unlisted files are unreachable by design.
    pub async fn read_prompt(&self, name: &str) -> Result<PromptDocument, PromptsUseCaseError> {
        let entries = self.repository.fetch_list().await?;
        let entry = entries
            .into_iter()
            .find(|entry| entry.name == name)
            .ok_or_else(|| PromptsUseCaseError::NotFound {
                name: name.to_string(),
            })?;
        let content = self.repository.fetch_prompt(entry.path.clone()).await?;
        Ok(PromptDocument { entry, content })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::PromptsRepositoryStub;

    fn entry(name: &str) -> PromptEntry {
        PromptEntry {
            name: name.to_string(),
            title: format!("Title of {name}"),
            description: None,
            arguments: Vec::new(),
            path: format!("{name}.md"),
        }
    }

    #[tokio::test]
    async fn read_prompt_resolves_path_via_list_json() {
        let stub = Arc::new(PromptsRepositoryStub::new());
        stub.enqueue_list(Ok(vec![entry("alpha"), entry("beta")]))
            .await;
        stub.enqueue_prompt(Ok("# Title of beta\n\nbody".to_string()))
            .await;
        let use_case = PromptsUseCase::new(stub.clone());

        let doc = use_case
            .read_prompt("beta")
            .await
            .expect("prompt should resolve");

        assert_eq!(doc.entry, entry("beta"));
        assert_eq!(doc.content, "# Title of beta\n\nbody");
        // The fetched path MUST be the one `list.json` declared.
        assert_eq!(stub.seen_paths().await, vec!["beta.md".to_string()]);
    }

    #[tokio::test]
    async fn read_prompt_rejects_unlisted_names_without_fetching() {
        let stub = Arc::new(PromptsRepositoryStub::new());
        stub.enqueue_list(Ok(vec![entry("alpha")])).await;
        let use_case = PromptsUseCase::new(stub.clone());

        let err = use_case
            .read_prompt("../../secret")
            .await
            .expect_err("unlisted name must not resolve");

        assert!(matches!(
            err,
            PromptsUseCaseError::NotFound { name } if name == "../../secret"
        ));
        // No second upstream request may have been issued.
        assert!(stub.seen_paths().await.is_empty());
    }

    #[tokio::test]
    async fn list_prompts_returns_index_entries_without_body_fetches() {
        let stub = Arc::new(PromptsRepositoryStub::new());
        stub.enqueue_list(Ok(vec![entry("alpha"), entry("beta")]))
            .await;
        let use_case = PromptsUseCase::new(stub.clone());

        let entries = use_case.list_prompts().await.expect("list should resolve");

        assert_eq!(entries, vec![entry("alpha"), entry("beta")]);
        // Metadata lives in `list.json`; listing must not fetch bodies.
        assert!(stub.seen_paths().await.is_empty());
    }

    #[tokio::test]
    async fn list_prompts_passes_repository_errors_through() {
        let stub = Arc::new(PromptsRepositoryStub::new());
        stub.enqueue_list(Err(PromptsRepositoryError::NotFound {
            url: "stub://list.json".to_string(),
        }))
        .await;
        let use_case = PromptsUseCase::new(stub);

        let err = use_case
            .list_prompts()
            .await
            .expect_err("repository error must propagate");
        assert!(matches!(err, PromptsUseCaseError::Repository(_)));
    }
}
