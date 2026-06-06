//! MCP prompt handlers — the thin adapter from the use case onto
//! `prompts/list` and `prompts/get`.
//!
//! Prompts are addressed by the `name` field of their `list.json`
//! entry. `prompts/get` substitutes provided argument values into the
//! markdown body the way Claude Code feeds arguments to skills — see
//! [`substitute`].

use rmcp::ErrorData as McpError;
use rmcp::model::{
    GetPromptRequestParams, GetPromptResult, JsonObject, ListPromptsResult, Prompt, PromptArgument,
    PromptMessage, PromptMessageRole,
};
use serde_json::json;

use crate::Server;
use crate::use_case::PromptsUseCaseError;

/// Handle `prompts/list` by projecting `list.json` entries onto MCP
/// [`Prompt`]s. The index already carries the frontmatter-derived
/// metadata, so listing needs no per-prompt fetches.
pub async fn list_prompts(server: &Server) -> Result<ListPromptsResult, McpError> {
    let entries = server
        .prompts_use_case()
        .list_prompts()
        .await
        .map_err(to_mcp_error)?;

    let prompts = entries
        .into_iter()
        .map(|entry| {
            let arguments = (!entry.arguments.is_empty()).then(|| {
                entry
                    .arguments
                    .iter()
                    .cloned()
                    .map(PromptArgument::new)
                    .collect()
            });
            let mut prompt = Prompt::new(entry.name, entry.description, arguments);
            prompt.title = Some(entry.title);
            prompt
        })
        .collect();

    Ok(ListPromptsResult::with_all_items(prompts))
}

/// Handle `prompts/get`: resolve the name through `list.json`, fetch
/// the body, and substitute any provided argument values.
pub async fn get_prompt(
    server: &Server,
    request: GetPromptRequestParams,
) -> Result<GetPromptResult, McpError> {
    let document = server
        .prompts_use_case()
        .read_prompt(&request.name)
        .await
        .map_err(to_mcp_error)?;

    let text = substitute(
        &document.content,
        &document.entry.arguments,
        request.arguments.as_ref(),
    );

    let mut result =
        GetPromptResult::new(vec![PromptMessage::new_text(PromptMessageRole::User, text)]);
    if let Some(description) = document.entry.description {
        result = result.with_description(description);
    }
    Ok(result)
}

/// Substitute argument values into `body`, mirroring how Claude Code
/// feeds arguments to skills:
///
/// - `$ARGUMENTS` is replaced with all provided values joined by a
///   space, in declared order;
/// - `$1`, `$2`, … are replaced with the value of the n-th declared
///   argument (empty when not provided);
/// - when the body contains no placeholder and at least one value was
///   provided, the joined values are appended after the body.
fn substitute(body: &str, declared: &[String], provided: Option<&JsonObject>) -> String {
    let values: Vec<Option<String>> = declared
        .iter()
        .map(|name| {
            provided
                .and_then(|arguments| arguments.get(name))
                .map(value_to_string)
        })
        .collect();
    let joined = values
        .iter()
        .flatten()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(" ");

    let positional = |index: usize| format!("${index}");
    let has_placeholder = body.contains("$ARGUMENTS")
        || (1..=values.len()).any(|index| body.contains(&positional(index)));

    if !has_placeholder {
        if joined.is_empty() {
            return body.to_string();
        }
        let mut out = body.trim_end().to_string();
        out.push_str("\n\n");
        out.push_str(&joined);
        out.push('\n');
        return out;
    }

    let mut out = body.replace("$ARGUMENTS", &joined);
    // Highest index first so `$1` does not clobber `$10`.
    for index in (1..=values.len()).rev() {
        out = out.replace(
            &positional(index),
            values[index - 1].as_deref().unwrap_or_default(),
        );
    }
    out
}

/// Render a JSON argument value as substitution text. The MCP spec
/// types prompt arguments as strings; non-string values are tolerated
/// by serializing them compactly.
fn value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

/// Map use case errors onto JSON-RPC error codes: an unknown name is
/// an invalid-params error (per the MCP spec for `prompts/get`);
/// everything else (network, bad deployment) is an internal error
/// carrying the upstream detail.
fn to_mcp_error(error: PromptsUseCaseError) -> McpError {
    match &error {
        PromptsUseCaseError::NotFound { name } => {
            McpError::invalid_params(error.to_string(), Some(json!({ "name": name })))
        }
        PromptsUseCaseError::Repository(_) => McpError::internal_error(error.to_string(), None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declared(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    fn provided(pairs: &[(&str, &str)]) -> JsonObject {
        pairs
            .iter()
            .map(|(key, value)| (key.to_string(), json!(value)))
            .collect()
    }

    #[test]
    fn substitutes_arguments_placeholder() {
        let arguments = provided(&[("topic", "rust"), ("audience", "beginners")]);
        assert_eq!(
            substitute(
                "Research $ARGUMENTS.",
                &declared(&["topic", "audience"]),
                Some(&arguments)
            ),
            "Research rust beginners."
        );
    }

    #[test]
    fn substitutes_positional_placeholders() {
        let arguments = provided(&[("topic", "rust"), ("audience", "beginners")]);
        assert_eq!(
            substitute(
                "Research $1 for $2.",
                &declared(&["topic", "audience"]),
                Some(&arguments)
            ),
            "Research rust for beginners."
        );
    }

    #[test]
    fn missing_positional_value_becomes_empty() {
        let arguments = provided(&[("topic", "rust")]);
        assert_eq!(
            substitute(
                "Research $1 for $2.",
                &declared(&["topic", "audience"]),
                Some(&arguments)
            ),
            "Research rust for ."
        );
    }

    #[test]
    fn appends_values_when_body_has_no_placeholder() {
        let arguments = provided(&[("topic", "rust")]);
        assert_eq!(
            substitute("Do the thing.\n", &declared(&["topic"]), Some(&arguments)),
            "Do the thing.\n\nrust\n"
        );
    }

    #[test]
    fn no_values_leaves_placeholder_free_body_unchanged() {
        assert_eq!(
            substitute("Do the thing.\n", &declared(&["topic"]), None),
            "Do the thing.\n"
        );
    }

    #[test]
    fn no_values_blanks_placeholders() {
        assert_eq!(
            substitute("Research $ARGUMENTS now: $1.", &declared(&["topic"]), None),
            "Research  now: ."
        );
    }

    #[test]
    fn undeclared_provided_arguments_are_ignored() {
        let arguments = provided(&[("unrelated", "x")]);
        assert_eq!(
            substitute("Body.\n", &declared(&["topic"]), Some(&arguments)),
            "Body.\n"
        );
    }

    #[test]
    fn non_string_values_serialize_compactly() {
        let mut arguments = JsonObject::new();
        arguments.insert("count".to_string(), json!(3));
        assert_eq!(
            substitute("Take $1.", &declared(&["count"]), Some(&arguments)),
            "Take 3."
        );
    }
}
