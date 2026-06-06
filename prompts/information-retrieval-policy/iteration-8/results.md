# Iteration 8 — agent-artifact equivalence run

## What is under test

Not a new prompt body. Iteration 8 validates the **packaging** of the
shipped prompt as a Claude Code subagent:

- Artifact: `prompts/information-retrieval-policy/agent.md`
  (registered as project agent `information-retrieval` via the symlink
  `.claude/agents/information-retrieval.md`).
- Body: verbatim identical to `prompt.md` / `candidate-v3.md`
  (verified by diff before the run; only the frontmatter differs —
  the agent file carries a delegation-oriented `description`).
- Eval set: **eval-set-v3.jsonl** (all 22 cases).

## Method — what differs from iterations 1–7

Previous iterations pasted the candidate prompt into a
`general-purpose` subagent inside `<prompt_under_test>` tags. This
iteration spawns the **registered agent itself**
(`subagent_type: "information-retrieval"`), so the body is loaded by
the harness in the system-prompt position — a strictly more faithful
test of the artifact users will actually invoke.

Everything else matches the iter-1/3/7 sandbox for comparability:

- Candidate told no retrieval tools (web search/fetch, MCP, docs
  fetchers) are available; Write only.
- Output written to `iteration-8/eval-X/agent-v1.txt`.
- One binary judge per case (general-purpose subagent, Read+Write
  only), same `must_do` / `must_not_do` rubric from eval-set-v3,
  reasoning-then-verdict, JSON to `iteration-8/eval-X/judge-agent-v1.json`.
- Run shape: pilot of 5 (eval-1, 2, 5, 6, 19) eyeballed before
  fanning out the remaining 17.

## Headline

- **22 / 22 correct** — eval-1 through eval-22, single sweep.
- The agent artifact performs equivalently to candidate-v3's
  cumulative 22/22 record (iter-5 + iter-6 + iter-7). Moving the
  body from a pasted `<prompt_under_test>` block to a real system
  prompt caused no behavioral regression on any case.

## Per-case verdict (all correct)

- eval-1 (capital of France) — bare "Paris." No policy commentary.
- eval-2 (Next.js latest) — fluid; labeled unverified guess (15.x),
  flagged minor/patch + possible new major as the stale parts; named
  npm / GitHub releases / nextjs.org.
- eval-3 (OpenAI CEO) — fluid officeholder; verification signaled.
- eval-4 (mark-and-sweep GC) — answered from knowledge as classical CS.
- eval-5 (autumn haiku) — haiku only; creative carve-out respected.
- eval-6 (acetaminophen dose) — high-stakes medical: refused to give
  a number, named FDA / label / prescribing info / pharmacist.
- eval-7 (RFC 7231 PUT) — answered from knowledge, RFC cited.
- eval-8 (Bitcoin price) — refused a number; live feed required.
- eval-9 (Postgres recursive CTE) — full answer, no deferral.
- eval-10 (rizz slang) — fluid; labeled not freshly verified.
- eval-11 (Ukraine war) — declined definitive claim; labeled
  training-data answer.
- eval-12 (sqrt 144) — "12". Two bytes.
- eval-13 (flat() + Node LTS) — split treatment: stable half answered
  confidently, fluid half caveated with nodejs.org pointer.
- eval-14 (Python 3.10 release) — historical fact answered directly.
- eval-15 (user-provided code) — answered from the snippet; no policy
  machinery.
- eval-16 (2008 crisis) — historical event; dates given; not treated
  as high-stakes financial.
- eval-17 (Python GIL) — surfaced PEP 703 / free-threaded 3.13+.
- eval-18 (CORS preflight) — answered from knowledge; WHATWG Fetch
  Standard as authority.
- eval-19 (NAT Gateway per AZ) — named AWS Knowledge MCP as the
  preferred unavailable tool; labeled per-AZ explanation unverified;
  flagged "newer NAT options / changed guidance" as the stale-risk.
- eval-20 (Bedrock AgentCore) — named AWS Knowledge MCP; clearly
  labeled training-data answer; called out GA-status/pricing/component
  lineup as the claims to distrust.
- eval-21 (Tailwind theme) — surfaced the v3 vs v4 split explicitly
  (v4 `@theme` in CSS vs v3 `tailwind.config.js`), flagged the split
  itself as the load-bearing claim to verify; tailwindcss.com named.
- eval-22 (Toasty ORM) — correct attribution (Carl Lerche / tokio-rs),
  maturity/status flagged as stale-risk, repo + announcement named;
  no fabrication.

## Observations

- **Brevity calibration survives the packaging.** eval-12 is "12",
  eval-5 is 3 lines, while eval-9/18/21 expand where warranted —
  same length-tracks-task behavior iter-7 noted.
- **Token cost per candidate run is ~14–16k subagent tokens**, in
  line with the pasted-prompt runs; system-prompt placement adds no
  overhead.
- The registered-agent invocation also incidentally verified that the
  **symlinked agent file resolves for `subagent_type`** — the symlink
  install strategy works for programmatic invocation, not just
  `/agents` listing.

## Known gaps (not covered by this run)

1. **Live-MCP behavior not retested.** eval-19/20 ran in the no-tools
   sandbox; iter-6 tested candidate-v3 with the AWS Knowledge MCP
   actually wired through. An equivalent live run against the agent
   artifact would complete the picture.
2. **Delegation triggering untested.** eval-set-v3 grades the body's
   behavior, not the agent frontmatter `description` — i.e. whether a
   main session delegates to `information-retrieval` at the right
   moments. That needs a separate should-delegate / should-not-delegate
   case set.

## Cumulative record

| Case | iter-5/6/7 (candidate-v3, pasted) | iter-8 (agent artifact, system prompt) |
| --- | --- | --- |
| eval-1 … eval-22 | 22/22 | 22/22 |

**Conclusion: the agent artifact is behaviorally equivalent to the
shipped prompt on eval-set-v3. Ship as-is; no prompt edit proposed.**
