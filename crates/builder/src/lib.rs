//! Builds the static distribution of prompts for GitHub Pages.
//!
//! Scans a repository root for top-level directories containing a
//! `prompt.md` — markdown following the Claude Skill format, with an
//! optional YAML frontmatter block (`name`, `description`,
//! `argument-hint`). For each prompt the builder:
//!
//! - parses the frontmatter into index metadata,
//! - writes the body (frontmatter stripped) to `<out>/prompts/<name>.md`,
//! - and records an entry in `<out>/prompts/list.json`.
//!
//! `list.json` carries everything a server needs to advertise the
//! prompts — consumers never have to parse frontmatter themselves.

/// Claude Skill–style frontmatter parsing and stripping.
pub mod frontmatter;

use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// A single prompt entry as it appears in `list.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PromptEntry {
    /// Prompt identifier: the frontmatter `name`, falling back to the
    /// prompt's directory name.
    pub name: String,
    /// Title taken from the body's first `#` heading; falls back to
    /// `name`.
    pub title: String,
    /// The frontmatter `description`, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Argument names parsed from the frontmatter `argument-hint`, in
    /// positional order. Omitted when the prompt takes no arguments.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<String>,
    /// Path of the markdown file relative to `prompts/`.
    pub path: String,
}

/// Errors that can occur while building the distribution.
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    /// An I/O operation failed.
    #[error("{path}: {source}")]
    Io {
        /// Path the failing operation was acting on.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// Serializing `list.json` failed.
    #[error("failed to serialize list.json: {0}")]
    Serialize(#[from] serde_json::Error),
}

fn io_err(path: &Path) -> impl FnOnce(std::io::Error) -> BuildError + '_ {
    move |source| BuildError::Io {
        path: path.to_path_buf(),
        source,
    }
}

/// Extracts the title from the first ATX `#` heading in `markdown`.
pub fn extract_title(markdown: &str) -> Option<&str> {
    markdown.lines().find_map(|line| {
        line.strip_prefix("# ")
            .map(str::trim)
            .filter(|title| !title.is_empty())
    })
}

/// Scans `root` for top-level directories containing a `prompt.md`.
///
/// Hidden directories (leading `.`) are skipped. Returns
/// `(entry, body)` pairs — the body has its frontmatter stripped —
/// sorted by name so output is deterministic.
pub fn collect_prompts(root: &Path) -> Result<Vec<(PromptEntry, String)>, BuildError> {
    let mut prompts = Vec::new();
    for dir_entry in fs::read_dir(root).map_err(io_err(root))? {
        let dir_path = dir_entry.map_err(io_err(root))?.path();
        if !dir_path.is_dir() {
            continue;
        }
        let Some(dir_name) = dir_path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if dir_name.starts_with('.') {
            continue;
        }
        let prompt_path = dir_path.join("prompt.md");
        if !prompt_path.is_file() {
            continue;
        }
        let content = fs::read_to_string(&prompt_path).map_err(io_err(&prompt_path))?;
        let metadata = frontmatter::parse(&content).unwrap_or_default();
        let body = frontmatter::strip(&content).to_owned();

        let arguments = metadata.argument_names();
        let name = metadata.name.unwrap_or_else(|| dir_name.to_owned());
        let title = extract_title(&body).unwrap_or(&name).to_owned();
        prompts.push((
            PromptEntry {
                path: format!("{name}.md"),
                title,
                description: metadata.description,
                arguments,
                name,
            },
            body,
        ));
    }
    prompts.sort_by(|a, b| a.0.name.cmp(&b.0.name));
    Ok(prompts)
}

/// Builds the static distribution under `out` and returns the index entries.
///
/// Produces `<out>/prompts/<name>.md` (frontmatter stripped) per prompt
/// and `<out>/prompts/list.json` as the index.
pub fn build(root: &Path, out: &Path) -> Result<Vec<PromptEntry>, BuildError> {
    let prompts_dir = out.join("prompts");
    fs::create_dir_all(&prompts_dir).map_err(io_err(&prompts_dir))?;

    let prompts = collect_prompts(root)?;
    let mut index = Vec::with_capacity(prompts.len());
    for (entry, body) in prompts {
        let target = prompts_dir.join(&entry.path);
        fs::write(&target, body).map_err(io_err(&target))?;
        index.push(entry);
    }

    let mut json = serde_json::to_string_pretty(&index)?;
    json.push('\n');
    let list_path = prompts_dir.join("list.json");
    fs::write(&list_path, json).map_err(io_err(&list_path))?;
    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::extract_title;

    #[test]
    fn title_from_first_heading() {
        assert_eq!(
            extract_title("# Information Retrieval Policy\n\nbody"),
            Some("Information Retrieval Policy")
        );
    }

    #[test]
    fn title_skips_leading_prose() {
        assert_eq!(
            extract_title("intro line\n\n# Real Title\n"),
            Some("Real Title")
        );
    }

    #[test]
    fn no_heading_yields_none() {
        assert_eq!(extract_title("just text\n## subheading only\n"), None);
    }

    #[test]
    fn empty_heading_yields_none() {
        assert_eq!(extract_title("#  \n"), None);
    }
}
