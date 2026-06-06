# prompts

Prompt projects, distributed as static files via GitHub Pages.

## Layout

Each top-level directory under `prompts/` is one prompt project. The
distributable prompt lives at `<name>/prompt.md`; everything else in the
directory (eval sets, iteration results) is working material and is not
published.

`prompt.md` follows the Claude Skill markdown format: an optional YAML
frontmatter block (`name`, `description`, `argument-hint`) followed by
the prompt body. Bracketed tokens in `argument-hint` (e.g.
`"[topic] [audience]"`) declare positional arguments, referenced in the
body as `$1`, `$2`, … or all at once as `$ARGUMENTS`.

## Distribution contract

`just build` (a Rust builder in `crates/builder`) scans `prompts/` and
writes:

- `dist/prompts/list.json` — JSON array of
  `{ name, title, description?, arguments?, path }` entries, sorted by
  name. `name` and `description` come from the frontmatter (`name` falls
  back to the directory name), `title` is the body's first `#` heading,
  `arguments` are the names parsed from `argument-hint`, and `path` is
  relative to `prompts/`.
- `dist/prompts/<name>.md` — each prompt body, frontmatter stripped
  (the metadata already lives in `list.json`).

Pushes to `main` deploy `dist/` to GitHub Pages via
`.github/workflows/deploy.yml`.

## MCP server

`crates/mcp-prompts-46ki75` re-exposes the published distribution over the
Model Context Protocol: `prompts/list` projects `list.json` onto MCP
prompts (name, title, description, arguments) and `prompts/get` returns a
prompt's markdown body with any provided argument values substituted
(`$ARGUMENTS`, `$1`, `$2`, …). Gets resolve through `list.json`, so
unlisted files are unreachable.

```bash
cargo run --package mcp-prompts-46ki75 -- stdio   # what an MCP host launches
cargo run --package mcp-prompts-46ki75 -- http    # streamable HTTP at /mcp
```

`--base-url` (env `MCP_PROMPTS_BASE_URL`) points the server at another
static host — e.g. a local preview of an unpublished `dist/`.

Example MCP host configuration:

```json
{
  "mcpServers": {
    "prompts": {
      "command": "46ki75-prompts",
      "args": ["stdio"]
    }
  }
}
```

## Development

Tasks are run through [`just`](https://github.com/casey/just):

```bash
just build      # build dist/
just ci         # fmt-check + clippy + tests (what PR CI runs)
just test-live  # #[ignore]'d tests against the real GitHub Pages site
just lint-md    # markdownlint (requires `pnpm install` once)
just coverage   # per-file coverage table with uncovered lines
```
