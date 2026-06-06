//! Use case sitting between the MCP handler and the repository.
//!
//! Owns the one piece of business logic this server has: prompt names
//! are resolved **through `list.json`**, never mapped to URLs directly.
//! That keeps unlisted files unreadable and makes `list.json` the
//! single source of truth for what exists.

use std::sync::Arc;

use crate::frontmatter::{self, Frontmatter};
use crate::repository::{PromptEntry, PromptsRepository, PromptsRepositoryError};

/// A prompt resolved to both its index entry and its markdown body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptDocument {
    /// The `list.json` entry the prompt was resolved from.
    pub entry: PromptEntry,
    /// The prompt markdown, with any leading frontmatter block
    /// removed — the metadata is already surfaced on the resource
    /// listing, so contents carry only the document body.
    pub content: String,
}

/// A `list.json` entry enriched with the prompt's frontmatter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptSummary {
    /// The `list.json` entry.
    pub entry: PromptEntry,
    /// Display title: the frontmatter `name` when present, otherwise
    /// the `list.json` `title` (the prompt's first `#` heading).
    pub title: String,
    /// The frontmatter `description`, when present.
    pub description: Option<String>,
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
    /// (the builder sorts it by name), enriched with frontmatter.
    ///
    /// Frontmatter lives in the prompt bodies, so each listed prompt
    /// is fetched (concurrently) alongside the index. The enrichment
    /// is best-effort: a body that fails to fetch or carries no
    /// parseable frontmatter degrades to the `list.json` metadata
    /// rather than failing the listing.
    pub async fn list_prompts(&self) -> Result<Vec<PromptSummary>, PromptsUseCaseError> {
        let entries = self.repository.fetch_list().await?;

        let mut fetches = tokio::task::JoinSet::new();
        for (index, entry) in entries.iter().enumerate() {
            let repository = Arc::clone(&self.repository);
            let path = entry.path.clone();
            fetches.spawn(async move { (index, repository.fetch_prompt(path).await) });
        }

        let mut frontmatters: Vec<Option<Frontmatter>> = vec![None; entries.len()];
        while let Some(joined) = fetches.join_next().await {
            if let Ok((index, Ok(body))) = joined {
                frontmatters[index] = frontmatter::parse(&body);
            }
        }

        Ok(entries
            .into_iter()
            .zip(frontmatters)
            .map(|(entry, frontmatter)| {
                let frontmatter = frontmatter.unwrap_or_default();
                PromptSummary {
                    title: frontmatter.name.unwrap_or_else(|| entry.title.clone()),
                    description: frontmatter.description,
                    entry,
                }
            })
            .collect())
    }

    /// Resolve `name` through `list.json` and fetch the prompt body,
    /// stripping any leading frontmatter block.
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
        Ok(PromptDocument {
            entry,
            content: frontmatter::strip(&content).to_string(),
        })
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
    async fn read_prompt_strips_frontmatter_from_content() {
        let stub = Arc::new(PromptsRepositoryStub::new());
        stub.enqueue_list(Ok(vec![entry("alpha")])).await;
        stub.enqueue_prompt(Ok(
            "---\nname: Alpha Prompt\n---\n\n# Title of alpha\n\nbody\n".to_string(),
        ))
        .await;
        let use_case = PromptsUseCase::new(stub);

        let doc = use_case
            .read_prompt("alpha")
            .await
            .expect("prompt should resolve");

        assert_eq!(doc.content, "# Title of alpha\n\nbody\n");
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
    async fn list_prompts_prefers_frontmatter_name_and_description() {
        let stub = Arc::new(PromptsRepositoryStub::new());
        stub.enqueue_list(Ok(vec![entry("alpha")])).await;
        stub.enqueue_prompt(Ok(
            "---\nname: Alpha Prompt\ndescription: Alpha summary.\n---\n\n# Title of alpha\n"
                .to_string(),
        ))
        .await;
        let use_case = PromptsUseCase::new(stub);

        let summaries = use_case.list_prompts().await.expect("list should resolve");

        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].title, "Alpha Prompt");
        assert_eq!(summaries[0].description.as_deref(), Some("Alpha summary."));
        assert_eq!(summaries[0].entry, entry("alpha"));
    }

    #[tokio::test]
    async fn list_prompts_falls_back_to_entry_title_without_frontmatter() {
        let stub = Arc::new(PromptsRepositoryStub::new());
        stub.enqueue_list(Ok(vec![entry("alpha")])).await;
        stub.enqueue_prompt(Ok("# Title of alpha\n\nno frontmatter\n".to_string()))
            .await;
        let use_case = PromptsUseCase::new(stub);

        let summaries = use_case.list_prompts().await.expect("list should resolve");

        assert_eq!(summaries[0].title, "Title of alpha");
        assert_eq!(summaries[0].description, None);
    }

    #[tokio::test]
    async fn list_prompts_tolerates_body_fetch_failures() {
        let stub = Arc::new(PromptsRepositoryStub::new());
        stub.enqueue_list(Ok(vec![entry("alpha")])).await;
        // No prompt enqueued — the stub's fetch_prompt errors.
        let use_case = PromptsUseCase::new(stub);

        let summaries = use_case
            .list_prompts()
            .await
            .expect("a failed body fetch must not fail the listing");

        assert_eq!(summaries[0].title, "Title of alpha");
        assert_eq!(summaries[0].description, None);
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
