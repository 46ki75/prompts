//! Claude Skill–style YAML frontmatter: parsing and stripping.
//!
//! Prompts MAY open with a `---`-delimited YAML block following the
//! Claude Skill frontmatter format (`name`, `description`,
//! `argument-hint`). Parsing is fallible by design: a missing,
//! unterminated, or malformed block yields [`None`] — never an error —
//! so the builder falls back to metadata derived from the document
//! itself.

use serde::Deserialize;

/// The frontmatter fields the distribution consumes. Unknown fields
/// are ignored so prompts can carry additional skill metadata without
/// breaking the build.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Frontmatter {
    /// Prompt identifier (kebab-case, matching the directory name in
    /// the Claude Skill format).
    #[serde(default)]
    pub name: Option<String>,
    /// One-line summary of what the prompt does.
    #[serde(default)]
    pub description: Option<String>,
    /// Claude Skill argument hint, e.g. `"[topic] [audience]"`.
    #[serde(default, rename = "argument-hint")]
    pub argument_hint: Option<String>,
}

impl Frontmatter {
    /// Argument names extracted from the bracketed tokens of
    /// `argument-hint`, in order of first appearance, deduplicated
    /// (alternative-syntax hints like `add [tag] | remove [tag]`
    /// declare each argument once).
    pub fn argument_names(&self) -> Vec<String> {
        let Some(hint) = &self.argument_hint else {
            return Vec::new();
        };
        let mut names: Vec<String> = Vec::new();
        let mut rest = hint.as_str();
        while let Some(open) = rest.find('[') {
            rest = &rest[open + 1..];
            let Some(close) = rest.find(']') else {
                break;
            };
            let token = rest[..close].trim();
            if !token.is_empty() && !names.iter().any(|name| name == token) {
                names.push(token.to_string());
            }
            rest = &rest[close + 1..];
        }
        names
    }
}

/// Split `markdown` into its frontmatter YAML and the rest of the
/// document, or [`None`] when the document does not open with a
/// well-delimited `---` block.
fn split(markdown: &str) -> Option<(&str, &str)> {
    let body = markdown.strip_prefix("---")?;
    let body = body
        .strip_prefix("\r\n")
        .or_else(|| body.strip_prefix('\n'))?;

    // The YAML block runs up to the first line that is exactly `---`
    // (modulo trailing whitespace / CR).
    let mut yaml_len = 0;
    for line in body.split_inclusive('\n') {
        if line.trim_end() == "---" {
            return Some((&body[..yaml_len], &body[yaml_len + line.len()..]));
        }
        yaml_len += line.len();
    }
    None
}

/// Parse the leading `---` YAML frontmatter block of `markdown`.
///
/// Returns [`None`] when the document does not start with `---`, the
/// block is never closed by a `---` line, or the block is not valid
/// YAML.
pub fn parse(markdown: &str) -> Option<Frontmatter> {
    let (yaml, _) = split(markdown)?;
    serde_saphyr::from_str(yaml).ok()
}

/// Return `markdown` without its leading frontmatter block, also
/// dropping the blank lines separating the block from the body.
///
/// Stripping is structural: a well-delimited block is removed even if
/// its YAML would not [`parse`]. A document without a (well-formed)
/// block is returned unchanged.
pub fn strip(markdown: &str) -> &str {
    match split(markdown) {
        Some((_, rest)) => rest.trim_start_matches(['\r', '\n']),
        None => markdown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_skill_frontmatter() {
        let markdown = "---\nname: alpha-prompt\ndescription: One-line summary.\nargument-hint: \"[topic] [audience]\"\n---\n\n# Alpha\n";
        let frontmatter = parse(markdown).expect("frontmatter should parse");
        assert_eq!(frontmatter.name.as_deref(), Some("alpha-prompt"));
        assert_eq!(
            frontmatter.description.as_deref(),
            Some("One-line summary.")
        );
        assert_eq!(
            frontmatter.argument_names(),
            vec!["topic".to_string(), "audience".to_string()]
        );
    }

    #[test]
    fn parses_folded_block_scalar_description() {
        let markdown = "---\nname: alpha\ndescription: >-\n  Folded first line,\n  folded second line.\n---\n\nbody\n";
        let frontmatter = parse(markdown).expect("frontmatter should parse");
        assert_eq!(
            frontmatter.description.as_deref(),
            Some("Folded first line, folded second line.")
        );
    }

    #[test]
    fn ignores_unknown_fields() {
        let markdown = "---\nname: alpha\nallowed-tools: Bash\n---\nbody\n";
        let frontmatter = parse(markdown).expect("frontmatter should parse");
        assert_eq!(frontmatter.name.as_deref(), Some("alpha"));
        assert_eq!(frontmatter.description, None);
    }

    #[test]
    fn missing_fields_stay_none() {
        let frontmatter = parse("---\ndescription: only this\n---\n").expect("should parse");
        assert_eq!(frontmatter.name, None);
        assert_eq!(frontmatter.description.as_deref(), Some("only this"));
        assert!(frontmatter.argument_names().is_empty());
    }

    #[test]
    fn document_without_frontmatter_is_none() {
        assert_eq!(parse("# Just a Heading\n\nbody\n"), None);
    }

    #[test]
    fn unterminated_block_is_none() {
        assert_eq!(parse("---\nname: alpha\n\n# Heading\n"), None);
    }

    #[test]
    fn invalid_yaml_is_none() {
        assert_eq!(parse("---\n: [unbalanced\n---\n"), None);
    }

    #[test]
    fn opening_delimiter_requires_its_own_line() {
        // A thematic break or stray `---` glued to text must not be
        // mistaken for an opening delimiter.
        assert_eq!(parse("--- not frontmatter\n"), None);
    }

    #[test]
    fn crlf_line_endings_parse() {
        let markdown = "---\r\nname: alpha\r\ndescription: CRLF summary.\r\n---\r\nbody\r\n";
        let frontmatter = parse(markdown).expect("frontmatter should parse");
        assert_eq!(frontmatter.name.as_deref(), Some("alpha"));
        assert_eq!(frontmatter.description.as_deref(), Some("CRLF summary."));
    }

    #[test]
    fn argument_names_dedupe_and_skip_empty_tokens() {
        let frontmatter = Frontmatter {
            argument_hint: Some("add [tag] | remove [tag] [] [ note ]".to_string()),
            ..Frontmatter::default()
        };
        assert_eq!(
            frontmatter.argument_names(),
            vec!["tag".to_string(), "note".to_string()]
        );
    }

    #[test]
    fn strip_removes_frontmatter_and_separating_blank_lines() {
        let markdown = "---\nname: alpha\n---\n\n# Alpha Title\n\nalpha body\n";
        assert_eq!(strip(markdown), "# Alpha Title\n\nalpha body\n");
    }

    #[test]
    fn strip_removes_structurally_valid_block_with_invalid_yaml() {
        let markdown = "---\n: [unbalanced\n---\nbody\n";
        assert_eq!(strip(markdown), "body\n");
    }

    #[test]
    fn strip_leaves_document_without_frontmatter_unchanged() {
        let markdown = "# Just a Heading\n\nbody\n";
        assert_eq!(strip(markdown), markdown);
    }

    #[test]
    fn strip_leaves_unterminated_block_unchanged() {
        let markdown = "---\nname: alpha\n\n# Heading\n";
        assert_eq!(strip(markdown), markdown);
    }

    #[test]
    fn strip_handles_closing_delimiter_at_end_of_input() {
        assert_eq!(strip("---\nname: alpha\n---"), "");
    }
}
