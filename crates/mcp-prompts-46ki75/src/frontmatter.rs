//! Best-effort extraction of YAML frontmatter from prompt markdown.
//!
//! Prompts MAY open with a `---`-delimited YAML block carrying `name`
//! and `description` fields. Parsing is fallible by design: a missing,
//! unterminated, or malformed block yields [`None`] — never an error —
//! so callers fall back to the `list.json` metadata.

use serde::Deserialize;

/// The frontmatter fields this server consumes. Unknown fields are
/// ignored so prompts can carry extra metadata without breaking the
/// listing.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Frontmatter {
    /// Human-readable display name; projected onto the MCP resource
    /// `title`.
    #[serde(default)]
    pub name: Option<String>,
    /// One-line summary; projected onto the MCP resource `description`.
    #[serde(default)]
    pub description: Option<String>,
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
    fn parses_name_and_description() {
        let markdown = "---\nname: Alpha Prompt\ndescription: One-line summary.\n---\n\n# Alpha\n";
        assert_eq!(
            parse(markdown),
            Some(Frontmatter {
                name: Some("Alpha Prompt".to_string()),
                description: Some("One-line summary.".to_string()),
            })
        );
    }

    #[test]
    fn parses_folded_block_scalar_description() {
        let markdown = "---\nname: Alpha\ndescription: >-\n  Folded first line,\n  folded second line.\n---\n\nbody\n";
        let frontmatter = parse(markdown).expect("frontmatter should parse");
        assert_eq!(
            frontmatter.description.as_deref(),
            Some("Folded first line, folded second line.")
        );
    }

    #[test]
    fn ignores_unknown_fields() {
        let markdown = "---\nname: Alpha\nauthor: someone\n---\nbody\n";
        let frontmatter = parse(markdown).expect("frontmatter should parse");
        assert_eq!(frontmatter.name.as_deref(), Some("Alpha"));
        assert_eq!(frontmatter.description, None);
    }

    #[test]
    fn missing_fields_stay_none() {
        let frontmatter = parse("---\ndescription: only this\n---\n").expect("should parse");
        assert_eq!(frontmatter.name, None);
        assert_eq!(frontmatter.description.as_deref(), Some("only this"));
    }

    #[test]
    fn document_without_frontmatter_is_none() {
        assert_eq!(parse("# Just a Heading\n\nbody\n"), None);
    }

    #[test]
    fn unterminated_block_is_none() {
        assert_eq!(parse("---\nname: Alpha\n\n# Heading\n"), None);
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
    fn strip_removes_frontmatter_and_separating_blank_lines() {
        let markdown = "---\nname: Alpha\n---\n\n# Alpha Title\n\nalpha body\n";
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
        let markdown = "---\nname: Alpha\n\n# Heading\n";
        assert_eq!(strip(markdown), markdown);
    }

    #[test]
    fn strip_handles_closing_delimiter_at_end_of_input() {
        assert_eq!(strip("---\nname: Alpha\n---"), "");
    }

    #[test]
    fn crlf_line_endings_parse() {
        let markdown = "---\r\nname: Alpha\r\ndescription: CRLF summary.\r\n---\r\nbody\r\n";
        let frontmatter = parse(markdown).expect("frontmatter should parse");
        assert_eq!(frontmatter.name.as_deref(), Some("Alpha"));
        assert_eq!(frontmatter.description.as_deref(), Some("CRLF summary."));
    }
}
