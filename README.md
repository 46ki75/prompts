# prompts

Prompt projects, distributed as static files via GitHub Pages.

## Layout

Each top-level directory is one prompt project. The distributable prompt
lives at `<name>/prompt.md`; everything else in the directory (eval sets,
iteration results) is working material and is not published.

## Distribution contract

`just build` (a Rust builder in `crates/builder`) scans the repository and
writes:

- `dist/resources/list.json` — JSON array of `{ name, title, path }`
  entries, sorted by name. `title` is the prompt's first `#` heading,
  `path` is relative to `resources/`.
- `dist/resources/<name>.md` — each prompt, copied verbatim.

Pushes to `main` deploy `dist/` to GitHub Pages via
`.github/workflows/deploy.yml`.

## MCP server

`crates/prompts-46ki75` re-exposes the published distribution over the
Model Context Protocol: `resources/list` projects `list.json` onto
`prompts://<name>` resources and `resources/read` returns a prompt's
markdown. Reads resolve through `list.json`, so unlisted files are
unreachable.

```bash
cargo run --package prompts-46ki75 -- stdio       # what an MCP host launches
cargo run --package prompts-46ki75 -- http        # streamable HTTP at /mcp
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
