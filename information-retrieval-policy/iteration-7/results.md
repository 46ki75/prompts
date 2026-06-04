# Iteration 7 — full regression sweep of candidate-v3

## Headline

- **15 / 15 correct** on the previously-unswept cases (eval-2, 4, 5, 7,
  8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18).
- Combined with iter-5 (3 regression spot-checks + 4 iter-4 retests on
  v3) and iter-6 (eval-19, 20 with AWS Knowledge MCP wired through),
  **v3 now stands at 22 / 22 across the entire eval suite** —
  eval-1 through eval-22.
- No new failure modes surfaced. The v2 → v3 edits did not regress
  any earlier behavior.

## What was tested in iter-7

The 15 cases not yet retested directly against v3 — i.e. everything
in iter-1 and iter-3 except the three spot-checks iter-5 already did.

Each candidate subagent:
- Was given only `prompts/candidate-v3.md` + the user question.
- Was told no retrieval tools were available (matching the iter-1/3
  sandbox setup so results are comparable).
- Wrote its answer to `iteration-7/eval-X/candidate-v3.txt`.

Each judge subagent:
- Read the candidate output fresh.
- Applied the same `must_do` / `must_not_do` rubric from the eval set.
- Wrote a binary verdict to `iteration-7/eval-X/judge-v3.json`.

## Per-case verdict

- eval-2 (Next.js latest version, fluid_software_version) — correct.
  Recognized fluid, flagged as unverified guess, named primary sources
  (GitHub releases, npm, official blog) and `context7` as the
  unavailable specialized tool.
- eval-4 (mark-and-sweep GC, stable_cs_concept) — correct. Answered
  from knowledge as classical CS; no retrieval requested.
- eval-5 (autumn haiku, out_of_scope_creative) — correct. Produced
  the haiku; no policy commentary.
- eval-7 (RFC 7231 PUT semantics, stable_spec) — correct. Answered
  from knowledge with section reference; also noted RFC 9110 as
  obsoleting RFC.
- eval-8 (Bitcoin price, fluid_market_data) — correct. Refused to
  give a number, named primary feeds.
- eval-9 (Postgres recursive CTE, stable_well_documented_api) —
  correct. Full WITH RECURSIVE walkthrough with examples; no
  deferral to web search.
- eval-10 (rizz slang, fluid_slang_trends) — correct. Treated as
  fluid; labeled guess as not freshly verified.
- eval-11 (Ukraine war, fluid_current_events) — correct. Declined a
  definitive claim, labeled training-data answer, named primary
  sources.
- eval-12 (sqrt 144, stable_math) — correct. Answered `12` — single
  character, exactly the trivial-stable shape the policy targets.
- eval-13 (Array.prototype.flat + Node LTS, mixed) — correct.
  Answered the flat() half from knowledge confidently; treated the
  Node LTS half as fluid and pointed at nodejs.org. The two halves
  got different treatment as the rubric required.
- eval-14 (Python 3.10 release, stable_disguised_as_fluid) — correct.
  Stated "October 4, 2021" directly without invoking the
  software-versions-are-fluid rule (which is about *current* versions,
  not *historical* releases).
- eval-15 (user-provided Python code, user_provided_context) —
  correct. Answered from the provided snippet; no external citations.
- eval-16 (2008 financial crisis, historical_event_in_fluid_domain) —
  correct. Answered with dates (Aug 9 2007 BNP Paribas; Sep 15 2008
  Lehman) without treating "financial" as high-stakes-fluid.
- eval-17 (Python GIL, stale_knowledge_trap) — correct. Explicitly
  acknowledged PEP 703 / free-threaded build in 3.13 and recommended
  python.org docs.
- eval-18 (CORS preflight, primary_vs_user_driven) — correct.
  Answered from knowledge and cited the WHATWG Fetch Standard as the
  authority (MDN only as a readable summary).

## Notes from skimming the candidate outputs

- **Output length is tracking the task.** eval-12 is 1 byte ("12"),
  eval-5 is a 3-line haiku, while eval-9 (CTE) and eval-18 (CORS)
  expand to 64–65 lines because the task genuinely warrants it.
  No bloat-on-trivial-cases regression — the trivial-stable carve-out
  in candidate-v3 is holding.
- **Step 4 stale-claims callouts fire where they should and stay
  silent where they shouldn't.** eval-2, 10, 11, 13, 17 all have
  pointed stale-claim flags; eval-4, 12, 14, 16 don't add any,
  because they're stable.
- **Specialized-tool naming is correct.** eval-2 names `context7`
  (library docs); eval-9 names a Postgres MCP / context7 as the
  preferred-if-available tool. No instances of substituting the
  wrong specialized tool (the Mode D failure v3 fixed).

## Combined v3 record across iterations

- iter-1 sandbox (eval-1 to eval-12): only 3 cases formally
  retested on v3 in iter-5; the other 9 now covered here.
- iter-3 sandbox (eval-13 to eval-18): none retested on v3 in iter-5;
  all 6 covered here.
- iter-4 sandbox (eval-19 to eval-22): all 4 retested on v3 in iter-5.
- iter-6 with AWS Knowledge MCP wired through (eval-19, 20): both
  passed with primary-source citations and freshness markers.

**Total: 22 / 22 across the suite, plus 2 / 2 in the tool-available
configuration for the AWS cases.** No previously-passing case
regressed under v3.

## Verdict

Regression confirmed. `prompts/candidate-v3.md` ships as-is.

## Files on disk

- `iteration-7/eval-{2,4,5,7,8,9,10,11,12,13,14,15,16,17,18}/candidate-v3.txt`
  — candidate outputs.
- `iteration-7/eval-{...}/judge-v3.json` — binary verdicts.
- `iteration-7/results.md` — this file.
