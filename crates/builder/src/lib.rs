//! Builds the static distribution of prompts for GitHub Pages.
//!
//! Scans a repository root for top-level directories containing a
//! `prompt.md`, copies each prompt to `<out>/resources/<name>.md`, and
//! writes an index of all prompts to `<out>/resources/list.json`.

use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// A single prompt entry as it appears in `list.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PromptEntry {
    /// Directory name of the prompt, e.g. `information-retrieval-policy`.
    pub name: String,
    /// Title taken from the prompt's first `#` heading; falls back to `name`.
    pub title: String,
    /// Path of the markdown file relative to `resources/`.
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
/// `(entry, content)` pairs sorted by name so output is deterministic.
pub fn collect_prompts(root: &Path) -> Result<Vec<(PromptEntry, String)>, BuildError> {
    let mut prompts = Vec::new();
    for dir_entry in fs::read_dir(root).map_err(io_err(root))? {
        let dir_path = dir_entry.map_err(io_err(root))?.path();
        if !dir_path.is_dir() {
            continue;
        }
        let Some(name) = dir_path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        let prompt_path = dir_path.join("prompt.md");
        if !prompt_path.is_file() {
            continue;
        }
        let content = fs::read_to_string(&prompt_path).map_err(io_err(&prompt_path))?;
        let title = extract_title(&content).unwrap_or(name).to_owned();
        prompts.push((
            PromptEntry {
                name: name.to_owned(),
                title,
                path: format!("{name}.md"),
            },
            content,
        ));
    }
    prompts.sort_by(|a, b| a.0.name.cmp(&b.0.name));
    Ok(prompts)
}

/// Builds the static distribution under `out` and returns the index entries.
///
/// Produces `<out>/resources/<name>.md` per prompt and
/// `<out>/resources/list.json` as the index.
pub fn build(root: &Path, out: &Path) -> Result<Vec<PromptEntry>, BuildError> {
    let resources = out.join("resources");
    fs::create_dir_all(&resources).map_err(io_err(&resources))?;

    let prompts = collect_prompts(root)?;
    let mut index = Vec::with_capacity(prompts.len());
    for (entry, content) in prompts {
        let target = resources.join(&entry.path);
        fs::write(&target, content).map_err(io_err(&target))?;
        index.push(entry);
    }

    let mut json = serde_json::to_string_pretty(&index)?;
    json.push('\n');
    let list_path = resources.join("list.json");
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
