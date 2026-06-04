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

## Development

Tasks are run through [`just`](https://github.com/casey/just):

```bash
just build      # build dist/
just ci         # fmt-check + clippy + tests (what PR CI runs)
just lint-md    # markdownlint (requires `npm install` once)
just coverage   # per-file coverage table with uncovered lines
```
